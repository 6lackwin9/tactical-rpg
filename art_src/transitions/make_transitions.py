#!/usr/bin/env python3
"""Transition art for TRANSITIONS1 (Art/UI request AI-18), deterministic placeholders in the locked style.

    python art_src/transitions/make_transitions.py

Writes games/emberwake/assets/ui/transitions/:
  ash_wipe_mask.png    1280 x 720 greyscale threshold map. The wipe at progress t (0..1) covers every pixel whose value
                       is below t * 255: it starts as soot blooming from the screen edges and closes on the centre,
                       with a ragged, flaking front (value noise), so one image drives the whole wipe at any length.
  ash_flecks.png       1280 x 720 RGBA ember-and-ash specks drawn over the wipe's front (optional layer).
  objective_banner.png 960 x 132 frame for the fight banner (name, objective, enemy count drawn by the UI).
  victory_banner.png   720 x 168 frame for the Victory banner (word drawn by the UI).
Banners follow STYLE_BIBLE F2: a basalt panel, a 2 px iron line #4b5563 and a 2 px oxblood top line #991b1b; the
banners keep their middle plain so text reads, and fade at both ends so they sit over the stage.
"""
import pathlib

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "games/emberwake/assets/ui/transitions"
IRON, OXBLOOD, PANEL, PANEL_HI = (0x4B, 0x55, 0x63), (0x99, 0x1B, 0x1B), (20, 18, 22), (34, 30, 34)
RNG = np.random.default_rng(455684)


def value_noise(w, h, cell):
    gw, gh = w // cell + 2, h // cell + 2
    g = RNG.random((gh, gw))
    img = Image.fromarray((g * 255).astype("uint8")).resize((gw * cell, gh * cell), Image.BICUBIC)
    return np.asarray(img, dtype=np.float32)[:h, :w] / 255.0


def ash_wipe(w=1280, h=720):
    ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
    # distance to the nearest screen edge, 0 at the edge and 1 at the centre: the soot closes in from the frame
    edge = np.minimum(np.minimum(xs, w - 1 - xs) / (w / 2), np.minimum(ys, h - 1 - ys) / (h / 2))
    n = 0.55 * value_noise(w, h, 160) + 0.3 * value_noise(w, h, 48) + 0.15 * value_noise(w, h, 12)
    v = 0.72 * edge ** 0.8 + 0.28 * n
    v = (v - v.min()) / (v.max() - v.min())
    return Image.fromarray((v * 255).astype("uint8"), "L")


def ash_flecks(w=1280, h=720, count=900):
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
    for _ in range(count):
        x, y = RNG.integers(0, w), RNG.integers(0, h)
        s = int(RNG.choice([1, 1, 2, 2, 3]))
        ember = RNG.random() < 0.18
        col = (217, 119, 6, 230) if ember else tuple(int(c) for c in RNG.integers(70, 120, 3)) + (200,)
        d.rectangle([x, y, x + s, y + s], fill=col)
    return im


def banner(w, h, fade=0.16):
    im = Image.new("RGBA", (w, h), PANEL + (235,)); d = ImageDraw.Draw(im)
    for y in range(h):                       # a faint vertical sheen: lighter at the top, like iron catching tallow
        k = 1 - y / h
        d.line([0, y, w, y], fill=tuple(round(PANEL[i] + (PANEL_HI[i] - PANEL[i]) * k * 0.6) for i in range(3)) + (235,))
    d.rectangle([0, 0, w - 1, h - 1], outline=IRON, width=2)
    d.rectangle([0, 0, w - 1, 1], fill=OXBLOOD)
    d.rectangle([6, 6, w - 7, h - 7], outline=(40, 44, 52), width=1)   # inner hairline
    a = np.asarray(im).astype(np.float32)
    xs = np.arange(w) / w
    ramp = np.clip(np.minimum(xs, 1 - xs) / fade, 0, 1)            # both ends fade into the stage
    a[..., 3] *= ramp[None, :]
    return Image.fromarray(a.astype("uint8"), "RGBA")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    ash_wipe().save(OUT / "ash_wipe_mask.png")
    ash_flecks().filter(ImageFilter.GaussianBlur(0.4)).save(OUT / "ash_flecks.png")
    banner(960, 132).save(OUT / "objective_banner.png")
    banner(720, 168, fade=0.2).save(OUT / "victory_banner.png")
    for f in sorted(OUT.glob("*.png")):
        print("wrote", f.relative_to(ROOT))


if __name__ == "__main__":
    main()
