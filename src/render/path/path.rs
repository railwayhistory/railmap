//! Paths.

use std::{cmp, ops};
use std::f64::consts::PI;
use std::sync::Arc;
use kurbo::{CubicBez, Line, ParamCurveArclen, Point};
use crate::theme::Style;
use super::super::canvas::Canvas;
use super::trace::{STORAGE_ACCURACY, Segment};


//----------- Constants ------------------------------------------------------

const M: f64 = 1_000. * (72./25.4);
const KM: f64 = 1_000. * M;
const EQUATOR_BP: f64 = 40075.016686 * KM;


//------------ Path ----------------------------------------------------------

/// An unmodified, imported path.
///
/// Paths form the basis of the definition of all features: They are
/// referenced when placing or tracing features.
///
/// Paths are shared objects: their data is kept behind an arc. They
/// can be cloned cheaply for passing around.
#[derive(Clone, Debug)]
pub struct Path {
    /// The elements of the stored path.
    ///
    /// For simplicity, we keep the initial point as an element, too.
    elements: Arc<Box<[Element]>>,
}

impl Path {
    /// Creates a builder for a path starting a the given point.
    pub fn builder(move_to: Point) -> PathBuilder {
        PathBuilder::new(move_to)
    }

    /// Returns the minimum valid location on the path.
    pub fn min_location(&self) -> Location {
        Location::new(SegTime::new(1, 0.), Vec::new())
    }

    /// Returns the last valid location of the path.
    pub fn max_location(&self) -> Location {
        Location::new(
            SegTime::new(self.elements.len() as u32 - 1, 1.),
            Vec::new(),
        )
    }

    /// Returns the number of nodes in the path.
    pub fn node_len(&self) -> u32 {
        self.elements.len() as u32
    }

    /// Returns that ends at the given node index.
    pub fn segment(&self, idx: u32) -> Option<Segment> {
        let idx = idx as usize;
        if idx == 0 || idx >= self.elements.len() {
            return None
        }
        let p0 = self.elements[idx - 1].point;
        let el = &self.elements[idx];
        match el.controls {
            Some((p1, p2)) => {
                Some(Segment::curve(p0, p1, p2, el.point, Some(el.arclen)))
            }
            None => Some(Segment::line(p0, el.point, Some(el.arclen)))
        }
    }

    /// Returns the partial segment before the given location.
    ///
    /// The returned segment will start at the node before the location and
    /// run to the location.
    ///
    /// # Panics
    ///
    /// The method panics if `loc` doesn’t describe a valid place on the path.
    pub fn segment_before(&self, loc: SegTime) -> Segment {
        self.segment(loc.seg).unwrap().sub(0., loc.time)
    }

    /// Returns the partial segment before the given location.
    ///
    /// The returned segment will start at the node before the location and
    /// run to the location.
    ///
    /// # Panics
    ///
    /// The method panics if `loc` doesn’t describe a valid place on the path.
    pub fn segment_after(&self, loc: SegTime) -> Segment {
        self.segment(loc.seg).unwrap().sub(loc.time, 1.)
    }

    /// Returns the location at a distance from a node.
    pub fn location(&self, mut node: u32, distance: Distance) -> Location {
        // `node` is the index of the referenced node. We need to convert
        // this to the segment index which is the index of the end node of
        // the segment.
        //
        // First let’s make sure there aren’t going to be any surprises.
        // Nodes are only referenced to by their name, we convert them into
        // indexes so there shouldn’t be an invalid index.
        assert!(node < self.node_len());

        match distance.world {
            None => {
                // Let’s do the simple case first. If we don’t have a world
                // component in `distance`, we only need to convert the node
                // index into a segment index.
                if node < self.node_len() - 1 {
                    Location::new(SegTime::new(node + 1, 0.), distance.map)
                }
                else {
                    Location::new(SegTime::new(node, 1.), distance.map)
                }
            }
            Some(world) if world < 0. => {
                // We have a negative world component. We need to go backwards
                // on segments starting with the one that ends at the node
                // index.
                let mut seg = self.segment(node).unwrap();
                let mut storage = to_storage_distance(-world, seg.p3());
                loop {
                    let arclen = seg.arclen_storage();
                    if storage >= arclen {
                        if node == 1 {
                            return self.min_location()
                        }
                        node -= 1;
                        seg = self.segment(node).unwrap();
                        storage -= arclen;
                    }
                    else {
                        return Location::new(
                            SegTime::new(
                                node,
                                1. - seg.rev().arctime_storage(storage)
                            ),
                            distance.map
                        );
                    }
                }
            }
            Some(world) => {
                // We have a positive world component. We need to go forward
                // starting with the segment beginning at the node, i.e,
                // segment node + 1. If we are already on the last segment,
                // we can bail out.
                if node == self.node_len() - 1 {
                    return self.max_location()
                }
                node += 1;
                let mut seg = self.segment(node).unwrap();
                let mut storage = to_storage_distance(world, seg.p0());
                loop {
                    let arclen = seg.arclen_storage();
                    if storage >= arclen {
                        if node == self.node_len() - 1 {
                            return self.max_location()
                        }
                        node += 1;
                        seg = self.segment(node).unwrap();
                        storage -= arclen
                    }
                    else {
                        return Location::new(
                            SegTime::new(
                                node,
                                seg.arctime_storage(storage)
                            ),
                            distance.map
                        );
                    }
                }
            }
        }
    }

    /// Returns the time value for a location on a given canvas.
    pub fn location_time(
        &self, location: &Location, canvas: &Canvas, style: &impl Style,
    ) -> SegTime {
        self._location_time(
            location.world,
            location.map.iter().map(
                |canv| style.resolve_distance(*canv)
            ).sum::<f64>() * canvas.canvas_bp(),
            canvas
        )
    }

    fn _location_time(
        &self, world: SegTime, map: f64, canvas: &Canvas
    ) -> SegTime {
        if map == 0. {
            world
        }
        else if map < 0. {
            let offset = -map;
            let seg = self.segment(
                world.seg
            ).unwrap().transform(canvas);
            let before = seg.sub(0., world.time);
            let arclen = before.arclen();
            if arclen >= offset {
                let len = seg.sub(world.time, 1.).arclen() + offset;
                SegTime::new(
                    world.seg,
                    1. - seg.rev().arctime(len)
                )
            }
            else if world.seg > 1 {
                self._location_time(
                    SegTime::new(world.seg - 1, 1.),
                    -(offset - arclen),
                    canvas
                )
            }
            else {
                SegTime::new(1, 0.)
            }
        }
        else {
            let offset = map;
            let seg = self.segment(
                world.seg
            ).unwrap().transform(canvas);
            let after = seg.sub(world.time, 1.);
            let arclen = after.arclen();
            if arclen > offset {
                let len = seg.sub(0., world.time).arclen() + offset;
                SegTime::new(
                    world.seg, seg.arctime(len)
                )
            }
            else if world.seg == self.node_len() - 1 {
                SegTime::new(world.seg, 1.)
            }
            else {
                self._location_time(
                    SegTime::new(world.seg + 1, 0.),
                    offset - arclen,
                    canvas
                )
            }
        }
    }
}


//------------ PathBuilder ---------------------------------------------------

/// A builder for a stored path.
pub struct PathBuilder {
    elements: Vec<Element>,
}

impl PathBuilder {
    /// Creates a new stored path starting at the given point.
    pub fn new(move_to: Point) -> Self {
        PathBuilder {
            elements: vec![Element::move_to(move_to)]
        }
    }

    pub fn line_to(&mut self, p1: Point) {
        let arclen = Line::new(
            self.elements.last().unwrap().point, p1
        ).arclen(STORAGE_ACCURACY);
        self.elements.push(Element::line_to(p1, arclen))
    }

    /// Adds a curve to a new point.
    pub fn curve_to(&mut self, c0: Point, c1: Point, p1: Point) {
        let p0 = self.elements.last().unwrap().point;
        if p0 == c0 && c1 == p1 {
            let arclen = Line::new(p0, p1).arclen(STORAGE_ACCURACY);
            self.elements.push(Element::line_to(p1, arclen))
        }
        else {
            let arclen = CubicBez::new(
                p0, c0, c1, p1
            ).arclen(STORAGE_ACCURACY);
            self.elements.push(Element::curve_to(c0, c1, p1, arclen))
        }
    }

    /// Finishes the builder and converts it into a stored path.
    pub fn finish(self) -> Path {
        Path {
            elements: Arc::new(self.elements.into_boxed_slice())
        }
    }
}


//------------ Element -------------------------------------------------------

/// A single element of a stored path.
///
/// Each element describes how to progress from the previous node of the path
/// to the next. We distinguish between two types of elements: straight lines
/// and bezier segments. This happens by making the pair of control points
/// optional and keep them at `None` in a straight line.
///
/// We also precompute the arc length of the element since that is somewhat
/// expensive for bezier segments and used quite often.
#[derive(Clone, Copy, Debug)]
struct Element {
    /// The optional control points of the element.
    ///
    /// If this is `None`, the element is a straight line.
    controls: Option<(Point, Point)>,

    /// The point of the destination node of the element.
    point: Point,

    /// The arc length of the segment leading to the point.
    arclen: f64,
}

impl Element {
    fn move_to(point: Point) -> Self {
        Element {
            controls: None,
            point,
            arclen: 0.
        }
    }

    fn line_to(point: Point, arclen: f64) -> Self {
        Element {
            controls: None,
            point,
            arclen
        }
    }

    fn curve_to(c0: Point, c1: Point, point: Point, arclen: f64) -> Self {
        Element {
            controls: Some((c0, c1)),
            point,
            arclen
        }
    }
}


//------------ MapDistance ---------------------------------------------------

/// A distance in map units.
///
/// The distance consists of a value and a unit that scales this value into
/// actual map coordinates. Since the actual factor for each unit may depend
/// on the style, the unit depends on the theme. To avoid having the make the
/// type generic over the unit, we use a simple index as a standin. This
/// index is chosen by the theme when evaluating the map rules and is
/// available to the style when resolving the map distance.
#[derive(Clone, Copy, Debug)]
pub struct MapDistance {
    /// The value of the distance.
    value: f64,

    /// The unit of the distance.
    unit: usize,
}

impl MapDistance {
    /// Creates a new map distance from a value and a unit index.
    pub fn new(value: f64, unit: usize) -> Self {
        MapDistance { value, unit }
    }

    pub fn resolve(self, style: &impl Style) -> f64 {
        style.resolve_distance(self)
    }

    pub fn value(self) -> f64 {
        self.value
    }

    pub fn unit(self) -> usize {
        self.unit
    }
}


//------------ Distance ------------------------------------------------------

/// Describes a distance from a point.
///
/// In feature definitions, locations on paths are defined relative to named
/// points on the path. They are described as a distance from well-defined
/// points which is combined from a world distance and a map distance. This
/// way we can create schematic representations that are pleasing at a range
/// of scales.
#[derive(Clone, Debug)]
pub struct Distance {
    /// The world component of the distance.
    ///
    /// This is not yet scaled to storage coordinates, i.e., this value is the
    /// acutal distance along the face of the Earth in _bp._
    pub world: Option<f64>,

    /// The map component of the distance.
    ///
    /// Since map distances can only be interpreted at rendering time, we
    /// need to keep all the given values.
    pub map: Vec<MapDistance>,
}

impl Distance {
    /// Creates a new distance from the world and canvas components.
    pub fn new(world: Option<f64>, map: Vec<MapDistance>) -> Self {
        Distance { world, map }
    }

    /// Returns whether both dimensions of the distance are `None`.
    pub fn is_none(&self) -> bool {
        self.world.is_none() && self.map.is_empty()
    }

    /// Resolves the distance at the given point in storage coordinates.
    pub fn resolve(
        &self, point: Point, canvas: &Canvas, style: &impl Style,
    ) -> f64 {
        let mut res = self.world.map(|world| {
            to_storage_distance(world, point) * canvas.equator_scale()
        }).unwrap_or(0.);

        for item in &self.map {
            res += item.resolve(style) * canvas.canvas_bp()
        }

        res
    }
}

impl Default for Distance {
    fn default() -> Self {
        Distance { world: None, map: Vec::new() }
    }
}

/*
impl ops::Add for Distance {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self += other;
        self
    }
}
*/

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
        self.map.extend_from_slice(&other.map);
    }
}

/*
impl ops::SubAssign for Distance {
    fn sub_assign(&mut self, other: Distance) {
        unimplemented!();
        /*
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
        */
    }
}
*/

impl ops::Neg for Distance {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.world = self.world.map(ops::Neg::neg);
        for item in &mut self.map {
            item.value = -item.value
        }
        self
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
#[derive(Clone, Debug)]
pub struct Location {
    /// The time value of the world location.
    pub world: SegTime,

    /// The distance from the time value on the canvas.
    ///
    /// Positive values are further along the path, negative values are
    /// backwards on the path.
    pub map: Vec<MapDistance>,
}

impl Location {
    /// Creates a new location from its components.
    pub fn new(world: SegTime, map: Vec<MapDistance>) -> Self {
        Location { world, map }
    }
}


//------------ SegTime -------------------------------------------------------

/// A segment and a time on this segment..
#[derive(Clone, Copy, Debug)]
pub struct SegTime {
    /// A segment index.
    ///
    /// Parts indexes are given as the index of the node where the segment
    /// _ends._ Thus, the smallest index is 1 while the largest index is one
    /// less than the number of nodes.
    pub seg: u32,

    /// The time on this index.
    ///
    /// This is a floating point value between 0 and 1 where 0 refers to the
    /// starting node, 1 refers to the end node, and values between are time
    /// values for the Bezier curve.
    pub time: f64,
}

impl SegTime {
    pub fn new(seg: u32, time: f64) -> Self {
        SegTime { seg, time }
    }

    /// Converts the segtime into a clean endpoint.
    pub fn end(self) -> Self {
        if self.time == 0. {
            SegTime::new(self.seg - 1, 1.)
        }
        else {
            self
        }
    }
}

impl PartialEq for SegTime {
    fn eq(&self, other: &SegTime) -> bool {
        self.seg == other.seg && self.time == other.time
    }
}

impl Eq for SegTime { }

impl PartialOrd for SegTime {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SegTime {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.seg.cmp(&other.seg) {
            cmp::Ordering::Equal => {
                self.time.partial_cmp(&other.time).unwrap()
            }
            other => other
        }
    }
}


//------------ Helper Functions ----------------------------------------------

/// Scale a world distance into a storage distance at the given point.
///
/// The point is already in storage coordinates.
fn to_storage_distance(world: f64, at: Point) -> f64 {
    (world / EQUATOR_BP) * scale_correction(at)
}

/// The scale correction at a given point
fn scale_correction(at: Point) -> f64 {
    (1. + (PI - at.y * 2. * PI).sinh().powi(2)).sqrt()
}

