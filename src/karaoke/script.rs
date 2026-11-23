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
    /// "the opposite (angle) at *C*" (II.10). The single letter is still the
    /// angle at that point; "opposite" names which right-angle, not a new figure.
    fn opposite_angle_parts(&self, word: &str) -> Vec<String> {
        self.angle_parts(word)
    }
    /// `*circle(ABCDE)*` lights the circle, not the pentagon of the same letters.
    fn circle_parts(&self, word: &str) -> Vec<String> {
        self.parts(word)
    }
    /// `*arc(ABCD)*` lights the circumference in letter order, not the closing chord.
    fn arc_parts(&self, word: &str) -> Vec<String> {
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

pub fn compile(phrases: &[crate::content::Phrase], map: &impl PartsMap, timing: Timing) -> Script {
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

/// The stored voice clip is longer than the default beat. Hold the highlight
/// until that clip can finish, so the next word is not already lit.
pub fn hold_for_voice(script: &mut Script) {
    for line in &mut script.lines {
        for tok in &mut line.tokens {
            if tok.cite || tok.dur_ms == 0 {
                continue;
            }
            let bare: String = tok
                .text
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect();
            let need = match bare.as_str() {
                "rectangle" => 920,
                "straightline" => 880,
                "aforementioned" => 900,
                "described" => 800,
                "CDEB" => 2200,
                // Four letters, joined from the E that stays E. The default
                // beat ends before the last letter.
                "CEFB" => 2000,
                // Same four-letter beat. The stored clip is CEFB's E joined to D.
                "CEFD" => 2000,
                _ => 0,
            };
            if need > tok.dur_ms {
                tok.dur_ms = need;
            }
        }
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
    if let Some(name) = triangle_object(text) {
        return name;
    }
    if let Some(name) = parallelogram_object(text) {
        return name;
    }
    if let Some(name) = rectangle_object(text) {
        return name;
    }
    if let Some(name) = gnomon_object(text) {
        return name;
    }
    if let Some(name) = circle_object(text) {
        return name;
    }
    if let Some(name) = arc_object(text) {
        return name;
    }
    if let Some(name) = pentagon_object(text) {
        return name;
    }
    text.to_string()
}

/// `*angle(DCE)*` is an angle object. The letters are the name; the wrapper
/// is the kind. Nearby words cannot turn it back into a triangle.
fn angle_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "angle")
}

/// `*triangle(DBC)*` is a triangle object. The letters are spoken; the
/// wrapper is the kind. "equal to" cannot turn it into an angle (I.39, I.40).
fn triangle_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "triangle")
}

/// `*parallelogram(EH)*` is a parallelogram named by opposite corners.
/// The letters are spoken; the wrapper is the kind. The straight-line
/// between the letters is not the figure (I.43).
fn parallelogram_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "parallelogram")
}

/// `*rectangle(BH)*` is a rectangle named by opposite corners.
/// The letters are spoken; the wrapper is the kind. The straight-line
/// between the letters is not the figure (II.1).
fn rectangle_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "rectangle")
}

/// `*gnomon(NOP)*` is the gnomon named by its three plate letters.
/// The letters are spoken; the wrapper is the kind. "equal to" cannot turn
/// the three letters into an angle, and the straight-lines between them are
/// not the figure (II.5).
fn gnomon_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "gnomon")
}

/// `*circle(ABCDE)*` is the circle, not the pentagon of the same letters (IV.11).
fn circle_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "circle")
}

/// `*arc(ABCD)*` is a circumference in letter order, not the polygon.
fn arc_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "arc")
}

/// `*pentagon(ABCDE)*` is the polygon. The letters are spoken.
fn pentagon_object(piece: &str) -> Option<String> {
    wrapped_letters(piece, "pentagon")
}

fn wrapped_letters(piece: &str, kind: &str) -> Option<String> {
    let t = piece.trim();
    let rest = t.strip_prefix(kind)?.strip_prefix('(')?.strip_suffix(')')?;
    let name: String = rest.chars().filter(|c| c.is_ascii_alphabetic()).collect();
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
    // "the angles at *G* and *H*" — the second letter has only "and" behind
    // it. The angle word sets this; a later noun, or a period, clears it.
    let mut angle_vertex = false;
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
            // Not a citation — those are wrapped in `{…}`. Italic runs inside
            // the brackets are markup, same as the rest of the sentence.
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            let inner: String = chars[start..i].iter().collect();
            if i < chars.len() {
                i += 1;
            }
            if inner.chars().any(|c| c.is_ascii_alphabetic()) {
                let before = out.len();
                let nested = tokenize(&inner, map, timing);
                out.extend(nested);
                if let Some(first) = out.get_mut(before) {
                    first.text.insert(0, '[');
                }
                if let Some(last) = out.iter_mut().rev().find(|t| !t.cite) {
                    last.text.push(']');
                }
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
                // angle(DCE) / triangle(DBC) / parallelogram(EH) is the object.
                // Words around it do not reclassify it.
                let wrapped = angle_object(piece);
                let as_triangle = triangle_object(piece);
                let as_parallelogram = parallelogram_object(piece);
                let as_rectangle = rectangle_object(piece);
                let as_gnomon = gnomon_object(piece);
                let as_circle = circle_object(piece);
                let as_arc = arc_object(piece);
                let as_pentagon = pentagon_object(piece);
                let spoken = wrapped
                    .as_deref()
                    .or(as_triangle.as_deref())
                    .or(as_parallelogram.as_deref())
                    .or(as_rectangle.as_deref())
                    .or(as_gnomon.as_deref())
                    .or(as_circle.as_deref())
                    .or(as_arc.as_deref())
                    .or(as_pentagon.as_deref())
                    .unwrap_or(piece);
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
                let nletters = name.chars().filter(|c| c.is_ascii_alphabetic()).count();
                // "the angles at *G* and *H* double the (angle) at *F*".
                // A single letter is the angle at that point, not the point.
                // "and *H*" carries the list; "at the point *A*" does not.
                let vertex = nletters == 1
                    && (introduced_as_vertex_angle(&before)
                        || (angle_vertex && carry_only(&before)));
                // "the opposite (angle) at *C*" names the other right-angle
                // at that letter. "the (angle) at *C*" does not.
                let opposite = vertex && introduced_as_opposite_angle(&before);
                let angle = as_triangle.is_none()
                    && as_parallelogram.is_none()
                    && as_rectangle.is_none()
                    && as_gnomon.is_none()
                    && as_circle.is_none()
                    && as_arc.is_none()
                    && as_pentagon.is_none()
                    && (wrapped.is_some()
                        || vertex
                        || (nletters == 3
                            && ((n == 0 || carried)
                                && letters_are_angles(
                                    if carried { &[] } else { &before },
                                    &following,
                                )
                                && !introduced_as_figure(&before)
                                && !paired_with_figure(&before, &following)
                                || carried)));
                if vertex {
                    angle_vertex = true;
                }
                let kind = if as_circle.is_some() {
                    FigKind::Circle
                } else if as_arc.is_some() {
                    FigKind::Arc
                } else if opposite {
                    FigKind::OppositeAngle
                } else if angle {
                    FigKind::Angle
                } else {
                    FigKind::Line
                };
                push_words(&mut out, spoken, true, map, timing, kind, &mut pending_open);
                if n + 1 < pieces.len() {
                    recent.clear();
                    angle_vertex = false;
                }
            }
            recent.clear();
            glue_punct(&mut out, &chars, &mut i, &mut pending_open);
            if out
                .last()
                .is_some_and(|t| !t.cite && (t.text.ends_with('.') || t.text.ends_with(';')))
            {
                angle_vertex = false;
            }
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
            while i < chars.len()
                && (opens_word(chars[i]) || chars[i].is_ascii_alphabetic() || chars[i] == '-')
            {
                i += 1;
            }
            if chars[start..i].iter().any(|c| c.is_ascii_alphabetic()) {
                let word: String = chars[start..i].iter().collect();
                recent.push(word.clone());
                note_angle_list(&word, &mut angle_vertex);
                push_words(
                    &mut out,
                    &word,
                    false,
                    map,
                    timing,
                    FigKind::Line,
                    &mut pending_open,
                );
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
                angle_vertex = false;
            }
        } else {
            // "(angle)" is the word "angle" with a parenthesis glued on.
            // Keep it in the introducer list; the parenthesis stays on the
            // following italic run via pending_open.
            let stem_src = word.trim_matches(|c: char| is_punct(c) && !opens_word(c));
            if !stem_src.is_empty() {
                recent.push(stem_src.to_string());
                note_angle_list(stem_src, &mut angle_vertex);
            }
            push_words(
                &mut out,
                &word,
                false,
                map,
                timing,
                FigKind::Line,
                &mut pending_open,
            );
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
/// comparison follows the name, so both letter-runs are angles. A longer
/// name is a figure (*ACDB* the parallelogram, *AB* a side) even when the
/// sentence says "equal".
/// "the angles at *G*" and "the (angle) at *F*" name the angle at that
/// letter. "the angle at (point) *B*" is still that angle. "at the point
/// *A*" is the point: "point" is the introducer, and no angle word precedes it.
fn introduced_as_vertex_angle(recent: &[String]) -> bool {
    for word in recent.iter().rev() {
        let w = stem(word);
        if w.is_empty() || is_list_connector(&w) || is_angle_adjective(&w) {
            continue;
        }
        if w == "point" || w == "points" {
            continue;
        }
        return w == "angle" || w == "angles";
    }
    false
}

/// "the opposite (angle) at *C*" — "opposite" sits with the angle word, so
/// the letter is the other right-angle at that point, not the one "the
/// (angle) at *C*" already named (II.10).
fn introduced_as_opposite_angle(recent: &[String]) -> bool {
    introduced_as_vertex_angle(recent) && recent.iter().any(|word| stem(word) == "opposite")
}

/// Words that may sit between the angle and the next letter of the same list
/// ("angles at *G* and *H*") without naming a new figure.
fn is_list_connector(w: &str) -> bool {
    matches!(
        w,
        "and" | "or" | "at" | "the" | "a" | "an" | "each" | "of" | "both" | "also"
    )
}

/// "*G* and *H*" — nothing but a connector between this letter and the last.
fn carry_only(words: &[String]) -> bool {
    words.iter().all(|w| {
        let s = stem(w);
        s.is_empty() || is_list_connector(&s)
    })
}

/// The angle word opens a vertex list. A later noun closes it, so "the
/// angles at *G* and *H* double the (angle) at *F*" does not carry into a
/// following point.
fn note_angle_list(word: &str, angle_vertex: &mut bool) {
    let w = stem(word);
    if w.is_empty() {
        return;
    }
    if w == "angle" || w == "angles" {
        *angle_vertex = true;
        return;
    }
    if is_list_connector(&w) || is_angle_adjective(&w) {
        return;
    }
    *angle_vertex = false;
}

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

/// "Let *ABC* and *DEF* be triangles" names both runs. The second has only
/// "and" behind it, and "equal to" in front, which would otherwise make DEF
/// an angle. A figure word after that "and" ("be triangles") still names this
/// run. An angle word before any figure word does not.
/// "angle *ABC* equal to *DEF*" has no figure word, so DEF stays the angle.
/// "equal to the given rectilinear angle *DCE*" is not paired with a figure:
/// "angle" comes before any figure word.
fn paired_with_figure(before: &[String], following: &[String]) -> bool {
    before.iter().any(|w| stem(w) == "and")
        && following.iter().any(|w| {
            let s = stem(w);
            s == "triangle" || s == "triangles"
        })
        && !following.iter().any(|w| {
            let s = stem(w);
            let fig = s == "triangle" || s == "triangles";
            !fig && (s.starts_with("angle") || s == "angles")
        })
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
            | "parallelogram"
            | "parallelograms"
            | "complement"
            | "complements"
            | "base"
            | "circle"
            | "circles"
            | "circumference"
            | "circumferences"
            | "pentagon"
            | "pentagons"
            | "straight-line"
            | "straight-lines"
            | "gnomon"
            | "gnomons"
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
        let boundary =
            chars[i] == '.' && (chars[i + 1].is_ascii_uppercase() || chars[i + 1].is_whitespace());
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

/// Which figure a spoken letter-run lights.
#[derive(Clone, Copy)]
enum FigKind {
    Line,
    Angle,
    OppositeAngle,
    Circle,
    Arc,
}

fn push_words(
    out: &mut Vec<Token>,
    chunk: &str,
    italic: bool,
    map: &impl PartsMap,
    timing: &Timing,
    kind: FigKind,
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
        push_one(out, &first, italic, map, timing, kind);
        for word in owned.into_iter().skip(1) {
            push_one(out, word, italic, map, timing, kind);
        }
        return;
    }
    for word in words {
        push_one(out, word, italic, map, timing, kind);
    }
}

fn push_one(
    out: &mut Vec<Token>,
    word: &str,
    italic: bool,
    map: &impl PartsMap,
    timing: &Timing,
    kind: FigKind,
) {
    let bare: String = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let parts = if italic {
        match kind {
            FigKind::Angle => map.angle_parts(&bare),
            FigKind::OppositeAngle => map.opposite_angle_parts(&bare),
            FigKind::Circle => map.circle_parts(&bare),
            FigKind::Arc => map.arc_parts(&bare),
            FigKind::Line => map.parts(&bare),
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
    use crate::figure::{book1_prop21, book1_prop23, book1_prop27, book1_prop32, book1_prop4};

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
        assert!(
            abc.iter().any(|s| s == "bc"),
            "triangle ABC includes the base: {abc:?}"
        );
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
            abe.iter().any(|s| s == "bd")
                && abe.iter().any(|s| s == "ab")
                && abe.iter().any(|s| s == "ae"),
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
            bdc.iter().any(|s| s == "bd")
                && bdc.iter().any(|s| s == "dc")
                && !bdc.iter().any(|s| s == "bc"),
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
        assert!(
            abc.iter().any(|s| s == "bc"),
            "a named triangle stays a triangle: {abc:?}"
        );
    }

    #[test]
    fn parallelogram_acdb_is_the_outline() {
        let phrases = crate::content::get(1, 34).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop34(), Timing::default());
        let acdb = first_parts(&script, "ACDB");
        assert!(
            acdb.iter().any(|s| s == "ab")
                && acdb.iter().any(|s| s == "ac")
                && acdb.iter().any(|s| s == "cd")
                && acdb.iter().any(|s| s == "bd"),
            "parallelogram ACDB lights its outline: {acdb:?}"
        );
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
        // I.38: "Let *ABC* and *DEF* be triangles … equal to triangle *DEF*".
        // "equal to" follows the first DEF, but the sentence already named it
        // a triangle alongside ABC.
        let i38 = compile(
            crate::content::get(1, 38).unwrap().phrases,
            &crate::figure::book1_prop38(),
            Timing::default(),
        );
        let def38 = first_parts(&i38, "DEF");
        assert!(
            def38.iter().any(|s| s == "de" || s == "ed")
                && def38.iter().any(|s| s == "ef" || s == "fe")
                && def38.iter().any(|s| s == "df" || s == "fd"),
            "I.38 DEF is triangle DEF, not ∠DEF: {def38:?}"
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
                && bca[0]
                    .iter()
                    .any(|s| s == "bc" || s == "cb" || s == "bh" || s == "hc")
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
        let say = phrases
            .iter()
            .find(|p| p.text.contains("added to both"))
            .unwrap()
            .text;
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
        for (word, closed) in [
            ("ACB", ["ab", "ba"]),
            ("CBA", ["ac", "ca"]),
            ("CAB", ["bc", "cb"]),
        ] {
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
            // The first ABC in the proof is the triangle. The internal angles
            // are the later angle(…) tokens in this sentence.
            let parts = script
                .lines
                .iter()
                .filter(|l| l.para == 1)
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .last()
                .map(|t| t.parts.clone())
                .unwrap_or_default();
            let rays_hit = rays
                .iter()
                .filter(|s| parts.iter().any(|p| p == *s))
                .count();
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
        assert!(
            !runs[0].iter().any(|s| s == "ac"),
            "first ABC is an angle: {runs:?}"
        );
        assert!(
            runs[1].iter().any(|s| s == "bc"),
            "second ABC is the triangle"
        );
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
                        t.parts
                            .iter()
                            .any(|p| p == "fd" || p == "df" || p == "dg" || p == "gd")
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

    #[test]
    fn i39_dbc_is_the_triangle() {
        // "Let *ABC* and *DBC* be equal triangles" — "equal" follows DBC, but
        // the sentence already named it a triangle alongside ABC.
        let phrases = crate::content::get(1, 39).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop39(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let triangle =
            |parts: &[String], sides: &[&str]| sides.iter().all(|s| parts.iter().any(|p| p == s));
        let dbc = runs("DBC");
        assert!(dbc.len() >= 3, "DBC runs: {dbc:?}");
        for (i, parts) in dbc.iter().enumerate() {
            assert!(
                triangle(parts, &["be", "ed", "dc", "bc"]),
                "DBC run {i} is △DBC, not ∠DBC: {parts:?}"
            );
        }
        let abc = runs("ABC");
        assert!(
            abc.len() >= 2 && triangle(&abc[0], &["ab", "ac", "bc"]),
            "opening ABC is △ABC: {abc:?}"
        );
        assert!(
            triangle(&abc[1], &["ab", "ac", "bc"]),
            "ABC equal to EBC is the triangle: {abc:?}"
        );
        let ebc = runs("EBC");
        assert!(
            ebc.len() >= 2
                && ebc.iter().all(|parts| triangle(parts, &["be", "bc", "ec"])
                    && !parts.iter().any(|p| p == "ab" || p == "ae")),
            "every EBC is △EBC: {ebc:?}"
        );
        assert_eq!(display_letters("triangle(DBC)"), "DBC");
    }

    #[test]
    fn i40_fce_is_the_triangle() {
        // "Let *ABC* and *CDE* be equal triangles" — "equal" follows CDE, but
        // the sentence already named it a triangle alongside ABC. Later,
        // "equal to" must not turn DCE or FCE into angles, and FCE is the
        // triangle on DC, not a walk through A.
        let phrases = crate::content::get(1, 40).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop40(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let triangle =
            |parts: &[String], sides: &[&str]| sides.iter().all(|s| parts.iter().any(|p| p == s));
        let cde = runs("CDE");
        assert!(
            cde.len() >= 1 && triangle(&cde[0], &["df", "fc", "de", "ce"]),
            "opening CDE is △CDE: {cde:?}"
        );
        let fce = runs("FCE");
        assert!(
            fce.len() >= 2
                && fce.iter().all(|parts| triangle(parts, &["fc", "ce", "fe"])
                    && !parts.iter().any(|p| p == "ab" || p == "af" || p == "ac")),
            "every FCE is △FCE: {fce:?}"
        );
        let dce = runs("DCE");
        assert!(
            dce.len() >= 2
                && dce
                    .iter()
                    .all(|parts| triangle(parts, &["df", "fc", "de", "ce"])),
            "every DCE is △DCE, not ∠DCE: {dce:?}"
        );
        let abc = runs("ABC");
        assert!(
            abc.len() >= 3 && abc.iter().all(|parts| triangle(parts, &["ab", "ac", "bc"])),
            "every ABC is △ABC: {abc:?}"
        );
        assert_eq!(display_letters("triangle(FCE)"), "FCE");
        assert_eq!(display_letters("triangle(DCE)"), "DCE");
    }

    #[test]
    fn i41_ebc_is_the_triangle_not_the_angle() {
        // Fitzpatrick writes "(EBC)" for the triangle already named, not an
        // angle. "equal to" must not light ∠EBC, and ABCD stays the
        // parallelogram (not the diagonal, not AE produced).
        let phrases = crate::content::get(1, 41).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop41(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let ebc = runs("EBC");
        assert!(ebc.len() >= 3, "EBC runs: {ebc:?}");
        for (i, parts) in ebc.iter().enumerate() {
            assert!(
                has(parts, "eb") && has(parts, "ec") && has(parts, "bc") && !has(parts, "ab"),
                "EBC run {i} is △EBC, not ∠EBC: {parts:?}"
            );
        }
        let bec = runs("BEC");
        assert!(
            bec.len() >= 1
                && bec
                    .iter()
                    .all(|p| has(p, "eb") && has(p, "ec") && has(p, "bc")),
            "BEC is △BEC: {bec:?}"
        );
        let abc = runs("ABC");
        assert!(
            abc.len() >= 2
                && abc
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "ac") && has(p, "bc")),
            "ABC is △ABC: {abc:?}"
        );
        let abcd = runs("ABCD");
        assert!(
            abcd.len() >= 3
                && abcd.iter().all(|p| {
                    has(p, "ab") && has(p, "bc") && has(p, "dc") && has(p, "ad") && !has(p, "ac")
                }),
            "ABCD is the parallelogram, not the diagonal: {abcd:?}"
        );
        let ae = runs("AE");
        assert!(
            ae.iter().any(|p| has(p, "ad") && has(p, "de")),
            "AE is the whole parallel, through D: {ae:?}"
        );
        assert_eq!(display_letters("triangle(EBC)"), "EBC");
    }

    #[test]
    fn i42_aec_is_the_triangle_and_cef_is_the_angle() {
        // Fitzpatrick writes "(AEC)" for the triangle already named, as "(EBC)"
        // in I.41. "equal to" must not light ∠AEC. ∠CEF is the constructed
        // angle, and D is the given mark, not a point of the figure.
        let phrases = crate::content::get(1, 42).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop42(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let triangle = |parts: &[String], sides: &[&str]| sides.iter().all(|s| has(parts, s));
        let aec = runs("AEC");
        assert!(aec.len() >= 4, "AEC runs: {aec:?}");
        for (i, parts) in aec.iter().enumerate() {
            assert!(
                triangle(parts, &["ae", "ec", "ac"]) && !has(parts, "ab"),
                "AEC run {i} is △AEC, not ∠AEC: {parts:?}"
            );
        }
        let abe = runs("ABE");
        assert!(
            abe.len() >= 1 && abe.iter().all(|p| triangle(p, &["ab", "ae", "be"])),
            "ABE is △ABE: {abe:?}"
        );
        let fecg = runs("FECG");
        assert!(
            fecg.len() >= 4
                && fecg.iter().all(|p| {
                    triangle(p, &["ef", "ec", "cg", "fg"]) && !has(p, "ae") && !has(p, "ac")
                }),
            "FECG is the parallelogram, not the diagonal: {fecg:?}"
        );
        let cef = runs("CEF");
        assert!(
            cef.len() >= 2
                && cef
                    .iter()
                    .all(|p| has(p, "ec") && has(p, "ef") && !has(p, "cf") && !has(p, "ac")),
            "CEF is ∠CEF: {cef:?}"
        );
        let d = runs("D");
        assert!(
            d.len() >= 3 && d.iter().all(|p| has(p, "ang-d")),
            "D is the given angle: {d:?}"
        );
        assert_eq!(display_letters("triangle(AEC)"), "AEC");
    }

    #[test]
    fn i43_complements_are_the_parallelograms() {
        // Fitzpatrick names the parallelograms about the diagonal, and the
        // complements, by opposite corners. "equal to" must not turn KFC or
        // KGC into angles, and EH must not light the straight-line that is
        // not on the plate.
        let phrases = crate::content::get(1, 43).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop43(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let eh = runs("EH");
        assert!(eh.len() >= 2, "EH runs: {eh:?}");
        for (i, parts) in eh.iter().enumerate() {
            assert!(
                has(parts, "ae")
                    && has(parts, "ek")
                    && has(parts, "hk")
                    && has(parts, "ah")
                    && !has(parts, "ak"),
                "EH run {i} is parallelogram EH, not the diagonal: {parts:?}"
            );
        }
        let fg = runs("FG");
        assert!(
            fg.len() >= 1
                && fg.iter().all(|p| has(p, "kf")
                    && has(p, "fc")
                    && has(p, "gc")
                    && has(p, "kg")
                    && !has(p, "kc")),
            "FG is parallelogram FG: {fg:?}"
        );
        let bk = runs("BK");
        assert!(
            bk.len() >= 3
                && bk
                    .iter()
                    .all(|p| has(p, "eb") && has(p, "ek") && has(p, "kg") && has(p, "bg")),
            "BK is complement BK: {bk:?}"
        );
        let kd = runs("KD");
        assert!(
            kd.len() >= 3
                && kd
                    .iter()
                    .all(|p| has(p, "kf") && has(p, "df") && has(p, "hd") && has(p, "hk")),
            "KD is complement KD: {kd:?}"
        );
        let kfc = runs("KFC");
        assert!(
            kfc.len() >= 2
                && kfc
                    .iter()
                    .all(|p| has(p, "kf") && has(p, "fc") && has(p, "kc") && !has(p, "kg")),
            "KFC is triangle KFC, not an angle and not a walk through G: {kfc:?}"
        );
        let kgc = runs("KGC");
        assert!(
            kgc.len() >= 2
                && kgc
                    .iter()
                    .all(|p| has(p, "kg") && has(p, "gc") && has(p, "kc")),
            "KGC is triangle KGC: {kgc:?}"
        );
        let ak = runs("AK");
        assert!(
            ak.len() >= 1 && ak.iter().all(|p| has(p, "ak") && !has(p, "kf")),
            "AK is the diagonal of EH: {ak:?}"
        );
        assert_eq!(display_letters("parallelogram(EH)"), "EH");
        assert_eq!(display_letters("triangle(KGC)"), "KGC");
    }

    #[test]
    fn ii1_rectangles_are_the_boundaries() {
        // Fitzpatrick names the rectangles by opposite corners. "equal to"
        // must not turn BH into the straight-line, and each piece is its own
        // rectangle, not a walk through the next one.
        let phrases = crate::content::get(2, 1).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop1(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let a = runs("A");
        assert!(
            a.len() >= 4
                && a.iter()
                    .all(|p| has(p, "seg-a") && has(p, "A") && !has(p, "bc")),
            "A is the given line: {a:?}"
        );
        let bc = runs("BC");
        assert!(
            bc.len() >= 2
                && bc
                    .iter()
                    .all(|p| has(p, "bd") && has(p, "de") && has(p, "ec")),
            "BC is the whole cut line: {bc:?}"
        );
        let bh = runs("BH");
        assert!(
            bh.len() >= 2
                && bh.iter().all(|p| {
                    has(p, "bd")
                        && has(p, "de")
                        && has(p, "ec")
                        && has(p, "ch")
                        && has(p, "gb")
                        && has(p, "gk")
                        && !has(p, "dk")
                }),
            "BH is the whole rectangle, not a piece: {bh:?}"
        );
        let bk = runs("BK");
        assert!(
            bk.len() >= 2
                && bk.iter().all(|p| has(p, "bd")
                    && has(p, "dk")
                    && has(p, "gk")
                    && has(p, "gb")
                    && !has(p, "de")),
            "BK is that rectangle: {bk:?}"
        );
        let dl = runs("DL");
        assert!(
            dl.len() >= 2
                && dl.iter().all(|p| has(p, "de")
                    && has(p, "el")
                    && has(p, "kl")
                    && has(p, "dk")
                    && !has(p, "ec")),
            "DL is that rectangle: {dl:?}"
        );
        let eh = runs("EH");
        assert!(
            eh.len() >= 2
                && eh.iter().all(|p| has(p, "ec")
                    && has(p, "ch")
                    && has(p, "lh")
                    && has(p, "el")
                    && !has(p, "bd")),
            "EH is that rectangle: {eh:?}"
        );
        assert_eq!(display_letters("rectangle(BH)"), "BH");
    }

    #[test]
    fn ii2_square_and_rectangles_are_the_boundaries() {
        // Fitzpatrick names the square by its four corners, and the pieces by
        // opposite corners. "equal to" must not turn AE into a diagonal, and
        // AF must not walk through the other piece.
        let phrases = crate::content::get(2, 2).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop2(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(
            spoken.contains("straight-line AB"),
            "AB is the given line: {spoken}"
        );
        assert!(spoken.contains("point C"), "C is the cut: {spoken}");
        assert!(
            spoken.contains("square ADEB"),
            "ADEB is the square: {spoken}"
        );
        assert!(spoken.contains("(square) AE"), "AE is the square: {spoken}");
        assert!(
            spoken.contains("(rectangles) AF"),
            "AF is a rectangle: {spoken}"
        );
        assert!(spoken.contains("and CE"), "CE is a rectangle: {spoken}");
        let ab = runs("AB");
        assert!(
            ab.len() >= 4
                && ab
                    .iter()
                    .all(|p| has(p, "ac") && has(p, "cb") && !has(p, "cf")),
            "AB is the whole cut line: {ab:?}"
        );
        let bc = runs("BC");
        assert!(
            bc.len() >= 2 && bc.iter().all(|p| has(p, "cb") && !has(p, "ac")),
            "BC is the piece: {bc:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 2 && ac.iter().all(|p| has(p, "ac") && !has(p, "cb")),
            "AC is the piece: {ac:?}"
        );
        let adeb = runs("ADEB");
        assert!(
            adeb.len() >= 1
                && adeb.iter().all(|p| {
                    has(p, "ad")
                        && has(p, "df")
                        && has(p, "fe")
                        && has(p, "be")
                        && has(p, "ac")
                        && has(p, "cb")
                        && !has(p, "cf")
                }),
            "ADEB is the square, not the cut: {adeb:?}"
        );
        let ae = runs("AE");
        assert!(
            ae.len() >= 2
                && ae.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cb")
                        && has(p, "be")
                        && has(p, "ad")
                        && has(p, "df")
                        && has(p, "fe")
                        && !has(p, "cf")
                }),
            "AE is the square, not a diagonal: {ae:?}"
        );
        let af = runs("AF");
        assert!(
            af.len() >= 2
                && af.iter().all(|p| has(p, "ac")
                    && has(p, "cf")
                    && has(p, "df")
                    && has(p, "ad")
                    && !has(p, "cb")),
            "AF is that rectangle: {af:?}"
        );
        let ce = runs("CE");
        assert!(
            ce.len() >= 2
                && ce.iter().all(|p| has(p, "cb")
                    && has(p, "be")
                    && has(p, "fe")
                    && has(p, "cf")
                    && !has(p, "ac")),
            "CE is that rectangle: {ce:?}"
        );
        let cf = runs("CF");
        assert!(
            cf.len() >= 1 && cf.iter().all(|p| has(p, "cf") && !has(p, "ac")),
            "CF is the parallel: {cf:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 1 && ad.iter().all(|p| has(p, "ad") && !has(p, "df")),
            "AD is the side: {ad:?}"
        );
        let be = runs("BE");
        assert!(
            be.len() >= 1 && be.iter().all(|p| has(p, "be") && !has(p, "fe")),
            "BE is the side: {be:?}"
        );
        assert_eq!(display_letters("rectangle(AE)"), "AE");
        assert_eq!(display_letters("rectangle(AF)"), "AF");
        assert_eq!(display_letters("rectangle(CE)"), "CE");
    }

    #[test]
    fn speak_dump() {
        // SPEAK=1 cargo test --bin euclids-elements speak_dump -- --nocapture --exact
        let book = std::env::var("SPEAK_BOOK")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let prop = std::env::var("SPEAK_PROP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        let Some(p) = crate::content::get(book, prop) else {
            return;
        };
        let mut script = compile(p.phrases, &(p.figure)(), Timing::default());
        crate::karaoke::hold_for_voice(&mut script);
        if std::env::var("SPEAK").is_err() {
            return;
        }
        let mut at = 0u32;
        for line in &script.lines {
            for tok in &line.tokens {
                if tok.cite || tok.dur_ms == 0 {
                    continue;
                }
                println!("{}\t{}\t{}", at, tok.dur_ms, display_letters(&tok.text));
                at += tok.dur_ms;
            }
        }
        eprintln!("TOTAL_MS {at}");
    }

    #[test]
    fn ii3_square_and_rectangles_are_the_boundaries() {
        // Fitzpatrick names the rectangle by opposite corners, and the square
        // on the piece the same way. "equal to" must not turn AE into a
        // diagonal, and AD must not walk through the square.
        let phrases = crate::content::get(2, 3).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop3(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(
            spoken.contains("straight-line AB"),
            "AB is the given line: {spoken}"
        );
        assert!(spoken.contains("(point) C."), "C is the cut: {spoken}");
        assert!(
            spoken.contains("square CDEB"),
            "CDEB is the square: {spoken}"
        );
        assert!(
            spoken.contains("(rectangle) AE"),
            "AE is the rectangle: {spoken}"
        );
        assert!(
            spoken.contains("(rectangle) AD"),
            "AD is a rectangle: {spoken}"
        );
        assert!(spoken.contains("(square) CE"), "CE is the square: {spoken}");
        assert!(
            spoken.contains("square on CB"),
            "DB is spoken as the square on CB: {spoken}"
        );
        let ab = runs("AB");
        assert!(
            ab.len() >= 4
                && ab
                    .iter()
                    .all(|p| has(p, "ac") && has(p, "cb") && !has(p, "af")),
            "AB is the whole cut line: {ab:?}"
        );
        let bc = runs("BC");
        assert!(
            bc.len() >= 2 && bc.iter().all(|p| has(p, "cb") && !has(p, "ac")),
            "BC is the piece: {bc:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 2 && ac.iter().all(|p| has(p, "ac") && !has(p, "cb")),
            "AC is the piece: {ac:?}"
        );
        let cb = runs("CB");
        assert!(
            cb.len() >= 2 && cb.iter().all(|p| has(p, "cb") && !has(p, "ac")),
            "CB is the piece: {cb:?}"
        );
        let cdeb = runs("CDEB");
        assert!(
            cdeb.len() >= 1
                && cdeb.iter().all(|p| has(p, "cd")
                    && has(p, "de")
                    && has(p, "be")
                    && has(p, "cb")
                    && !has(p, "ac")),
            "CDEB is the square, not the production: {cdeb:?}"
        );
        let ae = runs("AE");
        assert!(
            ae.len() >= 2
                && ae.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cb")
                        && has(p, "be")
                        && has(p, "de")
                        && has(p, "fd")
                        && has(p, "af")
                        && !has(p, "cd")
                }),
            "AE is the rectangle, not a diagonal: {ae:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 2
                && ad.iter().all(|p| has(p, "ac")
                    && has(p, "cd")
                    && has(p, "fd")
                    && has(p, "af")
                    && !has(p, "cb")),
            "AD is that rectangle: {ad:?}"
        );
        let ce = runs("CE");
        assert!(
            ce.len() >= 1
                && ce.iter().all(|p| has(p, "cb")
                    && has(p, "be")
                    && has(p, "de")
                    && has(p, "cd")
                    && !has(p, "ac")),
            "CE is the square: {ce:?}"
        );
        let db = runs("DB");
        assert!(
            db.len() >= 1
                && db.iter().all(|p| {
                    has(p, "de") && has(p, "be") && has(p, "cb") && has(p, "cd") && !has(p, "fd")
                }),
            "DB is the square, not the diagonal: {db:?}"
        );
        let af = runs("AF");
        assert!(
            af.len() >= 1 && af.iter().all(|p| has(p, "af") && !has(p, "ac")),
            "AF is the parallel: {af:?}"
        );
        let ed = runs("ED");
        assert!(
            ed.len() >= 1
                && ed
                    .iter()
                    .all(|p| has(p, "de") && !has(p, "fd") && !has(p, "be")),
            "ED is the side drawn through to F: {ed:?}"
        );
        let cd = runs("CD");
        assert!(
            cd.len() >= 1 && cd.iter().all(|p| has(p, "cd") && !has(p, "de")),
            "CD is the side: {cd:?}"
        );
        let be = runs("BE");
        assert!(
            be.len() >= 1 && be.iter().all(|p| has(p, "be") && !has(p, "de")),
            "BE is the side: {be:?}"
        );
        assert_eq!(display_letters("rectangle(AE)"), "AE");
        assert_eq!(display_letters("rectangle(AD)"), "AD");
        assert_eq!(display_letters("rectangle(CE)"), "CE");
    }

    #[test]
    fn ii4_square_and_rectangles_are_the_boundaries() {
        // Fitzpatrick names the square by its four corners, and the pieces by
        // opposite corners. "equal to" must not turn AG into a diagonal, and
        // HF must not walk through the other square.
        let phrases = crate::content::get(2, 4).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop4(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("square ADEB"), "ADEB is the square: {spoken}");
        assert!(spoken.contains("angle CGB"), "CGB is the angle: {spoken}");
        assert!(spoken.contains("(point) C"), "C is the cut: {spoken}");
        assert!(spoken.contains("(rectangle) AG"), "AG is a rectangle: {spoken}");
        assert!(spoken.contains("and GE"), "GE is a rectangle: {spoken}");
        let ab = runs("AB");
        assert!(
            ab.len() >= 4 && ab.iter().all(|p| has(p, "ac") && has(p, "cb") && !has(p, "cg")),
            "AB is the whole cut line: {ab:?}"
        );
        let adeb = runs("ADEB");
        assert!(
            adeb.len() >= 2
                && adeb.iter().all(|p| {
                    has(p, "ah")
                        && has(p, "hd")
                        && has(p, "df")
                        && has(p, "fe")
                        && has(p, "ke")
                        && has(p, "bk")
                        && has(p, "ac")
                        && has(p, "cb")
                        && !has(p, "cg")
                        && !has(p, "bg")
                }),
            "ADEB is the square, not the cuts: {adeb:?}"
        );
        let ag = runs("AG");
        assert!(
            ag.len() >= 2
                && ag.iter().all(|p| has(p, "ac")
                    && has(p, "cg")
                    && has(p, "hg")
                    && has(p, "ah")
                    && !has(p, "cb")
                    && !has(p, "bg")),
            "AG is that rectangle, not the diagonal: {ag:?}"
        );
        let ge = runs("GE");
        assert!(
            ge.len() >= 2
                && ge.iter().all(|p| has(p, "gk")
                    && has(p, "ke")
                    && has(p, "fe")
                    && has(p, "gf")
                    && !has(p, "cb")),
            "GE is that rectangle: {ge:?}"
        );
        let hf = runs("HF");
        assert!(
            hf.len() >= 2
                && hf.iter().all(|p| has(p, "hg")
                    && has(p, "gf")
                    && has(p, "df")
                    && has(p, "hd")
                    && !has(p, "ac")),
            "HF is the square: {hf:?}"
        );
        let cgkb = runs("CGKB");
        assert!(
            cgkb.len() >= 2
                && cgkb.iter().all(|p| has(p, "cb")
                    && has(p, "bk")
                    && has(p, "gk")
                    && has(p, "cg")
                    && !has(p, "ac")),
            "CGKB is the square: {cgkb:?}"
        );
        let cgb = runs("CGB");
        assert!(
            cgb.iter().all(|p| has(p, "cg") && has(p, "bg") && !has(p, "gf") && !has(p, "gd")),
            "angle CGB is the two rays: {cgb:?}"
        );
        assert_eq!(display_letters("rectangle(AG)"), "AG");
        assert_eq!(display_letters("rectangle(GE)"), "GE");
        assert_eq!(display_letters("angle(CGB)"), "CGB");
    }

    #[test]
    fn ii5_rectangles_and_the_gnomon_are_the_boundaries() {
        // Fitzpatrick names the complements and the added rectangles by opposite
        // corners, and the gnomon by its three plate letters. "equal to" must
        // not turn AH into a diagonal, CH into a walk across the square, or
        // NOP into an angle.
        let phrases = crate::content::get(2, 5).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop5(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("cut—equally at C"), "C is the equal cut: {spoken}");
        assert!(spoken.contains("unequally at D"), "D is the unequal cut: {spoken}");
        assert!(spoken.contains("square CEFB"), "CEFB is the square: {spoken}");
        assert!(spoken.contains("complement CH"), "CH is the complement: {spoken}");
        assert!(spoken.contains("complement HF"), "HF is the complement: {spoken}");
        assert!(spoken.contains("(square) DM"), "DM is the square: {spoken}");
        assert!(spoken.contains("gnomon NOP"), "NOP is the gnomon: {spoken}");
        assert!(spoken.contains("(square) LG"), "LG is the square on CD: {spoken}");
        assert!(
            spoken.contains("square on half"),
            "the close names the square on half: {spoken}"
        );
        let ab = runs("AB");
        assert!(
            ab.len() >= 2 && ab.iter().all(|p| has(p, "ac") && has(p, "cd") && has(p, "db") && !has(p, "ak")),
            "AB is the whole cut line: {ab:?}"
        );
        let cb = runs("CB");
        assert!(
            cb.len() >= 2 && cb.iter().all(|p| has(p, "cd") && has(p, "db") && !has(p, "ac")),
            "CB is the half: {cb:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.iter().all(|p| has(p, "ac") && !has(p, "cd")),
            "AC is the other half: {ac:?}"
        );
        let cefb = runs("CEFB");
        assert!(
            cefb.len() >= 2
                && cefb.iter().all(|p| {
                    has(p, "cl")
                        && has(p, "le")
                        && has(p, "eg")
                        && has(p, "gf")
                        && has(p, "mf")
                        && has(p, "bm")
                        && has(p, "db")
                        && has(p, "cd")
                        && !has(p, "bh")
                        && !has(p, "he")
                        && !has(p, "dh")
                        && !has(p, "lh")
                }),
            "CEFB is the square, not the diagonal or the cuts: {cefb:?}"
        );
        let ch = runs("CH");
        assert!(
            ch.len() >= 2
                && ch.iter().all(|p| has(p, "cd") && has(p, "dh") && has(p, "lh") && has(p, "cl") && !has(p, "db") && !has(p, "bh")),
            "CH is the complement, not a diagonal: {ch:?}"
        );
        let hf = runs("HF");
        assert!(
            hf.len() >= 1
                && hf.iter().all(|p| has(p, "hm") && has(p, "mf") && has(p, "gf") && has(p, "hg") && !has(p, "db")),
            "HF is the complement: {hf:?}"
        );
        let dm = runs("DM");
        assert!(
            dm.len() >= 1
                && dm.iter().all(|p| has(p, "db") && has(p, "bm") && has(p, "hm") && has(p, "dh") && !has(p, "mf")),
            "DM is the square on DB: {dm:?}"
        );
        let cm = runs("CM");
        assert!(
            cm.len() >= 2
                && cm.iter().all(|p| {
                    has(p, "cd") && has(p, "db") && has(p, "bm") && has(p, "hm") && has(p, "lh") && has(p, "cl") && !has(p, "le")
                }),
            "CM is the rectangle above KM: {cm:?}"
        );
        let df = runs("DF");
        assert!(
            df.len() >= 2
                && df.iter().all(|p| {
                    has(p, "db") && has(p, "bm") && has(p, "mf") && has(p, "gf") && has(p, "hg") && has(p, "dh") && !has(p, "ac")
                }),
            "DF is that rectangle: {df:?}"
        );
        let al = runs("AL");
        assert!(
            al.len() >= 2
                && al.iter().all(|p| has(p, "ac") && has(p, "cl") && has(p, "kl") && has(p, "ak") && !has(p, "cd")),
            "AL is that rectangle: {al:?}"
        );
        let ah = runs("AH");
        assert!(
            ah.len() >= 2
                && ah.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cd")
                        && has(p, "dh")
                        && has(p, "lh")
                        && has(p, "kl")
                        && has(p, "ak")
                        && !has(p, "db")
                        && !has(p, "bh")
                }),
            "AH is that rectangle, not the diagonal: {ah:?}"
        );
        let lg = runs("LG");
        assert!(
            lg.len() >= 2
                && lg.iter().all(|p| has(p, "lh") && has(p, "hg") && has(p, "eg") && has(p, "le") && !has(p, "cd")),
            "LG is the square on CD: {lg:?}"
        );
        let nop = runs("NOP");
        assert!(
            nop.len() >= 4
                && nop.iter().all(|p| {
                    has(p, "N")
                        && has(p, "O")
                        && has(p, "P")
                        && has(p, "cd")
                        && has(p, "db")
                        && has(p, "bm")
                        && has(p, "mf")
                        && has(p, "gf")
                        && has(p, "hg")
                        && has(p, "lh")
                        && has(p, "cl")
                        && !has(p, "le")
                        && !has(p, "eg")
                        && !has(p, "bh")
                        && !has(p, "he")
                        && !has(p, "ak")
                }),
            "NOP is the gnomon, not the angle or the square taken out: {nop:?}"
        );
        let be = runs("BE");
        assert!(
            be.len() >= 1 && be.iter().all(|p| has(p, "bh") && has(p, "he") && !has(p, "bm")),
            "BE is the diagonal, split at H: {be:?}"
        );
        let ce = runs("CE");
        assert!(
            ce.iter().all(|p| has(p, "cl") && has(p, "le") && !has(p, "cd")),
            "CE is the side: {ce:?}"
        );
        let dg = runs("DG");
        assert!(
            dg.iter().all(|p| has(p, "dh") && has(p, "hg") && !has(p, "db")),
            "DG is the parallel: {dg:?}"
        );
        let km = runs("KM");
        assert!(
            km.iter().all(|p| has(p, "kl") && has(p, "lh") && has(p, "hm") && !has(p, "ak")),
            "KM is the parallel: {km:?}"
        );
        let dh = runs("DH");
        assert!(
            dh.iter().all(|p| has(p, "dh") && !has(p, "hg") && !has(p, "db")),
            "DH is the upright: {dh:?}"
        );
        let db = runs("DB");
        assert!(
            db.len() >= 2 && db.iter().all(|p| has(p, "db") && !has(p, "dh") && !has(p, "cd")),
            "DB is the unequal piece: {db:?}"
        );
        assert_eq!(display_letters("rectangle(AH)"), "AH");
        assert_eq!(display_letters("gnomon(NOP)"), "NOP");
        assert_eq!(display_letters("rectangle(LG)"), "LG");
    }

    #[test]
    fn ii6_rectangles_and_the_gnomon_are_the_boundaries() {
        // Fitzpatrick names the rectangles by opposite corners, and the gnomon
        // by its three plate letters. "equal to" must not turn AM into a
        // diagonal, CH into a walk across the square, or NOP into an angle.
        let phrases = crate::content::get(2, 6).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop6(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("cut in half at point C"), "C is the midpoint: {spoken}");
        assert!(spoken.contains("straight-line BD"), "BD is the added line: {spoken}");
        assert!(spoken.contains("square CEFD"), "CEFD is the square: {spoken}");
        assert!(spoken.contains("(rectangle) AL"), "AL is the rectangle: {spoken}");
        assert!(spoken.contains("(rectangle) CH"), "CH is the rectangle: {spoken}");
        assert!(spoken.contains("(rectangle) HF"), "HF is the complement: {spoken}");
        assert!(spoken.contains("gnomon NOP"), "NOP is the gnomon: {spoken}");
        assert!(spoken.contains("(square) LG"), "LG is the square on BC: {spoken}");
        assert!(
            spoken.contains("having been added"),
            "the close keeps Fitzpatrick's having been added: {spoken}"
        );
        let ab = runs("AB");
        assert!(
            ab.len() >= 2 && ab.iter().all(|p| has(p, "ac") && has(p, "cb") && !has(p, "bd") && !has(p, "ak")),
            "AB is the original line, not the added piece: {ab:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 2 && ad.iter().all(|p| has(p, "ac") && has(p, "cb") && has(p, "bd") && !has(p, "ak")),
            "AD is the whole with the added line: {ad:?}"
        );
        let bd = runs("BD");
        assert!(
            bd.len() >= 1 && bd.iter().all(|p| has(p, "bd") && !has(p, "cb") && !has(p, "dm")),
            "BD is the added line: {bd:?}"
        );
        let cb = runs("CB");
        assert!(
            cb.len() >= 2 && cb.iter().all(|p| has(p, "cb") && !has(p, "ac") && !has(p, "bd")),
            "CB is the half: {cb:?}"
        );
        let cd = runs("CD");
        assert!(
            cd.len() >= 2 && cd.iter().all(|p| has(p, "cb") && has(p, "bd") && !has(p, "ac")),
            "CD is the half plus the added line: {cd:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.iter().all(|p| has(p, "ac") && !has(p, "cb")),
            "AC is the other half: {ac:?}"
        );
        let cefd = runs("CEFD");
        assert!(
            cefd.len() >= 2
                && cefd.iter().all(|p| {
                    has(p, "cl")
                        && has(p, "le")
                        && has(p, "eg")
                        && has(p, "gf")
                        && has(p, "mf")
                        && has(p, "dm")
                        && has(p, "bd")
                        && has(p, "cb")
                        && !has(p, "bh")
                        && !has(p, "hg")
                        && !has(p, "dh")
                        && !has(p, "he")
                        && !has(p, "lh")
                }),
            "CEFD is the square, not the diagonal or the cuts: {cefd:?}"
        );
        let al = runs("AL");
        assert!(
            al.len() >= 2
                && al.iter().all(|p| has(p, "ac") && has(p, "cl") && has(p, "kl") && has(p, "ak") && !has(p, "cb")),
            "AL is that rectangle: {al:?}"
        );
        let ch = runs("CH");
        assert!(
            ch.len() >= 2
                && ch.iter().all(|p| has(p, "cb") && has(p, "bh") && has(p, "lh") && has(p, "cl") && !has(p, "bd") && !has(p, "dh")),
            "CH is that rectangle, not a diagonal: {ch:?}"
        );
        let hf = runs("HF");
        assert!(
            hf.len() >= 2
                && hf.iter().all(|p| has(p, "hm") && has(p, "mf") && has(p, "gf") && has(p, "hg") && !has(p, "bd")),
            "HF is the complement: {hf:?}"
        );
        let cm = runs("CM");
        assert!(
            cm.len() >= 1
                && cm.iter().all(|p| {
                    has(p, "cb") && has(p, "bd") && has(p, "dm") && has(p, "hm") && has(p, "lh") && has(p, "cl") && !has(p, "le")
                }),
            "CM is the rectangle above KM: {cm:?}"
        );
        let am = runs("AM");
        assert!(
            am.len() >= 2
                && am.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cb")
                        && has(p, "bd")
                        && has(p, "dm")
                        && has(p, "hm")
                        && has(p, "kl")
                        && has(p, "ak")
                        && !has(p, "le")
                        && !has(p, "dh")
                        && !has(p, "he")
                }),
            "AM is that rectangle, not the diagonal: {am:?}"
        );
        let lg = runs("LG");
        assert!(
            lg.len() >= 2
                && lg.iter().all(|p| has(p, "lh") && has(p, "hg") && has(p, "eg") && has(p, "le") && !has(p, "cb")),
            "LG is the square on BC: {lg:?}"
        );
        let nop = runs("NOP");
        assert!(
            nop.len() >= 4
                && nop.iter().all(|p| {
                    has(p, "N")
                        && has(p, "O")
                        && has(p, "P")
                        && has(p, "cb")
                        && has(p, "bd")
                        && has(p, "dm")
                        && has(p, "mf")
                        && has(p, "gf")
                        && has(p, "hg")
                        && has(p, "lh")
                        && has(p, "cl")
                        && !has(p, "le")
                        && !has(p, "eg")
                        && !has(p, "bh")
                        && !has(p, "he")
                        && !has(p, "ak")
                }),
            "NOP is the gnomon, not the angle or the square taken out: {nop:?}"
        );
        let de = runs("DE");
        assert!(
            de.len() >= 1 && de.iter().all(|p| has(p, "dh") && has(p, "he") && !has(p, "dm")),
            "DE is the diagonal, split at H: {de:?}"
        );
        let bg = runs("BG");
        assert!(
            bg.iter().all(|p| has(p, "bh") && has(p, "hg") && !has(p, "bd")),
            "BG is the parallel: {bg:?}"
        );
        let km = runs("KM");
        assert!(
            km.iter().all(|p| has(p, "kl") && has(p, "lh") && has(p, "hm") && !has(p, "ak")),
            "KM is the parallel: {km:?}"
        );
        let ec = runs("EC");
        assert!(
            ec.iter().all(|p| has(p, "cl") && has(p, "le") && !has(p, "cb")),
            "EC is the side: {ec:?}"
        );
        let df = runs("DF");
        assert!(
            df.iter().all(|p| has(p, "dm") && has(p, "mf") && !has(p, "bd")),
            "DF is the side: {df:?}"
        );
        let dm = runs("DM");
        assert!(
            dm.len() >= 2 && dm.iter().all(|p| has(p, "dm") && !has(p, "mf") && !has(p, "bd")),
            "DM is the upright, equal to DB: {dm:?}"
        );
        let db = runs("DB");
        assert!(
            db.len() >= 2 && db.iter().all(|p| has(p, "bd") && !has(p, "dm") && !has(p, "cb")),
            "DB is the added line: {db:?}"
        );
        let bc = runs("BC");
        assert!(
            bc.len() >= 1 && bc.iter().all(|p| has(p, "cb") && !has(p, "ac") && !has(p, "bd")),
            "BC is the half the square LG stands on: {bc:?}"
        );
        assert_eq!(display_letters("rectangle(AM)"), "AM");
        assert_eq!(display_letters("gnomon(NOP)"), "NOP");
        assert_eq!(display_letters("rectangle(LG)"), "LG");
    }

    #[test]
    fn ii7_rectangles_and_the_gnomon_are_the_boundaries() {
        // Fitzpatrick names the complements and the added rectangles by opposite
        // corners, and the gnomon by its three plate letters. "equal to" must
        // not turn AF into a diagonal, CF into a walk across the square, or
        // KLM into an angle. N is the foot of the upright, not a gnomon letter.
        let phrases = crate::content::get(2, 7).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop7(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("at point C"), "C is the cut: {spoken}");
        assert!(spoken.contains("square ADEB"), "ADEB is the square: {spoken}");
        assert!(spoken.contains("(rectangle) AG"), "AG is the complement: {spoken}");
        assert!(spoken.contains("(rectangle) GE"), "GE is the complement: {spoken}");
        assert!(spoken.contains("(square) CF"), "CF is the square on BC: {spoken}");
        assert!(spoken.contains("gnomon KLM"), "KLM is the gnomon: {spoken}");
        assert!(spoken.contains("square on AC"), "DG is the square on AC: {spoken}");
        assert!(
            spoken.contains("squares on AB and BC"),
            "the close names both squares: {spoken}"
        );
        let ab = runs("AB");
        assert!(
            ab.len() >= 2 && ab.iter().all(|p| has(p, "ac") && has(p, "cb") && !has(p, "ah")),
            "AB is the whole cut line: {ab:?}"
        );
        let bc = runs("BC");
        assert!(
            bc.len() >= 2 && bc.iter().all(|p| has(p, "cb") && !has(p, "ac") && !has(p, "bf")),
            "BC is the piece: {bc:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 1 && ac.iter().all(|p| has(p, "ac") && !has(p, "cb")),
            "AC is the remaining piece: {ac:?}"
        );
        let ca = runs("CA");
        assert!(
            ca.iter().all(|p| has(p, "ac") && !has(p, "cb")),
            "CA is that same piece: {ca:?}"
        );
        let adeb = runs("ADEB");
        assert!(
            adeb.len() >= 2
                && adeb.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cb")
                        && has(p, "bf")
                        && has(p, "fe")
                        && has(p, "ne")
                        && has(p, "dn")
                        && has(p, "hd")
                        && has(p, "ah")
                        && !has(p, "bg")
                        && !has(p, "gd")
                        && !has(p, "cg")
                        && !has(p, "gn")
                        && !has(p, "hg")
                        && !has(p, "gf")
                }),
            "ADEB is the square, not the diagonal or the cuts: {adeb:?}"
        );
        let ag = runs("AG");
        assert!(
            ag.len() >= 1
                && ag.iter().all(|p| has(p, "ac") && has(p, "cg") && has(p, "hg") && has(p, "ah") && !has(p, "cb") && !has(p, "bg")),
            "AG is the complement, not a diagonal: {ag:?}"
        );
        let ge = runs("GE");
        assert!(
            ge.len() >= 1
                && ge.iter().all(|p| has(p, "gf") && has(p, "fe") && has(p, "ne") && has(p, "gn") && !has(p, "cb")),
            "GE is the complement: {ge:?}"
        );
        let cf = runs("CF");
        assert!(
            cf.len() >= 4
                && cf.iter().all(|p| has(p, "cb") && has(p, "bf") && has(p, "gf") && has(p, "cg") && !has(p, "ac") && !has(p, "bg")),
            "CF is the square on BC, not the diagonal: {cf:?}"
        );
        let dg = runs("DG");
        assert!(
            dg.len() >= 1
                && dg.iter().all(|p| has(p, "dn") && has(p, "gn") && has(p, "hg") && has(p, "hd") && !has(p, "gd") && !has(p, "cb")),
            "DG is the square on AC, not the diagonal: {dg:?}"
        );
        let af = runs("AF");
        assert!(
            af.len() >= 2
                && af.iter().all(|p| {
                    has(p, "ac") && has(p, "cb") && has(p, "bf") && has(p, "gf") && has(p, "hg") && has(p, "ah") && !has(p, "fe") && !has(p, "bg")
                }),
            "AF is that rectangle, not the diagonal: {af:?}"
        );
        let ce = runs("CE");
        assert!(
            ce.len() >= 2
                && ce.iter().all(|p| {
                    has(p, "cb") && has(p, "bf") && has(p, "fe") && has(p, "ne") && has(p, "gn") && has(p, "cg") && !has(p, "ac") && !has(p, "bg")
                }),
            "CE is that rectangle: {ce:?}"
        );
        let bg = runs("BG");
        assert!(
            bg.len() >= 2
                && bg.iter().all(|p| has(p, "bf") && has(p, "gf") && has(p, "cg") && has(p, "cb") && !has(p, "bg") && !has(p, "ac")),
            "BG is the square on BC, not the diagonal: {bg:?}"
        );
        let gd = runs("GD");
        assert!(
            gd.len() >= 2
                && gd.iter().all(|p| has(p, "gn") && has(p, "dn") && has(p, "hd") && has(p, "hg") && !has(p, "gd") && !has(p, "cb")),
            "GD is the square on AC, not the diagonal: {gd:?}"
        );
        let klm = runs("KLM");
        assert!(
            klm.len() >= 4
                && klm.iter().all(|p| {
                    has(p, "K")
                        && has(p, "L")
                        && has(p, "M")
                        && has(p, "ac")
                        && has(p, "cb")
                        && has(p, "bf")
                        && has(p, "fe")
                        && has(p, "ne")
                        && has(p, "gn")
                        && has(p, "hg")
                        && has(p, "ah")
                        && !has(p, "hd")
                        && !has(p, "dn")
                        && !has(p, "bg")
                        && !has(p, "gd")
                        && !has(p, "cg")
                }),
            "KLM is the gnomon, not the angle or the square taken out: {klm:?}"
        );
        let bd = runs("BD");
        assert!(bd.is_empty(), "the proof does not speak BD: {bd:?}");
        let bf = runs("BF");
        assert!(
            bf.len() >= 1 && bf.iter().all(|p| has(p, "bf") && !has(p, "fe") && !has(p, "cb")),
            "BF is the upright, equal to BC: {bf:?}"
        );
        assert_eq!(display_letters("rectangle(AF)"), "AF");
        assert_eq!(display_letters("gnomon(KLM)"), "KLM");
        assert_eq!(display_letters("rectangle(CF)"), "CF");
    }

    #[test]
    fn ii8_squares_rectangles_and_the_gnomon_are_the_boundaries() {
        // Fitzpatrick names the squares and rectangles by opposite corners, the
        // parallelograms the same way, and the gnomon by its three plate letters.
        // "equal to" must not turn CK into a diagonal, AK into a walk across the
        // square, or STU into an angle. OH is the square taken out of the gnomon.
        // ED is the plate's diagonal; the proof does not speak it.
        let phrases = crate::content::get(2, 8).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop8(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("at point C"), "C is the cut: {spoken}");
        assert!(spoken.contains("square AEFD"), "AEFD is the square: {spoken}");
        assert!(spoken.contains("parallelogram CP"), "CP is the parallelogram: {spoken}");
        assert!(spoken.contains("parallelogram ML"), "ML is the parallelogram: {spoken}");
        assert!(spoken.contains("gnomon STU"), "STU is the gnomon: {spoken}");
        assert!(spoken.contains("(square) OH"), "OH is the square on AC: {spoken}");
        assert!(spoken.contains("square on AD"), "the close names the square on AD: {spoken}");
        assert!(
            spoken.contains("square described on AB and BC"),
            "the close names the square on the sum: {spoken}"
        );
        let ab = runs("AB");
        assert!(
            ab.len() >= 2 && ab.iter().all(|p| has(p, "ac") && has(p, "cb") && !has(p, "bd") && !has(p, "ed")),
            "AB is the cut line, not the produced piece or the diagonal: {ab:?}"
        );
        let bc = runs("BC");
        assert!(
            bc.len() >= 2 && bc.iter().all(|p| has(p, "cb") && !has(p, "ac") && !has(p, "bd") && !has(p, "bk")),
            "BC is the piece: {bc:?}"
        );
        let cb = runs("CB");
        assert!(
            cb.iter().all(|p| has(p, "cb") && !has(p, "ac") && !has(p, "bd")),
            "CB is that same piece: {cb:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 1 && ac.iter().all(|p| has(p, "ac") && !has(p, "cb")),
            "AC is the remaining piece: {ac:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 1
                && ad.iter().all(|p| has(p, "ac") && has(p, "cb") && has(p, "bd") && !has(p, "ed") && !has(p, "dn")),
            "AD is the whole top, not the diagonal: {ad:?}"
        );
        let bd = runs("BD");
        assert!(
            bd.len() >= 2 && bd.iter().all(|p| has(p, "bd") && !has(p, "cb") && !has(p, "ed")),
            "BD is the produced line: {bd:?}"
        );
        let gk = runs("GK");
        assert!(
            gk.len() >= 2 && gk.iter().all(|p| has(p, "gk") && !has(p, "cg") && !has(p, "kn") && !has(p, "ed")),
            "GK is the side of the square on CB: {gk:?}"
        );
        let kn = runs("KN");
        assert!(
            kn.len() >= 1 && kn.iter().all(|p| has(p, "kn") && !has(p, "gk") && !has(p, "dn")),
            "KN is the side equal to BD: {kn:?}"
        );
        let bk = runs("BK");
        assert!(
            bk.len() >= 1 && bk.iter().all(|p| has(p, "bk") && !has(p, "kr") && !has(p, "cb") && !has(p, "ed")),
            "BK is the upright equal to BD: {bk:?}"
        );
        let cg = runs("CG");
        assert!(
            cg.len() >= 1 && cg.iter().all(|p| has(p, "cg") && !has(p, "gq") && !has(p, "ac") && !has(p, "ed")),
            "CG is the upright, not the column: {cg:?}"
        );
        let gq = runs("GQ");
        assert!(
            gq.len() >= 1 && gq.iter().all(|p| has(p, "gq") && !has(p, "cg") && !has(p, "qh")),
            "GQ is the upright equal to CB: {gq:?}"
        );
        let qr = runs("QR");
        assert!(
            qr.len() >= 1 && qr.iter().all(|p| has(p, "qr") && !has(p, "oq") && !has(p, "rp")),
            "QR is the side: {qr:?}"
        );
        let rp = runs("RP");
        assert!(
            rp.len() >= 1 && rp.iter().all(|p| has(p, "rp") && !has(p, "qr") && !has(p, "pf")),
            "RP is the side equal to QR: {rp:?}"
        );
        let ck = runs("CK");
        assert!(
            ck.len() >= 4
                && ck.iter().all(|p| {
                    has(p, "cb") && has(p, "bk") && has(p, "gk") && has(p, "cg") && !has(p, "ed") && !has(p, "ac") && !has(p, "kr")
                }),
            "CK is the square on BC, not the diagonal: {ck:?}"
        );
        let kd = runs("KD");
        assert!(
            kd.len() >= 2
                && kd.iter().all(|p| {
                    has(p, "bk") && has(p, "bd") && has(p, "dn") && has(p, "kn") && !has(p, "ed") && !has(p, "kr")
                }),
            "KD is that square, not the diagonal: {kd:?}"
        );
        let dk = runs("DK");
        assert!(
            dk.len() >= 1
                && dk.iter().all(|p| {
                    has(p, "bk") && has(p, "bd") && has(p, "dn") && has(p, "kn") && !has(p, "ed")
                }),
            "DK is the same square: {dk:?}"
        );
        let gr = runs("GR");
        assert!(
            gr.len() >= 2
                && gr.iter().all(|p| {
                    has(p, "gk") && has(p, "kr") && has(p, "qr") && has(p, "gq") && !has(p, "ed") && !has(p, "cb")
                }),
            "GR is that square, not the diagonal: {gr:?}"
        );
        let rn = runs("RN");
        assert!(
            rn.len() >= 2
                && rn.iter().all(|p| {
                    has(p, "kr") && has(p, "kn") && has(p, "np") && has(p, "rp") && !has(p, "ed") && !has(p, "pf")
                }),
            "RN is that square: {rn:?}"
        );
        let ag = runs("AG");
        assert!(
            ag.len() >= 2
                && ag.iter().all(|p| {
                    has(p, "ac") && has(p, "cg") && has(p, "mg") && has(p, "am") && !has(p, "cb") && !has(p, "ed")
                }),
            "AG is the rectangle, not a diagonal: {ag:?}"
        );
        let mq = runs("MQ");
        assert!(
            mq.len() >= 2
                && mq.iter().all(|p| {
                    has(p, "mg") && has(p, "gq") && has(p, "oq") && has(p, "mo") && !has(p, "gk") && !has(p, "ed")
                }),
            "MQ is that rectangle: {mq:?}"
        );
        let ql = runs("QL");
        assert!(
            ql.len() >= 2
                && ql.iter().all(|p| {
                    has(p, "qr") && has(p, "rl") && has(p, "hl") && has(p, "qh") && !has(p, "oq") && !has(p, "ed")
                }),
            "QL is that rectangle: {ql:?}"
        );
        let rf = runs("RF");
        assert!(
            rf.len() >= 2
                && rf.iter().all(|p| {
                    has(p, "rp") && has(p, "pf") && has(p, "lf") && has(p, "rl") && !has(p, "qr") && !has(p, "ed")
                }),
            "RF is that rectangle: {rf:?}"
        );
        let ak = runs("AK");
        assert!(
            ak.len() >= 2
                && ak.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cb")
                        && has(p, "bk")
                        && has(p, "gk")
                        && has(p, "mg")
                        && has(p, "am")
                        && !has(p, "cg")
                        && !has(p, "ed")
                        && !has(p, "bd")
                }),
            "AK is the rectangle contained by AB and BD, not a diagonal: {ak:?}"
        );
        let oh = runs("OH");
        assert!(
            oh.len() >= 2
                && oh.iter().all(|p| {
                    has(p, "oq") && has(p, "qh") && has(p, "eh") && has(p, "oe") && !has(p, "ed") && !has(p, "am") && !has(p, "cg")
                }),
            "OH is the square equal to the square on AC, not a diagonal: {oh:?}"
        );
        let cp = runs("CP");
        assert!(
            cp.len() >= 1
                && cp.iter().all(|p| {
                    has(p, "cb")
                        && has(p, "bd")
                        && has(p, "dn")
                        && has(p, "np")
                        && has(p, "rp")
                        && has(p, "qr")
                        && has(p, "gq")
                        && has(p, "cg")
                        && !has(p, "ed")
                        && !has(p, "am")
                        && !has(p, "eh")
                }),
            "CP is the parallelogram, not the diagonal: {cp:?}"
        );
        let ml = runs("ML");
        assert!(
            ml.len() >= 1
                && ml.iter().all(|p| {
                    has(p, "mg")
                        && has(p, "gk")
                        && has(p, "kr")
                        && has(p, "rl")
                        && has(p, "hl")
                        && has(p, "eh")
                        && has(p, "oe")
                        && has(p, "mo")
                        && !has(p, "ed")
                        && !has(p, "ac")
                        && !has(p, "bd")
                }),
            "ML is the parallelogram, not the diagonal: {ml:?}"
        );
        let aefd = runs("AEFD");
        assert!(
            aefd.len() >= 2
                && aefd.iter().all(|p| {
                    has(p, "ac")
                        && has(p, "cb")
                        && has(p, "bd")
                        && has(p, "dn")
                        && has(p, "np")
                        && has(p, "pf")
                        && has(p, "lf")
                        && has(p, "hl")
                        && has(p, "eh")
                        && has(p, "oe")
                        && has(p, "mo")
                        && has(p, "am")
                        && !has(p, "ed")
                        && !has(p, "cg")
                        && !has(p, "gq")
                        && !has(p, "qh")
                        && !has(p, "bk")
                        && !has(p, "gk")
                }),
            "AEFD is the square, not the diagonal or the cuts: {aefd:?}"
        );
        let stu = runs("STU");
        assert!(
            stu.len() >= 4
                && stu.iter().all(|p| {
                    has(p, "S")
                        && has(p, "T")
                        && has(p, "U")
                        && has(p, "ac")
                        && has(p, "cb")
                        && has(p, "bd")
                        && has(p, "dn")
                        && has(p, "np")
                        && has(p, "pf")
                        && has(p, "lf")
                        && has(p, "hl")
                        && has(p, "qh")
                        && has(p, "oq")
                        && has(p, "mo")
                        && has(p, "am")
                        && !has(p, "eh")
                        && !has(p, "oe")
                        && !has(p, "ed")
                        && !has(p, "cg")
                        && !has(p, "gq")
                }),
            "STU is the gnomon, not the angle or the square taken out: {stu:?}"
        );
        let ed = runs("ED");
        assert!(ed.is_empty(), "the proof does not speak the diagonal: {ed:?}");
        assert_eq!(display_letters("rectangle(CK)"), "CK");
        assert_eq!(display_letters("gnomon(STU)"), "STU");
        assert_eq!(display_letters("parallelogram(CP)"), "CP");
        assert_eq!(display_letters("rectangle(OH)"), "OH");
    }

    #[test]
    fn ii9_angles_and_sides_are_the_plate() {
        // Fitzpatrick names the angles, and the sides the squares stand on.
        // The angle at C is the right-angle ACE, not the whole pencil, and not G.
        // The angle at D is the right-angle FDB. A triangle name is the outline,
        // not the angle of the same letters. The plate draws no square.
        let phrases = crate::content::get(2, 9).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop9(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("equally at C"), "C is the equal cut: {spoken}");
        assert!(spoken.contains("unequally at D"), "D is the unequal cut: {spoken}");
        assert!(spoken.contains("angle EAC"), "EAC is an angle: {spoken}");
        assert!(spoken.contains("triangle AEC"), "AEC is the triangle: {spoken}");
        assert!(spoken.contains("(angle) AEB"), "AEB is the right-angle: {spoken}");
        assert!(spoken.contains("(angle) at C"), "the right-angle is at C: {spoken}");
        assert!(spoken.contains("angle at B"), "the half right-angle is at B: {spoken}");
        assert!(spoken.contains("angle at D"), "the right-angle is at D: {spoken}");
        assert!(spoken.contains("squares on AD and DB"), "the close names both squares: {spoken}");
        let ab = runs("AB");
        assert!(
            ab.len() >= 1 && ab.iter().all(|p| has(p, "ac") && has(p, "cd") && has(p, "db") && !has(p, "ea")),
            "AB is the whole cut line: {ab:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 2 && ac.iter().all(|p| has(p, "ac") && !has(p, "cd") && !has(p, "ea")),
            "AC is the half: {ac:?}"
        );
        let cb = runs("CB");
        assert!(
            cb.iter().all(|p| has(p, "cd") && has(p, "db") && !has(p, "ac")),
            "CB is the other half: {cb:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 2 && ad.iter().all(|p| has(p, "ac") && has(p, "cd") && !has(p, "db") && !has(p, "af")),
            "AD is the unequal piece, not the join: {ad:?}"
        );
        let db = runs("DB");
        assert!(
            db.len() >= 2 && db.iter().all(|p| has(p, "db") && !has(p, "cd") && !has(p, "df")),
            "DB is the other unequal piece: {db:?}"
        );
        let cd = runs("CD");
        assert!(
            cd.len() >= 2 && cd.iter().all(|p| has(p, "cd") && !has(p, "ac") && !has(p, "db")),
            "CD is the difference: {cd:?}"
        );
        let ce = runs("CE");
        assert!(
            ce.len() >= 2 && ce.iter().all(|p| has(p, "cg") && has(p, "ge") && !has(p, "ac")),
            "CE is the upright: {ce:?}"
        );
        let ea = runs("EA");
        assert!(
            ea.len() >= 2 && ea.iter().all(|p| has(p, "ea") && !has(p, "eb") && !has(p, "af")),
            "EA is the join: {ea:?}"
        );
        let eb = runs("EB");
        assert!(
            eb.len() >= 1 && eb.iter().all(|p| has(p, "ef") && has(p, "fb") && !has(p, "ea") && !has(p, "df")),
            "EB is the join, split at F: {eb:?}"
        );
        let df = runs("DF");
        assert!(
            df.len() >= 2 && df.iter().all(|p| has(p, "df") && !has(p, "db") && !has(p, "gf")),
            "DF is the parallel: {df:?}"
        );
        let fg = runs("FG");
        assert!(
            fg.iter().all(|p| has(p, "gf") && !has(p, "df") && !has(p, "ge")),
            "FG is the parallel: {fg:?}"
        );
        let af = runs("AF");
        assert!(
            af.len() >= 2 && af.iter().all(|p| has(p, "af") && !has(p, "ac") && !has(p, "df")),
            "AF is the join: {af:?}"
        );
        let eg = runs("EG");
        assert!(
            eg.len() >= 2 && eg.iter().all(|p| has(p, "ge") && !has(p, "cg") && !has(p, "gf")),
            "EG is the piece of the upright: {eg:?}"
        );
        let gf = runs("GF");
        assert!(
            gf.len() >= 2 && gf.iter().all(|p| has(p, "gf") && !has(p, "ge") && !has(p, "df")),
            "GF is the parallel, equal to CD: {gf:?}"
        );
        let ef = runs("EF");
        assert!(
            ef.len() >= 2 && ef.iter().all(|p| has(p, "ef") && !has(p, "fb") && !has(p, "ea") && !has(p, "df") && !has(p, "ge")),
            "EF is the piece of EB: {ef:?}"
        );
        let eac = runs("EAC");
        assert!(
            eac.len() >= 2
                && eac.iter().all(|p| has(p, "ea") && has(p, "ac") && !has(p, "af") && !has(p, "cg")),
            "EAC is the angle, not the triangle: {eac:?}"
        );
        let aec = runs("AEC");
        assert!(
            aec.iter().any(|p| has(p, "ea") && has(p, "ac") && has(p, "cg") && has(p, "ge")),
            "triangle AEC is the outline: {aec:?}"
        );
        assert!(
            aec.iter().any(|p| has(p, "ea") && has(p, "ge") && !has(p, "ac")),
            "angle AEC is the angle, not the triangle: {aec:?}"
        );
        let at_c = runs("C");
        assert!(
            at_c.iter().any(|p| has(p, "ac") && has(p, "cg") && has(p, "ge") && has(p, "A") && has(p, "E") && !has(p, "G") && !has(p, "cd") && !has(p, "ea")),
            "the angle at C is ACE, through to E, without lighting G: {at_c:?}"
        );
        let aeb = runs("AEB");
        assert!(
            aeb.len() >= 1
                && aeb.iter().all(|p| has(p, "ea") && has(p, "ef") && has(p, "fb") && !has(p, "af") && !has(p, "ge")),
            "AEB is the right-angle: {aeb:?}"
        );
        let gef = runs("GEF");
        assert!(
            gef.len() >= 2
                && gef.iter().all(|p| has(p, "ge") && has(p, "ef") && !has(p, "ea") && !has(p, "gf")),
            "GEF is the half right-angle: {gef:?}"
        );
        let egf = runs("EGF");
        assert!(
            egf.len() >= 1
                && egf.iter().all(|p| has(p, "ge") && has(p, "gf") && !has(p, "cg") && !has(p, "ef")),
            "EGF is the right-angle: {egf:?}"
        );
        let efg = runs("EFG");
        assert!(
            efg.len() >= 1
                && efg.iter().all(|p| has(p, "ef") && has(p, "gf") && !has(p, "df") && !has(p, "ge")),
            "EFG is the angle, not a walk: {efg:?}"
        );
        let ecb = runs("ECB");
        assert!(
            ecb.len() >= 1
                && ecb.iter().all(|p| has(p, "cg") && has(p, "ge") && has(p, "cd") && has(p, "db") && !has(p, "ea")),
            "ECB is the angle: {ecb:?}"
        );
        let fdb = runs("FDB");
        assert!(
            fdb.len() >= 1
                && fdb.iter().all(|p| has(p, "df") && has(p, "db") && !has(p, "cd") && !has(p, "ef")),
            "FDB is the right-angle: {fdb:?}"
        );
        let at_d = runs("D");
        assert!(
            at_d.iter().any(|p| has(p, "cd") && has(p, "df") && !has(p, "db") && !has(p, "af")),
            "the angle at D is ADF, the right-angle the squares stand on: {at_d:?}"
        );
        let bfd = runs("BFD");
        assert!(
            bfd.len() >= 1
                && bfd.iter().all(|p| has(p, "fb") && has(p, "df") && !has(p, "ef") && !has(p, "gf") && !has(p, "db")),
            "BFD is the angle: {bfd:?}"
        );
        let at_b = runs("B");
        assert!(
            at_b.iter().any(|p| has(p, "fb") && has(p, "db") && !has(p, "df")),
            "the angle at B is the half right-angle EBC: {at_b:?}"
        );
        let ace = runs("ACE");
        assert!(
            ace.len() >= 1
                && ace.iter().all(|p| has(p, "ac") && has(p, "cg") && has(p, "ge") && !has(p, "cd") && !has(p, "ea")),
            "ACE is the right-angle, through to E: {ace:?}"
        );
        let aef = runs("AEF");
        assert!(
            aef.len() >= 1
                && aef.iter().all(|p| has(p, "ea") && has(p, "ef") && !has(p, "af") && !has(p, "ge")),
            "AEF is the right-angle AEB: {aef:?}"
        );
        assert_eq!(display_letters("angle(EAC)"), "EAC");
        assert_eq!(display_letters("triangle(AEC)"), "AEC");
        assert_eq!(display_letters("angle(AEB)"), "AEB");
    }

    #[test]
    fn i44_names_light_the_plate() {
        // BHG and GFE are angles, not the triangles the three letters span.
        // AG and ME are the parallelograms about the diagonal; LB and BF the
        // complements. C is the given triangle. HB is the joined piece of the
        // diagonal, not a walk around the figure.
        let phrases = crate::content::get(1, 44).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop44(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let bhg = runs("BHG");
        assert!(
            bhg.len() >= 1
                && bhg
                    .iter()
                    .all(|p| has(p, "bh") && has(p, "gh") && !has(p, "gb")),
            "BHG is the angle, not the triangle: {bhg:?}"
        );
        let gfe = runs("GFE");
        assert!(
            gfe.len() >= 1
                && gfe
                    .iter()
                    .all(|p| has(p, "fg") && has(p, "fe") && !has(p, "eg")),
            "GFE is the angle, not the triangle: {gfe:?}"
        );
        let ag = runs("AG");
        assert!(
            ag.len() >= 1
                && ag.iter().all(|p| has(p, "ba")
                    && has(p, "gb")
                    && has(p, "gh")
                    && has(p, "ha")
                    && !has(p, "bh")),
            "AG is the parallelogram: {ag:?}"
        );
        let lb = runs("LB");
        assert!(
            lb.len() >= 2
                && lb
                    .iter()
                    .all(|p| has(p, "al") && has(p, "ba") && has(p, "bm") && has(p, "ml")),
            "LB is the complement: {lb:?}"
        );
        let bf = runs("BF");
        assert!(
            bf.len() >= 2
                && bf
                    .iter()
                    .all(|p| has(p, "eb") && has(p, "fe") && has(p, "fg") && has(p, "gb")),
            "BF is the complement: {bf:?}"
        );
        let c = runs("C");
        assert!(
            c.len() >= 3 && c.iter().all(|p| has(p, "seg-c")),
            "C is the given triangle: {c:?}"
        );
        let hb = runs("HB");
        assert!(
            hb.len() >= 1 && hb.iter().all(|p| has(p, "bh") && !has(p, "kb")),
            "HB is the joined piece: {hb:?}"
        );
        let fg = runs("FG");
        assert!(
            fg.len() >= 1 && fg.iter().all(|p| has(p, "fg") && !has(p, "fe")),
            "FG is the drawn line, not a parallelogram: {fg:?}"
        );
        assert_eq!(display_letters("parallelogram(LB)"), "LB");
        assert_eq!(display_letters("angle(BHG)"), "BHG");
    }

    #[test]
    fn i45_names_light_the_plate() {
        // FH and GM are the parallelograms, named by opposite corners. ABD
        // and DBC are the triangles the diagonal cuts. ABCD is the given
        // rectilinear figure, not its diagonal. The three-letter names in
        // the proof are angles. E is the given angle.
        let phrases = crate::content::get(1, 45).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop45(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let stroke = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let no_stroke = |parts: &[String], id: &str| !parts.iter().any(|p| p == id);
        let fh = runs("FH");
        assert!(
            fh.len() >= 2
                && fh.iter().all(|p| {
                    stroke(p, "kf")
                        && stroke(p, "kh")
                        && stroke(p, "hg")
                        && stroke(p, "fg")
                        && no_stroke(p, "gl")
                        && no_stroke(p, "ml")
                }),
            "FH is parallelogram FH, not the outer path: {fh:?}"
        );
        let gm = runs("GM");
        assert!(
            gm.len() >= 2
                && gm.iter().all(|p| has(p, "hg")
                    && has(p, "hm")
                    && has(p, "ml")
                    && has(p, "gl")
                    && !has(p, "fg")
                    && !has(p, "kf")),
            "GM is parallelogram GM, not the outer path: {gm:?}"
        );
        let abd = runs("ABD");
        assert!(
            abd.len() >= 2
                && abd
                    .iter()
                    .all(|p| has(p, "ad") && has(p, "db") && has(p, "ba") && !has(p, "dc")),
            "ABD is the triangle, not an angle: {abd:?}"
        );
        let dbc = runs("DBC");
        assert!(
            dbc.len() >= 2
                && dbc
                    .iter()
                    .all(|p| has(p, "db") && has(p, "cb") && has(p, "dc") && !has(p, "ad")),
            "DBC is the triangle: {dbc:?}"
        );
        let abcd = runs("ABCD");
        assert!(
            abcd.len() >= 2
                && abcd.iter().all(|p| has(p, "ad")
                    && has(p, "dc")
                    && has(p, "cb")
                    && has(p, "ba")
                    && !has(p, "db")),
            "ABCD is the rectilinear figure, not its diagonal: {abcd:?}"
        );
        let kflm = runs("KFLM");
        assert!(
            kflm.len() >= 2
                && kflm.iter().all(|p| has(p, "kf")
                    && has(p, "fg")
                    && has(p, "gl")
                    && has(p, "ml")
                    && has(p, "kh")
                    && has(p, "hm")),
            "KFLM is the outer parallelogram: {kflm:?}"
        );
        let hkf = runs("HKF");
        assert!(
            hkf.len() >= 1
                && hkf
                    .iter()
                    .all(|p| has(p, "kh") && has(p, "kf") && !has(p, "fg")),
            "HKF is the angle, not the triangle: {hkf:?}"
        );
        let ghm = runs("GHM");
        assert!(
            ghm.len() >= 1
                && ghm
                    .iter()
                    .all(|p| has(p, "hg") && has(p, "hm") && !has(p, "ml")),
            "GHM is the angle, not the triangle: {ghm:?}"
        );
        let fkm = runs("FKM");
        assert!(
            fkm.len() >= 1
                && fkm.iter().all(|p| has(p, "kf")
                    && (has(p, "km") || (has(p, "kh") && has(p, "hm")))
                    && !has(p, "ml")),
            "FKM is the angle the parallelogram stands in: {fkm:?}"
        );
        let e = runs("E");
        assert!(
            e.len() >= 2 && e.iter().all(|p| has(p, "ang-e")),
            "E is the given angle: {e:?}"
        );
        let fg = runs("FG");
        assert!(
            fg.len() >= 1 && fg.iter().all(|p| has(p, "fg") && !has(p, "gl")),
            "FG is the drawn line, not the whole top: {fg:?}"
        );
        let db = runs("DB");
        assert!(
            db.len() >= 1 && db.iter().all(|p| has(p, "db") && !has(p, "ad")),
            "DB is the joined diagonal: {db:?}"
        );
        assert_eq!(display_letters("parallelogram(FH)"), "FH");
        assert_eq!(display_letters("triangle(ABD)"), "ABD");
        assert_eq!(display_letters("angle(HKF)"), "HKF");
    }

    #[test]
    fn i46_names_light_the_plate() {
        // ADEB is the square, not the produced upright. AC is the whole
        // upright the construction draws. BAD, ADE, ABE, and BED are the
        // angles, not a walk around the square.
        let phrases = crate::content::get(1, 46).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop46(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let adeb = runs("ADEB");
        assert!(
            adeb.len() >= 3
                && adeb.iter().all(|p| has(p, "ab")
                    && has(p, "da")
                    && has(p, "ed")
                    && has(p, "be")
                    && !has(p, "cd")),
            "ADEB is the square, not the produced line: {adeb:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 1
                && ac
                    .iter()
                    .all(|p| has(p, "cd") && has(p, "da") && !has(p, "ab")),
            "AC is the whole upright: {ac:?}"
        );
        let bad = runs("BAD");
        assert!(
            bad.len() >= 1
                && bad
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "da") && !has(p, "ed") && !has(p, "be")),
            "BAD is the angle: {bad:?}"
        );
        let ade = runs("ADE");
        assert!(
            ade.len() >= 1
                && ade
                    .iter()
                    .all(|p| has(p, "da") && has(p, "ed") && !has(p, "ab") && !has(p, "be")),
            "ADE is the angle, not the square: {ade:?}"
        );
        let abe = runs("ABE");
        assert!(
            abe.len() >= 1
                && abe
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "be") && !has(p, "da") && !has(p, "ed")),
            "ABE is the angle: {abe:?}"
        );
        let bed = runs("BED");
        assert!(
            bed.len() >= 1
                && bed
                    .iter()
                    .all(|p| has(p, "be") && has(p, "ed") && !has(p, "ab") && !has(p, "da")),
            "BED is the angle: {bed:?}"
        );
        assert_eq!(display_letters("angle(BAD)"), "BAD");
    }

    #[test]
    fn i47_names_light_the_plate() {
        // GB and HC are the squares on the legs, named by opposite corners.
        // BL and CL are the parallelograms the proof cuts from square BDEC.
        // A walk through A is the triangle, not those figures. The three-letter
        // names in the proof are angles, except the triangles ABD and FBC.
        let phrases = crate::content::get(1, 47).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop47(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let gb = runs("GB");
        assert!(
            gb.len() >= 3
                && gb.iter().all(|p| has(p, "fg")
                    && has(p, "ga")
                    && has(p, "ab")
                    && has(p, "fb")
                    && !has(p, "ah")),
            "GB is the square, not a walk: {gb:?}"
        );
        let hc = runs("HC");
        assert!(
            hc.len() >= 3
                && hc.iter().all(|p| has(p, "ah")
                    && has(p, "ac")
                    && has(p, "kc")
                    && has(p, "hk")
                    && !has(p, "ab")),
            "HC is the square, not a walk: {hc:?}"
        );
        let bl = runs("BL");
        assert!(
            bl.len() >= 2
                && bl.iter().all(|p| has(p, "bd")
                    && has(p, "dl")
                    && has(p, "pl")
                    && has(p, "bp")
                    && !has(p, "ab")
                    && !has(p, "ap")),
            "BL is the parallelogram, not the path through A: {bl:?}"
        );
        let cl = runs("CL");
        assert!(
            cl.len() >= 1
                && cl.iter().all(|p| has(p, "ce")
                    && has(p, "le")
                    && has(p, "pl")
                    && has(p, "pc")
                    && !has(p, "ac")
                    && !has(p, "ap")),
            "CL is the parallelogram, not the path through A: {cl:?}"
        );
        let bdec = runs("BDEC");
        assert!(
            bdec.len() >= 3
                && bdec.iter().all(|p| has(p, "bd")
                    && has(p, "dl")
                    && has(p, "le")
                    && has(p, "ce")
                    && has(p, "bp")
                    && has(p, "pc")
                    && !has(p, "ad")),
            "BDEC is the square on BC: {bdec:?}"
        );
        let bac = runs("BAC");
        assert!(
            bac.len() >= 1
                && bac
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "ac") && !has(p, "bp") && !has(p, "pc")),
            "BAC is the angle, not the triangle: {bac:?}"
        );
        let bag = runs("BAG");
        assert!(
            bag.len() >= 1
                && bag
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "ga") && !has(p, "fb")),
            "BAG is the angle: {bag:?}"
        );
        let dbc = runs("DBC");
        assert!(
            dbc.len() >= 1
                && dbc
                    .iter()
                    .all(|p| has(p, "bd") && (has(p, "bp") || has(p, "pc")) && !has(p, "ad")),
            "DBC is the angle: {dbc:?}"
        );
        let dba = runs("DBA");
        assert!(
            dba.len() >= 1
                && dba
                    .iter()
                    .all(|p| has(p, "bd") && has(p, "ab") && !has(p, "ad")),
            "DBA is the angle, not triangle ABD: {dba:?}"
        );
        let fbc_ang = runs("FBC");
        assert!(
            fbc_ang
                .iter()
                .any(|p| has(p, "fb") && (has(p, "bp") || has(p, "pc")) && !has(p, "fc")),
            "FBC is an angle as well as a triangle: {fbc_ang:?}"
        );
        assert!(
            fbc_ang
                .iter()
                .any(|p| has(p, "fb") && has(p, "fc") && (has(p, "bp") || has(p, "pc"))),
            "FBC is the triangle as well as the angle: {fbc_ang:?}"
        );
        let abd = runs("ABD");
        assert!(
            abd.len() >= 1
                && abd
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "bd") && has(p, "ad") && !has(p, "bp")),
            "ABD is the triangle, not the angle: {abd:?}"
        );
        let gc = runs("GC");
        assert!(
            gc.len() >= 1
                && gc
                    .iter()
                    .all(|p| has(p, "ga") && has(p, "ac") && !has(p, "fb") && !has(p, "hk")),
            "GC is GA straight-on to AC: {gc:?}"
        );
        let al = runs("AL");
        assert!(
            al.len() >= 2
                && al
                    .iter()
                    .all(|p| has(p, "ap") && has(p, "pl") && !has(p, "ab")),
            "AL is the parallel, both pieces: {al:?}"
        );
        assert_eq!(display_letters("parallelogram(BL)"), "BL");
        assert_eq!(display_letters("triangle(ABD)"), "ABD");
        assert_eq!(display_letters("angle(BAC)"), "BAC");
    }

    #[test]
    fn i48_names_light_the_plate() {
        // BAC and DAC are the angles the proof compares. ABC is the triangle.
        // A three-letter name must not light the third side.
        let phrases = crate::content::get(1, 48).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book1_prop48(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let bac = runs("BAC");
        assert!(
            bac.len() >= 2
                && bac
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "ac") && !has(p, "bc")),
            "BAC is the angle, not the triangle: {bac:?}"
        );
        let dac = runs("DAC");
        assert!(
            dac.len() >= 2
                && dac
                    .iter()
                    .all(|p| has(p, "da") && has(p, "ac") && !has(p, "dc")),
            "DAC is the angle, not triangle DAC: {dac:?}"
        );
        let abc = runs("ABC");
        assert!(
            abc.len() >= 1
                && abc
                    .iter()
                    .all(|p| has(p, "ab") && has(p, "ac") && has(p, "bc")),
            "ABC is the triangle: {abc:?}"
        );
        let dc = runs("DC");
        assert!(
            dc.len() >= 2
                && dc
                    .iter()
                    .all(|p| has(p, "dc") && !has(p, "da") && !has(p, "ac")),
            "DC is the joined side: {dc:?}"
        );
        assert_eq!(display_letters("angle(BAC)"), "BAC");
        assert_eq!(display_letters("triangle(ABC)"), "ABC");
    }

    #[test]
    fn ii10_angles_and_sides_are_the_plate() {
        // Fitzpatrick names the angles, and the sides the squares stand on.
        // The angle at C is the right-angle ACE, not the whole pencil. The
        // opposite angle at C is DCE, the right-angle of the parallelogram,
        // not ACE. The angle at F is the right-angle EFD. A named end that
        // the ray only reaches by passing a letter still lights that ray.
        // The plate draws no square.
        let phrases = crate::content::get(2, 10).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop10(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("in half at (point) C"), "C is the midpoint: {spoken}");
        assert!(spoken.contains("straight-line BD"), "BD is the added line: {spoken}");
        assert!(spoken.contains("angle EAC"), "EAC is an angle: {spoken}");
        assert!(spoken.contains("(angle) at C"), "the right-angle is at C: {spoken}");
        assert!(
            spoken.contains("opposite (angle) at C"),
            "the opposite angle is at C, as Fitzpatrick writes it: {spoken}"
        );
        assert!(spoken.contains("(angle) AEB"), "AEB is the right-angle: {spoken}");
        assert!(spoken.contains("(angle) at F"), "the right-angle is at F: {spoken}");
        assert!(spoken.contains("squares on AD and DB"), "the close names both squares: {spoken}");
        let ab = runs("AB");
        assert!(
            ab.len() >= 1 && ab.iter().all(|p| has(p, "ac") && has(p, "cb") && !has(p, "bd") && !has(p, "ea")),
            "AB is the cut line, not the added piece: {ab:?}"
        );
        let bd = runs("BD");
        assert!(
            bd.len() >= 2 && bd.iter().all(|p| has(p, "bd") && !has(p, "cb") && !has(p, "dg")),
            "BD is the added line: {bd:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 2 && ad.iter().all(|p| has(p, "ac") && has(p, "cb") && has(p, "bd") && !has(p, "dg") && !has(p, "ag")),
            "AD is the whole with the added line: {ad:?}"
        );
        let db = runs("DB");
        assert!(
            db.len() >= 2 && db.iter().all(|p| has(p, "bd") && !has(p, "cb") && !has(p, "dg")),
            "DB is the added line: {db:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 2 && ac.iter().all(|p| has(p, "ac") && !has(p, "cb") && !has(p, "ea")),
            "AC is the half: {ac:?}"
        );
        let cd = runs("CD");
        assert!(
            cd.len() >= 2 && cd.iter().all(|p| has(p, "cb") && has(p, "bd") && !has(p, "ac") && !has(p, "dg")),
            "CD is the half plus the added line: {cd:?}"
        );
        let ce = runs("CE");
        assert!(
            ce.len() >= 2 && ce.iter().all(|p| has(p, "ce") && !has(p, "ac") && !has(p, "ef")),
            "CE is the upright: {ce:?}"
        );
        let ea = runs("EA");
        assert!(
            ea.len() >= 2 && ea.iter().all(|p| has(p, "ea") && !has(p, "eb") && !has(p, "ag")),
            "EA is the join: {ea:?}"
        );
        let eb = runs("EB");
        assert!(
            eb.len() >= 2 && eb.iter().all(|p| has(p, "eb") && !has(p, "bg") && !has(p, "ea") && !has(p, "ef")),
            "EB is the join, not the production: {eb:?}"
        );
        let ef = runs("EF");
        assert!(
            ef.len() >= 2 && ef.iter().all(|p| has(p, "ef") && !has(p, "fd") && !has(p, "eb")),
            "EF is the parallel: {ef:?}"
        );
        let fd = runs("FD");
        assert!(
            fd.len() >= 2 && fd.iter().all(|p| has(p, "fd") && !has(p, "dg") && !has(p, "ef")),
            "FD is the parallel, not the production: {fd:?}"
        );
        let ag = runs("AG");
        assert!(
            ag.len() >= 2 && ag.iter().all(|p| has(p, "ag") && !has(p, "ac") && !has(p, "dg")),
            "AG is the join: {ag:?}"
        );
        let eg = runs("EG");
        assert!(
            eg.len() >= 2 && eg.iter().all(|p| has(p, "eb") && has(p, "bg") && !has(p, "ef") && !has(p, "dg")),
            "EG is EB produced: {eg:?}"
        );
        let gf = runs("GF");
        assert!(
            gf.len() >= 2 && gf.iter().all(|p| has(p, "dg") && has(p, "fd") && !has(p, "ef") && !has(p, "bg")),
            "GF is FD produced: {gf:?}"
        );
        let eac = runs("EAC");
        assert!(
            eac.len() >= 2 && eac.iter().all(|p| has(p, "ea") && has(p, "ac") && !has(p, "ce") && !has(p, "ag")),
            "EAC is the angle, not the triangle: {eac:?}"
        );
        let aec = runs("AEC");
        assert!(
            aec.len() >= 2 && aec.iter().all(|p| has(p, "ea") && has(p, "ce") && !has(p, "ac")),
            "AEC is the angle: {aec:?}"
        );
        let at_c = runs("C");
        assert!(
            at_c.iter().any(|p| has(p, "ac") && has(p, "ce") && has(p, "A") && has(p, "E") && !has(p, "cb") && !has(p, "ea")),
            "the angle at C is ACE, not the pencil: {at_c:?}"
        );
        assert!(
            at_c.iter().any(|p| has(p, "cb") && has(p, "bd") && has(p, "ce") && has(p, "D") && has(p, "E") && !has(p, "ac") && !has(p, "ea") && !has(p, "dg")),
            "the opposite angle at C is DCE, not ACE: {at_c:?}"
        );
        let aeb = runs("AEB");
        assert!(
            aeb.len() >= 1 && aeb.iter().all(|p| has(p, "ea") && has(p, "eb") && !has(p, "bg") && !has(p, "ag")),
            "AEB is the right-angle: {aeb:?}"
        );
        let ebc = runs("EBC");
        assert!(
            ebc.len() >= 2 && ebc.iter().all(|p| has(p, "eb") && has(p, "cb") && !has(p, "bd") && !has(p, "ce")),
            "EBC is the half right-angle: {ebc:?}"
        );
        let dbg = runs("DBG");
        assert!(
            dbg.len() >= 2 && dbg.iter().all(|p| has(p, "bd") && has(p, "bg") && !has(p, "dg") && !has(p, "eb")),
            "DBG is the vertically opposite angle: {dbg:?}"
        );
        let bdg = runs("BDG");
        assert!(
            bdg.len() >= 1 && bdg.iter().all(|p| has(p, "bd") && has(p, "dg") && !has(p, "bg") && !has(p, "fd")),
            "BDG is the right-angle: {bdg:?}"
        );
        let dce = runs("DCE");
        assert!(
            dce.len() >= 1
                && dce.iter().all(|p| has(p, "cb") && has(p, "bd") && has(p, "ce") && !has(p, "ac") && !has(p, "ea") && !has(p, "dg")),
            "DCE is the opposite angle at C, not ACE: {dce:?}"
        );
        let dgb = runs("DGB");
        assert!(
            dgb.len() >= 2 && dgb.iter().all(|p| has(p, "dg") && has(p, "bg") && !has(p, "bd") && !has(p, "fd") && !has(p, "eb")),
            "DGB is the angle, not a walk: {dgb:?}"
        );
        let egf = runs("EGF");
        assert!(
            egf.len() >= 1 && egf.iter().all(|p| has(p, "eb") && has(p, "bg") && has(p, "dg") && has(p, "fd") && !has(p, "ef") && !has(p, "bd")),
            "EGF is the angle, through to F: {egf:?}"
        );
        let at_f = runs("F");
        assert!(
            at_f.iter().any(|p| has(p, "ef") && has(p, "fd") && has(p, "E") && has(p, "D") && !has(p, "dg") && !has(p, "eb")),
            "the angle at F is EFD: {at_f:?}"
        );
        let feg = runs("FEG");
        assert!(
            feg.len() >= 1 && feg.iter().all(|p| has(p, "ef") && has(p, "eb") && has(p, "bg") && !has(p, "fd") && !has(p, "dg") && !has(p, "B")),
            "FEG is the angle, through to G, without lighting B: {feg:?}"
        );
        assert_eq!(display_letters("angle(EAC)"), "EAC");
        assert_eq!(display_letters("angle(AEB)"), "AEB");
        assert_eq!(display_letters("angle(DGB)"), "DGB");
    }

    #[test]
    fn ii11_rectangles_and_squares_are_the_plate() {
        // Fitzpatrick names the rectangles and the squares by opposite
        // corners, and the sides they stand on. The angle at A is BAE:
        // the right-angle contained by BA and AE, the one Prop. 1.47
        // applies to triangle AEB. Not the square's corner through to C,
        // and not the whole pencil. A named end that the ray only reaches
        // by passing a letter still lights that ray. The plate draws no
        // diagonal of a named rectangle. FK is rectangle FGKC: the corners,
        // and the ink between them. A, E, and H lie on those sides.
        let phrases = crate::content::get(2, 11).unwrap().phrases;
        let script = compile(phrases, &crate::figure::book2_prop11(), Timing::default());
        let runs = |word: &str| -> Vec<Vec<String>> {
            script
                .lines
                .iter()
                .flat_map(|l| l.tokens.iter())
                .filter(|t| letter_key(&t.text) == word)
                .map(|t| t.parts.clone())
                .collect()
        };
        let has = |parts: &[String], id: &str| parts.iter().any(|p| p == id);
        let shown: Vec<String> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| !t.cite)
            .map(|t| display_letters(&t.text))
            .collect();
        let spoken = shown.join(" ");
        assert!(spoken.contains("straight-line AB"), "AB is the given line: {spoken}");
        assert!(spoken.contains("square ABDC"), "ABDC is the square: {spoken}");
        assert!(spoken.contains("in half at point E"), "E is the midpoint: {spoken}");
        assert!(spoken.contains("square FH"), "FH is the square on AF: {spoken}");
        assert!(spoken.contains("cut at H"), "H is the cut: {spoken}");
        assert!(spoken.contains("angle at A"), "the right-angle is at A: {spoken}");
        assert!(spoken.contains("square on HA"), "the close names the square on HA: {spoken}");
        let ab = runs("AB");
        assert!(
            ab.len() >= 2 && ab.iter().all(|p| has(p, "ah") && has(p, "hb") && !has(p, "ac") && !has(p, "bd")),
            "AB is the given line, not a side of the square: {ab:?}"
        );
        let ac = runs("AC");
        assert!(
            ac.len() >= 1 && ac.iter().all(|p| has(p, "ae") && has(p, "ec") && !has(p, "fa") && !has(p, "be")),
            "AC is the side, through E: {ac:?}"
        );
        let be = runs("BE");
        assert!(
            be.iter().any(|p| has(p, "be") && !has(p, "ae") && !has(p, "hb")),
            "BE is the join: {be:?}"
        );
        let ca = runs("CA");
        assert!(
            ca.len() >= 1 && ca.iter().all(|p| has(p, "ec") && has(p, "ae") && !has(p, "fa")),
            "CA is the side, not the production: {ca:?}"
        );
        let ef = runs("EF");
        assert!(
            ef.len() >= 2 && ef.iter().all(|p| has(p, "ae") && has(p, "fa") && !has(p, "ec") && !has(p, "be")),
            "EF is the added line, through A: {ef:?}"
        );
        let fa = runs("FA");
        assert!(
            fa.len() >= 2 && fa.iter().all(|p| has(p, "fa") && !has(p, "ae") && !has(p, "fg")),
            "FA is the side of the square on AF: {fa:?}"
        );
        let gh = runs("GH");
        assert!(
            gh.len() >= 1 && gh.iter().all(|p| has(p, "gh") && !has(p, "hk") && !has(p, "fg")),
            "GH is the top of the square, not the production: {gh:?}"
        );
        let cf = runs("CF");
        assert!(
            cf.len() >= 2 && cf.iter().all(|p| has(p, "ec") && has(p, "ae") && has(p, "fa") && !has(p, "be") && !has(p, "fg")),
            "CF is the side with the added line: {cf:?}"
        );
        let ae = runs("AE");
        assert!(
            ae.len() >= 2 && ae.iter().all(|p| has(p, "ae") && !has(p, "ec") && !has(p, "fa") && !has(p, "be")),
            "AE is the half: {ae:?}"
        );
        let eb = runs("EB");
        assert!(
            eb.len() >= 1 && eb.iter().all(|p| has(p, "be") && !has(p, "ae") && !has(p, "hb")),
            "EB is the join: {eb:?}"
        );
        let ba = runs("BA");
        assert!(
            ba.len() >= 1 && ba.iter().all(|p| has(p, "hb") && has(p, "ah") && !has(p, "bd") && !has(p, "be")),
            "BA is the given line: {ba:?}"
        );
        let bh = runs("BH");
        assert!(
            bh.len() >= 2 && bh.iter().all(|p| has(p, "hb") && !has(p, "ah") && !has(p, "bd") && !has(p, "hk")),
            "BH is the piece: {bh:?}"
        );
        let ah = runs("AH");
        assert!(
            ah.len() >= 2 && ah.iter().all(|p| has(p, "ah") && !has(p, "hb") && !has(p, "fa") && !has(p, "gh")),
            "AH is the remaining piece: {ah:?}"
        );
        let ha = runs("HA");
        assert!(
            ha.len() >= 1 && ha.iter().all(|p| has(p, "ah") && !has(p, "hb") && !has(p, "fa")),
            "HA is the remaining piece: {ha:?}"
        );
        let bd = runs("BD");
        assert!(
            bd.len() >= 1 && bd.iter().all(|p| has(p, "bd") && !has(p, "hb") && !has(p, "kd")),
            "BD is the side of the square: {bd:?}"
        );
        let fg = runs("FG");
        assert!(
            fg.len() >= 1 && fg.iter().all(|p| has(p, "fg") && !has(p, "gh") && !has(p, "fa")),
            "FG is the top of the square: {fg:?}"
        );
        let at_a = runs("A");
        assert!(
            at_a.iter().any(|p| {
                has(p, "ah")
                    && has(p, "hb")
                    && has(p, "ae")
                    && has(p, "B")
                    && has(p, "E")
                    && !has(p, "ec")
                    && !has(p, "C")
                    && !has(p, "fa")
                    && !has(p, "F")
                    && !has(p, "H")
            }),
            "the angle at A is BAE, not the square's corner through to C: {at_a:?}"
        );
        let abdc = runs("ABDC");
        assert!(
            abdc.len() == 1 && abdc.iter().all(|p| has(p, "ah") && has(p, "hb") && has(p, "bd") && has(p, "kd") && has(p, "ck") && has(p, "ae") && has(p, "ec") && !has(p, "be") && !has(p, "fa")),
            "ABDC is the square, not the diagonal: {abdc:?}"
        );
        let fh = runs("FH");
        assert!(
            fh.len() >= 1 && fh.iter().all(|p| has(p, "fa") && has(p, "ah") && has(p, "gh") && has(p, "fg") && !has(p, "hk") && !has(p, "be")),
            "FH is the square on AF: {fh:?}"
        );
        let fk = runs("FK");
        assert!(
            fk.len() >= 1
                && fk.iter().all(|p| {
                    has(p, "fg")
                        && has(p, "gh")
                        && has(p, "hk")
                        && has(p, "ck")
                        && has(p, "ec")
                        && has(p, "ae")
                        && has(p, "fa")
                        && has(p, "F")
                        && has(p, "G")
                        && has(p, "K")
                        && has(p, "C")
                        && !has(p, "A")
                        && !has(p, "E")
                        && !has(p, "H")
                        && !has(p, "ah")
                        && !has(p, "hb")
                        && !has(p, "be")
                }),
            "FK is rectangle FGKC, not the letters the sides pass through: {fk:?}"
        );
        let ad = runs("AD");
        assert!(
            ad.len() >= 1 && ad.iter().all(|p| has(p, "ah") && has(p, "hb") && has(p, "bd") && has(p, "kd") && has(p, "ck") && has(p, "ae") && has(p, "ec") && !has(p, "be")),
            "AD is the square on AB: {ad:?}"
        );
        let ak = runs("AK");
        assert!(
            ak.len() >= 1 && ak.iter().all(|p| has(p, "ah") && has(p, "hk") && has(p, "ck") && has(p, "ae") && has(p, "ec") && !has(p, "hb") && !has(p, "fa")),
            "AK is the rectangle taken off: {ak:?}"
        );
        let hd = runs("HD");
        assert!(
            hd.len() >= 1 && hd.iter().all(|p| has(p, "hb") && has(p, "bd") && has(p, "kd") && has(p, "hk") && !has(p, "ah") && !has(p, "be")),
            "HD is the remaining rectangle: {hd:?}"
        );
        assert_eq!(display_letters("rectangle(FK)"), "FK");
        assert_eq!(display_letters("rectangle(FH)"), "FH");
        assert_eq!(display_letters("rectangle(HD)"), "HD");
    }
}
