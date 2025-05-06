use nom::{bytes::tag, combinator::{map, map_parser}, sequence::delimited, Parser};

use crate::procedure::parser::markdown::{ast::{MarkdownElement, MarkdownElementCollection}, util::take_before1, MarkdownParser};

use super::{MarkdownExtension, MarkdownExtensionParser};

pub fn all<'a>(parser: &'a MarkdownParser) -> Vec<MarkdownParser<'a>> {
    vec![
        MarkdownExtension::new(parser, bold),
    ]
}

pub struct Bold(pub MarkdownElementCollection);

impl MarkdownElement for Bold {
    fn as_html(&self) -> String {
        todo!()
    }
}

pub fn bold<'a>(parser: &'a MarkdownParser<'a>) -> MarkdownExtensionParser {
    Box::new(|i: &'a str| {
        map(
            map_parser(
                delimited(tag("**"), take_before1(tag("**")), tag("**")),
                parser.markdown_element_collection(),
            ),
            |elements| Box::new(Bold(elements)) as Box<dyn MarkdownElement>,
        ).parse(i)
    })
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
