//! Rendering of guides.
//!
//! Guides are thin lines attaching a label to something.

use femtomap::path::Trace;
use femtomap::render::{Canvas, Color, LineWidth};
use kurbo::Rect;
use crate::import::eval;
use crate::import::Failed;
use super::super::class::Class;
use super::super::style::Style;
use super::super::theme::Railwayhistory;
use super::Shape;

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
    pub fn new(
        class: Class, linenum: bool, casing: bool, trace: Trace,
    ) -> Self {
        GuideContour { class, linenum, casing, trace }
    }

    pub fn from_arg(
        arg: eval::Expression<Railwayhistory>,
        trace: Trace,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let class = Class::from_symbols(&mut symbols);
        let linenum = symbols.take("linenum");
        let casing = symbols.take("casing");
        symbols.check_exhausted(err)?;
        Ok(GuideContour { class, linenum, casing, trace })
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn shape(
        &self, style: &Style, canvas: &Canvas
    ) -> Box<dyn Shape + '_> {
        Box::new(|style: &Style, canvas: &mut Canvas| {
            let color = if self.linenum {
                if !style.include_line_labels() {
                    return
                }
                style.label_color(&self.class)
            }
            else {
                style.track_color(&self.class)
            };

            let mut sketch = canvas.sketch();
            sketch.apply(self.trace.iter_outline(style));
            if self.casing {
                sketch.apply(LineWidth(
                    1.8 * style.dimensions().guide_width
                ));
                sketch.apply(Color::rgba(1., 1., 1., 0.7));
                sketch.stroke();
            }
            sketch.apply(LineWidth(style.dimensions().guide_width));
            sketch.apply(color);
            sketch.stroke();
        })
    }
}
