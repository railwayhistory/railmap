//! The style of the map to be rendered.

use std::collections::HashMap;
use std::str::FromStr;
use std::ops::MulAssign;
use std::sync::Arc;
use crate::render::color::Color;
use crate::render::path::MapDistance;
use crate::theme;
use super::class::{
    Category, Class, ElectricStatus, ElectricSystem, Pax, Status,
    VoltageGroup
};
use super::units;


//------------ Configurable Constants ----------------------------------------

/// Size correction for feature bounds.
///
/// This value will be multiplied with detail level, then length and height of
/// the bounding box and then added on each side.
///
/// Increase if features are missing.
const BOUNDS_CORRECTION: f64 = 0.3;


//------------ StyleId -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StyleId {
    /// Overview map with the given palette.
    Overview(Palette),

    /// Detail map with the given palette.
    Detail(Palette),
}

impl StyleId {
    pub fn detail(self, zoom: u8) -> u8 {
        match self {
            StyleId::Overview(_) => OVERVIEW_DETAILS[zoom as usize],
            StyleId::Detail(_) => DETAIL_DETAILS[zoom as usize],
        }
    }

    pub fn mag(self, zoom: u8) -> f64 {
        match self {
            StyleId::Overview(_) => OVERVIEW_MAG[zoom as usize],
            StyleId::Detail(_) => DETAIL_MAG[zoom as usize],
        }
    }
}

impl FromStr for StyleId {
    type Err = InvalidStyle;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sp" => Ok(StyleId::Overview(Palette::Pax)),
            "se" => Ok(StyleId::Overview(Palette::El)),
            "sx" => Ok(StyleId::Overview(Palette::Proof)),
            "lp" => Ok(StyleId::Detail(Palette::Pax)),
            "le" => Ok(StyleId::Detail(Palette::El)),
            "lx" => Ok(StyleId::Detail(Palette::Proof)),
            _ => Err(InvalidStyle)
        }
    }
}


//------------ Style ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Style {
    id: StyleId,
    detail: u8,
    mag: f64,
    dimensions: Dimensions,
    colors: Arc<ColorSet>,
}

impl theme::Style for Style {
    type StyleId = StyleId;

    fn bounds_correction(&self) -> f64 {
        BOUNDS_CORRECTION * (self.detail as f64)
    }

    fn mag(&self) -> f64 {
        self.mag
    }

    fn detail(&self) -> u8 {
        self.detail
    }

    fn scale(&mut self, canvas_bp: f64) {
        self.dimensions *= canvas_bp;
    }

    fn resolve_distance(&self, distance: MapDistance) -> f64 {
        distance.value()
        * units::MAP_DISTANCES[distance.unit()].1[self.detail as usize]
    }
}

impl Style {
    pub fn new(id: StyleId, zoom: u8, colors: Arc<ColorSet>) -> Self {
        let detail = id.detail(zoom);
        let dimensions = if detail < 3 {
            Dimensions::D0
        }
        else if detail < 4 {
            Dimensions::D3
        }
        else {
            Dimensions::D4
        };
        Style {
            id,
            detail,
            mag: id.mag(zoom),
            dimensions,
            colors,
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    fn palette(&self) -> Palette {
        match self.id {
            StyleId::Overview(pal) => pal,
            StyleId::Detail(pal) => pal
        }
    }

    pub fn include_line_labels(&self) -> bool {
        self.palette().show_linenum()
    }
}


//--- Colors
//
impl Style {
    /// Returns the color for a piece of track.
    pub fn track_color(&self, class: &Class) -> Color {
        self.colors.color(self.palette(), class)
    }

    /// Returns the color for cat markings if they should be drawn.
    pub fn cat_color(&self, class: &Class) -> Option<Color> {
        self.colors.cat_color(self.palette(), class)
    }

    /// Returns the color for third rail markings if they should be drawn.
    pub fn rail_color(&self, class: &Class) -> Option<Color> {
        self.colors.rail_color(self.palette(), class)
    }

    /// Returns the color for a station label.
    pub fn label_color(&self, class: &Class) -> Color {
        self.colors.color(self.palette(), class)
    }

    /// Returns the primary color for a marker.
    pub fn primary_marker_color(&self, class: &Class) -> Color {
        self.colors.color(self.palette(), class)
    }
}


//============ Detail Levels and Magnifications ==============================

/// The mapping of zoom levels to details.
const DETAIL_DETAILS: &[u8] = &[
    0, 0, 0, 0, 0,
    0, 0, 1, 1, 2,
    3, 3, 4, 4, 5,
    5, 5, 5, 5, 5,
];

/// The mapping of zoom levels to magnification.
const DETAIL_MAG: &[f64] = &[
    1., 1., 1., 1., 1., 
    1., 1., 1., 1.3, 1., 
    1., 1.3, 1., 1.5, 1.2,
    1.7, 2., 3., 1., 1.,
];

/// The mapping of zoom levels to details.
const OVERVIEW_DETAILS: &[u8] = &[
    0, 0, 0, 0, 0,
    0, 0, 1, 1, 2,
    2, 2, 2, 2, 2,
    2, 2, 2, 2, 2,
];

/// The mapping of zoom levels to magnification.
const OVERVIEW_MAG: &[f64] = &[
    1., 1., 1., 1., 1., 
    1., 1., 1., 1.3, 1., 
    1.3, 1.3, 1.3, 1.3, 1.3,
    1.3, 1.3, 1.3, 1.3, 1.3,
];


//============ Dimensions ====================================================

/// Adjustable values for the dimensions of things.
///
/// Values here are given in _bp_ normally or in canvas lengths if the
/// dimensions have been gained from style associated with a canvas.
#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
    /// The width of a line tracks.
    pub line_width: f64,

    /// The width of a station, private, or tram track.
    pub other_width: f64,

    pub guide_width: f64,

    /// The length of a segment of markings.
    pub seg: f64,

    /// The distance between two parallel tracks.
    pub dt: f64,

    /// The height of category markings.
    pub mark: f64,

    /// The height of tight single-track markings.
    pub tight_mark: f64,

    /// The width of a station symbol.
    pub sw: f64,

    /// The height of a station symbol.
    pub sh: f64,

    /// The width of a reduced size station symbol.
    pub ksw: f64,

    /// The height of a reduced size station symbol.
    pub ksh: f64,

    /// The radius of curves on station symbols.
    pub ds: f64,

    /// The line width of station symbols.
    pub sp: f64,

    /// The line width of border symbols.
    pub bp: f64,
}

impl Dimensions {
    const D0: Self = Self {
        line_width: 0.8,
        other_width: 0.5,
        guide_width: 0.3,
        seg: 5.0 * units::DT,
        dt: units::DT,
        mark: 0.6 *  units::DT,
        tight_mark: 0.55 * units::DT,
        sw: units::SSW,
        sh: 0.96 * units::SSW,
        ksw: units::SSW,
        ksh: 0.96 * units::SSW,
        ds: 0.05 * units::SSW,
        sp: 0.4,
        bp: 0.4,
    };

    const D3: Self = Self {
        line_width: 1.,
        dt: 0.7 * units::DT,
        .. Self::D0
    };

    const D4: Self = Self {
        line_width: 1.,
        mark: 0.8 * super::units::DT,
        sw: units::SW,
        sh: units::SH,
        ksw: 0.8 * units::SW,
        ksh: 0.8 * units::SH,
        ds: 0.05 * units::SH,
        sp: 0.8,
        bp: 0.6,
        .. Self::D0
    };

    pub fn mark(&self, tight: bool) -> f64 {
        if tight {
            self.tight_mark
        }
        else {
            self.mark
        }
    }
}

impl MulAssign<f64> for  Dimensions {
    fn mul_assign(&mut self, rhs: f64) {
        self.line_width *= rhs;
        self.other_width *= rhs;
        self.guide_width *= rhs;
        self.seg *= rhs;
        self.dt *= rhs;
        self.mark *= rhs;
        self.tight_mark *= rhs;
        self.sw *= rhs;
        self.sh *= rhs;
        self.ksw *= rhs;
        self.ksh *= rhs;
        self.ds *= rhs;
        self.sp *= rhs;
        self.bp *= rhs;
    }
}


//============ Colors ========================================================

//------------ Palette -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Palette {
    /// Highlight passenger service.
    Pax,

    /// Highlight electrification type.
    El,

    /// Optimized for proof-reading.
    Proof,
}


impl Palette {
    /// Returns whether to draw line numbers.
    pub fn show_linenum(self) -> bool {
        match self {
            Palette::Pax => false,
            _ => true
        }
    }
}


//------------ ColorSet ------------------------------------------------------

macro_rules! color_set {
    ( $( $name:ident, )* ) => {
        #[derive(Clone, Debug)]
        pub struct ColorSet {
            $(
                pub $name: Color,
            )*
        }

        impl ColorSet {
            pub fn update(&mut self, src: &HashMap<String, Color>) {
                $(
                    if let Some(color) = src.get(stringify!($name)) {
                        self.$name = *color;
                    }
                )*
            }
        }
    }
}

color_set! {
    el_ole_ac_low_pax,
    el_ole_ac_high_pax,
    el_ole_dc_low_pax,
    el_ole_dc_high_pax,
    el_rail_low_pax,
    el_rail_high_pax,
    el_none_pax,
    el_ole_ac_low,
    el_ole_ac_high,
    el_ole_dc_low,
    el_ole_dc_high,
    el_rail_low,
    el_rail_high,
    el_none,

    pax_full,
    pax_ltd,
    pax_none,
    pax_closed,

    closed,
    removed,
    gone,

    tram,
    tram_closed,
    tram_removed,
    tram_gone,

    toxic,
}

impl ColorSet {
    fn color(&self, palette: Palette, class: &Class) -> Color {
        match palette {
            Palette::Pax => self.pax_color(class),
            Palette::El => self.el_color(class),
            Palette::Proof => self.proof_color(class),
        }
    }

    fn cat_color(&self, palette: Palette, class: &Class) -> Option<Color> {
        match palette {
            Palette::Pax => self.pax_cat_color(class),
            Palette::El => self.el_cat_color(class),
            Palette::Proof => self.proof_cat_color(class),
        }
    }

    fn rail_color(&self, palette: Palette, class: &Class) -> Option<Color> {
        match palette {
            Palette::Pax => self.pax_rail_color(class),
            Palette::El => self.el_rail_color(class),
            Palette::Proof => self.proof_rail_color(class),
        }
    }

    fn el_color(&self, class: &Class) -> Color {
        use VoltageGroup::*;
        use ElectricSystem::*;

        if let Some(color) = self.common_color(class) {
            color
        }
        else if let Some(cat) = class.active_cat() {
            match (cat.system, cat.voltage_group(), class.pax().is_full()) {
                (Some(Ac), High, true) => self.el_ole_ac_high_pax,
                (Some(Ac), High, false) => self.el_ole_ac_high,
                (Some(Ac), Low, true) => self.el_ole_ac_low_pax,
                (Some(Ac), Low, false) => self.el_ole_ac_low,
                (Some(Dc), High, true) => self.el_ole_dc_high_pax,
                (Some(Dc), High, false) => self.el_ole_dc_high,
                (Some(Dc), Low, true) => self.el_ole_dc_low_pax,
                (Some(Dc), Low, false) => self.el_ole_dc_low,
                _ => self.toxic,
            }
        }
        else if let Some(rail) = class.active_rail() {
            match (rail.voltage_group(), class.pax().is_full()) {
                (High, true) => self.el_rail_high_pax,
                (High, false) => self.el_rail_high,
                (Low, true) => self.el_rail_low_pax,
                (Low, false) => self.el_rail_low,
                _ => self.toxic,
            }
        }
        else {
            if class.pax().is_full() {
                self.el_none_pax
            }
            else {
                self.el_none
            }
        }
    }

    fn pax_color(&self, class: &Class) -> Color {
        if !class.is_open() {
            self.pax_closed
        }
        else {
            match class.pax() {
                Pax::None => self.pax_none,
                Pax::Full => self.pax_full,
                _ => self.pax_ltd,
            }
        }
    }

    fn proof_color(&self, class: &Class) -> Color {
        self.el_color(class)
    }

    fn el_cat_color(&self, class: &Class) -> Option<Color> {
        use VoltageGroup::*;
        use ElectricSystem::*;

        class.cat().map(|cat| {
            if class.category().is_tram() {
                match cat.status {
                    ElectricStatus::Open => return self.tram,
                    ElectricStatus::Removed => return self.tram_removed,
                }
            }

            match cat.status {
                ElectricStatus::Open => {
                    match (
                        cat.system, cat.voltage_group(), class.pax().is_full()
                    ) {
                        (Some(Ac), High, true) => self.el_ole_ac_high_pax,
                        (Some(Ac), High, false) => self.el_ole_ac_high,
                        (Some(Ac), Low, true) => self.el_ole_ac_low_pax,
                        (Some(Ac), Low, false) => self.el_ole_ac_low,
                        (Some(Dc), High, true) => self.el_ole_dc_high_pax,
                        (Some(Dc), High, false) => self.el_ole_dc_high,
                        (Some(Dc), Low, true) => self.el_ole_dc_low_pax,
                        (Some(Dc), Low, false) => self.el_ole_dc_low,
                        _ => self.toxic,
                    }
                }
                ElectricStatus::Removed => self.removed,
            }
        })
    }

    fn pax_cat_color(&self, class: &Class) -> Option<Color> {
        class.cat().map(|cat| {
            match cat.status {
                ElectricStatus::Open => self.pax_color(class),
                ElectricStatus::Removed => self.removed,
            }
        })
    }

    fn proof_cat_color(&self, class: &Class) -> Option<Color> {
        self.el_cat_color(class)
    }

    fn el_rail_color(&self, class: &Class) -> Option<Color> {
        use VoltageGroup::*;

        class.rail().map(|rail| {
            if class.category().is_tram() {
                match rail.status {
                    ElectricStatus::Open => return self.tram,
                    ElectricStatus::Removed => return self.tram_removed,
                }
            }

            match rail.status {
                ElectricStatus::Open => {
                    match (rail.voltage_group(), class.pax().is_full()) {
                        (High, true) => self.el_rail_high_pax,
                        (High, false) => self.el_rail_high,
                        (Low, true) => self.el_rail_low_pax,
                        (Low, false) => self.el_rail_low,
                        _ => self.toxic,
                    }
                }
                ElectricStatus::Removed => self.removed,
            }
        })
    }

    fn pax_rail_color(&self, class: &Class) -> Option<Color> {
        class.rail().map(|rail| {
            match rail.status {
                ElectricStatus::Open => self.pax_color(class),
                ElectricStatus::Removed => self.removed,
            }
        })
    }

    fn proof_rail_color(&self, class: &Class) -> Option<Color> {
        self.el_rail_color(class)
    }

    fn common_color(&self, class: &Class) -> Option<Color> {
        if let Some(color) = self.tram_color(class) {
            return Some(color)
        }
        if let Some(color) = self.ex_color(class.status()) {
            return Some(color)
        }
        None
    }

    fn tram_color(&self, class: &Class) -> Option<Color> {
        if !matches!(class.category(), Category::Tram) {
            return None
        }

        match class.status() {
            Status::Open | Status::Planned => Some(self.tram),
            Status::Closed => Some(self.tram_closed),
            Status::Removed | Status::Explanned => Some(self.tram_removed),
            Status::Gone => Some(self.tram_gone),
        }
    }

    fn ex_color(&self, status: Status) -> Option<Color> {
        match status {
            Status::Open | Status::Planned => None,
            Status::Closed => Some(self.closed),
            Status::Removed | Status::Explanned => Some(self.removed),
            Status::Gone => Some(self.gone),
        }
    }
}

impl Default for ColorSet {
    fn default() -> Self {
        ColorSet {
            el_none_pax:        Color::hex("98690dff").unwrap(),
            el_ole_ac_high_pax: Color::hex("8845aaff").unwrap(),
            el_ole_ac_low_pax:  Color::hex("aa4689ff").unwrap(),
            el_ole_dc_high_pax: Color::hex("a51100ff").unwrap(),
            el_ole_dc_low_pax:  Color::hex("d05113ff").unwrap(),
            el_rail_high_pax:   Color::hex("007e49ff").unwrap(),
            el_rail_low_pax:   Color::hex("007e49ff").unwrap(),
            //el_rail_low_pax:    Color::hex("6e6e00ff").unwrap(),

            el_none:        Color::hex("523700ff").unwrap(),
            el_ole_ac_high: Color::hex("4d2263ff").unwrap(),
            el_ole_ac_low:  Color::hex("691f51ff").unwrap(),
            el_ole_dc_high: Color::hex("720c00ff").unwrap(),
            el_ole_dc_low:  Color::hex("ac3900ff").unwrap(),
            el_rail_high:   Color::hex("004f2eff").unwrap(),
            el_rail_low:   Color::hex("004f2eff").unwrap(),
            //el_rail_low:    Color::hex("444400ff").unwrap(),

            pax_full: Color::grey(0.1),
            pax_ltd: Color::grey(0.3),
            pax_none: Color::grey(0.7),
            pax_closed: Color::grey(0.9),

            closed:  Color::grey(0.550),
            removed: Color::grey(0.650),
            gone:    Color::grey(0.850),

            tram:         Color::hex("1c63abff").unwrap(),
            tram_closed:  Color::hex("5e8eb9ff").unwrap(),
            tram_removed: Color::hex("8fb0d1ff").unwrap(),
            tram_gone:    Color::hex("bed2e4ff").unwrap(),

            toxic: Color::rgb(0.824, 0.824, 0.0),
        }
    }
}

//------------ InvalidStyle --------------------------------------------------

pub struct InvalidStyle;

