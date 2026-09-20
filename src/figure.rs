/// Plane figures in y-up coordinates, emitted as SVG (y-down).

const LABEL_GAP: f64 = 16.0;
const PAD: f64 = 28.0;
const FONT: f64 = 20.0;
const STROKE: f64 = 1.6;
const DOT: f64 = 3.2;

#[derive(Clone, Copy)]
pub struct V2 {
    pub x: f64,
    pub y: f64,
}

impl V2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub enum Place {
    Left,
    Right,
    Above,
    Below,
}

enum Mark {
    Circle {
        id: &'static str,
        c: V2,
        r: f64,
    },
    Seg {
        id: &'static str,
        a: V2,
        b: V2,
    },
    Dot {
        id: &'static str,
        p: V2,
    },
}

struct Lab {
    id: &'static str,
    text: &'static str,
    at: V2,
    place: Place,
}

pub struct Figure {
    marks: Vec<Mark>,
    labels: Vec<Lab>,
    label_gap: f64,
}

impl Figure {
    pub fn new() -> Self {
        Self {
            marks: Vec::new(),
            labels: Vec::new(),
            label_gap: LABEL_GAP,
        }
    }

    pub fn circle(&mut self, id: &'static str, c: V2, r: f64) -> &mut Self {
        self.marks.push(Mark::Circle { id, c, r });
        self
    }

    pub fn seg(&mut self, id: &'static str, a: V2, b: V2) -> &mut Self {
        self.marks.push(Mark::Seg { id, a, b });
        self
    }

    pub fn dot(&mut self, id: &'static str, p: V2) -> &mut Self {
        self.marks.push(Mark::Dot { id, p });
        self
    }

    pub fn label(&mut self, id: &'static str, text: &'static str, at: V2, place: Place) -> &mut Self {
        self.labels.push(Lab {
            id,
            text,
            at,
            place,
        });
        self
    }

    pub fn svg(&self, on: &[&str]) -> String {
        let lighting = !on.is_empty();
        let cls = |id: &str, extra: &str| {
            let lit = on.iter().any(|p| *p == id);
            let state = if !lighting {
                ""
            } else if lit {
                " on"
            } else {
                " dim"
            };
            format!("{extra}{state}")
        };
        let (min_x, min_y, max_x, max_y) = self.bounds();
        let vb = format!(
            "{} {} {} {}",
            n(min_x),
            n(-max_y),
            n(max_x - min_x),
            n(max_y - min_y)
        );
        let mut out = String::new();
        out.push_str(&format!(
            r##"<svg class="figure" viewBox="{vb}" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" preserveAspectRatio="xMidYMid meet">"##
        ));
        out.push_str(&format!(
            r##"<g fill="none" stroke="#111" stroke-width="{}">"##,
            n(STROKE)
        ));
        for m in &self.marks {
            if let Mark::Circle { id, c, r } = m {
                out.push_str(&format!(
                    r##"<circle class="{}" cx="{}" cy="{}" r="{}"/>"##,
                    cls(id, "circ"),
                    n(c.x),
                    n(-c.y),
                    n(*r)
                ));
            }
        }
        out.push_str("</g>");
        out.push_str(&format!(
            r##"<g stroke="#111" stroke-width="{}" stroke-linecap="round">"##,
            n(STROKE + 0.4)
        ));
        for m in &self.marks {
            if let Mark::Seg { id, a, b } = m {
                out.push_str(&format!(
                    r##"<line class="{}" x1="{}" y1="{}" x2="{}" y2="{}"/>"##,
                    cls(id, "seg"),
                    n(a.x),
                    n(-a.y),
                    n(b.x),
                    n(-b.y)
                ));
            }
        }
        out.push_str("</g>");
        out.push_str(r##"<g fill="#111">"##);
        for m in &self.marks {
            if let Mark::Dot { id, p } = m {
                out.push_str(&format!(
                    r##"<circle class="{}" cx="{}" cy="{}" r="{}"/>"##,
                    cls(id, "dot"),
                    n(p.x),
                    n(-p.y),
                    n(DOT)
                ));
            }
        }
        out.push_str("</g>");
        out.push_str(&format!(
            r##"<g font-family="Georgia, 'Times New Roman', serif" font-size="{}" fill="#111">"##,
            n(FONT)
        ));
        for lab in &self.labels {
            let (x, y, anchor) = self.placed(lab);
            out.push_str(&format!(
                r##"<text class="{}" x="{}" y="{}" text-anchor="{anchor}" dominant-baseline="middle">{}</text>"##,
                cls(lab.id, "lab"),
                n(x),
                n(-y),
                lab.text
            ));
        }
        out.push_str("</g></svg>");
        out
    }

    fn placed(&self, lab: &Lab) -> (f64, f64, &'static str) {
        let g = self.label_gap;
        match lab.place {
            Place::Left => (lab.at.x - g, lab.at.y, "end"),
            Place::Right => (lab.at.x + g, lab.at.y, "start"),
            Place::Above => (lab.at.x, lab.at.y + g, "middle"),
            Place::Below => (lab.at.x, lab.at.y - g, "middle"),
        }
    }

    fn bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut add = |x: f64, y: f64| {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        };
        for m in &self.marks {
            match m {
                Mark::Circle { c, r, .. } => {
                    add(c.x - r, c.y - r);
                    add(c.x + r, c.y + r);
                }
                Mark::Seg { a, b, .. } => {
                    add(a.x, a.y);
                    add(b.x, b.y);
                }
                Mark::Dot { p, .. } => add(p.x, p.y),
            }
        }
        for lab in &self.labels {
            let (x, y, _) = self.placed(lab);
            add(x, y);
        }
        (min_x - PAD, min_y - PAD, max_x + PAD, max_y + PAD)
    }
}

fn n(v: f64) -> String {
    let s = format!("{v:.2}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// I.1 — equilateral triangle on AB. C above the base; D, E on the sides.
pub fn book1_prop1(on: &[&str]) -> String {
    let ab = 240.0;
    let a = V2::new(0.0, 0.0);
    let b = V2::new(ab, 0.0);
    let c = V2::new(ab / 2.0, ab * 3.0_f64.sqrt() / 2.0);
    let d = V2::new(-ab, 0.0);
    let e = V2::new(2.0 * ab, 0.0);
    let mut f = Figure::new();
    f.circle("circ-a", a, ab)
        .circle("circ-b", b, ab)
        .seg("ab", a, b)
        .seg("ac", a, c)
        .seg("bc", b, c)
        .dot("A", a)
        .dot("B", b)
        .dot("C", c)
        .label("A", "A", a, Place::Left)
        .label("B", "B", b, Place::Right)
        .label("C", "C", c, Place::Above)
        .label("D", "D", d, Place::Left)
        .label("E", "E", e, Place::Right);
    f.svg(on)
}
