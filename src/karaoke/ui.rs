use super::layout::{read_layout, roll_shift, Atom};
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
            request_animation_frame(move || {
                apply_roll(&script, player, playing, cursor, now_shift, prev_shift);
                mark_broken_words();
            });
        });
    });

    view! {
        <div class="karaoke" aria-live="polite">
            {script.lines.iter().enumerate().map(|(li, line)| {
                let prev_script = script.clone();
                let now_script = script.clone();
                let gone_script = script.clone();
                let word_script = script.clone();
                let style_script = script.clone();
                let chunks = lyric_chunks(&line.tokens);
                let n_chunks = chunks.len();
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
                            {chunks.into_iter().enumerate().map(|(ci, (ti, tok, cites))| {
                                let italic = tok.italic;
                                let gap = super::layout::demo_spaced(&line.tokens, ti);
                                let text = tok.text;
                                let broken = hyphen_at(&text);
                                let cite_break = !cites.is_empty() && ci + 1 < n_chunks;
                                let sing_script = word_script.clone();
                                let sung_script = word_script.clone();
                                view! {
                                    <span class="chunk">
                                        <span
                                            class="word"
                                            class:em=italic
                                            class:break=broken.is_some()
                                            class:cite-break=cite_break
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
                                            {cites.into_iter().map(|cite| {
                                                view! { <span class="cite">{cite}</span> }
                                            }).collect_view()}
                                        </span>
                                        {cite_break.then(|| view! { <br class="after-cite"/> })}
                                    </span>
                                }
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
/// word; `None` means the line is finished, so the wrap with the last
/// citation (else the last wrap) stays.
fn shift_px(script: &Script, line: usize, active: Option<usize>, window: usize) -> i32 {
    let Some((html, body)) = line_body(line) else {
        return 0;
    };
    let restore_display = html.client_width() == 0;
    if restore_display {
        let _ = html.style().set_property("display", "block");
    }
    let _ = html.style().set_property("max-height", "none");
    let _ = html.style().set_property("overflow", "visible");
    let saved_transform = body.style().get_property_value("transform").unwrap_or_default();
    let _ = body.style().set_property("transform", "none");
    let _ = body.offset_height();

    let y = if let Some(ti) = active {
        let (rows, line_h) = wrap_from_dom(&body);
        let body_i = script
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
            .unwrap_or(0);
        let shift = roll_shift(&rows, body_i, window);
        if shift == 0 || line_h <= 0.0 {
            0
        } else {
            -((shift as f32) * line_h).round() as i32
        }
    } else {
        -cited_wrap_top(&body)
    };

    if saved_transform.is_empty() {
        let _ = body.style().remove_property("transform");
    } else {
        let _ = body.style().set_property("transform", &saved_transform);
    }
    let _ = html.style().remove_property("max-height");
    let _ = html.style().remove_property("overflow");
    if restore_display {
        let _ = html.style().remove_property("display");
    }
    y
}

/// Pixel offset of the last cited wrap (else the last wrap), so `.prev`
/// keeps `[Prop. 1.1]` instead of rolling to “joined.” Viewport rects,
/// because `offsetTop` is relative to `.chunk` and is always 0.
fn cited_wrap_top(body: &web_sys::HtmlElement) -> i32 {
    let origin = body.get_bounding_client_rect().top();
    let words = body.get_elements_by_class_name("word");
    let mut last = 0.0;
    let mut cited = None;
    for i in 0..words.length() {
        let Some(n) = words.item(i).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok()) else {
            continue;
        };
        let top = n.get_bounding_client_rect().top() - origin;
        last = top;
        if n.class_list().contains("cite-break") || n.get_elements_by_class_name("cite").length() > 0
        {
            cited = Some(top);
        }
    }
    cited.unwrap_or(last).round() as i32
}

fn line_body(line: usize) -> Option<(web_sys::HtmlElement, web_sys::HtmlElement)> {
    let doc = web_sys::window()?.document()?;
    let lines = doc.get_elements_by_class_name("line");
    let line_el = lines.item(line as u32)?.dyn_into::<web_sys::Element>().ok()?;
    let html = line_el.dyn_into::<web_sys::HtmlElement>().ok()?;
    let body = html.get_elements_by_class_name("body").item(0)?;
    let body = body.dyn_into::<web_sys::HtmlElement>().ok()?;
    Some((html, body))
}

/// Wrap rows from laid-out word tops. A `.cite-break` starts a new row
/// even if `offsetTop` missed the `<br>` (`display: contents`).
fn wrap_from_dom(body: &web_sys::HtmlElement) -> (Vec<usize>, f32) {
    let origin = body.get_bounding_client_rect().top();
    let words = body.get_elements_by_class_name("word");
    let mut rows = Vec::new();
    let mut row = 0usize;
    let mut last_top: Option<i32> = None;
    let mut after_cite = false;
    let mut step = 0i32;
    for i in 0..words.length() {
        let Some(n) = words.item(i).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok()) else {
            continue;
        };
        let top = (n.get_bounding_client_rect().top() - origin).round() as i32;
        let visual = last_top.map(|p| top - p > 2).unwrap_or(false);
        if visual {
            let d = top - last_top.unwrap();
            if d > step {
                step = d;
            }
        }
        if visual || after_cite {
            row += 1;
        }
        after_cite = false;
        rows.push(row);
        last_top = Some(top);
        if n.class_list().contains("cite-break") {
            after_cite = true;
        }
    }
    let line_h = if step > 1 {
        step as f32
    } else {
        words
            .item(0)
            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
            .map(|n| n.offset_height() as f32)
            .filter(|h| *h > 1.0)
            .unwrap_or(0.0)
    };
    (rows, line_h)
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

/// Body words with the cites that follow them, so a margin cite rides
/// the same wrap as `CD.` instead of dropping onto the next lyric.
fn lyric_chunks(
    tokens: &[super::script::Token],
) -> Vec<(usize, super::script::Token, Vec<String>)> {
    let mut out: Vec<(usize, super::script::Token, Vec<String>)> = Vec::new();
    for (ti, tok) in tokens.iter().cloned().enumerate() {
        if tok.cite {
            if let Some((_, _, cites)) = out.last_mut() {
                cites.push(tok.text);
            }
            continue;
        }
        out.push((ti, tok, Vec::new()));
    }
    out
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
