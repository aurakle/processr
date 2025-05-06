use nom::{error::Error, IResult, OutputMode, PResult, Parser};

use super::{ast::MarkdownElement, MarkdownParser};

mod default;

pub use default::all as default;

pub type MarkdownExtensionParser = Box<dyn for<'a> Fn(&'a str) -> IResult<&str, Box<dyn MarkdownElement>>>;

pub struct MarkdownExtension<'a> {
    parser: &'a MarkdownParser<'a>,
    extension: Box<dyn for<'b> Fn(&MarkdownParser<'b>) -> MarkdownExtensionParser>,
}

impl MarkdownExtension<'_> {
    pub fn new<'a>(parser: &'a MarkdownParser<'a>, extension: impl Fn(&MarkdownParser) -> MarkdownExtensionParser) -> Self {
        Self {
            parser,
            extension: Box::new(extension),
        }
    }
}

impl<'a> Parser<&'a str> for MarkdownExtension<'_> {
    type Output = Box<dyn MarkdownElement>;
    type Error = Error<&'a str>;

    fn process<OM: OutputMode>(
        &mut self,
        input: &'a str,
      ) -> PResult<OM, &'a str, Self::Output, Self::Error> {
        (self.extension)(self.parser).process::<OM>(input)
    }
}
