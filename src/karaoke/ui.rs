use super::layout::{read_layout, Atom};
use super::player::Player;
use super::script::Script;
use leptos::prelude::*;

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
/// a comma-cite breaks so the next clause starts on a new line.
#[component]
pub fn KaraokeRead(script: Script) -> impl IntoView {
    view! {
        <div class="proof">
            {read_layout(&script).into_iter().map(|atoms| {
                view! {
                    <p class="para">
                        {atoms.into_iter().map(|atom| match atom {
                            Atom::Word { text, italic } => view! {
                                <span class="word" class:em=italic>{text}" "</span>
                            }.into_any(),
                            Atom::Cite(text) => view! {
                                <>
                                    <span class="sidenote">
                                        <span class="cite">{text}</span>
                                    </span>
                                    " "
                                </>
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
#[component]
pub fn KaraokeLyrics(script: Script, player: Player) -> impl IntoView {
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
                        <span class="body">
                            {body.into_iter().map(|(ti, tok)| {
                                let italic = tok.italic;
                                let text = format!("{} ", tok.text);
                                let sing_script = word_script.clone();
                                let sung_script = word_script.clone();
                                view! {
                                    <span
                                        class="word"
                                        class:em=italic
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
                                        {text}
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
