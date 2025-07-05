use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, newline}};
use extension::MarkdownExtension;

use crate::Meta;

use super::Parser as ParserProcedure;

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
        .collect();
    let list = text
        .separated_by(just(',').padded_by(whitespace()))
        .collect()
        .map(|strings| Meta(strings));
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
            just("\n\n\n")
                .ignore_then(any()
                    .and_is(just("\n\n\n").not())
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .try_map(closure.clone()))
                    .map(|s| format!("<p>{}</p>", s)),
            just("\n\n").to(format!("<br/>")),
            just("\n-#")
                .padded_by(whitespace())
                .ignore_then(any()
                    .and_is(just("\n").not())
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .try_map(closure.clone()))
                    .map(|s| format!("<p><small>{}</small></p>", s)),
            just("\n").to(format!("")),
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

fn whitespace<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
    any()
        .and_is(newline().not())
        .filter(|c: &char| c.is_whitespace())
        .ignored()
        .repeated()
        .collect::<Vec<()>>()
        .ignored()
}

#[cfg(test)]
mod tests {
    mod properties {
        use std::collections::HashMap;

        use chumsky::Parser;

        use crate::{procedure::parser::markdown::properties, Meta};

        #[test]
        fn text() {
            let res = properties().parse("---\nitem1: prrr\n---").into_result().unwrap();
            let mut expected = HashMap::new();

            expected.insert(format!("item1"), Meta::from(format!("prrr")));

            assert_eq!(expected, res);
        }

        #[test]
        fn list() {
            let res = properties().parse("---\nitem2: meow, mrrow\n---").into_result().unwrap();
            let mut expected = HashMap::new();

            expected.insert(format!("item2"), Meta(vec![format!("meow"), format!("mrrow")]));

            assert_eq!(expected, res);
        }

        #[test]
        fn mixed() {
            let res = properties().parse("---\nitem1: prrr\nitem2: meow, mrrow\n---").into_result().unwrap();
            let mut expected = HashMap::new();

            expected.insert(format!("item1"), Meta::from(format!("prrr")));
            expected.insert(format!("item2"), Meta(vec![format!("meow"), format!("mrrow")]));

            assert_eq!(expected, res);
        }
    }

    mod element {
        use chumsky::Parser;

        use crate::procedure::parser::markdown::{element, extension};

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
        fn strikethrough() {
            let res = element(&extension::default()).parse("~~meow~~").into_result().unwrap();
            let expected = format!("<s>meow</s>");

            assert_eq!(expected, res);
        }

        #[test]
        fn small() {
            let res = element(&extension::default()).parse("\n-# meow").into_result().unwrap();
            let expected = format!("<p><small>meow</small></p>");

            assert_eq!(expected, res);
        }
    }

    mod document {
        use chumsky::Parser;

        use crate::procedure::parser::markdown::{document, extension};

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
        fn strikethrough() {
            let res = document(&extension::default()).parse("meow~~meow~~meow").into_result().unwrap();
            let expected = format!("meow<s>meow</s>meow");

            assert_eq!(expected, res);
        }

        #[test]
        fn small() {
            let res = document(&extension::default()).parse("meow\n-# meow\nmeow").into_result().unwrap();
            let expected = format!("meow<p><small>meow</small></p>meow");

            assert_eq!(expected, res);
        }
    }
}
