//! The coloring rules for the passenger map.

use femtomap::render::Color;
use crate::class;

//------------ Colors --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Colors {
    full: Color,
    //ltd: Color,
    none: Color,
    closed: Color,
}

impl Colors {
    /// Returns the color for a piece of track.
    pub fn track_color(&self, class: &class::Railway) -> Color {
        if !class.is_open() {
            self.closed
        }
        else {
            match class.pax() {
                class::Pax::None => self.none,
                _ => self.full,
            }
        }
    }

    /// Returns the color for cat markings if they should be drawn.
    pub fn cat_color(&self, class: &class::Railway) -> Option<Color> {
        class.cat().map(|cat| {
            match cat.status {
                class::ElectricStatus::Open => self.track_color(class),
                class::ElectricStatus::Removed => self.closed,
            }
        })
    }

    /// Returns the color for third rail markings if they should be drawn.
    pub fn rail_color(&self, class: &class::Railway) -> Option<Color> {
        class.rail().map(|rail| {
            match rail.status {
                class::ElectricStatus::Open => self.track_color(class),
                class::ElectricStatus::Removed => self.closed,
            }
        })
    }

    /// Returns the color for a station label.
    pub fn label_color(&self, class: &class::Railway) -> Color {
        self.track_color(class)
    }

    /// Returns the primary color for a marker.
    pub fn primary_marker_color(&self, class: &class::Railway) -> Color {
        self.track_color(class)
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            full: Color::grey(0.1),
            //ltd: Color::grey(0.3),
            none: Color::grey(0.7),
            closed: Color::grey(0.9),
        }
    }
}

