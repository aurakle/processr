mod default;
pub mod extra;

pub use default::all as default;

#[derive(Debug, Clone)]
pub struct MarkdownExtension {
    pub start: String,
    pub end: String,
    pub wrapper: fn(String) -> String,
}
