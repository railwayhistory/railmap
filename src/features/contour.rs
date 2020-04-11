/// A feature drawing a line.
use std::fmt;
use std::sync::Arc;
use kurbo::Rect;
use crate::canvas::Canvas;
use super::color::Color;
use super::path::Path;


//------------ Contour -------------------------------------------------------

/// A feature drawing a line.
///
pub struct Contour {
    /// The path that is being rendered by the contour.
    path: Path,

    /// The renderer for this contour.
    ///
    render: ContourRule,
}

impl Contour {
    pub fn new(path: Path, render: ContourRule) -> Self {
        Contour { path, render }
    }

    pub fn bounding_box(&self) -> Rect {
        self.path.bounding_box()
    }

    pub fn render(&self, canvas: &Canvas) {
        self.render.0.render(canvas, &self.path)
    }
}


pub trait RenderContour: Send + Sync + 'static {
    fn render(&self, canvas: &Canvas, path: &Path);
}

impl<F: Fn(&Canvas, &Path) + Send + Sync + 'static> RenderContour for F {
    fn render(&self, canvas: &Canvas, path: &Path) {
        (*self)(canvas, path)
    }
}

#[derive(Clone)]
pub struct ContourRule(Arc<dyn RenderContour>);

impl fmt::Debug for ContourRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContourRule(...)")
    }
}


//------------ Contour Rendering Rules ---------------------------------------

pub fn simple(color: Color, width: f64) -> ContourRule {
    ContourRule(Arc::new(move |canvas: &Canvas, path: &Path| {
        color.apply(canvas);
        canvas.set_line_width(width * canvas.canvas_bp());
        path.apply(canvas);
        canvas.stroke();
    }))
}
