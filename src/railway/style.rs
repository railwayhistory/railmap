//! The rendering style.

use femtomap::path::{MapDistance, Transform};
use femtomap::render::Color;
use kurbo::{TranslateScale, Vec2};
use crate::tile::TileId;
use super::{class, measures};
use super::colors::{Colors, ColorSet};
use super::map::LayerId;
use super::measures::Measures;

// This module is organized slightly backwards: All the tweakable stuff is up
// top and the actual `Style` type is way at the bottom.


//============ Detail Levels and Magnifications ==============================

#[derive(Clone, Copy, Debug)]
struct Zoom {
    store_scale: f64,
    detail: u8,
    mag: f64,
    measures: Measures,
}

impl Zoom {
    const fn new(
        store_scale: f64, detail: u8, mag: f64, measures: Measures,
    ) -> Self {
        Zoom { store_scale, detail, mag, measures, }
    }
}

const ZOOM: &[Zoom] = &[
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),  // 0
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),  // 5
    Zoom::new(0.5, 0, 1.0, measures::BASE_D0),
    Zoom::new(1.0, 1, 1.0, measures::BASE_D1),
    Zoom::new(1.5, 1, 1.3, measures::BASE_D1),
    Zoom::new(2.0, 2, 1.0, measures::BASE_D2),
    Zoom::new(2.5, 2, 1.3, measures::BASE_D2), // 10
    Zoom::new(3.0, 3, 1.3, measures::BASE_D3),
    Zoom::new(3.5, 3, 1.6, measures::BASE_D3),
    Zoom::new(4.0, 4, 1.3, measures::BASE_D4),
    Zoom::new(4.5, 4, 1.6, measures::BASE_D4),
    Zoom::new(5.0, 5, 1.3, measures::BASE_D4), // 15
    Zoom::new(5.5, 5, 1.6, measures::BASE_D4),
    Zoom::new(5.5, 5, 1.9, measures::BASE_D4),
    Zoom::new(5.5, 5, 2.2, measures::BASE_D4),
    Zoom::new(5.5, 5, 2.5, measures::BASE_D4),
];

const PROOF_ZOOM: &[Zoom] = &[
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),  // 0
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),
    Zoom::new(0.0, 0, 1.0, measures::BASE_D0),  // 5
    Zoom::new(0.5, 0, 1.0, measures::BASE_D0),
    Zoom::new(1.0, 1, 1.0, measures::BASE_D1),
    Zoom::new(1.5, 1, 1.3, measures::BASE_D1),
    Zoom::new(2.0, 2, 1.0, measures::BASE_D2),
    Zoom::new(3.0, 3, 1.0, measures::BASE_D3), // 10
    Zoom::new(3.5, 3, 1.3, measures::BASE_D3),
    Zoom::new(4.0, 4, 1.0, measures::BASE_D4),
    Zoom::new(4.5, 4, 1.3, measures::BASE_D4),
    Zoom::new(5.0, 5, 1.0, measures::BASE_D4),
    Zoom::new(5.5, 5, 1.3, measures::BASE_D4), // 15
    Zoom::new(5.5, 5, 1.6, measures::BASE_D4),
    Zoom::new(5.5, 5, 1.9, measures::BASE_D4),
    Zoom::new(5.5, 5, 2.1, measures::BASE_D4),
];

/// Size correction for feature bounds.
///
/// This value will be multiplied with detail level, then length and height of
/// the bounding box and then added on each side.
///
/// Increase if features are missing.
const BOUNDS_CORRECTION: f64 = 0.3;


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

    /// The measures for this style.
    ///
    /// These are already scaled into canvas co-ordinates.
    measures: Measures,

    /// The map unit array for use with Femtomap transformation.
    map_units: [f64; 13],

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
}

impl Style {
    pub fn new(layer_id: LayerId, tile_id: &TileId, colors: &ColorSet) -> Self {
        let zoom = if tile_id.proof {
            PROOF_ZOOM[usize::from(tile_id.zoom)]
        }
        else {
            ZOOM[usize::from(tile_id.zoom)]
        };
        let measures = zoom.measures * tile_id.format.canvas_bp() * zoom.mag;
        let equator_scale = tile_id.scale();
        let style_id = layer_id.style_id();
        let latin_text = layer_id.latin_text();

        Self {
            store_scale: zoom.store_scale,
            detail: zoom.detail,
            pax_only: matches!(style_id, StyleId::Pax),
            map_units: measures.map_units(),
            measures,
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

    pub fn measures(&self) -> Measures {
        self.measures
    }

    pub fn latin_text(&self) -> bool {
        self.latin_text
    }

    pub fn canvas_bp(&self) -> f64 {
        self.measures.bp()
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
        BOUNDS_CORRECTION
            * if self.detail == 0 { 1. } else { self.detail as f64 }
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
        self.canvas_bp()
    }
}

