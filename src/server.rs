use std::io;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use lru::LruCache;
use tokio::net::TcpListener;
use crate::railway;
use crate::tile::TileId;


#[derive(Clone)]
pub struct Server {
    railway: Arc<railway::Map>,
    cache: Arc<Mutex<LruCache<TileId, Bytes>>>,
}


impl Server {
    pub fn new(railway: railway::Map) -> Self {
        Server {
            railway: Arc::new(railway),
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(10_000).unwrap())
            )),
        }
    }
}

impl Server {
    pub async fn run(&self, addr: SocketAddr) -> Result<(), io::Error>{
        let listener = TcpListener::bind(addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let stream = TokioIo::new(stream);
            let this = self.clone();
            tokio::task::spawn(async move {
                http1::Builder::new().serve_connection(
                    stream,
                    service_fn(|r| {
                        let this = this.clone();
                        async move { this.process(r).await }
                    })
                ).await
            });
        }
    }
}

impl Server {
    async fn process(
        &self, request: Request<Incoming>
    ) -> Result<Response<Full<Bytes>>, Infallible> {
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
                let bytes: Bytes = tile.render(&self.railway).into();
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

fn not_found() -> Response<Full<Bytes>> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/plain;charset=utf-8")
        .body(Full::new(Bytes::from("not found")))
        .unwrap()
}

pub struct Failed;

