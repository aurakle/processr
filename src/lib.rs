use std::{collections::HashMap, env, fmt::Display, fs, path::{Path, PathBuf}};
use anyhow::Result;
use procedure::SingleProcedure;

pub mod prelude;
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
}

impl SingleProcedure for Item {
    fn eval(&self) -> Result<Item> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meta(pub Vec<String>);

impl Meta {
    pub fn as_string(&self) -> String {
        self.0.join(", ")
    }
}

impl From<String> for Meta {
    fn from(value: String) -> Self {
        Self(vec![value])
    }
}

impl From<&str> for Meta {
    fn from(value: &str) -> Self {
        Self(vec![value.to_owned()])
    }
}

impl<T: Display> From<Vec<T>> for Meta {
    fn from(value: Vec<T>) -> Self {
        let mut result = Vec::new();

        for item in value {
            result.push(format!("{}", item));
        }

        Self(result)
    }
}

pub fn create(path: &str) -> Item {
    Item {
        path: PathBuf::from(path),
        bytes: Vec::new(),
        properties: HashMap::new(),
    }
}
