use crate::content::{get, propositions_in, Proposition, BOOKS};
use crate::karaoke::{
    compile, EuclidParts, KaraokeLyrics, KaraokePlay, KaraokeRead, Player, Timing,
};
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let book = RwSignal::new(1u8);
    let prop_n = RwSignal::new(1u8);

    view! {
        <div id="app">
            <header>
                <p class="brand">"Euclid’s Elements"</p>
                <h1>
                    {move || format!("Book {} Proposition {}", book.get(), prop_n.get())}
                </h1>
            </header>
            <main>
                {move || {
                    match get(book.get(), prop_n.get()) {
                        Some(p) => view! { <PropositionPage prop=*p /> }.into_any(),
                        None => view! {
                            <p class="empty">"This book is not yet written."</p>
                        }.into_any(),
                    }
                }}
            </main>
            <footer>
                <nav class="books" aria-label="Books">
                    <span class="lab">"Books"</span>
                    {BOOKS.into_iter().map(|n| {
                        view! {
                            <button
                                class:active=move || book.get() == n
                                class:soon=propositions_in(n).is_empty()
                                on:click=move |_| {
                                    book.set(n);
                                    prop_n.set(1);
                                }
                            >
                                {n}
                            </button>
                        }
                    }).collect_view()}
                </nav>
                <nav class="props" aria-label="Propositions">
                    <span class="lab">"Propositions"</span>
                    {move || {
                        let props = propositions_in(book.get());
                        if props.is_empty() {
                            view! { <span class="muted">"none yet"</span> }.into_any()
                        } else {
                            props.iter().map(|p| {
                                let n = p.number;
                                view! {
                                    <button
                                        class:active=move || prop_n.get() == n
                                        on:click=move |_| prop_n.set(n)
                                    >
                                        {format!("Prop. {n}")}
                                    </button>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </nav>
                <p class="copy">"© 수영 책방 Swimming Bookstore"</p>
            </footer>
        </div>
    }
}

#[component]
fn PropositionPage(prop: Proposition) -> impl IntoView {
    let texts: Vec<&str> = prop.phrases.iter().map(|p| p.text).collect();
    let script = compile(&texts, &EuclidParts, Timing::default());
    let record = RwSignal::new(wants_record());
    let player = Player::start(&script, record.get_untracked());
    let script_fig = script.clone();
    let script_read = script.clone();
    let script_lyr = script.clone();

    view! {
        <article
            class="stage"
            class:record=move || record.get()
            data-karaoke=move || if record.get() { "1" } else { "0" }
        >
            <div class="record-head">
                <p class="record-title">"Euclid’s Elements"</p>
                <h2 class="record-sub">
                    {format!("Book {} Proposition {}", prop.book, prop.number)}
                </h2>
            </div>
            <KaraokePlay player=player record=record />
            <div class="figure-wrap" inner_html=move || {
                if record.get() {
                    (prop.svg)(script_fig.parts_at(player.cursor.get()))
                } else {
                    (prop.svg)(&[])
                }
            }></div>
            <p class="enun"><strong>{prop.enunciation}</strong></p>
            {move || {
                if record.get() {
                    view! { <KaraokeLyrics script=script_lyr.clone() player=player /> }.into_any()
                } else {
                    view! { <KaraokeRead script=script_read.clone() /> }.into_any()
                }
            }}
            <p class="copy record-copy">"© 수영 책방 Swimming Bookstore"</p>
        </article>
    }
}

fn wants_record() -> bool {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| s.contains("record") || s.contains("autoplay"))
        .unwrap_or(false)
}
