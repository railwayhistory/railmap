//! The coloring rules for the electrification map.

use femtomap::render::Color;
use crate::railway::class;

//------------ Colors --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Colors {
    /// The color for no electrification.
    none: Color,

    /// The color for AC OLE electrification above 20kV.
    ac_high: Color,

    /// The color for AC OLE electrification below 20kV.
    ac_low: Color,

    /// The color for DC OLE electrification above 2kV.
    dc_high: Color,

    /// The color for DC OLE electrification below 2kV.
    dc_low: Color,

    /// The color for third rail electrification.
    rail: Color,

    /// The color for fourth rail electrification.
    four: Color,

    /// The color for trams.
    tram: Color,

    /// The color for closed railway.
    closed: Color,

    /// The color for removed railway.
    removed: Color,

    /// The color for gone railway.
    gone: Color,

    /// The color for labels on closed railways .
    closed_label: Color,

    /// The color for labels on removed railways.
    removed_label: Color,

    /// The color for labels on gone railways.
    gone_label: Color,

    /// The color for unknown things.
    toxic: Color,
}

impl Colors {
    /// Returns the color for a piece of track.
    pub fn track_color(&self, class: &class::Railway) -> Color {
        use self::class::VoltageGroup::*;
        use self::class::ElectricSystem::*;

        match class.status() {
            class::Status::Closed => self.closed,
            class::Status::Removed | class::Status::Explanned => self.removed,
            class::Status::Gone => self.gone,
            class::Status::Open | class::Status::Planned => {
                if matches!(class.category(), class::Category::Tram) {
                    self.tram
                }
                else if let Some(cat) = class.active_cat() {
                    match (cat.system, cat.voltage_group()) {
                        (Some(Ac), High) => self.ac_high,
                        (Some(Ac), Low) => self.ac_low,
                        (Some(Dc), High) => self.dc_high,
                        (Some(Dc), Low) => self.dc_low,
                        _ => self.toxic,
                    }
                }
                else if let Some(rail) = class.active_rail() {
                    if rail.fourth {
                        self.four
                    }
                    else {
                        self.rail
                    }
                }
                else {
                    self.none
                }
            }
        }
    }

    /// Returns the color for cat markings if they should be drawn.
    pub fn cat_color(&self, class: &class::Railway) -> Option<Color> {
        use self::class::VoltageGroup::*;
        use self::class::ElectricSystem::*;

        class.cat().and_then(|cat| {
            if class.category().is_tram() {
                return match cat.status {
                    class::ElectricStatus::None => None,
                    class::ElectricStatus::Open => Some(self.tram),
                    class::ElectricStatus::Removed => Some(self.removed),
                }
            }

            match cat.status {
                class::ElectricStatus::None => None,
                class::ElectricStatus::Open => {
                    Some(match (cat.system, cat.voltage_group()) {
                        (Some(Ac), High) => self.ac_high,
                        (Some(Ac), Low) => self.ac_low,
                        (Some(Dc), High) => self.dc_high,
                        (Some(Dc), Low) => self.dc_low,
                        _ => self.toxic,
                    })
                }
                class::ElectricStatus::Removed => Some(self.removed),
            }
        })
    }

    /// Returns the color for third rail markings if they should be drawn.
    pub fn rail_color(&self, class: &class::Railway) -> Option<Color> {
        class.rail().and_then(|rail| {
            if class.category().is_tram() {
                match rail.status {
                    class::ElectricStatus::None => None,
                    class::ElectricStatus::Open => Some(self.tram),
                    class::ElectricStatus::Removed => Some(self.removed),
                }
            }
            else {
                match rail.status {
                    class::ElectricStatus::None => None,
                    class::ElectricStatus::Open => {
                        if rail.fourth {
                            Some(self.four)
                        }
                        else {
                            Some(self.rail)
                        }
                    }
                    class::ElectricStatus::Removed => Some(self.removed),
                }
            }
        })
    }

    /// Returns the color for a station label.
    pub fn label_color(&self, class: &class::Railway) -> Color {
        match class.status() {
            class::Status::Closed => self.closed_label,
            class::Status::Removed | class::Status::Explanned => {
                self.removed_label
            }
            class::Status::Gone => self.gone_label,
            _ => self.track_color(class)
        }
    }

    /// Returns the primary color for a marker.
    pub fn primary_marker_color(&self, class: &class::Railway) -> Color {
        self.track_color(class)
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            none: Color::hex(NONE).unwrap(),
            ac_high: Color::hex(AC_HIGH).unwrap(),
            ac_low: Color::hex(AC_LOW).unwrap(),
            dc_high: Color::hex(DC_HIGH).unwrap(),
            dc_low: Color::hex(DC_LOW).unwrap(),
            rail: Color::hex(RAIL).unwrap(),
            four: Color::hex(FOUR).unwrap(),
            tram: Color::hex(TRAM).unwrap(),
            closed:  Color::grey(0.550),
            removed: Color::grey(0.650),
            gone:    Color::grey(0.850),
            closed_label:  Color::grey(0.250),
            removed_label: Color::grey(0.350),
            gone_label:    Color::grey(0.450),
            toxic:   Color::rgb(0.824, 0.824, 0.0),
        }
    }
}


//------------ Color Constants -----------------------------------------------

const NONE: &str = "5a3a29ff";
const AC_HIGH: &str  = "6c2b86ff";
const AC_LOW: &str = "812c5cff";
const DC_HIGH: &str = "9b2321ff";
const DC_LOW: &str = "b64f0dff";
const RAIL: &str = "007e40ff";
const FOUR: &str = "53633bff";
const TRAM: &str = "005387ff";

