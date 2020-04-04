use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use hyper::{Body, Request, Response};
use hyper::service::{make_service_fn, service_fn};
use crate::feature::FeatureSet;
use crate::tile::{Tile, TileId};
use crate::path::PathSet;

#[derive(Clone)]
pub struct Server {
    features: Arc<FeatureSet>,
}


impl Server {
    pub fn new(
        path_dir: impl AsRef<Path>,
        _feature_dir: impl AsRef<Path>,
    ) -> Result<Server, Failed> {
        let paths = match PathSet::load(path_dir) {
            Ok(paths) => paths,
            Err(err) => {
                eprintln!("{}", err);
                return Err(Failed)
            }
        };
        let features = Arc::new(FeatureSet::load(&paths));
        Ok(Server { features })
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
                .header("Content-Type", "text/html")
                .body(From::<&'static [u8]>::from(
                        include_bytes!("../html/index.html").as_ref()
                ))
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
            .body(tile.render(&self.features))
            .unwrap()
    }
}


pub struct Failed;

