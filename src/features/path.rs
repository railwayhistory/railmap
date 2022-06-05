/// Paths.
///

pub use self::stored::{
    Distance, Location, SegTime, StoredPath, StoredPathBuilder
};
pub use self::segment::Segment;

mod stored;
mod segment;


use std::slice;
use std::cmp::Ordering;
use std::f64::consts::PI;
use kurbo::{Point, Rect, Vec2};
use crate::canvas;
use crate::canvas::Canvas;


//------------ Configuration Constants ---------------------------------------

/// Accuracy for Kurbo arclen calculations in storage coordinates.
///
/// This value should provide centimetre accuracy in storage coordinates.
const STORAGE_ACCURACY: f64 = 1E-11;

/// Accuracy for Kurbo arclen calculations in canvas coordinates.
///
/// This value assumes about 192 dpi device resolution.
pub(crate) const CANVAS_ACCURACY: f64 = 0.025;


//------------ Path ----------------------------------------------------------

/// A path.
///
/// This path is constructed as a sequence of connected subpaths referencing
/// `StoredPath`s.
#[derive(Clone, Debug, Default)]
pub struct Path {
    /// The sequence of parts.
    ///
    /// The first two elements are the tensions when leaving the previous
    /// part and entering this part. The third element is the part itself.
    parts: Vec<(f64, f64, Section)>,
}

impl Path {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_subpath(&mut self, post: f64, pre: f64, section: Subpath) {
        self.parts.push((post, pre, Section::Subpath(section)))
    }

    pub fn push_line(&mut self, post: f64, pre: f64, section: Line) {
        self.parts.push((post, pre, Section::Line(section)))
    }

    pub fn push_path(&mut self, post: f64, pre: f64, path: &Path) {
        let first = match path.parts.first() {
            Some(first) => first,
            None => return
        };
        self.parts.push((post, pre, first.2.clone()));
        self.parts.extend_from_slice(&path.parts[1..]);
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

    fn parts(&self) -> PathPartsIter {
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
type PathPartsIter<'a> = slice::Iter<'a, (f64, f64, Section)>;


//------------ PathSegmentIter -----------------------------------------------

/// An iterator over the segments in a path.
#[derive(Clone, Debug)]
pub struct PathSegmentIter<'a> {
    /// An iterator producing the next part of the path.
    next_part: PathPartsIter<'a>,

    /// An iterator producing the next segment of the current part.
    ///
    /// If this is `None`, we need a new part.
    next_seg: SectionSegmentIter<'a>,

    /// The last segment we returned.
    ///
    /// This is necessary to build the connection between parts.
    last_seg: Option<Segment>,

    /// The canvas to use for transforming the path.
    canvas: &'a Canvas,
}

impl<'a> PathSegmentIter<'a> {
    fn new(path: &'a Path, canvas: &'a Canvas) -> Self {
        let mut next_part = path.parts();
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


//------------ Section -------------------------------------------------------

/// A section of a path.
#[derive(Clone, Debug)]
enum Section {
    Subpath(Subpath),
    Line(Line),
}

impl Section {
    fn storage_bounds(&self) -> Rect {
        match *self {
            Section::Subpath(ref section) => section.storage_bounds(),
            Section::Line(ref section) => section.storage_bounds(),
        }
    }

    fn iter<'a>(&'a self, canvas: &'a Canvas) -> SectionSegmentIter<'a> {
        match *self {
            Section::Subpath(ref subpath) => {
                SectionSegmentIter::Subpath(subpath.iter(canvas))
            }
            Section::Line(ref line) => {
                SectionSegmentIter::Line(line.iter(canvas))
            }
        }
    }
}


//------------ SectionSegmentIter --------------------------------------------

#[derive(Clone, Debug)]
enum SectionSegmentIter<'a> {
    Subpath(SubpathSegmentIter<'a>),
    Line(LineSegmentIter<'a>)
}

impl<'a> Iterator for SectionSegmentIter<'a> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            SectionSegmentIter::Subpath(ref mut section) => section.next(),
            SectionSegmentIter::Line(ref mut section) => section.next(),
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

    pub fn eval_full(path: StoredPath) -> Self {
        let start = path.min_location();
        let end = path.max_location();
        Subpath::new(path, start, end, None)
    }

    pub fn eval(
        path: StoredPath,
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
            Ordering::Equal => {
               self.path.segment_after(start).bounds()
            }
            Ordering::Less => {
                let mut res = self.path.segment_after(start).bounds();
                for seg in start.seg + 1..end.seg {
                    res = res.union(self.path.segment(seg).unwrap().bounds());
                }
                res.union(self.path.segment_before(end).bounds())
            }
            Ordering::Greater => {
                let mut res = self.path.segment_before(start).bounds();
                for seg in end.seg + 1..start.seg {
                    res = res.union(self.path.segment(seg).unwrap().bounds());
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
                    subpath.path.segment(start.seg).unwrap().sub(
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
            subpath, canvas,
            forward: true,
            first, middle, last
        }
    }

    fn new_reverse(
        subpath: &'a Subpath, canvas: &'a Canvas,
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
            let seg = self.subpath.path.segment(start).unwrap().transf_off(
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
            let seg = self.subpath.path.segment(
                start
            ).unwrap().rev().transf_off(
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


//------------ Line ----------------------------------------------------------

/// A straight line between two positions.
#[derive(Clone, Debug)]
pub struct Line {
    start: Position,
    end: Position
}

impl Line {
    pub fn new(start: Position, end: Position) -> Self {
        Line { start, end }
    }

    fn storage_bounds(&self) -> Rect {
        self.start.storage_bounds().union(
            self.end.storage_bounds()
        )
    }

    fn iter<'a>(&'a self, canvas: &'a Canvas) -> LineSegmentIter<'a> {
        LineSegmentIter {
            line: Some(self),
            canvas
        }
    }
}


//------------ LineSegmentIter -----------------------------------------------

#[derive(Clone, Debug)]
struct LineSegmentIter<'a> {
    line: Option<&'a Line>,
    canvas: &'a Canvas,
}

impl<'a> Iterator for LineSegmentIter<'a> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.line.take()?;
        let start = line.start.resolve(self.canvas).0;
        let end = line.end.resolve(self.canvas).0;
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
    path: StoredPath,

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
        path: StoredPath,
        location: Location,
        sideways: Option<Distance>,
        shift: Option<(Distance, Distance)>,
        rotation: Option<f64>,
    ) -> Self {
        Position { path, location, sideways, shift, rotation }
    }

    pub fn eval(
        path: StoredPath,
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

    pub fn storage_bounds(&self) -> Rect {
        let p = self.path.segment_after(
            self.location.world
        ).p0();
        (p, p).into()
    }

    pub fn resolve(&self, canvas: &Canvas) -> (Point, f64) {
        let loc = self.path.location_time(self.location, canvas);
        let seg = self.path.segment(loc.seg).unwrap();
        let storage_point = seg.point(loc.time);
        let dir = seg.dir(loc.time);
        let shift = self.shift.map(|shift| {
            Vec2::new(
                shift.0.resolve(storage_point, canvas),
                shift.1.resolve(storage_point, canvas)
            )
        });
        let mut point = canvas.transform() * storage_point;
        let angle = dir.atan2() + self.rotation.unwrap_or(0.);
        if let Some(sideways) = self.sideways {
            let sideways= sideways.resolve(storage_point, canvas);
            let dir = sideways * rot90(dir).normalize();
            point += dir;
        }
        if let Some(shift) = shift {
            point += shift
        }
        (point, angle)
    }

    pub fn resolve_label(
        &self, canvas: &Canvas, on_path: bool
    ) -> (Point, f64) {
        if on_path {
            let (point, mut angle) = self.resolve(canvas);

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
            (self.resolve(canvas).0, self.rotation.unwrap_or(0.))
        }
    }
}


//------------ Helper Functions ----------------------------------------------

/// Rotates a vector by 90°.
fn rot90(vec: Vec2) -> Vec2 {
    Vec2::new(vec.y, -vec.x)
}

