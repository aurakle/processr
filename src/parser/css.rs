use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use lightningcss::{printer::PrinterOptions, stylesheet::{MinifyOptions, ParserOptions, StyleSheet}, targets::Targets};

use crate::data::{Item, Value};

use super::ParserProcedure;

#[derive(Clone)]
pub struct CssCompressor();

impl CssCompressor {
    pub fn default() -> CssCompressor {
        CssCompressor()
    }
}

impl ParserProcedure for CssCompressor {
    fn process(&self, item: &Item) -> Result<(Vec<u8>, HashMap<String, Value>)> {
        let input = String::from_utf8(item.bytes.clone())?;
        let output = StyleSheet::parse(&input, ParserOptions { filename: item.get_filename()?, ..ParserOptions::default() })
            .map_err(|e| anyhow!("Parsing of css failed at {} due to {}", e.loc.map(|loc| loc.to_string()).unwrap_or("<unknown>".to_owned()), e.kind))?
            .to_css(PrinterOptions { minify: true, ..PrinterOptions::default() })
            .map_err(|e| anyhow!("Compression of css failed at {} due to {}", e.loc.map(|loc| loc.to_string()).unwrap_or("<unknown>".to_owned()), e.kind))?;

        Ok((output.code.as_bytes().to_vec(), item.properties.clone()))
    }
}
