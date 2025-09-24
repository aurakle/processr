use std::{collections::HashMap, env, fs, path::{Path, PathBuf}};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;
use sha_rs::{Sha, Sha256, Sha512};
use tera::{Tera, Value};

use crate::{error::FsError, prelude::SingleProcedure};

static SOURCES: &str = "sources.json";

#[derive(Debug)]
pub struct State {
    pub root: PathBuf,
    pub tera: Tera,
    pub cache: PathBuf,
    pub cached_resources: HashMap<String, String>,
}

impl State {
    pub fn new(root: &str, templates: &str) -> Result<Self> {
        let pwd = env::current_dir()?;
        let root = pwd.join(root);
        let mut tera = Tera::new(&format!("{}/**/*", templates))?;
        let cache = root.join(".cache");
        let cached_resources = Self::load_cc(&cache).unwrap_or(HashMap::new());

        fs::create_dir_all(&cache)?;
        tera.autoescape_on(vec![]);

        Ok(Self {
            root,
            tera,
            cache,
            cached_resources,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        fs::write(self.cache.join(SOURCES), serde_json::to_string(&self.cached_resources)?.as_bytes())?;

        Ok(())
    }

    fn load_cc(cache: &Path) -> Result<HashMap<String, String>> {
        let sources = fs::read_to_string(cache.join(SOURCES))?;
        let res = serde_json::from_str(&sources)?;

        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub properties: HashMap<String, Value>,
}

impl Item {
    pub fn write(&self, state: &State) -> Result<()> {
        let path = state.root.join(self.path.clone());

        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }

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
            properties,
            ..self.clone()
        }
    }

    pub fn set_path(&self, path: PathBuf) -> Self {
        Self {
            path,
            ..self.clone()
        }
    }

    pub fn get_filename(&self) -> Result<String> {
        Ok(self.path
            .file_name()
            .map(|os_str| Path::new(os_str))
            .ok_or(FsError::InvalidFileName)?
            .to_str()
            .ok_or(FsError::OsStringNotUtf8)?
            .to_owned())
    }

    pub fn properties_with_url_and_body(&self) -> Result<HashMap<String, Value>> {
        let mut props = self.properties.clone();

        props.insert(format!("url"), Value::String(format!("/{}", self.path.as_os_str().to_str().ok_or(anyhow!("File path {} is not valid UTF-8", self.path.display()))?)));
        props.insert(format!("body"), Value::String(String::from_utf8(self.bytes.clone())?));

        Ok(props)
    }

    pub fn into_meta(&self) -> Result<Value> {
        self.properties_with_url_and_body().map(|props| Value::Object({
            let mut map = tera::Map::new();
            map.extend(props);

            map
        }))
    }

    pub fn insert_into_cache(&mut self, state: &mut State, link: String, bytes: Vec<u8>, extension: Option<String>) -> Result<String> {
        let hasher = Sha256::new();
        let contents_hash = hasher.digest(bytes.as_slice());
        let filename = format!("{}-{}{}", contents_hash, bytes.len(), extension.map(|ext| format!(".{}", ext)).unwrap_or_else(String::new));

        let path = state.cache.join(filename.clone());

        if !path.exists() {
            println!("Writing cached resource {}", filename);
            fs::write(path, bytes.as_slice())?;
        }

        let cache_link = format!("/.cache/{}", filename);
        state.cached_resources.insert(link, cache_link.clone());

        Ok(cache_link)
    }
}

#[async_trait(?Send)]
impl SingleProcedure for Item {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        Ok(self.clone())
    }
}
