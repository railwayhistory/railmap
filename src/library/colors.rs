//! Color constants.

use crate::features::color::Color;
use crate::import::eval::SymbolSet;

#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// The palette for an open feature.
    pub open: Palette,

    /// The palette for a closed feature.
    pub closed: Palette,

    /// The palette for a removed feature.
    pub removed: Palette,

    /// The palette for a gone feature.
    pub gone: Palette,
}

impl Style {
    /// Returns the style for a given name.
    pub fn from_name(name: Option<&str>) -> &'static Self {
        match name {
            Some("red") => &RED,
            Some("de.sbahn") => &DE_SBAHN,
            _ => &DEFAULT,
        }
    }

    /// Returns the correct palette for a symbol set.
    pub fn palette(&self, symbols: &SymbolSet) -> &Palette {
        self.opt_palette(symbols).unwrap_or(&self.open)
    }

    pub fn opt_palette(&self, symbols: &SymbolSet) -> Option<&Palette> {
        if symbols.contains("gone") || symbols.contains("former") {
            Some(&self.gone)
        }
        else if symbols.contains("removed") {
            Some(&self.removed)
        }
        else if symbols.contains("closed") {
            Some(&self.closed)
        }
        else {
            None
        }
    }
}


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

/*
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
*/

static DEFAULT: Style = Style {
    open: Palette {
        stroke: Color::BLACK,
        pale_stroke: Color::BLACK,
        fill: Color::BLACK,
        platform: Color::grey(0.2),
        text: Color::BLACK,
    },
    closed: Palette {
        stroke: Color::grey(0.4),
        pale_stroke: Color::grey(0.7),
        fill: Color::grey(0.5),
        platform: Color::grey(0.6),
        text: Color::grey(0.2),
    },
    removed: Palette {
        stroke: Color::grey(0.6),
        pale_stroke: Color::grey(0.7),
        fill: Color::grey(0.7),
        platform: Color::grey(0.8),
        text: Color::grey(0.4),
    },
    gone: Palette {
        stroke: Color::grey(0.8),
        pale_stroke: Color::grey(0.9),
        fill: Color::grey(0.9),
        platform: Color::grey(0.9),
        text: Color::grey(0.6),
    }
};

static RED: Style = Style {
    open: Palette {
        stroke: Color::rgb(1.0, 0.0, 0.0),
        pale_stroke: Color::rgb(1.0, 0.0, 0.0),
        fill: Color::rgb(1.0, 0.0, 0.0),
        platform: Color::rgb(1.0, 0.0, 0.0),
        text: Color::rgb(1.0, 0.0, 0.0),
    },
    closed: Palette {
        stroke: Color::rgb(1.0, 0.0, 0.0),
        pale_stroke: Color::rgb(1.0, 0.0, 0.0),
        fill: Color::rgb(1.0, 0.0, 0.0),
        platform: Color::rgb(1.0, 0.0, 0.0),
        text: Color::rgb(1.0, 0.0, 0.0),
    },
    removed: Palette {
        stroke: Color::rgb(1.0, 0.0, 0.0),
        pale_stroke: Color::rgb(1.0, 0.0, 0.0),
        fill: Color::rgb(1.0, 0.0, 0.0),
        platform: Color::rgb(1.0, 0.0, 0.0),
        text: Color::rgb(1.0, 0.0, 0.0),
    },
    gone: Palette {
        stroke: Color::rgb(1.0, 0.0, 0.0),
        pale_stroke: Color::rgb(1.0, 0.0, 0.0),
        fill: Color::rgb(1.0, 0.0, 0.0),
        platform: Color::rgb(1.0, 0.0, 0.0),
        text: Color::rgb(1.0, 0.0, 0.0),
    },
};


const DE_SBAHN_OPEN: Color = Color::rgb(0., 0.474, 0.255);
const DE_SBAHN_CLOSED: Color = Color::rgb(0.212, 0.608, 0.416);
const DE_SBAHN_REMOVED: Color = Color::rgb(0.353, 0.674, 0.525);
const DE_SBAHN_GONE: Color = Color::rgb(0.647, 0.827, 0.745);

static DE_SBAHN: Style = Style {
    open: Palette {
        stroke: DE_SBAHN_OPEN,
        pale_stroke: DE_SBAHN_OPEN,
        fill: DE_SBAHN_OPEN,
        platform: DE_SBAHN_OPEN,
        text: DE_SBAHN_OPEN,
    },
    closed: Palette {
        stroke: DE_SBAHN_CLOSED,
        pale_stroke: DE_SBAHN_CLOSED,
        fill: DE_SBAHN_CLOSED,
        platform: DE_SBAHN_CLOSED,
        text: DE_SBAHN_CLOSED,
    },
    removed: Palette {
        stroke: DE_SBAHN_REMOVED,
        pale_stroke: DE_SBAHN_REMOVED,
        fill: DE_SBAHN_REMOVED,
        platform: DE_SBAHN_REMOVED,
        text: DE_SBAHN_REMOVED,
    },
    gone: Palette {
        stroke: DE_SBAHN_GONE,
        pale_stroke: DE_SBAHN_GONE,
        fill: DE_SBAHN_GONE,
        platform: DE_SBAHN_GONE,
        text: DE_SBAHN_GONE,
    },
};


/*
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
*/

