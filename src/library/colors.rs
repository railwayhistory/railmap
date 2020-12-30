//! Color constants.

use crate::features::color::Color;
use crate::import::eval::SymbolSet;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// The normal color for strokes.
    pub stroke: Color,

    /// The normal color for pale strokes.
    ///
    /// This color is used for cases where greyed out lines need to be
    /// a bit paler.
    pub pale_stroke: Color,

    /// The normal color for filling.
    pub fill: Color,

    /// The color for filling platforms.
    pub platform: Color,

    /// The color for text.
    pub text: Color,
}

impl Palette {
    /// Creates the correct palette for a symbol set.
    pub fn from_symbols(symbols: &SymbolSet) -> Self {
        Self::opt_from_symbols(symbols).unwrap_or(Palette::OPEN)
    }

    pub fn opt_from_symbols(symbols: &SymbolSet) -> Option<Self> {
        if symbols.contains("red") {
            Some(Palette::RED)
        }
        else if symbols.contains("gone") || symbols.contains("former") {
            Some(Palette::GONE)
        }
        else if symbols.contains("removed") {
            Some(Palette::REMOVED)
        }
        else if symbols.contains("closed") {
            Some(Palette::CLOSED)
        }
        else {
            None
        }
    }
}

impl Palette {
    pub const OPEN: Palette = Palette {
        stroke: Color::BLACK,
        pale_stroke: Color::BLACK,
        fill: Color::BLACK,
        platform: Color::grey(0.2),
        text: Color::BLACK,
    };

    pub const CLOSED: Palette = Palette {
        stroke: Color::grey(0.4),
        pale_stroke: Color::grey(0.7),
        fill: Color::grey(0.5),
        platform: Color::grey(0.6),
        text: Color::grey(0.2),
    };

    pub const REMOVED: Palette = Palette {
        stroke: Color::grey(0.6),
        pale_stroke: Color::grey(0.7),
        fill: Color::grey(0.7),
        platform: Color::grey(0.8),
        text: Color::grey(0.4),
    };

    pub const GONE: Palette = Palette {
        stroke: Color::grey(0.8),
        pale_stroke: Color::grey(0.9),
        fill: Color::grey(0.9),
        platform: Color::grey(0.9),
        text: Color::grey(0.6),
    };

    pub const RED: Palette = Palette {
        stroke: Color::rgb(1.0, 0.0, 0.0),
        pale_stroke: Color::rgb(1.0, 0.0, 0.0),
        fill: Color::rgb(1.0, 0.0, 0.0),
        platform: Color::rgb(1.0, 0.0, 0.0),
        text: Color::rgb(1.0, 0.0, 0.0),
    };
}
