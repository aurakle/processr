use std::{collections::HashMap, env, fs::{self, File}, path::{Path, PathBuf}};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha_rs::{Sha, Sha256, Sha512};
use tera::{Tera, Value};

use crate::{error::FsError, prelude::SingleProcedure};

static DATA: &str = "data.json";
static SOURCES: &str = "sources.json";

#[derive(Debug)]
pub struct State {
    pub root: PathBuf,
    pub tera: Tera,
    pub cache: PathBuf,
    pub cached_data: HashMap<String, Value>,
    pub cached_sources: HashMap<String, String>,
}

impl State {
    pub fn new(root: &str, templates: &str) -> Result<Self> {
        let pwd = env::current_dir()?;
        let root = pwd.join(root);
        let mut tera = Tera::new(&format!("{}/**/*", templates))?;
        let cache = root.join(".cache");
        let cached_data = Self::load_json(cache.join(DATA)).unwrap_or(HashMap::new());
        let cached_resources = Self::load_json(cache.join(SOURCES)).unwrap_or(HashMap::new());

        fs::create_dir_all(&cache)?;
        tera.autoescape_on(Vec::new());

        Ok(Self {
            root,
            tera,
            cache,
            cached_data,
            cached_sources: cached_resources,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        Self::save_json(self.cache.join(DATA), &self.cached_data)?;
        Self::save_json(self.cache.join(SOURCES), &self.cached_sources)?;

        Ok(())
    }

    pub fn property<S: Into<String>>(&mut self, key: S, value: Value) {
        self.cached_data.insert(key.into(), value);
    }

    fn save_json<P: AsRef<Path>, S: Serialize>(path: P, value: S) -> Result<()> {
        let text = serde_json::to_string(&value)?;
        fs::write(path, text.as_bytes())?;

        Ok(())
    }

    fn load_json<P: AsRef<Path>, D: for<'a> Deserialize<'a>>(path: P) -> Result<D> {
        let file = File::open(path)?;
        let res = serde_json::from_reader(file)?;

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
        state.cached_sources.insert(link, cache_link.clone());

        Ok(cache_link)
    }
}

#[async_trait(?Send)]
impl SingleProcedure for Item {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        Ok(self.clone())
    }
}
