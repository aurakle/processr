use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use chumsky::{prelude::*, text::newline};

use crate::data::{Item, Value};

pub mod markdown;
pub mod template;
pub mod html;
pub mod css;

#[async_trait(?Send)]
pub trait ParserProcedure: Clone {
    async fn process(&self, item: &Item) -> Result<Item>;
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

fn line_terminator<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
    choice((
        newline(),
        end(),
    ))
}

fn fail<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
    end().and_is(end().not())
}
