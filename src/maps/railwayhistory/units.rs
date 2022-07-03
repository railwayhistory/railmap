/// The units we understand.

use crate::render::path::{Distance, MapDistance};

// Various canvas units in bp.
pub const BP: f64 = 1.;
pub const CM: f64 = 72./2.54;
pub const IN: f64 = 72.;
pub const MM: f64 = 72./25.4;
pub const PT: f64 = 1.;

pub const DL: f64 = 0.66 * DT;
pub const DT: f64 = 0.75 * MM;
pub const SW: f64 = 3.2 * DT;
pub const SH: f64 = 3.0 * DT;
pub const SSW: f64 = 0.5 * SW;
pub const SSH: f64 = 0.5 * SH;

pub const M: f64 = 1_000. * (72./25.4);
pub const KM: f64 = 1_000. * M;


/// The list of world distance units.
///
/// These distance units refer to real world distance and need to be scaled
/// before using in a map. Nonetheless, _bp_ is the base unit here, too.
pub const WORLD_DISTANCES: &[(&str, f64)] = &[
    ("m",  M),
    ("km", KM),

    ("wl", 30. * M),
];


/// The list of map distance units.
///
/// The list contains a pair of the unit name and a value for each of the
/// 6 distance levels. The base distance unit is a Postscript point,
/// aka _bp._. It is exactly 1/72nd of an inch.
pub const MAP_DISTANCES: &[(&str, [f64; 6])] = &[
    // Real units.
    ("bp", [BP; 6]),
    ("pt", [PT; 6]),
    ("in", [IN; 6]),
    ("mm", [MM; 6]),
    ("cm", [CM; 6]),

    // Relative units.
    ("dt",  [DT, DT, DT, 0.7 * DT, DT, DT]),
    ("dl",  [DL, DL, DL, DL, DL, DL]),
    ("sw",  [SSW, SSW, SSW, SSW, SW, SW]),
    ("sh",  [SSH, SSH, SSH, SSH, SH, SH]),
    ("ssw", [SSW, SSW, SSW, SSW, SW, SW]), // deprecated!
];


pub fn resolve_unit(number: f64, unit_name: &str) -> Option<Distance> {
    for (unit, factor) in WORLD_DISTANCES {
        if unit_name == *unit {
            return Some(Distance::new(
                Some(number * factor), Vec::new(),
            ))
        }
    }
    for (index, (unit, _)) in MAP_DISTANCES.iter().enumerate() {
        if unit_name == *unit {
            return Some(Distance::new(
                None, vec![MapDistance::new(number, index)]
            ))
        }
    }
    None
}

