use std::{fmt, ops};
use std::str::FromStr;
use hyper::Body;

/// The maximum zoom level we support.
///
/// This **must** be less than 32 or stuff will break.
pub const MAX_ZOOM: u8 = 20;


//------------ Tile ----------------------------------------------------------

pub struct Tile {
    id: TileId,
}

impl Tile {
    pub fn new(id: TileId) -> Self {
        Tile { id }
    }

    pub fn content_type(&self) -> &'static str {
        match self.id.format {
            TileFormat::Png => "image/png",
            TileFormat::Svg => "image/svg+xml",
        }
    }

    pub fn into_body(self) -> Body {
        let surface = Surface::new(self.id.format);
        self.render(&surface);
        surface.into_body()
    }

    fn render(&self, surface: &cairo::Surface) {
        let context = cairo::Context::new(surface);
        context.set_source_rgba(1., 0., 0., 0.2);
        context.paint();
    }
}


//------------ TileId --------------------------------------------------------

pub struct TileId {
    pub zoom: u8,
    pub x: u32,
    pub y: u32,
    pub format: TileFormat,
}

impl TileId {
    /// Construct the tile ID from a URI path.
    ///
    /// The format of the path is expected to be:
    ///
    /// ```text
    /// /{zoom}/{x}/{y}.{fmt}
    /// ```
    pub fn from_path(path: &str) -> Result<Self, TileIdError> {
        let mut path = path.split('/');

        // Consume the initial "/"
        if path.next() != Some("") {
            return Err(TileIdError)
        }


        let zoom = u8::from_str(
            path.next().ok_or(TileIdError)?
        ).map_err(|_| TileIdError)?;
        if zoom > MAX_ZOOM {
            return Err(TileIdError);
        }

        let x = u32::from_str(
            path.next().ok_or(TileIdError)?
        ).map_err(|_| TileIdError)?;
        if x >= Self::coord_end(zoom) {
            return Err(TileIdError);
        }

        let mut next = path.next().ok_or(TileIdError)?.split(".");
        let y = u32::from_str(
            next.next().ok_or(TileIdError)?
        ).map_err(|_| TileIdError)?;
        if y >= Self::coord_end(zoom) {
            return Err(TileIdError);
        }

        let format = TileFormat::from_str(
            next.next().ok_or(TileIdError)?
        )?;

        if next.next().is_some() || path.next().is_some() {
            return Err(TileIdError)
        }

        Ok(TileId { zoom, x, y, format })
    }

    /// The upper bound for a coordinate in a zoom level.
    ///
    /// Any coordinate must be less (!) than this value.
    fn coord_end(zoom: u8) -> u32 {
        1 << usize::from(zoom)
    }
}

impl fmt::Display for TileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{}.{}", self.zoom, self.x, self.y, self.format)
    }
}



//------------ TileFormat ----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum TileFormat {
    Png,
    Svg,
}

impl FromStr for TileFormat {
    type Err = TileIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "png" => Ok(TileFormat::Png),
            "svg" => Ok(TileFormat::Svg),
            _ => Err(TileIdError),
        }
    }
}

impl fmt::Display for TileFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            TileFormat::Png => "png",
            TileFormat::Svg => "svg",
        })
    }
}


//------------ Surface -------------------------------------------------------

enum Surface {
    Png(cairo::ImageSurface),
    Svg(cairo::SvgSurface)
}

impl Surface {
    fn new(format: TileFormat) -> Self {
        match format {
            TileFormat::Png => {
                Surface::Png(cairo::ImageSurface::create(
                    cairo::Format::ARgb32, 512, 512
                ).unwrap())
            }
            TileFormat::Svg => {
                // We are assuming 192 dpi resolution at 512 px for now.
                // (That’s .375 pt for each pixel, which means 192 pt for
                // 512 px. I think.)
                Surface::Svg(cairo::SvgSurface::for_stream(
                    192., 192., Vec::new()
                ).unwrap())
            }
        }
    }

    fn into_body(self) -> Body {
        match self {
            Surface::Png(surface) => {
                let mut data = Vec::new();
                surface.write_to_png(&mut data).unwrap();
                data.into()
            }
            Surface::Svg(surface) => {
                let stream = surface.finish_output_stream().unwrap();
                let stream = *(stream.downcast::<Vec<u8>>().unwrap());
                stream.into()
            }
        }
    }
}

impl ops::Deref for Surface {
    type Target = cairo::Surface;

    fn deref(&self) -> &Self::Target {
        match *self {
            Surface::Png(ref surface) => surface,
            Surface::Svg(ref surface) => surface
        }
    }
}


//------------ TileIdError ---------------------------------------------------

pub struct TileIdError;


