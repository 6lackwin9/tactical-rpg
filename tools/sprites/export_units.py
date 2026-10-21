#!/usr/bin/env python3
# Writes the six class bodies of the RPG kit from the sprite rig (SPRITE1-A; ASSETORG2 section 4.1 and SPRITE1 section 4):
# one sheet per body, kits/rpg/assets/units/<class>/base/sheet.png plus clips.json (frame size, palette slots, clips).
#   python tools/sprites/export_units.py [class ...] [--root DIR] [--clips idle|all]
# DIR is a workspace root (default: this repository). Portraits do not come from the rig (ART2 portraits stay).
# --clips idle (default, until SPRITE1-A2): a one-frame sheet, the idle pose's first frame (the 48 px figure at 2x,
#   bottom-aligned in the 104 px square), clips.json frame [104, 104] with one idle clip.
# --clips all (SPRITE1-A2): one row per clip in clips.json order, frames left to right, each 48x48 (nearest, transparent);
#   every clip with the rig's per-frame ms; loop true for idle, walk and idle_guarded.
# Palette slots: the rig palette's base colours (keys without a _lo / _hi shade suffix) that occur in the sheet, so a
# palette file (units/<class>/base/palettes/<id>.json) can recolour the body; at most 16 (aegis_core::assets).
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from PIL import Image  # noqa: E402

import classes  # noqa: E402
import rig  # noqa: E402

WORKSPACE = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
KIT_ASSETS = ("kits", "rpg", "assets")
LOOPING = {"idle", "walk", "idle_guarded"}
SLOTS_MAX, MS_MIN, MS_MAX, FRAMES_MAX = 16, 16, 2000, 64


def token(frame: Image.Image) -> Image.Image:
    out = Image.new("RGBA", (104, 104), (0, 0, 0, 0))
    out.alpha_composite(frame.resize((96, 96), Image.NEAREST), (4, 8))
    return out


def palette_slots(cls: str, sheet: Image.Image) -> dict:
    present = {c[:3] for n, c in sheet.getcolors(maxcolors=1 << 20) if c[3] == 255}
    slots = {k: "#%02x%02x%02x" % v for k, v in classes.UNITS[cls]["pal"].items()
             if not k.endswith(("_lo", "_hi")) and tuple(v) in present}
    if len(slots) > SLOTS_MAX:
        sys.exit("export_units: %s has %d palette slots (at most %d): %s" % (cls, len(slots), SLOTS_MAX, ", ".join(slots)))
    return slots


def body(cls: str, mode: str):
    frames, times = classes.clips(cls)
    if mode == "idle":
        sheet = token(frames["idle"][0])
        return sheet, {"frame": [104, 104], "palette": palette_slots(cls, sheet),
                       "clips": [{"id": "idle", "frames": 1, "ms": [1000]}]}
    w = rig.W
    names = list(frames)
    sheet = Image.new("RGBA", (max(len(frames[n]) for n in names) * w, len(names) * w), (0, 0, 0, 0))
    clips = []
    for r, n in enumerate(names):
        ms = list(times[n]) if isinstance(times[n], (list, tuple)) else [times[n]] * len(frames[n])
        if not 1 <= len(frames[n]) <= FRAMES_MAX or len(ms) != len(frames[n]) or not all(MS_MIN <= t <= MS_MAX for t in ms):
            sys.exit("export_units: %s clip %s: %d frames, ms %s outside the sheet rules" % (cls, n, len(frames[n]), ms))
        for c, fr in enumerate(frames[n]):
            sheet.paste(fr, (c * w, r * w))
        clips.append({"id": n, "frames": len(frames[n]), "ms": [int(t) for t in ms], "loop": n in LOOPING})
    return sheet, {"frame": [w, w], "palette": palette_slots(cls, sheet), "clips": clips}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("classes", nargs="*", help="class ids (default: every rig class)")
    ap.add_argument("--root", default=WORKSPACE, help="workspace root holding kits/rpg/assets")
    ap.add_argument("--clips", choices=["idle", "all"], default="idle")
    args = ap.parse_args()
    for cls in args.classes or list(classes.UNITS):
        if cls not in classes.UNITS:
            sys.exit("export_units: unknown class %s (rig classes: %s)" % (cls, ", ".join(classes.UNITS)))
        sheet, clips = body(cls, args.clips)
        folder = os.path.join(args.root, *KIT_ASSETS, "units", cls, "base")
        os.makedirs(folder, exist_ok=True)
        sheet.save(os.path.join(folder, "sheet.png"))
        with open(os.path.join(folder, "clips.json"), "w", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(clips) + "\n")
        print("wrote kit:units/%s/base (%d clips, %d palette slots)" % (cls, len(clips["clips"]), len(clips["palette"])))


if __name__ == "__main__":
    main()
