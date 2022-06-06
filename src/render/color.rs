/// Colors.

use std::fmt;
use std::convert::TryFrom;
use std::num::ParseIntError;
use serde::Deserialize;
use super::canvas::Canvas;


/// A color.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(try_from = "&str")]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64
}

impl Color {
    pub const fn rgb(red: f64, green: f64, blue: f64) -> Self {
        Color { red, green, blue, alpha: 1. }
    }

    pub const fn rgba(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        Color { red, green, blue, alpha }
    }

    pub const fn grey(level: f64) -> Self {
        Color::rgb(level, level, level)
    }

    pub fn hex(mut hex: &str) -> Result<Self, InvalidHexColor> {
        if hex.starts_with('#') {
            hex = &hex[1..];
        }
        let (r, g, b, a) = if hex.len() == 6 {
            (
                u8::from_str_radix(&hex[0..2], 16)?,
                u8::from_str_radix(&hex[2..4], 16)?,
                u8::from_str_radix(&hex[4..6], 16)?,
                0xFF,
            )
        }
        else if hex.len() == 8 {
            (
                u8::from_str_radix(&hex[0..2], 16)?,
                u8::from_str_radix(&hex[2..4], 16)?,
                u8::from_str_radix(&hex[4..6], 16)?,
                u8::from_str_radix(&hex[6..8], 16)?,
            )
        }
        else {
            return Err(InvalidHexColor)
        };
        Ok(Color::rgba(
            r as f64 / 255.,
            g as f64 / 255.,
            b as f64 / 255.,
            a as f64 / 255.,
        ))
    }

    pub fn apply(self, canvas: &Canvas) {
        canvas.set_source_rgba(self.red, self.green, self.blue, self.alpha)
    }

    pub fn with_alpha(self, alpha: f64) -> Self {
        Color { red: self.red, green: self.green, blue: self.blue, alpha }
    }

    pub fn lighten(self, factor: f64) -> Self {
        fn component(x: f64, factor: f64) -> f64 {
            x * factor + 1. - factor
        }

        Color {
            red: component(self.red, factor),
            green: component(self.green, factor),
            blue: component(self.blue, factor),
            alpha: self.alpha
        }
    }
}

impl Color {
    pub const WHITE: Color = Color::rgb(1., 1., 1.);
    pub const BLACK: Color = Color::rgb(0., 0., 0.);
    pub const RED: Color = Color::rgb(1., 0., 0.);
    pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);
}

impl<'a> TryFrom<&'a str> for Color {
    type Error = InvalidHexColor;

    fn try_from(src: &'a str) -> Result<Self, Self::Error> {
        Self::hex(src)
    }
}


//------------ InvalidHexColor -----------------------------------------------

pub struct InvalidHexColor;

impl From<ParseIntError> for InvalidHexColor {
    fn from(_: ParseIntError) -> Self {
        InvalidHexColor
    }
}

impl fmt::Display for InvalidHexColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid color")
    }
}

