use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, newline}};
use extension::MarkdownExtension;
use fronma::parser::parse;
use markdown_ppp::{html_printer::{config::Config, render_html}, parser::{config::MarkdownParserConfig, parse_markdown, MarkdownParserState}};

use crate::data::Value;

use super::{whitespace, ParserProcedure};

pub mod extension;

#[derive(Clone)]
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

    fn configure(&self) -> MarkdownParserConfig {
        let mut config = MarkdownParserConfig::default();

        for extension in self.extensions.clone() {
            //TODO: this will only use the LAST EXTENSION REGISTERED
            // this must be fixed ASAP
            config = match extension {
                MarkdownExtension::Inline(func) => config.with_custom_inline_parser(func),
                MarkdownExtension::Block(func) => config.with_custom_block_parser(func),
            }
        }

        config
    }
}

impl ParserProcedure for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Value>) -> Result<(Vec<u8>, HashMap<String, Value>)> {
        let text = String::from_utf8(bytes.clone())?;
        let data = parse::<HashMap<String, Value>>(&text).map_err(|e| {
            match e {
                fronma::error::Error::MissingBeginningLine => anyhow!("Markdown document is missing frontmatter"),
                fronma::error::Error::MissingEndingLine => anyhow!("Frontmatter is missing closing triple dash"),
                fronma::error::Error::SerdeYaml(e) => anyhow!("Failed to parse YAML frontmatter: {}", e),
            }
        })?;

        let parser_config = self.configure();
        let state = MarkdownParserState::with_config(parser_config);
        let ast = parse_markdown(state, data.body).map_err(|e| anyhow!("Failed to parse markdown body: {}", e))?;

        let printer_config = Config::default();

        Ok((render_html(&ast, printer_config).as_bytes().to_vec(), data.headers))
    }
}
