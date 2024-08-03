use std::{fs, io};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};
use clap::Parser;
use femtomap::import::eval::Failed;
use femtomap::import::watch::WatchSet;
use notify::Watcher;
use railmap::MapConfig;
use railmap::railway;
use railmap::railway::import::load::LoadFeatures;
use railmap::server::{Server, ServerControl};
use tokio::sync::{mpsc, oneshot};

const DEFAULT_CONFIG_PATH: &str = "/etc/railmap.conf";

//------------ ConfigFile ----------------------------------------------------

#[derive(serde::Deserialize)]
struct ConfigFile {
    map: Option<PathBuf>,
    regions: Option<Vec<String>>,
    listen: Option<SocketAddr>,
}

//------------ Args ----------------------------------------------------------

#[derive(Parser)]
#[command(version, about, long_about = "renders a railway map")]
struct Args {
    /// The configuration file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// The map configuration file.
    #[arg(short, long, value_name = "FILE")]
    map: Option<PathBuf>,

    /// Select regions to render.
    #[arg(value_name = "NAME")]
    region: Vec<String>,

    /// The addr to listen on.
    #[arg(short, long, value_name = "ADDR")]
    listen: Option<SocketAddr>,

    /// Watch for changes in map files.
    #[arg(short, long)]
    watch: bool,

    /// Enable proof mode.
    #[arg(short, long)]
    proof: bool,
}


//------------ Config --------------------------------------------------------

struct Config {
    map: PathBuf,
    regions: Option<Vec<String>>,
    listen: SocketAddr,
    watch: bool,
    proof: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map: PathBuf::new(),
            regions: None,
            listen: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            watch: false,
            proof: false,
        }
    }
}

impl Config {
    pub fn get() -> Result<Self, Failed> {
        let args = Args::parse();

        let (config_path, insist) = match args.config.as_ref() {
            Some(path) => (path.clone(), true),
            None => (PathBuf::from(DEFAULT_CONFIG_PATH), false),
        };

        let mut config = Config::default();

        match fs::read_to_string(&config_path) {
            Ok(content) => {
                let value = match toml::from_str(&content) {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!(
                            "Failed to parse config file: {}: {}",
                            config_path.display(), err
                        );
                        return Err(Failed)
                    }
                };
                config.apply_toml(value);
            }
            Err(err) => {
                if
                    !matches!(err.kind(), io::ErrorKind::NotFound)
                    || insist
                {
                    eprintln!(
                        "Failed to read config file {}: {}",
                        config_path.display(), err
                    );
                    return Err(Failed)
                }
            }
        }

        config.apply_args(args);

        if !config.map.is_file() {
            eprintln!("Map configuration not provided or does not exist.");
            return Err(Failed)
        }

        Ok(config)
    }

    fn apply_args(&mut self, args: Args) {
        if let Some(map) = args.map {
            self.map = map;
        }
        if !args.region.is_empty() {
            self.regions = Some(args.region);
        }
        if let Some(addr) = args.listen {
            self.listen = addr;
        }
        self.watch = args.watch;
        self.proof = args.proof;
    }

    fn apply_toml(&mut self, toml: ConfigFile) {
        if let Some(map) = toml.map {
            self.map = map
        }
        if let Some(regions) = toml.regions {
            self.regions = Some(regions);
        }
        if let Some(listen) = toml.listen {
            self.listen = listen;
        }
    }

    pub async fn run(mut self) {
        if let Some(regions) = self.regions.as_mut() {
            regions.sort();
            regions.dedup();
        }

        let mut watch = WatchSet::default();
        if self.watch {
            watch.enable();
        }

        let map = match self.load_railway(&mut watch) {
            Some(map) => map,
            None => return,
        };

        let (server, ctrl) = Server::new(map, self.proof);
        let listen = self.listen;

        if self.watch {
            tokio::spawn(self.watch(ctrl, watch));
        }

        let _ = server.run(listen).await;
    }

    async fn watch(self, ctrl: ServerControl, mut watch: WatchSet) {
        loop {
            watch = match self.watch_step(&ctrl, watch).await {
                Ok(watch) => watch,
                Err(_) => break,
            }
        }
    }

    async fn watch_step(
        &self, ctrl: &ServerControl, watch: WatchSet
    ) -> Result<WatchSet, Failed> {
        let (ev_tx, mut ev_rx) = mpsc::channel(10);
        let (done_tx, done_rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let mut watcher = match notify::recommended_watcher(
                move |res: Result<notify::event::Event, _>| {
                    match res {
                        Ok(ev) => {
                            let mut skip = true;
                            for path in &ev.paths {
                                if let Some(name) = path.file_name() {
                                    if name.as_encoded_bytes().first()
                                        != Some(&b'.')
                                    {
                                        skip = false;
                                        break;
                                    }
                                }
                            }
                            if skip {
                                return
                            }
                            let _ = ev_tx.blocking_send(());
                        }
                        _ => { } // XXX Ignore errors for now.
                    }
                }
            ) {
                Ok(watcher) => watcher,
                Err(_) => return,
            };
            for item in watch.iter() {
                let _ = watcher.watch(
                    item, notify::RecursiveMode::NonRecursive
                );
            }
            let _ = done_rx.blocking_recv();
        });

        let _ = ev_rx.recv().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = done_tx.send(());

        let mut watch = WatchSet::default();
        watch.enable();
        if let Some(map) = self.load_railway(&mut watch) {
            ctrl.update_railway(map).await;
        }

        Ok(watch)
    }

    fn load_railway(
        &self, watch: &mut WatchSet,
    ) -> Option<railway::Map> {
        let map = match MapConfig::load(&self.map) {
            Ok(map) => map,
            Err(err) => {
                eprintln!(
                    "Failed to load map config {}: {}",
                    self.map.display(), err
                );
                return None
            }
        };

        let start = Instant::now();
        let mut features = LoadFeatures::new();
        match self.regions.as_ref() {
            Some(values) => {
                for value in values {
                    match map.regions.get(value) {
                        Some(region) => {
                            features.load_region(region, watch);
                        }
                        None => {
                            eprintln!("Unknown region '{}'.", value);
                            return None;
                        }
                    }
                }
            }
            None => {
                for region in map.regions.values() {
                    features.load_region(region, watch)
                }
            }
        }

        let features = match features.finalize() {
            Ok(features) => features,
            Err(err) => {
                eprintln!("{}", err);
                return None;
            }
        };

        eprintln!("Loaded map in {:.03}s.", start.elapsed().as_secs_f32());
        eprintln!(
            "Features:\n  \
               railway: {}\n  \
               line labels: {}\n  \
               timetable labels: {}\n  \
               borders: {}",
            features.railway.len(),
            features.line_labels.len(),
            features.tt_labels.len(),
            features.borders.len(),
        );

        Some(railway::Map::new(features))
    }
}

#[tokio::main]
async fn main() {
    let config = match Config::get() {
        Ok(config) => config,
        Err(_) =>  return,
    };

    config.run().await
}
