use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Instant;
use clap::{Arg, ArgAction, ArgMatches, Command, crate_version, crate_authors};
use railmap::{Config, LoadFeatures, Server};
use railmap::theme::Theme;

#[allow(dead_code)]
async fn run<T: Theme>(config: Config, matches: ArgMatches, mut theme: T) {
    let start = Instant::now();
    theme.config(&config);
    let mut features = LoadFeatures::new(&theme);
    match matches.get_many::<String>("region") {
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

    let addr_str: &String = matches.get_one("listen").unwrap();
    let addr = match SocketAddr::from_str(addr_str) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Invalid listen addr '{}'.", addr_str);
            std::process::exit(1);
        }
    };

    let server = Server::new(
        theme, features,
        matches.get_one::<String>("style").unwrap().clone(),
    );
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
            .action(ArgAction::Set)
            .required(true)
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
            .help("the addr to listen on")
            .action(ArgAction::Set)
            .default_value("127.0.0.1:8080")
        )
        .arg(Arg::new("style")
            .short('s')
            .long("style")
            .value_name("STYLE")
            .help("the style to use in the test map")
            .action(ArgAction::Set)
            .default_value("lx")
        )
        .get_matches();

    let config = match Config::load(
        matches.get_one::<String>("config").unwrap()
    ) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Failed to load map config {}: {}",
                matches.get_one::<String>("config").unwrap(),
                err
            );
            std::process::exit(1)
        }
    };

    run(config, matches, railmap::map::Railwayhistory::default()).await
}
