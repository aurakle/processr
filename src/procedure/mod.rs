use anyhow::Result;

use crate::Item;

mod util;
mod parser;

type Procedure<'a> = Box<dyn Fn(&Item) -> Result<Item> + 'a>;
