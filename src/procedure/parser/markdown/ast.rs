pub type MarkdownElementCollection = Vec<Box<dyn MarkdownElement>>;

impl From<Box<dyn MarkdownElement>> for MarkdownElementCollection {
    fn from(value: Box<dyn MarkdownElement>) -> Self {
        vec![value]
    }
}

pub struct MarkdownDocument(MarkdownElementCollection);

impl MarkdownDocument {
    pub fn as_html(&self) -> String {
        let mut result = String::new();

        for child in self.0 {
            result.push_str(child.as_html().as_str());
        }

        result
    }
}

impl From<MarkdownElementCollection> for MarkdownDocument {
    fn from(value: MarkdownElementCollection) -> Self {
        Self(value)
    }
}

pub trait MarkdownElement {
    fn as_html(&self) -> String;
}

pub struct Plain(pub String);

impl MarkdownElement for Plain {
    fn as_html(&self) -> String {
        self.0
    }
}
