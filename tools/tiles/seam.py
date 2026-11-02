"""LOOK1-T: seam check and wrap fix for a tile.

    seam.py score PNG            prints the ratio (wrap step mean over inner step mean; at most 1.5 is accepted)
    seam.py fix PNG [--out DST]  columns 0 and w - 1 take, per pixel, the colour of column 1 or w - 2 that is nearer
                                 in RGB to the tile's mean; rows alike; then the new score is printed
"""
import argparse
import sys

from PIL import Image


def _d(a, b):
    return (abs(a[0] - b[0]) + abs(a[1] - b[1]) + abs(a[2] - b[2])) / 3


def score(img):
    """Wrap step mean over inner step mean (0 for a flat tile, inf for a flat tile with a seam)."""
    img = img.convert("RGB")
    w, h = img.size
    px = img.load()
    wrap = [_d(px[w - 1, y], px[0, y]) for y in range(h)] + [_d(px[x, h - 1], px[x, 0]) for x in range(w)]
    inner = [_d(px[x, y], px[x + 1, y]) for y in range(h) for x in range(w - 1)]
    inner += [_d(px[x, y], px[x, y + 1]) for y in range(h - 1) for x in range(w)]
    wm = sum(wrap) / len(wrap)
    im = sum(inner) / len(inner)
    if im == 0:
        return 0.0 if wm == 0 else float("inf")
    return wm / im


def mean_colour(img):
    img = img.convert("RGB")
    px = [img.getpixel((x, y)) for y in range(img.size[1]) for x in range(img.size[0])]
    return tuple(sum(p[i] for p in px) / len(px) for i in range(3))


def fix(img):
    img = img.convert("RGB").copy()
    w, h = img.size
    mean = mean_colour(img)
    px = img.load()

    def pick(a, b):
        return a if _d(a, mean) <= _d(b, mean) else b

    for y in range(h):
        c = pick(px[1, y], px[w - 2, y])
        px[0, y], px[w - 1, y] = c, c
    for x in range(w):
        c = pick(px[x, 1], px[x, h - 2])
        px[x, 0], px[x, h - 1] = c, c
    return img


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("cmd", choices=["score", "fix"])
    ap.add_argument("png")
    ap.add_argument("--out", default="")
    a = ap.parse_args(argv)
    img = Image.open(a.png)
    if a.cmd == "score":
        print(f"{score(img):.3f}")
        return 0
    before = score(img)
    out = fix(img)
    dst = a.out or a.png
    out.save(dst)
    print(f"{a.png}: {before:.3f} -> {score(out):.3f} ({dst})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
