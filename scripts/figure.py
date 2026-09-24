#!/usr/bin/env python3
"""Author Euclidean plates as Python, emit src/figure/book1.rs.

    python3 scripts/figure.py --write
"""

from __future__ import annotations

import argparse
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "src" / "figure" / "book1.rs"


class Sketch:
    """Plane sketch, y-up, degrees from +x (same as `V2::polar`).

    Place points, then `require_angle` / `require_eq` against the plate.
    """

    def __init__(self):
        self.pts: dict[str, tuple[float, float]] = {}

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

    def beyond(self, a: str, b: str) -> tuple:
        """Letter past `b`, along `a` → `b`."""
        return ("deg", round(self.heading(a, b), 1))

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
}


def rust_place(place) -> str:
    if isinstance(place, tuple) and place[0] == "deg":
        return f"Place::Deg({place[1]})"
    return f"Place::{PLACES[place]}"


def f64(x: float) -> str:
    s = f"{x:.6g}"
    if "." not in s and "e" not in s and "E" not in s:
        s += ".0"
    return s


class Fig:
    def __init__(self, fn: str, doc: str):
        self.fn = fn
        self.doc = doc
        self.lines: list[str] = []

    def _add(self, s: str) -> None:
        self.lines.append(s)

    def put(self, name: str, x: float, y: float, place, r: float | None = None) -> None:
        if r is None:
            self._add(
                f'    d.put("{name}", V2::new({f64(x)}, {f64(y)}), {rust_place(place)});'
            )
        else:
            self._add(
                f'    d.put_r("{name}", V2::new({f64(x)}, {f64(y)}), {rust_place(place)}, {f64(r)});'
            )

    def pin(self, name: str, x: float, y: float) -> None:
        self._add(f'    d.pin("{name}", V2::new({f64(x)}, {f64(y)}));')

    def polar(self, name: str, origin: str, r: float, deg: float, place) -> None:
        self._add(
            f'    d.polar("{name}", "{origin}", {f64(r)}, {f64(deg)}, {rust_place(place)});'
        )

    def ray(self, name: str, origin: str, through: str, dist: float, place) -> None:
        self._add(
            f'    d.ray("{name}", "{origin}", "{through}", {f64(dist)}, {rust_place(place)});'
        )

    def join(self, a: str, b: str) -> None:
        self._add(f'    d.join("{a}", "{b}");')

    def chain(self, *names: str) -> None:
        inner = ", ".join(f'"{n}"' for n in names)
        self._add(f"    d.chain(&[{inner}]);")

    def circle(self, letters: str, center: str, through: str) -> None:
        self._add(f'    d.circle("{letters}", "{center}", "{through}");')

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

    def on_line(self, name: str, a: str, b: str, t: float, place) -> None:
        """Letter on segment `a`–`b` at fraction `t`."""
        self._add(
            f'    d.on_line("{name}", "{a}", "{b}", {f64(t)}, {rust_place(place)});'
        )

    def named_line(self, letters: str, a: str, b: str) -> None:
        self._add(f'    d.named_line("{letters}", "{a}", "{b}");')
        # Letter at the midpoint, above the stroke (I.3’s C).
        self.on_line(letters, a, b, 0.5, "above")

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


PLATES = [
    book1_prop1,
    book1_prop2,
    book1_prop3,
    book1_prop4,
    book1_prop5,
]


HEADER = """use super::diagram::Diagram;
use super::geom::{Place, V2};

// Generated by scripts/figure.py. Edit the Python plates, then:
//     python3 scripts/figure.py --write
"""


def rust_file() -> str:
    parts = [HEADER]
    for make in PLATES:
        parts.append(make().rust())
    return "\n".join(parts)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="write src/figure/book1.rs")
    parser.add_argument("--check", action="store_true", help="print measured angles")
    args = parser.parse_args()
    src = rust_file()
    if args.write:
        OUT.write_text(src)
        print(f"wrote {OUT}")
    elif not args.check:
        print(src)


if __name__ == "__main__":
    main()
