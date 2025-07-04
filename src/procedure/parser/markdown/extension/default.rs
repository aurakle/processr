use rand::Rng;

use crate::procedure::parser::markdown::MarkdownParser;

use super::MarkdownExtension;

pub fn all() -> Vec<MarkdownExtension> {
    vec![
        italic(),
        bold(),
        code(),
        strikethrough(),
        // spoiler(),
    ]
}

pub fn italic() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("*"),
        end: format!("*"),
        wrapper: |s| format!("<i>{}</i>", s),
    }
}

pub fn bold() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("**"),
        end: format!("**"),
        wrapper: |s| format!("<b>{}</b>", s),
    }
}

pub fn code() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("`"),
        end: format!("`"),
        wrapper: |s| format!("<code>{}</code>", s),
    }
}

pub fn strikethrough() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("~~"),
        end: format!("~~"),
        wrapper: |s| format!("<s>{}</s>", s),
    }
}

pub fn spoiler() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("||"),
        end: format!("||"),
        wrapper: todo!(),
    }
}
