//! Rendering of borders.

use kurbo::Rect;
use crate::import::Failed;
use crate::import::eval;
use crate::render::canvas::Canvas;
use crate::render::color::Color;
use crate::render::path::Trace;
use crate::theme::Style as _;
use super::super::style::Style;
use super::super::theme::Railwayhistory;

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
        arg: eval::Expression<Railwayhistory>,
        trace: Trace,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let category = Category::from_symbols(&mut symbols, err)?;
        let former = symbols.take("former");
        symbols.check_exhausted(err)?;
        Ok(BorderContour { category, former, trace })
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        if style.detail() <= 2 {
            self.render_low(style, canvas)
        }
        else {
            self.render_high(style, canvas)
        }
    }

    fn render_low(&self, style: &Style, canvas: &Canvas) {
        canvas.set_line_width(LOW_BORDER_WIDTH * style.canvas_bp());
        if self.former {
            LOW_FORMER_BORDER_COLOR.apply(canvas);
        }
        else {
            LOW_BORDER_COLOR.apply(canvas);
        }
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
    }

    fn render_high(&self, style: &Style, canvas: &Canvas) {
        self.category.apply_casing_width_high(style, canvas);
        if self.former {
            FORMER_CASING_COLOR.apply(canvas);
        }
        else {
            CASING_COLOR.apply(canvas);
        }
        self.trace.apply(canvas, style);
        canvas.stroke_preserve().unwrap();
        canvas.set_line_width(BORDER_WIDTH * style.canvas_bp());
        if self.former {
            FORMER_BORDER_COLOR.apply(canvas);
        }
        else {
            BORDER_COLOR.apply(canvas);
        }
        self.category.apply_dash_high(style, canvas);
        canvas.stroke().unwrap();
        canvas.set_dash(&[], 0.);
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
        symbols: &mut eval::SymbolSet,
        err: &mut eval::Error
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

    fn apply_casing_width_high(self, style: &Style, canvas: &Canvas) {
        match self {
            Category::National => {
                canvas.set_line_width(CASING_WIDTH * style.canvas_bp());
            }
            Category::State => {
                canvas.set_line_width(0.5 * CASING_WIDTH * style.canvas_bp());
            }
        }
    }

    fn apply_dash_high(self, style: &Style, canvas: &Canvas) {
        match self {
            Category::National => {
                canvas.set_dash(
                    &[
                        DASH_BASE * style.canvas_bp(),
                        0.4 * DASH_BASE * style.canvas_bp(),
                        0.1 * DASH_BASE * style.canvas_bp(),
                        0.4 * DASH_BASE * style.canvas_bp(),
                    ],
                    (DASH_BASE * 1.45 * DASH_BASE) * style.canvas_bp()
                );
            }
            Category::State => {
                canvas.set_dash(
                    &[
                        DASH_BASE * style.canvas_bp(),
                        0.6 * DASH_BASE * style.canvas_bp(),
                    ],
                    (DASH_BASE * 0.3 * DASH_BASE) * style.canvas_bp()
                );
            }
        }
    }
}

