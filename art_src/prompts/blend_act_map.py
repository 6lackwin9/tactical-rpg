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
#   python art_src/prompts/blend_act_map.py preview <layout.json> <cut DIR> <out.png> [--select ID]
#       the in-game look: the cut background with every cut landmark drawn over it (--select draws one landmark's
#       selection glow from its alpha, to check the highlight stays clean).
#   python art_src/prompts/blend_act_map.py codeblend <layout.json> [--out DIR]
#       the blend in code, no image model (Tushar 2026-11-13: keep the landmarks a separate layer so every edit made to
#       them stays and the highlights stay clean). Each landmark is toned toward the painting around it (`match`,
#       default 0.25; alpha untouched; its own lights keep full brightness) and its base rows darkened a little where it meets the ground; the BACKGROUND
#       gets what joins them: a soft contact shadow cast from each landmark's silhouette toward the lower right (the key
#       light is upper left) and a faint spill of its own lamps, fires and glows on the ground around it. Shadow and
#       light are stepped (4 levels) so they stay pixel art. Writes the landmark cuts at 96 x scale and the background at
#       1920 x 1080, like `cut`.
#
# layout.json: {"background": path, "target": "high_bastion",
#               "landmarks": {id: {"src": path, "pos": [x, y], "scale": s, "fix": ["--clear", "x0,y0,x1,y1", ...]}}}
# pos is the map_pos anchor (the landmark's ground point; its box bottom sits 8 logical px below it and it grows up).
# "match": 0..1 (optional, default 0) tones the landmark toward the painting around it: its colour mean moves that far
# toward the mean of the background ring around its box, and its contrast half that far (a colour transfer in code).
# "keep_sprite": true (optional) keeps the landmark's own cut instead of cutting it from the blended image, for a
# landmark the model redrew (its sprite then covers the redrawn copy; `match` tones it to the blended surroundings).
import json, pathlib, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "art_src" / "prompts"))
import cut_ai_sheets as C  # noqa: E402
from PIL import Image, ImageChops, ImageDraw  # noqa: E402

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


def preview(layout_path, cut_root, out_path, select=None):
    """The in-game look after `cut`: the blended background with every cut landmark drawn over it at its box."""
    layout = json.loads(pathlib.Path(layout_path).read_text())
    bg = Image.open(pathlib.Path(cut_root) / GAME / f"backgrounds/world_map/{layout['target']}.png").convert("RGBA")
    placed = boxes(layout, bg.size)
    for lid, (x, y, side) in sorted(placed.items(), key=lambda kv: kv[1][1] + kv[1][2]):
        icon = Image.open(pathlib.Path(cut_root) / GAME / f"map/landmarks/{lid}.png").convert("RGBA")
        icon = icon.resize((side, side), Image.NEAREST)
        if lid == select:  # the game's selection glow, from the landmark's own alpha
            from PIL import ImageFilter
            solid = icon.getchannel("A").point(lambda v: 255 if v else 0)
            outside = solid.copy(); ImageDraw.floodfill(outside, (0, 0), 128)   # the gaps inside count as the landmark
            filled = outside.point(lambda v: 0 if v == 128 else 255)
            ring = ImageChops.subtract(filled.filter(ImageFilter.MaxFilter(5)), filled)  # outside the outline only
            gold = Image.new("RGBA", icon.size, (251, 191, 36, 255)); gold.putalpha(ring)
            bg.alpha_composite(gold, (x, y))
        bg.alpha_composite(icon, (x, y))
    bg.convert("RGB").save(out_path)
    print(f"preview -> {out_path}")


def _emissive(spr):
    """The landmark's own lights: bright, saturated pixels (lamps, fires, candles, glows), with their colour."""
    import colorsys
    out = Image.new("RGBA", spr.size, (0, 0, 0, 0)); src = spr.load(); dst = out.load()
    for y in range(spr.height):
        for x in range(spr.width):
            r, g, b, a = src[x, y]
            if a:
                _, ss, vv = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
                if vv > 0.7 and ss > 0.4:
                    dst[x, y] = (r, g, b, 255)
    return out


def _steps(alpha, levels=4):
    """Quantizes a soft mask to a few levels so the shadow and glow stay pixel art."""
    return alpha.point(lambda v: round(v / 255 * levels) * (255 // levels))


def codeblend(layout_path, out_root):
    from PIL import ImageFilter
    layout = json.loads(pathlib.Path(layout_path).read_text())
    bg = Image.open(layout["background"]).convert("RGBA")
    base = bg.copy()
    placed = boxes(layout, bg.size)
    shadow = Image.new("L", bg.size, 0)
    glow = Image.new("RGBA", bg.size, (0, 0, 0, 0))
    for lid, (x, y, side) in placed.items():
        e = layout["landmarks"][lid]
        spr = sprite(e, side)
        bb = spr.getbbox()
        if not bb:
            continue
        # contact shadow: the silhouette squashed flat, sheared toward the lower right, its foot on the landmark's foot
        sil = spr.getchannel("A").crop(bb)
        w, h = sil.size
        flat = sil.resize((w, max(1, round(h * 0.34))), Image.BILINEAR)
        flat = flat.transform((w + flat.height, flat.height), Image.AFFINE, (1, -1, 0, 0, 1, 0), Image.BILINEAR)
        fx, fy = x + bb[0] + round(side * 0.03), y + bb[3] - round(flat.height * 0.75)
        layer = Image.new("L", bg.size, 0); layer.paste(flat, (fx, fy))
        shadow = ImageChops.lighter(shadow, layer)
        # light spill: the landmark's own lights, blurred wide onto the ground around it
        em = _emissive(spr)
        gl = Image.new("RGBA", bg.size, (0, 0, 0, 0)); gl.alpha_composite(em, (x, y))
        glow.alpha_composite(gl)
    r = max(2, round(min(placed.values(), key=lambda b: b[2])[2] * 0.05))
    shadow = _steps(shadow.filter(ImageFilter.GaussianBlur(r)).point(lambda v: round(v * 0.68)))
    dark = Image.new("RGBA", bg.size, (6, 6, 12, 255)); dark.putalpha(shadow)
    bg.alpha_composite(dark)
    gr = round(r * 3)
    spill = glow.filter(ImageFilter.GaussianBlur(gr))
    a = _steps(spill.getchannel("A").point(lambda v: min(255, round(v * 1.6))).filter(ImageFilter.GaussianBlur(1))).point(lambda v: round(v * 0.45))
    lit = Image.new("RGBA", bg.size)
    lit.paste(spill.convert("RGB").point(lambda v: min(255, v * 3)))
    lit.putalpha(a)
    bg = Image.alpha_composite(bg, lit)
    out_root = pathlib.Path(out_root)
    for lid, (x, y, side) in placed.items():
        e = layout["landmarks"][lid]
        final = round(96 * e.get("scale", 1))
        raw = sprite(e, final)
        icon = match_tone(raw, base, (x, y, side), e.get("match", 0.25))
        icon.paste(raw, (0, 0), _emissive(raw))  # the landmark's own lights keep their full brightness
        px = icon.load(); bb = icon.getbbox() or (0, 0, final, final)
        y0 = bb[3] - max(2, round((bb[3] - bb[1]) * 0.12))
        for yy in range(y0, bb[3]):  # the base meets the ground: its last rows a little darker
            k = 1 - 0.28 * (yy - y0 + 1) / (bb[3] - y0)
            for xx in range(final):
                rr, gg, b2, al = px[xx, yy]
                if al:
                    px[xx, yy] = (round(rr * k), round(gg * k), round(b2 * k), al)
        dest = out_root / GAME / f"map/landmarks/{lid}.png"
        dest.parent.mkdir(parents=True, exist_ok=True)
        icon.save(dest)
        print(f"{lid}: toned {e.get('match', 0.25)} -> {dest} {final}x{final}")
    dest = out_root / GAME / f"backgrounds/world_map/{layout['target']}.png"
    dest.parent.mkdir(parents=True, exist_ok=True)
    bg.convert("RGB").resize((1920, 1080), Image.LANCZOS).save(dest)
    print(f"background (shadows and light spill) -> {dest} 1920x1080")


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
        e = layout["landmarks"][lid]
        if e.get("keep_sprite"):  # the model redrew it: keep the landmark's own cut, toned to the blended painting
            cutout = match_tone(sprite(e, side), blend.convert("RGBA"), (x, y, side), e.get("match", 0.5))
        else:
            cutout = got.convert("RGBA"); cutout.putalpha(mask)
        final = round(96 * e.get("scale", 1))
        icon = cutout.resize((final, final), Image.BOX)
        icon.putalpha(icon.getchannel("A").point(lambda v: 255 if v >= 128 else 0))
        dest = pathlib.Path(out_root) / GAME / f"map/landmarks/{lid}.png"
        dest.parent.mkdir(parents=True, exist_ok=True)
        icon.save(dest)
        flag = "  (own sprite kept)" if e.get("keep_sprite") else ("  <- check: redrawn or moved?" if score > 40 else "")
        print(f"{lid}: shift ({dx}, {dy}), mismatch {score:.1f}{flag} -> {dest} {final}x{final}")
    dest = pathlib.Path(out_root) / GAME / f"backgrounds/world_map/{layout['target']}.png"
    dest.parent.mkdir(parents=True, exist_ok=True)
    blend.resize((1920, 1080), Image.LANCZOS).save(dest)
    print(f"background -> {dest} 1920x1080")


def main(argv):
    if len(argv) >= 3 and argv[0] == "compose":
        return compose(argv[1], argv[2])
    if len(argv) >= 4 and argv[0] == "preview":
        return preview(argv[1], argv[2], argv[3], argv[argv.index("--select") + 1] if "--select" in argv else None)
    if len(argv) >= 2 and argv[0] == "codeblend":
        return codeblend(argv[1], pathlib.Path(argv[argv.index("--out") + 1]) if "--out" in argv else ROOT)
    if len(argv) >= 4 and argv[0] == "cut":
        out = pathlib.Path(argv[argv.index("--out") + 1]) if "--out" in argv else ROOT
        return cut(argv[1], argv[2], argv[3], out)
    sys.exit("usage: blend_act_map.py compose <layout.json> <out.png> | cut <layout.json> <composite> <blended> [--out DIR]"
             " | preview <layout.json> <cut DIR> <out.png> [--select ID] | codeblend <layout.json> [--out DIR]")


if __name__ == "__main__":
    main(sys.argv[1:])
