#!/usr/bin/env python3
"""YouTube thumbnail for the Book I Proposition 39 demo.

1280×720. Cream plate, same family as the I.37 and I.38 cards. Two header
lines, same size — Euclid's Elements, then Book 1 Proposition 39 — sit close
over the Fitzpatrick figure: equal triangles ABC and DBC on the same base BC,
between the parallels AD and BC. E is the crossing of AE with BD. The
capital sits one letter under that junction, the same gap as the plate.
"""

from __future__ import annotations

import math
from pathlib import Path

import cairo

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "youtube-book1-proposition39.png"

W, H = 1280, 720
CREAM = (255, 249, 239)
NAVY = (29, 53, 80)
GOLD = (212, 184, 122)
INK = (28, 32, 38)
WASH = (232, 214, 176)

# Plate points, Fitzpatrick scale BC = 100, same as book1_prop39.
RAW = {
    "B": (0.0, 0.0),
    "C": (100.0, 0.05),
    "A": (5.77, 55.92),
    "D": (81.64, 56.2),
    "E": (71.8432, 49.456),
}
# E's capital is one plate letter under the crossing, not the ink-box
# coordinate. On a card the type is 32px, so the center sits 1.05em below
# the junction on the measured heading (−99.5°).
HEADING = {
    "A": 137.6,
    "B": -165.3,
    "C": -6.4,
    "D": 42.2,
    "E": -99.5,
}
RADIUS = {"A": 26, "B": 26, "C": 26, "D": 26, "E": 34}
STROKES = (
    ("A", "D"),
    ("B", "C"),
    ("A", "B"),
    ("A", "C"),
    ("B", "E"),
    ("E", "D"),
    ("D", "C"),
    ("A", "E"),
    ("E", "C"),
)


def rgb(c):
    return tuple(v / 255 for v in c)


def font(cr, size, bold=False, italic=False):
    cr.select_font_face(
        "Liberation Serif",
        cairo.FONT_SLANT_ITALIC if italic else cairo.FONT_SLANT_NORMAL,
        cairo.FONT_WEIGHT_BOLD if bold else cairo.FONT_WEIGHT_NORMAL,
    )
    cr.set_font_size(size)


def label_offset(name):
    ang = math.radians(HEADING[name])
    r = RADIUS[name]
    return r * math.cos(ang), -r * math.sin(ang)


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
    subtitle = "Book 1 Proposition 39"
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
    # I.39 is short (height/BC ≈ 0.56). The slot under the shared header
    # sets the scale; the width stays centered with room for the letters.
    slot_top, slot_bot = 228, 676
    pad = 30
    scale = (slot_bot - slot_top - 2 * pad) / (maxy - miny)
    ox = (W - (maxx + minx) * scale) / 2
    # y grows up in the plate and down on screen.
    oy = slot_top + pad + maxy * scale

    def pt(name):
        x, y = RAW[name]
        return ox + x * scale, oy - y * scale

    def pt_xy(x, y):
        return ox + x * scale, oy - y * scale

    def poly(names):
        cr.move_to(*pt(names[0]))
        for name in names[1:]:
            cr.line_to(*pt(name))
        cr.close_path()

    # The two equal triangles. A warm wash, not a highlight from the recording.
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
        font(cr, 32, italic=True)
        cr.set_source_rgb(*rgb(NAVY))
        ext = cr.text_extents(name)
        dx, dy = label_offset(name)
        cr.move_to(
            x + dx - (ext.x_bearing + ext.width / 2),
            y + dy - (ext.y_bearing + ext.height / 2),
        )
        cr.show_text(name)

    surf.write_to_png(OUT)
    print(OUT)


if __name__ == "__main__":
    main()
