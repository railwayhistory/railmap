/// A feature drawing a symbol.
use std::fmt;
use std::sync::Arc;
use kurbo::Rect;
use crate::library;
use crate::canvas::Canvas;
use super::color::Color;
use super::path::Position;

//------------ Symbol --------------------------------------------------------

/// A feature drawing a symbol.
///
pub struct Symbol {
    /// The position the symbol is attached to.
    position: Position,

    /// The rendering rule for the symbol.
    rule: SymbolRule,
}

impl Symbol {
    pub fn new(position: Position, rule: SymbolRule) -> Self {
        Symbol { position, rule }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, canvas: &Canvas) {
        self.rule.0.render(canvas, &self.position)
    }
}


//------------ RenderSymbol --------------------------------------------------

pub trait RenderSymbol: Send + Sync + 'static {
    fn render(&self, canvas: &Canvas, position: &Position);
}

impl<F: Fn(&Canvas, &Position) + Send + Sync + 'static> RenderSymbol for F {
    fn render(&self, canvas: &Canvas, position: &Position) {
        (*self)(canvas, position)
    }
}


//------------ SymbolRule ----------------------------------------------------

#[derive(Clone)]
pub struct SymbolRule(Arc<dyn RenderSymbol>);

impl fmt::Debug for SymbolRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SymbolRule(...)")
    }
}

impl<R: RenderSymbol> From<R> for SymbolRule {
    fn from(rule: R) -> Self {
        SymbolRule(Arc::new(rule))
    }
}


//------------ Concrete Symbol Rules -----------------------------------------

pub fn monochrome(
    symbol: &str,
    color: Color,
    rotation: f64
) -> Option<SymbolRule> {
    let symbol = library::Symbol::lookup(symbol)?;
    Some(SymbolRule(Arc::new(move |canvas: &Canvas, position: &Position| {
        let (point, angle) = position.resolve(canvas);
        canvas.translate(point.x, point.y);
        canvas.rotate(angle + rotation.to_radians());
        color.apply(canvas);
        symbol.render(canvas);
        canvas.identity_matrix();
    })))
}

