use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use anyhow::Result;

pub mod procedure;
pub mod selector;

#[derive(Debug, Clone)]
pub struct Item {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub properties: HashMap<String, Meta>,
}

impl Item {
    pub fn from_file(path: String) -> Result<Self> {
        Ok(Self {
            path: PathBuf::from(path.clone()),
            bytes: fs::read(Path::new(path.as_str()))?,
            properties: HashMap::new(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    List(Vec<Meta>),
    Text(String),
}
