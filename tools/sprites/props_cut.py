# STAGE1-A2: cuts generated prop sheets (objects on a flat magenta ground) into battle props at the sizes of
# games/emberwake/docs/specs/STAGE1.md 7.2: trees 40 x 52 (trunk base 6 px above the bottom, centred at x 20), stones
# 32 x 24 (in the lower half); a reduced palette, a transparent ground and a one-pixel outline in the spec's colour.
# Every object is written as tree_<n>.png / stones_<n>.png in reading order; the picks are copied into
# kits/rpg/assets/props/common/{tree,stones}/v<n>.png (records: art_src/props/stage_*.json).
#   <python with numpy, scipy, Pillow> tools/sprites/props_cut.py <trees sheet> <stones sheet> <out dir>
# (scipy is not in the repo's Python 3.12; the ComfyUI venv has it: C:/dev/localgen/comfyui/venv/Scripts/python.exe)
import sys, pathlib
from PIL import Image
import numpy as np
from scipy import ndimage


def objects(path, close=9, min_px=2000):
    """The sheet's objects: RGBA crops with the magenta ground (and its darker cast shadows) cleared."""
    a = np.asarray(Image.open(path).convert("RGB")).astype(int)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    ground = (r - g > 50) & (b - g > 30)
    fg = ~ground
    fg = ndimage.binary_opening(fg, iterations=1)
    lab, n = ndimage.label(ndimage.binary_closing(fg, iterations=close))
    out = []
    for i, sl in enumerate(ndimage.find_objects(lab), 1):
        m = (lab[sl] == i) & fg[sl]
        if m.sum() < min_px:
            continue
        rgba = np.dstack([a[sl], (m * 255)]).astype(np.uint8)
        out.append((sl[1].start, sl[0].start, Image.fromarray(rgba, "RGBA")))
    out.sort(key=lambda o: (o[1] // 200, o[0]))
    return [o[2] for o in out]

def fit(im, w, h):
    s = min(w / im.width, h / im.height)
    return im.resize((max(1, round(im.width * s)), max(1, round(im.height * s))), Image.LANCZOS)

def finish(im, size, at, outline, colours=12):
    """Places `im` on a transparent `size` canvas with its bottom centre at `at`, reduces its colours, cuts a hard
    alpha and draws a one-pixel outline around the shape."""
    W, H = size
    can = Image.new("RGBA", size, (0, 0, 0, 0))
    can.alpha_composite(im, (at[0] - im.width // 2, at[1] - im.height))
    a = np.asarray(can).copy()
    solid = a[..., 3] >= 128
    rgb = Image.fromarray(a[..., :3]).quantize(colors=colours, method=Image.Quantize.MEDIANCUT).convert("RGB")
    a[..., :3] = np.asarray(rgb); a[..., 3] = solid * 255
    edge = ndimage.binary_dilation(solid) & ~solid
    a[edge] = (*outline, 255)
    return Image.fromarray(a, "RGBA")

def main():
    trees_sheet, stones_sheet, out = sys.argv[1:4]
    OUT = pathlib.Path(out); OUT.mkdir(parents=True, exist_ok=True)
    trees = objects(trees_sheet)
    stones = objects(stones_sheet, close=4)
    print(len(trees), "trees,", len(stones), "stone groups")
    for i, t in enumerate(trees):
        finish(fit(t, 32, 44), (40, 52), (20, 45), (22, 26, 16), colours=15).save(OUT / f"tree_{i}.png")
    for i, s in enumerate(stones):
        finish(fit(s, 18, 9), (32, 24), (16, 22), (24, 24, 30), colours=6).save(OUT / f"stones_{i}.png")

if __name__ == "__main__":
    main()
