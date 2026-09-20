//! Play / pause / autoplay over a compiled script.

use super::script::Script;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub struct Player {
    pub cursor: RwSignal<usize>,
    pub playing: RwSignal<bool>,
    n: usize,
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
        let p = Self {
            cursor: RwSignal::new(start),
            playing: RwSignal::new(false),
            n,
        };
        if autoplay {
            later(400, move || p.playing.set(true));
        }
        let durs: Vec<u32> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter().map(|t| t.dur_ms))
            .collect();
        p.tick(durs, skip);
        p
    }

    pub fn restart(self) {
        self.cursor.set(0);
    }

    fn tick(self, durs: Vec<u32>, skip: Vec<bool>) {
        let n = self.n;
        Effect::new(move |_| {
            if !self.playing.get() {
                return;
            }
            let i = self.cursor.get();
            if skip.get(i).copied().unwrap_or(false) {
                if let Some(next) = next_playable(i + 1, &skip, n) {
                    self.cursor.set(next);
                } else {
                    self.playing.set(false);
                }
                return;
            }
            let dur = durs.get(i).copied().unwrap_or(400).max(1);
            let skip_later = skip.clone();
            later(dur, move || {
                if self.playing.get_untracked() {
                    if let Some(next) = next_playable(i + 1, &skip_later, n) {
                        self.cursor.set(next);
                    } else {
                        self.playing.set(false);
                    }
                }
            });
        });
    }
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
