pub type MarkdownElementCollection = Vec<Box<dyn MarkdownElement>>;

pub trait MarkdownElement {
    fn as_html(&self) -> String;
}

pub struct Plain(pub MarkdownElementCollection);

impl MarkdownElement for Plain {
    fn as_html(&self) -> String {
        todo!()
    }
}
