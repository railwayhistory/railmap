//! Configuration of the map.

use std::{fs, io};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

//------------ Config --------------------------------------------------------

/// The map configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// The theme to be used for interpreting the rules and rendering.
    pub theme: String,

    /// The regions of the map.
    pub regions: HashMap<String, Region>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let data = fs::read(path.as_ref())?;
        let mut data: Self = toml::from_slice(&data)?;
        if let Some(path) = path.as_ref().parent() {
            data.prepare(path);
        }
        Ok(data)
    }

    pub fn prepare(&mut self, base_dir: &Path) {
        self.regions.values_mut().for_each(|region| {
            region.prepare(base_dir)
        });
    }
}


//------------ Region --------------------------------------------------------

/// A region of the map.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Region {
    /// The directory where the paths live.
    pub paths: PathBuf,

    /// The directory where the rules live.
    pub rules: PathBuf,

    /// Include this region in the detailed map?
    #[serde(default)]
    pub detailed: bool,
}

impl Region {
    fn prepare(&mut self, base_dir: &Path) {
        self.paths = base_dir.join(&self.paths);
        self.rules = base_dir.join(&self.rules);
    }
}

