use super::{Phrase, Proposition};
use crate::figure::book1_prop1;

pub const PROP_1: Proposition = Proposition {
    book: 1,
    number: 1,
    enunciation: "To construct an equilateral triangle on a given finite straight-line.",
    svg: book1_prop1,
    phrases: &[
        Phrase { text: "Let *AB* be the given finite straight-line." },
        Phrase { text: "So it is required to construct an equilateral triangle on the straight-line *AB*." },
        Phrase { text: "Let the circle *BCD* with center *A* and radius *AB* have been drawn.{[Post. 3]}" },
        Phrase { text: "and again let the circle *ACE* with center *B* and radius *BA* have been drawn.{[Post. 3]}" },
        Phrase { text: "And let the straight-lines *CA* and *CB* have been joined from the point *C*,{[Post. 1]}" },
        Phrase { text: "where the circles cut one another, to the points *A* and *B*." },
        Phrase { text: "And since the point *A* is the center of the circle *CDB*, *AC* is equal to *AB*.{[Def. 1.15]}" },
        Phrase { text: "Again, since the point *B* is the center of the circle *CAE*, *BC* is equal to *BA*.{[Def. 1.15]}" },
        Phrase { text: "But *CA* was also shown (to be) equal to *AB*." },
        Phrase { text: "Thus, *CA* and *CB* are each equal to *AB*." },
        Phrase { text: "But things equal to the same thing are also equal to one another.{[C.N. 1]}" },
        Phrase { text: "Thus, *CA* is also equal to *CB*." },
        Phrase { text: "Thus, the three (straight-lines) *CA*, *AB*, and *BC* are equal to one another." },
        Phrase { text: "Thus, the triangle *ABC* is equilateral, and has been constructed on the given finite straight-line *AB*." },
        Phrase { text: "(Which is) the very thing it was required to do." },
    ],
};

pub const PROPOSITIONS: &[Proposition] = &[PROP_1];
