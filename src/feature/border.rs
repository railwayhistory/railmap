//! Rendering of borders.

use femtomap::world;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Trace;
use femtomap::render::{Canvas, DashPattern, Color, Outline, LineWidth};
use crate::import::eval::Expression;
use crate::style::Style;
use super::{AnyShape, Feature};

//------------ Configuration -------------------------------------------------

/// The color used for drawing the border in low detail.
///
/// This is `#e3c7ec`.
const LOW_BORDER_COLOR: Color = Color::rgb(0.89, 0.78, 0.925);

/// The color used for the border line in high detail.
///
/// This is `#b873cf`.
const BORDER_COLOR: Color = Color::rgb(0.722, 0.45, 0.822);

/// The color used for the border casing in high detail.
///
/// This is `#b873cf`.
const CASING_COLOR: Color = Color::rgba(0.722, 0.45, 0.822, 0.2);

/// The color used for drawing the border in low detail.
///
/// This is `#e3c7ec`.
const LOW_FORMER_BORDER_COLOR: Color = Color::rgb(0.96, 0.875, 0.992);

/// The color used for the border line in high detail.
///
/// This is `#b873cf`.
const FORMER_BORDER_COLOR: Color = Color::rgb(0.89, 0.78, 0.925);

/// The color used for the border casing in high detail.
///
/// This is `#b873cf`.
const FORMER_CASING_COLOR: Color = Color::rgba(0.89, 0.78, 0.925, 0.2);

/// The width in pt of the border line in low detail.
const LOW_BORDER_WIDTH: f64 = 1.0;

/// The width in pt of the border line in high detail.
const BORDER_WIDTH: f64 = 1.0;

/// The width in pt of border casing in high detail.
const CASING_WIDTH: f64 = 4.5;

/// The dash pattern base in pt.
const DASH_BASE: f64 = 8.0;



//------------ BorderContour -------------------------------------------------

/// The rendering rule for a border contour.
pub struct BorderContour {
    /// The category of the border.
    category: Category,

    /// Whether this is a former border.
    former: bool,

    /// The trace of the border.
    trace: Trace,
}

impl BorderContour {
    pub fn from_arg(
        arg: Expression,
        trace: Trace,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.eval(err)?;
        let category = Category::from_symbols(&mut symbols, err)?;
        let former = symbols.take("former");
        symbols.check_exhausted(err)?;
        Ok(BorderContour { category, former, trace })
    }
}

impl Feature for BorderContour {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        let outline = self.trace.outline(style);
        if style.detail() < 3 {
            AnyShape::single_stage(move |style: &Style, canvas: &mut Canvas| {
                self.render_low(&outline, style, canvas)
            })
        }
        else {
            AnyShape::single_stage(move |style: &Style, canvas: &mut Canvas| {
                self.render_high(&outline, style, canvas)
            })
        }
        
    }
}

impl BorderContour {
    fn render_low(
        &self, outline: &Outline, style: &Style, canvas: &mut Canvas
    ) {
        let mut canvas = canvas.sketch();
        canvas.apply(outline);
        canvas.apply(
            if self.former {
                LOW_FORMER_BORDER_COLOR
            }
            else {
                LOW_BORDER_COLOR
            }
        );
        canvas.apply(LineWidth(LOW_BORDER_WIDTH * style.canvas_bp()));
        canvas.stroke();
    }

    fn render_high(
        &self, outline: &Outline, style: &Style, canvas: &mut Canvas
    ) {
        let mut sketch = canvas.sketch();
        sketch.apply(outline);
        sketch.apply(
            if self.former {
                FORMER_CASING_COLOR
            }
            else {
                CASING_COLOR
            }
        );
        sketch.apply(LineWidth(self.category.casing_width_high(style)));
        sketch.stroke();
        sketch.apply(
            if self.former {
                FORMER_BORDER_COLOR
            }
            else {
                BORDER_COLOR
            }
        );
        sketch.apply(LineWidth(BORDER_WIDTH * style.canvas_bp()));
        sketch.apply(self.category.dash_high(style));
        sketch.stroke();
    }
}


//------------ Category ------------------------------------------------------

/// The category of a border.
#[derive(Clone, Copy, Debug)]
enum Category {
    National,
    State,
}

impl Category {
    fn from_symbols(
        symbols: &mut SymbolSet, err: &mut EvalErrors
    ) -> Result<Self, Failed> {
        if symbols.take("national") {
            Ok(Category::National)
        }
        else if symbols.take("state") {
            Ok(Category::State)
        }
        else {
            err.add(symbols.pos(), "missing border category");
            Err(Failed)
        }
    }

    fn casing_width_high(self, style: &Style) -> f64 {
        match self {
            Category::National => {
                CASING_WIDTH * style.canvas_bp()
            }
            Category::State => {
                0.5 * CASING_WIDTH * style.canvas_bp()
            }
        }
    }

    fn dash_high(self, style: &Style) -> DashPattern<4> {
        match self {
            Category::National => {
                DashPattern::new(
                    [
                        DASH_BASE * style.canvas_bp(),
                        0.4 * DASH_BASE * style.canvas_bp(),
                        0.1 * DASH_BASE * style.canvas_bp(),
                        0.4 * DASH_BASE * style.canvas_bp(),
                    ],
                    (DASH_BASE * 1.45 * DASH_BASE) * style.canvas_bp()
                )
            }
            Category::State => {
                DashPattern::new(
                    [
                        DASH_BASE * style.canvas_bp(),
                        0.6 * DASH_BASE * style.canvas_bp(),
                        DASH_BASE * style.canvas_bp(),
                        0.6 * DASH_BASE * style.canvas_bp(),
                    ],
                    (DASH_BASE * 0.3 * DASH_BASE) * style.canvas_bp()
                )
            }
        }
    }
}

