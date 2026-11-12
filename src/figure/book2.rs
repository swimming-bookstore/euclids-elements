use super::diagram::Diagram;
use super::geom::{Place, V2};

/// II.1 — Fitzpatrick English plate (Elements PDF p. 50, the right-hand figure).
///
/// Given straight-line A above the rectangle, letter outside the left end
/// (not over the middle of the stroke). BC is cut at D and E. BF is drawn
/// at right-angles to BC and produced a little past G; BG is made equal to
/// A. GH is parallel to BC. DK, EL, and CH are parallel to BG.
///
/// One scale BC = 100. The rectangle is right-angled: B, D, E, C share a row,
/// G, K, L, H the row below, and each upright is a column. BG/BC = 1/2, the
/// plate height. A is half of BC. F is the produced end of the upright, one
/// letter below G.
///
/// BH, BK, DL, and EH are named by opposite corners. The plate draws no
/// diagonal of those rectangles, so the spoken name is the boundary.
pub fn book2_prop1() -> Diagram {
    let mut d = Diagram::new();
    // Given line A, above the rectangle. The capital is the left end,
    // slightly above the stroke — the same place as the other given lines.
    d.pin("A0", V2::new(27.70, 17.01));
    d.pin("A1", V2::new(77.86, 17.01));
    d.named_line("A", "A0", "A1");
    d.put_line_end("A", V2::new(27.70, 17.01), true);
    d.put("B", V2::new(0.0, 0.0), Place::Deg(82.1));
    d.level("D", "B", 44.32, Place::Deg(75.2));
    d.level("E", "B", 72.25, Place::Deg(94.1));
    d.level("C", "B", 100.0, Place::Deg(76.1));
    d.plumb("G", "B", -49.93, Place::Deg(-175.3));
    d.corner("K", "D", "G", Place::Deg(-76.2));
    d.corner("L", "E", "G", Place::Deg(-97.6));
    d.corner("H", "C", "G", Place::Deg(-69.3));
    d.plumb("F", "B", -54.93, Place::Deg(-82.1));
    // BC and GH split at the cuts, so BD, DE, and EC are the pieces.
    d.chain(&["B", "D", "E", "C"]);
    d.chain(&["G", "K", "L", "H"]);
    // Uprights. BG is produced to F; DK, EL, and CH stop on GH.
    d.chain(&["F", "G", "B"]);
    d.join("D", "K");
    d.join("E", "L");
    d.join("C", "H");
    // Spoken by opposite corners. The plate draws no stroke across them.
    d.region("BH", &["B", "C", "H", "G"]);
    d.region("BK", &["B", "D", "K", "G"]);
    d.region("DL", &["D", "E", "L", "K"]);
    d.region("EH", &["E", "C", "H", "L"]);
    d.dots(&["B", "C", "D", "E", "F", "G", "H", "K", "L"]);
    d
}

/// II.2 — Fitzpatrick English plate (Elements PDF p. 51, the right-hand figure).
///
/// Square ADEB described on AB. AB is the top side, cut at random at C.
/// CF is drawn through C, parallel to AD and BE, meeting DE at F.
/// AE, AF, and CE are named by opposite corners. The plate draws no
/// diagonal, so those names are the boundary, not a walk across the cut.
///
/// One scale AB = 100. The plate square is 125.88 pt on a side, so AD = AB.
/// C is 0.2736 of the way from A to B — the ink column, not a round fraction.
/// Letters sit outside the square, on the measured tip→glyph heading.
pub fn book2_prop2() -> Diagram {
    let mut d = Diagram::new();
    d.put("A", V2::new(0.0, 0.0), Place::Deg(92.7));
    d.level("C", "A", 27.36, Place::Deg(78.5));
    d.level("B", "A", 100.0, Place::Deg(94.1));
    d.plumb("D", "A", -100.0, Place::Deg(-90.5));
    d.corner("F", "C", "D", Place::Deg(-80.8));
    d.corner("E", "B", "D", Place::Deg(-96.2));
    // AB and DE split at the cut, so AC, CB, DF, and FE are the pieces.
    d.chain(&["A", "C", "B"]);
    d.chain(&["D", "F", "E"]);
    d.join("A", "D");
    d.join("C", "F");
    d.join("B", "E");
    // Spoken by opposite corners. The plate draws no stroke across them.
    d.region("AE", &["A", "B", "E", "D"]);
    d.region("AF", &["A", "C", "F", "D"]);
    d.region("CE", &["C", "B", "E", "F"]);
    d.dots(&["A", "B", "C", "D", "E", "F"]);
    d
}
