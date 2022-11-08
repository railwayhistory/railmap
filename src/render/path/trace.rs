/// Traces.
///

use std::{cmp, slice};
use std::f64::consts::PI;
use kurbo::{
    CubicBez, Line, Rect, ParamCurve, ParamCurveArclen,
    ParamCurveExtrema, Point, Vec2
};
use crate::import::mp_path::velocity;
use crate::theme::Style;
use super::super::canvas;
use super::super::canvas::Canvas;
use super::path::{Distance, Location, SegTime, Path};


//------------ Configuration Constants ---------------------------------------

/// Accuracy for Kurbo arclen calculations in storage coordinates.
///
/// This value should provide centimetre accuracy in storage coordinates.
pub(crate) const STORAGE_ACCURACY: f64 = 1E-11;

/// Accuracy for Kurbo arclen calculations in canvas coordinates.
///
/// This value assumes about 192 dpi device resolution.
pub(crate) const CANVAS_ACCURACY: f64 = 0.025;


//------------ Trace ---------------------------------------------------------

/// Description of a trace along the canvas.
///
/// This trace is constructed as a sequence of connected subpaths referencing
/// `Path`s.
#[derive(Clone, Debug, Default)]
pub struct Trace {
    /// The sequence of parts.
    ///
    /// The first two elements are the tensions when leaving the previous
    /// part and entering this part. The third element is the part itself.
    parts: Vec<(f64, f64, Section)>,
}

impl Trace {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_subpath(&mut self, post: f64, pre: f64, section: Subpath) {
        self.parts.push((post, pre, Section::Subpath(section)))
    }

    pub fn push_edge(&mut self, post: f64, pre: f64, section: Edge) {
        self.parts.push((post, pre, Section::Edge(section)))
    }

    pub fn push_trace(&mut self, post: f64, pre: f64, trace: &Trace) {
        let first = match trace.parts.first() {
            Some(first) => first,
            None => return
        };
        self.parts.push((post, pre, first.2.clone()));
        self.parts.extend_from_slice(&trace.parts[1..]);
    }

    pub fn apply(&self, canvas: &Canvas, style: &impl Style) {
        let mut segments = self.segments(style);
        let seg = segments.next().unwrap();
        seg.apply_start(canvas);
        seg.apply_tail(canvas);
        segments.for_each(|seg| seg.apply_tail(canvas));
    }

    pub fn apply_offset(
        &self, offset: f64, canvas: &Canvas, style: &impl Style,
    ) {
        let mut segments = self.segments(style);
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

    fn parts(&self) -> PartsIter {
        self.parts.iter()
    }

    pub fn segments<'a, S: Style>(
        &'a self, style: &'a S,
    ) -> SegmentIter<'a, S> {
        SegmentIter::new(self, style)
    }

    pub fn partitions<'a, S: Style>(
        &'a self, part_len: f64, style: &'a S,
    ) -> PartitionIter<'a, S> {
        PartitionIter::new(
            self.segments(style),
            part_len * style.transform().canvas_bp()
        )
    }
}


//------------ PartsIter -----------------------------------------------------

/// An iterator over the parts of the path.
type PartsIter<'a> = slice::Iter<'a, (f64, f64, Section)>;


//------------ SegmentIter ---------------------------------------------------

/// An iterator over the segments in a path.
#[derive(Clone, Debug)]
pub struct SegmentIter<'a, S> {
    /// An iterator producing the next part of the path.
    next_part: PartsIter<'a>,

    /// An iterator producing the next segment of the current part.
    ///
    /// If this is `None`, we need a new part.
    next_seg: SectionSegmentIter<'a, S>,

    /// The last segment we returned.
    ///
    /// This is necessary to build the connection between parts.
    last_seg: Option<Segment>,

    /// The style for determining distances.
    style: &'a S,
}

impl<'a, S: Style> SegmentIter<'a, S> {
    fn new(path: &'a Trace, style: &'a S) -> Self {
        let mut next_part = path.parts();
        let &(_, _, ref part) = next_part.next().unwrap();
        SegmentIter {
            next_part,
            next_seg: part.iter(style),
            last_seg: None,
            style,
        }
    }
}

impl<'a, S: Style> Iterator for SegmentIter<'a, S> {
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
                self.next_seg = part.iter(self.style);

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


//------------ PartitionIter ------------------------------------------------

/// An iterator over equal-length partitions of a path.
///
/// The iterator produces items of type [`canvas::Path`] that can be applied
/// directly to the canvas or be meddled with.
///
/// [`canvas::Path`]: ../canvas/struct.Path.html
#[derive(Clone, Debug)]
pub struct PartitionIter<'a, S> {
    /// The current segment.
    ///
    /// If this is `None`, we have processed the whole segment and need to
    /// move on to the next.
    cur_seg: Option<Segment>,
    
    /// The iterator producing the next segment.
    next_seg: SegmentIter<'a, S>,
    
    /// The arclen of each partition.
    part_len: f64,
}

impl<'a, S> PartitionIter<'a, S> {
    fn new(segments: SegmentIter<'a, S>, part_len: f64) -> Self {
        PartitionIter {
            cur_seg: None,
            next_seg: segments,
            part_len
        }
    }

    /// Changes the partition length.
    ///
    /// The length is given in _bp_.
    pub fn set_part_len(&mut self, part_len: f64)
    where S: Style {
        self.part_len = part_len * self.next_seg.style.transform().canvas_bp();
    }
}

impl<'a, S: Style> Iterator for PartitionIter<'a, S> {
    type Item = canvas::Path;

    fn next(&mut self) -> Option<Self::Item> {
        let mut seg = match self.cur_seg {
            Some(seg) => seg,
            None => self.next_seg.next()? // Return on empty.
        };
        let mut res = canvas::Path::new();
        let mut part_len = self.part_len;
        res.move_to(seg.p0());

        loop {
            let seg_len = seg.arclen();
            match part_len.partial_cmp(&seg_len).unwrap() {
                cmp::Ordering::Less => {
                    let end = seg.arctime(part_len);
                    self.cur_seg = Some(seg.sub(end, 1.0));
                    let end = seg.sub(0., end);
                    res.curve_to(end.p1(), end.p2(), end.p3());
                    break
                }
                cmp::Ordering::Equal => {
                    self.cur_seg = None;
                    res.curve_to(seg.p1(), seg.p2(), seg.p3());
                    break
                }
                cmp::Ordering::Greater => {
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


//------------ Section -------------------------------------------------------

/// A section of a path.
#[derive(Clone, Debug)]
enum Section {
    Subpath(Subpath),
    Edge(Edge),
}

impl Section {
    fn storage_bounds(&self) -> Rect {
        match *self {
            Section::Subpath(ref section) => section.storage_bounds(),
            Section::Edge(ref section) => section.storage_bounds(),
        }
    }

    fn iter<'a, S: Style>(
        &'a self, style: &'a S,
    ) -> SectionSegmentIter<'a, S> {
        match *self {
            Section::Subpath(ref subpath) => {
                SectionSegmentIter::Subpath(subpath.iter(style))
            }
            Section::Edge(ref line) => {
                SectionSegmentIter::Edge(line.iter(style))
            }
        }
    }
}


//------------ SectionSegmentIter --------------------------------------------

#[derive(Debug)]
enum SectionSegmentIter<'a, S> {
    Subpath(SubpathSegmentIter<'a, S>),
    Edge(EdgeSegmentIter<'a, S>)
}

impl<'a, S> Clone for SectionSegmentIter<'a, S> {
    fn clone(&self) -> Self {
        match *self {
            SectionSegmentIter::Subpath(ref sub) => {
                SectionSegmentIter::Subpath(sub.clone())
            }
            SectionSegmentIter::Edge(ref edge) => {
                SectionSegmentIter::Edge(edge.clone())
            }
        }
    }
}

impl<'a, S: Style> Iterator for SectionSegmentIter<'a, S> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            SectionSegmentIter::Subpath(ref mut section) => section.next(),
            SectionSegmentIter::Edge(ref mut section) => section.next(),
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

    /// Offset from the original path.
    ///
    /// Given in canvas coordinates. Positive values are to the left of the
    /// path. I.e., this is the length of a tangent vector rotated 90°.
    offset: Option<Distance>,
}

impl Subpath {
    pub fn new(
        path: Path,
        start: Location, end: Location,
        offset: Option<Distance>
    ) -> Self {
        Subpath { path, start, end, offset }
    }

    pub fn eval_full(path: Path) -> Self {
        let start = path.min_location();
        let end = path.max_location();
        Subpath::new(path, start, end, None)
    }

    pub fn eval(
        path: Path,
        start_node: u32, start_distance: Distance,
        end_node: u32, end_distance: Distance,
        offset: Distance
    ) -> Self {
        let start = path.location(start_node, start_distance);
        let end = path.location(end_node, end_distance);
        let offset = if offset.is_none() {
            None
        }
        else {
            Some(offset)
        };
        Subpath::new(path, start, end, offset)
    }

    pub fn storage_bounds(&self) -> Rect {
        let start = self.start.world;
        let end = self.end.world;
        match start.seg.cmp(&end.seg) {
            cmp::Ordering::Equal => {
               self.path.segment_after(start).bounds()
            }
            cmp::Ordering::Less => {
                let mut res = self.path.segment_after(start).bounds();
                for seg in start.seg + 1..end.seg {
                    res = res.union(self.path.segment(seg).unwrap().bounds());
                }
                res.union(self.path.segment_before(end).bounds())
            }
            cmp::Ordering::Greater => {
                let mut res = self.path.segment_before(start).bounds();
                for seg in end.seg + 1..start.seg {
                    res = res.union(self.path.segment(seg).unwrap().bounds());
                }
                res.union(self.path.segment_after(end).bounds())
            }
        }
    }

    fn iter<'a, S: Style>(
        &'a self, style: &'a S,
    ) -> SubpathSegmentIter<'a, S> {
        SubpathSegmentIter::new(self, style)
    }
}


//------------ SubpathSegmentIter --------------------------------------------

/// An iterator over the segments in the subpath.
#[derive(Debug)]
pub struct SubpathSegmentIter<'a, S> {
    /// The subpath we are working on.
    subpath: &'a Subpath,

    /// The style for determining distances.
    style: &'a S,

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

impl<'a, S> Clone for SubpathSegmentIter<'a, S> {
    fn clone(&self) -> Self {
        SubpathSegmentIter {
            subpath: self.subpath,
            style: self.style,
            forward: self.forward,
            first: self.first,
            middle: self.middle,
            last: self.last
        }
    }
}

impl<'a, S: Style> SubpathSegmentIter<'a, S> {
    fn new(subpath: &'a Subpath, style: &'a S) -> Self {
        let start = subpath.path.location_time(&subpath.start, style);
        let end = subpath.path.location_time(&subpath.end, style);

        if start < end {
            Self::new_forward(subpath, style, start, end)
        }
        else {
            Self::new_reverse(subpath, style, start, end)
        }
    }

    fn new_forward(
        subpath: &'a Subpath, style: &'a S,
        start: SegTime, mut end: SegTime
    ) -> Self {
        if end.time == 0. {
            end = SegTime::new(end.seg - 1, 1.);
        }
        let (first, middle, last) = if start.seg == end.seg {
            (
                Some(
                    subpath.path.segment(start.seg).unwrap().sub(
                        start.time, end.time
                    ).transf_off(style, subpath.offset.as_ref())
                ),
                None,
                None
            )
        }
        else {
            (
                Some(subpath.path.segment_after(start).transf_off(
                    style, subpath.offset.as_ref()
                )),
                if start.seg + 2 > end.seg {
                    None
                }
                else {
                    Some((start.seg + 1, end.seg - 1))
                },
                Some(end)
            )
        };
        SubpathSegmentIter {
            subpath, style,
            forward: true,
            first, middle, last
        }
    }

    fn new_reverse(
        subpath: &'a Subpath, style: &'a S,
        mut start: SegTime, end: SegTime
    ) -> Self {
        if start.time == 0. && start.seg > 1 {
            start = SegTime::new(start.seg - 1, 1.);
        }
        let (first, middle, last) = if start.seg == end.seg {
            (
                Some(
                    subpath.path.segment(start.seg).unwrap().sub(
                        end.time, start.time
                    ).rev().transf_off(style, subpath.offset.as_ref())
                ),
                None,
                None
            )
        }
        else {
            (
                Some(subpath.path.segment_before(start).rev().transf_off(
                    style, subpath.offset.as_ref()
                )),
                if end.seg + 2 > start.seg {
                    None
                }
                else {
                    Some((start.seg - 1, end.seg + 1))
                },
                Some(end)
            )
        };
        SubpathSegmentIter {
            subpath, style,
            forward: false,
            first, middle, last
        }
    }

    fn next_forward(&mut self) -> Option<Segment> {
        if let Some(seg) = self.first.take() {
            return Some(seg)
        }
        if let Some((start, end)) = self.middle {
            let seg = self.subpath.path.segment(start).unwrap().transf_off(
                self.style, self.subpath.offset.as_ref()
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
                    self.style, self.subpath.offset.as_ref()
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
            let seg = self.subpath.path.segment(
                start
            ).unwrap().rev().transf_off(
               self.style, self.subpath.offset.as_ref()
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
                    self.style, self.subpath.offset.as_ref()
                )
            )
        }
        None
    }
}

impl<'a, S: Style> Iterator for SubpathSegmentIter<'a, S> {
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


//------------ Edge ----------------------------------------------------------

/// A straight edge between two positions.
#[derive(Clone, Debug)]
pub struct Edge {
    start: Position,
    end: Position
}

impl Edge {
    pub fn new(start: Position, end: Position) -> Self {
        Edge { start, end }
    }

    fn storage_bounds(&self) -> Rect {
        self.start.storage_bounds().union(
            self.end.storage_bounds()
        )
    }

    fn iter<'a, S>(
        &'a self, style: &'a S
    ) -> EdgeSegmentIter<'a, S> {
        EdgeSegmentIter {
            line: Some(self),
            style,
        }
    }
}


//------------ EdgeSegmentIter -----------------------------------------------

#[derive(Debug)]
struct EdgeSegmentIter<'a, S> {
    line: Option<&'a Edge>,
    style: &'a S,
}

impl<'a, S> Clone for EdgeSegmentIter<'a, S> {
    fn clone(&self) -> Self {
        EdgeSegmentIter {
            line: self.line,
            style: self.style
        }
    }
}

impl<'a, S: Style> Iterator for EdgeSegmentIter<'a, S> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.line.take()?;
        let start = line.start.resolve(self.style).0;
        let end = line.end.resolve(self.style).0;
        Some(Segment::line(start, end, None))
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
    /// Given in canvas coordinates. Positive values are to the left of the
    /// path. I.e., this is the length of a tangent vector rotated 90°.
    sideways: Option<Distance>,

    /// Shift of the resulting point.
    shift: Option<(Distance, Distance)>,

    /// Optional roation from the original direction.
    rotation: Option<f64>,
}

impl Position {
    pub fn new(
        path: Path,
        location: Location,
        sideways: Option<Distance>,
        shift: Option<(Distance, Distance)>,
        rotation: Option<f64>,
    ) -> Self {
        Position { path, location, sideways, shift, rotation }
    }

    pub fn eval(
        path: Path,
        node: u32,
        distance: Distance,
        sideways: Distance,
        shift: (Distance, Distance),
        rotation: Option<f64>
    ) -> Self {
        let location = path.location(node, distance);
        let rotation = rotation.map(f64::to_radians);
        let sideways = if sideways.is_none() {
            None
        }
        else {
            Some(sideways)
        };
        let shift = if shift.0.is_none() && shift.1.is_none() {
            None
        }
        else {
            Some(shift)
        };
        Position::new(path, location, sideways, shift, rotation)
    }

    pub fn shift_assign(&mut self, shift: (Distance, Distance)) {
        if let Some(curr) = self.shift.as_mut() {
            curr.0 += shift.0;
            curr.1 += shift.1;
        }
        else {
            self.shift = Some(shift)
        }
    }

    pub fn sideways(&self, sideways: Distance) -> Self {
        let mut res = self.clone();
        if let Some(curr) = res.sideways {
            res.sideways = Some(curr + sideways)
        }
        else {
            res.sideways = Some(sideways)
        }
        res
    }

    pub fn storage_bounds(&self) -> Rect {
        let p = self.path.segment_after(
            self.location.world
        ).p0();
        (p, p).into()
    }

    pub fn resolve(
        &self, style: &impl Style
    ) -> (Point, f64) {
        let loc = self.path.location_time(&self.location, style);
        let seg = self.path.segment(loc.seg).unwrap();
        let storage_point = seg.point(loc.time);
        let dir = seg.dir(loc.time);
        let shift = self.shift.as_ref().map(|shift| {
            Vec2::new(
                shift.0.resolve(storage_point, style),
                shift.1.resolve(storage_point, style)
            )
        });
        let mut point = style.transform().transform() * storage_point;
        let angle = dir.atan2() + self.rotation.unwrap_or(0.);
        if let Some(sideways) = self.sideways.as_ref() {
            let sideways= sideways.resolve(storage_point, style);
            let dir = sideways * rot90(dir).normalize();
            point += dir;
        }
        if let Some(shift) = shift {
            point += shift
        }
        (point, angle)
    }

    pub fn resolve_label(
        &self, style: &impl Style, on_path: bool
    ) -> (Point, f64) {
        if on_path {
            let (point, mut angle) = self.resolve(style);

            // Correct the angle so the label won’t be upside down.
            if angle.abs() > 0.5 * PI {
                if angle < 0. {
                    angle += PI
                }
                else {
                    angle -= PI
                }
            }
            (point, angle)
        }
        else {
            (self.resolve(style).0, self.rotation.unwrap_or(0.))
        }
    }
}


//------------ Segment -------------------------------------------------------

/// A path segment.
///
/// A segment connects exactly two points either in a straight line or via a
/// cubic bezier curve.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    /// The start point of the segment.
    start: Point,

    /// The optional control points of the segment.
    ///
    /// If this is `None`, the segment is a straight line.
    control: Option<(Point, Point)>,

    /// The end point of the segment.
    end: Point,

    /// The optional precomputed arc length of the segment.
    arclen: Option<f64>,
}

impl Segment {
    /// Creates a new straight segment.
    pub fn line(start: Point, end: Point, arclen: Option<f64>) -> Self {
        Segment {
            start,
            control: None,
            end,
            arclen,
        }
    }

    /// Creates a new cubic bezier segment.
    pub fn curve(
        start: Point, c0: Point, c1: Point, end: Point, arclen: Option<f64>
    ) -> Self {
        Segment {
            start,
            control: Some((c0, c1)),
            end,
            arclen,
        }
    }

    /// Creates a new segment connecting two other segments.
    pub fn connect(
        before: Segment, post: f64, pre: f64, after: Segment
    ) -> Segment {
        // Shortcut: if both tensions are infinite, we can just make a
        // straight line.
        if post.is_infinite() && pre.is_infinite() {
            return Segment::line(before.end, after.start, None)
        }

        let r = before.end;
        let s = after.start;

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

        let u = Point::new(
            r.x + (d.x * ct - d.y * st) * rr,
            r.y + (d.y * ct + d.x * st) * rr
        );
        let v = Point::new(
            s.x - (d.x * cf + d.y * sf) * ss,
            s.y - (d.y * cf - d.x * sf) * ss
        );

        // If both control points are at the end points: straight line.
        if r == u && v == s {
            Segment::line(r, s, None)
        }
        else {
            Segment::curve(r, u, v, s, None)
        }

    }

    /// Converts the segment into a kurbo segment.
    fn into_kurbo(self) -> Result<CubicBez, Line> {
        match self.control {
            Some((c0, c1)) => Ok(CubicBez::new(self.start, c0, c1, self.end)),
            None => Err(Line::new(self.start, self.end))
        }
    }

    /// Returns the start point of the segment.
    pub fn p0(self) -> Point {
        self.start
    }

    /// Returns the first control point of the segment.
    ///
    /// If the segment is a straight line, this is the start point.
    pub fn p1(self) -> Point {
        self.control.map(|c| c.0).unwrap_or_else(|| self.start)
    }

    /// Returns the second control point of the segment.
    ///
    /// If the segment is a straight line, this is the end point.
    pub fn p2(self) -> Point {
        self.control.map(|c| c.1).unwrap_or_else(|| self.end)
    }

    /// Returns the end point of the segment.
    pub fn p3(self) -> Point {
        self.end
    }

    /// Returns the point at the given times value.
    pub fn point(self, at: f64) -> Point {
        match self.into_kurbo() {
            Ok(seg) => seg.eval(at),
            Err(seg) => seg.eval(at)
        }
    }

    /// Returns a tangent vector at the given times value.
    ///
    /// The tangent vector will point into the direction of the path. It will
    /// _not_ have been normalized.
    pub fn dir(self, at: f64) -> Vec2 {
        match self.control {
            Some((c0, c1)) => {
                let ta = 1. - at;
                3. * ta * ta * (c0 - self.start)
                + 6. * ta * at * (c1 - c0)
                + 3. * at * at * (self.end - c1)
            }
            None => {
                self.end - self.start
            }
        }
    }

    /// Returns the direction when entering this segment.
    fn entry_dir(self) -> Vec2 {
        match self.control {
            Some((c0, c1)) => {
                if self.start == c0 {
                    if self.start == c1 {
                        self.end - self.start
                    }
                    else {
                        c1 - self.start
                    }
                }
                else {
                    c0 - self.start
                }
            }
            None => self.end - self.start
        }
    }

    /// Returns the direction when leaving the segment.
    fn exit_dir(self) -> Vec2 {
        match self.control {
            Some((c0, c1)) => {
                if self.end == c1 {
                    if self.end == c0 {
                        self.end - self.start
                    }
                    else {
                        self.end - c0
                    }
                }
                else {
                    self.end - c1
                }
            }
            None => self.end - self.start
        }
    }

    /// Returns whether the segment is straight.
    pub fn is_straight(self) -> bool {
        self.control.is_some()
    }

    /// Returns the bounding box of the segment.
    pub fn bounds(self) -> Rect {
        match self.into_kurbo() {
            Ok(seg) => seg.bounding_box(),
            Err(seg) => seg.bounding_box()
        }
    }

    /// Returns the arc length of the segment.
    pub fn arclen(self) -> f64 {
        match self.arclen {
            Some(arclen) => arclen,
            None => {
                match self.into_kurbo() {
                    Ok(seg) => seg.arclen(CANVAS_ACCURACY),
                    Err(seg) => seg.arclen(CANVAS_ACCURACY)
                }
            }
        }
    }

    /// Returns the arc length of the segment.
    pub fn arclen_storage(self) -> f64 {
        match self.arclen {
            Some(arclen) => arclen,
            None => {
                match self.into_kurbo() {
                    Ok(seg) => seg.arclen(STORAGE_ACCURACY),
                    Err(seg) => seg.arclen(STORAGE_ACCURACY)
                }
            }
        }
    }

    /// Returns the time of the point a given arc length away from the start.
    ///
    /// The result is accurate for use in canvas coordinates.
    pub fn arctime(self, arclen: f64) -> f64 {
        match self.into_kurbo() {
            Ok(seg) => seg.inv_arclen(arclen, CANVAS_ACCURACY),
            Err(seg) => seg.inv_arclen(arclen, CANVAS_ACCURACY),
        }
    }

    /// Returns the time of the point a given arc length away from the start.
    ///
    /// The result is accurate for use in storage coordinates.
    pub fn arctime_storage(self, arclen: f64) -> f64 {
        match self.into_kurbo() {
            Ok(seg) => seg.inv_arclen(arclen, STORAGE_ACCURACY),
            Err(seg) => seg.inv_arclen(arclen, STORAGE_ACCURACY),
        }
    }

    /// Reverses the segment.
    pub fn rev(self) -> Self {
        Segment {
            start: self.end,
            control: self.control.map(|(c0, c1)| (c1, c0)),
            end: self.start,
            arclen: self.arclen
        }
    }

    /// Transforms the segment for use with a style.
    pub fn transform(self, style: &impl Style) -> Self {
        Segment {
            start: style.transform().transform() * self.start,
            control: self.control.map(|(c0, c1)| {
                (
                    style.transform().transform() * c0,
                    style.transform().transform() * c1
                )
            }),
            end: style.transform().transform() * self.end,
            arclen: self.arclen.map(|arclen| {
                style.transform().transform().as_tuple().1 * arclen
            }),
        }
    }

    /// Scales the segment and then offsets it if necessary.
    pub fn transf_off(
        self, style: &impl Style, offset: Option<&Distance>
    ) -> Self {
        let res = self.transform(style);
        match offset {
            Some(offset) => {
                res.offset(offset.resolve(self.p0(), style))
            }
            None => res
        }
    }

    /// Returns the part of the segment between the two times.
    pub fn sub(self, start: f64, end: f64) -> Self {
        match self.into_kurbo() {
            Ok(bez) => {
                let bez = bez.subsegment(start..end);
                Segment {
                    start: bez.p0,
                    control: Some((bez.p1, bez.p2)),
                    end: bez.p3,
                    arclen: None
                }
            }
            Err(line) => {
                let line = line.subsegment(start..end);
                Segment {
                    start: line.p0,
                    control: None,
                    end: line.p1,
                    arclen: None
                }
            }
        }
    }

    /// Returns a path that is offset to the left by the given value.
    ///
    /// This uses the Tiller-Hanson method by just shifting the four points
    /// in the given direction and will break with tight curves. For now, we
    /// assume we don’t have those with railways and can get away with this
    /// approach.
    pub fn offset(self, offset: f64) -> Segment {
        // Let’s change the naming slighly. r and s are the end points, u and
        // v the control points, if we have them.
        //
        // To avoid weirdness, we treat control points that are at their
        // end points as non-existing. This may or may not be necessary
        // anymore.
        let (r, s) = (self.start, self.end);
        let (u, v) = match self.control {
            Some((p1, p2)) => {
                (
                    if p1 == r { None } else { Some(p1) },
                    if p2 == s { None } else { Some(p2) }
                )
            }
            None => (None, None)
        };

        // Since control points can be identical (too close to?) to their
        // nearest endpoint, we end up with four cases.
        match (u, v) {
            (Some(u), Some(v)) => {
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
                Segment::curve(rr, uu, vv, ss, None)
            }
            (None, Some(v)) => {
                // r and u are the same.
                //
                // We skip calculating uu and just set it rr.
                let wrv = v - r;
                let wvs = s - v;
                let rr = r + rot90(wrv).normalize() * offset;
                let ss = s + rot90(wvs).normalize() * offset;
                let vs = v + rot90(wvs).normalize() * offset;
                let vv = line_intersect(rr, wrv, vs, wvs);
                Segment::curve(rr, rr, vv, ss, None)
            }
            (Some(u), None) => {
                // v and s are the same.
                let wru = u - r;
                let wus = s - u;
                let rr = r + rot90(wru).normalize() * offset;
                let ss = s + rot90(wus).normalize() * offset;
                let us = u + rot90(wus).normalize() * offset;
                let uu = line_intersect(rr, wru, us, wus);
                Segment::curve(rr, uu, ss, ss, None)
            }
            (None, None) => {
                // Straight line.
                let d = rot90(s - r).normalize() * offset;
                Segment::line(r + d, s + d, None)
            }
        }
    }

    /// Applies the start of the segment to the canvas.
    pub fn apply_start(&self, canvas: &Canvas) {
        canvas.move_to(self.start.x, self.start.y);
    }

    /// Applies the tail of the segment to the canvas.
    pub fn apply_tail(&self, canvas: &Canvas) {
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
        match self.control {
            Some((c0, c1)) => {
                canvas.curve_to(
                    c0.x, c0.y,
                    c1.x, c1.y,
                    self.end.x, self.end.y,
                )
            }
            None => {
                canvas.line_to(self.end.x, self.end.y)
            }
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

