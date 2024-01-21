//! Rendering of guides.
//!
//! Guides are thin lines attaching a label to something.

use femtomap::world;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Trace;
use femtomap::render::{Canvas, Color, LineWidth};
use crate::class::Railway;
use crate::style::Style;
use super::{AnyShape, Feature};

//------------ GuideContour --------------------------------------------------

/// A contour drawing a guide.
pub struct GuideContour {
    /// The class of the guide,
    class: Railway,

    /// Should the guide have a casing?
    casing: bool,

    trace: Trace,
}

impl GuideContour {
    pub fn new(
        class: Railway, casing: bool, trace: Trace,
    ) -> Self {
        GuideContour { class, casing, trace }
    }

    pub fn from_symbols(
        mut class: SymbolSet,
        trace: Trace,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let railway = Railway::from_symbols(&mut class);
        let casing = class.take("casing");
        class.check_exhausted(err)?;
        Ok(GuideContour { class: railway, casing, trace })
    }
}

impl Feature for GuideContour {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn shape(
        &self, _style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        AnyShape::single_stage(|style: &Style, canvas: &mut Canvas| {
            let mut sketch = canvas.sketch();
            sketch.apply(self.trace.iter_outline(style));
            if self.casing {
                sketch.apply(LineWidth(
                    1.8 * style.units().guide_width
                ));
                sketch.apply(Color::rgba(1., 1., 1., 0.7));
                sketch.stroke();
            }
            sketch.apply(LineWidth(style.units().guide_width));
            sketch.apply(style.label_color(&self.class));
            sketch.stroke();
        })
    }
}

