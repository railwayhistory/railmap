/// Features are things that should be shown on the map.
use std::sync::Arc;
use rstar::{AABB, RTree, RTreeObject};
use crate::mp_path;
use crate::path::{Path, PathSet};
//use crate::render::Curve;
use crate::tile::TileId;

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

    pub fn locate<'a>(
        &'a self, zoom: u8, lon: [f64; 2], lat: [f64; 2]
    ) -> impl Iterator<Item = &'a Feature> {
        let zoom = f64::from(zoom);
        self.features.locate_in_envelope_intersecting(&AABB::from_corners(
            [zoom - 0.2, lon[0], lat[0]], [zoom + 0.2, lon[1], lat[1]]
        ))
    }
}


//------------ Feature -------------------------------------------------------

pub struct Feature {
    path: Arc<Path>,
    bounds: AABB<Point>,
}

impl Feature {
    fn new(path: Arc<Path>) -> Self {
        let mut lower = [-0.5, std::f64::MAX, std::f64::MAX];
        let mut upper = [20.5, std::f64::MIN, std::f64::MIN];
        for node in path.nodes() {
            if node.lon < lower[1] {
                lower[1] = node.lon
            }
            if node.lon > upper[1] {
                upper[1] = node.lon
            }
            if node.lat < lower[2] {
                lower[2] = node.lat
            }
            if node.lat > upper[2] {
                upper[2] = node.lat
            }
        }

        Feature {
            path,
            bounds: AABB::from_corners(lower, upper)
        }
    }

    pub fn render(&self, context: &cairo::Context, tile: &TileId) {
        context.set_source_rgb(0., 0., 0.);
        // Curve::new(&self.path, tile).apply(context);
        mp_path::render(&self.path, context, tile);
        context.stroke();
    }
}

impl RTreeObject for Feature {
    type Envelope = AABB<Point>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds.clone()
    }
}


//------------ Point ---------------------------------------------------------

// [zoom, lon, lat];
type Point = [f64; 3];

