/// The units we understand.

use femtomap::path::{Distance, MapDistance, MapDistanceVec};

// Various canvas units in bp.
/*
pub const BP: f64 = 1.;
pub const CM: f64 = 72./2.54;
pub const IN: f64 = 72.;
pub const PT: f64 = 1.;
*/
pub const MM: f64 = 72./25.4;

pub const DL: f64 = 0.66 * DT;
pub const DT: f64 = 0.75 * MM;
pub const SW: f64 = 3.2 * DT;
pub const SH: f64 = 3.0 * DT;
pub const S3W: f64 = 0.5 * SW;
pub const S3H: f64 = 0.6 * SH;
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

/// Absolute map distance units.
///
/// These units represent the same value no matter the detail level. Thus,
/// we can convert them into _bp_ which will be map distance unit 0.
const ABSOLUTE_MAP_DISTANCES: &[(&str, f64)] = &[
    ("bp", 1.),
    ("pt", 1.),
    ("in", 72.),
    ("mm", 72./25.4),
    ("cm", 72./2.54),
];


/// The list of map distance units.
///
/// The list contains a pair of the unit name and a value for each of the
/// 6 distance levels. The base distance unit is a Postscript point,
/// aka _bp._. It is exactly 1/72nd of an inch.
pub const MAP_DISTANCES: &[(&str, [f64; 6])] = &[
    // Relative units.
    ("bp",  [1., 1., 1., 1., 1., 1.,]),
    ("dt",  [DT, DT, DT, 0.8 * DT, DT, DT]),
    ("dl",  [DL, DL, DL, 0.8 * DL, DL, DL]),
    ("sw",  [SSW, SSW, SSW, S3W, SW, SW]),
    ("sh",  [SSH, SSH, SSH, S3H, SH, SH]),
    ("ssw", [SSW, SSW, SSW, S3W, SW, SW]), // deprecated!
];


pub fn resolve_unit(
    number: f64, unit_name: &str,
) -> Option<Distance> {
    for (unit, factor) in WORLD_DISTANCES {
        if unit_name == *unit {
            return Some(Distance::new(
                Some(number * factor), MapDistanceVec::new(),
            ))
        }
    }
    for (unit, factor) in ABSOLUTE_MAP_DISTANCES.iter() {
        if unit_name == *unit {
            let mut map = MapDistanceVec::new();
            map.push(MapDistance::new(number * factor, 0));
            return Some(Distance::new(None, map))
        }
    }

    for (index, (unit, _)) in MAP_DISTANCES.iter().enumerate() {
        if unit_name == *unit {
            let mut map = MapDistanceVec::new();
            map.push(MapDistance::new(number, index));
            return Some(Distance::new(None, map))
        }
    }
    None
}

