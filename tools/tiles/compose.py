"""LOOK1-T: compose the 14 transition pieces of a pair from two tiles.

    compose.py --upper U --lower L --size WxH --out DIR

U and L are PNG files or tile folders (their `v0.png`), already W x H. Both are tiled at an offset of half a cell (the
piece is centred on a corner point), the upper tile shows where the mask is set, and the lower tile gets a 1 px rim next
to it at 60 % brightness. Writes opaque `m01.png`-`m14.png` into DIR (use `tilesets/<set>/<upper>/over_<lower>`).
"""
import argparse
import os
import sys

from PIL import Image

import cut
import masks

RIM = 0.6


def load_tile(path, size):
    if os.path.isdir(path):
        path = os.path.join(path, "v0.png")
    img = Image.open(path).convert("RGBA")
    if img.size != size:
        raise ValueError(f"{path} is {img.size[0]}x{img.size[1]}, wanted {size[0]}x{size[1]}")
    return img


def piece(upper, lower, mask):
    w, h = upper.size
    grid = masks.mask_grid(mask, w, h)
    up, lo = upper.load(), lower.load()
    out = Image.new("RGBA", (w, h))
    for y in range(h):
        for x in range(w):
            sx, sy = (x + w // 2) % w, (y + h // 2) % h
            if grid[y][x]:
                c = up[sx, sy]
            else:
                c = lo[sx, sy]
                rim = any(
                    0 <= x + dx < w and 0 <= y + dy < h and grid[y + dy][x + dx]
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
                )
                if rim:
                    c = (int(c[0] * RIM), int(c[1] * RIM), int(c[2] * RIM), 255)
            out.putpixel((x, y), (c[0], c[1], c[2], 255))
    return out


def compose(upper_path, lower_path, size, out_dir):
    upper, lower = load_tile(upper_path, size), load_tile(lower_path, size)
    os.makedirs(out_dir, exist_ok=True)
    for m in range(1, 15):
        piece(upper, lower, m).save(os.path.join(out_dir, f"m{m:02d}.png"))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--upper", required=True)
    ap.add_argument("--lower", required=True)
    ap.add_argument("--size", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    try:
        compose(a.upper, a.lower, cut.parse_size(a.size), a.out)
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    print(f"14 pieces -> {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
