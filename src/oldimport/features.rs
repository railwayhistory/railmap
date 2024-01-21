
use std::{fmt, fs, io};
use std::path::Path as FsPath;
use femtomap::feature::FeatureSetBuilder;
use crate::theme::Theme;
use super::{ast, eval};
use super::path::PathSet;
use super::ast::StatementList;
use super::eval::Scope;


pub fn load<T: Theme>(
    theme: T, feature_dir: &FsPath, paths: &PathSet
) -> Result<FeatureSetBuilder<T::Feature>, FeatureSetError> {
    let mut features = FeatureSetBuilder::new();
    let mut err = FeatureSetError::default();
    
    load_dir::<T>(
        feature_dir, Scope::new(theme, paths), &mut features, &mut err
    );

    err.check()?;
    Ok(features)
}

pub fn load_dir<T: Theme>(
    path: &FsPath,
    mut context: Scope<T>,
    target: &mut FeatureSetBuilder<T::Feature>,
    err: &mut FeatureSetError,
) {
    // Before we do anything else, we run init.map if it is present on the
    // context so that it can make global definitions.
    let init = path.join("init.map");
    if fs::metadata(&init)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
    {
        load_file(&init, &mut context, target, err);
    }

    // Now we walk over the directory and load directories and all .map files.
    let dir = match fs::read_dir(path) {
        Ok(dir) => dir,
        Err(e) => {
            err.push(path, e);
            return;
        }
    };
    for entry in dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue, // Silently skip these errors.
        };
        let ftype = match entry.file_type() {
            Ok(ftype) => ftype,
            Err(_) => continue, // And these, too.
        };
        if ftype.is_dir() {
            load_dir::<T>(&entry.path(), context.clone(), target, err);
        }
        else if ftype.is_file() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let name = name.to_str().unwrap_or(&"");
                if name.starts_with(".") || name == "init.map" {
                    continue
                }
            }
            if path.extension().and_then(|s| s.to_str()) == Some("map") {
                load_file(&path, &mut context.clone(), target, err);
            }
        }
    }
}

fn load_file<T: Theme>(
    path: &FsPath,
    scope: &mut Scope<T>,
    target: &mut FeatureSetBuilder<T::Feature>,
    err: &mut FeatureSetError,
) {
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            err.push(path, e);
            return
        }
    };
    let stm = match StatementList::parse_str(&data) {
        Ok(stm) => stm,
        Err(e) => {
            err.push(path, e);
            return
        }
    };
    if let Err(e) = stm.eval_all(scope, target) {
        err.push(path, e);
    }
}


//------------ FeatureSetError -----------------------------------------------

#[derive(Default)]
pub struct FeatureSetError(Vec<(String, Error)>);

impl FeatureSetError {
    fn push(&mut self, path: &FsPath, err: impl Into<Error>) {
        self.0.push((path.to_string_lossy().into(), err.into()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn check(self) -> Result<(), Self> {
        if self.0.is_empty() {
            Ok(())
        }
        else {
            Err(self)
        }
    }
}

impl fmt::Display for FeatureSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &(ref path, ref err) in &self.0 {
            match *err {
                Error::Parse(ref err) => {
                    writeln!(f, "{}: {}", path, err)?;
                }
                Error::Eval(ref err) => {
                    for (pos, err) in err.iter() {
                        writeln!(f, "{}:{}: {}", path, pos, err)?;
                    }
                }
                Error::Io(ref err) => {
                    writeln!(f, "{}: {}", path, err)?;
                }
            }
        }
        Ok(())
    }
}


//------------ Error ---------------------------------------------------------

enum Error {
    Parse(ast::Error),
    Eval(eval::Error),
    Io(io::Error)
}

impl From<ast::Error> for Error {
    fn from(err: ast::Error) -> Error {
        Error::Parse(err)
    }
}

impl From<eval::Error> for Error {
    fn from(err: eval::Error) -> Error {
        Error::Eval(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

