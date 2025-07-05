use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, keyword, newline}};
use std::{collections::HashMap, env, fs, path::PathBuf};

use crate::Meta;

use super::ParserProcedure;

#[derive(Debug, Clone)]
pub struct TemplateParser();

impl TemplateParser {
    pub fn default() -> TemplateParser {
        Self()
    }

    fn make_parser<'src>(&self, properties: HashMap<String, Meta>) -> impl Parser<'src, &'src str, String> {
        element(properties)
            .repeated()
            .collect::<Vec<_>>()
            .map(|elements| elements.concat())
    }
}

impl ParserProcedure for TemplateParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)> {
        let text = String::from_utf8(bytes.clone())?;
        let parser = self.make_parser(properties.clone());
        let text = parser.parse(text.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;

        Ok((text.as_bytes().to_vec(), properties.clone()))
    }
}

fn element<'src>(properties: HashMap<String, Meta>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(move |this| {
        let this1 = this.clone();
        let this2 = this.clone();
        let this3 = this.clone();
        let this4 = this.clone();

        let props1 = properties.clone();
        let props2 = properties.clone();
        let props3 = properties.clone();

        let element = choice((
            keyword("partial")
                .ignore_then(none_of("\"")
                    .repeated()
                    .collect::<String>()
                    .delimited_by(just("(\""), just("\")")))
                .padded_by(just('$'))
                .try_map(|path, _span| {
                    fs::read_to_string(env::current_dir()
                        .map_err(|_e| EmptyErr::default())?
                        .join(path))
                        .map_err(|_e| EmptyErr::default())
                })
                .to_slice()
                .try_map(move |include, _span| {
                    this1.clone()
                        .repeated()
                        .collect::<Vec<_>>()
                        .map(|elements| elements.concat())
                        .parse(include)
                        .into_result()
                        .map_err(|_e| EmptyErr::default())
                }),
            keyword("for")
                .ignore_then(ident()
                    .delimited_by(just('('), just(')')))
                .padded_by(just('$'))
                .then(any()
                    .and_is(just("$endfor$").not())
                    .repeated()
                    .collect::<String>())
                .then_ignore(just("$endfor$"))
                .try_map(move |(key, inner), _span| {
                    let mut result = Vec::new();
                    let list = props1.get(key).and_then(Meta::as_list).unwrap_or_else(Vec::new);

                    for item in list {
                        let mut map = match item.clone() {
                            Meta::Map(map) => map,
                            _ => HashMap::new(),
                        };

                        map.insert(format!("i"), item);
                        result.push(element(map)
                            .repeated()
                            .collect::<Vec<_>>()
                            .map(|elements| elements.concat())
                            .parse(inner.as_ref())
                            .into_result()
                            .map_err(|_e| EmptyErr::default())?);
                    }

                    Ok(result.concat())
                }),
            keyword("if")
                .ignore_then(ident()
                    .delimited_by(just('('), just(')')))
                .padded_by(just('$'))
                .then(this3
                    .and_is(just("$endif$")
                        .or(just("$elseif$"))
                        .not())
                    .repeated()
                    .collect::<Vec<_>>()
                    .map(|elements| elements.concat()))
                .then(just("$elseif$")
                    .ignore_then(this4
                        .and_is(just("$endif$")
                            .not())
                        .repeated()
                        .collect::<Vec<_>>()
                        .map(|elements| elements.concat()))
                    .or_not())
                .then_ignore(just("$endif$"))
                .map(move |((key, then), otherwise)| {
                    let s = props2.get(key).and_then(Meta::as_string).unwrap_or_else(String::new);

                    if s.len() != 0 {
                        then
                    } else {
                        otherwise.unwrap_or_else(String::new)
                    }
                }),
            ident()
                .padded_by(just('$'))
                .map(move |key| props3.get(key).and_then(Meta::as_string).unwrap_or_else(String::new)),
        ));

        element.clone().or(any().and_is(element.not()).repeated().at_least(1).collect())
    })
}
