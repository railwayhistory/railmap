use std::convert::Infallible;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use hyper::{Body, Request, Response};
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use lru::LruCache;
use crate::colors::ColorSet;
use crate::feature::Store;
use crate::tile::{Tile, TileId};


#[derive(Clone)]
pub struct Server {
    features: Arc<Store>,
    cache: Arc<Mutex<LruCache<TileId, Bytes>>>,
    colors: ColorSet,
}


impl Server {
    pub fn new(features: Store) -> Self {
        Server {
            features: Arc::new(features),
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(10_000).unwrap())
            )),
            colors: Default::default(),
        }
    }
}

impl Server {
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
        let path = request.uri().path();

        match path {
            "/" => {
                return Ok(Response::builder()
                    .header("Content-Type", "text/html")
                    .body(From::<&'static [u8]>::from(
                        include_bytes!("../html/index.html")
                    ))
                    .unwrap()
                )
            }
            "/ol.js" => {
                return Ok(Response::builder()
                    .header(
                        "Content-Type",
                        "application/javascript;charset=utf-8"
                    )
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

        /*
        if path.starts_with("/key/") {
            let path = &mut path[5..].split('/');
            let zoom = match path.next() {
                Some(zoom) => {
                    match u8::from_str(zoom) {
                        Ok(zoom) => zoom,
                        Err(_) => return Ok(not_found())
                    }
                }
                None => return Ok(not_found())
            };
            let name = match path.next() {
                Some(style) => style,
                None => return Ok(not_found())
            };
            if path.next().is_some() {
                return Ok(not_found())
            }
            let mut name = name.split('.');
            let style = match name.next() {
                Some(style) => {
                    match StyleId::from_str(style) {
                        Ok(style) => style,
                        Err(_) => return Ok(not_found()),
                    }
                }
                None => return Ok(not_found())
            };
            let format = match name.next() {
                Some(format) => {
                    match TileFormat::from_str(format) {
                        Ok(format) => format,
                        Err(_) => return Ok(not_found()),
                    }
                }
                None => return Ok(not_found())
            };
            if name.next().is_some() {
                return Ok(not_found())
            }
            let body = self.theme.map_key(zoom, style, format);
            return Ok(Response::builder()
                .header("Content-Type", format.content_type())
                .body(body)
                .unwrap()
            )
        }
        */

        let tile = match TileId::from_path(
            &request.uri().path()[1..],
        ) {
            Ok(tile) => tile,
            Err(_) => {
                return Ok(not_found())
            }
        };
        let cached = self.cache.lock().unwrap().get(&tile).map(Clone::clone); 
        let body = match cached {
            Some(bytes) => bytes.into(),
            None => {
                let bytes: Bytes = Tile::new(tile.clone()).render(
                    &self.features, &self.colors,
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

fn not_found() -> Response<Body> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/plain;charset=utf-8")
        .body(Body::from("not found"))
        .unwrap()
}

pub struct Failed;

