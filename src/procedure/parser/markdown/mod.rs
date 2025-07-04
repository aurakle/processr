mod extension;
mod parser;

pub use parser::MarkdownParser;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::procedure::parser::Parser;

    use super::*;
}
