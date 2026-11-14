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

/// II.3 — Fitzpatrick English plate (Elements PDF p. 52, the right-hand figure).
///
/// AB is the top side, cut at random at C. The square CDEB is described on
/// CB. ED is drawn through to F, and AF is drawn through A, parallel to CD
/// and BE, so AF meets the production at F.
///
/// AE, AD, and CE are named by opposite corners. The plate draws no
/// diagonal, so those names are the boundary, not a walk across the cut.
/// CDEB is the square, named by its four corners. DB is that same square,
/// named by the opposite corners B and D.
///
/// One scale AB = 100. The plate square is the side CB, and CB is two
/// thirds of AB — the ink columns, not a round guess. C is one third of
/// the way from A to B. Letters sit outside the rectangle, on the measured
/// tip→glyph heading.
pub fn book2_prop3() -> Diagram {
    let mut d = Diagram::new();
    d.put("A", V2::new(0.0, 0.0), Place::Deg(93.6));
    d.level("C", "A", 100.0 / 3.0, Place::Deg(73.9));
    d.level("B", "A", 100.0, Place::Deg(95.9));
    d.plumb("F", "A", -200.0 / 3.0, Place::Deg(-98.3));
    d.corner("D", "C", "F", Place::Deg(-72.9));
    d.corner("E", "B", "F", Place::Deg(-96.6));
    // AB and FE split at the cuts, so AC, CB, FD, and DE are the pieces.
    d.chain(&["A", "C", "B"]);
    d.chain(&["F", "D", "E"]);
    d.join("A", "F");
    d.join("C", "D");
    d.join("B", "E");
    // Spoken by opposite corners. The plate draws no stroke across them.
    // DB is the square on CB, the same boundary as CE, named the other way.
    d.region("AE", &["A", "B", "E", "F"]);
    d.region("AD", &["A", "C", "D", "F"]);
    d.region("CE", &["C", "B", "E", "D"]);
    d.region("DB", &["D", "E", "B", "C"]);
    d.dots(&["A", "B", "C", "D", "E", "F"]);
    d
}

/// II.4 — Fitzpatrick English plate (Elements PDF p. 53, the right-hand figure).
///
/// Square ADEB described on AB. AB is the top side, cut at random at C.
/// BD is joined. CF is drawn through C, parallel to AD and BE, meeting DE
/// at F and BD at G. HK is drawn through G, parallel to AB and DE, meeting
/// AD at H and BE at K.
///
/// AG, GE, HF, and CK are named by opposite corners. The plate draws no
/// stroke across those rectangles, so the spoken name is the boundary.
/// ADEB, CGKB, and HF are the squares, named by their corners.
///
/// One scale AB = 100. The plate square is the side AB. C is 0.7709 of
/// the way from A to B — the ink column, not a round fraction. G is the
/// crossing of BD and CF, so HK is the row of that crossing and CGKB is
/// the square on CB. The plate letter G sits below-right of that crossing,
/// inside the square on CB. The other letters sit outside the square, on
/// the measured tip→glyph heading.
pub fn book2_prop4() -> Diagram {
    let mut d = Diagram::new();
    let ac = 77.09;
    d.put("A", V2::new(0.0, 0.0), Place::Deg(95.5));
    d.level("C", "A", ac, Place::Deg(86.3));
    d.level("B", "A", 100.0, Place::Deg(75.9));
    d.plumb("D", "A", -100.0, Place::Deg(-91.3));
    d.corner("F", "C", "D", Place::Deg(-90.3));
    d.corner("E", "B", "D", Place::Deg(-82.6));
    // G is BD × CF. On the English plate the capital sits below-right of
    // the crossing, in the square on CB, clear of both strokes — not in
    // the open angle above them. Heading −45° is tip→glyph (PDF p. 53).
    d.meet_pin("G", "B", "D", "C", "F");
    d.label("G", Place::Deg(-45.0));
    d.corner("H", "A", "G", Place::Deg(-178.9));
    d.corner("K", "B", "G", Place::Deg(-2.9));
    // AB, HK, and DE split at the cuts, so AC, CB, HG, GK, DF, and FE
    // are the pieces. AD and BE split at H and K.
    d.chain(&["A", "C", "B"]);
    d.chain(&["H", "G", "K"]);
    d.chain(&["D", "F", "E"]);
    d.chain(&["A", "H", "D"]);
    d.chain(&["B", "K", "E"]);
    d.chain(&["C", "G", "F"]);
    d.join("B", "G");
    d.join("G", "D");
    // Spoken by opposite corners. The plate draws no stroke across them.
    // HF is the square on HG, the same boundary as DH, named the other way.
    d.region("AG", &["A", "C", "G", "H"]);
    d.region("GE", &["G", "K", "E", "F"]);
    d.region("HF", &["H", "G", "F", "D"]);
    d.region("CK", &["C", "B", "K", "G"]);
    d.region("CGKB", &["C", "B", "K", "G"]);
    d.dots(&["A", "B", "C", "D", "E", "F", "G", "H", "K"]);
    d
}
