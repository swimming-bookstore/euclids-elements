//! Point-letter subsystem.
//!
//! Every letter’s **center** sits at a fixed radius from its point.
//! Direction is chosen to clear lines and other letters; distance is not.

use super::draw::Mark;
use super::geom::{Place, V2};
use super::plate;

pub(crate) struct Label {
    pub id: String,
    pub text: String,
    pub at: V2,
    pub place: Place,
}

pub(crate) struct Placed {
    pub id: String,
    pub text: String,
    pub x: f64,
    pub y: f64,
}

pub(crate) fn place(marks: &[Mark], clip: Option<(V2, V2)>, labels: &[Label]) -> Vec<Placed> {
    let mut out = Vec::new();
    for lab in labels {
        let p = best_center(marks, clip, lab, &out);
        out.push(Placed {
            id: lab.id.clone(),
            text: lab.text.clone(),
            x: p.x,
            y: p.y,
        });
    }
    out
}

/// HTML overlay. Size comes from CSS (`cqmin` of the figure frame).
pub(crate) fn layer<S: AsRef<str>>(placed: &[Placed], on: &[S], lighting: bool) -> String {
    let lit = |id: &str| on.iter().any(|p| p.as_ref() == id);
    let mut out = String::from(r##"<div class="letter-layer">"##);
    for lab in placed {
        let state = if !lighting {
            ""
        } else if lit(&lab.id) {
            " on"
        } else {
            " dim"
        };
        let left = lab.x / plate::WIDTH * 100.0;
        let top = (plate::HEIGHT - lab.y) / plate::HEIGHT * 100.0;
        out.push_str(&format!(
            r##"<span class="letter{state}" data-id="{}" style="left:{}%;top:{}%">{}</span>"##,
            lab.id,
            n(left),
            n(top),
            lab.text
        ));
    }
    out.push_str("</div>");
    out
}

fn best_center(marks: &[Mark], clip: Option<(V2, V2)>, lab: &Label, taken: &[Placed]) -> V2 {
    let at = lab.at;
    let prefer = lab.place.angle();
    let mut cands: Vec<f64> = (0..24)
        .map(|i| i as f64 * std::f64::consts::PI / 12.0)
        .collect();
    for extra in hint_angles(marks, at) {
        cands.push(extra);
    }
    if let Some(a) = prefer {
        cands.push(a);
    }
    let mut best = at.add(V2::new(plate::GAP, 0.0));
    let mut best_s = f64::INFINITY;
    for ang in cands {
        let p = at.add(V2::new(ang.cos(), ang.sin()).mul(plate::GAP));
        let s = score(marks, clip, lab, at, p, prefer, ang, taken);
        if s < best_s {
            best_s = s;
            best = p;
        }
    }
    debug_assert!((best.dist(at) - plate::GAP).abs() < 1e-6);
    best
}

fn hint_angles(marks: &[Mark], at: V2) -> Vec<f64> {
    let mut v = Vec::new();
    for m in marks {
        if let Mark::Circle { c, r, .. } = m {
            if (at.dist(*c) - *r).abs() < plate::FONT * 0.25 {
                let u = at.sub(*c).unit();
                v.push(u.y.atan2(u.x));
            }
        }
    }
    v
}

fn score(
    marks: &[Mark],
    clip: Option<(V2, V2)>,
    lab: &Label,
    at: V2,
    p: V2,
    prefer: Option<f64>,
    ang: f64,
    taken: &[Placed],
) -> f64 {
    let mut s = 0.0;
    let (x0, y0, x1, y1) = text_box(&lab.text, p);
    let mid = p;
    let near = plate::FONT * 0.45;

    for m in marks {
        match m {
            Mark::Seg { a, b, .. } => {
                let d = dist_seg(mid, *a, *b);
                if d < near {
                    s += (near - d) * (near - d);
                }
            }
            Mark::Circle { c, r, .. } => {
                let d = (mid.dist(*c) - *r).abs();
                if d < near {
                    s += (near - d) * 8.0;
                }
            }
            Mark::Dot { p: q, .. } => {
                if q.dist(mid) < plate::FONT * 0.5 {
                    s += 40.0;
                }
            }
        }
    }
    for t in taken {
        let (u0, v0, u1, v1) = text_box(&t.text, V2::new(t.x, t.y));
        if x0 < u1 && x1 > u0 && y0 < v1 && y1 > v0 {
            s += 120.0;
        }
        let d = mid.dist(V2::new(t.x, t.y));
        if d < plate::FONT {
            s += (plate::FONT - d) * 2.0;
        }
    }
    let pad = plate::FONT * 0.2;
    if x0 < pad || y0 < pad || x1 > plate::WIDTH - pad || y1 > plate::HEIGHT - pad {
        s += 40.0;
    }
    if let Some((min, max)) = clip {
        if p.x < min.x - pad || p.y < min.y - pad || p.x > max.x + pad || p.y > max.y + pad {
            s += 6.0;
        }
    }
    if let Some(pref) = prefer {
        let mut d = (ang - pref).abs() % (2.0 * std::f64::consts::PI);
        if d > std::f64::consts::PI {
            d = 2.0 * std::f64::consts::PI - d;
        }
        s += d * 14.0;
    }
    let _ = at;
    s
}

fn dist_seg(p: V2, a: V2, b: V2) -> f64 {
    let ab = b.sub(a);
    let l2 = ab.dot(ab);
    if l2 < 1e-12 {
        return p.dist(a);
    }
    let t = ((p.sub(a).dot(ab)) / l2).clamp(0.0, 1.0);
    p.dist(a.add(ab.mul(t)))
}

fn text_box(text: &str, center: V2) -> (f64, f64, f64, f64) {
    let w = plate::CHAR_W * text.chars().count() as f64;
    let h = plate::FONT * 0.85;
    (
        center.x - w / 2.0,
        center.y - h / 2.0,
        center.x + w / 2.0,
        center.y + h / 2.0,
    )
}

fn n(v: f64) -> String {
    let s = format!("{v:.2}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::book1_prop2;
    use crate::figure::draw::Figure;
    use crate::figure::geom::Place;

    #[test]
    fn centers_share_radius() {
        let mut fig = Figure::new();
        fig.label_at("A", V2::new(100.0, 100.0), Place::Left);
        fig.label_at("B", V2::new(400.0, 100.0), Place::Right);
        fig.label_at("C", V2::new(250.0, 300.0), Place::Above);
        let labels = [
            Label {
                id: "A".into(),
                text: "A".into(),
                at: V2::new(100.0, 100.0),
                place: Place::Left,
            },
            Label {
                id: "B".into(),
                text: "B".into(),
                at: V2::new(400.0, 100.0),
                place: Place::Right,
            },
            Label {
                id: "C".into(),
                text: "C".into(),
                at: V2::new(250.0, 300.0),
                place: Place::Above,
            },
        ];
        let placed = place(&[], None, &labels);
        for (lab, p) in labels.iter().zip(placed.iter()) {
            let d = lab.at.dist(V2::new(p.x, p.y));
            assert!(
                (d - plate::GAP).abs() < 1e-6,
                "{} dist {d} != {}",
                lab.id,
                plate::GAP
            );
        }
        let _ = book1_prop2;
        let _ = fig;
    }
}
