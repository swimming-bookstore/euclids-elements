# Euclid’s Elements

Leptos (WASM) shell for all 13 books. Book I Propositions 1–2 are filled in.

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

A new proposition is a `Proposition` in `src/content/bookN.rs` plus a `Diagram` in `src/figure/bookN.rs` (`put` / `join` / `circle` / `ray`). Karaoke lighting is derived from that graph.

Read mode flows Fitzpatrick paragraphs (`src/karaoke/layout.rs`): citations hang in the right margin; a cited clause starts the next sentence on a new line. A full stop with no cite stays in the paragraph. Record mode still steps one sentence at a time.

Figures are fitted to a 1000×1000 plate (`src/figure/plate.rs`). Point letters (`src/figure/labels.rs`) sit with their **center** at a fixed radius from the mark; type is CSS (`clamp` + `cqmin`) so it stays readable when the diagram resizes.
