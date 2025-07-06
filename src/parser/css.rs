use std::collections::HashMap;
use anyhow::{Result, anyhow};
use chumsky::prelude::*;

use crate::data::Value;

use super::ParserProcedure;

#[derive(Clone)]
pub struct CssCompressor();

impl CssCompressor {
    pub fn default() -> CssCompressor {
        CssCompressor()
    }
}

impl ParserProcedure for CssCompressor {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Value>) -> Result<(Vec<u8>, HashMap<String, Value>)> {
        let input = String::from_utf8(bytes.clone())?;
        let output = make_parser().parse(input.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse css"))?;

        Ok((output.as_bytes().to_vec(), properties.clone()))
    }
}

fn make_parser<'src>() -> impl Parser<'src, &'src str, String> {
    let escaped = choice((
        any()
            .and_is(just("*/").not())
            .repeated()
            .delimited_by(just("/*"), just("*/"))
            .to(String::new()),
        any()
            .and_is(just('\"').not())
            .repeated()
            .padded_by(just('\"'))
            .to_slice()
            .map(String::from),
        any()
            .and_is(just('\'').not())
            .repeated()
            .padded_by(just('\''))
            .to_slice()
            .map(String::from),
    ));

    escaped.clone()
        .or(any()
            .and_is(escaped.not())
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(|s| {
                let mut s = s.replace("\n", "");
                let mut last_len = s.len() * 2;

                while s.len() < last_len {
                    last_len = s.len();
                    s = s
                        .replace("{ ", "{")
                        .replace(" {", "{")
                        .replace("} ", "}")
                        .replace(" }", "}")
                        .replace(": ", ":")
                        .replace(" :", ":")
                        .replace("; ", ";")
                        .replace(" ;", ";")
                        .replace(", ", ",")
                        .replace(" ,", ",")
                        .replace("	", " ")
                        .replace("  ", " ");
                }

                s
            }))
        .repeated()
        .collect::<Vec<String>>()
        .map(|v| v.concat())
}
