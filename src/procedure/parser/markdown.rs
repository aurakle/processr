use anyhow::Result;

use super::Parser;

struct MarkdownParser {
    extensions: Vec<Box<dyn MarkdownExtension>>,
}

impl Parser for MarkdownParser {
    fn process(&self, bytes: &Vec<u8>) -> Result<Vec<u8>> {
        todo!()
    }
}

trait MarkdownExtension {

}
