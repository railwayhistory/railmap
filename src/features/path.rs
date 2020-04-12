/// Paths.

use std::ops;
//use std::convert::TryFrom;
use std::sync::Arc;
use kurbo::{
    BezPath, //CubicBez, ParamCurve, ParamCurveArclen, ParamCurveDeriv,
    /*PathEl, PathSeg, Point,*/ Rect, //Vec2
};
use crate::canvas::Canvas;


//------------ Configuration Constants ---------------------------------------

/*
/// Accuracy for Kurbo arclen calculations in storage coordinates.
///
/// This value should provide centimetre accuracy in storage coordinates.
const STORAGE_ACCURACY: f64 = 1E-10;

/// Accuracy for Kurbo arclen calculations in canvas coordinates.
///
/// This value assumes about 192 dpi device resolution.
const CANVAS_ACCURACY: f64 = 0.25;
*/


//------------ Path ----------------------------------------------------------

/// A path.
#[derive(Clone, Debug)]
pub struct Path {
    first: Segment,
    others: Vec<(f64, f64, Segment)>,
}

impl Path {
    pub fn new(first: Segment) -> Self {
        Path { first, others: Vec::new() }
    }

    pub fn push(&mut self, post: f64, pre: f64, segment: Segment) {
        self.others.push((post, pre, segment))
    }

    pub fn apply(&self, _canvas: &Canvas) {
        unimplemented!()
    }

    pub fn bounding_box(&self) -> Rect {
        unimplemented!()
    }
}


//------------ BasePath ------------------------------------------------------

/// A basic path.
#[derive(Clone, Debug)]
pub struct BasePath(Arc<BezPath>);

impl BasePath {
    pub fn new(path: BezPath) -> Self {
        BasePath(Arc::new(path))
    }

    /*
    /// Returns the segment ending at `idx`.
    ///
    /// In other words, the segment will cover the time values between
    /// `idx - 1` and `idx`.
    fn get_seg(&self, idx: u32) -> Option<CubicBez> {
        let idx = usize::try_from(idx).ok()?;
        match self.0.get_seg(idx)? {
            PathSeg::Cubic(seg) => Some(seg),
            _ => unreachable!()
        }
    }

    /// Returns the maximum time value on the path.
    fn max_time(&self) -> f64 {
        f64::from(u32::try_from(self.0.elements().len()).unwrap() - 1)
    }

    pub fn node(&self, at: u32) -> Option<Point> {
        if at == 0 {
            self.get_seg(1).map(|seg| seg.p0)
        }
        else {
            self.get_seg(at).map(|seg| seg.p1)
        }
    }

    /// Returns the time `distance` away from the given node.
    ///
    /// If `distance` is positive, the time is calculated towards the next
    /// node, otherwise towards the previous node.
    ///
    /// This method uses storage accuracy.
    pub fn arctime_node(&self, node: u32, distance: f64) -> f64 {
        if distance < 0. {
            if node == 0 {
                return 0.
            }
            let seg = match self.get_seg(node) {
                Some(seg) => seg,
                None => return self.max_time()
            };
            let seg = CubicBez::new(seg.p3, seg.p2, seg.p1, seg.p0);
            f64::from(node) - seg.inv_arclen(-distance, STORAGE_ACCURACY)
        }
        else {
            let seg = match self.get_seg(node + 1) {
                Some(seg) => seg,
                None => return self.max_time()
            };
            f64::from(node) + seg.inv_arclen(distance, STORAGE_ACCURACY)
        }
    }

    pub fn apply_start(&self, canvas: &Canvas) {
        match self.0.iter().next().unwrap() {
            PathEl::MoveTo(p) => {
                let p = canvas.transform() * p;
                canvas.move_to(p.x, p.y);
            }
            _ => unreachable!()
        }
    }

    pub fn apply_tail(&self, canvas: &Canvas) {
        for item in self.0.iter().skip(1) {
            match item {
                PathEl::CurveTo(p1, p2, p3) => {
                    let p1 = canvas.transform() * p1;
                    let p2 = canvas.transform() * p2;
                    let p3 = canvas.transform() * p3;
                    canvas.curve_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
                }
                _ => unreachable!()
            }
        }
    }
    */
}


//------------ Segment -------------------------------------------------------


/// A segment in a constructed path.
#[derive(Clone, Debug)]
pub enum Segment {
    Subpath(Subpath),
    Point(Position),
}

impl Segment {
    pub fn eval_full(_path: BasePath, _offset: Option<Distance>) -> Self {
        unimplemented!()
    }

    pub fn eval_subpath(
        _path: BasePath,
        _start_node: u32, _start_distance: Distance,
        _end_time: u32, _end_distance: Distance,
        _offset: Option<Distance>
    ) -> Self {
        unimplemented!()
    }

    pub fn eval_point(
        _path: BasePath,
        _node: u32, _distance: Distance,
        _offset: Option<Distance>
    ) -> Self {
        unimplemented!()
    }

    /*
    pub fn apply_start(&self, canvas: &Canvas) {
        match *self {
            Segment::Subpath(ref path) => path.apply_start(canvas),
            Segment::Point(ref path) => path.apply_start(canvas),
        }
    }

    pub fn apply_tail(&self, canvas: &Canvas) {
        match *self {
            Segment::Subpath(ref path) => path.apply_tail(canvas),
            Segment::Point(_) => { }
        }
    }
    */
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
    path: BasePath,

    /// The start location on `path`.
    start: Location,

    /// The end location on `path`.
    end: Location,

    /// Offset from the original path.
    ///
    /// Given in canvas coordinates. Positive values are to the left of the
    /// path. I.e., this is the length of a tangent vector rotated 90°.
    offset: Option<Distance>,
}

impl Subpath {
    pub fn new(
        path: BasePath,
        start: Location, end: Location,
        offset: Option<Distance>
    ) -> Self {
        Subpath { path, start, end, offset }
    }

    /*
    pub fn apply_start(&self, canvas: &Canvas) {
        let p = self.start_point(canvas);
        canvas.move_to(p.x, p.y)
    }

    pub fn apply_tail(&self, _canvas: &Canvas) {
        unimplemented!()
    }

    fn resolve_start(&self, _canvas: &Canvas) -> (u32, f64) {
        unimplemented!()
    }

    fn start_point(&self, canvas: &Canvas) -> Point {
        let (seg, time) = self.resolve_start(canvas);
        let seg = self.path.get_seg(seg).unwrap();
        let point = canvas.transform() * seg.eval(time);
        match self.offset {
            Some(offset) => {
                let tangent = seg.deriv().eval(time).to_vec2().normalize();
                // rotate tangent by -90° and scale by offset.
                point + Vec2::new(
                    tangent.y * offset, -tangent.x * offset
                )
            }
            None => {
                point
            }
        }
    }

    fn resolve_end(&self, _canvas: &Canvas) -> (u32, f64) {
        unimplemented!()
    }
    */
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
    path: BasePath,

    /// The location of the position on the path.
    location: Location,

    /// Offset from the original path.
    ///
    /// Given in canvas coordinates. Positive values are to the left of the
    /// path. I.e., this is the length of a tangent vector rotated 90°.
    offset: Option<f64>
}

impl Position {
    pub fn new(
        path: BasePath, location: Location, offset: Option<f64>
    ) -> Self {
        Position { path, location, offset }
    }

    pub fn apply_start(&self, _canvas: &Canvas) {
        unimplemented!()
    }
}


//------------ Distance ------------------------------------------------------

/// Describes a distance from a point.
///
/// In feature definitions, locations on paths are defined relative to named
/// points on the path. They are described as a distance from well-defined
/// points which is combined from a world distance and a map distance. Thus
/// way we can create schematic representations that are pleasing at a range
/// of scales.
#[derive(Clone, Copy, Debug, Default)]
pub struct Distance {
    /// The world component of the distance.
    ///
    /// This is not yet scaled to storage coordinates, i.e., this value is the
    /// acutal distance along the face of the Earth in _bp._
    pub world: Option<f64>,

    /// The canvas component of the distance.
    ///
    /// This is the distance along the canvas in _bp._
    pub canvas: Option<f64>,
}

impl Distance {
    /// Creates a new distance from the world and canvas components.
    pub fn new(world: Option<f64>, canvas: Option<f64>) -> Self {
        Distance { world, canvas }
    }
}

impl ops::AddAssign for Distance {
    fn add_assign(&mut self, other: Distance) {
        if let Some(o) = other.world {
            if let Some(s) = self.world.as_mut() {
                *s += o
            }
            else {
                self.world = Some(o)
            }
        }
        if let Some(o) = other.canvas {
            if let Some(s) = self.canvas.as_mut() {
                *s += o
            }
            else {
                self.canvas = Some(o)
            }
        }
    }
}

impl ops::SubAssign for Distance {
    fn sub_assign(&mut self, other: Distance) {
        if let Some(o) = other.world {
            if let Some(s) = self.world.as_mut() {
                *s -= o
            }
            else {
                self.world = Some(-o)
            }
        }
        if let Some(o) = other.canvas {
            if let Some(s) = self.canvas.as_mut() {
                *s -= o
            }
            else {
                self.canvas = Some(-o)
            }
        }
    }
}

impl ops::Neg for Distance {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Distance {
            world: self.world.map(|val| -val),
            canvas: self.canvas.map(|val| -val),
        }
    }
}


//------------ Location ------------------------------------------------------

/// Description of a location.
///
/// Points on a path are referenced via a floating point value called time.
/// Its integer part corresponds with the node index of the starting point of
/// the segment while the fractional part describes how far between start and
/// end point of the segment the location can be found.
///
/// Typically, locations are defined as [distances][`Distance`] from a known
/// point on a path specified by its time value. Because distances contain a
/// world and canvas component, we can only calculate the time value of the
/// location during rendering.
///
/// However, since storage coordinates currently are only a scaled value of
/// canvas coordinates, we can calculate the time value for the world distance
/// part of the relative location during evaluation. The map distance part
/// then needs to be added during rendering.
///
/// Thus, the location is described by two values: the time value of the point
/// including relative world distance and the relative distance from that
/// point on the canvas expressed in the standard canvas unit of _bp._ These
/// two values are represented by the fields `world`  and `canvas`
/// respectively.
///
/// [`Distance`]: struct.Distance.html
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

