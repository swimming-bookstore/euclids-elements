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
