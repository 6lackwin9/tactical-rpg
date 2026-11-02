"""REGIONGROUND1-T: is a ground texture quiet enough to repeat across a map under small figures?

    groundcheck.py score PNG --class natural|built   the table of scores against the limits, exit 1 on a failure
    groundcheck.py figures GROUND_DIR --art PATHS    warnings: a figure not well above a ground in range and busyness
    groundcheck.py pair GROUND_DIR                   warnings: two grounds whose means are too close for an edge to read
    groundcheck.py read FRAME GROUND_PNG [--option shadow|pool|rim] [--colour RRGGBB]
                                                     read_contrast and ring step of one figure frame on a ground

GROUND_DIR holds `<id>/v0.png`. Every score is taken on the 64 x 64 form (one box reduce), and L is Rec. 709 luminance on
0-255. The formulas and limits are REGIONGROUND1.md section 6 and REGIONGROUND1_SCENE.md section 2: each answers one thing
the user saw on real art (the plaid floor, the looping light and dark on dirt, the angled streaks, too strong a colour).
"""
import argparse
import math
import os
import sys

from PIL import Image

import seam

FORM = 64                       # the art px of a texture over 4 cells (EN 1.1)
BLUR_SIGMA = 5.0                # the broad-tone blur at FORM px wide; flatten.py scales it with the image
BLOTCH_SIGMA = 2.0              # a blotch's scale at FORM px wide: above the grain, below the broad tone
ANGLE_BINS = 8                  # orientation bins over 180 degrees, one centred on each axis
# Spec section 6 and SC 2. `banding` and the direction pair are the same for both classes.
LIMITS = {
    "natural": {"seam": 1.5, "broad": 1.2, "blotch": 1.4, "banding": 0.30, "peak": 1.8, "coherence": 0.25, "range": 10.0,
                "busy": 1.0, "colour": 12.0, "lum": 25.0},
    "built": {"seam": 1.5, "broad": 3.0, "blotch": 1.4, "banding": 0.30, "peak": 1.8, "coherence": 0.25, "range": 14.0,
              "busy": 2.0, "colour": 12.0, "lum": 25.0},
}
CHECKS = [
    ("seam", "Seam", "wrap step over inner step"),
    ("broad", "Broad tone", "stdev(L, wrapped blur) over mean(L), %"),
    ("blotch", "Blotch", "mean(L) - min(L, wrapped sigma 2 blur), over stdev(L)"),
    ("banding", "Banding", "stdev(row or column means) over stdev(L)"),
    ("peak", "Direction, peak", "gradient energy, largest of 8 angle bins over their mean"),
    ("coherence", "Direction, coherence", "structure tensor"),
    ("range", "Contrast, range", "L p95 - p5"),
    ("busy", "Contrast, busy", "mean step in L"),
    ("colour", "Colour", "mean(max - min of RGB)"),
    ("lum", "Luminance", "mean(L), % of 255"),
]
BUILT_GROUNDS = {"flagstone", "cobble_road", "paved_road", "iron_catwalk"}   # spec section 3, column "Class, gain"
MIN_PAIR_DISTANCE = 8.0         # RGB distance of two grounds' means, so their edge reads
FIGURE_RANGE, FIGURE_BUSY = 4.0, 3.0   # a figure against a ground, at least (SC 2)
FLAT_PROPS = {"cistern", "cobbles", "scorch_mark"}   # lie on the ground: not figures (SC 2)
OPAQUE = 128                    # alpha that counts as a figure's pixel (the engine's OUTLINE_ALPHA is its neighbour)
# REGIONGROUND1_SCENE 3.3, in cells; a cell is CELL sprite px wide on the frame read_contrast measures.
CELL = 32
SHADOW = {"alpha": 90, "rx": 0.45, "ry": 0.18}
POOL = {"alpha": 46, "rx": 0.7, "ry": 0.35}
RIM_ALPHA = 130
NPC_COLOUR = (230, 190, 80)     # region.rs marker_style for a person
BAND = 2                        # px of frame around the ring


def luma_of(p):
    return 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]


def pixels(img, bands=3):
    """The pixels of `img` in RGB (or RGBA at 4 bands) as tuples, row by row."""
    raw = img.convert("RGB" if bands == 3 else "RGBA").tobytes()
    return [tuple(raw[i:i + bands]) for i in range(0, len(raw), bands)]


def form(img):
    """The 64 x 64 form: one box reduce (a smaller image is only made RGB)."""
    img = img.convert("RGB")
    return img if img.size == (FORM, FORM) else img.resize((FORM, FORM), Image.BOX)


def luma_grid(img):
    """L per pixel as rows of floats."""
    w, h = img.size
    data = pixels(img)
    return [[luma_of(data[y * w + x]) for x in range(w)] for y in range(h)]


def form_luma(img):
    """L of the 64 x 64 form as rows of floats: the box means of L when the image is a whole multiple of the form, kept
    unrounded. Rounded to 8 bits, the quiet grain these limits ask for steps by 1 or 2 levels, its gradients pile up on
    four angles, and even noise scores 2.4 to 2.9 on direction's peak (probe 2026-11-02)."""
    w, h = img.size
    if w % FORM or h % FORM:
        return luma_grid(form(img))
    grid = luma_grid(img)
    bx, by = w // FORM, h // FORM
    if bx == 1 and by == 1:
        return grid
    area = bx * by
    return [[sum(sum(grid[y][x * bx:(x + 1) * bx]) for y in range(row * by, (row + 1) * by)) / area for x in range(FORM)]
            for row in range(FORM)]


def stdev(values):
    n = len(values)
    mean = sum(values) / n
    return math.sqrt(sum((v - mean) ** 2 for v in values) / n)


def percentile(values, q):
    """The q-th percentile with linear interpolation between ranks."""
    s = sorted(values)
    at = q / 100 * (len(s) - 1)
    lo = int(at)
    hi = min(lo + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (at - lo)


def wrapped_blur(grid, sigma):
    """A true Gaussian blur of rows of floats on the torus. Kept in floats: on a tile of mean L 30, rounding a blur to 8
    bits would add about .7 % to a broad-tone score whose limit is 1.2 %."""
    h, w = len(grid), len(grid[0])
    radius = int(3 * sigma + 0.5)
    kernel = [math.exp(-(i * i) / (2 * sigma * sigma)) for i in range(-radius, radius + 1)]
    total = sum(kernel)
    kernel = [k / total for k in kernel]
    rows = []
    for row in grid:
        rows.append([sum(k * row[(x + i - radius) % w] for i, k in enumerate(kernel)) for x in range(w)])
    return [[sum(k * rows[(y + i - radius) % h][x] for i, k in enumerate(kernel)) for x in range(w)] for y in range(h)]


def broad_tone(grid):
    flat = [v for row in grid for v in row]
    mean = sum(flat) / len(flat)
    if mean == 0:
        return 0.0
    low = wrapped_blur(grid, BLUR_SIGMA)
    return 100 * stdev([v for row in low for v in row]) / mean


def blotch(grid):
    """How far the darkest blotch sits below the mean, in stdevs of L. A dark patch repeats with the texture and is what
    the eye finds first; a tail of the histogram misses one that covers more than a few % of the tile, so this looks at
    the blur's lowest point. A ratio, so a lower gain does not hide a patch: it stays the most visible thing there."""
    spread = stdev([v for row in grid for v in row])
    if spread == 0:
        return 0.0
    flat = [v for row in grid for v in row]
    low = wrapped_blur(grid, BLOTCH_SIGMA)
    return (sum(flat) / len(flat) - min(min(row) for row in low)) / spread


def banding(grid):
    flat = [v for row in grid for v in row]
    spread = stdev(flat)
    if spread == 0:
        return 0.0
    h, w = len(grid), len(grid[0])
    rows = [sum(r) / w for r in grid]
    cols = [sum(grid[y][x] for y in range(h)) / h for x in range(w)]
    return max(stdev(rows), stdev(cols)) / spread


def direction(grid):
    """(peak, coherence) of the wrapped central differences of L. Orientation is taken over 180 degrees, so one set of
    stripes falls in one bin whichever way its light and dark run. A bin is centred on each axis, where the flattened
    plaid floor scores 2.76 and fails. With a bin boundary on the axis, an axis-lined edge splits between two bins, and
    that floor scores about 1.7 and passes."""
    h, w = len(grid), len(grid[0])
    bins = [0.0] * ANGLE_BINS
    jxx = jyy = jxy = 0.0
    width = math.pi / ANGLE_BINS
    for y in range(h):
        for x in range(w):
            gx = (grid[y][(x + 1) % w] - grid[y][(x - 1) % w]) / 2
            gy = (grid[(y + 1) % h][x] - grid[(y - 1) % h][x]) / 2
            energy = gx * gx + gy * gy
            if energy > 0:
                bins[int((math.atan2(gy, gx) % math.pi) / width + 0.5) % ANGLE_BINS] += energy
            jxx += gx * gx
            jyy += gy * gy
            jxy += gx * gy
    mean = sum(bins) / ANGLE_BINS
    trace = jxx + jyy
    if mean == 0 or trace == 0:
        return 0.0, 0.0
    return max(bins) / mean, math.sqrt((jxx - jyy) ** 2 + 4 * jxy * jxy) / trace


def busy(grid):
    """Mean step in L to the right and below, wrapped."""
    h, w = len(grid), len(grid[0])
    total = sum(abs(grid[y][x] - grid[y][(x + 1) % w]) + abs(grid[y][x] - grid[(y + 1) % h][x]) for y in range(h) for x in range(w))
    return total / (2 * w * h)


def colour_cap(img):
    data = pixels(img)
    return sum(max(p) - min(p) for p in data) / len(data)


def scores(img):
    """The checks' scores of `img`, taken on its 64 x 64 form."""
    small = form(img)
    grid = form_luma(img)
    flat = [v for row in grid for v in row]
    peak, coherence = direction(grid)
    return {
        "seam": seam.score(small),
        "broad": broad_tone(grid),
        "blotch": blotch(grid),
        "banding": banding(grid),
        "peak": peak,
        "coherence": coherence,
        "range": percentile(flat, 95) - percentile(flat, 5),
        "busy": busy(grid),
        "colour": colour_cap(small),
        "lum": sum(flat) / len(flat) / 2.55,
    }


def failures(result, cls):
    """The ids of the checks `result` fails as a ground of class `cls`."""
    limits = LIMITS[cls]
    return [c for c, _, _ in CHECKS if result[c] > limits[c]]


def size_warning(img):
    """A warning when the scores of `img` are taken on a rounded form (the limits are for an unrounded one, see
    `form_luma`), else None."""
    w, h = img.size
    if w % FORM or h % FORM:
        return (f"{w}x{h} is not a whole multiple of {FORM}: its scores come from a rounded form and can differ from a "
                f"master's. cut.py --form ground first.")
    return None


def table(result, cls):
    """The lines `score` prints."""
    limits = LIMITS[cls]
    out = []
    for c, name, formula in CHECKS:
        verdict = "FAIL" if result[c] > limits[c] else "pass"
        out.append(f"{name:<22}{result[c]:>8.2f}  limit {limits[c]:>5.2f}  {verdict}  {formula}")
    return out


def class_of(ground_id):
    return "built" if ground_id in BUILT_GROUNDS else "natural"


def find_grounds(ground_dir):
    """[(id, path)] of every `<id>/v0.png`, sorted."""
    out = []
    if os.path.isdir(ground_dir):
        for g in sorted(os.listdir(ground_dir)):
            p = os.path.join(ground_dir, g, "v0.png")
            if os.path.exists(p):
                out.append((g, p))
    return out


def mean_rgb(img):
    data = pixels(form(img))
    return tuple(sum(p[i] for p in data) / len(data) for i in range(3))


def pair_distance(a, b):
    """RGB distance of the means of two ground images."""
    ma, mb = mean_rgb(a), mean_rgb(b)
    return math.sqrt(sum((ma[i] - mb[i]) ** 2 for i in range(3)))


def pair_warnings(grounds):
    """Warnings for [(id, image)]: two grounds too close for their edge to read."""
    out = []
    for i, (ida, a) in enumerate(grounds):
        for idb, b in grounds[i + 1:]:
            d = pair_distance(a, b)
            if d < MIN_PAIR_DISTANCE:
                out.append(f"pair {ida} / {idb}: means {d:.1f} apart, under {MIN_PAIR_DISTANCE:.0f}: the edge may not read")
    return out


def figure_stats(img):
    """(range, busy) of the opaque pixels of a figure's art; steps are between two opaque neighbours."""
    w, h = img.size
    px = pixels(img, 4)
    lum = [luma_of(p) if p[3] >= OPAQUE else None for p in px]
    solid = [v for v in lum if v is not None]
    if not solid:
        return 0.0, 0.0
    steps = []
    for y in range(h):
        for x in range(w):
            v = lum[y * w + x]
            if v is None:
                continue
            if x + 1 < w and lum[y * w + x + 1] is not None:
                steps.append(abs(v - lum[y * w + x + 1]))
            if y + 1 < h and lum[(y + 1) * w + x] is not None:
                steps.append(abs(v - lum[(y + 1) * w + x]))
    return percentile(solid, 95) - percentile(solid, 5), (sum(steps) / len(steps) if steps else 0.0)


def figure_warnings(grounds, figures):
    """Warnings for [(ground id, image)] and [(figure name, image)]: SC 2's limit from the figure's side. Flat props are
    left out."""
    out = []
    ground_scores = [(g, scores(img)) for g, img in grounds]
    for name, fig in figures:
        if os.path.splitext(os.path.basename(name))[0] in FLAT_PROPS or os.path.basename(os.path.dirname(name)) in FLAT_PROPS:
            continue
        fr, fb = figure_stats(fig)
        for g, r in ground_scores:
            if fr < FIGURE_RANGE * r["range"]:
                out.append(f"{name} on {g}: range {fr:.1f} under {FIGURE_RANGE:.0f} x {r['range']:.1f}")
            if fb < FIGURE_BUSY * r["busy"]:
                out.append(f"{name} on {g}: busy {fb:.2f} under {FIGURE_BUSY:.0f} x {r['busy']:.2f}")
    return out


def soft_alpha(dx, dy, rx, ry, alpha):
    """`aegis_ui::ground::soft_ellipse`: alpha x (1 - d^2)^2, d the distance over the radii; 0 at and past them."""
    d2 = (dx / rx) ** 2 + (dy / ry) ** 2
    return 0.0 if d2 >= 1 else alpha * (1 - d2) ** 2


def _over(under, col, alpha):
    a = alpha / 255
    return tuple(under[i] * (1 - a) + col[i] * a for i in range(3))


def ring_and_band(sprite):
    """A figure's solid pixels, its ring (solid pixels touching a non-solid one, sides or corners, as `Image::outline`
    counts them) and the band (non-solid pixels at most BAND px from the ring). Sets of (x, y) in a frame padded by BAND
    on each side."""
    w, h = sprite.size
    data = pixels(sprite, 4)
    solid = {(x + BAND, y + BAND) for y in range(h) for x in range(w) if data[y * w + x][3] >= OPAQUE}
    ring = {(x, y) for x, y in solid if any((x + dx, y + dy) not in solid for dx in (-1, 0, 1) for dy in (-1, 0, 1))}
    band = set()
    for x, y in ring:
        for dy in range(-BAND, BAND + 1):
            for dx in range(-BAND, BAND + 1):
                if (x + dx, y + dy) not in solid:
                    band.add((x + dx, y + dy))
    return solid, ring, band


def compose_frame(sprite, ground, option="shadow", colour=NPC_COLOUR):
    """A figure's frame (RGBA, 48 x 48: 24 logical px at 2 physical px, as at 1280 x 720) on a `ground` texture, as the
    frame shows it under `option` (SC 3.3): rows of RGB floats in the padded frame of `ring_and_band`, ground art px
    doubled. Marker marks use `colour`. The exact order and sampling are in the README, for the Rust check to match; the
    shadow and pool take their radii on the square cell (CELL) in both views."""
    rgba = sprite.convert("RGBA")
    w, h = rgba.size
    tex = form(ground)
    tw, th = tex.size
    tpx = tex.load()
    solid, ring, _ = ring_and_band(rgba)
    cx, cy = w / 2 + BAND, max((y for _, y in solid), default=0) - 1    # a shadow's centre: the figure's feet
    sprite_px = rgba.load()
    frame = []
    for y in range(h + 2 * BAND):
        row = []
        for x in range(w + 2 * BAND):
            c = tuple(float(v) for v in tpx[(x // 2) % tw, (y // 2) % th])
            if option == "pool":
                c = _over(c, colour, soft_alpha(x - cx, y - cy, POOL["rx"] * CELL, POOL["ry"] * CELL, POOL["alpha"]))
            c = _over(c, (0, 0, 0), soft_alpha(x - cx, y - cy, SHADOW["rx"] * CELL, SHADOW["ry"] * CELL, SHADOW["alpha"]))
            row.append(c)
        frame.append(row)
    if option == "rim":
        # `Image::outline`: non-solid pixels touching a solid one, sides or corners.
        rim = {(x + dx, y + dy) for x, y in solid for dy in (-1, 0, 1) for dx in (-1, 0, 1)} - solid
        for x, y in rim:
            frame[y][x] = _over(frame[y][x], colour, RIM_ALPHA)
    for x, y in solid:
        p = sprite_px[x - BAND, y - BAND]
        frame[y][x] = _over(frame[y][x], p[:3], p[3])
    return frame, ring


def read_contrast(sprite, ground, option="shadow", colour=NPC_COLOUR):
    """SC 3.2: |mean L of a sprite's ring pixels - mean L of the BAND px band of the frame around them|. For the rogue on
    flat `soot_yard` under `shadow` this gives 7.8, against SC 3.2's "about 7.5"."""
    frame, ring = compose_frame(sprite, ground, option, colour)
    _, _, band = ring_and_band(sprite)
    ring_l = [luma_of(frame[y][x]) for x, y in ring]
    band_l = [luma_of(frame[y][x]) for x, y in band]
    return abs(sum(ring_l) / len(ring_l) - sum(band_l) / len(band_l)) if ring_l and band_l else 0.0


def ring_step(sprite, ground, option="shadow", colour=NPC_COLOUR):
    """The other reading of SC 3.2, kept beside it on the board: per ring pixel, |L - mean L of the band pixels within
    BAND of it|, averaged. A dark outline against a bright robe does not cancel here, as it does in `read_contrast`."""
    frame, ring = compose_frame(sprite, ground, option, colour)
    _, _, band = ring_and_band(sprite)
    total, n = 0.0, 0
    for x, y in ring:
        near = [luma_of(frame[y + dy][x + dx]) for dy in range(-BAND, BAND + 1) for dx in range(-BAND, BAND + 1)
                if (x + dx, y + dy) in band]
        if near:
            total += abs(luma_of(frame[y][x]) - sum(near) / len(near))
            n += 1
    return total / n if n else 0.0


def _load(path):
    with Image.open(path) as im:
        return im.convert("RGBA") if im.mode in ("RGBA", "P", "LA") else im.convert("RGB")


def _colour(text):
    v = text.lstrip("#")
    return tuple(int(v[i:i + 2], 16) for i in (0, 2, 4))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("score")
    s.add_argument("png")
    s.add_argument("--class", dest="cls", choices=sorted(LIMITS), required=True)
    f = sub.add_parser("figures")
    f.add_argument("ground_dir")
    f.add_argument("--art", nargs="+", required=True)
    p = sub.add_parser("pair")
    p.add_argument("ground_dir")
    r = sub.add_parser("read")
    r.add_argument("frame")
    r.add_argument("ground")
    r.add_argument("--option", choices=["shadow", "pool", "rim"], default="shadow")
    r.add_argument("--colour", default="e6be50")
    a = ap.parse_args(argv)
    if a.cmd == "score":
        img = _load(a.png)
        warning = size_warning(img)
        if warning:
            print("warning: " + warning, file=sys.stderr)
        result = scores(img)
        for line in table(result, a.cls):
            print(line)
        bad = failures(result, a.cls)
        print("FAIL: " + ", ".join(bad) if bad else "pass")
        return 1 if bad else 0
    if a.cmd == "read":
        frame, ground, colour = _load(a.frame), _load(a.ground), _colour(a.colour)
        print(f"read_contrast {read_contrast(frame, ground, a.option, colour):.1f}, ring step "
              f"{ring_step(frame, ground, a.option, colour):.1f}")
        return 0
    grounds = [(g, _load(path)) for g, path in find_grounds(a.ground_dir)]
    if a.cmd == "pair":
        lines = pair_warnings(grounds)
    else:
        lines = figure_warnings(grounds, [(path, _load(path)) for path in a.art])
    for line in lines:
        print("warning: " + line)
    print(f"{len(grounds)} grounds, {len(lines)} warnings")
    return 0


if __name__ == "__main__":
    sys.exit(main())
