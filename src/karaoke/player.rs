//! Play / pause / autoplay over a compiled script.
//!
//! The cursor is a token index. Play mode also keeps a clock: each spoken
//! token lasts `dur_ms`, and a drag on the bar seeks that clock earlier or
//! later. The figure and the lyric window both follow the cursor.

use super::script::Script;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub struct Player {
    pub cursor: RwSignal<usize>,
    pub playing: RwSignal<bool>,
    /// Elapsed spoken time, in milliseconds. The seek bar writes this.
    pub elapsed_ms: RwSignal<u32>,
    pub duration_ms: u32,
    pub start_at: usize,
    n: usize,
    /// Spoken length of each token, in order. A seek maps the bar onto this.
    durs: StoredValue<Vec<u32>>,
    /// Token that began at `anchor_ms`. A seek retargets both.
    anchor: RwSignal<usize>,
    anchor_ms: RwSignal<f64>,
}

impl Player {
    pub fn start(script: &Script, autoplay: bool) -> Self {
        let skip: Vec<bool> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter().map(|t| t.cite || t.dur_ms == 0))
            .collect();
        let n = skip.len();
        let start = next_playable(0, &skip, n).unwrap_or(0);
        let durs: Vec<u32> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter().map(|t| t.dur_ms))
            .collect();
        let duration_ms = durs.iter().copied().sum();
        let p = Self {
            cursor: RwSignal::new(start),
            playing: RwSignal::new(false),
            elapsed_ms: RwSignal::new(0),
            duration_ms,
            start_at: start,
            n,
            durs: StoredValue::new(durs.clone()),
            anchor: RwSignal::new(start),
            anchor_ms: RwSignal::new(0.0),
        };
        if autoplay {
            // Let the finished plate sit in solid black before karaoke.
            later(2500, move || p.playing.set(true));
        }
        p.tick(durs, skip);
        p
    }

    pub fn restart(self) {
        self.seek_ms(0);
    }

    /// The spoken clock has reached the end, so Play starts over.
    pub fn finished(self) -> bool {
        self.duration_ms > 0 && self.elapsed_ms.get() >= self.duration_ms
    }

    /// Jump the spoken clock. The cursor lands on the token that owns that
    /// time, so the lyric and the figure match the bar. A cite has no
    /// duration, so the cursor steps to the next spoken token. The bar's
    /// time is kept, and the next step waits only the rest of the word.
    pub fn seek_ms(self, ms: u32) {
        let ms = ms.min(self.duration_ms);
        let (i, _) = self.durs.with_value(|durs| token_at_ms(ms, durs));
        let i = self.playable_from(i);
        self.anchor.set(i);
        self.anchor_ms.set(now_ms());
        self.elapsed_ms.set(ms);
        // The end of the clock is not a word to hold.
        if ms >= self.duration_ms && self.duration_ms > 0 {
            self.playing.set(false);
        }
        if self.cursor.get_untracked() != i {
            self.cursor.set(i);
        }
    }

    fn playable_from(self, i: usize) -> usize {
        self.durs.with_value(|durs| {
            (i..durs.len())
                .find(|&j| durs[j] > 0)
                .unwrap_or(i.min(self.n.saturating_sub(1)))
        })
    }

    /// Spoken time of the token under the cursor, for the bar while playing.
    pub fn elapsed(self) -> u32 {
        if !self.playing.get() {
            return self.elapsed_ms.get();
        }
        let at = self.elapsed_ms.get().saturating_add(self.held());
        at.min(self.duration_ms)
    }

    fn held(self) -> u32 {
        (now_ms() - self.anchor_ms.get()).max(0.0) as u32
    }

    fn tick(self, durs: Vec<u32>, skip: Vec<bool>) {
        let n = self.n;
        let starts = starts_of(&durs);
        Effect::new(move |_| {
            let playing = self.playing.get();
            let i = self.cursor.get();
            if !playing {
                // Freeze the clock where the word actually was, not at its start.
                // A seek already stored that time, so only a live play adds the hold.
                let start = starts.get(i).copied().unwrap_or(0);
                if self.anchor.get_untracked() == i
                    && self.elapsed_ms.get_untracked() == start
                    && self.elapsed_ms.get_untracked() < self.duration_ms
                {
                    let held = self.held();
                    if held > 0 {
                        let dur = durs.get(i).copied().unwrap_or(0);
                        self.elapsed_ms.set((start + held).min(start + dur));
                    }
                }
                return;
            }
            // A seek already placed the anchor. A natural step has to.
            if self.anchor.get_untracked() != i {
                self.anchor.set(i);
                self.anchor_ms.set(now_ms());
                self.elapsed_ms.set(starts.get(i).copied().unwrap_or(0));
            }
            if skip.get(i).copied().unwrap_or(false) {
                if let Some(next) = next_playable(i + 1, &skip, n) {
                    self.cursor.set(next);
                } else {
                    self.playing.set(false);
                    self.elapsed_ms.set(self.duration_ms);
                }
                return;
            }
            let dur = durs.get(i).copied().unwrap_or(400).max(1);
            let skip_later = skip.clone();
            let into_word = self.elapsed_ms.get_untracked().saturating_sub(starts.get(i).copied().unwrap_or(0));
            let remaining = dur.saturating_sub(into_word);
            later(remaining.max(1), move || {
                if self.playing.get_untracked() && self.cursor.get_untracked() == i {
                    if let Some(next) = next_playable(i + 1, &skip_later, n) {
                        self.cursor.set(next);
                    } else {
                        self.playing.set(false);
                        self.elapsed_ms.set(self.duration_ms);
                    }
                }
            });
        });
    }
}

/// Cumulative start of each token. `ms` lands on the spoken token whose
/// span contains it. A cite has no duration, so it never owns a time; the
/// next spoken token does. Past the end, the last spoken token.
pub fn token_at_ms(ms: u32, durs: &[u32]) -> (usize, u32) {
    if durs.is_empty() {
        return (0, 0);
    }
    let mut at = 0u32;
    let mut last = 0usize;
    for (i, dur) in durs.iter().copied().enumerate() {
        if dur == 0 {
            continue;
        }
        last = i;
        let next = at.saturating_add(dur);
        if ms < next {
            return (i, at);
        }
        at = next;
    }
    (last, at.saturating_sub(durs[last]))
}

fn starts_of(durs: &[u32]) -> Vec<u32> {
    let mut at = 0u32;
    let mut starts = Vec::with_capacity(durs.len());
    for dur in durs {
        starts.push(at);
        at = at.saturating_add(*dur);
    }
    starts
}

fn next_playable(from: usize, skip: &[bool], n: usize) -> Option<usize> {
    (from..n).find(|&i| !skip.get(i).copied().unwrap_or(false))
}

fn later(ms: u32, f: impl FnOnce() + 'static) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let cb = Closure::once_into_js(f);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        ms as i32,
    );
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::token_at_ms;

    #[test]
    fn seek_lands_inside_the_token_that_owns_the_time() {
        let durs = [700, 1800, 0, 700];
        assert_eq!(token_at_ms(0, &durs), (0, 0));
        assert_eq!(token_at_ms(699, &durs), (0, 0));
        assert_eq!(token_at_ms(700, &durs), (1, 700));
        // The cite at index 2 has no duration, so 2500 is the word after it.
        assert_eq!(token_at_ms(2499, &durs), (1, 700));
        assert_eq!(token_at_ms(2500, &durs), (3, 2500));
        assert_eq!(token_at_ms(9_000, &durs), (3, 2500));
        assert_eq!(token_at_ms(0, &[]), (0, 0));
    }
}
