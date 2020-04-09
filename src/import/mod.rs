
pub mod features;
pub mod path;
pub mod mp_path;


use std::path::Path as FsPath;
use crate::features::FeatureSet;
use self::features::FeatureSetError;
use self::path::{PathSet, PathSetError};


pub fn load(import_dir: &FsPath) -> Result<FeatureSet, ImportError> {
    let paths = PathSet::load(&import_dir.join("paths"))?;
    features::load(&import_dir.join("map"), &paths).map_err(Into::into)
}


//------------ ImportError ---------------------------------------------------

pub struct ImportError;

impl From<PathSetError> for ImportError {
    fn from(_err: PathSetError) -> Self {
        ImportError
    }
}

impl From<FeatureSetError> for ImportError {
    fn from(_err: FeatureSetError) -> Self {
        ImportError
    }
}

