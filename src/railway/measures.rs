//! Tweakable measures.
//!
//! This module contains various sets of definitions of measures used when
//! rendering the map. The actual set can be chosen from these based on the
//! detail and zoom levels of the map.

use std::ops;
use super::class::Railway;


//------------ Measures ------------------------------------------------------

/// A set of measures.
///
/// Under the hood, this is just a `f64` array. The individual measures can
/// be addressed via the associated constants.
#[derive(Clone, Copy, Debug)]
pub struct Measures([f64; Self::LEN]);

/// # Primary measures
impl Measures {
    /// A Postscript point.
    ///
    /// This is the basis for all other distance measures is 1. in
    /// unscaled measures.
    pub const fn bp(self) -> f64 {
        1.
    }

    /// Nominally the distance between the centre of two parallel main tracks.
    ///
    /// Used as a small distance unit all over the place.
    pub const fn dt(self) -> f64 {
        self.0[0]
    }

    /// The stroke width of a main track.
    pub const fn main_track(self) -> f64 {
        self.0[1]
    }

    /// The stroke width of a double main track.
    ///
    /// Not used in detail 4 and above.
    pub const fn main_double(self) -> f64 {
        self.0[2]
    }

    /// The empty space between two parallel main tracks.
    pub const fn main_skip(self) -> f64 {
        self.0[3]
    }

    /// The stroke width of a light track.
    pub const fn light_track(self) -> f64 {
        self.0[4]
    }

    /// The stroke width of a double light track.
    ///
    /// Not used in detail 4 and above.
    pub const fn light_double(self) -> f64 {
        self.0[5]
    }

    /// The empty space between two parallel light tracks.
    pub const fn light_skip(self) -> f64 {
        self.0[6]
    }

    /// The stroke width of a guiding line.
    pub const fn guide_width(self) -> f64 {
        self.0[7]
    }

    /// The stroke width of the border line.
    pub const fn border_width(self) -> f64 {
        self.0[8]
    }

    /// The base length of a segment of line markings.
    pub const fn seg(self) -> f64 {
        self.0[9]
    }

    /// The width of a station symbol.
    pub const fn station_width(self) -> f64 {
        self.0[10]
    }

    /// The height of a station symbol.
    pub const fn station_height(self) -> f64 {
        self.0[11]
    }

    /// The height of a station symbol.
    pub const fn inside_station_height(self) -> f64 {
        self.0[12]
    }

    /// The font size of the xsmall font.
    pub const fn xsmall_font(self) -> f64 {
        self.0[13]
    }

    /// The font size of the small font.
    pub const fn small_font(self) -> f64 {
        self.0[14]
    }

    /// The font size of the medium font.
    pub const fn medium_font(self) -> f64 {
        self.0[15]
    }

    /// The font size of the large font.
    pub const fn large_font(self) -> f64 {
        self.0[16]
    }

    /// The font size of the extra large font.
    pub const fn xlarge_font(self) -> f64 {
        self.0[17]
    }
    
    /// The font size of the badge font.
    pub const fn badge_font(self) -> f64 {
       self.0[18]
    }

    /// The number of measures.
    const LEN: usize = 19;
}


/// # Derived measures
impl Measures {
    /// The length of a cross-over between two parallel tracks.
    pub fn dl(self) -> f64 {
        self.dt() * 2./3.
    }

    pub fn main_offset(self) -> f64 {
        self.main_track() + self.main_skip()
    }

    pub fn light_offset(self) -> f64 {
        self.light_track() + self.light_skip()
    }

    pub const fn sw(self) -> f64 {
        self.station_width()
    }

    pub const fn sh(self) -> f64 {
        self.station_height()
    }

    pub const fn insh(self) -> f64 {
        self.inside_station_height()
    }
}

/// # Other measures
impl Measures {
    /// Returns the import map units array.
    pub fn map_units(self) -> [f64; 14] {
        [
            self.bp(),
            self.dt(),
            self.dl(),
            self.main_track(),
            self.main_double(),
            self.main_skip(),
            self.main_offset(),
            self.light_track(),
            self.light_double(),
            self.light_skip(),
            self.light_offset(),
            self.station_width(),
            self.station_height(),
            self.inside_station_height(),
        ]
    }

    /// The deprecated sp measure.
    pub fn sp(self) -> f64 {
        self.light_track()
    }

    /// Returns whether we should use main or side track widths.
    fn is_main(class: &Railway) -> bool {
        class.category().is_main() && !class.gauge_group().is_narrow()
    }

    /// Returns the line width for the given class.
    pub fn class_track(self, class: &Railway) -> f64 {
        if Self::is_main(class) {
            self.main_track()
        }
        else {
            self.light_track()
        }
    }

    /// Returns the double track width for the given class.
    pub fn class_double(self, class: &Railway) -> f64 {
        if Self::is_main(class) {
            self.main_double()
        }
        else {
            self.light_double()
        }
    }

    /// Returns the skip for the given class.
    pub fn class_skip(self, class: &Railway) -> f64 {
        if Self::is_main(class) {
            self.main_skip()
        }
        else {
            self.light_skip()
        }
    }

    /// Returns the offset for the given class.
    pub fn class_offset(self, class: &Railway) -> f64 {
        if Self::is_main(class) {
            self.main_offset()
        }
        else {
            self.light_offset()
        }
    }
}


//--- Index

impl ops::Index<usize> for Measures {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}


//--- Mul, MulAssign

impl ops::Mul<f64> for Measures {
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self *= rhs;
        self
    }
}

impl ops::MulAssign<f64> for Measures {
    fn mul_assign(&mut self, rhs: f64) {
        (&mut self.0).into_iter().for_each(|value| *value *= rhs);
    }
}


//------------ Basic Set for Detail Level 0 ----------------------------------

/// The standard measures for detail level 0.
pub const BASE_D0: Measures = Measures([
    2.0,    // dt
    1.1,    // main track
    1.6,    // main double
    0.4,    // main skip
    0.7,    // light track
    1.2,    // light double
    0.3,    // light skip
    0.3,    // guide width
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw               = 3dt
    5.5,    // sh
    3.7,    // insh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 1.
pub const BASE_D1: Measures = BASE_D0;

/// The standard measures for detail level 2.
pub const BASE_D2: Measures = Measures([
    2.0,    // dt
    1.1,    // main track
    1.8,    // main double
    0.4,    // main skip
    0.7,    // light track
    1.4,    // light double
    0.3,    // light skip
    0.3,    // guide width
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw               = 3dt
    5.5,    // sh
    5.5,    // insh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 3.
pub const BASE_D3: Measures = Measures([
    1.8,    // dt
    1.4,    // main track
    2.6,    // main double
    0.4,    // main skip
    1.0,    // light track
    1.8,    // light double
    0.3,    // light skip
    0.3,    // guide width
    0.4,    // border width
    10.8,   // seg              = 6dt
    4.,     // sw
    4.,     // sh
    2.7,    // insh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 4.
pub const BASE_D4: Measures = Measures([
    2.0,    // dt
    1.1,    // main track
    1.1,    // main double
    0.9,    // main skip
    0.6,    // light track
    0.6,    // light double
    1.4,    // light skip
    0.3,    // guide width
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw
    6.,     // sh
    4.,     // insh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 4.
pub const BASE_D5: Measures = Measures([
    2.0,    // dt
    1.2,    // main track
    1.2,    // main double
    1.4,    // main skip
    0.8,    // light track
    0.8,    // light double
    1.2,    // light skip
    0.3,    // guide width
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw
    6.,     // sh
    4.,     // insh
    5.5,    // xsmall font
    6.25,   // small font
    7.5,    // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 5.
pub const BASE_D6: Measures = Measures([
    2.0,    // dt
    1.2,    // main track
    1.1,    // main double
    3.8,    // main skip
    1.0,    // light track
    0.6,    // light double
    1.4,    // light skip
    0.3,    // guide width
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw
    6.,     // sh
    4.,     // insh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

