use std::{collections::HashMap, env, fmt::Display, fs, path::{Path, PathBuf}};
use actix_files::{Files, NamedFile};
use actix_web::{web::{self, Data}, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::{anyhow, Context, Result};
use data::Item;
use procedure::SingleProcedure;

pub extern crate anyhow;
pub use actix_web;

pub mod data;
pub mod error;
pub mod prelude;
pub mod parser;
pub mod procedure;
pub mod selector;

#[derive(Debug, clap::Parser)]
#[command(name = "processr")]
#[command(about = "Static site generator configured through a Rust macro DSL", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn parse() -> Self {
        <Cli as clap::Parser>::parse()
    }
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
        #[::processr::actix_web::rt::main(system = "::processr::actix_web::rt::System")]
        async fn main() -> $crate::anyhow::Result<()> {
            fs::create_dir_all($out);

            match $crate::Cli::parse().command {
                $crate::Command::Serve(args) => {
                    build(args.clean)?;
                    $crate::serve($out, args.port).await
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
            .service(Files::new("/", path.clone())
                .use_hidden_files()
                .index_file("index.html")
                .prefer_utf8(true))
    })
        .bind(("localhost", port))
        .context("Could not bind to address")?;

    println!("Serving on http://localhost:{}", port);
    server.run().await?;
    Ok(())
}
