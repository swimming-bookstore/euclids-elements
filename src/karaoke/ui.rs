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

/// Full proof for reading: every line, citations on the right, no highlight.
#[component]
pub fn KaraokeRead(script: Script) -> impl IntoView {
    view! {
        <div class="proof">
            {script.lines.iter().map(|line| {
                let tokens = line.tokens.clone();
                view! {
                    <p class="pline">
                        {tokens.into_iter().map(|tok| {
                            let cite = tok.cite;
                            let italic = tok.italic;
                            let text = tok.text;
                            view! {
                                <span class="word" class:cite=cite class:em=italic>
                                    {text}
                                </span>
                            }
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
                let tokens = line.tokens.clone();
                let prev_script = script.clone();
                let now_script = script.clone();
                let gone_script = script.clone();
                let word_script = script.clone();
                view! {
                    <p
                        class="line"
                        class:prev=move || {
                            let cur = prev_script.get(player.cursor.get()).map(|(i, _, _)| i).unwrap_or(0);
                            li + 1 == cur
                        }
                        class:now=move || {
                            let cur = now_script.get(player.cursor.get()).map(|(i, _, _)| i).unwrap_or(0);
                            li == cur
                        }
                        class:gone=move || {
                            let cur = gone_script.get(player.cursor.get()).map(|(i, _, _)| i).unwrap_or(0);
                            li + 1 < cur
                        }
                    >
                        {tokens.into_iter().enumerate().map(|(ti, tok)| {
                            let cite = tok.cite;
                            let italic = tok.italic;
                            let text = tok.text;
                            let sing_script = word_script.clone();
                            let sung_script = word_script.clone();
                            view! {
                                <span
                                    class="word"
                                    class:cite=cite
                                    class:em=italic
                                    class:sing=move || {
                                        if cite {
                                            return false;
                                        }
                                        sing_script.get(player.cursor.get())
                                            .map(|(a, b, _)| a == li && b == ti)
                                            .unwrap_or(false)
                                    }
                                    class:sung=move || {
                                        if cite {
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
                    </p>
                }
            }).collect_view()}
        </div>
    }
}
