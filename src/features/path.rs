/// Paths.

use std::sync::Arc;
use kurbo::{BezPath, PathEl, Rect, Shape};
use crate::canvas::Canvas;


//------------ Path ----------------------------------------------------------

/// A path.
#[derive(Clone, Debug)]
pub enum Path {
    Simple(Arc<BezPath>),
    Constructed(Vec<Subpath>),
}

impl Path {
    pub fn apply(&self, canvas: &Canvas) {
        match *self {
            Path::Simple(ref path) => {
                for el in path.iter() {
                    match el {
                        PathEl::MoveTo(p) => {
                            let p = canvas.transform() * p;
                            canvas.move_to(p.x, p.y);
                        }
                        PathEl::CurveTo(p0, p1, p2) => {
                            let p0 = canvas.transform() * p0;
                            let p1 = canvas.transform() * p1;
                            let p2 = canvas.transform() * p2;
                            canvas.curve_to(
                                p0.x, p0.y, p1.x, p1.y, p2.x, p2.y
                            );
                        }
                        _ => unreachable!()
                    }
                }
            }
            Path::Constructed(_) => {
                unimplemented!()
            }
        }
    }

    pub fn bounding_box(&self) -> Rect {
        match *self {
            Path::Simple(ref path) => path.bounding_box(),
            Path::Constructed(_) => {
                unimplemented!()
            }
        }
    }
}


//------------ Subpath -------------------------------------------------------

/// Part of a path.
///
/// This type renders part of a referenced path described by the start end
/// end [locations][`Location`] on that path.
///
/// [`Location`]: struct.Location.html
#[derive(Clone, Debug)]
pub struct Subpath {
    /// The base path.
    path: Path,

    /// The start location on `path`.
    start: Location,

    /// The end location on `path`.
    end: Location,

    /// The tension left of the start.
    start_tension: f64,

    /// The tension right of the end.
    end_tension: f64,

    /// Offset from the original path.
    ///
    /// Given in canvas coordinates. Positive values are to the right of the
    /// path as seen ‘in the direction of travel.’
    offset: f64,
}

impl Subpath {
    pub fn apply(&self, _canvas: &Canvas) {
        unimplemented!()
    }

    pub fn bounding_box(&self) -> Rect {
        unimplemented!()
    }
}


//------------ Position ------------------------------------------------------

/// A point and direction derived from a path.
///
/// A position is given by choosing a point along a path, specified via a
/// [location][`Location`]. The position will be located at that point and
/// its direction is the tangent at the point that is facing towards growing
/// time values.
///
/// [`Location`]: struct.Location.html
#[derive(Clone, Debug)]
pub struct Position {
    /// The base path.
    path: Path,

    /// The location of the position on the path.
    location: Location,

    /// Offset from the original path.
    ///
    /// Given in canvas coordinates. Positive values are to the right of the
    /// path as seen ‘in the direction of travel.’
    offset: f64
}


//------------ Location ------------------------------------------------------

/// Description of a location.
///
/// Points on a path are referenced via a floating point value called time.
/// Its integer part corresponds with the node index of the starting point of
/// the segment while the fractional part describes how far between start and
/// end point of the segment the location can be found.
///
/// In the feature definition, locations are defined relative to named points
/// on the path. These points are always nodes on the path, so they have a
/// integral time value. The relative location is described as a distance from
/// that point and is combined from a world distance and a map distance. This
/// way we can create schematic representations that are pleasing at a range
/// of scales.
///
/// Since storage coordinates currently are only a scaled value of canvas
/// coordinates, we can calculate the time value for the world distance part
/// of the relative location during compilation. The map distance part then
/// needs to be added during rendering.
///
/// Thus, the location is described by two values: the time value of the point
/// including relative world distance and the relative distance from that
/// point on the canvas expressed in the standard canvas unit of _bp._ These
/// two values are represented by the fields `world`  and `canvas`
/// respectively.
#[derive(Clone, Copy, Debug)]
pub struct Location {
    /// The time value of the world location.
    pub world: f64,

    /// The distance from the time value on the canvas.
    ///
    /// Positive values are further along the path, negative values are
    /// backwards on the path.
    pub canvas: f64,
}

