"""LOOK1-T: cut a generated image (pixel art drawn at an unknown block size) down to its art-pixel size, scale only.

    cut.py SRC --size WxH --out DST

The block size is the largest b for which at least 95 % of the runs of equal colour (RGB step at most 12) along the
rows and columns, border runs left out, are multiples of b. Block centres are sampled. A uniform border of at most one
block on each side is trimmed. The result must be exactly W x H, else the tool exits with an error.
"""
import argparse
import sys

from PIL import Image

STEP = 12


def _same(a, b):
    return all(abs(a[i] - b[i]) <= STEP for i in range(3))


def _runs(line):
    out, n = [], 1
    for i in range(1, len(line)):
        if _same(line[i], line[i - 1]):
            n += 1
        else:
            out.append(n)
            n = 1
    out.append(n)
    return out


def block_size(img):
    """Pixel block size of `img` (RGB), 1 when no larger multiple fits."""
    w, h = img.size
    px = img.load()
    runs = []
    for y in range(h):
        runs += _runs([px[x, y] for x in range(w)])[1:-1]
    for x in range(w):
        runs += _runs([px[x, y] for y in range(h)])[1:-1]
    if not runs:
        return 1
    for b in range(max(runs), 1, -1):
        if sum(1 for r in runs if r % b == 0) >= 0.95 * len(runs):
            return b
    return 1


def _split(px, length, other, target, b, get):
    """(lead, trail) border widths along one axis so that length - lead - trail == target, each strip at most one block
    and uniform (every pixel within the RGB step of the strip's first pixel); None when no split fits."""
    extra = length - target
    if extra < 0 or extra > 2 * b:
        return None
    for lead in sorted(range(max(0, extra - b), min(b, extra) + 1), key=lambda k: abs(k - extra / 2)):
        trail = extra - lead
        ok = True
        for lo, hi in ((0, lead), (length - trail, length)):
            if hi > lo:
                ref = get(lo, 0)
                ok = ok and all(_same(get(i, j), ref) for i in range(lo, hi) for j in range(other))
        if ok:
            return lead, trail
    return None


def cut(img, size):
    """The `img` reduced to `size` (W, H) art pixels; ValueError when it does not fit.

    The border on each side (at most one block, uniform in colour) is found from the pixels, not assumed centred."""
    img = img.convert("RGB")
    b = block_size(img)
    w, h = img.size
    tw, th = size
    px = img.load()
    xs = _split(px, w, h, tw * b, b, lambda i, j: px[i, j])
    ys = _split(px, h, w, th * b, b, lambda i, j: px[j, i])
    if xs is None or ys is None:
        raise ValueError(f"block {b} px: {w}x{h} does not reduce to {tw}x{th} (uniform border of at most one block per side)")
    ox, oy = xs[0], ys[0]
    out = Image.new("RGB", size)
    for y in range(th):
        for x in range(tw):
            out.putpixel((x, y), px[ox + x * b + b // 2, oy + y * b + b // 2])
    return out


def parse_size(text):
    """\"WxH\" as (W, H); ValueError with a message when malformed."""
    try:
        w, h = text.lower().split("x")
        size = (int(w), int(h))
    except ValueError:
        raise ValueError(f"size '{text}' is not WxH (e.g. 32x24)") from None
    if size[0] < 1 or size[1] < 1:
        raise ValueError(f"size '{text}' must be positive")
    return size


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("src")
    ap.add_argument("--size", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    try:
        size = parse_size(a.size)
        with Image.open(a.src) as img:
            out = cut(img, size)
    except ValueError as e:
        print(f"error: {a.src}: {e}", file=sys.stderr)
        return 1
    out.save(a.out)
    print(f"{a.src} -> {a.out} ({a.size})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
