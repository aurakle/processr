use std::collections::HashMap;

use nom::{branch::{alt, Choice}, combinator::map, error::ParseError, multi::many0, IResult, Map, Parser};
use anyhow::Result;

use crate::procedure::parser::Parser as ParserProcedure;

use super::{ast::{MarkdownElement, MarkdownElementCollection, Plain}, extension::{self, MarkdownExtension}};

pub struct MarkdownParser {
    parsers: Vec<MarkdownExtension>,
}

impl MarkdownParser {
    fn make_parser(&self) -> impl Parser<&str, Output = Plain> {
        map(self.markdown_element_collection(), |elements| Plain(elements))
    }

    pub fn default() -> Self {
        Self(extension::default)
    }


    pub fn markdown_element(&self) -> impl Parser<&str> {
        alt(self.parsers.as_slice())
    }

    pub fn markdown_element_collection<'a>(&self) -> impl Fn(&'a str) -> IResult<&'a str, MarkdownElementCollection> {
        map(many0(self.markdown_element()), MarkdownElementCollection::from)
    }
}

impl ParserProcedure for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, String>) -> Result<(Vec<u8>, HashMap<String, String>)> {
        let mut nom_parser = self.make_parser();
        let text = String::from_utf8(bytes.clone())?;
        //TODO: parse properties and add them to the hash map
        let properties = HashMap::new();
        let (_no_idea_what_this_is, document) = nom_parser.parse(text.as_str())?;

        Ok((document.as_html().into(), properties))
    }
}
