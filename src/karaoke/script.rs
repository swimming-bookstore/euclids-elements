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

pub fn display_letters(text: &str) -> String {
    if let Some(name) = angle_object(text) {
        return name;
    }
    text.to_string()
}

/// `*angle(DCE)*` is an angle object. The letters are the name; the wrapper
/// is the kind. Nearby words cannot turn it back into a triangle.
fn angle_object(piece: &str) -> Option<String> {
    let t = piece.trim();
    let rest = t.strip_prefix("angle(")?.strip_suffix(')')?;
    let name: String = rest
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();
    // angle(s) is Fitzpatrick's plural, not an angle object.
    if name.len() < 2 || !name.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    Some(name)
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
        if chars[i] == '[' {
            // Fitzpatrick's square-bracket interpolation ("[And the halves…]").
            // Not a citation — those are wrapped in `{…}`.
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if word.chars().any(|c| c.is_ascii_alphabetic()) {
                recent.push(word.clone());
                push_words(&mut out, &word, false, map, timing, false, &mut pending_open);
            }
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
                // angle(DCE) is the object. Words around it do not reclassify it.
                let wrapped = angle_object(piece);
                let spoken = wrapped.as_deref().unwrap_or(piece);
                // "*DEF*, and *BCA*" — the comma is part of the italic run, but it
                // is not part of the name. Record the bare name before classifying
                // so "equal to" still names the next three-letter run.
                let name = spoken.trim_matches(|c: char| !c.is_ascii_alphabetic());
                if !name.is_empty() {
                    recent.push(name.to_string());
                    if recent.len() > 24 {
                        recent.remove(0);
                    }
                }
                let carried = n > 0 && letters_are_angles(&recent, &[]);
                // The name just pushed is this run, not an earlier introducer.
                let before: Vec<String> = if recent.last().is_some_and(|w| w == name) {
                    recent[..recent.len() - 1].to_vec()
                } else {
                    recent.clone()
                };
                let angle = wrapped.is_some()
                    || ((n == 0 || carried)
                        && letters_are_angles(if carried { &[] } else { &before }, &following)
                        && !introduced_as_figure(&before))
                    || carried;
                push_words(
                    &mut out,
                    spoken,
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
/// three-letter name next to "greater than" or "equal to" is the angle, not
/// the triangle. "In fact, *BAC* is not equal to *EDF*" is that case: the
/// comparison follows the name, so both letter-runs are angles.
fn letters_are_angles(recent: &[String], following: &[String]) -> bool {
    let mut angle = false;
    let mut compared = false;
    for word in recent {
        let w = stem(word);
        if w.is_empty() {
            continue;
        }
        if w.starts_with("angle") || w == "angles" {
            // "in triangle CDE the external angle BDC" names the angle
            // after the triangle. The earlier figure word no longer applies.
            angle = true;
            compared = false;
            continue;
        }
        if w == "than" || w == "equal" || w == "unequal" {
            compared = true;
        }
        // "But," is punctuation, not a new subject. It must not wipe the
        // comparison that names "*EFD* is equal to *BCA*".
        if matches!(w.as_str(), "but" | "and" | "thus" | "so" | "again" | "also") {
            continue;
        }
        // "the given rectilinear angle *DCE*" keeps the angle through the
        // adjective. A later figure word ("triangle *ABC*") still clears it.
        // A noun between "angle" and the letters ("point") clears it too.
        // "equal to" is the comparison itself. "to" must not clear it, or
        // "*BCA* to *EFD*" lights triangle EFD.
        if figure_word(&w) {
            angle = false;
            compared = false;
        } else if w != "to" && !is_angle_adjective(&w) {
            angle = false;
            if w != "than" && w != "equal" && w != "unequal" {
                compared = false;
            }
        }
    }
    // "be two triangles having the two angles" names this run as the triangle.
    // A later "angles" belongs to the next letter-runs, not this one.
    // "*DCE* the given rectilinear angle" still names the angle: no figure
    // word comes before it.
    let followed_by_angle = followed_by_angle_name(following);
    if following.iter().any(|word| {
        let w = stem(word);
        w == "than" || w == "equal" || w == "unequal"
    }) {
        compared = true;
    }
    // "(That is) *ABC* (equal) to *DEF*, and *BCA* to *EFD*".
    // The only word before the next name is "to", so this run is the angle
    // too. "to the points" still has an ordinary word after "to".
    if following.len() == 1 && following.first().is_some_and(|w| stem(w) == "to") {
        compared = true;
    }
    // "*BCA* to *EFD*." — "to" is the whole introducer, and the sentence ends.
    if recent.len() == 1 && recent.first().is_some_and(|w| stem(w) == "to") && following.is_empty()
    {
        compared = true;
    }
    // "Let *ABC* and *DEF* be two triangles having the two angles…"
    // The following clause names angles, but this letter-run is the triangle.
    // "having" is not a comparison ("equal to" / "greater than").
    if following.first().is_some_and(|word| stem(word) == "having") {
        compared = false;
    }
    // "and *DCE* the given rectilinear angle" names the angle after the
    // letters. "than" / "equal to" alone still only apply to a three-letter
    // name that an earlier "angle" introduced — a two-letter name is a side.
    // The caller also drops this when the letters were introduced as a figure
    // ("triangle *ABC* is equal to *DEF*").
    angle || followed_by_angle || compared
}

/// True when a later "angle" names this letter-run, not a figure introduced
/// before that word. "be two triangles having the two angles" is the triangle.
fn followed_by_angle_name(following: &[String]) -> bool {
    let mut saw_figure = false;
    for word in following {
        let w = stem(word);
        if figure_word(&w) {
            saw_figure = true;
        }
        if w.starts_with("angle") || w == "angles" {
            return !saw_figure;
        }
    }
    false
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

/// Words that may sit between "angle" and the letter name without naming
/// a different figure: "rectilinear angle *DCE*", "given angle *D*".
fn is_angle_adjective(w: &str) -> bool {
    matches!(
        w,
        "rectilinear"
            | "given"
            | "external"
            | "adjacent"
            | "remaining"
            | "equal"
            | "corresponding"
            | "right"
            | "right-angle"
            | "right-angles"
            | "vertical"
            | "interior"
            | "exterior"
            | "opposite"
    )
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
    let bare: String = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
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
    use crate::figure::{book1_prop4, book1_prop21, book1_prop23, book1_prop27, book1_prop32};

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
    fn equal_to_three_letters_is_an_angle() {
        // I.25: "In fact, *BAC* is not equal to *EDF*." BAC is an angle.
        // The comparison follows the name, so the triangle highlight is wrong.
        let phrases = crate::content::get(1, 25).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop25(), Timing::default());
        let bare: Vec<Vec<String>> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| letter_key(&t.text) == "BAC")
            .map(|t| t.parts.clone())
            .collect();
        assert!(bare.len() >= 2, "BAC runs: {bare:?}");
        for (i, parts) in bare.iter().enumerate() {
            assert!(
                parts.iter().any(|s| s == "ab" || s == "ba")
                    && parts.iter().any(|s| s == "ac" || s == "ca")
                    && !parts.iter().any(|s| s == "bc"),
                "BAC run {i} is an angle, not triangle ABC: {parts:?}"
            );
        }
        // I.19 uses the same "is not equal to" shape for sides. Those stay sides.
        let ac = parts_of("In fact, *AC* is not equal to *AB*.", "AC");
        assert!(
            ac.iter().any(|s| s == "ac" || s == "ca") && !ac.iter().any(|s| s == "bc"),
            "a two-letter name next to equal is a side: {ac:?}"
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
    fn triangles_having_angles_are_not_angles() {
        // I.26 opens "Let *ABC* and *DEF* be two triangles having the two angles…".
        // "having" is not "equal to". ABC is the triangle, not ∠ABC.
        let phrases = crate::content::get(1, 26).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop26(), Timing::default());
        let abc = first_parts(&script, "ABC");
        assert!(
            abc.iter().any(|s| s == "ab" || s == "ag")
                && abc.iter().any(|s| s == "bc" || s == "bh")
                && abc.iter().any(|s| s == "ac" || s == "ca"),
            "first ABC is triangle ABC, not ∠ABC: {abc:?}"
        );
        let def = first_parts(&script, "DEF");
        assert!(
            def.iter().any(|s| s == "de" || s == "ed")
                && def.iter().any(|s| s == "ef" || s == "fe")
                && def.iter().any(|s| s == "df" || s == "fd"),
            "first DEF is triangle DEF, not ∠DEF: {def:?}"
        );
        let def_angle = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| letter_key(&t.text) == "DEF")
            .nth(1)
            .map(|t| t.parts.clone())
            .unwrap_or_default();
        assert!(
            def_angle.iter().any(|s| s == "de" || s == "ed")
                && def_angle.iter().any(|s| s == "ef" || s == "fe")
                && !def_angle.iter().any(|s| s == "df" || s == "fd"),
            "DEF equal to ABC is ∠DEF: {def_angle:?}"
        );
        let later = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| letter_key(&t.text) == "ABC")
            .nth(1)
            .map(|t| t.parts.clone())
            .unwrap_or_default();
        assert!(
            later.iter().any(|s| s == "ag" || s == "gb")
                && later.iter().any(|s| s == "bc" || s == "bh")
                && later.iter().any(|s| s == "A")
                && !later.iter().any(|s| s == "ac" || s == "ca"),
            "the two angles ABC is ∠ABC, both rays: {later:?}"
        );
        // "(That is) *ABC* (equal) to *DEF*, and *BCA* to *EFD*."
        // The period after DEF splits the italic run. BCA is still the angle.
        let bca = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| letter_key(&t.text) == "BCA")
            .map(|t| t.parts.clone())
            .collect::<Vec<_>>();
        assert!(
            bca.len() >= 2
                && bca[0].iter().any(|s| s == "bc" || s == "cb" || s == "bh" || s == "hc")
                && bca[0].iter().any(|s| s == "ca" || s == "ac")
                && !bca[0].iter().any(|s| s == "ab" || s == "ba"),
            "BCA equal to EFD is ∠BCA, not triangle BCA: {bca:?}"
        );
        let efd = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| letter_key(&t.text) == "EFD")
            .map(|t| t.parts.clone())
            .collect::<Vec<_>>();
        assert!(
            efd.len() >= 2
                && efd.iter().all(|parts| {
                    parts.iter().any(|s| s == "ef" || s == "fe")
                        && parts.iter().any(|s| s == "fd" || s == "df")
                        && !parts.iter().any(|s| s == "de" || s == "ed")
                }),
            "every EFD in I.26 is ∠EFD, not triangle EFD: {efd:?}"
        );
    }
    fn parts_of_fig(text: &'static str, word: &str, fig: &impl PartsMap) -> Vec<String> {
        let phrases = [Phrase { para: 1, text }];
        let script = compile(&phrases, fig, Timing::default());
        first_parts(&script, word)
    }

    #[test]
    fn angle_object_is_not_the_triangle() {
        let dce = parts_of_fig(
            "and *angle(DCE)* the given rectilinear angle.",
            "DCE",
            &book1_prop23(),
        );
        assert!(
            dce.iter().any(|s| s == "cd" || s == "dc")
                && dce.iter().any(|s| s == "ce" || s == "ec")
                && !dce.iter().any(|s| s == "de" || s == "ed"),
            "angle(DCE) is the two rays: {dce:?}"
        );
        let shown = display_letters("angle(DCE)");
        assert_eq!(shown, "DCE");
        let phrases = crate::content::get(1, 23).unwrap().phrases;
        let script = compile(phrases, &book1_prop23(), Timing::default());
        let first = first_parts(&script, "DCE");
        assert!(
            first.iter().any(|s| s == "cd" || s == "dc")
                && first.iter().any(|s| s == "ce" || s == "ec")
                && !first.iter().any(|s| s == "de" || s == "ed"),
            "first spoken DCE is an angle: {first:?}"
        );
    }

    #[test]
    fn prop32_added_acb_is_an_angle() {
        let phrases = crate::content::get(1, 32).unwrap().phrases;
        let say = phrases.iter().find(|p| p.text.contains("added to both")).unwrap().text;
        assert!(say.contains("*angle(ACB)*"), "added ACB is an angle: {say}");
        let script = compile(phrases, &book1_prop32(), Timing::default());
        let parts = first_parts(&script, "ACB");
        assert!(
            parts.iter().any(|s| s == "ac" || s == "ca")
                && parts.iter().any(|s| s == "bc" || s == "cb")
                && !parts.iter().any(|s| s == "ab" || s == "ba"),
            "added ACB is the angle, not triangle ABC: {parts:?}"
        );
    }

    #[test]
    fn prop32_final_sum_names_angles() {
        let phrases = crate::content::get(1, 32).unwrap().phrases;
        let say = phrases
            .iter()
            .find(|p| p.text.contains("is also equal to two right-angles"))
            .unwrap()
            .text;
        assert!(
            say.contains("*angle(ACB)*")
                && say.contains("*angle(CBA)*")
                && say.contains("*angle(CAB)*"),
            "the final sum names angles: {say}"
        );
        let script = compile(phrases, &book1_prop32(), Timing::default());
        for (word, closed) in [("ACB", ["ab", "ba"]), ("CBA", ["ac", "ca"]), ("CAB", ["bc", "cb"])] {
            let parts = script
                .lines
                .iter()
                .flat_map(|l| &l.tokens)
                .filter(|t| {
                    t.text
                        .chars()
                        .filter(|c| c.is_ascii_alphabetic())
                        .collect::<String>()
                        == word
                })
                .last()
                .map(|t| t.parts.clone())
                .unwrap_or_default();
            assert!(
                closed.iter().all(|s| !parts.iter().any(|p| p == s)),
                "{word} in the final sum is an angle, not the triangle: {parts:?}"
            );
            assert!(parts.len() >= 2, "{word} lights both rays: {parts:?}");
        }
    }

    #[test]
    fn prop32_three_internal_angles_are_angles() {
        let phrases = crate::content::get(1, 32).unwrap().phrases;
        let script = compile(phrases, &book1_prop32(), Timing::default());
        let say = phrases[1].text;
        assert!(
            say.contains("*angle(ABC)*")
                && say.contains("*angle(BCA)*")
                && say.contains("*angle(CAB)*"),
            "the three internal angles are marked: {say}"
        );
        for (word, rays, closed) in [
            ("ABC", ["ab", "ba", "bc", "cb"], ["ac", "ca"]),
            ("BCA", ["bc", "cb", "ac", "ca"], ["ab", "ba"]),
            ("CAB", ["ac", "ca", "ab", "ba"], ["bc", "cb"]),
        ] {
            let parts = first_parts(&script, word);
            let rays_hit = rays.iter().filter(|s| parts.iter().any(|p| p == *s)).count();
            assert!(
                rays_hit >= 2 && closed.iter().all(|s| !parts.iter().any(|p| p == s)),
                "{word} is an angle, not the triangle: {parts:?}"
            );
        }
    }

    #[test]
    fn rectilinear_angle_is_not_the_triangle() {
        // "rectilinear" used to clear the angle flag, so *DCE* lit DE as well.
        let dce = parts_of_fig(
            "and *DCE* the given rectilinear angle.",
            "DCE",
            &book1_prop23(),
        );
        assert!(
            dce.iter().any(|s| s == "cd" || s == "dc")
                && dce.iter().any(|s| s == "ce" || s == "ec")
                && !dce.iter().any(|s| s == "de" || s == "ed"),
            "rectilinear angle DCE is the two rays: {dce:?}"
        );
        let again = parts_of_fig(
            "equal to the given rectilinear angle *DCE* at the point *A*",
            "DCE",
            &book1_prop23(),
        );
        assert!(
            !again.iter().any(|s| s == "de" || s == "ed"),
            "given rectilinear angle DCE: {again:?}"
        );
    }

    #[test]
    fn parenthetical_angle_does_not_stick_into_the_next_sentence() {
        let text = "I say that the angle *ABC* is equal to *ACB*. And they are at the base of triangle *ABC*.";
        let runs = all_parts(text, "ABC");
        assert_eq!(runs.len(), 2, "runs={runs:?}");
        assert!(!runs[0].iter().any(|s| s == "ac"), "first ABC is an angle: {runs:?}");
        assert!(runs[1].iter().any(|s| s == "bc"), "second ABC is the triangle");
    }

    #[test]
    fn prop27_efg_is_the_wedge_at_f() {
        let phrases = [Phrase {
            para: 2,
            text: "So, for the triangle *GEF*, the external angle *angle(AEF)* is equal to the interior and opposite (angle) *angle(EFG)*.",
        }];
        let script = compile(&phrases, &book1_prop27(), Timing::default());
        let mut found = false;
        for line in &script.lines {
            for t in &line.tokens {
                if t.text.contains("EFG") {
                    found = true;
                    assert!(
                        t.parts.iter().any(|p| p == "fd" || p == "df" || p == "dg" || p == "gd")
                            && t.parts.iter().any(|p| p == "ef" || p == "fe")
                            && !t.parts.iter().any(|p| p == "fg" || p == "gf"),
                        "sung ∠EFG lights the wedge at F, not a drawn FG, got {:?} for {:?}",
                        t.parts,
                        t.text
                    );
                }
            }
        }
        assert!(found, "no EFG token");
    }
}
