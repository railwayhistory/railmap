//! Coloring rules.

use femtomap::render::Color;
use super::class;

//------------ Colors for individual styles ----------------------------------

pub mod el;
pub mod pax;


//------------ Colors --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Colors {
    El(el::Colors),
    Pax(pax::Colors),
}

impl Colors {
    pub fn el() -> Self {
        Colors::El(Default::default())
    }

    pub fn pax() -> Self {
        Colors::Pax(Default::default())
    }

    pub fn track_color(&self, class: &class::Railway) -> Color {
        match self {
            Colors::El(colors) => colors.track_color(class),
            Colors::Pax(colors) => colors.track_color(class),
        }
    }

    pub fn cat_color(&self, class: &class::Railway) -> Option<Color> {
        match self {
            Colors::El(colors) => colors.cat_color(class),
            Colors::Pax(colors) => colors.cat_color(class),
        }
    }

    pub fn rail_color(&self, class: &class::Railway) -> Option<Color> {
        match self {
            Colors::El(colors) => colors.rail_color(class),
            Colors::Pax(colors) => colors.rail_color(class),
        }
    }

    pub fn label_color(&self, class: &class::Railway) -> Color {
        match self {
            Colors::El(colors) => colors.label_color(class),
            Colors::Pax(colors) => colors.label_color(class),
        }
    }

    pub fn primary_marker_color(&self, class: &class::Railway) -> Color {
        match self {
            Colors::El(colors) => colors.primary_marker_color(class),
            Colors::Pax(colors) => colors.primary_marker_color(class),
        }
    }
}


//------------ ColorSet ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ColorSet {
    pub el: Colors,
    pub pax: Colors,
}

impl Default for ColorSet {
    fn default() -> Self {
        ColorSet {
            el: Colors::el(),
            pax: Colors::pax(),
        }
    }
}

