/// A feature drawing a line.
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
    render: Box<dyn RenderContour>,
}

impl Contour {
    pub fn simple(path: Path, color: Color, width: f64) -> Contour {
        Contour {
            path,
            render: Box::new(move |canvas: &Canvas, path: &Path| {
                color.apply(canvas);
                canvas.set_line_width(width * canvas.canvas_bp());
                path.apply(canvas);
                canvas.stroke();
            })
        }
    }

    pub fn bounding_box(&self) -> Rect {
        self.path.bounding_box()
    }

    pub fn render(&self, canvas: &Canvas) {
        self.render.render(canvas, &self.path)
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

