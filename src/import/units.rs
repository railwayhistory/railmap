//! The named units we understand.

use femtomap::path::Distance;


//------------ World Distances -----------------------------------------------

/// The list of world distance units.
///
/// These distance units refer to real world distance and need to be scaled
/// before using in a map. We keep world distances in _bp_ as well, so all
/// acutal units need to be translated accordingly.
pub const WORLD_DISTANCES: &[(&str, f64)] = &[
    // A metre.
    ("m",  M),

    // A kilometre.
    ("km", 1000. * M),
];

/// The length of a metre in _bp._
const M: f64 = 1_000. * (72./25.4);

//------------ Map Distances -------------------------------------------------

/// The list of map distance units.
///
/// Because map distances can be different for different styles, they can only
/// be resolved at rendering time. The importer translates the unit into an
/// index into a unit table. This table maps the unit names to the indexes.
/// For each entry it contains the name, the index, and a factor to multiply
/// the value with to get to the index. The latter will allow us to have
/// aliases and keep all absolute units in one slot.
pub const MAP_DISTANCES: &[(&str, usize, f64)] = &[
    // A Postscript point.
    //
    // This is the base unit for all absolute values which have index 0.
    ("bp", 0, 1.),

    // A typographic point.
    //
    // We are using modern DTP point which is equal to 1bp.
    ("pt", 0, 1.),

    // An inch.
    ("in", 0, 72.),

    // Millimetre and centimetre.
    //
    // Note that metres and above are used as world units.
    ("mm", 0, MM),
    ("cm", 0, 10. * MM),

    // The distance between two parallel track lines (“delta track”).
    ("dt", 1, 1.),

    // Length of a cross-over between two parallel tracks (“delta length”).
    //
    // This is typically a bit shorter than 1dt. In higher detail levels you
    // should use world distances instead.
    ("dl", 2, 1.),

    // Stroke width of main line track (“stroke track”).
    ("st", 3, 1.), 

    // Width of the station symbol (“station width”).
    ("sw", 4, 1.),

    // Height of the station symbol (“station height”).
    ("sh", 5, 1.),
];

/// The length of a millimetre in bp.
pub const MM: f64 = 72./25.4;


pub fn dt(value: f64) -> Distance {
    Distance::map(value, 1)
}
