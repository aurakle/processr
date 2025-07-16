use std::hash::{DefaultHasher, Hash, Hasher};

use chumsky::{prelude::*, text::newline};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::parser::{fail, line_terminator};

#[derive(Clone)]
pub enum MarkdownExtension {
    Inline(String, String, fn(String) -> String),
    Block(String, fn(String) -> String, fn(Vec<String>) -> String),
}

impl MarkdownExtension {
    pub fn inline<L: Into<String>, R: Into<String>>(left_delimiter: L, right_delimiter: R, wrapper: fn(String) -> String) -> MarkdownExtension {
        MarkdownExtension::Inline(left_delimiter.into(), right_delimiter.into(), wrapper)
    }

    pub fn block<L: Into<String>>(line_start: L, line_wrapper: fn(String) -> String, block_wrapper: fn(Vec<String>) -> String) -> MarkdownExtension {
        MarkdownExtension::Block(line_start.into(), line_wrapper, block_wrapper)
    }
}

pub(crate) trait MarkdownExtensionList {
    fn build_inline_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String>;
    fn build_block_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String>;
}

impl MarkdownExtensionList for Vec<MarkdownExtension> {
    fn build_inline_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String> {
        self
            .iter()
            .fold(fail().to(String::new()).boxed(), |previous, current| {
                if let MarkdownExtension::Inline(l, r, wrapper) = current.clone() {
                    previous.clone()
                        .or(inline_parser.clone()
                            .nested_in(any()
                                .and_is(just(r.clone()).not())
                                .repeated()
                                .at_least(1)
                                .to_slice()
                                .delimited_by(just(l.clone()), just(r.clone())))
                            .map(wrapper))
                        .boxed()
                } else {
                    previous
                }
            })
    }

    fn build_block_parser<'src>(self, inline_parser: Boxed<'src, 'src, &'src str, String>) -> impl Parser<'src, &'src str, String> {
        self
            .iter()
            .fold(fail().to(String::new()).boxed(), |previous, current| {
                if let MarkdownExtension::Block(start, line_wrapper, block_wrapper) = current.clone() {
                    previous.clone()
                        .or(inline_parser.clone()
                            .nested_in(just(start.clone())
                                .ignore_then(any()
                                    .and_is(line_terminator().not())
                                    .repeated()
                                    .at_least(1)
                                    .to_slice()))
                            .map(line_wrapper)
                            .separated_by(newline())
                            .at_least(1)
                            .collect::<Vec<String>>()
                            .map(block_wrapper))
                        .boxed()
                } else {
                    previous
                }
            })
    }
}

pub fn small() -> MarkdownExtension {
    MarkdownExtension::block(
        "-# ",
        |s| format!("<br/><small>{}</small>", s),
        |lines| lines.concat(),
    )
}

pub fn quote() -> MarkdownExtension {
    MarkdownExtension::block(
        "> ",
        |s| s,
        |lines| format!("</p><blockquote>{}</blockquote><p>", lines.join("<br/>"))
    )
}

pub fn wobbly() -> MarkdownExtension {
    MarkdownExtension::inline("~{", "}~", |s| {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);

        let mut rng = StdRng::seed_from_u64(hasher.finish());
        let mut inners = Vec::new();

        for c in s.chars() {
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

            inners.push(s);
        }

        format!("<span aria-label=\"{}\"><style scoped>@keyframes spin {{ 100% {{ transform: rotate(360deg); }} }}</style>{}</span>", s, inners.concat())
    })
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::parser::markdown::{extension, make_parser};

    #[test]
    fn small() {
        let p = make_parser(&vec![extension::small()]);
        let res = p.parse("-# meow").into_result().unwrap();
        let expected = format!("<br/><small>meow</small>");

        assert_eq!(expected, res);
    }

    #[test]
    fn quote() {
        let p = make_parser(&vec![extension::quote()]);
        let res = p.parse("> a quote\n> with two lines, wow!").into_result().unwrap();
        let expected = format!("<blockquote>a quote<br/>with two lines, wow!</blockquote>");

        assert_eq!(expected, res)
    }

    #[test]
    fn wobbly() {
        let p = make_parser(&vec![extension::wobbly()]);
        let res = p.parse("~{meow}~").into_result().unwrap();
        let expected = String::from("<span aria-label=\"meow\"><style scoped>@keyframes spin { 100% { transform: rotate(360deg); } }</style><span style=\"display: inline-block; animation: 0.04992199295054167s spin linear infinite -0.19491445656372097s alternate; transform: translate(0.057578781559364016em, 0.06125794442563333em);\"><span style=\"display: inline-block; animation: 0.02836486607361373s spin linear infinite -0.18910278062296096s alternate; transform: translate(0.0033866612801565135em, 0.02205159464565092em);\"><span style=\"display: inline-block; animation: 0.10225773476256045s spin linear infinite -0.011829895806127255s alternate; transform: translate(-0.016100840198410876em, 0.07323118943374561em);\"><span style=\"display: inline-block; animation: 0.20798341012731938s spin linear infinite -0.09053936388016155s alternate; transform: translate(-0.04335564893854867em, 0.02711672596335244em);\"><span style=\"display: inline-block; animation: 0.28111209491099826s spin linear infinite -0.0439588598193879s alternate; transform: translate(0.01835655523426052em, 0.012897490567814064em);\"><span style=\"display: inline-block; animation: 0.1617409877487054s spin linear infinite -0.0939408153856864s alternate; transform: translate(-0.013806715809255438em, 0.026754235147636862em);\"><span style=\"display: inline-block; animation: 0.24651841651730738s spin linear infinite -0.10509681733372767s alternate; transform: translate(0.07713703564863185em, 0.012354069476432429em);\">m</span></span></span></span></span></span></span><span style=\"display: inline-block; animation: 0.3444820636305437s spin linear infinite -0.12711128187480586s alternate; transform: translate(0.046626216871658334em, 0.050490633720337845em);\"><span style=\"display: inline-block; animation: 0.363364458440325s spin linear infinite -0.10773461677492963s alternate; transform: translate(-0.041042555395263em, -0.03226776211913636em);\"><span style=\"display: inline-block; animation: 0.3109446973129759s spin linear infinite -0.05887254029808493s alternate; transform: translate(0.06586304139719207em, -0.06200348437822017em);\"><span style=\"display: inline-block; animation: 0.32392220519236137s spin linear infinite -0.1934195676927664s alternate; transform: translate(-0.04409174170276131em, 0.05918937254522481em);\"><span style=\"display: inline-block; animation: 0.2780108428279641s spin linear infinite -0.02315711400213596s alternate; transform: translate(0.019074668908285873em, -0.053545807966725034em);\"><span style=\"display: inline-block; animation: 0.14638983079099274s spin linear infinite -0.10792872992038494s alternate; transform: translate(0.02081087722530599em, 0.05488607408227806em);\"><span style=\"display: inline-block; animation: 0.3188835563791517s spin linear infinite -0.07573114158973211s alternate; transform: translate(0.06334767304421723em, 0.09803336106101042em);\">e</span></span></span></span></span></span></span><span style=\"display: inline-block; animation: 0.2512370553008831s spin linear infinite -0.14180430822893464s alternate; transform: translate(0.00995428512481988em, -0.03173411694855926em);\"><span style=\"display: inline-block; animation: 0.3569118274936637s spin linear infinite -0.09364218408792596s alternate; transform: translate(-0.06483567643613905em, 0.014330537199788962em);\"><span style=\"display: inline-block; animation: 0.230317382142898s spin linear infinite -0.023393857815571507s alternate; transform: translate(0.03736601532461551em, -0.09874948946275264em);\"><span style=\"display: inline-block; animation: 0.2684041735845751s spin linear infinite -0.025110400359935725s alternate; transform: translate(0.044781036457992855em, -0.09985761149163985em);\"><span style=\"display: inline-block; animation: 0.03624261384545098s spin linear infinite -0.18040972288442791s alternate; transform: translate(0.03232009897776951em, -0.07047689439658032em);\"><span style=\"display: inline-block; animation: 0.2522092607703049s spin linear infinite -0.05973920554662204s alternate; transform: translate(-0.050426377225447454em, -0.09458602022917557em);\"><span style=\"display: inline-block; animation: 0.17609741222187694s spin linear infinite -0.15561619279702124s alternate; transform: translate(0.043432540110114125em, 0.05630927897098659em);\">o</span></span></span></span></span></span></span><span style=\"display: inline-block; animation: 0.22775235806190333s spin linear infinite -0.010077911516403715s alternate; transform: translate(0.03459504158223657em, 0.05591659121292041em);\"><span style=\"display: inline-block; animation: 0.049121045527786136s spin linear infinite -0.036362186244833054s alternate; transform: translate(0.035272942328070565em, 0.07049242133513048em);\"><span style=\"display: inline-block; animation: 0.3825261377736466s spin linear infinite -0.00185558941917785s alternate; transform: translate(0.04602139192977326em, 0.0836687774687977em);\"><span style=\"display: inline-block; animation: 0.03341573318524498s spin linear infinite -0.003999534214708289s alternate; transform: translate(-0.035009791027761104em, -0.007351024490648262em);\"><span style=\"display: inline-block; animation: 0.2619923796803942s spin linear infinite -0.14477490450747474s alternate; transform: translate(0.049852836983535806em, 0.010999680821713476em);\"><span style=\"display: inline-block; animation: 0.38016561804022153s spin linear infinite -0.14250230868426192s alternate; transform: translate(-0.07778707960516075em, -0.003209103714134276em);\"><span style=\"display: inline-block; animation: 0.2784717735813666s spin linear infinite -0.17000909449641433s alternate; transform: translate(0.03657967764909161em, 0.038229689489982205em);\">w</span></span></span></span></span></span></span></span>");

        assert_eq!(expected, res);
    }
}
