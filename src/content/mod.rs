pub mod book1;

pub const BOOKS: [u8; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// One spoken sentence. `*AB*` italic, `{[Post. 3]}` right-margin citation.
/// `para` groups sentences into Fitzpatrick’s paragraphs for reading.
#[derive(Clone, Copy, Debug)]
pub struct Phrase {
    pub para: u8,
    pub text: &'static str,
}

pub const fn s(para: u8, text: &'static str) -> Phrase {
    Phrase { para, text }
}

#[derive(Clone, Copy, Debug)]
pub struct Proposition {
    pub book: u8,
    pub number: u8,
    pub enunciation: &'static str,
    pub phrases: &'static [Phrase],
    pub figure: fn() -> crate::figure::Diagram,
}

pub fn propositions_in(book: u8) -> &'static [Proposition] {
    match book {
        1 => book1::PROPOSITIONS,
        _ => &[],
    }
}

pub fn get(book: u8, number: u8) -> Option<&'static Proposition> {
    propositions_in(book).iter().find(|p| p.number == number)
}
