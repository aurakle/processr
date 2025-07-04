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
    let (base, file_name) = resolve_split_path(pat)?;
    let paths = recursive_search(&PathBuf::from(base), &FilesNamed::regex(file_name))?;

    Ok(make_selectors_for_paths(paths))
}

pub fn wild(pat: &str) -> Result<Vec<Selector>> {
    let (base, file_name) = resolve_split_path(pat)?;
    let paths = recursive_search(&PathBuf::from(base), &FilesNamed::wildmatch(file_name))?;

    Ok(make_selectors_for_paths(paths))
}

fn recursive_search(dir: &Path, matcher: &FilesNamed) -> Result<Vec<PathBuf>> {
    let mut result = matcher.within(dir).find()?;

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            let mut current = matcher.within(&path).find()?;
            let mut inner = recursive_search(&path, matcher)?;
            result.append(&mut current);
            result.append(&mut inner);
        }
    }

    Ok(result)
}

fn resolve_split_path(pat: &str) -> Result<(String, String)> {
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

    Ok((base.to_owned(), file_name.to_owned()))
}

fn make_selectors_for_paths(paths: Vec<PathBuf>) -> Vec<Selector> {
    let mut selectors = Vec::new();


    for path in paths {
        selectors.push(Selector(path));
    }

    selectors
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
