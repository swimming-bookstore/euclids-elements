//! SVG plate: geometry is fitted; letters are a separate overlay.

use super::geom::{Place, V2};
use super::labels::{self, Label};
use super::plate::{self, Fit};

pub(crate) enum Mark {
    Circle { id: String, c: V2, r: f64 },
    Seg { id: String, a: V2, b: V2 },
    Dot { id: String, p: V2 },
}

#[derive(Default)]
pub(crate) struct Figure {
    marks: Vec<Mark>,
    labels: Vec<Label>,
    clip: Option<(V2, V2)>,
}

impl Figure {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn clip_box(&mut self, min: V2, max: V2) {
        self.clip = Some((min, max));
    }

    pub(crate) fn circle(&mut self, id: impl Into<String>, c: V2, r: f64) {
        self.marks.push(Mark::Circle {
            id: id.into(),
            c,
            r,
        });
    }

    pub(crate) fn seg(&mut self, id: impl Into<String>, a: V2, b: V2) {
        self.marks.push(Mark::Seg {
            id: id.into(),
            a,
            b,
        });
    }

    pub(crate) fn dot(&mut self, id: impl Into<String>, p: V2) {
        self.marks.push(Mark::Dot { id: id.into(), p });
    }

    pub(crate) fn label_at(&mut self, id: impl Into<String>, at: V2, place: Place) {
        let id = id.into();
        self.labels.push(Label {
            text: id.clone(),
            id,
            at,
            place,
        });
    }

    pub(crate) fn svg<S: AsRef<str>>(&self, on: &[S]) -> String {
        let lighting = !on.is_empty();
        let lit = |id: &str| on.iter().any(|p| p.as_ref() == id);
        let cls = |id: &str, extra: &str| {
            let state = if !lighting {
                ""
            } else if lit(id) {
                " on"
            } else {
                " dim"
            };
            format!("{extra}{state}")
        };
        let fit = self.fit();
        let marks: Vec<Mark> = self.marks.iter().map(|m| xform(m, fit)).collect();
        let clip = self.clip.map(|(a, b)| (fit.map(a), fit.map(b)));
        let labels: Vec<Label> = self
            .labels
            .iter()
            .map(|l| Label {
                id: l.id.clone(),
                text: l.text.clone(),
                at: fit.map(self.anchor(l.at)),
                place: l.place,
            })
            .collect();
        let placed = labels::place(&marks, clip, &labels);
        let vb = format!("0 0 {} {}", n(plate::WIDTH), n(plate::HEIGHT));
        let mut out = String::new();
        out.push_str(r##"<div class="figure-frame">"##);
        out.push_str(&format!(
            r##"<svg class="figure" viewBox="{vb}" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" preserveAspectRatio="xMidYMid meet">"##
        ));
        if let Some((cmin, cmax)) = clip {
            out.push_str(&format!(
                r##"<defs><clipPath id="fig-clip"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath></defs>"##,
                n(cmin.x),
                n(plate::HEIGHT - cmax.y),
                n(cmax.x - cmin.x),
                n(cmax.y - cmin.y)
            ));
        }
        let y = |v: f64| plate::HEIGHT - v;
        out.push_str(&format!(
            r##"<g fill="none" stroke="#111" stroke-width="{}"{}>"##,
            n(plate::STROKE),
            if clip.is_some() {
                r##" clip-path="url(#fig-clip)""##
            } else {
                ""
            }
        ));
        for m in &marks {
            if let Mark::Circle { id, c, r } = m {
                out.push_str(&format!(
                    r##"<circle class="{}" cx="{}" cy="{}" r="{}"/>"##,
                    cls(id, "circ"),
                    n(c.x),
                    n(y(c.y)),
                    n(*r)
                ));
            }
        }
        out.push_str("</g>");
        out.push_str(&format!(
            r##"<g stroke="#111" stroke-width="{}" stroke-linecap="round">"##,
            n(plate::STROKE + 0.4)
        ));
        for m in &marks {
            if let Mark::Seg { id, a, b } = m {
                out.push_str(&format!(
                    r##"<line class="{}" x1="{}" y1="{}" x2="{}" y2="{}"/>"##,
                    cls(id, "seg"),
                    n(a.x),
                    n(y(a.y)),
                    n(b.x),
                    n(y(b.y))
                ));
            }
        }
        out.push_str("</g>");
        out.push_str(r##"<g fill="#111">"##);
        for m in &marks {
            if let Mark::Dot { id, p } = m {
                out.push_str(&format!(
                    r##"<circle class="{}" cx="{}" cy="{}" r="{}"/>"##,
                    cls(id, "dot"),
                    n(p.x),
                    n(y(p.y)),
                    n(plate::DOT)
                ));
            }
        }
        out.push_str("</g>");
        out.push_str("</svg>");
        out.push_str(&labels::layer(&placed, on, lighting));
        out.push_str("</div>");
        out
    }

    fn fit(&self) -> Fit {
        let (min, max) = self.geom_box();
        let gw = (max.x - min.x).max(1.0);
        let gh = (max.y - min.y).max(1.0);
        let inner_w = plate::WIDTH - 2.0 * plate::MARGIN;
        let inner_h = plate::HEIGHT - 2.0 * plate::MARGIN;
        let scale = (inner_w / gw).min(inner_h / gh);
        let ox = (plate::WIDTH - gw * scale) / 2.0 - min.x * scale;
        let oy = (plate::HEIGHT - gh * scale) / 2.0 - min.y * scale;
        Fit { ox, oy, scale }
    }

    fn geom_box(&self) -> (V2, V2) {
        if let Some((min, max)) = self.clip {
            return (min, max);
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut add = |p: V2| {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        };
        for m in &self.marks {
            match m {
                Mark::Circle { c, r, .. } => {
                    add(V2::new(c.x - r, c.y - r));
                    add(V2::new(c.x + r, c.y + r));
                }
                Mark::Seg { a, b, .. } => {
                    add(*a);
                    add(*b);
                }
                Mark::Dot { p, .. } => add(*p),
            }
        }
        if !min_x.is_finite() {
            return (V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        }
        (V2::new(min_x, min_y), V2::new(max_x, max_y))
    }

    fn anchor(&self, at: V2) -> V2 {
        let Some((min, max)) = self.clip else {
            return at;
        };
        if inside(at, min, max) {
            return at;
        }
        let mut best: Option<V2> = None;
        let mut best_d = f64::INFINITY;
        for m in &self.marks {
            if let Mark::Seg { a, b, .. } = m {
                for (p, q) in [(*a, *b), (*b, *a)] {
                    if p.dist(at) > 1.5 {
                        continue;
                    }
                    if inside(q, min, max) {
                        let hit = exit(q, p, min, max);
                        let d = hit.dist(q);
                        if d < best_d {
                            best_d = d;
                            best = Some(hit);
                        }
                    }
                }
            }
        }
        best.unwrap_or_else(|| clamp(at, min, max))
    }
}

fn xform(m: &Mark, fit: Fit) -> Mark {
    match m {
        Mark::Circle { id, c, r } => Mark::Circle {
            id: id.clone(),
            c: fit.map(*c),
            r: fit.dist(*r),
        },
        Mark::Seg { id, a, b } => Mark::Seg {
            id: id.clone(),
            a: fit.map(*a),
            b: fit.map(*b),
        },
        Mark::Dot { id, p } => Mark::Dot {
            id: id.clone(),
            p: fit.map(*p),
        },
    }
}

fn inside(p: V2, min: V2, max: V2) -> bool {
    p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
}

fn clamp(p: V2, min: V2, max: V2) -> V2 {
    V2::new(p.x.clamp(min.x, max.x), p.y.clamp(min.y, max.y))
}

fn exit(inside_pt: V2, outside_pt: V2, min: V2, max: V2) -> V2 {
    let dx = outside_pt.x - inside_pt.x;
    let dy = outside_pt.y - inside_pt.y;
    let mut t: f64 = 1.0;
    if dx > 1e-9 {
        t = t.min((max.x - inside_pt.x) / dx);
    } else if dx < -1e-9 {
        t = t.min((min.x - inside_pt.x) / dx);
    }
    if dy > 1e-9 {
        t = t.min((max.y - inside_pt.y) / dy);
    } else if dy < -1e-9 {
        t = t.min((min.y - inside_pt.y) / dy);
    }
    t = t.clamp(0.0, 1.0);
    V2::new(inside_pt.x + t * dx, inside_pt.y + t * dy)
}

fn n(v: f64) -> String {
    let s = format!("{v:.2}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}
