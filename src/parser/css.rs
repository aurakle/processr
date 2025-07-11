use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use lightningcss::{printer::PrinterOptions, stylesheet::{MinifyOptions, ParserOptions, StyleSheet}, targets::Targets};

use crate::data::{Item, State, Value};

use super::ParserProcedure;

#[derive(Clone)]
pub struct CssParser {
    minify: bool,
}

impl CssParser {
    pub fn minify(self) -> Self {
        Self {
            minify: true,
            ..self
        }
    }
}

#[async_trait(?Send)]
impl ParserProcedure for CssParser {
    fn default() -> Self {
        Self {
            minify: false,
        }
    }

    async fn process(&self, state: &mut State, item: &Item) -> Result<Item> {
        let input = String::from_utf8(item.bytes.clone())?;
        let output = StyleSheet::parse(&input, ParserOptions { filename: item.get_filename()?, ..ParserOptions::default() })
            .map_err(|e| anyhow!("Parsing of css failed at {} due to {}", e.loc.map(|loc| loc.to_string()).unwrap_or("<unknown>".to_owned()), e.kind))?
            .to_css(PrinterOptions { minify: self.minify, ..PrinterOptions::default() })
            .map_err(|e| anyhow!("Compression of css failed at {} due to {}", e.loc.map(|loc| loc.to_string()).unwrap_or("<unknown>".to_owned()), e.kind))?;

        Ok(Item {
            bytes: output.code.as_bytes().to_vec(),
            ..item.clone()
        })
    }
}
