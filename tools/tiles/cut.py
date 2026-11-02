"""LOOK1-T: cut a generated image (pixel art drawn at an unknown block size) down to its art-pixel size, scale only.

    cut.py SRC --size WxH [--reduce auto|box] [--colours N] --out DST
    cut.py SRC --form FORM [--colours N] --out DST

--reduce auto (the default): the block size is the largest b for which at least 95 % of the runs of equal colour (RGB
step at most 12) along the rows and columns, border runs left out, are multiples of b. Block centres are sampled. A
uniform border of at most one block on each side is trimmed. The result must be exactly W x H, else the tool exits with
an error.

--reduce box (LOOK1-T2, the web route's images have no clean pixel block): a box reduce of the whole image to W x H, no
block detection. A source smaller than W x H is refused (no upscale).

--form sets the size and the step, always a box reduce in one step from the source. A tile form takes a square and
squashes it when the size is not square: `stage` 64 x 48, `region` 32 x 32, `region_tilted` 32 x 24. A face form crops the
centred band of its aspect across the whole width (no squash) and reduces that: `stage_face` 64 x 24, `region_face`
32 x 16, `region_tilted_face` 32 x 12.

--colours N (1 to 256, default off) leaves at most N colours, every one a colour of the reduced image; no dither, no fixed
palette.
"""
import argparse
import sys

from PIL import Image

STEP = 12
# form: (size, crop the centred band of the size's aspect first; otherwise the source must be a square)
FORMS = {
    "stage": ((64, 48), False),
    "stage_face": ((64, 24), True),
    "region": ((32, 32), False),
    "region_tilted": ((32, 24), False),
    "region_face": ((32, 16), True),
    "region_tilted_face": ((32, 12), True),
}
MAX_COLOURS = 256                # what Pillow's quantize can group
SQUARE_TOL = 0.03                # width over height still taken as a square (artqueue.py's ASPECT_TOL at ingest)


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
        hint = "; no clean pixel block, --reduce box takes it" if b == 1 else ""
        raise ValueError(f"block {b} px: {w}x{h} does not reduce to {tw}x{th} (uniform border of at most one block per "
                         f"side){hint}")
    ox, oy = xs[0], ys[0]
    out = Image.new("RGB", size)
    for y in range(th):
        for x in range(tw):
            out.putpixel((x, y), px[ox + x * b + b // 2, oy + y * b + b // 2])
    return out


def box(img, size):
    """A box reduce of the whole `img` to `size` (W, H), in one step; ValueError when `img` is smaller (no upscale)."""
    w, h = img.size
    if w < size[0] or h < size[1]:
        raise ValueError(f"{w}x{h} is smaller than {size[0]}x{size[1]}: a box reduce does not scale up")
    return img.convert("RGB").resize(size, Image.BOX)


def band(img, size):
    """The centred horizontal band of `img` across its whole width, with the aspect of `size`; ValueError when the
    image is not tall enough."""
    w, h = img.size
    high = round(w * size[1] / size[0])
    if high > h:
        raise ValueError(f"{w}x{h} has no centred band of {w}x{high} ({size[0]}x{size[1]} aspect)")
    top = (h - high) // 2
    return img.crop((0, top, w, top + high))


def cut_form(img, form):
    """The `img` reduced as a form of FORMS; ValueError when a square form gets another shape."""
    size, banded = FORMS[form]
    w, h = img.size
    if banded:
        return box(band(img, size), size)
    if abs(w / h - 1.0) > SQUARE_TOL:
        raise ValueError(f"--form {form} takes a square, this is {w}x{h} (--reduce box --size for another shape)")
    return box(img, size)


def check_colours(n):
    """ValueError when `n` is outside what --colours takes."""
    if not 1 <= n <= MAX_COLOURS:
        raise ValueError(f"--colours {n} is out of range: give 1 to {MAX_COLOURS}")


def limit_colours(img, n):
    """`img` with at most `n` colours, each one a colour of `img`, no dither: median cut with k-means passes groups the
    colours (median cut alone lost the floor's small brown areas), each group takes the image colour nearest its
    average, and each pixel the nearest of those. Fewer colours than `n`: unchanged."""
    check_colours(n)
    img = img.convert("RGB")
    own = [c for _, c in img.getcolors(img.size[0] * img.size[1])]
    if len(own) <= n:
        return img.copy()

    def nearest(colour, among):
        return min(among, key=lambda o: (sum((colour[i] - o[i]) ** 2 for i in range(3)), o))

    grouped = img.quantize(colors=n, method=Image.Quantize.MEDIANCUT, kmeans=4)
    palette = grouped.getpalette()
    picks = sorted({nearest(tuple(palette[3 * i:3 * i + 3]), own) for _, i in grouped.getcolors()})
    table = {c: nearest(c, picks) for c in own}
    out = img.copy()
    px = out.load()
    for y in range(out.size[1]):
        for x in range(out.size[0]):
            px[x, y] = table[px[x, y]]
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
    ap.add_argument("--size", help="WxH (or --form)")
    ap.add_argument("--form", choices=sorted(FORMS), help="sets the size and the step (or --size)")
    ap.add_argument("--reduce", choices=["auto", "box"], help="with --size; default auto")
    ap.add_argument("--colours", type=int, help="at most N of the reduced image's own colours (default off)")
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    try:
        if bool(a.size) == bool(a.form):
            raise ValueError("give one of --size or --form")
        if a.form and a.reduce:
            raise ValueError(f"--form {a.form} sets the step itself, leave out --reduce")
        size = parse_size(a.size) if a.size else None
        if a.colours is not None:
            check_colours(a.colours)
        with Image.open(a.src) as img:
            if a.form:
                out = cut_form(img, a.form)
            elif a.reduce == "box":
                out = box(img, size)
            else:
                out = cut(img, size)
        if a.colours is not None:
            out = limit_colours(out, a.colours)
    except ValueError as e:
        print(f"error: {a.src}: {e}", file=sys.stderr)
        return 1
    out.save(a.out)
    print(f"{a.src} -> {a.out} ({out.size[0]}x{out.size[1]})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
