use std::collections::HashMap;

use nom::{branch::{alt, Choice}, combinator::map, error::Error, multi::many0, Parser};
use anyhow::{bail, Result};

use crate::procedure::parser::Parser as ParserProcedure;

use super::{ast::{MarkdownDocument, MarkdownElement, MarkdownElementCollection, Plain}, extension::{self, MarkdownExtension, MarkdownExtensionParser}, util::{rest1, take_before0}};

pub struct MarkdownParser<'a> {
    extensions: Vec<MarkdownExtension<'a>>,
    built_extensions: Option<Choice<&'a mut [MarkdownExtension<'a>]>>,
}

impl<'a> MarkdownParser<'a> {
    fn make_parser(self: &'a mut MarkdownParser<'a>) -> Result<impl Parser<&str, Output = MarkdownDocument>> {

        Ok(map(self.markdown_element_collection(), MarkdownDocument::from))
    }

    pub fn default() -> Self {
        Self(extension::default)
    }

    pub fn build(&mut self) -> Result<Self> {
        match self.built_extensions {
            Some(_) => bail!("This parser has already been built"),
            None => self.built_extensions = Some(alt(self.extensions.as_mut_slice())),
        }

        Ok(self)
    }

    pub fn extend(&mut self, extension: Box<dyn for<'b> Fn(&MarkdownParser<'b>) -> MarkdownExtensionParser>) -> Result<Self> {
        match self.built_extensions {
            Some(_) => bail!("This parser has already been built"),
            None => self.extensions.push(MarkdownExtension::new(self, extension)),
        }

        Ok(self)
    }

    pub fn markdown_element(&self) -> impl Parser<&str, Output = Box<dyn MarkdownElement>, Error = Error<&str>> {
        alt((self.markdown_element_not_plain(), self.plain()))
    }

    pub fn markdown_element_collection(&self) -> impl Parser<&str, Output = MarkdownElementCollection, Error = Error<&str>> {
        map(many0(self.markdown_element()), MarkdownElementCollection::from)
    }

    fn markdown_element_not_plain(&self) -> impl Parser<&str, Output = Box<dyn MarkdownElement>, Error = Error<&str>> {
        //TODO: can't copy :scream:
        self.built_extensions.unwrap() // this is safe because we always `make_parser` first
    }

    fn plain(&self) -> impl Parser<&str, Output = Box<dyn MarkdownElement>, Error = Error<&str>> {
        map(
            alt((take_before0(self.markdown_element_not_plain()), rest1)),
            |text| Box::new(Plain(text.to_string())) as Box<dyn MarkdownElement>,
        )
    }
}

impl ParserProcedure for MarkdownParser<'_> {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, String>) -> Result<(Vec<u8>, HashMap<String, String>)> {
        let mut nom_parser = self.make_parser();
        let text = String::from_utf8(bytes.clone())?;
        //TODO: parse properties and add them to the hash map
        let properties = HashMap::new();
        let (_no_idea_what_this_is, document) = nom_parser.parse(text.as_str())?;

        Ok((document.as_html().as_bytes().to_vec(), properties))
    }
}
