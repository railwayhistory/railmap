//! Rendering an area.

use kurbo::Rect;
use crate::render::canvas::Canvas;
use crate::render::path::Trace;
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

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        canvas.set_line_width(style.dimensions().guide_width);
        style.track_color(&self.class).apply(canvas);
        self.trace.apply(canvas, style);
        canvas.fill().unwrap();
    }
}

