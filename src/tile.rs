#![allow(dead_code)]
use std::{fmt, ops};
use std::str::FromStr;
use kurbo::Point;
use crate::railway;


//------------ Configurable Constants ----------------------------------------

/// The maximum zoom level we support.
///
/// This **must** be less than 32 or stuff will break.
const MAX_ZOOM: u8 = 20;


//------------ TileId --------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileId {
    pub layer: LayerId,
    pub zoom: u8,
    pub x: u32,
    pub y: u32,
    pub format: TileFormat,
    pub proof: bool,
}

impl TileId {
    /// Construct the tile ID from a URI path.
    ///
    /// The format of the path is expected to be:
    ///
    /// ```text
    /// {layer}/{zoom}/{x}/{y}.{fmt}
    /// ```
    pub fn from_path(path: &str, proof: bool) -> Result<Self, TileIdError> {
        let mut path = path.split('/');

        let layer = path.next().ok_or(TileIdError)?;
        let layer = LayerId::from_str(layer).map_err(|_| TileIdError)?;

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

        Ok(TileId { layer, zoom, x, y, format, proof, })
    }

    /// The upper bound for a coordinate in a zoom level.
    ///
    /// Any coordinate must be less (!) than this value.
    fn coord_end(zoom: u8) -> u32 {
        1 << usize::from(zoom)
    }

    pub fn n(&self) -> f64 {
        f64::from(Self::coord_end(self.zoom))
    }

    pub fn nw(&self) -> Point {
        Point::new(
            f64::from(self.x) / self.n(),
            f64::from(self.y) / self.n(),
        )
    }

    pub fn scale(&self) -> f64 {
        self.format.size() * self.n()
    }

    pub fn content_type(&self) -> &'static str {
        match self.format {
            TileFormat::Png => "image/png",
            TileFormat::Svg => "image/svg+xml",
        }
    }

    /*
    pub fn is_covered(&self, features: &FeatureSet) -> bool {
        features.is_covered(
            self.style.detail(self.zoom),
            Canvas::calc_feature_bounds(
                Point::new(1., 1.), self.nw(), self.n()
            )
        )
    }
    */

    pub fn render(self, map: &railway::Map) -> Vec<u8> {
        let surface = Surface::new(self.format);
        let _ = map.render(self, &surface);
        surface.finalize()
    }
}

impl fmt::Display for TileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{}.{}", self.zoom, self.x, self.y, self.format)
    }
}


//------------ LayerId -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LayerId {
    Railway(railway::LayerId),
}

impl FromStr for LayerId {
    type Err = TileIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        railway::LayerId::from_str(s).map(LayerId::Railway)
    }
}


//------------ TileFormat ----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TileFormat {
    Png,
    Svg,
}

impl TileFormat {
    pub fn size(self) -> f64 {
        match self {
            TileFormat::Png => 512.,
            TileFormat::Svg => 192.,
        }
    }
    
    pub fn canvas_bp(self) -> f64 {
        match self {
            TileFormat::Png => 192./72.,
            TileFormat::Svg => 1.,
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            TileFormat::Png => "image/png",
            TileFormat::Svg => "image/svg+xml",
        }
    }
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

pub enum Surface {
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

    pub fn new_map_key(format: TileFormat, size: Point) -> Self {
        let size = Point::new(
            size.x * format.canvas_bp(),
            size.y * format.canvas_bp(),
        );
        match format {
            TileFormat::Png => {
                Surface::Png(cairo::ImageSurface::create(
                    cairo::Format::ARgb32, size.x as i32, size.y as i32,
                ).unwrap())
            }
            TileFormat::Svg => {
                // We are assuming 192 dpi resolution at 512 px for now.
                // (That’s .375 pt for each pixel, which means 192 pt for
                // 512 px. I think.)
                Surface::Svg(cairo::SvgSurface::for_stream(
                    size.x, size.y, Vec::new()
                ).unwrap())
            }
        }
    }

    /*
    fn size(&self) -> f64 {
        match *self {
            Surface::Png(_) => 512.,
            Surface::Svg(_) => 192.,
        }
    }
    
    fn canvas_bp(&self) -> f64 {
        match *self {
            Surface::Png(_) => 192./72.,
            Surface::Svg(_) => 1.,
        }
    }
    */

    pub fn finalize(self) -> Vec<u8> {
        match self {
            Surface::Png(surface) => {
                let mut data = Vec::new();
                surface.write_to_png(&mut data).unwrap();
                data
            }
            Surface::Svg(surface) => {
                let stream = surface.finish_output_stream().unwrap();
                let stream = *(stream.downcast::<Vec<u8>>().unwrap());
                stream
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

