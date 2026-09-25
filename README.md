# Euclid’s Elements

Leptos (WASM) shell for all 13 books. Book I Propositions 1–7 are filled in.

```bash
trunk serve
```

http://127.0.0.1:8080/

Live: https://swimming-bookstore.github.io/euclids-elements/

## Karaoke

Word-by-word player in `src/karaoke/`:

- **Read** (default): full proof, full figure. `KaraokeRead`
- **Record** (`?record=1` or Play): karaoke. `KaraokeLyrics`

1. `compile(phrases, &diagram, Timing::default())` — `*AB*` italic, `{[Post. 3]}` cite
2. `Player::start(&script, autoplay)` — cursor + play/pause
3. `script.parts_at(cursor)` — figure ids from the construction graph

A new proposition is a `Proposition` in `src/content/bookN.rs` plus a plate in `scripts/figure.py`. Run `python3 scripts/figure.py --write` to emit `src/figure/book1.rs` (`put` / `join` / `circle` / `ray`). Karaoke lighting is derived from that graph.

Read mode flows Fitzpatrick paragraphs (`src/karaoke/layout.rs`): citations hang in the right margin; a cited clause starts the next sentence on a new line. A full stop with no cite stays in the paragraph. Record mode still steps one sentence at a time.

Figures are fitted to a plate (`src/figure/plate.rs`). Point letters (`src/figure/labels.rs`) sit with their **center** at `--r: 0.92em` from the mark, in the author’s direction — the same radius on every diagram, at every size.
