use super::diagram::Diagram;
use super::geom::{Place, V2};

/// I.1 — equilateral triangle on AB. C above the base; D, E on the sides.
pub fn book1_prop1() -> Diagram {
    let ab = 240.0;
    let mut d = Diagram::new();
    d.put("A", V2::new(0.0, 0.0), Place::Left);
    d.put("B", V2::new(ab, 0.0), Place::Right);
    d.put(
        "C",
        V2::new(ab / 2.0, ab * 3.0_f64.sqrt() / 2.0),
        Place::Above,
    );
    d.put("D", V2::new(-ab, 0.0), Place::Left);
    d.put("E", V2::new(2.0 * ab, 0.0), Place::Right);
    d.circle("BCD", "A", "B")
        .circle("ACE", "B", "A")
        .join("A", "B")
        .join("A", "C")
        .join("B", "C")
        .dots(&["A", "B", "C"]);
    d
}

/// I.2 — copy BC to A as AL. Equilateral DAB; DA, DB produced; circles B, D.
pub fn book1_prop2() -> Diagram {
    let ab = 120.0;
    let bc = 300.0;
    let extra = 88.0;
    let mut d = Diagram::new();
    d.put("D", V2::new(0.0, 0.0), Place::Left);
    d.polar("A", "D", ab, -82.0, Place::Left);
    d.polar("B", "D", ab, -22.0, Place::Right);
    d.ray("L", "D", "A", ab + bc, Place::Left);
    d.ray("G", "D", "B", ab + bc, Place::Right);
    d.ray("E", "D", "A", ab + bc + extra, Place::Below);
    d.ray("F", "D", "B", ab + bc + extra, Place::Right);
    let b = d.at("B");
    d.put("C", V2::new(b.x, b.y + bc), Place::Above);
    d.circle("CGH", "B", "C");
    d.circle("GKL", "D", "G");
    let rb = b.dist(d.at("C"));
    let rd = d.at("D").dist(d.at("G"));
    d.polar("H", "B", rb, 158.0, Place::Left);
    d.polar("K", "D", rd, 172.0, Place::Left);
    d.chain(&["D", "A", "L", "E"])
        .chain(&["D", "B", "G", "F"])
        .join("A", "B")
        .join("B", "C")
        .dots(&["A", "B", "C", "D", "G", "L"]);
    let e = d.at("E");
    let f = d.at("F");
    let c = d.at("C");
    d.clip(
        V2::new(-rd - 24.0, e.y - 56.0),
        V2::new(f.x + 56.0, c.y.max(rd) + 48.0),
    );
    d
}

/// I.3 — Fitzpatrick plate: AB the greater; C a short given line
/// drawn horizontally above the left of the circle; AD = C at A
/// (angle DAE ≈ 140°); circle DEF.
pub fn book1_prop3() -> Diagram {
    let r = 180.0;
    let ab = r * 2.35;
    let mut d = Diagram::new();
    d.put("A", V2::new(0.0, 0.0), Place::Left);
    d.put("B", V2::new(ab, 0.0), Place::Right);
    // Fitzpatrick p. 10: E a little right of vertical above the AB cut;
    // D left of the left-hand cut; F on the lower-right arc, letter outside
    // the circumference (not on the stroke).
    d.put("E", V2::new(r, 0.0), Place::Deg(50.0));
    d.polar("D", "A", r, 140.0, Place::Left);
    d.polar("F", "A", r, 300.0, Place::Deg(300.0));
    let c_y = r * 1.28;
    let c_x0 = -r * 0.28;
    d.pin("Cleft", V2::new(c_x0, c_y));
    d.pin("Cright", V2::new(c_x0 + r, c_y));
    d.put("C", V2::new(c_x0 + r / 2.0, c_y), Place::Above);
    d.named_line("C", "Cleft", "Cright");
    d.circle("DEF", "A", "D")
        .join("A", "D")
        .chain(&["A", "E", "B"])
        .dots(&["A", "B", "D", "E"]);
    d.clip(
        V2::new((-r).min(c_x0) - 40.0, d.at("F").y - 52.0),
        V2::new(ab + 52.0, c_y + 48.0),
    );
    d
}

/// I.4 — Fitzpatrick plate (Elements p. 10): two congruent scalene
/// triangles on one baseline. A sits toward C, D toward F; A, D above
/// the peaks; B, C, E, F under the bases; a small gap at CE.
pub fn book1_prop4() -> Diagram {
    let base = 100.0;
    let h = 80.0;
    let apex = 0.68 * base;
    let gap = 21.0;
    let mut d = Diagram::new();
    d.put("A", V2::new(apex, h), Place::Above);
    d.put("B", V2::new(0.0, 0.0), Place::Below);
    d.put("C", V2::new(base, 0.0), Place::Below);
    let shift = base + gap;
    d.put("D", V2::new(apex + shift, h), Place::Above);
    d.put("E", V2::new(shift, 0.0), Place::Below);
    d.put("F", V2::new(base + shift, 0.0), Place::Below);
    d.join("A", "B")
        .join("B", "C")
        .join("C", "A")
        .join("D", "E")
        .join("E", "F")
        .join("F", "D")
        .dots(&["A", "B", "C", "D", "E", "F"]);
    let e = d.at("E");
    let f = d.at("F");
    let mid = e.add(f).mul(0.5);
    d.arc("E", "F", V2::new(mid.x, mid.y - base * 0.08));
    d
}
