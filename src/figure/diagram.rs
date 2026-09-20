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
}

impl Diagram {
    pub fn new() -> Self {
        Self {
            fig: Figure::new(),
            pts: HashMap::new(),
            adj: HashMap::new(),
            circles: Vec::new(),
        }
    }

    pub fn at(&self, name: &str) -> V2 {
        *self
            .pts
            .get(name)
            .unwrap_or_else(|| panic!("unknown point {name}"))
    }

    /// Name a point and place its letter.
    pub fn put(&mut self, name: &'static str, p: V2, place: Place) -> V2 {
        self.pts.insert(name, p);
        self.fig.label_at(name, p, place);
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

    pub fn dots(&mut self, names: &[&'static str]) -> &mut Self {
        for name in names {
            let p = self.at(name);
            self.fig.dot(*name, p);
        }
        self
    }

    pub fn join(&mut self, a: &'static str, b: &'static str) -> &mut Self {
        let pa = self.at(a);
        let pb = self.at(b);
        let id = format!("{}{}", a.to_ascii_lowercase(), b.to_ascii_lowercase());
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

    pub fn clip(&mut self, min: V2, max: V2) -> &mut Self {
        self.fig.clip_box(min, max);
        self
    }

    pub fn svg<S: AsRef<str>>(&self, on: &[S]) -> String {
        self.fig.svg(on)
    }

    /// Figure ids to light for a spoken geometry word (`A`, `AL`, `DAB`, `CGH`).
    pub fn highlight(&self, word: &str) -> Vec<String> {
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
            1 => vec![letters[0].to_string()],
            2 => self.stroke(letters[0], letters[1]),
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

    fn stroke(&self, a: &'static str, b: &'static str) -> Vec<String> {
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
        let mut out = self.stroke(a, b);
        out.extend(self.stroke(b, c));
        out.extend(self.stroke(c, a));
        out.sort();
        out.dedup();
        out
    }

    fn named_circle(&self, key: &str) -> Option<String> {
        let want = sorted_letters(key);
        self.circles.iter().find_map(|c| {
            (sorted_letters(&c.letters) == want).then_some(c.id.clone())
        })
    }

    fn path(&self, start: &'static str, goal: &'static str) -> Vec<String> {
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
    use crate::figure::book1_prop2;

    fn has(v: &[String], id: &str) -> bool {
        v.iter().any(|s| s == id)
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
