#!/usr/bin/env python3
# The act-map integration pass (Art/UI 2026-11-13): landmarks blended into their act background by an image model, then
# cut back out so the game still draws each landmark as its own sprite (hover, selection glow, the grey locked look and
# the quest badge need that) exactly over its painted copy.
#
#   python art_src/prompts/blend_act_map.py compose <layout.json> <out.png>
#       every landmark cut from its source (with its fixes), sized and placed as the game's map draws it, on the
#       background at the background's own resolution; no labels, routes, badges or anything else the game adds.
#       Also writes <out>.masks.json: each landmark's box in the image, so `cut` can find it again.
#   python art_src/prompts/blend_act_map.py cut <layout.json> <composite.png> <blended.png> [--out DIR]
#       each landmark cut out of the blended image with the silhouette it had in the composite (after finding the
#       shift of up to 4 px that lines it up best), downscaled to 96 x its map scale -> map/landmarks/<id>.png;
#       the blended image itself, scaled to 1920 x 1080 -> backgrounds/world_map/<background target>.png.
#       It prints how well each landmark lines up: a high mismatch means the model moved or redrew it (redo that one).
#
# layout.json: {"background": path, "target": "high_bastion",
#               "landmarks": {id: {"src": path, "pos": [x, y], "scale": s, "fix": ["--clear", "x0,y0,x1,y1", ...]}}}
# pos is the map_pos anchor (the landmark's ground point; its box bottom sits 8 logical px below it and it grows up).
# "match": 0..1 (optional, default 0) tones the landmark toward the painting around it: its colour mean moves that far
# toward the mean of the background ring around its box, and its contrast half that far (a colour transfer in code).
import json, pathlib, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "art_src" / "prompts"))
import cut_ai_sheets as C  # noqa: E402
from PIL import Image, ImageChops  # noqa: E402

VIEW_W, VIEW_H, PAD_X, PAD_Y = 496, 328, 34, 22  # the world map viewport (tools/mock_ui/previews/world_diamond.py)
GAME = "games/emberwake/assets"


def fixes(spec):
    out = {"tints": [], "clears": [], "desmokes": [], "grades": []}
    names = {"--tint": "tints", "--clear": "clears", "--desmoke": "desmokes", "--grade": "grades"}
    for flag, value in zip(spec[::2], spec[1::2]):
        out[names[flag]].append(value)
    return out


def sprite(entry, side):
    """The landmark exactly as the cut makes it, at `side` x `side` pixels."""
    with tempfile.TemporaryDirectory() as tmp:
        C.cut_landmark(entry["src"], "x", tmp, size=side, **fixes(entry.get("fix", [])))
        return Image.open(pathlib.Path(tmp) / GAME / "map/landmarks/x.png").convert("RGBA")


def match_tone(spr, bg, box, strength):
    """Moves the sprite's per-channel mean `strength` of the way to the surrounding background's, and its spread half
    as far, so a landmark takes on the colour and contrast of the painting it stands in."""
    if not strength:
        return spr
    x, y, side = box
    pad = side // 3
    ring = bg.crop((max(0, x - pad), max(0, y - pad), x + side + pad, y + side + pad)).convert("RGB")
    rp = list(ring.getdata())
    sp = [p for p in spr.getdata() if p[3]]
    def stats(px, c):
        vals = [p[c] for p in px]; m = sum(vals) / len(vals)
        return m, max(1.0, (sum((v - m) ** 2 for v in vals) / len(vals)) ** 0.5)
    out = spr.copy(); data = list(out.getdata())
    for c in range(3):
        ms, ss = stats(sp, c); mb, sb = stats(rp, c)
        mt, st = ms + (mb - ms) * strength, ss + (sb - ss) * strength / 2
        data = [tuple(max(0, min(255, round((p[i] - ms) / ss * st + mt))) if i == c and p[3] else p[i] for i in range(4))
                for p in data]
    out.putdata(data)
    return out


def boxes(layout, size):
    """Each landmark's square in the background image: the viewport is the image's centre crop (object-fit: cover)."""
    W, H = size
    k = H / VIEW_H                      # image pixels per logical map pixel
    x0 = (W - VIEW_W * k) / 2
    out = {}
    for lid, e in layout["landmarks"].items():
        mx, my = e["pos"]
        x = PAD_X + mx * (VIEW_W - 2 * PAD_X); y = PAD_Y + my * (VIEW_H - 2 * PAD_Y)
        side = round(48 * e.get("scale", 1) * k)
        out[lid] = (round(x0 + x * k - side / 2), round((y + 8) * k - side), side)
    return out


def compose(layout_path, out_path):
    layout = json.loads(pathlib.Path(layout_path).read_text())
    bg = Image.open(layout["background"]).convert("RGBA")
    placed = boxes(layout, bg.size)
    base = bg.copy()
    for lid, (x, y, side) in sorted(placed.items(), key=lambda kv: kv[1][1] + kv[1][2]):  # back to front
        e = layout["landmarks"][lid]
        bg.alpha_composite(match_tone(sprite(e, side), base, (x, y, side), e.get("match", 0)), (x, y))
    bg.convert("RGB").save(out_path)
    pathlib.Path(str(out_path) + ".masks.json").write_text(json.dumps(placed, indent=1))
    print(f"composite {bg.size[0]}x{bg.size[1]} -> {out_path}")


def cut(layout_path, composite_path, blended_path, out_root):
    layout = json.loads(pathlib.Path(layout_path).read_text())
    comp = Image.open(composite_path).convert("RGB")
    blend = Image.open(blended_path).convert("RGB")
    if blend.size != comp.size:
        blend = blend.resize(comp.size, Image.LANCZOS)
    placed = boxes(layout, comp.size)
    for lid, (x, y, side) in placed.items():
        mask = sprite(layout["landmarks"][lid], side).getchannel("A")  # the shape only; tone comes from the blend
        ref = comp.crop((x, y, x + side, y + side))
        best = None
        for dy in range(-4, 5):            # the shift that lines the blended landmark up with the composite best
            for dx in range(-4, 5):
                got = blend.crop((x + dx, y + dy, x + dx + side, y + dy + side))
                diff = ImageChops.difference(got, ref).convert("L")
                score = sum(d * (m > 127) for d, m in zip(diff.getdata(), mask.getdata())) / max(1, sum(m > 127 for m in mask.getdata()))
                if best is None or score < best[0]:
                    best = (score, dx, dy, got)
        score, dx, dy, got = best
        cutout = got.convert("RGBA"); cutout.putalpha(mask)
        e = layout["landmarks"][lid]
        final = round(96 * e.get("scale", 1))
        icon = cutout.resize((final, final), Image.BOX)
        icon.putalpha(icon.getchannel("A").point(lambda v: 255 if v >= 128 else 0))
        dest = pathlib.Path(out_root) / GAME / f"map/landmarks/{lid}.png"
        dest.parent.mkdir(parents=True, exist_ok=True)
        icon.save(dest)
        flag = "  <- check: redrawn or moved?" if score > 40 else ""
        print(f"{lid}: shift ({dx}, {dy}), mismatch {score:.1f}{flag} -> {dest} {final}x{final}")
    dest = pathlib.Path(out_root) / GAME / f"backgrounds/world_map/{layout['target']}.png"
    dest.parent.mkdir(parents=True, exist_ok=True)
    blend.resize((1920, 1080), Image.LANCZOS).save(dest)
    print(f"background -> {dest} 1920x1080")


def main(argv):
    if len(argv) >= 3 and argv[0] == "compose":
        return compose(argv[1], argv[2])
    if len(argv) >= 4 and argv[0] == "cut":
        out = pathlib.Path(argv[argv.index("--out") + 1]) if "--out" in argv else ROOT
        return cut(argv[1], argv[2], argv[3], out)
    sys.exit("usage: blend_act_map.py compose <layout.json> <out.png> | cut <layout.json> <composite> <blended> [--out DIR]")


if __name__ == "__main__":
    main(sys.argv[1:])
