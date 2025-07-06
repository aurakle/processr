use std::collections::HashMap;
use anyhow::{Result, anyhow};
use chumsky::prelude::*;

use crate::Meta;

use super::ParserProcedure;

#[derive(Clone)]
pub struct CssCompressor();

impl CssCompressor {
    pub fn default() -> CssCompressor {
        CssCompressor()
    }
}

impl ParserProcedure for CssCompressor {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)> {
        let input = String::from_utf8(bytes.clone())?;
        let output = make_parser().parse(input.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse css"))?;

        Ok((output.as_bytes().to_vec(), properties.clone()))
    }
}

fn make_parser<'src>() -> impl Parser<'src, &'src str, String> {
    let escaped = choice((
        any()
            .and_is(just('\'').not())
            .repeated()
            .collect()
            .padded_by(just('\'')),
        any()
            .and_is(just('\"').not())
            .repeated()
            .collect()
            .padded_by(just('\"')),
    ));

    escaped
        .or(any()
            .and_is(escaped.not())
            .repeated()
            .collect::<String>()
            .map(|s| {
                let mut s = s;
                let mut last_len = s.len() * 2;

                while s.len() < last_len {
                    last_len = s.len();
                    s = s
                        .replace("{ ", "{")
                        .replace(" {", "{")
                        .replace("} ", "}")
                        .replace(" }", "}")
                        .replace(": ", ":")
                        .replace("; ", ";")
                        .replace("	", " ")
                        .replace("  ", " ");
                }

                s
            }))
        .repeated()
        .collect::<Vec<String>>()
        .map(|v| v.concat())
}
