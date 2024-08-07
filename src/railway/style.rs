//! The rendering style.

use std::ops;
use femtomap::path::{MapDistance, Transform};
use femtomap::render::Color;
use kurbo::{TranslateScale, Vec2};
use crate::tile::TileId;
use super::class;
use super::colors::{Colors, ColorSet};
use super::import::units::MM;
use super::map::LayerId;

// This module is organized slightly backwards: All the tweakable stuff is up
// top and the actual `Style` type is way at the bottom.


//============ Detail Levels and Magnifications ==============================

#[derive(Clone, Copy, Debug)]
struct Zoom {
    store_scale: f64,
    detail: u8,
    mag: f64,
}

impl Zoom {
    const fn new(store_scale: f64, detail: u8, mag: f64) -> Self {
        Zoom { store_scale, detail, mag }
    }
}

const ZOOM: &[Zoom] = &[
    Zoom::new(0.0, 0, 1.0),  // 0
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),  // 5
    Zoom::new(0.5, 0, 1.0),
    Zoom::new(1.0, 1, 1.0),
    Zoom::new(1.5, 1, 1.3),
    Zoom::new(2.0, 2, 1.0),
    Zoom::new(2.5, 2, 1.3), // 10
    Zoom::new(3.0, 3, 1.3),
    Zoom::new(3.5, 3, 1.6),
    Zoom::new(4.0, 4, 1.3),
    Zoom::new(4.5, 4, 1.6),
    Zoom::new(5.0, 5, 1.3), // 15
    Zoom::new(5.5, 5, 1.6),
    Zoom::new(5.5, 5, 1.9),
    Zoom::new(5.5, 5, 2.2),
    Zoom::new(5.5, 5, 2.5),
];

const PROOF_ZOOM: &[Zoom] = &[
    Zoom::new(0.0, 0, 1.0),  // 0
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),
    Zoom::new(0.0, 0, 1.0),  // 5
    Zoom::new(0.5, 0, 1.0),
    Zoom::new(1.0, 1, 1.0),
    Zoom::new(1.5, 1, 1.3),
    Zoom::new(2.0, 2, 1.0),
    Zoom::new(3.0, 3, 1.0), // 10
    Zoom::new(3.5, 3, 1.3),
    Zoom::new(4.0, 4, 1.0),
    Zoom::new(4.5, 4, 1.3),
    Zoom::new(5.0, 5, 1.0),
    Zoom::new(5.5, 5, 1.3), // 15
    Zoom::new(5.5, 5, 1.6),
    Zoom::new(5.5, 5, 1.9),
    Zoom::new(5.5, 5, 2.1),
];

/// Size correction for feature bounds.
///
/// This value will be multiplied with detail level, then length and height of
/// the bounding box and then added on each side.
///
/// Increase if features are missing.
const BOUNDS_CORRECTION: f64 = 0.3;


//============ Distances =====================================================
//
// While world distances are always kept in bp and just need to be scaled
// according to the map’s scale at the point where they apply, map distances
// are subject to interpretation by the style. The importer translates the
// unit into a value and an index into a unit array. When creating the style,
// this unit array has to be created.
//
// In addition, we keep a number of map distance units that aren’t available
// in the importer but are used by the features when rendering. We have a
// regular struct for that and also the array for imported distances. The
// latter is then created from the former during style initialization.

/// Named map distance units.
#[derive(Clone, Copy, Debug, Default)]
pub struct Units {
    /// The size of a bp.
    ///
    /// This needs to be included here because the values of this struct
    /// are scaled to canvas coordinates.
    pub bp: f64,

    /// The distance between two parallel tracks.
    ///
    /// This serves as a base unit for everything else.
    pub dt: f64,

    /// The Length of a cross-over between two parallel tracks.
    pub dl: f64,

    /// The stroke width of main line track.
    pub line_width: f64,

    /// The stroke width of any other track.
    pub other_width: f64,

    /// The stroke width of line markings.
    pub mark_width: f64,

    /// The stroke width of a guiding line.
    pub guide_width: f64,

    /// The stroke width of border symbols.
    pub border_width: f64,

    /// The standard length of a segment of line markings.
    pub seg: f64,

    /// The width of a station symbol.
    pub sw: f64,

    /// The height of a station symbol.
    pub sh: f64,

    /// The radius of curves on station symbols.
    pub ds: f64,

    /// The stroke width of station symbols.
    pub sp: f64,
}


impl Units {
    fn new(detail: u8, mag: f64) -> Self {
        (match detail {
            0 => Self::d0(),
            1 => Self::d1(),
            2 => Self::d2(),
            3 => Self::d3(),
            4 => Self::d4(),
            _ => Self::d5(),
        }) * mag
    }

    /// Creates the value for detail level 0.
    fn d0() -> Self {
        Self {
            line_width: 0.8,
            other_width: 0.5,
            mark_width: 0.5,
            guide_width: 0.3,
            border_width: 0.4,
            .. Self::standard(0.75 * MM, 1.2 * MM, 1.125 * MM)
        }
    }

    /// Creates the value for detail level 1.
    fn d1() -> Self {
        Self::d0()
    }

    /// Creates the value for detail level 2.
    fn d2() -> Self {
        Self {
            line_width: 1.1,
            other_width: 0.6,
            .. Self::d0()
        }
    }

    /// Creates the value for detail level 3.
    fn d3() -> Self {
        Self {
            line_width: 1.,
            other_width: 0.7,
            mark_width: 0.7,
            guide_width: 0.3,
            border_width: 0.4,
            .. Self::standard(0.6 * MM, 1.2 * MM, 1.35 * MM)
        }
    }

    /// Creates the value for detail level 4.
    fn d4() -> Self {
        Self {
            line_width: 0.8,
            other_width: 0.7,
            mark_width: 0.5,
            guide_width: 0.5,
            border_width: 0.6,
            .. Self::standard(0.75 * MM, 2.4 * MM, 2.25 * MM)
        }
    }

    /// Creates the value for detail level 2.
    fn d5() -> Self {
        Self::d4()
    }

    /// Creates a value based on the value of _dt_ and _sw._
    fn standard(dt: f64, sw: f64, sh: f64) -> Self {
        Self {
            bp: 1.,
            dt,
            dl: 0.66 * dt,
            seg: 6. * dt,
            sw: sw,
            sh: sh,
            sp: 1.0,
            .. Default::default()
        }
    }

    /// Creates the map unit array.
    fn map_units(self) -> [f64; 6] {
        [
            self.bp,
            self.dt,
            self.dl,
            self.line_width, // "st"
            self.sw,
            self.sh,
        ]
    }
}

impl ops::Mul<f64> for Units {
    type Output = Self;

    fn mul(self, mag: f64) -> Self {
        Self {
            bp: self.bp * mag,
            dt: self.dt * mag,
            dl: self.dl * mag,
            line_width: self.line_width * mag,
            other_width: self.other_width * mag,
            mark_width: self.mark_width * mag,
            guide_width: self.guide_width * mag,
            border_width: self.border_width * mag,
            seg: self.seg * mag,
            sw: self.sw * mag,
            sh: self.sh * mag,
            ds: self.ds * mag,
            sp: self.sp * mag,
        }
    }
}


//============ Style =========================================================

//------------ StyleId -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StyleId {
    /// Electrification map.
    El,

    /// Passenger map.
    Pax,
}

impl StyleId {
    fn colors(self, colors: &ColorSet) -> Colors {
        match self {
            StyleId::El => colors.el,
            StyleId::Pax  => colors.pax,
        }
    }
}


//------------ Style --------------------------------------------------------- 

pub struct Style {
    /// The scale value for the feature store.
    store_scale: f64,

    /// The detail level.
    detail: u8,

    /// Is this a pax-only map?
    pax_only: bool,

    /// The map units definition.
    ///
    /// These are already scaled into canvas co-ordinates.
    units: Units,

    /// The map unit array for use with Femtomap transformation.
    map_units: [f64; 6],

    /// The coloring rules.
    colors: Colors,

    /// Are we using latin text only?
    latin_text: bool,

    /// The transformation from storage to canvas coordinates.
    ///
    /// Storage coordinates are Spherical Mercator with a range of `0. .. 1.`
    /// for both x and y. Because we are only supporting Spherical Mercator
    /// for output, too, we can use scaling and translation for this.
    ///
    /// Note that in a `TranslateScale` the scaling happens first and the
    /// translation needs to be in scaled up coordinates.
    transform: TranslateScale,

    /// The size of a bp in storage coordinates.
    equator_scale: f64,

    /// The size of a bp in canvas coordinates.
    canvas_bp: f64,
}

impl Style {
    pub fn new(layer_id: LayerId, tile_id: &TileId, colors: &ColorSet) -> Self {
        let zoom = if tile_id.proof {
            PROOF_ZOOM[usize::from(tile_id.zoom)]
        }
        else {
            ZOOM[usize::from(tile_id.zoom)]
        };
        let canvas_bp = tile_id.format.canvas_bp() * zoom.mag;
        let units = Units::new(zoom.detail, canvas_bp);
        let equator_scale = tile_id.scale();
        let style_id = layer_id.style_id();
        let latin_text = layer_id.latin_text();

        Self {
            store_scale: zoom.store_scale,
            detail: zoom.detail,
            pax_only: matches!(style_id, StyleId::Pax),
            map_units: units.map_units(),
            units,
            colors: style_id.colors(colors),
            latin_text,
            transform: TranslateScale::new(
                Vec2::new(
                    -tile_id.nw().x * equator_scale,
                    -tile_id.nw().y * equator_scale
                ),
                equator_scale
            ),
            equator_scale,
            canvas_bp,
        }
    }

    pub fn store_scale(&self) -> f64 {
        self.store_scale
    }

    pub fn detail(&self) -> u8 {
        self.detail
    }

    pub fn pax_only(&self) -> bool {
        self.pax_only
    }

    pub fn units(&self) -> Units {
        self.units
    }

    pub fn latin_text(&self) -> bool {
        self.latin_text
    }

    pub fn canvas_bp(&self) -> f64 {
        self.canvas_bp
    }

    pub fn track_color(&self, class: &class::Railway) -> Color {
        self.colors.track_color(class)
    }

    pub fn cat_color(&self, class: &class::Railway) -> Option<Color> {
        self.colors.cat_color(class)
    }

    pub fn rail_color(&self, class: &class::Railway) -> Option<Color> {
        self.colors.rail_color(class)
    }

    pub fn label_color(&self, class: &class::Railway) -> Color {
        self.colors.label_color(class)
    }

    pub fn primary_marker_color(&self, class: &class::Railway) -> Color {
        self.colors.primary_marker_color(class)
    }

    pub fn casing_color(&self) -> Color {
        self.colors.casing_color()
    }

    pub fn bounds_correction(&self) -> f64 {
        BOUNDS_CORRECTION * (self.detail as f64)
    }
}

impl Transform for Style {
    fn distance(&self, distance: MapDistance) -> f64 {
        distance.value() * self.map_units[distance.unit()]
    }

    fn transform(&self) -> TranslateScale {
        self.transform
    }

    fn equator_scale(&self) -> f64 {
        self.equator_scale
    }

    fn canvas_bp(&self) -> f64 {
        self.canvas_bp
    }
}

