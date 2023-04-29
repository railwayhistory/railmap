//! Rendering an area.

use femtomap::path::Trace;
use femtomap::render::Canvas;
use kurbo::{BezPath,Rect};
use super::Shape;
use super::super::class::Class;
use super::super::style::Style;

//------------ AreaContour ---------------------------------------------------

/// A contour drawing an area.
pub struct AreaContour {
    class: Class,
    trace: Trace,
}

impl AreaContour {
    pub fn new(class: Class, trace: Trace) -> Self {
        AreaContour { class, trace }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> Box<dyn Shape> {
        let color = style.track_color(&self.class);
        let outline = self.trace.outline(style);

        Box::new(move |_style: &Style, canvas: &mut Canvas| {
            let mut canvas = canvas.sketch();
            canvas.apply(&outline);
            canvas.apply(color);
            canvas.fill();
        })
    }
}


//------------ PlatformContour -----------------------------------------------

/// A contour drawing an area.
pub struct PlatformContour {
    class: Class,
    trace: Trace,
}

impl PlatformContour {
    pub fn new(class: Class, trace: Trace) -> Self {
        PlatformContour { class, trace }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> Box<dyn Shape> {
        let color = style.track_color(&self.class);
        let outline = self.trace.outline(style);

        Box::new(move |_style: &Style, canvas: &mut Canvas| {
            let mut canvas = canvas.sketch();
            canvas.apply(&outline);
            canvas.apply(color);
            canvas.fill();
        })
    }
}

