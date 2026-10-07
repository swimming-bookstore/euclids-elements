#!/usr/bin/env python3
"""Author Euclidean plates as Python, emit src/figure/book1.rs.

Figures copy Fitzpatrick’s English plate (Elements.pdf), not a “nice”
construction. Vertices are **ink tips**, not letter boxes — a letter
names the nearest stroke-point outside its glyph, so segment lengths
stay in plate proportion.

    python3 scripts/figure.py --measure PAGE
        rasterize the English column, blank the capitals, take the
        nearest ink to each letter. Shared *ink* rows (`level`) and
        columns (`plumb`) snap; a cut is `meet`. Prints one-scale
        ratios and angles.

    Put those in `book1_propN` as a Sketch (`put` / `polar` / `level` /
    `plumb` / `meet`, then `require_*`). Copy `s.letters(...)` from
    `--measure` (tip→glyph headings). `Fig(..., sketch=s)` then places
    every letter that way unless you pass a `place`.

    python3 scripts/figure.py --write
        emit src/figure/book1.rs
"""

from __future__ import annotations

import argparse
import math
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "src" / "figure" / "book1.rs"
DEFAULT_PDF = Path("/tmp/euclid-pdf/Elements.pdf")
ENG_X0_PT = 310.0


class Sketch:
    """Plane sketch, y-up, degrees from +x (same as `V2::polar`).

    Place points, then `require_angle` / `require_eq` against the plate.
    """

    def __init__(self):
        self.pts: dict[str, tuple[float, float]] = {}
        self._letters: dict[str, float] = {}

    def put(self, name: str, x: float, y: float) -> tuple[float, float]:
        self.pts[name] = (x, y)
        return self.pts[name]

    def polar(self, name: str, origin: str, r: float, deg: float) -> tuple[float, float]:
        ox, oy = self.pts[origin]
        a = math.radians(deg)
        return self.put(name, ox + r * math.cos(a), oy + r * math.sin(a))

    def ray(self, name: str, origin: str, through: str, dist: float) -> tuple[float, float]:
        ox, oy = self.pts[origin]
        tx, ty = self.pts[through]
        dx, dy = tx - ox, ty - oy
        n = math.hypot(dx, dy)
        return self.put(name, ox + dx / n * dist, oy + dy / n * dist)

    def meet(self, name: str, a: str, b: str, c: str, d: str) -> tuple[float, float]:
        """`name` at the intersection of lines `a``b` and `c``d`."""
        ax, ay = self.pts[a]
        bx, by = self.pts[b]
        cx, cy = self.pts[c]
        dx, dy = self.pts[d]
        den = (ax - bx) * (cy - dy) - (ay - by) * (cx - dx)
        if abs(den) < 1e-12:
            raise SystemExit(f"{a}{b} ∥ {c}{d}")
        t = ((ax - cx) * (cy - dy) - (ay - cy) * (cx - dx)) / den
        return self.put(name, ax + t * (bx - ax), ay + t * (by - ay))

    def level(self, name: str, of: str, x: float) -> tuple[float, float]:
        """`name` at (`x`, y of `of`) — same height as a plate letter-row."""
        return self.put(name, x, self.pts[of][1])

    def plumb(self, name: str, of: str, y: float) -> tuple[float, float]:
        """`name` at (x of `of`, `y`) — same column as a plate letter-column."""
        return self.put(name, self.pts[of][0], y)

    def corner(self, name: str, x_of: str, y_of: str) -> tuple[float, float]:
        """`name` at (x of `x_of`, y of `y_of`) — a letter-column × letter-row."""
        return self.put(name, self.pts[x_of][0], self.pts[y_of][1])

    def copy_length(self, name: str, origin: str, through: str, a: str, b: str) -> tuple[float, float]:
        """`origin`–`name` on the ray `origin` → `through`, equal to `a`–`b`."""
        return self.ray(name, origin, through, self.dist(a, b))

    def turn(
        self,
        name: str,
        vertex: str,
        along: str,
        deg: float,
        dist: float | None = None,
    ) -> tuple[float, float]:
        """Point at `dist` from `vertex`; ray `vertex`→`along` rotated `deg` (ccw +)."""
        if dist is None:
            dist = self.dist(vertex, along)
        return self.polar(name, vertex, dist, self.heading(vertex, along) + deg)

    def copy_angle(
        self,
        name: str,
        vertex: str,
        along: str,
        p: str,
        v: str,
        q: str,
        dist: float | None = None,
        side: float = 1.0,
    ) -> tuple[float, float]:
        """`name` so ∠(`along`, `vertex`, `name`) = ∠`p``v``q`.

        `side` +1 is left of `vertex` → `along` (ccw); −1 is right (cw).
        """
        return self.turn(name, vertex, along, side * self.angle(p, v, q), dist)

    def at(self, name: str) -> tuple[float, float]:
        return self.pts[name]

    def dist(self, a: str, b: str) -> float:
        ax, ay = self.pts[a]
        bx, by = self.pts[b]
        return math.hypot(bx - ax, by - ay)

    def angle(self, p: str, v: str, q: str) -> float:
        """∠pvq in degrees."""
        px, py = self.pts[p]
        vx, vy = self.pts[v]
        qx, qy = self.pts[q]
        ax, ay = px - vx, py - vy
        bx, by = qx - vx, qy - vy
        na, nb = math.hypot(ax, ay), math.hypot(bx, by)
        return math.degrees(math.acos(max(-1.0, min(1.0, (ax * bx + ay * by) / (na * nb)))))

    def line_angle(self, a: str, b: str, c: str, d: str) -> float:
        """Smaller angle between lines ab and cd."""
        ax, ay = self.pts[a]
        bx, by = self.pts[b]
        cx, cy = self.pts[c]
        dx, dy = self.pts[d]
        ux, uy = bx - ax, by - ay
        vx, vy = dx - cx, dy - cy
        nu, nv = math.hypot(ux, uy), math.hypot(vx, vy)
        ang = math.degrees(math.acos(max(-1.0, min(1.0, abs(ux * vx + uy * vy) / (nu * nv)))))
        return min(ang, 180.0 - ang)

    def require_angle(self, p: str, v: str, q: str, deg: float, eps: float = 0.05) -> None:
        got = self.angle(p, v, q)
        if abs(got - deg) > eps:
            raise SystemExit(f"∠{p}{v}{q} = {got:.4f}°, want {deg}")

    def require_eq(self, a: str, b: str, c: str, d: str, eps: float = 1e-6) -> None:
        ga, gb = self.dist(a, b), self.dist(c, d)
        if abs(ga - gb) > eps:
            raise SystemExit(f"{a}{b} = {ga:.6g}, {c}{d} = {gb:.6g}")

    def require_same_angle(
        self, p: str, v: str, q: str, a: str, b: str, c: str, eps: float = 0.05
    ) -> None:
        got, want = self.angle(p, v, q), self.angle(a, b, c)
        if abs(got - want) > eps:
            raise SystemExit(f"∠{p}{v}{q} = {got:.4f}°, ∠{a}{b}{c} = {want:.4f}°")

    def require_line_angle(self, a: str, b: str, c: str, d: str, deg: float, eps: float = 0.05) -> None:
        got = self.line_angle(a, b, c, d)
        if abs(got - deg) > eps:
            raise SystemExit(f"∠({a}{b},{c}{d}) = {got:.4f}°, want {deg}")

    def require_ratio(self, a: str, b: str, c: str, d: str, ratio: float, eps: float = 0.002) -> None:
        got = self.dist(a, b) / self.dist(c, d)
        if abs(got - ratio) > eps:
            raise SystemExit(f"{a}{b}/{c}{d} = {got:.4f}, want {ratio}")

    def heading(self, a: str, b: str) -> float:
        ax, ay = self.pts[a]
        bx, by = self.pts[b]
        return math.degrees(math.atan2(by - ay, bx - ax))

    def outward(self, along_a: str, along_b: str, sign: float) -> tuple:
        """Letter on the ±90° normal of `along_a` → `along_b`."""
        return ("deg", round(self.heading(along_a, along_b) + 90.0 * sign, 1))

    def beside(self, a: str, b: str, sign: float, tilt: float = 40.0) -> tuple:
        """Letter at `b`, beside segment `a`–`b` (not past the tip).

        `sign` +1 is left of `a` → `b`; −1 is right. `tilt` pulls the glyph
        back along the stroke so an endpoint letter sits on the side.
        """
        return ("deg", round(self.heading(a, b) + sign * (90.0 + tilt), 1))

    def beyond(self, a: str, b: str) -> tuple:
        """Letter past `b`, along `a` → `b` — only when the plate does that."""
        return ("deg", round(self.heading(a, b), 1))

    def letters(self, **headings: float) -> None:
        """Tip→glyph headings from `--measure PAGE` (`s.letters(A=92.4, …)`)."""
        self._letters.update({k: float(v) for k, v in headings.items()})

    def glyph(self, name_or_deg: str | float) -> tuple:
        """Letter center from the ink tip toward the Fitzpatrick glyph.

        A name looks up `letters(...)`. A number is the heading itself.
        """
        if isinstance(name_or_deg, str):
            if name_or_deg not in self._letters:
                raise SystemExit(
                    f"no tip→glyph heading for {name_or_deg}; "
                    "s.letters(...) from --measure PAGE"
                )
            name_or_deg = self._letters[name_or_deg]
        return ("deg", round(float(name_or_deg), 1))

    def report(self, title: str, angles: list[tuple]) -> None:
        print(title)
        for item in angles:
            if len(item) == 3:
                p, v, q = item
                print(f"  ∠{p}{v}{q} = {self.angle(p, v, q):.3f}°")
            else:
                a, b, c, d = item
                print(f"  ∠({a}{b},{c}{d}) = {self.line_angle(a, b, c, d):.3f}°")
        for name, (x, y) in self.pts.items():
            print(f"  {name}: ({x:.3f}, {y:.3f})")


PLACES = {
    "auto": "Auto",
    "left": "Left",
    "right": "Right",
    "above": "Above",
    "below": "Below",
    "above_left": "AboveLeft",
    "above_right": "AboveRight",
    "below_left": "BelowLeft",
    "below_right": "BelowRight",
    "line_left": "LineLeft",
    "line_right": "LineRight",
}


def rust_place(place) -> str:
    if isinstance(place, tuple) and place[0] == "deg":
        return f"Place::Deg({place[1]})"
    return f"Place::{PLACES[place]}"


def f64(x: float) -> str:
    s = f"{x:.9g}"
    if "." not in s and "e" not in s and "E" not in s:
        s += ".0"
    return s


class Fig:
    def __init__(self, fn: str, doc: str, sketch: Sketch | None = None):
        self.fn = fn
        self.doc = doc
        self.sketch = sketch
        self.lines: list[str] = []

    def _add(self, s: str) -> None:
        self.lines.append(s)

    def _place(self, name: str, place) -> str:
        if place is None:
            if self.sketch is None:
                raise SystemExit(
                    f"{self.fn}: {name} needs a place, or Fig(..., sketch=s) with s.letters(...)"
                )
            place = self.sketch.glyph(name)
        return rust_place(place)

    def put(self, name: str, x: float, y: float, place=None, r: float | None = None) -> None:
        if r is None:
            self._add(
                f'    d.put("{name}", V2::new({f64(x)}, {f64(y)}), {self._place(name, place)});'
            )
        else:
            self._add(
                f'    d.put_r("{name}", V2::new({f64(x)}, {f64(y)}), {self._place(name, place)}, {f64(r)});'
            )

    def put_line_end(self, name: str, x: float, y: float, left: bool) -> None:
        """Given-line end: letter outside the end, slightly above the stroke."""
        side = "true" if left else "false"
        self._add(
            f'    d.put_line_end("{name}", V2::new({f64(x)}, {f64(y)}), {side});'
        )

    def level(self, name: str, of: str, x: float, place=None) -> None:
        self._add(
            f'    d.level("{name}", "{of}", {f64(x)}, {self._place(name, place)});'
        )

    def plumb(self, name: str, of: str, y: float, place=None) -> None:
        self._add(
            f'    d.plumb("{name}", "{of}", {f64(y)}, {self._place(name, place)});'
        )

    def corner(self, name: str, x_of: str, y_of: str, place=None) -> None:
        self._add(
            f'    d.corner("{name}", "{x_of}", "{y_of}", {self._place(name, place)});'
        )

    def meet(self, name: str, a: str, b: str, c: str, d: str, place=None) -> None:
        self._add(
            f'    d.meet("{name}", "{a}", "{b}", "{c}", "{d}", {self._place(name, place)});'
        )

    def pin(self, name: str, x: float, y: float) -> None:
        self._add(f'    d.pin("{name}", V2::new({f64(x)}, {f64(y)}));')

    def polar(self, name: str, origin: str, r: float, deg: float, place=None) -> None:
        self._add(
            f'    d.polar("{name}", "{origin}", {f64(r)}, {f64(deg)}, {self._place(name, place)});'
        )

    def ray(self, name: str, origin: str, through: str, dist: float, place=None) -> None:
        self._add(
            f'    d.ray("{name}", "{origin}", "{through}", {f64(dist)}, {self._place(name, place)});'
        )

    def join(self, a: str, b: str) -> None:
        self._add(f'    d.join("{a}", "{b}");')

    def chain(self, *names: str) -> None:
        inner = ", ".join(f'"{n}"' for n in names)
        self._add(f"    d.chain(&[{inner}]);")

    def base(self, a: str, b: str) -> None:
        """The given straight-line (drawn even if later joins share its ends)."""
        self._add(f'    d.base("{a}", "{b}");')

    def circle(self, letters: str, center: str, through: str) -> None:
        self._add(f'    d.circle("{letters}", "{center}", "{through}");')

    def circle_r(self, letters: str, center: str, r: float) -> None:
        """Named circle of a given radius. The plate's circle is not always the compass circle through the named point."""
        self._add(f'    d.circle_r("{letters}", "{center}", {f64(r)});')

    def circle_arc(self, letters: str, center: str, on: str) -> None:
        """The named circle, drawn in full as on the plate."""
        self.circle(letters, center, on)

    def on_circle(
        self, name: str, center: str, through: str, deg: float, inside: bool = False
    ) -> None:
        """Letter on the circumference at `deg` from +x.

        Default: radially outside. `inside=True` sits the glyph in the disk.
        """
        method = "in_circle" if inside else "on_circle"
        self._add(
            f'    d.{method}("{name}", "{center}", "{through}", {f64(deg)});'
        )

    def on_line(self, name: str, a: str, b: str, t: float, place=None) -> None:
        """Letter on segment `a`–`b` at fraction `t`."""
        self._add(
            f'    d.on_line("{name}", "{a}", "{b}", {f64(t)}, {self._place(name, place)});'
        )

    def named_line(self, letters: str, a: str, b: str, place=None) -> None:
        """The spoken stroke. A letter at `place` (t, side) overrides the midpoint.

        `place=False` draws the stroke only — the letter is placed elsewhere
        (I.22's given lines, letter at the left end).
        """
        self._add(f'    d.named_line("{letters}", "{a}", "{b}");')
        if place is False:
            return
        if place is None:
            # Letter at the midpoint, above the stroke (I.3’s C).
            self.on_line(letters, a, b, 0.5, "above")
        else:
            t, side = place
            self.on_line(letters, a, b, t, side)

    def arc(self, a: str, b: str, x: float, y: float) -> None:
        self._add(
            f'    d.arc("{a}", "{b}", V2::new({f64(x)}, {f64(y)}));'
        )

    def bow(self, a: str, b: str, sag: float) -> None:
        """Circular arc on chord `a`–`b`. Negative sag hangs below a left-to-right chord."""
        self._add(f'    d.bow("{a}", "{b}", {f64(sag)});')

    def dots(self, *names: str) -> None:
        inner = ", ".join(f'"{n}"' for n in names)
        self._add(f"    d.dots(&[{inner}]);")

    def clip(self, x0: float, y0: float, x1: float, y1: float) -> None:
        self._add(
            f"    d.clip(\n"
            f"        V2::new({f64(x0)}, {f64(y0)}),\n"
            f"        V2::new({f64(x1)}, {f64(y1)}),\n"
            f"    );"
        )

    def rust(self) -> str:
        doc = "\n".join(f"/// {line}" for line in self.doc.splitlines())
        body = "\n".join(self.lines)
        return (
            f"{doc}\n"
            f"pub fn {self.fn}() -> Diagram {{\n"
            f"    let mut d = Diagram::new();\n"
            f"{body}\n"
            f"    d\n"
            f"}}\n"
        )


def book1_prop1() -> Fig:
    ab = 240.0
    f = Fig(
        "book1_prop1",
        "I.1 — equilateral triangle on AB. C above the base; D, E on the sides.",
    )
    f.put("A", 0.0, 0.0, "left")
    f.put("B", ab, 0.0, "right")
    f.put("C", ab / 2.0, ab * math.sqrt(3.0) / 2.0, "above")
    f.circle("BCD", "A", "B")
    f.circle("ACE", "B", "A")
    # D, E name the outer cuts; letters sit *inside* the disks (Fitzpatrick).
    f.on_circle("D", "A", "B", 180.0, inside=True)
    f.on_circle("E", "B", "A", 0.0, inside=True)
    f.join("A", "B")
    f.join("A", "C")
    f.join("B", "C")
    f.dots("A", "B", "C")
    return f


def book1_prop2() -> Fig:
    # Fitzpatrick p. 9, from the PDF line art. One scale: DA = 1.
    #   ∠ADB = 53.92°, DA : DB : BC = 1 : 1.025 : 2.463.
    #   BC is vertical; DA heading −81.90°, DB −27.99°.
    da = 120.0
    db = da * 1.024895
    bc = da * 2.463469
    s = Sketch()
    s.put("D", 0.0, 0.0)
    s.polar("A", "D", da, -81.904)
    s.polar("B", "D", db, -27.986)
    s.polar("C", "B", bc, 90.0)  # BC vertical
    s.ray("G", "D", "B", db + bc)  # BG = BC (circle CGH)
    s.ray("L", "D", "A", db + bc)  # DL = DG (circle GKL)
    s.ray("E", "D", "A", da * 5.129262)
    s.ray("F", "D", "B", da * 4.838225)
    s.require_angle("A", "D", "B", 53.92, eps=0.05)
    s.require_ratio("D", "B", "D", "A", 1.0249, eps=0.0002)
    s.require_ratio("B", "C", "D", "A", 2.4635, eps=0.0002)
    s.require_eq("D", "L", "D", "G")
    s.require_eq("B", "G", "B", "C")
    s.report(
        "I.2",
        [
            ("A", "D", "B"),
            ("D", "A", "B"),
            ("A", "B", "D"),
            ("D", "B", "C"),
        ],
    )
    print(
        f"  DA={s.dist('D','A'):.3f} DB={s.dist('D','B'):.3f} AB={s.dist('A','B'):.3f} "
        f"BC={s.dist('B','C'):.3f} DL={s.dist('D','L'):.3f} DE={s.dist('D','E'):.3f} DF={s.dist('D','F'):.3f}"
    )
    f = Fig(
        "book1_prop2",
        "I.2 — copy BC to A as AL. DAB (plate ∠ADB = 53.92°); DA, DB produced;\n"
        "circles B, D. One scale: DA : DB : BC = 1 : 1.025 : 2.463.",
    )
    f.put("D", *s.at("D"), "above_left")
    f.put("A", *s.at("A"), "left")
    f.put("B", *s.at("B"), ("deg", 43.0))
    f.put("C", *s.at("C"), "above")
    f.put("L", *s.at("L"), ("deg", -117.0))
    f.put("G", *s.at("G"), ("deg", 23.0))
    f.put("E", *s.at("E"), ("deg", -98.0))
    f.put("F", *s.at("F"), ("deg", -10.0))
    f.circle("CGH", "B", "C")
    f.circle("GKL", "D", "G")
    f.on_circle("H", "B", "C", 140.0)
    f.on_circle("K", "D", "G", 169.0)
    f.chain("D", "A", "L", "E")
    f.chain("D", "B", "G", "F")
    f.join("A", "B")
    f.join("B", "C")
    f.dots("A", "B", "C", "D", "G", "L")
    rd = s.dist("D", "G")
    ex, ey = s.at("E")
    fx, _ = s.at("F")
    _, cy = s.at("C")
    f.clip(-rd - 24.0, ey - 56.0, fx + 56.0, max(cy, rd) + 48.0)
    return f


def book1_prop3() -> Fig:
    # Fitzpatrick p. 10, from the PDF line art. One scale: AD = 1.
    #   AB horizontal, AD at 132.14°, BC given line C horizontal above the
    #   left of the circle; C/AD ≈ 1.024, Cy/AD = 1.342, Cx0/AD = −0.247.
    r = 180.0
    ab = r / 0.456770
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.put("B", ab, 0.0)
    s.put("E", r, 0.0)
    s.polar("D", "A", r, 132.137)
    s.polar("F", "A", r, -56.215)
    c_y = r * 1.34180
    c_x0 = -r * 0.24716
    c_len = r * 1.02400
    s.put("Cleft", c_x0, c_y)
    s.put("Cright", c_x0 + c_len, c_y)
    s.put("C", c_x0 + c_len / 2.0, c_y)
    s.require_angle("D", "A", "E", 132.14, eps=0.05)
    s.require_eq("A", "D", "A", "E")
    s.require_eq("A", "D", "A", "F")
    s.require_ratio("A", "B", "A", "D", 2.1893, eps=0.001)
    s.require_ratio("Cleft", "Cright", "A", "D", 1.024, eps=0.001)
    f = Fig(
        "book1_prop3",
        "I.3 — Fitzpatrick plate (Elements p. 10): AB the greater; C a short\n"
        "given line above the left of the circle; AD = AE at A (∠DAE = 132.14°);\n"
        "circle DEF. One scale: AD : AB = 1 : 2.189.",
    )
    f.put("A", *s.at("A"), ("deg", -136.0))
    f.put("B", *s.at("B"), ("deg", -3.0))
    f.put("E", *s.at("E"), ("deg", 56.0))
    f.pin("Cleft", *s.at("Cleft"))
    f.pin("Cright", *s.at("Cright"))
    f.named_line("C", "Cleft", "Cright")
    f.circle("DEF", "A", "E")
    f.put("D", *s.at("D"), ("deg", 111.0))
    f.on_circle("F", "A", "E", -56.215)
    f.join("A", "D")
    f.chain("A", "E", "B")
    f.dots("A", "B", "D", "E")
    fx, fy = s.at("F")
    f.clip(min(-r, c_x0) - 40.0, fy - 52.0, ab + 52.0, c_y + 48.0)
    return f


def book1_prop4() -> Fig:
    # Fitzpatrick p. 10 English plate. One scale: BC = 1.
    #   A along BC = 0.705, h/BC = 0.864, CE/BC = 0.364.
    base = 100.0
    h = 86.365
    apex = 0.70456 * base
    gap = 0.36363 * base
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.put("C", base, 0.0)
    s.put("A", apex, h)
    shift = base + gap
    s.put("E", shift, 0.0)
    s.put("F", base + shift, 0.0)
    s.put("D", apex + shift, h)
    s.require_eq("A", "B", "D", "E")
    s.require_eq("A", "C", "D", "F")
    s.require_eq("B", "C", "E", "F")
    s.require_ratio("E", "C", "B", "C", 0.3636, eps=0.002)
    f = Fig(
        "book1_prop4",
        "I.4 — Fitzpatrick plate (Elements p. 10): two congruent scalene\n"
        "triangles on one baseline. A along BC = 0.705, h/BC = 0.864, CE/BC = 0.364.",
    )
    f.put("A", *s.at("A"), ("deg", 93.0))
    f.put("B", *s.at("B"), ("deg", -177.0))
    f.put("C", *s.at("C"), ("deg", -4.0))
    f.put("D", *s.at("D"), ("deg", 77.0))
    f.put("E", *s.at("E"), ("deg", 179.0))
    f.put("F", *s.at("F"), ("deg", -4.0))
    f.join("A", "B")
    f.join("B", "C")
    f.join("C", "A")
    f.join("D", "E")
    f.join("E", "F")
    f.join("F", "D")
    f.dots("A", "B", "C", "D", "E", "F")
    # PDF cubic under EF: sag / EF = 79.14 / 870.70 = 0.0909.
    f.bow("E", "F", -base * 0.0909)
    return f


def book1_prop5() -> Fig:
    # One unit: AB. Fitzpatrick p. 11, from the PDF line art:
    #   ∠BAC = 43.43°, BC horizontal, AB = AC.
    #   AB : BF : FD = 1 : 0.333 : 0.542  (same scale).
    unit = 100.0
    bac = 43.43
    af = (1.0 + 0.333) * unit
    ad = (1.0 + 0.333 + 0.542) * unit
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.polar("B", "A", unit, -90.0 - bac / 2.0)
    s.turn("C", "A", "B", bac, unit)  # AC = AB, ∠BAC copied
    s.ray("F", "A", "B", af)
    s.copy_length("G", "A", "C", "A", "F")  # AG = AF
    s.ray("D", "A", "B", ad)
    s.copy_length("E", "A", "C", "A", "D")  # AE = AD
    s.require_angle("B", "A", "C", bac)
    s.require_eq("A", "B", "A", "C")
    s.require_eq("A", "F", "A", "G")
    s.require_eq("A", "D", "A", "E")
    s.require_eq("B", "F", "C", "G")
    s.require_same_angle("B", "A", "C", "C", "A", "B")
    s.require_ratio("A", "F", "A", "B", 1.333)
    s.require_ratio("B", "F", "A", "B", 0.333)
    s.require_ratio("F", "D", "A", "B", 0.542)
    s.require_ratio("A", "D", "A", "B", 1.875)
    s.require_ratio("A", "G", "A", "C", 1.333)
    s.require_ratio("A", "E", "A", "C", 1.875)
    s.require_line_angle("B", "G", "C", "F", 39.4, eps=0.3)
    s.report(
        "I.5",
        [
            ("B", "A", "C"),
            ("A", "B", "C"),
            ("B", "C", "A"),
            ("C", "B", "F"),
            ("B", "C", "G"),
            ("B", "F", "C"),
            ("C", "G", "B"),
            ("B", "G", "C", "F"),
        ],
    )
    print(
        f"  AB={s.dist('A','B'):.3f} AF={s.dist('A','F'):.3f} AD={s.dist('A','D'):.3f} "
        f"BC={s.dist('B','C'):.3f} BF={s.dist('B','F'):.3f} FD={s.dist('F','D'):.3f}"
    )
    f = Fig(
        "book1_prop5",
        "I.5 — Fitzpatrick plate (Elements p. 11): isosceles ABC, AB = AC;\n"
        "equal sides produced to D and E; F on BD, AG = AF on AE; FC and GB.\n"
        "One scale: AB : BF : FD = 1 : 0.333 : 0.542; ∠BAC = 43.43°, ∠(BG,CF) = 39.4°.",
    )
    # Letters sit on the outward normal of the stroke they name.
    f.put("A", *s.at("A"), ("deg", 97.0))
    f.put("B", *s.at("B"), ("deg", 164.0))
    f.put("C", *s.at("C"), ("deg", 18.0))
    f.put("F", *s.at("F"), ("deg", 171.0))
    f.put("G", *s.at("G"), ("deg", 10.0))
    f.put("D", *s.at("D"), ("deg", -127.0))
    f.put("E", *s.at("E"), ("deg", -87.0))
    f.chain("A", "B", "F", "D")
    f.chain("A", "C", "G", "E")
    f.join("B", "C")
    f.join("F", "C")
    f.join("G", "B")
    f.dots("A", "B", "C", "D", "E", "F", "G")
    return f


def book1_prop6() -> Fig:
    # Fitzpatrick p. 12–13 English plate. Reductio on ABC:
    # BC horizontal, A on the midline, D on AB (AD/AB = 0.312), DC joined.
    base = 100.0
    h = 91.666
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.put("C", base, 0.0)
    s.put("A", base / 2.0, h)
    t = 0.31241
    ax, ay = s.at("A")
    bx, by = s.at("B")
    s.put("D", ax + t * (bx - ax), ay + t * (by - ay))
    s.require_eq("A", "B", "A", "C")
    s.require_same_angle("A", "B", "C", "A", "C", "B")
    s.require_ratio("A", "D", "A", "B", 0.3124, eps=0.001)
    f = Fig(
        "book1_prop6",
        "I.6 — Fitzpatrick plate (Elements p. 12–13): triangle ABC with\n"
        "∠ABC = ∠ACB; D on AB (AD/AB = 0.312), DC joined.",
    )
    f.put("A", *s.at("A"), ("deg", 87.0))
    f.put("B", *s.at("B"), ("deg", 179.0))
    f.put("C", *s.at("C"), ("deg", -3.0))
    f.put("D", *s.at("D"), ("deg", 140.0))
    f.chain("A", "D", "B")
    f.join("B", "C")
    f.join("C", "A")
    f.join("D", "C")
    f.dots("A", "B", "C", "D")
    return f


def book1_prop7() -> Fig:
    # Fitzpatrick p. 13 English plate. AB horizontal; C and D above,
    # C higher and left of D; AC, CB, AD, DB, CD.
    ab = 100.0
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.put("B", ab, 0.0)
    s.put("C", 0.63460 * ab, 0.71152 * ab)
    s.put("D", 0.86538 * ab, 0.59614 * ab)
    s.require_ratio("C", "A", "A", "B", 0.9534, eps=0.002)
    f = Fig(
        "book1_prop7",
        "I.7 — Fitzpatrick plate (Elements p. 13): AB the base; C and D\n"
        "above on the same side; AC, CB and AD, DB; CD joined.",
    )
    f.put("A", *s.at("A"), ("deg", 180.0))
    f.put("B", *s.at("B"), ("deg", 0.0))
    f.put("C", *s.at("C"), ("deg", 90.0))
    f.put("D", *s.at("D"), ("deg", 17.0))
    f.base("A", "B")
    f.join("A", "C")
    f.join("C", "B")
    f.join("A", "D")
    f.join("D", "B")
    f.join("C", "D")
    f.dots("A", "B", "C", "D")
    return f


def book1_prop8() -> Fig:
    # Fitzpatrick p. 14 English plate, from the PDF line art (scale BC = 100).
    # Bases tilt ~23°. DEF is ABC translated; G is the miss of BA, CA.
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.put("C", 92.0578, 39.0560)
    s.put("A", 36.2635, 142.2726)
    dx, dy = 127.9108, 9.3765
    ax, ay = s.at("A")
    bx, by = s.at("B")
    cx, cy = s.at("C")
    s.put("E", bx + dx, by + dy)
    s.put("F", cx + dx, cy + dy)
    s.put("D", ax + dx, ay + dy)
    s.put("G", 198.0560, 142.2726)
    s.require_eq("A", "B", "D", "E")
    s.require_eq("A", "C", "D", "F")
    s.require_eq("B", "C", "E", "F")
    s.require_same_angle("B", "A", "C", "E", "D", "F")
    f = Fig(
        "book1_prop8",
        "I.8 — Fitzpatrick plate (Elements p. 14): congruent ABC and DEF;\n"
        "G the miss of BA, CA onto ED, DF (EG, GF).",
    )
    f.put("A", *s.at("A"), ("deg", 95.0))
    f.put("B", *s.at("B"), ("deg", -177.0))
    f.put("C", *s.at("C"), ("deg", 1.0))
    f.put("D", *s.at("D"), ("deg", 96.0))
    f.put("E", *s.at("E"), ("deg", -170.0))
    f.put("F", *s.at("F"), ("deg", 12.0))
    f.put("G", *s.at("G"), ("deg", 88.0))
    f.join("A", "B")
    f.join("B", "C")
    f.join("C", "A")
    f.join("D", "E")
    f.join("E", "F")
    f.join("F", "D")
    f.join("E", "G")
    f.join("G", "F")
    f.dots("A", "B", "C", "D", "E", "F", "G")
    return f


def book1_prop9() -> Fig:
    # Fitzpatrick p. 15 English plate. ∠BAC = 43.61°; AD = AE;
    # equilateral DEF on the far side of DE; AF the bisector (vertical).
    ad = 100.0
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.polar("D", "A", ad, -111.803)
    s.polar("E", "A", ad, -68.200)
    s.polar("B", "A", 186.45, -111.803)
    s.polar("C", "A", 186.45, -68.200)
    s.turn("F", "D", "E", -60.0)  # away from A
    s.require_eq("A", "D", "A", "E")
    s.require_eq("D", "E", "D", "F")
    s.require_eq("D", "F", "E", "F")
    s.require_same_angle("D", "A", "F", "F", "A", "E")
    f = Fig(
        "book1_prop9",
        "I.9 — Fitzpatrick plate (Elements p. 15): ∠BAC; D on AB, E on AC,\n"
        "AD = AE; equilateral DEF; AF the bisector.",
    )
    f.put("A", *s.at("A"), ("deg", 88.0))
    f.put("B", *s.at("B"), ("deg", -98.0))
    f.put("C", *s.at("C"), ("deg", -76.0))
    f.put("D", *s.at("D"), ("deg", 162.0))
    f.put("E", *s.at("E"), ("deg", 17.0))
    f.put("F", *s.at("F"), ("deg", -96.0))
    f.chain("A", "D", "B")
    f.chain("A", "E", "C")
    f.join("D", "E")
    f.join("D", "F")
    f.join("E", "F")
    f.join("A", "F")
    f.dots("A", "B", "C", "D", "E", "F")
    return f


def book1_prop10() -> Fig:
    # Fitzpatrick p. 16 English plate. ABC on the given AB; CD bisects
    # ∠ACB and cuts AB in half. The plate is isosceles, not equilateral:
    # letter centers give height/AB = 0.741 (equilateral would be 0.866).
    ab = 100.0
    h = 0.7412 * ab
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.put("B", ab, 0.0)
    s.put("C", ab / 2.0, h)
    s.put("D", ab / 2.0, 0.0)
    s.require_eq("A", "C", "B", "C")
    s.require_eq("A", "D", "D", "B")
    s.require_same_angle("A", "C", "D", "D", "C", "B")
    s.require_angle("A", "D", "C", 90.0)
    s.require_ratio("A", "C", "A", "B", math.hypot(0.5, 0.7412), eps=0.001)
    f = Fig(
        "book1_prop10",
        "I.10 — Fitzpatrick plate (Elements p. 16): ABC on the given finite\n"
        "straight-line AB; CD bisects ∠ACB and cuts AB in half.\n"
        "One scale: isosceles, height/AB = 0.741.",
    )
    f.put("A", *s.at("A"), ("deg", 180.0))
    f.put("B", *s.at("B"), ("deg", 0.0))
    f.put("C", *s.at("C"), ("deg", 90.0))
    f.put("D", *s.at("D"), ("deg", -90.0))
    f.base("A", "B")
    f.join("A", "C")
    f.join("B", "C")
    f.join("C", "D")
    f.dots("A", "B", "C", "D")
    return f


def book1_prop11() -> Fig:
    # Fitzpatrick p. 16 English plate. AB the given line, C on it;
    # D on AC, CE = CD, equilateral FDE, FC the perpendicular.
    # Letter centers: AD:DC:CE:EB = 0.223 : 0.288 : 0.277 : 0.212.
    # Construction still wants CE = CD; take DC as the mean of the two
    # plate segments so DE sits as on the plate (DE/AB = 0.565).
    ab = 100.0
    ad = 0.2225 * ab
    de = 0.5651 * ab
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.put("B", ab, 0.0)
    s.put("D", ad, 0.0)
    s.put("C", ad + de / 2.0, 0.0)
    s.put("E", ad + de, 0.0)
    s.turn("F", "D", "E", 60.0)  # equilateral, above DE
    s.require_eq("D", "C", "C", "E")
    s.require_eq("D", "E", "D", "F")
    s.require_eq("D", "F", "E", "F")
    s.require_angle("D", "C", "F", 90.0)
    s.require_same_angle("D", "C", "F", "F", "C", "E")
    s.require_ratio("A", "D", "A", "B", 0.2225, eps=0.001)
    s.require_ratio("D", "E", "A", "B", 0.5651, eps=0.001)
    s.require_ratio("E", "B", "A", "B", 0.2124, eps=0.001)
    f = Fig(
        "book1_prop11",
        "I.11 — Fitzpatrick plate (Elements p. 16): AB the given line, C on it;\n"
        "D on AC, CE = CD; equilateral FDE; FC perpendicular to AB.\n"
        "One scale: AD : DE : EB = 0.223 : 0.565 : 0.212.",
    )
    # Fitzpatrick: A/B sit outside the ends, slightly above AB (D/C/E stay below).
    f.put_line_end("A", *s.at("A"), left=True)
    f.put_line_end("B", *s.at("B"), left=False)
    f.put("D", *s.at("D"), ("deg", -90.0))
    f.put("C", *s.at("C"), ("deg", -90.0))
    f.put("E", *s.at("E"), ("deg", -90.0))
    f.put("F", *s.at("F"), ("deg", 90.0))
    f.chain("A", "D", "C", "E", "B")
    f.join("D", "F")
    f.join("E", "F")
    f.join("F", "C")
    f.dots("A", "B", "C", "D", "E", "F")
    return f


def book1_prop12() -> Fig:
    # Fitzpatrick p. 17 English plate. AB the infinite line; C not on it;
    # D on the other side; circle EFG center C through D cuts AB at G and E;
    # H the midpoint of EG (foot of the perpendicular from C).
    # Letter centers on the plate (AB = 100): GE/AB = 0.457, CH/AB = 0.248,
    # D at x = 62.0, depth 6.7 (the small cap below GE).
    ab = 100.0
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.put("B", ab, 0.0)
    s.put("C", ab / 2.0, 0.248 * ab)
    s.put("D", 0.620 * ab, -0.067 * ab)
    r = s.dist("C", "D")
    s.polar("F", "C", r, 90.0)
    h = s.at("C")[1]
    half = math.sqrt(r * r - h * h)
    s.put("G", ab / 2.0 - half, 0.0)
    s.put("E", ab / 2.0 + half, 0.0)
    s.put("H", ab / 2.0, 0.0)
    s.require_eq("C", "D", "C", "E")
    s.require_eq("C", "E", "C", "G")
    s.require_eq("C", "G", "C", "F")
    s.require_eq("G", "H", "H", "E")
    s.require_angle("C", "H", "E", 90.0)
    s.require_angle("C", "H", "G", 90.0)
    s.require_line_angle("C", "H", "A", "B", 90.0)
    s.require_ratio("G", "E", "A", "B", 0.457, eps=0.004)
    f = Fig(
        "book1_prop12",
        "I.12 — Fitzpatrick plate (Elements p. 17): AB the infinite line;\n"
        "C not on it; D on the other side; circle EFG center C through D\n"
        "cuts AB at G and E; H the midpoint of EG; CH the perpendicular.\n"
        "One scale: C over the midpoint of AB, GE/AB = 0.457.",
    )
    # Fitzpatrick: A/B sit outside the ends, slightly above the infinite line.
    f.put_line_end("A", *s.at("A"), left=True)
    f.put_line_end("B", *s.at("B"), left=False)
    # Fitzpatrick: C on the vertical CH/CF, above the mark (same column as F).
    f.put("C", *s.at("C"), ("deg", 90.0))
    f.put("D", *s.at("D"), ("deg", -90.0))
    # G/E sit below AB, slightly outward so the glyph clears the chord.
    f.put("E", *s.at("E"), ("deg", -70.0))
    f.put("G", *s.at("G"), ("deg", -110.0))
    f.put("H", *s.at("H"), ("deg", -90.0))
    f.circle("EFG", "C", "D")
    f.on_circle("F", "C", "D", 90.0)
    f.chain("A", "G", "H", "E", "B")
    f.join("C", "G")
    f.join("C", "H")
    f.join("C", "E")
    f.dots("A", "B", "C", "D", "E", "G", "H")
    return f


def book1_prop13() -> Fig:
    # Fitzpatrick p. 18 English plate. CD horizontal, D left of B, C right;
    # AB stood on it at B, leaning toward C; BE up from B, taller than A.
    # E sits on the tip of BE. Ink: BD/BC = 0.624, AB/BE = 1.036,
    # ∠CBA = 64.01°. One scale: BC = 100.
    bc = 100.0
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.put("C", bc, 0.0)
    s.put("D", -0.6243 * bc, 0.0)
    s.polar("A", "B", 1.6137 * bc, 64.01)
    s.put("E", 0.0, 1.5574 * bc)
    s.require_angle("C", "B", "A", 64.01)
    s.require_angle("C", "B", "E", 90.0)
    s.require_angle("E", "B", "D", 90.0)
    s.require_ratio("B", "D", "B", "C", 0.6243, eps=0.001)
    s.require_ratio("A", "B", "B", "E", 1.0361, eps=0.001)
    f = Fig(
        "book1_prop13",
        "I.13 — Fitzpatrick plate (Elements p. 18): AB stood on CD, making\n"
        "∠CBA and ∠ABD; BE drawn from B at right-angles to CD.\n"
        "D left, C right; AB leans toward C; E on the tip of BE.\n"
        "One scale: BD/BC = 0.624, AB/BE = 1.036, ∠CBA = 64.01°.",
    )
    # Fitzpatrick: D below-left of the left end, C below-right of the right.
    f.put("D", *s.at("D"), ("deg", -135.0))
    f.put("B", *s.at("B"), ("deg", -90.0))
    f.put("C", *s.at("C"), ("deg", -45.0))
    f.put("A", *s.at("A"), s.beyond("B", "A"))
    f.put("E", *s.at("E"), s.beyond("B", "E"))
    f.chain("C", "B", "D")
    f.join("A", "B")
    f.join("B", "E")
    f.dots("A", "B", "C", "D", "E")
    return f


def book1_prop14() -> Fig:
    # Fitzpatrick p. 19 English plate. Ink tips (letters blanked): C,B,D one
    # base; A,E one height. E is *past* D, not the column of D. One scale BC = 100.
    # --measure 19: BD/BC = 1.1363, ∠ABC = 61.03°.
    bc = 100.0
    s = Sketch()
    s.put("C", -bc, 0.0)
    s.level("B", "C", 0.0)
    s.level("D", "C", 1.13625 * bc)
    s.put("A", -0.82263 * bc, 1.48587 * bc)
    s.level("E", "A", 1.19538 * bc)
    s.require_angle("C", "B", "A", 61.03, eps=0.05)
    s.require_ratio("B", "D", "B", "C", 1.13625, eps=0.001)
    s.require_ratio("A", "B", "B", "C", 1.6986, eps=0.002)
    s.require_ratio("B", "E", "B", "C", 1.9070, eps=0.002)
    s.letters(A=92.4, B=-88.9, C=-118.2, D=-63.0, E=89.0)
    f = Fig(
        "book1_prop14",
        "I.14 — Fitzpatrick plate (Elements p. 19): BC and BD on opposite sides\n"
        "of AB at B; C–B–D collinear (`level`); A and E share a height (`level`);\n"
        "BE the reductio, E past D (not the column of D). One scale: BD/BC = 1.136.",
        sketch=s,
    )
    f.put("C", *s.at("C"))
    f.level("B", "C", s.at("B")[0])
    f.level("D", "C", s.at("D")[0])
    f.put("A", *s.at("A"))
    f.level("E", "A", s.at("E")[0])
    f.chain("C", "B", "D")
    f.join("A", "B")
    f.join("B", "E")
    f.dots("A", "B", "C", "D", "E")
    return f



def book1_prop15() -> Fig:
    # Fitzpatrick p. 20 English plate. Ink tips: D–C one base; E = AB ∩ CD
    # (`meet`). B sits below-right, slightly past C — not the column of C.
    # One scale CD = 100. --measure 20: CE/CD = 0.4995, AE/CD = 0.672, EB/CD = 0.682.
    cd = 100.0
    s = Sketch()
    s.put("D", 0.0, 0.0)
    s.level("C", "D", cd)
    s.put("A", -0.01334 * cd, 0.43333 * cd)
    s.put("B", 1.02222 * cd, -0.44000 * cd)
    s.meet("E", "A", "B", "D", "C")
    s.require_angle("A", "E", "B", 180.0, eps=0.05)
    s.require_angle("D", "E", "C", 180.0, eps=0.05)
    s.require_same_angle("A", "E", "C", "D", "E", "B")
    s.require_same_angle("C", "E", "B", "A", "E", "D")
    s.require_ratio("C", "E", "C", "D", 0.4995, eps=0.003)
    s.require_ratio("A", "E", "C", "D", 0.6722, eps=0.003)
    s.require_ratio("E", "B", "C", "D", 0.6825, eps=0.003)
    s.letters(A=46.0, B=-92.4, C=-64.3, D=-116.9, E=65.9)
    f = Fig(
        "book1_prop15",
        "I.15 — Fitzpatrick plate (Elements p. 20): AB and CD cut one another\n"
        "at E (`meet`); D–C `level`; A above-left, B below-right slightly past C\n"
        "(not a letter-column). One scale: CE/CD = 0.500, AE/EB = 0.672/0.682.",
        sketch=s,
    )
    f.put("D", *s.at("D"))
    f.level("C", "D", s.at("C")[0])
    f.put("A", *s.at("A"))
    f.put("B", *s.at("B"))
    f.meet("E", "A", "B", "D", "C")
    f.chain("D", "E", "C")
    f.chain("A", "E", "B")
    f.dots("A", "B", "C", "D", "E")
    return f


def book1_prop16() -> Fig:
    # Fitzpatrick p. 21 English plate. Ink tips: B–C–D one base (`level`);
    # E midpoint of AC; F on BE produced with EF = BE; G on AC produced.
    # One scale BD = 100. Ink: BC/BD = 0.494, hA/BD = 0.521, CG/BD = 0.437.
    bd = 100.0
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.level("C", "B", 0.494377 * bd)
    s.level("D", "B", bd)
    s.put("A", 0.293684 * bd, 0.520953 * bd)
    s.ray("E", "A", "C", s.dist("A", "C") / 2.0)
    s.ray("F", "B", "E", 2.0 * s.dist("B", "E"))
    s.ray("G", "A", "C", s.dist("A", "C") + 0.4365 * bd)
    s.require_eq("A", "E", "E", "C")
    s.require_eq("B", "E", "E", "F")
    s.require_eq("A", "B", "F", "C")
    s.require_same_angle("B", "A", "E", "E", "C", "F")
    s.require_same_angle("A", "E", "B", "F", "E", "C")
    s.require_ratio("B", "C", "B", "D", 0.4944, eps=0.001)
    s.require_ratio("A", "B", "B", "D", 0.5980, eps=0.001)
    s.require_ratio("C", "G", "B", "D", 0.4365, eps=0.002)
    # E in ∠AEB (bisector ~162°). Ink 136.8° sits on AC.
    # C from the true vertex BD∩AG (Fitzpatrick: under the base, left of the
    # junction). Nearest-ink −145.9° is the FC stroke, not the meet.
    s.letters(A=85.0, B=163.9, C=-121.5, D=14.2, E=162.0, F=33.6, G=-8.5)
    f = Fig(
        "book1_prop16",
        "I.16 — Fitzpatrick plate (Elements p. 21): triangle ABC, BC produced to D\n"
        "(`level`); E midpoint of AC; BE produced to F with EF = BE; AC through to G.\n"
        "One scale: BC/BD = 0.494, height/BD = 0.521.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.level("C", "B", s.at("C")[0])
    f.level("D", "B", s.at("D")[0])
    f.put("A", *s.at("A"))
    f.ray("E", "A", "C", s.dist("A", "C") / 2.0)
    f.ray("F", "B", "E", 2.0 * s.dist("B", "E"))
    f.ray("G", "A", "C", s.dist("A", "G"))
    f.chain("B", "C", "D")
    f.chain("A", "E", "C", "G")
    f.chain("B", "E", "F")
    f.join("A", "B")
    f.join("F", "C")
    f.dots("A", "B", "C", "D", "E", "F", "G")
    return f


def book1_prop17() -> Fig:
    # Fitzpatrick p. 21 English plate (lower). ABC; BC produced to D (`level`).
    # One scale BC = 100. Ink: CD/BC = 0.376, Ax/BC = −0.357, Ay/BC = 1.007.
    bc = 100.0
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.level("C", "B", bc)
    s.level("D", "B", 1.375839 * bc)
    s.put("A", -0.356925 * bc, 1.006711 * bc)
    s.require_ratio("C", "D", "B", "C", 0.3758, eps=0.001)
    s.require_ratio("A", "B", "B", "C", 1.0681, eps=0.001)
    s.require_ratio("A", "C", "B", "C", 1.6896, eps=0.001)
    # Under BD, as on Fitzpatrick lower p. 21 (not above the base).
    s.letters(A=89.9, B=-118.3, C=-90.6, D=-63.3)
    f = Fig(
        "book1_prop17",
        "I.17 — Fitzpatrick plate (Elements p. 21): triangle ABC, BC produced to D\n"
        "(`level`). A sits left of B (not a letter-column). One scale: CD/BC = 0.376,\n"
        "height/BC = 1.007.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.level("C", "B", s.at("C")[0])
    f.level("D", "B", s.at("D")[0])
    f.put("A", *s.at("A"))
    f.chain("B", "C", "D")
    f.join("A", "B")
    f.join("A", "C")
    f.dots("A", "B", "C", "D")
    return f


def book1_prop18() -> Fig:
    # Fitzpatrick p. 22 English plate. ABC with AC > AB; D on AC with AD = AB;
    # BD joined. BC one base (`level`); D on AC (not a letter-row).
    # One scale BC = 100. Ink: AB/BC = 0.940, AC/BC = 1.698, AD = AB.
    bc = 100.0
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.level("C", "B", bc)
    s.put("A", -0.50000 * bc, 0.79632 * bc)
    s.ray("D", "A", "C", s.dist("A", "B"))
    s.require_eq("A", "D", "A", "B")
    s.require_ratio("A", "B", "B", "C", 0.9403, eps=0.001)
    s.require_ratio("A", "C", "B", "C", 1.6983, eps=0.001)
    s.letters(A=102.5, B=-136.1, C=-3.7, D=46.5)
    f = Fig(
        "book1_prop18",
        "I.18 — Fitzpatrick plate (Elements p. 22): triangle ABC, AC > AB; D on AC\n"
        "with AD = AB; BD joined. BC `level`. One scale: AB/BC = 0.940, AC/BC = 1.698.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.level("C", "B", s.at("C")[0])
    f.put("A", *s.at("A"))
    f.ray("D", "A", "C", s.dist("A", "D"))
    f.join("A", "B")
    f.chain("A", "D", "C")
    f.join("B", "C")
    f.join("B", "D")
    f.dots("A", "B", "C", "D")
    return f


def book1_prop19() -> Fig:
    # Fitzpatrick p. 23 English plate (upper). ABC only; ∠ABC > ∠BCA so AC > AB.
    # A and C share a letter-column (`plumb`). One scale AC = 100.
    # Ink: AB/AC = 0.485, BC/AC = 0.807.
    ac = 100.0
    s = Sketch()
    s.put("C", 0.0, 0.0)
    s.plumb("A", "C", ac)
    s.put("B", -0.38737 * ac, 0.70737 * ac)
    s.require_ratio("A", "B", "A", "C", 0.4855, eps=0.001)
    s.require_ratio("B", "C", "A", "C", 0.8065, eps=0.001)
    s.letters(A=94.1, B=159.4, C=-94.1)
    f = Fig(
        "book1_prop19",
        "I.19 — Fitzpatrick plate (Elements p. 23): triangle ABC, ∠ABC > ∠BCA.\n"
        "A sits on the column of C (`plumb`). One scale: AB/AC = 0.485, BC/AC = 0.807.",
        sketch=s,
    )
    f.put("C", *s.at("C"))
    f.plumb("A", "C", s.at("A")[1])
    f.put("B", *s.at("B"))
    f.join("A", "B")
    f.join("B", "C")
    f.join("A", "C")
    f.dots("A", "B", "C")
    return f


def book1_prop20() -> Fig:
    # Fitzpatrick p. 23 English plate (lower). ABC; BA produced through A to D
    # with AD = CA; DC joined. BC one base (`level`). One scale BC = 100.
    # Ink: AB/BC = 0.509, AC/BC = 0.857.
    bc = 100.0
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.level("C", "B", bc)
    s.put("A", 0.26221 * bc, 0.43619 * bc)
    s.polar("D", "A", s.dist("A", "C"), s.heading("B", "A"))
    s.require_eq("A", "D", "A", "C")
    s.require_ratio("A", "B", "B", "C", 0.5089, eps=0.001)
    s.require_ratio("A", "C", "B", "C", 0.8571, eps=0.001)
    s.letters(A=137.4, B=-136.3, C=-41.3, D=70.1)
    f = Fig(
        "book1_prop20",
        "I.20 — Fitzpatrick plate (Elements p. 23): triangle ABC, BA produced to D\n"
        "with AD = CA; DC joined. BC `level`. One scale: AB/BC = 0.509, AC/BC = 0.857.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.level("C", "B", s.at("C")[0])
    f.put("A", *s.at("A"))
    f.polar("D", "A", s.dist("A", "D"), s.heading("B", "A"))
    f.chain("B", "A", "D")
    f.join("A", "C")
    f.join("B", "C")
    f.join("D", "C")
    f.dots("A", "B", "C", "D")
    return f


def book1_prop21() -> Fig:
    # Fitzpatrick p. 24 English plate. Triangle ABC; internal BD, DC from the
    # ends of BC; BD produced through D to E on AC. BC one base (`level`).
    # One scale BC = 100. Ink: AB/BC = 0.600, AC/BC = 0.894, BD/BC = 0.464,
    # DC/BC = 0.760. E is the meet of BD and AC.
    bc = 100.0
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.level("C", "B", bc)
    s.put("A", 0.2806 * bc, 0.5306 * bc)
    s.put("D", 0.3192 * bc, 0.3370 * bc)
    s.meet("E", "B", "D", "A", "C")
    s.require_ratio("A", "B", "B", "C", 0.6003, eps=0.001)
    s.require_ratio("A", "C", "B", "C", 0.8945, eps=0.001)
    s.require_ratio("B", "D", "B", "C", 0.4641, eps=0.001)
    s.require_ratio("D", "C", "B", "C", 0.7601, eps=0.001)
    s.letters(A=84.6, B=-169.3, C=-13.5, D=136.8, E=43.5)
    f = Fig(
        "book1_prop21",
        "I.21 — Fitzpatrick plate (Elements p. 24): triangle ABC; internal BD, DC\n"
        "from the ends of BC; BD produced to E on AC. BC `level`. One scale:\n"
        "AB/BC = 0.600, AC/BC = 0.894, BD/BC = 0.464, DC/BC = 0.760.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.level("C", "B", s.at("C")[0])
    f.put("A", *s.at("A"))
    f.put("D", *s.at("D"))
    f.meet("E", "B", "D", "A", "C")
    f.join("A", "B")
    f.chain("A", "E", "C")
    f.chain("B", "D", "E")
    f.join("B", "C")
    f.join("D", "C")
    f.dots("A", "B", "C", "D", "E")
    return f


def book1_prop22() -> Fig:
    # Fitzpatrick p. 25 English plate, ink tips at 200 dpi, D at the origin, y up.
    # Given strokes left-aligned, 94 px left of D, pitch 27, 197 px down to DE:
    # A 192, B 160, C 102. The letters stand at the left end of each stroke.
    # The construction is not a compass copy of those strokes. Circle DKL is
    # center F radius FD. Circle KLH is center G0 (6 px below the spoken G)
    # radius 105; H is its right-hand cut of DE, not the radius. K is the upper
    # crossing. DFGHE `level`.
    s = Sketch()
    s.put("D", 0.0, 0.0)
    s.level("F", "D", 194.15)
    s.level("G", "D", 362.0)
    s.level("H", "D", 467.03)
    s.level("E", "D", 529.0)
    s.put("G0", 362.0, -6.0)
    r_dkl, r_klh = 194.18, 105.2
    s.put("K", 361.08, 99.20)
    s.put("L", 353.57, -110.86)
    assert abs(s.dist("F", "K") - s.dist("F", "D")) < 0.05
    assert abs(s.dist("G0", "K") - r_klh) < 0.5
    assert abs(s.dist("G0", "L") - r_klh) < 0.05
    assert abs(s.dist("G0", "H") - r_klh) < 0.05
    rule_x, pitch, gap = -94.0, 27.0, 197.0
    top = gap + 2 * pitch
    s.put("A0", rule_x, top)
    s.level("A1", "A0", rule_x + 192.0)
    s.put("B0", rule_x, top - pitch)
    s.level("B1", "B0", rule_x + 160.0)
    s.put("C0", rule_x, top - 2 * pitch)
    s.level("C1", "C0", rule_x + 102.0)
    s.letters(D=150.8, F=-100.1, G=-90.1, H=-140.0, E=4.5, K=72.5)
    f = Fig(
        "book1_prop22",
        "I.22 — Fitzpatrick plate (Elements p. 25), measured at 200 dpi.\n"
        "Given strokes at the left, left-aligned, letters at the left end:\n"
        "A 192, B 160, C 102. Circle DKL center F through D; circle KLH center\n"
        "just below G, radius shorter than GH; K their upper crossing. DFGHE `level`.",
        sketch=s,
    )
    f.pin("A0", *s.at("A0"))
    f.pin("A1", *s.at("A1"))
    f.named_line("A", "A0", "A1", False)
    f.put_line_end("A", *s.at("A0"), left=True)
    f.pin("B0", *s.at("B0"))
    f.pin("B1", *s.at("B1"))
    f.named_line("B", "B0", "B1", False)
    f.put_line_end("B", *s.at("B0"), left=True)
    f.pin("C0", *s.at("C0"))
    f.pin("C1", *s.at("C1"))
    f.named_line("C", "C0", "C1", False)
    f.put_line_end("C", *s.at("C0"), left=True)
    f.put("D", *s.at("D"))
    f.level("F", "D", s.at("F")[0])
    f.pin("G0", *s.at("G0"))
    f.level("G", "D", s.at("G")[0])
    f.level("H", "D", s.at("H")[0])
    f.level("E", "D", s.at("E")[0])
    f.put("K", *s.at("K"))
    f.pin("L", *s.at("L"))
    f.chain("D", "F", "G", "H", "E")
    f.circle_r("DKL", "F", r_dkl)
    f.circle_r("KLH", "G0", r_klh)
    f.join("K", "F")
    f.join("K", "G")
    f.dots("D", "F", "G", "H", "E", "K")
    return f


def book1_prop23() -> Fig:
    # Fitzpatrick p. 26 English plate, 300 dpi, letters blanked. Vertices are
    # the ink nearest each glyph (y-up, A at the origin). AB is the given line
    # (`level`). Angle DCE stands above the left; triangle AFG on AB.
    # The plate produces CD and CE only a short way past the letter tips
    # (300 dpi: 27.9 pt past D, 29.7 pt past E — not out to the margin).
    # D0 and E0 are those ink ends (not letters). DE stops at the tips.
    # One scale AB = 100.
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.level("B", "A", 100.0)
    s.put("F", 28.93, 31.04)
    s.level("G", "A", 56.2)
    s.put("C", 4.71, 77.79)
    s.put("D", 48.75, 96.37)
    s.put("E", 62.73, 63.38)
    s.ray("D0", "C", "D", 66.71)
    s.ray("E0", "C", "E", 79.88)
    s.letters(A=172.0, B=70.6, C=-178.4, D=139.8, E=-133.8, F=87.6, G=44.8)
    f = Fig(
        "book1_prop23",
        "I.23 — Fitzpatrick plate (Elements p. 26), measured at 300 dpi.\n"
        "Given line AB; angle DCE above the left; triangle AFG on AB. AB `level`.\n"
        "CD and CE are produced a short way past the letter tips (D0, E0);\n"
        "DE stops at D and E. One scale AB = 100.",
        sketch=s,
    )
    f.put("A", *s.at("A"))
    f.level("B", "A", s.at("B")[0])
    f.put("F", *s.at("F"))
    f.level("G", "A", s.at("G")[0])
    f.put("C", *s.at("C"))
    f.put("D", *s.at("D"))
    f.put("E", *s.at("E"))
    f.pin("D0", *s.at("D0"))
    f.pin("E0", *s.at("E0"))
    f.base("A", "B")
    f.chain("C", "D", "D0")
    f.chain("C", "E", "E0")
    f.join("D", "E")
    f.join("A", "F")
    f.join("A", "G")
    f.join("F", "G")
    f.dots("A", "B", "C", "D", "E", "F", "G")
    return f


def book1_prop24() -> Fig:
    # Fitzpatrick p. 27 English plate. Stroke endpoints, y-up, C at the
    # origin. Triangle ABC on the left; DEF on the right with G inside,
    # EG and FG joined. DF is not collinear with DG, and the plate does
    # not make AB = DE. One scale BC = 100.
    s = Sketch()
    s.put("C", 0.0, 0.0)
    s.put("B", 54.70, 28.01)
    s.put("A", 6.12, 119.03)
    s.put("D", 54.01, 120.47)
    s.put("E", 130.11, 41.28)
    s.put("G", 92.68, 1.21)
    s.put("F", 118.14, 10.94)
    s.letters(A=90.7, B=-16.8, C=-90.8, D=104.2, E=-13.9, F=-60.9, G=-147.8)
    f = Fig(
        "book1_prop24",
        "I.24 — Fitzpatrick plate (Elements p. 27): triangles ABC and DEF;\n"
        "G inside DEF, EG and FG joined. DF is not on DG. One scale BC = 100.\n"
        "Plate ratios: AB/BC = 1.679, AC/BC = 1.939, DE/BC = 1.797.",
        sketch=s,
    )
    f.put("C", *s.at("C"))
    f.put("B", *s.at("B"))
    f.put("A", *s.at("A"))
    f.put("D", *s.at("D"))
    f.put("E", *s.at("E"))
    f.put("G", *s.at("G"))
    f.put("F", *s.at("F"))
    f.join("A", "B")
    f.join("B", "C")
    f.join("C", "A")
    f.join("D", "E")
    f.join("E", "F")
    f.join("F", "D")
    f.join("E", "G")
    f.join("F", "G")
    f.join("D", "G")
    f.dots("A", "B", "C", "D", "E", "F", "G")
    return f


def book1_prop25() -> Fig:
    # Fitzpatrick printed p. 28 (PDF p. 28). Two triangles only — the
    # reductio is not drawn. B and E share a column (`plumb`); EF `level`.
    # One scale EF = 100. Ink: BC/EF = 1.003, AB/DE = 1.011, AC/DF = 1.009.
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.put("A", 40.079, 64.047)
    s.put("C", 91.945, 40.079)
    s.put("D", 61.886, 6.287)
    s.plumb("E", "B", -35.560)
    s.level("F", "E", 100.0)
    s.letters(A=93.7, B=-154.0, C=-4.5, D=70.5, E=-135.6, F=-41.8)
    f = Fig(
        "book1_prop25",
        "I.25 — Fitzpatrick plate (Elements p. 28): triangles ABC and DEF.\n"
        "No construction marks. B and E `plumb`; EF `level`. One scale EF = 100.\n"
        "Plate: BC/EF = 1.003, AB/DE = 1.011, AC/DF = 1.009.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.put("A", *s.at("A"))
    f.put("C", *s.at("C"))
    f.put("D", *s.at("D"))
    f.plumb("E", "B", s.at("E")[1])
    f.level("F", "E", s.at("F")[0])
    f.join("A", "B")
    f.join("B", "C")
    f.join("C", "A")
    f.join("D", "E")
    f.join("E", "F")
    f.join("F", "D")
    f.dots("A", "B", "C", "D", "E", "F")
    return f


def book1_prop26() -> Fig:
    # Fitzpatrick printed p. 29 (PDF p. 29). Triangles ABC and DEF.
    # G on AB (BG = DE); GC joined. H on BC (BH = EF); AH joined.
    # B, C, H share a row (`level`). One scale BC = 100.
    # Ink: AB/DE = 1.000, AC/DF = 0.999, EF/BC = 1.003.
    s = Sketch()
    s.put("B", 0.0, 0.0)
    s.put("A", 15.072, 66.377)
    s.level("C", "B", 100.0)
    s.put("G", 8.116, 36.232)
    s.level("H", "B", 78.550)
    s.put("D", 141.594, 99.420)
    s.put("E", 126.376, 33.043)
    s.put("F", 226.666, 32.753)
    s.letters(
        A=92.4, B=-151.1, C=-23.5, D=90.3, E=-156.0, F=-25.8, G=134.5, H=-89.2
    )
    f = Fig(
        "book1_prop26",
        "I.26 — Fitzpatrick plate (Elements p. 29): triangles ABC and DEF.\n"
        "G on AB with GC joined; H on BC with AH joined. B, C, H `level`.\n"
        "One scale BC = 100. Plate: AB/DE = 1.000, AC/DF = 0.999, EF/BC = 1.003.",
        sketch=s,
    )
    f.put("B", *s.at("B"))
    f.put("A", *s.at("A"))
    f.level("C", "B", s.at("C")[0])
    f.put("G", *s.at("G"))
    f.level("H", "B", s.at("H")[0])
    f.put("D", *s.at("D"))
    f.put("E", *s.at("E"))
    f.put("F", *s.at("F"))
    f.chain("A", "G", "B")
    f.join("B", "C")
    f.join("C", "A")
    f.join("G", "C")
    f.chain("B", "H", "C")
    f.join("A", "H")
    f.join("D", "E")
    f.join("E", "F")
    f.join("F", "D")
    f.dots("A", "B", "C", "D", "E", "F", "G", "H")
    return f


def book1_prop27() -> Fig:
    # Fitzpatrick printed p. 30 (PDF p. 30). AB ∥ CD, transversal EF.
    # The reductio meets them, produced, at G (not drawn). A, E, B `level`;
    # C, F, D `level`. One scale AB = 100. Plate: CD/AB = 1.116, EF not on AB.
    s = Sketch()
    s.put("A", 0.0, 0.0)
    s.level("E", "A", 54.45)
    s.level("B", "A", 100.0)
    s.put("C", -3.35, -37.43)
    s.level("F", "C", 28.38)
    s.level("D", "C", 108.24)
    s.put("G", 158.33, -16.13)
    s.letters(A=91.0, B=90.0, C=-90.0, D=-90.0, E=90.0, F=-90.0, G=-8.0)
    f = Fig(
        "book1_prop27",
        "I.27 — Fitzpatrick plate (Elements p. 30): AB ∥ CD, transversal EF.\n"
        "G is where the lines would meet if produced (not drawn). A, E, B `level`;\n"
        "C, F, D `level`. One scale AB = 100. Plate: CD/AB = 1.116.",
        sketch=s,
    )
    f.put("A", *s.at("A"))
    f.level("E", "A", s.at("E")[0])
    f.level("B", "A", s.at("B")[0])
    f.put("C", *s.at("C"))
    f.level("F", "C", s.at("F")[0])
    f.level("D", "C", s.at("D")[0])
    f.put("G", *s.at("G"))
    f.chain("A", "E", "B")
    f.chain("C", "F", "D")
    f.join("E", "F")
    f.dots("A", "B", "C", "D", "E", "F", "G")
    return f


def book1_prop28() -> Fig:
    # Fitzpatrick printed p. 31 (PDF p. 31). AB ∥ CD. Transversal EF cuts AB
    # at G and CD at H. A, G, B `level`; C, H, D `level`; A, C `plumb`;
    # B, D `plumb`. One scale CD = 100. Plate: AB/CD = 1.000, GH/CD = 0.400.
    s = Sketch()
    s.put("C", 0.0, 0.0)
    s.plumb("A", "C", 39.96)
    s.level("B", "A", 100.0)
    s.plumb("D", "B", 0.0)
    s.put("E", 19.89, 60.61)
    s.put("F", 72.22, -24.93)
    s.meet("G", "A", "B", "E", "F")
    s.meet("H", "C", "D", "E", "F")
    s.letters(A=91.0, B=90.0, C=-90.0, D=-90.0, E=43.0, F=46.0, G=90.0, H=-90.0)
    f = Fig(
        "book1_prop28",
        "I.28 — Fitzpatrick plate (Elements p. 31): AB ∥ CD, transversal EF\n"
        "cutting AB at G and CD at H. A, G, B and C, H, D `level`; A plumb with C,\n"
        "B plumb with D. One scale CD = 100. Plate: AB/CD = 1.000.",
        sketch=s,
    )
    f.put("C", *s.at("C"))
    f.plumb("A", "C", s.at("A")[1])
    f.level("B", "A", s.at("B")[0])
    f.plumb("D", "B", s.at("D")[1])
    f.put("E", *s.at("E"))
    f.put("F", *s.at("F"))
    f.meet("G", "A", "B", "E", "F")
    f.meet("H", "C", "D", "E", "F")
    f.chain("A", "G", "B")
    f.chain("C", "H", "D")
    f.chain("E", "G", "H", "F")
    f.dots("A", "B", "C", "D", "E", "F", "G", "H")
    return f


PLATES = [
    book1_prop1,
    book1_prop2,
    book1_prop3,
    book1_prop4,
    book1_prop5,
    book1_prop6,
    book1_prop7,
    book1_prop8,
    book1_prop9,
    book1_prop10,
    book1_prop11,
    book1_prop12,
    book1_prop13,
    book1_prop14,
    book1_prop15,
    book1_prop16,
    book1_prop17,
    book1_prop18,
    book1_prop19,
    book1_prop20,
    book1_prop21,
    book1_prop22,
    book1_prop23,
    book1_prop24,
    book1_prop25,
    book1_prop26,
    book1_prop27,
    book1_prop28,
]


HEADER = """use super::diagram::Diagram;
use super::geom::{Place, V2};

// Generated by scripts/figure.py. Edit the Python plates, then:
//     python3 scripts/figure.py --write
// Ratios, angles, and `s.letters(...)` come from `--measure PAGE`
// (ink tips; `level` / `plumb` / `meet` only when the ink lines up).
"""


def rust_file() -> str:
    parts = [HEADER]
    for make in PLATES:
        parts.append(make().rust())
    return "\n".join(parts)


# --- Fitzpatrick plate measurement ------------------------------------------
#
# Capitals in the English column *name* the points. Vertices are the
# nearest ink to each glyph (letters blanked so they are not the stroke).
# Shared *ink* rows (`level`) and columns (`plumb`) snap; a letter on two
# named lines is their `meet`. One scale is the longest nearly-horizontal
# segment. Segment ratios are ink-tip to ink-tip.

WORD_RE = re.compile(
    r'<word xMin="([^"]+)" yMin="([^"]+)" xMax="([^"]+)" yMax="([^"]+)">([^<]*)</word>'
)


def _bbox_html(pdf: Path, page: int) -> str:
    with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
        out = Path(f.name)
    try:
        subprocess.run(
            ["pdftotext", "-f", str(page), "-l", str(page), "-bbox", str(pdf), str(out)],
            check=True,
            capture_output=True,
        )
        return out.read_text(errors="replace")
    finally:
        out.unlink(missing_ok=True)


def _figure_letters(html: str) -> list[tuple[str, float, float, float, float]]:
    """Isolated A–Z in the English column: (name, cx, cy, w, h), PDF y down.

    Drops the running “I.” of “Proposition N” (a very narrow glyph) and the
    body-text letters. The plate is the highest (smallest y) cluster with the
    most unique names — proof letters sit lower on the page.
    """
    marks: list[tuple[str, float, float, float, float]] = []
    for a, b, c, d, w in WORD_RE.findall(html):
        x0, y0, x1, y1 = map(float, (a, b, c, d))
        if x0 < ENG_X0_PT or not re.fullmatch(r"[A-Z]", w):
            continue
        ww, hh = x1 - x0, y1 - y0
        # Plate capitals are ~6 pt; body letters in the proof are ~9 pt.
        if hh < 5.5 or hh > 9.0 or ww < 4.0:
            continue
        marks.append((w, (x0 + x1) / 2.0, (y0 + y1) / 2.0, ww, hh))
    if not marks:
        return []

    def near(p, q) -> bool:
        return abs(p[1] - q[1]) < 200.0 and abs(p[2] - q[2]) < 180.0

    def spread(items):
        xs = [m[1] for m in items]
        ys = [m[2] for m in items]
        return (max(xs) - min(xs)) * (max(ys) - min(ys) + 1.0)

    def meant(items):
        return sum(m[2] for m in items) / len(items)

    best: list[tuple[str, float, float, float, float]] = []
    for seed in marks:
        cluster = [m for m in marks if near(seed, m)]
        names = {m[0] for m in cluster}
        if not best:
            best = cluster
            continue
        bn = {m[0] for m in best}
        if len(names) > len(bn):
            best = cluster
        elif len(names) == len(bn):
            if meant(cluster) < meant(best) - 8.0:
                best = cluster
            elif abs(meant(cluster) - meant(best)) <= 8.0 and spread(cluster) < spread(best):
                best = cluster
    cy = sum(m[2] for m in best) / len(best)
    by_name: dict[str, tuple[str, float, float, float, float]] = {}
    for m in best:
        prev = by_name.get(m[0])
        if prev is None or abs(m[2] - cy) < abs(prev[2] - cy):
            by_name[m[0]] = m
    return list(by_name.values())


def _read_pgm(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if data.startswith(b"P5"):
        header, raw = data.split(b"\x0a255\x0a", 1)
        dims = [ln for ln in header.split(b"\n") if ln and not ln.startswith(b"#") and ln != b"P5"]
        w, h = map(int, dims[-1].split())
        return w, h, raw[: w * h]
    if not data.startswith(b"P6"):
        raise SystemExit(f"not a PGM/PPM: {path}")
    header, raw = data.split(b"\x0a255\x0a", 1)
    dims = [ln for ln in header.split(b"\n") if ln and not ln.startswith(b"#") and ln != b"P6"]
    w, h = map(int, dims[-1].split())
    rgb = raw[: w * h * 3]
    gray = bytes((rgb[i] + rgb[i + 1] + rgb[i + 2]) // 3 for i in range(0, len(rgb), 3))
    return w, h, gray


def _raster_page(pdf: Path, page: int, dpi: int) -> tuple[int, int, bytearray]:
    with tempfile.TemporaryDirectory() as td:
        prefix = str(Path(td) / "p")
        subprocess.run(
            ["pdftoppm", "-f", str(page), "-l", str(page), "-gray", "-r", str(dpi), str(pdf), prefix],
            check=True,
            capture_output=True,
        )
        files = sorted(Path(td).glob("*.pgm")) or sorted(Path(td).glob("*.ppm"))
        if not files:
            raise SystemExit("pdftoppm wrote no image")
        w, h, raw = _read_pgm(files[0])
        return w, h, bytearray(raw)


def _blank_letters(
    gray: bytearray,
    w: int,
    h: int,
    letters: list[tuple[str, float, float, float, float]],
    sx: float,
    sy: float,
) -> None:
    """Paint each capital white so the glyph is not the stroke."""
    for _n, cx, cy, ww, hh in letters:
        pad = 1.4
        x0 = max(0, int((cx - ww / 2 - pad) * sx))
        x1 = min(w - 1, int((cx + ww / 2 + pad) * sx))
        y0 = max(0, int((cy - hh / 2 - pad) * sy))
        y1 = min(h - 1, int((cy + hh / 2 + pad) * sy))
        for y in range(y0, y1 + 1):
            i = y * w
            for x in range(x0, x1 + 1):
                gray[i + x] = 255


def _ink(gray: bytes, w: int, x: int, y: int) -> bool:
    return gray[y * w + x] < 110


def _nearest_ink(
    gray: bytes, w: int, h: int, cx: float, cy: float, rmax: float
) -> tuple[float, float] | None:
    """First ink ring around a blanked letter, then the dark centroid there."""
    x0, y0 = int(round(cx)), int(round(cy))
    rmax_i = max(2, int(rmax))
    for r in range(1, rmax_i + 1):
        sx = sy = n = 0.0
        for dy in range(-r, r + 1):
            for dx in range(-r, r + 1):
                if max(abs(dx), abs(dy)) != r:
                    continue
                x, y = x0 + dx, y0 + dy
                if 0 <= x < w and 0 <= y < h and _ink(gray, w, x, y):
                    sx += x
                    sy += y
                    n += 1
        if n:
            return sx / n, sy / n
    return None


def _snap_axes(pts: dict[str, tuple[float, float]], tol: float = 1.6):
    """Union-find snap of x (plumb) and y (level) among *ink* vertices."""
    names = list(pts)
    parent = {n: n for n in names}

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[rb] = ra

    for axis in (0, 1):
        parent = {n: n for n in names}
        for i, a in enumerate(names):
            for b in names[i + 1 :]:
                if abs(pts[a][axis] - pts[b][axis]) <= tol:
                    union(a, b)
        groups: dict[str, list[str]] = {}
        for n in names:
            groups.setdefault(find(n), []).append(n)
        for members in groups.values():
            if len(members) < 2:
                continue
            mean = sum(pts[n][axis] for n in members) / len(members)
            kind = "level" if axis == 1 else "plumb"
            print(f"  {kind}: {', '.join(sorted(members))}")
            for n in members:
                x, y = pts[n]
                pts[n] = (mean, y) if axis == 0 else (x, mean)
    return pts


def _seg_dist(p, a, b) -> float:
    ax, ay = a
    bx, by = b
    px, py = p
    abx, aby = bx - ax, by - ay
    l2 = abx * abx + aby * aby
    if l2 < 1e-12:
        return math.hypot(px - ax, py - ay)
    t = max(0.0, min(1.0, ((px - ax) * abx + (py - ay) * aby) / l2))
    return math.hypot(px - ax - t * abx, py - ay - t * aby)


def _cut(a, b, c, d):
    ax, ay = a
    bx, by = b
    cx, cy = c
    dx, dy = d
    den = (ax - bx) * (cy - dy) - (ay - by) * (cx - dx)
    if abs(den) < 1e-9:
        return None
    t = ((ax - cx) * (cy - dy) - (ay - cy) * (cx - dx)) / den
    return (ax + t * (bx - ax), ay + t * (by - ay))


def _meet_interiors(pts: dict[str, tuple[float, float]], tol: float = 8.0):
    """A letter close to two segments between *other* letters is their meet."""
    names = list(pts)
    segs = [(a, b) for i, a in enumerate(names) for b in names[i + 1 :]]
    for n in names:
        p = pts[n]
        close = [(a, b) for a, b in segs if n not in (a, b) and _seg_dist(p, pts[a], pts[b]) <= tol]
        # unique undirected lines
        uniq = []
        for a, b in close:
            if all({a, b} != {c, d} for c, d in uniq):
                uniq.append((a, b))
        if len(uniq) < 2:
            continue
        hit = _cut(pts[uniq[0][0]], pts[uniq[0][1]], pts[uniq[1][0]], pts[uniq[1][1]])
        if hit is None:
            continue
        print(f"  meet {n}: {uniq[0][0]}{uniq[0][1]} × {uniq[1][0]}{uniq[1][1]}")
        pts[n] = hit
    return pts


def measure_page(pdf: Path, page: int, dpi: int = 200) -> None:
    html = _bbox_html(pdf, page)
    letters = _figure_letters(html)
    if len(letters) < 2:
        raise SystemExit(f"no English-plate letters on page {page}")
    w, h, gray = _raster_page(pdf, page, dpi)
    sx = w / 612.0
    sy = h / 792.0
    _blank_letters(gray, w, h, letters, sx, sy)
    # Ink vertices in PDF pt (y down). Search ~3 em from each glyph.
    pts: dict[str, tuple[float, float]] = {}
    for n, cx, cy, ww, hh in letters:
        hit = _nearest_ink(gray, w, h, cx * sx, cy * sy, rmax=max(ww, hh) * sx * 2.4)
        if hit is None:
            raise SystemExit(f"no ink near letter {n}")
        pts[n] = (hit[0] / sx, hit[1] / sy)
        print(f"  ink {n}: letter ({cx:.2f},{cy:.2f}) → tip ({pts[n][0]:.2f},{pts[n][1]:.2f})")
    print("letters:", " ".join(sorted(pts)))
    print("snaps:")
    # ~1.2 pt of collinearity on a 200 dpi plate.
    pts = _snap_axes(pts, tol=1.2)
    # y-up, origin at the left-most of the lowest letter-row
    min_y = max(p[1] for p in pts.values())  # PDF y down
    low = [n for n, p in pts.items() if p[1] >= min_y - 2.5]
    origin = min(low, key=lambda n: pts[n][0])
    ox, oy = pts[origin]
    plane = {n: (p[0] - ox, oy - p[1]) for n, p in pts.items()}
    plane = _meet_interiors(plane)

    names = sorted(plane)

    def dist(a, b):
        ax, ay = plane[a]
        bx, by = plane[b]
        return math.hypot(bx - ax, by - ay)

    def heading(a, b):
        ax, ay = plane[a]
        bx, by = plane[b]
        return math.degrees(math.atan2(by - ay, bx - ax))

    def angle(p, v, q):
        px, py = plane[p]
        vx, vy = plane[v]
        qx, qy = plane[q]
        ax, ay = px - vx, py - vy
        bx, by = qx - vx, qy - vy
        na, nb = math.hypot(ax, ay), math.hypot(bx, by)
        return math.degrees(math.acos(max(-1.0, min(1.0, (ax * bx + ay * by) / (na * nb)))))

    horiz = []
    for i, a in enumerate(names):
        for b in names[i + 1 :]:
            ang = abs(heading(a, b)) % 180.0
            if ang > 90:
                ang = 180 - ang
            if ang < 12:
                horiz.append((dist(a, b), a, b))
    horiz.sort(reverse=True)
    if horiz:
        scale_name = f"{horiz[0][1]}{horiz[0][2]}"
        scale = horiz[0][0]
    else:
        pairs = [(dist(a, b), a, b) for i, a in enumerate(names) for b in names[i + 1 :]]
        pairs.sort(reverse=True)
        scale_name = f"{pairs[0][1]}{pairs[0][2]}"
        scale = pairs[0][0]

    print(f"I.? page {page}  origin {origin}  scale {scale_name} = {scale:.3f} pt → 100")
    print("vertices (one-scale, y-up). Use level/plumb/corner/meet in the Sketch:")
    k = 100.0 / scale
    for n in names:
        x, y = plane[n]
        print(f"  {n}: ({x * k:.3f}, {y * k:.3f})")
    print("polar from nearest (r / scale, heading deg):")
    for n in names:
        if n == origin:
            continue
        parent = min((dist(n, o), o) for o in names if o != n)[1]
        print(
            f"  {n} from {parent}: r={dist(n, parent) * k:.3f}  heading={heading(parent, n):.2f}"
        )
    print("angles at junctions:")
    for v in names:
        others = [o for o in names if o != v]
        if len(others) < 2:
            continue
        for i, p in enumerate(others):
            for q in others[i + 1 :]:
                if dist(p, v) < 1 or dist(q, v) < 1:
                    continue
                print(f"  ∠{p}{v}{q} = {angle(p, v, q):.2f}°")
    print("ratios (over scale):")
    for i, a in enumerate(names):
        for b in names[i + 1 :]:
            print(f"  {a}{b}/{scale_name} = {dist(a, b) / scale:.4f}")
    print("letter from tip (y-up heading deg) — paste into the Sketch:")
    letter_xy = {n: (cx, cy) for n, cx, cy, _w, _h in letters}
    heads = []
    for n in names:
        cx, cy = letter_xy[n]
        lx, ly = cx - ox, oy - cy
        tx, ty = plane[n]
        deg = math.degrees(math.atan2(ly - ty, lx - tx))
        heads.append(f"{n}={deg:.1f}")
        print(f"  {n}: {deg:.1f}")
    print(f"  s.letters({', '.join(heads)})")
    print("Fig(..., sketch=s) then places every letter that way. python3 scripts/figure.py --write")



def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="write src/figure/book1.rs")
    parser.add_argument("--check", action="store_true", help="print measured angles")
    parser.add_argument(
        "--measure",
        type=int,
        metavar="PAGE",
        help="rasterize Fitzpatrick PAGE and print plate ratios",
    )
    parser.add_argument(
        "--pdf",
        type=Path,
        default=DEFAULT_PDF,
        help="Fitzpatrick Elements.pdf (default /tmp/euclid-pdf/Elements.pdf)",
    )
    args = parser.parse_args()
    if args.measure:
        measure_page(args.pdf, args.measure)
        return
    src = rust_file()
    if args.write:
        OUT.write_text(src)
        print(f"wrote {OUT}")
    elif not args.check:
        print(src)


if __name__ == "__main__":
    main()
