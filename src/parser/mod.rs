use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use chumsky::{prelude::*, text::newline};

use crate::data::{Item, State, Value};

pub mod markdown;
pub mod template;
pub mod image;
pub mod html;
pub mod css;

#[async_trait(?Send)]
pub trait ParserProcedure: Clone {
    fn default() -> Self;
    async fn process(&self, state: &mut State, item: &Item) -> Result<Item>;
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
