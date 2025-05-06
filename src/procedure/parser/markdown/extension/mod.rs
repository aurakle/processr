use nom::{error::Error, IResult, Parser};

use super::{ast::MarkdownElement, MarkdownParser};

mod default;

pub use default::all as default;
