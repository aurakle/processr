use std::{env, fs, path::{Path, PathBuf}};

use regex::Regex;
use anyhow::{anyhow, Context, Result};
use thiserror::Error;
use wildmatch::WildMatch;

use crate::{procedure::SingleProcedure, Item};

#[derive(Clone)]
pub struct Selector(PathBuf);

impl SingleProcedure for Selector {
    fn eval(&self) -> Result<Item> {
        Item::from_file(&self.0)
    }
}

pub fn exact(path: &str) -> Result<Selector> {
    fs::exists(path).map_err(|e| FindError::IoError(e)).and_then(|b| {
        if b {
            Ok(Selector(PathBuf::from(path)))
        } else {
            Err(FindError::FileNotFound)
        }
    }).with_context(|| format!("Failed to locate file at '{}'", path))
}

pub fn regex(pat: &str) -> Result<Vec<Selector>> {
    let (base, file_name) = resolve_split_path(pat)?;
    let r = Regex::new(file_name.as_str())?;
    let paths = recursive_search(&PathBuf::from(base), &|p| r.is_match(p))?;

    Ok(make_selectors_for_paths(paths))
}

pub fn wild(pat: &str) -> Result<Vec<Selector>> {
    let (base, file_name) = resolve_split_path(pat)?;
    let r = WildMatch::new(file_name.as_str());
    let paths = recursive_search(&PathBuf::from(base), &|p| r.is_match(p))?;

    Ok(make_selectors_for_paths(paths))
}

fn recursive_search<F>(dir: &Path, matcher: &F) -> Result<Vec<PathBuf>>
where
    F: Fn(&str) -> bool,
{
    let mut result = Vec::new();

    println!("Searching {}", dir.display());

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            let mut inner = recursive_search(&path, matcher)?;
            result.append(&mut inner);
        }

        let file_name = path
            .file_name()
            .map(|os_str| Path::new(os_str))
            .ok_or(FindError::InvalidFileName)?
            .to_str()
            .ok_or(FindError::OsStringNotUtf8)?;

        if matcher(file_name) {
            result.push(path);
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
    #[error("File not found")]
    FileNotFound,
    #[error("Not a valid file name")]
    InvalidFileName,
    #[error("No valid base file")]
    InvalidBaseFile,
    #[error("An OS string is not valid utf-8")]
    OsStringNotUtf8,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
