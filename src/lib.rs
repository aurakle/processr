use std::{collections::HashMap, env, fmt::Display, fs, path::{Path, PathBuf}};
use anyhow::{Result, anyhow};
use procedure::SingleProcedure;

pub mod prelude;
pub mod parser;
pub mod procedure;
pub mod selector;
pub mod extractor;

#[macro_export]
macro_rules! processr {
    ($out:literal <- $(rule $rules:expr)+) => {
        fn main() -> anyhow::Result<()> {
            $($crate::procedure::Procedure::write(&$rules, $out)?;)+

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
        let pwd = env::current_dir()?;
        let path = pwd.join(root).join(self.path.clone());

        fs::create_dir_all(path.parent().unwrap_or(&pwd))?;
        Ok(fs::write(path, self.bytes.as_slice())?)
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
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

    pub fn properties_with_url_and_body(&self) -> Result<HashMap<String, Meta>> {
        let mut props = self.properties.clone();

        props.insert(format!("url"), Meta::from(self.path.as_os_str().to_str().ok_or(anyhow!("File path {} is not valid UTF-8", self.path.display()))?));
        props.insert(format!("body"), Meta::from(String::from_utf8(self.bytes.clone())?));

        Ok(props)
    }

    pub fn into_meta(&self) -> Result<Meta> {
        self.properties_with_url_and_body().map(|props| Meta::from(props))
    }
}

impl SingleProcedure for Item {
    fn eval(&self) -> Result<Item> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    Map(HashMap<String, Meta>),
    List(Vec<Meta>),
    Text(String),
}

impl From<HashMap<String, Meta>> for Meta {
    fn from(value: HashMap<String, Meta>) -> Self {
        Self::Map(value)
    }
}

impl From<Vec<Meta>> for Meta {
    fn from(value: Vec<Meta>) -> Self {
        Self::List(value)
    }
}

impl From<String> for Meta {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for Meta {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

pub fn create(path: &str) -> Item {
    Item {
        path: PathBuf::from(path),
        bytes: Vec::new(),
        properties: HashMap::new(),
    }
}
