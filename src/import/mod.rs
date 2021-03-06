
pub mod ast;
pub mod eval;
pub mod features;
pub mod path;
pub mod mp_path;


use std::fmt;
use std::path::Path as FsPath;
use crate::features::FeatureSet;
use self::features::FeatureSetError;
use self::path::{PathSet, PathSetError};


pub fn load(import_dir: &FsPath) -> Result<FeatureSet, ImportError> {
    let paths = PathSet::load(&import_dir.join("paths"))?;
    features::load(&import_dir.join("rules"), &paths).map_err(Into::into)
}


//------------ ImportError ---------------------------------------------------

pub enum ImportError {
    Path(PathSetError),
    Feature(FeatureSetError),
}

impl From<PathSetError> for ImportError {
    fn from(err: PathSetError) -> Self {
        ImportError::Path(err)
    }
}

impl From<FeatureSetError> for ImportError {
    fn from(err: FeatureSetError) -> Self {
        ImportError::Feature(err)
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImportError::Path(ref err) => err.fmt(f),
            ImportError::Feature(ref err) => err.fmt(f),
        }
    }
}


//------------ Failed --------------------------------------------------------

/// A marker type indicating that an operation has failed.
///
/// This type is used as the error type of a result in cases where the actual
/// error has been been added to an error collection.
#[derive(Copy, Clone, Debug)]
pub struct Failed;

impl<T> From<Failed> for Result<T, Failed> {
    fn from(_: Failed) -> Result<T, Failed> {
        Err(Failed)
    }
}

