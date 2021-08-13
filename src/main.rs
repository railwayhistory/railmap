use std::net::SocketAddr;
use std::path::PathBuf;
use railmap::Server;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let name = args.next().unwrap(); // Skip own name.

    let first_dir = match args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "Usage: {} (<import-dir> | <path-dir> <rules-dir>)",
                name
            );
            std::process::exit(1)
        }
    };

    let (path_dir, rules_dir) = match args.next() {
        Some(dir) => (first_dir, PathBuf::from(dir)),
        None => (first_dir.join("paths"), first_dir.join("rules"))
    };

    let server = match Server::new(path_dir, rules_dir) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
    };
    eprintln!("Server ready.");
    server.run(
        SocketAddr::from(([0, 0, 0, 0], 8080))
    ).await;
}
