use std::{cell::RefCell, rc::Rc};

use rand::Rng;

#[derive(Clone)]
pub enum MarkdownExtension {
    Inline(String, String, fn(String) -> String),
}

// pub fn wobbly() -> MarkdownExtension {
//     MarkdownExtension::inline(Box::new(|input: &str| {
//         delimited(tag("~{"), take_until1("}~"), tag("}~"))
//             .map(|s| {
//                 let s = String::from(s);
//                 let mut inners = Vec::new();
//
//                 for c in s.chars() {
//                     inners.push(recursive_wobble(c));
//                 }
//
//                 vec![Inline::Html(format!("<span aria-label=\"{}\"><style scoped>@keyframes spin {{ 100% {{ transform: rotate(360deg); }} }}</style>{}</span>", s, inners.concat()))]
//             })
//             .parse(input)
//     }))
// }

fn recursive_wobble(c: char) -> String {
    let mut rng = rand::rng();
    let mut s = if c.is_whitespace() { format!("&nbsp;") } else { format!("{}", c) };

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
