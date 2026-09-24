//! Point-letter subsystem.
//!
//! The letter’s **center** sits at `LETTER_R` em from the mark (`--dx`/`--dy`)
//! — the same radius on every plate, at every size. Direction is the author’s
//! `Place` (only `Auto` may hunt).

use super::draw::Mark;
use super::geom::{Place, V2};
use super::plate;

pub(crate) struct Label {
    pub id: String,
    pub text: String,
    pub at: V2,
    pub place: Place,
    /// Center-to-mark radius, in em of the letter type.
    pub r_em: f64,
}

pub(crate) struct Placed {
    pub id: String,
    pub text: String,
    /// Mark, SVG y-up.
    pub at: V2,
    /// Direction of the letter center, y-up from +x.
    pub ang: f64,
    pub r_em: f64,
}

pub(crate) fn place(
    marks: &[Mark],
    clip: Option<(V2, V2)>,
    labels: &[Label],
    size: plate::Size,
) -> Vec<Placed> {
    let mut out = Vec::new();
    for lab in labels {
        let ang = direction(marks, clip, lab, &out, size);
        out.push(Placed {
            id: lab.id.clone(),
            text: lab.text.clone(),
            at: lab.at,
            ang,
            r_em: lab.r_em,
        });
    }
    out
}

/// HTML overlay. Size and radius come from CSS (`em` / `cqmin`).
pub(crate) fn layer<S: AsRef<str>>(
    placed: &[Placed],
    on: &[S],
    lighting: bool,
    size: plate::Size,
) -> String {
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
        let left = lab.at.x / size.w * 100.0;
        let top = (size.h - lab.at.y) / size.h * 100.0;
        // Same em radius on every plate. CSS y is down, so sin is flipped.
        let dx = lab.ang.cos() * lab.r_em;
        let dy = -lab.ang.sin() * lab.r_em;
        out.push_str(&format!(
            r##"<span class="letter{state}" data-id="{}" style="left:{}%;top:{}%;--dx:{}em;--dy:{}em">{}</span>"##,
            lab.id,
            n(left),
            n(top),
            n(dx),
            n(dy),
            lab.text
        ));
    }
    out.push_str("</div>");
    out
}

fn direction(
    marks: &[Mark],
    clip: Option<(V2, V2)>,
    lab: &Label,
    taken: &[Placed],
    size: plate::Size,
) -> f64 {
    if let Some(ang) = lab.place.angle() {
        return ang;
    }
    let at = lab.at;
    let mut cands: Vec<f64> = (0..24)
        .map(|i| i as f64 * std::f64::consts::PI / 12.0)
        .collect();
    cands.extend(hint_angles(marks, at));
    let mut best = 0.0;
    let mut best_s = f64::INFINITY;
    for ang in cands {
        let p = at.add(V2::new(ang.cos(), ang.sin()).mul(plate::GAP));
        let s = score(marks, clip, lab, p, ang, taken, size);
        if s < best_s {
            best_s = s;
            best = ang;
        }
    }
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
    p: V2,
    _ang: f64,
    taken: &[Placed],
    size: plate::Size,
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
                    s += (near - d) * (near - d) * 6.0;
                }
            }
            Mark::Circle { c, r, .. } => {
                let d = (mid.dist(*c) - *r).abs();
                if d < near {
                    s += (near - d) * 8.0;
                }
            }
            Mark::Arc { a, b, via, .. } => {
                let d = dist_seg(mid, *a, *via).min(dist_seg(mid, *via, *b));
                if d < near {
                    s += (near - d) * (near - d);
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
        let other = t.at.add(V2::new(t.ang.cos(), t.ang.sin()).mul(plate::GAP));
        let (u0, v0, u1, v1) = text_box(&t.text, other);
        if x0 < u1 && x1 > u0 && y0 < v1 && y1 > v0 {
            s += 120.0;
        }
        let d = mid.dist(other);
        if d < plate::FONT {
            s += (plate::FONT - d) * 2.0;
        }
    }
    let pad = plate::FONT * 0.2;
    if x0 < pad || y0 < pad || x1 > size.w - pad || y1 > size.h - pad {
        s += 40.0;
    }
    if let Some((min, max)) = clip {
        if p.x < min.x - pad || p.y < min.y - pad || p.x > max.x + pad || p.y > max.y + pad {
            s += 6.0;
        }
    }
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

    fn ang_of(p: Place) -> f64 {
        p.angle().unwrap()
    }

    #[test]
    fn author_place_is_the_direction() {
        let labels = [
            Label {
                id: "A".into(),
                text: "A".into(),
                at: V2::new(100.0, 100.0),
                place: Place::Left,
                r_em: plate::LETTER_R,
            },
            Label {
                id: "B".into(),
                text: "B".into(),
                at: V2::new(400.0, 100.0),
                place: Place::Right,
                r_em: plate::LETTER_R,
            },
            Label {
                id: "C".into(),
                text: "C".into(),
                at: V2::new(250.0, 300.0),
                place: Place::Above,
                r_em: plate::LETTER_R,
            },
        ];
        let placed = place(&[], None, &labels, plate::Size { w: 1000.0, h: 1000.0 });
        for (lab, p) in labels.iter().zip(placed.iter()) {
            assert!((p.ang - ang_of(lab.place)).abs() < 1e-9, "{} ang", lab.id);
            assert!((p.at.x - lab.at.x).abs() < 1e-9);
        }
        let _ = book1_prop2;
        let _ = Figure::new();
    }

    #[test]
    fn auto_does_not_sit_on_the_stroke() {
        let marks = [Mark::Seg {
            id: "ab".into(),
            a: V2::new(0.0, 100.0),
            b: V2::new(400.0, 100.0),
        }];
        let labels = [Label {
            id: "A".into(),
            text: "A".into(),
            at: V2::new(100.0, 100.0),
            place: Place::Auto,
            r_em: plate::LETTER_R,
        }];
        let placed = place(&marks, None, &labels, plate::Size { w: 1000.0, h: 1000.0 });
        let ang = placed[0].ang;
        let p = V2::new(100.0, 100.0).add(V2::new(ang.cos(), ang.sin()).mul(plate::GAP));
        let d = dist_seg(p, V2::new(0.0, 100.0), V2::new(400.0, 100.0));
        assert!(
            d > plate::FONT * 0.25,
            "letter must leave the stroke, dist {d}"
        );
    }

    #[test]
    fn place_is_the_plate() {
        let at = V2::new(100.0, 100.0);
        let labels = [Label {
            id: "B".into(),
            text: "B".into(),
            at,
            place: Place::BelowLeft,
            r_em: plate::LETTER_R,
        }];
        let placed = place(&[], None, &labels, plate::Size { w: 1000.0, h: 1000.0 });
        assert!((placed[0].ang - ang_of(Place::BelowLeft)).abs() < 1e-9);
        assert!((placed[0].at.x - at.x).abs() < 1e-9 && (placed[0].at.y - at.y).abs() < 1e-9);
    }
}
