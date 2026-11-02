"""REGIONGROUND1-T: take the broad light and dark out of a ground texture, keep part of its fine grain.

    flatten.py SRC --out DST [--class natural|built] [--gain G] [--colour RRGGBB] [--isotropic] [--record JSON]

Per channel, `mean + (T - low) x G`, `low` a Gaussian blur of the image tiled 3 x 3 (centre crop), sigma 5 at 64 px wide,
so 20 at 256: broad tone in a texture reads as a loop across the map, and the map-wide tone (EN 2.2) carries it instead.

Gain: natural from .55 down to .17, built from .45 down to .25, by .05 steps (the last to the floor) until groundcheck.py's
checks that scale with the gain pass (broad tone, range, busy); else the exit is 1, a re-roll, and the last try is still
written so it can be looked at. --gain sets one gain and takes no steps. --record merges what was done into the art's
record, a JSON file (`flatten`: class, gain, isotropic). Direction, banding, blotch and colour do not move with the gain.

--colour: set the result's mean per channel to a ground's colour from the spec's table, by one shift per channel, so the
pattern stays whole and it stays seamless (decision "Ground textures: pattern from art, colour from code"). The art's own
tint no longer decides hue or value. The gain's steps score the coloured result.

--isotropic: blend_proto6.py's average of the 8 rotations and mirrors x 2.1, plus its speckle at art px. It cancels an
angle but leaves static-like grain: a fallback for one texture at a time on the user's say, never a default.
"""
import argparse
import json
import os
import random
import sys

from PIL import Image, ImageFilter

import groundcheck

GAINS = {"natural": 0.55, "built": 0.45}
GAIN_FLOORS = {"natural": 0.17, "built": 0.25}   # natural: about what the approved `dirt_flat` shows
GAIN_STEP = 0.05
SIGMA_AT_64 = 5.0
ISOTROPIC_GAIN = 2.1
SPECKLE_SEED = 31
SPECKLE_DARK, SPECKLE_LIGHT = (0.86, -5.0), (0.10, 4.0)   # a draw above .86 is darker by 5, one under .10 lighter by 4
GAIN_CHECKS = ("broad", "range", "busy")   # what a lower gain can bring down
MAX_SOURCE = 4096               # px a side: refused above it (the art is 1024, a master 256)
MEAN_PASSES = 16                # shifts of set_mean: more than one only where a clipped pixel moved the mean


def _clip(v):
    return 0 if v < 0 else 255 if v > 255 else int(v + 0.5)


def low_of(img):
    """The broad tone of `img` (RGB) on the torus: a Gaussian blur of the image tiled 3 x 3, centre crop."""
    w, h = img.size
    big = Image.new("RGB", (3 * w, 3 * h))
    for y in range(3):
        for x in range(3):
            big.paste(img, (x * w, y * h))
    return big.filter(ImageFilter.GaussianBlur(SIGMA_AT_64 * w / 64)).crop((w, h, 2 * w, 2 * h))


def channel_means(raw):
    n = len(raw) // 3
    return [sum(raw[c::3]) / n for c in range(3)]


def flatten(img, gain, colour=None):
    """`img` (RGB) with its broad tone out and its fine detail x `gain`; the channel means stay, or sit at `colour`."""
    img = img.convert("RGB")
    raw = img.tobytes()
    low = low_of(img).tobytes()
    means = list(colour) if colour is not None else channel_means(raw)
    out = bytearray(len(raw))
    for i in range(len(raw)):
        out[i] = _clip(means[i % 3] + (raw[i] - low[i]) * gain)
    return Image.frombytes("RGB", img.size, bytes(out))


def parse_colour(text):
    """(r, g, b) of `RRGGBB` or `#RRGGBB`; ValueError on anything else."""
    v = text[1:] if text.startswith("#") else text
    if len(v) != 6 or any(c not in "0123456789abcdefABCDEF" for c in v):
        raise ValueError(f"colour {text!r} is not RRGGBB")
    return tuple(int(v[i:i + 2], 16) for i in (0, 2, 4))


def set_mean(img, colour):
    """`img` (RGB) shifted per channel so its mean sits at `colour`, within half a level (whole levels: a pixel moves by the
    same amount as its neighbours, so the pattern and the seam stay). A pixel that clips at 0 or 255 pulls the mean back:
    another shift makes that up, and a mean that no shift can reach is left at the nearest."""
    out = img.convert("RGB")
    for _ in range(MEAN_PASSES):
        raw = out.tobytes()
        shift = [int(round(c - m)) for c, m in zip(colour, channel_means(raw))]
        if not any(shift):
            break
        table = [_clip(v + d) for d in shift for v in range(256)]
        out = out.point(table)
    return out


def _turns(rows):
    """The 8 rotations and mirrors of a square of rows."""
    out = []
    for grid in (rows, [row[::-1] for row in rows]):
        for _ in range(4):
            out.append(grid)
            grid = [list(r) for r in zip(*grid[::-1])]
    return out


def isotropic(img):
    """`img` (RGB, square) averaged with its rotations and mirrors x 2.1, plus the speckle at art px (a 64th of its
    width), so the speckle survives the reduce to the 64 x 64 form."""
    img = img.convert("RGB")
    w, h = img.size
    if w != h:
        raise ValueError(f"--isotropic takes a square, this is {w}x{h}")
    raw = img.tobytes()
    means = channel_means(raw)
    block = max(1, w // groundcheck.FORM)
    rnd = random.Random(SPECKLE_SEED)
    side = w // block + 1
    speckle = []
    for _ in range(side * side):
        v = rnd.random()
        speckle.append(SPECKLE_DARK[1] if v > SPECKLE_DARK[0] else SPECKLE_LIGHT[1] if v < SPECKLE_LIGHT[0] else 0.0)
    out = bytearray(len(raw))
    for c in range(3):
        rows = [[raw[(y * w + x) * 3 + c] - means[c] for x in range(w)] for y in range(h)]
        turns = _turns(rows)
        for y in range(h):
            for x in range(w):
                avg = sum(t[y][x] for t in turns) / len(turns)
                sp = speckle[(y // block) * side + x // block]
                out[(y * w + x) * 3 + c] = _clip(means[c] + avg * ISOTROPIC_GAIN + sp)
    return Image.frombytes("RGB", (w, h), bytes(out))


def run(img, cls, gain=None, iso=False, colour=None):
    """(flattened image, the gain used, the GAIN_CHECKS it still fails). An explicit `gain` takes no steps. `colour` sets its
    mean; the scores are of the coloured image, since broad tone is relative to the mean."""
    g = gain if gain is not None else GAINS[cls]
    floor = GAIN_FLOORS[cls]
    while True:
        out = flatten(img, g, colour)
        if iso:
            out = isotropic(out)
        if colour is not None:
            out = set_mean(out, colour)
        bad = [c for c in groundcheck.failures(groundcheck.scores(out), cls) if c in GAIN_CHECKS]
        if not bad or gain is not None or g <= floor:
            return out, round(g, 2), bad
        g = max(floor, round(g - GAIN_STEP, 2))


def merge_record(path, entry):
    """Merge {"flatten": entry} into the JSON file `path` (made when missing)."""
    record = {}
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            record = json.load(f)
    record["flatten"] = entry
    with open(path, "w", encoding="utf-8") as f:
        json.dump(record, f, indent=2)
        f.write("\n")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("src")
    ap.add_argument("--out", required=True)
    ap.add_argument("--class", dest="cls", choices=sorted(GAINS), default="natural")
    ap.add_argument("--gain", type=float, help="one gain, no steps (default: by class)")
    ap.add_argument("--colour", help="set the mean to this ground's colour, RRGGBB or #RRGGBB, keeping the pattern")
    ap.add_argument("--isotropic", action="store_true", help="a fallback for one texture on the user's say, never a default")
    ap.add_argument("--record", help="the art's record (JSON): merge the class and gain used")
    a = ap.parse_args(argv)
    if a.gain is not None and a.gain < 0:
        print(f"error: --gain {a.gain} is negative", file=sys.stderr)
        return 1
    try:
        colour = parse_colour(a.colour) if a.colour is not None else None
        with Image.open(a.src) as img:
            if max(img.size) > MAX_SOURCE:
                raise ValueError(f"{img.size[0]}x{img.size[1]} is over {MAX_SOURCE} px a side")
            out, gain, bad = run(img.convert("RGB"), a.cls, a.gain, a.isotropic, colour)
    except ValueError as e:
        print(f"error: {a.src}: {e}", file=sys.stderr)
        return 1
    out.save(a.out)
    if a.record:
        entry = {"class": a.cls, "gain": gain, "isotropic": a.isotropic}
        if colour is not None:
            entry["colour"] = "#%02x%02x%02x" % colour
        merge_record(a.record, entry)
    print(f"{a.src} -> {a.out} (gain {gain:.2f})")
    if bad:
        print(f"still over the limits a gain can reach ({', '.join(bad)}): re-roll", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
