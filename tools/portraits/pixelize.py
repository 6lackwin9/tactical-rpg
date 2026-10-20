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


def pixelize(src, grid=96, crop=None, shades=4, families=10, size=240, border=(24, 16, 18), trim=(150, 116, 60)):
    """The framed game portrait and its colour count."""
    fig, ramp = figure(src, grid, crop, shades, families)
    return finish(fig, size, border, trim), len(ramp) - 1


def finish(fig, size=240, border=(24, 16, 18), trim=(150, 116, 60)):
    return P.frame(fig, size, BG, dither=True, border=(border, trim))


def figure(src, grid=96, crop=None, shades=4, families=10):
    """The reduced, snapped and outlined figure on a clear background, and its ramps (`shades` colours per family,
    then the outline colour)."""
    im = Image.open(src).convert("RGBA")
    if crop:
        im = im.crop(crop)
    im = cut_background(clear_corner(im))
    small = im.resize((grid, grid), Image.BOX)
    px = small.load()
    # the figure's own colours, then its ramps
    opaque = Image.new("RGB", (grid, grid))
    op = opaque.load(); mask = []
    for y in range(grid):
        for x in range(grid):
            r, g, b, a = px[x, y]
            if a >= 128:
                op[x, y] = (r, g, b); mask.append((x, y))
    q = opaque.quantize(colors=40, method=Image.Quantize.MEDIANCUT)
    pal = q.getpalette()[:40 * 3]
    colors = [tuple(pal[i:i + 3]) for i in range(0, len(pal), 3)]
    ramp = ramps(colors, families, shades) + [P.OUT]
    fig = Image.new("RGBA", (grid, grid), (0, 0, 0, 0)); fp = fig.load()
    for x, y in mask:
        r, g, b = op[x, y]
        fp[x, y] = min(ramp, key=lambda c: (c[0] - r) ** 2 * 3 + (c[1] - g) ** 2 * 4 + (c[2] - b) ** 2 * 2) + (255,)
    # outline the silhouette
    ring = [(x, y) for y in range(grid) for x in range(grid) if fp[x, y][3] == 0 and
            any(0 <= x + dx < grid and 0 <= y + dy < grid and fp[x + dx, y + dy][3] for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
    for x, y in ring:
        fp[x, y] = P.OUT + (255,)
    return fig, ramp


if __name__ == "__main__":
    import sys
    src, out = sys.argv[1], sys.argv[2]
    grid = int(sys.argv[3]) if len(sys.argv) > 3 else 96
    img, n = pixelize(src, grid)
    img.save(out)
    print(out, grid, n, "colours")
