use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, newline}};
use extension::MarkdownExtension;

use crate::Meta;

use super::{whitespace, ParserProcedure};

pub mod extension;

#[derive(Debug, Clone)]
pub struct MarkdownParser {
    extensions: Vec<MarkdownExtension>,
}

impl MarkdownParser {
    pub fn default() -> Self {
        Self {
            extensions: extension::default(),
        }
    }

    pub fn extend(&self, extension: MarkdownExtension) -> Self {
        let mut extensions = self.extensions.clone();

        extensions.push(extension);

        Self {
            extensions
        }
    }

    fn make_parser<'src>(&self, old_properties: HashMap<String, Meta>) -> impl Parser<'src, &'src str, (String, HashMap<String, Meta>)> {
        properties().or_not().then(document(&self.extensions)).map(move |(new_properties, document)| {
            let mut final_properties = old_properties.clone();
            final_properties.extend(new_properties.unwrap_or_else(HashMap::new));

            (document, final_properties)
        })
    }
}

impl ParserProcedure for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)> {
        let text = String::from_utf8(bytes.clone())?;
        let parser = self.make_parser(properties.clone());
        let (text, properties) = parser.parse(text.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;

        Ok((text.as_bytes().to_vec(), properties))
    }
}

fn properties<'src>() -> impl Parser<'src, &'src str, HashMap<String, Meta>> {
    yaml().padded_by(just("---").padded())
}

fn yaml<'src>() -> impl Parser<'src, &'src str, HashMap<String, Meta>> {
    let text = none_of(",:\n")
        .repeated()
        .collect::<String>()
        .map(Meta::from);
    let list = text
        .separated_by(just(',').padded_by(whitespace()))
        .collect::<Vec<_>>()
        .map(Meta::from);
    let entry = ident()
        .map(str::to_owned)
        .then_ignore(just(':').padded_by(whitespace()))
        .then(list)
        .then_ignore(newline().padded_by(whitespace()));

    entry
        .padded_by(whitespace())
        .repeated()
        .collect()
        .map(|entries: Vec<(String, Meta)>| {
            let mut result = HashMap::new();

            for entry in entries {
                result.insert(entry.0, entry.1);
            }

            result
        })
}

fn document<'src>(extensions: &Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> {
    element(extensions)
        .repeated()
        .collect::<Vec<String>>()
        .map(|elements| elements.concat())
}

fn element<'src>(extensions: &Vec<MarkdownExtension>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive::<'src, 'src>(|this| {
        let closure = move |inner, _span| this.parse(inner).into_result().map_err(|_e| EmptyErr::default());
        let mut element = choice((
            just("\\")
                .ignore_then(any()
                    .map(|c| format!("{}", c))),
            just("\n\n\n")
                .ignore_then(any()
                    .and_is(just("\n\n\n").not())
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .try_map(closure.clone()))
                    .map(|s| format!("<p>{}</p>", s)),
            just("\n\n").to(format!("<br/>")),
            just('\n').to(format!("")),
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
            any()
                .and_is(just('`').not())
                .repeated()
                .to_slice()
                .padded_by(just('`'))
                .map(|inner| format!("<code>{}</code>", html_escape::encode_safe(inner))),
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
            //TODO: add image and link support
        )).boxed();

        for extension in extensions {
            element = element
                .or(any()
                    .and_is(just(extension.end.clone()).not())
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .try_map(closure.clone())
                    .map(extension.wrapper.clone())
                    .delimited_by(just(extension.start.clone()), just(extension.end.clone())))
                .boxed();
        }

        element.clone().or(any().and_is(element.not()).repeated().at_least(1).collect())
    })
}

#[cfg(test)]
mod tests {
    mod properties {
        use std::collections::HashMap;

        use chumsky::Parser;

        use crate::{parser::markdown::properties, Meta};

        #[test]
        fn text() {
            let res = properties().parse("---\nitem1: prrr\n---").into_result().unwrap();
            let mut expected = HashMap::new();

            expected.insert(format!("item1"), Meta::from(vec!(Meta::from(format!("prrr")))));

            assert_eq!(expected, res);
        }

        #[test]
        fn list() {
            let res = properties().parse("---\nitem2: meow, mrrow\n---").into_result().unwrap();
            let mut expected = HashMap::new();

            expected.insert(format!("item2"), Meta::from(vec![Meta::from(format!("meow")), Meta::from(format!("mrrow"))]));

            assert_eq!(expected, res);
        }

        #[test]
        fn mixed() {
            let res = properties().parse("---\nitem1: prrr\nitem2: meow, mrrow\n---").into_result().unwrap();
            let mut expected = HashMap::new();

            expected.insert(format!("item1"), Meta::from(vec!(Meta::from(format!("prrr")))));
            expected.insert(format!("item2"), Meta::from(vec![Meta::from(format!("meow")), Meta::from(format!("mrrow"))]));

            assert_eq!(expected, res);
        }
    }

    mod element {
        use chumsky::Parser;

        use crate::parser::markdown::{element, extension};

        #[test]
        fn plain() {
            let res = element(&extension::default()).parse("meow meow mrrp").into_result().unwrap();
            let expected = format!("meow meow mrrp");

            assert_eq!(expected, res);
        }

        #[test]
        fn single_newline() {
            let res = element(&extension::default()).parse("\n").into_result().unwrap();
            let expected = format!("");

            assert_eq!(expected, res);
        }

        #[test]
        fn double_newline() {
            let res = element(&extension::default()).parse("\n\n").into_result().unwrap();
            let expected = format!("<br/>");

            assert_eq!(expected, res);
        }

        #[test]
        fn triple_newline() {
            let res = element(&extension::default()).parse("\n\n\nmeow").into_result().unwrap();
            let expected = format!("<p>meow</p>");

            assert_eq!(expected, res);
        }

        #[test]
        fn italic() {
            let res = element(&extension::default()).parse("*meow*").into_result().unwrap();
            let expected = format!("<i>meow</i>");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold() {
            let res = element(&extension::default()).parse("**meow**").into_result().unwrap();
            let expected = format!("<b>meow</b>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code() {
            let res = element(&extension::default()).parse("`meow`").into_result().unwrap();
            let expected = format!("<code>meow</code>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_block() {
            let res = element(&extension::default()).parse("```meow```").into_result().unwrap();
            let expected = format!("<pre><code>meow</code></pre>");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_block_with_lang() {
            let res = element(&extension::default()).parse("```rs\nmeow```").into_result().unwrap();
            let expected = format!("<pre><code class=\"language-rs\">meow</code></pre>");

            assert_eq!(expected, res);
        }

        #[test]
        fn strikethrough() {
            let res = element(&extension::default()).parse("~~meow~~").into_result().unwrap();
            let expected = format!("<s>meow</s>");

            assert_eq!(expected, res);
        }

        #[test]
        fn underline() {
            let res = element(&extension::default()).parse("__meow__").into_result().unwrap();
            let expected = format!("<u>meow</u>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header1() {
            let res = element(&extension::default()).parse("# meow\n").into_result().unwrap();
            let expected = format!("<h1>meow</h1>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header2() {
            let res = element(&extension::default()).parse("## meow\n").into_result().unwrap();
            let expected = format!("<h2>meow</h2>");

            assert_eq!(expected, res);
        }

        #[test]
        fn header3() {
            let res = element(&extension::default()).parse("### meow\n").into_result().unwrap();
            let expected = format!("<h3>meow</h3>");

            assert_eq!(expected, res);
        }

        #[test]
        fn small() {
            let res = element(&extension::default()).parse("-# meow\n").into_result().unwrap();
            let expected = format!("<small>meow</small>");

            assert_eq!(expected, res);
        }

        #[test]
        fn link_with_text() {
            let res = element(&extension::default()).parse("[mraow](prrr)").into_result().unwrap();
            let expected = format!("<a href=\"prrr\">mraow</a>");

            assert_eq!(expected, res);
        }

        #[test]
        fn empty_link() {
            let res = element(&extension::default()).parse("[]()").into_result().unwrap();
            let expected = format!("<a href=\"\"></a>");

            assert_eq!(expected, res);
        }

        #[test]
        fn link_without_text() {
            let res = element(&extension::default()).parse("[](prrr)").into_result().unwrap();
            let expected = format!("<a href=\"prrr\"></a>");

            assert_eq!(expected, res);
        }

        #[test]
        fn text_without_link() {
            let res = element(&extension::default()).parse("[mraow]()").into_result().unwrap();
            let expected = format!("<a href=\"\">mraow</a>");

            assert_eq!(expected, res);
        }
    }

    mod document {
        use std::fs;

        use chumsky::Parser;

        use crate::parser::markdown::{document, extension};

        #[test]
        fn plain() {
            let res = document(&extension::default()).parse("meow meow mrrp").into_result().unwrap();
            let expected = format!("meow meow mrrp");

            assert_eq!(expected, res);
        }

        #[test]
        fn single_newline() {
            let res = document(&extension::default()).parse("meow\nmeow").into_result().unwrap();
            let expected = format!("meowmeow");

            assert_eq!(expected, res);
        }

        #[test]
        fn double_newline() {
            let res = document(&extension::default()).parse("meow\n\nmeow").into_result().unwrap();
            let expected = format!("meow<br/>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn triple_newline() {
            let res = document(&extension::default()).parse("meow\n\n\nmeow").into_result().unwrap();
            let expected = format!("meow<p>meow</p>");

            assert_eq!(expected, res);
        }

        #[test]
        fn italic() {
            let res = document(&extension::default()).parse("meow*meow*meow").into_result().unwrap();
            let expected = format!("meow<i>meow</i>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn bold() {
            let res = document(&extension::default()).parse("meow**meow**meow").into_result().unwrap();
            let expected = format!("meow<b>meow</b>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn code() {
            let res = document(&extension::default()).parse("meow`meow`meow").into_result().unwrap();
            let expected = format!("meow<code>meow</code>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_block() {
            let res = document(&extension::default()).parse("meow```meow```meow").into_result().unwrap();
            let expected = format!("meow<pre><code>meow</code></pre>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn code_block_with_lang() {
            let res = document(&extension::default()).parse("meow```rs\nmeow```meow").into_result().unwrap();
            let expected = format!("meow<pre><code class=\"language-rs\">meow</code></pre>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn strikethrough() {
            let res = document(&extension::default()).parse("meow~~meow~~meow").into_result().unwrap();
            let expected = format!("meow<s>meow</s>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn underline() {
            let res = document(&extension::default()).parse("meow__meow__meow").into_result().unwrap();
            let expected = format!("meow<u>meow</u>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn header1() {
            let res = document(&extension::default()).parse("meow\n# meow\nmeow").into_result().unwrap();
            let expected = format!("meow<h1>meow</h1>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn header2() {
            let res = document(&extension::default()).parse("meow\n## meow\nmeow").into_result().unwrap();
            let expected = format!("meow<h2>meow</h2>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn header3() {
            let res = document(&extension::default()).parse("meow\n### meow\nmeow").into_result().unwrap();
            let expected = format!("meow<h3>meow</h3>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn small() {
            let res = document(&extension::default()).parse("meow\n-# meow\nmeow").into_result().unwrap();
            let expected = format!("meow<small>meow</small>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn link_with_text() {
            let res = document(&extension::default()).parse("meow[mraow](prrr)meow").into_result().unwrap();
            let expected = format!("meow<a href=\"prrr\">mraow</a>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn empty_link() {
            let res = document(&extension::default()).parse("meow[]()meow").into_result().unwrap();
            let expected = format!("meow<a href=\"\"></a>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn link_without_text() {
            let res = document(&extension::default()).parse("meow[](prrr)meow").into_result().unwrap();
            let expected = format!("meow<a href=\"prrr\"></a>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn text_without_link() {
            let res = document(&extension::default()).parse("meow[mraow]()meow").into_result().unwrap();
            let expected = format!("meow<a href=\"\">mraow</a>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn whole_document() {
            let text = fs::read_to_string("test/test.md").unwrap();
            let res = document(&extension::default()).parse(&text).into_result().unwrap();
            let expected = fs::read_to_string("test/test.html").unwrap();

            fs::write("test/test.out.txt", res.clone());
            assert_eq!(expected, res);
        }
    }
}
