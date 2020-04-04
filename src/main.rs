use std::net::SocketAddr;
use railmap::Server;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let name = args.next().unwrap(); // Skip own name.

    let path_dir = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: {} <path-dir> <feature-dir>", name);
            std::process::exit(1)
        }
    };
    let feature_dir = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: {} <path-dir> <feature-dir>", name);
            std::process::exit(1)
        }
    };

    let server = match Server::new(path_dir, feature_dir) {
        Ok(server) => server,
        Err(_) => std::process::exit(1)
    };
    eprintln!("Server ready.");
    server.run(
        SocketAddr::from(([127, 0, 0, 1], 8080))
    ).await;
}
