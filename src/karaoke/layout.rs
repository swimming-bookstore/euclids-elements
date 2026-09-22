//! Read-mode layout: Fitzpatrick paragraphs.
//! Citations hang in the margin. A cited clause breaks so the next
//! sentence starts on a new line; a full stop with no cite does not.

use super::script::{Line, Script};

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
    let mut groups: Vec<(u8, Vec<&Line>)> = Vec::new();
    for line in lines {
        match groups.last_mut() {
            Some((p, ls)) if *p == line.para => ls.push(line),
            _ => groups.push((line.para, vec![line])),
        }
    }
    groups
        .into_iter()
        .map(|(_, ls)| {
            let mut atoms = Vec::new();
            for (i, line) in ls.iter().enumerate() {
                for t in &line.tokens {
                    if t.cite {
                        atoms.push(Atom::Cite(t.text.clone()));
                    } else {
                        atoms.push(Atom::Word {
                            text: t.text.clone(),
                            italic: t.italic,
                        });
                    }
                }
                if i + 1 < ls.len() && line.tokens.iter().any(|t| t.cite) {
                    atoms.push(Atom::Break);
                }
            }
            atoms
        })
        .collect()
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

    /// Same order and spaces as `KaraokeRead`: word, space, cite, space, break.
    fn para_text(para: &[Atom]) -> String {
        let mut s = String::new();
        for a in para {
            match a {
                Atom::Word { text, .. } => {
                    s.push_str(text);
                    s.push(' ');
                }
                Atom::Cite(c) => {
                    s.push_str(c);
                    s.push(' ');
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
    fn i1_construction_breaks_after_cites() {
        let i1 = layout_of(1, 1);
        assert_eq!(i1.len(), 5);
        let construction = para_text(&i1[2]);
        assert!(
            construction.contains("drawn, [Post. 3] \nand again"),
            "comma-cite must break after the margin cite: {construction}"
        );
        assert!(
            construction.contains("drawn. [Post. 3] \nAnd let the straight-lines"),
            "period-cite must start the next sentence on a new line: {construction}"
        );
        assert!(
            !construction.contains("[Post. 3] And let"),
            "must not stay on the same line: {construction}"
        );
        assert_eq!(
            cites(&i1[2]),
            vec!["[Post. 3]", "[Post. 3]", "[Post. 1]"]
        );
    }

    #[test]
    fn i2_argument_breaks_after_cites_only() {
        let i2 = layout_of(1, 2);
        assert_eq!(i2.len(), 4, "I.2 has four Fitzpatrick paragraphs");
        let argument = &i2[2];
        assert_eq!(
            cites(argument),
            vec!["[Def. 1.15]", "[Def. 1.15]", "[C.N. 3]", "[C.N. 1]"]
        );
        let text = para_text(argument);
        assert!(text.contains("BG. [Def. 1.15] \nAgain,"));
        assert!(text.contains("DG. [Def. 1.15] \nAnd within these,"));
        assert!(text.contains("BG. [C.N. 3] \nBut BC was also shown"));
        assert!(text.contains("another. [C.N. 1] \nThus, AL is also equal to BC."));
        assert!(
            text.contains("DB. Thus, the remainder")
                && text.contains("BG. Thus, AL and BC")
                && text.contains("BG. But things equal"),
            "plain full stops stay in the paragraph: {text}"
        );
        assert!(
            !text.contains("DB. \n") && !text.contains("BG. \nThus, AL and"),
            "plain full stop must not break: {text}"
        );
    }

    #[test]
    fn i2_construction_breaks_after_cites() {
        let i2 = layout_of(1, 2);
        let construction = para_text(&i2[1]);
        assert!(
            construction.contains("drawn, [Post. 3] \nand again let"),
            "comma-cite must break after the margin cite: {construction}"
        );
        assert!(construction.contains("[Post. 1] \nand let the equilateral"));
        assert!(
            construction.contains("upon it. [Prop. 1.1] \nAnd let the straight-lines"),
            "period-cite must break: {construction}"
        );
    }

    #[test]
    fn i3_fitzpatrick_paragraphs() {
        let i3 = layout_of(1, 3);
        assert_eq!(i3.len(), 4, "I.3 has four Fitzpatrick paragraphs");
        assert_eq!(cites(&i3[1]), vec!["[Prop. 1.2]", "[Post. 3]"]);
        assert_eq!(cites(&i3[2]), vec!["[Def. 1.15]", "[C.N. 1]"]);
        let given = para_text(&i3[0]);
        assert!(given.contains("Let AB and C be the two given unequal straight-lines"));
        assert!(given.contains("AB. So it is required"), "plain full stop stays: {given}");
        let construction = para_text(&i3[1]);
        assert!(
            construction.contains("point A. [Prop. 1.2] \nAnd let the circle"),
            "period-cite must break: {construction}"
        );
        let argument = para_text(&i3[2]);
        assert!(argument.contains("AD. [Def. 1.15] \nBut, C is also equal"));
        assert!(
            argument.contains("AD. Thus, AE and C") && argument.contains("AD. So AE is also"),
            "plain full stops stay: {argument}"
        );
        assert!(argument.contains("C. [C.N. 1]"));
    }
}
