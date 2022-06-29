pub mod border;
pub mod guide;
pub mod label;
pub mod markers;
pub mod route;

use kurbo::Rect;
use crate::render::canvas::Canvas;
use crate::render::label::Label;
use super::Overnight;
use super::style::Style;

pub enum Feature {
    Border(self::border::BorderContour),
    Guide(self::guide::GuideContour),
    Label(Label<Overnight>),
    Route(self::route::RouteContour),
    Statdot(self::markers::Statdot),
}

impl crate::render::feature::Feature<Overnight> for Feature {
    fn storage_bounds(&self) -> Rect {
        match self {
            Feature::Border(value) => value.storage_bounds(),
            Feature::Guide(value) => value.storage_bounds(),
            Feature::Label(value) => value.storage_bounds(),
            Feature::Route(value) => value.storage_bounds(),
            Feature::Statdot(value) => value.storage_bounds(),
        }
    }

    fn render(&self, style: &Style, canvas: &Canvas) {
        match self {
            Feature::Border(value) => value.render(style, canvas),
            Feature::Guide(value) => value.render(style, canvas),
            Feature::Label(value) => value.render(style, canvas),
            Feature::Route(value) => value.render(style, canvas),
            Feature::Statdot(value) => value.render(style, canvas),
        }
    }
}

