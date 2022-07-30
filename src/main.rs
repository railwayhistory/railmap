use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Instant;
use clap::{Arg, ArgMatches, Command, crate_version, crate_authors};
use railmap::{Config, LoadFeatures, Server};
use railmap::theme::Theme;
use railmap::maps::overnight::Overnight;
use railmap::maps::railwayhistory::Railwayhistory;

async fn run<T: Theme>(config: Config, matches: ArgMatches, mut theme: T) {
    let start = Instant::now();
    theme.config(&config);
    let mut features = LoadFeatures::new(&theme);
    match matches.values_of("region") {
        Some(values) => {
            let mut values: Vec<_> = values.collect();
            values.sort();
            values.dedup();
            for value in values {
                match config.regions.get(value) {
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
            for region in config.regions.values() {
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

    let addr_str = matches.value_of("listen").unwrap();
    let addr = match SocketAddr::from_str(addr_str) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Invalid listen addr '{}'.", addr_str);
            std::process::exit(1);
        }
    };

    let server = Server::new(theme, features);
    eprintln!("Server ready after {:.03}s.", start.elapsed().as_secs_f32());
    server.run(addr).await;
}

#[tokio::main]
async fn main() {
    let matches = Command::new("railmap")
        .version(crate_version!())
        .author(crate_authors!())
        .about("renders a railway map")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("the map configuration file")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::new("region")
            .short('r')
            .long("region")
            .value_name("NAME")
            .help("select a region to render")
            .takes_value(true)
            .multiple_occurrences(true)
            .multiple_values(true)
        )
        .arg(Arg::new("listen")
            .short('l')
            .long("listen")
            .value_name("ADDR")
            .help("the addr to listen on")
            .takes_value(true)
            .default_value("127.0.0.1:8080")
        )
        .get_matches();

    let config = match Config::load(matches.value_of("config").unwrap()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Failed to load map config {}: {}",
                matches.value_of("config").unwrap(),
                err
            );
            std::process::exit(1)
        }
    };

    match config.theme.as_ref() {
        "railwayhistory" => {
            run(config, matches, Railwayhistory::default()).await
        }
        "overnight" => run(config, matches, Overnight).await,
        theme => {
            eprintln!("Unknown theme '{}' in config.", theme);
            std::process::exit(1)
        }
    }
}
