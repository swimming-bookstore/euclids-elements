#!/usr/bin/env python3
"""YouTube thumbnail for the Book I Proposition 38 demo.

1280×720. Cream plate, same family as the I.37 card. Two header lines,
same size — Euclid's Elements, then Book 1 Proposition 38 — sit close over the
Fitzpatrick figure: triangles ABC and DEF on equal bases BC and EF, between
the parallels BF and GH.
"""

from __future__ import annotations

import math
from pathlib import Path

import cairo

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "youtube-book1-proposition38.png"

W, H = 1280, 720
CREAM = (255, 249, 239)
NAVY = (29, 53, 80)
GOLD = (212, 184, 122)
INK = (28, 32, 38)
WASH = (232, 214, 176)

# Plate points, Fitzpatrick scale BC = 100, then grown to the frame.
# `level` x is absolute, same as book1_prop38 — not an offset from G.
# Adding G's x parked A, D, and H on the wrong side of their bases.
RAW = {
    "B": (0.0, 0.0),
    "C": (100.0, 0.0),
    "E": (375.82, 0.0),
    "F": (500.55, 0.0),
    "G": (64.29, 187.91),
    "A": (162.64, 187.91),
    "D": (268.13, 187.91),
    "H": (389.01, 187.91),
}
STROKES = (
    ("G", "H"),
    ("B", "F"),
    ("G", "B"),
    ("A", "B"),
    ("A", "C"),
    ("D", "E"),
    ("D", "F"),
    ("H", "F"),
)
# Off the stroke, the way the plate sets the letters.
LABEL = {
    "G": (8, -18),
    "A": (0, -22),
    "D": (0, -22),
    "H": (-8, -18),
    "B": (-14, 20),
    "C": (0, 24),
    "E": (0, 24),
    "F": (14, 20),
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


def main():
    surf = cairo.ImageSurface(cairo.FORMAT_ARGB32, W, H)
    cr = cairo.Context(surf)

    pat = cairo.LinearGradient(0, 0, W, H)
    pat.add_color_stop_rgb(0, *rgb(CREAM))
    pat.add_color_stop_rgb(1, 0.965, 0.925, 0.875)
    cr.set_source(pat)
    cr.paint()

    # Two header lines, every word the same size, close over the figure.
    # One shared baseline per line so a descending p does not lift "Proposition".
    title = "Euclid's Elements"
    subtitle = "Book 1 Proposition 38"
    size = 48
    font(cr, size, bold=True)
    title_w = cr.text_extents(title).x_advance
    sub_w = cr.text_extents(subtitle).x_advance
    cr.set_source_rgb(*rgb(NAVY))
    font(cr, size, bold=True)
    cr.move_to((W - title_w) / 2, 132)
    cr.show_text(title)
    cr.move_to((W - sub_w) / 2, 188)
    cr.show_text(subtitle)

    xs = [p[0] for p in RAW.values()]
    ys = [p[1] for p in RAW.values()]
    minx, maxx = min(xs), max(xs)
    miny, maxy = min(ys), max(ys)
    # Same figure slot as the I.37 card: top letters on the A/D line,
    # base letters on the B/C line. I.38 is the same plate family but
    # taller and wider (height/BC = 1.879, width/BC = 5.006). Fitting
    # the width at the I.37 scale either crowds the subtitle or runs
    # the bases off the bottom. The slot's height sets the scale.
    i37_top_label_y = 226.8
    i37_bottom_label_y = 581.2
    top_dy = min(dy for _, dy in LABEL.values())
    bot_dy = max(dy for _, dy in LABEL.values())
    scale = (i37_bottom_label_y - i37_top_label_y - (bot_dy - top_dy)) / (maxy - miny)
    ox = (W - (maxx + minx) * scale) / 2
    # y grows up in the plate and down on screen.
    oy = i37_top_label_y - top_dy + maxy * scale

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
    poly(("D", "E", "F"))
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
