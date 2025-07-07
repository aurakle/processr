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

        let res = make_parser(&self.extensions).parse(data.body).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;
        let mut properties = properties.clone();

        properties.extend(data.headers);

        Ok((res.as_bytes().to_vec(), properties))
    }
}

fn make_parser<'src>(extensions: &Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    element(extensions)
        .repeated()
        .collect::<Vec<String>>()
        .map(|elements| elements.concat())
}

fn element<'src>(extensions: &Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive::<'src, 'src>(|this| {
        let closure = move |inner, _span| this.parse(inner).into_result().map_err(|_e| EmptyErr::default());
        let mut element = choice((
            // escape char
            just("\\")
                .ignore_then(any()
                    .map(|c| format!("{}", c))),
            // just("\n\n\n")
            //     .ignore_then(any()
            //         .and_is(just("\n\n\n").not())
            //         .repeated()
            //         .at_least(1)
            //         .to_slice()
            //         .try_map(closure.clone()))
            //         .map(|s| format!("<p>{}</p>", s)),
            // just("\n\n").to(format!("<br/>")),
            // just('\n').to(format!("")),
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
                .to_slice()
                .padded_by(just('`'))
                .map(|inner| format!("<code>{}</code>", html_escape::encode_safe(inner))),
            // image
            just('!')
                .ignore_then(
                    group((
                        any()
                            .and_is(just(']').not())
                            .repeated()
                            .to_slice()
                            .try_map(closure.clone())
                            .or_not()
                            .delimited_by(just('['), just(']')),
                        any()
                            .and_is(just(')').not())
                            .repeated()
                            .to_slice()
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
                    .to_slice()
                    .try_map(closure.clone())
                    .or_not()
                    .delimited_by(just('['), just(']')),
                any()
                    .and_is(just(')').not())
                    .repeated()
                    .to_slice()
                    .try_map(closure.clone())
                    .or_not()
                    .delimited_by(just('('), just(')')),
            ))
                .map(|(text, link)| {
                    format!("<a href=\"{}\">{}</a>", link.unwrap_or_else(String::new), text.unwrap_or_else(String::new))
                }),
            // bold
            any()
                .and_is(just("**").not())
                .repeated()
                .to_slice()
                .padded_by(just("**"))
                .map(|inner| format!("<b>{}</b>", inner)),
            // italic
            any()
                .and_is(just('*').not())
                .repeated()
                .to_slice()
                .padded_by(just('*'))
                .map(|inner| format!("<i>{}</i>", inner)),
        ));

        for extension in extensions {
            // a = a
            //     .or(any()
            //         .and_is(just(extension.end.clone()).not())
            //         .repeated()
            //         .at_least(1)
            //         .to_slice()
            //         .try_map(closure.clone())
            //         .map(extension.wrapper.clone())
            //         .delimited_by(just(extension.start.clone()), just(extension.end.clone())))
            //     .boxed();
        }

        element.clone().or(any().and_is(element.not()).repeated().at_least(1).collect())
    })
}
