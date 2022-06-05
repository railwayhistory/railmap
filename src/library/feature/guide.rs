//! Rendering of guides.
//!
//! Guides are thin lines attaching a label to something.

use crate::canvas::Canvas;
use crate::features::contour::RenderContour;
use crate::features::path::Path;
use crate::import::eval;
use crate::import::Failed;
use super::super::class::Class;

//------------ GuideContour --------------------------------------------------

/// A contour drawing a guide.
pub struct GuideContour {
    /// The class of the guide,
    class: Class,

    /// Is this a guide for a line number?
    linenum: bool,

    /// Should the guide have a casing?
    casing: bool,
}

impl GuideContour {
    pub fn from_arg(
        arg: eval::Expression,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?.0;
        let class = Class::from_symbols(&mut symbols);
        let linenum = symbols.take("linenum");
        let casing = symbols.take("casing");
        symbols.check_exhausted(err)?;
        Ok(GuideContour { class, linenum, casing })
    }
}

impl RenderContour for GuideContour {
    fn render(&self, canvas: &Canvas, path: &Path) {
        let color = if self.linenum {
            if !canvas.style().include_line_labels() {
                return
            }
            canvas.style().label_color(&self.class)
        }
        else {
            canvas.style().track_color(&self.class)
        };
        path.apply(canvas);
        if self.casing {
            canvas.set_line_width(
                1.8 * canvas.style().dimensions().guide_width
            );
            canvas.set_source_rgba(1., 1., 1., 0.7);
            canvas.stroke_preserve();
        }
        canvas.set_line_width(canvas.style().dimensions().guide_width);
        color.apply(canvas);
        canvas.stroke();
    }
}

