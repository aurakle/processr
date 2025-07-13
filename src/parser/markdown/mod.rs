use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chumsky::{prelude::*, text::{ident, newline}};
use extension::{MarkdownExtension, MarkdownExtensionList};
use fronma::parser::parse;

use crate::data::{Item, State, Value};

use super::{line_terminator, ParserProcedure};

pub mod extension;

#[derive(Clone)]
pub struct MarkdownParser {
    extensions: Vec<MarkdownExtension>,
}

impl MarkdownParser {
    pub fn extend(&self, extension: MarkdownExtension) -> Self {
        let mut extensions = self.extensions.clone();

        extensions.push(extension);

        Self {
            extensions
        }
    }
}

#[async_trait(?Send)]
impl ParserProcedure for MarkdownParser {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    async fn process(&self, state: &mut State, item: &Item) -> Result<Item> {
        let text = String::from_utf8(item.bytes.clone())?;
        let data = parse::<HashMap<String, Value>>(&text).map_err(|e| {
            match e {
                fronma::error::Error::MissingBeginningLine => anyhow!("Markdown document is missing frontmatter"),
                fronma::error::Error::MissingEndingLine => anyhow!("Frontmatter is missing closing triple dash"),
                fronma::error::Error::SerdeYaml(e) => anyhow!("Failed to parse YAML frontmatter: {}", e),
            }
        })?;

        let body = format!("\n\n\n{}", data.body.to_owned().trim_start());
        let res = make_parser(&self.extensions).parse(&body).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;

        let mut properties = item.properties.clone();
        properties.extend(data.headers);

        Ok(Item {
            bytes: res.as_bytes().to_vec(),
            properties,
            ..item.clone()
        })
    }
}

fn make_parser<'src>(extensions: &Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(|this| {
        let block = block(this.clone(), extensions.clone()).boxed();
        choice((
            block.clone(),
            inline(this.clone(), block, extensions.clone()),
        ))
            .repeated()
            .at_least(1)
            .collect::<Vec<String>>()
            .map(|elements| elements.concat())
    })
}

fn block<'src>(parser: Recursive<dyn Parser<'src, &'src str, String> + 'src>, extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(|this| {
        let inline = inline(parser.clone(), this.clone().boxed(), extensions.clone());
        let block = newline()
            .repeated()
            .ignore_then(
                choice((
                    // headers
                    inline.clone()
                        .nested_in(just("### ")
                            .ignore_then(any()
                                .and_is(line_terminator().not())
                                .repeated()
                                .at_least(1)
                                .to_slice()))
                        .map(|s| format!("<h3>{}</h3>", s)),
                    inline.clone()
                        .nested_in(just("## ")
                            .ignore_then(any()
                                .and_is(line_terminator().not())
                                .repeated()
                                .at_least(1)
                                .to_slice()))
                        .map(|s| format!("<h2>{}</h2>", s)),
                    inline.clone()
                        .nested_in(just("# ")
                            .ignore_then(any()
                                .and_is(line_terminator().not())
                                .repeated()
                                .at_least(1)
                                .to_slice()))
                        .map(|s| format!("<h1>{}</h1>", s)),
                    // thematic break
                    just("---")
                        .to(format!("<hr/>")),
                    extensions.build_block_parser(inline.clone().boxed()),
                )))
            .boxed();
        let paragraph = recursive(|paragraph| {
            this.clone()
                .and_is(paragraph.not())
                .or(inline)
                .repeated()
                .collect::<Vec<String>>()
                .map(|elements| elements.concat())
                .nested_in(newline()
                    .repeated()
                    .at_least(3)
                    .ignore_then(any()
                        .and_is(newline()
                            .repeated()
                            .at_least(3)
                            .not())
                        .repeated()
                        .at_least(1)
                        .to_slice()))
                .map(|s| format!("<p>{}</p>", s))
        });

        choice((
            block,
            paragraph,
            // line break
            newline().repeated().exactly(2).to(format!("<br/>")),
        ))
    })
}

fn inline<'src>(parser: Recursive<dyn Parser<'src, &'src str, String> + 'src>, block: Boxed<'src, 'src, &'src str, String>, extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(|this| {
        let inline = choice((
            // image
            just('!')
                .ignore_then(
                    group((
                        this.clone()
                            .nested_in(any()
                                .and_is(just(']').not())
                                .repeated()
                                .to_slice())
                            .or_not()
                            .delimited_by(just('['), just(']')),
                        this.clone()
                            .nested_in(any()
                                .and_is(just(')').not())
                                .repeated()
                                .to_slice())
                            .or_not()
                            .delimited_by(just('('), just(')')),
                    )))
                .map(|(text, link)| {
                    format!("<img src=\"{}\" alt=\"{}\"/>", link.unwrap_or_else(String::new), text.unwrap_or_else(String::new))
                }),
            // link
            group((
                this.clone()
                    .nested_in(any()
                        .and_is(just(']').not())
                        .repeated()
                        .to_slice())
                    .or_not()
                    .delimited_by(just('['), just(']')),
                this.clone()
                    .nested_in(any()
                        .and_is(just(')').not())
                        .repeated()
                        .to_slice())
                    .or_not()
                    .delimited_by(just('('), just(')')),
            ))
                .map(|(text, link)| {
                    format!("<a href=\"{}\">{}</a>", link.unwrap_or_else(String::new), text.unwrap_or_else(String::new))
                }),
            // code block
            just("```")
                .ignore_then(any()
                    .and_is(newline().not())
                    .and_is(just("```").not())
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .then_ignore(newline())
                    .or_not())
                .then(any()
                    .and_is(just("```").not())
                    .repeated()
                    .at_least(1)
                    .to_slice())
                .then_ignore(just("```"))
                .map(|(special, inner)| {
                    let inner = html_escape::encode_safe(inner);
                    match special {
                        Some(special) => match special.rsplit_once(".") {
                            Some((_, language)) => format!("<pre><small>{}</small><code class=\"language-{}\">{}</code></pre>", special, language, inner),
                            None => format!("<pre><code class=\"language-{}\">{}</code></pre>", special, inner),
                        },
                        None => format!("<pre><code>{}</code></pre>", inner),
                    }
                }),
            // code line
            any()
                .and_is(just('`').not())
                .repeated()
                .at_least(1)
                .to_slice()
                .padded_by(just('`'))
                .map(|inner| format!("<code>{}</code>", html_escape::encode_safe(inner))),
            // bold
            this.clone()
                .nested_in(just("**")
                    .ignore_then(any()
                        .and_is(just("**").not())
                        .repeated()
                        .at_least(1)
                        .then(just('*')
                            .and_is(just("***"))
                            .repeated())
                        .to_slice())
                    .then_ignore(just("**")))
                .map(|inner| format!("<b>{}</b>", inner)),
            // italic
            this.clone()
                .nested_in(just('*')
                    .ignore_then(any()
                        .and_is(just('*').not())
                        .repeated()
                        .at_least(1)
                        .then(just('*')
                            .and_is(just("**"))
                            .repeated())
                        .to_slice())
                    .then_ignore(just('*')))
                .map(|inner| format!("<i>{}</i>", inner)),
            // strikethrough
            this.clone()
                .nested_in(just("~~")
                    .ignore_then(any()
                        .and_is(just("~~").not())
                        .repeated()
                        .at_least(1)
                        .then(just('~')
                            .and_is(just("~~~"))
                            .repeated())
                        .to_slice())
                    .then_ignore(just("~~")))
                .map(|inner| format!("<s>{}</s>", inner)),
            // underline
            this.clone()
                .nested_in(just("__")
                    .ignore_then(any()
                        .and_is(just("__").not())
                        .repeated()
                        .at_least(1)
                        .then(just('_')
                            .and_is(just("___"))
                            .repeated())
                        .to_slice())
                    .then_ignore(just("__")))
                .map(|inner| format!("<u>{}</u>", inner)),
            extensions.build_inline_parser(this.boxed()),
        )).boxed();

        choice((
            // escape char
            just("\\")
                .ignore_then(any()
                    .map(|c| format!("{}", c))),
            // manual wrapping
            newline()
                .repeated()
                .exactly(1)
                .to(format!("")),
            inline.clone(),
            any()
                .and_is(newline().not())
                .and_is(inline.not())
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )).and_is(block.not()).repeated().at_least(1).collect::<Vec<String>>().map(|elements| elements.concat())
    })
}

#[cfg(test)]
mod tests {
    mod block {
        use chumsky::Parser;

        use crate::parser::markdown::make_parser;

        #[test]
        fn header1() {
            let p = make_parser(&vec![]);
            let res = p.parse("# meow").into_result().unwrap();
            let expected = format!("<h1>meow</h1>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header2() {
            let p = make_parser(&vec![]);
            let res = p.parse("## meow").into_result().unwrap();
            let expected = format!("<h2>meow</h2>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header3() {
            let p = make_parser(&vec![]);
            let res = p.parse("### meow").into_result().unwrap();
            let expected = format!("<h3>meow</h3>");

            assert_eq!(expected, res);
        }

        #[test]
        fn paragraph() {
            let p = make_parser(&vec![]);
            let res = p.parse("\n\n\nmeow").into_result().unwrap();
            let expected = format!("<p>meow</p>");

            assert_eq!(expected, res);
        }

        #[test]
        fn paragraph_with_bold_and_italics() {
            let p = make_parser(&vec![]);
            let res = p.parse("\n\n\n***meow***").into_result().unwrap();
            let expected = format!("<p><b><i>meow</i></b></p>");

            assert_eq!(expected, res);
        }

        #[test]
        fn paragraph_followed_by_header() {
            let p = make_parser(&vec![]);
            let res = p.parse("\n\n\nmeow\n# Some header").into_result().unwrap();
            let expected = format!("<p>meow<h1>Some header</h1></p>");

            assert_eq!(expected, res);
        }

        #[test]
        fn paragraph_followed_by_fake_header() {
            let p = make_parser(&vec![]);
            let res = p.parse("\n\n\nmeow# Some header").into_result().unwrap();
            let expected = format!("<p>meow# Some header</p>");

            assert_eq!(expected, res);
        }
    }

    mod inline {
        use chumsky::Parser;

        use crate::parser::markdown::make_parser;

        #[test]
        fn image_embed() {
            let p = make_parser(&vec![]);
            let res = p.parse("![this is an image](https://it.is.from.here)").into_result().unwrap();
            let expected = format!("<img src=\"https://it.is.from.here\" alt=\"this is an image\"/>");

            assert_eq!(expected, res);
        }

        #[test]
        fn link_embed() {
            let p = make_parser(&vec![]);
            let res = p.parse("[this is a link](https://it.goes.here)").into_result().unwrap();
            let expected = format!("<a href=\"https://it.goes.here\">this is a link</a>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_block() {
            let p = make_parser(&vec![]);
            let res = p.parse("```meow```").into_result().unwrap();
            let expected = format!("<pre><code>meow</code></pre>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_line() {
            let p = make_parser(&vec![]);
            let res = p.parse("`meow`").into_result().unwrap();
            let expected = format!("<code>meow</code>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold() {
            let p = make_parser(&vec![]);
            let res = p.parse("**meow**").into_result().unwrap();
            let expected = format!("<b>meow</b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn italic() {
            let p = make_parser(&vec![]);
            let res = p.parse("*meow*").into_result().unwrap();
            let expected = format!("<i>meow</i>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold_and_italic() {
            let p = make_parser(&vec![]);
            let res = p.parse("***meow***").into_result().unwrap();
            let expected = format!("<b><i>meow</i></b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn strikethrough() {
            let p = make_parser(&vec![]);
            let res = p.parse("~~meow~~").into_result().unwrap();
            let expected = format!("<s>meow</s>");

            assert_eq!(expected, res);
        }

        #[test]
        fn underline() {
            let p = make_parser(&vec![]);
            let res = p.parse("__meow__").into_result().unwrap();
            let expected = format!("<u>meow</u>");

            assert_eq!(expected, res);
        }
    }
}
