use nom::{error::{Error, ParseError}, IResult, OutputMode, PResult, Parser};

use super::{ast::MarkdownElement, MarkdownParser};

mod default;

pub use default::all as default;

pub struct MarkdownExtension {
    parser: Box<dyn Fn(&str) -> IResult<&str, Box<dyn MarkdownElement>>>,
}

impl MarkdownExtension {
    pub fn new(func: dyn Fn(&str) -> IResult<&str, Box<dyn MarkdownElement>>) -> Self {
        Self {
            parser: func,
        }
    }
}

impl<'a> Parser<&'a str> for MarkdownExtension {
    type Output = Box<dyn MarkdownElement>;
    type Error = Error<&'a str>;

    fn process<OM: OutputMode>(
        &mut self,
        input: &'a str,
      ) -> PResult<OM, &'a str, Self::Output, Self::Error> {
        self.parser.process(input)
    }
}

impl<F> From<F> for MarkdownExtension
where
    F: Fn(&str) -> IResult<&str, Box<dyn MarkdownElement>>,
{
    fn from(value: F) -> Self {
        Self::new(Box::new(value))
    }
}
