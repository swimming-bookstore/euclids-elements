//! Read-mode layout: Fitzpatrick paragraphs.
//! Citations hang in the margin. A cited clause breaks so the next
//! sentence starts on a new line; a full stop with no cite does not.
//!
//! Record mode keeps a two-line lyric window. A sentence that wraps
//! past those two lines rolls up: older wraps leave, the active wrap
//! stays on the second line.

use super::script::{Line, Script};

#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    Word {
        text: String,
        italic: bool,
        spaced: bool,
    },
    Cite(String),
    Break,
}

/// How many wrapped rows a word occupies in the record lyric column.
///
/// `widths` are advance widths in the same unit as `width` (em, px, …).
/// A word that does not fit the remaining space of the current row starts
/// the next row. The returned row index is 0-based.
pub fn wrap_rows(word_widths: &[f32], width: f32) -> Vec<usize> {
    let mut rows = Vec::with_capacity(word_widths.len());
    let mut row = 0usize;
    let mut used = 0f32;
    for &w in word_widths {
        let w = if w.is_finite() { w.max(0.0) } else { 0.0 };
        if used > 0.0 && used + w > width {
            row += 1;
            used = 0.0;
        }
        rows.push(row);
        used += w;
    }
    rows
}

/// How many wrapped rows to shift a finished sentence so only its last
/// wrap remains in a one-line previous slot.
pub fn last_row(rows: &[usize]) -> usize {
    rows.last().copied().unwrap_or(0)
}

/// Wrap to keep in the one-line previous slot: the wrap of the last
/// cited word, so a margin cite is not clipped when the sentence
/// continues (`[Prop. 1.1]` on *DE,* then “and let *FC*…”).
pub fn prev_row(rows: &[usize], cited: &[bool]) -> usize {
    cited
        .iter()
        .zip(rows.iter())
        .rev()
        .find(|(c, _)| **c)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| last_row(rows))
}

/// `rows[i]` is the wrap row of token `i`. `active` is the token being
/// sung. Returns how many wrap rows to shift up so the active row sits
/// on the last line of the window (or at its natural row, if it already
/// fits). The previous wrap stays visible above it.
pub fn roll_shift(rows: &[usize], active: usize, window: usize) -> usize {
    let window = window.max(1);
    let Some(&row) = rows.get(active) else {
        return 0;
    };
    row.saturating_sub(window - 1)
}

/// Demo mode parks citations in the margin, so a comma before `{[…]}`
/// still needs a space before the next body word on the lyric line.
pub fn demo_spaced(tokens: &[super::script::Token], i: usize) -> bool {
    if tokens[i].cite {
        return false;
    }
    let next = tokens.get(i + 1);
    if next.is_some_and(|t| t.cite) {
        return tokens[i + 1..].iter().any(|t| !t.cite);
    }
    tokens[i].spaced
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
                if i > 0 {
                    // Phrase breaks are not wrap points. A comma (or any
                    // mark) at the end of a karaoke line still needs a
                    // space before the next phrase in the paragraph.
                    if let Some(Atom::Word { spaced, .. }) = atoms.last_mut() {
                        *spaced = true;
                    }
                }
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
                            spaced: t.spaced,
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
                Atom::Word { text, spaced, .. } => {
                    s.push_str(text);
                    if *spaced {
                        s.push(' ');
                    }
                }
                Atom::Cite(c) => {
                    s.push(' ');
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
    fn wrap_rows_breaks_when_the_next_word_does_not_fit() {
        // Four words of width 3 into a column of width 7: 3+3 fit, then wrap.
        let rows = wrap_rows(&[3.0, 3.0, 3.0, 3.0], 7.0);
        assert_eq!(rows, vec![0, 0, 1, 1]);
    }

    #[test]
    fn wrap_rows_keeps_a_single_overlong_word_on_its_row() {
        let rows = wrap_rows(&[10.0, 2.0], 7.0);
        assert_eq!(rows, vec![0, 1]);
    }

    #[test]
    fn roll_keeps_the_active_row_on_the_last_line_of_a_two_line_window() {
        // I.7's conclusion wraps to four rows. While the cursor is still
        // on the first two, nothing rolls. On the third row the first
        // row leaves; on the fourth, two rows have left.
        let rows = vec![0, 0, 1, 1, 2, 2, 3, 3];
        assert_eq!(roll_shift(&rows, 0, 2), 0);
        assert_eq!(roll_shift(&rows, 3, 2), 0);
        assert_eq!(roll_shift(&rows, 4, 2), 1);
        assert_eq!(roll_shift(&rows, 7, 2), 2);
    }

    #[test]
    fn roll_does_not_shift_a_sentence_that_already_fits() {
        let rows = vec![0, 0, 1];
        assert_eq!(roll_shift(&rows, 2, 2), 0);
        assert_eq!(last_row(&rows), 1);
    }

    #[test]
    fn finished_prev_keeps_the_wrap_with_the_cite() {
        // I.11 demo breaks after the cite, so [Prop. 1.1] is wrap 1
        // and “and let FC have been joined.” is wrap 2.
        let rows = vec![0, 0, 1, 1, 2];
        let cited = vec![false, true, false, true, false];
        assert_eq!(prev_row(&rows, &cited), 1);
        assert_eq!(last_row(&rows), 2);
        let cited_last = vec![false, false, false, false, true];
        assert_eq!(prev_row(&rows, &cited_last), 2);
        assert_eq!(prev_row(&rows, &[false, false, false, false, false]), 2);
    }

    #[test]
    fn demo_spaces_after_a_comma_cite() {
        struct NoneMap;
        impl crate::karaoke::PartsMap for NoneMap {
            fn parts(&self, _: &str) -> Vec<String> {
                Vec::new()
            }
        }
        let phrases = [Phrase {
            para: 1,
            text: "constructed upon *DE,*{[Prop. 1.1]} and let *AF* have been joined.",
        }];
        let script = compile(&phrases, &NoneMap, Timing::default());
        let toks = &script.lines[0].tokens;
        let body: Vec<_> = toks
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.cite)
            .map(|(i, t)| (t.text.as_str(), demo_spaced(toks, i)))
            .collect();
        assert_eq!(
            body,
            vec![
                ("constructed", true),
                ("upon", true),
                ("DE,", true),
                ("and", true),
                ("let", true),
                ("AF", true),
                ("have", true),
                ("been", true),
                ("joined.", false),
            ]
        );
    }

    #[test]
    fn em_dash_glues_to_the_italic_name() {
        struct NoneMap;
        impl crate::karaoke::PartsMap for NoneMap {
            fn parts(&self, _: &str) -> Vec<String> {
                Vec::new()
            }
        }
        let phrases = [Phrase {
            para: 1,
            text: "(that) *BCG*—that is to say, *ACD*—(is) also",
        }];
        let script = compile(&phrases, &NoneMap, Timing::default());
        let texts: Vec<_> = script.lines[0]
            .tokens
            .iter()
            .map(|t| (t.text.as_str(), t.spaced))
            .collect();
        assert_eq!(
            texts,
            vec![
                ("(that)", true),
                ("BCG—", false),
                ("that", true),
                ("is", true),
                ("to", true),
                ("say,", true),
                ("ACD—", false),
                ("(is)", true),
                ("also", false),
            ]
        );
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
            .map(|t| (t.text.as_str(), t.italic, t.cite, t.spaced))
            .collect();
        assert_eq!(
            texts,
            vec![
                ("Let", false, false, true),
                ("AB", true, false, true),
                ("be", false, false, true),
                ("drawn,", false, false, false),
                ("[Post. 3]", false, true, false),
                ("and", false, false, true),
                ("again.", false, false, false),
            ]
        );
    }

    #[test]
    fn paren_does_not_take_a_gap_inside_the_marks() {
        struct NoneMap;
        impl crate::karaoke::PartsMap for NoneMap {
            fn parts(&self, _: &str) -> Vec<String> {
                Vec::new()
            }
        }
        let phrases = [Phrase {
            para: 1,
            text: "constructed upon (*AB*),{[Prop. 1.1]} and let *DE,* be joined.",
        }];
        let script = compile(&phrases, &NoneMap, Timing::default());
        let texts: Vec<_> = script.lines[0]
            .tokens
            .iter()
            .map(|t| (t.text.as_str(), t.spaced))
            .collect();
        assert_eq!(
            texts,
            vec![
                ("constructed", true),
                ("upon", true),
                ("(AB),", false),
                ("[Prop. 1.1]", false),
                ("and", true),
                ("let", true),
                ("DE,", true),
                ("be", true),
                ("joined.", false),
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
        assert!(
            qed.contains("respectively, and have the angle")
                && qed.contains("equal, then they will also have the base"),
            "comma at a phrase join must keep a space: {qed}"
        );
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

    #[test]
    fn i10_fitzpatrick_paragraphs() {
        let i10 = layout_of(1, 10);
        assert_eq!(i10.len(), 4, "I.10 has four Fitzpatrick paragraphs");
        assert!(cites(&i10[0]).is_empty());
        assert_eq!(cites(&i10[1]), vec!["[Prop. 1.1]", "[Prop. 1.9]"]);
        assert_eq!(cites(&i10[2]), vec!["[Prop. 1.4]"]);
        assert!(cites(&i10[3]).is_empty());
        let given = para_text(&i10[0]);
        assert!(given.contains("Let AB be the given finite straight-line"));
        assert!(given.contains("cut the finite straight-line AB in half."));
        let construction = para_text(&i10[1]);
        assert!(construction.contains(
            "upon (AB), [Prop. 1.1] \nand let the angle ACB have been cut in half"
        ));
        assert!(construction.contains("straight-line CD. [Prop. 1.9] \nI say that"));
        let proof = para_text(&i10[2]);
        assert!(proof.contains("For since AC is equal to CB, and CD (is) common"));
        assert!(proof.contains("equal to the base BD. [Prop. 1.4]"));
        let qed = para_text(&i10[3]);
        assert!(qed.contains(
            "cut in half at (point) D. (Which is) the very thing it was required to do."
        ));
    }

    #[test]
    fn i11_fitzpatrick_paragraphs() {
        let i11 = layout_of(1, 11);
        assert_eq!(i11.len(), 4, "I.11 has four Fitzpatrick paragraphs");
        assert!(cites(&i11[0]).is_empty());
        assert_eq!(cites(&i11[1]), vec!["[Prop. 1.3]", "[Prop. 1.1]"]);
        assert_eq!(cites(&i11[2]), vec!["[Prop. 1.8]", "[Def. 1.10]"]);
        assert!(cites(&i11[3]).is_empty());
        let given = para_text(&i11[0]);
        assert!(given.contains("Let AB be the given straight-line, and C the given point"));
        let construction = para_text(&i11[1]);
        assert!(construction.contains("equal to CD, [Prop. 1.3] \nand let the equilateral"));
        assert!(construction.contains("on DE, [Prop. 1.1] \nand let FC have been joined."));
        let proof = para_text(&i11[2]);
        assert!(proof.contains("For since DC is equal to CE, and CF is common"));
        assert!(proof.contains(
            "equal to the angle ECF, [Prop. 1.8] \nand they are adjacent."
        ));
        assert!(proof.contains("is a right-angle. [Def. 1.10] \nThus, each of the (angles) DCF"));
        assert!(
            proof.contains("respectively. And the base DF is equal to the base FE."),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i11[3]);
        assert!(qed.contains(
            "at right-angles to the given straight-line AB from the given point C on it. (Which is) the very thing it was required to do."
        ));
    }

    #[test]
    fn i12_fitzpatrick_paragraphs() {
        let i12 = layout_of(1, 12);
        assert_eq!(i12.len(), 4, "I.12 has four Fitzpatrick paragraphs");
        assert!(cites(&i12[0]).is_empty());
        assert_eq!(cites(&i12[1]), vec!["[Post. 3]", "[Prop. 1.10]"]);
        assert_eq!(cites(&i12[2]), vec!["[Prop. 1.8]", "[Def. 1.10]"]);
        assert!(cites(&i12[3]).is_empty());
        let given = para_text(&i12[0]);
        assert!(given.contains("Let AB be the given infinite straight-line"));
        assert!(given.contains("which is not on (AB)"));
        let construction = para_text(&i12[1]);
        assert!(construction.contains("radius CD, [Post. 3] \nand let the straight-line EG"));
        assert!(construction.contains("at (point) H, [Prop. 1.10] \nand let the straight-lines CG"));
        let proof = para_text(&i12[2]);
        assert!(proof.contains("For since GH is equal to HE, and HC (is) common"));
        assert!(proof.contains(
            "equal to the angle EHC, [Prop. 1.8] \nand they are adjacent."
        ));
        assert!(proof.contains(
            "upon which it stands. [Def. 1.10]"
        ));
        assert!(
            proof.contains("respectively, and the base CG is equal to the base CE."),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i12[3]);
        assert!(qed.contains(
            "which is not on (AB). (Which is) the very thing it was required to do."
        ));
    }

    #[test]
    fn i13_fitzpatrick_paragraphs() {
        let i13 = layout_of(1, 13);
        assert_eq!(i13.len(), 3, "I.13 has three Fitzpatrick paragraphs");
        assert!(cites(&i13[0]).is_empty());
        assert_eq!(
            cites(&i13[1]),
            vec!["[Def. 1.10]", "[Prop. 1.11]", "[C.N. 2]", "[C.N. 2]", "[C.N. 1]"]
        );
        assert!(cites(&i13[2]).is_empty());
        let given = para_text(&i13[0]);
        assert!(given.contains("For let some straight-line AB stood on the straight-line CD"));
        assert!(given.contains("(have a sum) equal to two right-angles"));
        let proof = para_text(&i13[1]);
        assert!(proof.contains("two right-angles. [Def. 1.10] \nBut, if not"));
        assert!(proof.contains("at right-angles to CD. [Prop. 1.11] \nThus, CBE and EBD"));
        assert!(proof.contains("and EBD. [C.N. 2] \nAgain, since DBA"));
        assert!(proof.contains("and ABC. [C.N. 2] \nBut (the sum of) CBE"));
        assert!(proof.contains("one another. [C.N. 1] \nTherefore, (the sum of) CBE"));
        assert!(
            proof.contains("two right-angles. And since CBE is equal"),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i13[2]);
        assert!(qed.contains(
            "equal to two right-angles. (Which is) the very thing it was required to show."
        ));
    }

    #[test]
    fn i16_fitzpatrick_paragraphs() {
        let i16 = layout_of(1, 16);
        assert_eq!(i16.len(), 4, "I.16 has four Fitzpatrick paragraphs");
        assert!(cites(&i16[0]).is_empty());
        assert_eq!(cites(&i16[1]), vec!["[Prop. 1.10]", "[Prop. 1.3]"]);
        assert_eq!(cites(&i16[2]), vec!["[Prop. 1.15]", "[Prop. 1.4]"]);
        assert!(cites(&i16[3]).is_empty());
        let given = para_text(&i16[0]);
        assert!(given.contains("Let ABC be a triangle, and let one of its sides BC"));
        assert!(given.contains("internal and opposite angles, CBA and BAC."));
        let construction = para_text(&i16[1]);
        assert!(construction.contains("at (point) E. [Prop. 1.10] \nAnd BE being joined"));
        assert!(construction.contains("equal to BE, [Prop. 1.3] \nand let FC have been joined"));
        let proof = para_text(&i16[2]);
        assert!(proof.contains("vertically opposite. [Prop. 1.15] \nThus, the base AB"));
        assert!(proof.contains(
            "corresponding remaining angles. [Prop. 1.4] \nThus, BAE is equal to ECF."
        ));
        assert!(
            proof.contains("greater than ECF. Thus, ACD is greater than BAE."),
            "plain full stops stay: {proof}"
        );
        assert!(
            proof.contains("BCG—that is to say, ACD—(is) also greater"),
            "Fitzpatrick has no space around the em dashes: {proof}"
        );
        let qed = para_text(&i16[3]);
        assert!(qed.contains(
            "internal and opposite angles. (Which is) the very thing it was required to show."
        ));
    }

    #[test]
    fn i17_fitzpatrick_paragraphs() {
        let i17 = layout_of(1, 17);
        assert_eq!(i17.len(), 4, "I.17 has four Fitzpatrick paragraphs");
        assert!(cites(&i17[0]).is_empty());
        assert!(cites(&i17[1]).is_empty());
        assert_eq!(cites(&i17[2]), vec!["[Prop. 1.16]", "[Prop. 1.13]"]);
        assert!(cites(&i17[3]).is_empty());
        let given = para_text(&i17[0]);
        assert!(given.contains("Let ABC be a triangle."));
        assert!(given.contains("less than two right-angles."));
        let construction = para_text(&i17[1]);
        assert!(construction.contains("For let BC have been produced to D."));
        let proof = para_text(&i17[2]);
        assert!(proof.contains("opposite angle ABC. [Prop. 1.16] \nLet ACB have been added"));
        assert!(proof.contains("two right-angles. [Prop. 1.13] \nThus, (the sum of) ABC"));
        assert!(
            proof.contains("less than two right-angles. Similarly, we can show"),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i17[3]);
        assert!(qed.contains(
            "less than two right-angles. (Which is) the very thing it was required to show."
        ));
    }

    #[test]
    fn i18_fitzpatrick_paragraphs() {
        let i18 = layout_of(1, 18);
        assert_eq!(i18.len(), 4, "I.18 has four Fitzpatrick paragraphs");
        assert!(cites(&i18[0]).is_empty());
        assert_eq!(cites(&i18[1]), vec!["[Prop. 1.3]"]);
        assert_eq!(cites(&i18[2]), vec!["[Prop. 1.16]", "[Prop. 1.5]"]);
        assert!(cites(&i18[3]).is_empty());
        let given = para_text(&i18[0]);
        assert!(given.contains("For let ABC be a triangle having side AC greater than AB."));
        assert!(given.contains("angle ABC is also greater than BCA."));
        let construction = para_text(&i18[1]);
        assert!(construction.contains(
            "equal to AB, [Prop. 1.3] \nand let BD have been joined."
        ));
        let proof = para_text(&i18[2]);
        assert!(proof.contains("opposite (angle) DCB. [Prop. 1.16] \nBut ADB (is) equal"));
        assert!(proof.contains("equal to side AD. [Prop. 1.5] \nThus, ABD is also greater"));
        assert!(
            proof.contains("greater than ACB. Thus, ABC is much greater than ACB."),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i18[3]);
        assert!(qed.contains(
            "the greater angle. (Which is) the very thing it was required to show."
        ));
    }

    #[test]
    fn i19_fitzpatrick_paragraphs() {
        let i19 = layout_of(1, 19);
        assert_eq!(i19.len(), 3, "I.19 has three Fitzpatrick paragraphs");
        assert!(cites(&i19[0]).is_empty());
        assert_eq!(cites(&i19[1]), vec!["[Prop. 1.5]", "[Prop. 1.18]"]);
        assert!(cites(&i19[2]).is_empty());
        let given = para_text(&i19[0]);
        assert!(given.contains("Let ABC be a triangle having the angle ABC greater than BCA."));
        assert!(given.contains("side AC is also greater than side AB."));
        let proof = para_text(&i19[1]);
        assert!(proof.contains("equal to ACB. [Prop. 1.5] \nBut it is not."));
        assert!(proof.contains("less than ACB. [Prop. 1.18] \nBut it is not."));
        assert!(
            proof.contains("not equal to AB. Neither, indeed, is AC less than AB."),
            "plain full stops stay: {proof}"
        );
        let qed = para_text(&i19[2]);
        assert!(qed.contains(
            "by the greater side. (Which is) the very thing it was required to show."
        ));
    }

    #[test]
    fn i21_fitzpatrick_paragraphs() {
        let i21 = layout_of(1, 21);
        assert_eq!(i21.len(), 4, "I.21 has four Fitzpatrick paragraphs");
        assert!(cites(&i21[0]).is_empty());
        assert_eq!(cites(&i21[1]), vec!["[Prop. 1.20]"]);
        assert_eq!(cites(&i21[2]), vec!["[Prop. 1.16]"]);
        assert!(cites(&i21[3]).is_empty());
        let given = para_text(&i21[0]);
        assert!(given.contains("straight-lines BD and DC have been constructed"));
        assert!(given.contains("angle BDC greater than BAC."));
        let sides = para_text(&i21[1]);
        assert!(sides.contains("remaining (side), [Prop. 1.20] \nin triangle ABE"));
        assert!(sides.contains("much greater than (the sum of) BD and DC."));
        let angles = para_text(&i21[2]);
        assert!(angles.contains("opposite (angles), [Prop. 1.16] \nin triangle CDE"));
        assert!(angles.contains("Thus, BDC is much greater than BAC."));
        let qed = para_text(&i21[3]);
        assert!(qed.contains(
            "a greater angle. (Which is) the very thing it was required to show."
        ));
    }

    #[test]
    fn i22_fitzpatrick_paragraphs() {
        let i22 = layout_of(1, 22);
        assert_eq!(i22.len(), 4, "I.22 has four Fitzpatrick paragraphs");
        assert!(cites(&i22[0]).is_empty());
        assert_eq!(cites(&i22[1]), vec!["[Prop. 1.3]"]);
        assert!(cites(&i22[2]).is_empty());
        assert!(cites(&i22[3]).is_empty());
        let given = para_text(&i22[0]);
        assert!(given.contains("Let A, B, and C be the three given straight-lines"));
        assert!(given.contains("equal to A, B, and C."));
        let construction = para_text(&i22[1]);
        assert!(construction.contains("equal to C. [Prop. 1.3] \nAnd let the circle DKL"));
        assert!(construction.contains("triangle KFG has been constructed"));
        let proof = para_text(&i22[2]);
        assert!(proof.contains("FD is equal to FK."));
        assert!(proof.contains("equal to A, B, and C (respectively)."));
        let qed = para_text(&i22[3]);
        assert!(qed.contains(
            "(respectively). (Which is) the very thing it was required to do."
        ));
    }

    #[test]
    fn i23_fitzpatrick_paragraphs() {
        let i23 = layout_of(1, 23);
        assert_eq!(i23.len(), 4, "I.23 has four Fitzpatrick paragraphs");
        assert!(cites(&i23[0]).is_empty());
        assert_eq!(cites(&i23[1]), vec!["[Prop. 1.22]"]);
        assert_eq!(cites(&i23[2]), vec!["[Prop. 1.8]"]);
        assert!(cites(&i23[3]).is_empty());
        let given = para_text(&i23[0]);
        assert!(given.contains("Let AB be the given straight-line"));
        assert!(given.contains("on the given straight-line AB."));
        let construction = para_text(&i23[1]);
        assert!(construction.contains("and let DE have been joined. And let the triangle AFG"));
        assert!(construction.contains("DE to FG. [Prop. 1.22]"));
        let proof = para_text(&i23[2]);
        assert!(proof.contains("the angle DCE is thus equal to the angle FAG. [Prop. 1.8]"));
        let qed = para_text(&i23[3]);
        assert!(qed.contains(
            "AB. (Which is) the very thing it was required to do."
        ));
    }

    #[test]
    fn i24_fitzpatrick_paragraphs() {
        let i24 = layout_of(1, 24);
        assert_eq!(i24.len(), 4, "I.24 has four Fitzpatrick paragraphs");
        assert!(cites(&i24[0]).is_empty());
        assert_eq!(cites(&i24[1]), vec!["[Prop. 1.23]", "[Prop. 1.3]"]);
        assert_eq!(
            cites(&i24[2]),
            vec!["[Prop. 1.4]", "[Prop. 1.5]", "[Prop. 1.19]"]
        );
        assert!(cites(&i24[3]).is_empty());
        let given = para_text(&i24[0]);
        assert!(given.contains("Let ABC and DEF be two triangles"));
        assert!(given.contains("greater than the base EF."));
        let construction = para_text(&i24[1]);
        assert!(construction.contains("on the straight-line DE. [Prop. 1.23] \nAnd let DG"));
        assert!(construction.contains("AC or DF, [Prop. 1.3] \nand let EG and FG have been joined."));
        let proof = para_text(&i24[2]);
        assert!(proof.contains("the base BC is equal to the base EG. [Prop. 1.4] \nAgain"));
        assert!(proof.contains("angle DFG. [Prop. 1.5] \nThus, DFG"));
        assert!(proof.contains("greater side, [Prop. 1.19] \nside EG"));
        assert!(proof.contains("Thus, BC (is) also greater than EF."));
        let qed = para_text(&i24[3]);
        assert!(qed.contains(
            "(of the latter). (Which is) the very thing it was required to show."
        ));
    }
}
