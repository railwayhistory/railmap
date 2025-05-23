//! Classification of features.
//!
//! In order to be able to draw features differently for different styles, we
//! need to describe them in an abstract fashion. This is what classes do.
//!
//! Since railways are very comples, there are plenty extra types here that
//! help with defining the railway class.
#![allow(dead_code)]

use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use super::import::eval::{Expression, Scope, ScopeExt};


//------------ Railway -------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Railway {
    category: Option<Category>,
    status: Option<Status>,
    surface: Option<Surface>,
    cat: Option<ElectricCat>,
    rail: Option<ElectricRail>,
    speed: Option<Speed>,
    pax: Option<Pax>,
    gauge_group: Option<GaugeGroup>,
    gauge: Option<Gauge>,
    double: Option<bool>,
    station: Option<bool>,
}

impl Railway {
    pub fn from_arg_only(
        arg: Expression,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.eval::<SymbolSet>(err)?;
        let mut class = Self::default();
        class.apply_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(class)
    }

    pub fn from_arg(
        arg: Expression,
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.eval::<SymbolSet>(err)?;
        let mut class = scope.railway().clone();
        class.apply_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(class)
    }

    pub fn from_symbols(symbols: &mut SymbolSet, scope: &Scope) -> Self {
        let mut class = scope.railway().clone();
        class.apply_symbols(symbols);
        class.gauge_group = Some(GaugeGroup::new(class.gauge, scope));
        class
    }

    pub fn from_symbols_only(symbols: &mut SymbolSet) -> Self {
        let mut class = Self::default();
        class.apply_symbols(symbols);
        class
    }

    pub fn from_scope(scope: &Scope) -> Self {
        scope.railway().clone()
    }

    fn apply_symbols(&mut self, symbols: &mut SymbolSet) {
        if let Some(category) = Category::from_symbols(symbols) {
            self.category = Some(category)
        }
        if let Some(status) = Status::from_symbols(symbols) {
            self.status = Some(status)
        }
        if let Some(surface) = Surface::from_symbols(symbols) {
            self.surface = Some(surface)
        }
        if let Some(cat) = ElectricCat::from_symbols(symbols) {
            self.cat = Some(cat)
        }
        if let Some(rail) = ElectricRail::from_symbols(symbols) {
            self.rail = Some(rail)
        }
        if let Some(speed) = Speed::from_symbols(symbols) {
            self.speed = Some(speed)
        }
        if let Some(pax) = Pax::from_symbols(symbols) {
            self.pax = Some(pax)
        }
        if let Some(gauge) = Gauge::from_symbols(symbols) {
            self.gauge = Some(gauge)
        }
        GaugeGroup::take_symbols(symbols);

        if symbols.take("double") {
            self.double = Some(true)
        }
        else if symbols.take("single") {
            self.double = Some(false)
        }

        if symbols.take("station") {
            self.station = Some(true)
        }
        else if symbols.take("nostation") {
            self.station = Some(false)
        }
    }

    pub fn update(&mut self, class: &Self) {
        if self.category.is_none() {
            self.category = class.category
        }
        if self.status.is_none() {
            self.status = class.status
        }
        if self.surface.is_none() {
            self.surface = class.surface
        }
        if self.cat.is_none() {
            self.cat = class.cat
        }
        if self.rail.is_none() {
            self.rail = class.rail
        }
        if self.speed.is_none() {
            self.speed = class.speed
        }
        if self.pax.is_none() {
            self.pax = class.pax
        }
        if self.gauge.is_none() {
            self.gauge = class.gauge
        }
        if self.gauge_group.is_none() {
            self.gauge_group = class.gauge_group
        }
        if self.double.is_none() {
            self.double = class.double
        }
        if self.station.is_none() {
            self.station = class.station
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

    pub fn is_open_no_pax(&self) -> bool {
        self.is_open() && !matches!(self.pax(), Pax::Full)
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

    pub fn opt_pax(&self) -> Option<Pax> {
        self.pax
    }

    pub fn gauge(&self) -> Gauge {
        self.gauge.unwrap_or_default()
    }

    pub fn gauge_group(&self) -> GaugeGroup {
        self.gauge_group.unwrap_or_default()
    }

    pub fn double(&self) -> bool {
        self.double.unwrap_or_default()
    }

    pub fn station(&self) -> bool {
        self.station.unwrap_or_default()
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
        else if symbols.take("side") {
            Some(Category::Siding)
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

    /// Returns whether the category is a railway line.
    ///
    /// This excludes trams, private, and sidings.
    pub fn is_railway(self) -> bool {
        !matches!(self, Self::Tram | Self::Private | Self::Siding)
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
///
/// The variants are ordered so that features with these statuses are drawn
/// in a  sensible order way atop each other.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Status {
    /// The feature has been removed a long time ago.
    Gone = 0,

    /// The feature was planned but abandoned.
    Explanned,

    /// The feature has been removed.
    Removed,

    /// The feature is closed but still present.
    Closed,

    /// The feature is technically open but unusable.
    Suspended,

    /// The feature is planned or under construction.
    Planned,

    /// The feature is open and in use.
    ///
    /// This is the default if the status is not explicitly given.
    Open,
}

impl Status {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        let res = if symbols.take("exproject") {
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
        else if symbols.take("suspended") {
            Some(Status::Suspended)
        }
        else if symbols.take("removed") {
            Some(Status::Removed)
        }
        else if symbols.take("gone") {
            Some(Status::Gone)
        }
        else {
            None
        };

        // Overwrite for labels with :former
        if symbols.contains("former") {
            Some(Status::Removed)
        }
        else {
            res
        }
    }

    pub fn is_open(self) -> bool {
        matches!(self, Status::Open)
    }

    pub fn is_project(self) -> bool {
        matches!(self, Status::Explanned | Status::Planned)
    }

    pub fn layer_offset(self) -> i16 {
        match self {
            Status::Open => 0,
            Status::Closed | Status::Suspended => -10,
            Status::Removed => -20,
            Status::Gone => -30,
            _ => -40,
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
        else if symbols.take("nocat") {
            return Some(ElectricCat {
                status: ElectricStatus::None,
                voltage: None,
                system: None,
            })
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
        ("ac65", 6500, Ac),
        ("ac15", 15000, Ac),
        ("ac11", 11000, Ac),
        ("ac25", 25000, Ac),
        ("dc30", 3000, Dc),
        ("dc33", 3300, Dc),
        ("dc3", 3000, Dc), // XXX Temporary. Fix in data!
        ("dc18", 1800, Dc),
        ("dc15", 1500, Dc),
        ("dc12", 1200, Dc),
        ("dc10", 1000, Dc),
        ("dc9", 900, Dc),
        ("dc85", 850, Dc),
        ("dc8", 800, Dc),
        ("dc75", 750, Dc),
        ("dc7", 700, Dc),
        ("dc6", 600, Dc),
        ("dc55", 550, Dc),
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
        else if symbols.take("norail") {
            return Some(ElectricRail {
                status: ElectricStatus::None,
                voltage: None,
                fourth: false,
            })
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
    None,
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Pax {
    /// There is no passenger service.
    None = 0,

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
        else if
               symbols.take("heritage")
            || symbols.take("museum")
            || symbols.take("tourist")
        {
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
    main: u16,

    /// The secondary gauge in mm if present.
    ///
    /// This is only present for three or four rail track.
    secondary: Option<u16>,
}

impl Gauge {
    pub fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        let mut main = None;
        for &(name, gauge) in Self::MAIN_GAUGES {
            if symbols.take(name) {
                main = Some(gauge);
                break;
            }
        }
        let main = main?;
        let mut secondary = None;
        for &(name, gauge) in Self::SECONDARY_GAUGES {
            if symbols.take(name) {
                secondary = Some(gauge);
                break;
            }
        }
        Some(Gauge { main, secondary })
    }

    const MAIN_GAUGES: &'static [(&'static str, u16)] = &[
        ("g600", 600),
        ("g750", 750),
        ("g760", 760),
        ("g762", 762),
        ("g785", 785),
        ("g800", 800),
        ("g802", 802),
        ("g891", 891),
        ("g900", 900),
        ("g950", 950),
        ("g1000", 1000),
        ("g1093", 1093),
        ("g1100", 1100),
        ("g1101", 1101),
        ("g1200", 1200),
        ("g1435", 1435),
        ("g1520", 1520),
        ("g1524", 1524),
    ];

    const SECONDARY_GAUGES: &'static [(&'static str, u16)] = &[
        ("gg750", 750),
        ("gg1000", 1000),
        ("gg1435", 1435),
        ("gg1524", 1524),
    ];

    pub fn main(self) -> u16 {
        self.main
    }

    pub fn secondary(self) -> Option<u16> {
        self.secondary
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Gauge { main: 1435, secondary: None }
    }
}


//------------ GaugeGroup ----------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum GaugeGroup {
    /// A narrow gauge below the region’s base.
    Narrow,

    /// Standard gauge in regions with a broad base gauge.
    StandardNarrow,

    /// The base gauge of the region.
    #[default]
    Base,

    /// Standard gauge in regions with a narrow base gauge.
    StandardBroad,

    /// A gauge broader than the region’s base.
    Broad,
}

impl GaugeGroup {
    fn new(gauge: Option<Gauge>, scope: &Scope) -> Self {
        let gauge = match gauge {
            Some(gauge) => gauge.main,
            None => return Self::Base,
        };
        let base_gauge = scope.base_gauge();

        if base_gauge < 1435 {
            if gauge == 1435 {
                Self::StandardBroad
            }
            else if gauge < base_gauge {
                Self::Narrow
            }
            else if gauge == base_gauge {
                Self::Base
            }
            else {
                Self::Broad
            }
        }
        else if base_gauge == 1435 {
            if gauge < 1435 {
                Self::Narrow
            }
            else if gauge == 1435 {
                Self::Base
            }
            else {
                Self::Broad
            }
        }
        else {
            if gauge == 1435 {
                Self::StandardNarrow
            }
            else if gauge < base_gauge {
                Self::Narrow
            }
            else if gauge == base_gauge {
                Self::Base
            }
            else {
                Self::Broad
            }
        }
    }

    fn take_symbols(symbols: &mut SymbolSet) {
        if symbols.take("narrower") {
            return
        }
        else if symbols.take("narrow") {
            return
        }
        else if symbols.take("standard") {
            return
        }
        else if symbols.take("broad") {
            return
        }
        else if symbols.take("broader") {
            return
        }
    }

    pub fn is_narrow(self) -> bool {
        matches!(self, Self::Narrow)
    }
}

