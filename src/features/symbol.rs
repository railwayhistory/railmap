/// A feature drawing a symbol.
use std::fmt;
use std::sync::Arc;
use kurbo::Rect;
use crate::canvas::Canvas;
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

