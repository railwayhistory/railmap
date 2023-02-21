//! Classes of features.
//!
//! In order to be able to draw features differently for different styles, we
//! need to describe them in an abstract fashion. This is what classes do.
//!
//! There is one big [`Class`] type that aggregates various sub-classes the
//! describing various more specific things. In the map source, the class of
//! a feature is typically given through symbol set.
#![allow(dead_code)]

use crate::import::eval;
use crate::import::Failed;
use crate::import::eval::{Expression, SymbolSet};
use super::theme::Railwayhistory;


//------------ Class ---------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Class {
    category: Option<Category>,
    status: Option<Status>,
    surface: Option<Surface>,
    cat: Option<ElectricCat>,
    rail: Option<ElectricRail>,
    speed: Option<Speed>,
    pax: Option<Pax>,
}

impl Class {
    pub fn from_arg(
        arg: Expression<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let class = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(class)
    }

    pub fn from_symbols(symbols: &mut SymbolSet) -> Self {
        Class {
            category: Category::from_symbols(symbols),
            status: Status::from_symbols(symbols),
            surface: Surface::from_symbols(symbols),
            cat: ElectricCat::from_symbols(symbols),
            rail: ElectricRail::from_symbols(symbols),
            speed: Speed::from_symbols(symbols),
            pax: Pax::from_symbols(symbols),
        }
    }

    pub fn update(&self, class: &Class) -> Class {
        Class {
            category: {
                if let Some(category) = class.category {
                    Some(category)
                }
                else {
                    self.category
                }
            },
            status: {
                if let Some(status) = class.status {
                    Some(status)
                }
                else {
                    self.status
                }
            },
            surface: {
                if let Some(surface) = class.surface {
                    Some(surface)
                }
                else {
                    self.surface
                }
            },
            cat: {
                if let Some(cat) = class.cat {
                    Some(cat)
                }
                else {
                    self.cat
                }
            },
            rail: {
                if let Some(rail) = class.rail {
                    Some(rail)
                }
                else {
                    self.rail
                }
            },
            speed: {
                if let Some(speed) = class.speed {
                    Some(speed)
                }
                else {
                    self.speed
                }
            },
            pax: {
                if let Some(pax) = class.pax {
                    Some(pax)
                }
                else {
                    self.pax
                }
            }
        }
    }

    pub fn category(&self) -> Category {
        self.category.unwrap_or_default()
    }

    pub fn status(&self) -> Status {
        self.status.unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

    pub fn is_open(&self) -> bool {
        matches!(self.status(), Status::Open)
    }

    pub fn surface(&self) -> Surface {
        self.surface.unwrap_or_default()
    }

    pub fn cat(&self) -> Option<ElectricCat> {
        self.cat
    }

    pub fn has_active_cat(&self) -> bool {
        if let Some(cat) = self.cat {
            matches!(cat.status, ElectricStatus::Open)
        }
        else {
            false
        }
    }

    pub fn active_cat(&self) -> Option<ElectricCat> {
        if let Some(cat) = self.cat {
            if matches!(cat.status, ElectricStatus::Open) {
                return Some(cat)
            }
        }
        None
    }


    pub fn rail(&self) -> Option<ElectricRail> {
        self.rail
    }

    pub fn has_active_rail(&self) -> bool {
        if let Some(rail) = self.rail {
            matches!(rail.status, ElectricStatus::Open)
        }
        else {
            false
        }
    }

    pub fn active_rail(&self) -> Option<ElectricRail> {
        if let Some(rail) = self.rail {
            if matches!(rail.status, ElectricStatus::Open) {
                return Some(rail)
            }
        }
        None
    }

    pub fn speed(&self) -> Speed {
        self.speed.unwrap_or_default()
    }

    pub fn pax(&self) -> Pax {
        self.pax.unwrap_or_default()
    }

    /// Returns the layer offset for this class.
    ///
    /// Add this to your layer to correctly order features of the same type
    /// with different classes.
    ///
    /// Class layer offsets are in the range of -0.005 to 0.
    pub fn layer_offset(&self) -> f64 {
        let base = if self.pax().is_full() { 0. }
        else if self.category().is_tram() { -0.001 }
        else { -0.002 };
        let electric = if self.has_active_cat() { -0.00001 }
                       else if self.has_active_rail() { -0.00005 }
                       else { -0.00008 };
        base + self.status().layer_offset() + electric
    }
}


//------------ Category ------------------------------------------------------

/// The category of railway this feature is for.
#[derive(Clone, Copy, Debug)]
pub enum Category {
    /// First-class public railway.
    First,

    /// Second-class public railway.
    Second,

    /// Third-class public railway.
    Third,

    /// Tram.
    Tram,

    /// A non-public railway.
    Private,

    /// Sidings tracks.
    ///
    /// For historical reasons, this is the default category.
    Siding,
}

impl Category {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("first") {
            Some(Category::First)
        }
        else if symbols.take("second") {
            Some(Category::Second)
        }
        else if symbols.take("third") {
            Some(Category::Third)
        }
        else if symbols.take("tram") {
            Some(Category::Tram)
        }
        else if symbols.take("private") {
            Some(Category::Private)
        }
        else {
            None
        }
    }

    /// Returns whether the category is a first or second class.
    pub fn is_main(self) -> bool {
        match self {
            Category::First | Category::Second => true,
            _ => false
        }
    }

    /// Returns whether the category is a tram.
    pub fn is_tram(self) -> bool {
        matches!(self, Category::Tram)
    }
}

impl Default for Category {
    fn default() -> Self {
        Category::Siding
    }
}


//------------ Status --------------------------------------------------------

/// The status of the feature.
#[derive(Clone, Copy, Debug)]
pub enum Status {
    /// The feature was planned but abandoned.
    Explanned,

    /// The feature is planned or under construction.
    Planned,

    /// The feature is open and in use.
    ///
    /// This is the default if the status is not explicitly given.
    Open,

    /// The feature is closed but still present.
    Closed,

    /// The feature has been removed.
    Removed,

    /// The feature has been removed a long time ago.
    Gone
}

impl Status {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("exproject") {
            Some(Status::Explanned)
        }
        else if symbols.take("project") {
            if symbols.take("removed") {
                Some(Status::Explanned)
            }
            else {
                Some(Status::Planned)
            }
        }
        else if symbols.take("open") {
            Some(Status::Open)
        }
        else if symbols.take("closed") {
            Some(Status::Closed)
        }
        else if symbols.take("removed") {
            Some(Status::Removed)
        }
        else if symbols.take("gone") {
            Some(Status::Gone)
        }
        else {
            None
        }
    }

    pub fn is_project(self) -> bool {
        matches!(self, Status::Explanned | Status::Planned)
    }

    pub fn layer_offset(self) -> f64 {
        match self {
            Status::Open => 0.,
            Status::Closed => -0.0001,
            Status::Removed => -0.0002,
            Status::Gone => -0.0003,
            _ => -0.0004,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Open
    }
}


//------------ Surface -------------------------------------------------------

/// The surface type the track is laid on.
#[derive(Clone, Copy, Debug)]
pub enum Surface {
    /// The track sits on regular ground.
    ///
    /// This is the default.
    Ground,

    /// The track is on a bridge.
    Bridge,

    /// The track is in a tunnel.
    Tunnel,
}

impl Default for Surface {
    fn default() -> Self {
        Surface::Ground
    }
}

impl Surface {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("ground") {
            Some(Surface::Ground)
        }
        else if symbols.take("bridge") {
            Some(Surface::Bridge)
        }
        else if symbols.take("tunnel") {
            Some(Surface::Tunnel)
        }
        else {
            None
        }
    }

    pub fn is_tunnel(self) -> bool {
        matches!(self, Surface::Tunnel)
    }
}


//------------ ElectricCat ---------------------------------------------------

/// The electrification system for overhead line electrification.
#[derive(Clone, Copy, Debug)]
pub struct ElectricCat {
    /// The status of the system.
    pub status: ElectricStatus,

    /// The nominal voltage of the system.
    pub voltage: Option<u16>,

    /// The type (?) of current in use.
    pub system: Option<ElectricSystem>,
}

impl ElectricCat {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        let mut res = if symbols.take("cat") {
            ElectricCat {
                status: ElectricStatus::Open,
                voltage: None,
                system: None,
            }
        }
        else if symbols.take("excat") {
            ElectricCat {
                status: ElectricStatus::Removed,
                voltage: None,
                system: None,
            }
        }
        else {
            return None
        };
        
        for &(name, voltage, system) in Self::SYSTEMS {
            if symbols.take(name) {
                res.voltage = Some(voltage);
                res.system = Some(system);
                break
            }
        }
        Some(res)
    }

    const SYSTEMS: &'static [(&'static str, u16, ElectricSystem)] = &[
        ("ac0k725", 725, Ac),
        ("ac6k6", 6600, Ac),
        ("ac15", 15000, Ac),
        ("ac11", 11000, Ac),
        ("ac25", 25000, Ac),
        ("dc30", 3000, Dc),
        ("dc3", 3000, Dc), // XXX Temporary. Fix in data!
        ("dc15", 1500, Dc),
        ("dc9", 900, Dc),
        ("dc85", 850, Dc),
        ("dc75", 750, Dc),
        ("dc7", 700, Dc),
        ("dc6", 600, Dc),
    ];

    pub fn voltage_group(self) -> VoltageGroup {
        let (voltage, system) = match (self.voltage, self.system) {
            (Some(voltage), Some(system)) => (voltage, system),
            _ => return VoltageGroup::Unknown
        };
        match (system, voltage) {
            (Ac, voltage) if voltage >= 20000 => VoltageGroup::High,
            (Ac, _) => VoltageGroup::Low,
            (Dc, voltage) if voltage >= 2000 => VoltageGroup::High,
            (Dc, _) => VoltageGroup::Low,
        }
    }
}


//------------ ElectricRail ---------------------------------------------------

/// The electrification system for third and fourth rail electrification.
#[derive(Clone, Copy, Debug)]
pub struct ElectricRail {
    /// The status of the system.
    pub status: ElectricStatus,

    /// The nominal voltage of the system.
    pub voltage: Option<u16>,

    /// Is the system a fourth rail system using two power rails?
    pub fourth: bool,
}

impl ElectricRail {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        let mut res = if symbols.take("rail") {
            ElectricRail {
                status: ElectricStatus::Open,
                voltage: None,
                fourth: false,
            }
        }
        else if symbols.take("exrail") {
            ElectricRail {
                status: ElectricStatus::Removed,
                voltage: None,
                fourth: false,
            }
        }
        else if symbols.take("rail4") {
            ElectricRail {
                status: ElectricStatus::Open,
                voltage: None,
                fourth: true,
            }
        }
        else if symbols.take("exrail4") {
            ElectricRail {
                status: ElectricStatus::Removed,
                voltage: None,
                fourth: true,
            }
        }
        else {
            return None
        };
        
        for &(name, voltage) in Self::SYSTEMS {
            if symbols.take(name) {
                res.voltage = Some(voltage);
                break
            }
        }
        Some(res)
    }

    const SYSTEMS: &'static [(&'static str, u16)] = &[
        ("rc12", 1200),
        ("rc85", 850),
        ("rc75", 750),
        ("rc63", 630),
    ];

    pub fn voltage_group(self) -> VoltageGroup {
        match self.voltage {
            Some(voltage) => {
                if voltage >= 1000 {
                    VoltageGroup::High
                }
                else {
                    VoltageGroup::Low
                }
            }
            None => VoltageGroup::Unknown
        }
    }
}


//------------ ElectricStatus ------------------------------------------------

/// The status of the feature.
#[derive(Clone, Copy, Debug)]
pub enum ElectricStatus {
    Open,
    Removed
}


//------------ ElectricSystem ------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum ElectricSystem {
    Ac,
    Dc,
}

use ElectricSystem::*;


//------------ VoltageGroup --------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum VoltageGroup {
    Low,
    High,
    Unknown
}


//------------ Speed ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Speed {
    V160,
    V200,
    V250,
    V300,
}

impl Speed {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("v160") {
            Some(Speed::V160)
        }
        else if symbols.take("v200") {
            Some(Speed::V200)
        }
        else if symbols.take("v250") {
            Some(Speed::V250)
        }
        else if symbols.take("v300") {
            Some(Speed::V300)
        }
        else {
            None
        }
    }

    pub fn is_hsl(self) -> bool {
        !matches!(self, Speed::V160)
    }
}

impl Default for Speed {
    fn default() -> Self {
        Speed::V160
    }
}


//------------ Pax -----------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Pax {
    /// There is no passenger service.
    None,

    /// There is heritage passenger service.
    Heritage,

    /// Passenger service is either seasonal or not all week.
    Seasonal,

    /// Scheduled, daily passenger service.
    Full,
}

impl Pax {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("nopax") {
            Some(Pax::None)
        }
        else if symbols.take("pax") {
            Some(Pax::Full)
        }
        else if symbols.take("heritage") || symbols.take("museum") {
            Some(Pax::Heritage)
        }
        else if symbols.take("seasonal") {
            Some(Pax::Seasonal)
        }
        else {
            None
        }
    }

    pub fn is_full(self) -> bool {
        matches!(self, Pax::Full)
    }
}

impl Default for Pax {
    fn default() -> Self {
        Pax::None
    }
}


//------------ Gauge ---------------------------------------------------------

/// The track gauge.
#[derive(Clone, Copy, Debug)]
pub struct Gauge {
    /// The main gauge in mm.
    ///
    /// If this `None` when finally evaluating, it is actually 1435.
    main: Option<u16>,

    /// The secondary gauge in mm if present.
    ///
    /// This is only present for three or four rail track.
    secondary: Option<u16>,
}

impl Gauge {
    pub fn from_symbols(symbols: &mut SymbolSet) -> Self {
        // XXX Temporarily accept deprecated symbols.
        symbols.take("narrow");
        symbols.take("narrower");

        let mut res = Gauge { main: None, secondary: None };
        for &(name, gauge) in Self::MAIN_GAUGES {
            if symbols.take(name) {
                res.main = Some(gauge);
                break;
            }
        }
        if res.main.is_none() {
            return res
        }
        for &(name, gauge) in Self::SECONDARY_GAUGES {
            if symbols.take(name) {
                res.secondary = Some(gauge);
                break;
            }
        }
        res
    }

    const MAIN_GAUGES: &'static [(&'static str, u16)] = &[
        ("g600", 600),
        ("g750", 750),
        ("g785", 785),
        ("g800", 800),
        ("g900", 900),
        ("g1000", 1000),
        ("g1435", 1435),
        ("g1520", 1520),
        ("g1524", 1524),
    ];

    const SECONDARY_GAUGES: &'static [(&'static str, u16)] = &[
        ("gg750", 750),
        ("gg1000", 1000),
        ("gg1435", 1435),
    ];

    pub fn main(self) -> u16 {
        self.main.unwrap_or(1435)
    }

    pub fn main_group(self) -> GaugeGroup {
        if self.main() == 1435 {
            GaugeGroup::Standard
        }
        else if self.main() < 600 {
            GaugeGroup::Minimum
        }
        else if self.main() < 1000 {
            GaugeGroup::Narrower 
        }
        else if self.main() < 1435 {
            GaugeGroup::Narrow
        }
        else {
            GaugeGroup::Broad
        }
    }

    pub fn secondary(self) -> Option<u16> {
        self.secondary
    }
}


//------------ GaugeGroup ----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum GaugeGroup {
    Minimum,
    Narrower,
    Narrow,
    Standard,
    Broad,
}

impl GaugeGroup {
    /// Returns whether the group is standard.
    pub fn is_standard(self) -> bool {
        matches!(self, GaugeGroup::Standard)
    }
}

