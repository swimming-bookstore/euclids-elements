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
                for (ti, t) in line.tokens.iter().enumerate() {
                    if t.cite {
                        atoms.push(Atom::Cite(t.text.clone()));
                        let more_here = line.tokens[ti + 1..]
                            .iter()
                            .any(|u| !u.cite);
                        let more_later = ls[i + 1..].iter().any(|l| !l.tokens.is_empty());
                        if more_here || more_later {
                            atoms.push(Atom::Break);
                        }
                    } else {
                        atoms.push(Atom::Word {
                            text: t.text.clone(),
                            italic: t.italic,
                        });
                    }
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

    #[test]
    fn i4_fitzpatrick_paragraphs() {
        let i4 = layout_of(1, 4);
        assert_eq!(i4.len(), 3, "I.4 has three Fitzpatrick paragraphs");
        assert!(cites(&i4[0]).is_empty());
        assert_eq!(
            cites(&i4[1]),
            vec!["[Post. 1]", "[C.N. 4]", "[C.N. 4]", "[C.N. 4]", "[C.N. 4]"]
        );
        assert!(cites(&i4[2]).is_empty());
        let given = para_text(&i4[0]);
        assert!(given.contains("Let ABC and DEF be two triangles"));
        assert!(given.contains("respectively. (That is) AB to DE"));
        assert!(given.contains("angle BAC (be) equal to the angle EDF"));
        assert!(given.contains("corresponding remaining angles. (That is) ABC to DEF"));
        let proof = para_text(&i4[1]);
        assert!(proof.contains("For if triangle ABC is applied"));
        assert!(proof.contains("encompass an area. The very thing is impossible. [Post. 1]"));
        assert!(proof.contains("impossible. [Post. 1] \nThus, the base BC"));
        assert!(proof.contains("equal to it. [C.N. 4] \nSo the whole triangle"));
        assert!(proof.contains("equal to them. [C.N. 4] \n(That is) ABC to DEF"));
        assert!(
            proof.contains("DE. So (because of)") && proof.contains("DF. But, point B"),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i4[2]);
        assert!(qed.contains("equal straight-line equal"));
        assert!(qed.contains("corresponding remaining angles. (Which is) the very thing it was required to show."));
    }

    #[test]
    fn i5_fitzpatrick_paragraphs() {
        let i5 = layout_of(1, 5);
        assert_eq!(i5.len(), 4, "I.5 has four Fitzpatrick paragraphs");
        assert_eq!(cites(&i5[0]), vec!["[Post. 2]"]);
        assert_eq!(cites(&i5[1]), vec!["[Prop. 1.3]", "[Post. 1]"]);
        assert_eq!(
            cites(&i5[2]),
            vec!["[Prop. 1.4]", "[C.N. 3]", "[Prop. 1.4]", "[C.N. 3]"]
        );
        assert!(cites(&i5[3]).is_empty());
        let given = para_text(&i5[0]);
        assert!(given.contains("Let ABC be an isosceles triangle"));
        assert!(given.contains("(respectively). [Post. 2] \nI say that the angle ABC"));
        assert!(given.contains("equal to ACB, and (angle) CBD to BCE."));
        let construction = para_text(&i5[1]);
        assert!(construction.contains("lesser AF. [Prop. 1.3] \nAlso, let the straight-lines"));
        let proof = para_text(&i5[2]);
        assert!(proof.contains("In fact, since AF is equal to AG"));
        assert!(proof.contains("remaining angles. [Prop. 1.4] \n(That is) ACF to ABG"));
        assert!(proof.contains("to AGB. And since the whole of AF"));
        assert!(proof.contains("within which AB is equal to AC"));
        assert!(proof.contains("remainder CG. [C.N. 3] \nBut FC was also shown"));
        assert!(proof.contains("common to them. Thus, the triangle BFC"));
        assert!(proof.contains("remaining angles. [Prop. 1.4] \nThus, FBC is equal to GCB"));
        assert!(proof.contains("to CBG. Therefore, since the whole angle ABG"));
        assert!(proof.contains("within which CBG is equal to BCF"));
        assert!(proof.contains("remainder ACB. [C.N. 3] \nAnd they are at the base"));
        assert!(proof.contains("ABC. And FBC was also shown"));
        assert!(proof.contains("under the base."));
        let qed = para_text(&i5[3]);
        assert!(qed.contains("under the base will be equal to one another. (Which is) the very thing it was required to show."));
    }

    #[test]
    fn i6_fitzpatrick_paragraphs() {
        let i6 = layout_of(1, 6);
        assert_eq!(i6.len(), 4, "I.6 has four Fitzpatrick paragraphs");
        assert!(cites(&i6[0]).is_empty());
        assert_eq!(cites(&i6[1]), vec!["[Prop. 1.3]", "[Post. 1]"]);
        assert_eq!(cites(&i6[2]), vec!["[Prop. 1.4]", "[C.N. 5]"]);
        assert!(cites(&i6[3]).is_empty());
        let given = para_text(&i6[0]);
        assert!(given.contains("Let ABC be a triangle having the angle ABC"));
        assert!(given.contains("I say that side AB is also equal to side AC."));
        let construction = para_text(&i6[1]);
        assert!(construction.contains("greater. Let AB be greater."));
        assert!(construction.contains("greater AB. [Prop. 1.3] \nAnd let DC have been joined. [Post. 1]"));
        let reductio = para_text(&i6[2]);
        assert!(reductio.contains("triangle ACB, [Prop. 1.4] \nthe lesser to the greater."));
        assert!(reductio.contains("absurd. [C.N. 5] \nThus, AB is not unequal"));
        assert!(
            reductio.contains("greater. The very notion") && reductio.contains("AC. Thus, (it is) equal."),
            "plain full stops stay: {reductio}"
        );
        let qed = para_text(&i6[3]);
        assert!(qed.contains("equal to one another. (Which is) the very thing it was required to show."));
    }

    #[test]
    fn i7_fitzpatrick_paragraphs() {
        let i7 = layout_of(1, 7);
        assert_eq!(i7.len(), 3, "I.7 has three Fitzpatrick paragraphs");
        assert_eq!(cites(&i7[0]), vec!["[Post. 1]"]);
        assert_eq!(
            cites(&i7[1]),
            vec!["[Prop. 1.5]", "[C.N. 5]", "[C.N. 5]", "[Prop. 1.5]"]
        );
        assert!(cites(&i7[2]).is_empty());
        let given = para_text(&i7[0]);
        assert!(given.contains("For, if possible, let the two straight-lines AC, CB"));
        assert!(given.contains("joined. [Post. 1]"));
        let proof = para_text(&i7[1]);
        assert!(proof.contains("angle ADC. [Prop. 1.5] \nThus, ADC (is) greater"));
        assert!(proof.contains("DCB. [C.N. 5] \nThus, CDB is much greater"));
        assert!(proof.contains("angle DCB. [Prop. 1.5] \nBut it was shown"));
        assert!(
            proof.contains("latter). The very thing is impossible."),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i7[2]);
        assert!(qed.contains("given straight-lines. (Which is) the very thing it was required to show."));
    }

    #[test]
    fn i8_fitzpatrick_paragraphs() {
        let i8 = layout_of(1, 8);
        assert_eq!(i8.len(), 3, "I.8 has three Fitzpatrick paragraphs");
        assert!(cites(&i8[0]).is_empty());
        assert_eq!(cites(&i8[1]), vec!["[Prop. 1.7]", "[C.N. 4]"]);
        assert!(cites(&i8[2]).is_empty());
        let given = para_text(&i8[0]);
        assert!(given.contains("Let ABC and DEF be two triangles"));
        assert!(given.contains("respectively. (That is) AB to DE"));
        assert!(given.contains("base BC equal to the base EF"));
        let proof = para_text(&i8[1]);
        assert!(proof.contains("For if triangle ABC is applied to triangle DEF"));
        assert!(proof.contains("like EG and GF (in the above figure)"));
        assert!(proof.contains("cannot be constructed. [Prop. 1.7] \nThus, the base BC"));
        assert!(proof.contains("equal to it. [C.N. 4]"));
        assert!(
            proof.contains("respectively). Thus, they will coincide."),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i8[2]);
        assert!(qed.contains("two side, respectively"));
        assert!(qed.contains("equal straight-lines. (Which is) the very thing it was required to show."));
    }

    #[test]
    fn i9_fitzpatrick_paragraphs() {
        let i9 = layout_of(1, 9);
        assert_eq!(i9.len(), 4, "I.9 has four Fitzpatrick paragraphs");
        assert!(cites(&i9[0]).is_empty());
        assert_eq!(cites(&i9[1]), vec!["[Prop. 1.3]", "[Prop. 1.1]"]);
        assert_eq!(cites(&i9[2]), vec!["[Prop. 1.8]"]);
        assert!(cites(&i9[3]).is_empty());
        let given = para_text(&i9[0]);
        assert!(given.contains("Let BAC be the given rectilinear angle"));
        let construction = para_text(&i9[1]);
        assert!(construction.contains("cut off from AC, [Prop. 1.3] \nand let DE have been joined."));
        assert!(construction.contains("constructed upon DE, [Prop. 1.1] \nand let AF have been joined."));
        let proof = para_text(&i9[2]);
        assert!(proof.contains("For since AD is equal to AE, and AF is common, the two (straight-lines) DA, AF"));
        assert!(proof.contains("angle EAF. [Prop. 1.8]"));
        let qed = para_text(&i9[3]);
        assert!(qed.contains("cut in half by the straight-line AF. (Which is) the very thing it was required to do."));
    }
}
