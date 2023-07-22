//! The style of the map to be rendered.

use std::collections::HashMap;
use std::str::FromStr;
use std::ops::MulAssign;
use std::sync::Arc;
use femtomap::path::MapDistance;
use femtomap::render::Color;
use crate::theme;
use crate::tile::{TileId, TileFormat};
use crate::transform::Transform;
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
    pub fn detail(self, zoom: u8) -> f64 {
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

    pub fn palette(self) -> Palette {
        match self {
            StyleId::Overview(pal) => pal,
            StyleId::Detail(pal) => pal,
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
    detail: f64,
    mag: f64,
    dimensions: Dimensions,
    colors: Arc<ColorSet>,
    transform: Transform,
}

impl Style {
    pub fn detail(&self) -> f64 {
        self.detail
    }

    pub fn detail_step(&self) -> u8 {
        self.detail as u8
    }

    pub fn resolve_distance(&self, distance: MapDistance) -> f64 {
        distance.value()
        * units::MAP_DISTANCES[distance.unit()].1[self.detail as usize]
    }
}

impl theme::Style for Style {
    type StyleId = StyleId;

    fn bounds_correction(&self) -> f64 {
        BOUNDS_CORRECTION * (self.detail as f64)
    }

    fn mag(&self) -> f64 {
        self.mag
    }

    fn detail(&self) -> f64 {
        self.detail
    }

    fn scale(&mut self, canvas_bp: f64) {
        self.dimensions *= canvas_bp;
    }

    fn resolve_distance(&self, distance: MapDistance) -> f64 {
        distance.value()
        * units::MAP_DISTANCES[distance.unit()].1[self.detail as usize]
    }

    fn transform(&self) -> Transform {
        self.transform
    }
}

impl femtomap::path::Style for Style {
    fn resolve_distance(&self, distance: MapDistance) -> f64 {
        distance.value()
        * units::MAP_DISTANCES[distance.unit()].1[self.detail as usize]
    }

    fn transform(&self) -> Transform {
        self.transform
    }
}


impl Style {
    pub fn new(
        id: &TileId<StyleId>,
        colors: Arc<ColorSet>,
    ) -> Self {
        let detail = id.style.detail(id.zoom);
        let mag = id.style.mag(id.zoom);
        let canvas_bp = id.format.canvas_bp() * mag;
        let scale = id.format.size() * id.n();
        let mut dimensions = if detail < 3. {
            Dimensions::D0
        }
        else if detail < 4. {
            Dimensions::D3
        }
        else {
            Dimensions::D4
        };
        dimensions *= canvas_bp;
        Style {
            id: id.style,
            detail,
            mag,
            dimensions,
            colors,
            transform: Transform::new( canvas_bp, id.nw(), scale),
        }
    }

    pub fn new_map_key(
        zoom: u8,
        id: StyleId,
        format: TileFormat,
        colors: Arc<ColorSet>,
    ) -> Self {
        let canvas_bp = format.canvas_bp();
        let detail = id.detail(zoom);
        let mut dimensions = if detail < 3. {
            Dimensions::D0
        }
        else if detail < 4. {
            Dimensions::D3
        }
        else {
            Dimensions::D4
        };
        dimensions *= canvas_bp;

        Style {
            id,
            detail,
            mag: 1.,
            dimensions,
            colors,
            transform: Transform::new_map_key(canvas_bp)
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    pub fn palette(&self) -> Palette {
        match self.id {
            StyleId::Overview(pal) => pal,
            StyleId::Detail(pal) => pal
        }
    }

    pub fn include_line_labels(&self) -> bool {
        self.palette().show_linenum()
    }

    pub fn canvas_bp(&self) -> f64 {
        self.transform.canvas_bp()
    }
}


//--- Colors
//
impl Style {
    /// Returns the color for a piece of track.
    pub fn track_color(&self, class: &Class) -> Color {
        self.colors.color(self.palette(), class)
    }

    /// Returns the color for the track underlay.
    pub fn track_underlay_color(&self, class: &Class) -> Color {
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
        self.colors.label_color(self.palette(), class)
    }

    /// Returns the primary color for a marker.
    pub fn primary_marker_color(&self, class: &Class) -> Color {
        self.colors.color(self.palette(), class)
    }
}


//============ Detail Levels and Magnifications ==============================

/// The mapping of zoom levels to details.
const DETAIL_DETAILS: &[f64] = &[
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.5, 1.0, 1.5, 2.0,
    3.0, 3.5, 4.0, 4.5, 5.0,
    5.5, 5.5, 5.5, 5.5, 5.5,
];

/// The mapping of zoom levels to magnification.
const DETAIL_MAG: &[f64] = &[
    1., 1., 1., 1., 1., 
    1., 1., 1., 1.3, 1., 
    1., 1.3, 1., 1.5, 1.2,
    1.7, 2., 3., 1., 1.,
];

/// The mapping of zoom levels to details.
const OVERVIEW_DETAILS: &[f64] = &[
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.5, 1.0, 1.5, 2.0,
    2.5, 2.5, 2.5, 2.5, 2.5,
    2.5, 2.5, 2.5, 2.5, 2.5,
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

    pub mark_width: f64,

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

    /// The line width of station symbols casing.
    pub csp: f64,

    /// The line width of border symbols.
    pub bp: f64,

    /// The line width of border symbols casing.
    pub cbp: f64,
}

impl Dimensions {
    const D0: Self = Self {
        line_width: 0.8,
        other_width: 0.5,
        mark_width: 0.5,
        guide_width: 0.3,
        seg: 6. * units::DT,
        dt: units::DT,
        mark: 0.6 *  units::DT,
        tight_mark: 0.55 * units::DT,
        sw: units::SSW,
        sh: 0.96 * units::SSW,
        ksw: units::SSW,
        ksh: 0.96 * units::SSW,
        ds: 0.05 * units::SSW,
        sp: 0.4,
        csp: 4. * 0.4,
        bp: 0.4,
        cbp: 4. * 0.4,
    };

    const D3: Self = Self {
        line_width: 1.,
        other_width: 0.7,
        mark_width: 0.7,
        dt: 0.8 * units::DT,
        mark: 0.6 * units::DT,
        tight_mark: 0.4 * units::DT,
        seg: 0.8 * 5. * units::DT,
        sw: units::S3W,
        sh: units::S3H,
        ds: 0.15 * units::S3W,
        sp: 0.8,
        csp: 4. * 0.8,
        .. Self::D0
    };

    const D4: Self = Self {
        line_width: 1.1,
        other_width: 0.8,
        mark_width: 0.5,
        mark: 0.8 * units::DT,
        sw: units::SW,
        sh: units::SH,
        ksw: 0.8 * units::SW,
        ksh: 0.8 * units::SH,
        ds: 0.075 * units::SH,
        sp: 0.8,
        csp: 8. * 0.8,
        bp: 0.6,
        cbp: 8. * 0.6,
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
        self.mark_width *= rhs;
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
    el_rail_pax,
    el_rail_four_pax,
    el_none_pax,
    el_ole_ac_low,
    el_ole_ac_high,
    el_ole_dc_low,
    el_ole_dc_high,
    el_rail,
    el_rail_four,
    el_none,

    pax_full,
    pax_ltd,
    pax_none,
    pax_closed,

    closed,
    removed,
    gone,

    closed_text,
    removed_text,
    gone_text,

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

    fn label_color(&self, palette: Palette, class: &Class) -> Color {
        if let Some(color) = self.ex_label_color(class) {
            color
        }
        else {
            self.color(palette, class)
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
            match (rail.fourth, class.pax().is_full()) {
                (false, true) => self.el_rail_pax,
                (false, false) => self.el_rail,
                (true, true) => self.el_rail_four_pax,
                (true, false) => self.el_rail_four,
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
                _ => self.pax_full,
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
        class.rail().map(|rail| {
            if class.category().is_tram() {
                match rail.status {
                    ElectricStatus::Open => return self.tram,
                    ElectricStatus::Removed => return self.tram_removed,
                }
            }

            match rail.status {
                ElectricStatus::Open => {
                    match (rail.fourth, class.pax().is_full()) {
                        (false, true) => self.el_rail_pax,
                        (false, false) => self.el_rail,
                        (true, true) => self.el_rail_four_pax,
                        (true, false) => self.el_rail_four,
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

    fn ex_label_color(&self, class: &Class) -> Option<Color> {
        if let Some(color) = self.tram_color(class) {
            return Some(color)
        }
        match class.status() {
            Status::Open | Status::Planned => None,
            Status::Closed => Some(self.closed_text),
            Status::Removed | Status::Explanned => Some(self.removed_text),
            Status::Gone => Some(self.gone_text),
        }
    }
}

#[allow(dead_code)]
impl ColorSet {
    fn colorful() -> Self {
        ColorSet {
            el_none_pax:        Color::hex(EL_NONE).unwrap(),
            el_ole_ac_high_pax: Color::hex(EL_OLE_AC_HIGH).unwrap(),
            el_ole_ac_low_pax:  Color::hex(EL_OLE_AC_LOW).unwrap(),
            el_ole_dc_high_pax: Color::hex(EL_OLE_DC_HIGH).unwrap(),
            el_ole_dc_low_pax:  Color::hex(EL_OLE_DC_LOW).unwrap(),
            el_rail_pax:        Color::hex(EL_RAIL).unwrap(),
            el_rail_four_pax:   Color::hex(EL_RAIL_FOUR).unwrap(),

            el_none:        Color::hex(EL_NONE).unwrap(),
            el_ole_ac_high: Color::hex(EL_OLE_AC_HIGH).unwrap(),
            el_ole_ac_low:  Color::hex(EL_OLE_AC_LOW).unwrap(),
            el_ole_dc_high: Color::hex(EL_OLE_DC_HIGH).unwrap(),
            el_ole_dc_low:  Color::hex(EL_OLE_DC_LOW).unwrap(),
            el_rail:        Color::hex(EL_RAIL).unwrap(),
            el_rail_four:   Color::hex(EL_RAIL).unwrap(),

            pax_full: Color::grey(0.1),
            pax_ltd: Color::grey(0.3),
            pax_none: Color::grey(0.7),
            pax_closed: Color::grey(0.9),

            closed:  Color::grey(0.550),
            removed: Color::grey(0.650),
            gone:    Color::grey(0.850),

            closed_text:  Color::grey(0.500),
            removed_text: Color::grey(0.500),
            gone_text:    Color::grey(0.500),

            tram:         Color::hex(TRAM).unwrap(),
            tram_closed:  Color::grey(0.550),
            tram_removed: Color::grey(0.650),
            tram_gone:    Color::grey(0.850),
            /*
            tram_closed:  Color::hex("5e8eb9ff").unwrap(),
            tram_removed: Color::hex("8fb0d1ff").unwrap(),
            tram_gone:    Color::hex("bed2e4ff").unwrap(),
            */

            toxic: Color::rgb(0.824, 0.824, 0.0),
        }
    }

    pub fn bw() -> Self {
        let color = Color::grey(0.1);
        ColorSet {
            el_none_pax:        color,
            el_ole_ac_high_pax: color,
            el_ole_ac_low_pax:  color,
            el_ole_dc_high_pax: color,
            el_ole_dc_low_pax:  color,
            el_rail_pax:        color,
            el_rail_four_pax:   color,

            el_none:        color,
            el_ole_ac_high: color,
            el_ole_ac_low:  color,
            el_ole_dc_high: color,
            el_ole_dc_low:  color,
            el_rail:        color,
            el_rail_four:   color,

            pax_full: Color::grey(0.1),
            pax_ltd: Color::grey(0.3),
            pax_none: Color::grey(0.7),
            pax_closed: Color::grey(0.9),

            closed:  Color::grey(0.550),
            removed: Color::grey(0.650),
            gone:    Color::grey(0.850),

            closed_text:  Color::grey(0.500),
            removed_text: Color::grey(0.500),
            gone_text:    Color::grey(0.500),

            tram:         Color::hex("1c63abff").unwrap(),
            tram_closed:  Color::grey(0.550),
            tram_removed: Color::grey(0.650),
            tram_gone:    Color::grey(0.850),

            toxic: Color::rgb(0.824, 0.824, 0.0),
        }
    }
}

impl Default for ColorSet {
    fn default() -> Self {
        Self::colorful()
    }
}

//------------ InvalidStyle --------------------------------------------------

pub struct InvalidStyle;


//------------ Color Constants -----------------------------------------------

const EL_NONE: &str = "5a3a29ff";
const EL_OLE_AC_HIGH: &str  = "6c2b86ff";
const EL_OLE_AC_LOW: &str = "812c5cff";
const EL_OLE_DC_HIGH: &str = "9b2321ff";
const EL_OLE_DC_LOW: &str = "b64f0dff";
const EL_RAIL: &str = "007e40ff";
const EL_RAIL_FOUR: &str = "53633bff";
const TRAM: &str = "005387ff";

#[allow(dead_code)]
const BORDER: &str = "cb6894ff";
