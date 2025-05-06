use nom::{branch::alt, bytes::tag, combinator::{map, map_parser}, sequence::delimited, Parser};

use crate::procedure::parser::markdown::{ast::{MarkdownElement, MarkdownElementCollection}, util::take_before1, MarkdownParser};

pub fn all<'a>(parser: &'a MarkdownParser) -> impl Parser<&'a str> {
    alt((
        bold(parser),
        code(parser),
    ))
}

pub struct Bold(MarkdownElementCollection);

impl MarkdownElement for Bold {
    fn as_html(&self) -> String {
        todo!()
    }
}

pub fn bold<'a>(parser: &'a MarkdownParser) -> impl Parser<&'a str> {
    map(
        map_parser(
            delimited(tag("**"), take_before1(tag("**")), tag("**")),
            parser.markdown_element_collection(),
        ),
        |elements| Bold(elements),
    )
}

pub struct Code(MarkdownElementCollection);

impl MarkdownElement for Code {
    fn as_html(&self) -> String {
        todo!()
    }
}

pub fn code<'a>(parser: &'a MarkdownParser) -> impl Parser<&'a str> {
    map(
        map_parser(
            delimited(tag("```"), take_before1(tag("```")), tag("```")),
            parser.markdown_element_collection(),
        ),
        |elements| Code(elements),
    )
}
