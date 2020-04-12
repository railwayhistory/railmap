use std::{fmt, io, mem, path};
use std::collections::{HashMap, HashSet};
use std::f64::INFINITY;
use std::f64::consts::PI;
use std::fs::File;
use std::path::Path as FsPath;
use std::str::FromStr;
use std::sync::{Mutex};
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use kurbo::Vec2;
use osmxml::elements::{MemberType, Osm, Relation};
use osmxml::read::read_xml;
use crate::features::path::BasePath;
use super::mp_path;


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


//------------ ImportPath ----------------------------------------------------

#[derive(Clone, Debug)]
pub struct ImportPath {
    path: BasePath,
    len: usize,
    node_names: HashMap<String, u32>,
}

impl ImportPath {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn path(&self) -> BasePath {
        self.path.clone()
    }

    pub fn get_named(&self, name: &str) -> Option<u32> {
        self.node_names.get(name).cloned()
    }
}


impl ImportPath {
    pub fn from_osm(
        mut relation: Relation,
        osm: &Osm,
    ) -> Result<(String, Self), PathError> {
        let mut err = PathError::new();
        if relation.tags().get("type") != Some("path") {
            err.add(Error::NonPathRelation { rel: relation.id() });
        }
        
        let key = match relation.tags_mut().remove("key") {
            Some(key) => key,
            None => {
                err.add(Error::MissingKey { rel: relation.id() });
                String::new()
            }
        };
        let (nodes, node_names) = Self::load_nodes(
            &mut relation, osm, &mut err
        );
        err.check()?;

        Ok((
            key,
            ImportPath {
                len: nodes.len(),
                path: Self::create_final_path(&nodes),
                node_names
            }
        ))
    }

    fn load_nodes(
        relation: &mut Relation,
        osm: &Osm,
        err: &mut PathError,
    ) -> (Vec<Node>, HashMap<String, u32>) {
        let mut nodes = Vec::new();
        let mut node_names = HashMap::new();

        let mut last_id = None;
        let mut last_tension = false; // last node has explicit post tension
        for member in relation.members() {
            if member.mtype() != MemberType::Way {
                err.add(Error::NonWayMember {
                    rel: relation.id(), target: member.id()
                });
                continue;
            }
            let way = match osm.get_way(member.id()) {
                Some(way) => way,
                None => {
                    err.add(Error::MissingWay {
                        rel: relation.id(), way: member.id()
                    });
                    continue
                }
            };
            let tension = match way.tags().get("type") {
                None => 1.,
                Some("curved") => 1.,
                Some("straight") => INFINITY,
                Some(value) => {
                    err.add(Error::IllegalWayType {
                        way: way.id(), value: value.into()
                    });
                    1.
                }
            };

            if way.nodes().is_empty() {
                err.add(Error::EmptyWay { way: way.id() });
                continue;
            }
            let mut way_nodes = way.nodes().iter();
            if let Some(last) = last_id {
                let id = way_nodes.next().unwrap();
                if last != id {
                    err.add(Error::NonContiguous {
                        rel: relation.id(),
                        way: way.id()
                    });
                    // That’s the end of this relation, really.
                    return (nodes, node_names)
                }
                if !last_tension {
                    nodes.last_mut().unwrap().post = tension;
                }
            }
            for id in way_nodes {
                let (node, name, post_tension)
                    = Self::load_node(*id, osm, tension, err);
                if let Some(name) = name {
                    if node_names.insert(
                        name.clone(), nodes.len() as u32
                    ).is_some() {
                        err.add(Error::DuplicateName {
                            rel: relation.id(), name
                        });
                    }
                }
                nodes.push(node);
                last_tension = post_tension;
                last_id = Some(id);
            }
        }
        (nodes, node_names)
    }

    fn load_node(
        id: i64,
        osm: &Osm,
        tension: f64,
        err: &mut PathError
    ) -> (Node, Option<String>, bool) {
        let node = match osm.get_node(id) {
            Some(node) => node,
            None => {
                err.add(Error::MissingNode { node: id });
                return (Node::default(), None, false)
            }
        };
        let pre = match node.tags().get("pre") {
            Some(pre) => match f64::from_str(pre).ok() {
                Some(pre) => pre,
                None => {
                    err.add(Error::InvalidPre { node: id });
                    tension
                }
            },
            None => tension
        };
        let (post, have_post) = match node.tags().get("post") {
            Some(post) => match f64::from_str(post).ok() {
                Some(post) => (post, true),
                None => {
                    err.add(Error::InvalidPost { node: id });
                    (tension, false)
                }
            },
            None => (tension, false)
        };
        let name = node.tags().get("name").map(String::from);
        (
            Node::new(node.lon(), node.lat(), pre, post),
            name,
            have_post
        )
    }

    fn create_final_path(nodes: &[Node]) -> BasePath {
        let segment = mp_path::Segment::from_vec(
            nodes.iter().map(|node| {
                mp_path::Knot::new(
                    node.normalized(),
                    node.pre, node.post
                )
            }).collect()
        );
        BasePath::new(segment.to_bez_path())
    }
}


//------------ Node ----------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub lon: f64,
    pub lat: f64,
    pub pre: f64,
    pub post: f64,
}

impl Node {
    pub fn new(lon: f64, lat: f64, pre: f64, post: f64) -> Self {
        Node { lon, lat, pre, post }
    }

    pub fn lonlat(&self) -> (f64, f64) {
        (self.lon, self.lat)
    }

    pub fn normalized(&self) -> Vec2 {
        Vec2::new(
            (self.lon + 180.) / 360.,
            (1.0 - self.lat.to_radians().tan().asinh() / PI) / 2.0
        )
    }
}


//------------ PathSetError --------------------------------------------------

pub struct PathSetError(Vec<(String, PathError)>);

impl PathSetError {
    pub fn new() -> Self {
        PathSetError(Vec::new())
    }

    pub fn add(&mut self, path: impl AsRef<path::Path>, err: PathError) {
        self.0.push((format!("{}", path.as_ref().display()), err))
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
            for err in &err_set.0 {
                writeln!(f, "{}: {}", path, err)?;
            }
        }
        Ok(())
    }
}

//------------ PathError -----------------------------------------------------

pub struct PathError(Vec<Error>);

impl PathError {
    pub fn new() -> Self {
        PathError(Vec::new())
    }

    pub fn add(&mut self, err: Error) {
        self.0.push(err)
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

impl From<Error> for PathError {
    fn from(err: Error) -> Self {
        PathError(vec![err])
    }
}

impl From<io::Error> for PathError {
    fn from(err: io::Error) -> Self {
        PathError(vec![Error::Io(err)])
    }
}



//------------ Error ---------------------------------------------------------

pub enum Error {
    NonPathRelation { rel: i64 },
    MissingKey { rel: i64 },
    NonWayMember { rel: i64, target: i64 },
    MissingWay { rel: i64, way: i64 },
    IllegalWayType { way: i64, value: String },
    EmptyWay { way: i64 },
    NonContiguous { rel: i64, way: i64 },
    MissingNode { node: i64 },
    InvalidPre { node: i64 },
    InvalidPost { node: i64 },
    DuplicateName { rel: i64, name: String },
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _ => f.write_str("error"),
        }
    }
}

