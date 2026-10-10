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

/// A Euclidean figure: name the points, join them, draw circles.
pub struct Diagram {
    fig: Figure,
    pts: HashMap<&'static str, V2>,
    adj: HashMap<&'static str, Vec<Edge>>,
    circles: Vec<Circ>,
    named: Vec<Circ>,
}

impl Diagram {
    pub fn new() -> Self {
        Self {
            fig: Figure::new(),
            pts: HashMap::new(),
            adj: HashMap::new(),
            circles: Vec::new(),
            named: Vec::new(),
        }
    }

    pub fn at(&self, name: &str) -> V2 {
        *self
            .pts
            .get(name)
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

    /// Name a point; letter center at `r_em` from the mark.
    pub fn put_r(&mut self, name: &'static str, p: V2, place: Place, r_em: f64) -> V2 {
        self.pin(name, p);
        self.fig.label_at_r(name, p, place, r_em);
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

    pub fn dots(&mut self, names: &[&'static str]) -> &mut Self {
        for name in names {
            let p = self.at(name);
            self.fig.dot(*name, p);
        }
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
            1 => self.single(letters[0], &key),
            2 => self.pair(letters[0], letters[1]),
            3 if angle => self.angle(letters[0], letters[1], letters[2], &key),
            3 => self.triple(letters[0], letters[1], letters[2], &key),
            4 if !angle => self.quad(letters[0], letters[1], letters[2], letters[3]),
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

    fn single(&self, a: &'static str, key: &str) -> Vec<String> {
        let mut out = vec![a.to_string()];
        if let Some(id) = self.named_id(key) {
            out.push(id);
            if let Some(edges) = self.adj.get(a) {
                for e in edges {
                    if e.id == out[1] {
                        out.push(e.to.to_string());
                    }
                }
            }
        }
        out
    }

    fn pair(&self, a: &'static str, b: &'static str) -> Vec<String> {
        let mut out = vec![a.to_string(), b.to_string()];
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

    /// A four-letter figure name (*ACDB*) is the polygon of those vertices in order.
    fn quad(
        &self,
        a: &'static str,
        b: &'static str,
        c: &'static str,
        d: &'static str,
    ) -> Vec<String> {
        let mut out = self.pair(a, b);
        out.extend(self.pair(b, c));
        out.extend(self.pair(c, d));
        out.extend(self.pair(d, a));
        out.sort();
        out.dedup();
        out
    }

    fn angle(&self, a: &'static str, v: &'static str, c: &'static str, key: &str) -> Vec<String> {
        if let Some(circ) = self.named_circle(key) {
            return vec![circ, a.to_string(), v.to_string(), c.to_string()];
        }
        let mut out = self.pair(v, a);
        out.extend(self.pair(v, c));
        out.sort();
        out.dedup();
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
        let mut ids = Vec::new();
        for w in on.windows(2) {
            let id = self.edge_id(w[0].0, w[1].0)?;
            ids.push(id);
        }
        Some(ids)
    }

    fn edge_id(&self, a: &str, b: &str) -> Option<String> {
        self.adj.get(a)?.iter().find(|e| e.to == b).map(|e| e.id.clone())
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
    };

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
        assert!(has(&ab, "ab") || has(&ab, "ba"), "AB: {ab:?}");
        let cd = d.highlight("CD");
        assert!(has(&cd, "cd") || has(&cd, "dc"), "CD: {cd:?}");
        let ef = d.highlight("EF");
        assert!(has(&ef, "ef") || has(&ef, "fe"), "EF: {ef:?}");
        let bg = d.highlight("BG");
        assert!(has(&bg, "bg") || has(&bg, "gb"), "AB produced to G: {bg:?}");
        let dg = d.highlight("DG");
        assert!(has(&dg, "dg") || has(&dg, "gd"), "CD produced to G: {dg:?}");
        let aef = d.highlight_angle("AEF");
        assert!(
            (has(&aef, "ef") || has(&aef, "fe")),
            "∠AEF lights the transversal: {aef:?}"
        );
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9, "B level with A");
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9, "D level with C");
        assert!((d.at("A").x - d.at("C").x).abs() < 1e-9, "A plumb with C");
        assert!(d.at("A").y > d.at("C").y, "AB above CD");
        // G is the meeting of AB and CD produced, on the B/D side.
        assert!(d.at("G").x > d.at("B").x && d.at("G").x > d.at("D").x, "G beyond B and D");
        let ab_len = d.at("A").dist(d.at("B"));
        assert!((d.at("C").dist(d.at("D")) / ab_len - 1.083).abs() < 0.01, "CD/AB");
        // EF crosses both parallels; E and F are the crossings, not the tips.
        assert!((d.at("E").x - d.at("F").x).abs() > 10.0, "EF should lean");
        assert!(d.at("E").y > d.at("A").y && d.at("F").y < d.at("C").y, "EF crosses both");
        // G sits off the wedge, on the side away from EF.
        let html = d.svg(&[] as &[String]);
        let (gdx, gdy) = letter_offset(&html, "G");
        assert!(gdx > 0.3, "G right of the meeting, --dx={gdx} --dy={gdy}");
        // English plate: E on AB, glyph above the line; F on CD, glyph below.
        // CSS y is down. E at 90° points up the page (negative dy); F at -90°
        // points down (positive dy).
        let (edx, edy) = letter_offset(&html, "E");
        assert!(edy < -0.5 && edx.abs() < 0.2, "E above AB, --dx={edx} --dy={edy}");
        let (fdx, fdy) = letter_offset(&html, "F");
        assert!(fdy > 0.5 && fdx.abs() < 0.2, "F below CD, --dx={fdx} --dy={fdy}");
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
        assert!(edx > 0.2 && edy > 0.2, "E below-right of the tip, --dx={edx} --dy={edy}");
        assert!((d.at("A").dist(d.at("B")) / cd_len - 1.0).abs() < 0.01, "AB/CD");
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
        assert!(d.at("H").on_seg(d.at("E"), d.at("F"), 1e-3));
        assert!((d.at("A").y - d.at("B").y).abs() < 1e-9);
        assert!((d.at("E").y - d.at("F").y).abs() < 1e-9);
        assert!((d.at("C").y - d.at("D").y).abs() < 1e-9);
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
            has(&abcd, "ab") && has(&abcd, "ad") && has(&abcd, "dc") && has(&abcd, "bc"),
            "ABCD: {abcd:?}"
        );
        let ebcf = d.highlight("EBCF");
        assert!(
            has(&ebcf, "eb") && has(&ebcf, "bc") && has(&ebcf, "fc") && has(&ebcf, "ef"),
            "EBCF: {ebcf:?}"
        );
        let af = d.highlight("AF");
        assert!(has(&af, "ad") && has(&af, "de") && has(&af, "ef"), "AF: {af:?}");
        let ae = d.highlight("AE");
        assert!(has(&ae, "ad") && has(&ae, "de"), "AE: {ae:?}");
        let df = d.highlight("DF");
        assert!(has(&df, "de") && has(&df, "ef"), "DF: {df:?}");
        let eb = d.highlight("EB");
        assert!(has(&eb, "eb"), "EB: {eb:?}");
        let fc = d.highlight("FC");
        assert!(has(&fc, "fc"), "FC: {fc:?}");
        let eab = d.highlight("EAB");
        assert!(has(&eab, "ab") && has(&eab, "eb"), "△EAB: {eab:?}");
        let dfc = d.highlight("DFC");
        assert!(has(&dfc, "fc") && has(&dfc, "dc"), "△DFC: {dfc:?}");
        let gb = d.highlight("GB");
        assert!(has(&gb, "G") && has(&gb, "B"), "GB: {gb:?}");
        let gc = d.highlight("GC");
        assert!(has(&gc, "G") && has(&gc, "C"), "GC: {gc:?}");
        let gbc = d.highlight("GBC");
        assert!(has(&gbc, "G") && has(&gbc, "B") && has(&gbc, "C"), "△GBC: {gbc:?}");
        let fdc = d.highlight_angle("FDC");
        assert!(has(&fdc, "de") && has(&fdc, "dc"), "∠FDC: {fdc:?}");
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

    fn letter_offset(html: &str, id: &str) -> (f64, f64) {
        let needle = format!("data-id=\"{id}\"");
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
