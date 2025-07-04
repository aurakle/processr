use rand::Rng;

use crate::parser::markdown::MarkdownParser;

use super::MarkdownExtension;

pub fn all() -> Vec<MarkdownExtension> {
    vec![
        header1(),
        header2(),
        header3(),
        small(),
        italic(),
        bold(),
        code(),
        strikethrough(),
        underline(),
    ]
}

pub fn header1() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("# "),
        end: format!("\n"),
        wrapper: |s| format!("<h1>{}</h1>", s),
    }
}

pub fn header2() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("## "),
        end: format!("\n"),
        wrapper: |s| format!("<h2>{}</h2>", s),
    }
}

pub fn header3() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("### "),
        end: format!("\n"),
        wrapper: |s| format!("<h3>{}</h3>", s),
    }
}

pub fn small() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("-# "),
        end: format!("\n"),
        wrapper: |s| format!("<small>{}</small>", s),
    }
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

pub fn underline() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("__"),
        end: format!("__"),
        wrapper: |s| format!("<u>{}</u>", s),
    }
}
