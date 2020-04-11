/// Features are things that should be shown on the map.
///

pub use self::color::Color;
pub use self::contour::{Contour, RenderContour};
pub use self::path::Path;

pub mod color;
pub mod contour;
pub mod path;


use std::ops;
use kurbo::Rect;
use rstar::{AABB, RTree, RTreeObject};
use crate::canvas::Canvas;


//------------ FeatureSet ----------------------------------------------------

#[derive(Default)]
pub struct FeatureSet {
    features: RTree<StoredFeature>,
}

impl FeatureSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(
        &mut self,
        feature: impl Into<Feature>,
        detail: (u8, u8),
        layer: f64
    ) {
        self.features.insert(
            StoredFeature::new(feature.into(), detail, layer)
        )
    }

    pub fn render(&self, canvas: &Canvas) {
        for feature in self.locate(canvas.detail(), canvas.feature_bounds()) {
            feature.render(canvas)
        }
    }

    pub fn locate<'a>(
        &'a self, detail: u8, bounds: Rect,
    ) -> impl Iterator<Item = &'a StoredFeature> {
        let detail = f64::from(detail);
        let mut res: Vec<_> = self.features.locate_in_envelope_intersecting(
            &AABB::from_corners(
                [detail - 0.2, bounds.x0, bounds.y0],
                [detail + 0.2, bounds.x1, bounds.y1]
            )
        ).collect();
        res.sort_unstable_by(|left, right| {
            left.layer.partial_cmp(&right.layer).unwrap()
        });
        res.into_iter()
    }
}


//------------ StoredFeature -------------------------------------------------

pub struct StoredFeature {
    feature: Feature,
    layer: f64,
    bounds: AABB<[f64; 3]>
}

impl StoredFeature {
    pub fn new(feature: Feature, detail: (u8, u8), layer: f64) -> Self {
        let bounds = feature.bounding_box();
        let detail = if detail.0 < detail.1 {
            (f64::from(detail.0), f64::from(detail.1))
        }
        else {
            (f64::from(detail.1), f64::from(detail.0))
        };
        StoredFeature {
            feature,
            layer,
            bounds: AABB::from_corners(
                [detail.0 - 0.5, bounds.x0, bounds.y0],
                [detail.1 + 0.5, bounds.x1, bounds.y1]
            )
        }
    }
}

impl RTreeObject for StoredFeature {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

impl ops::Deref for StoredFeature {
    type Target = Feature;

    fn deref(&self) -> &Self::Target {
        &self.feature
    }
}


//------------ Feature -------------------------------------------------------

pub enum Feature {
    Contour(Contour),
}

impl Feature {
    pub fn bounding_box(&self) -> Rect {
        match *self {
            Feature::Contour(ref contour) => contour.bounding_box(),
        }
    }

    pub fn render(&self, canvas: &Canvas) {
        match *self {
            Feature::Contour(ref contour) => contour.render(canvas)
        }
    }
}


//--- From

impl From<Contour> for Feature {
    fn from(contour: Contour) -> Feature {
        Feature::Contour(contour)
    }
}

