#![allow(dead_code, unused_imports, unused_variables)]

pub mod area;
pub mod border;
pub mod dot;
pub mod guide;
pub mod label;
pub mod markers;
pub mod track;

use femtomap::render::canvas;
use femtomap::world;
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
        /*
    pub fn render(&self, _style: &Style, _canvas: &Canvas) {
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
        */
}

impl femtomap::feature::Feature for Feature {
    type Style = Style;
    type Shape<'a> = Box<dyn Shape + 'a>;

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

    fn shape(
        &self, style: &Style, canvas: &canvas::Canvas
    ) -> Option<Box<dyn Shape + '_>> {
        match self {
            Feature::Area(value) => Some(value.shape(style, canvas)),
            Feature::Border(value) => Some(value.shape(style, canvas)),
            Feature::Casing(value) => Some(value.shape(style, canvas)),
            Feature::Dot(value) => Some(value.shape(style, canvas)),
            Feature::Guide(value) => Some(value.shape(style, canvas)),
            Feature::Label(value) => value.shape(style, canvas),
            Feature::Marker(value) => Some(value.shape(style, canvas)),
            Feature::Platform(value) => Some(value.shape(style, canvas)),
            Feature::Track(value) => Some(value.shape(style, canvas)),
        }
    }
}

pub trait Shape {
    fn render(&self, stage: Stage, style: &Style, canvas: canvas::Group);
}

impl<'a, S: Shape + 'a> From<S> for Box<dyn Shape + 'a> {
    fn from(src: S) -> Self {
        Box::new(src)
    }
}

impl<F: Fn(&Style, canvas::Group)> Shape for F {
    fn render(&self, stage: Stage, style: &Style, canvas: canvas::Group) {
        if matches!(stage, Stage::Base) {
           (*self)(style, canvas)
        }
    }
}


//------------ Stage ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum Stage {
    #[default]
    Back,
    Casing,
    Base,
    Inside,
}

impl IntoIterator for Stage {
    type Item = Self;
    type IntoIter = StageIter;

    fn into_iter(self) -> Self::IntoIter {
        StageIter(Some(self))
    }
}


#[derive(Clone, Copy, Debug)]
pub struct StageIter(Option<Stage>);

impl Iterator for StageIter {
    type Item = Stage;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0;
        if let Some(stage) = self.0 {
            let next = match stage {
                Stage::Back => Some(Stage::Casing),
                Stage::Casing => Some(Stage::Base),
                Stage::Base => Some(Stage::Inside),
                Stage::Inside => None
            };
            self.0 = next;
        }
        res 
    }
}

