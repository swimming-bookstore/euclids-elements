#!/usr/bin/env python3
"""Read Fitzpatrick’s English column from Elements.pdf.

The right-hand column is x ≳ 310 pt. A new paragraph is an indent of
about 15 pt past the body left (314 → 329). Hyphenated line-ends are
joined. Figure letters and running heads are dropped.

    python3 scripts/fitzpatrick.py --from 11 --to 12
    python3 scripts/fitzpatrick.py --from 11 --to 12 --json
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pdf", type=Path, default=DEFAULT_PDF)
    parser.add_argument("--from", dest="first", type=int, required=True)
    parser.add_argument("--to", dest="last", type=int, default=None)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    last = args.last or args.first
    html = bbox_html(args.pdf, args.first, last)
    lines = english_lines(html)
    paras = paragraphs(lines)
    if args.json:
        print(json.dumps(paras, indent=2, ensure_ascii=False))
        return
    for i, p in enumerate(paras, 1):
        print(f"— {i} —")
        print(p)
        print()


if __name__ == "__main__":
    main()
