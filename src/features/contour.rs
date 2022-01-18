/// A feature drawing a line.
use std::fmt;
use std::sync::Arc;
use kurbo::Rect;
use smallvec::SmallVec;
use crate::canvas::Canvas;
use super::color::Color;
use super::path::Path;


//------------ Contour -------------------------------------------------------

/// A feature drawing a line.
///
pub struct Contour {
    /// The path that is being rendered by the contour.
    path: Path,

    /// The renderers for this contour.
    ///
    render: SmallVec<[ContourRule; 3]>,
}

impl Contour {
    pub fn new(
        path: Path,
        render: impl Into<SmallVec<[ContourRule; 3]>>
    ) -> Self {
        Contour { path, render: render.into() }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.path.storage_bounds()
    }

    pub fn render(&self, canvas: &Canvas) {
        for item in &self.render {
            item.0.render(canvas, &self.path)
        }
    }
}


pub trait RenderContour: Send + Sync + 'static {
    fn render(&self, canvas: &Canvas, path: &Path);

    fn into_rule(self) -> ContourRule
    where Self: Sized {
        ContourRule(Arc::new(self))
    }
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

pub fn dashed_line(
    color: Color,
    width: f64,
    on: f64,
    off: f64,
    offset: Option<f64>,
) -> ContourRule {
    ContourRule(Arc::new(move |canvas: &Canvas, path: &Path| {
        let on = on * canvas.canvas_bp();
        let off = off * canvas.canvas_bp();
        color.apply(canvas);
        canvas.set_line_width(width * canvas.canvas_bp());
        canvas.set_dash(&[on, off], on + off / 2.);
        match offset {
            Some(offset) => {
                path.apply_offset(offset * canvas.canvas_bp(), canvas)
            }
            None => path.apply(canvas),
        }
        canvas.stroke();
        canvas.set_dash(&[], 0.);
    }))
}

pub fn fill(color: Color) -> ContourRule {
    ContourRule(Arc::new(move |canvas: &Canvas, path: &Path| {
        color.apply(canvas);
        path.apply(canvas);
        canvas.fill();
    }))
}

