//! Word-by-word karaoke: compile markup, play a cursor, paint a two-line lyric window.
//!
//! Markup:
//! - `*AB*` italic geometry word
//! - `{[Post. 3]}` right-margin citation
//!
//! A `Diagram` is the `PartsMap`: sung letters light the construction graph.

mod layout;
mod player;
mod script;
mod ui;

pub use player::Player;
pub use script::{compile, PartsMap, Timing};
pub use ui::{KaraokeLyrics, KaraokePlay, KaraokeRead};

use crate::figure::Diagram;

impl PartsMap for Diagram {
    fn parts(&self, word: &str) -> Vec<String> {
        self.highlight(word)
    }
}
