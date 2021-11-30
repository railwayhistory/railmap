
pub mod ast;
pub mod eval;
pub mod features;
pub mod path;
pub mod mp_path;


use std::fmt;
use crate::config::Region;
use crate::features::FeatureSet;
use self::eval::Scope;
use self::features::FeatureSetError;
use self::path::{PathSet, PathSetError};


#[derive(Default)]
pub struct LoadFeatures {
    features: FeatureSet,
    err: ImportError,
}

impl LoadFeatures {
    pub fn load_region(
        &mut self,
        region: &Region,
    ) {
        let paths = match PathSet::load(&region.paths) {
            Ok(paths) => paths,
            Err(err) => {
                self.err.paths.extend(err);
                return
            }
        };
        features::load_dir(
            &region.rules, Scope::new(&paths),
            &mut self.features, &mut self.err.rules
        );
    }

    pub fn finalize(
        self
    ) -> Result<FeatureSet, ImportError> {
        self.err.check()?;
        Ok(self.features)
    }
}


//------------ ImportError ---------------------------------------------------

#[derive(Default)]
pub struct ImportError {
    paths: PathSetError,
    rules: FeatureSetError,
}

impl ImportError {
    fn check(self) -> Result<(), Self> {
        if !self.paths.is_empty() || !self.rules.is_empty() {
            Err(self)
        }
        else {
            Ok(())
        }
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.paths.fmt(f)?;
        self.rules.fmt(f)
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

