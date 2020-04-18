/// Paths.

use std::{cmp, ops, slice};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::sync::Arc;
use kurbo::{
    BezPath, CubicBez, ParamCurve, ParamCurveArclen, ParamCurveExtrema,
    PathSeg, Point, Rect, Vec2
};
use crate::canvas;
use crate::canvas::Canvas;
use crate::import::mp_path::velocity;


//------------ Configuration Constants ---------------------------------------

/// Accuracy for Kurbo arclen calculations in storage coordinates.
///
/// This value should provide centimetre accuracy in storage coordinates.
const STORAGE_ACCURACY: f64 = 1E-11;

/// Accuracy for Kurbo arclen calculations in canvas coordinates.
///
/// This value assumes about 192 dpi device resolution.
pub(crate) const CANVAS_ACCURACY: f64 = 0.025;


//------------ StoredPath ----------------------------------------------------

/// A basic path.
#[derive(Clone, Debug)]
pub struct StoredPath(Arc<BezPath>);

impl StoredPath {
    pub fn new(path: BezPath) -> Self {
        StoredPath(Arc::new(path))
    }

    /// Returns the minimum location of the path.
    fn min_location(&self) -> Location {
        Location::new(SegTime::new(1, 0.), 0.)
    }

    /// Returns the maximum location of the path.
    fn max_location(&self) -> Location {
        Location::new(SegTime::new(self.0.elements().len() as u32 - 1, 1.), 0.)
    }

    /// Returns the location at a distance from a node.
    fn location(&self, mut node: u32, distance: Distance) -> Location {
        let conv_pt = self.node_trimmed(node);
        let time = match distance.world {
            Some(world) => {
                let mut storage = to_storage_distance(world, conv_pt);
                if storage < 0. {
                    if node >= self.node_len() {
                        return self.max_location()
                    }
                    storage = -storage;
                    loop {
                        let seg = self.get_seg(node).unwrap();
                        let arclen = seg.arclen_storage();
                        if storage >= arclen {
                            match node.checked_sub(1) {
                                Some(val) => node = val,
                                None => return self.min_location()
                            }
                            storage -= arclen;
                        }
                        else {
                            break 1. - seg.rev().arctime_storage(storage)
                        }
                    }
                }
                else {
                    loop {
                        let seg = match self.get_seg(node + 1) {
                            Some(seg) => seg,
                            None => return self.max_location()
                        };
                        let arclen = seg.arclen_storage();
                        if storage >= arclen {
                            node += 1;
                            storage -= arclen
                        }
                        else {
                            break seg.arctime_storage(storage)
                        }
                    }
                }
            }
            None => 0.
        };
        let canvas = distance.canvas.unwrap_or(0.);
        if node == self.node_len() {
            Location::new(SegTime::new(node - 1, 1.), canvas)
        }
        else {
            Location::new(SegTime::new(node, time), canvas)
        }
    }

    /// Returns the time value for a location on a given canvas.
    fn location_time(&self, location: Location, canvas: &Canvas) -> SegTime {
        self._location_time(
            Location::new(
                location.world,
                location.canvas * canvas.canvas_bp()
            ),
            canvas
        )
    }

    fn _location_time(&self, location: Location, canvas: &Canvas) -> SegTime {
        if location.canvas == 0. {
            location.world
        }
        else if location.canvas < 0. {
            let offset = -location.canvas;
            let seg = self.segment(location.world.seg).transform(canvas);
            let before = seg.sub(0., location.world.time);
            let arclen = before.arclen();
            if arclen >= offset {
                let len = seg.sub(location.world.time, 1.).arclen() + offset;
                SegTime::new(
                    location.world.seg,
                    1. - seg.rev().arctime(len)
                )
            }
            else if location.world.seg > 1 {
                self._location_time(
                    Location::new(
                        SegTime::new(location.world.seg - 1, 1.),
                        -(offset - arclen)
                    ),
                    canvas
                )
            }
            else {
                SegTime::new(1, 0.)
            }
        }
        else {
            let offset = location.canvas;
            let seg = self.segment(location.world.seg).transform(canvas);
            let after = seg.sub(location.world.time, 1.);
            let arclen = after.arclen();
            if arclen > offset {
                let len = seg.sub(0., location.world.time).arclen() + offset;
                SegTime::new(
                    location.world.seg, seg.arctime(len)
                )
            }
            else if location.world.seg == self.node_len() - 1 {
                SegTime::new(location.world.seg, 1.)
            }
            else {
                self._location_time(
                    Location::new(
                        SegTime::new(location.world.seg + 1, 0.),
                        offset - arclen
                    ),
                    canvas
                )
            }
        }
    }

    /// Returns the complete segment with the given index.
    ///
    /// # Panic
    ///
    /// The method panics of the segment index is out of bounds.
    fn segment(&self, seg: u32) -> Segment {
        match self.0.get_seg(seg as usize).unwrap() {
            PathSeg::Cubic(seg) => Segment(seg),
            _ => unreachable!()
        }
    }

    /// Returns the partial segment before the given location.
    ///
    /// The returned segment will start at the node before the location and
    /// run to the location.
    fn segment_before(&self, loc: SegTime) -> Segment {
        self.segment(loc.seg).sub(0., loc.time)
    }

    /// Returns the partial segment before the given location.
    ///
    /// The returned segment will start at the node before the location and
    /// run to the location.
    fn segment_after(&self, loc: SegTime) -> Segment {
        self.segment(loc.seg).sub(loc.time, 1.)
    }

    fn node_len(&self) -> u32 {
        self.0.elements().len() as u32
    }

    /// Returns the segment ending at `idx`.
    ///
    /// In other words, the segment will cover the time values between
    /// `idx - 1` and `idx`.
    fn get_seg(&self, idx: u32) -> Option<Segment> {
        let idx = usize::try_from(idx).ok()?;
        match self.0.get_seg(idx)? {
            PathSeg::Cubic(seg) => Some(Segment(seg)),
            _ => unreachable!()
        }
    }

    /// Returns the node at the given index.
    ///
    /// If the index is out of bounds. Returns the last point.
    pub fn node_trimmed(&self, at: u32) -> Point {
        if at == 0 {
            return self.get_seg(1).unwrap().p0()
        }
        let at = cmp::min(at, self.node_len() - 1);
        self.get_seg(at).unwrap().p3()
    }
}


//------------ Path ----------------------------------------------------------

/// A path.
///
/// This path is constructed as a sequence of connected subpaths referencing
/// `StoredPath`s.
#[derive(Clone, Debug)]
pub struct Path {
    /// The sequence of parts.
    ///
    /// The first two elements are the tensions when leaving the previous
    /// part and entering this part. The third element is the part itself.
    parts: Vec<(f64, f64, Subpath)>,
}

impl Path {
    pub fn new(first: Subpath) -> Self {
        Path {
            parts: vec![(1., 1., first)]
        }
    }

    pub fn push(&mut self, post: f64, pre: f64, segment: Subpath) {
        self.parts.push((post, pre, segment))
    }

    pub fn apply(&self, canvas: &Canvas) {
        let mut segments = self.segments(canvas);
        let seg = segments.next().unwrap();
        seg.apply_start(canvas);
        seg.apply_tail(canvas);
        segments.for_each(|seg| seg.apply_tail(canvas));
    }

    pub fn apply_offset(&self, offset: f64, canvas: &Canvas) {
        let mut segments = self.segments(canvas);
        let seg = segments.next().unwrap().offset(offset);
        seg.apply_start(canvas);
        seg.apply_tail(canvas);
        segments.for_each(|seg| seg.offset(offset).apply_tail(canvas));
    }


    pub fn storage_bounds(&self) -> Rect {
        let mut parts = self.parts.iter();
        let mut res = parts.next().unwrap().2.storage_bounds();
        for item in parts {
            res = res.union(item.2.storage_bounds())
        }
        res
    }

    pub fn parts(&self) -> PathPartsIter {
        self.parts.iter()
    }

    pub fn segments<'a>(&'a self, canvas: &'a Canvas) -> PathSegmentIter<'a> {
        PathSegmentIter::new(self, canvas)
    }

    pub fn partitions<'a>(
        &'a self, part_len: f64, canvas: &'a Canvas
    ) -> PathPartitionIter<'a> {
        PathPartitionIter::new(
            self.segments(canvas),
            part_len * canvas.canvas_bp()
        )
    }
}


//------------ PathPartsIter -------------------------------------------------

/// An iterator over the parts of the path.
pub type PathPartsIter<'a> = slice::Iter<'a, (f64, f64, Subpath)>;


//------------ PathSegmentIter -----------------------------------------------

/// An iterator over the segments in a path.
#[derive(Clone, Debug)]
pub struct PathSegmentIter<'a> {
    /// An iterator producing the next part of the path.
    next_part: PathPartsIter<'a>,

    /// An iterator producing the next segment of the current part.
    ///
    /// If this is `None`, we need a new part.
    next_seg: SubpathSegmentIter<'a>,

    /// The last segment we returned.
    ///
    /// This is necessary to build the connection between parts.
    last_seg: Option<Segment>,

    /// The canvas to use for transforming the path.
    canvas: &'a Canvas,
}

impl<'a> PathSegmentIter<'a> {
    fn new(path: &'a Path, canvas: &'a Canvas) -> Self {
        let mut next_part = path.parts.iter();
        let &(_, _, ref part) = next_part.next().unwrap();
        PathSegmentIter {
            next_part,
            next_seg: part.iter(canvas),
            last_seg: None,
            canvas
        }
    }
}

impl<'a> Iterator for PathSegmentIter<'a> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_seg.next() {
            Some(seg) => {
                // We have one more segment in the current part.
                self.last_seg = Some(seg);
                Some(seg)
            }
            None => {
                // We’ve run out of segments in the current part.
                //
                // Grab the next part or return if we are done.
                let &(post, pre, ref part) = self.next_part.next()?;
                self.next_seg = part.iter(self.canvas);

                // We need to produce a connection between the last and next
                // segment.
                //
                // Grab the cached segment and first segment of the new part.
                // Neither of them can be None at this point lest we have
                // empty parts.
                let before = self.last_seg.take().unwrap();

                // We take the first segment from a copy of the iterator so
                // we continue with returning the first segment next time.
                //
                // (Using a clone here is cheaper since the iterator does all
                // the expensive stuff during its creation.)
                let after = self.next_seg.clone().next().unwrap();

                Some(Segment::connect(before, post, pre, after))
            }
        }
    }
}


//------------ PathPartitionIter ---------------------------------------------

/// An iterator over equal-length partitions of a path.
///
/// The iterator produces items of type [`canvas::Path`] that can be applied
/// directly to the canvas or be meddled with.
///
/// [`canvas::Path`]: ../canvas/struct.Path.html
#[derive(Clone, Debug)]
pub struct PathPartitionIter<'a> {
    /// The current segment.
    ///
    /// If this is `None`, we have processed the whole segment and need to
    /// move on to the next.
    cur_seg: Option<Segment>,
    
    /// The iterator producing the next segment.
    next_seg: PathSegmentIter<'a>,
    
    /// The arclen of each partition.
    part_len: f64,
}

impl<'a> PathPartitionIter<'a> {
    fn new(segments: PathSegmentIter<'a>, part_len: f64) -> Self {
        PathPartitionIter {
            cur_seg: None,
            next_seg: segments,
            part_len
        }
    }

    /// Changes the partition length.
    ///
    /// The length is given in _bp_.
    pub fn set_part_len(&mut self, part_len: f64) {
        self.part_len = part_len * self.next_seg.canvas.canvas_bp();
    }
}

impl<'a> Iterator for PathPartitionIter<'a> {
    type Item = canvas::Path<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut seg = match self.cur_seg {
            Some(seg) => seg,
            None => self.next_seg.next()? // Return on empty.
        };
        let mut res = canvas::Path::new(self.next_seg.canvas);
        let mut part_len = self.part_len;
        res.move_to(seg.p0());

        loop {
            let seg_len = seg.arclen();
            match part_len.partial_cmp(&seg_len).unwrap() {
                Ordering::Less => {
                    let end = seg.arctime(part_len);
                    self.cur_seg = Some(seg.sub(end, 1.0));
                    let end = seg.sub(0., end);
                    res.curve_to(end.p1(), end.p2(), end.p3());
                    break
                }
                Ordering::Equal => {
                    self.cur_seg = None;
                    res.curve_to(seg.p1(), seg.p2(), seg.p3());
                    break
                }
                Ordering::Greater => {
                    res.curve_to(seg.p1(), seg.p2(), seg.p3());
                    part_len -= seg_len;
                    match self.next_seg.next() {
                        Some(next_seg) => seg = next_seg,
                        None => {
                            self.cur_seg = None;
                            break
                        }
                    }
                }
            }
        }
        Some(res)
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
    path: StoredPath,

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
        path: StoredPath,
        start: Location, end: Location,
        offset: Option<Distance>
    ) -> Self {
        Subpath { path, start, end, offset }
    }

    pub fn eval_full(path: StoredPath, offset: Option<Distance>) -> Self {
        let start = path.min_location();
        let end = path.max_location();
        Subpath::new(path, start, end, offset)
    }

    pub fn eval(
        path: StoredPath,
        start_node: u32, start_distance: Distance,
        end_node: u32, end_distance: Distance,
        offset: Option<Distance>
    ) -> Self {
        let start = path.location(start_node, start_distance);
        let end = path.location(end_node, end_distance);
        Subpath::new(path, start, end, offset)
    }

    pub fn storage_bounds(&self) -> Rect {
        let start = self.start.world;
        let end = self.end.world;
        match start.seg.cmp(&end.seg) {
            Ordering::Equal => {
               self.path.segment_after(start).bounds()
            }
            Ordering::Less => {
                let mut res = self.path.segment_after(start).bounds();
                for seg in start.seg + 1..end.seg {
                    res = res.union(self.path.segment(seg).bounds());
                }
                res.union(self.path.segment_before(end).bounds())
            }
            Ordering::Greater => {
                let mut res = self.path.segment_before(start).bounds();
                for seg in end.seg + 1..start.seg {
                    res = res.union(self.path.segment(seg).bounds());
                }
                res.union(self.path.segment_after(end).bounds())
            }
        }
    }

    fn iter<'a>(&'a self, canvas: &'a Canvas) -> SubpathSegmentIter<'a> {
        SubpathSegmentIter::new(self, canvas)
    }
}


//------------ SubpathSegmentIter --------------------------------------------

/// An iterator over the segments in the subpath.
#[derive(Clone, Debug)]
pub struct SubpathSegmentIter<'a> {
    /// The subpath we are working on.
    subpath: &'a Subpath,

    /// The canvas to transform the base path onto.
    canvas: &'a Canvas,

    /// Is this subpath forward or backward over the base path?
    forward: bool,

    /// The first segment of the subpath.
    ///
    /// This is precomputed since it is used twice by the path iterator. It
    /// will be `None` if we are past the first segment.
    ///
    /// If the subpath only has only one segment, this is it.
    first: Option<Segment>,

    /// The middle of the subpath.
    ///
    /// This is the range of segment indexes left to iterate over. It will be
    /// `None` if we are past the middle or there isn’t one.
    middle: Option<(u32, u32)>,

    /// The last segment of the subpath.
    ///
    /// Contains the location of the end of the segment. Will be `None` if we
    /// are past this part or if there isn’t one.
    last: Option<SegTime>,
}

impl<'a> SubpathSegmentIter<'a> {
    fn new(subpath: &'a Subpath, canvas: &'a Canvas) -> Self {
        let start = subpath.path.location_time(subpath.start, canvas);
        let end = subpath.path.location_time(subpath.end, canvas);

        if start < end {
            Self::new_forward(subpath, canvas, start, end)
        }
        else {
            Self::new_reverse(subpath, canvas, start, end)
        }
    }

    fn new_forward(
        subpath: &'a Subpath, canvas: &'a Canvas,
        start: SegTime, mut end: SegTime
    ) -> Self {
        if end.time == 0. {
            end = SegTime::new(end.seg - 1, 1.);
        }
        let (first, middle, last) = if start.seg == end.seg {
            (
                Some(
                    subpath.path.segment(start.seg).sub(
                        start.time, end.time
                    ).transf_off(canvas, subpath.offset)
                ),
                None,
                None
            )
        }
        else {
            (
                Some(subpath.path.segment_after(start).transf_off(
                    canvas, subpath.offset
                )),
                if start.seg + 2 >= end.seg {
                    None
                }
                else {
                    Some((start.seg + 1, end.seg - 1))
                },
                Some(end)
            )
        };
        SubpathSegmentIter {
            subpath, canvas,
            forward: true,
            first, middle, last
        }
    }

    fn new_reverse(
        subpath: &'a Subpath, canvas: &'a Canvas,
        mut start: SegTime, end: SegTime
    ) -> Self {
        if start.time == 0. {
            start = SegTime::new(start.seg - 1, 1.);
        }
        let (first, middle, last) = if start.seg == end.seg {
            (
                Some(
                    subpath.path.segment(start.seg).sub(
                        end.time, start.time
                    ).rev().transf_off(canvas, subpath.offset)
                ),
                None,
                None
            )
        }
        else {
            (
                Some(subpath.path.segment_before(start).rev().transf_off(
                    canvas, subpath.offset
                )),
                if end.seg + 2 >= start.seg {
                    None
                }
                else {
                    Some((start.seg - 1, end.seg + 1))
                },
                Some(end)
            )
        };
        SubpathSegmentIter {
            subpath, canvas,
            forward: false,
            first, middle, last
        }
    }

    fn next_forward(&mut self) -> Option<Segment> {
        if let Some(seg) = self.first.take() {
            return Some(seg)
        }
        if let Some((start, end)) = self.middle {
            let seg = self.subpath.path.segment(start).transf_off(
                self.canvas, self.subpath.offset
            );
            let start = start + 1;
            self.middle = if start > end {
                None
            }
            else {
                Some((start, end))
            };
            return Some(seg)
        }
        if let Some(end) = self.last.take() {
            return Some(
                self.subpath.path.segment_before(end).transf_off(
                    self.canvas, self.subpath.offset
                )
            )
        }
        None
    }

    fn next_reverse(&mut self) -> Option<Segment> {
        if let Some(seg) = self.first.take() {
            return Some(seg)
        }
        if let Some((start, end)) = self.middle {
            let seg = self.subpath.path.segment(start).rev().transf_off(
                self.canvas, self.subpath.offset
            );
            let start = start - 1;
            self.middle = if start < end {
                None
            }
            else {
                Some((start, end))
            };
            return Some(seg)
        }
        if let Some(end) = self.last.take() {
            return Some(
                self.subpath.path.segment_after(end).rev().transf_off(
                    self.canvas, self.subpath.offset
                )
            )
        }
        None
    }
}

impl<'a> Iterator for SubpathSegmentIter<'a> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.forward {
            self.next_forward()
        }
        else {
            self.next_reverse()
        }
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
    path: StoredPath,

    /// The location of the position on the path.
    location: Location,

    /// Offset from the original path.
    ///
    /// Given in canvas coordinates. Positive values are to the left of the
    /// path. I.e., this is the length of a tangent vector rotated 90°.
    offset: Option<Distance>
}

impl Position {
    pub fn new(
        path: StoredPath, location: Location, offset: Option<Distance>
    ) -> Self {
        Position { path, location, offset }
    }

    /*
    fn storage_bounds(&self) -> Rect {
        let p = self.path.segment_after(
            self.location.world
        ).p0();
        (p, p).into()
    }
    */
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

    /// Resolves the distance at the given point in storage coordinates.
    fn resolve(self, point: Point, canvas: &Canvas) -> f64 {
        let world = match self.world {
            Some(world) => {
                // XXX I think this is correct?
                world * (1. - point.y.tanh().powi(2)).sqrt()
                * canvas.storage_bp()
            }
            None => 0.
        };
        let canv = match self.canvas {
            Some(canv) => canv * canvas.canvas_bp(),
            None => 0.
        };
        world + canv
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
    pub world: SegTime,

    /// The distance from the time value on the canvas.
    ///
    /// Positive values are further along the path, negative values are
    /// backwards on the path.
    pub canvas: f64,
}

impl Location {
    /// Creates a new location from its components.
    pub fn new(world: SegTime, canvas: f64) -> Self {
        Location { world, canvas }
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


//------------ Segment -------------------------------------------------------

/// A Bezier segment.
#[derive(Clone, Copy, Debug)]
pub struct Segment(CubicBez);

impl Segment {
    /// Creates a new segment from its points.
    fn new<P: Into<Point>>(p0: P, p1: P, p2: P, p3: P) -> Self {
        Segment(CubicBez::new(p0, p1, p2, p3))
    }

    /// Create a segment connecting two path locations.
    fn connect(
        before: Segment, post: f64, pre: f64, after: Segment
    ) -> Segment {
        let r = before.p3();
        let s = after.p0();

        let d = s - r;
        let aa = d.atan2();
        let theta = before.exit_dir().atan2() - aa;
        let phi = after.entry_dir().atan2() - aa;
        let (st, ct) = (theta.sin(), theta.cos());
        let (sf, cf) = (-phi.sin(), phi.cos());
        let rr = velocity(st, ct, sf, cf, post);
        let ss = velocity(sf, cf, st, ct, pre);

        // XXX We are ignoring negative tension ("at least") here because
        //     we don’t have that in our path expressions (yet).

        Segment::new(
            r,
            Point::new(
                r.x + (d.x * ct - d.y * st) * rr,
                r.y + (d.y * ct + d.x * st) * rr
            ),
            Point::new(
                s.x - (d.x * cf + d.y * sf) * ss,
                s.y - (d.y * cf - d.x * sf) * ss
            ),
            s
        )
    }

    /// Returns the bounding box of the segment.
    fn bounds(self) -> Rect {
        self.0.bounding_box()
    }

    /// Returns the arc length of the segment.
    fn arclen(self) -> f64 {
        self.0.arclen(CANVAS_ACCURACY)
    }

    /// Returns the arc length of the segment.
    fn arclen_storage(self) -> f64 {
        self.0.arclen(STORAGE_ACCURACY)
    }

    /// Returns the time of the point arclen along the path.
    fn arctime(self, arclen: f64) -> f64 {
        self.0.inv_arclen(arclen, CANVAS_ACCURACY)
    }

    /// Returns the time of the point arclen along the path.
    fn arctime_storage(self, arclen: f64) -> f64 {
        self.0.inv_arclen(arclen, STORAGE_ACCURACY)
    }

    /// Reverses the segment.
    fn rev(self) -> Self {
        Segment::new(self.p3(), self.p2(), self.p1(), self.p0())
    }

    /// Scale the segment for use with a canvas.
    fn transform(self, canvas: &Canvas) -> Segment {
        Segment(canvas.transform() * self.0)
    }

    /// Scales the segment and then offsets it if necessary.
    fn transf_off(self, canvas: &Canvas, offset: Option<Distance>) -> Segment {
        match offset {
            Some(offset) => {
                let offset = offset.resolve(self.p0(), canvas);
                Segment(canvas.transform() * self.0).offset(offset)
            }
            None => Segment(canvas.transform() * self.0),
        }
    }

    /// Returns the part of the segment between the two given times.
    fn sub(self, start: f64, end: f64) -> Self {
        Segment(self.0.subsegment(start..end))
    }

    fn p0(self) -> Point { self.0.p0 }
    fn p1(self) -> Point { self.0.p1 }
    fn p2(self) -> Point { self.0.p2 }
    fn p3(self) -> Point { self.0.p3 }

    /// Returns a path that is offset to the left by the given value.
    ///
    /// This uses the Tiller-Hanson method by just shifting the four points
    /// in the given direction and will break with tight curves. For now, we
    /// assume we don’t have those with railways and can get away with this
    /// approach.
    fn offset(self, offset: f64) -> Segment {
        // Let’s change the naming slighly. r and s are the end points, u and
        // v the control points.
        let (r, u, v, s) = (self.0.p0, self.0.p1, self.0.p2, self.0.p3);

        // Since control points can be identical (too close to?) to their
        // nearest endpoint, we end up with four cases.
        match (r == u, v == s) {
            (false, false) => {
                // Four unique points.
                // 
                // Get direction vectors out for the connecting lines:
                let wru = u - r; // direction from r to u.
                let wuv = v - u; // direction from u to v.
                let wvs = s - v; // direction from v to s.

                // The start and end points are just out along wru and wvs
                // rotated 90° and scaled accordingly.
                let rr = r + rot90(wru).normalize() * offset;
                let ss = s + rot90(wvs).normalize() * offset;

                // The control points are where the connecting lines meet
                // after they have been moved out. To construct these we
                // need a start point for the middle line which we get by
                // just moving u out along wuv:
                let uv = u + rot90(wuv).normalize() * offset;

                // Now we can interset the lines.
                let uu = line_intersect(rr, wru, uv, wuv);
                let vv = line_intersect(uv, wuv, ss, wvs);
                Segment::new(rr, uu, vv, ss)
            }
            (true, false) => {
                // r and u are the same.
                //
                // We skip calculating uu and just set it rr.
                let wrv = v - r;
                let wvs = s - v;
                let rr = r + rot90(wrv).normalize() * offset;
                let ss = s + rot90(wvs).normalize() * offset;
                let vs = v + rot90(wvs).normalize() * offset;
                let vv = line_intersect(rr, wrv, vs, wvs);
                Segment::new(rr, rr, vv, ss)
            }
            (false, true) => {
                // v and s are the same.
                let wru = u - r;
                let wus = s - u;
                let rr = r + rot90(wru).normalize() * offset;
                let ss = s + rot90(wus).normalize() * offset;
                let us = u + rot90(wus).normalize() * offset;
                let uu = line_intersect(rr, wru, us, wus);
                Segment::new(rr, uu, ss, ss)
            }
            (true, true) => {
                // Straight line.
                let d = rot90(s - r).normalize() * offset;
                Segment::new(r + d, u + d, v + d, s + d)
            }
        }
    }

    /// Applies the start of the segment to the canvas.
    fn apply_start(&self, canvas: &Canvas) {
        canvas.move_to(self.0.p0.x, self.0.p0.y);
    }

    /// Applies the tail of the segment to the canvas.
    fn apply_tail(&self, canvas: &Canvas) {
        /*
        canvas.stroke();
        canvas.save();
        canvas.set_source_rgb(1.0, 0., 0.);
        canvas.set_line_width(0.5 * canvas.canvas_bp());
        for p in &[self.p1(), self.p2()] {
            canvas.arc(
                p.x, p.y, 1. * canvas.canvas_bp(),
                0., 2. * std::f64::consts::PI
            );
            canvas.stroke();
        }
        canvas.set_source_rgb(0., 0., 1.);
        canvas.arc(
            self.p3().x, self.p3().y, 1. * canvas.canvas_bp(),
            0., 2. * std::f64::consts::PI
        );
        canvas.stroke();
        canvas.restore();
        canvas.move_to(self.0.p0.x, self.0.p0.y);
        */
        canvas.curve_to(
            self.0.p1.x, self.0.p1.y,
            self.0.p2.x, self.0.p2.y,
            self.0.p3.x, self.0.p3.y,
        )
    }

    /// Returns the direction when entering this segment.
    fn entry_dir(self) -> Vec2 {
        if self.p0() == self.p1() {
            if self.p0() == self.p2() {
                self.p3() - self.p0()
            }
            else {
                self.p2() - self.p0()
            }
        }
        else {
            self.p1() - self.p0()
        }
    }

    /// Returns the direction when leaving the segment.
    fn exit_dir(self) -> Vec2 {
        if self.p3() == self.p2() {
            if self.p3() == self.p1() {
                self.p3() - self.p0()
            }
            else {
                self.p3() - self.p1()
            }
        }
        else {
            self.p3() - self.p2()
        }
    }


}


//------------ Helper Functions ----------------------------------------------

/// Rotates a vector by 90°.
fn rot90(vec: Vec2) -> Vec2 {
    Vec2::new(vec.y, -vec.x)
}

/// Determines the intersection point between two lines.
///
/// Each line is given by a point and a direction vector.
fn line_intersect(p1: Point, d1: Vec2, p2: Point, d2: Vec2) -> Point {
    let t = (-(p1.x - p2.x) * d2.y + (p1.y - p2.y) * d2.x)
          / (d1.x * d2.y - d1.y * d2.x);

    p1 + t * d1
}

/// Scale a world distance into a storage distance at the given point.
///
/// The point is already in storage coordinates.
fn to_storage_distance(world: f64, at: Point) -> f64 {
    const EQUATOR: f64 = (40_075_016_686. / (25.4/72.)); // in bp

    (world / EQUATOR) * (1. - at.y.tanh().powi(2)).sqrt()
}


//============ Tests =========================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_line_intersect() {
        assert_eq!(
            line_intersect(
                Point::new(32.,30.), Vec2::new(6., -6.),
                Point::new(30.,26.), Vec2::new(15., 3.)
            ),
            Point::new(35.,27.)
        );
    }
}
