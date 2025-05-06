use std::collections::HashMap;

use nom::{branch::{alt, Choice}, combinator::map, error::ParseError, multi::many0, Parser};
use anyhow::Result;

use crate::procedure::parser::Parser as ParserProcedure;

use super::{ast::{MarkdownElement, MarkdownElementCollection, Plain}, extension};

pub struct MarkdownParser<'a, A: for<'b> Parser<&'b str, Output = dyn MarkdownElement, Error = ParseError<&'a str>>> {
    element_parser: Choice<&'a mut [A]>,
}

impl MarkdownParser<'_> {
    fn make_parser(&self) -> impl Parser<&str, Output = Plain> {
        map(self.markdown_element_collection(), |elements| Plain(elements))
    }

    pub fn default() -> Self {
        Self(alt(())).extend(extension::default)
    }

    pub fn extend(self, extension: fn(&Self) -> Parser<&str, Output = dyn MarkdownElement, Error = ParseError<&str>>) -> Self {
        Self(alt((self.element_parser, extension)))
    }

    pub fn markdown_element(&self) -> impl Parser<&str> {
        self.element_parser
    }

    pub fn markdown_element_collection(&self) -> impl Parser<&str> {
        map(many0(self.markdown_element()), MarkdownElementCollection::from)
    }
}

impl ParserProcedure for MarkdownParser<'_> {
    fn process(&self, bytes: &Vec<u8>) -> Result<(Vec<u8>, HashMap<String, String>)> {
        let mut nom_parser = self.make_parser();
        let text = String::from_utf8(bytes.clone())?;
        //TODO: parse properties and add them to the hash map
        let properties = HashMap::new();
        let (_no_idea_what_this_is, document) = nom_parser.parse(text.as_str())?;

        Ok((document.as_html().into(), properties))
    }
}
