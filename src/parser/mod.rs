use std::collections::HashMap;

use anyhow::Result;
use chumsky::{prelude::*, text::newline};

use crate::{Item, Meta};

use super::SingleProcedure;

pub mod markdown;
pub mod template;
pub mod css;

pub trait ParserProcedure: Clone {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)>;
}

fn whitespace<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
    any()
        .and_is(newline().not())
        .filter(|c: &char| c.is_whitespace())
        .ignored()
        .repeated()
        .collect::<Vec<()>>()
        .ignored()
}
