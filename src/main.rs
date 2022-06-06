use std::net::SocketAddr;
use std::time::Instant;
use clap::{Arg, App, crate_version, crate_authors};
use railmap::{Config, LoadFeatures, Server};
use railmap::maps::railwayhistory::Railwayhistory;

#[tokio::main]
async fn main() {
    let matches = App::new("railmap")
        .version(crate_version!())
        .author(crate_authors!())
        .about("renders a railway map")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("the map configuration file")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("region")
            .short("r")
            .long("region")
            .value_name("NAME")
            .help("select a region to render")
            .takes_value(true)
            .multiple(true)
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

    let start = Instant::now();
    let mut features = LoadFeatures::new(Railwayhistory);
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

    let server = Server::new(Railwayhistory, features);
    eprintln!("Server ready after {:.03}s.", start.elapsed().as_secs_f32());
    server.run(
        SocketAddr::from(([0, 0, 0, 0], 8080))
    ).await;
}
