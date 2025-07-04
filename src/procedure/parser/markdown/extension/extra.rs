use rand::Rng;

use super::MarkdownExtension;

pub fn wobbly() -> MarkdownExtension {
    MarkdownExtension {
        start: format!("~>"),
        end: format!("<~"),
        wrapper: |s| {
            let mut inners = Vec::new();

            for c in s.chars() {
                inners.push(recursive_wobble(c));
            }

            format!("<span aria-label=\"{}\" style=\"@keyframes spin {{ 100% {{ transform: rotate(360deg); }} }}\">{}</span>", s, inners.join(""))
        },
    }
}

fn recursive_wobble(c: char) -> String {
    let mut rng = rand::rng();
    let mut s = format!("{}", c);

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
