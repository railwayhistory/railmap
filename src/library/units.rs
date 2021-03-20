/// The units we understand.

// Various canvas units in bp.
pub const BP: f64 = 1.;
pub const CM: f64 = 72./2.54;
pub const IN: f64 = 72.;
pub const MM: f64 = 72./25.4;
pub const PT: f64 = 1.;

pub const DL: f64 = 0.66 * DT;
pub const DT: f64 = 0.75 * MM;
pub const SW: f64 = 3.2 * DT;
pub const SH: f64 = 3. * DT;
pub const SSW: f64 = 1.6 * DT;

pub const M: f64 = 1_000. * (72./25.4);
pub const KM: f64 = 1_000. * M;

/// The list of canvas distance units.
/// 
/// The list contains pairs of the unit’s name and how many base units fit
/// into it. The base distance unit is a Postscript point, aka _bp._. It is
/// exactly 1/72nd of an inch.
pub const CANVAS_DISTANCES: &[(&str, f64)] = &[
    // Real units.
    ("bp", BP),
    ("pt", PT),
    ("in", IN),
    ("mm", MM),
    ("cm", CM),

    // Relative units.
    ("dt", DT), // Distance between tracks. 0.5 mm
    ("dl", DL), // Length of a track crossing between two tracks one dt apart.
    ("sw", SW),
    ("sh", SH),
    ("ssw", SSW),
];


/// The list of world distance units.
///
/// These distance units refer to real world distance and need to be scaled
/// before using in a map. Nonetheless, _bp_ is the base unit here, too.
pub const WORLD_DISTANCES: &[(&str, f64)] = &[
    ("m",  M),
    ("km", KM),

    ("wl", 30. * M),
];


/// The size of the equator in _bp._
pub const EQUATOR_BP: f64 = 40075.016686 * KM;

