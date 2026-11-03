#!/usr/bin/env python3
"""YouTube thumbnail for the Book I Proposition 40 demo.

1280×720. Cream plate, same family as the I.37–I.39 cards. Two header
lines, same size — Euclid's Elements, then Book 1 Proposition 40 — sit close
over the Fitzpatrick figure: equal triangles ABC and CDE on the equal bases
BC and CE, between the parallels AD and BE. F is the crossing of AF with DC.
The capital sits one letter off that junction, the same gap as the plate.
"""

from __future__ import annotations

import math
from pathlib import Path

import cairo

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "youtube-book1-proposition40.png"

W, H = 1280, 720
CREAM = (255, 249, 239)
NAVY = (29, 53, 80)
GOLD = (212, 184, 122)
INK = (28, 32, 38)
WASH = (232, 214, 176)

# Plate points, Fitzpatrick scale BC = 100, same as book1_prop40.
RAW = {
    "B": (0.0, 0.0),
    "C": (100.0, 0.0),
    "E": (194.46, 0.0),
    "A": (15.57, 92.0),
    "D": (166.44, 92.0),
    "F": (151.53, 71.34),
}
# F's capital is one plate letter off the crossing, not the ink-box
# coordinate. On a card the type is 32px, so the center sits 1.05em from
# the junction on the measured heading (−151.5°).
HEADING = {
    "A": 166.4,
    "B": -178.7,
    "C": -77.0,
    "D": 11.1,
    "E": -5.8,
    "F": -151.5,
}
RADIUS = {"A": 26, "B": 26, "C": 26, "D": 26, "E": 26, "F": 34}
STROKES = (
    ("A", "D"),
    ("B", "C"),
    ("C", "E"),
    ("A", "B"),
    ("A", "C"),
    ("D", "F"),
    ("F", "C"),
    ("D", "E"),
    ("A", "F"),
    ("F", "E"),
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
    subtitle = "Book 1 Proposition 40"
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
    # I.40 is wider than I.39 (width/BC ≈ 1.95, height/BC = 0.92). The slot
    # under the shared header sets the scale; letters stay inside the frame.
    slot_top, slot_bot = 228, 676
    pad_x, pad_y = 56, 48
    scale = min(
        (W - 2 * pad_x) / (maxx - minx),
        (slot_bot - slot_top - 2 * pad_y) / (maxy - miny),
    )
    ox = (W - (maxx + minx) * scale) / 2
    # y grows up in the plate and down on screen.
    oy = slot_top + pad_y + maxy * scale

    def pt(name):
        x, y = RAW[name]
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
    poly(("C", "D", "E"))
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
