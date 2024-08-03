use std::io;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use lru::LruCache;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use crate::railway;
use crate::tile::TileId;


//------------ Server --------------------------------------------------------

pub struct Server {
    railway: ArcSwap<railway::Map>,
    cache: Arc<Mutex<LruCache<TileId, Bytes>>>,
    rx: Option<mpsc::Receiver<ServerCommand>>,
    proof: bool,
}


impl Server {
    pub fn new(railway: railway::Map, proof: bool) -> (Self, ServerControl) {
        let (tx, rx) = mpsc::channel(10);
        (
            Server {
                railway: Arc::new(railway).into(),
                cache: Arc::new(Mutex::new(
                    LruCache::new(NonZeroUsize::new(10_000).unwrap())
                )),
                rx: Some(rx),
                proof,
            },
            ServerControl { tx },
        )
    }
}

impl Server {
    pub async fn run(mut self, addr: SocketAddr) -> Result<(), io::Error> {
        let listener = TcpListener::bind(addr).await?;
        let rx = self.rx.take().unwrap();
        let this = Arc::new(self);
        tokio::spawn(this.clone().run_control(rx));
        loop {
            let (stream, _) = listener.accept().await?;
            let stream = TokioIo::new(stream);
            let this = this.clone();
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

    async fn run_control(
        self: Arc<Self>, mut rx: mpsc::Receiver<ServerCommand>
    ) {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                ServerCommand::UpdateRailway(map) => {
                    self.railway.store(map.into());
                    self.cache.lock().unwrap().clear();
                }
            }
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
            &request.uri().path()[1..], self.proof,
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
                let bytes: Bytes = tile.render(&self.railway.load()).into();
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


//------------ ServerControl -------------------------------------------------

#[derive(Clone)]
pub struct ServerControl {
    tx: mpsc::Sender<ServerCommand>,
}

impl ServerControl {
    pub async fn update_railway(&self, map: railway::Map) {
        let _ = self.tx.send(ServerCommand::UpdateRailway(map)).await;
    }
}


//------------ ServerCommand -------------------------------------------------

enum ServerCommand {
    UpdateRailway(railway::Map),
}

