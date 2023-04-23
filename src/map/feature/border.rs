//! Rendering of borders.

use femtomap::path::Trace;
use femtomap::render::canvas;
use femtomap::render::canvas::DashPattern;
use femtomap::render::pattern::Color;
use kurbo::{BezPath, Rect};
use crate::import::Failed;
use crate::import::eval;
use crate::theme::Style as _;
use super::super::style::Style;
use super::super::theme::Railwayhistory;
use super::Shape;

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


    pub fn shape(
        &self, style: &Style, _canvas: &canvas::Canvas
    ) -> Box<dyn Shape + '_> {
        let outline = self.trace.outline(style);
        if style.detail() <= 2 {
            Box::new(move |style: &Style, canvas: canvas::Group| {
                self.render_low(&outline, style, canvas)
            })
        }
        else {
            Box::new(move |style: &Style, canvas: canvas::Group| {
                self.render_high(&outline, style, canvas)
            })
        }
        
    }

    fn render_low(
        &self, outline: &BezPath, style: &Style, mut canvas: canvas::Group
    ) {
        canvas.apply_line_width(LOW_BORDER_WIDTH * style.canvas_bp());
        if self.former {
            canvas.apply(LOW_FORMER_BORDER_COLOR);
        }
        else {
            canvas.apply(LOW_BORDER_COLOR);
        }
        canvas.apply(outline);
        canvas.stroke()
    }

    fn render_high(
        &self, outline: &BezPath, style: &Style, mut canvas: canvas::Group
    ) {
        self.category.apply_casing_width_high(style, &mut canvas);
        if self.former {
            canvas.apply(FORMER_CASING_COLOR);
        }
        else {
            canvas.apply(CASING_COLOR);
        }
        canvas.apply(outline);
        canvas.stroke();
        canvas.apply_line_width(BORDER_WIDTH * style.canvas_bp());
        if self.former {
            canvas.apply(FORMER_BORDER_COLOR)
        }
        else {
            canvas.apply(BORDER_COLOR);
        }
        self.category.apply_dash_high(style, &mut canvas);
        canvas.stroke()
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

    fn apply_casing_width_high(
        self, style: &Style, canvas: &mut canvas::Group
    ) {
        match self {
            Category::National => {
                canvas.apply_line_width(CASING_WIDTH * style.canvas_bp());
            }
            Category::State => {
                canvas.apply_line_width(0.5 * CASING_WIDTH * style.canvas_bp());
            }
        }
    }

    fn apply_dash_high(
        self, style: &Style, canvas: &mut canvas::Group
    ) {
        match self {
            Category::National => {
                canvas.apply(DashPattern::new(
                    [
                        DASH_BASE * style.canvas_bp(),
                        0.4 * DASH_BASE * style.canvas_bp(),
                        0.1 * DASH_BASE * style.canvas_bp(),
                        0.4 * DASH_BASE * style.canvas_bp(),
                    ],
                    (DASH_BASE * 1.45 * DASH_BASE) * style.canvas_bp()
                ));
            }
            Category::State => {
                canvas.apply(DashPattern::new(
                    [
                        DASH_BASE * style.canvas_bp(),
                        0.6 * DASH_BASE * style.canvas_bp(),
                    ],
                    (DASH_BASE * 0.3 * DASH_BASE) * style.canvas_bp()
                ));
            }
        }
    }
}

