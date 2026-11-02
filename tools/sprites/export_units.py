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
#   every clip with the rig's per-frame ms; loop true for idle, walk and idle_guarded. FX1-L1: an attack clip (ATTACKS)
#   also has "impact", the index of the frame the rig marked with rig.impact (the blow lands or the shot leaves; the
#   export exits when one is unmarked); a view row copies its side clip's. WINDUP1: an attack clip also has "strike"
#   (rig.strike, else exit) and a wind-up row "loop_from" (rig.loop_start); the six wind-up rows of rig.WINDUP_ROWS
#   (a creature's three, rig.CREATURE_ROWS; WINDUP1-R3) come after today's side rows and are checked (check_windups:
#   X1 - X3, X5); --review adds windup_review's board and GIFs (X6). DASH1: a side `dash` row just before them and
#   dash_<view> rows: not looped, rig.DASH_MS, adding no palette slot (check_dash, else exit).
# Palette slots: a class body's are the rig palette's base colours (keys without a _lo / _hi shade suffix) that occur in
# the sheet; a variant lists its own (shades included, the first key wins when two share a colour); at most 16
# (aegis_core::assets). A palette file may only name its body's slots (checked here and by content check A4).
# Looks (WAR2H1, variants.LOOKS): a class base's looks are bodies of their own (units/<class>/1h_buckler/ and
# units/<class>/2h_<family>/, kit); a look's clips.json palette is its body's, exactly (DIR4 L3; the export exits when a look adds a slot colour), and the
# body's palette files are written into each look's folder too.
# --review OUT: per body a contact sheet (every clip's frames at 3x, one row per clip), GIFs of idle and the first attack
#   clip, and per palette file the idle frame recoloured; for the review page (nothing is written under OUT otherwise);
#   per rig body also the DIR4 turnaround board (<body>_turnaround.png: idle, walk, the attack in side right, side left,
#   down and up at 3x and at the 28 px token size; DIR4-R2: plus the four wind-up holds) and <body>_walk_4dir.gif (plus
#   <body>_<attack>_views.gif once the grip has an attack view); a body without view drawings is shown with
#   views.STAND_INS, named on the board; and once per run models.png (model_sheet: side, front and back of each body
#   with views, idle and windup hold, to check they are one model).
# Views (DIR4, tools/sprites/views.py): a body whose parts and grip all have view drawings (views.supports) gets the
#   view clips after its side clips, each with "of" and "view", its side clip's frames, ms and loop (R3, else exit);
#   the view rows add no colour outside the unit's palette, rig OUT and WHITE and no palette slot (R4, else exit naming
#   it); idle_down and idle_up must differ in 10 % of their opaque pixels at 28 px (6.4, else exit); every palette slot
#   colour of the side idle frame shows in the front or back idle frame (check_view_model, else exit naming it). Other bodies
#   are written without views and the exporter prints what they lack.
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from PIL import Image, ImageDraw  # noqa: E402

import classes  # noqa: E402
import rig  # noqa: E402
import variants  # noqa: E402
import views  # noqa: E402

WORKSPACE = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
LAYERS = {"kit": ("kits", "rpg", "assets"), "game": ("games", "emberwake", "assets")}
LOOPING = {"idle", "walk", "idle_guarded", "dazed"} | set(rig.WINDUPS)
SLOTS_MAX, MS_MIN, MS_MAX, FRAMES_MAX = 16, 16, 2000, 64
STANDARD = {"idle", "walk", "hurt", "death", "defend", "idle_guarded", "hurt_guarded", "idle_focus"}
# FX1-L1: the attack clips, each of which must mark its impact frame (rig.impact)
ATTACKS = {"attack", "jab", "haymaker", "kick", "sweep_kick", "shield_bash", "spit"}
# WINDUP1 7.3: X2 the windup hold and the attack frame it strikes from differ in at most this share at 48 px; X3 the held
# poses, dazed and idle differ pairwise in at least this share at 28 px
WINDUP_CONTINUITY, WINDUP_APART = 0.15, 0.06
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
    clips = []
    for n in names:
        ms = list(times[n]) if isinstance(times[n], (list, tuple)) else [times[n]] * len(frames[n])
        if not 1 <= len(frames[n]) <= FRAMES_MAX or len(ms) != len(frames[n]) or not all(MS_MIN <= t <= MS_MAX for t in ms):
            sys.exit("export_units: %s clip %s: %d frames, ms %s outside the sheet rules" % (key, n, len(frames[n]), ms))
        clip = {"id": n, "frames": len(frames[n]), "ms": [int(t) for t in ms], "loop": n in LOOPING}
        start = rig.loop_frame(frames[n])  # WINDUP1: the first looped frame the rig marked (rig.loop_start)
        if start is not None:
            if not clip["loop"] or not 1 <= start < len(frames[n]):
                sys.exit("export_units: %s clip %s marks loop_from %d (a looping clip, 1 to frames - 1)" % (key, n, start))
            clip["loop_from"] = start
        hit = rig.impact_frame(frames[n])  # FX1-L1: the frame the rig marked (rig.impact), written only when marked
        if hit is not None:
            clip["impact"] = hit
        elif n in ATTACKS:
            sys.exit("export_units: %s clip %s marks no impact frame (rig.impact on the frame the blow lands or the shot leaves)" % (key, n))
        first = rig.strike_frame(frames[n])  # WINDUP1: the frame after the clip's own wind-up (rig.strike)
        if first is not None:
            if n not in ATTACKS or not 1 <= first <= hit:
                sys.exit("export_units: %s clip %s marks strike %s (an attack clip, 1 to its impact %s)" % (key, n, first, hit))
            clip["strike"] = first
        elif n in ATTACKS:
            sys.exit("export_units: %s clip %s marks no strike frame (rig.strike on the first frame after its wind-up, WINDUP1 7.2)" % (key, n))
        clips.append(clip)
    check_windups(key, frames, clips, pal, keys)
    check_dash(key, frames, clips, pal, keys)
    side = paste(frames, names)
    rows = dict(frames)
    for vid, vfr in view_rows(key, clips, frames["idle"][0]).items():  # DIR4: the view clips after the side clips (of, view)
        base, view = views.of(vid)
        side_clip = next(c for c in clips if c["id"] == base)
        rows[vid] = vfr
        clips.append({"id": vid, "of": base, "view": view, "frames": side_clip["frames"], "ms": side_clip["ms"],
                      "loop": side_clip["loop"], **{k: side_clip[k] for k in ("loop_from", "impact", "strike") if k in side_clip}})
        check_view_marks(key, vid, vfr, side_clip)
    sheet = paste(rows, [c["id"] for c in clips])
    if len(clips) > len(names):
        check_view_colours(key, pal, keys, side, sheet)
        for view in views.VIEWS:  # WINDUP1 X2 and X3 on the view rows too
            attack = main_attack(frames)
            if "windup_" + view in rows and attack + "_" + view in rows:
                s = next(c["strike"] for c in clips if c["id"] == attack)
                share = views.difference(rows["windup_" + view][rig.LOOP_FROM], rows["%s_%s" % (attack, view)][s - 1], size=rig.W)
                if share > WINDUP_CONTINUITY:
                    sys.exit("export_units: %s windup_%s frame %d and %s_%s frame %d differ in %.1f%% (at most %d%%, WINDUP1 X2)"
                             % (key, view, rig.LOOP_FROM, attack, view, s - 1, share * 100, WINDUP_CONTINUITY * 100))
            close = told_apart(rows, "_" + view)
            if close:
                sys.exit("export_units: %s poses too alike at %d px (at least %d%%, WINDUP1 X3): %s"
                         % (key, views.TOKEN, WINDUP_APART * 100, "; ".join("%s / %s %.1f%%" % (a, b, s * 100) for a, b, s in close)))
    return layer, folder, rows, sheet, {"frame": [w, w], "palette": look_slots(key, pal, keys, sheet), "clips": clips}


def paste(frames: dict, names: list) -> Image.Image:
    """One sheet: a row per clip of `names`, frames left to right."""
    w = rig.W
    sheet = Image.new("RGBA", (max(len(frames[n]) for n in names) * w, len(names) * w), (0, 0, 0, 0))
    for r, n in enumerate(names):
        for c, fr in enumerate(frames[n]):
            sheet.paste(fr, (c * w, r * w))
    return sheet


def unit_of(key: str):
    """(class, rig unit) of a body drawn on the humanoid rig; None for the dummy and the creatures (no views, R5)."""
    if key in classes.UNITS:
        return key, classes.UNITS[key]
    if key in variants.looks():
        cls, look = variants.looks()[key]
        return cls, variants.look_unit(classes.UNITS[cls], look)
    if key in variants.BODIES:
        _, cls, u, _ = variants.BODIES[key]
        return cls, u
    return None


def check_view_model(key: str, u: dict, side_idle: Image.Image, A: dict) -> None:
    """Decision "Front and back views follow the side model" (user 2026-10-29): every palette slot colour (a base key
    of the unit's palette, no _lo / _hi shade) that the side idle frame 0 shows also shows in idle_down or idle_up
    frame 0 (the hood, the hair, the headband, the cloak, the weapon...); exits naming the slots a view leaves out."""
    slots = {}
    for k, v in u["pal"].items():
        if not k.endswith(("_lo", "_hi")):
            slots.setdefault(tuple(v[:3]), k)
    shown = colours(A["idle_down"][0]) | colours(A["idle_up"][0])
    lost = sorted(slots[c] for c in colours(side_idle) if c in slots and c not in shown)
    if lost:
        sys.exit("export_units: %s front and back idle frames leave out the side model's slot colours: %s (decision "
                 "\"Front and back views follow the side model\")" % (key, ", ".join(lost)))


def view_rows(key: str, clips: list, side_idle: Image.Image = None) -> dict:
    """DIR4 view clips of body `key` in sheet order (each side clip's down then up), {} when the body has no views:
    views.supports (R5; prints what is missing), equal frames and ms to the side clip (R3, exits otherwise), the
    readability check (6.4, exits when idle_down and idle_up do not differ enough at the token size) and, given the side
    idle frame 0, the side-model check (check_view_model)."""
    cu = unit_of(key)
    if cu is None:
        return {}
    cls, u = cu
    miss = views.missing(u, cls)
    if miss:
        print("views: %s keeps side views only (no view drawing: %s)" % (key, ", ".join(miss)))
        return {}
    A, T = views.clips(cls, u)
    side = {c["id"]: c for c in clips}
    for vid in A:
        base, _ = views.of(vid)
        ms = list(T[vid]) if isinstance(T[vid], (list, tuple)) else [T[vid]] * len(A[vid])
        if base not in side or len(A[vid]) != side[base]["frames"] or [int(t) for t in ms] != side[base]["ms"]:
            sys.exit("export_units: %s view clip %s: %d frames, ms %s; its side clip %s has %s (DIR4 R3: equal frames and ms)"
                     % (key, vid, len(A[vid]), ms, base, "%d frames, ms %s" % (side[base]["frames"], side[base]["ms"])
                        if base in side else "no row"))
    share = views.difference(A["idle_down"][0], A["idle_up"][0])
    print("views: %s idle_down / idle_up differ in %.0f%% of their opaque pixels at %d px" % (key, share * 100, views.TOKEN))
    if share < views.READABLE:
        sys.exit("export_units: %s idle_down and idle_up differ in %.1f%% of their opaque pixels at %d px (at least %d%%, DIR4 6.4)"
                 % (key, share * 100, views.TOKEN, views.READABLE * 100))
    if side_idle is not None:
        check_view_model(key, u, side_idle, A)
    order = []
    for c in clips:
        order += ["%s_%s" % (c["id"], v) for v in views.VIEWS if "%s_%s" % (c["id"], v) in A]
    return {vid: A[vid] for vid in order}


def main_attack(frames: dict):
    """The clip a body's `windup` holds before (WINDUP1 K2): its attack, or the brawler's jab."""
    return next((n for n in ("attack", "jab") if n in frames), None)


def told_apart(frames: dict, suffix: str = "") -> list:
    """WINDUP1 X3: the (a, b, share) pairs among the wind-up hold frames (frame rig.LOOP_FROM), dazed frame 0 and idle
    frame 0 (clip ids + suffix, e.g. '_down') that differ in less than WINDUP_APART of their opaque pixels at 28 px."""
    shown = [(n, frames[n + suffix][rig.LOOP_FROM]) for n in rig.WINDUPS if n + suffix in frames]
    shown += [(n, frames[n + suffix][0]) for n in ("dazed", "idle") if n + suffix in frames]
    out = []
    for i, (a, fa) in enumerate(shown):
        for b, fb in shown[i + 1:]:
            share = views.difference(fa, fb)
            if share < WINDUP_APART:
                out.append((a + suffix, b + suffix, share))
    return out


def check_windups(key: str, frames: dict, clips: list, pal: dict, keys) -> None:
    """WINDUP1 7.3 X1 - X3 and X5 on a body's side rows (exits on a failure): X1 a body of a class with wind-ups
    (classes.WINDUP_CLASSES, every class since WINDUP1-R2) has the six rows, a creature (variants.CREATURES, WINDUP1-R3)
    the rows of rig.CREATURE_ROWS and no other wind-up, and any wind-up row has the frames, ms, loop and loop_from of
    section 4; X2 `windup` frame loop_from and the main attack's frame strike - 1 differ in at
    most WINDUP_CONTINUITY of their opaque pixels at 48 px; X3 the held poses read apart (told_apart); X5 every new
    frame draws the figure, the new rows add no colour but the rig's effect colours and no palette slot."""
    by_id = {c["id"]: c for c in clips}
    cu = unit_of(key)
    if cu is not None and cu[0] in classes.WINDUP_CLASSES:
        lacking = [n for n in rig.WINDUP_ROWS if n not in by_id]
        if lacking:
            sys.exit("export_units: %s lacks the wind-up rows %s (WINDUP1 X1)" % (key, ", ".join(lacking)))
    if key in variants.CREATURES:  # WINDUP1-R3: a creature's skills are attacks (section 4, A7)
        lacking = [n for n in rig.CREATURE_ROWS if n not in by_id]
        extra = [n for n in rig.WINDUP_ROWS if n in by_id and n not in rig.CREATURE_ROWS]
        if lacking or extra:
            sys.exit("export_units: %s creature rows: lacks %s, has %s (WINDUP1 X1: %s only)"
                     % (key, ", ".join(lacking) or "none", ", ".join(extra) or "none", ", ".join(rig.CREATURE_ROWS)))
    new = [n for n in rig.WINDUP_ROWS if n in by_id]
    if not new:  # a body without wind-ups (the dummy prop)
        return
    want = {n: (7, rig.WINDUP_MS[n], True, rig.LOOP_FROM) for n in rig.WINDUPS}
    want.update(stagger=(5, rig.STAGGER_MS, False, None), dazed=(8, rig.DAZED_MS, True, None))
    for n in new:
        c = by_id[n]
        got = (c["frames"], c["ms"], c["loop"], c.get("loop_from"))
        if got != want[n]:
            sys.exit("export_units: %s clip %s has frames, ms, loop, loop_from %s; WINDUP1 section 4 says %s (X1)" % (key, n, got, want[n]))
    if [c["id"] for c in clips][-len(new):] != new:
        sys.exit("export_units: %s: the wind-up rows come after today's rows, in the order %s (K7)" % (key, ", ".join(new)))
    attack = main_attack(frames)
    if "windup" in by_id and attack:
        s = by_id[attack]["strike"]
        share = views.difference(frames["windup"][rig.LOOP_FROM], frames[attack][s - 1], size=rig.W)
        print("windup: %s windup hold / %s frame %d differ in %.0f%% at %d px (at most %d%%)"
              % (key, attack, s - 1, share * 100, rig.W, WINDUP_CONTINUITY * 100))
        if share > WINDUP_CONTINUITY:
            sys.exit("export_units: %s windup frame %d and %s frame %d differ in %.1f%% of their opaque pixels (at most %d%%, WINDUP1 X2)"
                     % (key, rig.LOOP_FROM, attack, s - 1, share * 100, WINDUP_CONTINUITY * 100))
    close = told_apart(frames)
    if close:
        sys.exit("export_units: %s poses too alike at %d px (at least %d%%, WINDUP1 X3): %s"
                 % (key, views.TOKEN, WINDUP_APART * 100, "; ".join("%s / %s %.1f%%" % (a, b, s * 100) for a, b, s in close)))
    for n in new:
        for i, fr in enumerate(frames[n]):
            if sum(1 for a in fr.getchannel("A").tobytes() if a) < 150:
                sys.exit("export_units: %s clip %s frame %d is blank (WINDUP1 X5)" % (key, n, i))
    today = paste(frames, [n for n in frames if n not in new])
    allowed = colours(today) | {tuple(v[:3]) for v in pal.values()} | {rig.OUT, rig.WHITE, rig.GOLDEN, rig.GOLDEN_HI}
    added = sorted(colours(paste(frames, new)) - allowed)
    if added:
        sys.exit("export_units: %s wind-up rows add colours: %s (WINDUP1 K5)" % (key, ", ".join(hexc(c) for c in added)))
    before, after = palette_slots(key, pal, keys, today), palette_slots(key, pal, keys, paste(frames, list(frames)))
    if before != after:
        sys.exit("export_units: %s wind-up rows add palette slots: %s (WINDUP1 K5)"
                 % (key, ", ".join("%s %s" % kv for kv in after.items() if before.get(kv[0]) != kv[1])))


def check_dash(key: str, frames: dict, clips: list, pal: dict, keys) -> None:
    """DASH1: the side `dash` row has rig.DASH_MS (the battle screen plays it unstretched over its 250 ms slide), does not
    loop, and leaves the palette slots as they are without it (dust and speed lines add none); exits on a failure."""
    if "dash" not in frames:
        return
    clip = next(c for c in clips if c["id"] == "dash")
    if clip["ms"] != rig.DASH_MS or clip["loop"]:
        sys.exit("export_units: %s dash: ms %s, loop %s (rig.DASH_MS, not looped)" % (key, clip["ms"], clip["loop"]))
    others = [n for n in frames if n != "dash"]
    before, after = palette_slots(key, pal, keys, paste(frames, others)), palette_slots(key, pal, keys, paste(frames, others + ["dash"]))
    if before != after:
        extra = {k: v for k, v in after.items() if before.get(k) != v}
        sys.exit("export_units: %s dash adds palette slots: %s" % (key, ", ".join("%s %s" % kv for kv in extra.items())))


def check_view_marks(key: str, vid: str, vfr: list, side_clip: dict) -> None:
    """WINDUP1 X4: a view row marks (rig.loop_start) its side row's loop_from, or nothing."""
    start = rig.loop_frame(vfr)
    if start is not None and start != side_clip.get("loop_from"):
        sys.exit("export_units: %s view clip %s marks loop_from %d; its side clip %s has %s (WINDUP1 X4)"
                 % (key, vid, start, side_clip["id"], side_clip.get("loop_from")))


def colours(sheet: Image.Image) -> set:
    return {c[:3] for _, c in sheet.getcolors(maxcolors=1 << 20) if c[3] == 255}


def check_view_colours(key: str, pal: dict, keys, side: Image.Image, sheet: Image.Image) -> None:
    """DIR4 R4: the view rows draw only with the unit's palette, rig.OUT and rig.WHITE (or a colour the side rows use
    already), and the palette slots are the same with and without them; exits naming any colour a view adds."""
    allowed = colours(side) | {tuple(v[:3]) for v in pal.values()} | {rig.OUT, rig.WHITE}
    added = sorted(colours(sheet) - allowed)
    if added:
        sys.exit("export_units: %s view rows add colours outside its palette: %s" % (key, ", ".join(hexc(c) for c in added)))
    before, after = palette_slots(key, pal, keys, side), palette_slots(key, pal, keys, sheet)
    if before != after:
        extra = {k: v for k, v in after.items() if before.get(k) != v}
        sys.exit("export_units: %s view rows add palette slots: %s" % (key, ", ".join("%s %s" % kv for kv in extra.items())))


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


def turnaround(out: str, key: str, frames: dict, times: dict) -> list:
    """DIR4 6.4 review of a rig body: a turnaround board (rows idle, walk, the attack and, DIR4-R2, the four WINDUP1
    holds (frame rig.LOOP_FROM); columns side right, side left mirrored, down, up) at 3x and at the battle's token size
    (actual size and 3x), a GIF of walk in the four directions
    and one of the attack in down and up. A body without view support is drawn with the views.STAND_INS (named on the
    board); nothing here is written to the sheets. Returns the paths written ([] for the dummy and the creatures)."""
    cu = unit_of(key)
    if cu is None:
        return []
    cls, u = cu
    miss = views.missing(u, cls)
    V, VT = views.clips(cls, u, stand_in=bool(miss))
    z, w, tk, label = 3, rig.W, views.TOKEN, 96
    name = key.replace("/", "_")
    attack = next((n for n in frames if n not in STANDARD), None)
    rows = [("idle", 0), ("walk", 2)] + ([(attack, len(frames[attack]) // 2)] if attack else [])
    rows += [(n, rig.LOOP_FROM) for n in rig.WINDUPS if n in frames]
    dirs = ("side right", "side left", "down", "up")

    def cell(clip, n, d):
        if d == "side right":
            return frames[clip][n]
        if d == "side left":
            return frames[clip][n].transpose(Image.FLIP_LEFT_RIGHT)  # the battle mirrors the whole frame
        vid = "%s_%s" % (clip, d)
        return V[vid][n] if vid in V else None

    top, foot, cz = 18, 30, w * z
    x_tok = label + 4 * cz + 12
    x_tok3 = x_tok + 4 * (tk + 6) + 12
    board = Image.new("RGBA", (x_tok3 + 4 * tk * z, top + len(rows) * cz + foot), REVIEW_BG + (255,))
    d = ImageDraw.Draw(board)
    for i, dn in enumerate(dirs):
        d.text((label + i * cz + 6, 4), dn, fill=(230, 226, 210))
        d.text((x_tok3 + i * tk * z + 4, 4), dn, fill=(230, 226, 210))
    d.text((x_tok, 4), "%d px" % tk, fill=(230, 226, 210))
    for r, (clip, n) in enumerate(rows):
        y = top + r * cz
        d.text((6, y + 8), clip, fill=(230, 226, 210))
        d.text((6, y + 22), "frame %d" % n, fill=(160, 166, 150))
        for i, dn in enumerate(dirs):
            fr = cell(clip, n, dn)
            if fr is None:
                d.text((label + i * cz + 6, y + cz // 2), "no view yet", fill=(160, 166, 150))
                continue
            board.alpha_composite(fr.resize((cz, cz), Image.NEAREST), (label + i * cz, y))
            t = views.token(fr)
            board.alpha_composite(t, (x_tok + i * (tk + 6), y + cz - tk - 4))
            board.alpha_composite(t.resize((tk * z, tk * z), Image.NEAREST), (x_tok3 + i * tk * z, y + cz - tk * z))
    share = views.difference(V["idle_down"][0], V["idle_up"][0])
    note = "idle_down / idle_up differ %.0f%% at %d px (at least %d%%)" % (share * 100, tk, views.READABLE * 100)
    if miss:
        note += "; stand-ins for: " + ", ".join(miss)
    d.text((6, top + len(rows) * cz + 8), note, fill=(230, 226, 210))
    paths = [os.path.join(out, "%s_turnaround.png" % name)]
    board.save(paths[-1])

    def gif(panels, ms, path):
        g = []
        for k in range(len(panels[0])):
            b = Image.new("RGBA", (w * len(panels), w), REVIEW_BG + (255,))
            for p, frs in enumerate(panels):
                b.alpha_composite(frs[k], (p * w, 0))
            g.append(b.resize((b.width * 4, w * 4), Image.NEAREST).convert("P", palette=Image.ADAPTIVE))
        g[0].save(path, save_all=True, append_images=g[1:], duration=list(ms), loop=0)
        paths.append(path)

    wt = times["walk"] if isinstance(times["walk"], (list, tuple)) else [times["walk"]] * len(frames["walk"])
    gif([frames["walk"], [f.transpose(Image.FLIP_LEFT_RIGHT) for f in frames["walk"]], V["walk_down"], V["walk_up"]], wt,
        os.path.join(out, "%s_walk_4dir.gif" % name))
    if attack and "%s_down" % attack in V:
        gif([V["%s_down" % attack], V["%s_up" % attack]], VT["%s_down" % attack], os.path.join(out, "%s_%s_views.gif" % (name, attack)))
    return paths


def windup_review(out: str, key: str, frames: dict, times: dict) -> list:
    """WINDUP1 X6: per body with wind-up rows a pose board (<body>_windup_board.png: rows idle frame 0, the four held
    poses (frame rig.LOOP_FROM), the stagger strip and dazed; side, and down and up for a body with views; each frame
    at 3x and, beside it, at the battle's 28 px token size shown at 3x) and GIFs: <body>_windup_strike.gif (the windup
    intro, two holds and the attack from its strike frame, as the battle plays it), <body>_windups.gif (the four
    wind-ups, intro and two holds, side by side) and <body>_stagger_dazed.gif (stagger into two dazed loops); the GIFs
    show side, down and up side by side when the body has views. Returns the paths written ([] without wind-up rows)."""
    if "windup" not in frames:
        return []
    rows = dict(frames)
    dirs = [("side", "")]
    cu = unit_of(key)
    if cu is not None and views.supports(cu[1], cu[0]):
        V, VT = views.clips(*cu)
        rows.update(V)
        times = dict(times, **VT)
        dirs += [(v, "_" + v) for v in views.VIEWS]
    z, w, tk, label = 3, rig.W, views.TOKEN, 170
    name = key.replace("/", "_")
    lines = []
    for dn, sfx in dirs:
        lines.append(("%s idle" % dn, [rows["idle" + sfx][0]]))
        lines.append(("%s holds" % dn, [rows[n + sfx][rig.LOOP_FROM] for n in rig.WINDUPS if n + sfx in rows]))
        lines.append(("%s stagger" % dn, rows["stagger" + sfx]))
        lines.append(("%s dazed" % dn, rows["dazed" + sfx]))
    cols = max(len(f) for _, f in lines)
    cz, tz = w * z, tk * z
    x_tok = label + cols * cz + 24
    board = Image.new("RGBA", (x_tok + cols * (tz + 4), 18 + len(lines) * cz), REVIEW_BG + (255,))
    d = ImageDraw.Draw(board)
    d.text((label, 4), "48 px frames at 3x", fill=(230, 226, 210))
    d.text((x_tok, 4), "at the %d px token size, shown at 3x" % tk, fill=(230, 226, 210))
    for r, (title, frs) in enumerate(lines):
        y = 18 + r * cz
        d.text((6, y + 8), title, fill=(230, 226, 210))
        if title.endswith("holds"):
            d.text((6, y + 22), ", ".join(n.replace("windup_", "") for n in rig.WINDUPS if n in frames), fill=(160, 166, 150))
        for c, fr in enumerate(frs):
            board.alpha_composite(fr.resize((cz, cz), Image.NEAREST), (label + c * cz, y))
            board.alpha_composite(views.token(fr).resize((tz, tz), Image.NEAREST), (x_tok + c * (tz + 4), y + cz - tz))
    paths = [os.path.join(out, "%s_windup_board.png" % name)]
    board.save(paths[-1])

    def ms_of(clip):
        t = times[clip]
        return list(t) if isinstance(t, (list, tuple)) else [t] * len(rows[clip])

    def gif(panels, ms, path):
        g = []
        for k in range(len(ms)):
            b = Image.new("RGBA", (w * len(panels), w), REVIEW_BG + (255,))
            for p, frs in enumerate(panels):
                b.alpha_composite(frs[k], (p * w, 0))
            g.append(b.resize((b.width * 4, w * 4), Image.NEAREST).convert("P", palette=Image.ADAPTIVE))
        g[0].save(path, save_all=True, append_images=g[1:], duration=ms, loop=0)
        paths.append(path)

    def held(clip, sfx):  # the intro, then the hold loop twice
        f, t = rows[clip + sfx], ms_of(clip + sfx)
        loop = list(range(rig.LOOP_FROM, len(f)))
        idx = list(range(rig.LOOP_FROM)) + loop + loop
        return [f[i] for i in idx], [t[i] for i in idx]

    attack = main_attack(frames)
    strike = rig.strike_frame(frames[attack])
    seq = []
    for _, sfx in dirs:
        hf, hm = held("windup", sfx)
        af = rows[attack + sfx][strike:]
        seq.append((hf + af, hm + ms_of(attack)[strike:]))
    gif([f for f, _ in seq], seq[0][1], os.path.join(out, "%s_windup_strike.gif" % name))
    for _, sfx in dirs[:1]:
        panels = [held(n, sfx) for n in rig.WINDUPS if n + sfx in rows]
        n = min(len(f) for f, _ in panels)
        gif([f[:n] for f, _ in panels], panels[0][1][:n], os.path.join(out, "%s_windups.gif" % name))
    seq = [(rows["stagger" + sfx] + rows["dazed" + sfx] * 2, ms_of("stagger") + ms_of("dazed") * 2) for _, sfx in dirs]
    gif([f for f, _ in seq], seq[0][1], os.path.join(out, "%s_stagger_dazed.gif" % name))
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
            for p in review(args.review, key, fr, times, pals) + turnaround(args.review, key, fr, times) + windup_review(args.review, key, fr, times):
                print("review", p)
    if args.review:
        p = model_sheet(args.review, args.bodies or list(classes.UNITS) + variants.all_bodies())
        if p:
            print("review", p)


def model_sheet(out: str, keys: list):
    """DIR4-R2 review (decision "Front and back views follow the side model"): models.png, one row per body of `keys`
    with view clips: idle frame 0 side, front (down) and back (up), then the windup hold (frame rig.LOOP_FROM) the same
    way, at 4x, so the three views can be checked as one model. Returns the path written (None: no body has views)."""
    rows = []
    for key in keys:
        cu = unit_of(key)
        if cu is None or not views.supports(cu[1], cu[0]):
            continue
        side = source(key)[2]
        V = views.clips(*cu)[0]
        fr = [side["idle"][0], V["idle_down"][0], V["idle_up"][0]]
        if "windup" in side and "windup_down" in V:
            fr += [None] + [side["windup"][rig.LOOP_FROM], V["windup_down"][rig.LOOP_FROM], V["windup_up"][rig.LOOP_FROM]]
        rows.append((key, fr))
    if not rows:
        return None
    z, label = 4, 130
    cz = rig.W * z
    board = Image.new("RGBA", (label + 7 * cz, 20 + len(rows) * cz), REVIEW_BG + (255,))
    d = ImageDraw.Draw(board)
    for i, t in enumerate(("idle side", "idle front (down)", "idle back (up)", "", "windup side", "windup front", "windup back")):
        d.text((label + i * cz + 6, 4), t, fill=(230, 226, 210))
    for r, (key, fr) in enumerate(rows):
        y = 20 + r * cz
        d.text((6, y + 8), key, fill=(230, 226, 210))
        for c, f in enumerate(fr):
            if f is not None:
                board.alpha_composite(f.resize((cz, cz), Image.NEAREST), (label + c * cz, y))
    path = os.path.join(out, "models.png")
    board.save(path)
    return path


if __name__ == "__main__":
    main()
