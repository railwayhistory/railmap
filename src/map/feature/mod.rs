pub mod area;
pub mod border;
pub mod dot;
pub mod guide;
pub mod label;
pub mod markers;
pub mod track;

use femtomap::world;
use crate::render::canvas::Canvas;
//use super::Railwayhistory;
use super::style::Style;


pub enum Feature {
    Area(area::AreaContour),
    Border(border::BorderContour),
    Casing(track::TrackCasing),
    Dot(dot::DotMarker),
    Guide(guide::GuideContour),
    Label(label::Feature),
    Marker(markers::StandardMarker),
    Platform(area::PlatformContour),
    Track(track::TrackContour),
}

impl Feature {
    pub fn render(&self, style: &Style, canvas: &Canvas) {
        match self {
            Feature::Area(value) => value.render(style, canvas),
            Feature::Border(value) => value.render(style, canvas),
            Feature::Casing(value) => value.render(style, canvas),
            Feature::Dot(value) => value.render(style, canvas),
            Feature::Guide(value) => value.render(style, canvas),
            Feature::Label(value) => value.render(style, canvas),
            Feature::Marker(value) => value.render(style, canvas),
            Feature::Platform(value) => value.render(style, canvas),
            Feature::Track(value) => value.render(style, canvas, 0),
        }
    }
}

impl femtomap::feature::Feature for Feature {
    type Style = Style;
    type Shape<'a> = &'a Self;

    fn storage_bounds(&self) -> world::Rect {
        match self {
            Feature::Area(value) => value.storage_bounds(),
            Feature::Border(value) => value.storage_bounds(),
            Feature::Casing(value) => value.storage_bounds(),
            Feature::Dot(value) => value.storage_bounds(),
            Feature::Guide(value) => value.storage_bounds(),
            Feature::Label(value) => value.storage_bounds(),
            Feature::Marker(value) => value.storage_bounds(),
            Feature::Platform(value) => value.storage_bounds(),
            Feature::Track(value) => value.storage_bounds(),
        }.into()
    }

    fn shape(&self, _: &Style) -> &Self {
        self
    }
}

