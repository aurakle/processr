use std::{collections::HashMap, env, fmt::Display, fs, path::{Path, PathBuf}};
use actix_files::{Files, NamedFile};
use actix_web::{web::{self, Data}, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::{Result, anyhow};
use procedure::SingleProcedure;

pub extern crate anyhow;
pub extern crate actix_web;

pub mod prelude;
pub mod parser;
pub mod procedure;
pub mod selector;
pub mod extractor;

#[derive(Debug, clap::Parser)]
#[command(name = "processr")]
#[command(about = "Static site generator configured through a Rust macro DSL", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Serve(ServeArgs),
    Build(BuildArgs),
}

#[derive(clap::Args, Debug, Clone)]
#[command(about = "Build the website and serve it on localhost", long_about = None)]
pub struct ServeArgs {
    #[arg(short, long, default_value_t = 80, help = "The port to serve files on")]
    pub port: u16,
    #[arg(short, long, help = "Clean output directory before building")]
    pub clean: bool,
}

#[derive(clap::Args, Debug, Clone)]
#[command(about = "Build the website", long_about = None)]
pub struct BuildArgs {
    #[arg(short, long, help = "Clean output directory before building")]
    pub clean: bool,
}

#[macro_export]
macro_rules! processr {
    ($out:literal <- $($names:ident $rules:expr)+) => {
        #[$crate::actix_web::main]
        fn main() -> $crate::anyhow::Result<()> {
            match $crate::Cli::parse().command {
                $crate::Command::Serve(args) => {
                    build(args.clean)?;
                    $crate::serve($out, args.port)
                },
                $crate::Command::Build(args) => {
                    build(args.clean)
                }
            }
        }

        fn build(clean: bool) -> $crate::anyhow::Result<()> {
            if clean {
                $crate::clean($out)?
            }

            $(let $names = $rules; $crate::procedure::Procedure::write(&$names, $out)?;)+

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

impl Meta {
    //TODO: add support for maps
    pub fn as_string(&self) -> Option<String> {
        match self {
            Meta::Map(map) => None,
            Meta::List(items) => Some(items
                .iter()
                .flat_map(|m| m.as_string())
                .collect::<Vec<_>>()
                .join(", ")),
            Meta::Text(s) => Some(s.clone()),
        }
    }

    pub fn as_list(&self) -> Vec<Meta> {
        match self {
            Meta::Map(map) => vec![Meta::Map(map.clone())],
            Meta::List(items) => items.clone(),
            Meta::Text(s) => vec![Meta::Text(s.clone())],
        }
    }

    pub fn as_map(&self) -> HashMap<String, Meta> {
        match self {
            Meta::Map(map) => map.clone(),
            Meta::List(items) => {
                let mut map = HashMap::new();
                map.insert(format!("i"), Meta::List(items.clone()));

                map
            },
            Meta::Text(s) => {
                let mut map = HashMap::new();
                map.insert(format!("i"), Meta::Text(s.clone()));

                map
            },
        }
    }
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

pub fn clean(path: &str) -> Result<()> {
    let pwd = env::current_dir()?;
    let path = pwd.join(path);

    fs::remove_dir_all(path)?;
    Ok(())
}

pub async fn serve(path: &str, port: u16) -> Result<()> {
    let path = path.to_owned();
    let server = HttpServer::new(move || {
        App::new()
            .service(Files::new("/", path.clone()).prefer_utf8(true))
    })
    .bind(("localhost", port))?;

    server.run().await?;
    Ok(())
}
