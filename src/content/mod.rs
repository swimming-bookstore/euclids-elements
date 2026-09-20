pub mod book1;

#[derive(Clone, Copy, Debug)]
pub struct Book {
    pub number: u8,
    pub title: &'static str,
}

pub const BOOKS: [Book; 13] = [
    Book { number: 1, title: "Fundamentals of plane geometry" },
    Book { number: 2, title: "Geometric algebra" },
    Book { number: 3, title: "Circles" },
    Book { number: 4, title: "Inscription and circumscription" },
    Book { number: 5, title: "Proportion" },
    Book { number: 6, title: "Similar figures" },
    Book { number: 7, title: "Elementary number theory" },
    Book { number: 8, title: "Continued proportion" },
    Book { number: 9, title: "Applications of number theory" },
    Book { number: 10, title: "Incommensurables" },
    Book { number: 11, title: "Spatial geometry" },
    Book { number: 12, title: "Method of exhaustion" },
    Book { number: 13, title: "Regular solids" },
];

/// One proof line. `*AB*` italic, `{[Post. 3]}` right-margin citation.
#[derive(Clone, Copy, Debug)]
pub struct Phrase {
    pub text: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct Proposition {
    #[allow(dead_code)]
    pub book: u8,
    pub number: u8,
    pub enunciation: &'static str,
    pub phrases: &'static [Phrase],
    pub svg: fn(on: &[&str]) -> String,
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
