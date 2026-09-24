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

    /// Circle named by three letters, center `center`, through `through`.
    pub fn circle(&mut self, letters: &'static str, center: &str, through: &str) -> &mut Self {
        let c = self.at(center);
        let r = c.dist(self.at(through));
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
    use crate::figure::{book1_prop1, book1_prop2, book1_prop3, book1_prop4, book1_prop5};

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
