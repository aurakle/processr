use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, newline}};
use extension::MarkdownExtension;
use fronma::parser::parse;
use markdown_ppp::{html_printer::{config::Config, render_html}, parser::{parse_markdown, MarkdownParserState}};

use crate::data::Value;

use super::{whitespace, ParserProcedure};

pub mod extension;

#[derive(Debug, Clone)]
pub struct MarkdownParser {
    extensions: Vec<MarkdownExtension>,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    pub fn extend(&self, extension: MarkdownExtension) -> Self {
        let mut extensions = self.extensions.clone();

        extensions.push(extension);

        Self {
            extensions
        }
    }
}

impl ParserProcedure for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Value>) -> Result<(Vec<u8>, HashMap<String, Value>)> {
        let text = String::from_utf8(bytes.clone())?;
        let data = parse::<HashMap<String, Value>>(&text).map_err(|e| anyhow!("Failed to parse markdown frontmatter"))?;
        let config = Config::default();
        let state = MarkdownParserState::default();
        //TODO: add extensions
        let ast = parse_markdown(state, data.body).map_err(|e| anyhow!("Failed to parse markdown body"))?;

        Ok((render_html(&ast, config).as_bytes().to_vec(), data.headers))
    }
}
