//! Rendering an area.

use crate::canvas::Canvas;
use crate::features::contour::RenderContour;
use crate::features::path::Path;
use super::super::class::Class;

//------------ AreaContour ---------------------------------------------------

/// A contour drawing an area.
pub struct AreaContour {
    class: Class
}

impl AreaContour {
    pub fn new(class: Class) -> Self {
        AreaContour { class }
    }
}

impl RenderContour for AreaContour {
    fn render(&self, canvas: &Canvas, path: &Path) {
        canvas.set_line_width(canvas.style().dimensions().guide_width);
        canvas.style().track_color(&self.class).apply(canvas);
        path.apply(canvas);
        canvas.fill();
    }
}

