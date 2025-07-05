use anyhow::{anyhow, Result};
use chumsky::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::Meta;

use super::Parser as ParserProcedure;

#[derive(Debug, Clone)]
pub struct TemplateParser {
    template: String,
}

impl TemplateParser {
    pub fn new(path: PathBuf) -> Result<TemplateParser> {
        Ok(Self {
            template: fs::read_to_string(path)?,
        })
    }

    fn make_parser<'src>(&self, properties: HashMap<String, Meta>) -> impl Parser<'src, &'src str, String> {
        todo()
    }
}

impl ParserProcedure for TemplateParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)> {
        let mut props = properties.clone();
        props.insert(format!("body"), Meta::from(String::from_utf8(bytes.clone())?));

        let parser = self.make_parser(props);
        let text = parser.parse(self.template.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse template"))?;

        Ok((text.as_bytes().to_vec(), properties.clone()))
    }
}
