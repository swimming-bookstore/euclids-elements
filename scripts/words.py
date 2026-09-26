#!/usr/bin/env python3
"""Read Fitzpatrick’s English column from Elements.pdf.

The right-hand column is x ≳ 310 pt. A new paragraph is an indent of
about 15 pt past the body left (314 → 329). Hyphenated line-ends are
joined. Figure letters and running heads are dropped.

    python3 scripts/words.py --from 15 --to 15
    python3 scripts/words.py --from 15 --to 15 --phrases --start 4
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PDF = Path("/tmp/euclid-pdf/Elements.pdf")
ENG_X0 = 300.0
BODY_X = 314.0
INDENT = 10.0  # pt past body left → new paragraph
LINE_DY = 5.0
HEADER_Y = 90.0
FOOT_Y = 740.0


def bbox_html(pdf: Path, first: int, last: int) -> str:
    with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
        out = Path(f.name)
    try:
        subprocess.run(
            [
                "pdftotext",
                "-f",
                str(first),
                "-l",
                str(last),
                "-bbox",
                str(pdf),
                str(out),
            ],
            check=True,
            capture_output=True,
        )
        return out.read_text(errors="replace")
    finally:
        out.unlink(missing_ok=True)


WORD_RE = re.compile(
    r'<word xMin="([^"]+)" yMin="([^"]+)" xMax="([^"]+)" yMax="([^"]+)">([^<]*)</word>'
)


def english_lines(html: str) -> list[tuple[int, float, float, str]]:
    """(page, y, x0, text) for each English line."""
    pages = html.split("<page ")
    out: list[tuple[int, float, float, str]] = []
    for pi, page in enumerate(pages[1:], start=1):
        words = [
            (float(a), float(b), float(c), w)
            for a, b, c, _, w in WORD_RE.findall(page)
            if float(a) >= ENG_X0 and HEADER_Y < float(b) < FOOT_Y
        ]
        lines: list[list] = []
        for x, y, x2, w in words:
            if not lines or abs(y - lines[-1][0]) > LINE_DY:
                lines.append([y, [(x, x2, w)]])
            else:
                lines[-1][1].append((x, x2, w))
        for y, ws in lines:
            ws.sort()
            x0 = ws[0][0]
            text = " ".join(t for _, _, t in ws)
            text = re.sub(r"\s+", " ", text).strip()
            if not text or _skip_line(text, x0):
                continue
            out.append((pi, y, x0, text))
    return out


def _skip_line(text: str, x0: float) -> bool:
    if re.fullmatch(r"[A-Z]", text):
        return True
    if re.fullmatch(r"[A-Z]( [A-Z])+", text) and len(text) < 20:
        return True
    if text.startswith("ELEMENTS") or text.startswith("BOOK"):
        return True
    if re.fullmatch(r"\d+", text):
        return True
    # Footnote bodies under the dagger/double-dagger.
    if text in {"postulate.", "points is unique."}:
        return True
    return False


def dehyphenate(a: str, b: str) -> str | None:
    """Join a wrapped line. Keep the hyphen only on compounds (straight-lines)."""
    if not (a.endswith("-") and b and b[0].islower()):
        if a.endswith("-,") and b:
            return a[:-1] + b
        return None
    stem = re.search(r"([A-Za-z]+)-$", a)
    keep = stem and stem.group(1).lower() in {"straight", "well", "self", "non"}
    if keep:
        return a + b
    return a[:-1] + b


def paragraphs(lines: list[tuple[int, float, float, str]]) -> list[str]:
    paras: list[list[str]] = []
    body_left: float | None = None
    for _page, _y, x0, text in lines:
        if text.startswith("Proposition "):
            paras.append([text])
            continue
        if body_left is None or abs(x0 - BODY_X) < 4:
            if abs(x0 - BODY_X) < 4:
                body_left = min(body_left or x0, x0)
        indent = body_left is not None and x0 >= body_left + INDENT
        if not paras or indent or text.startswith("Proposition "):
            paras.append([text])
            continue
        last = paras[-1]
        joined = dehyphenate(last[-1], text)
        if joined is not None:
            last[-1] = joined
        else:
            last.append(text)
    return [" ".join(p) for p in paras if p]


CITE_RE = re.compile(r"\[([^\]]+)\]")
GEOM_RE = re.compile(
    r"\b((?:straight-lines?|triangle|triangles|angle|angles|side|sides|"
    r"base|bases|point|points|circle|circles)?\s*)"
    r"([A-Z]{1,4}(?:\s*,\s*[A-Z]{1,4})*)\b"
)


def rust_escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def mark_geometry(text: str) -> str:
    """Wrap spoken figure names in *…* for karaoke lighting."""

    def repl(m: re.Match) -> str:
        prefix, names = m.group(1), m.group(2)
        bits = re.split(r"(\s*,\s*)", names)
        out = []
        for b in bits:
            if re.fullmatch(r"[A-Z]{1,4}", b) and b != "I":
                out.append(f"*{b}*")
            else:
                out.append(b)
        return prefix + "".join(out)

    return GEOM_RE.sub(repl, text)


def sentences(para: str) -> list[str]:
    # Don't split on the period inside [Prop. 1.3].
    held: list[str] = []

    def stash(m: re.Match) -> str:
        held.append(m.group(0))
        return f"\x00{len(held) - 1}\x00"

    tmp = CITE_RE.sub(stash, para.strip())
    parts = re.split(r"(?<=[.!?])\s+", tmp)
    out = []
    for p in parts:
        p = p.strip()
        if not p:
            continue
        p = re.sub(r"\x00(\d+)\x00", lambda m: held[int(m.group(1))], p)
        out.append(p)
    return out


def phrase_line(para_n: int, sent: str) -> str:
    body = CITE_RE.sub(r"{[\1]}", sent)
    body = re.sub(r"\s+", " ", body).strip()
    body = re.sub(r"\s+\{", "{", body)
    # Comma/period before a cite sits on the word, like "DE,{[Prop. 1.1]}"
    body = re.sub(r"\{\[([^\]]+)\]\}([,.;:])", r"\2{[\1]}", body)
    body = mark_geometry(body)
    return f'        s({para_n}, "{rust_escape(body)}"),'


def phrases_from_paras(paras: list[str], start: int = 1) -> list[str]:
    """Karaoke phrases. `start` is the first body paragraph to keep
    (skip running QED / previous prop). Stops at the next Proposition."""
    out: list[str] = []
    n = 0
    for p in paras[start - 1 :]:
        if p.startswith("Proposition ") and n:
            break
        if p.startswith("Proposition "):
            continue
        n += 1
        for sent in sentences(p):
            if sent in {"postulate.", "points is unique."}:
                continue
            out.append(phrase_line(n, sent))
    return out


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pdf", type=Path, default=DEFAULT_PDF)
    parser.add_argument("--from", dest="first", type=int, required=True)
    parser.add_argument("--to", dest="last", type=int, default=None)
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "--phrases",
        action="store_true",
        help="print karaoke s(para, …) lines from English paragraphs",
    )
    parser.add_argument(
        "--start",
        type=int,
        default=1,
        help="1-based paragraph index to start at (after --phrases)",
    )
    args = parser.parse_args()
    last = args.last or args.first
    html = bbox_html(args.pdf, args.first, last)
    lines = english_lines(html)
    paras = paragraphs(lines)
    if args.phrases:
        for line in phrases_from_paras(paras, args.start):
            print(line)
        return
    if args.json:
        print(json.dumps(paras, indent=2, ensure_ascii=False))
        return
    for i, p in enumerate(paras, 1):
        print(f"— {i} —")
        print(p)
        print()


if __name__ == "__main__":
    main()
