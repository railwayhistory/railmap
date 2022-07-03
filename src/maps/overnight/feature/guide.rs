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
use super::super::theme::Overnight;

//------------ GuideContour --------------------------------------------------

/// A contour drawing a guide.
pub struct GuideContour {
    /// The class of the guide,
    class: Class,

    trace: Trace,
}

impl GuideContour {
    pub fn from_arg(
        arg: eval::Expression<Overnight>,
        trace: Trace,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?.0;
        let class = Class::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(GuideContour { class, trace })
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        style.marker_color(&self.class).apply(canvas);
        canvas.set_line_width(style.dimensions().guide_width);
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
    }
}

