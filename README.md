# Euclid’s Elements

Leptos (WASM) shell for all 13 books. Book I Proposition 1 is filled in.

```bash
trunk serve
```

http://127.0.0.1:8080/

Live: https://swimming-bookstore.github.io/euclids-elements/

## Karaoke

Word-by-word player in `src/karaoke/`:

- **Read** (default): full proof, full figure. `KaraokeRead`
- **Record** (`?record=1` or Play): karaoke. `KaraokeLyrics`

1. `compile(phrases, &EuclidParts, Timing::default())` — `*AB*` italic, `{[Post. 3]}` cite
2. `Player::start(&script, autoplay)` — cursor + play/pause
3. `script.parts_at(cursor)` — figure ids for the sung word

A new proposition is a `Proposition` in `src/content/bookN.rs` (enunciation, phrases, `svg`) plus a `Figure` in `src/figure.rs`. Register it in `propositions_in`.
