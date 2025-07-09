use chumsky::{prelude::*, text::newline};
use rand::Rng;

use crate::parser::fail;

#[derive(Clone)]
pub enum MarkdownExtension {
    Inline(String, String, fn(String) -> String),
    Block(String, String, fn(String) -> String, fn(Vec<String>) -> String),
}

impl MarkdownExtension {
    pub fn inline<L: Into<String>, R: Into<String>>(left_delimiter: L, right_delimiter: R, wrapper: fn(String) -> String) -> MarkdownExtension {
        MarkdownExtension::Inline(left_delimiter.into(), right_delimiter.into(), wrapper)
    }

    pub fn block<L: Into<String>, R: Into<String>>(left_delimiter: L, right_delimiter: R, line_wrapper: fn(String) -> String, block_wrapper: fn(Vec<String>) -> String) -> MarkdownExtension {
        MarkdownExtension::Block(left_delimiter.into(), right_delimiter.into(), line_wrapper, block_wrapper)
    }
}

pub(crate) trait MarkdownExtensionList {
    fn build_inline_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String>;
    fn build_block_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String>;
}

impl MarkdownExtensionList for Vec<MarkdownExtension> {
    fn build_inline_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String> {
        self
            .iter()
            .fold(fail().to(String::new()).boxed(), |previous, current| {
                if let MarkdownExtension::Inline(l, r, wrapper) = current.clone() {
                    previous.clone()
                        .or(inline_parser.clone()
                            .nested_in(any()
                                .and_is(just(r.clone()).not())
                                .repeated()
                                .at_least(1)
                                .to_slice()
                                .delimited_by(just(l.clone()), just(r.clone())))
                            .map(wrapper))
                        .boxed()
                } else {
                    previous
                }
            })
    }

    fn build_block_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String> {
        self
            .iter()
            .fold(fail().to(String::new()).boxed(), |previous, current| {
                if let MarkdownExtension::Block(l, r, line_wrapper, block_wrapper) = current.clone() {
                    previous.clone()
                        .or(inline_parser.clone()
                            .nested_in(any()
                                .and_is(just(r.clone()).not())
                                .repeated()
                                .at_least(1)
                                .to_slice()
                                .delimited_by(just(l.clone()), just(r.clone())))
                            .map(line_wrapper)
                            .separated_by(newline())
                            .at_least(1)
                            .collect::<Vec<String>>()
                            .map(block_wrapper))
                        .boxed()
                } else {
                    previous
                }
            })
    }
}

pub fn wobbly() -> MarkdownExtension {
    MarkdownExtension::inline("~{", "}~", |s| {
        let s = String::from(s);
        let mut inners = Vec::new();
        let mut rng = rand::rng();

        for c in s.chars() {
            let mut s = if c.is_whitespace() { format!("&nbsp;") } else { format!("{}", c) };

            for _ in 0..7 {
                s = format!(
                    "<span style=\"display: inline-block; animation: {}s spin linear infinite {}s alternate; transform: translate({}em, {}em);\">{}</span>",
                    rng.random::<f64>() * 0.4,
                    -rng.random::<f64>() * 0.2,
                    (rng.random::<f64>() * 2.0 - 1.0) * 0.08,
                    (rng.random::<f64>() * 2.0 - 1.0) * 0.1,
                    s
                );
            }

            inners.push(s);
        }

        format!("<span aria-label=\"{}\"><style scoped>@keyframes spin {{ 100% {{ transform: rotate(360deg); }} }}</style>{}</span>", s, inners.concat())
    })
}

pub fn small() -> MarkdownExtension {
    MarkdownExtension::block(
        "-# ",
        "",
        |s| format!("<br/><small>{}</small>", s),
        |lines| lines.concat(),
    )
}
