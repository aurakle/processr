use anyhow::{anyhow, Result};
use chumsky::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::Meta;

use super::Parser as ParserProcedure;

#[derive(Debug, Clone)]
pub struct TemplateParser();

impl TemplateParser {
    pub fn default() -> TemplateParser {
        Self()
    }

    fn make_parser<'src>(&self, properties: HashMap<String, Meta>) -> impl Parser<'src, &'src str, String> {
        todo()
    }
}

impl ParserProcedure for TemplateParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)> {
        let text = String::from_utf8(bytes.clone())?;
        let parser = self.make_parser(properties.clone());
        let text = parser.parse(text.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;

        Ok((text.as_bytes().to_vec(), properties.clone()))
    }
}
