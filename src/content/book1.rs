use super::{s, Proposition};
use crate::figure::{book1_prop1, book1_prop2, book1_prop3};

pub const PROP_1: Proposition = Proposition {
    book: 1,
    number: 1,
    enunciation: "To construct an equilateral triangle on a given finite straight-line.",
    figure: book1_prop1,
    phrases: &[
        s(1, "Let *AB* be the given finite straight-line."),
        s(2, "So it is required to construct an equilateral triangle on the straight-line *AB*."),
        s(3, "Let the circle *BCD* with center *A* and radius *AB* have been drawn,{[Post. 3]}"),
        s(3, "and again let the circle *ACE* with center *B* and radius *BA* have been drawn.{[Post. 3]}"),
        s(3, "And let the straight-lines *CA* and *CB* have been joined from the point *C*, where the circles cut one another, to the points *A* and *B* (respectively).{[Post. 1]}"),
        s(4, "And since the point *A* is the center of the circle *CDB*, *AC* is equal to *AB*.{[Def. 1.15]}"),
        s(4, "Again, since the point *B* is the center of the circle *CAE*, *BC* is equal to *BA*.{[Def. 1.15]}"),
        s(4, "But *CA* was also shown (to be) equal to *AB*."),
        s(4, "Thus, *CA* and *CB* are each equal to *AB*."),
        s(4, "But things equal to the same thing are also equal to one another.{[C.N. 1]}"),
        s(4, "Thus, *CA* is also equal to *CB*."),
        s(4, "Thus, the three (straight-lines) *CA*, *AB*, and *BC* are equal to one another."),
        s(5, "Thus, the triangle *ABC* is equilateral, and has been constructed on the given finite straight-line *AB*."),
        s(5, "(Which is) the very thing it was required to do."),
    ],
};

pub const PROP_2: Proposition = Proposition {
    book: 1,
    number: 2,
    enunciation: "To place a straight-line equal to a given straight-line at a given point (as an extremity).",
    figure: book1_prop2,
    phrases: &[
        s(1, "Let *A* be the given point, and *BC* the given straight-line."),
        s(1, "So it is required to place a straight-line at point *A* equal to the given straight-line *BC*."),
        s(2, "For let the straight-line *AB* have been joined from point *A* to point *B*,{[Post. 1]}"),
        s(2, "and let the equilateral triangle *DAB* have been constructed upon it.{[Prop. 1.1]}"),
        s(2, "And let the straight-lines *AE* and *BF* have been produced in a straight-line with *DA* and *DB* (respectively).{[Post. 2]}"),
        s(2, "And let the circle *CGH* with center *B* and radius *BC* have been drawn,{[Post. 3]}"),
        s(2, "and again let the circle *GKL* with center *D* and radius *DG* have been drawn.{[Post. 3]}"),
        s(3, "Therefore, since the point *B* is the center of (the circle) *CGH*, *BC* is equal to *BG*.{[Def. 1.15]}"),
        s(3, "Again, since the point *D* is the center of the circle *GKL*, *DL* is equal to *DG*.{[Def. 1.15]}"),
        s(3, "And within these, *DA* is equal to *DB*."),
        s(3, "Thus, the remainder *AL* is equal to the remainder *BG*.{[C.N. 3]}"),
        s(3, "But *BC* was also shown (to be) equal to *BG*."),
        s(3, "Thus, *AL* and *BC* are each equal to *BG*."),
        s(3, "But things equal to the same thing are also equal to one another.{[C.N. 1]}"),
        s(3, "Thus, *AL* is also equal to *BC*."),
        s(4, "Thus, the straight-line *AL*, equal to the given straight-line *BC*, has been placed at the given point *A*."),
        s(4, "(Which is) the very thing it was required to do."),
    ],
};

pub const PROP_3: Proposition = Proposition {
    book: 1,
    number: 3,
    enunciation: "For two given unequal straight-lines, to cut off from the greater a straight-line equal to the lesser.",
    figure: book1_prop3,
    phrases: &[
        s(1, "Let *AB* and *C* be the two given unequal straight-lines, of which let the greater be *AB*."),
        s(1, "So it is required to cut off a straight-line equal to the lesser *C* from the greater *AB*."),
        s(2, "Let the line *AD*, equal to the straight-line *C*, have been placed at point *A*.{[Prop. 1.2]}"),
        s(2, "And let the circle *DEF* have been drawn with center *A* and radius *AD*.{[Post. 3]}"),
        s(3, "And since point *A* is the center of circle *DEF*, *AE* is equal to *AD*.{[Def. 1.15]}"),
        s(3, "But, *C* is also equal to *AD*."),
        s(3, "Thus, *AE* and *C* are each equal to *AD*."),
        s(3, "So *AE* is also equal to *C*.{[C.N. 1]}"),
        s(4, "Thus, for two given unequal straight-lines, *AB* and *C*, the (straight-line) *AE*, equal to the lesser *C*, has been cut off from the greater *AB*."),
        s(4, "(Which is) the very thing it was required to do."),
    ],
};

pub const PROPOSITIONS: &[Proposition] = &[PROP_1, PROP_2, PROP_3];
