#!/usr/bin/env python3
"""YouTube thumbnail for the Book I Proposition 37 demo.

1280×720. Cream plate, same family as the LinkedIn card. The three lines sit as
a header; the Fitzpatrick figure fills the field under them: triangles ABC and
DBC on the same base BC, between the parallels EF and BC.
"""

from __future__ import annotations

import math
from pathlib import Path

import cairo

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "youtube-book1-proposition37.png"

W, H = 1280, 720
CREAM = (255, 249, 239)
NAVY = (29, 53, 80)
NAVY_DEEP = (16, 30, 48)
GOLD = (212, 184, 122)
INK = (28, 32, 38)
WASH = (232, 214, 176)

# Plate points, Fitzpatrick scale BC = 100, then grown to the frame.
RAW = {
    "B": (0.0, 0.0),
    "C": (100.0, 0.0),
    "E": (-59.09, 69.32),
    "A": (34.85, 69.32),
    "D": (79.17, 69.32),
    "F": (176.14, 69.32),
}
STROKES = (
    ("E", "F"),
    ("B", "C"),
    ("A", "B"),
    ("A", "C"),
    ("D", "B"),
    ("D", "C"),
    ("E", "B"),
    ("F", "C"),
)
# Off the stroke, the way the plate sets the letters.
LABEL = {
    "E": (-16, -4),
    "A": (0, -20),
    "D": (2, -20),
    "F": (16, -4),
    "B": (-2, 22),
    "C": (4, 22),
}


def rgb(c):
    return tuple(v / 255 for v in c)


def font(cr, size, bold=False, italic=False):
    cr.select_font_face(
        "Liberation Serif",
        cairo.FONT_SLANT_ITALIC if italic else cairo.FONT_SLANT_NORMAL,
        cairo.FONT_WEIGHT_BOLD if bold else cairo.FONT_WEIGHT_NORMAL,
    )
    cr.set_font_size(size)


def center_text(cr, text, cx, cy, size, fill, bold=False, italic=False):
    font(cr, size, bold, italic)
    cr.set_source_rgb(*rgb(fill))
    ext = cr.text_extents(text)
    cr.move_to(cx - (ext.x_bearing + ext.width / 2), cy - (ext.y_bearing + ext.height / 2))
    cr.show_text(text)


def tracked(cr, text, x, cy, size, fill, tracking, bold=False):
    font(cr, size, bold)
    cr.set_source_rgb(*rgb(fill))
    # One baseline for the whole line. Centering each glyph on its own box
    # lifts round letters ("oo") above letters with a taller box ("k").
    ext = cr.text_extents(text)
    baseline = cy - (ext.y_bearing + ext.height / 2)
    x0 = x
    for ch in text:
        chext = cr.text_extents(ch)
        cr.move_to(x0 - chext.x_bearing, baseline)
        cr.show_text(ch)
        x0 += chext.x_advance + tracking


def wrap(cr, text, size, width, italic=False):
    font(cr, size, italic=italic)
    lines, cur = [], ""
    for word in text.split():
        trial = word if not cur else f"{cur} {word}"
        if cr.text_extents(trial).x_advance <= width:
            cur = trial
        else:
            lines.append(cur)
            cur = word
    if cur:
        lines.append(cur)
    return lines


def main():
    surf = cairo.ImageSurface(cairo.FORMAT_ARGB32, W, H)
    cr = cairo.Context(surf)

    pat = cairo.LinearGradient(0, 0, W, H)
    pat.add_color_stop_rgb(0, *rgb(CREAM))
    pat.add_color_stop_rgb(1, 0.965, 0.925, 0.875)
    cr.set_source(pat)
    cr.paint()

    # Header. Same size, same navy, one shared baseline. Centering each phrase
    # on its own ink box lifts "Proposition" (the p descends) above the others.
    size = 46
    gap = 28
    lines = ("Euclid's Elements", "Book 1", "Proposition 37")
    font(cr, size, bold=True)
    widths = [cr.text_extents(line).x_advance for line in lines]
    block_w = sum(widths) + gap * (len(lines) - 1)
    x = (W - block_w) / 2
    baseline = 92
    cr.set_source_rgb(*rgb(NAVY))
    for line, width in zip(lines, widths):
        cr.move_to(x, baseline)
        cr.show_text(line)
        x += width + gap

    # Figure, centered in the field under the header. Side padding includes the
    # letter labels, which sit outside the strokes.
    xs = [p[0] for p in RAW.values()]
    ys = [p[1] for p in RAW.values()]
    minx, maxx = min(xs), max(xs)
    miny, maxy = min(ys), max(ys)
    top = 150
    side = 110
    bottom = 64
    scale = min((W - 2 * side) / (maxx - minx), (H - top - bottom) / (maxy - miny))
    ox = (W - (maxx - minx) * scale) / 2 - minx * scale
    # y grows up in the plate and down on screen. Mid-height of the
    # construction lands on the mid-height of the field under the header.
    oy = (top + H - bottom) / 2 + (maxy + miny) / 2 * scale

    def pt(name):
        x, y = RAW[name]
        return ox + x * scale, oy - y * scale

    def poly(names):
        cr.move_to(*pt(names[0]))
        for name in names[1:]:
            cr.line_to(*pt(name))
        cr.close_path()

    # The two triangles. A warm wash, not a highlight from the recording,
    # so both halves read as the equal pair.
    cr.set_source_rgba(*rgb(WASH), 0.85)
    poly(("A", "B", "C"))
    cr.fill()
    cr.set_source_rgba(*rgb(GOLD), 0.28)
    poly(("D", "B", "C"))
    cr.fill()

    cr.set_source_rgb(*rgb(INK))
    cr.set_line_width(3.2)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    cr.set_line_join(cairo.LINE_JOIN_ROUND)
    for a, b in STROKES:
        cr.move_to(*pt(a))
        cr.line_to(*pt(b))
        cr.stroke()

    for name in RAW:
        x, y = pt(name)
        cr.set_source_rgb(*rgb(INK))
        cr.arc(x, y, 4.2, 0, math.tau)
        cr.fill()
        dx, dy = LABEL[name]
        font(cr, 32, italic=True)
        cr.set_source_rgb(*rgb(NAVY))
        ext = cr.text_extents(name)
        cr.move_to(
            x + dx - (ext.x_bearing + ext.width / 2),
            y + dy - (ext.y_bearing + ext.height / 2),
        )
        cr.show_text(name)

    surf.write_to_png(OUT)
    print(OUT)


if __name__ == "__main__":
    main()
