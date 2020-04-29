//! Font-related processing.

use crate::canvas::FontFace;
use crate::features::label::Font;
use crate::import::eval::SymbolSet;
use super::colors::Palette;


// Font sizes
//
pub const SIZE_XS: f64 = 5.;
pub const SIZE_S: f64 = 6.;
pub const SIZE_M: f64 = 7.;
pub const SIZE_L: f64 = 9.;
pub const SIZE_XL: f64 = 11.;

pub const SIZE_LINE_BADGE: f64 = 5.5;


pub fn font_from_symbols(symbols: &SymbolSet) -> Font {
    Font::new(
        FontFace::from_symbols(symbols),
        Palette::opt_from_symbols(symbols).map(|pal| pal.stroke),
        if symbols.contains("xsmall") {
            Some(SIZE_XS)
        }
        else if symbols.contains("small") {
            Some(SIZE_S)
        }
        else if symbols.contains("medium") {
            Some(SIZE_M)
        }
        else if symbols.contains("large") {
            Some(SIZE_L)
        }
        else if symbols.contains("xlarge") {
            Some(SIZE_XL)
        }
        else {
            None
        }
    )
}

