#!/usr/bin/env python3
"""YouTube thumbnails for Book I demos that do not yet have one.

1280×720 cream plate, same family as the I.37–I.40 cards: two header lines,
same size, then the figure the video draws. Geometry is the plate in
src/figure/book1.rs (Fitzpatrick ink, not a redraw). A wash is only the
closed figure the enunciation is about — the two equal triangles, the two
equal parallelograms, the constructed triangle — never a construction line
and never an angle the proposition does not fill. A proposition about an
angle or parallels gets no wash: a wash there would light the wrong region.
A proposition about a line has no interior to wash. I.42 is a construction:
the given triangle and the parallelogram equal to it are the two regions, and
the two equal angles take the rose pie. Yellow sits on the gold wash, so it
does not read as a corner of the parallelogram. I.2 and I.3 light the
equal lines the enunciation names, and leave the construction ink at the
same weight as I.1. Dimming that ink to push the subject made the plate
look unfinished next to the first card. The wash gold is too close to the
cream to read as a line, so those lines take a brick that sits with the
navy type. I.5 keeps the whole plate — F, G, FC, GB and the rest — and
adds a highlight only. The subject is the angles, not the triangle, so the
mark is a pie in each corner the enunciation names: ∠ABC and ∠ACB at the
base, ∠CBD and ∠BCE under it. The two equal pairs are two marks — yellow
and larger at the base, rose and smaller under it — so a pair reads as one
set and the other pair as the other. Both hues sit with the plate: the
yellow is the angle highlight, the rose is the brick of the line cards
lifted so it still reads as a wash. A pie fills the angle; it does not
replace a stroke or a letter. Labels stay navy, the color every other card
uses.
"""

from __future__ import annotations

import math
import re
import sys
from pathlib import Path

import cairo

ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / "src" / "figure" / "book1.rs"
DOCS = ROOT / "docs"

W, H = 1280, 720
CREAM = (255, 249, 239)
NAVY = (29, 53, 80)
GOLD = (212, 184, 122)
# Line subject. The wash gold disappears on cream; a brick reads against
# both the cream and the navy letters, and is not a second ink.
BRICK = (154, 62, 46)
INK = (28, 32, 38)
WASH = (232, 214, 176)
# Angle subject. A filled corner, not an arc. Yellow reads as a highlight
# on the plate; the wash gold is too close to the cream to mark an angle.
# The second set is a coral lifted from the brick. A cool accent, or the
# brick left dark, fights the cream.
PIE = (234, 188, 64)
PIE_ALT = (216, 122, 100)
LETTER_R = 0.82  # plate::LETTER_R, em of the letter type

# Subject regions, vertex order around the boundary. Two regions: warm, then gold,
# the same pair as the I.37–I.40 cards. One region: the warm wash alone.
WASHES: dict[int, list[tuple[str, ...]]] = {
    1: [("A", "B", "C")],
    4: [("A", "B", "C"), ("D", "E", "F")],
    6: [("A", "B", "C")],
    7: [("A", "C", "B"), ("A", "D", "B")],
    8: [("A", "B", "C"), ("D", "E", "F")],
    16: [("A", "B", "C")],
    17: [("A", "B", "C")],
    18: [("A", "B", "C")],
    19: [("A", "B", "C")],
    20: [("A", "B", "C")],
    21: [("A", "B", "C")],
    22: [("K", "F", "G")],
    24: [("A", "B", "C"), ("D", "E", "F")],
    25: [("A", "B", "C"), ("D", "E", "F")],
    26: [("A", "B", "C"), ("D", "E", "F")],
    32: [("A", "B", "C")],
    33: [("B", "A", "C", "D")],
    34: [("A", "B", "D", "C")],
    35: [("A", "B", "C", "D"), ("E", "B", "C", "F")],
    36: [("A", "B", "C", "D"), ("E", "F", "G", "H")],
    # Parallelogram ABCD and triangle EBC on the same base. The triangle
    # wash covers the overlap, so the parallelogram's uncovered half stays
    # the warm region and EBC reads as the second figure.
    41: [("A", "B", "C", "D"), ("E", "B", "C")],
    # Triangle ABC and the parallelogram FECG equal to it, in the given
    # angle. FECG is drawn after ABC, so the gold covers the shared half
    # and the warm region is the triangle's uncovered half. The angles
    # are pies (ANGLES), not a third wash.
    42: [("A", "B", "C"), ("F", "E", "C", "G")],
}

# Lines the card lights. A produced ray is not the subject: only the named piece.
LINES: dict[int, list[tuple[str, str]]] = {
    # AL placed at A, equal to the given BC.
    2: [("A", "L"), ("B", "C")],
    # AE cut from the greater AB, equal to the lesser C.
    3: [("A", "E"), ("Cleft", "Cright")],
}

# Equal angles share a mark: (arm, vertex, arm), grouped by equality.
# I.5: ∠ABC = ∠ACB at the base, then ∠CBD = ∠BCE under it. Each group is
# one set: same color, same radius. The next group is the other set.
ANGLES: dict[int, list[list[tuple[str, str, str]]]] = {
    5: [
        [("A", "B", "C"), ("A", "C", "B")],
        [("C", "B", "D"), ("B", "C", "E")],
    ],
    # ∠D = ∠FEC. One set, so one mark. Arm ends of D are not lettered.
    42: [[("Dleft", "D", "Dright"), ("F", "E", "C")]],
}

# Per equality group, in ANGLES order: fill, then radius as a fraction of
# the shorter arm. Same within a pair; different between pairs, so the two
# sets stay distinct at thumbnail size. The larger pie is drawn first.
ANGLE_MARK: dict[int, list[tuple[tuple[int, int, int], float]]] = {
    5: [(PIE, 0.42), (PIE_ALT, 0.22)],
    # Rose, not yellow: the pie sits on the gold parallelogram, and yellow
    # reads as more of that wash. One plate radius for the pair — a fraction
    # of each arm would draw the longer angle larger. Short of the letter
    # inside ∠D.
    42: [(PIE_ALT, 14.0)],
}

PLACE_DEG = {
    "Right": 0.0,
    "AboveRight": 45.0,
    "Above": 90.0,
    "AboveLeft": 135.0,
    "Left": 180.0,
    "BelowLeft": -135.0,
    "Below": -90.0,
    "BelowRight": -45.0,
    "LineLeft": 135.0,
    "LineRight": 45.0,
}


class V2:
    def __init__(self, x: float, y: float):
        self.x = float(x)
        self.y = float(y)

    def add(self, o: V2) -> V2:
        return V2(self.x + o.x, self.y + o.y)

    def sub(self, o: V2) -> V2:
        return V2(self.x - o.x, self.y - o.y)

    def mul(self, s: float) -> V2:
        return V2(self.x * s, self.y * s)

    def dist(self, o: V2) -> float:
        return math.hypot(self.x - o.x, self.y - o.y)

    def polar(self, r: float, deg: float) -> V2:
        a = math.radians(deg)
        return V2(self.x + r * math.cos(a), self.y + r * math.sin(a))

    def toward(self, o: V2, dist: float) -> V2:
        n = self.dist(o) or 1.0
        return V2(self.x + (o.x - self.x) / n * dist, self.y + (o.y - self.y) / n * dist)

    def xy(self) -> tuple[float, float]:
        return (self.x, self.y)


class Place:
    def __init__(self, deg: float):
        self.deg = deg


class Eval:
    def __init__(self, env: dict):
        self.env = env
        self.s = ""
        self.i = 0

    def eval(self, s: str):
        self.s = s.strip()
        self.i = 0
        v = self.expr()
        self.skip()
        if self.i < len(self.s):
            raise SystemExit(f"leftover {self.s[self.i:]!r} in {s!r}")
        return v

    def skip(self) -> None:
        while self.i < len(self.s) and self.s[self.i].isspace():
            self.i += 1

    def peek(self) -> str:
        self.skip()
        return self.s[self.i] if self.i < len(self.s) else ""

    def startswith(self, t: str) -> bool:
        self.skip()
        return self.s.startswith(t, self.i)

    def expr(self):
        v = self.term()
        while (ch := self.peek()) and ch in "+-":
            op = self.s[self.i]
            self.i += 1
            r = self.term()
            v = v + r if op == "+" else v - r
        return v

    def term(self):
        v = self.unary()
        while (ch := self.peek()) and ch in "*/":
            op = self.s[self.i]
            self.i += 1
            r = self.unary()
            v = v * r if op == "*" else v / r
        return v

    def unary(self):
        if self.peek() == "-":
            self.i += 1
            return -self.unary()
        return self.postfix()

    def postfix(self):
        v = self.primary()
        while self.peek() == ".":
            self.i += 1
            name = self.ident()
            self.expect("(")
            args = self.args()
            self.expect(")")
            v = getattr(v, name)(*args)
        return v

    def primary(self):
        self.skip()
        if self.startswith("V2::new"):
            self.i += len("V2::new")
            self.expect("(")
            x, y = self.args()
            self.expect(")")
            return V2(x, y)
        if self.startswith("Place::Deg"):
            self.i += len("Place::Deg")
            self.expect("(")
            deg = self.expr()
            self.expect(")")
            return Place(float(deg))
        if self.startswith("Place::"):
            self.i += len("Place::")
            name = self.ident()
            if name not in PLACE_DEG:
                raise SystemExit(f"unknown Place::{name}")
            return Place(PLACE_DEG[name])
        ch = self.peek()
        if ch == "(":
            self.i += 1
            v = self.expr()
            if self.peek() == ",":
                pair = [v]
                while self.peek() == ",":
                    self.i += 1
                    if self.peek() == ")":
                        break
                    pair.append(self.expr())
                self.expect(")")
                return tuple(pair)
            self.expect(")")
            return v
        if ch == "&":
            self.i += 1
            return self.array()
        if ch == "[":
            return self.array()
        if ch == '"':
            return self.string()
        if ch.isdigit() or ch == ".":
            return self.number()
        name = self.ident()
        if name == "true":
            return True
        if name == "false":
            return False
        if name not in self.env:
            raise SystemExit(f"unknown name {name}")
        return self.env[name]

    def array(self):
        self.expect("[")
        out = []
        if self.peek() != "]":
            out.append(self.expr())
            while self.peek() == ",":
                self.i += 1
                if self.peek() == "]":
                    break
                out.append(self.expr())
        self.expect("]")
        return out

    def args(self):
        out = []
        if self.peek() not in ")]":
            out.append(self.expr())
            while self.peek() == ",":
                self.i += 1
                if self.peek() in ")]":
                    break
                out.append(self.expr())
        return out

    def string(self) -> str:
        self.expect('"')
        j = self.s.index('"', self.i)
        v = self.s[self.i : j]
        self.i = j + 1
        return v

    def number(self) -> float:
        self.skip()
        j = self.i
        while j < len(self.s) and (self.s[j].isdigit() or self.s[j] in "._"):
            j += 1
        if self.s.startswith("f64", j) or self.s.startswith("f32", j):
            j += 3
        raw = self.s[self.i : j].replace("_f64", "").replace("_f32", "").replace("_", "")
        self.i = j
        return float(raw)

    def ident(self) -> str:
        self.skip()
        j = self.i
        while j < len(self.s) and (self.s[j].isalnum() or self.s[j] == "_"):
            j += 1
        if j == self.i:
            raise SystemExit(f"expected ident at {self.s[self.i : self.i + 20]!r}")
        v = self.s[self.i : j]
        self.i = j
        return v

    def expect(self, t: str) -> None:
        self.skip()
        if not self.s.startswith(t, self.i):
            raise SystemExit(f"expected {t!r} at {self.s[self.i : self.i + 24]!r}")
        self.i += len(t)


class Scene:
    def __init__(self):
        self.pts: dict[str, V2] = {}
        self.labels: dict[str, tuple[V2, float, float]] = {}
        self.segs: list[tuple[str, str]] = []
        self.circs: list[tuple[str, float]] = []
        self.arc_list: list[tuple[V2, V2, V2]] = []
        self.dot_pts: list[V2] = []
        self.clip_box: tuple[V2, V2] | None = None
        # Arm ends of a given angle (I.42's Dleft, Dright). Not lettered.
        self.arms: dict[str, V2] = {}

    def at(self, name: str) -> V2:
        if name in self.pts:
            return self.pts[name]
        if name in self.arms:
            return self.arms[name]
        raise SystemExit(f"unknown point {name}")

    def pin(self, name: str, p: V2) -> None:
        self.pts[name] = p

    def _label(self, name: str, at: V2, place: Place, r_em: float = LETTER_R) -> None:
        self.labels[name] = (at, place.deg, r_em)

    def put(self, name: str, p: V2, place: Place) -> None:
        self.pin(name, p)
        self._label(name, p, place)

    def put_line_end(self, name: str, p: V2, left: bool) -> None:
        self.put(name, p, Place(135.0 if left else 45.0))

    def level(self, name: str, of: str, x: float, place: Place) -> None:
        self.put(name, V2(x, self.at(of).y), place)

    def plumb(self, name: str, of: str, y: float, place: Place) -> None:
        self.put(name, V2(self.at(of).x, y), place)

    def meet(self, name: str, a: str, b: str, c: str, d: str, place: Place) -> None:
        pa, pb, pc, pd = self.at(a), self.at(b), self.at(c), self.at(d)
        den = (pa.x - pb.x) * (pc.y - pd.y) - (pa.y - pb.y) * (pc.x - pd.x)
        if abs(den) < 1e-12:
            raise SystemExit(f"{a}{b} parallel to {c}{d}")
        t = ((pa.x - pc.x) * (pc.y - pd.y) - (pa.y - pc.y) * (pc.x - pd.x)) / den
        self.put(name, V2(pa.x + t * (pb.x - pa.x), pa.y + t * (pb.y - pa.y)), place)

    def label(self, name: str, place: Place) -> None:
        self._label(name, self.at(name), place)

    def label_r(self, name: str, place: Place, r_em: float) -> None:
        self._label(name, self.at(name), place, r_em)

    def label_at(self, name: str, at: V2, place: Place) -> None:
        self._label(name, at, place)

    def polar(self, name: str, origin: str, r: float, deg: float, place: Place) -> None:
        self.put(name, self.at(origin).polar(r, deg), place)

    def ray(self, name: str, origin: str, through: str, dist: float, place: Place) -> None:
        self.put(name, self.at(origin).toward(self.at(through), dist), place)

    def on_line(self, name: str, a: str, b: str, t: float, place: Place) -> None:
        pa, pb = self.at(a), self.at(b)
        self.pin(name, V2(pa.x + (pb.x - pa.x) * t, pa.y + (pb.y - pa.y) * t))
        self._label(name, self.at(name), place)

    def _circle_letter(self, name: str, center: str, through: str, deg: float, inside: bool) -> None:
        c = self.at(center)
        p = c.polar(c.dist(self.at(through)), deg)
        self.pin(name, p)
        self._label(name, p, Place(deg + 180.0 if inside else deg))

    def on_circle(self, name: str, center: str, through: str, deg: float) -> None:
        self._circle_letter(name, center, through, deg, False)

    def in_circle(self, name: str, center: str, through: str, deg: float) -> None:
        self._circle_letter(name, center, through, deg, True)

    def join(self, a: str, b: str) -> None:
        self.segs.append((a, b))

    def chain(self, names: list[str]) -> None:
        for a, b in zip(names, names[1:]):
            self.join(a, b)

    def base(self, a: str, b: str) -> None:
        self.join(a, b)

    def named_line(self, _letters: str, a: str, b: str) -> None:
        self.join(a, b)

    def circle(self, _letters: str, center: str, through: str) -> None:
        c = self.at(center)
        self.circs.append((center, c.dist(self.at(through))))

    def circle_r(self, _letters: str, center: str, r: float) -> None:
        self.circs.append((center, r))

    def bow(self, a: str, b: str, sag: float) -> None:
        pa, pb = self.at(a), self.at(b)
        mid = V2((pa.x + pb.x) / 2, (pa.y + pb.y) / 2)
        nx, ny = pa.y - pb.y, pb.x - pa.x
        nlen = math.hypot(nx, ny) or 1.0
        via = V2(mid.x + nx / nlen * sag, mid.y + ny / nlen * sag)
        self.arc_list.append((pa, pb, via))

    def dots(self, names: list[str]) -> None:
        for name in names:
            self.dot_pts.append(self.at(name))

    def dot_at(self, _name: str, p: V2) -> None:
        self.dot_pts.append(p)

    def clip(self, a: V2, b: V2) -> None:
        self.clip_box = (a, b)

    def given_angle(
        self,
        name: str,
        from_: str,
        vertex: tuple[float, float],
        left: tuple[float, float],
        right: tuple[float, float],
        glyph: V2,
    ) -> None:
        self.given_angle_arms(name, from_, vertex, [left, right], glyph)

    def given_angle_arms(
        self,
        name: str,
        from_: str,
        vertex: tuple[float, float],
        arms: list,
        glyph: V2,
    ) -> None:
        # Same construction as Diagram::given_angle_arms: vertex from a point,
        # then each arm, then the plate glyph. The strokes share one mark.
        v = self.at(from_).polar(vertex[0], vertex[1])
        self.pin(name, v)
        side = ["left", "right"]
        for i, arm in enumerate(arms):
            length, heading = arm
            end = v.polar(length, heading)
            suffix = side[i] if i < 2 else f"arm{i}"
            self.arms[f"{name}{suffix}"] = end
            self.segs.append((name, f"{name}{suffix}"))
        # The capital is the plate glyph, not an offset from the vertex.
        self.labels[name] = (glyph, 90.0, 0.0)

    def bounds(self) -> tuple[float, float, float, float]:
        if self.clip_box:
            a, b = self.clip_box
            return min(a.x, b.x), min(a.y, b.y), max(a.x, b.x), max(a.y, b.y)
        xs: list[float] = []
        ys: list[float] = []

        def add(p: V2) -> None:
            xs.append(p.x)
            ys.append(p.y)

        for a, b in self.segs:
            add(self.at(a))
            add(self.at(b))
        for name, r in self.circs:
            c = self.at(name)
            xs.extend((c.x - r, c.x + r))
            ys.extend((c.y - r, c.y + r))
        for a, b, via in self.arc_list:
            add(a)
            add(b)
            add(via)
        for p in self.dot_pts:
            add(p)
        for name in self.labels:
            if name in self.pts:
                add(self.pts[name])
        if not xs:
            return 0.0, 0.0, 1.0, 1.0
        return min(xs), min(ys), max(xs), max(ys)


def statements(src: str) -> list[str]:
    lines = []
    for line in src.splitlines():
        if "//" in line:
            line = line[: line.index("//")]
        lines.append(line)
    text = "\n".join(lines)
    out = []
    i, n = 0, len(text)
    while i < n:
        while i < n and text[i].isspace():
            i += 1
        if i >= n:
            break
        if text.startswith("let ", i) or text.startswith("d.", i):
            start = i
            depth = 0
            while i < n:
                c = text[i]
                if c in "({[":
                    depth += 1
                elif c in ")}]":
                    depth = max(0, depth - 1)
                elif c == ";" and depth == 0:
                    i += 1
                    break
                i += 1
            out.append(text[start:i].strip())
        else:
            i += 1
    return out


def load_scene(n: int, body: str) -> Scene:
    env: dict = {}
    ev = Eval(env)
    scene = Scene()
    for stmt in statements(body):
        if stmt.startswith("let mut "):
            continue
        if stmt.startswith("let "):
            rest = stmt[4:].strip().rstrip(";").strip()
            name, expr = rest.split("=", 1)
            name = name.strip()
            expr = expr.strip()
            if expr.startswith("{"):
                env[name] = eval_block(expr, ev, env)
            else:
                env[name] = ev.eval(expr)
            continue
        call = stmt[2:].strip().rstrip(";").strip()
        m = re.match(r"([A-Za-z_][A-Za-z0-9_]*)\((.*)\)\s*$", call, re.S)
        if not m:
            raise SystemExit(f"I.{n}: cannot parse {stmt}")
        method, arg_src = m.group(1), m.group(2)
        args = ev.eval(f"[{arg_src}]")
        getattr(scene, method)(*args)
    return scene


def eval_block(expr: str, ev: Eval, env: dict):
    inner = expr.strip()
    if not (inner.startswith("{") and inner.endswith("}")):
        raise SystemExit(f"not a block: {expr!r}")
    inner = inner[1:-1].strip()
    parts: list[str] = []
    i, n = 0, len(inner)
    while i < n:
        while i < n and inner[i].isspace():
            i += 1
        if i >= n:
            break
        start = i
        depth = 0
        while i < n:
            c = inner[i]
            if c in "({[":
                depth += 1
            elif c in ")}]":
                depth -= 1
            elif c == ";" and depth == 0:
                i += 1
                parts.append(inner[start:i].strip())
                break
            i += 1
        else:
            parts.append(inner[start:].strip())
            break
    value = None
    for part in parts:
        if part.startswith("let "):
            _bind(part if part.endswith(";") else part + ";", ev, env)
        else:
            value = ev.eval(part.rstrip(";").strip())
    if value is None:
        raise SystemExit(f"block has no value: {expr!r}")
    return value


def _bind(stmt: str, ev: Eval, env: dict) -> None:
    rest = stmt[4:].strip().rstrip(";").strip()
    name, expr = rest.split("=", 1)
    env[name.strip()] = ev.eval(expr.strip())


def functions() -> dict[int, str]:
    text = RUST.read_text()
    out = {}
    for chunk in re.split(r"\npub fn ", text)[1:]:
        m = re.match(r"book1_prop(\d+)", chunk)
        if not m:
            continue
        n = int(m.group(1))
        body = chunk.split("{", 1)[1]
        # Function body ends at the matching brace. The chunk already stops
        # at the next pub fn, and the last line is the closing brace.
        depth = 1
        i = 0
        while i < len(body) and depth:
            if body[i] == "{":
                depth += 1
            elif body[i] == "}":
                depth -= 1
            i += 1
        out[n] = body[: i - 1]
    return out


def rgb(c):
    return tuple(v / 255 for v in c)


def font(cr, size, bold=False, italic=False):
    cr.select_font_face(
        "Liberation Serif",
        cairo.FONT_SLANT_ITALIC if italic else cairo.FONT_SLANT_NORMAL,
        cairo.FONT_WEIGHT_BOLD if bold else cairo.FONT_WEIGHT_NORMAL,
    )
    cr.set_font_size(size)


def arc_points(a: V2, b: V2, via: V2, n: int = 48) -> list[tuple[float, float]]:
    ax, ay, bx, by, cx, cy = a.x, a.y, b.x, b.y, via.x, via.y
    d = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    if abs(d) < 1e-9:
        return [(ax, ay), (bx, by)]
    a2, b2, c2 = ax * ax + ay * ay, bx * bx + by * by, cx * cx + cy * cy
    ux = (a2 * (by - cy) + b2 * (cy - ay) + c2 * (ay - by)) / d
    uy = (a2 * (cx - bx) + b2 * (ax - cx) + c2 * (bx - ax)) / d
    r = math.hypot(ax - ux, ay - uy)

    def ang(x, y):
        return math.atan2(y - uy, x - ux) % math.tau

    aa, bb, vv = ang(ax, ay), ang(bx, by), ang(cx, cy)
    sweep = (bb - aa) % math.tau
    if (vv - aa) % math.tau > sweep:
        sweep -= math.tau
    return [
        (ux + r * math.cos(aa + sweep * i / n), uy + r * math.sin(aa + sweep * i / n))
        for i in range(n + 1)
    ]


def angle_wedge(scene: Scene, arm: str, vertex: str, other: str, radius: float) -> list[V2]:
    """Sector of ∠(arm, vertex, other), radius in plate units.

    The lesser turn is the angle the figure draws. A reflex would fill the
    outside and read as the triangle again.
    """
    v = scene.at(vertex)
    a = math.atan2((scene.at(arm).y - v.y), (scene.at(arm).x - v.x))
    b = math.atan2((scene.at(other).y - v.y), (scene.at(other).x - v.x))
    sweep = (b - a + math.pi) % math.tau - math.pi
    n = max(12, int(abs(sweep) / math.tau * 64))
    pts = [v]
    for i in range(n + 1):
        t = a + sweep * i / n
        pts.append(V2(v.x + radius * math.cos(t), v.y + radius * math.sin(t)))
    return pts


def prepare(n: int, scene: Scene) -> None:
    """Check the angle marks. The plate itself is not edited."""
    for group in ANGLES.get(n, []):
        for arm, vertex, other in group:
            for name in (arm, vertex, other):
                # Arm ends of a given angle live in scene.arms, not scene.pts.
                try:
                    scene.at(name)
                except SystemExit:
                    raise SystemExit(f"I.{n}: angle {arm}{vertex}{other} missing {name}") from None


def render(n: int, scene: Scene, out: Path) -> None:
    prepare(n, scene)
    for poly in WASHES.get(n, []):
        for name in poly:
            if name not in scene.pts:
                raise SystemExit(f"I.{n}: wash vertex {name} missing")

    surf = cairo.ImageSurface(cairo.FORMAT_ARGB32, W, H)
    cr = cairo.Context(surf)
    pat = cairo.LinearGradient(0, 0, W, H)
    pat.add_color_stop_rgb(0, *rgb(CREAM))
    pat.add_color_stop_rgb(1, 0.965, 0.925, 0.875)
    cr.set_source(pat)
    cr.paint()

    title = "Euclid's Elements"
    subtitle = f"Book 1 Proposition {n}"
    size = 48
    font(cr, size, bold=True)
    title_w = cr.text_extents(title).x_advance
    sub_w = cr.text_extents(subtitle).x_advance
    cr.set_source_rgb(*rgb(NAVY))
    cr.move_to((W - title_w) / 2, 132)
    cr.show_text(title)
    cr.move_to((W - sub_w) / 2, 188)
    cr.show_text(subtitle)

    letter = 32.0
    minx, miny, maxx, maxy = scene.bounds()
    slot_top, slot_bot = 216, 688
    slot_l, slot_r = 36, W - 36
    # Letters sit 0.82em off the mark. Reserve that, plus half a glyph,
    # so a capital on the rim is not clipped.
    for _ in range(8):
        pad = letter * 1.45
        avail_w = slot_r - slot_l - 2 * pad
        avail_h = slot_bot - slot_top - 2 * pad
        gw, gh = max(maxx - minx, 1.0), max(maxy - miny, 1.0)
        scale = min(avail_w / gw, avail_h / gh)
        ox = W / 2 - (minx + maxx) / 2 * scale
        oy = (slot_top + pad + slot_bot - pad) / 2 + (miny + maxy) / 2 * scale

        def pt_of(p: V2) -> tuple[float, float]:
            return ox + p.x * scale, oy - p.y * scale

        boxes = []
        overflow = False
        for _name, (anchor, deg, r_em) in scene.labels.items():
            x, y = pt_of(anchor)
            ang = math.radians(deg)
            x += math.cos(ang) * r_em * letter
            y += -math.sin(ang) * r_em * letter
            font(cr, letter, italic=True)
            ext = cr.text_extents(_name[:1] or "A")
            half_w = ext.width / 2 + 2
            half_h = ext.height / 2 + 2
            boxes.append((x - half_w, y - half_h, x + half_w, y + half_h))
            if x - half_w < 16 or x + half_w > W - 16 or y - half_h < slot_top - 8 or y + half_h > H - 16:
                overflow = True
        overlap = False
        for i, a in enumerate(boxes):
            for b in boxes[i + 1 :]:
                if a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]:
                    # A hair of overlap is the italic sidebearing, not a collision.
                    ix = min(a[2], b[2]) - max(a[0], b[0])
                    iy = min(a[3], b[3]) - max(a[1], b[1])
                    if ix > 3 and iy > 3:
                        overlap = True
        if not overflow and not overlap:
            break
        letter = max(18.0, letter * 0.9)
        if letter <= 18.0:
            break

    def pt(name: str) -> tuple[float, float]:
        return pt_of(scene.at(name))

    def poly(names: tuple[str, ...]) -> None:
        x, y = pt(names[0])
        cr.move_to(x, y)
        for name in names[1:]:
            cr.line_to(*pt(name))
        cr.close_path()

    regions = WASHES.get(n, [])
    for i, names in enumerate(regions):
        if len(regions) == 1:
            cr.set_source_rgba(*rgb(WASH), 0.85)
        elif i == 0:
            cr.set_source_rgba(*rgb(WASH), 0.85)
        else:
            cr.set_source_rgba(*rgb(GOLD), 0.28)
        poly(names)
        cr.fill()

    def angle_radius(arm: str, vertex: str, other: str, reach: float) -> float:
        vv = scene.at(vertex)
        span = min(vv.dist(scene.at(arm)), vv.dist(scene.at(other)))
        # A fraction of the shorter arm. At least 1 is a plate radius, shared
        # by every angle in the group so a longer arm does not draw a bigger pie.
        if reach >= 1.0:
            return min(reach, span * 0.92)
        return reach * span

    # Pies under the ink, so the plate's strokes and letters stay as they
    # are. Each equality group is one color and one radius; the next group
    # is the other set. Larger first, so a smaller pie is not covered.
    marks = ANGLE_MARK.get(n, [])
    groups = ANGLES.get(n, [])
    if len(marks) != len(groups):
        raise SystemExit(f"I.{n}: angle mark count {len(marks)} != groups {len(groups)}")
    order = sorted(range(len(groups)), key=lambda i: marks[i][1], reverse=True)
    for i in order:
        color, reach = marks[i]
        cr.set_source_rgba(*rgb(color), 0.86)
        for arm, vertex, other in groups[i]:
            pts = angle_wedge(scene, arm, vertex, other, angle_radius(arm, vertex, other, reach))
            cr.move_to(*pt_of(pts[0]))
            for p in pts[1:]:
                cr.line_to(*pt_of(p))
            cr.close_path()
            cr.fill()

    def clip_circles() -> None:
        if not scene.clip_box:
            return
        a, b = scene.clip_box
        x0, y0 = pt_of(a)
        x1, y1 = pt_of(b)
        cr.rectangle(min(x0, x1), min(y0, y1), abs(x1 - x0), abs(y1 - y0))
        cr.clip()

    cr.save()
    clip_circles()
    # show_text leaves the pen at the end of the subtitle. cairo_arc
    # joins that point to the arc, which drew a stroke from "Proposition"
    # into the circle (I.1, I.2, I.3, I.12, I.22).
    cr.new_path()
    lit_segs = LINES.get(n, [])
    for a, b in lit_segs:
        if a not in scene.pts or b not in scene.pts:
            raise SystemExit(f"I.{n}: line {a}{b} missing")
    lit_ink = {(a, b) for a, b in lit_segs} | {(b, a) for a, b in lit_segs}
    # Construction stays the same ink as I.1. Only the named lines are lit.
    cr.set_source_rgb(*rgb(INK))
    cr.set_line_width(3.2)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    for name, r in scene.circs:
        x, y = pt(name)
        cr.arc(x, y, r * scale, 0, math.tau)
        cr.stroke()
    for a, b, via in scene.arc_list:
        pts = arc_points(a, b, via)
        if not pts:
            continue
        cr.move_to(*pt_of(V2(*pts[0])))
        for x, y in pts[1:]:
            cr.line_to(*pt_of(V2(x, y)))
        cr.stroke()
    cr.restore()

    cr.set_source_rgb(*rgb(INK))
    cr.set_line_width(3.2)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    cr.set_line_join(cairo.LINE_JOIN_ROUND)
    for a, b in scene.segs:
        if (a, b) in lit_ink:
            continue
        cr.move_to(*pt(a))
        cr.line_to(*pt(b))
        cr.stroke()

    # Brick, not the wash gold: gold on cream reads as mustard and fights
    # the navy type. Labels stay navy, the color every other card uses.
    cr.set_source_rgb(*rgb(BRICK))
    cr.set_line_width(13)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    for a, b in lit_segs:
        cr.move_to(*pt(a))
        cr.line_to(*pt(b))
        cr.stroke()

    cr.new_path()
    for p in scene.dot_pts:
        x, y = pt_of(p)
        cr.set_source_rgb(*rgb(INK))
        cr.arc(x, y, 4.2, 0, math.tau)
        cr.fill()

    for name, (anchor, deg, r_em) in scene.labels.items():
        x, y = pt_of(anchor)
        ang = math.radians(deg)
        x += math.cos(ang) * r_em * letter
        y += -math.sin(ang) * r_em * letter
        font(cr, letter, italic=True)
        cr.set_source_rgb(*rgb(NAVY))
        ext = cr.text_extents(name)
        cr.move_to(x - (ext.x_bearing + ext.width / 2), y - (ext.y_bearing + ext.height / 2))
        cr.show_text(name)

    surf.write_to_png(out)


def main() -> None:
    fns = functions()
    missing = []
    wanted = [int(a) for a in sys.argv[1:] if a.isdigit()] or (list(range(1, 37)) + [41, 42])
    for n in wanted:
        out = DOCS / f"youtube-book1-proposition{n}.png"
        if out.exists() and "--force" not in sys.argv:
            continue
        missing.append(n)
    if not missing:
        print("no missing thumbnails")
        return
    for n in missing:
        scene = load_scene(n, fns[n])
        out = DOCS / f"youtube-book1-proposition{n}.png"
        render(n, scene, out)
        print(out)


if __name__ == "__main__":
    main()
