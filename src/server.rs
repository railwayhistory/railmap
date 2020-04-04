use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response};
use hyper::service::{make_service_fn, service_fn};
use crate::tile::{Tile, TileId};

#[derive(Clone)]
pub struct Server;


impl Server {
    pub fn new() -> Server {
        Server
    }

    pub async fn run(&self, addr: SocketAddr) {
        let make_svc = make_service_fn(move |_conn| {
            let this = self.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |r| {
                    let this = this.clone();
                    async move { this.process(r).await }
                }))
            }
        });

        let server = hyper::Server::bind(&addr).serve(make_svc);

        // Run this server for... forever!
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}

impl Server {
    async fn process(
        &self, request: Request<Body>
    ) -> Result<Response<Body>, Infallible> {
        if request.uri().path() == "/" {
            return Ok(Response::builder()
                .header("Contet-Type", "text/html")
                .body(Body::from(include_bytes!("../html/index.html").as_ref()))
                .unwrap()
            )
        }

        let tile = match TileId::from_path(request.uri().path()) {
            Ok(tile) => tile,
            Err(_) => {
                return Ok(Response::builder()
                    .status(404)
                    .header("Content-Type", "text/plain;charset=utf-8")
                    .body(Body::from("not found"))
                    .unwrap()
                )
            }
        };
        Ok(self.render(tile))
    }

    fn render(&self, tile: TileId) -> Response<Body> {
        let tile = Tile::new(tile);
        Response::builder()
            .header("Content-Type", tile.content_type())
            .body(tile.into_body())
            .unwrap()
    }
}

