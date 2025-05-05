use nom::{bytes::complete::is_not, character::complete::char, error::ParseError, sequence::delimited, AsChar, IResult, Input, Parser};
use anyhow::Result;

use super::Parser as ProcessrParser;

mod extension;

struct MarkdownParser {
    extensions: Vec<()>,
}

impl MarkdownParser {
    fn default() -> Self {
        Self {
            extensions: vec![]
        }
    }
}

impl ProcessrParser for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>) -> Result<Vec<u8>> {
        let text = String::from_utf8(bytes)?;
        todo!()
    }
}

fn brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('['), is_not("]"), char(']')).parse(input)
}

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")"), char(')')).parse(input)
}
