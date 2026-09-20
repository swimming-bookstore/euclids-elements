//! Word-by-word karaoke: compile markup, play a cursor, paint a two-line lyric window.
//!
//! Markup:
//! - `*AB*` italic geometry word
//! - `{[Post. 3]}` right-margin citation
//!
//! Plug in a `PartsMap` so a sung word can light matching figure ids.

mod player;
mod script;
mod ui;

pub use player::Player;
pub use script::{compile, PartsMap, Script, Timing};
pub use ui::{KaraokeLyrics, KaraokePlay, KaraokeRead};

/// Euclid-style labels: a letter, a segment, a triangle, or a named circle.
pub struct EuclidParts;

impl PartsMap for EuclidParts {
    fn parts(&self, word: &str) -> Vec<&'static str> {
        let key: String = word
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        match key.as_str() {
            "A" => vec!["A"],
            "B" => vec!["B"],
            "C" => vec!["C"],
            "D" => vec!["D"],
            "E" => vec!["E"],
            "F" => vec!["F"],
            "G" => vec!["G"],
            "H" => vec!["H"],
            "AB" | "BA" => vec!["ab", "A", "B"],
            "AC" | "CA" => vec!["ac", "A", "C"],
            "BC" | "CB" => vec!["bc", "B", "C"],
            "ABC" | "BAC" | "CAB" => vec!["ab", "ac", "bc", "A", "B", "C"],
            "BCD" | "CDB" | "DBC" => vec!["circ-a", "B", "C", "D"],
            "ACE" | "CAE" | "EAC" => vec!["circ-b", "A", "C", "E"],
            _ => Vec::new(),
        }
    }
}


