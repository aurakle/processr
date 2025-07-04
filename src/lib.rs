use std::{collections::HashMap, env, fs, path::{Path, PathBuf}};
use anyhow::Result;

pub mod procedure;
pub mod selector;

#[macro_export]
macro_rules! processr {
    ($out:literal <- $(rule $rules:expr)+) => {
        fn main() -> anyhow::Result<()> {
            let procedures = vec![$($rules),+];

            for p in procedures {
                p.eval()?.write($out)?;
            }

            Ok(())
        }
    };
}

#[derive(Debug, Clone)]
pub struct Item {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub properties: HashMap<String, Meta>,
}

impl Item {
    pub fn write(&self, root: &str) -> Result<()> {
        let path = env::current_dir()?.join(root).join(self.path.clone());
        Ok(fs::write(path, self.bytes.as_slice())?)
    }

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
