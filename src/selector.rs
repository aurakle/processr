use std::{env, fs, path::{Path, PathBuf}};

use file_matcher::{FileNamed, FilesNamed};
use regex::Regex;
use anyhow::{anyhow, Context, Result};
use thiserror::Error;

use crate::{procedure::SingleProcedure, Item};

pub struct Selector(PathBuf);

impl SingleProcedure for Selector {
    fn eval(&self) -> Result<Item> {
        Item::from_file(&self.0)
    }
}

pub fn single(path: &str) -> Result<Selector> {
    fs::exists(path).with_context(|| "Could not check whether file exists").and_then(|b| {
        if b {
            Ok(Selector(PathBuf::from(path)))
        } else {
            Err(anyhow!("File does not exist"))
        }
    })
}

pub fn regex(pat: &str) -> Result<Vec<Selector>> {
    let current_dir = env::current_dir()?;
    let path = PathBuf::from(pat);
    let base = path
        .parent()
        .unwrap_or(&current_dir)
        .to_str()
        .ok_or(FindError::OsStringNotUtf8)?;
    let file_name = path
        .file_name()
        .map(|os_str| Path::new(os_str))
        .ok_or(FindError::InvalidFileName)?
        .to_str()
        .ok_or(FindError::OsStringNotUtf8)?;
    let paths = FilesNamed::regex(file_name)
        .within(base)
        .find()?;
    let mut selectors = Vec::new();


    for path in paths {
        selectors.push(Selector(path));
    }

    Ok(selectors)
}

#[derive(Error, Debug)]
enum FindError {
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error("Not a valid file name")]
    InvalidFileName,
    #[error("No valid base file")]
    InvalidBaseFile,
    #[error("An OS string is not valid utf-8")]
    OsStringNotUtf8,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
