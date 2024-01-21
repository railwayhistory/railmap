use std::{fs, io};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use clap::{
    Arg, ArgAction, ArgMatches, Command, crate_version, crate_authors,
    value_parser,
};
use railmap::{LoadFeatures, MapConfig, Server};
use femtomap::import::eval::Failed;

const DEFAULT_CONFIG_PATH: &str = "/etc/railmap.conf";


struct Config {
    map: PathBuf,
    regions: Option<Vec<String>>,
    listen: SocketAddr,
}

#[derive(serde::Deserialize)]
struct ConfigFile {
    map: Option<PathBuf>,
    regions: Option<Vec<String>>,
    listen: Option<SocketAddr>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map: PathBuf::new(),
            regions: None,
            listen: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
        }
    }
}

impl Config {
    pub fn get() -> Result<Self, Failed> {
        let mut matches = Self::get_matches();

        let (config_path, insist) = match matches.remove_one::<PathBuf>(
            "config"
        ) {
            Some(path) => (path, true),
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

        config.apply_matches(matches);

        if !config.map.is_file() {
            eprintln!("Map configuration not provided or does not exist.");
            return Err(Failed)
        }

        Ok(config)
    }

    fn get_matches() -> ArgMatches {
        Command::new("railmap")
            .version(crate_version!())
            .author(crate_authors!())
            .about("renders a railway map")
            .arg(Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .help("the configuration file")
                .action(ArgAction::Set)
            )
            .arg(Arg::new("map")
                .short('m')
                .long("map")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .help("the map configuration file")
                .action(ArgAction::Set)
            )
            .arg(Arg::new("region")
                .short('r')
                .long("region")
                .value_name("NAME")
                .help("select a region to render")
                .num_args(1..)
                .action(ArgAction::Append)
            )
            .arg(Arg::new("listen")
                .short('l')
                .long("listen")
                .value_name("ADDR")
                .value_parser(value_parser!(SocketAddr))
                .help("the addr to listen on")
                .action(ArgAction::Set)
                .default_value("127.0.0.1:8080")
            )
            .get_matches()
    }

    fn apply_matches(&mut self, mut matches: ArgMatches) {
        if let Some(map) = matches.remove_one("map") {
            self.map = map;
        }
        if let Some(regions) = matches.remove_many("region") {
            self.regions = Some(regions.collect());
        }
        if let Some(addr) = matches.remove_one("listen") {
            self.listen = addr;
        }
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

    pub async fn run(self) {
        let map = match MapConfig::load(&self.map) {
            Ok(map) => map,
            Err(err) => {
                eprintln!(
                    "Failed to load map config {}: {}",
                    self.map.display(), err
                );
                return
            }
        };

        let start = Instant::now();
        let mut features = LoadFeatures::new();
        match self.regions {
            Some(mut values) => {
                values.sort();
                values.dedup();
                for value in values {
                    match map.regions.get(&value) {
                        Some(region) => {
                            features.load_region(region);
                        }
                        None => {
                            eprintln!("Unknown region '{}'.", value);
                            std::process::exit(1);
                        }
                    }
                }
            }
            None => {
                for region in map.regions.values() {
                    features.load_region(region)
                }
            }
        }

        let features = match features.finalize() {
            Ok(features) => features,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };

        let server = Server::new(features);
        eprintln!("Server ready after {:.03}s.", start.elapsed().as_secs_f32());
        server.run(self.listen).await;
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
