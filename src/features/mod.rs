/// Features are things that should be shown on the map.
///

pub use self::color::Color;
pub use self::contour::{Contour, ContourRule};
pub use self::label::{Font, Label, Layout};
pub use self::path::{Distance, Location, Path, Position};
pub use self::marker::{Marker, MarkerRule};

pub mod color;
pub mod contour;
pub mod label;
//pub mod label2;
pub mod path;
pub mod marker;


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
        let bounds = feature.storage_bounds();
        let detail = (f64::from(detail.0), f64::from(detail.1));
        StoredFeature {
            feature,
            layer,
            bounds: AABB::from_corners(
                [detail.0 - 0.4, bounds.x0, bounds.y0],
                [detail.1 + 0.4, bounds.x1, bounds.y1]
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
    Label(Label),
    Marker(Marker),
}

impl Feature {
    pub fn storage_bounds(&self) -> Rect {
        match *self {
            Feature::Contour(ref contour) => contour.storage_bounds(),
            Feature::Label(ref label) => label.storage_bounds(),
            Feature::Marker(ref marker) => marker.storage_bounds(),
        }
    }

    pub fn render(&self, canvas: &Canvas) {
        match *self {
            Feature::Contour(ref contour) => contour.render(canvas),
            Feature::Label(ref label) => label.render(canvas),
            Feature::Marker(ref marker) => marker.render(canvas),
        }
    }
}


//--- From

impl From<Contour> for Feature {
    fn from(contour: Contour) -> Feature {
        Feature::Contour(contour)
    }
}

impl From<Label> for Feature {
    fn from(label: Label) -> Feature {
        Feature::Label(label)
    }
}

impl From<Marker> for Feature {
    fn from(marker: Marker) -> Feature {
        Feature::Marker(marker)
    }
}

