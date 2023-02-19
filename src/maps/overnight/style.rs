//! The style of the map to be rendered.

use std::str::FromStr;
use std::ops::MulAssign;
use lazy_static::lazy_static;
use crate::theme;
use crate::render::color::Color;
use crate::render::path::MapDistance;
use crate::tile::TileId;
use crate::transform::Transform;
use super::class::{RouteColor, Class};
use super::units;


//------------ Configurable Constants ----------------------------------------

/// Size correction for feature bounds.
///
/// This value will be multiplied with detail level, then length and height of
/// the bounding box and then added on each side.
///
/// Increase if features are missing.
const BOUNDS_CORRECTION: f64 = 0.3;


//------------ StyleId -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StyleId;

impl StyleId {
    pub fn detail(self, zoom: u8) -> u8 {
        DETAILS[zoom as usize]
    }

    pub fn mag(self, zoom: u8) -> f64 {
        MAG[zoom as usize]
    }
}

impl FromStr for StyleId {
    type Err = InvalidStyle;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(StyleId),
            "lx" => Ok(StyleId),
            _ => Err(InvalidStyle)
        }
    }
}


//------------ Style ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Style {
    detail: u8,
    mag: f64,
    dimensions: Dimensions,
    transform: Transform,
}

impl theme::Style for Style {
    type StyleId = StyleId;

    fn bounds_correction(&self) -> f64 {
        BOUNDS_CORRECTION * 1.5 * (self.detail as f64)
    }

    fn mag(&self) -> f64 {
        self.mag
    }

    fn detail(&self) -> u8 {
        self.detail
    }

    fn scale(&mut self, canvas_bp: f64) {
        self.dimensions *= canvas_bp;
    }

    fn resolve_distance(&self, distance: MapDistance) -> f64  {
        distance.value()
    }

    fn transform(&self) -> Transform {
        self.transform
    }
}

impl Style {
    pub fn new(id: &TileId<StyleId>) -> Self {
        let detail = id.style.detail(id.zoom);
        let mag = id.style.mag(id.zoom);
        let canvas_bp = id.format.canvas_bp() * mag;
        let scale = id.format.size() * id.n();
        let mut dimensions = if detail < 3 {
            Dimensions::D0
        }
        else if detail < 4 {
            Dimensions::D3
        }
        else {
            Dimensions::D4
        };
        dimensions *= canvas_bp;
        Style {
            detail,
            mag,
            dimensions,
            transform: Transform::new(canvas_bp, id.nw(), scale),
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }
}


//--- Colors
//
impl Style {
    /// Returns the color for a station label.
    pub fn label_color(&self, class: &Class) -> Color {
        COLORS.color(class)
    }

    /// Returns the color for a station marker.
    pub fn marker_color(&self, _class: &Class) -> Color {
        COLORS.black
    }

    /// Returns the color for a route label.
    pub fn route_color(&self, class: &Class) -> Color {
        COLORS.color(class)
    }

    pub fn canvas_bp(&self) -> f64 {
        self.transform.canvas_bp()
    }
}


//============ Detail Levels and Magnifications ==============================

/// The mapping of zoom levels to details.
const DETAILS: &[u8] = &[
    0, 0, 0, 0, 0,
    0, 0, 1, 1, 2,
    2, 3, 4, 4, 5,
    5, 5, 5, 5, 5,
];

/// The mapping of zoom levels to magnification.
const MAG: &[f64] = &[
    1., 1., 1., 1., 1., 
    1., 1., 1., 1., 1., 
    1.3, 1., 1., 1.5, 1.2,
    1.7, 2., 3., 1., 1.,
];


//============ Dimensions ====================================================

/// Adjustable values for the dimensions of things.
///
/// Values here are given in _bp_ normally or in canvas lengths if the
/// dimensions have been gained from style associated with a canvas.
#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
    /// The width of a line tracks.
    pub line_width: f64,

    /// The width of a station, private, or tram track.
    pub other_width: f64,

    pub guide_width: f64,

    /// The length of a segment of markings.
    pub seg: f64,

    /// The distance between two parallel tracks.
    pub dt: f64,

    /// The height of category markings.
    pub mark: f64,

    /// The height of tight single-track markings.
    pub tight_mark: f64,

    /// The width of a station symbol.
    pub sw: f64,

    /// The height of a station symbol.
    pub sh: f64,

    /// The width of a reduced size station symbol.
    pub ksw: f64,

    /// The height of a reduced size station symbol.
    pub ksh: f64,

    /// The radius of curves on station symbols.
    pub ds: f64,

    /// The line width of station symbols.
    pub sp: f64,

    /// The line width of border symbols.
    pub bp: f64,
}

impl Dimensions {
    const D0: Self = Self {
        line_width: 0.6 * units::DT,
        other_width: 0.5,
        guide_width: 0.3,
        seg: 5.0 * units::DT,
        dt: units::DT,
        mark: 0.6 *  units::DT,
        tight_mark: 0.55 * units::DT,
        sw: units::SSW,
        sh: 0.96 * units::SSW,
        ksw: units::SSW,
        ksh: 0.96 * units::SSW,
        ds: 0.05 * units::SSW,
        sp: 0.4,
        bp: 0.4,
    };

    const D3: Self = Self {
        line_width: 1.,
        .. Self::D0
    };

    const D4: Self = Self {
        mark: 0.8 * super::units::DT,
        sw: units::SW,
        sh: units::SH,
        ksw: 0.8 * units::SW,
        ksh: 0.8 * units::SH,
        ds: 0.05 * units::SH,
        sp: 0.8,
        bp: 0.6,
        .. Self::D3
    };

    pub fn mark(&self, tight: bool) -> f64 {
        if tight {
            self.tight_mark
        }
        else {
            self.mark
        }
    }
}

impl MulAssign<f64> for  Dimensions {
    fn mul_assign(&mut self, rhs: f64) {
        self.line_width *= rhs;
        self.other_width *= rhs;
        self.guide_width *= rhs;
        self.seg *= rhs;
        self.dt *= rhs;
        self.mark *= rhs;
        self.tight_mark *= rhs;
        self.sw *= rhs;
        self.sh *= rhs;
        self.ksw *= rhs;
        self.ksh *= rhs;
        self.ds *= rhs;
        self.sp *= rhs;
        self.bp *= rhs;
    }
}


//------------ ColorSet ------------------------------------------------------

#[derive(Clone, Default)]
pub struct ColorSet {
    pub azure: Color,
    pub black: Color,
    pub blue: Color,
    pub brown: Color,
    pub cyan: Color,
    pub green: Color,
    pub orange: Color,
    pub pine: Color,
    pub pink: Color,
    pub purple: Color,
    pub red: Color,
    pub scarlet: Color,
    pub yellow: Color,
}

impl ColorSet {
    fn color(&self, class: &Class) -> Color {
        use self::RouteColor::*;

        match class.color() {
            Azure => self.azure,
            Black => self.black,
            Blue => self.blue,
            Brown => self.brown,
            Cyan => self.cyan,
            Green => self.green,
            Orange => self.orange,
            Pine => self.pine,
            Pink => self.pink,
            Purple => self.purple,
            Red => self.red,
            Scarlet => self.scarlet,
            Yellow => self.yellow,
        }
    }
}

lazy_static! {
    static ref COLORS: ColorSet = {
        ColorSet {
            azure:   Color::hex("2d9edf").unwrap(),
            black:   Color::hex("231f20").unwrap(),
            blue:    Color::hex("263d96").unwrap(),
            brown:   Color::hex("ae530e").unwrap(),
            green:   Color::hex("6ec72e").unwrap(),
            cyan:    Color::hex("87d5b4").unwrap(),
            orange:  Color::hex("f46f1b").unwrap(),
            pine:    Color::hex("0a8137").unwrap(),
            pink:    Color::hex("f468a1").unwrap(),
            purple:  Color::hex("96016e").unwrap(),
            red:     Color::hex("ee2722").unwrap(),
            scarlet: Color::hex("a40000").unwrap(),
            yellow:  Color::hex("fcd006").unwrap(),
        }
    };
}


//------------ InvalidStyle --------------------------------------------------

pub struct InvalidStyle;

