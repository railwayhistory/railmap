/// What we are drawing on.
use std::ops;
use std::convert::TryInto;
use kurbo::{
    BezPath, PathEl, ParamCurve, ParamCurveArclen, PathSeg, Point, Rect,
    TranslateScale, Vec2
};
use crate::features::path::{CANVAS_ACCURACY, SegTime};


//------------ Configurable Constants ----------------------------------------

/// Size correction for feature bounds.
///
/// This value will be multiplied with length and height of the bounding box
/// and then added on each side.
const BOUNDS_CORRECTION: f64 = 0.5;


//------------ Canvas --------------------------------------------------------

/// The virtual surface to draw the map on.
///
/// The type not only provides means for actual drawing, it also provides
/// access to it dimensions, resolution, and map projection.
///
/// Drawing is currently done directly via deref-ing to a cairo context for
/// now. This, however, may change later, so this should probably be done at
/// as few places as possible.
///
/// The canvas keeps its bounding box in storage coordinates for selecting
/// features. This box is a little bigger than the canvas’s own extend to
/// correct for features that can only provide approximate bounds.
///
/// The canvas also provides the transformation for converting storage
/// coordinates into canvas coordinates and a measure for its resolution,
/// with is the size of a _bp_ (i.e., a ’PostScript point’).
#[derive(Debug)]
pub struct Canvas {
    /// The Cairo context for actual rendering.
    context: cairo::Context,

    /// The feature bounding box.
    ///
    /// This is in storage coordinates and should be big enough that all
    /// features with approximate bounds are covered, too.
    feature_bounds: Rect,

    /// The transformation from storage to canvas coordinates.
    ///
    /// Storage coordinates are Spherical Mercator with a range of `0. .. 1.`
    /// for both x and y. Because we are only supporting Spherical Mercator
    /// for output, too, we can use scaling and translation for this.
    ///
    /// Note that in a `TranslateScale` the scaling happens first and the
    /// translation needs to be in scaled up coordinates.
    transform: TranslateScale,

    /// The size of a bp in storage coordinates.
    storage_bp: f64,

    /// The size of a bp in canvas coordinates.
    canvas_bp: f64,

    /// Detail level.
    detail: u8,

    fira: cairo::FontFace
}

impl Canvas {
    /// Creates a new canvas.
    ///
    /// The canvas will have a size of `size` units in canvas coordinates.
    /// One _bp_ will be `canvas_bp` units in canvas coordinates.
    ///
    /// The nort-west corner will be at `nw` in storage coordinates and the
    /// storage coordinates will be mulitplied by `scale` when translating
    /// into canvas coordinates.
    pub fn new(
        surface: &cairo::Surface,
        size: Point,
        canvas_bp: f64,
        nw: Point,
        scale: f64,
        detail: u8,
    ) -> Self {
        // The size in storage coordinates.
        let feature_size = Point::new(size.x / scale, size.y / scale);

        // The bounds correction in storage coordinates.
        let correct = Point::new(
            feature_size.x * BOUNDS_CORRECTION,
            feature_size.y * BOUNDS_CORRECTION,
        );

        let context = cairo::Context::new(surface);
        context.move_to(0.,0.);
        context.line_to(size.x, 0.);
        context.line_to(size.x, size.y);
        context.line_to(0., size.y);
        context.close_path();
        context.clip();

        Canvas {
            context,
            feature_bounds: Rect::new(
                nw.x - correct.x,
                nw.y - correct.y,
                nw.x + feature_size.x + correct.x,
                nw.y + feature_size.y + correct.y,
            ),
            transform: TranslateScale::new(
                Vec2::new(-nw.x * scale, -nw.y * scale),
                scale
            ),
            storage_bp: canvas_bp / scale,
            canvas_bp,
            detail,
            fira: cairo::FontFace::toy_create(
                "Fira Sans",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            ),
        }
    }

    /// Returns a reference to the Cairo rendering context.
    pub fn context(&self) -> &cairo::Context {
        &self.context
    }

    /// Returns the feature bounding box.
    ///
    /// This is the bounding box of the canvase in storage coordinates and
    /// can be used to select the feature to render onto the canvas. The make
    /// sure all features are selected, it has been inflated and is larger
    /// than the actual extent of the canvase.
    pub fn feature_bounds(&self) -> Rect {
        self.feature_bounds
    }

    /// Returns the feature transformation.
    ///
    /// This is the transformation that needs to be applied to all features
    /// before rendering them onto the canvas.
    pub fn transform(&self) -> TranslateScale {
        self.transform
    }

    /// Returns the size of a _bp_ at the equator in storage coordinates.
    pub fn storage_bp(&self) -> f64 {
        self.storage_bp
    }

    /// Returns the size of a _bp_ in canvas coordinates.
    pub fn canvas_bp(&self) -> f64 {
        self.canvas_bp
    }

    /// Returns the detail level.
    pub fn detail(&self) -> u8 {
        self.detail
    }

    pub fn fira(&self) -> &cairo::FontFace {
        &self.fira
    }

    pub fn mark_point(&self, point: Point) {
        self.set_source_rgb(1., 0., 0.);
        self.set_line_width(0.4 * self.canvas_bp());
        self.new_path();
        self.arc(point.x, point.y, self.canvas_bp(), 0., 2. * std::f64::consts::PI);
        self.stroke();
    }
}


//--- Deref

impl ops::Deref for Canvas {
    type Target = cairo::Context;

    fn deref(&self) -> &Self::Target {
        self.context()
    }
}


//------------ Path ----------------------------------------------------------

/// A path bound to a canvas.
///
/// The path provides a number of convenient method for manipulation. It can
/// also be applied to its canvas at any time.
///
/// All the path’s points are in canvas coordinates. All lengths are canvas
/// lengths in _bp_.
#[derive(Clone, Debug)]
pub struct Path<'a> {
    path: BezPath,
    canvas: &'a Canvas,
}

impl<'a> Path<'a> {
    pub fn new(canvas: &'a Canvas) -> Self {
        Path {
            path: BezPath::new(),
            canvas
        }
    }

    pub fn move_to(&mut self, p: Point) {
        self.path.move_to(p);
    }

    pub fn line_to(&mut self, p: Point) {
        self.path.line_to(p);
    }

    pub fn curve_to(&mut self, p0: Point, p1: Point, p2: Point) {
        self.path.curve_to(p0, p1, p2)
    }

    pub fn line_append(&mut self, path: &Path) {
        let mut segs = path.path.segments();
        if let Some(seg) = segs.next() {
            self.path.line_to(first_point(seg));
            self.append_seg(seg);
        }
        for seg in segs {
            self.append_seg(seg);
        }
    }

    pub fn curve_append(&mut self, p1: Point, p2: Point, path: &Path) {
        let mut segs = path.path.segments();
        if let Some(seg) = segs.next() {
            self.path.curve_to(p1, p2, first_point(seg));
            self.append_seg(seg);
        }
        for seg in segs {
            self.append_seg(seg);
        }
    }

    pub fn apply(&self) {
        self.path.iter().for_each(|el| match el {
            PathEl::MoveTo(p) => self.canvas.move_to(p.x, p.y),
            PathEl::LineTo(p) => self.canvas.line_to(p.x, p.y),
            PathEl::QuadTo(..) => unreachable!(),
            PathEl::CurveTo(u, v, s) => {
                self.canvas.curve_to(u.x, u.y, v.x, v.y, s.x, s.y)
            }
            PathEl::ClosePath => self.canvas.close_path(),
        })
    }
}

impl<'a> Path<'a> {
    /// Returns the number of nodes in the path.
    pub fn node_len(&self) -> u32 {
        self.path.elements().len().try_into().unwrap()
    }

    /// Returns the arc length of the path.
    pub fn arclen(&self) -> f64 {
        self.path.segments().fold(0., |len, seg| {
            len + seg.arclen(CANVAS_ACCURACY)
        })
    }

    /// Returns the path time where the arc length reaches the given value.
    ///
    /// If `arclen` is greater than the path’s arc length, returns the time
    /// value of the end of the path.
    pub fn arctime(&self, arclen: f64) -> f64 {
        let mut arclen = arclen * self.canvas.canvas_bp();
        let mut i = 0.; // avoid int-to-float conversion 
        for seg in self.path.segments() {
            let seg_arclen = seg.arclen(CANVAS_ACCURACY);
            if seg_arclen > arclen {
                let time = seg.inv_arclen(arclen, CANVAS_ACCURACY);
                return i + time;
            }
            arclen -= seg_arclen;
            i += 1.
        }
        i
    }

    /// Returns the subpath between the two given path times.
    pub fn subpath(&self, start_time: f64, end_time: f64) -> Self {
        let mut start = self.resolve_time(start_time);
        let end = self.resolve_time(end_time);
        let mut res = Path::new(self.canvas);
        if start.seg == end.seg {
            let seg = self.get_seg(start).subsegment(start.time..end.time);
            res.move_to_seg(seg);
            res.append_seg(seg);
        }
        else if start <= end {
            let first = self.get_seg(start).subsegment(start.time..1.);
            res.move_to_seg(first);
            res.append_seg(first);
            start.seg += 1;
            while start.seg < end.seg {
                res.append_seg(self.get_seg(start))
            }
            let last = self.get_seg(end).subsegment(0. .. end.time);
            res.append_seg(last);
        }
        else {
            let first = self.get_seg(start)
                .subsegment(0. .. start.time)
                .reverse();
            res.move_to_seg(first);
            res.append_seg(first);
            start.seg -= 1;
            while start.seg > end.seg {
                res.append_seg(self.get_seg(start).reverse())
            }
            let last = self.get_seg(end)
                .subsegment(end.time .. 1.)
                .reverse();
            res.append_seg(last);
        }
        res
    }
}

/// # Internal Helpers
///
impl<'a> Path<'a> {
    /// Resolves path time into a location.
    ///
    /// The integer part of the path time denotes the segment as one less the
    /// segment index. The fractional part of the path time denotes the time
    /// on the segment.
    ///
    /// Negative path times are truncated to zero. Path times beyond the end
    /// of the path are truncated to the end of the path.
    fn resolve_time(&self, time: f64) -> SegTime {
        if time < 0. {
            return SegTime::new(0, 0.)
        }

        // Safely convert the integer part to a u32. Avoid current undefined
        // behaviour in float-to-int conversion.
        let seg = if time >= std::u32::MAX as f64 { std::u32::MAX - 1 }        
        else { time as u32 };
        
        let seg = seg + 1;
        let time = time.fract();

        if seg >= self.node_len() {
            SegTime::new(self.node_len() - 1, 1.)
        }
        else {
            SegTime::new(seg, time)
        }
    }
    
    /// Returns the complete path segment with the given index.
    fn get_seg(&self, loc: SegTime) -> PathSeg {
        self.path.get_seg(loc.seg as usize).unwrap()
    }

    /// Moves to the beginning of the segment.
    fn move_to_seg(&mut self, seg: PathSeg) {
        self.path.move_to(match seg {
            PathSeg::Line(line) => line.p0,
            PathSeg::Quad(..) => unreachable!(),
            PathSeg::Cubic(cubic) => cubic.p0
        })
    }

    /// Appends the tail end of the segment.
    ///
    /// This assumes that the last point on the path is already the start
    /// point of the segment.
    fn append_seg(&mut self, seg: PathSeg) {
        match seg {
            PathSeg::Line(line) => self.path.line_to(line.p1),
            PathSeg::Quad(..) => unreachable!(),
            PathSeg::Cubic(cubic) => {
                self.path.curve_to(cubic.p1, cubic.p2, cubic.p3)
            }
        }
    }
}


//------------ Helper Functions ----------------------------------------------

fn first_point(seg: PathSeg) -> Point {
    match seg {
        PathSeg::Line(line) => line.p0,
        PathSeg::Quad(..) => unreachable!(),
        PathSeg::Cubic(cubic) => cubic.p0
    }
}

