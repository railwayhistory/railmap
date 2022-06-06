//! Features are things that should be shown on the map.

use kurbo::Rect;
use rstar::{AABB, Envelope, RTree, RTreeObject};
use crate::theme::{Style, Theme};
use super::canvas::Canvas;


//------------ FeatureSet ----------------------------------------------------

pub struct FeatureSet<T: Theme> {
    features: RTree<StoredFeature<T>>,
    bounds: Option<AABB<[f64; 3]>>
}

impl<T: Theme> Default for FeatureSet<T> {
    fn default() -> Self {
        FeatureSet {
            features: RTree::default(),
            bounds: None
        }
    }
}

impl<T: Theme> FeatureSet<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(
        &mut self,
        feature: T::Feature,
        detail: (u8, u8),
        layer: f64
    ) {
        let feature = StoredFeature::new(feature, detail, layer);
        if let Some(bounds) = self.bounds.as_mut() {
            bounds.merge(&feature.bounds)
        }
        else {
            self.bounds = Some(feature.bounds)
        };
        self.features.insert(feature);
    }

    pub fn render(&self, style: &T::Style, canvas: &Canvas) {
        for feature in self.locate(style.detail(), canvas.feature_bounds()) {
            feature.feature.render(style, canvas)
        }
    }

    pub fn is_covered(&self, detail: u8, bounds: Rect) -> bool {
        match self.bounds {
            Some(feature_bounds) => {
                let detail = f64::from(detail);
                feature_bounds.contains_envelope(
                    &AABB::from_corners(
                        [detail - 0.2, bounds.x0, bounds.y0],
                        [detail + 0.2, bounds.x1, bounds.y1]
                    )
                )
            }
            None => false
        }
    }

    pub fn locate<'a>(
        &'a self, detail: u8, bounds: Rect,
    ) -> impl Iterator<Item = &'a StoredFeature<T>> {
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

pub struct StoredFeature<T: Theme> {
    feature: T::Feature,
    layer: f64,
    bounds: AABB<[f64; 3]>
}

impl<T: Theme> StoredFeature<T> {
    pub fn new(feature: T::Feature, detail: (u8, u8), layer: f64) -> Self {
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

impl<T: Theme> RTreeObject for StoredFeature<T> {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}


//------------ Feature -------------------------------------------------------

pub trait Feature<T: Theme> {
    fn storage_bounds(&self) -> Rect;
    fn render(&self, style: &T::Style, canvas: &Canvas);
}
