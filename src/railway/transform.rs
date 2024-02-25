//! Transformations from storage into map space.

pub use femtomap::path::Transform;

/*
use kurbo::{Point, TranslateScale, Vec2};

//------------ Transform -----------------------------------------------------

#[derive(Clone, Copy, Debug, Default,)]
pub struct Transform {
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
}

impl Transform {
    /// Creates a new transform.
    ///
    /// The nort-west corner will be at `nw` in storage coordinates and the
    /// storage coordinates will be mulitplied by `scale` when translating
    /// into canvas coordinates.
    ///
    /// One _bp_ will be `canvas_bp` units in canvas coordinates.
    pub fn new(
        canvas_bp: f64,
        nw: Point,
        scale: f64,
    ) -> Self {
        Transform {
            transform: TranslateScale::new(
                Vec2::new(-nw.x * scale, -nw.y * scale),
                scale
            ),
            equator_scale: scale,
            canvas_bp,
        }
    }

    pub fn new_map_key(canvas_bp: f64) -> Self {
        Transform {
            transform: Default::default(),
            equator_scale: 1.,
            canvas_bp
        }
    }

    pub fn transform(self) -> TranslateScale {
        self.transform
    }

    pub fn equator_scale(self) -> f64 {
        self.equator_scale
    }

    pub fn canvas_bp(&self) -> f64 {
        self.canvas_bp
    }
}
*/
