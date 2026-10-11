//! Compile proof lines into a karaoke script.

pub struct Timing {
    pub word_ms: u32,
    pub hit_ms: u32,
    pub cite_ms: u32,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            word_ms: 700,
            hit_ms: 1800,
            cite_ms: 0,
        }
    }
}

/// Map a spoken word onto figure part ids.
pub trait PartsMap {
    fn parts(&self, word: &str) -> Vec<String>;
    fn angle_parts(&self, word: &str) -> Vec<String> {
        self.parts(word)
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub italic: bool,
    pub cite: bool,
    /// A space follows this token. False when the next mark is punctuation
    /// (`(*AB*)`, `*DE,*`) so the gap is not painted inside the marks.
    pub spaced: bool,
    pub parts: Vec<String>,
    pub dur_ms: u32,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub para: u8,
    pub tokens: Vec<Token>,
}

#[derive(Clone, Debug)]
pub struct Script {
    pub lines: Vec<Line>,
}

impl Script {
    pub fn get(&self, n: usize) -> Option<(usize, usize, &Token)> {
        token_at(&self.lines, n)
    }

    pub fn parts_at(&self, n: usize) -> &[String] {
        for i in (0..=n).rev() {
            if let Some((_, _, t)) = self.get(i) {
                if !t.parts.is_empty() {
                    return t.parts.as_slice();
                }
            }
        }
        &[]
    }
}

pub fn compile(
    phrases: &[crate::content::Phrase],
    map: &impl PartsMap,
    timing: Timing,
) -> Script {
    // "the angle *BAC*" and a later "triangle *ABC*" share a phrase list.
    // Each letter-run is classified from the words that introduce it, not
    // from a flag that leaks across sentences.
    Script {
        lines: phrases
            .iter()
            .map(|p| Line {
                para: p.para,
                tokens: tokenize(p.text, map, &timing),
            })
            .collect(),
    }
}

fn token_at(lines: &[Line], mut n: usize) -> Option<(usize, usize, &Token)> {
    for (li, line) in lines.iter().enumerate() {
        if n < line.tokens.len() {
            return Some((li, n, &line.tokens[n]));
        }
        n -= line.tokens.len();
    }
    None
}

fn tokenize(src: &str, map: &impl PartsMap, timing: &Timing) -> Vec<Token> {
    let mut out = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut pending_open = String::new();
    let mut recent: Vec<String> = Vec::new();
    while i < chars.len() {
        if chars[i] == '{' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            let cite: String = chars[start..i].iter().collect();
            if i < chars.len() {
                i += 1;
            }
            out.push(Token {
                text: cite,
                italic: false,
                cite: true,
                spaced: false,
                parts: Vec::new(),
                dur_ms: timing.cite_ms,
            });
            continue;
        }
        if chars[i] == '*' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '*' {
                i += 1;
            }
            let inner: String = chars[start..i].iter().collect();
            if i < chars.len() {
                i += 1;
            }
            let pieces = split_sentence(&inner);
            for (n, piece) in pieces.iter().enumerate() {
                // A period inside the italic run starts a new sentence, so
                // the next piece is not still "the angle …".
                let following = words_after(&chars, i);
                let angle = n == 0
                    && letters_are_angles(&recent, &following)
                    && !introduced_as_figure(&recent);
                push_words(
                    &mut out,
                    piece,
                    true,
                    map,
                    timing,
                    angle,
                    &mut pending_open,
                );
                if n + 1 < pieces.len() {
                    recent.clear();
                }
            }
            recent.clear();
            glue_punct(&mut out, &chars, &mut i, &mut pending_open);
            continue;
        }
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if opens_word(chars[i]) {
            // "(angle)" is one introducer. A parenthesis with no letters
            // still waits to glue onto the next word.
            let start = i;
            while i < chars.len() && (opens_word(chars[i]) || chars[i].is_ascii_alphabetic() || chars[i] == '-')
            {
                i += 1;
            }
            if chars[start..i].iter().any(|c| c.is_ascii_alphabetic()) {
                let word: String = chars[start..i].iter().collect();
                recent.push(word.clone());
                push_words(&mut out, &word, false, map, timing, false, &mut pending_open);
            } else {
                pending_open.push(chars[start]);
                i = start + 1;
            }
            continue;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '*' && chars[i] != '{' {
            i += 1;
        }
        let word: String = chars[start..i].iter().collect();
        if is_punct_only(&word) {
            glue_text(&mut out, &word);
            if word.contains('.') || word.contains(';') {
                recent.clear();
            }
        } else {
            // "(angle)" is the word "angle" with a parenthesis glued on.
            // Keep it in the introducer list; the parenthesis stays on the
            // following italic run via pending_open.
            let stem_src = word.trim_matches(|c: char| is_punct(c) && !opens_word(c));
            if !stem_src.is_empty() {
                recent.push(stem_src.to_string());
            }
            push_words(&mut out, &word, false, map, timing, false, &mut pending_open);
        }
    }
    mark_spaces(&mut out);
    out
}

/// Words after an italic run, up to the next letter-run or sentence end.
/// "BDC was shown greater than CEB" puts the comparison after the name.
fn words_after(chars: &[char], mut i: usize) -> Vec<String> {
    let mut words = Vec::new();
    while i < chars.len() && chars[i] != '*' && chars[i] != '.' && chars[i] != ';' {
        if chars[i].is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '-') {
                i += 1;
            }
            words.push(chars[start..i].iter().collect());
            continue;
        }
        i += 1;
    }
    words
}

/// True when the words around this italic run name an angle.
///
/// "the angle *BAC*" and "(angle) *BAC*" are angles. "triangle *ABC*" is not.
/// "angle *BDC* greater than (angle) *BAC*" is two angles. "in triangle *CDE*
/// the external angle *BDC*" names the angle after the triangle. "But, *BDC*
/// was shown greater than *CEB*" has the comparison after the name — a
/// three-letter name next to "greater than" is the angle, not the triangle.
fn letters_are_angles(recent: &[String], following: &[String]) -> bool {
    let mut angle = false;
    let mut compared = false;
    let mut figure = false;
    for word in recent {
        let w = stem(word);
        if w.is_empty() {
            continue;
        }
        if w.starts_with("angle") || w == "angles" {
            // "in triangle CDE the external angle BDC" names the angle
            // after the triangle. The earlier figure word no longer applies.
            angle = true;
            figure = false;
            compared = false;
            continue;
        }
        if w == "than" || w == "to" {
            compared = true;
        }
        if figure_word(&w) {
            angle = false;
            figure = true;
            compared = false;
        }
    }
    if following.iter().any(|word| {
        let w = stem(word);
        w == "than" || w.starts_with("angle")
    }) {
        compared = true;
    }
    angle || compared
}

/// "triangle *ABE*" and "side *AB*" name a figure even when a later clause
/// says "greater than". "in triangle *CDE* the external angle *BDC*" does
/// not — the angle word comes after the figure word.
fn introduced_as_figure(recent: &[String]) -> bool {
    let mut figure = false;
    for word in recent {
        let w = stem(word);
        if w.starts_with("angle") || w == "angles" {
            figure = false;
        } else if figure_word(&w) {
            figure = true;
        }
    }
    figure
}

fn stem(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_ascii_alphabetic() || *c == '-')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn figure_word(w: &str) -> bool {
    matches!(
        w,
        "triangle"
            | "triangles"
            | "base"
            | "circle"
            | "straight-line"
            | "straight-lines"
            | "point"
            | "points"
            | "side"
            | "sides"
    )
}

/// A space follows a word. It does not follow a word whose next mark is a
/// citation (`AB,{[Prop. 1.1]}` → `AB, [Prop. 1.1]`), and it does not sit
/// inside a parenthesis that was attached to the word.
fn mark_spaces(tokens: &mut [Token]) {
    for i in 0..tokens.len() {
        if tokens[i].cite {
            continue;
        }
        let next = tokens.get(i + 1);
        if tokens[i].spaced {
            if matches!(next, Some(t) if t.cite) {
                tokens[i].spaced = false;
            }
            continue;
        }
        if tokens[i].text.ends_with('—') || tokens[i].text.ends_with('–') {
            tokens[i].spaced = false;
            continue;
        }
        tokens[i].spaced = match next {
            Some(t) if t.cite => false,
            Some(_) => true,
            None => false,
        };
    }
}

fn opens_word(c: char) -> bool {
    matches!(c, '(' | '[')
}

/// `*D. (Which*` and `*respectively.And*` close the sentence inside the
/// italic run. Split so the period keeps the space before the next word,
/// and so the words after the stop do not inherit the previous figure.
fn split_sentence(inner: &str) -> Vec<String> {
    let chars: Vec<char> = inner.chars().collect();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + 1 < chars.len() {
        let boundary = chars[i] == '.'
            && (chars[i + 1].is_ascii_uppercase() || chars[i + 1].is_whitespace());
        if boundary {
            out.push(chars[start..=i].iter().collect());
            start = i + 1;
            while start < chars.len() && chars[start].is_whitespace() {
                start += 1;
            }
            i = start;
            continue;
        }
        i += 1;
    }
    if start < chars.len() {
        out.push(chars[start..].iter().collect());
    }
    out
}

fn is_punct(c: char) -> bool {
    matches!(
        c,
        ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '(' | '[' | '—' | '–'
    )
}

fn is_punct_only(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_punct)
}

fn glue_text(out: &mut Vec<Token>, extra: &str) {
    if extra.is_empty() || extra.chars().all(opens_word) {
        return;
    }
    if let Some(last) = out.iter_mut().rev().find(|t| !t.cite) {
        last.text.push_str(extra);
        if extra.contains('.') {
            last.spaced = true;
        }
    }
}

fn glue_punct(out: &mut Vec<Token>, chars: &[char], i: &mut usize, pending_open: &mut String) {
    while *i < chars.len() && is_punct(chars[*i]) && !opens_word(chars[*i]) {
        glue_text(out, &chars[*i].to_string());
        *i += 1;
    }
    while *i < chars.len() && opens_word(chars[*i]) {
        pending_open.push(chars[*i]);
        *i += 1;
    }
}

fn push_words(
    out: &mut Vec<Token>,
    chunk: &str,
    italic: bool,
    map: &impl PartsMap,
    timing: &Timing,
    angle: bool,
    pending_open: &mut String,
) {
    if chunk.is_empty() {
        return;
    }
    let mut words: Vec<&str> = chunk.split_whitespace().collect();
    if words.is_empty() {
        return;
    }
    if !pending_open.is_empty() {
        let first = format!("{}{}", pending_open, words[0]);
        pending_open.clear();
        let owned = std::mem::take(&mut words);
        push_one(out, &first, italic, map, timing, angle);
        for word in owned.into_iter().skip(1) {
            push_one(out, word, italic, map, timing, angle);
        }
        return;
    }
    for word in words {
        push_one(out, word, italic, map, timing, angle);
    }
}

fn push_one(
    out: &mut Vec<Token>,
    word: &str,
    italic: bool,
    map: &impl PartsMap,
    timing: &Timing,
    angle: bool,
) {
    let bare: String = word.chars().filter(|c| !opens_word(*c)).collect();
    let parts = if italic {
        if angle {
            map.angle_parts(&bare)
        } else {
            map.parts(&bare)
        }
    } else {
        Vec::new()
    };
    let dur = if parts.is_empty() {
        timing.word_ms
    } else {
        timing.hit_ms
    };
    out.push(Token {
        text: word.to_string(),
        italic,
        cite: false,
        spaced: false,
        parts,
        dur_ms: dur,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Phrase;
    use crate::figure::{book1_prop4, book1_prop21};

    fn parts_of(text: &'static str, word: &str) -> Vec<String> {
        let phrases = [Phrase { para: 1, text }];
        let script = compile(&phrases, &book1_prop4(), Timing::default());
        first_parts(&script, word)
    }

    fn all_parts(text: &'static str, word: &str) -> Vec<Vec<String>> {
        let phrases = [Phrase { para: 1, text }];
        let script = compile(&phrases, &book1_prop4(), Timing::default());
        script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| letter_key(&t.text) == word)
            .map(|t| t.parts.clone())
            .collect()
    }

    fn letter_key(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase())
            .collect()
    }

    fn first_parts(script: &Script, word: &str) -> Vec<String> {
        script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .find(|t| letter_key(&t.text) == word)
            .map(|t| t.parts.clone())
            .unwrap_or_default()
    }

    fn is_angle(parts: &[String]) -> bool {
        parts.iter().any(|s| s == "ab")
            && parts.iter().any(|s| s == "ca" || s == "ac")
            && !parts.iter().any(|s| s == "bc")
    }

    #[test]
    fn angle_is_not_the_triangle() {
        let bac = parts_of("the angle *BAC*", "BAC");
        assert!(is_angle(&bac), "∠BAC is not triangle ABC: {bac:?}");
        let abc = parts_of("triangle *ABC*", "ABC");
        assert!(abc.iter().any(|s| s == "bc"), "triangle ABC includes the base: {abc:?}");
    }

    #[test]
    fn later_bac_after_a_triangle_is_still_an_angle() {
        // Sticky angle-mode used to die at "triangle" and never return,
        // so the second BAC lit the whole triangle.
        let text = "the external angle *CEB* of the triangle *ABE* is also greater than *BAC*.";
        let phrases = [Phrase { para: 1, text }];
        let script = compile(&phrases, &book1_prop21(), Timing::default());
        let bac = first_parts(&script, "BAC");
        assert!(
            !bac.iter().any(|s| s == "bc"),
            "greater than BAC is an angle: {bac:?}"
        );
        let abe = first_parts(&script, "ABE");
        assert!(
            abe.iter().any(|s| s == "bd") && abe.iter().any(|s| s == "ab") && abe.iter().any(|s| s == "ae"),
            "triangle ABE keeps its sides: {abe:?}"
        );
        let bdc = first_parts(
            &compile(
                &[Phrase {
                    para: 1,
                    text: "an angle *BDC* greater than (angle) *BAC*.",
                }],
                &book1_prop21(),
                Timing::default(),
            ),
            "BDC",
        );
        assert!(
            bdc.iter().any(|s| s == "bd") && bdc.iter().any(|s| s == "dc") && !bdc.iter().any(|s| s == "bc"),
            "∠BDC is not a triangle: {bdc:?}"
        );
        // "in triangle CDE the external angle BDC" used to keep the figure
        // flag, so BDC lit triangle BDC (including BC).
        let later = first_parts(
            &compile(
                &[Phrase {
                    para: 1,
                    text: "in triangle *CDE* the external angle *BDC* is thus greater than *CED*.",
                }],
                &book1_prop21(),
                Timing::default(),
            ),
            "BDC",
        );
        assert!(
            later.iter().any(|s| s == "bd")
                && later.iter().any(|s| s == "dc")
                && !later.iter().any(|s| s == "bc"),
            "external angle BDC is not triangle BDC: {later:?}"
        );
        let shown = first_parts(
            &compile(
                &[Phrase {
                    para: 1,
                    text: "But, *BDC* was shown (to be) greater than *CEB*.",
                }],
                &book1_prop21(),
                Timing::default(),
            ),
            "BDC",
        );
        assert!(
            !shown.iter().any(|s| s == "bc"),
            "BDC greater than CEB is an angle: {shown:?}"
        );
    }

    #[test]
    fn compared_side_is_not_an_angle() {
        let ab = parts_of("having side *AC* greater than *AB*.", "AB");
        assert!(ab.iter().any(|s| s == "ab") && !ab.iter().any(|s| s == "bc"));
        let abc = parts_of("of the triangle *ABC* greater than *AB*", "ABC");
        assert!(abc.iter().any(|s| s == "bc"), "a named triangle stays a triangle: {abc:?}");
    }
    #[test]
    fn parenthetical_angle_does_not_stick_into_the_next_sentence() {
        let text = "I say that the angle *ABC* is equal to *ACB*. And they are at the base of triangle *ABC*.";
        let runs = all_parts(text, "ABC");
        assert_eq!(runs.len(), 2, "runs={runs:?}");
        assert!(!runs[0].iter().any(|s| s == "ac"), "first ABC is an angle: {runs:?}");
        assert!(runs[1].iter().any(|s| s == "bc"), "second ABC is the triangle");
    }
}
