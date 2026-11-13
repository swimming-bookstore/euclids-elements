use super::{s, Proposition};
use crate::figure::{book2_prop1, book2_prop2, book2_prop3};

pub const PROP_1: Proposition = Proposition {
    book: 2,
    number: 1,
    enunciation: "If there are two straight-lines, and one of them is cut into any number of pieces whatsoever, then the rectangle contained by the two straight-lines is equal to the (sum of the) rectangles contained by the uncut (straight-line), and every one of the pieces (of the cut straight-line).",
    figure: book2_prop1,
    phrases: &[
        s(1, "Let *A* and *BC* be the two straight-lines, and let *BC* be cut, at random, at points *D* and *E*."),
        s(1, "I say that the rectangle contained by *A* and *BC* is equal to the rectangle(s) contained by *A* and *BD*, by *A* and *DE*, and, finally, by *A* and *EC*."),
        s(2, "For let *BF* have been drawn from point *B*, at right-angles to *BC*,{[Prop. 1.11]} and let *BG* be made equal to *A*,{[Prop. 1.3]} and let *GH* have been drawn through (point) *G*, parallel to *BC*,{[Prop. 1.31]} and let *DK*, *EL*, and *CH* have been drawn through (points) *D*, *E*, and *C* (respectively), parallel to *BG*.{[Prop. 1.31]}"),
        s(3, "So the (rectangle) *rectangle(BH)* is equal to the (rectangles) *rectangle(BK)*, *rectangle(DL)*, and *rectangle(EH)*."),
        s(3, "And *rectangle(BH)* is the (rectangle contained) by *A* and *BC*."),
        s(3, "For it is contained by *GB* and *BC*, and *BG* (is) equal to *A*."),
        s(3, "And *rectangle(BK)* (is) the (rectangle contained) by *A* and *BD*."),
        s(3, "For it is contained by *GB* and *BD*, and *BG* (is) equal to *A*."),
        s(3, "And *rectangle(DL)* (is) the (rectangle contained) by *A* and *DE*."),
        s(3, "For *DK*, that is to say *BG*,{[Prop. 1.34]} (is) equal to *A*."),
        s(3, "Similarly, *rectangle(EH)* (is) also the (rectangle contained) by *A* and *EC*."),
        s(3, "Thus, the (rectangle contained) by *A* and *BC* is equal to the (rectangles contained) by *A* and *BD*, by *A* and *DE*, and, finally, by *A* and *EC*."),
        s(4, "Thus, if there are two straight-lines, and one of them is cut into any number of pieces whatsoever, then the rectangle contained by the two straight-lines is equal to the (sum of the) rectangles contained by the uncut (straight-line), and every one of the pieces (of the cut straight-line)."),
        s(4, "(Which is) the very thing it was required to show."),
    ],
};

pub const PROP_2: Proposition = Proposition {
    book: 2,
    number: 2,
    enunciation: "If a straight-line is cut at random then the (sum of the) rectangle(s) contained by the whole (straight-line), and each of the pieces (of the straight-line), is equal to the square on the whole.",
    figure: book2_prop2,
    phrases: &[
        s(1, "For let the straight-line *AB* have been cut, at random, at point *C*."),
        s(1, "I say that the rectangle contained by *AB* and *BC*, plus the rectangle contained by *BA* and *AC*, is equal to the square on *AB*."),
        s(2, "For let the square *ADEB* have been described on *AB*,{[Prop. 1.46]} and let *CF* have been drawn through *C*, parallel to either of *AD* or *BE*.{[Prop. 1.31]}"),
        s(3, "So the (square) *rectangle(AE)* is equal to the (rectangles) *rectangle(AF)* and *rectangle(CE)*."),
        s(3, "And *rectangle(AE)* is the square on *AB*."),
        s(3, "And *rectangle(AF)* (is) the rectangle contained by the (straight-lines) *BA* and *AC*."),
        s(3, "For it is contained by *DA* and *AC*, and *AD* (is) equal to *AB*."),
        s(3, "And *rectangle(CE)* (is) the (rectangle contained) by *AB* and *BC*."),
        s(3, "For *BE* (is) equal to *AB*."),
        s(3, "Thus, the (rectangle contained) by *BA* and *AC*, plus the (rectangle contained) by *AB* and *BC*, is equal to the square on *AB*."),
        s(4, "Thus, if a straight-line is cut at random then the (sum of the) rectangle(s) contained by the whole (straight-line), and each of the pieces (of the straight-line), is equal to the square on the whole."),
        s(4, "(Which is) the very thing it was required to show."),
    ],
};


pub const PROP_3: Proposition = Proposition {
    book: 2,
    number: 3,
    enunciation: "If a straight-line is cut at random then the rectangle contained by the whole (straight-line), and one of the pieces (of the straight-line), is equal to the rectangle contained by (both of) the pieces, and the square on the aforementioned piece.",
    figure: book2_prop3,
    phrases: &[
        s(1, "For let the straight-line *AB* have been cut, at random, at (point) *C*."),
        s(1, "I say that the rectangle contained by *AB* and *BC* is equal to the rectangle contained by *AC* and *CB*, plus the square on *BC*."),
        s(2, "For let the square *CDEB* have been described on *CB*,{[Prop. 1.46]} and let *ED* have been drawn through to *F*, and let *AF* have been drawn through *A*, parallel to either of *CD* or *BE*.{[Prop. 1.31]}"),
        s(3, "So the (rectangle) *rectangle(AE)* is equal to the (rectangle) *rectangle(AD)* and the (square) *rectangle(CE)*."),
        s(3, "And *rectangle(AE)* is the rectangle contained by *AB* and *BC*."),
        s(3, "For it is contained by *AB* and *BE*, and *BE* (is) equal to *BC*."),
        s(3, "And *rectangle(AD)* (is) the (rectangle contained) by *AC* and *CB*."),
        s(3, "For *DC* (is) equal to *CB*."),
        s(3, "And *DB* (is) the square on *CB*."),
        s(3, "Thus, the rectangle contained by *AB* and *BC* is equal to the rectangle contained by *AC* and *CB*, plus the square on *BC*."),
        s(4, "Thus, if a straight-line is cut at random then the rectangle contained by the whole (straight-line), and one of the pieces (of the straight-line), is equal to the rectangle contained by (both of) the pieces, and the square on the aforementioned piece."),
        s(4, "(Which is) the very thing it was required to show."),
    ],
};

pub const PROPOSITIONS: &[Proposition] = &[PROP_1, PROP_2, PROP_3];
