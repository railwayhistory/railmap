use std::net::SocketAddr;
use railmap::Server;

#[tokio::main]
async fn main() {
    Server::new().run(SocketAddr::from(([127, 0, 0, 1], 8080))).await;
}
