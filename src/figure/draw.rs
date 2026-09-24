//! SVG plate: geometry is fitted; letters are a separate overlay.

use super::geom::{Place, V2};
use super::labels::{self, Label};
use super::plate::{self, Fit, Size};

pub(crate) enum Mark {
    Circle { id: String, c: V2, r: f64 },
    Seg { id: String, a: V2, b: V2 },
    /// Circular arc `a` → `b` through `via`.
    Arc { id: String, a: V2, b: V2, via: V2 },
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

    pub(crate) fn arc(&mut self, id: impl Into<String>, a: V2, b: V2, via: V2) {
        self.marks.push(Mark::Arc {
            id: id.into(),
            a,
            b,
            via,
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
            r_em: plate::LETTER_R,
        });
    }

    pub(crate) fn label_at_r(
        &mut self,
        id: impl Into<String>,
        at: V2,
        place: Place,
        r_em: f64,
    ) {
        let id = id.into();
        self.labels.push(Label {
            text: id.clone(),
            id,
            at,
            place,
            r_em,
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
                at: fit.map(l.at),
                place: l.place,
                r_em: l.r_em,
            })
            .collect();
        let placed = labels::place(&marks, clip, &labels, fit.size);
        let vb = format!("0 0 {} {}", n(fit.size.w), n(fit.size.h));
        let mut out = String::new();
        out.push_str(&format!(
            r##"<div class="figure-frame" style="--fw:{};--fh:{};aspect-ratio: {} / {}">"##,
            n(fit.size.w),
            n(fit.size.h),
            n(fit.size.w),
            n(fit.size.h)
        ));
        out.push_str(&format!(
            r##"<svg class="figure" viewBox="{vb}" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" preserveAspectRatio="xMidYMid meet">"##
        ));
        if let Some((cmin, cmax)) = clip {
            out.push_str(&format!(
                r##"<defs><clipPath id="fig-clip"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath></defs>"##,
                n(cmin.x),
                n(fit.size.h - cmax.y),
                n(cmax.x - cmin.x),
                n(cmax.y - cmin.y)
            ));
        }
        let y = |v: f64| fit.size.h - v;
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
            match m {
                Mark::Circle { id, c, r } => {
                    out.push_str(&format!(
                        r##"<circle class="{}" cx="{}" cy="{}" r="{}"/>"##,
                        cls(id, "circ"),
                        n(c.x),
                        n(y(c.y)),
                        n(*r)
                    ));
                }
                Mark::Arc { id, a, b, via } => {
                    if let Some(d) = arc_d(*a, *b, *via, y) {
                        // Decorative bows stay stroke-only; never fill, never dim.
                        out.push_str(&format!(
                            r##"<path class="{}" d="{}" fill="none"/>"##,
                            cls(id, "arc"),
                            d
                        ));
                    }
                }
                _ => {}
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
        out.push_str(&labels::layer(&placed, on, lighting, fit.size));
        out.push_str("</div>");
        out
    }

    fn fit(&self) -> Fit {
        let (min, max) = self.geom_box();
        let gw = (max.x - min.x).max(1.0);
        let gh = (max.y - min.y).max(1.0);
        let scale = 1000.0 / gw.max(gh);
        let size = Size {
            w: gw * scale + 2.0 * plate::MARGIN,
            h: gh * scale + 2.0 * plate::MARGIN,
        };
        let ox = plate::MARGIN - min.x * scale;
        let oy = plate::MARGIN - min.y * scale;
        Fit { ox, oy, scale, size }
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
                Mark::Arc { a, b, via, .. } => {
                    add(*a);
                    add(*b);
                    add(*via);
                }
                Mark::Dot { p, .. } => add(*p),
            }
        }
        if !min_x.is_finite() {
            return (V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        }
        (V2::new(min_x, min_y), V2::new(max_x, max_y))
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
        Mark::Arc { id, a, b, via } => Mark::Arc {
            id: id.clone(),
            a: fit.map(*a),
            b: fit.map(*b),
            via: fit.map(*via),
        },
        Mark::Dot { id, p } => Mark::Dot {
            id: id.clone(),
            p: fit.map(*p),
        },
    }
}

fn circle3(a: V2, b: V2, c: V2) -> Option<(V2, f64)> {
    let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    if d.abs() < 1e-9 {
        return None;
    }
    let a2 = a.x * a.x + a.y * a.y;
    let b2 = b.x * b.x + b.y * b.y;
    let c2 = c.x * c.x + c.y * c.y;
    let ux = (a2 * (b.y - c.y) + b2 * (c.y - a.y) + c2 * (a.y - b.y)) / d;
    let uy = (a2 * (c.x - b.x) + b2 * (a.x - c.x) + c2 * (b.x - a.x)) / d;
    let o = V2::new(ux, uy);
    Some((o, o.dist(a)))
}

fn arc_d(a: V2, b: V2, via: V2, y: impl Fn(f64) -> f64) -> Option<String> {
    let (c, r) = circle3(a, b, via)?;
    let ang = |p: V2| (p.y - c.y).atan2(p.x - c.x);
    let mut sweep = (ang(b) - ang(a)).rem_euclid(std::f64::consts::TAU);
    let via_off = (ang(via) - ang(a)).rem_euclid(std::f64::consts::TAU);
    if via_off > sweep {
        sweep = std::f64::consts::TAU - sweep;
    }
    let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
    // y-up CCW becomes clockwise in SVG (y-down).
    let ccw = (b.x - a.x) * (via.y - a.y) - (b.y - a.y) * (via.x - a.x) > 0.0;
    let sweep_flag = if ccw { 1 } else { 0 };
    Some(format!(
        "M {} {} A {} {} 0 {} {} {} {}",
        n(a.x),
        n(y(a.y)),
        n(r),
        n(r),
        large,
        sweep_flag,
        n(b.x),
        n(y(b.y))
    ))
}

fn n(v: f64) -> String {
    let s = format!("{v:.2}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}
