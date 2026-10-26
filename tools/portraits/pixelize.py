# Pixelize (ART2): turns a source portrait (an image model's or an artist's, on a flat dark background) into a game
# portrait: the corner watermark cleared, the background cut away by a flood fill from the edges, the figure reduced to
# the pixel grid, its colours snapped to ramps of a few shades per colour family, outlined, then framed like every
# other portrait (the dithered backdrop and border of portrait.frame) and scaled up to the portrait size.
# Since PIXEL3 (decision "Portrait backdrops and rims", user 2026-10-27) the backdrop is the character's faction ramp
# (BACKDROPS; the `faction` key of its art_src/<id>.json record) in the "corner" style, and a dark silhouette pixel
# that would merge with the backdrop behind it is lifted up its own ramp (edge_rim).
# Pillow only, so the tool can be ported to Rust (decision "Art tools in Python, portable to Rust").
# The reduction to the grid is "mode" (most common colour per cell, 10 x 4 palette) for a true pixel-art source and
# "smooth" (Lanczos, 14 x 5) for a smooth one; "auto" picks by the source's pixel grid (grid_period).
# CLI: pixelize.py SRC OUT [GRID] [--faction F] [--backdrop corner|glow] [--rim edge|lift|none] [--no-mark]
#      [--reduce auto|mode|smooth] (no --faction: the `faction` of the record beside SRC, else hearth)
import argparse
import colorsys
import json
import pathlib
from collections import deque
from PIL import Image, ImageChops

import portrait as P

BG_TOL = 34          # how far from the background colour still counts as background (RGB distance)
GRID = 120           # the game portrait grid, 2x into the 240 px frame (decision "Portrait grid 120 px", was 96)
# Accents (PIXEL2): a small, saturated colour of its own hue (a violet vein glow, a gem, a lit eye) keeps a family.
ACCENT_SAT = 0.55              # HLS saturation from which a colour can be an accent
ACCENT_LIGHT = (0.30, 0.90)    # HLS lightness range of an accent (not a near-black, not a white glint)
ACCENT_SHARE = (0.0005, 0.03)  # share of the figure's pixels: rarer is noise, commoner is a main colour (gold armour)
ACCENT_CELL = 0.15             # share of a grid cell's figure samples that makes the cell an accent
HUE_BINS = 24
# Rim (PIXEL2): a dark figure pixel on the silhouette is lifted up its own ramp, so dark hair does not merge with the
# dark outline and background.
RIM_DARK = 0.26                # HLS lightness under which a silhouette pixel is lifted
# Faction backdrops (PIXEL3): each a ramp from dark to light, tunable (C:/dev/localgen/probe/faction_bg2.py tested
# hearth, castellan, clerisy and guild). hearth is the PIXEL2 background.
BACKDROPS = {
    "hearth": [(18, 14, 18), (30, 22, 26), (46, 30, 30), (64, 38, 32)],      # warm brown
    "castellan": [(20, 10, 12), (36, 14, 16), (56, 18, 20), (80, 24, 24)],   # deep red
    "clerisy": [(18, 10, 28), (30, 14, 48), (46, 20, 72), (64, 28, 98)],     # saturated dark violet
    "guild": [(10, 18, 10), (16, 30, 14), (24, 44, 18), (34, 60, 22)],       # rich dark green
    "corrupted": [(16, 10, 20), (26, 14, 34), (40, 20, 52), (56, 26, 72)],   # violet-black
    "creature": [(10, 16, 18), (14, 24, 28), (20, 34, 40), (26, 46, 54)],    # cold dark teal
}
FACTIONS = tuple(BACKDROPS)
DEFAULT_FACTION = "hearth"
BACKDROP_STYLES = ("corner", "glow")
# The edge rim (PIXEL3, user 2026-10-27: backdrop-aware and even, from the portrait's own palette, no faction tint): a
# pixel on the outer silhouette (next to the backdrop, not the frame's edge and not a hole inside the figure) needs a
# lift when its luma (Rec. 601) is within RIM_CONTRAST of the backdrop pixel directly behind it; it needs as many
# shades up its own family's ramp as reach that contrast, at most RIM_SHADES. Needing pixels form runs (8-connected
# along the edge); a run shorter than RIM_MIN_RUN is left (no speckles), a longer one is lifted as a whole by its
# largest need, so it reads as one clean highlight. An edge already apart from its backdrop (over the lighter corners,
# or the lit side) is untouched.
RIM_CONTRAST = 24              # luma difference (0-255) from which an edge stands apart from the backdrop
RIM_SHADES = 2
RIM_MIN_RUN = 3
RIMS = ("edge", "lift", "none")
# The reduction (PIXEL3, user 2026-10-27): a true pixel-art source (the Gemini cast: a steady pixel grid) keeps the
# most-common-colour reduction and 10 families x 4 shades; a smooth source (a local Klein image) loses thin features to
# that per-cell vote, so it takes a smooth downscale (Lanczos, the alpha box-resized and thresholded) and a richer
# palette, 14 x 5. "auto" picks by grid_period.
REDUCES = ("auto", "mode", "smooth")
PALETTE = {"mode": (10, 4), "smooth": (14, 5)}   # (families, shades)
GRID_SCORE = 0.15        # comb score from which a source has a steady pixel grid (Gemini about 0.5-0.8, Klein under 0.02)
GRID_PERIODS = (3.0, 40.0)   # the source pixel sizes searched (source px per art pixel)
GRID_LAGS = 64           # autocorrelation lags looked at


def clear_corner(im, frac=0.08):
    """Removes an image model's mark from the bottom-right corner: its pale, colourless pixels take the colour of the
    picture a little further left (over the background that is the background, over the figure the figure)."""
    w, h = im.size
    x0, y0 = int(w * (1 - frac)), int(h * (1 - frac)); shift = int(w * frac * 0.9)
    px = im.load()
    for y in range(y0, h):
        for x in range(x0, w):
            r, g, b = px[x, y][:3]
            hi, lo = max(r, g, b), min(r, g, b)
            if hi > 120 and hi - lo < 28:
                px[x, y] = px[x - shift, y]
    return im


def clear_mark(im, cx=0.946, cy=0.946, half=0.03, ground=0.9):
    """Removes the image model's four-point star mark where it sits on the background (ART3): in the small box round
    the mark's place, when nearly all of the box's edge is background, every colourless pixel in it that is not
    already background (the star and its soft rim) becomes the background colour. A box the figure crosses is left
    to clear_corner. Runs before clear_corner, so that one finds nothing pale left in the box."""
    w, h = im.size; px = im.load(); bg = px[2, 2][:3]
    x0, x1 = int(w * (cx - half)), int(w * (cx + half)); y0, y1 = int(h * (cy - half)), int(h * (cy + half))
    near = lambda c: sum((a - b) ** 2 for a, b in zip(c[:3], bg)) <= BG_TOL ** 2
    edge = [(x, y0) for x in range(x0, x1)] + [(x, y1 - 1) for x in range(x0, x1)] + \
           [(x0, y) for y in range(y0, y1)] + [(x1 - 1, y) for y in range(y0, y1)]
    if sum(near(px[x, y]) for x, y in edge) < ground * len(edge):
        return im
    for y in range(y0, y1):
        for x in range(x0, x1):
            c = px[x, y]
            if not near(c) and max(c[:3]) - min(c[:3]) < 28:
                px[x, y] = bg + tuple(c[3:])
    return im


def cut_background(im):
    """Alpha 0 on the background: a flood fill from every edge pixel through colours near the corner colour."""
    w, h = im.size; px = im.load(); bg = px[2, 2][:3]
    near = lambda c: sum((a - b) ** 2 for a, b in zip(c[:3], bg)) <= BG_TOL ** 2
    seen = bytearray(w * h); q = deque()
    for x in range(w):
        q.append((x, 0)); q.append((x, h - 1))
    for y in range(h):
        q.append((0, y)); q.append((w - 1, y))
    while q:
        x, y = q.popleft()
        if not (0 <= x < w and 0 <= y < h) or seen[y * w + x]:
            continue
        seen[y * w + x] = 1
        if not near(px[x, y]):
            continue
        px[x, y] = (0, 0, 0, 0)
        q.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    return im


def hls_of(c):
    return colorsys.rgb_to_hls(*(v / 255 for v in c[:3]))


def hue_bin(h):
    return int(h * HUE_BINS) % HUE_BINS


def saturated(c):
    h, l, s = hls_of(c)
    return s >= ACCENT_SAT and ACCENT_LIGHT[0] <= l <= ACCENT_LIGHT[1]


def accent_bins(im):
    """The hue bins of the accents: saturated figure colours whose hue (with two neighbour bins on each side) is a
    small share of the figure, so a thin glow counts and a gold breastplate does not."""
    px = im.load(); w, h = im.size
    count = [0] * HUE_BINS; total = 0
    for y in range(0, h, 2):
        for x in range(0, w, 2):
            c = px[x, y]
            if c[3] < 128:
                continue
            total += 1
            if saturated(c):
                count[hue_bin(hls_of(c)[0])] += 1
    if not total:
        return set()
    lo, hi = ACCENT_SHARE
    near = lambda b: sum(count[(b + d) % HUE_BINS] for d in range(-2, 3))
    return {b for b in range(HUE_BINS) if count[b] >= lo * total and near(b) <= hi * total}


def is_accent(c, bins):
    return bool(bins) and saturated(c) and hue_bin(hls_of(c)[0]) in bins


def mode_reduce(im, grid, colors=64, accents=None):
    """Reduces to `grid` x `grid` by the most common colour of each cell (not the mean), so a pixel-art source keeps
    clean, flat clusters instead of blurring its pixels together; a cell mostly background stays clear. `accents`
    (accent_bins): a cell whose samples are ACCENT_CELL accent takes their mean colour, so a thin glow survives."""
    w, h = im.size
    alpha = im.getchannel("A")
    q = im.convert("RGB").quantize(colors=colors, method=Image.Quantize.MEDIANCUT)
    pal = q.getpalette(); qp = q.load(); ap = alpha.load()
    out = Image.new("RGBA", (grid, grid), (0, 0, 0, 0)); op = out.load()
    full = im.load() if accents else None
    for gy in range(grid):
        y0, y1 = gy * h // grid, (gy + 1) * h // grid
        for gx in range(grid):
            x0, x1 = gx * w // grid, (gx + 1) * w // grid
            count, clear, acc = {}, 0, []
            for y in range(y0, y1, 2):
                for x in range(x0, x1, 2):
                    if ap[x, y] < 128:
                        clear += 1
                    else:
                        i = qp[x, y]; count[i] = count.get(i, 0) + 1
                        if full is not None and is_accent(full[x, y], accents):
                            acc.append(full[x, y])
            if not count or clear > sum(count.values()):
                continue
            if acc and len(acc) >= ACCENT_CELL * sum(count.values()):
                op[gx, gy] = tuple(sum(c[k] for c in acc) // len(acc) for k in range(3)) + (255,)
                continue
            i = max(count, key=count.get)
            op[gx, gy] = tuple(pal[i * 3:i * 3 + 3]) + (255,)
    return out


def ramps(colors, families=7, shades=4):
    """Groups the colours into hue families (greys apart) and gives each family `shades` evenly spaced lightness
    levels at its mean hue and saturation: the ramps the figure is snapped to."""
    hls = [colorsys.rgb_to_hls(*(v / 255 for v in c)) for c in colors]
    greys = [i for i, (h, l, s) in enumerate(hls) if s < 0.12]
    hued = sorted((i for i in range(len(colors)) if i not in greys), key=lambda i: hls[i][0])
    groups = [greys] if greys else []
    if hued:
        k = max(1, families - len(groups)); step = max(1, len(hued) // k)
        groups += [hued[j:j + step] for j in range(0, len(hued), step)]
    out = []
    for g in groups:
        if not g:
            continue
        h = sum(hls[i][0] for i in g) / len(g); s = sum(hls[i][2] for i in g) / len(g)
        if g is greys:
            s = min(s, 0.05)   # greys stay neutral (steel, mail), not tinted by their average hue
        lo = min(hls[i][1] for i in g); hi = max(hls[i][1] for i in g)
        for k in range(shades):
            l = lo + (hi - lo) * k / max(1, shades - 1)
            out.append(tuple(int(v * 255) for v in colorsys.hls_to_rgb(h, l, s)))
    return out


BG = BACKDROPS["hearth"]


def faction_key(faction):
    """The faction's table key: None is the default (hearth); an unknown name is an error, not a silent fallback."""
    f = faction or DEFAULT_FACTION
    if f not in BACKDROPS:
        raise ValueError(f"unknown faction {f!r} (one of {', '.join(FACTIONS)})")
    return f


def faction_of(src):
    """The `faction` of the record beside a source (`<stem>.json` next to `<stem>.png`), else None (hearth)."""
    rec = pathlib.Path(src).with_suffix(".json")
    if not rec.is_file():
        return None
    return json.loads(rec.read_text(encoding="utf-8")).get("faction")


def edge_profile(im):
    """The horizontal edge profile: per column, the mean brightness step to the next column."""
    g = im.convert("L"); w, h = g.size
    d = ImageChops.difference(g.crop((0, 0, w - 1, h)), g.crop((1, 0, w, h))).resize((w - 1, 1), Image.BOX)
    return list(d.tobytes())


def grid_period(im):
    """(score, period) of the steadiest pixel grid in the source: the edge profile's autocorrelation, high-passed
    (minus its 7-lag mean), averaged over the lags round(m * p) of a period p in GRID_PERIODS (step 0.05), each of
    them a local peak. A pixel-art source (art pixels of a steady size) scores high at its pixel size; a smooth image
    has no such comb. (0.0, None) when nothing qualifies."""
    e = edge_profile(im); n = len(e); kmax = GRID_LAGS
    if n <= kmax + 4:
        return 0.0, None
    m = sum(e) / n; z = [v - m for v in e]
    c0 = sum(v * v for v in z)
    if not c0:
        return 0.0, None
    ac = [sum(z[i] * z[i + k] for i in range(n - k)) / c0 for k in range(kmax + 4)]
    hp = []
    for k in range(kmax + 1):
        win = ac[max(0, k - 3):k + 4]
        hp.append(ac[k] - sum(win) / len(win))
    best = (0.0, None)
    lo, hi = GRID_PERIODS
    for i in range(int(lo * 20), int(hi * 20) + 1):
        p = i / 20
        ks = [round(j * p) for j in range(1, int(kmax / p) + 1)]
        if len(ks) < 3 or not all(hp[k] >= max(hp[k - 1], hp[k + 1]) for k in ks if 0 < k < kmax):
            continue
        s = sum(hp[k] for k in ks) / len(ks)
        if s > best[0]:
            best = (s, p)
    return best


def reduce_of(src, crop=None, reduce="auto"):
    """The reduction a source gets: `reduce` itself, or for "auto" "mode" when grid_period finds a steady pixel grid
    (GRID_SCORE) and "smooth" otherwise."""
    if reduce not in REDUCES:
        raise ValueError(f"unknown reduce {reduce!r} (one of {', '.join(REDUCES)})")
    if reduce != "auto":
        return reduce
    im = Image.open(src)
    if crop:
        im = im.crop(crop)
    return "mode" if grid_period(im)[0] >= GRID_SCORE else "smooth"


def smooth_reduce(im, grid):
    """A smooth downscale to `grid` x `grid`: RGB by Lanczos premultiplied by alpha (so the cleared background adds
    no colour and no ringing fringe to the silhouette), alpha by box average thresholded at 128."""
    a = im.getchannel("A").resize((grid, grid), Image.BOX).point(lambda v: 255 if v >= 128 else 0)
    out = im.convert("RGBa").resize((grid, grid), Image.LANCZOS).convert("RGBA")
    out.putalpha(a)
    return out


def near_silhouette(px, grid, x, y):
    """True when a clear pixel (or the grid's edge) is within one grid pixel of (x, y), diagonals included."""
    return any(not (0 <= x + dx < grid and 0 <= y + dy < grid) or px[x + dx, y + dy][3] < 128
               for dx in (-1, 0, 1) for dy in (-1, 0, 1))


def pixelize(src, grid=GRID, crop=None, shades=None, families=None, size=240, border=(24, 16, 18),
             trim=(150, 116, 60), mark=True, accents=True, rim=True, faction=None, backdrop="corner", reduce="auto"):
    """The framed game portrait and its colour count. `faction` picks the backdrop ramp (None: hearth); `backdrop`
    "corner" or "glow" (the PIXEL2 look); `rim` and `reduce` see figure. faction=None, backdrop="glow", rim="lift",
    reduce="mode" is the PIXEL2 portrait."""
    fig, ramp = figure(src, grid, crop, shades, families, mark, accents, rim, faction, reduce=reduce, backdrop=backdrop)
    return finish(fig, size, border, trim, faction, backdrop), len(ramp) - 1


def finish(fig, size=240, border=(24, 16, 18), trim=(150, 116, 60), faction=None, backdrop="corner"):
    if backdrop not in BACKDROP_STYLES:
        raise ValueError(f"unknown backdrop {backdrop!r} (one of {', '.join(BACKDROP_STYLES)})")
    return P.frame(fig, size, BACKDROPS[faction_key(faction)], dither=True, border=(border, trim), style=backdrop)


def accent_ramps(colors, bins, shades=4):
    """One family per run of neighbouring accent hue bins: `shades` lightness levels over its pixels' range at their
    mean hue and saturation."""
    start = {}
    for b in bins:
        a = b
        while (a - 1) % HUE_BINS in bins and (a - 1) % HUE_BINS != b:
            a = (a - 1) % HUE_BINS
        start[b] = a
    groups = {}
    for c in colors:
        v = hls_of(c)
        groups.setdefault(start.get(hue_bin(v[0]), hue_bin(v[0])), []).append(v)
    out = []
    for b in sorted(groups):
        g = groups[b]
        h = sum(v[0] for v in g) / len(g); s = sum(v[2] for v in g) / len(g)
        lo = min(v[1] for v in g); hi = max(v[1] for v in g)
        for k in range(shades):
            l = lo + (hi - lo) * k / max(1, shades - 1)
            out.append(tuple(int(v * 255) for v in colorsys.hls_to_rgb(h, l, s)))
    return out


def nearest(ramp, c):
    r, g, b = c[:3]
    return min(ramp, key=lambda q: (q[0] - r) ** 2 * 3 + (q[1] - g) ** 2 * 4 + (q[2] - b) ** 2 * 2)


def lift_rim(fp, grid, ramp, shades):
    """A dark figure pixel next to the clear background moves up its family's ramp (one shade at a time, at most to
    the top) until it is no darker than RIM_DARK: a one pixel rim in the local colour, not a halo."""
    fams = len(ramp) - 1
    edge = [(x, y) for y in range(grid) for x in range(grid) if fp[x, y][3] and
            any(not (0 <= x + dx < grid and 0 <= y + dy < grid) or fp[x + dx, y + dy][3] == 0
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
    for x, y in edge:
        c = fp[x, y][:3]
        if c not in ramp[:fams]:
            continue
        i = ramp.index(c); top = (i // shades) * shades + shades - 1
        while i < top and hls_of(ramp[i])[1] < RIM_DARK:
            i += 1
        fp[x, y] = ramp[i] + (255,)


def outer_silhouette(fp, grid):
    """The figure pixels that border the backdrop: a 4-neighbour in the clear area reached from the grid's edge.
    Not the pixels on the frame's edge (the cut-off shoulders) and not those round a hole inside the figure."""
    outside = bytearray(grid * grid); q = deque()
    for i in range(grid):
        q.extend(((i, 0), (i, grid - 1), (0, i), (grid - 1, i)))
    while q:
        x, y = q.popleft()
        if not (0 <= x < grid and 0 <= y < grid) or outside[y * grid + x] or fp[x, y][3]:
            continue
        outside[y * grid + x] = 1
        q.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    return [(x, y) for y in range(grid) for x in range(grid) if fp[x, y][3] and
            any(0 <= x + dx < grid and 0 <= y + dy < grid and outside[(y + dy) * grid + x + dx]
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]


def luma(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def rim_need(c, ramp, shades, back):
    """How many shades up its own family's ramp colour `c` must move to stand RIM_CONTRAST apart (in luma) from the
    backdrop colour `back`: 0 when it already does, or when it is the outline colour or outside the ramps (they stay);
    at most RIM_SHADES and never past the family's top (the cap when even that does not reach the contrast)."""
    fams = len(ramp) - 1
    if c not in ramp[:fams] or abs(luma(c) - luma(back)) >= RIM_CONTRAST:
        return 0
    i = ramp.index(c); cap = min((i // shades) * shades + shades - 1 - i, RIM_SHADES)
    for n in range(1, cap + 1):
        if abs(luma(ramp[i + n]) - luma(back)) >= RIM_CONTRAST:
            return n
    return cap


def edge_rim(fp, grid, ramp, shades, bg, style):
    """The edge rim (PIXEL3): the outer silhouette's pixels that need a lift against the backdrop pixel behind them
    (rim_need against portrait.backdrop_pixel of ramp `bg` in `style`) are grouped into 8-connected runs; a run of at
    least RIM_MIN_RUN pixels moves up its own ramps by its largest need (each pixel at most to its family's top), a
    shorter one stays. The inside of the figure is untouched."""
    need = {}
    for x, y in outer_silhouette(fp, grid):
        n = rim_need(fp[x, y][:3], ramp, shades, P.backdrop_pixel(grid, x, y, bg, style))
        if n:
            need[x, y] = n
    seen = set()
    for start in sorted(need, key=lambda p: (p[1], p[0])):
        if start in seen:
            continue
        run, todo = [], [start]
        seen.add(start)
        while todo:
            x, y = todo.pop(); run.append((x, y))
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    q = (x + dx, y + dy)
                    if q in need and q not in seen:
                        seen.add(q); todo.append(q)
        if len(run) < RIM_MIN_RUN:
            continue
        lift = max(need[p] for p in run)
        for x, y in run:
            i = ramp.index(fp[x, y][:3]); top = (i // shades) * shades + shades - 1
            fp[x, y] = ramp[min(top, i + lift)] + (255,)


def figure(src, grid=GRID, crop=None, shades=None, families=None, mark=True, accents=True, rim=True, faction=None,
           outline=True, reduce="auto", backdrop="corner", info=None):
    """The reduced, snapped and outlined figure on a clear background, and its ramps (`shades` colours per family,
    the accent families, then the outline colour). `mark` clears the image model's star first (clear_mark; whole
    images only, not crops); `accents` keeps small saturated colours of their own hue (accent_bins) in families of
    their own; `rim`: True or "edge" the edge rim against the backdrop the figure will sit on (edge_rim, PIXEL3:
    `faction` and `backdrop` name it), "lift" the PIXEL2 lift (lift_rim), False or "none" no rim; `outline` False
    leaves the dark outline ring off (the silhouette as the rim saw it); `reduce` "mode" (mode_reduce), "smooth"
    (smooth_reduce) or "auto" (by grid_period); `families` and `shades` default to the reduction's PALETTE. `info`
    (a dict) is filled with the reduction used ("reduce") and the accent ramps ("accent_ramp")."""
    rim = "edge" if rim is True else "none" if rim is False else rim
    if rim not in RIMS:
        raise ValueError(f"unknown rim {rim!r} (one of {', '.join(RIMS)})")
    if backdrop not in BACKDROP_STYLES:
        raise ValueError(f"unknown backdrop {backdrop!r} (one of {', '.join(BACKDROP_STYLES)})")
    bg = BACKDROPS[faction_key(faction)]
    if reduce not in REDUCES:
        raise ValueError(f"unknown reduce {reduce!r} (one of {', '.join(REDUCES)})")
    im = Image.open(src).convert("RGBA")
    if crop:
        im = im.crop(crop)
    if reduce == "auto":
        reduce = "mode" if grid_period(im)[0] >= GRID_SCORE else "smooth"
    families = families or PALETTE[reduce][0]
    shades = shades or PALETTE[reduce][1]
    im = cut_background(clear_corner(clear_mark(im) if mark and not crop else im))
    bins = accent_bins(im) if accents else set()
    small = mode_reduce(im, grid, accents=bins) if reduce == "mode" else smooth_reduce(im, grid)
    px = small.load()
    # the figure's own colours, then its ramps; accent pixels apart
    opaque = Image.new("RGB", (grid, grid))
    op = opaque.load(); mask = []; acc = []
    for y in range(grid):
        for x in range(grid):
            r, g, b, a = px[x, y]
            if a >= 128:
                op[x, y] = (r, g, b)
                # the smooth path takes no accents next to the silhouette: a fringe there is not a glow
                edge = reduce == "smooth" and near_silhouette(px, grid, x, y)
                (acc if not edge and is_accent((r, g, b), bins) else mask).append((x, y))
    q = opaque.quantize(colors=40, method=Image.Quantize.MEDIANCUT)
    pal = q.getpalette()[:40 * 3]
    colors = [tuple(pal[i:i + 3]) for i in range(0, len(pal), 3)]
    main = ramps(colors, families, shades)
    extra = accent_ramps([op[x, y] for x, y in acc], bins, shades)
    ramp = main + extra + [P.OUT]
    fig = Image.new("RGBA", (grid, grid), (0, 0, 0, 0)); fp = fig.load()
    for x, y in mask:
        fp[x, y] = nearest(main + [P.OUT], op[x, y]) + (255,)
    for x, y in acc:
        fp[x, y] = nearest(extra, op[x, y]) + (255,)
    if info is not None:
        info.update(reduce=reduce, accent_ramp=list(extra))
    if rim == "lift":
        lift_rim(fp, grid, ramp, shades)
    elif rim == "edge":
        edge_rim(fp, grid, ramp, shades, bg, backdrop)
    if not outline:
        return fig, ramp
    # outline the silhouette
    ring = [(x, y) for y in range(grid) for x in range(grid) if fp[x, y][3] == 0 and
            any(0 <= x + dx < grid and 0 <= y + dy < grid and fp[x + dx, y + dy][3] for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
    for x, y in ring:
        fp[x, y] = P.OUT + (255,)
    return fig, ramp


def main(argv=None):
    p = argparse.ArgumentParser(description="A source portrait to the framed game portrait.")
    p.add_argument("src"); p.add_argument("out")
    p.add_argument("grid", nargs="?", type=int, default=GRID)
    p.add_argument("--faction", choices=FACTIONS, help="the backdrop (default: the record beside SRC, else hearth)")
    p.add_argument("--backdrop", choices=BACKDROP_STYLES, default="corner")
    p.add_argument("--rim", choices=RIMS, default="edge")
    p.add_argument("--no-mark", action="store_true", help="the source has no image model's star mark to clear")
    p.add_argument("--reduce", choices=REDUCES, default="auto",
                   help="mode for pixel-art sources, smooth for smooth ones (default: auto, by the source's pixel grid)")
    a = p.parse_args(argv)
    faction = a.faction or faction_of(a.src)
    reduce = reduce_of(a.src, reduce=a.reduce)
    img, n = pixelize(a.src, a.grid, mark=not a.no_mark, rim=a.rim, faction=faction, backdrop=a.backdrop, reduce=reduce)
    img.save(a.out)
    print(a.out, a.grid, n, "colours", faction_key(faction), a.backdrop, reduce)


if __name__ == "__main__":
    main()
