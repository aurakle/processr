use std::{env, fs, path::PathBuf};

use regex::Regex;
use anyhow::{Result, anyhow};
use thiserror::Error;

use crate::{procedure::Procedure, Item};

pub struct Selector {
    path: String,
}

impl Procedure for Selector {
    fn eval(&self) -> Result<Item> {
        Item::from_file(self.path)
    }
}

pub fn single(path: &str) -> Result<Selector> {
    match fs::exists(path) {
        Ok(b) => if b { Ok(Selector { path: path.to_owned() }) } else { anyhow!("File does not exist") },
        e => e,
    }
}

pub fn regex(pat: &str) -> Result<Vec<Selector>> {
    let paths = find(pat)?;
    let mut selectors = Vec::new();

    for path in paths {
        selectors.push(Selector { path });
    }

    Ok(selectors)
}

// https://stackoverflow.com/questions/71918788/find-files-that-match-a-dynamic-pattern
fn find(foo: &str) -> Result<Vec<String>, FindError> {
    let current_dir = env::current_dir()?;
    let path = PathBuf::from(foo);
    let base = path
        .parent()
        .unwrap_or(&current_dir)
        .to_str()
        .ok_or(FindError::OsStringNotUtf8)?;
    let pattern = format!(r"{}", foo);
    let expression = Regex::new(&pattern)?;
    Ok(
        fs::read_dir(&base)?
            .map(|entry| Ok(
                entry?
                .path()
                .file_name()
                .ok_or(FindError::InvalidFileName)?
                .to_str()
                .ok_or(FindError::OsStringNotUtf8)?
                .to_string()
            ))
            .collect::<Result<Vec<_>, FindError>>()?
            .into_iter()
            .filter(|file_name| expression.is_match(&file_name))
            .collect()
    )
}

#[derive(Error, Debug)]
enum FindError {
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error("File name has no extension")]
    NoFileExtension,
    #[error("Not a valid file name")]
    InvalidFileName,
    #[error("No valid base file")]
    InvalidBaseFile,
    #[error("An OS string is not valid utf-8")]
    OsStringNotUtf8,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
