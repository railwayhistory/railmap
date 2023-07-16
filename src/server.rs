use std::convert::Infallible;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use femtomap::feature::FeatureSet;
use hyper::{Body, Request, Response};
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use lru::LruCache;
use crate::tile::{Tile, TileId, TileFormat};
use crate::theme::{Style, Theme};


pub struct Server<T: Theme> {
    theme: T,
    features: Arc<FeatureSet<T::Feature>>,
    cache: Arc<Mutex<LruCache<TileId<<T::Style as Style>::StyleId>, Bytes>>>,
    test_style: String,
}


impl<T: Theme> Server<T> {
    pub fn new(
        theme: T,
        features: FeatureSet<T::Feature>,
        test_style: String,
    ) -> Self {
        Server {
            theme,
            features: Arc::new(features),
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(10_000).unwrap())
            )),
            test_style,
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
        let path = request.uri().path();

        match path {
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
                    match <T::Style as Style>::StyleId::from_str(style) {
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

        let tile = match TileId::from_path(
            &request.uri().path()[1..],
            &self.test_style,
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
            cache: self.cache.clone(),
            test_style: self.test_style.clone(),
        }
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

