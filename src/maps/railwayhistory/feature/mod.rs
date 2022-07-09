pub mod area;
pub mod border;
pub mod guide;
pub mod label;
pub mod markers;
pub mod track;

use kurbo::Rect;
use crate::render::canvas::Canvas;
use super::Railwayhistory;
use super::style::Style;

pub enum Feature {
    Area(area::AreaContour),
    Border(border::BorderContour),
    Casing(track::TrackCasing),
    Guide(guide::GuideContour),
    Label(label::Feature),
    Marker(markers::StandardMarker),
    Track(track::TrackContour),
}

impl crate::render::feature::Feature<Railwayhistory> for Feature {
    fn storage_bounds(&self) -> Rect {
        match self {
            Feature::Area(value) => value.storage_bounds(),
            Feature::Border(value) => value.storage_bounds(),
            Feature::Casing(value) => value.storage_bounds(),
            Feature::Guide(value) => value.storage_bounds(),
            Feature::Label(value) => value.storage_bounds(),
            Feature::Marker(value) => value.storage_bounds(),
            Feature::Track(value) => value.storage_bounds(),
        }
    }

    fn render(&self, style: &Style, canvas: &Canvas, depth: usize) {
        match self {
            Feature::Area(value) => value.render(style, canvas),
            Feature::Border(value) => value.render(style, canvas),
            Feature::Casing(value) => value.render(style, canvas),
            Feature::Guide(value) => value.render(style, canvas),
            Feature::Label(value) => value.render(style, canvas),
            Feature::Marker(value) => value.render(style, canvas),
            Feature::Track(value) => value.render(style, canvas, depth),
        }
    }
}

