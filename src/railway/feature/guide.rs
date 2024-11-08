//! Rendering of guides.
//!
//! Guides are thin lines attaching a label to something.

use femtomap::world;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Trace;
use femtomap::render::{Canvas, LineWidth, Outline};
use crate::railway::class::Railway;
use crate::railway::import::eval::Scope;
use crate::railway::style::Style;
use super::{AnyShape, Category, Group, Feature, Shape, Stage, StageSet};

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
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let railway = Railway::from_symbols(&mut class, scope);
        let casing = class.take("casing");
        class.check_exhausted(err)?;
        Ok(GuideContour { class: railway, casing, trace })
    }
}

impl Feature for GuideContour {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn group(&self) -> Group {
        Group::with_category(Category::Label)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        GuideShape {
            contour: self, trace: self.trace.outline(style)
        }.into()
    }
}


//------------ GuideShape ----------------------------------------------------

struct GuideShape<'a> {
    contour: &'a GuideContour,
    trace: Outline,
}

impl<'a> Shape<'a> for GuideShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if self.contour.casing {
                    let mut sketch = canvas.sketch();
                    sketch.apply(&self.trace);
                    sketch.apply(LineWidth(
                        3. * style.measures().guide_width()
                    ));
                    sketch.apply(style.casing_color());
                    sketch.stroke();
                }
            }
            Stage::Base => {
                let mut sketch = canvas.sketch();
                sketch.apply(&self.trace);
                sketch.apply(LineWidth(style.measures().guide_width()));
                sketch.apply(style.label_color(&self.contour.class));
                sketch.stroke();
            }
            _ => { }
        }
    }

    fn stages(&self) -> StageSet {
        let res = StageSet::from(Stage::Base);
        if self.contour.casing {
            res.add(Stage::Casing)
        }
        else {
            res
        }
    }
}

