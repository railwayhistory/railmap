//! Rendering of guides.
//!
//! Guides are thin lines attaching a label to something.

use kurbo::Rect;
use crate::import::eval;
use crate::import::Failed;
use super::super::class::Class;
use crate::render::canvas::Canvas;
use crate::render::path::Trace;
use super::super::style::Style;
use super::super::theme::Railwayhistory;

//------------ GuideContour --------------------------------------------------

/// A contour drawing a guide.
pub struct GuideContour {
    /// The class of the guide,
    class: Class,

    /// Is this a guide for a line number?
    linenum: bool,

    /// Should the guide have a casing?
    casing: bool,

    trace: Trace,
}

impl GuideContour {
    pub fn from_arg(
        arg: eval::Expression<Railwayhistory>,
        trace: Trace,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?.0;
        let class = Class::from_symbols(&mut symbols);
        let linenum = symbols.take("linenum");
        let casing = symbols.take("casing");
        symbols.check_exhausted(err)?;
        Ok(GuideContour { class, linenum, casing, trace })
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        let color = if self.linenum {
            if !style.include_line_labels() {
                return
            }
            style.label_color(&self.class)
        }
        else {
            style.track_color(&self.class)
        };
        self.trace.apply(canvas, style);
        if self.casing {
            canvas.set_line_width(
                1.8 * style.dimensions().guide_width
            );
            canvas.set_source_rgba(1., 1., 1., 0.7);
            canvas.stroke_preserve().unwrap();
        }
        canvas.set_line_width(style.dimensions().guide_width);
        color.apply(canvas);
        canvas.stroke().unwrap();
    }
}

