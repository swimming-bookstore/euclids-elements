//! Named-point construction. Karaoke hits are the segment graph plus named circles.

use super::draw::Figure;
use super::geom::{Place, V2};
use std::collections::{HashMap, VecDeque};

struct Edge {
    to: &'static str,
    id: String,
}

struct Circ {
    id: String,
    letters: String,
}

/// A spoken name whose ink is not the straight-line between the letters.
///
/// I.43 calls the parallelograms about the diagonal *EH* and *FG*, and the
/// complements *BK* and *KD*. Those are opposite corners. The plate draws no
/// stroke EH, FG, BK, or KD — the name is the boundary, not the diagonal.
struct Region {
    letters: String,
    boundary: Vec<&'static str>,
}

/// A spoken straight-line the plate draws bent.
///
/// I.47's *GC* is G–A–C: the proof calls it one line (CA straight-on to AG),
/// and the plate bends a few degrees at A. The short graph path is G–F–C,
/// which is the wrong ink. The letters are the ends; the points are the
/// drawn chain.
struct Poly {
    letters: String,
    points: Vec<&'static str>,
}

/// One stroke of a given angle (I.42's D). Named so tests can measure it;
/// not a lettered point of the figure.
struct Arm {
    name: String,
    at: V2,
}

/// A Euclidean figure: name the points, join them, draw circles.
pub struct Diagram {
    fig: Figure,
    pts: HashMap<&'static str, V2>,
    adj: HashMap<&'static str, Vec<Edge>>,
    circles: Vec<Circ>,
    named: Vec<Circ>,
    arms: Vec<Arm>,
    regions: Vec<Region>,
    polys: Vec<Poly>,
}

impl Diagram {
    pub fn new() -> Self {
        Self {
            fig: Figure::new(),
            pts: HashMap::new(),
            adj: HashMap::new(),
            circles: Vec::new(),
            named: Vec::new(),
            arms: Vec::new(),
            regions: Vec::new(),
            polys: Vec::new(),
        }
    }

    pub fn at(&self, name: &str) -> V2 {
        if let Some(p) = self.pts.get(name) {
            return *p;
        }
        self.arms
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.at)
            .unwrap_or_else(|| panic!("unknown point {name}"))
    }

    /// Name a point (no letter).
    pub fn pin(&mut self, name: &'static str, p: V2) -> V2 {
        self.pts.insert(name, p);
        p
    }

    /// Name a point and place its letter.
    pub fn put(&mut self, name: &'static str, p: V2, place: Place) -> V2 {
        self.pin(name, p);
        self.fig.label_at(name, p, place);
        p
    }

    /// End of a given straight-line (Fitzpatrick): letter outside the end,
    /// slightly above the stroke. `left` is the lesser-x end on a left-to-right line.
    pub fn put_line_end(&mut self, name: &'static str, p: V2, left: bool) -> V2 {
        self.put(
            name,
            p,
            if left {
                Place::LineLeft
            } else {
                Place::LineRight
            },
        )
    }

    /// Letter on an already named point, shared radius.
    pub fn label(&mut self, name: &'static str, place: Place) -> &mut Self {
        self.fig.label_at(name, self.at(name), place);
        self
    }

    /// Letter on an already named point, at `r_em` instead of the shared radius.
    ///
    /// I.39's E is one plate letter under the crossing. A fixed glyph center
    /// (the ink box) sits too low once the type is smaller than Fitzpatrick's.
    pub fn label_r(&mut self, name: &'static str, place: Place, r_em: f64) -> &mut Self {
        self.fig.label_at_r(name, self.at(name), place, r_em);
        self
    }

    /// Name a point; letter center at `r_em` from the mark.
    pub fn put_r(&mut self, name: &'static str, p: V2, place: Place, r_em: f64) -> V2 {
        self.pin(name, p);
        self.fig.label_at_r(name, p, place, r_em);
        p
    }

    /// Name a point, then drop its letter by `down` em.
    ///
    /// The shared radius and heading stay. `down` is extra, straight down, so
    /// the letter clears a figure sitting above the mark (I.44 F, E, and K,
    /// which otherwise read as labels of triangle C). The mark stays put.
    pub fn put_down(
        &mut self,
        name: &'static str,
        p: V2,
        place: Place,
        down: f64,
    ) -> V2 {
        self.pin(name, p);
        self.fig.label_at_r_drop(name, p, place, super::plate::LETTER_R, down);
        p
    }

    pub fn polar(
        &mut self,
        name: &'static str,
        origin: &str,
        r: f64,
        deg: f64,
        place: Place,
    ) -> V2 {
        let p = self.at(origin).polar(r, deg);
        self.put(name, p, place)
    }

    /// Point on the ray `origin` → `through`, at distance `dist` from the origin.
    pub fn ray(
        &mut self,
        name: &'static str,
        origin: &str,
        through: &str,
        dist: f64,
        place: Place,
    ) -> V2 {
        let p = self.at(origin).toward(self.at(through), dist);
        self.put(name, p, place)
    }

    /// Same height as `of` (a Fitzpatrick letter-row).
    pub fn level(&mut self, name: &'static str, of: &str, x: f64, place: Place) -> V2 {
        self.put(name, V2::new(x, self.at(of).y), place)
    }

    /// `level`, with the letter center at `r_em` from the mark.
    pub fn level_r(&mut self, name: &'static str, of: &str, x: f64, place: Place, r_em: f64) -> V2 {
        self.put_r(name, V2::new(x, self.at(of).y), place, r_em)
    }

    /// Same column as `of` (a Fitzpatrick letter-column).
    pub fn plumb(&mut self, name: &'static str, of: &str, y: f64, place: Place) -> V2 {
        self.put(name, V2::new(self.at(of).x, y), place)
    }

    /// `plumb`, with the letter center at `r_em` from the mark.
    pub fn plumb_r(&mut self, name: &'static str, of: &str, y: f64, place: Place, r_em: f64) -> V2 {
        self.put_r(name, V2::new(self.at(of).x, y), place, r_em)
    }

    /// Column of `x_of` × row of `y_of`.
    pub fn corner(&mut self, name: &'static str, x_of: &str, y_of: &str, place: Place) -> V2 {
        self.put(name, V2::new(self.at(x_of).x, self.at(y_of).y), place)
    }

    /// Intersection of lines `a``b` and `c``d`. No letter.
    pub fn meet_pin(
        &mut self,
        name: &'static str,
        a: &str,
        b: &str,
        c: &str,
        d: &str,
    ) -> V2 {
        let pa = self.at(a);
        let pb = self.at(b);
        let pc = self.at(c);
        let pd = self.at(d);
        let den = (pa.x - pb.x) * (pc.y - pd.y) - (pa.y - pb.y) * (pc.x - pd.x);
        if den.abs() < 1e-12 {
            panic!("{a}{b} parallel to {c}{d}");
        }
        let t = ((pa.x - pc.x) * (pc.y - pd.y) - (pa.y - pc.y) * (pc.x - pd.x)) / den;
        self.pin(
            name,
            V2::new(pa.x + t * (pb.x - pa.x), pa.y + t * (pb.y - pa.y)),
        )
    }

    /// Intersection of lines `a``b` and `c``d`.
    pub fn meet(
        &mut self,
        name: &'static str,
        a: &str,
        b: &str,
        c: &str,
        d: &str,
        place: Place,
    ) -> V2 {
        let pa = self.at(a);
        let pb = self.at(b);
        let pc = self.at(c);
        let pd = self.at(d);
        let den = (pa.x - pb.x) * (pc.y - pd.y) - (pa.y - pb.y) * (pc.x - pd.x);
        if den.abs() < 1e-12 {
            panic!("{a}{b} parallel to {c}{d}");
        }
        let t = ((pa.x - pc.x) * (pc.y - pd.y) - (pa.y - pc.y) * (pc.x - pd.x)) / den;
        self.put(
            name,
            V2::new(pa.x + t * (pb.x - pa.x), pa.y + t * (pb.y - pa.y)),
            place,
        )
    }

    /// Letter on the circle `center`–`through`, at `deg` from +x.
    /// The glyph sits radially **outside** the circumference.
    pub fn on_circle(
        &mut self,
        name: &'static str,
        center: &str,
        through: &str,
        deg: f64,
    ) -> V2 {
        self.circle_letter(name, center, through, deg, false)
    }

    /// Same as `on_circle`, but the glyph sits radially **inside** the disk.
    pub fn in_circle(
        &mut self,
        name: &'static str,
        center: &str,
        through: &str,
        deg: f64,
    ) -> V2 {
        self.circle_letter(name, center, through, deg, true)
    }

    fn circle_letter(
        &mut self,
        name: &'static str,
        center: &str,
        through: &str,
        deg: f64,
        inside: bool,
    ) -> V2 {
        let c = self.at(center);
        let p = c.polar(c.dist(self.at(through)), deg);
        self.pin(name, p);
        let ang = if inside { deg + 180.0 } else { deg };
        self.fig.label_at(name, p, Place::Deg(ang));
        p
    }

    /// Letter on segment `a`–`b` at fraction `t` (0 at `a`, 1 at `b`).
    pub fn on_line(
        &mut self,
        name: &'static str,
        a: &str,
        b: &str,
        t: f64,
        place: Place,
    ) -> V2 {
        let pa = self.at(a);
        let pb = self.at(b);
        let p = pa.add(pb.sub(pa).mul(t));
        self.pin(name, p);
        self.fig.label_at(name, p, place);
        p
    }

    /// Letter measured from `at`, not from the named point (plate label that
    /// does not sit on the stroke end).
    pub fn label_at(&mut self, name: &'static str, at: V2, place: Place) -> &mut Self {
        self.fig.label_at(name, at, place);
        self
    }

    /// Letter whose center is the plate glyph, not an offset from the mark.
    /// Use this when the capital does not sit on the stroke (I.39 E, I.40 F).
    pub fn glyph(&mut self, name: &'static str, at: V2) -> &mut Self {
        self.label_fixed(name, at)
    }

    /// Letter whose center is exactly `at` (a plate glyph, not an offset from a mark).
    pub fn label_fixed(&mut self, name: &'static str, at: V2) -> &mut Self {
        self.fig.label_at_r(name, at, Place::Above, 0.0);
        self
    }

    /// A given rectilinear angle, as on the plate (I.23, I.42): not a point of the
    /// figure. Measure the vertex from `from`, then each arm from the vertex, then
    /// the letter.
    ///
    /// `vertex` is `(r, heading°)` from `from`. `left` and `right` are
    /// `(length, heading°)` from that vertex — the two strokes, not a bisector.
    /// `glyph` is the plate letter center. Arm ends are `{name}left` and
    /// `{name}right` (reachable from `at`, not lettered points).
    pub fn given_angle(
        &mut self,
        name: &'static str,
        from: &str,
        vertex: (f64, f64),
        left: (f64, f64),
        right: (f64, f64),
        glyph: V2,
    ) -> &mut Self {
        self.given_angle_arms(name, from, vertex, &[left, right], glyph)
    }

    /// As `given_angle`, but the plate's mark may have more than two strokes
    /// from the vertex. Every arm shares the one id, so the spoken letter
    /// lights the whole mark.
    pub fn given_angle_arms(
        &mut self,
        name: &'static str,
        from: &str,
        vertex: (f64, f64),
        arms: &[(f64, f64)],
        glyph: V2,
    ) -> &mut Self {
        let v = self.at(from).polar(vertex.0, vertex.1);
        self.pin(name, v);
        let id = format!("ang-{}", name.to_ascii_lowercase());
        let side = ["left", "right"];
        for (i, (len, heading)) in arms.iter().enumerate() {
            let end = v.polar(*len, *heading);
            self.fig.seg(&id, end, v);
            let arm = side.get(i).copied().unwrap_or("arm");
            let suffix = if i < 2 { arm } else { "arm" };
            let label = if i < 2 {
                format!("{name}{suffix}")
            } else {
                format!("{name}arm{i}")
            };
            self.arms.push(Arm { name: label, at: end });
        }
        self.named.push(Circ {
            id,
            letters: name.to_ascii_uppercase(),
        });
        self.glyph(name, glyph)
    }

    pub fn dots(&mut self, names: &[&'static str]) -> &mut Self {
        for name in names {
            let p = self.at(name);
            self.fig.dot(*name, p);
        }
        self
    }

    /// A plate mark that is not the named point (I.32 D sits on the stroke
    /// above its letter, not at the end of the produced side).
    pub fn dot_at(&mut self, name: &'static str, p: V2) -> &mut Self {
        self.fig.dot(name, p);
        self
    }

    pub fn join(&mut self, a: &'static str, b: &'static str) -> &mut Self {
        let id = format!("{}{}", a.to_ascii_lowercase(), b.to_ascii_lowercase());
        self.stroke(a, b, id)
    }

    /// Spoken name for a segment (`C` in I.3).
    pub fn named_line(&mut self, letters: &'static str, a: &'static str, b: &'static str) -> &mut Self {
        let id = format!("seg-{}", letters.to_ascii_lowercase());
        self.named.push(Circ {
            id: id.clone(),
            letters: letters.to_ascii_uppercase(),
        });
        self.stroke(a, b, id)
    }

    fn stroke(&mut self, a: &'static str, b: &'static str, id: String) -> &mut Self {
        let pa = self.at(a);
        let pb = self.at(b);
        self.fig.seg(&id, pa, pb);
        self.adj.entry(a).or_default().push(Edge {
            to: b,
            id: id.clone(),
        });
        self.adj.entry(b).or_default().push(Edge { to: a, id });
        self
    }

    /// A figure named by opposite corners, not by a drawn stroke.
    ///
    /// `letters` is the spoken name (*EH*, *BK*). `boundary` is the polygon in
    /// order around the ink. Highlighting the name lights that boundary. It
    /// does not light the diagonal the letters span, and it does not apply
    /// when those letters are a side of some other named figure.
    pub fn region(&mut self, letters: &'static str, boundary: &[&'static str]) -> &mut Self {
        self.regions.push(Region {
            letters: letters.to_ascii_uppercase(),
            boundary: boundary.to_vec(),
        });
        self
    }

    /// A spoken straight-line drawn as a bent chain (`GC` as G–A–C).
    ///
    /// The ends are the spoken letters. Highlighting the name lights that
    /// chain, not another walk of the same length.
    pub fn along(&mut self, letters: &'static str, points: &[&'static str]) -> &mut Self {
        self.polys.push(Poly {
            letters: letters.to_ascii_uppercase(),
            points: points.to_vec(),
        });
        self
    }

    /// A figure spoken by one letter, whose ink is the polygon.
    ///
    /// I.44's *C* is the given triangle, not a point of the construction. The
    /// sides share one id, so the letter lights the outline. The corners stay
    /// unlettered.
    pub fn named_outline(&mut self, letters: &'static str, boundary: &[&'static str]) -> &mut Self {
        let id = format!("seg-{}", letters.to_ascii_lowercase());
        self.named.push(Circ {
            id: id.clone(),
            letters: letters.to_ascii_uppercase(),
        });
        if boundary.len() >= 2 {
            for w in boundary.windows(2) {
                self.stroke(w[0], w[1], id.clone());
            }
            let last = boundary[boundary.len() - 1];
            let first = boundary[0];
            if last != first {
                self.stroke(last, first, id);
            }
        }
        self
    }

    /// The minor circumference from `a` to `b` on the circle about `center`.
    ///
    /// Drawn only while lit, so it does not grey out the circle. `*arc(AB)*`
    /// lights this arc, not the chord.
    pub fn highlight_arc(&mut self, a: &'static str, b: &'static str, center: &str) -> &mut Self {
        let id = format!("hl-{}{}", a.to_ascii_lowercase(), b.to_ascii_lowercase());
        let c = self.at(center);
        let pa = self.at(a);
        let pb = self.at(b);
        let via = minor_via(c, pa, pb);
        self.fig.arc(&id, pa, pb, via);
        self.named.push(Circ {
            id,
            letters: format!("{a}{b}").to_ascii_uppercase(),
        });
        self
    }

    /// Consecutive segments along a produced line.
    pub fn chain(&mut self, names: &[&'static str]) -> &mut Self {
        for w in names.windows(2) {
            self.join(w[0], w[1]);
        }
        self
    }

    /// The given straight-line of the proposition (drawn even if later
    /// construction lines share its ends).
    pub fn base(&mut self, a: &'static str, b: &'static str) -> &mut Self {
        self.join(a, b)
    }

    /// Circle named by three letters, center `center`, through `through`.
    pub fn circle(&mut self, letters: &'static str, center: &str, through: &str) -> &mut Self {
        let c = self.at(center);
        self.circle_r(letters, center, c.dist(self.at(through)))
    }

    /// Circle named by three letters, center `center`, given radius.
    /// The plate's circle is not always the compass circle through the named point.
    pub fn circle_r(&mut self, letters: &'static str, center: &str, r: f64) -> &mut Self {
        let c = self.at(center);
        let id = format!("circ-{}", letters.to_ascii_lowercase());
        self.fig.circle(&id, c, r);
        self.circles.push(Circ {
            id,
            letters: letters.to_ascii_uppercase(),
        });
        self
    }

    /// Decorative circular arc `a` → `b` through `via` (not a spoken name).
    pub fn arc(&mut self, a: &str, b: &str, via: V2) -> &mut Self {
        let id = format!("arc-{}{}", a.to_ascii_lowercase(), b.to_ascii_lowercase());
        self.fig.arc(&id, self.at(a), self.at(b), via);
        self
    }

    

    /// Circular bow on chord `a`–`b`. `sag` is signed along the left normal
    /// of `a` → `b` (y-up). Negative sag hangs **below** a left-to-right chord.
    pub fn bow(&mut self, a: &str, b: &str, sag: f64) -> &mut Self {
        let pa = self.at(a);
        let pb = self.at(b);
        let mid = pa.add(pb.sub(pa).mul(0.5));
        let n = V2::new(pa.y - pb.y, pb.x - pa.x).unit();
        self.arc(a, b, mid.add(n.mul(sag)))
    }

    pub fn clip(&mut self, min: V2, max: V2) -> &mut Self {
        self.fig.clip_box(min, max);
        self
    }

    /// Ids of the drawn strokes (`ae`, `ek`, …), one per undirected stroke.
    pub fn stroke_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for edges in self.adj.values() {
            for e in edges {
                if !ids.contains(&e.id) {
                    ids.push(e.id.clone());
                }
            }
        }
        ids.sort();
        ids
    }

    pub fn svg<S: AsRef<str>>(&self, on: &[S]) -> String {
        self.fig.svg(on)
    }

    /// Figure ids to light for a spoken geometry word (`A`, `AL`, `DAB`, `CGH`).
    pub fn highlight(&self, word: &str) -> Vec<String> {
        self.hit(word, false)
    }

    /// ∠BAC lights rays BA and AC at vertex A — not the opposite side.
    pub fn highlight_angle(&self, word: &str) -> Vec<String> {
        self.hit(word, true)
    }

    fn hit(&self, word: &str, angle: bool) -> Vec<String> {
        let key: String = word
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        if key.is_empty() {
            return Vec::new();
        }
        let letters: Vec<&'static str> = key.chars().filter_map(|c| self.letter(c)).collect();
        if letters.len() != key.len() {
            return Vec::new();
        }
        match letters.len() {
            // "the angle at *G*" is the angle, not the point (IV.11).
            1 if angle => self.angle_at(letters[0]),
            1 => self.single(letters[0], &key),
            2 => self.pair(letters[0], letters[1]),
            3 if angle => self.angle(letters[0], letters[1], letters[2], &key),
            3 => self.triple(letters[0], letters[1], letters[2], &key),
            4 if !angle => self.quad(letters[0], letters[1], letters[2], letters[3]),
            n if n >= 5 && !angle => self.polygon(&letters),
            _ => Vec::new(),
        }
    }

    fn letter(&self, c: char) -> Option<&'static str> {
        let u = c.to_ascii_uppercase();
        self.pts
            .keys()
            .copied()
            .find(|n| n.len() == 1 && n.starts_with(u))
    }

    /// The angle at a single letter: every stroke that meets there.
    ///
    /// ∠G of triangle FGH is the two sides. A given mark (*D*, *E*) keeps its
    /// own ink. A point with no stroke is still just the point.
    fn angle_at(&self, v: &'static str) -> Vec<String> {
        if self.named_id(v).is_some() {
            return self.single(v, v);
        }
        let mut out = vec![v.to_string()];
        if let Some(edges) = self.adj.get(v) {
            for e in edges {
                out.push(e.to.to_string());
                out.push(e.id.clone());
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn single(&self, a: &'static str, key: &str) -> Vec<String> {
        let mut out = vec![a.to_string()];
        if let Some(id) = self.named_id(key) {
            out.push(id.clone());
            if let Some(edges) = self.adj.get(a) {
                for e in edges {
                    if e.id == id {
                        out.push(e.to.to_string());
                    }
                }
            }
        }
        out
    }

    fn pair(&self, a: &'static str, b: &'static str) -> Vec<String> {
        let mut out = vec![a.to_string(), b.to_string()];
        out.extend(self.link(a, b));
        out
    }

    /// The ink a two-letter name lights.
    ///
    /// The drawn straight-line wins: a direct stroke, then a collinear chain
    /// (A–E–B for AB). A registered opposite-corner name (*EH*, *BK*) is the
    /// boundary, not a path across the diagonal. A bent spoken line (*GC* as
    /// G–A–C) is that chain, not another walk of the same length. A corner
    /// that only lies on a longer stroke (K on AC) still names that stroke —
    /// a walk around the figure would light the wrong piece. Anything else
    /// is the graph path.
    fn link(&self, a: &'static str, b: &'static str) -> Vec<String> {
        if let Some(ids) = self.straight(a, b) {
            return ids;
        }
        if let Some(id) = self.edge_id(a, b) {
            return vec![id];
        }
        if let Some(ids) = self.region_sides(a, b) {
            return ids;
        }
        if let Some(ids) = self.along_sides(a, b) {
            return ids;
        }
        if let Some(id) = self.stroke_through(a, b).or_else(|| self.stroke_through(b, a)) {
            return vec![id];
        }
        self.bfs(a, b)
    }

    /// Strokes of a bent spoken line (`GC` as G–A–C), not another walk.
    fn along_sides(&self, a: &'static str, b: &'static str) -> Option<Vec<String>> {
        let want = sorted_letters(&format!("{a}{b}"));
        let poly = self.polys.iter().find(|p| sorted_letters(&p.letters) == want)?;
        let pts = &poly.points;
        if pts.len() < 2 {
            return None;
        }
        let ends = (*pts.first().unwrap() == a && *pts.last().unwrap() == b)
            || (*pts.first().unwrap() == b && *pts.last().unwrap() == a);
        if !ends {
            return None;
        }
        let mut out = Vec::new();
        for w in pts.windows(2) {
            if let Some(ids) = self.straight(w[0], w[1]) {
                out.extend(ids);
            } else if let Some(id) = self.edge_id(w[0], w[1]) {
                out.push(id);
            } else {
                return None;
            }
        }
        out.sort();
        out.dedup();
        Some(out)
    }

    /// Boundary strokes of the parallelogram named by opposite corners `a`, `b`.
    fn region_sides(&self, a: &'static str, b: &'static str) -> Option<Vec<String>> {
        let want = sorted_letters(&format!("{a}{b}"));
        let region = self.regions.iter().find(|r| sorted_letters(&r.letters) == want)?;
        let n = region.boundary.len();
        if n < 3 {
            return None;
        }
        let mut out = Vec::new();
        for i in 0..n {
            out.extend(self.side(region.boundary[i], region.boundary[(i + 1) % n]));
        }
        out.sort();
        out.dedup();
        Some(out)
    }

    /// `pair`, plus a corner that only lies on a longer stroke (G on DC).
    /// A triangle must not take this shortcut: G on AB is not side AG when
    /// AG is drawn separately (I.23).
    fn pair_on(&self, a: &'static str, b: &'static str) -> Vec<String> {
        let mut out = vec![a.to_string(), b.to_string()];
        if let Some(id) = self.stroke_through(a, b).or_else(|| self.stroke_through(b, a)) {
            out.push(id);
            return out;
        }
        out.extend(self.path(a, b));
        out
    }

    fn triple(&self, a: &'static str, b: &'static str, c: &'static str, key: &str) -> Vec<String> {
        if let Some(circ) = self.named_circle(key) {
            return vec![
                circ,
                a.to_string(),
                b.to_string(),
                c.to_string(),
            ];
        }
        let mut out = self.pair(a, b);
        out.extend(self.pair(b, c));
        out.extend(self.pair(c, a));
        out.sort();
        out.dedup();
        out
    }

    /// A five-or-more letter figure (*ABCDE* the pentagon) is the polygon of
    /// those vertices in spoken order. The closing side is the last letter
    /// back to the first. A circumference of the same letters is an `arc`,
    /// not this walk.
    fn polygon(&self, letters: &[&'static str]) -> Vec<String> {
        let n = letters.len();
        let mut out = Vec::new();
        for i in 0..n {
            out.extend(self.side(letters[i], letters[(i + 1) % n]));
        }
        out.sort();
        out.dedup();
        out
    }

    /// `*circle(ABCDE)*` lights the named circle, not the pentagon.
    pub fn circle_highlight(&self, word: &str) -> Vec<String> {
        let key = letter_key(word);
        if let Some(id) = self.named_circle(&key) {
            let mut out = vec![id];
            for ch in key.chars() {
                if let Some(name) = self.letter(ch) {
                    out.push(name.to_string());
                }
            }
            return out;
        }
        self.highlight(word)
    }

    /// `*arc(ABCD)*` lights the circumference in letter order.
    ///
    /// A two-letter arc that was drawn on the circle (`hl-ab`) lights that
    /// arc, not the chord. A longer name chains those arcs. Anything else
    /// is the drawn path, not closed.
    pub fn arc_highlight(&self, word: &str) -> Vec<String> {
        let key = letter_key(word);
        let letters: Vec<&'static str> = key.chars().filter_map(|c| self.letter(c)).collect();
        if letters.len() != key.len() || letters.len() < 2 {
            return Vec::new();
        }
        let mut out = Vec::new();
        for w in letters.windows(2) {
            if let Some(id) = self.named_arc(w[0], w[1]) {
                out.push(w[0].to_string());
                out.push(w[1].to_string());
                out.push(id);
            } else {
                out.extend(self.side(w[0], w[1]));
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn named_arc(&self, a: &str, b: &str) -> Option<String> {
        let want = sorted_letters(&format!("{a}{b}"));
        self.named.iter().find_map(|c| {
            (c.id.starts_with("hl-") && sorted_letters(&c.letters) == want).then(|| c.id.clone())
        })
    }
    /// A four-letter figure name (*ACDB*, trapezium *ABGD*) is the polygon of
    /// those vertices in order. A corner that only lies on a stroke (G, the
    /// crossing EB × DC) still names that stroke, so ABGD turns at G instead
    /// of taking the diagonal shortcut through the interior.
    fn quad(
        &self,
        a: &'static str,
        b: &'static str,
        c: &'static str,
        d: &'static str,
    ) -> Vec<String> {
        let mut out = self.side(a, b);
        out.extend(self.side(b, c));
        out.extend(self.side(c, d));
        out.extend(self.side(d, a));
        out.sort();
        out.dedup();
        out
    }

    /// One side of a named polygon, from `a` to `b`.
    ///
    /// The spoken straight-line wins when it is drawn (AF along A–D–E–F, or
    /// a single stroke AB). A corner that only lies on a longer stroke (G on
    /// DC, G on EB) still names that stroke. Anything else is the graph path.
    fn side(&self, a: &'static str, b: &'static str) -> Vec<String> {
        if let Some(ids) = self.straight(a, b) {
            let mut out = vec![a.to_string(), b.to_string()];
            out.extend(ids);
            return out;
        }
        if self.edge_id(a, b).is_some() {
            return self.pair(a, b);
        }
        if self.stroke_through(a, b).is_some() || self.stroke_through(b, a).is_some() {
            return self.pair_on(a, b);
        }
        self.pair(a, b)
    }

    fn angle(&self, a: &'static str, v: &'static str, c: &'static str, key: &str) -> Vec<String> {
        if let Some(circ) = self.named_circle(key) {
            return vec![circ, a.to_string(), v.to_string(), c.to_string()];
        }
        // The two rays from the vertex. A chain that detours through
        // another point (A–E–C when E lies on AC) is the triangle, not
        // the angle. A named end that is not on a ray from the vertex
        // (∠EFG: the plate has no FG) still names the ray that leaves the
        // vertex toward that point (FD–DG).
        let mut out = self.angle_ray(v, a);
        out.extend(self.angle_ray(v, c));
        if !self.ray_reaches(v, a) {
            out.extend(self.ray_toward(v, a));
        }
        if !self.ray_reaches(v, c) {
            out.extend(self.ray_toward(v, c));
        }
        out.sort();
        out.dedup();
        out
    }

    fn ray_reaches(&self, v: &'static str, end: &'static str) -> bool {
        self.straight(v, end).is_some() || self.edge_id(v, end).is_some()
    }

    /// The drawn ray that leaves `v` toward `end` when `end` itself is not
    /// on a stroke from `v`. Prefer the neighbour of `v` whose direction is
    /// closest to `end`, then keep walking the drawn chain while each next
    /// stroke still heads toward `end` (F→D→G — the plate bends at D).
    fn ray_toward(&self, v: &'static str, end: &'static str) -> Vec<String> {
        let pv = self.at(v);
        let pe = self.at(end);
        let want = pe.sub(pv);
        let Some(edges) = self.adj.get(v) else {
            return vec![v.to_string()];
        };
        let best = edges.iter().filter_map(|e| {
            let d = self.at(e.to).sub(pv);
            let cross = want.x * d.y - want.y * d.x;
            let dot = want.dot(d);
            (dot > 0.0).then_some((cross.abs() / dot, e.to))
        }).min_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        let Some((_, name)) = best else {
            return vec![v.to_string()];
        };
        let mut ids = self.angle_ray(v, name);
        let mut cur = name;
        loop {
            let here = self.at(cur);
            let Some(nexts) = self.adj.get(cur) else { break };
            let step = nexts.iter().filter(|e| !ids.iter().any(|id| id == e.to)).filter_map(|e| {
                let d = self.at(e.to).sub(here);
                let aim = pe.sub(here);
                let dot = aim.dot(d);
                (dot > 1e-6).then_some((
                    (aim.x * d.y - aim.y * d.x).abs() / dot,
                    e.to,
                    e.id.clone(),
                ))
            }).min_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
            // Stop once the chain has bent away from the named point.
            let Some((err, to, id)) = step else { break };
            if err > 0.4 {
                break;
            }
            ids.push(to.to_string());
            ids.push(id);
            if to == end {
                break;
            }
            cur = to;
        }
        ids
    }

    /// Edges from `v` toward `end`: the collinear segment if one exists,
    /// otherwise only a direct edge. Never a path around the figure.
    fn angle_ray(&self, v: &'static str, end: &'static str) -> Vec<String> {
        let mut out = vec![v.to_string()];
        if let Some(ids) = self.straight(v, end) {
            out.push(end.to_string());
            out.extend(ids);
            return out;
        }
        if let Some(id) = self.edge_id(v, end) {
            out.push(end.to_string());
            out.push(id);
        }
        out
    }

    fn named_circle(&self, key: &str) -> Option<String> {
        self.named_in(&self.circles, key)
    }

    fn named_id(&self, key: &str) -> Option<String> {
        self.named_in(&self.named, key)
    }

    fn named_in(&self, xs: &[Circ], key: &str) -> Option<String> {
        let want = sorted_letters(key);
        xs.iter()
            .find_map(|c| (sorted_letters(&c.letters) == want).then_some(c.id.clone()))
    }

    fn path(&self, start: &'static str, goal: &'static str) -> Vec<String> {
        if start == goal {
            return Vec::new();
        }
        if let Some(ids) = self.straight(start, goal) {
            return ids;
        }
        self.bfs(start, goal)
    }

    /// Segments along the spoken straight-line (collinear named points).
    fn straight(&self, start: &'static str, goal: &'static str) -> Option<Vec<String>> {
        let a = self.at(start);
        let b = self.at(goal);
        let mut on: Vec<(&'static str, f64)> = self
            .pts
            .iter()
            .filter(|(n, p)| n.len() == 1 && p.on_seg(a, b, 1e-4))
            .map(|(n, p)| (*n, a.dist(*p)))
            .collect();
        on.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap());
        if on.len() < 2 || on[0].0 != start || on.last().map(|x| x.0) != Some(goal) {
            return None;
        }
        // A longer collinear chain (B–C–E) still names the short piece (C–E)
        // when every step of it is drawn.
        let mut ids = Vec::new();
        for w in on.windows(2) {
            let id = self.edge_id(w[0].0, w[1].0)?;
            ids.push(id);
        }
        Some(ids)
    }

    fn edge_id(&self, a: &str, b: &str) -> Option<String> {
        self.adj
            .get(a)
            .and_then(|es| es.iter().find(|e| e.to == b).map(|e| e.id.clone()))
    }

    /// Id of the drawn stroke that runs through `b` with `a` as an end.
    ///
    /// G in I.35 is the crossing EB × DC, not an end, so GB has no edge of
    /// its own — but G lies on stroke `eb`, and that stroke is what the
    /// trapezium lights. A merely collinear neighbour is not a hit: `b` must
    /// sit on the segment that was actually drawn.
    fn stroke_through(&self, a: &str, b: &str) -> Option<String> {
        let pb = self.at(b);
        self.adj.get(a)?.iter().find_map(|e| {
            (e.to != b && pb.on_seg(self.at(a), self.at(e.to), 1e-3)).then(|| e.id.clone())
        })
    }

    fn bfs(&self, start: &'static str, goal: &'static str) -> Vec<String> {
        if start == goal {
            return Vec::new();
        }
        let mut prev: HashMap<&str, (&str, String)> = HashMap::new();
        let mut q = VecDeque::from([start]);
        let mut seen = vec![start];
        while let Some(cur) = q.pop_front() {
            if cur == goal {
                break;
            }
            let Some(edges) = self.adj.get(cur) else {
                continue;
            };
            for e in edges {
                if seen.contains(&e.to) {
                    continue;
                }
                seen.push(e.to);
                prev.insert(e.to, (cur, e.id.clone()));
                q.push_back(e.to);
            }
        }
        let mut ids = Vec::new();
        let mut cur = goal;
        while let Some((p, id)) = prev.get(cur) {
            ids.push(id.clone());
            cur = p;
            if cur == start {
                break;
            }
        }
        ids
    }
}

fn sorted_letters(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v.sort_unstable();
    v.into_iter().collect()
}

fn letter_key(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Midpoint of the minor arc from `a` to `b` about `c`.
fn minor_via(c: V2, a: V2, b: V2) -> V2 {
    let ua = a.sub(c).unit();
    let ub = b.sub(c).unit();
    let sum = ua.add(ub);
    let dir = if sum.len() < 1e-6 {
        V2::new(-ua.y, ua.x)
    } else {
        sum.unit()
    };
    c.add(dir.mul(c.dist(a)))
}

#[cfg(test)]
mod tests {
    use crate::figure::{
        book1_prop1, book1_prop2, book1_prop3, book1_prop4, book1_prop5, book1_prop6,
        book1_prop7, book1_prop8, book1_prop9, book1_prop10, book1_prop11, book1_prop12,
        book1_prop13, book1_prop14, book1_prop15, book1_prop16, book1_prop17,
        book1_prop18, book1_prop19, book1_prop20, book1_prop21, book1_prop22,
        book1_prop23, book1_prop24, book1_prop25, book1_prop26, book1_prop27,
        book1_prop28, book1_prop29, book1_prop30, book1_prop31, book1_prop32,
        book1_prop33, book1_prop34, book1_prop35, book1_prop36,
        book1_prop37, book1_prop38, book1_prop39, book1_prop40,
        book1_prop41, book1_prop42, book1_prop43, book1_prop44,
        book1_prop45, book1_prop46, book1_prop47, book1_prop48,
    };
    use crate::figure::book2_prop1;
    use crate::figure::book2_prop2;
    use crate::figure::book2_prop3;

    fn has(v: &[String], id: &str) -> bool {
        v.iter().any(|s| s == id)
    }

    #[test]
    fn prop1_circle_and_line_letters() {
        let d = book1_prop1();
        let a = d.at("A");
        let b = d.at("B");
        let dpt = d.at("D");
        let e = d.at("E");
        let r = a.dist(b);
        assert!((a.dist(dpt) - r).abs() < 1e-6, "D on circle BCD");
        assert!((b.dist(e) - r).abs() < 1e-6, "E on circle ACE");
        assert!(dpt.x < a.x && (dpt.y - a.y).abs() < 1e-6, "D is the left cut");
        assert!(e.x > b.x && (e.y - b.y).abs() < 1e-6, "E is the right cut");
        let bcd = d.highlight("BCD");
        assert!(has(&bcd, "circ-bcd") && has(&bcd, "D"));
        let ace = d.highlight("ACE");
        assert!(has(&ace, "circ-ace") && has(&ace, "E"));
        let html = d.svg(&[] as &[String]);
        assert!(html.contains("data-id=\"D\"") && html.contains("data-id=\"E\""));
        assert!(html.contains("data-id=\"C\""));
    }

    #[test]
    fn prop2_paths() {
        let d = book1_prop2();
        let ae = d.highlight("AE");
        assert!(has(&ae, "A") && has(&ae, "E"));
        assert!(has(&ae, "al") && has(&ae, "le"));
        let dl = d.highlight("DL");
        assert!(has(&dl, "da") && has(&dl, "al"));
        let cgh = d.highlight("CGH");
        assert!(has(&cgh, "circ-cgh"));
        assert!(has(&cgh, "C") && has(&cgh, "G") && has(&cgh, "H"));
        let dab = d.highlight("DAB");
        assert!(has(&dab, "da") && has(&dab, "ab") && has(&dab, "db"));
        let da = d.at("D").dist(d.at("A"));
        let db = d.at("D").dist(d.at("B"));
        let bc = d.at("B").dist(d.at("C"));
        assert!((db / da - 1.0249).abs() < 0.001, "DB/DA from the plate");
        assert!((bc / da - 2.4635).abs() < 0.002, "BC/DA from the plate");
        let adb = angle_at(&d, "A", "D", "B").to_degrees();
        assert!((adb - 53.92).abs() < 0.05, "∠ADB from the plate, got {adb}");
        assert!((d.at("D").dist(d.at("L")) - d.at("D").dist(d.at("G"))).abs() < 0.05, "DL = DG");
        assert!((d.at("B").dist(d.at("G")) - bc).abs() < 0.05, "BG = BC");
        let bc_head = {
            let v = d.at("C").sub(d.at("B"));
            v.y.atan2(v.x).to_degrees()
        };
        assert!((bc_head - 90.0).abs() < 0.05, "BC vertical, got {bc_head}");
        assert!((d.at("B").x - d.at("C").x).abs() < 1e-6, "B and C share x");
    }

    #[test]
    fn prop3_paths() {
        let d = book1_prop3();
        let c = d.highlight("C");
        assert!(has(&c, "C") && has(&c, "seg-c"));
        let ae = d.highlight("AE");
        assert!(has(&ae, "ae") && has(&ae, "A") && has(&ae, "E"));
        let def = d.highlight("DEF");
        assert!(has(&def, "circ-def"));
        assert!(has(&def, "D") && has(&def, "E") && has(&def, "F"));
        let ad = d.highlight("AD");
        assert!(has(&ad, "ad"));
        let a = d.at("A");
        let b = d.at("B");
        let e = d.at("E");
        let left = d.at("Cleft");
        let right = d.at("Cright");
        let c = d.at("C");
        assert!((left.y - right.y).abs() < 1e-9, "C is horizontal");
        assert!((left.dist(right) / a.dist(e) - 1.024).abs() < 0.002, "C ≈ AD on the plate");
        assert!(left.y > d.at("D").y, "C sits above the circle");
        assert!(c.y == left.y && c.x > left.x && c.x < right.x);
        assert!(left.x < a.x, "C starts left of A");
        assert!(right.x < e.x, "C stays over the left of the circle");
        assert!(right.x < b.x, "C is not past B");
        let dd = d.at("D");
        let ang = (dd.y - a.y).atan2(dd.x - a.x).to_degrees();
        assert!(
            (ang - 132.14).abs() < 0.5,
            "angle DAE (AD from AE) should be 132.14°, got {ang}"
        );
        let fpt = d.at("F");
        let fang = (fpt.y - a.y).atan2(fpt.x - a.x).to_degrees();
        assert!((fang + 56.22).abs() < 0.5, "F at −56° on the circle, got {fang}");
        assert!((b.x / a.dist(e) - 2.189).abs() < 0.01, "AB/AD from the plate");
    }

    fn angle_at(d: &crate::figure::Diagram, p: &str, v: &str, q: &str) -> f64 {
        let a = d.at(p).sub(d.at(v)).unit();
        let b = d.at(q).sub(d.at(v)).unit();
        a.dot(b).clamp(-1.0, 1.0).acos()
    }

    #[test]
    fn prop4_paths() {
        let d = book1_prop4();
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ab") && has(&abc, "bc") && has(&abc, "ca"));
        let def = d.highlight("DEF");
        assert!(has(&def, "de") && has(&def, "ef") && has(&def, "fd"));
        let bac = d.highlight_angle("BAC");
        assert!(has(&bac, "ab") && has(&bac, "ca") && has(&bac, "A"));
        assert!(!has(&bac, "bc"), "an angle is not the whole triangle");
        let edf = d.highlight_angle("EDF");
        assert!(has(&edf, "de") && has(&edf, "fd"));
        assert!(!has(&edf, "ef"));
        let abc_ang = d.highlight_angle("ABC");
        assert!(has(&abc_ang, "ab") && has(&abc_ang, "bc") && !has(&abc_ang, "ca"));
        let ab = d.highlight("AB");
        assert!(has(&ab, "ab") && has(&ab, "A") && has(&ab, "B"));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let e = d.at("E");
        let f = d.at("F");
        let dd = d.at("D");
        assert!((a.dist(b) - dd.dist(e)).abs() < 1e-6, "AB = DE");
        assert!((a.dist(c) - dd.dist(f)).abs() < 1e-6, "AC = DF");
        assert!((b.dist(c) - e.dist(f)).abs() < 1e-6, "BC = EF");
        assert!(
            (angle_at(&d, "B", "A", "C") - angle_at(&d, "E", "D", "F")).abs() < 1e-9,
            "∠BAC = ∠EDF"
        );
        assert!((b.y - e.y).abs() < 1e-9 && (c.y - f.y).abs() < 1e-9, "one baseline");
        assert!((a.y - dd.y).abs() < 1e-9, "apexes A and D level");
        assert!(c.x < e.x, "a gap between the triangles");
        assert!(
            ((e.x - c.x) / (c.x - b.x) - 0.364).abs() < 0.01,
            "CE/BC from the plate, got {}",
            (e.x - c.x) / (c.x - b.x)
        );
        assert!(a.y > b.y && dd.y > e.y, "peaks A and D above the bases");
        let bc = c.x - b.x;
        let a_along = (a.x - b.x) / bc;
        let d_along = (dd.x - e.x) / (f.x - e.x);
        assert!(
            (a_along - 0.705).abs() < 0.01 && (d_along - 0.705).abs() < 0.01,
            "A toward C, D toward F (Fitzpatrick), got {a_along} {d_along}"
        );
        let h = a.y - b.y;
        assert!(
            (h / bc - 0.864).abs() < 0.01,
            "Fitzpatrick I.4 height, got {}",
            h / bc
        );
        let html = d.svg(&[] as &[String]);
        assert!(html.contains("class=\"arc\""), "bow under EF");
        assert!(html.contains("<path"), "arc is an SVG path");
    }

    #[test]
    fn prop5_paths() {
        let d = book1_prop5();
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ab") && has(&abc, "bc") && has(&abc, "ac"));
        let abc_ang = d.highlight_angle("ABC");
        assert!(has(&abc_ang, "ab") && has(&abc_ang, "bc") && !has(&abc_ang, "ac"));
        let cbd = d.highlight_angle("CBD");
        assert!(has(&cbd, "bc") && (has(&cbd, "bf") || has(&cbd, "bd")));
        assert!(!has(&cbd, "ab"), "∠CBD is under the base, not ∠ABC");
        let af = d.highlight("AF");
        assert!(has(&af, "ab") && has(&af, "bf"));
        let ag = d.highlight("AG");
        assert!(has(&ag, "ac") && has(&ag, "cg"));
        let afc = d.highlight("AFC");
        assert!(has(&afc, "fc") && has(&afc, "ac"));
        let bfc = d.highlight("BFC");
        assert!(has(&bfc, "bf") && has(&bfc, "fc") && has(&bfc, "bc"));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        assert!((a.dist(b) - a.dist(c)).abs() < 1e-6, "AB = AC");
        assert!((a.dist(d.at("F")) - a.dist(d.at("G"))).abs() < 1e-6, "AF = AG");
        assert!((b.dist(d.at("F")) - c.dist(d.at("G"))).abs() < 1e-6, "BF = CG");
        assert!(d.at("F").y < b.y && d.at("D").y < d.at("F").y, "D beyond F beyond B");
        assert!(d.at("G").y < c.y && d.at("E").y < d.at("G").y, "E beyond G beyond C");
        assert!((b.y - c.y).abs() < 1e-9, "BC horizontal");
        assert!((a.x - (b.x + c.x) / 2.0).abs() < 1e-6, "isosceles on the plate");
        let bac = angle_at(&d, "B", "A", "C").to_degrees();
        assert!((bac - 43.43).abs() < 0.05, "∠BAC from the plate, got {bac}");
        let bg_cf = {
            let u = d.at("G").sub(d.at("B")).unit();
            let v = d.at("F").sub(d.at("C")).unit();
            let a = u.dot(v).abs().clamp(-1.0, 1.0).acos().to_degrees();
            a.min(180.0 - a)
        };
        assert!(
            (bg_cf - 39.4).abs() < 0.3,
            "BG and CF meet under the base at 39.4°, got {bg_cf}"
        );
        let ab = a.dist(b);
        let bf = b.dist(d.at("F"));
        let fd = d.at("F").dist(d.at("D"));
        assert!((bf / ab - 0.333).abs() < 0.004, "BF/AB from the plate, got {}", bf / ab);
        assert!((fd / ab - 0.542).abs() < 0.004, "FD/AB from the plate, got {}", fd / ab);
        assert!((a.dist(d.at("F")) / ab - 1.333).abs() < 0.004, "AF/AB");
        assert!((a.dist(d.at("D")) / ab - 1.875).abs() < 0.004, "AD/AB");
        assert!((a.dist(d.at("G")) / a.dist(c) - 1.333).abs() < 0.004, "AG/AC");
        assert!((a.dist(d.at("E")) / a.dist(c) - 1.875).abs() < 0.004, "AE/AC");
    }

    #[test]
    fn prop6_paths() {
        let d = book1_prop6();
        let abc = d.highlight("ABC");
        assert!(
            has(&abc, "ad") && has(&abc, "db") && has(&abc, "bc") && (has(&abc, "ac") || has(&abc, "ca")),
            "ABC is A–D–B–C: {abc:?}"
        );
        let ab = d.highlight("AB");
        assert!(has(&ab, "ad") && has(&ab, "db"));
        let abc_ang = d.highlight_angle("ABC");
        assert!(
            (has(&abc_ang, "db") || has(&abc_ang, "ad"))
                && has(&abc_ang, "bc")
                && !has(&abc_ang, "ac")
        );
        let acb = d.highlight_angle("ACB");
        assert!(
            (has(&acb, "ac") || has(&acb, "ca")) && has(&acb, "bc") && !has(&acb, "ad")
        );
        let dbc = d.highlight_angle("DBC");
        assert!(has(&dbc, "db") && has(&dbc, "bc"));
        let db = d.highlight("DB");
        assert!(has(&db, "db") && has(&db, "D") && has(&db, "B"));
        let dc = d.highlight("DC");
        assert!(has(&dc, "dc"));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        assert!((a.dist(b) - a.dist(c)).abs() < 1e-6, "AB = AC on the plate");
        assert!((b.y - c.y).abs() < 1e-9, "BC horizontal");
        assert!((a.x - (b.x + c.x) / 2.0).abs() < 1e-6, "A on the midline");
        assert!(dd.on_seg(a, b, 1e-4), "D on AB");
        assert!((a.dist(dd) / a.dist(b) - 0.3124).abs() < 0.002, "AD/AB");
        let abc_deg = angle_at(&d, "A", "B", "C").to_degrees();
        let acb_deg = angle_at(&d, "A", "C", "B").to_degrees();
        assert!((abc_deg - acb_deg).abs() < 0.05, "∠ABC = ∠ACB");
    }

    #[test]
    fn prop7_paths() {
        let d = book1_prop7();
        let ac = d.highlight("AC");
        assert!(has(&ac, "ac") || has(&ac, "ca"));
        let ad = d.highlight("AD");
        assert!(has(&ad, "ad") || has(&ad, "da"));
        let cb = d.highlight("CB");
        assert!(has(&cb, "cb") || has(&cb, "bc"));
        let db = d.highlight("DB");
        assert!(has(&db, "db") || has(&db, "bd"));
        let cd = d.highlight("CD");
        assert!(has(&cd, "cd") || has(&cd, "dc"));
        let ab = d.highlight("AB");
        assert!(
            has(&ab, "ab") || has(&ab, "ba"),
            "the given base AB is drawn: {ab:?}"
        );
        let acd = d.highlight_angle("ACD");
        assert!(
            (has(&acd, "ac") || has(&acd, "ca")) && (has(&acd, "cd") || has(&acd, "dc"))
        );
        let adc = d.highlight_angle("ADC");
        assert!(
            (has(&adc, "ad") || has(&adc, "da")) && (has(&adc, "cd") || has(&adc, "dc"))
        );
        let dcb = d.highlight_angle("DCB");
        assert!(
            (has(&dcb, "cd") || has(&dcb, "dc")) && (has(&dcb, "cb") || has(&dcb, "bc"))
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        assert!((a.y - b.y).abs() < 1e-9, "AB horizontal");
        assert!(c.y > a.y && dd.y > a.y, "C and D above AB");
        assert!(c.x > a.x && c.x < b.x && dd.x > c.x && dd.x < b.x);
        assert!(c.y > dd.y, "C higher than D on the plate");
        let ab = a.dist(b);
        assert!(((c.x - a.x) / ab - 0.635).abs() < 0.01, "C along AB");
        assert!(((dd.x - a.x) / ab - 0.865).abs() < 0.01, "D along AB");
    }

    #[test]
    fn prop8_paths() {
        let d = book1_prop8();
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ab") && has(&abc, "bc") && (has(&abc, "ca") || has(&abc, "ac")));
        let def = d.highlight("DEF");
        assert!(has(&def, "de") && has(&def, "ef") && (has(&def, "fd") || has(&def, "df")));
        let eg = d.highlight("EG");
        assert!(has(&eg, "eg") || has(&eg, "ge"));
        let gf = d.highlight("GF");
        assert!(has(&gf, "gf") || has(&gf, "fg"));
        let bac = d.highlight_angle("BAC");
        assert!(has(&bac, "ab") && (has(&bac, "ac") || has(&bac, "ca")) && !has(&bac, "bc"));
        let edf = d.highlight_angle("EDF");
        assert!(has(&edf, "de") && (has(&edf, "df") || has(&edf, "fd")) && !has(&edf, "ef"));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        let f = d.at("F");
        let g = d.at("G");
        assert!((a.dist(b) - dd.dist(e)).abs() < 1e-4, "AB = DE");
        assert!((a.dist(c) - dd.dist(f)).abs() < 1e-4, "AC = DF");
        assert!((b.dist(c) - e.dist(f)).abs() < 1e-4, "BC = EF");
        assert!(c.x > b.x && f.x > e.x && e.x > c.x, "ABC left of DEF");
        assert!(a.y > b.y && dd.y > e.y && g.y > e.y, "apexes and G above the bases");
        assert!(g.x > e.x && g.x < f.x, "G between E and F");
        let bac_deg = angle_at(&d, "B", "A", "C").to_degrees();
        let edf_deg = angle_at(&d, "E", "D", "F").to_degrees();
        assert!((bac_deg - edf_deg).abs() < 0.05, "∠BAC = ∠EDF");
        let bc_head = (c.y - b.y).atan2(c.x - b.x).to_degrees();
        assert!((bc_head - 23.0).abs() < 0.5, "bases tilt ~23° on the plate, got {bc_head}");
    }

    #[test]
    fn prop9_paths() {
        let d = book1_prop9();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ad") && has(&ab, "db"));
        let ac = d.highlight("AC");
        assert!(has(&ac, "ae") && has(&ac, "ec"));
        let ad = d.highlight("AD");
        assert!(has(&ad, "ad"));
        let ae = d.highlight("AE");
        assert!(has(&ae, "ae"));
        let de = d.highlight("DE");
        assert!(has(&de, "de") || has(&de, "ed"));
        let def = d.highlight("DEF");
        assert!(
            (has(&def, "de") || has(&def, "ed"))
                && (has(&def, "ef") || has(&def, "fe"))
                && (has(&def, "fd") || has(&def, "df"))
        );
        let af = d.highlight("AF");
        assert!(has(&af, "af") || has(&af, "fa"));
        let daf = d.highlight_angle("DAF");
        assert!(has(&daf, "ad") && (has(&daf, "af") || has(&daf, "fa")));
        let eaf = d.highlight_angle("EAF");
        assert!(has(&eaf, "ae") && (has(&eaf, "af") || has(&eaf, "fa")));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        let f = d.at("F");
        assert!(dd.on_seg(a, b, 1e-3), "D on AB");
        assert!(e.on_seg(a, c, 1e-3), "E on AC");
        assert!((a.dist(dd) - a.dist(e)).abs() < 0.05, "AD = AE");
        assert!((dd.dist(f) - e.dist(f)).abs() < 0.05, "DF = EF");
        let daf_deg = angle_at(&d, "D", "A", "F").to_degrees();
        let eaf_deg = angle_at(&d, "E", "A", "F").to_degrees();
        assert!((daf_deg - eaf_deg).abs() < 0.05, "AF bisects ∠DAE");
        assert!((f.x - a.x).abs() < 0.05, "AF vertical on the plate");
    }

    #[test]
    fn prop10_paths() {
        let d = book1_prop10();
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        assert!((a.y - b.y).abs() < 1e-9, "AB horizontal");
        assert!(dd.on_seg(a, b, 1e-4) && (a.dist(dd) - dd.dist(b)).abs() < 1e-6);
        assert!((a.dist(c) - b.dist(c)).abs() < 1e-6, "isosceles");
        assert!((c.y / a.dist(b) - 0.7412).abs() < 0.002, "plate height/AB");
        assert!((angle_at(&d, "A", "D", "C").to_degrees() - 90.0).abs() < 0.05);
    }

    #[test]
    fn prop11_paths() {
        let d = book1_prop11();
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        let f = d.at("F");
        let ab = a.dist(b);
        assert!(dd.on_seg(a, b, 1e-4) && c.on_seg(a, b, 1e-4) && e.on_seg(a, b, 1e-4));
        assert!((dd.dist(c) - c.dist(e)).abs() < 1e-6, "CE = CD");
        assert!((dd.dist(e) - dd.dist(f)).abs() < 1e-6, "equilateral FDE");
        assert!((a.dist(dd) / ab - 0.2225).abs() < 0.002, "AD/AB");
        assert!((dd.dist(e) / ab - 0.5651).abs() < 0.002, "DE/AB");
        assert!((e.dist(b) / ab - 0.2124).abs() < 0.002, "EB/AB");
        assert!((angle_at(&d, "D", "C", "F").to_degrees() - 90.0).abs() < 0.05);
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(adx < -0.4 && ady < -0.4, "A above-left of the end, --dx={adx} --dy={ady}");
        assert!(bdx > 0.4 && bdy < -0.4, "B above-right of the end, --dx={bdx} --dy={bdy}");
    }

    #[test]
    fn prop12_paths() {
        let d = book1_prop12();
        let ab = d.highlight("AB");
        assert!(
            has(&ab, "ag") && has(&ab, "gh") && has(&ab, "he") && has(&ab, "eb"),
            "AB is the given line through G, H, E: {ab:?}"
        );
        let efg = d.highlight("EFG");
        assert!(has(&efg, "circ-efg") && has(&efg, "E") && has(&efg, "F") && has(&efg, "G"));
        let eg = d.highlight("EG");
        assert!(has(&eg, "gh") && has(&eg, "he"), "EG is the chord G–H–E: {eg:?}");
        let ch = d.highlight("CH");
        assert!(has(&ch, "ch") || has(&ch, "hc"));
        let chg = d.highlight_angle("CHG");
        assert!(
            (has(&chg, "ch") || has(&chg, "hc")) && has(&chg, "gh") && !has(&chg, "he"),
            "∠CHG is the left adjacent right-angle: {chg:?}"
        );
        let ehc = d.highlight_angle("EHC");
        assert!((has(&ehc, "ch") || has(&ehc, "hc")) && has(&ehc, "he"));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        let g = d.at("G");
        let h = d.at("H");
        assert!((a.y - b.y).abs() < 1e-9, "AB horizontal");
        assert!(c.y > a.y, "C not on AB");
        assert!(dd.y < a.y, "D on the other side of AB");
        assert!(e.on_seg(a, b, 1e-4) && g.on_seg(a, b, 1e-4) && h.on_seg(a, b, 1e-4), "EG on AB");
        assert!((c.dist(dd) - c.dist(e)).abs() < 1e-6, "E on circle through D");
        assert!((c.dist(e) - c.dist(g)).abs() < 1e-6, "G on the same circle");
        assert!((g.dist(h) - h.dist(e)).abs() < 1e-6, "H midpoint of EG");
        assert!((g.dist(e) / a.dist(b) - 0.457).abs() < 0.004, "GE/AB from the plate");
        assert!(dd.y > -8.0, "small cap below GE, D not deep");
        assert!((angle_at(&d, "C", "H", "E").to_degrees() - 90.0).abs() < 0.05);
        assert!((angle_at(&d, "C", "H", "G").to_degrees() - 90.0).abs() < 0.05);
        let html = d.svg(&[] as &[String]);
        let (dx, dy) = letter_offset(&html, "C");
        assert!(dx.abs() < 0.2, "C on the vertical CH, --dx={dx}");
        assert!(dy < -0.5, "C above the mark, same column as F, --dy={dy}");
        let (adx, ady) = letter_offset(&html, "A");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(adx < -0.4 && ady < -0.4, "A above-left of the infinite line, --dx={adx} --dy={ady}");
        assert!(bdx > 0.4 && bdy < -0.4, "B above-right of the infinite line, --dx={bdx} --dy={bdy}");
    }

    #[test]
    fn prop13_paths() {
        let d = book1_prop13();
        let cd = d.highlight("CD");
        assert!(has(&cd, "cb") && has(&cd, "bd"), "CD through B: {cd:?}");
        let ab = d.highlight("AB");
        assert!(has(&ab, "ab") || has(&ab, "ba"));
        let be = d.highlight("BE");
        assert!(has(&be, "be") || has(&be, "eb"));
        let cba = d.highlight_angle("CBA");
        assert!(has(&cba, "cb") && (has(&cba, "ab") || has(&cba, "ba")));
        let abd = d.highlight_angle("ABD");
        assert!((has(&abd, "ab") || has(&abd, "ba")) && has(&abd, "bd"));
        let cbe = d.highlight_angle("CBE");
        assert!(has(&cbe, "cb") && (has(&cbe, "be") || has(&cbe, "eb")));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        assert!((c.y - dd.y).abs() < 1e-9 && (b.y - c.y).abs() < 1e-9, "CD horizontal");
        assert!(b.on_seg(c, dd, 1e-4), "B on CD");
        assert!((angle_at(&d, "C", "B", "E").to_degrees() - 90.0).abs() < 0.05);
        assert!((angle_at(&d, "E", "B", "D").to_degrees() - 90.0).abs() < 0.05);
        let cba_deg = angle_at(&d, "C", "B", "A").to_degrees();
        assert!((cba_deg - 64.01).abs() < 0.05, "∠CBA from the plate, got {cba_deg}");
        assert!((dd.dist(b) / b.dist(c) - 0.6243).abs() < 0.002, "BD/BC");
        assert!((a.dist(b) / b.dist(e) - 1.0361).abs() < 0.002, "AB/BE");
        assert!(e.y > a.y, "BE taller than A, E at the top");
        assert!(dd.x < b.x && b.x < c.x, "plate is D–B–C left to right");
        assert!(a.x > e.x, "AB leans toward C, right of BE");
        let html = d.svg(&[] as &[String]);
        let (dx, dy) = letter_offset(&html, "E");
        assert!(dx.abs() < 0.2, "E centered on BE, --dx={dx}");
        assert!(dy < -0.5, "E sits on the tip of BE, --dy={dy}");
        let (ddx, ddy) = letter_offset(&html, "D");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(ddx < -0.4 && ddy > 0.4, "D below-left of CD, --dx={ddx} --dy={ddy}");
        assert!(cdx > 0.4 && cdy > 0.4, "C below-right of CD, --dx={cdx} --dy={cdy}");
    }

    #[test]
    fn prop14_paths() {
        let d = book1_prop14();
        let cbd = d.highlight("CBD");
        assert!(has(&cbd, "cb") && has(&cbd, "bd"), "CBD through B: {cbd:?}");
        let cbe = d.highlight("CBE");
        assert!(has(&cbe, "cb") && (has(&cbe, "be") || has(&cbe, "eb")));
        let abc = d.highlight_angle("ABC");
        assert!((has(&abc, "ab") || has(&abc, "ba")) && has(&abc, "cb"));
        let abd = d.highlight_angle("ABD");
        assert!((has(&abd, "ab") || has(&abd, "ba")) && has(&abd, "bd"));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        assert!((c.y - b.y).abs() < 1e-9 && (dd.y - b.y).abs() < 1e-9, "CBD horizontal");
        assert!(b.on_seg(c, dd, 1e-4), "B on CD");
        assert!(c.x < b.x && b.x < dd.x, "plate is C–B–D left to right");
        assert!(a.x < e.x, "AB leans toward C, left of BE");
        assert!((a.y - e.y).abs() < 1e-6, "A and E share an ink-row");
        assert!(e.x > dd.x, "E is past D, not the column of D");
        let abc_deg = angle_at(&d, "A", "B", "C").to_degrees();
        assert!((abc_deg - 61.03).abs() < 0.05, "∠ABC from the plate, got {abc_deg}");
        assert!((dd.dist(b) / b.dist(c) - 1.13625).abs() < 0.002, "BD/BC");
        assert!((a.dist(b) / b.dist(c) - 1.6986).abs() < 0.003, "AB/BC");
        assert!((e.dist(b) / b.dist(c) - 1.9070).abs() < 0.003, "BE/BC");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        let (edx, edy) = letter_offset(&html, "E");
        // Fitzpatrick: A and E almost above the tips — not `beyond` BA / BE.
        assert!(adx.abs() < 0.15 && ady < -0.7, "A above the tip, --dx={adx} --dy={ady}");
        assert!(edx.abs() < 0.15 && edy < -0.7, "E above the tip, --dx={edx} --dy={edy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        let (ddx, ddy) = letter_offset(&html, "D");
        // Fitzpatrick: C ~-118°, D ~-63° from the tip — not ±45° corners.
        assert!(cdx < -0.3 && cdy > 0.6, "C below-left of CBD, --dx={cdx} --dy={cdy}");
        assert!(ddx > 0.3 && ddy > 0.6, "D below-right of CBD, --dx={ddx} --dy={ddy}");
    }

    #[test]
    fn prop15_paths() {
        let d = book1_prop15();
        let ab = d.highlight("AB");
        assert!(
            (has(&ab, "ae") || has(&ab, "ea")) && (has(&ab, "eb") || has(&ab, "be")),
            "AB through E: {ab:?}"
        );
        let cd = d.highlight("CD");
        assert!(
            (has(&cd, "de") || has(&cd, "ed")) && (has(&cd, "ec") || has(&cd, "ce")),
            "CD through E: {cd:?}"
        );
        let aec = d.highlight_angle("AEC");
        assert!(
            (has(&aec, "ae") || has(&aec, "ea")) && (has(&aec, "ec") || has(&aec, "ce")),
            "∠AEC: {aec:?}"
        );
        let deb = d.highlight_angle("DEB");
        assert!((has(&deb, "de") || has(&deb, "ed")) && (has(&deb, "eb") || has(&deb, "be")));
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        assert!((dd.y - c.y).abs() < 1e-9 && (e.y - c.y).abs() < 1e-9, "CD horizontal");
        assert!(e.on_seg(c, dd, 1e-4), "E on CD");
        assert!(e.on_seg(a, b, 1e-4), "E on AB");
        assert!(dd.x < e.x && e.x < c.x, "plate is D–E–C left to right");
        assert!(b.x > c.x, "B sits slightly past C, not a letter-column");
        assert!((angle_at(&d, "A", "E", "B").to_degrees() - 180.0).abs() < 0.05);
        assert!((angle_at(&d, "C", "E", "D").to_degrees() - 180.0).abs() < 0.05);
        let aec_deg = angle_at(&d, "A", "E", "C").to_degrees();
        let deb_deg = angle_at(&d, "D", "E", "B").to_degrees();
        assert!((aec_deg - deb_deg).abs() < 0.05, "vertically opposite");
        assert!((c.dist(e) / c.dist(dd) - 0.4995).abs() < 0.003, "CE/CD");
        assert!((a.dist(e) / c.dist(dd) - 0.6722).abs() < 0.004, "AE/CD");
        assert!((e.dist(b) / c.dist(dd) - 0.6825).abs() < 0.004, "EB/CD");
        assert!(a.y > e.y && a.x < e.x, "A above-left of E");
        assert!(b.y < e.y && b.x > e.x, "B below-right of E");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        let (bdx, bdy) = letter_offset(&html, "B");
        // Fitzpatrick: A above-right of the tip, B under the tip — not `beyond` AB.
        assert!(adx > 0.3 && ady < -0.3, "A above-right of the tip, --dx={adx} --dy={ady}");
        assert!(bdx.abs() < 0.15 && bdy > 0.7, "B under the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        let (ddx, ddy) = letter_offset(&html, "D");
        // Fitzpatrick: D ~-117°, C ~-64° from the tip — not ±45° corners.
        assert!(ddx < -0.3 && ddy > 0.6, "D below-left of CD, --dx={ddx} --dy={ddy}");
        assert!(cdx > 0.3 && cdy > 0.6, "C below-right of CD, --dx={cdx} --dy={cdy}");
        let (edx, edy) = letter_offset(&html, "E");
        // --measure 20: E 65.9° (above-right of the crossing), not straight up.
        assert!(edx > 0.2 && edx < 0.5 && edy < -0.6, "E above-right of the crossing, --dx={edx} --dy={edy}");
    }

    #[test]
    fn prop16_paths() {
        let d = book1_prop16();
        let bcd = d.highlight("BCD");
        assert!(has(&bcd, "bc") && has(&bcd, "cd"), "BCD through C: {bcd:?}");
        let acg = d.highlight("ACG");
        assert!(
            (has(&acg, "ae") || has(&acg, "ea")) && has(&acg, "ec") && has(&acg, "cg"),
            "ACG through E, C: {acg:?}"
        );
        let bef = d.highlight("BEF");
        assert!(
            (has(&bef, "be") || has(&bef, "eb")) && (has(&bef, "ef") || has(&bef, "fe")),
            "BEF: {bef:?}"
        );
        let acd = d.highlight_angle("ACD");
        assert!(
            (has(&acd, "ac") || has(&acd, "ca") || has(&acd, "ec") || has(&acd, "ae"))
                && has(&acd, "cd"),
            "∠ACD: {acd:?}"
        );
        let aeb = d.highlight_angle("AEB");
        assert!(
            (has(&aeb, "ae") || has(&aeb, "ea")) && (has(&aeb, "be") || has(&aeb, "eb"))
        );
        let fec = d.highlight_angle("FEC");
        assert!(
            (has(&fec, "ef") || has(&fec, "fe")) && (has(&fec, "ec") || has(&fec, "ce"))
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        let f = d.at("F");
        let g = d.at("G");
        assert!((b.y - c.y).abs() < 1e-9 && (c.y - dd.y).abs() < 1e-9, "BCD horizontal");
        assert!(c.on_seg(b, dd, 1e-4), "C on BD");
        assert!(e.on_seg(a, c, 1e-4), "E on AC");
        assert!((a.dist(e) - e.dist(c)).abs() < 1e-6, "E midpoint of AC");
        assert!(e.on_seg(b, f, 1e-4), "E on BF");
        assert!((b.dist(e) - e.dist(f)).abs() < 1e-6, "EF = BE");
        assert!(c.on_seg(a, g, 1e-4), "C on AG");
        assert!(g.y < c.y, "G below BC");
        assert!((a.dist(b) - f.dist(c)).abs() < 1e-4, "AB = FC");
        assert!((angle_at(&d, "A", "E", "B") - angle_at(&d, "F", "E", "C")).abs() < 1e-6);
        assert!((b.dist(c) / b.dist(dd) - 0.4944).abs() < 0.002, "BC/BD");
        assert!((a.y / b.dist(dd) - 0.5210).abs() < 0.002, "height/BD");
        assert!((c.dist(g) / b.dist(dd) - 0.4365).abs() < 0.003, "CG/BD");
        assert!((a.x - f.x).abs() > 1.0, "F is not the column of A");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        let (fdx, fdy) = letter_offset(&html, "F");
        // Fitzpatrick: A and F almost above the tips.
        assert!(adx.abs() < 0.2 && ady < -0.7, "A above the tip, --dx={adx} --dy={ady}");
        assert!(fdx > 0.4 && fdy < -0.3, "F above-right of the tip, --dx={fdx} --dy={fdy}");
        let (bdx, bdy) = letter_offset(&html, "B");
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(bdx < -0.6 && bdy.abs() < 0.4, "B left of the tip, --dx={bdx} --dy={bdy}");
        assert!(ddx > 0.6 && ddy.abs() < 0.4, "D right of the tip, --dx={ddx} --dy={ddy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        // True vertex BD∩AG: C under the base, left of the junction (not on FC).
        assert!(cdx < -0.25 && cdy > 0.55, "C below-left of the tip, --dx={cdx} --dy={cdy}");
        let (gdx, gdy) = letter_offset(&html, "G");
        // --measure 21: G −8.5° (right of the tip, slightly below) — not under AG.
        assert!(gdx > 0.7 && gdy.abs() < 0.25, "G right of the tip, --dx={gdx} --dy={gdy}");
        let (edx, edy) = letter_offset(&html, "E");
        // Bisector of ∠AEB, not on AC (ink 136.8° sat on the stroke).
        assert!(edx < -0.5 && edy < 0.0, "E in ∠AEB, --dx={edx} --dy={edy}");
    }

    #[test]
    fn prop17_paths() {
        let d = book1_prop17();
        let bcd = d.highlight("BCD");
        assert!(has(&bcd, "bc") && has(&bcd, "cd"), "BCD through C: {bcd:?}");
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba")) && has(&abc, "bc") && (has(&abc, "ac") || has(&abc, "ca")),
            "ABC: {abc:?}"
        );
        let acd = d.highlight_angle("ACD");
        assert!(
            (has(&acd, "ac") || has(&acd, "ca")) && has(&acd, "cd"),
            "∠ACD: {acd:?}"
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        assert!((b.y - c.y).abs() < 1e-9 && (c.y - dd.y).abs() < 1e-9, "BCD horizontal");
        assert!(c.on_seg(b, dd, 1e-4), "C on BD");
        assert!(a.x < b.x, "A sits left of B, not a letter-column");
        assert!((dd.dist(c) / b.dist(c) - 0.3758).abs() < 0.002, "CD/BC");
        assert!((a.dist(b) / b.dist(c) - 1.0681).abs() < 0.002, "AB/BC");
        assert!((a.dist(c) / b.dist(c) - 1.6896).abs() < 0.002, "AC/BC");
        assert!((a.y / b.dist(c) - 1.0067).abs() < 0.003, "height/BC");
        let abc_deg = angle_at(&d, "A", "B", "C").to_degrees();
        let acb_deg = angle_at(&d, "A", "C", "B").to_degrees();
        let acd_deg = angle_at(&d, "A", "C", "D").to_degrees();
        assert!((abc_deg - 109.52).abs() < 0.05, "∠ABC from the plate, got {abc_deg}");
        assert!((acd_deg - 143.43).abs() < 0.05, "∠ACD from the plate, got {acd_deg}");
        assert!(acd_deg > abc_deg, "external greater than opposite interior");
        assert!((acb_deg + acd_deg - 180.0).abs() < 0.05, "ACB + ACD = two right-angles");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx.abs() < 0.15 && ady < -0.7, "A above the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        let (cdx, cdy) = letter_offset(&html, "C");
        let (ddx, ddy) = letter_offset(&html, "D");
        // Under BD (Fitzpatrick lower p. 21), not above the base.
        assert!(bdx < -0.2 && bdy > 0.5, "B below-left of the tip, --dx={bdx} --dy={bdy}");
        assert!(cdx.abs() < 0.15 && cdy > 0.7, "C below the tip, --dx={cdx} --dy={cdy}");
        assert!(ddx > 0.2 && ddy > 0.5, "D below-right of the tip, --dx={ddx} --dy={ddy}");
    }

    #[test]
    fn prop18_paths() {
        let d = book1_prop18();
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba"))
                && has(&abc, "bc")
                && (has(&abc, "ac") || has(&abc, "ca") || has(&abc, "ad") || has(&abc, "dc")),
            "ABC: {abc:?}"
        );
        let adc = d.highlight("ADC");
        assert!(
            (has(&adc, "ad") || has(&adc, "da")) && (has(&adc, "dc") || has(&adc, "cd")),
            "ADC through D: {adc:?}"
        );
        let bcd = d.highlight("BCD");
        assert!(
            has(&bcd, "bc") && (has(&bcd, "bd") || has(&bcd, "db")) && (has(&bcd, "cd") || has(&bcd, "dc")),
            "BCD: {bcd:?}"
        );
        let adb = d.highlight_angle("ADB");
        assert!(
            (has(&adb, "ad") || has(&adb, "da")) && (has(&adb, "bd") || has(&adb, "db")),
            "∠ADB: {adb:?}"
        );
        let dcb = d.highlight_angle("DCB");
        assert!(
            (has(&dcb, "dc") || has(&dcb, "cd")) && has(&dcb, "bc"),
            "∠DCB: {dcb:?}"
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        assert!((b.y - c.y).abs() < 1e-9, "BC horizontal");
        assert!(dd.on_seg(a, c, 1e-4), "D on AC");
        assert!((a.dist(dd) - a.dist(b)).abs() < 1e-6, "AD = AB");
        assert!(a.dist(c) > a.dist(b), "AC > AB");
        assert!((a.dist(b) / b.dist(c) - 0.9403).abs() < 0.002, "AB/BC");
        assert!((a.dist(c) / b.dist(c) - 1.6983).abs() < 0.002, "AC/BC");
        let abc_deg = angle_at(&d, "A", "B", "C").to_degrees();
        let bca_deg = angle_at(&d, "B", "C", "A").to_degrees();
        let adb_deg = angle_at(&d, "A", "D", "B").to_degrees();
        let dcb_deg = angle_at(&d, "D", "C", "B").to_degrees();
        assert!(abc_deg > bca_deg, "greater side subtends greater angle");
        assert!(adb_deg > dcb_deg, "external ADB greater than opposite DCB");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx < -0.1 && ady < -0.6, "A above-left of the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(bdx < -0.4 && bdy > 0.4, "B below-left of the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(cdx > 0.7 && cdy.abs() < 0.2, "C right of the tip, --dx={cdx} --dy={cdy}");
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(ddx > 0.4 && ddy < -0.4, "D above-right of the tip, --dx={ddx} --dy={ddy}");
    }

    #[test]
    fn prop19_paths() {
        let d = book1_prop19();
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba"))
                && (has(&abc, "bc") || has(&abc, "cb"))
                && (has(&abc, "ac") || has(&abc, "ca")),
            "ABC: {abc:?}"
        );
        let abc_ang = d.highlight_angle("ABC");
        assert!(
            (has(&abc_ang, "ab") || has(&abc_ang, "ba")) && (has(&abc_ang, "bc") || has(&abc_ang, "cb")),
            "∠ABC: {abc_ang:?}"
        );
        let bca = d.highlight_angle("BCA");
        assert!(
            (has(&bca, "bc") || has(&bca, "cb")) && (has(&bca, "ac") || has(&bca, "ca")),
            "∠BCA: {bca:?}"
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        assert!((a.x - c.x).abs() < 1e-9, "A on the column of C");
        assert!(a.dist(c) > a.dist(b), "AC > AB");
        assert!((a.dist(b) / a.dist(c) - 0.4855).abs() < 0.002, "AB/AC");
        assert!((b.dist(c) / a.dist(c) - 0.8065).abs() < 0.002, "BC/AC");
        let abc_deg = angle_at(&d, "A", "B", "C").to_degrees();
        let bca_deg = angle_at(&d, "B", "C", "A").to_degrees();
        assert!((abc_deg - 98.36).abs() < 0.05, "∠ABC from the plate, got {abc_deg}");
        assert!((bca_deg - 28.71).abs() < 0.05, "∠BCA from the plate, got {bca_deg}");
        assert!(abc_deg > bca_deg, "greater angle opposite greater side");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx.abs() < 0.15 && ady < -0.7, "A above the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(bdx < -0.6 && bdy < 0.0, "B above-left of the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(cdx.abs() < 0.15 && cdy > 0.7, "C below the tip, --dx={cdx} --dy={cdy}");
    }

    #[test]
    fn prop20_paths() {
        let d = book1_prop20();
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba"))
                && (has(&abc, "bc") || has(&abc, "cb"))
                && (has(&abc, "ac") || has(&abc, "ca")),
            "ABC: {abc:?}"
        );
        let dcb = d.highlight("DCB");
        assert!(
            (has(&dcb, "dc") || has(&dcb, "cd"))
                && (has(&dcb, "cb") || has(&dcb, "bc"))
                && (has(&dcb, "bd") || has(&dcb, "db") || has(&dcb, "ba") || has(&dcb, "ad")),
            "DCB: {dcb:?}"
        );
        let adc = d.highlight_angle("ADC");
        assert!(
            (has(&adc, "ad") || has(&adc, "da")) && (has(&adc, "dc") || has(&adc, "cd")),
            "∠ADC: {adc:?}"
        );
        let acd = d.highlight_angle("ACD");
        assert!(
            (has(&acd, "ac") || has(&acd, "ca")) && (has(&acd, "dc") || has(&acd, "cd")),
            "∠ACD: {acd:?}"
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        assert!((b.y - c.y).abs() < 1e-9, "BC horizontal");
        assert!(a.on_seg(b, dd, 1e-4), "A on BD");
        assert!((a.dist(dd) - a.dist(c)).abs() < 1e-6, "AD = AC");
        assert!(b.dist(a) + a.dist(c) > b.dist(c), "BA + AC > BC");
        assert!((a.dist(b) / b.dist(c) - 0.5089).abs() < 0.002, "AB/BC");
        assert!((a.dist(c) / b.dist(c) - 0.8571).abs() < 0.002, "AC/BC");
        let adc_deg = angle_at(&d, "A", "D", "C").to_degrees();
        let acd_deg = angle_at(&d, "A", "C", "D").to_degrees();
        let bcd_deg = angle_at(&d, "B", "C", "D").to_degrees();
        let bdc_deg = angle_at(&d, "B", "D", "C").to_degrees();
        assert!((adc_deg - acd_deg).abs() < 0.05, "isosceles ADC = ACD");
        assert!(bcd_deg > adc_deg, "BCD greater than ADC");
        assert!(bcd_deg > bdc_deg, "greater angle BCD opposite greater side DB");
        assert!(b.dist(dd) > b.dist(c), "DB > BC");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx < -0.4 && ady < -0.4, "A above-left of the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(bdx < -0.4 && bdy > 0.4, "B below-left of the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(cdx > 0.4 && cdy > 0.4, "C below-right of the tip, --dx={cdx} --dy={cdy}");
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(ddx > 0.2 && ddy < -0.6, "D above-right of the tip, --dx={ddx} --dy={ddy}");
    }

    #[test]
    fn prop21_paths() {
        let d = book1_prop21();
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba"))
                && has(&abc, "bc")
                && (has(&abc, "ac") || has(&abc, "ae") || has(&abc, "ec")),
            "ABC: {abc:?}"
        );
        let bdc = d.highlight_angle("BDC");
        assert!(
            (has(&bdc, "bd") || has(&bdc, "db")) && (has(&bdc, "dc") || has(&bdc, "cd")),
            "∠BDC: {bdc:?}"
        );
        let bac = d.highlight_angle("BAC");
        assert!(
            (has(&bac, "ba") || has(&bac, "ab")) && (has(&bac, "ac") || has(&bac, "ae")),
            "∠BAC: {bac:?}"
        );
        let a = d.at("A");
        let b = d.at("B");
        let c = d.at("C");
        let dd = d.at("D");
        let e = d.at("E");
        assert!((b.y - c.y).abs() < 1e-9, "BC horizontal");
        assert!(e.on_seg(a, c, 1e-4), "E on AC");
        assert!(dd.on_seg(b, e, 1e-4), "D on BE");
        assert!(b.dist(a) + a.dist(c) > b.dist(dd) + dd.dist(c), "BA + AC > BD + DC");
        assert!(
            angle_at(&d, "B", "D", "C").to_degrees() > angle_at(&d, "B", "A", "C").to_degrees(),
            "∠BDC > ∠BAC"
        );
        assert!((a.dist(b) / b.dist(c) - 0.6003).abs() < 0.002, "AB/BC");
        assert!((a.dist(c) / b.dist(c) - 0.8945).abs() < 0.002, "AC/BC");
        assert!((b.dist(dd) / b.dist(c) - 0.4641).abs() < 0.002, "BD/BC");
        assert!((dd.dist(c) / b.dist(c) - 0.7601).abs() < 0.002, "DC/BC");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx.abs() < 0.2 && ady < -0.7, "A above the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(bdx < -0.7 && bdy.abs() < 0.2, "B left of the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(cdx > 0.6 && cdy < 0.3, "C right of the tip, --dx={cdx} --dy={cdy}");
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(ddx < -0.5 && ddy < -0.4, "D above-left of the tip, --dx={ddx} --dy={ddy}");
        let (edx, edy) = letter_offset(&html, "E");
        assert!(edx > 0.4 && edy < -0.4, "E above-right of the tip, --dx={edx} --dy={edy}");
    }

    #[test]
    fn prop22_paths() {
        let d = book1_prop22();
        let a = d.highlight("A");
        assert!(has(&a, "seg-a"), "given line A: {a:?}");
        let de = d.highlight("DE");
        assert!(
            has(&de, "df") && has(&de, "fg") && has(&de, "gh") && has(&de, "he"),
            "DE along the base: {de:?}"
        );
        let dkl = d.highlight("DKL");
        assert!(has(&dkl, "circ-dkl"), "circle DKL: {dkl:?}");
        let klh = d.highlight("KLH");
        assert!(has(&klh, "circ-klh"), "circle KLH: {klh:?}");
        let kfg = d.highlight("KFG");
        assert!(
            (has(&kfg, "kf") || has(&kfg, "fk"))
                && has(&kfg, "fg")
                && (has(&kfg, "kg") || has(&kfg, "gk")),
            "KFG: {kfg:?}"
        );
        assert!((d.at("F").dist(d.at("K")) - 194.18).abs() < 0.05, "K on circle DKL");
        assert!((d.at("G0").dist(d.at("L")) - 105.2).abs() < 0.05, "L on circle KLH");
        assert!((d.at("G0").dist(d.at("K")) - 105.2).abs() < 0.5, "K on circle KLH");
        assert!(d.at("K").y > d.at("D").y && d.at("L").y < d.at("D").y, "K above, L below");
        let df = d.at("D").dist(d.at("F"));
        let fg = d.at("F").dist(d.at("G"));
        let gh = d.at("G").dist(d.at("H"));
        assert!((df - 194.15).abs() < 0.02, "DF from the plate");
        assert!((d.at("F").y - d.at("E").y).abs() < 1e-9, "DFGHE level");
        assert!((fg / df - 167.85 / 194.15).abs() < 0.01, "FG/DF");
        assert!((gh / df - 105.03 / 194.15).abs() < 0.02, "GH/DF");
        assert!(df + fg > gh && df + gh > fg && fg + gh > df, "triangle inequality");
        let html = d.svg(&[] as &[String]);
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(ddx < -0.6 && ddy < -0.3, "D above-left of the tip, --dx={ddx} --dy={ddy}");
        let (fdx, fdy) = letter_offset(&html, "F");
        assert!(fdx < 0.2 && fdy > 0.6, "F below the tip, --dx={fdx} --dy={fdy}");
        let (gdx, gdy) = letter_offset(&html, "G");
        assert!(gdx.abs() < 0.15 && gdy > 0.7, "G below the tip, --dx={gdx} --dy={gdy}");
        let (hdx, hdy) = letter_offset(&html, "H");
        assert!(hdx < -0.4 && hdy > 0.4, "H below-left of the tip, --dx={hdx} --dy={hdy}");
        let (edx, edy) = letter_offset(&html, "E");
        assert!(edx > 0.7 && edy.abs() < 0.15, "E right of the tip, --dx={edx} --dy={edy}");
        let (kdx, kdy) = letter_offset(&html, "K");
        assert!(kdx > 0.15 && kdy < -0.7, "K above the tip, --dx={kdx} --dy={kdy}");
        for id in ["A", "B", "C"] {
            let (dx, dy) = letter_offset(&html, id);
            assert!(
                dx < -0.5 && dy < -0.5,
                "{id} at the left of its stroke, --dx={dx} --dy={dy}"
            );
        }
    }

    #[test]
    fn prop23_paths() {
        let d = book1_prop23();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ab"), "given line AB: {ab:?}");
        let dce = d.highlight_angle("DCE");
        assert!(
            (has(&dce, "cd") || has(&dce, "dc")) && (has(&dce, "ce") || has(&dce, "ec")),
            "∠DCE: {dce:?}"
        );
        let fag = d.highlight_angle("FAG");
        assert!(
            (has(&fag, "af") || has(&fag, "fa")) && (has(&fag, "ag") || has(&fag, "ga")),
            "∠FAG: {fag:?}"
        );
        let afg = d.highlight("AFG");
        assert!(
            (has(&afg, "af") || has(&afg, "fa"))
                && (has(&afg, "fg") || has(&afg, "gf"))
                && (has(&afg, "ag") || has(&afg, "ga")),
            "AFG: {afg:?}"
        );
        let a = d.at("A");
        let b = d.at("B");
        assert!((a.y - b.y).abs() < 1e-9, "AB level");
        assert!(d.at("G").y == a.y, "G on AB");
        assert!(d.at("F").y > a.y, "F above AB");
        assert!(d.at("C").y > a.y && d.at("D").y > d.at("C").y, "angle DCE above");
        let cd = d.at("C").dist(d.at("D"));
        let ce = d.at("C").dist(d.at("E"));
        assert!(
            d.at("C").dist(d.at("D0")) > cd + 10.0 && d.at("C").dist(d.at("D0")) < cd + 25.0,
            "CD produced a short way past D"
        );
        assert!(
            d.at("C").dist(d.at("E0")) > ce + 10.0 && d.at("C").dist(d.at("E0")) < ce + 28.0,
            "CE produced a short way past E"
        );
        let lit = d.svg(&["ab", "A", "B"]);
        let ag = lit.find("data-id=\"ag\"").expect("ag");
        let abp = lit.find("data-id=\"ab\"").expect("ab");
        assert!(
            abp > ag,
            "lit AB must paint after dim AG so the shared ink stays red"
        );
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx < -0.7 && ady.abs() < 0.2, "A left of the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(bdx > 0.1 && bdx < 0.45 && bdy < -0.7, "B above the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(cdx < -0.7 && cdy.abs() < 0.15, "C left of the tip, --dx={cdx} --dy={cdy}");
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(ddx < -0.3 && ddy < -0.5, "D above-left of the tip, --dx={ddx} --dy={ddy}");
        let (edx, edy) = letter_offset(&html, "E");
        assert!(edx < -0.5 && edy > 0.5, "E below-left of the tip, --dx={edx} --dy={edy}");
        let (fdx, fdy) = letter_offset(&html, "F");
        assert!(fdx.abs() < 0.15 && fdy < -0.7, "F above the tip, --dx={fdx} --dy={fdy}");
        let (gdx, gdy) = letter_offset(&html, "G");
        assert!(gdx > 0.5 && gdy < -0.5, "G above-right of the tip, --dx={gdx} --dy={gdy}");
    }

    #[test]
    fn prop24_paths() {
        let d = book1_prop24();
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba"))
                && (has(&abc, "bc") || has(&abc, "cb"))
                && (has(&abc, "ac") || has(&abc, "ca")),
            "ABC: {abc:?}"
        );
        let def = d.highlight("DEF");
        assert!(
            (has(&def, "de") || has(&def, "ed"))
                && (has(&def, "ef") || has(&def, "fe"))
                && (has(&def, "df") || has(&def, "fd")),
            "DEF: {def:?}"
        );
        let bac = d.highlight_angle("BAC");
        assert!(
            (has(&bac, "ba") || has(&bac, "ab")) && (has(&bac, "ac") || has(&bac, "ca")),
            "∠BAC: {bac:?}"
        );
        let edg = d.highlight_angle("EDG");
        assert!(
            (has(&edg, "de") || has(&edg, "ed")) && (has(&edg, "dg") || has(&edg, "gd")),
            "∠EDG: {edg:?}"
        );
        let efg = d.highlight_angle("EFG");
        assert!(
            (has(&efg, "ef") || has(&efg, "fe")) && (has(&efg, "fg") || has(&efg, "gf")),
            "∠EFG: {efg:?}"
        );
        let egf = d.highlight_angle("EGF");
        assert!(
            (has(&egf, "eg") || has(&egf, "ge")) && (has(&egf, "fg") || has(&egf, "gf")),
            "∠EGF: {egf:?}"
        );
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").dist(d.at("B")) / bc - 1.679).abs() < 0.01, "AB/BC");
        assert!((d.at("A").dist(d.at("C")) / bc - 1.939).abs() < 0.01, "AC/BC");
        assert!((d.at("D").dist(d.at("E")) / bc - 1.797).abs() < 0.01, "DE/BC");
        assert!(d.at("E").dist(d.at("G")) > d.at("E").dist(d.at("F")), "EG > EF");
        assert!(d.at("B").dist(d.at("C")) > d.at("E").dist(d.at("F")), "BC > EF");
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(adx.abs() < 0.15 && ady < -0.7, "A above the tip, --dx={adx} --dy={ady}");
        let (bdx, bdy) = letter_offset(&html, "B");
        assert!(bdx > 0.6 && bdy > -0.3, "B right of the tip, --dx={bdx} --dy={bdy}");
        let (cdx, cdy) = letter_offset(&html, "C");
        assert!(cdx.abs() < 0.15 && cdy > 0.7, "C below the tip, --dx={cdx} --dy={cdy}");
        let (ddx, ddy) = letter_offset(&html, "D");
        assert!(ddx < -0.1 && ddx > -0.4 && ddy < -0.7, "D above the tip, --dx={ddx} --dy={ddy}");
        let (edx, edy) = letter_offset(&html, "E");
        assert!(edx > 0.6 && edy > -0.3, "E right of the tip, --dx={edx} --dy={edy}");
        let (fdx, fdy) = letter_offset(&html, "F");
        assert!(fdx > 0.3 && fdy > 0.5, "F below-right of the tip, --dx={fdx} --dy={fdy}");
        let (gdx, gdy) = letter_offset(&html, "G");
        assert!(gdx < -0.5 && gdy > 0.4, "G below-left of the tip, --dx={gdx} --dy={gdy}");
    }

    #[test]
    fn prop25_paths() {
        let d = book1_prop25();
        let abc = d.highlight("ABC");
        assert!(
            (has(&abc, "ab") || has(&abc, "ba"))
                && (has(&abc, "bc") || has(&abc, "cb"))
                && (has(&abc, "ac") || has(&abc, "ca")),
            "ABC: {abc:?}"
        );
        let def = d.highlight("DEF");
        assert!(
            (has(&def, "de") || has(&def, "ed"))
                && (has(&def, "ef") || has(&def, "fe"))
                && (has(&def, "df") || has(&def, "fd")),
            "DEF: {def:?}"
        );
        let bac = d.highlight_angle("BAC");
        assert!(
            (has(&bac, "ba") || has(&bac, "ab")) && (has(&bac, "ac") || has(&bac, "ca")),
            "∠BAC: {bac:?}"
        );
        let edf = d.highlight_angle("EDF");
        assert!(
            (has(&edf, "de") || has(&edf, "ed")) && (has(&edf, "df") || has(&edf, "fd")),
            "∠EDF: {edf:?}"
        );
        assert!(d.at("B").dist(d.at("C")) > d.at("E").dist(d.at("F")));
    }

    #[test]
    fn prop26_paths() {
        let d = book1_prop26();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ag") && has(&ab, "gb"), "AB via G: {ab:?}");
        let bc = d.highlight("BC");
        assert!(has(&bc, "bh") && has(&bc, "hc"), "BC via H: {bc:?}");
        let gc = d.highlight("GC");
        assert!(has(&gc, "gc") || has(&gc, "cg"), "GC: {gc:?}");
        let ah = d.highlight("AH");
        assert!(has(&ah, "ah") || has(&ah, "ha"), "AH: {ah:?}");
        let gbc = d.highlight_angle("GBC");
        assert!(
            has(&gbc, "gb") && (has(&gbc, "bc") || has(&gbc, "bh")),
            "∠GBC: {gbc:?}"
        );
        let bha = d.highlight_angle("BHA");
        assert!(
            has(&bha, "bh") && (has(&bha, "ah") || has(&bha, "ha")),
            "∠BHA: {bha:?}"
        );
        assert!(d.at("G").on_seg(d.at("A"), d.at("B"), 1e-3), "G on AB");
        assert!(d.at("H").on_seg(d.at("B"), d.at("C"), 1e-3), "H on BC");
    }

    #[test]
    fn prop27_paths() {
        let d = book1_prop27();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ae") && has(&ab, "eb"), "AB via E: {ab:?}");
        let cd = d.highlight("CD");
        assert!(has(&cd, "cf") && has(&cd, "fd"), "CD via F: {cd:?}");
        let ef = d.highlight("EF");
        assert!(has(&ef, "ef") || has(&ef, "fe"), "EF: {ef:?}");
        let bg = d.highlight("BG");
        assert!(has(&bg, "bg") || has(&bg, "gb"), "AB produced to G: {bg:?}");
        let dg = d.highlight("DG");
        assert!(has(&dg, "dg") || has(&dg, "gd"), "CD produced to G: {dg:?}");
        let efg = d.highlight_angle("EFG");
        assert!(
            (has(&efg, "ef") || has(&efg, "fe"))
                && (has(&efg, "fd") || has(&efg, "df"))
                && (has(&efg, "dg") || has(&efg, "gd")),
            "∠EFG is the transversal and CD produced, not a new FG: {efg:?}"
        );
        assert!(
            !efg.iter().any(|id| id == "fg" || id == "gf"),
            "plate has no FG stroke: {efg:?}"
        );
        let aef = d.highlight_angle("AEF");
        assert!(
            (has(&aef, "ae") || has(&aef, "ea")) && (has(&aef, "ef") || has(&aef, "fe")),
            "∠AEF is AE and EF, not a detour: {aef:?}"
        );
        let efd = d.highlight_angle("EFD");
        assert!(
            (has(&efd, "ef") || has(&efd, "fe")) && (has(&efd, "fd") || has(&efd, "df")),
            "∠EFD is EF and FD: {efd:?}"
        );
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9, "B level with A");
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9, "D level with C");
        assert!((d.at("A").x - d.at("C").x).abs() < 1e-9, "A plumb with C");
        assert!(d.at("A").y > d.at("C").y, "AB above CD");
        // G is the meeting of AB and CD produced, on the B/D side.
        assert!(d.at("G").x > d.at("B").x && d.at("G").x > d.at("D").x, "G beyond B and D");
        let ab_len = d.at("A").dist(d.at("B"));
        assert!((d.at("C").dist(d.at("D")) / ab_len - 1.0875).abs() < 0.01, "CD/AB");
        assert!(d.at("E").on_seg(d.at("A"), d.at("B"), 1e-3), "E on AB");
        assert!(d.at("F").on_seg(d.at("C"), d.at("D"), 1e-3), "F on CD");
        assert!(d.at("E").on_seg(d.at("Etip"), d.at("Ftip"), 1e-3), "E on the transversal");
        assert!(d.at("F").on_seg(d.at("Etip"), d.at("Ftip"), 1e-3), "F on the transversal");
        // G sits off the wedge, on the side away from EF.
        let html = d.svg(&[] as &[String]);
        let (gdx, gdy) = letter_offset(&html, "G");
        assert!(gdx > 0.3, "G right of the meeting, --dx={gdx} --dy={gdy}");
        // English plate: E glyph on AB, above the line; F glyph on CD, below.
        // CSS y is down. E at 90° points up the page (negative dy); F at -90°
        // points down (positive dy). The marks themselves are the crossings.
        let (edx, edy) = letter_offset(&html, "E");
        assert!(edy < -0.5 && edx.abs() < 0.2, "E above AB, --dx={edx} --dy={edy}");
        let (fdx, fdy) = letter_offset(&html, "F");
        assert!(fdy > 0.5 && fdx.abs() < 0.2, "F below CD, --dx={fdx} --dy={fdy}");
        for name in ["AEF", "EFD", "EFG"] {
            let ids = d.highlight_angle(name);
            std::fs::write(format!("/tmp/p27/{name}.svg"), d.svg(&ids)).ok();
            eprintln!("{name}: {ids:?}");
        }
    }

    #[test]
    fn prop28_paths() {
        let d = book1_prop28();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ag") && has(&ab, "gb"), "AB via G: {ab:?}");
        let cd = d.highlight("CD");
        assert!(has(&cd, "ch") && has(&cd, "hd"), "CD via H: {cd:?}");
        let ef = d.highlight("EF");
        assert!(
            has(&ef, "eg") && has(&ef, "gh") && has(&ef, "hf"),
            "EF via G, H: {ef:?}"
        );
        let egb = d.highlight_angle("EGB");
        assert!(
            has(&egb, "eg") && has(&egb, "gb"),
            "∠EGB: {egb:?}"
        );
        let ghd = d.highlight_angle("GHD");
        assert!(
            has(&ghd, "gh") && has(&ghd, "hd"),
            "∠GHD: {ghd:?}"
        );
        let agh = d.highlight_angle("AGH");
        assert!(
            has(&agh, "ag") && has(&agh, "gh"),
            "∠AGH: {agh:?}"
        );
        let bgh = d.highlight_angle("BGH");
        assert!(
            has(&bgh, "gb") && has(&bgh, "gh"),
            "∠BGH: {bgh:?}"
        );
        assert!(d.at("G").on_seg(d.at("A"), d.at("B"), 1e-3), "G on AB");
        assert!(d.at("H").on_seg(d.at("C"), d.at("D"), 1e-3), "H on CD");
        assert!(d.at("G").on_seg(d.at("E"), d.at("F"), 1e-3), "G on EF");
        assert!(d.at("H").on_seg(d.at("E"), d.at("F"), 1e-3), "H on EF");
        assert!((d.at("A").x - d.at("C").x).abs() < 1e-9, "A plumb with C");
        assert!((d.at("B").x - d.at("D").x).abs() < 1e-9, "B plumb with D");
        let cd_len = d.at("C").dist(d.at("D"));
        let html = d.svg(&[] as &[String]);
        let (gdx, gdy) = letter_offset(&html, "G");
        let (edx, edy) = letter_offset(&html, "E");
        assert!(gdx > 0.15 && gdy < -0.45, "G above-right of the crossing, --dx={gdx} --dy={gdy}");
        // The stroke leaves E at about −59°. A letter on that heading is cut by
        // the line. The plate capital sits just right of the tip, a little below
        // it (CSS y down, so a small positive dy).
        assert!(
            edx > 0.4 && edy > 0.0 && edy < 0.5,
            "E just right of the tip, a little below, --dx={edx} --dy={edy}"
        );
        assert!((d.at("A").dist(d.at("B")) / cd_len - 1.0).abs() < 0.01, "AB/CD");
        let ef_len = d.at("E").dist(d.at("F")) / cd_len;
        assert!((ef_len - 1.005).abs() < 0.02, "EF/CD = {ef_len}");
        let ac = (d.at("A").y - d.at("C").y).abs() / cd_len;
        assert!((ac - 0.382).abs() < 0.01, "AC/CD = {ac}");
        let gh = d.at("G").dist(d.at("H")) / cd_len;
        assert!((gh - 0.447).abs() < 0.02, "GH/CD = {gh}");
    }

    #[test]
    fn prop29_paths() {
        let d = book1_prop29();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ag") && has(&ab, "gb"), "AB via G: {ab:?}");
        let cd = d.highlight("CD");
        assert!(has(&cd, "ch") && has(&cd, "hd"), "CD via H: {cd:?}");
        let ef = d.highlight("EF");
        assert!(has(&ef, "eg") && has(&ef, "gh") && has(&ef, "hf"), "EF: {ef:?}");
        assert!(d.at("G").on_seg(d.at("A"), d.at("B"), 1e-3));
        assert!(d.at("H").on_seg(d.at("C"), d.at("D"), 1e-3));
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9, "AB level");
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9, "CD level");
    }

    #[test]
    fn prop30_paths() {
        let d = book1_prop30();
        let gk = d.highlight("GK");
        assert!(has(&gk, "gh") && has(&gk, "hk"), "GK via H: {gk:?}");
        assert!(d.at("G").on_seg(d.at("A"), d.at("B"), 1e-3), "G on AB");
        assert!(d.at("H").on_seg(d.at("E"), d.at("F"), 1e-3), "H on EF");
        assert!(d.at("K").on_seg(d.at("C"), d.at("D"), 1e-3), "K on CD");
        assert!(d.at("G").on_seg(d.at("G0"), d.at("K0"), 1e-3), "G on GK");
        assert!(d.at("K").on_seg(d.at("G0"), d.at("K0"), 1e-3), "K on GK");
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9);
        assert!((d.at("E").y - d.at("F").y).abs() < 1e-9);
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9);
        assert!((d.at("A").x - d.at("E").x).abs() < 1e-9, "A plumb with E");
        assert!((d.at("E").x - d.at("C").x).abs() < 1e-9, "E plumb with C");
        let ab = d.at("A").dist(d.at("B"));
        assert!((d.at("A").dist(d.at("E")) / ab - 40.0 / 170.0).abs() < 0.001, "AE/AB");
        assert!((d.at("A").dist(d.at("C")) / ab - 80.0 / 170.0).abs() < 0.001, "AC/AB");
        // Plate transversal falls down-left: G is right of K.
        assert!(d.at("G").x > d.at("K").x, "GK leans down-left, as on the plate");
        let gh = d.at("G").dist(d.at("H"));
        let hk = d.at("H").dist(d.at("K"));
        assert!((gh - hk).abs() < 1e-6, "equal gaps GH and HK");
    }

    #[test]
    fn prop31_paths() {
        let d = book1_prop31();
        let eaf = d.highlight("EAF");
        assert!(has(&eaf, "ea") && has(&eaf, "af"), "EAF via A: {eaf:?}");
        let bc = d.highlight("BC");
        assert!(has(&bc, "bd") && has(&bc, "dc"), "BC via D: {bc:?}");
        let ad = d.highlight("AD");
        assert!(has(&ad, "ad"), "AD: {ad:?}");
        let ead = d.highlight_angle("EAD");
        assert!(has(&ead, "ea") && has(&ead, "ad"), "∠EAD: {ead:?}");
        let adc = d.highlight_angle("ADC");
        assert!(has(&adc, "ad") && has(&adc, "dc"), "∠ADC: {adc:?}");
        assert!((d.at("E").y - d.at("F").y).abs() < 1e-9, "EF level");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-9, "BC level");
        assert!((d.at("B").dist(d.at("C")) / d.at("E").dist(d.at("F")) - 1.0).abs() < 0.01);
        assert!((d.at("E").dist(d.at("B")) / d.at("E").dist(d.at("F")) - 1.0 / 3.0).abs() < 0.01);
        assert!((d.at("E").dist(d.at("A")) / d.at("E").dist(d.at("F")) - 4.0 / 7.0).abs() < 0.01);
        assert!((d.at("B").dist(d.at("D")) / d.at("B").dist(d.at("C")) - 1.0 / 3.0).abs() < 0.01);
        assert!((d.at("A").dist(d.at("D")) / d.at("E").dist(d.at("F")) - 0.410).abs() < 0.002);
        assert!(d.at("A").on_seg(d.at("E"), d.at("F"), 1e-6), "A on EF");
        assert!(d.at("D").on_seg(d.at("B"), d.at("C"), 1e-6), "D on BC");
        assert!(d.at("D").x < d.at("A").x, "AD leans left, as on the plate");
    }

    #[test]
    fn prop32_paths() {
        let d = book1_prop32();
        let bc = d.highlight("BC");
        assert!(has(&bc, "bc"), "BC: {bc:?}");
        let bd = d.highlight("BD");
        assert!(has(&bd, "bc") && has(&bd, "cd"), "BD via C: {bd:?}");
        let ce = d.highlight("CE");
        assert!(has(&ce, "ec"), "CE: {ce:?}");
        let acd = d.highlight_angle("ACD");
        assert!(has(&acd, "ac") && has(&acd, "cd"), "∠ACD: {acd:?}");
        let bac = d.highlight_angle("BAC");
        assert!(has(&bac, "ab") && has(&bac, "ac"), "∠BAC: {bac:?}");
        let ace = d.highlight_angle("ACE");
        assert!(has(&ace, "ac") && has(&ace, "ec"), "∠ACE: {ace:?}");
        let ecd = d.highlight_angle("ECD");
        assert!(has(&ecd, "ec") && has(&ecd, "cd"), "∠ECD: {ecd:?}");
        assert!(d.at("C").on_seg(d.at("B"), d.at("D"), 1e-3), "C on BD");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-9, "BC level");
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9, "CD level");
        let bc_len = d.at("B").dist(d.at("C"));
        assert!((d.at("A").dist(d.at("B")) / bc_len - 1.394).abs() < 0.01, "AB/BC");
        assert!((d.at("A").dist(d.at("C")) / bc_len - 1.333).abs() < 0.01, "AC/BC");
        assert!((d.at("C").dist(d.at("D")) / bc_len - 1.145).abs() < 0.01, "CD/BC");
    }

    #[test]
    fn prop33_paths() {
        let d = book1_prop33();
        let ab = d.highlight("AB");
        assert!(has(&ab, "ba"), "AB: {ab:?}");
        let cd = d.highlight("CD");
        assert!(has(&cd, "dc"), "CD: {cd:?}");
        let ac = d.highlight("AC");
        assert!(has(&ac, "ac"), "AC: {ac:?}");
        let bd = d.highlight("BD");
        assert!(has(&bd, "bd"), "BD: {bd:?}");
        let bc = d.highlight("BC");
        assert!(has(&bc, "bc"), "BC: {bc:?}");
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ba") && has(&abc, "bc") && has(&abc, "ac"), "△ABC: {abc:?}");
        let dcb = d.highlight("DCB");
        assert!(has(&dcb, "dc") && has(&dcb, "bc") && has(&dcb, "bd"), "△DCB: {dcb:?}");
        let ang_abc = d.highlight_angle("ABC");
        assert!(has(&ang_abc, "ba") && has(&ang_abc, "bc"), "∠ABC: {ang_abc:?}");
        let ang_bcd = d.highlight_angle("BCD");
        assert!(has(&ang_bcd, "bc") && has(&ang_bcd, "dc"), "∠BCD: {ang_bcd:?}");
        let ang_acb = d.highlight_angle("ACB");
        assert!(has(&ang_acb, "ac") && has(&ang_acb, "bc"), "∠ACB: {ang_acb:?}");
        let ang_cbd = d.highlight_angle("CBD");
        assert!(has(&ang_cbd, "bc") && has(&ang_cbd, "bd"), "∠CBD: {ang_cbd:?}");
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9, "AB level");
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9, "CD level");
        let ab_len = d.at("A").dist(d.at("B"));
        assert!((d.at("C").dist(d.at("D")) / ab_len - 1.0).abs() < 0.01, "CD/AB");
        assert!((d.at("A").dist(d.at("C")) / ab_len - 0.610).abs() < 0.01, "AC/AB");
        assert!((d.at("B").dist(d.at("D")) / ab_len - 0.610).abs() < 0.01, "BD/AB");
    }

    #[test]
    fn prop34_paths() {
        let d = book1_prop34();
        let acdb = d.highlight("ACDB");
        assert!(
            has(&acdb, "ac") && has(&acdb, "cd") && has(&acdb, "bd") && has(&acdb, "ab"),
            "ACDB: {acdb:?}"
        );
        let bc = d.highlight("BC");
        assert!(has(&bc, "bc"), "BC: {bc:?}");
        let abd = d.highlight_angle("ABD");
        assert!(has(&abd, "ab") && has(&abd, "bd"), "∠ABD: {abd:?}");
        let acd = d.highlight_angle("ACD");
        assert!(has(&acd, "ac") && has(&acd, "cd"), "∠ACD: {acd:?}");
        let bac = d.highlight_angle("BAC");
        assert!(has(&bac, "ab") && has(&bac, "ac"), "∠BAC: {bac:?}");
        let cdb = d.highlight_angle("CDB");
        assert!(has(&cdb, "cd") && has(&cdb, "bd"), "∠CDB: {cdb:?}");
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9, "AB level");
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9, "CD level");
        let ab_len = d.at("A").dist(d.at("B"));
        assert!((d.at("C").dist(d.at("D")) / ab_len - 1.0).abs() < 0.01, "CD/AB");
        assert!((d.at("A").dist(d.at("C")) / ab_len - 0.594).abs() < 0.01, "AC/AB");
        assert!((d.at("B").dist(d.at("D")) / ab_len - 0.594).abs() < 0.01, "BD/AB");
    }

    #[test]
    fn prop35_paths() {
        let d = book1_prop35();
        let abcd = d.highlight("ABCD");
        assert!(
            has(&abcd, "ab")
                && has(&abcd, "ad")
                && (has(&abcd, "dg") || has(&abcd, "gd"))
                && (has(&abcd, "gc") || has(&abcd, "cg"))
                && has(&abcd, "bc"),
            "ABCD: {abcd:?}"
        );
        let ebcf = d.highlight("EBCF");
        assert!(
            (has(&ebcf, "ge") || has(&ebcf, "eg"))
                && (has(&ebcf, "gb") || has(&ebcf, "bg"))
                && has(&ebcf, "bc")
                && has(&ebcf, "fc")
                && has(&ebcf, "ef"),
            "EBCF: {ebcf:?}"
        );
        let af = d.highlight("AF");
        assert!(has(&af, "ad") && has(&af, "de") && has(&af, "ef"), "AF: {af:?}");
        let ae = d.highlight("AE");
        assert!(has(&ae, "ad") && has(&ae, "de"), "AE: {ae:?}");
        let df = d.highlight("DF");
        assert!(has(&df, "de") && has(&df, "ef"), "DF: {df:?}");
        let eb = d.highlight("EB");
        assert!((has(&eb, "ge") || has(&eb, "eg")) && (has(&eb, "gb") || has(&eb, "bg")), "EB: {eb:?}");
        let fc = d.highlight("FC");
        assert!(has(&fc, "fc"), "FC: {fc:?}");
        let eab = d.highlight("EAB");
        assert!(
            has(&eab, "ab")
                && (has(&eab, "ge") || has(&eab, "eg"))
                && (has(&eab, "gb") || has(&eab, "bg")),
            "△EAB: {eab:?}"
        );
        let dfc = d.highlight("DFC");
        assert!(
            has(&dfc, "fc")
                && (has(&dfc, "dg") || has(&dfc, "gd"))
                && (has(&dfc, "gc") || has(&dfc, "cg")),
            "△DFC: {dfc:?}"
        );
        let gb = d.highlight("GB");
        assert!(has(&gb, "gb") || has(&gb, "bg"), "GB: {gb:?}");
        let gc = d.highlight("GC");
        assert!(has(&gc, "gc") || has(&gc, "cg"), "GC: {gc:?}");
        let gbc = d.highlight("GBC");
        assert!(
            (has(&gbc, "gb") || has(&gbc, "bg"))
                && (has(&gbc, "gc") || has(&gbc, "cg"))
                && has(&gbc, "bc"),
            "△GBC: {gbc:?}"
        );
        // △DGE is taken away from both. It is the two cut pieces meeting at G,
        // not the whole of EB and DC, and not the trapezium underneath.
        let dge = d.highlight("DGE");
        assert!(
            (has(&dge, "dg") || has(&dge, "gd"))
                && (has(&dge, "ge") || has(&dge, "eg"))
                && has(&dge, "de"),
            "△DGE: {dge:?}"
        );
        assert!(
            !has(&dge, "gc") && !has(&dge, "cg") && !has(&dge, "gb") && !has(&dge, "bg"),
            "DGE stops at G: {dge:?}"
        );
        // What remains turns at G: AB–BG–GD–DA, and EG–GC–CF–FE.
        let abgd = d.highlight("ABGD");
        assert!(
            has(&abgd, "ab")
                && (has(&abgd, "gb") || has(&abgd, "bg"))
                && (has(&abgd, "dg") || has(&abgd, "gd"))
                && has(&abgd, "ad"),
            "trapezium ABGD: {abgd:?}"
        );
        assert!(
            !has(&abgd, "de") && !has(&abgd, "ge") && !has(&abgd, "eg") && !has(&abgd, "bc"),
            "ABGD no DE/GE/BC: {abgd:?}"
        );
        let egcf = d.highlight("EGCF");
        assert!(
            (has(&egcf, "ge") || has(&egcf, "eg"))
                && (has(&egcf, "gc") || has(&egcf, "cg"))
                && has(&egcf, "fc")
                && has(&egcf, "ef"),
            "trapezium EGCF: {egcf:?}"
        );
        assert!(
            !has(&egcf, "de") && !has(&egcf, "dg") && !has(&egcf, "gd") && !has(&egcf, "bc"),
            "EGCF no DE/DG/BC: {egcf:?}"
        );
        let fdc = d.highlight_angle("FDC");
        assert!(has(&fdc, "de") && (has(&fdc, "dg") || has(&fdc, "gd")), "∠FDC: {fdc:?}");
        let eab_ang = d.highlight_angle("EAB");
        assert!(has(&eab_ang, "ad") && has(&eab_ang, "ab"), "∠EAB: {eab_ang:?}");
        assert!(d.at("G").on_seg(d.at("E"), d.at("B"), 1e-3), "G on EB");
        assert!(d.at("G").on_seg(d.at("D"), d.at("C"), 1e-3), "G on DC");
        assert!(d.at("D").on_seg(d.at("A"), d.at("F"), 1e-3), "D on AF");
        assert!(d.at("E").on_seg(d.at("A"), d.at("F"), 1e-3), "E on AF");
        assert!((d.at("A").y - d.at("F").y).abs() < 1e-9, "AF level");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-9, "BC level");
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").dist(d.at("D")) / bc - 1.0).abs() < 0.01, "AD/BC");
        assert!((d.at("E").dist(d.at("F")) / bc - 1.004).abs() < 0.01, "EF/BC");
        assert!((d.at("A").dist(d.at("B")) / bc - 1.673).abs() < 0.01, "AB/BC");
        assert!((d.at("D").dist(d.at("C")) / bc - 1.673).abs() < 0.01, "DC/BC");
    }

    #[test]
    fn prop36_paths() {
        let d = book1_prop36();
        let abcd = d.highlight("ABCD");
        assert!(
            has(&abcd, "ab") && has(&abcd, "ad") && has(&abcd, "dc") && has(&abcd, "bc"),
            "ABCD: {abcd:?}"
        );
        let efgh = d.highlight("EFGH");
        assert!(
            has(&efgh, "ef") && has(&efgh, "fg") && has(&efgh, "hg") && has(&efgh, "eh"),
            "EFGH: {efgh:?}"
        );
        let ebch = d.highlight("EBCH");
        assert!(
            has(&ebch, "be") && has(&ebch, "bc") && has(&ebch, "ch") && has(&ebch, "eh"),
            "EBCH: {ebch:?}"
        );
        let be = d.highlight("BE");
        assert!(has(&be, "be"), "BE: {be:?}");
        let ch = d.highlight("CH");
        assert!(has(&ch, "ch"), "CH: {ch:?}");
        let hc = d.highlight("HC");
        assert!(has(&hc, "ch"), "HC: {hc:?}");
        let eh = d.highlight("EH");
        assert!(has(&eh, "eh"), "EH: {eh:?}");
        let ah = d.highlight("AH");
        assert!(has(&ah, "ad") && has(&ah, "de") && has(&ah, "eh"), "AH: {ah:?}");
        let bg = d.highlight("BG");
        assert!(has(&bg, "bc") && has(&bg, "cf") && has(&bg, "fg"), "BG: {bg:?}");
        assert!(d.at("C").on_seg(d.at("B"), d.at("G"), 1e-3), "C on BG");
        assert!(d.at("F").on_seg(d.at("B"), d.at("G"), 1e-3), "F on BG");
        assert!(d.at("D").on_seg(d.at("A"), d.at("H"), 1e-3), "D on AH");
        assert!(d.at("E").on_seg(d.at("A"), d.at("H"), 1e-3), "E on AH");
        assert!((d.at("A").y - d.at("H").y).abs() < 1e-9, "AH level");
        assert!((d.at("B").y - d.at("G").y).abs() < 1e-9, "BG level");
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").dist(d.at("D")) / bc - 1.0).abs() < 0.01, "AD/BC");
        assert!((d.at("F").dist(d.at("G")) / bc - 1.163).abs() < 0.01, "FG/BC");
        assert!((d.at("E").dist(d.at("H")) / bc - 1.108).abs() < 0.01, "EH/BC");
        assert!((d.at("A").dist(d.at("B")) / bc - 2.266).abs() < 0.01, "AB/BC");
        assert!((d.at("D").dist(d.at("C")) / bc - 2.266).abs() < 0.01, "DC/BC");
    }

    #[test]
    fn prop37_matches_english_plate() {
        let d = book1_prop37();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").y - d.at("E").y).abs() < 1e-6, "EF level");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-9, "BC level");
        assert!(((d.at("A").y - d.at("B").y) / bc - 0.693).abs() < 0.01, "height");
        assert!((d.at("E").dist(d.at("A")) / bc - 0.939).abs() < 0.01, "EA");
        assert!((d.at("A").dist(d.at("D")) / bc - 0.443).abs() < 0.01, "AD");
        assert!((d.at("D").dist(d.at("F")) / bc - 0.970).abs() < 0.01, "DF");
        // Same base, same parallels: EBCA and DBCF are parallelograms.
        let eb = d.at("E").sub(d.at("B"));
        let ac = d.at("A").sub(d.at("C"));
        let s = (eb.x * ac.y - eb.y * ac.x).abs() / (eb.len() * ac.len());
        assert!(s < 1.0, "figure built");
    }

    #[test]
    fn prop38_matches_english_plate() {
        let d = book1_prop38();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").y - d.at("G").y).abs() < 1e-6, "GH level");
        assert!((d.at("B").y - d.at("F").y).abs() < 1e-9, "BF level");
        assert!(((d.at("A").y - d.at("B").y) / bc - 1.879).abs() < 0.01, "height");
        assert!((d.at("E").dist(d.at("F")) / bc - 1.247).abs() < 0.01, "EF");
        assert!((d.at("C").dist(d.at("E")) / bc - 2.758).abs() < 0.01, "CE");
        assert!((d.at("G").dist(d.at("A")) / bc - 0.984).abs() < 0.01, "GA");
        assert!((d.at("D").dist(d.at("H")) / bc - 1.209).abs() < 0.01, "DH");
        let bg = d.at("G").sub(d.at("B"));
        let ac = d.at("C").sub(d.at("A"));
        let s = (bg.x * ac.y - bg.y * ac.x).abs() / (bg.len() * ac.len());
        assert!(s < 0.02, "BG ∥ AC, sin {s}");
        let fh = d.at("H").sub(d.at("F"));
        let de = d.at("E").sub(d.at("D"));
        let s2 = (fh.x * de.y - fh.y * de.x).abs() / (fh.len() * de.len());
        assert!(s2 < 0.02, "FH ∥ DE, sin {s2}");
    }

    #[test]
    fn prop39_matches_english_plate() {
        let d = book1_prop39();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").y - d.at("D").y).abs() < 0.4, "AD level with the base");
        assert!((d.at("B").y - d.at("C").y).abs() < 0.1, "BC level");
        assert!(d.at("E").x > d.at("A").x && d.at("E").x < d.at("D").x, "E between A and D");
        assert!(d.at("E").y < d.at("A").y && d.at("E").y > d.at("B").y, "E between AD and BC");
        assert!(d.at("E").on_seg(d.at("B"), d.at("D"), 1e-3), "E on BD");
        assert!(((d.at("A").y - d.at("B").y) / bc - 0.560).abs() < 0.01, "height");
        assert!((d.at("A").dist(d.at("D")) / bc - 0.759).abs() < 0.01, "AD");
        assert!((d.at("A").dist(d.at("B")) / bc - 0.562).abs() < 0.01, "AB");
        assert!((d.at("D").dist(d.at("C")) / bc - 0.591).abs() < 0.01, "DC");
        // AE joins A to E; EC joins E to C. E is the crossing on BD.
        let ae = d.at("A").dist(d.at("E")) / bc;
        let ec = d.at("E").dist(d.at("C")) / bc;
        assert!((ae - 0.664).abs() < 0.01, "AE {ae}");
        assert!((ec - 0.569).abs() < 0.01, "EC {ec}");
        let ea = d.at("A").sub(d.at("E"));
        let ecv = d.at("C").sub(d.at("E"));
        let cos = (ea.x * ecv.x + ea.y * ecv.y) / (ea.len() * ecv.len());
        let deg = cos.acos().to_degrees();
        assert!((deg - 125.3).abs() < 1.0, "∠AEC {deg}");
        let aeh = d.highlight("AE");
        assert!(has(&aeh, "ae"), "AE joined: {aeh:?}");
        let ech = d.highlight("EC");
        assert!(has(&ech, "ec"), "EC joined: {ech:?}");
        let html = d.svg(&[] as &[String]);
        let (edx, edy) = letter_offset(&html, "E");
        // Plate: E's ink center is one letter (10 pt) under the crossing, heading −99.5°.
        // A BC-unit glyph sits too low once the type is smaller than Fitzpatrick's.
        assert!(
            (edx - (-0.173)).abs() < 0.03 && (edy - 1.036).abs() < 0.03,
            "E is 1.05em under the crossing, not on it and not a BC-unit drop: --dx={edx} --dy={edy}"
        );
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ab") && has(&abc, "ac") && has(&abc, "bc"), "△ABC: {abc:?}");
        let dbc = d.highlight("DBC");
        assert!(
            has(&dbc, "be") && has(&dbc, "ed") && has(&dbc, "dc") && has(&dbc, "bc"),
            "△DBC: {dbc:?}"
        );
        let ebc = d.highlight("EBC");
        assert!(
            has(&ebc, "be") && has(&ebc, "bc") && has(&ebc, "ec") && !has(&ebc, "ab") && !has(&ebc, "ae"),
            "△EBC is the triangle on BD, not a walk through A: {ebc:?}"
        );
        let ad = d.highlight("AD");
        assert!(has(&ad, "ad"), "AD: {ad:?}");
    }

    #[test]
    fn prop40_matches_english_plate() {
        let d = book1_prop40();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").y - d.at("D").y).abs() < 0.1, "AD level");
        assert!((d.at("B").y - d.at("E").y).abs() < 0.1, "BE level");
        assert!(d.at("C").on_seg(d.at("B"), d.at("E"), 1e-6), "C on BE");
        assert!(d.at("F").on_seg(d.at("D"), d.at("C"), 1e-3), "F on DC");
        assert!(d.at("F").y < d.at("D").y && d.at("F").y > d.at("C").y, "F between D and C");
        assert!(((d.at("A").y - d.at("B").y) / bc - 0.920).abs() < 0.01, "height");
        assert!((d.at("A").dist(d.at("D")) / bc - 1.509).abs() < 0.01, "AD");
        assert!((d.at("C").dist(d.at("E")) / bc - 0.945).abs() < 0.01, "CE");
        assert!((d.at("B").dist(d.at("E")) / bc - 1.945).abs() < 0.01, "BE");
        assert!((d.at("A").dist(d.at("F")) / bc - 1.375).abs() < 0.01, "AF");
        assert!((d.at("F").dist(d.at("E")) / bc - 0.833).abs() < 0.01, "FE");
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ab") && has(&abc, "ac") && has(&abc, "bc"), "△ABC: {abc:?}");
        let cde = d.highlight("CDE");
        assert!(
            has(&cde, "df") && has(&cde, "fc") && has(&cde, "de") && has(&cde, "ce"),
            "△CDE turns at F: {cde:?}"
        );
        let af = d.highlight("AF");
        assert!(has(&af, "af"), "AF joined: {af:?}");
        let fe = d.highlight("FE");
        assert!(has(&fe, "fe"), "FE joined: {fe:?}");
        let ad = d.highlight("AD");
        assert!(has(&ad, "ad"), "AD: {ad:?}");
        let be = d.highlight("BE");
        assert!(has(&be, "bc") && has(&be, "ce"), "BE: {be:?}");
        let fce = d.highlight("FCE");
        assert!(
            has(&fce, "fc") && has(&fce, "ce") && has(&fce, "fe") && !has(&fce, "ab") && !has(&fce, "af"),
            "△FCE is the triangle on DC, not a walk through A: {fce:?}"
        );
        let dce = d.highlight("DCE");
        assert!(
            has(&dce, "df") && has(&dce, "fc") && has(&dce, "de") && has(&dce, "ce"),
            "△DCE turns at F: {dce:?}"
        );
        let html = d.svg(&[] as &[String]);
        let (fdx, fdy) = letter_offset(&html, "F");
        // Plate: F's ink center is one letter off the crossing, heading −151.5°.
        assert!(
            (fdx - (-0.923)).abs() < 0.03 && (fdy - 0.502).abs() < 0.03,
            "F is 1.05em off the crossing: --dx={fdx} --dy={fdy}"
        );
    }

    #[test]
    fn prop41_matches_english_plate() {
        let d = book1_prop41();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("A").y - d.at("E").y).abs() < 1e-6, "AE level");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-6, "BC level");
        assert!(d.at("D").on_seg(d.at("A"), d.at("E"), 1e-6), "D on AE");
        assert!(d.at("D").x > d.at("C").x, "right side leans out, as on the English plate");
        assert!(((d.at("A").y - d.at("B").y) / bc - 1.091).abs() < 0.01, "height");
        assert!((d.at("A").dist(d.at("E")) / bc - 1.909).abs() < 0.02, "AE");
        assert!((d.at("A").dist(d.at("D")) / bc - 1.0).abs() < 0.01, "AD = BC");
        assert!((d.at("A").dist(d.at("B")) / bc - 1.095).abs() < 0.02, "AB");
        assert!((d.at("C").dist(d.at("D")) / bc - 1.095).abs() < 0.02, "CD");
        assert!((d.at("B").dist(d.at("E")) / bc - 2.278).abs() < 0.02, "BE");
        let abcd = d.highlight("ABCD");
        assert!(
            has(&abcd, "ab") && has(&abcd, "dc") && has(&abcd, "ad") && has(&abcd, "bc"),
            "parallelogram ABCD: {abcd:?}"
        );
        let ebc = d.highlight("EBC");
        assert!(has(&ebc, "eb") && has(&ebc, "ec") && has(&ebc, "bc"), "triangle EBC: {ebc:?}");
        let ac = d.highlight("AC");
        assert!(has(&ac, "ac"), "diagonal AC: {ac:?}");
    }

    #[test]
    fn prop42_matches_english_plate() {
        let d = book1_prop42();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-6, "BC level");
        assert!((d.at("A").y - d.at("G").y).abs() < 1e-6, "AG level");
        assert!(d.at("E").on_seg(d.at("B"), d.at("C"), 1e-6), "E on BC");
        assert!((d.at("B").dist(d.at("E")) - d.at("E").dist(d.at("C"))).abs() < 1e-6);
        assert!(((d.at("A").y - d.at("B").y) / bc - 0.750).abs() < 0.01, "height");
        assert!((d.at("A").dist(d.at("G")) / bc - 1.242).abs() < 0.02, "AG");
        assert!((d.at("E").dist(d.at("F")) / bc - 0.804).abs() < 0.02, "EF");
        assert!((d.at("C").dist(d.at("G")) / bc - 0.804).abs() < 0.02, "CG");
        let cg = d.at("G").sub(d.at("C"));
        let ef = d.at("F").sub(d.at("E"));
        let cross = cg.x * ef.y - cg.y * ef.x;
        let sin = cross.abs() / (cg.len() * ef.len());
        assert!(sin < 0.003, "CG parallel to EF, sin {sin}");
        let a = d.at("A");
        let dv = d.at("D");
        let from_a = dv.sub(a);
        assert!((from_a.len() / bc - 0.371).abs() < 0.01, "D from A");
        assert!((from_a.y.atan2(from_a.x).to_degrees() + 165.4).abs() < 0.5, "D heading");
        let left = d.at("Dleft").dist(dv) / bc;
        let right = d.at("Dright").dist(dv) / bc;
        assert!((left - 0.346).abs() < 0.01, "left arm {left}");
        assert!((right - 0.363).abs() < 0.01, "right arm {right}");
        let u = d.at("Dleft").sub(dv);
        let v = d.at("Dright").sub(dv);
        let open = (u.x * v.x + u.y * v.y) / (u.len() * v.len());
        let deg = open.clamp(-1.0, 1.0).acos().to_degrees();
        assert!((deg - 43.9).abs() < 0.4, "angle at D {deg}");
        let abc = d.highlight("ABC");
        assert!(has(&abc, "ab") && has(&abc, "ac") && has(&abc, "be") && has(&abc, "ec"), "triangle ABC: {abc:?}");
        let fecg = d.highlight("FECG");
        assert!(has(&fecg, "ef") && has(&fecg, "cg") && has(&fecg, "fg") && has(&fecg, "ec"),
            "parallelogram FECG: {fecg:?}"
        );
        let ang = d.highlight("D");
        assert!(has(&ang, "ang-d"), "given angle D: {ang:?}");
    }

    #[test]
    fn prop43_matches_english_plate() {
        let d = book1_prop43();
        let bc = d.at("B").dist(d.at("C"));
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-6, "BC level");
        assert!((d.at("A").y - d.at("D").y).abs() < 1e-6, "AD level");
        assert!((d.at("E").y - d.at("F").y).abs() < 1e-6, "EF level");
        assert!((d.at("K").y - d.at("E").y).abs() < 1e-6, "K on the mid row");
        assert!(d.at("E").on_seg(d.at("A"), d.at("B"), 1e-6), "E on AB");
        assert!(d.at("F").on_seg(d.at("D"), d.at("C"), 1e-6), "F on DC");
        assert!(d.at("H").on_seg(d.at("A"), d.at("D"), 1e-6), "H on AD");
        assert!(d.at("G").on_seg(d.at("B"), d.at("C"), 1e-6), "G on BC");
        assert!(d.at("K").on_seg(d.at("H"), d.at("G"), 0.5), "K on HG");
        assert!(d.at("K").on_seg(d.at("A"), d.at("C"), 1e-4), "K on AC");
        assert!(d.at("K").on_seg(d.at("E"), d.at("F"), 1e-4), "K on the mid row, so AK produced is AF");
        assert!(((d.at("A").y - d.at("B").y) / bc - 0.588).abs() < 0.01, "height");
        assert!((d.at("A").dist(d.at("B")) / bc - 0.614).abs() < 0.01, "AB");
        assert!((d.at("D").dist(d.at("C")) / bc - 0.614).abs() < 0.01, "DC");
        assert!((d.at("A").dist(d.at("D")) / bc - 1.0).abs() < 0.01, "AD");
        assert!((d.at("B").dist(d.at("G")) / bc - 0.295).abs() < 0.01, "BG");
        assert!((d.at("A").dist(d.at("H")) / d.at("A").dist(d.at("D")) - 0.298).abs() < 0.01, "AH");
        let abcd = d.highlight("ABCD");
        assert!(has(&abcd, "ah") && has(&abcd, "hd"), "AD: {abcd:?}");
        assert!(has(&abcd, "ae") && has(&abcd, "eb"), "AB: {abcd:?}");
        assert!(has(&abcd, "df") && has(&abcd, "fc"), "DC: {abcd:?}");
        assert!(has(&abcd, "bg") && has(&abcd, "gc"), "BC: {abcd:?}");
        assert!(!has(&abcd, "ac") && !has(&abcd, "ak") && !has(&abcd, "kc"), "ABCD is not the diagonal: {abcd:?}");
        let segs = d.stroke_ids();
        assert!(segs.iter().any(|id| id == "ek"), "EK stroke: {segs:?}");
        assert!(segs.iter().any(|id| id == "kf"), "KF, the rest of the page's EF: {segs:?}");
        assert!(!segs.iter().any(|id| id == "eh"), "plate has no EH stroke: {segs:?}");
        assert!(!segs.iter().any(|id| id == "fg"), "plate has no FG stroke: {segs:?}");
        let ac = d.highlight("AC");
        assert!(has(&ac, "ak") && has(&ac, "kc"), "diagonal AC: {ac:?}");
        // EH and FG are the parallelograms about the diagonal, named by
        // opposite corners. BK and KD are the complements, the same way.
        // None of those names is the straight-line between the letters.
        let eh = d.highlight("EH");
        assert!(has(&eh, "ae") && has(&eh, "ek") && has(&eh, "hk") && has(&eh, "ah"), "parallelogram EH: {eh:?}");
        assert!(!has(&eh, "ak"), "EH is not its diagonal AK: {eh:?}");
        let fg = d.highlight("FG");
        assert!(has(&fg, "kf") && has(&fg, "fc") && has(&fg, "gc") && has(&fg, "kg"), "parallelogram FG: {fg:?}");
        assert!(!has(&fg, "kc"), "FG is not its diagonal KC: {fg:?}");
        let bk = d.highlight("BK");
        assert!(has(&bk, "eb") && has(&bk, "ek") && has(&bk, "kg") && has(&bk, "bg"), "complement BK: {bk:?}");
        assert!(!has(&bk, "ae") && !has(&bk, "gc"), "BK is the complement, not the outer half: {bk:?}");
        let kd = d.highlight("KD");
        assert!(has(&kd, "kf") && has(&kd, "df") && has(&kd, "hd") && has(&kd, "hk"), "complement KD: {kd:?}");
        assert!(!has(&kd, "ah") && !has(&kd, "fc"), "KD is the complement, not the outer half: {kd:?}");
        let ak = d.highlight("AK");
        assert!(has(&ak, "ak") && !has(&ak, "kc") && !has(&ak, "kf"), "AK is the diagonal of EH: {ak:?}");
        let kfc = d.highlight("KFC");
        assert!(has(&kfc, "kf") && has(&kfc, "fc") && has(&kfc, "kc"), "triangle KFC: {kfc:?}");
        assert!(!has(&kfc, "kg"), "KFC does not walk around through G: {kfc:?}");
        let kgc = d.highlight("KGC");
        assert!(has(&kgc, "kg") && has(&kgc, "gc") && has(&kgc, "kc"), "triangle KGC: {kgc:?}");
        let aek = d.highlight("AEK");
        assert!(has(&aek, "ae") && has(&aek, "ek") && has(&aek, "ak"), "triangle AEK: {aek:?}");
        let ahk = d.highlight("AHK");
        assert!(has(&ahk, "ah") && has(&ahk, "hk") && has(&ahk, "ak"), "triangle AHK: {ahk:?}");
    }

    #[test]
    fn prop44_matches_english_plate() {
        let d = book1_prop44();
        let ab = d.at("A").dist(d.at("B"));
        assert!((ab - 100.0).abs() < 0.05, "AB scale {ab}");
        // Three level rows, three uprights leaning the same way.
        assert!((d.at("H").y - d.at("L").y).abs() < 1e-6, "HL level");
        assert!((d.at("G").y - d.at("M").y).abs() < 1e-6, "GM level");
        assert!((d.at("F").y - d.at("K").y).abs() < 1e-6, "FK level");
        assert!((d.at("B").y - d.at("G").y).abs() < 0.02, "B on the GM row");
        assert!(d.at("G").on_seg(d.at("F"), d.at("H"), 1e-3), "G on FH");
        assert!(d.at("B").on_seg(d.at("E"), d.at("A"), 1e-3), "B on EA");
        assert!(d.at("M").on_seg(d.at("K"), d.at("L"), 1e-3), "M on KL");
        // KH is the diagonal of HLKF. The plate draws it from K to H, and B,
        // the proof's meeting of HB with the mid row, lies on it.
        assert!(d.at("B").on_seg(d.at("K"), d.at("H"), 1e-3), "B on KH");
        assert!(d.edge_id("K", "B").is_some() && d.edge_id("B", "H").is_some(), "HB joined, produced to K");
        // The middle upright is EA, AB straight-on to BE, from the top row to HL.
        assert!(d.edge_id("E", "B").is_some() && d.edge_id("B", "A").is_some());
        assert!(d.edge_id("H", "A").is_some() && d.edge_id("A", "L").is_some(), "HL split at A");
        let hl = d.at("H").dist(d.at("L")) / ab;
        assert!((hl - 2.959).abs() < 0.01, "HL {hl}");
        assert!(((d.at("F").y - d.at("A").y) / ab - 2.959).abs() < 0.01, "height");
        let fh = d.at("F").sub(d.at("H"));
        let ea = d.at("E").sub(d.at("A"));
        let kl = d.at("K").sub(d.at("L"));
        let sin = |u: crate::figure::geom::V2, v: crate::figure::geom::V2| {
            (u.x * v.y - u.y * v.x).abs() / (u.len() * v.len())
        };
        assert!(sin(fh, ea) < 0.004, "FH parallel to EA");
        assert!(sin(fh, kl) < 0.004, "FH parallel to KL");
        let hlkf = d.highlight("HLKF");
        assert!(has(&hlkf, "hl") || (has(&hlkf, "ha") && has(&hlkf, "al")), "bottom row: {hlkf:?}");
        assert!(has(&hlkf, "fk") || (has(&hlkf, "fe") && has(&hlkf, "ek")), "top row: {hlkf:?}");
        assert!(has(&hlkf, "fg") && has(&hlkf, "gh"), "FH: {hlkf:?}");
        assert!(has(&hlkf, "km") && has(&hlkf, "ml"), "KL: {hlkf:?}");
        assert!(!has(&hlkf, "bh") && !has(&hlkf, "kb"), "HLKF is not its diagonal: {hlkf:?}");
        let befg = d.highlight("BEFG");
        assert!(
            has(&befg, "eb") && has(&befg, "fe") && has(&befg, "fg") && has(&befg, "gb"),
            "BEFG: {befg:?}"
        );
        assert!(!has(&befg, "ba") && !has(&befg, "ek"), "BEFG is the inner parallelogram: {befg:?}");
        let hk = d.highlight("HK");
        assert!(has(&hk, "kb") && has(&hk, "bh"), "diagonal HK: {hk:?}");
        let hb = d.highlight("HB");
        assert!(has(&hb, "bh") && !has(&hb, "kb"), "HB is the joined piece, not the production: {hb:?}");
        let ag = d.highlight("AG");
        assert!(
            has(&ag, "ba") && has(&ag, "gb") && has(&ag, "gh") && has(&ag, "ha"),
            "parallelogram AG: {ag:?}"
        );
        assert!(!has(&ag, "bh"), "AG is not a walk along the diagonal: {ag:?}");
        let me = d.highlight("ME");
        assert!(
            has(&me, "eb") && has(&me, "ek") && has(&me, "km") && has(&me, "bm"),
            "parallelogram ME: {me:?}"
        );
        let lb = d.highlight("LB");
        assert!(
            has(&lb, "al") && has(&lb, "ba") && has(&lb, "bm") && has(&lb, "ml"),
            "complement LB: {lb:?}"
        );
        assert!(!has(&lb, "ha") && !has(&lb, "eb"), "LB is the complement, not the outer half: {lb:?}");
        let bf = d.highlight("BF");
        assert!(
            has(&bf, "eb") && has(&bf, "fe") && has(&bf, "fg") && has(&bf, "gb"),
            "complement BF: {bf:?}"
        );
        assert!(!has(&bf, "ek") && !has(&bf, "gh"), "BF is the complement, not the outer half: {bf:?}");
        let tri = d.highlight("C");
        assert!(has(&tri, "seg-c"), "triangle C: {tri:?}");
        // Base parallel to FE, apex above it.
        assert!((d.at("P").y - d.at("Q").y).abs() < 1e-6, "base level");
        assert!(d.at("R").y > d.at("P").y, "apex above the base");
        // D is the bent mark: vertex, upper arm, lower arm. Not a downward V.
        let v = d.at("D");
        let upper = d.at("Dleft");
        let lower = d.at("Dright");
        assert!(upper.y > v.y && lower.y < v.y, "D bends down from the upper arm");
        let ang = d.highlight("D");
        assert!(has(&ang, "ang-d"), "given angle D: {ang:?}");
        let bhg = d.highlight_angle("BHG");
        assert!(has(&bhg, "bh") && has(&bhg, "gh") && !has(&bhg, "gb"), "∠BHG is the two rays: {bhg:?}");
        let gfe = d.highlight_angle("GFE");
        assert!(has(&gfe, "fg") && has(&gfe, "fe") && !has(&gfe, "eg"), "∠GFE is the two rays: {gfe:?}");
        let abm = d.highlight_angle("ABM");
        assert!(has(&abm, "ba") && has(&abm, "bm") && !has(&abm, "ml"), "∠ABM is the two rays: {abm:?}");
        let html = d.svg(&[] as &[String]);
        let (fdx, fdy) = letter_offset(&html, "F");
        let (edx, edy) = letter_offset(&html, "E");
        // Above the FK line: CSS y is down, so a negative --dy is above the mark.
        // Shorter than the shared 0.82 em radius.
        assert!(fdy < -0.15, "F above FK, --dy={fdy}");
        assert!(edy < -0.15, "E above FK, --dy={edy}");
        assert!((fdx * fdx + fdy * fdy).sqrt() < 0.6, "F short radius");
        assert!((edx * edx + edy * edy).sqrt() < 0.6, "E short radius");
    }

    #[test]
    fn prop45_matches_english_plate() {
        let d = book1_prop45();
        let hm = d.at("H").dist(d.at("M"));
        assert!((hm - 100.0).abs() < 0.05, "HM scale {hm}");
        assert!((d.at("K").y - d.at("H").y).abs() < 1e-6, "KH level");
        assert!(d.at("K").x < d.at("H").x && d.at("H").x < d.at("M").x);
        assert!((d.at("F").y - d.at("G").y).abs() < 1e-6, "FG level");
        assert!((d.at("G").y - d.at("L").y).abs() < 1e-6, "GL level");
        assert!(d.at("G").on_seg(d.at("F"), d.at("L"), 1e-3), "G on FL");
        assert!(d.at("H").on_seg(d.at("K"), d.at("M"), 1e-3), "H on KM");
        assert!(d.edge_id("D", "B").is_some(), "DB joined");
        // The plate draws the sides, not the diagonals. FH is FK–KH–HG–GF;
        // GM is GH–HM–ML–LG.
        let segs = d.stroke_ids();
        assert!(d.edge_id("F", "H").is_none(), "plate has no FH diagonal: {segs:?}");
        assert!(d.edge_id("G", "M").is_none(), "plate has no GM diagonal: {segs:?}");
        for (a, b) in [("F", "K"), ("K", "H"), ("H", "G"), ("G", "F"), ("G", "H"), ("H", "M"), ("M", "L"), ("L", "G")] {
            assert!(d.edge_id(a, b).is_some(), "parallelogram side {a}{b}: {segs:?}");
        }
        assert!(d.edge_id("K", "F").is_some() && d.edge_id("M", "L").is_some());
        // Page-45 English path: height/HM = 800/3, upright run/HM = 200/3.
        let height = (d.at("F").y - d.at("H").y) / hm;
        assert!((height - 8.0 / 3.0).abs() < 0.01, "height {height}");
        let run = (d.at("F").x - d.at("K").x) / hm;
        assert!((run - 2.0 / 3.0).abs() < 0.01, "upright run {run}");
        // KFLM is the outer parallelogram. H and G lie on its sides, so the
        // spoken name lights those pieces rather than a diagonal.
        let kflm = d.highlight("KFLM");
        assert!(
            has(&kflm, "kf") && has(&kflm, "fg") && has(&kflm, "gl") && has(&kflm, "ml") && has(&kflm, "kh") && has(&kflm, "hm"),
            "KFLM: {kflm:?}"
        );
        assert!(!kflm.iter().any(|id| id == "fh" || id == "gm"), "no diagonal: {kflm:?}");
        // FH and GM are the parallelograms the proof names, by opposite
        // corners. Neither name is a diagonal, and neither walks the long
        // way around KFLM.
        let fh = d.highlight("FH");
        assert!(
            has(&fh, "kf") && has(&fh, "kh") && has(&fh, "hg") && has(&fh, "fg"),
            "parallelogram FH: {fh:?}"
        );
        assert!(!has(&fh, "gl") && !has(&fh, "ml") && !has(&fh, "hm"), "FH is not the outer path: {fh:?}");
        let gm = d.highlight("GM");
        assert!(
            has(&gm, "hg") && has(&gm, "hm") && has(&gm, "ml") && has(&gm, "gl"),
            "parallelogram GM: {gm:?}"
        );
        assert!(!has(&gm, "fg") && !has(&gm, "kf") && !has(&gm, "kh"), "GM is not the outer path: {gm:?}");
        let abd = d.highlight("ABD");
        assert!(has(&abd, "ad") && has(&abd, "db") && has(&abd, "ba"), "triangle ABD: {abd:?}");
        assert!(!has(&abd, "dc") && !has(&abd, "bc"), "ABD is not the quadrilateral: {abd:?}");
        let dbc = d.highlight("DBC");
        assert!(has(&dbc, "db") && has(&dbc, "cb") && has(&dbc, "dc"), "triangle DBC: {dbc:?}");
        let abcd = d.highlight("ABCD");
        assert!(
            has(&abcd, "ad") && has(&abcd, "dc") && has(&abcd, "cb") && has(&abcd, "ba"),
            "rectilinear ABCD: {abcd:?}"
        );
        assert!(!has(&abcd, "db"), "ABCD is not its diagonal: {abcd:?}");
        let e = d.highlight("E");
        assert!(has(&e, "ang-e"), "given angle E: {e:?}");
        let hkf = d.highlight_angle("HKF");
        assert!(has(&hkf, "kh") && has(&hkf, "kf") && !has(&hkf, "fg"), "∠HKF is the two rays: {hkf:?}");
        let ghm = d.highlight_angle("GHM");
        assert!(has(&ghm, "hg") && has(&ghm, "hm") && !has(&ghm, "ml"), "∠GHM is the two rays: {ghm:?}");
        let fkm = d.highlight_angle("FKM");
        assert!(has(&fkm, "kf") && has(&fkm, "kh") && has(&fkm, "hm") && !has(&fkm, "ml"), "∠FKM is the two rays: {fkm:?}");
    }

    #[test]
    fn prop46_matches_english_plate() {
        let d = book1_prop46();
        let ab = d.at("A").dist(d.at("B"));
        assert!((ab - 100.0).abs() < 0.05, "AB scale {ab}");
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-6, "AB level");
        assert!((d.at("D").y - d.at("E").y).abs() < 1e-6, "DE level");
        assert!((d.at("A").x - d.at("D").x).abs() < 1e-6, "AD plumb");
        assert!((d.at("B").x - d.at("E").x).abs() < 1e-6, "BE plumb");
        assert!((d.at("A").dist(d.at("D")) - ab).abs() < 0.05, "AD = AB");
        assert!((d.at("D").dist(d.at("E")) - ab).abs() < 0.05, "DE = AB");
        assert!(d.at("D").on_seg(d.at("C"), d.at("A"), 1e-3), "D on CA");
        assert!(d.at("C").y > d.at("D").y, "C past D");
        assert!((angle_at(&d, "B", "A", "D").to_degrees() - 90.0).abs() < 0.05, "right angle at A");
    }

    #[test]
    fn prop47_matches_english_plate() {
        let d = book1_prop47();
        let bc = d.at("B").dist(d.at("C"));
        assert!((bc - 100.0).abs() < 0.05, "BC scale {bc}");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-6, "BC level");
        assert!((d.at("D").y - d.at("E").y).abs() < 1e-6, "DE level");
        assert!((d.at("B").x - d.at("D").x).abs() < 1e-6, "BD plumb");
        assert!((d.at("C").x - d.at("E").x).abs() < 1e-6, "CE plumb");
        assert!((d.at("A").x - d.at("L").x).abs() < 1e-6, "AL plumb");
        assert!(d.at("L").on_seg(d.at("D"), d.at("E"), 1e-3), "L on DE");
        assert!((angle_at(&d, "B", "A", "C").to_degrees() - 85.4).abs() < 0.2, "plate angle BAC");
        assert!((d.at("A").dist(d.at("B")) / bc - 0.469).abs() < 0.01, "AB/BC");
        assert!((d.at("A").dist(d.at("C")) / bc - 0.921).abs() < 0.01, "AC/BC");
        assert!((d.at("F").dist(d.at("B")) / bc - 0.470).abs() < 0.01, "FB/BC");
        assert!((d.at("A").dist(d.at("G")) / bc - 0.471).abs() < 0.01, "AG/BC");
        assert!((d.at("A").dist(d.at("H")) / bc - 0.939).abs() < 0.01, "AH/BC");
        assert!((d.at("C").dist(d.at("K")) / bc - 0.934).abs() < 0.01, "CK/BC");
        for (a, b) in [("A", "D"), ("F", "C"), ("A", "E"), ("B", "K"), ("A", "P"), ("P", "L"), ("B", "P"), ("P", "C")] {
            assert!(d.edge_id(a, b).is_some(), "stroke {a}{b}");
        }
        assert!(d.edge_id("A", "L").is_none(), "AL is split at the crossing");
        assert!(d.edge_id("B", "C").is_none(), "BC is split at the crossing");
        assert!(d.at("P").on_seg(d.at("A"), d.at("L"), 1e-3), "P on AL");
        assert!(d.at("P").on_seg(d.at("B"), d.at("C"), 1e-3), "P on BC");
        let bdec = d.highlight("BDEC");
        assert!(
            has(&bdec, "bd") && has(&bdec, "dl") && has(&bdec, "le") && has(&bdec, "ce") && has(&bdec, "bp") && has(&bdec, "pc"),
            "square BDEC: {bdec:?}"
        );
        assert!(!has(&bdec, "ad") && !has(&bdec, "fc"), "BDEC is not the joins: {bdec:?}");
        let gb = d.highlight("GB");
        assert!(
            has(&gb, "fg") && has(&gb, "ga") && has(&gb, "ab") && has(&gb, "fb"),
            "square GB: {gb:?}"
        );
        assert!(!has(&gb, "ah") && !has(&gb, "fc"), "GB is not a walk off the square: {gb:?}");
        let hc = d.highlight("HC");
        assert!(
            has(&hc, "ah") && has(&hc, "ac") && has(&hc, "kc") && has(&hc, "hk"),
            "square HC: {hc:?}"
        );
        assert!(!has(&hc, "ab") && !has(&hc, "ce"), "HC is not a walk off the square: {hc:?}");
        let bl = d.highlight("BL");
        assert!(
            has(&bl, "bd") && has(&bl, "dl") && has(&bl, "pl") && has(&bl, "bp") && !has(&bl, "ab") && !has(&bl, "ap"),
            "parallelogram BL, not the path through A: {bl:?}"
        );
        let cl = d.highlight("CL");
        assert!(
            has(&cl, "ce") && has(&cl, "le") && has(&cl, "pl") && has(&cl, "pc") && !has(&cl, "ac") && !has(&cl, "ap"),
            "parallelogram CL, not the path through A: {cl:?}"
        );
        let gc = d.highlight("GC");
        assert!(
            has(&gc, "ga") && has(&gc, "ac") && !has(&gc, "fb") && !has(&gc, "hk"),
            "GC is the straight-on GA–AC, not a walk: {gc:?}"
        );
    }

    #[test]
    fn prop48_matches_english_plate() {
        let d = book1_prop48();
        let db = d.at("D").dist(d.at("B"));
        assert!((db - 100.0).abs() < 0.05, "DB scale {db}");
        assert!((d.at("D").y - d.at("B").y).abs() < 1e-6, "DB level");
        assert!(d.at("A").on_seg(d.at("D"), d.at("B"), 1e-6), "A on DB");
        assert!((d.at("D").dist(d.at("A")) - d.at("A").dist(d.at("B"))).abs() < 1e-6, "DA = AB");
        assert!((d.at("A").x - d.at("C").x).abs() < 1e-6, "AC plumb");
        assert!(((d.at("C").y - d.at("A").y) / db - 7.0 / 6.0).abs() < 0.01, "AC/DB");
        assert!((angle_at(&d, "D", "A", "C").to_degrees() - 90.0).abs() < 0.05, "right angle DAC");
        assert!((angle_at(&d, "B", "A", "C").to_degrees() - 90.0).abs() < 0.05, "right angle BAC");
        for (a, b) in [("D", "A"), ("A", "B"), ("A", "C"), ("D", "C"), ("B", "C")] {
            assert!(d.edge_id(a, b).is_some(), "stroke {a}{b}");
        }
    }

    #[test]
    fn prop2_1_matches_english_plate() {
        let d = book2_prop1();
        let bc = d.at("B").dist(d.at("C"));
        assert!((bc - 100.0).abs() < 0.05, "BC scale {bc}");
        assert!((d.at("B").y - d.at("C").y).abs() < 1e-6, "BC level");
        assert!((d.at("G").y - d.at("H").y).abs() < 1e-6, "GH level");
        assert!((d.at("B").x - d.at("G").x).abs() < 1e-6, "BG plumb");
        assert!((d.at("D").x - d.at("K").x).abs() < 1e-6, "DK plumb");
        assert!((d.at("E").x - d.at("L").x).abs() < 1e-6, "EL plumb");
        assert!((d.at("C").x - d.at("H").x).abs() < 1e-6, "CH plumb");
        assert!(d.at("D").on_seg(d.at("B"), d.at("C"), 1e-3), "D on BC");
        assert!(d.at("E").on_seg(d.at("B"), d.at("C"), 1e-3), "E on BC");
        assert!(d.at("K").on_seg(d.at("G"), d.at("H"), 1e-3), "K on GH");
        assert!(d.at("G").on_seg(d.at("F"), d.at("B"), 1e-3), "G on FB");
        assert!((d.at("B").dist(d.at("G")) / bc - 0.499).abs() < 0.002, "BG/BC");
        assert!((d.at("B").dist(d.at("D")) / bc - 0.443).abs() < 0.002, "BD/BC");
        assert!((d.at("B").dist(d.at("E")) / bc - 0.723).abs() < 0.002, "BE/BC");
        assert!((angle_at(&d, "G", "B", "C").to_degrees() - 90.0).abs() < 0.05, "right angle at B");
        let a = d.highlight("A");
        assert!(has(&a, "seg-a") && has(&a, "A"), "A is the given line: {a:?}");
        assert!(!has(&a, "bg") && !has(&a, "bc"), "A is not the rectangle: {a:?}");
        // English plate: the capital is the left end, not the middle of the stroke.
        assert!(
            (d.at("A").x - d.at("A0").x).abs() < 1e-6 && d.at("A").x + 20.0 < d.at("A1").x,
            "A is the left end of the given line"
        );
        let html = d.svg(&[] as &[String]);
        let (adx, ady) = letter_offset(&html, "A");
        assert!(
            adx < -0.4 && ady < -0.4,
            "A above-left of the left end, not on the stroke, --dx={adx} --dy={ady}"
        );
        let bh = d.highlight("BH");
        assert!(
            has(&bh, "bd") && has(&bh, "de") && has(&bh, "ec") && has(&bh, "ch")
                && has(&bh, "lh") && has(&bh, "kl") && has(&bh, "gk") && has(&bh, "gb")
                && !has(&bh, "dk") && !has(&bh, "el"),
            "rectangle BH is the boundary, not a piece: {bh:?}"
        );
        let bk = d.highlight("BK");
        assert!(
            has(&bk, "bd") && has(&bk, "dk") && has(&bk, "gk") && has(&bk, "gb") && !has(&bk, "de"),
            "rectangle BK: {bk:?}"
        );
        let dl = d.highlight("DL");
        assert!(
            has(&dl, "de") && has(&dl, "el") && has(&dl, "kl") && has(&dl, "dk") && !has(&dl, "ec"),
            "rectangle DL: {dl:?}"
        );
        let eh = d.highlight("EH");
        assert!(
            has(&eh, "ec") && has(&eh, "ch") && has(&eh, "lh") && has(&eh, "el") && !has(&eh, "bd"),
            "rectangle EH: {eh:?}"
        );
        let bc_line = d.highlight("BC");
        assert!(has(&bc_line, "bd") && has(&bc_line, "de") && has(&bc_line, "ec"), "BC is the whole line: {bc_line:?}");
        let bd = d.highlight("BD");
        assert!(has(&bd, "bd") && !has(&bd, "de"), "BD is the piece: {bd:?}");
    }

    #[test]
    fn prop2_2_matches_english_plate() {
        let d = book2_prop2();
        let ab = d.at("A").dist(d.at("B"));
        assert!((ab - 100.0).abs() < 0.05, "AB scale {ab}");
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-6, "AB level");
        assert!((d.at("D").y - d.at("E").y).abs() < 1e-6, "DE level");
        assert!((d.at("A").x - d.at("D").x).abs() < 1e-6, "AD plumb");
        assert!((d.at("C").x - d.at("F").x).abs() < 1e-6, "CF plumb");
        assert!((d.at("B").x - d.at("E").x).abs() < 1e-6, "BE plumb");
        assert!(d.at("C").on_seg(d.at("A"), d.at("B"), 1e-3), "C on AB");
        assert!(d.at("F").on_seg(d.at("D"), d.at("E"), 1e-3), "F on DE");
        assert!((d.at("A").dist(d.at("D")) - ab).abs() < 0.05, "AD = AB");
        assert!((d.at("D").dist(d.at("E")) - ab).abs() < 0.05, "DE = AB");
        assert!((d.at("A").dist(d.at("C")) / ab - 0.2736).abs() < 0.001, "AC/AB from the plate");
        assert!((angle_at(&d, "D", "A", "B").to_degrees() - 90.0).abs() < 0.05, "right angle at A");
        assert!((angle_at(&d, "A", "B", "E").to_degrees() - 90.0).abs() < 0.05, "right angle at B");
        let adeb = d.highlight("ADEB");
        assert!(
            has(&adeb, "ad") && has(&adeb, "df") && has(&adeb, "fe") && has(&adeb, "be")
                && has(&adeb, "ac") && has(&adeb, "cb") && !has(&adeb, "cf"),
            "square ADEB is the boundary, not the cut: {adeb:?}"
        );
        let ae = d.highlight("AE");
        assert!(
            has(&ae, "ac") && has(&ae, "cb") && has(&ae, "be") && has(&ae, "fe") && has(&ae, "df")
                && has(&ae, "ad") && !has(&ae, "cf"),
            "square AE is the boundary, not the diagonal or the cut: {ae:?}"
        );
        let af = d.highlight("AF");
        assert!(
            has(&af, "ac") && has(&af, "cf") && has(&af, "df") && has(&af, "ad") && !has(&af, "cb"),
            "rectangle AF: {af:?}"
        );
        let ce = d.highlight("CE");
        assert!(
            has(&ce, "cb") && has(&ce, "be") && has(&ce, "fe") && has(&ce, "cf") && !has(&ce, "ac"),
            "rectangle CE: {ce:?}"
        );
        let ab_line = d.highlight("AB");
        assert!(has(&ab_line, "ac") && has(&ab_line, "cb") && !has(&ab_line, "ad"), "AB is the whole line: {ab_line:?}");
        let ac = d.highlight("AC");
        assert!(has(&ac, "ac") && !has(&ac, "cb"), "AC is the piece: {ac:?}");
        let bc = d.highlight("BC");
        assert!(has(&bc, "cb") && !has(&bc, "ac"), "BC is the piece: {bc:?}");
        let cf = d.highlight("CF");
        assert!(has(&cf, "cf") && !has(&cf, "ac") && !has(&cf, "cb"), "CF is the parallel: {cf:?}");
    }


    #[test]
    fn prop2_3_matches_english_plate() {
        let d = book2_prop3();
        let ab = d.at("A").dist(d.at("B"));
        assert!((ab - 100.0).abs() < 0.05, "AB scale {ab}");
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-6, "AB level");
        assert!((d.at("F").y - d.at("E").y).abs() < 1e-6, "FE level");
        assert!((d.at("A").x - d.at("F").x).abs() < 1e-6, "AF plumb");
        assert!((d.at("C").x - d.at("D").x).abs() < 1e-6, "CD plumb");
        assert!((d.at("B").x - d.at("E").x).abs() < 1e-6, "BE plumb");
        assert!(d.at("C").on_seg(d.at("A"), d.at("B"), 1e-3), "C on AB");
        assert!(d.at("D").on_seg(d.at("F"), d.at("E"), 1e-3), "D on FE");
        let cb = d.at("C").dist(d.at("B"));
        assert!((d.at("C").dist(d.at("D")) - cb).abs() < 0.05, "CD = CB");
        assert!((d.at("D").dist(d.at("E")) - cb).abs() < 0.05, "DE = CB");
        assert!((d.at("B").dist(d.at("E")) - cb).abs() < 0.05, "BE = CB");
        assert!((d.at("A").dist(d.at("C")) / ab - 1.0 / 3.0).abs() < 0.001, "AC/AB from the plate");
        assert!((cb / ab - 2.0 / 3.0).abs() < 0.001, "CB/AB from the plate");
        assert!((angle_at(&d, "F", "A", "B").to_degrees() - 90.0).abs() < 0.05, "right angle at A");
        assert!((angle_at(&d, "A", "B", "E").to_degrees() - 90.0).abs() < 0.05, "right angle at B");
        assert!((angle_at(&d, "D", "C", "B").to_degrees() - 90.0).abs() < 0.05, "right angle at C");
        let cdeb = d.highlight("CDEB");
        assert!(
            has(&cdeb, "cd") && has(&cdeb, "de") && has(&cdeb, "be") && has(&cdeb, "cb") && !has(&cdeb, "ac")
                && !has(&cdeb, "fd"),
            "square CDEB is the boundary, not the production: {cdeb:?}"
        );
        let ae = d.highlight("AE");
        assert!(
            has(&ae, "ac") && has(&ae, "cb") && has(&ae, "be") && has(&ae, "de") && has(&ae, "fd")
                && has(&ae, "af") && !has(&ae, "cd"),
            "rectangle AE is the boundary, not the diagonal or the cut: {ae:?}"
        );
        let ad = d.highlight("AD");
        assert!(
            has(&ad, "ac") && has(&ad, "cd") && has(&ad, "fd") && has(&ad, "af") && !has(&ad, "cb"),
            "rectangle AD: {ad:?}"
        );
        let ce = d.highlight("CE");
        assert!(
            has(&ce, "cb") && has(&ce, "be") && has(&ce, "de") && has(&ce, "cd") && !has(&ce, "ac"),
            "square CE: {ce:?}"
        );
        let db = d.highlight("DB");
        assert!(
            has(&db, "de") && has(&db, "be") && has(&db, "cb") && has(&db, "cd") && !has(&db, "fd")
                && !has(&db, "ac"),
            "square DB is the boundary, not the diagonal: {db:?}"
        );
        let ab_line = d.highlight("AB");
        assert!(has(&ab_line, "ac") && has(&ab_line, "cb") && !has(&ab_line, "af"), "AB is the whole line: {ab_line:?}");
        let ac = d.highlight("AC");
        assert!(has(&ac, "ac") && !has(&ac, "cb"), "AC is the piece: {ac:?}");
        let bc = d.highlight("BC");
        assert!(has(&bc, "cb") && !has(&bc, "ac"), "BC is the piece: {bc:?}");
        let cb_line = d.highlight("CB");
        assert!(has(&cb_line, "cb") && !has(&cb_line, "ac"), "CB is the piece: {cb_line:?}");
        let af = d.highlight("AF");
        assert!(has(&af, "af") && !has(&af, "ac"), "AF is the parallel: {af:?}");
        let ef = d.highlight("EF");
        assert!(has(&ef, "de") && has(&ef, "fd") && !has(&ef, "be"), "EF is the side drawn through to F: {ef:?}");
        let cd = d.highlight("CD");
        assert!(has(&cd, "cd") && !has(&cd, "de"), "CD is the side: {cd:?}");
        let be = d.highlight("BE");
        assert!(has(&be, "be") && !has(&be, "de"), "BE is the side: {be:?}");
        let dc = d.highlight("DC");
        assert!(has(&dc, "cd") && !has(&dc, "fd"), "DC is the side: {dc:?}");
    }

    fn letter_offset(html: &str, id: &str) -> (f64, f64) {
        let needle = format!("class=\"letter\" data-id=\"{id}\"");
        let start = html.find(&needle).expect(id);
        let chunk = &html[start..html[start..].find("</span>").map(|i| start + i).unwrap()];
        let grab = |key: &str| {
            let k = format!("{key}:");
            let i = chunk.find(&k).unwrap() + k.len();
            chunk[i..]
                .split(|c: char| c == 'e' || c == ';')
                .next()
                .unwrap()
                .parse::<f64>()
                .unwrap()
        };
        (grab("--dx"), grab("--dy"))
    }

    fn letter_percent(html: &str, id: &str) -> (f64, f64) {
        let needle = format!("class=\"letter\" data-id=\"{id}\"");
        let start = html.find(&needle).expect(id);
        let chunk = &html[start..html[start..].find("</span>").map(|i| start + i).unwrap()];
        let grab = |key: &str| {
            let k = format!("{key}:");
            let i = chunk.find(&k).unwrap() + k.len();
            chunk[i..]
                .split(|c: char| c == '%' || c == ';')
                .next()
                .unwrap()
                .parse::<f64>()
                .unwrap()
        };
        (grab("left"), grab("top"))
    }

    fn dot_xy(html: &str, id: &str) -> (f64, f64, f64, f64) {
        let vb = html.find("viewBox=\"0 0 ").unwrap() + "viewBox=\"0 0 ".len();
        let dims = &html[vb..html[vb..].find('"').map(|i| vb + i).unwrap()];
        let mut parts = dims.split_whitespace();
        let vb_w: f64 = parts.next().unwrap().parse().unwrap();
        let vb_h: f64 = parts.next().unwrap().parse().unwrap();
        let needle = format!("data-id=\"{id}\" cx=\"");
        let start = html.find(&needle).unwrap() + needle.len();
        let cx: f64 = html[start..].split('"').next().unwrap().parse().unwrap();
        let cy_at = html[start..].find("cy=\"").unwrap() + start + 4;
        let cy: f64 = html[cy_at..].split('"').next().unwrap().parse().unwrap();
        (cx, cy, vb_w, vb_h)
    }

    fn has_layer(html: &str) -> bool {
        html.contains("letter-layer") && html.contains("class=\"letter")
    }

    #[test]
    fn labels_share_plate_type() {
        use crate::figure::book1_prop1;
        let a = book1_prop1().svg(&[] as &[String]);
        let b = book1_prop2().svg(&[] as &[String]);
        assert!(has_layer(&a));
        assert!(has_layer(&b));
        assert!(a.contains("letter-layer"));
        assert!(a.contains("class=\"letter"));
        assert!(b.contains("class=\"letter"));
    }
}
