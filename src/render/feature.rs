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
        layer: f64,
        depth: usize,
    ) {
        let feature = StoredFeature::new(feature, detail, layer, depth);
        if let Some(bounds) = self.bounds.as_mut() {
            bounds.merge(&feature.bounds)
        }
        else {
            self.bounds = Some(feature.bounds)
        };
        self.features.insert(feature);
    }

    pub fn render(
        &self, style: &T::Style, canvas: &Canvas, bounds: Rect,
    ) {
        let features = self.locate(style.detail(), bounds);
        let mut features = features.as_slice();
        while let Some(layer) = features.first().map(|item| item.layer) {
            let mut max_depth = 1;
            let split = features.iter().enumerate().find_map(
                |(idx, item)| {
                    if item.layer == layer {
                        if item.depth > max_depth {
                            max_depth = item.depth;
                        }
                        None
                    }
                    else {
                        Some(idx)
                    }
                }
            );
            let (now, next) = match split {
                Some(split) => features.split_at(split),
                None => (features, [].as_ref())
            };
            for depth in (0..max_depth).rev() {
                for feature in now {
                    feature.feature.render(style, canvas, depth)
                }
            }
            features = next;
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
    ) -> Vec<&'a StoredFeature<T>> {
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
        res
    }
}


//------------ StoredFeature -------------------------------------------------

pub struct StoredFeature<T: Theme> {
    feature: T::Feature,
    layer: f64,
    bounds: AABB<[f64; 3]>,
    depth: usize,
}

impl<T: Theme> StoredFeature<T> {
    pub fn new(
        feature: T::Feature, detail: (u8, u8), layer: f64, depth: usize
    ) -> Self {
        let bounds = feature.storage_bounds();
        let detail = (f64::from(detail.0), f64::from(detail.1));
        StoredFeature {
            feature,
            layer,
            bounds: AABB::from_corners(
                [detail.0 - 0.4, bounds.x0, bounds.y0],
                [detail.1 + 0.4, bounds.x1, bounds.y1]
            ),
            depth,
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
    fn render(&self, style: &T::Style, canvas: &Canvas, depth: usize);
}

