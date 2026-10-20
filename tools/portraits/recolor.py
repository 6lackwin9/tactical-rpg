# Recolor (ART2): a new look from one pixelized portrait without new source art. The figure's colours come in shade
# families (pixelize: `shades` colours each); a family is picked by a sample point (the hair at the crown, the armour
# at the chest) and moved to another hue, lightness and saturation as a whole, so its shading steps stay. Small pixel
# edits (a scar removed or added) are stamps of skin shades. Pillow only, so the tool can be ported to Rust.
import colorsys

import pixelize as X


def hls(c): return colorsys.rgb_to_hls(*(v / 255 for v in c[:3]))


def rgb(h, l, s): return tuple(int(v * 255) for v in colorsys.hls_to_rgb(h % 1.0, max(0.0, min(1.0, l)), max(0.0, min(1.0, s))))


def family_of(fig, ramp, shades, point):
    c = fig.getpixel(point)[:3]
    return ramp.index(c) // shades if c in ramp[:-1] else None


def grow(fig, ramp, shades, seeds, allowed, box):
    """The pixels reached from `seeds` through colours of the `allowed` families inside `box` (x0, y0, x1, y1)."""
    px = fig.load(); x0, y0, x1, y1 = box
    fam = lambda c: ramp.index(c[:3]) // shades if c[3] and c[:3] in ramp[:-1] else None
    seen, todo = set(), list(seeds)
    while todo:
        x, y = todo.pop()
        if (x, y) in seen or not (x0 <= x < x1 and y0 <= y < y1) or fam(px[x, y]) not in allowed:
            continue
        seen.add((x, y)); todo.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    return seen


def recolor(fig, ramp, shades, regions):
    """regions: [(seeds, allowed families, box, hue or None, lightness shift, saturation factor)]; a new figure."""
    out = fig.copy(); px = out.load()
    for seeds, allowed, box, hue, dl, sf in regions:
        for x, y in grow(fig, ramp, shades, seeds, allowed, box):
            h, l, s = hls(px[x, y])
            px[x, y] = rgb(h if hue is None else hue, l + dl, s * sf) + (255,)
    return out


def look(src, regions, grid=96, crop=None, shades=4, families=10):
    fig, ramp = X.figure(src, grid, crop, shades, families)
    return X.finish(recolor(fig, ramp, shades, regions))
