//! Color constants.

use crate::features::color::Color;
use crate::import::eval::SymbolSet;

#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// The name of the style.
    pub name: &'static str,

    /// The palette for an feature with passenger service.
    pub pax: Palette,

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
            Some("tram") => &TRAM,
            Some("de.sbahn") => &DE_SBAHN,
            Some("nl.ns.ic") => &NL_NS_IC,
            Some("nl.ns.spr") => &NL_NS_SPR,
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
        else if symbols.contains("pax") {
            Some(&self.pax)
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

const PAX_STROKE: Color = Color::rgb(0.80, 0.36, 0.);

static DEFAULT: Style = Style {
    name: "default",
    pax: Palette {
        stroke: PAX_STROKE, 
        pale_stroke: PAX_STROKE,
        fill: PAX_STROKE,
        platform: PAX_STROKE,
        text: PAX_STROKE,
    },
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

const RED_COLOR: Color = Color::rgb(0.78, 0.0, 0.0);

static RED: Style = Style {
    name: "red",
    pax: Palette {
        stroke: RED_COLOR,
        pale_stroke: RED_COLOR,
        fill: RED_COLOR,
        platform: RED_COLOR,
        text: RED_COLOR,
    },
    open: Palette {
        stroke: RED_COLOR,
        pale_stroke: RED_COLOR,
        fill: RED_COLOR,
        platform: RED_COLOR,
        text: RED_COLOR,
    },
    closed: Palette {
        stroke: RED_COLOR,
        pale_stroke: RED_COLOR,
        fill: RED_COLOR,
        platform: RED_COLOR,
        text: RED_COLOR,
    },
    removed: Palette {
        stroke: RED_COLOR,
        pale_stroke: RED_COLOR,
        fill: RED_COLOR,
        platform: RED_COLOR,
        text: RED_COLOR,
    },
    gone: Palette {
        stroke: RED_COLOR,
        pale_stroke: RED_COLOR,
        fill: RED_COLOR,
        platform: RED_COLOR,
        text: RED_COLOR,
    },
};


const TRAM_OPEN: Color = Color::rgb(0., 0.329, 0.561);
const TRAM_CLOSED: Color = Color::rgb(0.145, 0.420, 0.612);
const TRAM_REMOVED: Color = Color::rgb(0.475, 0.623, 0.733);
const TRAM_GONE: Color = Color::rgb(0.690, 0.784, 0.851);

static TRAM: Style = Style {
    name: "tram",
    pax: Palette {
        stroke: TRAM_OPEN,
        pale_stroke: TRAM_OPEN,
        fill: TRAM_OPEN,
        platform: TRAM_OPEN,
        text: TRAM_OPEN,
    },
    open: Palette {
        stroke: TRAM_OPEN,
        pale_stroke: TRAM_OPEN,
        fill: TRAM_OPEN,
        platform: TRAM_OPEN,
        text: TRAM_OPEN,
    },
    closed: Palette {
        stroke: TRAM_CLOSED,
        pale_stroke: TRAM_CLOSED,
        fill: TRAM_CLOSED,
        platform: TRAM_CLOSED,
        text: TRAM_CLOSED,
    },
    removed: Palette {
        stroke: TRAM_REMOVED,
        pale_stroke: TRAM_REMOVED,
        fill: TRAM_REMOVED,
        platform: TRAM_REMOVED,
        text: TRAM_REMOVED,
    },
    gone: Palette {
        stroke: TRAM_GONE,
        pale_stroke: TRAM_GONE,
        fill: TRAM_GONE,
        platform: TRAM_GONE,
        text: TRAM_GONE,
    },
};


const DE_SBAHN_OPEN: Color = Color::rgb(0., 0.474, 0.255);
const DE_SBAHN_CLOSED: Color = Color::rgb(0.212, 0.608, 0.416);
const DE_SBAHN_REMOVED: Color = Color::rgb(0.353, 0.674, 0.525);
const DE_SBAHN_GONE: Color = Color::rgb(0.647, 0.827, 0.745);

static DE_SBAHN: Style = Style {
    name: "de.sbahn",
    pax: Palette {
        stroke: DE_SBAHN_OPEN,
        pale_stroke: DE_SBAHN_OPEN,
        fill: DE_SBAHN_OPEN,
        platform: DE_SBAHN_OPEN,
        text: DE_SBAHN_OPEN,
    },
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


const NL_NS_IC_ALL: Color = Color::rgb(0.706, 0.565, 0.071);
const NL_NS_SPR_ALL: Color = Color::rgb(0., 0.188, 0.510);

static NL_NS_IC: Style = Style {
    name: "ns.ic",
    pax: Palette {
        stroke: NL_NS_IC_ALL,
        pale_stroke: NL_NS_IC_ALL,
        fill: NL_NS_IC_ALL,
        platform: NL_NS_IC_ALL,
        text: NL_NS_IC_ALL,
    },
    open: Palette {
        stroke: NL_NS_IC_ALL,
        pale_stroke: NL_NS_IC_ALL,
        fill: NL_NS_IC_ALL,
        platform: NL_NS_IC_ALL,
        text: NL_NS_IC_ALL,
    },
    closed: Palette {
        stroke: NL_NS_IC_ALL,
        pale_stroke: NL_NS_IC_ALL,
        fill: NL_NS_IC_ALL,
        platform: NL_NS_IC_ALL,
        text: NL_NS_IC_ALL,
    },
    removed: Palette {
        stroke: NL_NS_IC_ALL,
        pale_stroke: NL_NS_IC_ALL,
        fill: NL_NS_IC_ALL,
        platform: NL_NS_IC_ALL,
        text: NL_NS_IC_ALL,
    },
    gone: Palette {
        stroke: NL_NS_IC_ALL,
        pale_stroke: NL_NS_IC_ALL,
        fill: NL_NS_IC_ALL,
        platform: NL_NS_IC_ALL,
        text: NL_NS_IC_ALL,
    },
};

static NL_NS_SPR: Style = Style {
    name: "ns.spr",
    pax: Palette {
        stroke: NL_NS_SPR_ALL,
        pale_stroke: NL_NS_SPR_ALL,
        fill: NL_NS_SPR_ALL,
        platform: NL_NS_SPR_ALL,
        text: NL_NS_SPR_ALL,
    },
    open: Palette {
        stroke: NL_NS_SPR_ALL,
        pale_stroke: NL_NS_SPR_ALL,
        fill: NL_NS_SPR_ALL,
        platform: NL_NS_SPR_ALL,
        text: NL_NS_SPR_ALL,
    },
    closed: Palette {
        stroke: NL_NS_SPR_ALL,
        pale_stroke: NL_NS_SPR_ALL,
        fill: NL_NS_SPR_ALL,
        platform: NL_NS_SPR_ALL,
        text: NL_NS_SPR_ALL,
    },
    removed: Palette {
        stroke: NL_NS_SPR_ALL,
        pale_stroke: NL_NS_SPR_ALL,
        fill: NL_NS_SPR_ALL,
        platform: NL_NS_SPR_ALL,
        text: NL_NS_SPR_ALL,
    },
    gone: Palette {
        stroke: NL_NS_SPR_ALL,
        pale_stroke: NL_NS_SPR_ALL,
        fill: NL_NS_SPR_ALL,
        platform: NL_NS_SPR_ALL,
        text: NL_NS_SPR_ALL,
    },
};

