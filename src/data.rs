use std::{collections::HashMap, env, fs, path::PathBuf};

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::prelude::SingleProcedure;

#[derive(Debug, Clone)]
pub struct Item {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub properties: HashMap<String, Value>,
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

    pub fn set_property<S: Into<String>, M: Into<Value>>(&self, key: S, value: M) -> Self {
        let mut properties = self.properties.clone();

        properties.insert(key.into(), value.into());

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

    pub fn properties_with_url_and_body(&self) -> Result<HashMap<String, Value>> {
        let mut props = self.properties.clone();

        props.insert(format!("url"), Value::from(self.path.as_os_str().to_str().ok_or(anyhow!("File path {} is not valid UTF-8", self.path.display()))?));
        props.insert(format!("body"), Value::from(String::from_utf8(self.bytes.clone())?));

        Ok(props)
    }

    pub fn into_meta(&self) -> Result<Value> {
        self.properties_with_url_and_body().map(|props| Value::from(props))
    }
}

impl SingleProcedure for Item {
    fn eval(&self) -> Result<Item> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Map(HashMap<String, Value>),
    List(Vec<Value>),
    Text(String),
}

impl Value {
    //TODO: add support for maps
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::Map(map) => None,
            Value::List(items) => Some(items
                .iter()
                .flat_map(|m| m.as_string())
                .collect::<Vec<_>>()
                .join(", ")),
            Value::Text(s) => Some(s.clone()),
        }
    }

    pub fn as_list(&self) -> Vec<Value> {
        match self {
            Value::Map(map) => vec![Value::Map(map.clone())],
            Value::List(items) => items.clone(),
            Value::Text(s) => vec![Value::Text(s.clone())],
        }
    }

    pub fn as_map(&self) -> HashMap<String, Value> {
        match self {
            Value::Map(map) => map.clone(),
            Value::List(items) => {
                let mut map = HashMap::new();
                map.insert(format!("i"), Value::List(items.clone()));

                map
            },
            Value::Text(s) => {
                let mut map = HashMap::new();
                map.insert(format!("i"), Value::Text(s.clone()));

                map
            },
        }
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(value: HashMap<String, Value>) -> Self {
        Self::Map(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Self::List(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}
