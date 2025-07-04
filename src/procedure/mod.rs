use anyhow::Result;

use crate::Item;

pub mod util;
pub mod parser;
pub mod extractor;

type Procedure<'a> = Box<dyn Fn(&Item) -> Result<Item> + 'a>;
