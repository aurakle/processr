use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsError {
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
