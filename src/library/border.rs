//! Rendering of borders.

use crate::canvas::Canvas;
use crate::import::eval::SymbolSet;
use crate::features::color::Color;
use crate::features::contour::RenderContour;
use crate::features::path::Path;

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
}

impl BorderContour {
    pub fn new(symbols: SymbolSet) -> Self {
        BorderContour {
            category: Category::from_symbols(&symbols),
            former: symbols.contains("former"),
        }
    }
}

impl RenderContour for BorderContour {
    fn render(&self, canvas: &Canvas, path: &Path) {
        if canvas.detail() <= 2 {
            self.render_low(canvas, path)
        }
        else {
            self.render_high(canvas, path)
        }
    }
}

impl BorderContour {
    fn render_low(&self, canvas: &Canvas, path: &Path) {
        canvas.set_line_width(LOW_BORDER_WIDTH * canvas.canvas_bp());
        if self.former {
            LOW_FORMER_BORDER_COLOR.apply(canvas);
        }
        else {
            LOW_BORDER_COLOR.apply(canvas);
        }
        path.apply(canvas);
        canvas.stroke();
    }

    fn render_high(&self, canvas: &Canvas, path: &Path) {
        self.category.apply_casing_width_high(canvas);
        if self.former {
            FORMER_CASING_COLOR.apply(canvas);
        }
        else {
            CASING_COLOR.apply(canvas);
        }
        path.apply(canvas);
        canvas.stroke_preserve();
        canvas.set_line_width(BORDER_WIDTH * canvas.canvas_bp());
        if self.former {
            FORMER_BORDER_COLOR.apply(canvas);
        }
        else {
            BORDER_COLOR.apply(canvas);
        }
        self.category.apply_dash_high(canvas);
        canvas.stroke();
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
    fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("national") { Category::National }
        else if symbols.contains("state") { Category::State }
        else { Category::National }
    }

    fn apply_casing_width_high(self, canvas: &Canvas) {
        match self {
            Category::National => {
                canvas.set_line_width(CASING_WIDTH * canvas.canvas_bp());
            }
            Category::State => {
                canvas.set_line_width(0.5 * CASING_WIDTH * canvas.canvas_bp());
            }
        }
    }

    fn apply_dash_high(self, canvas: &Canvas) {
        match self {
            Category::National => {
                canvas.set_dash(
                    &[
                        DASH_BASE * canvas.canvas_bp(),
                        0.4 * DASH_BASE * canvas.canvas_bp(),
                        0.1 * DASH_BASE * canvas.canvas_bp(),
                        0.4 * DASH_BASE * canvas.canvas_bp(),
                    ],
                    (DASH_BASE * 1.45 * DASH_BASE) * canvas.canvas_bp()
                );
            }
            Category::State => {
                canvas.set_dash(
                    &[
                        DASH_BASE * canvas.canvas_bp(),
                        0.6 * DASH_BASE * canvas.canvas_bp(),
                    ],
                    (DASH_BASE * 0.3 * DASH_BASE) * canvas.canvas_bp()
                );
            }
        }
    }
}

