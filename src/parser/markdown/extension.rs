use std::{cell::RefCell, rc::Rc};

use markdown_ppp::ast::{Block, Inline};
use nom::{bytes::complete::{tag, take_until1}, sequence::delimited, IResult, Parser};
use rand::Rng;

pub type InlineExtensionFn = Rc<RefCell<Box<dyn for<'a> FnMut(&'a str) -> IResult<&'a str, Vec<Inline>>>>>;
pub type BlockExtensionFn = Rc<RefCell<Box<dyn for<'a> FnMut(&'a str) -> IResult<&'a str, Vec<Block>>>>>;

#[derive(Clone)]
pub enum MarkdownExtension {
    Inline(InlineExtensionFn),
    Block(BlockExtensionFn),
}

impl MarkdownExtension {
    pub fn inline(func: Box<dyn for<'a> FnMut(&'a str) -> IResult<&'a str, Vec<Inline>>>) -> Self {
        MarkdownExtension::Inline(Rc::new(RefCell::new(func)))
    }

    pub fn block(func: Box<dyn for<'a> FnMut(&'a str) -> IResult<&'a str, Vec<Block>>>) -> Self {
        MarkdownExtension::Block(Rc::new(RefCell::new(func)))
    }
}

pub fn wobbly() -> MarkdownExtension {
    MarkdownExtension::inline(Box::new(|input: &str| {
        delimited(tag("~{"), take_until1("}~"), tag("}~"))
            .map(|s| {
                let s = String::from(s);
                let mut inners = Vec::new();

                for c in s.chars() {
                    inners.push(recursive_wobble(c));
                }

                vec![Inline::Html(format!("<span aria-label=\"{}\"><style scoped>@keyframes spin {{ 100% {{ transform: rotate(360deg); }} }}</style>{}</span>", s, inners.concat()))]
            })
            .parse(input)
    }))
}

fn recursive_wobble(c: char) -> String {
    let mut rng = rand::rng();
    let mut s = format!("{}", c);

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

    s
}
