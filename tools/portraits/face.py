# Portrait anchors (PORTRAITFX1-T): writes and checks the face sidecar `<portrait>.face.json` beside a portrait PNG
# (the format of aegis_core::assets 4.1: eye boxes, glow anchors, line roots, the rim key, idle overlays, all in image
# px) and draws review pictures so the anchors can be read off and checked by eye.
#
#   face.py grid PNG OUT                   the portrait at 4x, a line every 8 image px, a label every 40
#   face.py set PNG [--eyes x,y,w,h ...] [--no-blink | --blink] [--lid #rrggbb] [--glow x,y,r ...]
#               [--lines x,y,deg,len ...] [--mouth x,y,w,h] [--face x,y,w,h] [--rim #rrggbb] [--border n]
#               [--idle effect[:#rrggbb[:intensity]] ...] [--fresh]
#                                          writes the sidecar beside the PNG (`size` from its width); keys not given
#                                          keep the existing sidecar's values (--fresh: start from a blank one);
#                                          a sidecar with problems is refused with the A11 text and nothing written
#   face.py show PNG OUT                   the anchors drawn over the 4x portrait
#   face.py check [ROOT ...]               the A11 rules over every sidecar under ROOT/portraits (ROOT: an asset
#                                          layer; default the game's and the kit's), one "path: problem" line each
#   face.py contact OUT [PNG ...]          every portrait (default: all of both layers) with its anchors on one sheet
#
# The rules mirror `parse_face` and `face_problems` (aegis_core/src/assets.rs) message for message; the fixtures in
# fixtures/face/ with their expected_check.txt are asserted by both this tool's test (test_face.py) and the engine's
# (aegis_core/tests/face_fixtures.rs), which keeps the two in step. FB-206 (2b): the overlay primitives have generic
# names; a sidecar with the old ones (`veins`, `eye_glow`, `ash`, `smoke`) still reads (PRIMITIVE_ALIASES, the key
# alias FACE_KEY_ALIASES), and `set` writes the new ones (`--veins` is an old spelling of `--lines`).
# Standard library and Pillow only, so the tool can be ported to Rust (decision "Art tools in Python, portable to Rust").
import argparse
import json
import math
import pathlib
import struct
import sys

from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parents[2]
LAYERS = [ROOT / "games" / "emberwake" / "assets", ROOT / "kits" / "rpg" / "assets"]

FACE_KEYS = ["size", "border", "eyes", "blink", "lid", "glow", "lines", "mouth", "face", "rim", "idle"]
FACE_KEY_ALIASES = [("veins", "lines")]
IDLE_KEYS = ["effect", "color", "intensity"]
IDLE_OVERLAYS = ["glow", "lines", "point_glow", "particles_rise", "drift"]
PRIMITIVE_ALIASES = [("veins", "lines"), ("eye_glow", "point_glow"), ("ash", "particles_rise"), ("smoke", "drift")]
FACE_SIZE_MIN, FACE_SIZE_MAX = 16, 1024
SCALE = 4

# Review colours (RGBA): eye boxes, glow anchors, line roots, the mouth, the inner rect, grid lines and labels.
EYE = (0, 255, 255, 255)
GLOW = (255, 0, 255, 255)
LINE = (255, 255, 0, 255)
MOUTH = (255, 128, 0, 255)
FACE = (120, 255, 120, 255)
INNER = (255, 255, 255, 160)
GRID = (255, 255, 255, 70)
GRID_MAJOR = (255, 255, 255, 150)
LABEL = (255, 255, 255, 255)
LABEL_BG = (0, 0, 0, 200)


# ---- the rules (aegis_core::assets parse_face and face_problems) --------------------------------------------------

def tdiv(a, b):
    """Rust's integer division (towards zero)."""
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def default_border(size):
    """round(size / 30), halves away from zero (Rust's f64::round)."""
    x = size / 30.0
    return int(math.floor(x + 0.5)) if x >= 0 else -int(math.floor(-x + 0.5))


def f32(x):
    """`x` rounded to an f32, as the engine stores an intensity."""
    return struct.unpack("f", struct.pack("f", x))[0]


def f32_text(x):
    """An f32 as Rust's Display writes it: the shortest digits that read back to the same f32, no exponent."""
    if x == int(x) and abs(x) < 1e16:
        return str(int(x))
    for p in range(1, 10):
        s = f"{x:.{p}g}"
        if f32(float(s)) == x:
            break
    if "e" in s:
        s = format(float(s), "f").rstrip("0").rstrip(".")
    return s


def whole(v):
    """A JSON number with no fraction as an int (None otherwise; true and false are not numbers)."""
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        return None
    if isinstance(v, float) and (not math.isfinite(v) or not v.is_integer()):
        return None
    return int(v) if abs(v) <= 2 ** 31 - 1 else None


def hex_colour(s):
    h = s[1:] if isinstance(s, str) and s.startswith("#") else None
    if h is None or len(h) != 6 or not all(c in "0123456789abcdefABCDEF" for c in h):
        return None
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def _colour_or_empty(o, key, label, p):
    if key not in o or o[key] == "":
        return None
    c = hex_colour(o[key])
    if c is None:
        p.append(f'{label} must be "" or a #rrggbb colour.')
    return c


def _tuples(o, key, n, shape, p):
    if key not in o:
        return []
    raw = o[key]
    if not isinstance(raw, list):
        p.append(f"{key} must be a list of {shape} lists of whole numbers.")
        return []
    out = []
    for item in raw:
        nums = [whole(v) for v in item] if isinstance(item, list) and len(item) == n else None
        if nums is None or None in nums:
            p.append(f"{key} must be a list of {shape} lists of whole numbers.")
            return out
        out.append(nums)
    return out


def _one_box(v, key, p):
    """[] or absent: None; else [x, y, w, h] of whole numbers (a problem otherwise)."""
    if key not in v or v[key] == []:
        return None
    m = v[key]
    nums = [whole(x) for x in m] if isinstance(m, list) and len(m) == 4 else None
    if nums is None or None in nums:
        p.append(f"{key} must be [] or [x, y, w, h] of whole numbers.")
        return None
    return nums


def canonical_primitive(name):
    """An old primitive name's new one (PRIMITIVE_ALIASES), any other name unchanged."""
    return dict(PRIMITIVE_ALIASES).get(name, name)


def parse_face(v):
    """(face dict, []) or (None, problems): the keys and their types, as `parse_face` reads them."""
    p = []
    if not isinstance(v, dict):
        return None, ["a face sidecar must be an object {size, eyes, ...}."]
    olds = dict(FACE_KEY_ALIASES)
    for k in v:
        if k not in FACE_KEYS and k not in olds:
            p.append(f"the face sidecar has no key '{k}' (keys: {', '.join(FACE_KEYS)}).")
    for old, new in FACE_KEY_ALIASES:
        if old in v and new in v:
            p.append(f"the face sidecar holds both '{new}' and its old name '{old}' (keep '{new}').")
    v = dict(v)
    for old, new in FACE_KEY_ALIASES:
        if old in v and new not in v:
            v[new] = v[old]
    if "size" in v:
        size = whole(v["size"])
        if size is None:
            p.append("size must be a whole number.")
            size = 0
    else:
        p.append("size is required.")
        size = 0
    if "border" in v:
        border = whole(v["border"])
        if border is None:
            p.append("border must be a whole number.")
            border = default_border(size)
    else:
        border = default_border(size)
    eyes = _tuples(v, "eyes", 4, "[x, y, w, h]", p)
    blink = True
    if "blink" in v:
        if isinstance(v["blink"], bool):
            blink = v["blink"]
        else:
            p.append("blink must be true or false.")
    lid = _colour_or_empty(v, "lid", "lid", p)
    glow = _tuples(v, "glow", 3, "[x, y, r]", p)
    lines = _tuples(v, "lines", 4, "[x, y, deg, len]", p)
    mouth = _one_box(v, "mouth", p)
    face_box = _one_box(v, "face", p)
    rim = _colour_or_empty(v, "rim", "rim", p)
    idle = []
    if "idle" in v:
        if isinstance(v["idle"], list):
            for i, e in enumerate(v["idle"]):
                n = i + 1
                if not isinstance(e, dict):
                    p.append(f"idle entry {n} must be an object {{effect, color, intensity}}.")
                    continue
                for k in e:
                    if k not in IDLE_KEYS:
                        p.append(f"idle entry {n} has no key '{k}' (keys: {', '.join(IDLE_KEYS)}).")
                effect = e.get("effect")
                if not isinstance(effect, str):
                    p.append(f"idle entry {n}: effect must be a name.")
                    effect = ""
                effect = canonical_primitive(effect)
                color = _colour_or_empty(e, "color", f"idle entry {n}: color", p)
                intensity = 1.0
                if "intensity" in e:
                    x = e["intensity"]
                    if isinstance(x, bool) or not isinstance(x, (int, float)):
                        p.append(f"idle entry {n}: intensity must be a number.")
                    else:
                        intensity = f32(float(x))
                idle.append({"effect": effect, "color": color, "intensity": intensity})
        else:
            p.append("idle must be a list of {effect, color, intensity}.")
    if p:
        return None, p
    return {"size": size, "border": border, "eyes": eyes, "blink": blink, "lid": lid, "glow": glow, "lines": lines,
            "mouth": mouth, "face": face_box, "rim": rim, "idle": idle}, []


def face_problems(face, png_w):
    """Every range and anchor rule of a parsed sidecar, as `face_problems` writes them (png_w 0: no width check)."""
    out = []
    s = face["size"]
    if not FACE_SIZE_MIN <= s <= FACE_SIZE_MAX:
        out.append(f"size must be {FACE_SIZE_MIN}-{FACE_SIZE_MAX} (is {s}).")
    if png_w > 0 and png_w != s:
        out.append(f"size {s} differs from the PNG's width {png_w}: the anchors are scaled by {png_w} / {s}.")
    if not 0 <= face["border"] <= tdiv(s, 8):
        out.append(f"border must be 0-{tdiv(s, 8)} (is {face['border']}).")
    lo, hi = face["border"], s - face["border"]
    inside = lambda x, y: lo <= x < hi and lo <= y < hi
    if len(face["eyes"]) > 2:
        out.append(f"eyes holds at most 2 boxes (has {len(face['eyes'])}).")
    for n, (x, y, w, h) in enumerate(face["eyes"], 1):
        if x < lo or y < lo or x + w > hi or y + h > hi:
            out.append(f"eye box {n} [{x}, {y}, {w}, {h}] must lie inside the inner rect ({lo}-{hi}).")
        if not 2 <= w <= tdiv(s, 6):
            out.append(f"eye box {n} width must be 2-{tdiv(s, 6)} (is {w}).")
        if not 1 <= h <= tdiv(s, 10):
            out.append(f"eye box {n} height must be 1-{tdiv(s, 10)} (is {h}).")
    if len(face["glow"]) > 3:
        out.append(f"glow holds at most 3 anchors (has {len(face['glow'])}).")
    for n, (x, y, r) in enumerate(face["glow"], 1):
        if not inside(x, y):
            out.append(f"glow anchor {n} centre ({x}, {y}) must lie inside the inner rect ({lo}-{hi}).")
        if not 4 <= r <= tdiv(s, 2):
            out.append(f"glow anchor {n} radius must be 4-{tdiv(s, 2)} (is {r}).")
    if len(face["lines"]) > 3:
        out.append(f"lines holds at most 3 roots (has {len(face['lines'])}).")
    for n, (x, y, deg, ln) in enumerate(face["lines"], 1):
        if not inside(x, y):
            out.append(f"line root {n} ({x}, {y}) must lie inside the inner rect ({lo}-{hi}).")
        if not -180 <= deg <= 360:
            out.append(f"line root {n} deg must be -180-360 (is {deg}).")
        if not 6 <= ln <= tdiv(s, 2):
            out.append(f"line root {n} len must be 6-{tdiv(s, 2)} (is {ln}).")
    if face["mouth"] is not None:
        x, y, w, h = face["mouth"]
        if x < lo or y < lo or x + w > hi or y + h > hi or w < 1 or h < 1:
            out.append(f"mouth [{x}, {y}, {w}, {h}] must lie inside the inner rect ({lo}-{hi}).")
    if face["face"] is not None:
        x, y, w, h = face["face"]
        if x < lo or y < lo or x + w > hi or y + h > hi:
            out.append(f"face box [{x}, {y}, {w}, {h}] must lie inside the inner rect ({lo}-{hi}).")
        if w < tdiv(s, 8) or h < tdiv(s, 8):
            out.append(f"face box width and height must be at least {tdiv(s, 8)} (is {w} x {h}).")
    if len(face["idle"]) > 4:
        out.append(f"idle holds at most 4 overlays (has {len(face['idle'])}).")
    for o in face["idle"]:
        e = o["effect"]
        if e not in IDLE_OVERLAYS:
            out.append(f"idle '{e}' is not an overlay (one of {', '.join(IDLE_OVERLAYS)}).")
        if e == "glow" and not face["glow"]:
            out.append("idle glow needs a glow anchor.")
        elif e == "lines" and not face["lines"]:
            out.append("idle lines needs a line root.")
        elif e == "point_glow" and not face["eyes"]:
            out.append("idle point_glow needs an eye box.")
        if not f32(0.05) <= o["intensity"] <= 1.0:
            out.append(f"idle '{e}' intensity must be 0.05-1.0 (is {f32_text(o['intensity'])}).")
    return out


# ---- files ----------------------------------------------------------------------------------------------------------

def face_file(png):
    """The sidecar of a portrait: `.png` replaced by `.face.json`."""
    png = pathlib.Path(png)
    return png.with_name((png.name[:-4] if png.name.endswith(".png") else png.name) + ".face.json")


def width_of(png):
    """A PNG's width in px."""
    with Image.open(png) as im:
        return im.size[0]


def png_of(sidecar):
    sidecar = pathlib.Path(sidecar)
    return sidecar.with_name(sidecar.name[:-len(".face.json")] + ".png")


def sidecar_problems(sidecar):
    """The problems of one sidecar file as A11 states them (without the path): its PNG missing, then the parse or the
    range and anchor problems against the PNG's width."""
    out = []
    png = png_of(sidecar)
    width = 0
    if png.exists():
        width = width_of(png)
    else:
        out.append(f"no portrait {png.name} beside the sidecar.")
    try:
        raw = json.loads(pathlib.Path(sidecar).read_text(encoding="utf-8"))
    except (ValueError, OSError) as e:
        return out + [f"the sidecar is not JSON ({e})."]
    face, bad = parse_face(raw)
    return out + (bad if face is None else face_problems(face, width))


def check(roots):
    """`path: problem` lines over every sidecar under each layer root's portraits/ (paths relative to the root, files
    in sorted order); returns (lines, sidecar count)."""
    lines, count = [], 0
    for root in map(pathlib.Path, roots):
        base = root / "portraits" if (root / "portraits").is_dir() else root
        for f in sorted(base.rglob("*.face.json"), key=lambda q: q.relative_to(root).as_posix()):
            count += 1
            rel = f.relative_to(root).as_posix()
            lines += [f"{rel}: {m}" for m in sidecar_problems(f)]
    return lines, count


def load(png):
    """The PNG's sidecar parsed: (face or None, problems; [] and None when there is no sidecar)."""
    side = face_file(png)
    if not side.exists():
        return None, []
    try:
        raw = json.loads(side.read_text(encoding="utf-8"))
    except ValueError as e:
        return None, [f"the sidecar is not JSON ({e})."]
    return parse_face(raw)


def hex_of(c):
    return "" if c is None else "#%02x%02x%02x" % c


def dump(face):
    """The sidecar text: every key of FACE_KEYS in order, one per line, lists written flat; `face` only when set
    (PORTRAITFX1-EX: the sidecars written before it keep their bytes)."""
    data = {"size": face["size"], "border": face["border"], "eyes": face["eyes"], "blink": face["blink"],
            "lid": hex_of(face["lid"]), "glow": face["glow"], "lines": face["lines"], "mouth": face["mouth"] or []}
    if face["face"] is not None:
        data["face"] = face["face"]
    data |= {"rim": hex_of(face["rim"]),
            "idle": [{"effect": o["effect"], "color": hex_of(o["color"]), "intensity": float(f32_text(o["intensity"]))}
                     for o in face["idle"]]}
    body = ",\n".join(f"  {json.dumps(k)}: {json.dumps(v, separators=(', ', ': '))}" for k, v in data.items())
    return "{\n" + body + "\n}\n"


# ---- set -------------------------------------------------------------------------------------------------------------

def _ints(text, n, what):
    parts = text.split(",")
    if len(parts) != n:
        raise SystemExit(f"{what} takes {n} whole numbers joined by commas (got '{text}').")
    try:
        return [int(x) for x in parts]
    except ValueError:
        raise SystemExit(f"{what} takes whole numbers (got '{text}').")


def _idle(text):
    effect, *rest = text.split(":")
    entry = {"effect": effect, "color": rest[0] if rest else "", "intensity": 1.0}
    if len(rest) > 1 and rest[1] != "":
        try:
            entry["intensity"] = float(rest[1])
        except ValueError:
            raise SystemExit(f"--idle intensity must be a number (got '{rest[1]}').")
    if len(rest) > 2:
        raise SystemExit(f"--idle takes effect[:#rrggbb[:intensity]] (got '{text}').")
    return entry


def set_face(png, a):
    """Builds the sidecar from the existing one (unless --fresh) and the options, checks it against the PNG; writes it
    only when sound. Returns the problems (empty: written)."""
    png = pathlib.Path(png)
    width = width_of(png)
    side = face_file(png)
    raw = {}
    if side.exists() and not a.fresh:
        raw = json.loads(side.read_text(encoding="utf-8"))
        for old, new in FACE_KEY_ALIASES:  # a sidecar written before the rename: its old key becomes the new one
            if old in raw and new not in raw:
                raw[new] = raw.pop(old)
    raw["size"] = width
    if a.border is not None:
        raw["border"] = a.border
    if a.eyes is not None:
        raw["eyes"] = [_ints(t, 4, "--eyes") for t in a.eyes]
    if a.blink is not None:
        raw["blink"] = a.blink
    if a.lid is not None:
        raw["lid"] = a.lid
    if a.glow is not None:
        raw["glow"] = [_ints(t, 3, "--glow") for t in a.glow]
    if a.lines is not None:
        raw["lines"] = [_ints(t, 4, "--lines") for t in a.lines]
    if a.mouth is not None:
        raw["mouth"] = [] if a.mouth == "" else _ints(a.mouth, 4, "--mouth")
    if a.face is not None:
        raw["face"] = [] if a.face == "" else _ints(a.face, 4, "--face")
    if a.rim is not None:
        raw["rim"] = a.rim
    if a.idle is not None:
        raw["idle"] = [_idle(t) for t in a.idle]
    face, bad = parse_face(raw)
    if face is None:
        return bad
    bad = face_problems(face, width)
    if bad:
        return bad
    side.write_text(dump(face), encoding="utf-8", newline="\n")
    return []


# ---- pictures --------------------------------------------------------------------------------------------------------

def scaled(png, scale=SCALE):
    with Image.open(png) as src:
        im = src.convert("RGBA")
    return im.resize((im.size[0] * scale, im.size[1] * scale), Image.NEAREST)


def box(d, x, y, w, h, colour, scale):
    """A 1 device px outline round image px [x, x+w) x [y, y+h) at `scale`."""
    d.rectangle([x * scale, y * scale, (x + w) * scale - 1, (y + h) * scale - 1], outline=colour)


def draw_anchors(im, face, scale=SCALE):
    """The inner rect, eye boxes, glow circles (centre dot), line roots (a dot and a line of `len` along `deg`), the
    mouth box and the face box over a portrait already scaled by `scale`."""
    ov = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    b, s = face["border"], face["size"]
    box(d, b, b, s - 2 * b, s - 2 * b, INNER, scale)
    for x, y, w, h in face["eyes"]:
        box(d, x, y, w, h, EYE, scale)
    for x, y, r in face["glow"]:
        cx, cy = (x + 0.5) * scale, (y + 0.5) * scale
        d.ellipse([cx - r * scale, cy - r * scale, cx + r * scale, cy + r * scale], outline=GLOW)
        box(d, x, y, 1, 1, GLOW, scale)
    for x, y, deg, ln in face["lines"]:
        cx, cy = (x + 0.5) * scale, (y + 0.5) * scale
        t = math.radians(deg)
        d.line([cx, cy, cx + math.cos(t) * ln * scale, cy + math.sin(t) * ln * scale], fill=LINE, width=1)
        box(d, x, y, 1, 1, LINE, scale)
    if face["mouth"] is not None:
        box(d, *face["mouth"], MOUTH, scale)
    if face.get("face") is not None:
        box(d, *face["face"], FACE, scale)
    return Image.alpha_composite(im, ov)


def font():
    return ImageFont.load_default()


def label(d, xy, text):
    l, t, r, b = d.textbbox(xy, text, font=font())
    d.rectangle([l - 1, t - 1, r + 1, b + 1], fill=LABEL_BG)
    d.text(xy, text, fill=LABEL, font=font())


def grid(png, scale=SCALE):
    """The portrait at `scale` with a line every 8 image px (brighter every 40) and the coordinate of every 40th line
    along the top and the left edge."""
    im = scaled(png, scale)
    w, h = im.size[0] // scale, im.size[1] // scale
    ov = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    for v in range(0, max(w, h) + 1, 8):
        c = GRID_MAJOR if v % 40 == 0 else GRID
        if v <= w:
            d.line([v * scale, 0, v * scale, im.size[1] - 1], fill=c)
        if v <= h:
            d.line([0, v * scale, im.size[0] - 1, v * scale], fill=c)
    for v in range(40, max(w, h), 40):
        if v < w:
            label(d, (v * scale + 2, 2), str(v))
        if v < h:
            label(d, (2, v * scale + 2), str(v))
    return Image.alpha_composite(im, ov)


def show(png, scale=SCALE):
    face, bad = load(png)
    if face is None:
        raise SystemExit(f"{face_file(png)}: " + ("; ".join(bad) if bad else "no sidecar."))
    return draw_anchors(scaled(png, scale), face, scale)


def all_portraits():
    out = []
    for layer in LAYERS:
        out += sorted((layer / "portraits").rglob("*.png"), key=lambda q: q.relative_to(layer).as_posix())
    return out


def contact(pngs, scale=SCALE, cols=6):
    """Every portrait at `scale` with its anchors (or a "no sidecar" / "bad sidecar" note) and its name under it."""
    tiles = []
    for png in pngs:
        face, bad = load(png)
        im = scaled(png, scale)
        note = "" if face else ("bad sidecar" if bad else "no sidecar")
        tiles.append((draw_anchors(im, face, scale) if face else im, pathlib.Path(png).stem, note))
    cell_w = max((t[0].size[0] for t in tiles), default=1)
    cell_h = max((t[0].size[1] for t in tiles), default=1)
    cap = 14
    cols = max(1, min(cols, len(tiles)))
    rows = max(1, -(-len(tiles) // cols))
    sheet = Image.new("RGBA", (cols * cell_w, rows * (cell_h + cap)), (16, 16, 20, 255))
    d = ImageDraw.Draw(sheet)
    for i, (im, name, note) in enumerate(tiles):
        x, y = (i % cols) * cell_w, (i // cols) * (cell_h + cap)
        sheet.paste(im, (x, y))
        d.text((x + 2, y + cell_h + 1), name + (f" ({note})" if note else ""), fill=LABEL, font=font())
    return sheet


# ---- command line ----------------------------------------------------------------------------------------------------

def main(argv=None):
    ap = argparse.ArgumentParser(prog="face.py", description="Portrait face sidecars (PORTRAITFX1): anchors and checks.")
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("grid", help="the portrait at 4x with a coordinate grid")
    g.add_argument("png"); g.add_argument("out")
    s = sub.add_parser("set", help="write the sidecar beside the PNG")
    s.add_argument("png")
    s.add_argument("--eyes", nargs="*", metavar="x,y,w,h")
    s.add_argument("--no-blink", dest="blink", action="store_false", default=None)
    s.add_argument("--blink", dest="blink", action="store_true")
    s.add_argument("--lid", metavar="#rrggbb")
    s.add_argument("--glow", nargs="*", metavar="x,y,r")
    s.add_argument("--lines", "--veins", dest="lines", nargs="*", metavar="x,y,deg,len")
    s.add_argument("--mouth", metavar="x,y,w,h")
    s.add_argument("--face", metavar="x,y,w,h", help="the face box of the expression effects (\"\": none, derived from the eyes)")
    s.add_argument("--rim", metavar="#rrggbb")
    s.add_argument("--border", type=int)
    s.add_argument("--idle", nargs="*", metavar="effect[:#rrggbb[:intensity]]")
    s.add_argument("--fresh", action="store_true", help="ignore the existing sidecar")
    w = sub.add_parser("show", help="the anchors over the 4x portrait")
    w.add_argument("png"); w.add_argument("out")
    c = sub.add_parser("check", help="the A11 rules over every sidecar")
    c.add_argument("roots", nargs="*")
    k = sub.add_parser("contact", help="every portrait with its anchors on one sheet")
    k.add_argument("out"); k.add_argument("pngs", nargs="*")
    k.add_argument("--cols", type=int, default=6)
    a = ap.parse_args(argv)

    if a.cmd == "grid":
        grid(a.png).save(a.out)
        print(f"wrote {a.out}")
    elif a.cmd == "set":
        bad = set_face(a.png, a)
        if bad:
            for m in bad:
                print(f"{face_file(a.png).name}: {m}", file=sys.stderr)
            print("refused: nothing written.", file=sys.stderr)
            return 1
        print(f"wrote {face_file(a.png)}")
    elif a.cmd == "show":
        show(a.png).save(a.out)
        print(f"wrote {a.out}")
    elif a.cmd == "check":
        lines, count = check(a.roots or [str(p) for p in LAYERS])
        for line in lines:
            print(line)
        print(f"{count} sidecars, {len(lines)} problems.", file=sys.stderr)
        return 1 if lines else 0
    elif a.cmd == "contact":
        contact(a.pngs or all_portraits(), cols=a.cols).save(a.out)
        print(f"wrote {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
