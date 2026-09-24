mod get;
mod run;

pub use get::*;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct PodmanCore {
    path: PathBuf,
    version: String,
    env: HashMap<String, String>,
}

impl PodmanCore {
    fn new(path: PathBuf, version: String, env: HashMap<String, String>) -> Self {
        Self { path, version, env }
    }
    pub fn get_version(&self) -> &String {
        &self.version
    }
}
