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
    let mut angle = false;
    Script {
        lines: phrases
            .iter()
            .map(|p| Line {
                para: p.para,
                tokens: tokenize(p.text, map, &timing, &mut angle),
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

fn tokenize(src: &str, map: &impl PartsMap, timing: &Timing, angle: &mut bool) -> Vec<Token> {
    let mut out = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut pending_open = String::new();
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
            for piece in split_sentence(&inner) {
                push_words(
                    &mut out,
                    &piece,
                    true,
                    map,
                    timing,
                    *angle,
                    &mut pending_open,
                );
            }
            glue_punct(&mut out, &chars, &mut i, &mut pending_open);
            continue;
        }
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if opens_word(chars[i]) {
            pending_open.push(chars[i]);
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '*' && chars[i] != '{' {
            i += 1;
        }
        let word: String = chars[start..i].iter().collect();
        if is_punct_only(&word) {
            glue_text(&mut out, &word);
        } else {
            note_angle(angle, &word);
            push_words(&mut out, &word, false, map, timing, false, &mut pending_open);
        }
    }
    mark_spaces(&mut out);
    out
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
/// italic run. Split so the period keeps the space before the next word.
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

fn note_angle(angle: &mut bool, word: &str) {
    let w: String = word
        .chars()
        .filter(|c| c.is_ascii_alphabetic() || *c == '-')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if w.starts_with("angle") {
        *angle = true;
        return;
    }
    if matches!(
        w.as_str(),
        "triangle"
            | "triangles"
            | "base"
            | "circle"
            | "straight-line"
            | "straight-lines"
            | "point"
            | "points"
    ) {
        *angle = false;
        return;
    }
    if !matches!(
        w.as_str(),
        "the"
            | "a"
            | "an"
            | "to"
            | "and"
            | "or"
            | "of"
            | "equal"
            | "being"
            | "be"
            | "let"
            | "is"
            | "that"
            | "this"
            | "also"
            | "corresponding"
            | "remaining"
            | "enclosed"
            | "by"
            | "will"
            | "are"
            | "with"
            | "one"
            | "other"
            | "another"
            | "respectively"
    ) && !w.is_empty()
    {
        // Keep angle mode across "to DEF" / "(That is) ABC".
    }
}

fn is_punct(c: char) -> bool {
    matches!(
        c,
        ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '(' | '['
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
    use crate::figure::book1_prop4;

    fn parts_of(text: &'static str, word: &str) -> Vec<String> {
        let phrases = [Phrase { para: 1, text }];
        let script = compile(&phrases, &book1_prop4(), Timing::default());
        script
            .lines[0]
            .tokens
            .iter()
            .find(|t| t.text.trim_matches(|c: char| !c.is_ascii_alphabetic()) == word)
            .map(|t| t.parts.clone())
            .unwrap_or_default()
    }

    #[test]
    fn angle_is_not_the_triangle() {
        let bac = parts_of("the angle *BAC*", "BAC");
        assert!(bac.iter().any(|s| s == "ab") && bac.iter().any(|s| s == "ca"));
        assert!(!bac.iter().any(|s| s == "bc"), "∠BAC is not triangle ABC: {bac:?}");
        let abc = parts_of("triangle *ABC*", "ABC");
        assert!(abc.iter().any(|s| s == "bc"), "triangle ABC includes the base: {abc:?}");
    }
}
