#!/usr/bin/env python3
# Writes the unit bodies from the sprite rig (SPRITE1-A, SPRITE2; ASSETORG2 section 4.1 and SPRITE1 section 4): one sheet
# per body, units/<class>/<variant>/sheet.png plus clips.json (frame size, palette slots, clips), and the palette files
# units/<class>/<variant>/palettes/<id>.json that recolour a body (tools/sprites/variants.py lists them).
#   python tools/sprites/export_units.py [body ...] [--root DIR] [--clips idle|all] [--review OUT]
# body: a class id (its kit body <class>/base) or a variant key <class>/<variant> of variants.py (default: every class
# and every variant). Bodies land in their layer: the class bodies, the goblin variants, the dummy and the creature
# bases (beast/base, crawler/base; CREATURE1) in the kit (kits/rpg/assets/), the named bodies, the named creatures and
# the character / ash palettes in the game (games/emberwake/assets/). A palette
# file is written with its body. DIR is a workspace root (default: this repository). Portraits do not come from the rig.
# --clips idle (the pre-SPRITE1-A2 form): a one-frame sheet, the idle pose's first frame (the 48 px figure at 2x,
#   bottom-aligned in the 104 px square), clips.json frame [104, 104] with one idle clip.
# --clips all (default since SPRITE1-A2): one row per clip in clips.json order, frames left to right, each 48x48 (nearest, transparent);
#   every clip with the rig's per-frame ms; loop true for idle, walk and idle_guarded.
# Palette slots: a class body's are the rig palette's base colours (keys without a _lo / _hi shade suffix) that occur in
# the sheet; a variant lists its own (shades included, the first key wins when two share a colour); at most 16
# (aegis_core::assets). A palette file may only name its body's slots (checked here and by content check A4).
# Looks (WAR2H1, variants.LOOKS): a class base's looks are bodies of their own (units/<class>/1h_buckler/ and
# units/<class>/2h_<family>/, kit); a look's clips.json palette is its body's, exactly (DIR4 L3; the export exits when a look adds a slot colour), and the
# body's palette files are written into each look's folder too.
# --review OUT: per body a contact sheet (every clip's frames at 3x, one row per clip), GIFs of idle and the first attack
#   clip, and per palette file the idle frame recoloured; for the review page (nothing is written under OUT otherwise).
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from PIL import Image, ImageDraw  # noqa: E402

import classes  # noqa: E402
import rig  # noqa: E402
import variants  # noqa: E402

WORKSPACE = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
LAYERS = {"kit": ("kits", "rpg", "assets"), "game": ("games", "emberwake", "assets")}
LOOPING = {"idle", "walk", "idle_guarded"}
SLOTS_MAX, MS_MIN, MS_MAX, FRAMES_MAX = 16, 16, 2000, 64
STANDARD = {"idle", "walk", "hurt", "death", "defend", "idle_guarded", "hurt_guarded", "idle_focus"}
REVIEW_BG = (46, 52, 44)


def hexc(v) -> str:
    return "#%02x%02x%02x" % tuple(v[:3])


def token(frame: Image.Image) -> Image.Image:
    out = Image.new("RGBA", (104, 104), (0, 0, 0, 0))
    out.alpha_composite(frame.resize((96, 96), Image.NEAREST), (4, 8))
    return out


def palette_slots(key: str, pal: dict, keys, sheet: Image.Image) -> dict:
    present = {c[:3] for n, c in sheet.getcolors(maxcolors=1 << 20) if c[3] == 255}
    if keys is None:
        slots = {k: hexc(v) for k, v in pal.items() if not k.endswith(("_lo", "_hi")) and tuple(v) in present}
    else:
        slots, seen = {}, set()
        for k in keys:
            if k in pal and tuple(pal[k]) in present and tuple(pal[k]) not in seen:
                slots[k] = hexc(pal[k])
                seen.add(tuple(pal[k]))
    if len(slots) > SLOTS_MAX:
        sys.exit("export_units: %s has %d palette slots (at most %d): %s" % (key, len(slots), SLOTS_MAX, ", ".join(slots)))
    return slots


def source(key: str):
    """(layer, folder key <class>/<variant>, frames, ms, rig palette, slot keys) of a body."""
    if key in classes.UNITS:
        frames, times = classes.clips(key)
        return "kit", key + "/base", frames, times, classes.UNITS[key]["pal"], None
    if key in variants.all_bodies():
        frames, times, pal, slots = variants.clips(key)
        return variants.layer(key), key, frames, times, pal, slots
    sys.exit("export_units: unknown body %s (classes: %s; variants: %s)"
             % (key, ", ".join(classes.UNITS), ", ".join(variants.all_bodies())))


def body(key: str, mode: str):
    layer, folder, frames, times, pal, keys = source(key)
    if mode == "idle":
        sheet = token(frames["idle"][0])
        return layer, folder, frames, sheet, {"frame": [104, 104], "palette": palette_slots(key, pal, keys, sheet),
                                              "clips": [{"id": "idle", "frames": 1, "ms": [1000]}]}
    w = rig.W
    names = list(frames)
    sheet = Image.new("RGBA", (max(len(frames[n]) for n in names) * w, len(names) * w), (0, 0, 0, 0))
    clips = []
    for r, n in enumerate(names):
        ms = list(times[n]) if isinstance(times[n], (list, tuple)) else [times[n]] * len(frames[n])
        if not 1 <= len(frames[n]) <= FRAMES_MAX or len(ms) != len(frames[n]) or not all(MS_MIN <= t <= MS_MAX for t in ms):
            sys.exit("export_units: %s clip %s: %d frames, ms %s outside the sheet rules" % (key, n, len(frames[n]), ms))
        for c, fr in enumerate(frames[n]):
            sheet.paste(fr, (c * w, r * w))
        clips.append({"id": n, "frames": len(frames[n]), "ms": [int(t) for t in ms], "loop": n in LOOPING})
    return layer, folder, frames, sheet, {"frame": [w, w], "palette": look_slots(key, pal, keys, sheet), "clips": clips}


def look_slots(key: str, pal: dict, keys, sheet: Image.Image) -> dict:
    """A body's palette slots; a look (WAR2H1) takes its base body's, exactly (DIR4 L3), and may not add a colour the
    slot rule would make a slot."""
    own = palette_slots(key, pal, keys, sheet)
    base = variants.look_base(key)
    if base is None:
        return own
    slots = body(base, "all")[4]["palette"]
    extra = {k: v for k, v in own.items() if slots.get(k) != v}
    if extra:
        sys.exit("export_units: look %s adds palette colours its body %s lacks: %s" % (key, base, extra))
    return slots


def recolour(im: Image.Image, body_slots: dict, slots: dict) -> Image.Image:
    # the engine's palette swap (ASSETORG2 4.1): every pixel of exactly a slot's body colour takes the new colour
    swap = {}
    for k, v in slots.items():
        old = body_slots[k].lstrip("#")
        swap[tuple(int(old[i:i + 2], 16) for i in (0, 2, 4))] = tuple(v[:3])
    out = im.copy()
    px = out.load()
    for y in range(out.height):
        for x in range(out.width):
            c = px[x, y]
            if c[3] and c[:3] in swap:
                px[x, y] = swap[c[:3]] + (c[3],)
    return out


def review(out: str, key: str, frames: dict, times: dict, pals: list) -> list:
    """Contact sheet and GIFs of one body for the review page; returns the paths written."""
    z, w, label = 3, rig.W, 96
    name = key.replace("/", "_")
    names = list(frames)
    cols = max(len(frames[n]) for n in names)
    board = Image.new("RGBA", (label + cols * w * z, len(names) * w * z), REVIEW_BG + (255,))
    d = ImageDraw.Draw(board)
    for r, n in enumerate(names):
        d.text((6, r * w * z + 8), n, fill=(230, 226, 210))
        d.text((6, r * w * z + 22), "%d f" % len(frames[n]), fill=(160, 166, 150))
        for c, fr in enumerate(frames[n]):
            board.alpha_composite(fr.resize((w * z, w * z), Image.NEAREST), (label + c * w * z, r * w * z))
    paths = [os.path.join(out, "%s_clips.png" % name)]
    board.save(paths[-1])

    def gif(clip, path, recol=None):
        fr = [f if recol is None else recol(f) for f in frames[clip]]
        g = []
        for f in fr:
            b = Image.new("RGBA", (w, w), REVIEW_BG + (255,))
            b.alpha_composite(f)
            g.append(b.resize((w * 4, w * 4), Image.NEAREST).convert("P", palette=Image.ADAPTIVE))
        ms = times[clip] if isinstance(times[clip], (list, tuple)) else [times[clip]] * len(fr)
        g[0].save(path, save_all=True, append_images=g[1:], duration=list(ms), loop=0)
        paths.append(path)

    gif("idle", os.path.join(out, "%s_idle.gif" % name))
    attack = next((n for n in names if n not in STANDARD), "hurt" if "hurt" in names else None)
    if attack:
        gif(attack, os.path.join(out, "%s_%s.gif" % (name, attack)))
    for pid, body_slots, slots in pals:
        recol = lambda f, b=body_slots, s=slots: recolour(f, b, s)
        strip = Image.new("RGBA", (2 * w * z, w * z), REVIEW_BG + (255,))
        strip.alpha_composite(frames["idle"][0].resize((w * z, w * z), Image.NEAREST), (0, 0))
        strip.alpha_composite(recol(frames["idle"][0]).resize((w * z, w * z), Image.NEAREST), (w * z, 0))
        paths.append(os.path.join(out, "%s_palette_%s.png" % (name, pid)))
        strip.save(paths[-1])
        if attack:
            gif(attack, os.path.join(out, "%s_palette_%s_%s.gif" % (name, pid, attack)), recol)
    return paths


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("bodies", nargs="*", help="class ids or variant keys (default: every class and every variant)")
    ap.add_argument("--root", default=WORKSPACE, help="workspace root holding kits/rpg/assets and games/emberwake/assets")
    ap.add_argument("--clips", choices=["idle", "all"], default="all")
    ap.add_argument("--review", metavar="OUT", help="also write contact sheets and GIFs for the review page to OUT")
    args = ap.parse_args()
    if args.review:
        os.makedirs(args.review, exist_ok=True)
    for key in args.bodies or list(classes.UNITS) + variants.all_bodies():
        layer, folder, frames, sheet, clips = body(key, args.clips)
        base = os.path.join(args.root, *LAYERS[layer], "units", *folder.split("/"))
        os.makedirs(base, exist_ok=True)
        sheet.save(os.path.join(base, "sheet.png"))
        with open(os.path.join(base, "clips.json"), "w", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(clips) + "\n")
        print("wrote %s:units/%s (%d clips, %d palette slots)" % (layer, folder, len(clips["clips"]), len(clips["palette"])))
        pals = []
        for (pal_layer, pal_body, pid), colours in variants.PALETTES.items():
            if pal_body not in (key, variants.look_base(key)):  # a body's palette files recolour its looks too
                continue
            slots = {k: hexc(v) for k, v in colours.items() if k in clips["palette"]}
            missing = [k for k in colours if k not in clips["palette"]]
            if pid in variants.SHARED_PALETTES:  # one colour set for several bodies: keep the slots this body has
                missing = []
            if missing or not slots:
                sys.exit("export_units: palette %s of %s names slots the body lacks: %s (slots: %s)"
                         % (pid, key, ", ".join(missing) or "none left", ", ".join(clips["palette"])))
            pdir = os.path.join(args.root, *LAYERS[pal_layer], "units", *folder.split("/"), "palettes")
            os.makedirs(pdir, exist_ok=True)
            with open(os.path.join(pdir, pid + ".json"), "w", encoding="utf-8", newline="\n") as f:
                f.write(json.dumps({"slots": slots}) + "\n")
            print("wrote %s:units/%s/palettes/%s.json (%d slots)" % (pal_layer, folder, pid, len(slots)))
            pals.append((pid, clips["palette"], {k: colours[k] for k in slots}))
        if args.review:
            _, _, fr, times, _, _ = source(key)
            for p in review(args.review, key, fr, times, pals):
                print("review", p)


if __name__ == "__main__":
    main()
