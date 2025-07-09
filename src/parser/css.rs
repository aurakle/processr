use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use lightningcss::{printer::PrinterOptions, stylesheet::{MinifyOptions, ParserOptions, StyleSheet}, targets::Targets};

use crate::data::{Item, Value};

use super::ParserProcedure;

#[derive(Clone)]
pub struct CssParser {
    minify: bool,
}

impl CssParser {
    pub fn default() -> Self {
        Self {
            minify: false,
        }
    }

    pub fn minify(self) -> Self {
        Self {
            minify: true,
            ..self
        }
    }
}

impl ParserProcedure for CssParser {
    fn process(&self, item: &Item) -> Result<Item> {
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
