use std::{collections::HashMap, env, fs, path::{Path, PathBuf}};
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
    pub fn from_file(path: &String) -> Result<Self> {
        let path = PathBuf::from(path);

        Ok(Self {
            path: PathBuf::from(path.strip_prefix(env::current_dir()?).unwrap_or(&path)),
            bytes: fs::read(path)?,
            properties: HashMap::new(),
        })
    }

    pub fn set_property(&self, key: String, value: Meta) -> Self {
        let mut properties = self.properties.clone();

        properties.insert(key, value);

        Self {
            path: self.path.clone(),
            bytes: self.bytes.clone(),
            properties,
        }
    }

    pub fn set_path(&self, path: PathBuf) -> Self {
        Self {
            path,
            bytes: self.bytes.clone(),
            properties: self.properties.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    List(Vec<Meta>),
    Text(String),
}
