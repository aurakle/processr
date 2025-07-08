use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::ident};
use extension::MarkdownExtension;
use fronma::parser::parse;

use crate::data::Value;

use super::ParserProcedure;

pub mod extension;

#[derive(Clone)]
pub struct MarkdownParser {
    extensions: Vec<MarkdownExtension>,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    pub fn extend(&self, extension: MarkdownExtension) -> Self {
        let mut extensions = self.extensions.clone();

        extensions.push(extension);

        Self {
            extensions
        }
    }
}

impl ParserProcedure for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Value>) -> Result<(Vec<u8>, HashMap<String, Value>)> {
        let text = String::from_utf8(bytes.clone())?;
        let data = parse::<HashMap<String, Value>>(&text).map_err(|e| {
            match e {
                fronma::error::Error::MissingBeginningLine => anyhow!("Markdown document is missing frontmatter"),
                fronma::error::Error::MissingEndingLine => anyhow!("Frontmatter is missing closing triple dash"),
                fronma::error::Error::SerdeYaml(e) => anyhow!("Failed to parse YAML frontmatter: {}", e),
            }
        })?;

        let res = make_parser(&self.extensions).parse(data.body).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;
        let mut properties = properties.clone();

        properties.extend(data.headers);

        Ok((res.as_bytes().to_vec(), properties))
    }
}

fn make_parser<'src>(extensions: &Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    block(extensions.clone())
        .repeated()
        .collect::<Vec<String>>()
        .map(|elements| elements.concat())
}

fn block<'src>(extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(|this| {
        let inline = inline(this.clone(), extensions.clone());
        let mut block = choice((
            // headers
            inline.clone()
                .nested_in(just("-# ")
                    .ignore_then(any()
                        .and_is(just('\n').not())
                        .repeated()
                        .at_least(1)
                        .to_slice()))
                //TODO: is the <br/> really necessary?
                .map(|s| format!("<br/><small>{}</small>", s)),
            inline.clone()
                .nested_in(just("### ")
                    .ignore_then(any()
                        .and_is(just('\n').not())
                        .repeated()
                        .at_least(1)
                        .to_slice()))
                .map(|s| format!("<h3>{}</h3>", s)),
            inline.clone()
                .nested_in(just("## ")
                    .ignore_then(any()
                        .and_is(just('\n').not())
                        .repeated()
                        .at_least(1)
                        .to_slice()))
                .map(|s| format!("<h2>{}</h2>", s)),
            inline.clone()
                .nested_in(just("# ")
                    .ignore_then(any()
                        .and_is(just('\n').not())
                        .repeated()
                        .at_least(1)
                        .to_slice()))
                .map(|s| format!("<h1>{}</h1>", s)),
        )).boxed();

        choice((
            block,
            // paragraph
            this.clone()
                .nested_in(just("\n\n\n")
                    .ignore_then(any()
                        .and_is(just("\n\n\n").not())
                        .repeated()
                        .at_least(1)
                        .to_slice()))
                .map(|s| format!("<p>{}</p>", s)),
            // line break
            just("\n\n").to(format!("<br/>")),
            // everything else
            inline,
        ))
    })
}

fn inline<'src>(block: Recursive<dyn Parser<'src, &'src str, String> + 'src>, extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(|this| {
        let mut inline = choice((
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
                .ignore_then(ident().then_ignore(just('\n')).or_not())
                .then(any()
                    .and_is(just("```").not())
                    .repeated()
                    .at_least(1)
                    .to_slice())
                .then_ignore(just("```"))
                .map(|(lang, inner)| {
                    let inner = html_escape::encode_safe(inner);
                    match lang {
                        Some(lang) => format!("<pre><code class=\"language-{}\">{}</code></pre>", lang, inner),
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
                        .collect::<String>()
                        .then(just('*')
                            .and_is(just("***"))
                            .repeated()
                            .collect::<String>())
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
                        .collect::<String>()
                        .then(just('*')
                            .and_is(just("**"))
                            .repeated()
                            .collect::<String>())
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
                        .collect::<String>()
                        .then(just('~')
                            .and_is(just("~~~"))
                            .repeated()
                            .collect::<String>())
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
                        .collect::<String>()
                        .then(just('_')
                            .and_is(just("___"))
                            .repeated()
                            .collect::<String>())
                        .to_slice())
                    .then_ignore(just("__")))
                .map(|inner| format!("<u>{}</u>", inner)),
        )).boxed();

        choice((
            // escape char
            just("\\")
                .ignore_then(any()
                    .map(|c| format!("{}", c))),
            // manual wrapping
            just('\n')
                .and_is(block.not())
                .to(format!("")),
            inline.clone(),
            none_of("\n")
                .and_is(inline.not())
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )).repeated().at_least(1).collect::<Vec<String>>().map(|elements| elements.concat())
    })
}

#[cfg(test)]
mod tests {
    mod document {
        use chumsky::Parser;

        use crate::parser::markdown::make_parser;
    }

    mod block {
        use chumsky::Parser;

        use crate::parser::markdown::block;

        #[test]
        fn header1() {
            let p = block(vec![]);
            let res = p.parse("# meow").into_result().unwrap();
            let expected = format!("<h1>meow</h1>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header2() {
            let p = block(vec![]);
            let res = p.parse("## meow").into_result().unwrap();
            let expected = format!("<h2>meow</h2>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header3() {
            let p = block(vec![]);
            let res = p.parse("### meow").into_result().unwrap();
            let expected = format!("<h3>meow</h3>");

            assert_eq!(expected, res);
        }

        #[test]
        fn small() {
            let p = block(vec![]);
            let res = p.parse("-# meow").into_result().unwrap();
            let expected = format!("<br/><small>meow</small>");

            assert_eq!(expected, res);
        }

        #[test]
        fn paragraph() {
            let p = block(vec![]);
            let res = p.parse("\n\n\nmeow").into_result().unwrap();
            let expected = format!("<p>meow</p>");

            assert_eq!(expected, res);
        }

        #[test]
        fn paragraph_with_bold_and_italics() {
            let p = block(vec![]);
            let res = p.parse("\n\n\n***meow***").into_result().unwrap();
            let expected = format!("<p><b><i>meow</i></b></p>");

            assert_eq!(expected, res);
        }
    }

    mod inline {
        use chumsky::Parser;

        use crate::parser::markdown::block;

        #[test]
        fn code_block() {
            let p = block(vec![]);
            let res = p.parse("```meow```").into_result().unwrap();
            let expected = format!("<pre><code>meow</code></pre>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_line() {
            let p = block(vec![]);
            let res = p.parse("`meow`").into_result().unwrap();
            let expected = format!("<code>meow</code>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold() {
            let p = block(vec![]);
            let res = p.parse("**meow**").into_result().unwrap();
            let expected = format!("<b>meow</b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn italic() {
            let p = block(vec![]);
            let res = p.parse("*meow*").into_result().unwrap();
            let expected = format!("<i>meow</i>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold_and_italic() {
            let p = block(vec![]);
            let res = p.parse("***meow***").into_result().unwrap();
            let expected = format!("<b><i>meow</i></b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn strikethrough() {
            let p = block(vec![]);
            let res = p.parse("~~meow~~").into_result().unwrap();
            let expected = format!("<s>meow</s>");

            assert_eq!(expected, res);
        }

        #[test]
        fn underline() {
            let p = block(vec![]);
            let res = p.parse("__meow__").into_result().unwrap();
            let expected = format!("<u>meow</u>");

            assert_eq!(expected, res);
        }
    }
}
