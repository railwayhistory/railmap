use std::{fmt, io, mem, path};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path as FsPath;
use std::sync::Mutex;
use femtomap::import::path::{ImportPath, PathError};
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use osmxml::read::read_xml;


//------------ PathSet -------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct PathSet {
    paths: Vec<ImportPath>,
    names: HashMap<String, usize>,
}

impl PathSet {
    pub fn load(path: &FsPath) -> Result<Self, PathSetError> {
        let mut types = TypesBuilder::new();
        types.add("osm", "*.osm").unwrap();
        let walk = WalkBuilder::new(path)
            .types(types.select("osm").build().unwrap())
            .build_parallel();
        let res = Mutex::new(PathSet::default());
        let errors = Mutex::new(PathSetError::new());
        walk.run(|| {
            Box::new(|path| {
                let path = match path {
                    Ok(path) => path,
                    Err(_) => return WalkState::Continue
                };
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }

                let path = path.path();
                let mut file = match File::open(&path) {
                    Ok(file) => file,
                    Err(err) => {
                        errors.lock().unwrap().add(path, err.into());
                        return WalkState::Continue
                    }
                };
                let mut osm = match read_xml(&mut file) {
                    Ok(osm) => osm,
                    Err(_) => {
                        errors.lock().unwrap().add(
                            path,
                            io::Error::new(
                                io::ErrorKind::Other, "XML error"
                            ).into()
                        );
                        return WalkState::Continue
                    }
                };

                // Swap out the relations so we don’t hold a mutable reference
                // to `osm` while draining the relations.
                let mut relations = HashSet::new();
                mem::swap(osm.relations_mut(), &mut relations);
                for relation in relations.drain() {
                    match ImportPath::from_osm(relation, &osm) {
                        Ok((key, path)) => {
                            {
                                let mut res = res.lock().unwrap();
                                let idx = res.paths.len();
                                res.names.insert(key, idx);
                                res.paths.push(path);
                            }
                        }
                        Err(err) => {
                            errors.lock().unwrap().add(path, err);
                        }
                    }
                }

                WalkState::Continue
            })
        });
        let errors = errors.into_inner().unwrap();
        errors.check()?;
        Ok(res.into_inner().unwrap())
    }

    pub fn lookup(&self, key: &str) -> Option<usize> {
        self.names.get(key).cloned()
    }

    pub fn get(&self, idx: usize) -> Option<&ImportPath> {
        self.paths.get(idx)
    }

    pub fn iter<'a>(
        &'a self
    ) -> impl Iterator<Item = &'a ImportPath> {
        self.paths.iter()
    }
}


//------------ PathSetError --------------------------------------------------

#[derive(Default)]
pub struct PathSetError(Vec<(String, PathError)>);

impl PathSetError {
    pub fn new() -> Self {
        PathSetError(Vec::new())
    }

    pub fn add(&mut self, path: impl AsRef<path::Path>, err: PathError) {
        self.0.push((format!("{}", path.as_ref().display()), err))
    }

    pub fn extend(&mut self, err: Self) {
        self.0.extend(err.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn check(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(())
        }
        else {
            Err(self)
        }
    }
}

impl fmt::Display for PathSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &(ref path, ref err_set) in &self.0 {
            for err in err_set {
                writeln!(f, "{}: {}", path, err)?;
            }
        }
        Ok(())
    }
}

