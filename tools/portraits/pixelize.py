# Pixelize (ART2): turns a source portrait (an image model's or an artist's, on a flat dark background) into a game
# portrait: the corner watermark cleared, the background cut away by a flood fill from the edges, the figure reduced to
# the pixel grid, its colours snapped to ramps of a few shades per colour family, outlined, then framed like every
# other portrait (the dithered glow and border of portrait.frame) and scaled up to the portrait size.
# Pillow only, so the tool can be ported to Rust (decision "Art tools in Python, portable to Rust").
import colorsys
from collections import deque
from PIL import Image

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


BG = [(18, 14, 18), (30, 22, 26), (46, 30, 30), (64, 38, 32)]


def pixelize(src, grid=GRID, crop=None, shades=4, families=10, size=240, border=(24, 16, 18), trim=(150, 116, 60),
             mark=True, accents=True, rim=True):
    """The framed game portrait and its colour count."""
    fig, ramp = figure(src, grid, crop, shades, families, mark, accents, rim)
    return finish(fig, size, border, trim), len(ramp) - 1


def finish(fig, size=240, border=(24, 16, 18), trim=(150, 116, 60)):
    return P.frame(fig, size, BG, dither=True, border=(border, trim))


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


def figure(src, grid=GRID, crop=None, shades=4, families=10, mark=True, accents=True, rim=True):
    """The reduced, snapped and outlined figure on a clear background, and its ramps (`shades` colours per family,
    the accent families, then the outline colour). `mark` clears the image model's star first (clear_mark; whole
    images only, not crops); `accents` keeps small saturated colours of their own hue (accent_bins) in families of
    their own; `rim` lifts the dark silhouette pixels (lift_rim)."""
    im = Image.open(src).convert("RGBA")
    if crop:
        im = im.crop(crop)
    im = cut_background(clear_corner(clear_mark(im) if mark and not crop else im))
    bins = accent_bins(im) if accents else set()
    small = mode_reduce(im, grid, accents=bins)
    px = small.load()
    # the figure's own colours, then its ramps; accent pixels apart
    opaque = Image.new("RGB", (grid, grid))
    op = opaque.load(); mask = []; acc = []
    for y in range(grid):
        for x in range(grid):
            r, g, b, a = px[x, y]
            if a >= 128:
                op[x, y] = (r, g, b)
                (acc if is_accent((r, g, b), bins) else mask).append((x, y))
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
    if rim:
        lift_rim(fp, grid, ramp, shades)
    # outline the silhouette
    ring = [(x, y) for y in range(grid) for x in range(grid) if fp[x, y][3] == 0 and
            any(0 <= x + dx < grid and 0 <= y + dy < grid and fp[x + dx, y + dy][3] for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
    for x, y in ring:
        fp[x, y] = P.OUT + (255,)
    return fig, ramp


if __name__ == "__main__":
    import sys
    src, out = sys.argv[1], sys.argv[2]
    grid = int(sys.argv[3]) if len(sys.argv) > 3 else GRID
    img, n = pixelize(src, grid)
    img.save(out)
    print(out, grid, n, "colours")
