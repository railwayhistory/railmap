/// What we are drawing on.
use std::ops;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use kurbo::{
    BezPath, PathEl, ParamCurve, ParamCurveArclen, PathSeg, Point, Rect,
    TranslateScale, Vec2
};
use crate::features::path::{CANVAS_ACCURACY, SegTime};
use crate::library::Style;


//------------ Configurable Constants ----------------------------------------

/// Size correction for feature bounds.
///
/// This value will be multiplied with detail level, then length and height of
/// the bounding box and then added on each side.
const BOUNDS_CORRECTION: f64 = 0.3;


//------------ Canvas --------------------------------------------------------

/// The virtual surface to draw the map on.
///
/// The type not only provides means for actual drawing, it also provides
/// access to its dimensions, resolution, and map projection.
///
/// Drawing is currently done directly via deref-ing to a cairo context.
/// This, however, may change later, so this should probably be done at
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
    equator_scale: f64,

    /// The size of a bp in canvas coordinates.
    canvas_bp: f64,

    /// The map style.
    style: Style,

    /// The font table.
    fonts: RefCell<FontTable>,
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
        mut style: Style,
    ) -> Self {
        // The size in storage coordinates.
        let feature_size = Point::new(size.x / scale, size.y / scale);

        // The bounds correction in storage coordinates.
        let correct = Point::new(
            feature_size.x * BOUNDS_CORRECTION * style.detail() as f64,
            feature_size.y * BOUNDS_CORRECTION * style.detail() as f64,
        );

        let canvas_bp = canvas_bp * style.mag();
        let context = cairo::Context::new(surface);
        context.move_to(0.,0.);
        context.line_to(size.x, 0.);
        context.line_to(size.x, size.y);
        context.line_to(0., size.y);
        context.close_path();
        context.clip();

        style.scale(canvas_bp);
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
            equator_scale: scale,
            canvas_bp,
            style,
            fonts: RefCell::new(FontTable::new()),
        }
    }

    /// Returns the feature bounds for the given parameters.
    pub fn calc_feature_bounds(
        size: Point, nw: Point, scale: f64
    ) -> Rect {
        // The size in storage coordinates.
        let feature_size = Point::new(size.x / scale, size.y / scale);

        // The bounds correction in storage coordinates.
        let correct = Point::new(
            feature_size.x * BOUNDS_CORRECTION,
            feature_size.y * BOUNDS_CORRECTION,
        );

        Rect::new(
            nw.x - correct.x,
            nw.y - correct.y,
            nw.x + feature_size.x + correct.x,
            nw.y + feature_size.y + correct.y,
        )
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

    /// Returns the map scale at the equator.
    pub fn equator_scale(&self) -> f64 {
        self.equator_scale
    }

    /// Returns the size of a _bp_ in canvas coordinates.
    pub fn canvas_bp(&self) -> f64 {
        self.canvas_bp
    }

    /// Returns the style.
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// Returns the detail level.
    pub fn detail(&self) -> u8 {
        self.style.detail()
    }

    pub fn apply_font(&self, face: FontFace, size: f64) {
        self.context.set_font_face(self.fonts.borrow_mut().get(face));
        self.set_font_size(size * self.canvas_bp());
    }

    pub fn mark_point(&self, point: Point) {
        self.set_source_rgb(1., 0., 0.);
        self.set_line_width(0.4 * self.canvas_bp());
        self.new_path();
        self.arc(
            point.x, point.y, self.canvas_bp(), 0., 2. * std::f64::consts::PI
        );
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


//------------ FontFamily ----------------------------------------------------

/// The font family of a font face.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FontFamily {
    FiraSans,
}

impl FontFamily {
    pub fn normal(self, slant: FontSlant, weight: FontWeight) -> FontFace {
        FontFace::new(self, FontStretch::default(), slant, weight)
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        FontFamily::FiraSans
    }
}


//------------ FontStretch ---------------------------------------------------

/// The stretch of the font
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FontStretch {
    Regular,
    Condensed,
}

impl Default for FontStretch {
    fn default() -> Self {
        FontStretch::Regular
    }
}


//------------ FontSlant -----------------------------------------------------

/// The slant of the font.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FontSlant {
    Upright,
    Italic,
}

impl Default for FontSlant {
    fn default() -> Self {
        FontSlant::Upright
    }
}


//------------ FontWeight ----------------------------------------------------

/// The weight of the font face.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FontWeight {
    Light,
    Book,
    Bold,
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Book
    }
}


//------------ FontFace ------------------------------------------------------

/// A font face.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FontFace {
    pub family: FontFamily,
    pub stretch: FontStretch,
    pub slant: FontSlant,
    pub weight: FontWeight,
}

impl FontFace {
    pub fn new(
        family: FontFamily, stretch: FontStretch,
        slant: FontSlant, weight: FontWeight,
    ) -> Self {
        FontFace { family, stretch, slant, weight }
    }

    pub fn bold() -> Self {
        FontFace::new(
            FontFamily::FiraSans, FontStretch::default(),
            FontSlant::default(), FontWeight::Bold,
        )
    }
}


//------------ FontTable -----------------------------------------------------

/// Global information shared by all canvases.
#[derive(Clone, Debug, Default)]
pub struct FontTable {
    font_faces: HashMap<FontFace, cairo::FontFace>,
}

impl FontTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&mut self, face: FontFace) -> &cairo::FontFace {
        self.font_faces.entry(face).or_insert_with(|| {
            Self::create_font(face)
        })
    }

    fn create_font(face: FontFace) -> cairo::FontFace {
        use cairo::FontSlant::{Italic, Normal};
        use cairo::FontWeight::{Bold, Normal as Book};

        const FONT_NORMAL: &str = "Fira Sans Book";
        const FONT_BOLD: &str = "Fira Sans";
        const FONT_LIGHT: &str = "Fira Sans ExtraLight";
        const FONT_NORMAL_COND: &str = "Fira Sans Condensed";
        const FONT_BOLD_COND: &str = "Fira Sans Condensed";
        const FONT_LIGHT_COND: &str = "Fira Sans Condensed Light";

        let (family, slant, weight) = match (face.family, face.stretch) {
            (FontFamily::FiraSans, FontStretch::Regular) => {
                match (face.slant, face.weight) {
                    (FontSlant::Upright, FontWeight::Book) => {
                        (FONT_NORMAL, Normal, Book)
                    }
                    (FontSlant::Upright, FontWeight::Bold) => {
                        (FONT_BOLD, Normal, Bold)
                    }
                    (FontSlant::Upright, FontWeight::Light) => {
                        (FONT_LIGHT, Normal, Book)
                    }
                    (FontSlant::Italic, FontWeight::Book) => {
                        (FONT_NORMAL, Italic, Book)
                    }
                    (FontSlant::Italic, FontWeight::Bold) => {
                        (FONT_BOLD, Italic, Bold)
                    }
                    (FontSlant::Italic, FontWeight::Light) => {
                        (FONT_LIGHT, Italic, Book)
                    }
                }
            }
            (FontFamily::FiraSans, FontStretch::Condensed) => {
                match (face.slant, face.weight) {
                    (FontSlant::Upright, FontWeight::Book) => {
                        (FONT_NORMAL_COND, Normal, Book)
                    }
                    (FontSlant::Upright, FontWeight::Bold) => {
                        (FONT_BOLD_COND, Normal, Bold)
                    }
                    (FontSlant::Upright, FontWeight::Light) => {
                        (FONT_LIGHT_COND, Normal, Book)
                    }
                    (FontSlant::Italic, FontWeight::Book) => {
                        (FONT_NORMAL_COND, Italic, Book)
                    }
                    (FontSlant::Italic, FontWeight::Bold) => {
                        (FONT_BOLD_COND, Italic, Bold)
                    }
                    (FontSlant::Italic, FontWeight::Light) => {
                        (FONT_LIGHT_COND, Italic, Book)
                    }
                }
            }
        };

        cairo::FontFace::toy_create(family, slant, weight)
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

