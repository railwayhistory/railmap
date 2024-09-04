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

/// # Access to the individual measures
impl Measures {
    /// A Postscript point.
    ///
    /// This is the basis for all other distance measures and should be 1 in
    /// unscaled measures.
    pub const fn bp(self) -> f64 {
        self.0[0]
    }

    /// The distance between two parallel tracks.
    pub const fn dt(self) -> f64 {
        self.0[1]
    }

    /// The empty space between two parallel tracks.
    pub const fn ds(self) -> f64 {
        self.0[2]
    }

    /// The length of a cross-over between two parallel tracks.
    pub const fn dl(self) -> f64 {
        self.0[3]
    }

    /// The stroke width of a main line track.
    pub const fn main_width(self) -> f64 {
        self.0[4]
    }

    /// The stroke width of the inside of a main line track.
    pub const fn main_inside(self) -> f64 {
        self.0[5]
    }

    /// The stroke width of the casing of a main line track.
    pub const fn main_case(self) -> f64 {
        self.0[6]
    }

    /// The stroke width of the a double main line track.
    pub const fn double_width(self) -> f64 {
        self.0[7]
    }

    /// The stroke width of the inside of a double main line track.
    pub const fn double_inside(self) -> f64 {
        self.0[8]
    }

    /// The stroke width of the casing of a double main line track.
    pub const fn double_case(self) -> f64 {
        self.0[9]
    }

    /// The stroke width of a side track.
    pub const fn side_width(self) -> f64 {
        self.0[10]
    }

    /// The stroke width of the inside of a side track.
    pub const fn side_inside(self) -> f64 {
        self.0[11]
    }

    /// The stroke width of the casing of a side track.
    pub const fn side_case(self) -> f64 {
        self.0[12]
    }

    /// The stroke width of a guiding  line.
    pub const fn guide_width(self) -> f64 {
        self.0[13]
    }

    /// The stroke width of the casing of a guiding line.
    pub const fn guide_case(self) -> f64 {
        self.0[14]
    }

    /// The stroke width of the border line.
    pub const fn border_width(self) -> f64 {
        self.0[15]
    }

    /// The base length of a segment of line markings.
    pub const fn seg(self) -> f64 {
        self.0[16]
    }

    /// The width of a station symbol.
    pub const fn sw(self) -> f64 {
        self.0[17]
    }

    /// The height of a station symbol.
    pub const fn sh(self) -> f64 {
        self.0[18]
    }

    /// The font size of the xsmall font.
    pub const fn xsmall_font(self) -> f64 {
        self.0[19]
    }

    /// The font size of the small font.
    pub const fn small_font(self) -> f64 {
        self.0[20]
    }

    /// The font size of the medium font.
    pub const fn medium_font(self) -> f64 {
        self.0[21]
    }

    /// The font size of the large font.
    pub const fn large_font(self) -> f64 {
        self.0[22]
    }

    /// The font size of the extra large font.
    pub const fn xlarge_font(self) -> f64 {
        self.0[23]
    }

    /// The font size of the badge font.
    pub const fn badge_font(self) -> f64 {
       self.0[24]
    }

    /// The number of measures.
    const LEN: usize = 25;
}

/// # Derived measures
impl Measures {
    /// Returns the import map units array.
    pub fn map_units(self) -> [f64; 8] {
        [
            self.bp(),
            self.dt(),
            self.ds(),
            self.dl(),
            self.main_width(), // "st"
            self.double_width(), // "dst"
            self.sw(),
            self.sh(),
        ]
    }

    /// The deprecated sp measure.
    pub fn sp(self) -> f64 {
        self.main_width()
    }

    /// Returns the line width for the given class.
    pub fn line_width(self, class: &Railway) -> f64 {
        if class.category().is_main() {
            self.main_width()
        }
        else {
            self.side_width()
        }
    }

    /// Returns the line inside for the given class.
    pub fn line_inside(self, class: &Railway) -> f64 {
        if class.category().is_main() {
            self.main_inside()
        }
        else {
            self.side_width()
        }
    }

    /// Returns the line casine for the given class.
    pub fn line_case(self, class: &Railway) -> f64 {
        if class.category().is_main() {
            self.main_case()
        }
        else {
            self.side_case()
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
    1.,     // bp
    2.,     // dt
    1.2,    // ds
    1.33,   // dl               = 2/3dt
    0.8,    // main width
    0.5,    // main inside
    1.6,    // main case
    1.4,    // double width
    0.5,    // double inside
    1.6,    // double case
    0.5,    // side width
    0.3,    // side inside
    1.0,    // side case
    0.3,    // guide width
    0.6,    // guide case
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw               = 3dt
    5.5,    // sh
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
    1.,     // bp
    2.,     // dt
    2.,     // ds
    0.9,    // dl               = 2/3dt
    1.1,    // main width
    0.9,    // main inside
    1.6,    // main case
    1.8,    // double width
    0.5,    // double inside
    1.6,    // double case
    0.6,    // side width
    0.4,    // side inside
    1.0,    // side case
    0.3,    // guide width
    0.6,    // guide case
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw               = 3dt
    5.5,    // sh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 3.
pub const BASE_D3: Measures = Measures([
    1.,     // bp
    2.2,    // dt
    0.8,    // ds
    1.5,    // dl
    1.4,    // main width
    0.2,    // main inside
    1.6,    // main case
    2.6,    // double width
    0.4,    // double inside
    1.6,    // double case
    1.0,    // side width
    0.4,    // side inside
    1.0,    // side case
    0.3,    // guide width
    0.6,    // guide case
    0.4,    // border width
    8.,     // seg              = 5dt
    4.,     // sw               = 2dt
    4.,    // sh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

/// The standard measures for detail level 4.
pub const BASE_D4: Measures = Measures([
    1.,     // bp
    2.,     // dt
    0.9,    // ds
    1.33,   // dl               = 2/3dt
    1.1,    // main width
    0.9,    // main inside
    1.6,    // main case
    0.8,    // double width
    0.5,    // double inside
    1.6,    // double case
    0.6,    // side width
    0.4,    // side inside
    1.0,    // side case
    0.3,    // guide width
    0.6,    // guide case
    0.4,    // border width
    12.,    // seg              = 6dt
    6.,     // sw               = 3dt
    6.,     // sh
    5.,     // xsmall font
    6.,     // small font
    7.,     // medium font
    9.,     // large font
    11.,    // xlarge font
    5.4,    // badge font
]);

