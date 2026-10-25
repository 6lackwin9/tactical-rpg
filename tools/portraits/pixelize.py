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


def mode_reduce(im, grid, colors=64):
    """Reduces to `grid` x `grid` by the most common colour of each cell (not the mean), so a pixel-art source keeps
    clean, flat clusters instead of blurring its pixels together; a cell mostly background stays clear."""
    w, h = im.size
    alpha = im.getchannel("A")
    q = im.convert("RGB").quantize(colors=colors, method=Image.Quantize.MEDIANCUT)
    pal = q.getpalette(); qp = q.load(); ap = alpha.load()
    out = Image.new("RGBA", (grid, grid), (0, 0, 0, 0)); op = out.load()
    for gy in range(grid):
        y0, y1 = gy * h // grid, (gy + 1) * h // grid
        for gx in range(grid):
            x0, x1 = gx * w // grid, (gx + 1) * w // grid
            count, clear = {}, 0
            for y in range(y0, y1, 2):
                for x in range(x0, x1, 2):
                    if ap[x, y] < 128:
                        clear += 1
                    else:
                        i = qp[x, y]; count[i] = count.get(i, 0) + 1
            if not count or clear > sum(count.values()):
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


def pixelize(src, grid=96, crop=None, shades=4, families=10, size=240, border=(24, 16, 18), trim=(150, 116, 60),
             mark=True):
    """The framed game portrait and its colour count."""
    fig, ramp = figure(src, grid, crop, shades, families, mark)
    return finish(fig, size, border, trim), len(ramp) - 1


def finish(fig, size=240, border=(24, 16, 18), trim=(150, 116, 60)):
    return P.frame(fig, size, BG, dither=True, border=(border, trim))


def figure(src, grid=96, crop=None, shades=4, families=10, mark=True):
    """The reduced, snapped and outlined figure on a clear background, and its ramps (`shades` colours per family,
    then the outline colour). `mark` clears the image model's star first (clear_mark; whole images only, not crops)."""
    im = Image.open(src).convert("RGBA")
    if crop:
        im = im.crop(crop)
    im = cut_background(clear_corner(clear_mark(im) if mark and not crop else im))
    small = mode_reduce(im, grid)
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
