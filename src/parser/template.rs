use std::{collections::HashMap, env, fs};

use anyhow::{anyhow, Result};
use chumsky::{prelude::*, text::{ident, keyword, newline}};

use crate::Meta;

use super::ParserProcedure;

#[derive(Debug, Clone)]
pub struct TemplateParser();

impl TemplateParser {
    pub fn default() -> TemplateParser {
        Self()
    }
}

impl ParserProcedure for TemplateParser {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)> {
        let text = String::from_utf8(bytes.clone())?;
        let parser = make_parser(properties.clone());
        let text = parser.parse(text.as_str()).into_result().map_err(|_e| anyhow!("Failed to parse markdown"))?;

        Ok((text.as_bytes().to_vec(), properties.clone()))
    }
}

fn make_parser<'src>(properties: HashMap<String, Meta>) -> impl Parser<'src, &'src str, String> + Clone {
    recursive(move |this| {
        let this1 = this.clone();
        let this2 = this.clone();
        let this3 = this.clone();
        let this4 = this.clone();

        let props1 = properties.clone();
        let props2 = properties.clone();
        let props3 = properties.clone();
        let props4 = properties.clone();

        let include = keyword("include")
            .ignore_then(any()
                .and_is(just("\")").not())
                .repeated()
                .collect::<String>()
                .try_map(|path, _span| {
                    fs::read_to_string(env::current_dir()
                        .map_err(|_e| EmptyErr::default())?
                        .join(path))
                        .map_err(|_e| EmptyErr::default())
                })
                .try_map(move |inner, _span| {
                    make_parser(props1.clone()).parse(inner.as_ref())
                        .into_result()
                        .map_err(|_e| EmptyErr::default())
                })
                .delimited_by(just("(\""), just("\")")))
            .padded_by(just('$'));

        let foreach = keyword("for")
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
                let list = props2.get(key).map(Meta::as_list).unwrap_or_else(Vec::new);

                for item in list {
                    result.push(make_parser(item.as_map())
                        .parse(inner.as_ref())
                        .into_result()
                        .map_err(|_e| EmptyErr::default())?);
                }

                Ok(result.concat())
            });

        let if_else = keyword("if")
            .ignore_then(ident().delimited_by(just('('), just(')')))
            .padded_by(just('$'))
            .then(this3)
            .then(just("$else$")
                .ignore_then(this4)
                .or_not())
            .then_ignore(just("$endif$"))
            .map(move |((key, then), otherwise)| {
                let s = props3.get(key).and_then(Meta::as_string).unwrap_or_else(String::new);

                if s.len() != 0 {
                    then
                } else {
                    otherwise.unwrap_or_else(String::new)
                }
            });

        let access = ident()
            .padded_by(just('$'))
            .map(move |key| props4.get(key).and_then(Meta::as_string).unwrap_or_else(String::new));

        let element = choice((
            include,
            foreach,
            if_else,
            access,
        ));

        element.clone()
            .or(any().and_is(element.not()).repeated().at_least(1).collect::<String>())
            .and_is(just("$else$").not())
            .and_is(just("$endif$").not())
            .repeated()
            .collect::<Vec<String>>()
            .map(|elements| elements.concat())
    })
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use chumsky::Parser;

    use crate::Meta;

    use super::make_parser;

    #[test]
    fn plain() {
        let props = HashMap::new();
        let parser = make_parser(props);
        let res = parser.parse("meow meow").into_result().unwrap();
        let expected = format!("meow meow");

        assert_eq!(expected, res);
    }

    #[test]
    fn text_access() {
        let mut props = HashMap::new();
        props.insert(format!("mrrp"), Meta::from("prrr"));
        let parser = make_parser(props);
        let res = parser.parse("$mrrp$").into_result().unwrap();
        let expected = format!("prrr");

        assert_eq!(expected, res);
    }

    #[test]
    fn list_access() {
        let mut props = HashMap::new();
        props.insert(format!("mrrp"), Meta::from(vec![Meta::from("prrr")]));
        let parser = make_parser(props);
        let res = parser.parse("$mrrp$").into_result().unwrap();
        let expected = format!("prrr");

        assert_eq!(expected, res);
    }

    #[test]
    fn map_access() {
        let mut props = HashMap::new();
        let mut map = HashMap::new();
        map.insert(format!("bwa"), Meta::from("pain"));
        props.insert(format!("mrrp"), Meta::from(map));
        let parser = make_parser(props);
        let res = parser.parse("$mrrp$").into_result().unwrap();
        let expected = format!("");

        assert_eq!(expected, res);
    }

    #[test]
    fn include() {
        let mut props = HashMap::new();
        let parser = make_parser(props);
        let res = parser.parse("$include(\"test/templates/partial.txt\")$").into_result().unwrap();
        let expected = fs::read_to_string("test/templates/partial.txt").unwrap();

        assert_eq!(expected, res);
    }

    #[test]
    fn for_each() {
        let mut props = HashMap::new();
        let mut m1 = HashMap::new();
        m1.insert(format!("url"), Meta::from("test1"));
        m1.insert(format!("body"), Meta::from("meow"));
        m1.insert(format!("field1"), Meta::from("pr"));
        let mut m2 = HashMap::new();
        m1.insert(format!("url"), Meta::from("test2"));
        m1.insert(format!("body"), Meta::from("meow meow"));
        m1.insert(format!("field1"), Meta::from("prr"));
        let mut m3 = HashMap::new();
        m1.insert(format!("url"), Meta::from("test3"));
        m1.insert(format!("body"), Meta::from("meow meow meow"));
        m1.insert(format!("field1"), Meta::from("prrr"));
        let l = vec![Meta::from(m1), Meta::from(m2), Meta::from(m3)];
        props.insert(format!("items"), Meta::from(l));
        let parser = make_parser(props);
        let res = parser.parse("$for(items)$$url$$body$$field1$$endfor$").into_result().unwrap();
        let expected = format!("test1meowprtest2meow meowprrtest3meow meow meowprrr");

        assert_eq!(expected, res);
    }

    #[test]
    fn if_else_true() {
        let mut props = HashMap::new();
        props.insert(format!("b"), Meta::from("yay"));
        let parser = make_parser(props);
        let res = parser.parse("$if(b)$meow$else$prrr$endif$").into_result().unwrap();
        let expected = format!("meow");

        assert_eq!(expected, res);
    }

    #[test]
    fn if_else_false() {
        let mut props = HashMap::new();
        let parser = make_parser(props);
        let res = parser.parse("$if(b)$meow$else$prrr$endif$").into_result().unwrap();
        let expected = format!("prrr");

        assert_eq!(expected, res);
    }

    #[test]
    fn if_true() {
        let mut props = HashMap::new();
        props.insert(format!("b"), Meta::from("yay"));
        let parser = make_parser(props);
        let res = parser.parse("$if(b)$meow$endif$").into_result().unwrap();
        let expected = format!("meow");

        assert_eq!(expected, res);
    }

    #[test]
    fn if_false() {
        let mut props = HashMap::new();
        let parser = make_parser(props);
        let res = parser.parse("$if(b)$meow$endif$").into_result().unwrap();
        let expected = format!("");

        assert_eq!(expected, res);
    }
}
