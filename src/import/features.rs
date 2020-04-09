
use std::path::Path as FsPath;
use crate::features::FeatureSet;
use crate::features::{Color, Contour};
use super::path::PathSet;


pub fn load(
    _feature_dir: &FsPath, paths: &PathSet
) -> Result<FeatureSet, FeatureSetError> {
    let mut features = FeatureSet::new();

    for (_, path) in paths.iter() {
        features.insert(
            Contour::simple(path.path().clone(), Color::RED, 0.2),
            (0, 20)
        )
    }

    Ok(features)
}


//------------ FeatureSetError -----------------------------------------------

pub struct FeatureSetError;

