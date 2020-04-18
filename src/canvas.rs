/// What we are drawing on.
use std::ops;
use kurbo::{Point, Rect, TranslateScale, Vec2};

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

        Canvas {
            context: cairo::Context::new(surface),
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
}


//--- Deref

impl ops::Deref for Canvas {
    type Target = cairo::Context;

    fn deref(&self) -> &Self::Target {
        self.context()
    }
}

