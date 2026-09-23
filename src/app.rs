use crate::content::{get, propositions_in, Proposition, BOOKS};
use crate::karaoke::{compile, KaraokeLyrics, KaraokePlay, KaraokeRead, Player, Timing};
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsValue;

#[component]
pub fn App() -> impl IntoView {
    let (start_book, start_prop) = read_route();
    let book = RwSignal::new(start_book);
    let prop_n = RwSignal::new(start_prop);
    write_route(start_book, start_prop, false);

    Effect::new(move |_| {
        let b = book.get();
        let p = prop_n.get();
        set_title(b, p);
    });

    window_event_listener(ev::hashchange, move |_| {
        let (b, p) = read_route();
        if book.get_untracked() != b {
            book.set(b);
        }
        if prop_n.get_untracked() != p {
            prop_n.set(p);
        }
    });

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
                    <span class="nav-lab">"Books"</span>
                    {BOOKS.into_iter().map(|n| {
                        view! {
                            <button
                                class:active=move || book.get() == n
                                class:soon=propositions_in(n).is_empty()
                                on:click=move |_| {
                                    book.set(n);
                                    prop_n.set(1);
                                    write_route(n, 1, true);
                                }
                            >
                                {n}
                            </button>
                        }
                    }).collect_view()}
                </nav>
                <nav class="props" aria-label="Propositions">
                    <span class="nav-lab">"Propositions"</span>
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
                                        on:click=move |_| {
                                            prop_n.set(n);
                                            write_route(book.get_untracked(), n, true);
                                        }
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
    let script = compile(prop.phrases, &(prop.figure)(), Timing::default());
    let capture = wants_record();
    let record = RwSignal::new(capture);
    let player = Player::start(&script, record.get_untracked());
    let script_fig = script.clone();
    let script_read = script.clone();
    let script_lyr = script.clone();

    view! {
        <article
            class="stage"
            class:record=move || record.get()
            class:capture=capture
            data-karaoke=move || if record.get() { "1" } else { "0" }
            data-ready="1"
        >
            <div class="record-head">
                <p class="record-title">"Euclid’s Elements"</p>
                <h2 class="record-sub">
                    {format!("Book {} Proposition {}", prop.book, prop.number)}
                </h2>
            </div>
            <KaraokePlay player=player record=record />
            <div class="figure-wrap" inner_html=move || {
                let fig = (prop.figure)();
                let cursor = player.cursor.get();
                let lighting = record.get()
                    && (player.playing.get() || cursor != player.start_at);
                if lighting {
                    fig.svg(script_fig.parts_at(cursor))
                } else {
                    fig.svg(&[] as &[String])
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

fn read_route() -> (u8, u8) {
    let loc = web_sys::window().map(|w| w.location());
    let Some(loc) = loc else {
        return (1, 1);
    };
    if let Ok(hash) = loc.hash() {
        if let Some(pair) = parse_pair(&hash) {
            return pair;
        }
    }
    if let Ok(search) = loc.search() {
        if let Some(pair) = parse_query(&search) {
            return pair;
        }
    }
    if let Ok(path) = loc.pathname() {
        if let Some(pair) = parse_pair(&path) {
            return pair;
        }
    }
    (1, 1)
}

fn parse_query(search: &str) -> Option<(u8, u8)> {
    let s = search.trim_start_matches('?');
    let mut book = None;
    let mut prop = None;
    for part in s.split('&') {
        let (k, v) = part.split_once('=')?;
        let v = v.parse::<u8>().ok()?;
        match k {
            "book" | "b" => book = Some(v),
            "prop" | "proposition" | "p" => prop = Some(v),
            _ => {}
        }
    }
    match (book, prop) {
        (Some(b), Some(p)) => Some(clamp(b, p)),
        (Some(b), None) => Some(clamp(b, 1)),
        _ => None,
    }
}

fn parse_pair(raw: &str) -> Option<(u8, u8)> {
    let s = raw
        .trim()
        .trim_start_matches('#')
        .trim_start_matches('/')
        .split(['?', '&', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s
        .split(['/', '.', '-', '_'])
        .filter(|p| !p.is_empty())
        .collect();
    let (b, p) = match parts.as_slice() {
        ["book", b, "prop", p]
        | ["book", b, "proposition", p]
        | ["b", b, "p", p] => (b.parse().ok()?, p.parse().ok()?),
        ["book", b] | ["b", b] => (b.parse().ok()?, 1),
        [b, p] => (b.parse().ok()?, p.parse().ok()?),
        [b] => (b.parse().ok()?, 1),
        _ => return None,
    };
    Some(clamp(b, p))
}

fn clamp(book: u8, prop: u8) -> (u8, u8) {
    (book.clamp(1, 13), prop.max(1))
}

fn write_route(book: u8, prop: u8, push: bool) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let loc = win.location();
    let path = loc.pathname().unwrap_or_else(|_| "/".into());
    let search = loc.search().unwrap_or_default();
    let hash = format!("#/{book}/{prop}");
    if loc.hash().ok().as_deref() == Some(hash.as_str()) {
        return;
    }
    let url = format!("{path}{search}{hash}");
    if let Ok(history) = win.history() {
        let r = if push {
            history.push_state_with_url(&JsValue::NULL, "", Some(&url))
        } else {
            history.replace_state_with_url(&JsValue::NULL, "", Some(&url))
        };
        if r.is_ok() {
            return;
        }
    }
    let _ = loc.set_hash(&format!("/{book}/{prop}"));
}

fn set_title(book: u8, prop: u8) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        doc.set_title(&format!("Book {book} Proposition {prop} — Euclid’s Elements"));
    }
}
