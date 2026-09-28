use super::layout::{last_row, read_layout, roll_shift, wrap_rows, Atom};
use super::player::Player;
use super::script::Script;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn KaraokePlay(player: Player, record: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="karaoke-play">
            <button
                type="button"
                class="play"
                on:click=move |_| {
                    if record.get() {
                        if player.playing.get() {
                            player.playing.set(false);
                        } else {
                            player.restart();
                            player.playing.set(true);
                        }
                    } else {
                        record.set(true);
                        player.restart();
                        player.playing.set(true);
                    }
                }
            >
                {move || {
                    if !record.get() {
                        "Play"
                    } else if player.playing.get() {
                        "Pause"
                    } else {
                        "Play"
                    }
                }}
            </button>
            {move || {
                record.get().then(|| view! {
                    <button
                        type="button"
                        class="mode"
                        on:click=move |_| {
                            player.playing.set(false);
                            record.set(false);
                        }
                    >
                        "Read"
                    </button>
                })
            }}
        </div>
    }
}

/// Full proof: Fitzpatrick paragraphs. Cites hang in the margin;
/// a cited clause breaks so the next sentence starts on a new line.
#[component]
pub fn KaraokeRead(script: Script) -> impl IntoView {
    view! {
        <div class="proof">
            {read_layout(&script).into_iter().map(|atoms| {
                view! {
                    <p class="para">
                        {atoms.into_iter().map(|atom| match atom {
                            Atom::Word { text, italic, spaced } => view! {
                                <span class="word" class:em=italic>{text}{space(spaced)}</span>
                            }.into_any(),
                            Atom::Cite(text) => view! {
                                <span class="sidenote">
                                    <span class="cite">{text}</span>
                                </span>
                            }.into_any(),
                            Atom::Break => view! { <br class="after-cite"/> }.into_any(),
                        }).collect_view()}
                    </p>
                }
            }).collect_view()}
        </div>
    }
}

/// Current line wraps in full. Previous line sits above, then leaves.
///
/// The record window stays two lines high. A wrapped sentence rolls up
/// so the active wrap is the second line and the wrap above it remains.
/// A finished sentence keeps only its last wrap in the previous slot.
#[component]
pub fn KaraokeLyrics(script: Script, player: Player) -> impl IntoView {
    let now_shift = RwSignal::new(0i32);
    let prev_shift = RwSignal::new(0i32);
    let script_roll = script.clone();
    Effect::new(move |_| {
        let playing = player.playing.get();
        let cursor = player.cursor.get();
        let script = script_roll.clone();
        request_animation_frame(move || {
            apply_roll(&script, player, playing, cursor, now_shift, prev_shift);
            mark_broken_words();
        });
    });

    view! {
        <div class="karaoke" aria-live="polite">
            {script.lines.iter().enumerate().map(|(li, line)| {
                let mut body = Vec::new();
                let mut cites = Vec::new();
                for (ti, tok) in line.tokens.iter().cloned().enumerate() {
                    if tok.cite {
                        cites.push(tok);
                    } else {
                        body.push((ti, tok));
                    }
                }
                let prev_script = script.clone();
                let now_script = script.clone();
                let gone_script = script.clone();
                let word_script = script.clone();
                let style_script = script.clone();
                view! {
                    <p
                        class="line"
                        class:prev=move || {
                            if !player.playing.get() && player.cursor.get() == player.start_at {
                                return false;
                            }
                            let cur = prev_script.get(player.cursor.get()).map(|(i, _, _)| i).unwrap_or(0);
                            li + 1 == cur
                        }
                        class:now=move || {
                            if !player.playing.get() && player.cursor.get() == player.start_at {
                                return false;
                            }
                            let cur = now_script.get(player.cursor.get()).map(|(i, _, _)| i).unwrap_or(0);
                            li == cur
                        }
                        class:gone=move || {
                            if !player.playing.get() && player.cursor.get() == player.start_at {
                                return false;
                            }
                            let cur = gone_script.get(player.cursor.get()).map(|(i, _, _)| i).unwrap_or(0);
                            li + 1 < cur
                        }
                    >
                        <span
                            class="body"
                            style=move || {
                                let cur = style_script
                                    .get(player.cursor.get())
                                    .map(|(i, _, _)| i)
                                    .unwrap_or(0);
                                let y = if cur == li {
                                    now_shift.get()
                                } else if cur == li + 1 {
                                    prev_shift.get()
                                } else {
                                    0
                                };
                                if y == 0 {
                                    String::new()
                                } else {
                                    format!("transform: translateY({y}px)")
                                }
                            }
                        >
                            {body.into_iter().map(|(ti, tok)| {
                                let italic = tok.italic;
                                let gap = super::layout::demo_spaced(&line.tokens, ti);
                                let text = tok.text;
                                let broken = hyphen_at(&text);
                                let sing_script = word_script.clone();
                                let sung_script = word_script.clone();
                                view! {
                                    <span
                                        class="word"
                                        class:em=italic
                                        class:break=broken.is_some()
                                        class:sing=move || {
                                            if !player.playing.get() && player.cursor.get() == player.start_at {
                                                return false;
                                            }
                                            sing_script.get(player.cursor.get())
                                                .map(|(a, b, _)| a == li && b == ti)
                                                .unwrap_or(false)
                                        }
                                        class:sung=move || {
                                            if !player.playing.get() && player.cursor.get() == player.start_at {
                                                return false;
                                            }
                                            match sung_script.get(player.cursor.get()) {
                                                Some((a, b, _)) if a > li || (a == li && b > ti) => true,
                                                _ => false,
                                            }
                                        }
                                    >
                                        {match broken {
                                            Some(at) => view! {
                                                <span class="pre">{text[..at].to_string()}</span>
                                                <span class="hy">"-"</span>
                                                <span class="post">{text[at..].to_string()}</span>
                                                {space(gap)}
                                            }.into_any(),
                                            None => view! { <span class="pre">{text}{space(gap)}</span> }.into_any(),
                                        }}
                                    </span>
                                }
                            }).collect_view()}
                        </span>
                        <span class="cites">
                            {cites.into_iter().map(|tok| {
                                view! { <span class="word cite">{tok.text}</span> }
                            }).collect_view()}
                        </span>
                    </p>
                }
            }).collect_view()}
        </div>
    }
}

/// Shift the current sentence up so its active wrap sits on the second
/// line of the two-line window. Measured after layout, from real word boxes.
fn apply_roll(
    script: &Script,
    player: Player,
    playing: bool,
    cursor: usize,
    now_shift: RwSignal<i32>,
    prev_shift: RwSignal<i32>,
) {
    let idle = !playing && cursor == player.start_at;
    if idle {
        now_shift.set(0);
        prev_shift.set(0);
        return;
    }
    let Some((cur, ti, _)) = script.get(cursor) else {
        now_shift.set(0);
        prev_shift.set(0);
        return;
    };
    let now = shift_px(script, cur, Some(ti), 2);
    let prev = if cur > 0 {
        shift_px(script, cur - 1, None, 1)
    } else {
        0
    };
    if now_shift.get_untracked() != now {
        now_shift.set(now);
    }
    if prev_shift.get_untracked() != prev {
        prev_shift.set(prev);
    }
}

/// Negative pixels to translate a line's body. `active` is the sung body
/// word; `None` means the line is finished, so only its last wrap stays.
fn shift_px(script: &Script, line: usize, active: Option<usize>, window: usize) -> i32 {
    let Some(body) = line_body(line) else {
        return 0;
    };
    let width = body.client_width() as f32;
    if width <= 0.0 {
        return 0;
    }
    let nodes = body.children();
    let mut widths = Vec::new();
    for i in 0..nodes.length() {
        let w = nodes
            .item(i)
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
            .map(|n| {
                let box_w = n.get_bounding_client_rect().width() as f32;
                let margin = n
                    .owner_document()
                    .and_then(|d| d.default_view())
                    .and_then(|w| w.get_computed_style(&n).ok().flatten())
                    .and_then(|s| s.get_property_value("margin-right").ok())
                    .and_then(|v| v.trim_end_matches("px").parse::<f32>().ok())
                    .unwrap_or(0.0);
                box_w + margin
            })
            .unwrap_or(0.0);
        widths.push(w);
    }
    let rows = wrap_rows(&widths, width);
    let body_i = match active {
        Some(ti) => script
            .lines
            .get(line)
            .map(|line| {
                line.tokens
                    .iter()
                    .take(ti + 1)
                    .filter(|t| !t.cite)
                    .count()
                    .saturating_sub(1)
            })
            .unwrap_or(0),
        None => rows.len().saturating_sub(1),
    };
    let shift = if active.is_some() {
        roll_shift(&rows, body_i, window)
    } else {
        last_row(&rows).saturating_sub(window - 1)
    };
    let line_h = line_height_px(&body);
    if shift == 0 || line_h <= 0.0 {
        0
    } else {
        -((shift as f32) * line_h).round() as i32
    }
}

fn line_body(line: usize) -> Option<web_sys::HtmlElement> {
    let doc = web_sys::window()?.document()?;
    let lines = doc.get_elements_by_class_name("line");
    let line_el = lines.item(line as u32)?.dyn_into::<web_sys::Element>().ok()?;
    // A finished line is display:none until it becomes .prev, so its wrap
    // width is 0 unless we measure it laid out.
    let html = line_el.dyn_ref::<web_sys::HtmlElement>()?;
    let hidden = html.client_width() == 0;
    if hidden {
        let _ = html.style().set_property("display", "grid");
    }
    let body = line_el.get_elements_by_class_name("body").item(0)?;
    let body = body.dyn_into::<web_sys::HtmlElement>().ok()?;
    if hidden {
        let _ = html.style().remove_property("display");
    }
    Some(body)
}

fn line_height_px(body: &web_sys::HtmlElement) -> f32 {
    body.owner_document()
        .and_then(|d| d.default_view())
        .and_then(|w| w.get_computed_style(body).ok().flatten())
        .and_then(|s| s.get_property_value("line-height").ok())
        .and_then(|v| v.trim_end_matches("px").parse().ok())
        .unwrap_or(0.0)
}

fn space(spaced: bool) -> &'static str {
    if spaced {
        " "
    } else {
        ""
    }
}

/// Where a long word may break, so a wrap can show a hyphen. `None` if the
/// word is short, a name, or already hyphenated.
fn hyphen_at(text: &str) -> Option<usize> {
    let bare: String = text
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();
    if bare.len() < 10 || bare.chars().any(|c| c.is_ascii_uppercase()) {
        return None;
    }
    if text.contains('-') {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut alpha = 0usize;
    let mut at = None;
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_alphabetic() {
            alpha += 1;
            if alpha == bare.len() / 2 {
                at = Some(i + 1);
            }
        }
    }
    at.filter(|&i| i > 0 && i < chars.len())
}

/// A `.break` word whose pieces landed on different rows gets `.broken`,
/// which paints the hyphen at the end of the first row.
fn mark_broken_words() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let words = doc.get_elements_by_class_name("word break");
    for i in 0..words.length() {
        let Some(el) = words.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) else {
            continue;
        };
        let pre = el.get_elements_by_class_name("pre").item(0);
        let post = el.get_elements_by_class_name("post").item(0);
        let broken = match (pre, post) {
            (Some(a), Some(b)) => {
                let ay = a.get_bounding_client_rect().top();
                let by = b.get_bounding_client_rect().top();
                (by - ay).abs() > 2.0
            }
            _ => false,
        };
        if broken {
            let _ = el.class_list().add_1("broken");
        } else {
            let _ = el.class_list().remove_1("broken");
        }
    }
}

fn request_animation_frame(f: impl FnOnce() + 'static) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let cb = wasm_bindgen::closure::Closure::once_into_js(f);
    let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
}
