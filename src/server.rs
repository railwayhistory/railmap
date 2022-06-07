use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use hyper::{Body, Request, Response};
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use lru::LruCache;
use crate::render::feature::FeatureSet;
use crate::tile::{Tile, TileId};
use crate::theme::{Style, Theme};


pub struct Server<T: Theme> {
    theme: T,
    features: Arc<FeatureSet<T>>,
    cache: Arc<Mutex<LruCache<TileId<<T::Style as Style>::StyleId>, Bytes>>>,
}


impl<T: Theme> Server<T> {
    pub fn new(
        theme: T,
        features: FeatureSet<T>,
    ) -> Self {
        Server {
            theme,
            features: Arc::new(features),
            cache: Arc::new(Mutex::new(LruCache::new(10_000))),
        }
    }
}

impl<T: Theme> Server<T> {
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

impl<T: Theme> Server<T> {
    async fn process(
        &self, request: Request<Body>
    ) -> Result<Response<Body>, Infallible> {
        match request.uri().path() {
            "/" => {
                return Ok(Response::builder()
                    .header("Content-Type", "text/html")
                    .body(From::<&'static [u8]>::from(
                        self.theme.index_page()
                    ))
                    .unwrap()
                )
            }
            "/ol.js" => {
                return Ok(Response::builder()
                    .header("Content-Type", "application/javascript")
                    .body(From::<&'static [u8]>::from(
                            include_bytes!("../html/ol.js").as_ref()
                    ))
                    .unwrap()
                )
            }
            "/ol.css" => {
                return Ok(Response::builder()
                    .header("Content-Type", "text/css")
                    .body(From::<&'static [u8]>::from(
                            include_bytes!("../html/ol.css").as_ref()
                    ))
                    .unwrap()
                )
            }
            _ => { }
        }

        let tile = match TileId::from_path(&request.uri().path()[1..]) {
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
        let cached = self.cache.lock().unwrap().get(&tile).map(Clone::clone); 
        let body = match cached {
            Some(bytes) => bytes.into(),
            None => {
                let bytes: Bytes = Tile::new(&self.theme, tile.clone()).render(
                    &self.features
                ).into();
                self.cache.lock().unwrap().put(tile.clone(), bytes.clone());
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

impl<T: Theme> Clone for Server<T> {
    fn clone(&self) -> Self {
        Server {
            theme: self.theme.clone(),
            features: self.features.clone(),
            cache: self.cache.clone()
        }
    }
}

pub struct Failed;

