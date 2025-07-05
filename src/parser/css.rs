use std::collections::HashMap;
use anyhow::Result;

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
        let mut text = String::from_utf8(bytes.clone())?.replace("\n", "");
        let mut last_len = text.len() * 2;

        while text.len() < last_len {
            last_len = text.len();
            text = text
                .replace("{ ", "{")
                .replace("} ", "}")
                .replace(": ", ":")
                .replace("; ", ";")
                .replace("	", " ")
                .replace("  ", " ");
        }

        Ok((text.as_bytes().to_vec(), properties.clone()))
    }
}
