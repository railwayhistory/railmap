//! Color constants.

use crate::features::color::Color;
use crate::import::eval::SymbolSet;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// The normal color for strokes.
    pub stroke: Color,

    /// The normal color for filling.
    pub fill: Color,
}

impl Palette {
    /// Creates the correct palette for a symbol set.
    pub fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("removed") {
            Palette::REMOVED
        }
        else if symbols.contains("closed") {
            Palette::CLOSED
        }
        else {
            Palette::OPEN
        }
    }
}

impl Palette {
    pub const OPEN: Palette = Palette {
        stroke: Color::BLACK,
        fill: Color::BLACK,
    };

    pub const CLOSED: Palette = Palette {
        stroke: Color::grey(0.4),
        fill: Color::grey(0.4),
    };

    pub const REMOVED: Palette = Palette {
        stroke: Color::grey(0.7),
        fill: Color::grey(0.7),
    };
}
