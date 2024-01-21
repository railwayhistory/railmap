
use std::fmt;
use femtomap::import::eval::{Builtin as _, LoadErrors};
use femtomap::import::path::{ImportPathSet, PathSetError};
use crate::config::Region;
use crate::feature::{Store, StoreBuilder};
use super::eval::Builtin;


//------------ LoadFeatures --------------------------------------------------

pub struct LoadFeatures {
    features: StoreBuilder,
    err: ImportError,
}

impl LoadFeatures {
    pub fn new() -> Self {
        LoadFeatures {
            features: Default::default(),
            err: Default::default(),
        }
    }

    pub fn load_region(
        &mut self,
        region: &Region,
    ) {
        let builtin = match ImportPathSet::load(&region.paths) {
            Ok(paths) => Builtin::new(paths),
            Err(err) => {
                self.err.paths.extend(err);
                return
            }
        };
        if let Err(err) = builtin.load(&region.rules) {
            self.err.rules.extend(err);
        }
    }

    pub fn finalize(
        self
    ) -> Result<Store, ImportError> {
        self.err.check()?;
        Ok(self.features.finalize())
    }
}


//------------ ImportError ---------------------------------------------------

#[derive(Default)]
pub struct ImportError {
    paths: PathSetError,
    rules: LoadErrors,
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

