/// Features are things that should be shown on the map.
use std::sync::Arc;
use kurbo::Rect;
use rstar::{AABB, RTree, RTreeObject};
use crate::path::{Path, PathSet};
use crate::canvas::Canvas;


//------------ FeatureSet ----------------------------------------------------

pub struct FeatureSet {
    features: RTree<Feature>,
}

impl FeatureSet {
    pub fn load(paths: &PathSet) -> Self {
        let mut features = RTree::new();
        for path in paths.iter() {
            features.insert(Feature::new(path.1.clone()))
        }
        FeatureSet { features }
    }

    pub fn render(&self, canvas: &Canvas) {
        for feature in self.locate(canvas.detail(), canvas.feature_bounds()) {
            feature.render(canvas)
        }
    }

    pub fn locate<'a>(
        &'a self, detail: u8, bounds: Rect,
    ) -> impl Iterator<Item = &'a Feature> {
        let detail = f64::from(detail);
        self.features.locate_in_envelope_intersecting(&AABB::from_corners(
            [detail - 0.2, bounds.x0, bounds.y0],
            [detail + 0.2, bounds.x1, bounds.y1]
        ))
    }
}


//------------ Feature -------------------------------------------------------

pub struct Feature {
    path: Arc<Path>,
    bounds: AABB<[f64; 3]>,
}

impl Feature {
    fn new(path: Arc<Path>) -> Self {
        let bounds = path.curve().bounding_box();

        Feature {
            path,
            bounds: AABB::from_corners(
                [-0.5, bounds.x0, bounds.y0],
                [20.5, bounds.x1, bounds.y1]
            )
        }
    }

    pub fn render(&self, canvas: &Canvas) {
        canvas.set_source_rgb(0., 0., 0.);
        self.path.curve().apply(canvas);
        canvas.stroke();
    }
}

impl RTreeObject for Feature {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds.clone()
    }
}


