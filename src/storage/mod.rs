mod asset;

pub use self::asset::*;

use std::path::{Path, PathBuf};

pub struct Storage {
    assets_dir: PathBuf
}

impl Storage {
    pub fn new<T: AsRef<Path>>(assets_dir: T) -> Self {
        let assets_dir = assets_dir.as_ref().to_path_buf();
        Storage {
            assets_dir
        }
    }

    pub fn get(&self, name: &str) -> Asset {
        let path = format!("{}/{}", self.assets_dir.to_string_lossy(), name);
        Asset::new(path)
    }

    pub fn path(&self) -> &Path {
        &self.assets_dir
    }
}