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

/// II.5 — Fitzpatrick English plate (Elements PDF p. 54, the right-hand figure).
///
/// AB is cut equally at C and unequally at D. The square CEFB is described
/// on CB. BE is joined. DG is drawn through D, parallel to CE and BF, and
/// meets the diagonal at H. KM is drawn through H, parallel to AB and EF.
/// AK is drawn through A, parallel to CL and BM.
///
/// CH and HF are the complements. DM and LG are the squares on DB and CD.
/// CM, DF, AL, and AH are the rectangles the proof adds. Each is named by
/// opposite corners. The plate draws no stroke across them, so the spoken
/// name is the boundary, not a diagonal. NOP is the gnomon: the square on
/// CB with the square LG taken out. N, O, and P are that gnomon's plate
/// letters, not vertices.
///
/// One scale CB = 100. The plate square is the side CB, and C is the
/// midpoint of AB, so AC = CB. D is a fifth of the way from C to B — the
/// ink column, 20.60 pt on a side of 103.20. H is BE × DG, so KM is the
/// row of that crossing and LG is the square on CD. Vertex letters sit on
/// the measured tip→glyph heading. N, O, and P sit at the measured glyph
/// centers.
pub fn book2_prop5() -> Diagram {
    let mut d = Diagram::new();
    d.put("A", V2::new(0.0, 0.0), Place::Deg(94.7));
    d.level("C", "A", 100.0, Place::Deg(95.0));
    d.level("D", "A", 120.0, Place::Deg(72.0));
    d.level("B", "A", 200.0, Place::Deg(76.6));
    d.plumb("K", "A", -80.0, Place::Deg(-95.6));
    d.corner("L", "C", "K", Place::Deg(-129.2));
    d.corner("H", "D", "K", Place::Deg(130.5));
    d.corner("M", "B", "K", Place::Deg(-2.2));
    d.plumb("E", "C", -100.0, Place::Deg(-97.4));
    d.corner("G", "D", "E", Place::Deg(-73.4));
    d.corner("F", "B", "E", Place::Deg(-64.7));
    // AB, KM, and the base of the square split at the cuts. The uprights
    // split where KM crosses them. BE splits at H.
    d.chain(&["A", "C", "D", "B"]);
    d.chain(&["K", "L", "H", "M"]);
    d.chain(&["E", "G", "F"]);
    d.chain(&["A", "K"]);
    d.chain(&["C", "L", "E"]);
    d.chain(&["D", "H", "G"]);
    d.chain(&["B", "M", "F"]);
    d.chain(&["B", "H", "E"]);
    // The plate's dotted stroke about H is three quadrants, not a circle.
    // PDF p. 54, the English figure (not p. 55, which is II.6): moveto the
    // left point, then cubics to the top, the right, and the bottom, and
    // stop. The missing piece is the lower-left, toward the square LG. The
    // ink sits on the upper-right of H, inside CEFB. Radius 15.48 pt on the
    // 103.19 pt square, so 15.0 on this scale — 0.150 of the side, not the
    // 0.222 ring from II.6. Dash and gap are 2.34 pt, 0.151 of that radius.
    d.dotted_ring("H", 15.0, -135.0);
    // Spoken by opposite corners. The plate draws no stroke across them.
    d.region("CH", &["C", "D", "H", "L"]);
    d.region("HF", &["H", "M", "F", "G"]);
    d.region("DM", &["D", "B", "M", "H"]);
    d.region("CM", &["C", "B", "M", "L"]);
    d.region("DF", &["D", "B", "F", "G"]);
    d.region("AL", &["A", "C", "L", "K"]);
    d.region("AH", &["A", "D", "H", "K"]);
    d.region("LG", &["L", "H", "G", "E"]);
    // Square on CB, minus the square on CD. The outline is the L.
    d.region("NOP", &["C", "B", "F", "G", "H", "L"]);
    // Gnomon letters. The center is the plate glyph, not an offset from a
    // vertex — a heading would walk them out of the square they name.
    d.pin("N", V2::new(105.48, -84.25));
    d.glyph("N", V2::new(105.48, -84.25));
    d.pin("O", V2::new(126.58, -61.25));
    d.glyph("O", V2::new(126.58, -61.25));
    d.pin("P", V2::new(114.62, -93.78));
    d.glyph("P", V2::new(114.62, -93.78));
    d.dots(&["A", "B", "C", "D", "E", "F", "G", "H", "K", "L", "M"]);
    d
}

/// II.6 — Fitzpatrick English plate (Elements PDF p. 55, the right-hand figure).
///
/// AB is cut in half at C, and BD is added straight-on. The square CEFD is
/// described on CD. DE is joined. BG is drawn through B, parallel to EC and
/// DF, and meets the diagonal at H. KM is drawn through H, parallel to AB
/// and EF. AK is drawn through A, parallel to CL and DM.
///
/// AL and CH are the rectangles the proof equates. HF is the complement of
/// CH. CM and AM are the rectangles the proof adds. LG is the square on BC.
/// Each is named by opposite corners. The plate draws no stroke across them,
/// so the spoken name is the boundary, not a diagonal. NOP is the gnomon:
/// the square on CD with the square LG taken out. N, O, and P are that
/// gnomon's plate letters, not vertices.
///
/// One scale CD = 100. The plate square is the side CD. C is the midpoint
/// of AB, so AC = CB = 200/3, and the added line BD is the remaining
/// third. H is DE × BG, one third of the way down the diagonal, so KM is
/// that row and LG is the square on CB. Vertex letters sit on the measured
/// tip→glyph heading. N, O, and P sit at the measured glyph centers.
///
/// The dotted stroke about H is three quadrants. PDF p. 55 starts at the
/// left and runs top, right, bottom, then stops. The gap is the lower-left,
/// toward LG. P sits in that gap, on the circle just past the bottom end,
/// so the stroke does not join P. Carrying the arc on through P paints the
/// opening shut. Radius 22.42 pt on the 100.89 pt square, so 22.2 on this
/// scale — 0.222 of the side, not the 0.150 ring from II.5. Dash and gap
/// are 3.36 pt, 0.150 of that radius. The dash that starts at the left
/// lands short of the bottom end, clear of P.
pub fn book2_prop6() -> Diagram {
    let mut d = Diagram::new();
    let cb = 200.0 / 3.0;
    let bd = 100.0 / 3.0;
    let cd = cb + bd;
    let km = bd;
    d.put("C", V2::new(0.0, 0.0), Place::Deg(95.0));
    d.level("B", "C", cb, Place::Deg(96.5));
    d.level("D", "C", cd, Place::Deg(71.9));
    d.level("A", "C", -cb, Place::Deg(94.6));
    d.plumb("E", "C", -cd, Place::Deg(-79.0));
    d.corner("F", "D", "E", Place::Deg(-64.7));
    d.corner("G", "B", "E", Place::Deg(-91.5));
    d.plumb("K", "A", -km, Place::Deg(-95.6));
    d.corner("L", "C", "K", Place::Deg(-129.2));
    // H is DE × BG. On the English plate the capital sits up-left of that
    // crossing, in the rectangle CH, clear of the diagonal and of BG.
    // Heading 129.5° is tip→glyph (PDF p. 55).
    d.meet_pin("H", "D", "E", "B", "G");
    d.label("H", Place::Deg(129.5));
    d.corner("M", "D", "K", Place::Deg(-2.1));
    // AB, KM, and the base of the square split at the cuts. The uprights
    // split where KM crosses them. DE splits at H.
    d.chain(&["A", "C", "B", "D"]);
    d.chain(&["K", "L", "H", "M"]);
    d.chain(&["E", "G", "F"]);
    d.chain(&["A", "K"]);
    d.chain(&["C", "L", "E"]);
    d.chain(&["B", "H", "G"]);
    d.chain(&["D", "M", "F"]);
    d.chain(&["D", "H", "E"]);
    // PDF p. 55: left, top, right, bottom, then stop. The missing quadrant
    // is the lower-left, toward LG. P sits in that opening, so the stroke
    // is not carried through it. Radius 22.2, 0.222 of CD.
    d.dotted_ring("H", 22.2, -135.0);
    // Spoken by opposite corners. The plate draws no stroke across them.
    d.region("AL", &["A", "C", "L", "K"]);
    d.region("CH", &["C", "B", "H", "L"]);
    d.region("HF", &["H", "M", "F", "G"]);
    d.region("CM", &["C", "D", "M", "L"]);
    d.region("AM", &["A", "D", "M", "K"]);
    d.region("LG", &["L", "H", "G", "E"]);
    // Square on CD, minus the square on BC. The outline is the L.
    d.region("NOP", &["C", "D", "F", "G", "H", "L"]);
    // Gnomon letters. The center is the plate glyph, not an offset from a
    // vertex — a heading would walk them out of the square they name.
    // PDF p. 55, glyph center minus the C tip, on the side of 100.89.
    d.pin("N", V2::new(46.09, -38.64));
    d.glyph("N", V2::new(46.09, -38.64));
    d.pin("O", V2::new(76.72, -6.94));
    d.glyph("O", V2::new(76.72, -6.94));
    d.pin("P", V2::new(61.80, -55.31));
    d.glyph("P", V2::new(61.80, -55.31));
    d.dots(&["A", "B", "C", "D", "E", "F", "G", "H", "K", "L", "M"]);
    d
}
