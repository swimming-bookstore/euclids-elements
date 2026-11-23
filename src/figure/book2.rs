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

/// II.7 — Fitzpatrick English plate (Elements PDF p. 56, the right-hand figure).
///
/// Square ADEB described on AB. AB is the top side, cut at random at C. BD is
/// joined. The upright through C, parallel to AD and BE, meets BD at G and DE
/// at N. HK is the row of that crossing: H on AD, F on BE. CF is the square
/// on BC, named by the opposite corners C and F. DG is the square on AC,
/// named the other way as GD. AG and GE are the complements about the
/// diagonal. AF and CE are the rectangles the proof adds. Each is named by
/// opposite corners. The plate draws no stroke across them, so the spoken
/// name is the boundary, not a diagonal. KLM is the gnomon: the square on AB
/// with the square on AC taken out. K, L, and M are that gnomon's plate
/// letters, not vertices. N is the foot of the upright, not a gnomon letter.
///
/// One scale AB = 100. C is 0.6154 of the way from A to B — the ink column,
/// not a round fraction. G is BD × CN, so HK is that row and CF is the square
/// on CB. Vertex letters sit on the measured tip→glyph heading. K, L, and M
/// sit at the measured glyph centers.
///
/// The dotted stroke about G is three quadrants. PDF p. 56 starts at the left
/// and runs top, right, bottom, then stops. The gap is the lower-left, toward
/// the square on AC, and M sits in that gap, so the stroke does not join M.
/// Radius 25.45 pt on the 132.32 pt square, so 19.23 on this scale — 0.192 of
/// the side, not the 0.150 ring from II.5 or the 0.222 ring from II.6. Dash
/// and gap are 3.05 pt, 0.120 of that radius.
pub fn book2_prop7() -> Diagram {
    let mut d = Diagram::new();
    let ac = 61.54;
    d.put("A", V2::new(0.0, 100.0), Place::Deg(94.6));
    d.level("C", "A", ac, Place::Deg(94.4));
    d.level("B", "A", 100.0, Place::Deg(76.5));
    d.plumb("H", "A", ac, Place::Deg(-177.0));
    d.plumb("D", "A", 0.0, Place::Deg(-73.9));
    d.corner("G", "C", "H", Place::Deg(-47.8));
    d.corner("F", "B", "H", Place::Deg(-3.6));
    d.corner("N", "C", "D", Place::Deg(-90.1));
    d.corner("E", "B", "D", Place::Deg(-79.1));
    // AB, HK, and DE split at the cuts. The uprights split where HK crosses
    // them. BD splits at G.
    d.chain(&["A", "C", "B"]);
    d.chain(&["H", "G", "F"]);
    d.chain(&["D", "N", "E"]);
    d.chain(&["A", "H", "D"]);
    d.chain(&["B", "F", "E"]);
    d.chain(&["C", "G", "N"]);
    // The plate joins BD. The proof never calls that diagonal by its ends:
    // BG and DG are the squares. A join would light the diagonal for those
    // names.
    d.ink("B", "G");
    d.ink("G", "D");
    // PDF p. 56: left, top, right, bottom, then stop. The missing quadrant is
    // the lower-left, toward the square on AC. M sits in that opening.
    // Radius 19.23, 0.192 of AB. Dash 0.120 of the radius.
    d.dotted_ring_dash("G", 19.23, -135.0, 0.120);
    // Spoken by opposite corners. The plate draws no stroke across them.
    // BG is the square on BC, the same boundary as CF, named the other way.
    // GD is the square on AC, the same boundary as DG.
    d.region("AG", &["A", "C", "G", "H"]);
    d.region("GE", &["G", "F", "E", "N"]);
    d.region("CF", &["C", "B", "F", "G"]);
    d.region("AF", &["A", "B", "F", "H"]);
    d.region("CE", &["C", "B", "E", "N"]);
    d.region("BG", &["B", "F", "G", "C"]);
    d.region("GD", &["G", "N", "D", "H"]);
    d.region("DG", &["D", "N", "G", "H"]);
    // Square on AB, minus the square on AC. The outline is the L.
    d.region("KLM", &["A", "B", "E", "N", "G", "H"]);
    d.region("ADEB", &["A", "B", "E", "D"]);
    // Gnomon letters. The center is the plate glyph, not an offset from a
    // vertex — a heading would walk L into the ring and K out of the square.
    // PDF p. 56, glyph center minus the D tip, on the side of 132.32.
    d.pin("K", V2::new(42.98, 57.42));
    d.glyph("K", V2::new(42.98, 57.42));
    d.pin("L", V2::new(64.90, 85.16));
    d.glyph("L", V2::new(64.90, 85.16));
    d.pin("M", V2::new(56.39, 42.84));
    d.glyph("M", V2::new(56.39, 42.84));
    d.dots(&["A", "B", "C", "D", "E", "F", "G", "H", "N"]);
    d
}

/// II.8 — Fitzpatrick English plate (Elements PDF p. 58, the right-hand figure).
///
/// AB is cut at random at C, and BD is produced equal to CB. The square AEFD
/// is described on AD. The figure is drawn double: a two-by-two block of the
/// square on CB sits at the upper right, and the square equal to the square
/// on AC sits at the lower left, named OH by the opposite corners. ED is the
/// diagonal. It passes through Q and K. The proof never calls that stroke by
/// its ends. Spoken squares are registered boundaries, so the join is not CK
/// or KD.
///
/// CK, KD, GR, and RN are the four squares on CB. AG, MQ, QL, and RF are the
/// four rectangles. AK is the rectangle contained by AB and BD. CP and ML are
/// the parallelograms whose complements the proof names. STU is the gnomon:
/// the square on AD with OH taken out. S, T, and U are that gnomon's plate
/// letters, not vertices.
///
/// One scale AD = 100. The plate columns are 5 : 2 : 2 — AC is 5/9 of AD, and
/// CB and BD are each 2/9. The rows repeat that, so the lower band is AC and
/// OH is a square on a side equal to AC. Vertex letters sit on the measured
/// tip→glyph heading. S, T, and U sit at the measured glyph centers.
///
/// The dotted stroke is not the three-quadrant ring of II.5–II.7. PDF p. 58
/// centers it on the diagonal, inside square GR, not on a vertex. Radius
/// 39.67 pt on the 157.55 pt square, so 25.18 on this scale — 0.252 of the
/// side. The gap is 36.4°, centered lower-left, toward OH. Dash and gap are
/// 2.63 pt, 0.066 of that radius. Rc is the center, not a plate letter.
pub fn book2_prop8() -> Diagram {
    let mut d = Diagram::new();
    let ad = 100.0;
    let cb = ad * 2.0 / 9.0;
    let ac = ad * 5.0 / 9.0;
    let ab = ac + cb;
    let upper = ad - cb;
    let mid = ad - 2.0 * cb;
    d.put("A", V2::new(0.0, ad), Place::Deg(95.5));
    d.level("C", "A", ac, Place::Deg(95.9));
    d.level("B", "A", ab, Place::Deg(97.7));
    d.level("D", "A", ad, Place::Deg(69.0));
    d.plumb("E", "A", 0.0, Place::Deg(-96.5));
    d.corner("F", "D", "E", Place::Deg(-82.1));
    d.plumb("M", "A", upper, Place::Deg(175.8));
    d.corner("G", "C", "M", Place::Deg(46.2));
    d.corner("K", "B", "M", Place::Deg(-54.8));
    d.corner("N", "D", "M", Place::Deg(5.2));
    d.plumb("O", "A", mid, Place::Deg(179.6));
    d.corner("Q", "C", "O", Place::Deg(-50.7));
    d.corner("R", "B", "O", Place::Deg(-53.6));
    d.corner("P", "D", "O", Place::Deg(9.4));
    d.corner("H", "C", "E", Place::Deg(-89.3));
    d.corner("L", "B", "E", Place::Deg(-98.9));
    // AD, the two rows, and EF split at the cuts. The uprights split where
    // the rows cross them.
    d.chain(&["A", "C", "B", "D"]);
    d.chain(&["M", "G", "K", "N"]);
    d.chain(&["O", "Q", "R", "P"]);
    d.chain(&["E", "H", "L", "F"]);
    d.chain(&["A", "M", "O", "E"]);
    d.chain(&["C", "G", "Q", "H"]);
    d.chain(&["B", "K", "R", "L"]);
    d.chain(&["D", "N", "P", "F"]);
    // The plate joins ED. Spoken squares keep the registered boundary, so
    // this stroke is not CK or KD.
    d.join("E", "D");
    // PDF p. 58. Centered on the diagonal, inside GR. The gap is 36.4°, not
    // a quadrant, and it faces OH. Radius 25.18. Dash 0.066 of the radius.
    d.pin("Rc", V2::new(66.92, 66.92));
    d.dotted_ring_span("Rc", 25.18, -135.0, 36.4, 0.066);
    // Spoken by opposite corners. The plate draws no stroke across them.
    d.region("CK", &["C", "B", "K", "G"]);
    d.region("KD", &["K", "B", "D", "N"]);
    d.region("GR", &["G", "K", "R", "Q"]);
    d.region("RN", &["R", "K", "N", "P"]);
    d.region("AG", &["A", "C", "G", "M"]);
    d.region("MQ", &["M", "G", "Q", "O"]);
    d.region("QL", &["Q", "R", "L", "H"]);
    d.region("RF", &["R", "P", "F", "L"]);
    d.region("AK", &["A", "B", "K", "M"]);
    d.region("OH", &["O", "Q", "H", "E"]);
    d.region("CP", &["C", "D", "P", "Q"]);
    d.region("ML", &["M", "K", "L", "E"]);
    // Square on AD, minus OH. The outline is the L.
    d.region("STU", &["A", "D", "F", "H", "Q", "O"]);
    // Gnomon letters. The center is the plate glyph, not an offset from a
    // vertex — a heading would walk S and U out of OH and T out of KD.
    // PDF p. 58, glyph center minus the E tip, on the side of 157.55.
    d.pin("S", V2::new(45.34, 52.15));
    d.glyph("S", V2::new(45.34, 52.15));
    d.pin("T", V2::new(82.39, 90.19));
    d.glyph("T", V2::new(82.39, 90.19));
    d.pin("U", V2::new(52.13, 44.63));
    d.glyph("U", V2::new(52.13, 44.63));
    d.dots(&[
        "A", "B", "C", "D", "E", "F", "G", "H", "K", "L", "M", "N", "O", "P", "Q", "R",
    ]);
    d
}

/// II.9 — Fitzpatrick English plate (Elements PDF p. 59, the right-hand figure).
///
/// AB is cut equally at C and unequally at D. CE is drawn from C at
/// right-angles to AB and made equal to each of AC and CB, so ACE and BCE
/// are isosceles right triangles. EA and EB are joined. DF is drawn through
/// D, parallel to EC, and meets EB at F. FG is drawn through F, parallel to
/// AB, and meets CE at G. AF is joined.
///
/// One scale AB = 100. C is the midpoint. CE equals AC, so E sits 50 above
/// C. D is 0.722 of the way from A to B — the ink, not a round fraction.
/// F is EB × the upright through D, and G is CE × the row of F. Vertex
/// letters sit on the measured tip→glyph heading.
///
/// The proof names the angles, the sides, and the squares on those sides.
/// The plate draws no square, so a square name is not a boundary to light.
pub fn book2_prop9() -> Diagram {
    let mut d = Diagram::new();
    let ab = 100.0;
    let ac = ab / 2.0;
    // PDF p. 59. AD is 0.7222 of AB, not a round cut.
    let ad = 72.22;
    d.put("A", V2::new(0.0, 0.0), Place::Deg(-94.2));
    d.level("C", "A", ac, Place::Deg(-94.5));
    d.level("D", "A", ad, Place::Deg(-73.5));
    d.level("B", "A", ab, Place::Deg(-77.7));
    d.plumb("E", "C", ac, Place::Deg(98.6));
    // F is EB × the upright through D. meet refuses a vertical line, so the
    // crossing is the column of D on EB. G is that row on CE.
    let d_x = ad;
    let t = (d_x - ac) / (ab - ac);
    d.pin("F", V2::new(d_x, ac + t * (0.0 - ac)));
    d.label("F", Place::Deg(13.8));
    d.corner("G", "C", "F", Place::Deg(164.9));
    d.chain(&["A", "C", "D", "B"]);
    d.chain(&["C", "G", "E"]);
    d.chain(&["D", "F"]);
    d.chain(&["G", "F"]);
    d.join("E", "A");
    // F is EB × DF, so the join splits there. EF and FB are the pieces the
    // angles name. An unsplit stroke would walk those angles around GF.
    d.chain(&["E", "F", "B"]);
    d.join("A", "F");
    d.dots(&["A", "B", "C", "D", "E", "F", "G"]);
    d
}

/// II.10 — Fitzpatrick English plate (Elements PDF p. 61).
///
/// AB is cut in half at C, and BD is added straight-on. CE is drawn from C
/// at right-angles to AB and made equal to each of AC and CB, so ACE and
/// BCE are isosceles right triangles. EA and EB are joined. EF is drawn
/// through E, parallel to AD, and FD through D, parallel to CE. EB and FD
/// are produced and meet at G. AG is joined.
///
/// One scale AB = 100. C is the midpoint. CE equals AC, so E sits 50 above
/// C. BD is 0.287 of AB — the ink, not a round fraction. F is the row of E
/// on the upright through D. G is EB produced to that upright. Vertex
/// letters sit on the measured tip→glyph heading. G's letter is the
/// measured glyph, not that heading: the heading lands in the angle.
///
/// The proof names the angles, the sides, and the squares on those sides.
/// The plate draws no square, so a square name is not a boundary to light.
pub fn book2_prop10() -> Diagram {
    let mut d = Diagram::new();
    let ab = 100.0;
    let ac = ab / 2.0;
    // PDF p. 61. BD is 44.2 pt on an AB of 154.2, so 0.2866 of AB.
    let bd = ab * 44.2 / 154.2;
    d.put("A", V2::new(0.0, 0.0), Place::Deg(-118.4));
    d.level("C", "A", ac, Place::Deg(-94.6));
    d.level("B", "A", ab, Place::Deg(-115.3));
    d.level("D", "A", ab + bd, Place::Deg(-5.3));
    d.plumb("E", "C", ac, Place::Deg(97.5));
    d.corner("F", "D", "E", Place::Deg(50.8));
    // G is EB produced to the upright through D. EB is the 45° from E, so
    // the drop past B equals BD. The plate letter sits just outside the
    // join, right and below — the measured glyph, not a heading from the
    // join, which would land in the angle.
    d.pin("G", V2::new(ab + bd, -bd));
    // PDF p. 61. The glyph center is 7.0 pt right and 3.4 pt below the join,
    // on an AB of 154.2 pt. A heading from the join lands in the angle.
    let gx = ab * 7.0 / 154.2;
    let gy = ab * 3.4 / 154.2;
    d.glyph("G", V2::new(ab + bd + gx, -bd - gy));
    d.chain(&["A", "C", "B", "D"]);
    d.chain(&["C", "E"]);
    d.chain(&["E", "F"]);
    d.chain(&["F", "D", "G"]);
    // B is EB × AB, and G is the produced end, so the join splits there.
    // EB and BG are the pieces the angles name.
    d.chain(&["E", "B", "G"]);
    d.join("E", "A");
    d.join("A", "G");
    d.dots(&["A", "B", "C", "D", "E", "F", "G"]);
    d
}

/// II.11 — Fitzpatrick English plate (Elements PDF p. 63, the right-hand figure).
///
/// AB is the given straight-line. Square ABDC is described on AB, so the
/// square stands below the line. AC is cut in half at E. BE is joined. CA
/// is drawn through to F, and EF is made equal to BE. Square FH is described
/// on AF, and GH is drawn through to K.
///
/// One scale AB = 100. The plate square is 110.4 pt on a side, so AC equals
/// AB. E is the midpoint of AC, and it lies on the left side. F and G are
/// the top of the square on AF. AF/AB is 0.5833 — the ink, not a round
/// fraction. H is AF to the right of A, on AB. K is that column on CD.
///
/// Vertex letters sit on the measured tip→glyph heading. H's letter is the
/// measured glyph: the heading from the join lands on the stroke.
///
/// FH, FK, AD, AK, and HD are named by opposite corners. The plate draws no
/// diagonal of those rectangles, so the spoken name is the boundary. FK is
/// the rectangle contained by CF and FA: corners F, G, K, C. A, E, and H
/// lie on those sides; they are not corners of the name.
pub fn book2_prop11() -> Diagram {
    let mut d = Diagram::new();
    let ab = 100.0;
    // PDF p. 63. The square is 110.4 pt on a side. AF is 64.4 pt, so 0.5833
    // of AB. The golden cut is near 0.618; the plate is the shorter ink.
    let af = ab * 64.4 / 110.4;
    d.put("A", V2::new(0.0, 0.0), Place::Deg(176.9));
    // H is the join. Its capital sits up and to the right, off both strokes.
    // A heading from the join lands on AB, so the letter is the measured glyph.
    d.pin("H", V2::new(af, 0.0));
    d.level("B", "A", ab, Place::Deg(18.1));
    d.plumb("F", "A", af, Place::Deg(99.5));
    d.corner("G", "H", "F", Place::Deg(92.2));
    d.plumb("E", "A", -ab / 2.0, Place::Deg(-178.9));
    d.plumb("C", "A", -ab, Place::Deg(-93.8));
    d.corner("K", "H", "C", Place::Deg(-96.2));
    d.corner("D", "B", "C", Place::Deg(-72.5));
    // PDF p. 63. The glyph center is 6.95 pt right and 6.65 pt above the
    // join, on a square of 110.4 pt.
    let hx = ab * 6.95 / 110.4;
    let hy = ab * 6.65 / 110.4;
    d.glyph("H", V2::new(af + hx, hy));
    d.chain(&["A", "H", "B"]);
    d.chain(&["C", "K", "D"]);
    d.chain(&["F", "G"]);
    d.chain(&["A", "E", "C"]);
    d.chain(&["F", "A"]);
    d.chain(&["G", "H", "K"]);
    d.chain(&["B", "D"]);
    // E is AC × BE, so the join splits there. AE and EB are the pieces the
    // proof names.
    d.chain(&["B", "E"]);
    d.region("FH", &["F", "A", "H", "G"]);
    // FK is the rectangle contained by CF and FA. The corners are F, G, K, C
    // — not A, which only lies on CF. Order around the ink, not the diagonal.
    d.region("FK", &["F", "G", "K", "C"]);
    d.region("AD", &["A", "B", "D", "C"]);
    d.region("AK", &["A", "H", "K", "C"]);
    d.region("HD", &["H", "B", "D", "K"]);
    d.dots(&["A", "B", "C", "D", "E", "F", "G", "H", "K"]);
    d
}
