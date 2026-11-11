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
