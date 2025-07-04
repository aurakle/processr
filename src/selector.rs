use std::{fs, path::PathBuf};

use regex::Regex;
use anyhow::Result;
use thiserror::Error;

use crate::Item;

pub fn regex(pat: &str) -> Result<Vec<Item>> {
    let paths = find(pat)?;
    let mut items = Vec::new();

    for path in paths {
        items.push(Item::from_file(path)?);
    }

    Ok(items)
}

// https://stackoverflow.com/questions/71918788/find-files-that-match-a-dynamic-pattern
fn find(foo: &str) -> Result<Vec<String>, FindError> {
    let path = PathBuf::from(foo);
    let base = path
        .parent()
        .ok_or(FindError::InvalidBaseFile)?
        .to_str()
        .ok_or(FindError::OsStringNotUtf8)?;
    let file_name = path
        .file_stem()
        .ok_or(FindError::InvalidFileName)?
        .to_str()
        .ok_or(FindError::OsStringNotUtf8)?;
    let file_extension = path
        .extension()
        .ok_or(FindError::NoFileExtension)?
        .to_str()
        .ok_or(FindError::OsStringNotUtf8)?;
    let pattern = format!(r"{}\.\d{{3}}\.{}", file_name, file_extension);
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
