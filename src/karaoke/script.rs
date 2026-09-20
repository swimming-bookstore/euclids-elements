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
}

#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub italic: bool,
    pub cite: bool,
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
            push_words(&mut out, &inner, true, map, timing);
            glue_punct(&mut out, &chars, &mut i);
            continue;
        }
        if chars[i].is_whitespace() {
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
            push_words(&mut out, &word, false, map, timing);
        }
    }
    out
}

fn is_punct(c: char) -> bool {
    matches!(c, ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']')
}

fn is_punct_only(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_punct)
}

fn glue_text(out: &mut Vec<Token>, extra: &str) {
    if let Some(last) = out.last_mut() {
        if !last.cite {
            last.text.push_str(extra);
            return;
        }
    }
}

fn glue_punct(out: &mut Vec<Token>, chars: &[char], i: &mut usize) {
    while *i < chars.len() && is_punct(chars[*i]) {
        glue_text(out, &chars[*i].to_string());
        *i += 1;
    }
}

fn push_words(
    out: &mut Vec<Token>,
    chunk: &str,
    italic: bool,
    map: &impl PartsMap,
    timing: &Timing,
) {
    if chunk.is_empty() {
        return;
    }
    for word in chunk.split_whitespace() {
        let parts = if italic {
            map.parts(word)
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
            parts,
            dur_ms: dur,
        });
    }
}
