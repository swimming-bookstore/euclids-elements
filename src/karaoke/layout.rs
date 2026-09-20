//! Read-mode layout: Fitzpatrick paragraphs, margin cites, break after a comma-cite.

use super::script::{Line, Script, Token};

#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    Word { text: String, italic: bool },
    Cite(String),
    Break,
}

pub fn read_layout(script: &Script) -> Vec<Vec<Atom>> {
    read_layout_lines(&script.lines)
}

pub fn read_layout_lines(lines: &[Line]) -> Vec<Vec<Atom>> {
    let mut out: Vec<(u8, Vec<Atom>)> = Vec::new();
    for line in lines {
        let mut chunk = Vec::new();
        let mut comma_cite = false;
        for t in &line.tokens {
            if t.cite {
                chunk.push(Atom::Cite(t.text.clone()));
            } else {
                chunk.push(Atom::Word {
                    text: t.text.clone(),
                    italic: t.italic,
                });
                comma_cite = t.text.ends_with(',');
            }
        }
        if comma_cite {
            chunk.push(Atom::Break);
        }
        match out.last_mut() {
            Some((p, atoms)) if *p == line.para => atoms.extend(chunk),
            _ => out.push((line.para, chunk)),
        }
    }
    out.into_iter().map(|(_, a)| a).collect()
}

pub fn split_cites(tokens: &[Token]) -> (Vec<Token>, Vec<Token>) {
    let mut body = Vec::new();
    let mut cites = Vec::new();
    for t in tokens {
        if t.cite {
            cites.push(t.clone());
        } else {
            body.push(t.clone());
        }
    }
    (body, cites)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{get, Phrase};
    use crate::karaoke::{compile, Timing};

    fn layout_of(book: u8, n: u8) -> Vec<Vec<Atom>> {
        let p = get(book, n).expect("proposition");
        let script = compile(p.phrases, &(p.figure)(), Timing::default());
        read_layout(&script)
    }

    fn para_text(para: &[Atom]) -> String {
        let mut s = String::new();
        for a in para {
            match a {
                Atom::Word { text, .. } => {
                    if !s.is_empty() && !s.ends_with('\n') {
                        s.push(' ');
                    }
                    s.push_str(text);
                }
                Atom::Cite(c) => {
                    s.push(' ');
                    s.push_str(c);
                }
                Atom::Break => s.push('\n'),
            }
        }
        s
    }

    fn cites(para: &[Atom]) -> Vec<&str> {
        para.iter()
            .filter_map(|a| match a {
                Atom::Cite(c) => Some(c.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn tokenize_italic_cite_and_punct() {
        struct NoneMap;
        impl crate::karaoke::PartsMap for NoneMap {
            fn parts(&self, _: &str) -> Vec<String> {
                Vec::new()
            }
        }
        let phrases = [Phrase {
            para: 1,
            text: "Let *AB* be drawn,{[Post. 3]} and again.",
        }];
        let script = compile(&phrases, &NoneMap, Timing::default());
        let texts: Vec<_> = script.lines[0]
            .tokens
            .iter()
            .map(|t| (t.text.as_str(), t.italic, t.cite))
            .collect();
        assert_eq!(
            texts,
            vec![
                ("Let", false, false),
                ("AB", true, false),
                ("be", false, false),
                ("drawn,", false, false),
                ("[Post. 3]", false, true),
                ("and", false, false),
                ("again.", false, false),
            ]
        );
    }

    #[test]
    fn comma_cite_breaks_next_clause() {
        let i1 = layout_of(1, 1);
        assert_eq!(i1.len(), 5);
        let construction = para_text(&i1[2]);
        assert!(
            construction.contains("drawn,\n"),
            "comma-cite must break: {construction}"
        );
        assert!(construction.contains("[Post. 3]\nand again"));
        assert_eq!(
            cites(&i1[2]),
            vec!["[Post. 3]", "[Post. 3]", "[Post. 1]"]
        );
    }

    #[test]
    fn period_cite_does_not_break_paragraph() {
        let i2 = layout_of(1, 2);
        assert_eq!(i2.len(), 4, "I.2 has four Fitzpatrick paragraphs");
        let argument = &i2[2];
        assert!(
            !argument.iter().any(|a| matches!(a, Atom::Break)),
            "argument is running text, not one block per sentence"
        );
        assert_eq!(
            cites(argument),
            vec!["[Def. 1.15]", "[Def. 1.15]", "[C.N. 3]", "[C.N. 1]"]
        );
        let text = para_text(argument);
        assert!(text.contains("BG. [Def. 1.15] Again,"));
        assert!(text.contains("DG. [Def. 1.15] And within these,"));
        assert!(text.contains("BG. [C.N. 3] But BC was also shown"));
        assert!(text.contains("another. [C.N. 1] Thus, AL is also equal to BC."));
    }

    #[test]
    fn i2_construction_breaks_after_post_three() {
        let i2 = layout_of(1, 2);
        let construction = para_text(&i2[1]);
        assert!(construction.contains("drawn,\n"));
        assert!(construction.contains("[Post. 3]\nand again let"));
        assert!(construction.contains("[Post. 1]\nand let the equilateral"));
    }
}
