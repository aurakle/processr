use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, keyword, newline}};
use extension::MarkdownExtension;
use fronma::parser::parse;

use crate::data::Value;

use super::{whitespace, ParserProcedure};

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

        let res = make_parser(self.extensions.clone()).parse(data.body).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;
        let mut properties = properties.clone();

        properties.extend(data.headers);

        Ok((res.as_bytes().to_vec(), properties))
    }
}

fn make_parser<'src>(extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    block(extensions.clone())
        .or(inline(extensions))
        .repeated()
        .collect::<Vec<String>>()
        .map(|elements| elements.concat())
}

fn block<'src>(extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    let extensions1 = extensions.clone();
    let closure = move |inner: String, _span| {
        inline(extensions1.clone()).parse(inner.as_ref()).into_result().map_err(|_e| EmptyErr::default())
    };
    let mut block = choice((
        // headers
        //TODO: headers, due to requiring a newline, don't behave well when the other parsers
        //match
        just("# ")
            .ignore_then(any()
                .and_is(just('\n').not())
                .repeated()
                .collect()
                .try_map(closure.clone()))
                .map(|s| format!("<h1>{}</h1>", s)),
        just("## ")
            .ignore_then(any()
                .and_is(just('\n').not())
                .repeated()
                .collect()
                .try_map(closure.clone()))
                .map(|s| format!("<h2>{}</h2>", s)),
        just("### ")
            .ignore_then(any()
                .and_is(just('\n').not())
                .repeated()
                .collect()
                .try_map(closure.clone()))
                .map(|s| format!("<h3>{}</h3>", s)),
        just("-# ")
            .ignore_then(any()
                .and_is(just('\n').not())
                .repeated()
                .collect()
                .try_map(closure.clone()))
                .map(|s| format!("<br/><small>{}</small>", s)),
        // paragraphs
        just("\n\n")
            .ignore_then(any()
                .and_is(just("\n\n").not())
                .repeated()
                .at_least(1)
                .collect()
                .try_map(closure.clone()))
                .map(|s| format!("<p>{}</p>", s)),
        // line breaks
        just("\n\n").to(format!("<br/>")),
        inline(extensions.clone()),
    )).padded_by(just('\n'));

    block
}

fn inline<'src>(extensions: Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    let extensions1 = extensions.clone();
    let closure = move |inner: String, _span| {
        inline(extensions1.clone()).parse(inner.as_ref()).into_result().map_err(|_e| EmptyErr::default())
    };
    let mut inline = choice((
        // escape char
        just("\\")
            .ignore_then(any()
                .map(|c| format!("{}", c))),
        // manual wrapping
        just('\n').to(format!("")),
        //TODO: move the elements above to a different parser??
        // image
        just('!')
            .ignore_then(
                group((
                    any()
                        .and_is(just(']').not())
                        .repeated()
                        .collect()
                        .try_map(closure.clone())
                        .or_not()
                        .delimited_by(just('['), just(']')),
                    any()
                        .and_is(just(')').not())
                        .repeated()
                        .collect()
                        .try_map(closure.clone())
                        .or_not()
                        .delimited_by(just('('), just(')')),
                )))
            .map(|(text, link)| {
                format!("<img src=\"{}\" alt=\"{}\"/>", link.unwrap_or_else(String::new), text.unwrap_or_else(String::new))
            }),
        // link
        group((
            any()
                .and_is(just(']').not())
                .repeated()
                .collect()
                .try_map(closure.clone())
                .or_not()
                .delimited_by(just('['), just(']')),
            any()
                .and_is(just(')').not())
                .repeated()
                .collect()
                .try_map(closure.clone())
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
        just("**")
            .ignore_then(any()
                .and_is(just("**").not())
                .repeated()
                .at_least(1)
                .collect::<String>()
                .then(any()
                    .and_is(just('*'))
                    .repeated()
                    .at_least(2)
                    .collect::<String>()
                    // this unwrap's default probably isn't necessary
                    .map(|s| s.strip_suffix("**").map(|s| s.to_owned()).unwrap_or(s))))
            .map(|(left, right)| format!("{}{}", left, right))
            .try_map(closure.clone())
            .map(|inner| format!("<b>{}</b>", inner)),
        // italic
        just('*')
            .ignore_then(any()
                .and_is(just('*').not())
                .repeated()
                .at_least(1)
                .collect::<String>()
                .then(any()
                    // this maybe shouldn't match if you have a case such as * some text **
                    .and_is(just('*'))
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    // this unwrap's default probably isn't necessary
                    .map(|s| s.strip_suffix("*").map(|s| s.to_owned()).unwrap_or(s))))
            .map(|(left, right)| format!("{}{}", left, right))
            .try_map(closure.clone())
            .map(|inner| format!("<i>{}</i>", inner)),
        // strikethrough
        any()
            .and_is(just("~~").not())
            .repeated()
            .at_least(1)
            .to_slice()
            .padded_by(just("~~"))
            .map(|inner| format!("<s>{}</s>", inner)),
        // underline
        any()
            .and_is(just("__").not())
            .repeated()
            .at_least(1)
            .to_slice()
            .padded_by(just("__"))
            .map(|inner| format!("<u>{}</u>", inner)),
    ));

    inline.clone().or(any().and_is(inline.not()).repeated().at_least(1).collect())
}

#[cfg(test)]
mod tests {
    mod block {
        use chumsky::Parser;

        use crate::parser::markdown::block;

        #[test]
        fn header1() {
            let p = block(vec![]);
            let res = p.parse("\n# meow\n").into_result().unwrap();
            let expected = format!("<h1>meow</h1>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header2() {
            let p = block(vec![]);
            let res = p.parse("\n## meow\n").into_result().unwrap();
            let expected = format!("<h2>meow</h2>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header3() {
            let p = block(vec![]);
            let res = p.parse("\n### meow\n").into_result().unwrap();
            let expected = format!("<h3>meow</h3>");

            assert_eq!(expected, res);
        }

        #[test]
        fn small() {
            let p = block(vec![]);
            let res = p.parse("\n-# meow\n").into_result().unwrap();
            let expected = format!("<br/><small>meow</small>");

            assert_eq!(expected, res);
        }
    }

    mod inline {
        use chumsky::Parser;

        use crate::parser::markdown::inline;

        #[test]
        fn code_block() {
            let p = inline(vec![]);
            let res = p.parse("```meow```").into_result().unwrap();
            let expected = format!("<pre><code>meow</code></pre>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_line() {
            let p = inline(vec![]);
            let res = p.parse("`meow`").into_result().unwrap();
            let expected = format!("<code>meow</code>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold() {
            let p = inline(vec![]);
            let res = p.parse("**meow**").into_result().unwrap();
            let expected = format!("<b>meow</b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn italic() {
            let p = inline(vec![]);
            let res = p.parse("*meow*").into_result().unwrap();
            let expected = format!("<i>meow</i>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold_and_italic() {
            let p = inline(vec![]);
            let res = p.parse("***meow***").into_result().unwrap();
            let expected = format!("<b><i>meow</i></b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn strikethrough() {
            let p = inline(vec![]);
            let res = p.parse("~~meow~~").into_result().unwrap();
            let expected = format!("<s>meow</s>");

            assert_eq!(expected, res);
        }

        #[test]
        fn underline() {
            let p = inline(vec![]);
            let res = p.parse("__meow__").into_result().unwrap();
            let expected = format!("<u>meow</u>");

            assert_eq!(expected, res);
        }
    }
}
