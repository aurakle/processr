use nom::{branch::alt, bytes::tag, combinator::{map, map_parser}, error::ParseError, sequence::delimited, Parser};

use crate::procedure::parser::markdown::{ast::{MarkdownElement, MarkdownElementCollection}, util::take_before1, MarkdownParser};

use super::MarkdownExtension;

pub fn all() -> Vec<MarkdownExtension> {
    vec![
        MarkdownExtension::from(bold)
    ]
}

pub struct Bold(pub MarkdownElementCollection);

impl MarkdownElement for Bold {
    fn as_html(&self) -> String {
        todo!()
    }
}

pub fn bold<'a>(parser: &MarkdownParser) -> impl Parser<&'a str, Output = Box<dyn MarkdownElement>> {
    map(
        map_parser(
            delimited(tag("**"), take_before1(tag("**")), tag("**")),
            parser.markdown_element_collection(),
        ),
        |elements| Box::new(Bold(elements)),
    )
}

// pub struct Code(pub MarkdownElementCollection);
//
// impl MarkdownElement for Code {
//     fn as_html(&self) -> String {
//         todo!()
//     }
// }
//
// pub fn code(parser: &MarkdownParser) -> impl Parser<&str> {
//     map(
//         map_parser(
//             delimited(tag("```"), take_before1(tag("```")), tag("```")),
//             parser.markdown_element_collection(),
//         ),
//         |elements| Code(elements),
//     )
// }
