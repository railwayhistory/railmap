use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use hyper::{Body, Request, Response};
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use lru::LruCache;
use crate::features::FeatureSet;
use crate::import;
use crate::tile::{Tile, TileId};

#[derive(Clone)]
pub struct Server {
    features: Arc<FeatureSet>,
    cache: Arc<Mutex<LruCache<TileId, Bytes>>>,
}


impl Server {
    pub fn new(
        import_dir: impl AsRef<Path>,
    ) -> Result<Server, import::ImportError> {
        import::load(import_dir.as_ref()).map(|features| {
            Server {
                features: Arc::new(features),
                cache: Arc::new(Mutex::new(LruCache::new(10_000))),
            }
        })
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

        // Run this server for ... ever!
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
        let body = match self.cache.lock().unwrap().get(&tile) {
            Some(bytes) => bytes.clone().into(),
            None => {
                let bytes = Tile::new(tile).render(&self.features);
                self.cache.lock().unwrap().put(tile, bytes.clone());
                bytes.into()
            }
        };
        Ok(Response::builder()
            .header("Content-Type", tile.content_type())
            .body(body)
            .unwrap()
        )
    }
}


pub struct Failed;

