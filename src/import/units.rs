/// The units we understand.

// Various canvas units in bp.
pub const MM: f64 = 72./25.4;
pub const DT: f64 = 0.75 * MM;
pub const PT: f64 = 1.;

/// The list of canvas distance units.
/// 
/// The list contains pairs of the unit’s name and how many base units fit
/// into it. The base distance unit is a Postscript point, aka _bp._. It is
/// exactly 1/72nd of an inch.
pub const CANVAS_DISTANCES: &[(&str, f64)] = &[
    // Real units.
    ("bp", 1.),
    ("in", 72.),
    ("mm", 72./25.4),
    ("cm", 72./2.54),

    // Relative units.
    ("dt", 0.75 * (72./25.4)), // Distance between tracks. 0.5 mm
    ("dl", 0.5 * (72./25.4)),  // Length of a track crossing between two
                               // tracks one dt apart.
    ("sw", 2.25 * MM),
];


/// The list of world distance units.
///
/// These distance units refer to real world distance and need to be scaled
/// before using in a map. Nonetheless, _bp_ is the base unit here, too.
pub const WORLD_DISTANCES: &[(&str, f64)] = &[
    ("m",  1_000. * (72./25.4)),
    ("km", 1_000_000. * (72./25.4)),
];


/// The size of the equator in _bp._
///
//  The equator is 40075.016686 km long, so in bp that’s:
pub const EQUATOR_BP: f64 = 40075.016686 * 1_000_000. * (72./25.4);
