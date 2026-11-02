#!/usr/bin/env python3
# Writes the generated placeholder art (ART1-A) on the ASSETORG2 layered tree (docs/specs/ASSETORG2.md section 3):
# flat-shaded shapes with dark outlines in an Ashlands palette.
#   python tools/gen_art.py [group ...] [--layer engine|kit|game ...] [--root DIR] [--overwrite] [--list]
#   python tools/gen_art.py --compare DIR [--spot OUT]
# Groups: bodies portraits items skills statuses terrain overlays region markers pins chips world_map backdrops menu stage_tiles battle_backdrops
# story world_card menu_sunless sunk_backdrops family_icons ui_frames.
# ui_frames (LOOK1-G1) = the Tallow and Iron nine-slice frames ui/frames/<role>.png + .json (exact pixels; the ornate
# frame is high-resolution, its corner and tip from the picked LOOK1-A8 kit, or --ornate SHEET; no sheet, no ornate file written).
# The last five (STORYART1) are the Sunk Ward slice's placeholders in the art bible's 16 hues: story = the vignette
# pictures backgrounds/story/<id>.png (1120 x 472, 2x STORYUI1 V9's 560 x 236 picture area: intro_1-4, meat_market,
# hearth), world_card = backgrounds/title/sunk_ward.png, menu_sunless = backgrounds/menu/main_menu.png (the game's main menu, MENU2; the
# old sunny menu drawing is retired, the menu group now draws only the Ashlands card), sunk_backdrops = the seven zones' backgrounds/battle/<zone>.png, family_icons = the
# scourge and sprayer items/<family>/base.png in the ART2 icon look (they replace CAST1-A7's byte copies of the kit's
# mace and staff icons only while those copies are still in place).
# Layers (default all): engine = engine/assets/ (the common tileset and the common_stage 4:3 battle tiles), kit = kits/rpg/assets/ (class variant bodies,
# common portraits, item bases and generic variants, skill kind icons, statuses, map markers, UI pins, intents, badges
# and chips), game = games/emberwake/assets/ (named bodies and portraits, skill family icons, the named items, the
# Ashlands tiles, backgrounds, backgrounds/battle zone backdrops). DIR is a workspace root (default: this repository); the three asset roots sit under it.
# An existing file is kept (printed "kept"): the committed art includes ART2 portraits and icons made from art_src/
# by tools/portraits/, which these placeholders must not replace; --overwrite rewrites them.
# The six class bodies (units/<class>/base/) and every body of tools/sprites/variants.py (SPRITE2: the goblin variants,
# the dummy, Grakka, the Warchief, the NPCs) belong to the sprite rig (tools/sprites/export_units.py) and are never
# written here; a body drawn here (none since SPRITE2) is a one-frame sheet.png (the 104 px figure) plus clips.json.
# Portraits go to portraits/ (every portrait, rig body or not).
# --compare DIR checks a regeneration (this tool and export_units.py run with --root DIR into an empty folder): the
# same PNG paths and pixel sizes as the committed files, every body with its clips.json; --spot OUT writes committed |
# regenerated pictures of a few bodies side by side. Pixels may drift with numpy and Pillow versions.
# Needs Pillow and numpy. Output is deterministic (seeded noise). Prints "wrote <layer>:<path>" per file.
import argparse
import hashlib
import importlib
import json
import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont, UnidentifiedImageError

OUT = "#14110f"
CHAR = "#1d1c22"
BAS = "#3a3d45"
BAS2 = "#565a66"
EMBER = "#e8742a"
EMBER2 = "#ffb347"
ASH = "#d8d4cc"
BLOOD = "#7a1f1f"
GOB = "#6f8a5a"
GOB2 = "#4f6640"
STEEL = "#9aa6b4"
STEEL2 = "#6c7886"
GOLD = "#e0b43c"
LEATHER = "#7a5233"
WOOD = "#8a6a44"
BONE = "#e8e0c8"
SKIN = "#d9a679"

WORKSPACE = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
GAME, KIT = "emberwake", "rpg"
LAYERS = {"engine": ("engine", "assets"), "kit": ("kits", KIT, "assets"), "game": ("games", GAME, "assets")}
# a placeholder body: one 104 px frame (ASSETORG2 section 8; SPRITE1-A2 replaces the class bodies with rig sheets)
BODY_CLIPS = {"frame": [104, 104], "clips": [{"id": "idle", "frames": 1, "ms": [1000]}]}


def rgb(h: str, a: int = 255):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


def shade(h: str, f: float) -> str:
    r, g, b, _ = rgb(h)
    if f >= 0:
        r, g, b = [int(v + (255 - v) * f) for v in (r, g, b)]
    else:
        r, g, b = [int(v * (1 + f)) for v in (r, g, b)]
    return "#%02x%02x%02x" % (r, g, b)


def seed_of(name: str) -> int:
    return int(hashlib.md5(name.encode()).hexdigest()[:8], 16)


class Cv:
    # Canvas drawn in 100-unit space (x across the width, y scaled by the same factor), supersampled and downsized.
    def __init__(self, w: int, h: int = 0, ss: int = 4, bg=None):
        self.w, self.h, self.ss = w, h or w, ss
        self.im = Image.new("RGBA", (self.w * ss, self.h * ss), bg or (0, 0, 0, 0))
        self.d = ImageDraw.Draw(self.im)
        self.k = self.w * ss / 100.0

    def _p(self, pts):
        return [(x * self.k, y * self.k) for x, y in pts]

    def poly(self, pts, fill, ol=OUT, w=2.0):
        p = self._p(pts)
        self.d.polygon(p, fill=rgb(fill) if fill else None)
        if ol:
            self.d.line(p + [p[0]], fill=rgb(ol), width=max(1, int(w * self.k / 2)), joint="curve")

    def ell(self, cx, cy, rx, ry, fill, ol=OUT, w=2.0):
        box = [(cx - rx) * self.k, (cy - ry) * self.k, (cx + rx) * self.k, (cy + ry) * self.k]
        self.d.ellipse(box, fill=rgb(fill) if fill else None, outline=rgb(ol) if ol else None, width=max(1, int(w * self.k / 2)))

    def rect(self, x0, y0, x1, y1, fill, ol=OUT, w=2.0, r=0.0):
        box = [x0 * self.k, y0 * self.k, x1 * self.k, y1 * self.k]
        if r > 0:
            self.d.rounded_rectangle(box, radius=r * self.k, fill=rgb(fill) if fill else None, outline=rgb(ol) if ol else None, width=max(1, int(w * self.k / 2)))
        else:
            self.d.rectangle(box, fill=rgb(fill) if fill else None, outline=rgb(ol) if ol else None, width=max(1, int(w * self.k / 2)))

    def line(self, pts, col, w=3.0, ol=OUT):
        p = self._p(pts)
        if ol:
            self.d.line(p, fill=rgb(ol), width=int((w + 2.2) * self.k / 2), joint="curve")
        self.d.line(p, fill=rgb(col), width=max(1, int(w * self.k / 2)), joint="curve")

    def arc(self, cx, cy, r, a0, a1, col, w=3.0, ol=OUT):
        box = [(cx - r) * self.k, (cy - r) * self.k, (cx + r) * self.k, (cy + r) * self.k]
        if ol:
            self.d.arc(box, a0, a1, fill=rgb(ol), width=int((w + 2.2) * self.k / 2))
        self.d.arc(box, a0, a1, fill=rgb(col), width=max(1, int(w * self.k / 2)))

    def done(self) -> Image.Image:
        return self.im.resize((self.w, self.h), Image.LANCZOS)


def layer_root(root: str, layer: str) -> str:
    return os.path.join(root, *LAYERS[layer])


class Out:
    # Where files go: a workspace root, the layers to write, and whether existing files are replaced.
    def __init__(self, root: str, layers, overwrite: bool):
        self.root, self.layers, self.overwrite = root, set(layers), overwrite

    def target(self, layer: str, rel: str):
        if layer not in self.layers:
            return None
        path = os.path.join(layer_root(self.root, layer), rel)
        if os.path.exists(path) and not self.overwrite:
            print("kept %s:%s" % (layer, rel))
            return None
        os.makedirs(os.path.dirname(path), exist_ok=True)
        return path


def save(img: Image.Image, layer: str, rel: str, out: Out) -> None:
    path = out.target(layer, rel)
    if path:
        img.save(path, optimize=True)
        print("wrote %s:%s" % (layer, rel))


def save_body(img: Image.Image, layer: str, body: str, out: Out) -> None:
    # A placeholder body: units/<class>/<variant>/sheet.png (one idle frame) and its clips.json.
    save(img, layer, "units/%s/sheet.png" % body, out)
    path = out.target(layer, "units/%s/clips.json" % body)
    if path:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(BODY_CLIPS) + "\n")
        print("wrote %s:units/%s/clips.json" % (layer, body))


# ------------------------------------------------------------------------------
# Characters and class tokens (one figure drawing at 104 and 240 px; the team backdrop and rim come from code)
# ------------------------------------------------------------------------------

HUMAN = {"skin": SKIN, "ears": "human"}
GOBLIN = {"skin": GOB, "ears": "goblin", "tusks": True, "eyes": "#f2d23c"}

CHARACTERS = {
    "mara_voss": {**HUMAN, "body": "#4a4f5c", "trim": EMBER, "helm": "steel", "plume": EMBER, "prop": "sword", "bw": 1.05, "scar": True, "eyes": "#3b2a1a", "cloak": BLOOD},
    "tev_quill": {**HUMAN, "skin": "#c99466", "body": "#4d5a3f", "trim": LEATHER, "helm": "hood", "hood": "#3f4a33", "prop": "bow", "eyes": "#3b2a1a", "cloak": "#33402a"},
    "hest_the_quartermaster": {**HUMAN, "skin": "#e0b48c", "body": "#6b4a2f", "trim": GOLD, "helm": "cap", "hair": "#b9b2a4", "beard": "#b9b2a4", "prop": "ledger", "bw": 1.15, "eyes": "#3b2a1a"},
    "bryn_hale": {**HUMAN, "skin": "#d9a679", "body": "#4a4f5c", "trim": EMBER, "helm": "steel", "plume": "#9aa6b4", "prop": "sword", "bw": 1.1, "eyes": "#3b2a1a"},
    "sella_marsh": {**HUMAN, "skin": "#d9ad86", "body": "#52603f", "trim": LEATHER, "helm": "hood", "hood": "#445234", "prop": "bow", "quiver": True, "eyes": "#3b2a1a"},
    "wenn_tallow": {**HUMAN, "skin": "#e6bf99", "body": "#8a6a42", "trim": "#c9b27a", "helm": "cap", "hair": "#a8672e", "prop": "bow", "bw": 0.9, "eyes": "#3b2a1a"},
    "kit_vane": {**HUMAN, "skin": "#e0b48c", "body": "#35323a", "trim": "#6a5a7a", "helm": "hood", "hood": "#2b2830", "prop": "dagger", "bw": 0.9, "eyes": "#3b2a1a"},
    "oren_dusk": {**HUMAN, "skin": "#c99466", "body": "#6a3a2e", "trim": EMBER, "helm": "wizard", "hood": "#4a2a26", "beard": "#8a8a92", "prop": "staff", "eyes": "#3b2a1a"},
    "alda_wren": {**HUMAN, "skin": "#e6bf99", "body": "#d8d4cc", "trim": GOLD, "helm": "cowl", "hood": "#c9c4b8", "prop": "mace", "eyes": "#3b2a1a"},
    "dagny_coal": {**HUMAN, "skin": "#b98458", "body": "#5a3a30", "trim": LEATHER, "helm": "band", "prop": "fists", "bw": 1.25, "scar": True, "eyes": "#3b2a1a"},
    "brask_toller": {**HUMAN, "skin": "#d2a074", "body": "#5a4a36", "trim": GOLD, "helm": "cap", "hair": "#8a7a68", "beard": "#8a7a68", "prop": "ledger", "bw": 1.15, "eyes": "#3b2a1a"},
    "skarn_ironjaw": {**GOBLIN, "body": "#5a3f2c", "trim": STEEL2, "helm": "bare", "prop": "club", "bw": 1.3, "jaw": True, "scar": True},
    "vex_cutpurse": {**GOBLIN, "skin": "#7c9763", "body": "#3a2f33", "trim": BLOOD, "helm": "hood", "hood": "#2d2530", "prop": "dagger", "bw": 0.85},
    "nok_flamebow": {**GOBLIN, "skin": "#667f52", "body": "#5c4a35", "trim": EMBER, "helm": "band", "prop": "bow", "quiver": True, "bw": 0.9},
    "gruk_smokecaller": {**GOBLIN, "skin": "#7a8a62", "body": "#4a3048", "trim": "#8e6fb0", "helm": "skull", "prop": "staff", "eyes": "#c7a3ff", "bw": 0.9},
    "bolg_redhand": {**GOBLIN, "skin": "#78875a", "body": "#7a3b2a", "trim": BLOOD, "helm": "mohawk", "hair": "#a82a20", "paint": True, "prop": "axe", "bw": 1.25},
    "ash_brawler": {**GOBLIN, "body": "#4a4036", "trim": STEEL2, "helm": "bare", "prop": "club", "bw": 1.25},
    "ash_skirmisher": {**GOBLIN, "skin": "#7c9763", "body": "#38323a", "trim": "#8a5a3a", "helm": "cap", "prop": "dagger", "bw": 0.85},
    "ash_archer": {**GOBLIN, "skin": "#667f52", "body": "#5c4a35", "trim": "#8a5a3a", "helm": "hood", "hood": "#4a4030", "prop": "bow", "bw": 0.9},
    "ash_shaman": {**GOBLIN, "skin": "#7a8a62", "body": "#3f3a4c", "trim": "#6f9f8f", "helm": "skull", "prop": "staff", "eyes": "#9fe0c8", "bw": 0.9},
    "ash_berserker": {**GOBLIN, "skin": "#78875a", "body": "#6a3a2a", "trim": BLOOD, "helm": "mohawk", "hair": "#ba5a22", "paint": True, "prop": "axe", "bw": 1.2},
    "ash_warchief": {**GOBLIN, "skin": "#6a8052", "body": "#3d3a40", "trim": STEEL, "helm": "horned", "prop": "axe", "bw": 1.3, "scar": True, "cloak": BLOOD},
    "grakka_ashmaw": {**GOBLIN, "skin": "#5f7a4a", "body": "#2f2a2d", "trim": GOLD, "helm": "crown", "prop": "greataxe", "bw": 1.45, "scar": True, "cloak": "#8e2020", "big": True, "eyes": "#ff6a2a", "paint": True},
}

CLASSES = {
    "warrior": {**HUMAN, "body": "#4a4f5c", "trim": STEEL, "helm": "steel", "plume": "#9aa6b4", "prop": "sword", "bw": 1.1, "eyes": "#3b2a1a"},
    "ranger": {**HUMAN, "body": "#4d5a3f", "trim": LEATHER, "helm": "hood", "hood": "#3f4a33", "prop": "bow", "eyes": "#3b2a1a"},
    "brawler": {**HUMAN, "skin": "#c99466", "body": "#6b3d2e", "trim": LEATHER, "helm": "band", "prop": "fists", "bw": 1.3, "scar": True, "eyes": "#3b2a1a"},
    "priest": {**HUMAN, "body": "#d8d4cc", "trim": GOLD, "helm": "cowl", "hood": "#c9c4b8", "prop": "mace", "eyes": "#3b2a1a"},
    "rogue": {**HUMAN, "body": "#35323a", "trim": "#6a5a7a", "helm": "hood", "hood": "#2b2830", "prop": "dagger", "bw": 0.9, "eyes": "#3b2a1a"},
    "mage": {**HUMAN, "body": "#3a3f7a", "trim": "#8fa0ff", "helm": "wizard", "hood": "#2f3466", "beard": "#cfcfd8", "prop": "staff", "eyes": "#3b2a1a"},
    "archer": {**GOBLIN, "skin": "#667f52", "body": "#5c4a35", "trim": EMBER, "helm": "band", "prop": "bow", "bw": 0.9},
    "scout": {**GOBLIN, "skin": "#7c9763", "body": "#4a4a38", "trim": "#8a5a3a", "helm": "hood", "hood": "#3f4030", "prop": "dagger", "bw": 0.8},
    "berserker": {**GOBLIN, "skin": "#78875a", "body": "#6a3a2a", "trim": BLOOD, "helm": "mohawk", "hair": "#a82a20", "paint": True, "prop": "axe", "bw": 1.2},
    "shaman": {**GOBLIN, "skin": "#7a8a62", "body": "#4a3048", "trim": "#8e6fb0", "helm": "skull", "prop": "staff", "eyes": "#c7a3ff", "bw": 0.9},
    "skirmisher": {**GOBLIN, "skin": "#7c9763", "body": "#38323a", "trim": BLOOD, "helm": "cap", "prop": "dagger", "bw": 0.85},
    "goblin_brawler": {**GOBLIN, "body": "#5a3f2c", "trim": STEEL2, "helm": "bare", "prop": "club", "bw": 1.3, "jaw": True},
}


def dummy(c: Cv) -> None:
    c.rect(46, 40, 54, 100, WOOD, w=1.6)
    c.rect(18, 62, 82, 71, "#9a7a50", w=1.6, r=2)
    c.ell(50, 30, 15, 17, "#c9a875")
    for x in (43, 50, 57):
        c.line([(x, 20), (x, 40)], "#a98a58", w=1.0, ol=None)
    c.line([(40, 28), (47, 33)], OUT, w=2, ol=None)
    c.line([(47, 28), (40, 33)], OUT, w=2, ol=None)
    c.line([(53, 28), (60, 33)], OUT, w=2, ol=None)
    c.line([(60, 28), (53, 33)], OUT, w=2, ol=None)
    c.rect(30, 71, 70, 84, "#7a5233", w=1.6, r=2)
    c.rect(26, 92, 74, 100, BAS, w=1.6)
    c.ell(24, 66, 7, 7, "#c9a875", w=1.4)
    c.ell(76, 66, 7, 7, "#c9a875", w=1.4)


def figure(c: Cv, s: dict) -> None:
    skin, bw = s["skin"], s.get("bw", 1.0)
    gob = s["ears"] == "goblin"
    skin_d = shade(skin, -0.25)
    big = s.get("big", False)
    cy = 46 if not big else 44
    if s.get("cloak"):
        c.poly([(50 - 34 * bw, 100), (50 - 20 * bw, 66), (50 + 20 * bw, 66), (50 + 34 * bw, 100)], s["cloak"])
    prop = s.get("prop")
    if prop == "bow":
        c.arc(24, 62, 28, 270, 90, WOOD, w=3.2)
        c.line([(24, 34), (24, 90)], BONE, w=0.8, ol=None)
    if prop == "staff":
        c.line([(82, 22), (82, 100)], WOOD, w=3.4)
        c.ell(82, 20, 7, 7, s["trim"] if gob else "#8fa0ff", w=1.6)
    if prop in ("axe", "greataxe"):
        c.line([(82, 100), (82, 38)], WOOD, w=3.2)
        r = 15 if prop == "greataxe" else 11
        c.poly([(82, 38), (82 + r + 3, 30), (82 + r, 52), (82, 50)], STEEL, w=1.6)
        if prop == "greataxe":
            c.poly([(82, 38), (82 - r - 3, 30), (82 - r, 52), (82, 50)], STEEL, w=1.6)
    if prop == "club":
        c.line([(84, 100), (80, 42)], WOOD, w=4.5)
        c.ell(80, 38, 9, 11, "#6a4a2c", w=1.6)
        for dx, dy in ((-3, 34), (3, 40), (0, 30)):
            c.ell(80 + dx, dy, 1.5, 1.5, STEEL, ol=None)
    if prop == "sword":
        c.line([(84, 100), (84, 30)], "#c9d2dc", w=4.5)
        c.line([(77, 74), (91, 74)], GOLD, w=3.2)
    if prop == "mace":
        c.line([(82, 100), (82, 40)], WOOD, w=3.2)
        c.ell(82, 34, 8, 8, GOLD, w=1.6)
    if prop == "quiver" or s.get("quiver"):
        c.rect(66, 44, 76, 84, LEATHER, w=1.4)
        for x in (68, 71, 74):
            c.line([(x, 44), (x + 1, 34)], BONE, w=1.1)
    # body
    c.poly([(50 - 30 * bw, 101), (50 - 26 * bw, 76), (50 - 12, 68), (50 + 12, 68), (50 + 26 * bw, 76), (50 + 30 * bw, 101)], s["body"])
    c.poly([(50 - 12, 68), (50, 84), (50 + 12, 68), (50 + 6, 66), (50 - 6, 66)], s["trim"], w=1.4)
    for sx in (-1, 1):
        c.ell(50 + sx * 27 * bw, 78, 9, 7, s["trim"], w=1.6)
    c.rect(43, 56, 57, 70, skin_d, w=1.6)
    # hood / cowl / wizard hat behind the face
    if s.get("helm") in ("hood", "cowl"):
        c.poly([(26, 70), (28, 36), (50, 20), (72, 36), (74, 70), (62, 62), (38, 62)], s["hood"])
    # ears and head
    if gob:
        for sx in (-1, 1):
            c.poly([(50 + sx * 17, cy - 4), (50 + sx * 44, cy - 16), (50 + sx * 38, cy - 2), (50 + sx * 17, cy + 8)], skin, w=1.6)
            c.poly([(50 + sx * 21, cy - 2), (50 + sx * 37, cy - 11), (50 + sx * 33, cy)], shade(skin, -0.3), ol=None)
        c.ell(50, cy, 21 if not big else 23, 18, skin)
    else:
        for sx in (-1, 1):
            c.ell(50 + sx * 15, cy + 2, 3.2, 4.5, skin, w=1.4)
        c.ell(50, cy, 15.5, 19, skin)
    ey = cy - 2
    eye_col = s.get("eyes", "#3b2a1a")
    for sx in (-1, 1):
        if gob:
            c.poly([(50 + sx * 5, ey + 1), (50 + sx * 15, ey - 3), (50 + sx * 14, ey + 3)], eye_col, w=1.0)
            c.ell(50 + sx * 11, ey, 1.2, 1.6, OUT, ol=None)
        else:
            c.ell(50 + sx * 6.5, ey, 1.9, 2.2, eye_col, w=1.0)
            c.line([(50 + sx * 4, ey - 5), (50 + sx * 9.5, ey - 5.5)], shade(skin, -0.45), w=1.2, ol=None)
    if s.get("paint"):
        for sx in (-1, 1):
            c.line([(50 + sx * 9, cy + 1), (50 + sx * 17, cy + 5)], BLOOD, w=2.0, ol=None)
            c.line([(50 + sx * 8, cy + 5), (50 + sx * 15, cy + 9)], BLOOD, w=2.0, ol=None)
    if gob:
        c.ell(50, cy + 5, 2, 1.3, skin_d, ol=None)
        c.line([(41, cy + 11), (59, cy + 11)], OUT, w=1.6, ol=None)
        if s.get("tusks"):
            for sx in (-1, 1):
                c.poly([(50 + sx * 7, cy + 11), (50 + sx * 10, cy + 3), (50 + sx * 11, cy + 11)], BONE, w=1.0)
    else:
        c.line([(45, cy + 11), (55, cy + 11)], shade(skin, -0.5), w=1.4, ol=None)
    if s.get("beard"):
        c.poly([(36, cy + 6), (50, cy + 12), (64, cy + 6), (60, cy + 24), (50, cy + 30), (40, cy + 24)], s["beard"], w=1.4)
        c.line([(45, cy + 11), (55, cy + 11)], shade(s["beard"], -0.5), w=1.2, ol=None)
    if s.get("jaw"):
        c.rect(38, cy + 10, 62, cy + 17, skin, w=1.4, r=3)
    if s.get("scar"):
        c.line([(41, cy - 12), (58, cy + 6)], "#c0392b" if not gob else "#4a2a24", w=1.6, ol=None)
    # headgear
    h = s.get("helm", "bare")
    if h in ("steel", "horned"):
        c.poly([(33, cy - 3), (34, cy - 20), (50, cy - 28), (66, cy - 20), (67, cy - 3), (60, cy - 10), (40, cy - 10)], STEEL)
        c.rect(47, cy - 12, 53, cy + 8, STEEL2, w=1.2)
        if s.get("plume"):
            c.poly([(46, cy - 27), (50, cy - 40), (54, cy - 27)], s["plume"], w=1.4)
        if h == "horned":
            for sx in (-1, 1):
                c.poly([(50 + sx * 15, cy - 22), (50 + sx * 30, cy - 38), (50 + sx * 24, cy - 16)], BONE, w=1.4)
    elif h == "cap":
        c.poly([(33, cy - 6), (36, cy - 20), (50, cy - 24), (64, cy - 20), (67, cy - 6), (50, cy - 10)], s.get("trim", LEATHER) if gob else "#6b4a2f", w=1.6)
        if s.get("hair"):
            c.line([(33, cy - 4), (31, cy + 8)], s["hair"], w=3, ol=None)
            c.line([(67, cy - 4), (69, cy + 8)], s["hair"], w=3, ol=None)
    elif h == "band":
        c.rect(33, cy - 10, 67, cy - 4, s["trim"], w=1.4)
    elif h in ("hood", "cowl"):
        c.poly([(32, cy - 4), (36, cy - 20), (50, cy - 26), (64, cy - 20), (68, cy - 4), (60, cy - 12), (40, cy - 12)], s["hood"], w=1.6)
    elif h == "wizard":
        c.poly([(28, cy - 8), (72, cy - 8), (60, cy - 18), (54, cy - 46), (40, cy - 18)], s["hood"], w=1.8)
        c.rect(30, cy - 12, 70, cy - 6, s["trim"], w=1.2)
    elif h == "mohawk":
        for i, x in enumerate((36, 43, 50, 57, 64)):
            hgt = 18 + (6 if i == 2 else 0) - abs(i - 2) * 2
            c.poly([(x - 4, cy - 14), (x, cy - 14 - hgt), (x + 4, cy - 14)], s["hair"], w=1.2)
    elif h == "skull":
        c.poly([(36, cy - 14), (44, cy - 26), (56, cy - 26), (64, cy - 14)], BONE, w=1.6)
        c.ell(45, cy - 20, 2, 2.4, OUT, ol=None)
        c.ell(55, cy - 20, 2, 2.4, OUT, ol=None)
        for sx in (-1, 1):
            c.poly([(50 + sx * 10, cy - 24), (50 + sx * 24, cy - 40), (50 + sx * 17, cy - 20)], BONE, w=1.3)
        for dx, col in ((-6, s["trim"]), (0, EMBER2), (6, s["trim"])):
            c.line([(50 + dx, cy - 26), (50 + dx * 1.6, cy - 38)], col, w=2.0, ol=None)
    elif h == "crown":
        c.poly([(33, cy - 12), (33, cy - 30), (41, cy - 20), (50, cy - 34), (59, cy - 20), (67, cy - 30), (67, cy - 12)], GOLD, w=1.8)
        c.ell(50, cy - 20, 2.2, 2.2, "#d6382b", w=1.0)
    if prop == "dagger":
        c.poly([(86, 100), (83, 56), (89, 56)], "#c9d2dc", w=1.4)
        c.rect(80, 54, 92, 58, LEATHER, w=1.2)
    if prop == "fists":
        for sx in (-1, 1):
            c.ell(50 + sx * 32, 90, 9, 9, skin, w=1.6)
    if prop == "ledger":
        c.rect(32, 84, 68, 100, "#5b3b22", w=1.6)
        c.rect(36, 87, 64, 100, "#e6dcc0", w=1.0)
        c.line([(40, 92), (60, 92)], OUT, w=0.9, ol=None)
        c.line([(40, 96), (56, 96)], OUT, w=0.9, ol=None)


# Where each drawing goes on the ASSETORG2 tree (spec sections 3 and 9b; tools/assetorg2_move.py made the same move).
# The six class bodies (<class>/base) are the sprite rig's (tools/sprites/export_units.py): only their common portraits
# are drawn here. Party heroes and Gruk use a class or variant body: only their portraits are drawn.
RIG_CLASSES = ("warrior", "ranger", "brawler", "priest", "rogue", "mage")
# kit variant bodies: drawing key -> body (<class>/<variant>); their portraits are portraits/common/<class>_<variant>.png
VARIANT_BODIES = {"goblin_brawler": "brawler/goblin", "skirmisher": "rogue/goblin", "archer": "ranger/goblin", "shaman": "priest/goblin",
                  "berserker": "brawler/goblin_berserker", "scout": "rogue/goblin_scout", "dummy": "dummy/base"}
# the game's named bodies: character -> class (body <class>/<character>); all on the rig since SPRITE2 (the named goblins
# Skarn, Vex, Nok and Bolg wear their goblin body with a palette and have no body of their own)
NAMED_BODIES = {"hest_the_quartermaster": "warrior", "brask_toller": "warrior", "wenn_tallow": "ranger", "ash_warchief": "brawler",
                "grakka_ashmaw": "warrior"}
# the game's unique portraits: character -> portraits/<folder>/<character>.png
PORTRAIT_FOLDER = {"mara_voss": "heroes", "bryn_hale": "heroes", "tev_quill": "heroes", "sella_marsh": "heroes", "kit_vane": "heroes",
                   "oren_dusk": "heroes", "alda_wren": "heroes", "dagny_coal": "heroes", "hest_the_quartermaster": "npcs",
                   "brask_toller": "npcs", "wenn_tallow": "npcs", "skarn_ironjaw": "elites", "vex_cutpurse": "elites",
                   "nok_flamebow": "elites", "gruk_smokecaller": "elites", "bolg_redhand": "elites", "ash_warchief": "elites",
                   "grakka_ashmaw": "bosses"}
# ASSETORG2 open question 3: the ash_* mobs show the goblin variants; their drawings are not written
RETIRED = {"ash_brawler", "ash_skirmisher", "ash_archer", "ash_shaman", "ash_berserker"}


def draw_figure(spec, size: int) -> Image.Image:
    c = Cv(size)
    if spec is None:
        dummy(c)
    else:
        figure(c, spec)
    return c.done()


def drawings():
    # (key, spec) of every figure drawing: the characters, the class and variant figures, the dummy (spec None).
    return list(CHARACTERS.items()) + list(CLASSES.items()) + [("dummy", None)]


def check_figure_tables() -> None:
    for key, _ in drawings():
        known = key in RIG_CLASSES or key in VARIANT_BODIES or key in PORTRAIT_FOLDER or key in RETIRED
        if not known:
            sys.exit("gen_art: no ASSETORG2 place for the drawing '%s' (add it to the tables above figure())" % key)


def rig_bodies() -> set:
    # SPRITE2: the bodies the sprite rig writes (tools/sprites/variants.py), never written here
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "sprites"))
    variants = importlib.import_module("variants")  # the rig's body list (tools/sprites/; needs Pillow like this tool)
    return set(variants.all_bodies())


def g_bodies(out: Out) -> None:
    check_figure_tables()
    rig = rig_bodies()
    for key, spec in drawings():
        if key in VARIANT_BODIES and VARIANT_BODIES[key] not in rig:
            save_body(draw_figure(spec, 104), "kit", VARIANT_BODIES[key], out)
        elif key in NAMED_BODIES and "%s/%s" % (NAMED_BODIES[key], key) not in rig:
            save_body(draw_figure(spec, 104), "game", "%s/%s" % (NAMED_BODIES[key], key), out)


def g_portraits(out: Out) -> None:
    check_figure_tables()
    for key, spec in drawings():
        if key in RIG_CLASSES:
            save(draw_figure(spec, 240), "kit", "portraits/common/%s_base.png" % key, out)
        elif key in VARIANT_BODIES:
            cls, var = VARIANT_BODIES[key].split("/")
            save(draw_figure(spec, 240), "kit", "portraits/common/%s_%s.png" % (cls, var), out)
        elif key in PORTRAIT_FOLDER:
            save(draw_figure(spec, 240), "game", "portraits/%s/%s.png" % (PORTRAIT_FOLDER[key], key), out)


# ------------------------------------------------------------------------------
# Icon glyphs (100-unit space; shared by items, skills and statuses)
# ------------------------------------------------------------------------------

def blade(c: Cv, x0, y0, x1, y1, w=6.0, col="#c9d2dc", guard=True, hilt="#7a5233"):
    dx, dy = x1 - x0, y1 - y0
    n = math.hypot(dx, dy)
    ux, uy = dx / n, dy / n
    px, py = -uy, ux
    c.poly([(x0 + px * w / 2, y0 + py * w / 2), (x1, y1), (x0 - px * w / 2, y0 - py * w / 2)], col, w=1.8)
    if guard:
        gx, gy = x0, y0
        c.line([(gx - px * w * 1.3, gy - py * w * 1.3), (gx + px * w * 1.3, gy + py * w * 1.3)], GOLD, w=4)
        c.line([(gx, gy), (gx - ux * 14, gy - uy * 14)], hilt, w=5)


def gl_slash(c, col=ASH):
    blade(c, 28, 78, 78, 24, 11, col)


def gl_cross(c, col=ASH):
    blade(c, 28, 78, 80, 26, 9, col)
    blade(c, 72, 78, 20, 26, 9, col)


def gl_stab(c, col=ASH):
    blade(c, 50, 78, 50, 18, 10, col)


def gl_cleave(c, col=ASH):
    c.line([(26, 84), (66, 30)], WOOD, w=5)
    c.poly([(60, 22), (84, 32), (74, 54), (58, 44)], STEEL, w=1.8)


def gl_hammer(c, col=ASH):
    c.line([(30, 84), (64, 40)], WOOD, w=5)
    c.rect(48, 18, 84, 44, STEEL2, w=1.8, r=3)


def gl_arrow(c, col=ASH):
    c.line([(18, 50), (72, 50)], col, w=6)
    c.poly([(70, 30), (92, 50), (70, 70)], col, w=1.8)


def gl_arrows(c, col=ASH):
    for dy in (-14, 0, 14):
        c.line([(16, 50 + dy), (70, 50 + dy)], col, w=3.4)
        c.poly([(68, 44 + dy), (84, 50 + dy), (68, 56 + dy)], col, w=1.4)


def gl_target(c, col=ASH):
    c.ell(50, 50, 32, 32, None, ol=col, w=5)
    c.ell(50, 50, 18, 18, None, ol=col, w=5)
    c.ell(50, 50, 6, 6, EMBER, w=1.5)


def gl_shield(c, col=STEEL):
    c.poly([(22, 22), (78, 22), (78, 54), (50, 84), (22, 54)], col, w=2.4)
    c.poly([(50, 22), (78, 22), (78, 54), (50, 84)], shade(col, -0.25), ol=None)
    c.line([(50, 26), (50, 78)], GOLD, w=3, ol=None)


def gl_plus(c, col="#6fbf73"):
    c.poly([(40, 18), (60, 18), (60, 40), (82, 40), (82, 60), (60, 60), (60, 82), (40, 82), (40, 60), (18, 60), (18, 40), (40, 40)], col, w=2.4)


def gl_flame(c, col=EMBER):
    c.poly([(50, 10), (70, 38), (80, 60), (70, 84), (50, 92), (30, 84), (20, 60), (32, 38), (40, 52)], col, w=2.4)
    c.poly([(50, 44), (62, 62), (58, 82), (50, 86), (42, 82), (38, 62)], EMBER2, ol=None)


def gl_snow(c, col="#9fd8f0"):
    for a in (0, 60, 120):
        r = math.radians(a)
        c.line([(50 - 36 * math.cos(r), 50 - 36 * math.sin(r)), (50 + 36 * math.cos(r), 50 + 36 * math.sin(r))], col, w=5)
    c.ell(50, 50, 7, 7, ASH, w=1.4)


def gl_drop(c, col=BLOOD):
    c.poly([(50, 12), (72, 52), (74, 66), (62, 82), (50, 86), (38, 82), (26, 66), (28, 52)], col, w=2.4)
    c.ell(42, 62, 5, 8, shade(col, 0.4), ol=None)


def gl_skull(c, col=BONE):
    c.ell(50, 44, 28, 26, col, w=2.2)
    c.rect(36, 62, 64, 82, col, w=2.0, r=3)
    c.ell(39, 46, 7, 8, OUT, ol=None)
    c.ell(61, 46, 7, 8, OUT, ol=None)
    c.poly([(47, 58), (53, 58), (50, 64)], OUT, ol=None)


def gl_fist(c, col=SKIN):
    c.rect(26, 34, 76, 80, col, w=2.4, r=9)
    for x in (31, 43, 55, 67):
        c.line([(x, 34), (x, 52)], shade(col, -0.45), w=1.8, ol=None)
    c.rect(66, 52, 86, 66, col, w=2.0, r=5)


def gl_boot(c, col=LEATHER):
    c.poly([(36, 14), (62, 14), (62, 56), (88, 66), (88, 84), (30, 84), (30, 50)], col, w=2.4)
    c.line([(30, 84), (88, 84)], STEEL2, w=4, ol=None)


def gl_spiral(c, col=ASH):
    pts = []
    for i in range(60):
        t = i / 59.0
        a = t * 4.2 * math.pi
        r = 6 + 32 * t
        pts.append((50 + r * math.cos(a), 50 + r * math.sin(a)))
    c.line(pts, col, w=5)


def gl_eye(c, col=ASH):
    c.poly([(8, 50), (30, 28), (50, 24), (70, 28), (92, 50), (70, 72), (50, 76), (30, 72)], col, w=2.4)
    c.ell(50, 50, 14, 14, "#3a6fb0", w=2)
    c.ell(50, 50, 6, 6, OUT, ol=None)


def gl_star(c, col=GOLD):
    pts = []
    for i in range(10):
        r = 40 if i % 2 == 0 else 17
        a = math.radians(-90 + i * 36)
        pts.append((50 + r * math.cos(a), 50 + r * math.sin(a)))
    c.poly(pts, col, w=2.4)


def gl_bolt(c, col=GOLD):
    c.poly([(58, 8), (24, 54), (46, 54), (38, 92), (78, 40), (54, 40)], col, w=2.4)


def gl_shout(c, col=ASH):
    c.poly([(16, 40), (40, 40), (70, 20), (70, 80), (40, 60), (16, 60)], col, w=2.4)
    c.arc(74, 50, 12, -50, 50, col, w=4)
    c.arc(74, 50, 24, -50, 50, col, w=4)


def gl_wave(c, col=ASH):
    for dy, a in ((0, 1.0), (22, 0.7)):
        pts = [(10 + i * 4, 40 + dy + math.sin(i * 0.7) * 10) for i in range(21)]
        c.line(pts, col, w=6)


def gl_bomb(c, col=CHAR):
    c.ell(46, 58, 26, 26, "#2e2e36", w=2.4)
    c.ell(38, 50, 6, 6, BAS2, ol=None)
    c.line([(62, 38), (74, 24)], LEATHER, w=4)
    c.poly([(72, 14), (82, 20), (76, 28), (68, 22)], EMBER2, w=1.4)


def gl_root(c, col="#6a8f3c"):
    c.line([(50, 12), (50, 60)], col, w=7)
    for x0, x1 in ((50, 20), (50, 80), (50, 36), (50, 64)):
        c.line([(x0, 56), (x1, 88)], col, w=5)
    c.line([(26, 36), (50, 52), (74, 36)], col, w=5)


def gl_banner(c, col=EMBER):
    c.line([(28, 14), (28, 90)], WOOD, w=5)
    c.poly([(30, 18), (80, 24), (68, 40), (80, 56), (30, 54)], col, w=2.0)


def gl_swap(c, col=ASH):
    c.line([(20, 36), (76, 36)], col, w=5)
    c.poly([(70, 22), (90, 36), (70, 50)], col, w=1.6)
    c.line([(80, 64), (24, 64)], col, w=5)
    c.poly([(30, 50), (10, 64), (30, 78)], col, w=1.6)


def gl_dizzy(c, col=GOLD):
    c.ell(50, 54, 28, 26, "#d9a679", w=2.2)
    for ang in (-60, 40, 150):
        a = math.radians(ang)
        c.ell(50 + 40 * math.cos(a), 28 + 10 * math.sin(a), 6, 6, col, w=1.4)
    c.line([(38, 52), (46, 60)], OUT, w=2.4, ol=None)
    c.line([(46, 52), (38, 60)], OUT, w=2.4, ol=None)
    c.line([(54, 52), (62, 60)], OUT, w=2.4, ol=None)
    c.line([(62, 52), (54, 60)], OUT, w=2.4, ol=None)


def gl_moon(c, col=ASH):
    c.ell(50, 50, 32, 32, col, w=2.4)
    c.ell(64, 42, 28, 28, "#1d2438", ol=None)


def gl_claw(c, col=ASH):
    for dx in (-18, 0, 18):
        c.poly([(40 + dx, 14), (50 + dx, 26), (62 + dx, 86), (46 + dx, 52)], col, w=1.8)


def gl_chev_up(c, col=ASH):
    c.poly([(14, 52), (50, 18), (86, 52), (86, 70), (50, 36), (14, 70)], col, w=2.0)


def gl_chev_down(c, col="#c9605a"):
    c.poly([(14, 30), (50, 64), (86, 30), (86, 48), (50, 82), (14, 48)], col, w=2.0)
    c.poly([(14, 8), (50, 42), (86, 8), (86, 20), (50, 54), (14, 20)], col, w=2.0)


def gl_sun(c, col=GOLD):
    for a in range(0, 360, 45):
        r = math.radians(a)
        c.line([(50 + 26 * math.cos(r), 50 + 26 * math.sin(r)), (50 + 42 * math.cos(r), 50 + 42 * math.sin(r))], col, w=5)
    c.ell(50, 50, 22, 22, col, w=2.2)


def gl_sparkle(c, col="#bfe9ff"):
    c.poly([(50, 8), (58, 42), (92, 50), (58, 58), (50, 92), (42, 58), (8, 50), (42, 42)], col, w=2.2)


def gl_clock(c, col=ASH):
    c.ell(50, 50, 34, 34, col, w=2.6)
    c.line([(50, 50), (50, 26)], OUT, w=4, ol=None)
    c.line([(50, 50), (66, 58)], OUT, w=4, ol=None)


GLYPHS = {n[3:]: f for n, f in list(globals().items()) if n.startswith("gl_")}

SKILL_KIND_COL = {"ATTACK": EMBER, "DEFEND": "#6f8fb8", "SUPPORT": "#6fbf73", "DISENGAGE": "#d6c14a", "FOCUS": "#a07fd0",
                  "MOVE": "#8aa0b0", "TARGET": "#8aa0b0", "STATUS": "#b0556a", "REST": "#7aa0c0"}


def icon_frame(c: Cv, col: str) -> None:
    c.rect(3, 3, 97, 97, "#15171f", ol=col, w=4, r=12)
    c.rect(9, 9, 91, 91, None, ol=shade(col, -0.55), w=1.2, r=8)


def glyph_icon(size: int, glyph: str, col: str, inset: float = 0.0) -> Image.Image:
    c = Cv(size)
    icon_frame(c, col)
    g = Cv(size)
    GLYPHS[glyph](g)
    gi = g.done()
    s = int(size * (0.72 - inset))
    gi = gi.resize((s, s), Image.LANCZOS)
    img = c.done()
    img.alpha_composite(gi, ((size - s) // 2, (size - s) // 2))
    return img


SKILL_GLYPH = {
    "strike": "slash", "offhand_jab": "stab", "shield_bash": "shield", "hamstring": "root", "shockwave": "wave", "volley": "arrows",
    "firebomb": "bomb", "guard": "shield", "focus": "eye", "move": "boot", "switch_target": "swap", "staggered": "dizzy",
    "stunned": "star", "rest": "moon", "sunder": "cleave", "reckless_swing": "slash", "aimed_shot": "target", "pinning_shot": "root",
    "arcane_bolt": "bolt", "hex": "skull", "haymaker": "fist", "body_blow": "fist", "quick_stab": "stab", "mend": "plus",
    "rally": "banner", "taunt_shout": "shout", "lunge": "arrow", "bleeding_cut": "drop", "riposte": "cross", "purify": "sparkle",
    "backflip": "chev_up", "sweep_kick": "boot", "barbed_arrow": "arrows", "pounce": "claw", "smite": "bolt", "deadeye": "target",
    "judgement": "sun", "charge_in": "arrow", "lacerate": "claw", "assassinate": "skull", "greater_mend": "plus", "renewal": "spiral",
    "leap_back": "chev_up", "arc_lance": "stab", "intimidate": "shout", "cyclone_kick": "spiral", "inferno": "flame",
    "glacial_burst": "snow", "shield_wall": "shield", "crippling_shot": "clock", "rend_armor": "cleave", "envenom": "drop",
    "aegis": "shield", "rejuvenate": "plus", "feint": "swap", "crushing_blow": "hammer", "opportunist": "eye",
}
SKILL_KIND = {
    "offhand_jab": "ATTACK", "guard": "DEFEND", "focus": "FOCUS", "move": "MOVE", "switch_target": "TARGET", "staggered": "STATUS",
    "stunned": "STATUS", "rest": "REST", "mend": "SUPPORT", "rally": "SUPPORT", "taunt_shout": "SUPPORT", "riposte": "DEFEND",
    "purify": "SUPPORT", "backflip": "DISENGAGE", "leap_back": "DISENGAGE", "greater_mend": "SUPPORT", "renewal": "SUPPORT",
    "shield_wall": "DEFEND", "aegis": "SUPPORT", "rejuvenate": "SUPPORT",
}
# the basic skills whose icon is the kit's kind icon (skills/kinds/<kind>.png); every other family is the game's
SKILL_KIND_ICON = {"strike": "attack", "guard": "defend", "focus": "focus", "mend": "support", "backflip": "disengage", "rest": "rest",
                   "move": "move", "switch_target": "target", "staggered": "status"}


def g_skills(out: Out) -> None:
    for fam, glyph in SKILL_GLYPH.items():
        col = SKILL_KIND_COL[SKILL_KIND.get(fam, "ATTACK")]
        if fam in SKILL_KIND_ICON:
            save(glyph_icon(64, glyph, col), "kit", "skills/kinds/%s.png" % SKILL_KIND_ICON[fam], out)
        else:
            save(glyph_icon(64, glyph, col), "game", "skills/%s.png" % fam, out)


STATUS_GLYPH = {
    "stagger": ("dizzy", "bad"), "stun": ("star", "bad"), "root": ("root", "bad"), "bleed": ("drop", "bad"), "weaken": ("chev_down", "bad"),
    "haste": ("bolt", "good"), "taunt": ("shout", "bad"), "riposte": ("cross", "good"), "slow": ("clock", "bad"), "expose": ("target", "bad"),
    "fortify": ("shield", "good"), "regen": ("plus", "good"), "poison": ("drop", "bad"), "burn": ("flame", "bad"),
}


def g_statuses(out: Out) -> None:
    for sid, (glyph, kind) in STATUS_GLYPH.items():
        col = "#58b36a" if kind == "good" else "#c9503f"
        img = glyph_icon(64, glyph, col)
        if sid == "poison":   # green drop
            c = Cv(64)
            icon_frame(c, col)
            g = Cv(64)
            gl_drop(g, "#6aa83c")
            gi = g.done().resize((46, 46), Image.LANCZOS)
            img = c.done()
            img.alpha_composite(gi, (9, 9))
        save(img, "kit", "statuses/%s.png" % sid, out)


# ------------------------------------------------------------------------------
# Items (96x96)
# ------------------------------------------------------------------------------

def item_img(draw_fn, frame="#6d7684") -> Image.Image:
    c = Cv(96)
    c.rect(3, 3, 97, 97, "#161a24", ol=frame, w=3.5, r=11)
    inner = Cv(96)
    draw_fn(inner)
    gi = inner.done().resize((72, 72), Image.LANCZOS)
    img = c.done()
    img.alpha_composite(gi, (12, 12))
    return img


def it_sword(c): blade(c, 28, 76, 80, 20, 12, "#c9d2dc")
def it_greatsword(c): blade(c, 24, 82, 84, 14, 16, "#d3dae2")
def it_shortsword(c): blade(c, 34, 72, 72, 30, 12, "#c0c9d3")


def it_dagger(c):
    blade(c, 38, 70, 68, 28, 9, "#c9d2dc")


def it_hammer(c): gl_hammer(c)


def it_axe(c): gl_cleave(c)


def it_bow(c):
    c.arc(30, 50, 38, -70, 70, WOOD, w=5)
    c.line([(46, 16), (46, 84)], BONE, w=1.6, ol=None)
    c.line([(26, 50), (82, 50)], "#c9d2dc", w=2.6)
    c.poly([(80, 42), (94, 50), (80, 58)], STEEL, w=1.4)


def it_staff(c):
    c.line([(30, 90), (66, 24)], WOOD, w=5.5)
    c.ell(68, 20, 9, 9, "#8fa0ff", w=2)


def it_wand(c):
    c.line([(28, 84), (70, 34)], "#4a3a2a", w=4.5)
    c.poly([(70, 14), (76, 30), (92, 32), (80, 42), (84, 58), (70, 48), (56, 58), (60, 42), (48, 32), (64, 30)], GOLD, w=1.6)


def it_tome(c):
    c.rect(24, 16, 76, 84, "#5a2f3a", w=2.4, r=4)
    c.rect(32, 16, 76, 84, "#7a3f4c", w=2.0, r=3)
    c.ell(54, 50, 10, 10, GOLD, w=1.6)
    c.line([(44, 24), (64, 24)], GOLD, w=2, ol=None)
    c.line([(44, 76), (64, 76)], GOLD, w=2, ol=None)


def it_knuckles(c):
    c.rect(18, 36, 82, 66, STEEL, w=2.4, r=8)
    for x in (28, 42, 56, 70):
        c.ell(x, 51, 6, 7, STEEL2, w=1.6)


def it_shield(c): gl_shield(c, "#7a5233")


def it_plate(c):
    c.poly([(16, 28), (36, 14), (64, 14), (84, 28), (76, 46), (72, 86), (28, 86), (24, 46)], STEEL, w=2.6)
    c.poly([(50, 14), (64, 14), (84, 28), (76, 46), (72, 86), (50, 86)], STEEL2, ol=None)
    c.line([(50, 18), (50, 82)], BAS, w=3, ol=None)
    c.ell(24, 30, 10, 8, STEEL, w=2)
    c.ell(76, 30, 10, 8, STEEL, w=2)


def it_leather(c):
    c.poly([(16, 28), (36, 14), (64, 14), (84, 28), (76, 46), (72, 86), (28, 86), (24, 46)], LEATHER, w=2.6)
    c.line([(36, 14), (50, 36), (64, 14)], "#4a301e", w=3, ol=None)
    for y in (52, 64, 76):
        c.line([(32, y), (68, y)], "#4a301e", w=2.4, ol=None)


def it_robe(c):
    c.poly([(30, 14), (70, 14), (86, 90), (14, 90)], "#3a3f7a", w=2.6)
    c.poly([(42, 14), (58, 14), (50, 38)], ASH, w=1.6)
    c.line([(50, 38), (50, 90)], "#8fa0ff", w=2.4, ol=None)
    c.rect(32, 56, 68, 62, GOLD, w=1.4)


def it_ring(c):
    c.ell(50, 58, 24, 24, None, ol=GOLD, w=10)
    c.poly([(42, 28), (58, 28), (64, 14), (50, 6), (36, 14)], "#d6382b", w=2)


def it_amulet(c):
    c.arc(50, 34, 28, 0, 180, GOLD, w=3.2)
    c.poly([(50, 54), (68, 70), (50, 92), (32, 70)], "#5aa0d8", w=2.4)


def it_potion(c):
    c.rect(42, 10, 58, 28, WOOD, w=1.8, r=2)
    c.poly([(42, 28), (58, 28), (76, 62), (68, 90), (32, 90), (24, 62)], "#d63a3a", w=2.6)
    c.poly([(30, 62), (70, 62), (66, 84), (34, 84)], "#ef6a6a", ol=None)


def it_elixir(c):
    c.rect(42, 10, 58, 28, WOOD, w=1.8, r=2)
    c.poly([(42, 28), (58, 28), (80, 60), (72, 90), (28, 90), (20, 60)], "#3a8fd6", w=2.6)
    c.ell(50, 66, 12, 12, "#9fd8f0", ol=None)


def it_bomb(c): gl_bomb(c)


def it_weapon(c):
    blade(c, 30, 74, 70, 30, 10, STEEL)


def it_misc(c):
    c.poly([(30, 28), (70, 28), (80, 84), (20, 84)], LEATHER, w=2.6)
    c.rect(36, 18, 64, 30, "#4a301e", w=2, r=3)
    c.ell(50, 60, 8, 8, GOLD, w=1.6)


def it_titan_warhammer(c):
    c.line([(26, 88), (62, 34)], WOOD, w=6)
    c.rect(38, 12, 92, 42, STEEL2, w=2.6, r=3)
    c.rect(38, 12, 54, 42, STEEL, w=2.0, r=3)
    c.line([(66, 20), (84, 20)], EMBER, w=2.4, ol=None)


def it_runed_staff(c):
    c.line([(28, 90), (64, 26)], "#4a3a5a", w=6)
    c.ell(66, 20, 11, 11, "#9a7fe0", w=2.4)
    c.ell(66, 20, 4, 4, ASH, ol=None)
    for t in (0.25, 0.5, 0.7):
        x, y = 28 + 36 * t, 90 - 64 * t
        c.line([(x - 4, y), (x + 4, y)], EMBER2, w=2, ol=None)


def it_ember_brigandine(c):
    it_leather(c)
    for x, y in ((38, 40), (58, 40), (48, 54), (38, 68), (58, 68)):
        c.poly([(x, y - 6), (x + 6, y), (x, y + 6), (x - 6, y)], EMBER, w=1.4)


# Family bases that ART2 added (their committed icons are ART2's; these placeholders fill an empty tree).
def it_fist(c): gl_fist(c)


def it_cloth(c):
    c.poly([(22, 22), (40, 14), (60, 14), (78, 22), (72, 44), (70, 84), (30, 84), (28, 44)], "#6a5a7a", w=2.6)
    c.line([(40, 14), (50, 30), (60, 14)], "#3f344a", w=3, ol=None)
    c.rect(32, 52, 68, 58, "#c9b27a", w=1.4)


def it_mace(c):
    c.line([(28, 88), (60, 38)], WOOD, w=5)
    c.ell(66, 30, 16, 16, STEEL2, w=2.4)
    for dx, dy in ((0, -20), (20, 0), (0, 20), (-20, 0)):
        c.poly([(66 + dx * 0.6 - 4, 30 + dy * 0.6), (66 + dx, 30 + dy), (66 + dx * 0.6 + 4, 30 + dy * 0.6)], STEEL, w=1.4)


def it_orb(c):
    c.poly([(34, 88), (40, 70), (60, 70), (66, 88)], WOOD, w=2.2)
    c.ell(50, 44, 26, 26, "#5a7fd0", w=2.6)
    c.ell(42, 36, 8, 8, "#bfe9ff", ol=None)


def it_crossbow(c):
    c.line([(20, 80), (70, 30)], WOOD, w=7)
    c.arc(66, 34, 30, 135, 315, STEEL2, w=5)
    c.line([(45, 13), (87, 55)], BONE, w=1.6, ol=None)
    c.line([(54, 46), (84, 16)], "#c9d2dc", w=2.6)


def it_material(c):
    c.poly([(16, 70), (30, 42), (70, 42), (84, 70), (70, 84), (30, 84)], "#8a8a92", w=2.4)
    c.poly([(30, 42), (70, 42), (62, 56), (38, 56)], "#b0b0b8", ol=None)
    c.ell(56, 66, 6, 5, GOLD, ol=None)


def it_quest(c):
    c.rect(24, 22, 76, 78, "#e6dcc0", w=2.4, r=3)
    c.ell(24, 50, 8, 28, "#cbbf9f", w=2)
    c.ell(76, 50, 8, 28, "#cbbf9f", w=2)
    c.ell(50, 62, 9, 9, BLOOD, w=1.6)
    for y in (34, 44):
        c.line([(36, y), (64, y)], "#6a5a46", w=2, ol=None)


def it_battle_axe(c):
    c.line([(28, 88), (64, 28)], WOOD, w=5)
    c.poly([(54, 14), (82, 22), (78, 50), (60, 40)], STEEL, w=1.8)
    c.poly([(54, 14), (40, 30), (48, 52), (60, 40)], STEEL2, w=1.8)


ITEM_KEYS = ["sword", "greatsword", "shortsword", "dagger", "hammer", "axe", "bow", "staff", "wand", "tome", "knuckles", "shield", "plate",
             "leather", "robe", "ring", "amulet", "potion", "elixir", "bomb", "weapon", "misc", "titan_warhammer", "runed_staff", "ember_brigandine",
             "fist", "cloth", "mace", "orb", "crossbow", "material", "quest", "battle_axe"]
ITEM_FRAME = {"titan_warhammer": GOLD, "runed_staff": "#9a7fe0", "ember_brigandine": EMBER, "potion": "#8a4a4a", "elixir": "#4a6a8a"}
# drawing -> (layer, path under items/ without .png): family bases, the kit's generic variants, the game's named items
ITEM_PATHS = {k: ("kit", k + "/base") for k in ("sword", "dagger", "hammer", "axe", "bow", "staff", "wand", "tome", "shield", "plate",
                                                 "leather", "fist", "cloth", "mace", "orb", "crossbow", "material", "quest")}
ITEM_PATHS.update({k: ("kit", v) for k, v in {
    "knuckles": "fist/knuckles", "robe": "cloth/robe", "greatsword": "sword/greatsword", "shortsword": "sword/shortsword",
    "battle_axe": "axe/battle_axe", "ring": "accessory/base", "amulet": "accessory/amulet", "potion": "consumable/base",
    "elixir": "consumable/elixir", "bomb": "consumable/bomb", "weapon": "any/weapon", "misc": "any/base"}.items()})
ITEM_PATHS.update({"titan_warhammer": ("game", "hammer/titan_warhammer"), "runed_staff": ("game", "staff/runed_staff"),
                   "ember_brigandine": ("game", "leather/ember_brigandine")})


def g_items(out: Out) -> None:
    for k in ITEM_KEYS:
        layer, rel = ITEM_PATHS[k]
        save(item_img(globals()["it_" + k], ITEM_FRAME.get(k, "#6d7684")), layer, "items/%s.png" % rel, out)


# ------------------------------------------------------------------------------
# Terrain tiles (108x108) and overlays
# ------------------------------------------------------------------------------

def noise(rng, w, h, cell):
    small = rng.random((h // cell + 2, w // cell + 2))
    im = Image.fromarray((small * 255).astype(np.uint8)).resize((w + cell * 2, h + cell * 2), Image.BICUBIC)
    return np.asarray(im.crop((0, 0, w, h)), dtype=np.float32) / 255.0


def tint(base: str, n, amp: float):
    r, g, b, _ = rgb(base)
    arr = np.zeros(n.shape + (4,), dtype=np.float32)
    f = 1.0 + (n - 0.5) * amp
    arr[..., 0], arr[..., 1], arr[..., 2], arr[..., 3] = r * f, g * f, b * f, 255
    return arr


TERRAIN = {
    "floor": ("#4a4a52", 0.25), "grass": ("#6a6a3c", 0.4), "dirt": ("#6a4c34", 0.35), "stone": ("#5c5f68", 0.3), "sand": ("#a89878", 0.25),
    "snow": ("#d8d8dc", 0.18), "shallow_water": ("#2f5f70", 0.3), "deep_water": ("#14304a", 0.3), "mud": ("#4a3828", 0.4),
    "lava": ("#c24a1a", 0.6), "wall": ("#2c2e36", 0.3), "tree": ("#4a4a2c", 0.4), "bridge": ("#3a2a1e", 0.2), "rubble": ("#585550", 0.4),
    "burning_ground": ("#3a2a26", 0.4), "toxic_bog": ("#3a5a2a", 0.5), "ice": ("#a8d0e0", 0.2),
}


def terrain_tile(name: str, v: int) -> Image.Image:
    rng = np.random.default_rng(seed_of("%s%d" % (name, v)))
    base, amp = TERRAIN[name]
    n = noise(rng, 108, 108, 12) * 0.6 + noise(rng, 108, 108, 4) * 0.4
    img = Image.fromarray(np.clip(tint(base, n, amp), 0, 255).astype(np.uint8), "RGBA")
    c = Cv(108)
    r = rng

    def pt(m=10):
        return float(r.uniform(m, 100 - m)), float(r.uniform(m, 100 - m))
    if name == "floor":
        for x in (0, 50):
            c.line([(x, 0), (x, 100)], "#2c2c34", w=1.6, ol=None)
        c.line([(0, 50 + v * 3), (100, 50 + v * 3)], "#2c2c34", w=1.6, ol=None)
    elif name == "stone":
        for _ in range(7):
            x, y = pt()
            c.poly([(x, y), (x + 14, y + 3), (x + 12, y + 14), (x - 2, y + 11)], "#6e727c", ol="#33353c", w=1.2)
    elif name == "grass":
        for _ in range(16):
            x, y = pt(6)
            c.line([(x, y), (x + 2, y - 7)], "#8a8a4a", w=1.2, ol=None)
    elif name == "dirt":
        for _ in range(6):
            x, y = pt()
            c.ell(x, y, 3, 2, "#4c3524", ol=None)
    elif name == "sand":
        for i in range(4):
            y = 18 + i * 22 + v * 3
            c.line([(8 + i * 3, y), (50, y - 4), (92 - i * 3, y)], "#8a7a5a", w=1.2, ol=None)
    elif name == "snow":
        for _ in range(10):
            x, y = pt()
            c.ell(x, y, 2, 1.5, "#f4f4f6", ol=None)
    elif name in ("shallow_water", "deep_water"):
        col = "#6aa0b0" if name == "shallow_water" else "#2a5a80"
        for i in range(3):
            x, y = pt(16)
            c.line([(x - 10, y), (x - 4, y - 3), (x + 4, y + 3), (x + 10, y)], col, w=1.6, ol=None)
    elif name == "mud":
        for _ in range(5):
            x, y = pt()
            c.ell(x, y, 5, 3, "#33261a", ol=None)
    elif name == "lava":
        for _ in range(4):
            x, y = pt(10)
            c.line([(x, y), (x + 14, y + 8), (x + 10, y + 22)], EMBER2, w=2.4, ol="#7a2a0a")
    elif name == "wall":
        for row in range(4):
            y = row * 25
            c.line([(0, y), (100, y)], "#15161b", w=2, ol=None)
            for x in range(row % 2 * 17, 100, 34):
                c.line([(x, y), (x, y + 25)], "#15161b", w=2, ol=None)
        c.rect(0, 0, 100, 8, "#4a4d58", ol=None)
    elif name == "tree":
        c.line([(50, 92), (50, 40)], "#2a1e16", w=5)
        for dx, dy, ex, ey in ((50, 60, 30, 40), (50, 52, 72, 34), (50, 44, 40, 20), (50, 70, 70, 56)):
            c.line([(dx, dy), (ex, ey)], "#2a1e16", w=2.6)
    elif name == "bridge":
        for i in range(6):
            c.rect(2, 3 + i * 16.5, 98, 16 + i * 16.5, "#7a5a3a", w=1.4)
    elif name == "rubble":
        for _ in range(9):
            x, y = pt()
            c.poly([(x, y), (x + 9, y + 2), (x + 7, y + 10), (x - 3, y + 7)], "#78746c", ol="#2c2a28", w=1.2)
    elif name == "burning_ground":
        for _ in range(3):
            x, y = pt(16)
            c.poly([(x, y - 10), (x + 6, y), (x + 3, y + 8), (x - 3, y + 8), (x - 6, y)], EMBER, w=1.2)
        for _ in range(5):
            x, y = pt()
            c.ell(x, y, 2, 2, EMBER2, ol=None)
    elif name == "toxic_bog":
        for _ in range(4):
            x, y = pt(14)
            c.ell(x, y, 5, 5, "#8ac03a", ol="#2c4a1a", w=1)
    elif name == "ice":
        for _ in range(3):
            x, y = pt(12)
            c.line([(x, y), (x + 20, y + 14)], "#f0fbff", w=1.4, ol=None)
    img.alpha_composite(c.done())
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, 107, 107], outline=(0, 0, 0, 70))
    return img


def g_terrain(out: Out) -> None:
    for name in TERRAIN:
        for v in range(4):
            save(terrain_tile(name, v), "engine", "tilesets/common/%s/v%d.png" % (name, v), out)


def marker(kind: str, size: int = 32) -> Image.Image:
    c = Cv(size)
    if kind == "npc":
        c.ell(50, 34, 17, 19, "#5fc8e0")
        c.poly([(18, 92), (24, 62), (50, 54), (76, 62), (82, 92)], "#3a9ab8")
    elif kind in ("chest", "chest_open"):
        c.rect(14, 44, 86, 86, "#9a7a2a", w=3)
        c.rect(14, 44, 86, 58, "#c9a43c", w=3)
        if kind == "chest":
            c.poly([(14, 44), (24, 22), (76, 22), (86, 44)], "#c9a43c", w=3)
        else:
            c.poly([(14, 44), (24, 16), (76, 16), (86, 44), (80, 40), (20, 40)], "#8a6a20", w=3)
            c.rect(24, 40, 76, 54, "#2a1a10", w=1.5)
        c.rect(44, 52, 56, 68, "#f0d060", w=2)
    elif kind == "lore":
        c.poly([(8, 28), (48, 36), (48, 84), (8, 76)], "#9a6ad8")
        c.poly([(92, 28), (52, 36), (52, 84), (92, 76)], "#b48ae8")
        for y in (46, 58, 70):
            c.line([(16, y), (40, y + 4)], "#2a1a40", w=2, ol=None)
    elif kind in ("exit", "exit_locked"):
        col = "#5fd078" if kind == "exit" else EMBER
        c.rect(12, 12, 52, 88, col, w=3, r=4)
        c.poly([(40, 38), (66, 38), (66, 26), (94, 50), (66, 74), (66, 62), (40, 62)], "#e8fff0" if kind == "exit" else "#ffe0b0", w=3)
        if kind == "exit_locked":
            c.rect(52, 52, 82, 82, GOLD, w=3, r=3)
            c.arc(67, 52, 9, 180, 360, GOLD, w=4)
    elif kind == "encounter":
        blade(c, 18, 82, 82, 18, 11, "#e0e6ec")
        blade(c, 82, 82, 18, 18, 11, "#e0e6ec")
    elif kind == "roamer":
        c.ell(50, 50, 38, 38, None, ol="#d03a3a", w=5)
        for a in (45, 135):
            r = math.radians(a)
            c.line([(50 - 24 * math.cos(r), 50 - 24 * math.sin(r)), (50 + 24 * math.cos(r), 50 + 24 * math.sin(r))], "#e0e6ec", w=7)
    elif kind == "poi":
        c.ell(50, 50, 34, 34, None, ol="#f4f4f4", w=7)
        c.ell(50, 50, 8, 8, "#f4f4f4", w=1.5)
    elif kind == "objective":
        c.ell(50, 50, 44, 44, None, ol="#7a3a12", w=9)
        c.ell(50, 50, 44, 44, None, ol=GOLD, w=5, )
        c.ell(50, 50, 36, 36, None, ol=EMBER, w=2.5, )
    elif kind == "badge_quest":
        c.ell(50, 50, 44, 44, "#1d1810", ol=GOLD, w=5)
        c.rect(42, 16, 58, 58, GOLD, ol=None)
        c.ell(50, 76, 8, 8, GOLD, ol=None)
    elif kind == "badge_ready":
        c.ell(50, 50, 44, 44, "#1d1810", ol=GOLD, w=5)
        c.arc(50, 38, 17, 180, 40, GOLD, w=9, ol=None)
        c.rect(46, 50, 58, 64, GOLD, ol=None)
        c.ell(52, 78, 8, 8, GOLD, ol=None)
    return c.done()


OVERLAYS = ["npc", "chest", "chest_open", "lore", "exit", "exit_locked", "encounter", "roamer", "poi", "objective"]


def g_overlays(out: Out) -> None:
    for k in OVERLAYS:
        save(marker(k), "kit", "map/markers/%s.png" % k, out)


def g_markers(out: Out) -> None:
    for k in ("quest", "ready"):
        save(marker("badge_" + k), "kit", "ui/badges/%s.png" % k, out)


# ART2-A: intent kind chips (hostile-red rim), foresight/slay/guest chips, one 32 px each (ui namespace).
HOSTILE = "#c23a32"


def cg_burst(c, col=EMBER):
    pts = []
    for i in range(16):
        r = 44 if i % 2 == 0 else 22
        a = math.radians(-90 + i * 22.5)
        pts.append((50 + r * math.cos(a), 50 + r * math.sin(a)))
    c.poly(pts, col, w=2.4)


def cg_back(c, col=ASH):
    c.poly([(14, 50), (46, 18), (46, 38), (86, 38), (86, 62), (46, 62), (46, 82)], col, w=2.4)


def cg_exit(c, col=ASH):
    c.rect(56, 16, 86, 84, None, ol=col, w=5, r=3)
    c.poly([(10, 40), (36, 40), (36, 26), (62, 50), (36, 74), (36, 60), (10, 60)], col, w=2.4)


def cg_question(c, col=ASH):
    c.arc(50, 36, 18, 180, 70, col, w=10)
    c.line([(52, 52), (50, 64)], col, w=10)
    c.ell(50, 82, 7, 7, col, w=1.5)


def cg_chevron(c, col=GOLD):
    c.poly([(16, 20), (46, 20), (80, 50), (46, 80), (16, 80), (50, 50)], col, w=2.4)


def cg_crown(c, col=GOLD):
    c.poly([(14, 76), (14, 32), (34, 52), (50, 24), (66, 52), (86, 32), (86, 76)], col, w=2.4)
    c.line([(18, 66), (82, 66)], shade(col, -0.35), w=3, ol=None)


def cg_guest(c, col="#6fbf9a"):
    gl_shield(c, col)


CHIP_GLYPHS = {
    "attack": lambda c: gl_slash(c, "#e8edf2"), "area": cg_burst, "support": lambda c: gl_plus(c, "#6fbf73"),
    "guard": lambda c: gl_shield(c, "#8fa8c8"), "focus": gl_star, "rest": gl_moon, "retreat": cg_back,
    "move": gl_boot, "flee": None, "switch": gl_swap, "stunned": gl_dizzy, "act": cg_question, "arriving": cg_chevron,
}


def chip(glyph_fn, rim: str, bg: str = "#1d1214", size: int = 32) -> Image.Image:
    c = Cv(size)
    c.ell(50, 50, 46, 46, bg, ol=rim, w=7)
    base = c.done()
    g = Cv(size)
    glyph_fn(g)
    s = int(size * 0.66)
    base.alpha_composite(g.done().resize((s, s), Image.LANCZOS), ((size - s) // 2, (size - s) // 2))
    return base


def flee_chip(size: int = 32) -> Image.Image:
    c = Cv(size)
    c.ell(50, 50, 46, 46, "#1d1214", ol=HOSTILE, w=7)
    base = c.done()
    for fn, box, off in ((lambda g: gl_boot(g, LEATHER), 20, (3, 9)), (cg_exit, 17, (13, 6))):
        g = Cv(size)
        fn(g)
        base.alpha_composite(g.done().resize((box, box), Image.LANCZOS), off)
    return base


def g_chips(out: Out) -> None:
    for k, fn in CHIP_GLYPHS.items():
        img = flee_chip() if k == "flee" else chip(fn, HOSTILE)
        save(img, "kit", "ui/intents/%s.png" % k, out)
    save(chip(gl_eye, GOLD, "#1d1810"), "kit", "ui/badges/foresight.png", out)
    save(chip(cg_crown, GOLD, "#1d1810"), "kit", "ui/chips/slay.png", out)
    save(chip(cg_guest, "#4f9ad0", "#101a22"), "kit", "ui/chips/guest.png", out)


# ------------------------------------------------------------------------------
# Region map tiles (32x32 pixel art, ui icon keys region_*_<n>)
# ------------------------------------------------------------------------------

def px_tile(base: str, seed: str, spots=(), amp=0.22) -> Image.Image:
    rng = np.random.default_rng(seed_of(seed))
    n = rng.random((32, 32))
    img = Image.fromarray(np.clip(tint(base, n, amp), 0, 255).astype(np.uint8), "RGBA")
    d = ImageDraw.Draw(img)
    for (col, cnt, kind) in spots:
        for _ in range(cnt):
            x, y = int(rng.integers(1, 30)), int(rng.integers(1, 30))
            if kind == "dot":
                d.point((x, y), fill=rgb(col))
            elif kind == "h":
                d.line([(x, y), (x + int(rng.integers(2, 5)), y)], fill=rgb(col))
            elif kind == "blob":
                d.rectangle([x, y, x + 1, y + 1], fill=rgb(col))
    return img


def region_tiles() -> dict:
    t = {}
    for i in range(3):
        t["ash_floor_%d" % i] = px_tile("#4a4743", "af%d" % i, [("#5c5852", 14, "dot"), ("#35332f", 10, "dot"), ("#6b665f", 3, "blob")])
        im = px_tile("#2b2d34", "bw%d" % i, [("#1a1b20", 10, "dot")], 0.2)
        d = ImageDraw.Draw(im)
        for y in (0, 10, 21):
            d.line([(0, y), (31, y)], fill=rgb("#14151a"))
        for y, off in ((0, 8), (10, 20), (21, 4)):
            d.line([(off + i * 3, y), (off + i * 3, y + 10)], fill=rgb("#14151a"))
        d.line([(0, 1), (31, 1)], fill=rgb("#4a4d58"))
        d.line([(0, 11), (31, 11)], fill=rgb("#41444e"))
        t["basalt_wall_%d" % i] = im
        im = px_tile("#27495a", "wa%d" % i, [("#4a8aa0", 5, "h"), ("#1a3544", 6, "dot")], 0.2)
        t["water_%d" % i] = im
    for i in range(2):
        t["road_%d" % i] = px_tile("#6a5a46", "rd%d" % i, [("#82705a", 12, "dot"), ("#4a3e2e", 12, "dot")])
        t["lava_%d" % i] = px_tile("#b8441a", "lv%d" % i, [("#ffb347", 10, "dot"), ("#5a1a08", 12, "blob")], 0.4)
        t["rubble_%d" % i] = px_tile("#55524d", "rb%d" % i, [("#78746c", 8, "blob"), ("#2c2a28", 8, "blob")], 0.3)
        t["scrub_%d" % i] = px_tile("#4f4b38", "sc%d" % i, [("#7a7a3c", 8, "h"), ("#33321f", 8, "dot")])
    g = px_tile("#2b2d34", "gate", [("#1a1b20", 6, "dot")])
    d = ImageDraw.Draw(g)
    d.rectangle([6, 4, 25, 31], fill=rgb("#3a2a1e"), outline=rgb("#14110f"))
    for x in (10, 15, 20):
        d.line([(x, 5), (x, 30)], fill=rgb("#6a4a2c"))
    d.rectangle([0, 0, 31, 3], fill=rgb("#4a4d58"))
    t["door_0"] = g
    return t


REGION_TILE_NAMES = {"ash_floor": "floor", "basalt_wall": "wall", "water": "shallow_water"}   # region map characters . # ~


def g_region(out: Out) -> None:
    for k, im in region_tiles().items():
        name, n = k.rsplit("_", 1)
        save(im, "game", "tilesets/ashlands/%s/v%s.png" % (REGION_TILE_NAMES.get(name, name), n), out)


# ------------------------------------------------------------------------------
# Pins (24x24)
# ------------------------------------------------------------------------------

def pin(kind: str) -> Image.Image:
    c = Cv(24, ss=8)
    if kind == "fortress":
        c.poly([(14, 90), (14, 40), (26, 40), (26, 28), (38, 28), (38, 40), (62, 40), (62, 28), (74, 28), (74, 40), (86, 40), (86, 90)], "#9aa0ac", w=4)
        c.rect(40, 62, 60, 90, "#3a2a1e", w=3)
    elif kind == "town":
        c.poly([(10, 90), (10, 52), (50, 18), (90, 52), (90, 90)], "#d8b078", w=4)
        c.rect(40, 62, 60, 90, "#4a301e", w=3)
    elif kind == "camp":
        c.poly([(6, 90), (50, 14), (94, 90)], "#a8794a", w=4)
        c.poly([(40, 90), (50, 54), (60, 90)], "#2a1a10", w=3)
    elif kind == "ruin":
        c.poly([(16, 90), (16, 40), (30, 24), (44, 40), (44, 90)], "#8a8680", w=4)
        c.poly([(56, 90), (56, 60), (68, 50), (84, 62), (84, 90)], "#78746c", w=4)
    elif kind == "gate":
        c.poly([(12, 90), (12, 40), (24, 22), (76, 22), (88, 40), (88, 90), (70, 90), (70, 50), (30, 50), (30, 90)], "#b0b6c0", w=4)
    elif kind == "wild":
        c.poly([(50, 10), (82, 70), (18, 70)], "#4f6640", w=4)
        c.rect(44, 70, 56, 92, "#5a3f2c", w=3)
    elif kind == "here":
        c.ell(50, 50, 40, 40, None, ol=GOLD, w=10)
        c.ell(50, 50, 12, 12, GOLD, w=3)
    return c.done()


def g_pins(out: Out) -> None:
    for k in ("fortress", "town", "camp", "ruin", "gate", "wild", "here"):
        save(pin(k), "kit", "ui/pins/%s.png" % k, out)


# ------------------------------------------------------------------------------
# Scenes (world map, backdrops, menu, title card)
# ------------------------------------------------------------------------------

def vgrad(w, h, top, bot):
    t = np.linspace(0, 1, h)[:, None, None]
    a, b = np.array(rgb(top)[:3], dtype=np.float32), np.array(rgb(bot)[:3], dtype=np.float32)
    arr = a * (1 - t) + b * t
    arr = np.broadcast_to(arr, (h, w, 3)).copy()
    return arr


def to_img(arr) -> Image.Image:
    return Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8), "RGB").convert("RGBA")


def ridge(img: Image.Image, rng, base_y, amp, col, jag=60, top_col=None, w=None):
    w = w or img.width
    xs = np.arange(0, w + jag, jag)
    ys = base_y + (rng.random(len(xs)) - 0.5) * 2 * amp
    pts = [(int(x), int(y)) for x, y in zip(xs, ys)]
    d = ImageDraw.Draw(img)
    d.polygon(pts + [(w + jag, img.height), (0, img.height)], fill=rgb(col))
    if top_col:
        d.line(pts, fill=rgb(top_col), width=3)


def blotch(img: Image.Image, rng, amp=26):
    n = noise(rng, img.width, img.height, 90) * 0.6 + noise(rng, img.width, img.height, 25) * 0.4
    arr = np.asarray(img.convert("RGB"), dtype=np.float32) + (n[..., None] - 0.5) * amp * 2
    return to_img(arr)


def glow(img: Image.Image, x, y, r, col, a=150):
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    d.ellipse([x - r, y - r, x + r, y + r], fill=rgb(col, a))
    layer = layer.filter(ImageFilter.GaussianBlur(r / 2.2))
    img.alpha_composite(layer)


def vignette(img: Image.Image, strength=0.55):
    w, h = img.size
    yy, xx = np.mgrid[0:h, 0:w]
    d = np.sqrt(((xx - w / 2) / (w / 2)) ** 2 + ((yy - h / 2) / (h / 2)) ** 2)
    f = 1 - np.clip(d - 0.55, 0, 1) * strength
    arr = np.asarray(img.convert("RGB"), dtype=np.float32) * f[..., None]
    return to_img(arr)


def embers(img: Image.Image, rng, n=140):
    d = ImageDraw.Draw(img)
    for _ in range(n):
        x, y = int(rng.integers(0, img.width)), int(rng.integers(0, img.height))
        r = int(rng.integers(1, 4))
        d.ellipse([x - r, y - r, x + r, y + r], fill=rgb(EMBER2 if rng.random() > 0.4 else EMBER, int(rng.integers(120, 230))))


def keep_shape(img: Image.Image, x, y, s=1.0, col="#17171c", lit=True):
    c = Cv(int(400 * s), ss=2)
    def P(pts):
        return [(px / 4.0, py / 4.0 * 1.0) for px, py in pts]
    c.poly(P([(20, 400), (20, 150), (60, 150), (60, 100), (100, 100), (100, 150), (300, 150), (300, 100), (340, 100), (340, 150), (380, 150), (380, 400)]), col, ol=None)
    c.poly(P([(140, 150), (140, 60), (160, 40), (180, 60), (180, 150)]), col, ol=None)
    layer = c.done()
    img.alpha_composite(layer, (int(x - layer.width / 2), int(y - layer.height)))
    if lit:
        d = ImageDraw.Draw(img)
        for wx, wy in ((x - 30 * s, y - 130 * s), (x + 20 * s, y - 130 * s), (x - 5 * s, y - 190 * s)):
            d.rectangle([wx, wy, wx + 8 * s + 4, wy + 14 * s + 4], fill=rgb(EMBER2))
        glow(img, x, y - 110 * s, 70 * s, EMBER, 70)


def tent(d, x, y, s, col="#6a5240"):
    d.polygon([(x - 40 * s, y), (x, y - 56 * s), (x + 40 * s, y)], fill=rgb(col), outline=rgb(OUT))
    d.polygon([(x - 8 * s, y), (x, y - 22 * s), (x + 8 * s, y)], fill=rgb("#1a120c"))


def g_world_map(out: Out) -> None:
    rng = np.random.default_rng(seed_of("world"))
    W, H = 1920, 1080
    img = to_img(vgrad(W, H, "#3a3a3e", "#2a2420"))
    img = blotch(img, rng, 18)
    for i, (y, col, top) in enumerate([(300, "#34312f", "#4a4642"), (440, "#2e2b29", "#413d39"), (620, "#2a2725", "#3b3733"), (800, "#262321", "#35312e")]):
        ridge(img, rng, y, 50 + i * 12, col, 70, top)
    img = blotch(img, rng, 10)
    d = ImageDraw.Draw(img, "RGBA")
    # Cinder Pass gorge (a dark crack from lower-left of centre up to the gate)
    gorge = [(760, 760), (820, 700), (880, 640), (940, 590), (1000, 520), (1060, 450), (1110, 380), (1140, 330)]
    left = [(x - 34, y) for x, y in gorge]
    right = [(x + 34, y) for x, y in gorge]
    d.polygon(left + right[::-1], fill=rgb("#0f0d0e"))
    d.line(left, fill=rgb("#5a3a2a"), width=4)
    d.line(right, fill=rgb("#5a3a2a"), width=4)
    d.line(gorge, fill=rgb(EMBER, 120), width=6)
    glow(img, 940, 590, 120, EMBER, 60)
    d = ImageDraw.Draw(img, "RGBA")
    # roads (keep -> courtyard -> gate -> camp)
    roads = [[(360, 800), (480, 760), (600, 700), (700, 620)], [(700, 620), (820, 560), (960, 480), (1100, 380)], [(1100, 380), (1260, 340), (1400, 320), (1500, 300)]]
    for r in roads:
        d.line(r, fill=rgb("#14100c"), width=14, joint="curve")
        d.line(r, fill=rgb("#8a7a60"), width=8, joint="curve")
    # keep
    keep_shape(img, 360, 810, 0.9)
    d = ImageDraw.Draw(img, "RGBA")
    # courtyard ruins
    for dx in (-60, -20, 20, 60):
        h = 40 + (dx % 3) * 8
        d.rectangle([700 + dx - 8, 620 - h, 700 + dx + 8, 620], fill=rgb("#26242a"), outline=rgb(OUT))
    d.arc([660, 560, 740, 640], 180, 360, fill=rgb("#26242a"), width=8)
    # gate arch
    d.rectangle([1070, 330, 1090, 390], fill=rgb("#26242a"), outline=rgb(OUT))
    d.rectangle([1120, 330, 1140, 390], fill=rgb("#26242a"), outline=rgb(OUT))
    d.polygon([(1062, 336), (1148, 336), (1130, 312), (1080, 312)], fill=rgb("#2f2c33"), outline=rgb(OUT))
    # camp
    for i, (dx, dy) in enumerate([(-70, 20), (0, 40), (70, 20), (-30, -30), (40, -30)]):
        tent(d, 1500 + dx, 300 + dy, 1.0, "#6a5240" if i % 2 == 0 else "#5a4636")
    glow(img, 1500, 310, 90, EMBER, 120)
    d = ImageDraw.Draw(img, "RGBA")
    # scorched scrub
    for _ in range(90):
        x, y = int(rng.integers(0, W)), int(rng.integers(300, H))
        d.line([(x, y), (x + int(rng.integers(-6, 6)), y - int(rng.integers(8, 18)))], fill=rgb("#14100c", 160), width=2)
    embers(img, rng, 90)
    img = vignette(img, 0.6)
    save(img, "game", "backgrounds/world_map/ashlands.png", out)


def sky(w, h, top, bot, rng):
    return blotch(to_img(vgrad(w, h, top, bot)), rng, 14)


def g_backdrops(out: Out) -> None:
    W, H = 1280, 720
    # Warden Keep: stone hall with a hearth
    rng = np.random.default_rng(1)
    img = sky(W, H, "#1c1a1e", "#2c2420", rng)
    d = ImageDraw.Draw(img, "RGBA")
    for x in range(0, W, 160):
        d.rectangle([x, 0, x + 18, H], fill=rgb("#15141a"))
    for r in range(0, 520, 52):
        for x in range((r // 52 % 2) * 40, W, 80):
            d.rectangle([x, r, x + 78, r + 50], outline=rgb("#14121a", 150))
    d.rectangle([0, 520, W, H], fill=rgb("#201c1a"))
    d.polygon([(480, 520), (480, 250), (560, 190), (720, 190), (800, 250), (800, 520)], fill=rgb("#14110f"))
    d.polygon([(520, 520), (520, 280), (580, 240), (700, 240), (760, 280), (760, 520)], fill=rgb("#0a0807"))
    glow(img, 640, 470, 230, EMBER, 200)
    glow(img, 640, 500, 110, EMBER2, 190)
    d = ImageDraw.Draw(img, "RGBA")
    for bx in (200, 1000):
        d.polygon([(bx, 80), (bx + 120, 80), (bx + 120, 300), (bx + 60, 340), (bx, 300)], fill=rgb(BLOOD), outline=rgb(OUT))
        d.ellipse([bx + 40, 150, bx + 80, 190], fill=rgb(EMBER))
    embers(img, rng, 40)
    save(vignette(img, 0.7), "game", "backgrounds/location/warden_keep.png", out)
    # Ruined courtyard: broken arches under an ember sky
    rng = np.random.default_rng(2)
    img = sky(W, H, "#3a2a2a", "#7a4a30", rng)
    ridge(img, rng, 430, 40, "#2c2626", 80, "#4a3a36")
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([0, 520, W, H], fill=rgb("#2a2624"))
    for r in range(540, H, 36):
        d.line([(0, r), (W, r)], fill=rgb("#1a1614", 120), width=2)
    for ax, broken in ((180, False), (560, True), (940, False)):
        d.rectangle([ax, 250, ax + 36, 540], fill=rgb("#201c1c"), outline=rgb(OUT))
        top = 330 if broken else 250
        d.rectangle([ax + 160, top, ax + 196, 540], fill=rgb("#201c1c"), outline=rgb(OUT))
        if not broken:
            d.pieslice([ax, 190, ax + 196, 310], 180, 360, fill=rgb("#201c1c"), outline=rgb(OUT))
    for _ in range(14):
        x, y = int(rng.integers(40, W - 40)), int(rng.integers(560, 700))
        d.polygon([(x, y), (x + 30, y - 6), (x + 36, y + 14), (x + 4, y + 18)], fill=rgb("#3a3532"), outline=rgb(OUT))
    glow(img, 900, 160, 200, EMBER, 100)
    embers(img, rng, 70)
    save(vignette(img, 0.6), "game", "backgrounds/location/ruined_courtyard.png", out)
    # Cinder Gate: collapsed toll arch over the gorge
    rng = np.random.default_rng(3)
    img = sky(W, H, "#241c26", "#8a4a2a", rng)
    ridge(img, rng, 380, 70, "#2a2226", 90, "#4a3a3a")
    ridge(img, rng, 500, 50, "#1c181c", 70, "#33292c")
    d = ImageDraw.Draw(img, "RGBA")
    d.polygon([(0, 560), (500, 540), (640, 720), (0, 720)], fill=rgb("#18141a"))
    d.polygon([(1280, 560), (780, 540), (640, 720), (1280, 720)], fill=rgb("#18141a"))
    d.rectangle([400, 180, 470, 560], fill=rgb("#26222a"), outline=rgb(OUT))
    d.rectangle([810, 300, 880, 560], fill=rgb("#26222a"), outline=rgb(OUT))
    d.polygon([(400, 190), (560, 190), (640, 250), (470, 250)], fill=rgb("#2f2b34"), outline=rgb(OUT))
    for (x, y) in ((700, 560), (740, 580), (600, 600)):
        d.polygon([(x, y), (x + 40, y - 10), (x + 48, y + 20), (x + 6, y + 26)], fill=rgb("#3a3538"), outline=rgb(OUT))
    glow(img, 640, 640, 220, EMBER, 120)
    embers(img, rng, 60)
    save(vignette(img, 0.6), "game", "backgrounds/location/cinder_gate.png", out)
    # Ashmaw Camp: hide tents around fires on the wastes
    rng = np.random.default_rng(4)
    img = sky(W, H, "#2a2224", "#6a4230", rng)
    ridge(img, rng, 400, 40, "#2c2624", 80, "#463a36")
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([0, 480, W, H], fill=rgb("#2a2522"))
    for tx, ty, s in ((200, 520, 2.2), (1040, 530, 2.4), (600, 470, 1.6), (880, 470, 1.5), (80, 600, 1.8)):
        tent(d, tx, ty + 60, s, "#6a5240")
    for fx in (460, 760):
        glow(img, fx, 600, 130, EMBER, 180)
        d = ImageDraw.Draw(img, "RGBA")
        d.polygon([(fx - 18, 620), (fx, 570), (fx + 18, 620)], fill=rgb(EMBER2), outline=rgb(OUT))
        d.line([(fx - 30, 628), (fx + 30, 616)], fill=rgb("#3a2a1e"), width=6)
    for px in (330, 960):
        d.line([(px, 380), (px, 560)], fill=rgb("#3a2a1e"), width=6)
        d.polygon([(px, 380), (px + 60, 394), (px, 420)], fill=rgb(BLOOD))
    embers(img, rng, 90)
    save(vignette(img, 0.6), "game", "backgrounds/location/ashmaw_camp.png", out)


def g_menu(out: Out) -> None:
    # the Ashlands world card only; the main menu picture is the sunless one (g_menu_sunless, MENU2)
    rng = np.random.default_rng(8)
    img = sky(1280, 360, "#160f14", "#3a1a12", rng)
    glow(img, 640, 330, 340, EMBER, 120)
    ridge(img, rng, 300, 25, "#0e0b0d", 60, "#2a1c1a")
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([20, 20, 1259, 339], outline=rgb(GOLD, 200), width=3)
    d.rectangle([30, 30, 1249, 329], outline=rgb(GOLD, 90), width=1)
    try:
        font = ImageFont.load_default(size=130)
        small = ImageFont.load_default(size=34)
    except TypeError:
        font = small = ImageFont.load_default()
    text = "ASHLANDS"
    tw = d.textlength(text, font=font)
    d.text(((1280 - tw) / 2 + 4, 76 + 4), text, font=font, fill=rgb("#000000", 200))
    d.text(((1280 - tw) / 2, 76), text, font=font, fill=rgb(ASH))
    sub = "The Ember Wardens"
    sw = d.textlength(sub, font=small)
    d.text(((1280 - sw) / 2, 250), sub, font=small, fill=rgb(EMBER2))
    save(img, "game", "backgrounds/title/ashlands.png", out)


# ------------------------------------------------------------------------------
# Stage art (STAGE1-T): 4:3 ground tiles, wall faces, battle props, zone backdrops
# ------------------------------------------------------------------------------

def quant16(img: Image.Image, n: int = 16, keep_top=None) -> Image.Image:
    # Style A: at most 16 colours per image; only opaque pixels are quantised, transparent ones stay fully transparent
    rgba = img.convert("RGBA")
    a = np.asarray(rgba).copy()
    opaque = a[..., 3] > 0
    if not opaque.any():
        return rgba
    a[~opaque, :3] = a[opaque][0, :3]
    q = Image.fromarray(a[..., :3], "RGB").quantize(colors=n - (1 if keep_top else 0), method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert("RGB")
    out = np.asarray(q).copy()
    res = np.zeros(a.shape, dtype=np.uint8)
    res[..., :3] = out
    res[..., 3] = a[..., 3]
    res[~opaque] = 0
    if keep_top:  # the backdrop's top row stays exactly this colour (one palette slot reserved)
        res[0, :, :3] = keep_top
    return Image.fromarray(res, "RGBA")


def tnoise(rng, w, h, cell):
    # value noise that wraps at the edges, so a tile runs on into its neighbour
    small = rng.random((h // cell, w // cell))
    big = np.tile(small, (3, 3))
    im = Image.fromarray((big * 255).astype(np.uint8)).resize((w * 3, h * 3), Image.BICUBIC)
    return np.asarray(im.crop((w, h, w * 2, h * 2)), dtype=np.float32) / 255.0


class WCv:
    # A 128 x 96 tile canvas in 100-unit x space (y runs 0-75); every shape is drawn at nine offsets so features wrap.
    def __init__(self):
        self.c = Cv(128, 96)

    def _each(self, fn, pts):
        for ox in (-100, 0, 100):
            for oy in (-75, 0, 75):
                fn([(x + ox, y + oy) for x, y in pts])

    def ell(self, x, y, rx, fill, ol=None, w=1.2, ry=None):
        ry = rx * 0.75 if ry is None else ry
        self._each(lambda p: self.c.ell(p[0][0], p[0][1], rx, ry, fill, ol=ol, w=w), [(x, y)])

    def poly(self, pts, fill, ol=None, w=1.2):
        self._each(lambda p: self.c.poly(p, fill, ol=ol, w=w), pts)

    def line(self, pts, col, w=1.4):
        self._each(lambda p: self.c.line(p, col, w=w, ol=None), pts)


def squash(pts, x, y):
    return [(x + dx, y + dy * 0.75) for dx, dy in pts]


def stage_tile(name: str, v: int) -> Image.Image:
    rng = np.random.default_rng(seed_of("stage%s%d" % (name, v)))
    base, amp = TERRAIN[name]
    n = tnoise(rng, 128, 96, 16) * 0.6 + tnoise(rng, 128, 96, 4) * 0.4
    img = Image.fromarray(np.clip(tint(base, n, amp), 0, 255).astype(np.uint8), "RGBA")
    c = WCv()

    def pt():
        return float(rng.uniform(0, 100)), float(rng.uniform(0, 75))
    if name == "floor":
        for y in (0, 37.5):
            c.line([(0, y + v * 2), (100, y + v * 2)], "#2c2c34", w=1.6)
        for row, y in enumerate((0, 37.5)):
            for x in range(row * 25 + v * 5, 100 + row * 25, 50):
                c.line([(x, y + v * 2), (x, y + v * 2 + 37.5)], "#2c2c34", w=1.6)
    elif name == "stone":
        for _ in range(7):
            x, y = pt()
            c.poly(squash([(0, 0), (14, 3), (12, 14), (-2, 11)], x, y), "#6e727c", ol="#33353c")
    elif name == "grass":
        for _ in range(18):
            x, y = pt()
            c.line([(x, y), (x + 2, y - 7)], "#8a8a4a", w=1.2)
    elif name == "dirt":
        for _ in range(8):
            x, y = pt()
            c.ell(x, y, 3, "#4c3524")
    elif name == "sand":
        for i in range(4):
            y = 8 + i * 17 + v * 2
            c.line([(8 + i * 3, y), (50, y - 3), (92 - i * 3, y)], "#8a7a5a", w=1.2)
    elif name == "snow":
        for _ in range(12):
            x, y = pt()
            c.ell(x, y, 2, "#f4f4f6")
    elif name in ("shallow_water", "deep_water"):
        col = "#6aa0b0" if name == "shallow_water" else "#2a5a80"
        for _ in range(4):
            x, y = pt()
            c.line([(x - 10, y), (x - 4, y - 2), (x + 4, y + 2), (x + 10, y)], col, w=1.6)
    elif name == "mud":
        for _ in range(6):
            x, y = pt()
            c.ell(x, y, 5, "#33261a")
    elif name == "lava":
        for _ in range(5):
            x, y = pt()
            c.line(squash([(0, 0), (14, 8), (10, 22)], x, y), EMBER2, w=2.4)
            c.line(squash([(0, 0), (14, 8), (10, 22)], x, y), "#7a2a0a", w=0.8)
    elif name == "wall":
        for row in range(4):
            y = row * 18.75
            c.line([(0, y), (100, y)], "#15161b", w=2)
            for x in range(row % 2 * 17, 100, 34):
                c.line([(x, y), (x, y + 18.75)], "#15161b", w=2)
        c.poly([(0, 0), (100, 0), (100, 6), (0, 6)], "#4a4d58")
    elif name == "bridge":
        for i in range(6):
            y = i * 12.5
            c.poly([(0, y + 1), (100, y + 1), (100, y + 12), (0, y + 12)], "#7a5a3a", ol="#3a2a1e", w=1.2)
    elif name == "rubble":
        for _ in range(9):
            x, y = pt()
            c.poly(squash([(0, 0), (9, 2), (7, 10), (-3, 7)], x, y), "#78746c", ol="#2c2a28")
    elif name == "burning_ground":
        for _ in range(3):
            x, y = pt()
            c.poly([(x, y - 10), (x + 6, y), (x + 3, y + 7), (x - 3, y + 7), (x - 6, y)], EMBER, ol="#3a2a26")
        for _ in range(6):
            x, y = pt()
            c.ell(x, y, 2, EMBER2)
    elif name == "toxic_bog":
        for _ in range(5):
            x, y = pt()
            c.ell(x, y, 5, "#8ac03a", ol="#2c4a1a", w=1)
    elif name == "ice":
        for _ in range(4):
            x, y = pt()
            c.line([(x, y), (x + 14, y + 11)], "#f0fbff", w=1.4)
    img.alpha_composite(c.c.done())
    return img


def wall_face(v: int) -> Image.Image:
    # 128 x 48 (drawn 32 x 12): two brick courses of the wall texture at 75 % brightness, mortar every 6 drawn px
    rng = np.random.default_rng(seed_of("wallface%d" % v))
    base = shade(TERRAIN["wall"][0], -0.25)
    n = tnoise(rng, 128, 48, 16) * 0.5 + tnoise(rng, 128, 48, 4) * 0.5
    img = Image.fromarray(np.clip(tint(base, n, 0.3), 0, 255).astype(np.uint8), "RGBA")
    d = ImageDraw.Draw(img)
    mortar = rgb("#101115")
    for row in range(2):
        y = row * 24
        d.rectangle([0, y, 127, y + 2], fill=mortar)
        d.rectangle([0, y + 3, 127, y + 4], fill=rgb(shade(base, 0.18)))
        for x in range((row + v) % 2 * 20 + v * 6, 128 + 40, 40):
            xx = x % 128
            d.rectangle([xx, y, xx + 2, y + 23], fill=mortar)
    return img


def g_stage_tiles(out: Out) -> None:
    for name in TERRAIN:
        if name == "tree":
            continue
        for v in range(4):
            save(quant16(stage_tile(name, v)), "engine", "tilesets/common_stage/%s/v%d.png" % (name, v), out)
    for v in range(4):
        save(quant16(wall_face(v)), "engine", "tilesets/common_stage/wall_face/v%d.png" % v, out)




# The battle props (props/common/{tree,stones}) are generated art since STAGE1-A2 (2026-10-27): art_src/props/stage_*.png
# cut by tools/sprites/props_cut.py; the code-drawn trees and stones of STAGE1-T are retired.


# Zone backdrops: 384 x 52, opaque; bottom 4 px far ground, 12 columns of 32 px line up with the board
BD_W, BD_H = 384, 52
BD_TOP = (10, 14, 24)
ZONE_GROUND = {"stone_arena": "stone", "forest_clearing": "grass", "river_crossing": "grass", "ruined_courtyard": "stone",
               "desert_outpost": "sand", "frozen_pass": "snow", "lava_cavern": "floor", "swamp_bog": "mud"}


def bd_gap(x: int, cols) -> bool:
    return any(c0 * 32 <= x < (c1 + 1) * 32 for c0, c1 in cols)


def bd_gradient(top, bottom, y0=0, y1=BD_H - 4):
    arr = np.zeros((BD_H, BD_W, 3), dtype=np.float32)
    for y in range(BD_H):
        t = min(1.0, max(0.0, (y - y0) / max(1, y1 - y0)))
        arr[y, :, :] = [top[i] + (bottom[i] - top[i]) * t for i in range(3)]
    return Image.fromarray(arr.astype(np.uint8), "RGB").convert("RGBA")


def bd_finish(img: Image.Image, zone: str) -> Image.Image:
    d = ImageDraw.Draw(img)
    r, g, b, _ = rgb(shade(TERRAIN[ZONE_GROUND[zone]][0], -0.4))
    d.rectangle([0, BD_H - 4, BD_W - 1, BD_H - 1], fill=(r, g, b, 255))
    d.line([(0, BD_H - 5), (BD_W - 1, BD_H - 5)], fill=(max(0, r - 14), max(0, g - 14), max(0, b - 14), 255))
    d.line([(0, 0), (BD_W - 1, 0)], fill=BD_TOP + (255,))
    return img


def bd_trees(d, rng, col, gaps, base_y, h_lo, h_hi, step, pine=False):
    d.rectangle([0, base_y, BD_W - 1, BD_H - 5], fill=col)
    x = int(rng.integers(0, step))
    while x < BD_W:
        if not bd_gap(x, gaps):
            h = int(rng.integers(h_lo, h_hi))
            w = int(rng.integers(8, 14))
            if pine:
                d.polygon([(x - w // 2, base_y), (x + w // 2, base_y), (x, base_y - h)], fill=col)
            else:
                d.ellipse([x - w, base_y - h, x + w, base_y + 3], fill=col)
        x += int(rng.integers(step // 2, step + 3))


def bd_forest(zone, gaps, far=(16, 28, 16), near=(24, 40, 22), bottom=(18, 30, 20)):
    rng = np.random.default_rng(seed_of("bd" + zone))
    img = bd_gradient(BD_TOP, bottom)
    d = ImageDraw.Draw(img)
    bd_trees(d, rng, far + (255,), gaps, 24, 8, 16, 14)
    bd_trees(d, rng, near + (255,), gaps, 34, 10, 20, 17)
    return img


def bd_stone_arena():
    img = bd_gradient(BD_TOP, (28, 28, 34))
    d = ImageDraw.Draw(img)
    d.rectangle([0, 20, BD_W - 1, 21], fill=(20, 20, 26, 255))  # parapet line
    for course, y in enumerate((22, 36)):
        d.rectangle([0, y, BD_W - 1, y + 12], fill=(78, 80, 90, 255) if course else (66, 68, 78, 255))
        d.line([(0, y), (BD_W - 1, y)], fill=(104, 106, 118, 255))
        for x in range(course * 12, BD_W, 24):
            d.line([(x, y), (x, y + 12)], fill=(30, 30, 38, 255))
        d.line([(0, y + 12), (BD_W - 1, y + 12)], fill=(30, 30, 38, 255))
    for x in range(48, BD_W, 96):  # a dim torch
        d.rectangle([x - 1, 26, x + 1, 33], fill=(60, 44, 30, 255))
        d.ellipse([x - 3, 20, x + 3, 27], fill=(224, 130, 50, 255))
        d.ellipse([x - 1, 22, x + 1, 25], fill=(255, 210, 120, 255))
    return img


def bd_river():
    img = bd_forest("river_crossing", ((8, 10),), bottom=(18, 30, 20))
    d = ImageDraw.Draw(img)
    for y in range(8, BD_H - 4):  # the river narrows upward between dark banks
        half = int(6 + (y - 8) * 0.97)
        cx = 304
        d.line([(cx - half - 3, y), (cx + half + 3, y)], fill=(14, 22, 14, 255))
        t = (y - 8) / (BD_H - 12)
        d.line([(cx - half, y), (cx + half, y)], fill=(int(16 + 20 * t), int(36 + 40 * t), int(56 + 40 * t), 255))
    return img


def bd_courtyard():
    rng = np.random.default_rng(seed_of("bdcourtyard"))
    img = bd_gradient(BD_TOP, (34, 32, 38))
    d = ImageDraw.Draw(img)
    gaps = ((3, 3), (7, 7))
    for x0 in range(0, BD_W, 32):
        c = x0 // 32
        if bd_gap(x0, gaps):
            for _ in range(5):  # rubble at the foot of the gap
                rx = x0 + int(rng.integers(2, 24))
                d.ellipse([rx, 40 + int(rng.integers(0, 4)), rx + 6, 46], fill=(88, 84, 80, 255), outline=(36, 34, 32, 255))
            continue
        top = 14 + (c * 7) % 10  # ragged top
        d.rectangle([x0, top, x0 + 31, 47], fill=(70, 70, 80, 255))  # cap
        d.rectangle([x0, top, x0 + 31, top + 2], fill=(98, 100, 112, 255))
        d.rectangle([x0, 34, x0 + 31, 47], fill=(52, 52, 60, 255))  # face
        d.line([(x0, 34), (x0 + 31, 34)], fill=(24, 24, 30, 255))
        for x in range(x0 + (c % 2) * 8, x0 + 32, 16):
            d.line([(x, 35), (x, 47)], fill=(30, 30, 38, 255))
        d.line([(x0, 41), (x0 + 31, 41)], fill=(30, 30, 38, 255))
    return img


def bd_desert():
    rng = np.random.default_rng(seed_of("bddesert"))
    img = bd_gradient((10, 14, 24), (196, 120, 80), 0, 30)
    d = ImageDraw.Draw(img)
    d.polygon([(250, 36), (256, 18), (300, 18), (312, 36)], fill=(104, 62, 50, 255))  # mesa
    d.rectangle([256, 18, 300, 20], fill=(130, 80, 62, 255))
    for col, base, amp, per in (((150, 112, 86, 255), 36, 5, 90), ((190, 160, 118, 255), 42, 4, 60)):
        ph = float(rng.uniform(0, 6))
        for x in range(BD_W):
            top = int(base - amp * (0.5 + 0.5 * math.sin(x / per * 6.28 + ph)))
            d.line([(x, top), (x, BD_H - 5)], fill=col)
    return img


def bd_frozen():
    rng = np.random.default_rng(seed_of("bdfrozen"))
    img = bd_gradient(BD_TOP, (60, 80, 110))
    d = ImageDraw.Draw(img)
    gaps = ((3, 8),)
    for col, base, amp, per, hi in (((70, 92, 128, 255), 30, 14, 130, (150, 175, 205, 255)), ((100, 126, 160, 255), 38, 8, 80, (200, 220, 240, 255))):
        ph = float(rng.uniform(0, 6))
        for x in range(BD_W):
            k = 0.5 + 0.5 * math.sin(x / per * 6.28 + ph)
            if bd_gap(x, gaps):
                k *= 0.15
            top = int(base - amp * k)
            d.line([(x, top), (x, BD_H - 5)], fill=col)
            d.point((x, top), fill=hi)
    bd_trees(d, rng, (30, 50, 80, 255), gaps, 40, 8, 16, 18, pine=True)
    return img


def bd_lava():
    rng = np.random.default_rng(seed_of("bdlava"))
    img = bd_gradient(BD_TOP, (36, 30, 34))
    n = tnoise(rng, BD_W, BD_H // 4 * 4 if BD_H % 4 == 0 else 52, 4)
    px = np.asarray(img).astype(np.float32)
    px[..., :3] *= (0.85 + 0.3 * n)[..., None]
    img = quant16(Image.fromarray(np.clip(px, 0, 255).astype(np.uint8), "RGBA"), 8)
    d = ImageDraw.Draw(img)
    for x in range(0, BD_W, 11):  # stalactite tips along the top
        h = int(rng.integers(5, 14))
        d.polygon([(x, 1), (x + 9, 1), (x + 4, 1 + h)], fill=(70, 60, 72, 255), outline=(14, 12, 18, 255))
    for _ in range(14):  # glowing cracks
        x, y = int(rng.integers(0, BD_W)), int(rng.integers(14, 44))
        pts = [(x, y)]
        for _ in range(4):
            x += int(rng.integers(-5, 6))
            y += int(rng.integers(1, 5))
            pts.append((x, min(y, BD_H - 6)))
        d.line(pts, fill=(190, 60, 16, 255), width=4)
        d.line(pts, fill=(255, 190, 60, 255), width=2)
    return img


def bd_swamp():
    rng = np.random.default_rng(seed_of("bdswamp"))
    img = quant16(bd_gradient(BD_TOP, (62, 84, 66)), 6)
    d = ImageDraw.Draw(img)
    sil = (6, 12, 8, 255)
    x = 10
    while x < BD_W:  # dead trees
        h = int(rng.integers(18, 34))
        d.line([(x, 46), (x, 46 - h)], fill=sil, width=2)
        for k in range(3):
            y = 46 - h + 4 + k * 6
            d.line([(x, y), (x + int(rng.integers(5, 10)) * (1 if k % 2 else -1), y - 5)], fill=sil, width=1)
        x += int(rng.integers(40, 56))
    for x in range(0, BD_W, 5):  # reeds
        h = int(rng.integers(5, 12))
        d.line([(x, 47), (x + int(rng.integers(-1, 2)), 47 - h)], fill=sil)
    mist = Image.new("RGBA", (BD_W, BD_H), (0, 0, 0, 0))
    md = ImageDraw.Draw(mist)
    for y in range(BD_H - 10, BD_H):  # pale mist over the bottom 10 px
        md.line([(0, y), (BD_W, y)], fill=(200, 222, 208, int(70 + 110 * (y - (BD_H - 10)) / 10)))
    img.alpha_composite(mist)
    return img


def g_battle_backdrops(out: Out) -> None:
    zones = {"stone_arena": bd_stone_arena(), "forest_clearing": bd_forest("forest_clearing", ((3, 4), (7, 8))),
             "river_crossing": bd_river(), "ruined_courtyard": bd_courtyard(), "desert_outpost": bd_desert(),
             "frozen_pass": bd_frozen(), "lava_cavern": bd_lava(), "swamp_bog": bd_swamp()}
    for zone, img in zones.items():
        save(quant16(bd_finish(img, zone), keep_top=BD_TOP).convert("RGB"), "game", "backgrounds/battle/%s.png" % zone, out)


# ------------------------------------------------------------------------------
# Story art (STORYART1): code-drawn placeholders for the Sunk Ward slice in the art bible's 16 hues
# (games/emberwake/docs/lore/specs/art_and_visual_style.md section 3): the story scenes, the sunk_ward world card,
# a sunless main menu, the seven Sunk Ward zone backdrops and the scourge and sprayer family icons
# ------------------------------------------------------------------------------

VOID, BASALT, SLATE, OAK = "#0a0b10", "#14171f", "#1e232d", "#2c221e"
IRON, PSTEEL, LINEN, CINDER = "#4b5563", "#9ca3af", "#d9cbb4", "#6b7280"
BLEATHER, TALLOW, CANDLE, ARTERIAL = "#92400e", "#d97706", "#fbbf24", "#e11d48"
CRIMSON, VIOLET, ORCHID, CYAN = "#991b1b", "#9333ea", "#c084fc", "#06b6d4"
ST_W, ST_H = 1120, 472   # story scenes: 2x the vignette picture area (STORYUI1 V9, 560 x 236), filling it unletterboxed


def mix(a: str, b: str, t: float) -> str:
    # b at t over a (the way STORYUI1_TASKS 1.1 states its backdrop colours)
    ca, cb = rgb(a), rgb(b)
    return "#%02x%02x%02x" % tuple(int(round(ca[i] + (cb[i] - ca[i]) * t)) for i in range(3))


def st_canvas(w, h, top, bot, seed, amp=8):
    rng = np.random.default_rng(seed_of(seed))
    return rng, blotch(to_img(vgrad(w, h, top, bot)), rng, amp)


def shroud(img: Image.Image, rng, y0, y1, col, a=170, cell=80, thr=0.48):
    # banded cloud in three flat steps: noise over thr, strongest mid-band
    w, h = img.size
    n = noise(rng, w, h, cell) * 0.7 + noise(rng, w, h, max(4, cell // 3)) * 0.3
    yy = np.arange(h, dtype=np.float32)[:, None]
    band = np.clip(1 - np.abs((yy - (y0 + y1) / 2) / max(1, (y1 - y0) / 2)), 0, 1)
    step = np.floor(np.clip((n - thr) * 4 * band, 0, 0.999) * 3) / 3
    layer = np.zeros((h, w, 4), dtype=np.uint8)
    layer[..., :3] = rgb(col)[:3]
    layer[..., 3] = (step * a).astype(np.uint8)
    img.alpha_composite(Image.fromarray(layer, "RGBA"))


ASH_COLS = (mix(CINDER, VOID, 0.3), CINDER, mix(CINDER, TALLOW, 0.35), mix(TALLOW, BLEATHER, 0.4))   # grey to ember, never white


def ash_fall(img: Image.Image, rng, n, cols=ASH_COLS, rmax=4.5, drift=(-0.7, 1.0)):
    # falling ash: ragged flakes of varied size, each with a faint diagonal streak behind it along drift (dx, dy per
    # unit of fall), so no flake reads as a star; sizes scale with the image width (1120 px = 1x)
    d = ImageDraw.Draw(img, "RGBA")
    k = max(1.0, img.width / 1120.0)
    for _ in range(n):
        x, y = float(rng.integers(0, img.width)), float(rng.integers(0, img.height))
        r = float(rng.uniform(1.4, rmax)) * k
        col = cols[int(rng.integers(0, len(cols)))]
        a = int(rng.integers(90, 200))
        ln = r * float(rng.uniform(1.5, 3.2))
        d.line([(x - drift[0] * ln, y - drift[1] * ln), (x, y)], fill=rgb(col, a // 5), width=max(1, int(r * 0.9)))
        ang = float(rng.uniform(0, math.pi))
        pts = [(x + r * (1.0 if j % 2 == 0 else 0.55) * math.cos(ang + j * math.pi / 3),
                y + r * 0.7 * (1.0 if j % 2 == 0 else 0.55) * math.sin(ang + j * math.pi / 3)) for j in range(6)]
        d.polygon(pts, fill=rgb(col, a))


def stretched_noise(rng, w, h, cell, s):
    # noise with cells s times wider than tall (slow horizontal banding)
    n = noise(rng, max(8, w // s), h, max(2, cell // s))
    return np.asarray(Image.fromarray(n.astype(np.float32), "F").resize((w, h), Image.BICUBIC), dtype=np.float32)


def shroud_sky(img: Image.Image, rng, horizon, a=235):
    # the Shroud: a low, heavy violet-black cloud mass in slow horizontal bands down to horizon, its underside lit by a
    # faint sickly glow from beneath; no sun, moon or stars
    w, h = img.size
    glow(img, w // 2, horizon, int(w * 0.42), mix(ORCHID, CANDLE, 0.25), 55)
    glow(img, int(w * 0.3), horizon + h // 20, int(w * 0.25), mix(VIOLET, CINDER, 0.3), 60)
    yy = np.arange(h, dtype=np.float32)[:, None]
    layers = ((0.0, 0.55, mix(VOID, VIOLET, 0.12), 0.40), (0.15, 0.75, mix(VOID, VIOLET, 0.2), 0.47),
              (0.4, 0.95, mix(VOID, VIOLET, 0.3), 0.52), (0.62, 1.05, mix(mix(VOID, VIOLET, 0.32), ORCHID, 0.18), 0.58))
    for i, (f0, f1, col, thr) in enumerate(layers):
        y0, y1 = horizon * f0, horizon * f1
        n = stretched_noise(rng, w, h, max(24, h // 6), 6) * 0.75 + stretched_noise(rng, w, h, max(8, h // 20), 3) * 0.25
        band = np.clip(1 - np.abs((yy - (y0 + y1) / 2) / max(1, (y1 - y0) / 2)), 0, 1) ** 0.6
        step = np.floor(np.clip((n - thr) * 3.5 * band, 0, 0.999) * 3) / 3
        layer = np.zeros((h, w, 4), dtype=np.uint8)
        layer[..., :3] = rgb(col)[:3]
        layer[..., 3] = (step * a).astype(np.uint8)
        img.alpha_composite(Image.fromarray(layer, "RGBA"))
        if i == len(layers) - 1:   # the lit underside: a thin brighter rim under the lowest bank
            lit = step > 0
            under = lit & ~np.roll(lit, -max(2, h // 120), axis=0)
            rim = np.zeros((h, w, 4), dtype=np.uint8)
            rim[..., :3] = rgb(mix(ORCHID, CANDLE, 0.2))[:3]
            rim[..., 3] = np.where(under, 70, 0).astype(np.uint8)
            img.alpha_composite(Image.fromarray(rim, "RGBA"))


def rain(img: Image.Image, rng, n, col=PSTEEL, a=45, ln=22):
    d = ImageDraw.Draw(img, "RGBA")
    for _ in range(n):
        x, y = int(rng.integers(0, img.width + 40)), int(rng.integers(-ln, img.height))
        d.line([(x, y), (x - 5, y + ln)], fill=rgb(col, a), width=1)


def soft(img: Image.Image, draw_fn, blur):
    # draws on a clear layer, blurs it and lays it over img (smoke, steam, mist)
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    draw_fn(ImageDraw.Draw(layer, "RGBA"))
    img.alpha_composite(layer.filter(ImageFilter.GaussianBlur(blur)))


def arch_pts(x0, x1, ys, yb, f=0.8, n=24):
    # a pointed arch: springing line ys between x0 and x1, jambs down to yb; f 0.5 = round, 1.0 = equilateral
    span = x1 - x0
    r = span * f
    a_end = math.acos(max(-1.0, min(1.0, (span / 2 - r) / r)))
    left = [(x0 + r + r * math.cos(math.pi + (a_end - math.pi) * i / n), ys - r * math.sin(math.pi + (a_end - math.pi) * i / n)) for i in range(n + 1)]
    right = [(x0 + x1 - x, y) for x, y in reversed(left)]
    return [(x0, yb)] + left + right + [(x1, yb)]


def person(d, x, y, h, col, ol=None, bow=0.0, coat=True, bowl=None, arms_out=0.0):
    # a standing adult silhouette (head about 1/7 of the height), feet at (x, y); bow tips the head forward (+x) and down
    u = h / 100.0
    hx, hy = x + bow * 5 * u, y - 92 * u + abs(bow) * 4 * u
    sh, hip = y - 80 * u, y - 46 * u
    reach = 15 + 10 * arms_out
    arm_l = [(x - 12 * u, sh), (x - 15 * u, sh + 4 * u), (x - reach * u, hip + 2 * u), (x - (reach - 4) * u, hip + 3 * u), (x - 10 * u, sh + 10 * u)]
    arm_r = [(2 * x - px, py) for px, py in arm_l]
    if bowl:
        arm_r = [(x + 12 * u, sh), (x + 15 * u, sh + 6 * u), (x + 14 * u, y - 56 * u), (x + 6 * u, y - 54 * u), (x + 9 * u, sh + 10 * u)]
    if coat:
        body = [(x - 12 * u, sh), (x + 12 * u, sh), (x + 14 * u, y - 12 * u), (x + 6 * u, y - 10 * u), (x + 7 * u, y), (x + 2 * u, y),
                (x, y - 14 * u), (x - 2 * u, y), (x - 7 * u, y), (x - 6 * u, y - 10 * u), (x - 14 * u, y - 12 * u)]
    else:
        body = [(x - 12 * u, sh), (x + 12 * u, sh), (x + 10 * u, hip), (x + 8 * u, y), (x + 2 * u, y), (x, y - 38 * u), (x - 2 * u, y),
                (x - 8 * u, y), (x - 10 * u, hip)]
    neck = [(x - 3 * u, sh + 1), (x + 3 * u, sh + 1), (hx + 3 * u, hy + 4 * u), (hx - 3 * u, hy + 4 * u)]
    for pts in (arm_l, arm_r, body, neck):
        d.polygon(pts, fill=rgb(col), outline=rgb(ol) if ol else None)
    d.ellipse([hx - 6.5 * u, hy - 7.5 * u, hx + 6.5 * u, hy + 7.5 * u], fill=rgb(col), outline=rgb(ol) if ol else None)
    if bowl:
        d.ellipse([x + 4 * u, y - 59 * u, x + 18 * u, y - 53 * u], fill=rgb(bowl), outline=rgb(VOID))
    return (hx, hy, u)


def chain(d, p0, p1, sag, col=IRON, hi=PSTEEL, link=7):
    # iron links along a sagging curve from p0 to p1
    (x0, y0), (x1, y1) = p0, p1
    n = max(2, int((math.hypot(x1 - x0, y1 - y0) + sag) / link))
    r = link * 0.55
    for i in range(n + 1):
        t = i / n
        x = x0 + (x1 - x0) * t
        y = y0 + (y1 - y0) * t + sag * 4 * t * (1 - t)
        box = [x - r, y - r * 0.55, x + r, y + r * 0.55] if i % 2 else [x - r * 0.55, y - r, x + r * 0.55, y + r]
        d.ellipse(box, outline=rgb(VOID), width=4)
        d.ellipse(box, outline=rgb(col if i % 2 else hi), width=2)


def iron_mask(d, hx, hy, u):
    # a riveted iron face plate with eye slits over the head (faceless, nothing shown beneath)
    box = [hx - 8 * u, hy - 9 * u, hx + 8 * u, hy + 9.5 * u]
    d.ellipse(box, fill=rgb(IRON), outline=rgb(VOID), width=max(1, int(u)))
    d.chord(box, 200, 340, fill=rgb(mix(IRON, PSTEEL, 0.45)))
    d.line([(hx, hy - 8 * u), (hx, hy + 8 * u)], fill=rgb(mix(IRON, VOID, 0.45)), width=max(1, int(u * 0.8)))
    d.rectangle([hx - 6 * u, hy - 2 * u, hx - 1.5 * u, hy - 0.4 * u], fill=rgb(VOID))
    d.rectangle([hx + 1.5 * u, hy - 2 * u, hx + 6 * u, hy - 0.4 * u], fill=rgb(VOID))
    for rx, ry in ((-6, -4), (6, -4), (-5, 5), (5, 5), (0, 8)):
        d.ellipse([hx + (rx - 0.8) * u, hy + (ry - 0.8) * u, hx + (rx + 0.8) * u, hy + (ry + 0.8) * u], fill=rgb(PSTEEL))


def skyline(img: Image.Image, rng, base_y, col, h_lo, h_hi, lit=0.0, x0=0, x1=None, wmin=40, wmax=110, win=TALLOW, ol=None, bottom=None):
    # tenement roofs down to bottom (default the image's foot): gables, stepped tops and chimneys; windows lit with
    # probability lit
    x1 = x1 or img.width
    bottom = img.height if bottom is None else bottom
    d = ImageDraw.Draw(img, "RGBA")
    x = x0 - int(rng.integers(0, wmin))
    while x < x1:
        w = int(rng.integers(wmin, wmax))
        top = base_y - int(rng.integers(h_lo, h_hi))
        kind = int(rng.integers(0, 3))
        pts = [(x, bottom), (x, top)]
        if kind == 0:
            pts += [(x + w // 2, top - w // 3)]
        elif kind == 1:
            pts += [(x + w // 3, top), (x + w // 3, top - w // 4), (x + w * 2 // 3, top - w // 4), (x + w * 2 // 3, top)]
        pts += [(x + w, top), (x + w, bottom)]
        d.polygon(pts, fill=rgb(col), outline=rgb(ol) if ol else None)
        if rng.random() < 0.6:
            cx = x + int(rng.integers(4, max(5, w - 14)))
            ch = int(rng.integers(h_lo // 4 + 4, h_lo // 2 + 10))
            d.rectangle([cx, top - ch, cx + max(4, w // 9), top + 2], fill=rgb(col), outline=rgb(ol) if ol else None)
        if lit > 0:
            ww = max(3, w // 14)
            for wy in range(top + ww * 2, base_y, ww * 4):
                for wx in range(x + ww * 2, x + w - ww * 2, ww * 4):
                    if rng.random() < lit:
                        d.rectangle([wx, wy, wx + ww, wy + int(ww * 1.5)], fill=rgb(win, int(rng.integers(150, 255))))
        x += w + int(rng.integers(-6, 10))


def cathedral(img: Image.Image, x, base_y, s, col, lit=TALLOW, glow_a=60):
    # a gothic silhouette: nave, two west towers with spires, the bell tower over the crossing, lancet and rose windows
    d = ImageDraw.Draw(img, "RGBA")

    def P(pts):
        return [(x + px * s, base_y + py * s) for px, py in pts]

    def box(x0, y0, x1, y1):
        (a, b), (c, e) = P([(x0, y0), (x1, y1)])
        return [a, b, c, e]

    d.polygon(P([(-170, 0), (-170, -120), (-60, -175), (60, -175), (170, -120), (170, 0)]), fill=rgb(col))
    for tx in (-150, 110):
        d.polygon(P([(tx, 0), (tx, -230), (tx + 20, -330), (tx + 40, -230), (tx + 40, 0)]), fill=rgb(col))
    d.polygon(P([(-28, -170), (-28, -280), (0, -300), (28, -280), (28, -170)]), fill=rgb(col))
    d.polygon(P([(-22, -296), (0, -420), (22, -296)]), fill=rgb(col))
    if lit:
        for wx in (-110, -70, 60, 100):
            d.polygon(P([(wx, -30), (wx, -80), (wx + 7, -92), (wx + 14, -80), (wx + 14, -30)]), fill=rgb(lit, 200))
        d.ellipse(box(-26, -150, 26, -98), fill=rgb(mix(lit, CRIMSON, 0.45), 190))
        for k in range(8):   # rose window tracery
            a = k * math.pi / 4
            d.line(P([(0, -124), (26 * math.cos(a), -124 + 26 * math.sin(a))]), fill=rgb(col), width=max(1, int(3 * s)))
        d.ellipse(box(-26, -150, 26, -98), outline=rgb(col), width=max(1, int(4 * s)))
        d.ellipse(box(-9, -133, 9, -115), fill=rgb(col))
        d.rectangle(box(-6, -262, 6, -240), fill=rgb(mix(lit, CANDLE, 0.4), 230))
        glow(img, x, base_y - 120 * s, 130 * s, lit, glow_a)


def wall_courses(img: Image.Image, y0, y1, col, a=110, ch=34, cw=70):
    d = ImageDraw.Draw(img, "RGBA")
    for i, y in enumerate(range(y0, y1, ch)):
        d.line([(0, y), (img.width, y)], fill=rgb(col, a), width=2)
        for x in range((i % 2) * cw // 2, img.width, cw):
            d.line([(x, y), (x, min(y1, y + ch))], fill=rgb(col, a), width=2)


def st_intro_1():
    # the Shroud where the stars and the sun were: a low cloud mass over the city, ash falling (no sun, moon or stars)
    rng, img = st_canvas(ST_W, ST_H, VOID, mix(VOID, VIOLET, 0.25), "story_intro_1")
    shroud_sky(img, rng, 400)
    skyline(img, rng, 420, mix(VOID, BASALT, 0.6), 30, 80, 0.08, wmin=50, wmax=120)
    cathedral(img, 760, 430, 0.62, VOID, TALLOW, 40)
    skyline(img, rng, 455, VOID, 10, 50, 0.12, wmin=60, wmax=140)
    ash_fall(img, rng, 220)
    return vignette(img, 0.6)


def st_intro_2():
    # the earth in cross-section: the city on the crust, strata, the sleeping titan's bones, black veins up to the wells
    rng, img = st_canvas(ST_W, ST_H, VOID, mix(VOID, VIOLET, 0.25), "story_intro_2", 4)
    surf = 92
    arr = np.asarray(img).astype(np.float32)
    xs = np.arange(ST_W)
    tex = np.floor(noise(rng, ST_W, ST_H, 18) * 3) / 3
    bands = [(surf, mix(OAK, BASALT, 0.3)), (surf + 46, mix(SLATE, OAK, 0.35)), (surf + 104, BASALT), (surf + 168, mix(BASALT, VOID, 0.45))]
    bands = [(by, mix(col, CINDER, 0.2)) for by, col in bands]   # one step lighter so the layers read
    tops = []
    for i, (by, col) in enumerate(bands):
        ph = float(rng.uniform(0, 6))
        top = by + (np.sin(xs / (90 + 30 * i) + ph) * (4 + 3 * i) + np.sin(xs / 23 + ph * 2) * 2).astype(int)
        if i == 0:
            top = np.full(ST_W, surf)
        tops.append(top)
        c = np.array(rgb(col)[:3], dtype=np.float32)
        for x in range(ST_W):
            arr[top[x]:, x, :3] = c * (0.88 + 0.22 * tex[top[x]:, x, None])
    img = to_img(arr[..., :3])
    d = ImageDraw.Draw(img, "RGBA")
    for i, top in enumerate(tops):
        if i:
            d.line(list(zip(xs.tolist(), (top - 1).tolist())), fill=rgb(VOID, 200), width=2)
        nxt = tops[i + 1] if i + 1 < len(tops) else np.full(ST_W, ST_H)
        for f in (0.33, 0.66):   # fossil-like bedding lines inside each layer
            ys = top + (nxt - top) * f + np.sin(xs / 37.0 + i * 2 + f * 5) * 2
            d.line(list(zip(xs[::4].tolist(), ys[::4].tolist())), fill=rgb(mix(bands[i][1], LINEN, 0.14), 120), width=1)
    # the titan's hollow and bones
    d.ellipse([60, 270, 1080, 462], fill=rgb(mix(VOID, VIOLET, 0.07)), outline=rgb(VOID), width=3)
    bone, bone_hi, bone_lo = mix(LINEN, CINDER, 0.45), mix(LINEN, CINDER, 0.15), mix(LINEN, VOID, 0.6)
    spine = [(250 + i * 26, 382 + 16 * math.sin(i / 5.0)) for i in range(30)]
    for i, (sx, sy) in enumerate(spine[:22]):   # ribs over the front half
        if 2 <= i <= 16 and i % 2 == 0:
            k = 1.0 - abs(i - 8) / 12.0
            rib = [(sx + 40 * k * t * 0.4 + 30 * t * t, sy - 95 * k * math.sin(t * math.pi * 0.85)) for t in np.linspace(0, 1, 14)]
            d.line(rib, fill=rgb(VOID), width=13, joint="curve")
            d.line(rib, fill=rgb(bone), width=8, joint="curve")
            d.line([(px - 2, py - 2) for px, py in rib[2:9]], fill=rgb(bone_hi), width=2)
    for i, (sx, sy) in enumerate(spine):
        r = 11 - i * 0.22
        d.ellipse([sx - r, sy - r * 0.8, sx + r, sy + r * 0.8], fill=rgb(bone), outline=rgb(VOID), width=3)
        d.line([(sx, sy - r * 0.8), (sx + 4, sy - r * 1.6)], fill=rgb(bone_lo), width=4)
    arm = [(760, 386), (800, 330), (842, 300)]   # the forelimb reaching up toward the crust
    for a, b in zip(arm, arm[1:]):
        d.line([a, b], fill=rgb(VOID), width=18)
        d.line([a, b], fill=rgb(bone), width=12)
    for fx, fy in ((818, 262), (846, 252), (874, 262), (890, 284)):
        d.line([(842, 300), (fx, fy)], fill=rgb(VOID), width=11)
        d.line([(842, 300), (fx, fy)], fill=rgb(bone), width=6)
        d.ellipse([fx - 4, fy - 4, fx + 4, fy + 4], fill=rgb(bone_hi), outline=rgb(VOID))
    # the skull: long and heavy, jaw shut, half buried in the rock of the hollow's floor
    d.polygon([(292, 318), (318, 300), (356, 290), (402, 296), (362, 302), (334, 316), (318, 330)], fill=rgb(bone_lo), outline=rgb(VOID), width=4)   # a swept-back horn
    d.line([(318, 302), (356, 292), (390, 296)], fill=rgb(bone), width=3)
    skull = [(84, 378), (100, 356), (160, 338), (232, 318), (290, 308), (336, 324), (352, 356), (330, 392), (220, 400), (110, 394)]
    d.polygon(skull, fill=rgb(bone), outline=rgb(VOID), width=4)
    d.polygon([(104, 354), (160, 336), (232, 316), (290, 306), (330, 320), (290, 318), (232, 328), (160, 346)], fill=rgb(bone_hi))   # the brow and snout ridge
    d.polygon([(226, 346), (262, 334), (284, 340), (264, 352)], fill=rgb(VOID))   # a long, shut eye socket
    d.ellipse([100, 360, 112, 368], fill=rgb(VOID))   # nostril
    d.line([(96, 380), (180, 378), (300, 380)], fill=rgb(VOID), width=3)   # the closed jaw
    d.line([(190, 352), (240, 366), (300, 360)], fill=rgb(bone_lo), width=3)   # cheekbone
    for cx0, cy0 in ((150, 352), (196, 344), (310, 340)):   # cracks
        d.line([(cx0, cy0), (cx0 + 8, cy0 + 10), (cx0 + 4, cy0 + 18)], fill=rgb(bone_lo), width=2)
    rock = mix(mix(BASALT, VOID, 0.2), CINDER, 0.2)
    ridge_pts = [(x, 368 + 6 * math.sin(x / 23.0) + 4 * math.sin(x / 9.0) + max(0, x - 360) * 0.9) for x in range(40, 471, 10)]
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))   # the rock mound, kept inside the hollow
    ld = ImageDraw.Draw(layer, "RGBA")
    ld.polygon([(40, ST_H)] + ridge_pts + [(470, ST_H)], fill=rgb(rock))
    ld.line(ridge_pts, fill=rgb(VOID), width=3)
    for j in range(3):   # bedding lines in the rock over the buried jaw
        ld.line([(x, 388 + j * 18 + 3 * math.sin(x / 29.0 + j)) for x in range(40, 470, 6)], fill=rgb(mix(rock, LINEN, 0.14), 140), width=1)
    hollow = Image.new("L", img.size, 0)
    ImageDraw.Draw(hollow).ellipse([62, 272, 1078, 460], fill=255)
    layer.putalpha(Image.fromarray(np.minimum(np.asarray(layer)[..., 3], np.asarray(hollow))))
    img.alpha_composite(layer)
    d = ImageDraw.Draw(img, "RGBA")
    # black veins from the bones up through the strata to the wells
    wells = []
    for sx, sy in ((290, 330), (430, 300), (560, 300), (690, 330), (840, 260), (990, 360)):
        x, y = float(sx), float(sy)
        path = [(x, y)]
        while y > surf + 2:
            y -= float(rng.uniform(14, 26))
            x += float(rng.uniform(-16, 16))
            path.append((x, max(y, surf + 2)))
        vein = [(int(px), int(py)) for px, py in path]
        soft(img, lambda dd, p=vein: dd.line(p, fill=rgb(VIOLET, 170), width=12, joint="curve"), 5)
        d = ImageDraw.Draw(img, "RGBA")
        d.line(vein, fill=rgb("#030305"), width=5, joint="curve")
        for j in range(1, len(vein) - 1, 2):   # branchlets
            bx, by = vein[j]
            d.line([(bx, by), (bx + int(rng.integers(-34, 34)), by - int(rng.integers(6, 22)))], fill=rgb("#030305"), width=2)
        wells.append(vein[-1])
    # the crust: the ward's roofs and the wells the marrow bleeds into
    skyline(img, rng, surf, mix(VOID, BASALT, 0.5), 8, 30, 0.1, wmin=24, wmax=60, bottom=surf)
    cathedral(img, 620, surf, 0.2, mix(VOID, BASALT, 0.5), TALLOW, 30)
    d = ImageDraw.Draw(img, "RGBA")
    d.line([(0, surf), (ST_W, surf)], fill=rgb(VOID), width=3)
    for wx, _ in wells:
        d.rectangle([wx - 9, surf - 9, wx + 9, surf], fill=rgb(IRON), outline=rgb(VOID), width=2)
        d.rectangle([wx - 5, surf - 5, wx + 5, surf + 2], fill=rgb(mix(VOID, VIOLET, 0.6)))
        glow(img, wx, surf, 18, VIOLET, 120)
    return vignette(img, 0.5)


def st_intro_3():
    # crypt arches; iron masks over chained adult figures; tallow candles on the wet floor
    rng, img = st_canvas(ST_W, ST_H, mix(BASALT, VOID, 0.4), BASALT, "story_intro_3", 6)
    wall_courses(img, 6, 384, VOID, 120, 38, 76)
    d = ImageDraw.Draw(img, "RGBA")
    floor = 392
    d.rectangle([0, floor, ST_W, ST_H], fill=rgb(SLATE))
    for y in range(floor + 14, ST_H, 18):
        d.line([(0, y), (ST_W, y)], fill=rgb(VOID, 110), width=2)
    stone, stone_hi = mix(BASALT, IRON, 0.35), mix(BASALT, PSTEEL, 0.3)
    for cx in (215, 560, 905):
        x0, x1 = cx - 118, cx + 118
        outer = arch_pts(x0 - 22, x1 + 22, 200, floor, 0.8)
        d.polygon(outer, fill=rgb(stone), outline=rgb(VOID), width=4)
        d.polygon(arch_pts(x0, x1, 200, floor, 0.8), fill=rgb(mix(VOID, VIOLET, 0.08)), outline=rgb(VOID), width=4)
        glow(img, cx, floor - 70, 120, mix(VOID, VIOLET, 0.5), 90)
        d = ImageDraw.Draw(img, "RGBA")
        apex = 200 - 0.74 * (x1 - x0)
        d.polygon([(cx - 14, apex - 22), (cx + 14, apex - 22), (cx + 9, apex + 6), (cx - 9, apex + 6)], fill=rgb(stone_hi), outline=rgb(VOID), width=3)
        for rx in (x0 + 10, x1 - 10):   # ring bolts on the jambs
            d.ellipse([rx - 8, 288, rx + 8, 304], outline=rgb(VOID), width=6)
            d.ellipse([rx - 8, 288, rx + 8, 304], outline=rgb(IRON), width=3)
        cloth = mix(SLATE, CINDER, 0.3)
        hx, hy, u = person(d, cx, floor + 2, 200, cloth, VOID, arms_out=0.6)
        d.polygon([(cx - 10 * u, floor - 78 * u), (cx + 10 * u, floor - 78 * u), (cx + 4 * u, floor - 50 * u), (cx - 4 * u, floor - 50 * u)], fill=rgb(mix(cloth, LINEN, 0.15)))
        iron_mask(d, hx, hy, u)
        for sx in (-1, 1):
            hand = (cx + sx * 25 * u, floor + 2 - 44 * u)
            d.rectangle([hand[0] - 3 * u, hand[1] - 4 * u, hand[0] + 3 * u, hand[1] + 1 * u], fill=rgb(IRON), outline=rgb(VOID))
            chain(d, hand, ((x0 + 10) if sx < 0 else (x1 - 10), 300), 18)
    for kx in (40, 388, 732, 1080):   # candles between the arches
        d = ImageDraw.Draw(img, "RGBA")
        for j, (dx, hgt) in enumerate(((-12, 34), (2, 48), (14, 26))):
            d.rectangle([kx + dx - 4, floor - hgt, kx + dx + 4, floor + 4], fill=rgb(LINEN if j != 1 else mix(LINEN, CINDER, 0.3)), outline=rgb(VOID), width=2)
            d.polygon([(kx + dx - 3, floor - hgt - 2), (kx + dx, floor - hgt - 14), (kx + dx + 3, floor - hgt - 2)], fill=rgb(CANDLE))
        glow(img, kx, floor - 40, 70, TALLOW, 110)
        d = ImageDraw.Draw(img, "RGBA")
        for j in range(4):   # the flames' reflection on the wet floor
            d.line([(kx - 12 + j * 8, floor + 10), (kx - 12 + j * 8, floor + 26 + (j % 2) * 10)], fill=rgb(TALLOW, 70), width=3)
    soft(img, lambda dd: [dd.ellipse([x - 160, floor - 24, x + 160, floor + 30], fill=rgb(mix(VOID, VIOLET, 0.4), 90)) for x in range(0, ST_W + 160, 220)], 18)
    return vignette(img, 0.55)


def st_intro_4():
    # a Watch insignia, half sunk in bloody mud under rain; boot prints lead past it
    rng, img = st_canvas(ST_W, ST_H, mix(VOID, OAK, 0.5), mix(OAK, SLATE, 0.4), "story_intro_4", 6)
    n = noise(rng, ST_W, ST_H, 40) * 0.6 + noise(rng, ST_W, ST_H, 12) * 0.4
    arr = np.asarray(img).astype(np.float32)
    arr[..., :3] *= (0.8 + 0.4 * (np.floor(n * 4) / 4))[..., None]
    pud = noise(rng, ST_W, ST_H, 90)
    yy = np.linspace(0.6, 1.0, ST_H)[:, None]
    blood = (pud * yy > 0.6)
    arr[blood, :3] = np.array(rgb(mix(SLATE, CRIMSON, 0.5))[:3], dtype=np.float32) * (0.85 + 0.3 * n[blood, None])
    edge = blood & ~np.roll(blood, 3, axis=0)
    arr[edge, :3] = rgb(mix(CRIMSON, ARTERIAL, 0.3))[:3]
    img = to_img(arr[..., :3])
    d = ImageDraw.Draw(img, "RGBA")
    for i in range(9):   # boot prints, oblique
        bx, by = 120 + i * 120 + (i % 2) * 22, 430 - i * 22 + (i % 2) * 18
        d.ellipse([bx - 15, by - 8, bx + 15, by + 8], fill=rgb(mix(VOID, OAK, 0.4)))
        d.ellipse([bx + 12, by - 6, bx + 24, by + 6], fill=rgb(mix(VOID, OAK, 0.4)))
    # the insignia (a heater shield with the weeping eye), seen at a slant
    cx, cy, sq = 560, 236, 0.62

    def S(pts):
        return [(cx + px, cy + py * sq) for px, py in pts]

    d.polygon(S([(-170, -150), (170, -150), (160, 30), (90, 140), (0, 200), (-90, 140), (-160, 30)]), fill=rgb(VOID))
    d.polygon(S([(-160, -150), (160, -150), (150, 30), (84, 132), (0, 188), (-84, 132), (-150, 30)]), fill=rgb(IRON), outline=rgb(VOID), width=5)
    d.polygon(S([(-160, -150), (160, -150), (150, -112), (-150, -112)]), fill=rgb(mix(IRON, PSTEEL, 0.5)))
    d.polygon(S([(-130, -112), (130, -112), (120, 24), (66, 112), (0, 160), (-66, 112), (-120, 24)]), fill=rgb(mix(IRON, VOID, 0.35)), outline=rgb(VOID), width=3)
    d.polygon(S([(-100, -20), (-50, -64), (0, -78), (50, -64), (100, -20), (50, 22), (0, 34), (-50, 22)]), fill=rgb(mix(CANDLE, OAK, 0.45)), outline=rgb(VOID), width=4)
    d.ellipse([cx - 30, cy - 50 * sq, cx + 30, cy + 10 * sq], fill=rgb(mix(CRIMSON, VOID, 0.2)), outline=rgb(VOID), width=3)
    d.ellipse([cx - 11, cy - 30 * sq, cx + 11, cy - 6 * sq], fill=rgb(VOID))
    d.polygon(S([(-8, 40), (8, 40), (14, 92), (0, 112), (-14, 92)]), fill=rgb(CRIMSON), outline=rgb(VOID), width=3)
    for rx, ry in ((-140, -132), (140, -132), (-120, 10), (120, 10)):
        d.ellipse([cx + rx - 7, cy + ry * sq - 7, cx + rx + 7, cy + ry * sq + 7], fill=rgb(PSTEEL), outline=rgb(VOID))
    # the mud closes over its lower left; blood streaks across the plate
    mud = []
    for k in range(40):   # an irregular lip of mud over the lower left, its rim wet with blood
        a = k / 40 * 2 * math.pi
        rr = 1.0 + 0.18 * math.sin(a * 5 + 1.3) + 0.08 * math.sin(a * 11)
        mud.append((cx - 150 + 170 * rr * math.cos(a), cy + 120 + 70 * rr * math.sin(a)))
    d.polygon(mud, fill=rgb(mix(mix(OAK, SLATE, 0.4), CRIMSON, 0.18)), outline=rgb(VOID), width=3)
    d.line([p for p in mud if p[1] < cy + 120] + [mud[0]], fill=rgb(mix(CRIMSON, ARTERIAL, 0.25)), width=3)
    for _ in range(14):
        x, y = cx - 150 + int(rng.integers(-130, 130)), cy + 120 + int(rng.integers(-30, 40))
        d.ellipse([x - 9, y - 3, x + 9, y + 3], fill=rgb(mix(OAK, VOID, 0.4)))
    for sx, sy, ex, ey in ((cx + 40, cy - 70, cx + 130, cy - 20), (cx + 70, cy - 60, cx + 120, cy - 4), (cx - 150, cy - 80, cx - 90, cy - 70)):
        d.line([(sx, sy), (ex, ey)], fill=rgb(mix(CRIMSON, VOID, 0.2), 220), width=7)
    rain(img, rng, 420, PSTEEL, 55, 26)
    d = ImageDraw.Draw(img, "RGBA")
    for _ in range(40):   # ripples
        x, y = int(rng.integers(0, ST_W)), int(rng.integers(120, ST_H))
        r = int(rng.integers(5, 14))
        d.ellipse([x - r, y - r * 0.35, x + r, y + r * 0.35], outline=rgb(PSTEEL, 70), width=1)
    return vignette(img, 0.6)


def st_meat_market():
    # the Lower Meat Market rendering station: cauldrons over fire pits, chimneys, a hook gantry, the Spent in queues
    rng, img = st_canvas(ST_W, ST_H, VOID, mix(VOID, VIOLET, 0.18), "story_meat_market", 6)
    shroud(img, rng, 0, 120, mix(VOID, VIOLET, 0.3), 160, 90)
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([0, 150, ST_W, 372], fill=rgb(mix(BASALT, OAK, 0.4)))
    wall_courses(img, 160, 372, VOID, 90, 26, 60)
    d = ImageDraw.Draw(img, "RGBA")
    for ax in (110, 1010):   # dim hall arches
        d.polygon(arch_pts(ax - 60, ax + 60, 260, 372, 0.7), fill=rgb(mix(VOID, TALLOW, 0.12)), outline=rgb(VOID), width=3)
    for cx, top in ((250, 10), (870, 30)):   # chimneys and their smoke
        d.polygon([(cx - 34, 160), (cx - 24, top), (cx + 24, top), (cx + 34, 160)], fill=rgb(mix(BASALT, VOID, 0.3)), outline=rgb(VOID), width=3)
        d.rectangle([cx - 30, top - 8, cx + 30, top + 4], fill=rgb(mix(BASALT, IRON, 0.3)), outline=rgb(VOID), width=3)
        soft(img, lambda dd, c=cx, t=top: [dd.ellipse([c - 30 + k * 26, t - 40 - k * 30, c + 40 + k * 34, t + 10 - k * 30], fill=rgb(CINDER, 120 - k * 18)) for k in range(5)], 10)
        d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([60, 140, 1060, 154], fill=rgb(OAK), outline=rgb(VOID), width=3)   # the hook gantry
    for i, hx in enumerate(range(110, 1040, 64)):
        if 440 < hx < 680:
            continue
        drop = 30 + (i % 3) * 10
        d.line([(hx, 154), (hx, 154 + drop)], fill=rgb(IRON), width=3)
        d.arc([hx - 8, 150 + drop, hx + 8, 166 + drop], 0, 200, fill=rgb(PSTEEL), width=3)
        if i % 2 == 0:   # wrapped bundles
            d.ellipse([hx - 16, 162 + drop, hx + 16, 228 + drop], fill=rgb(mix(LINEN, CINDER, 0.55)), outline=rgb(VOID), width=3)
            for k in range(3):
                d.line([(hx - 14, 182 + drop + k * 14), (hx + 14, 178 + drop + k * 14)], fill=rgb(mix(LINEN, OAK, 0.6)), width=2)
    for cx in (430, 690):   # cauldrons over fire pits
        glow(img, cx, 340, 170, TALLOW, 150)
        d = ImageDraw.Draw(img, "RGBA")
        d.rectangle([cx - 110, 340, cx + 110, 382], fill=rgb(mix(OAK, CRIMSON, 0.25)), outline=rgb(VOID), width=4)
        for bx in range(cx - 100, cx + 100, 30):
            d.line([(bx, 340), (bx, 382)], fill=rgb(VOID, 140), width=2)
        d.rectangle([cx - 70, 356, cx + 70, 382], fill=rgb(TALLOW), outline=rgb(VOID), width=3)
        for fx in range(cx - 60, cx + 60, 24):
            d.polygon([(fx, 380), (fx + 12, 352 - (fx % 3) * 6), (fx + 24, 380)], fill=rgb(CANDLE))
        d.chord([cx - 120, 220, cx + 120, 380], 0, 180, fill=rgb(IRON), outline=rgb(VOID), width=5)
        d.chord([cx - 120, 220, cx + 120, 380], 20, 70, fill=rgb(mix(IRON, PSTEEL, 0.35)))
        d.ellipse([cx - 120, 280, cx + 120, 320], fill=rgb(mix(TALLOW, BLEATHER, 0.35)), outline=rgb(VOID), width=5)
        d.ellipse([cx - 90, 288, cx + 90, 312], fill=rgb(TALLOW))
        for bx, by in ((cx - 40, 298), (cx + 24, 294), (cx + 60, 302)):
            d.ellipse([bx - 7, by - 4, bx + 7, by + 4], outline=rgb(CANDLE), width=2)
        soft(img, lambda dd, c=cx: [dd.ellipse([c - 70 + k * 10, 220 - k * 40, c + 60 + k * 16, 290 - k * 40], fill=rgb(LINEN, 70 - k * 12)) for k in range(5)], 14)
        d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([520, 330, 600, 392], fill=rgb(OAK), outline=rgb(VOID), width=3)   # the ladling counter
    for bx in range(530, 596, 16):
        d.ellipse([bx - 6, 322, bx + 6, 330], fill=rgb(mix(OAK, LINEN, 0.3)), outline=rgb(VOID))
    d.rectangle([0, 382, ST_W, ST_H], fill=rgb(SLATE))
    for y in range(392, ST_H, 16):
        for x in range((y // 16 % 2) * 20, ST_W, 40):
            d.rectangle([x, y, x + 38, y + 14], outline=rgb(VOID, 120), width=2)
    rim = mix(VOID, TALLOW, 0.45)
    for (sx, sy, sh), (ex, ey, eh) in (((40, 470, 230), (470, 400, 96)), ((1090, 470, 230), (650, 400, 96))):
        steps = 6
        for k in range(steps, -1, -1):   # from the back of the queue to the front, larger toward the viewer
            t = k / steps
            x, y, h = ex + (sx - ex) * t, ey + (sy - ey) * t, eh + (sh - eh) * t
            person(d, x, y, h, mix(VOID, SLATE, 0.55), rim, bow=0.8 if sx < ex else -0.8, bowl=mix(OAK, LINEN, 0.25) if sx < ex else None)
    ash_fall(img, rng, 70, rmax=3.0)
    return vignette(img, 0.55)


def st_hearth():
    # the Sluice Undercroft: ribbed vaults receding, the Water-Lock Basin, Bram's forge glowing at the Tinker's Hearth
    rng, img = st_canvas(ST_W, ST_H, VOID, mix(VOID, BASALT, 0.6), "story_hearth", 4)
    vx, vy = 560, 300
    for k in range(5, -1, -1):   # vault rings from the back to the front
        s = min(1.25, 0.28 + 0.16 * (5 - k) ** 1.15)
        hw, base = 470 * s, vy + 175 * s
        warm = mix(mix(BASALT, IRON, 0.08 + 0.05 * (5 - k)), TALLOW, 0.06 * (5 - k))
        outer = arch_pts(vx - hw, vx + hw, vy - 40 * s, base, 0.62)
        inner = arch_pts(vx - hw + 46 * s, vx + hw - 46 * s, vy - 40 * s, base, 0.62)
        ring = Image.new("L", img.size, 0)
        md = ImageDraw.Draw(ring)
        md.polygon(outer, fill=255)
        md.polygon(inner, fill=0)
        img.paste(Image.new("RGBA", img.size, rgb(warm)), (0, 0), ring)
        d = ImageDraw.Draw(img, "RGBA")
        d.line(outer[1:-1], fill=rgb(VOID), width=4)
        d.line(inner[1:-1], fill=rgb(VOID), width=4)
        d.line(arch_pts(vx - hw + 6, vx + hw - 6, vy - 40 * s + 6, base, 0.62)[1:14], fill=rgb(mix(warm, PSTEEL, 0.15)), width=2)
    d = ImageDraw.Draw(img, "RGBA")
    d.polygon([(0, ST_H), (0, 410), (vx - 60, vy + 50), (vx + 60, vy + 50), (ST_W, 410), (ST_W, ST_H)], fill=rgb(SLATE))
    for t in np.linspace(0, 1, 7):
        d.line([(vx - 60 + 120 * t, vy + 50), (-200 + (ST_W + 400) * t, ST_H)], fill=rgb(VOID, 110), width=2)
    for y in (vy + 62, vy + 82, vy + 112, vy + 152):
        d.line([(0, y), (ST_W, y)], fill=rgb(VOID, 90), width=2)
    # the Water-Lock Basin, foreground left
    d.polygon([(30, 400), (420, 400), (470, 472), (0, 472)], fill=rgb(mix(BASALT, IRON, 0.35)), outline=rgb(VOID), width=4)
    d.polygon([(54, 412), (404, 412), (440, 472), (24, 472)], fill=rgb(mix(VOID, CYAN, 0.16)), outline=rgb(VOID), width=3)
    for i in range(6):
        y = 424 + i * 8
        d.line([(80 + i * 6, y), (240 + i * 14, y)], fill=rgb(mix(VOID, CYAN, 0.4), 160), width=2)
        d.line([(300, y + 2), (330 + i * 8, y + 2)], fill=rgb(TALLOW, 120), width=2)
    for x in (150, 300):
        d.ellipse([x - 20, 436, x + 20, 446], outline=rgb(mix(VOID, CYAN, 0.55)), width=2)
    # the forge, foreground right
    glow(img, 900, 360, 260, TALLOW, 150)
    d = ImageDraw.Draw(img, "RGBA")
    d.polygon([(820, 200), (980, 200), (1000, 120), (800, 120)], fill=rgb(mix(BASALT, IRON, 0.25)), outline=rgb(VOID), width=4)
    d.rectangle([860, 0, 940, 122], fill=rgb(mix(BASALT, IRON, 0.2)), outline=rgb(VOID), width=4)
    d.rectangle([790, 200, 1010, 420], fill=rgb(mix(OAK, CRIMSON, 0.3)), outline=rgb(VOID), width=5)
    for i, y in enumerate(range(212, 420, 18)):
        d.line([(792, y), (1008, y)], fill=rgb(VOID, 140), width=2)
        for x in range(796 + (i % 2) * 18, 1008, 36):
            d.line([(x, y), (x, y + 18)], fill=rgb(VOID, 140), width=2)
    d.polygon(arch_pts(840, 960, 300, 392, 0.55), fill=rgb(TALLOW), outline=rgb(VOID), width=5)
    d.polygon(arch_pts(860, 940, 320, 392, 0.55), fill=rgb(CANDLE))
    for cx in (880, 905, 925):
        d.polygon([(cx - 10, 392), (cx, 340 + (cx % 7) * 3), (cx + 10, 392)], fill=rgb(mix(CANDLE, LINEN, 0.5)))
    d.polygon([(700, 420), (720, 396), (800, 396), (812, 380), (840, 380), (820, 404), (790, 404), (780, 420), (790, 440), (708, 440)], fill=rgb(mix(VOID, IRON, 0.5)), outline=rgb(VOID), width=3)
    d.line([(722, 396), (800, 396)], fill=rgb(TALLOW), width=2)
    for _ in range(26):   # sparks
        x, y = int(rng.integers(820, 1000)), int(rng.integers(160, 340))
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=rgb(CANDLE, int(rng.integers(120, 255))))
    # the Apothecary's Slab, mid-depth left
    d.rectangle([330, 330, 450, 344], fill=rgb(mix(LINEN, CINDER, 0.6)), outline=rgb(VOID), width=3)
    for lx in (338, 438):
        d.rectangle([lx, 344, lx + 6, 384], fill=rgb(OAK), outline=rgb(VOID))
    for bx, bc in ((350, VIOLET), (372, CYAN), (392, CRIMSON), (414, TALLOW)):
        d.rectangle([bx, 312, bx + 10, 330], fill=rgb(mix(bc, VOID, 0.35)), outline=rgb(VOID), width=2)
    for _ in range(18):   # drips from the vault
        x, y = int(rng.integers(40, ST_W - 40)), int(rng.integers(20, 240))
        d.line([(x, y), (x, y + int(rng.integers(8, 20)))], fill=rgb(mix(VOID, CYAN, 0.5), 140), width=2)
    return vignette(img, 0.55)


def g_story(out: Out) -> None:
    scenes = {"intro_1": st_intro_1, "intro_2": st_intro_2, "intro_3": st_intro_3, "intro_4": st_intro_4,
              "meat_market": st_meat_market, "hearth": st_hearth}
    for sid, fn in scenes.items():
        save(fn(), "game", "backgrounds/story/%s.png" % sid, out)


def g_world_card(out: Out) -> None:
    # the sunk_ward world card (backgrounds/title/<world>.png, as the Ashlands card): the ward's skyline under the Shroud
    W, H = 1280, 360
    rng, img = st_canvas(W, H, VOID, mix(VOID, VIOLET, 0.25), "world_card_sunk_ward", 6)
    shroud_sky(img, rng, 290)
    skyline(img, rng, 300, mix(VOID, BASALT, 0.6), 20, 70, 0.08, wmin=40, wmax=100)
    cathedral(img, 1060, 312, 0.55, VOID, TALLOW, 40)
    skyline(img, rng, 318, VOID, 4, 30, 0.12, wmin=50, wmax=110)
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([0, 318, W, H], fill=rgb(mix(VOID, CYAN, 0.07)))
    for _ in range(40):   # lit windows on the canal
        x, y = int(rng.integers(0, W)), int(rng.integers(322, H))
        d.line([(x, y), (x + int(rng.integers(6, 24)), y)], fill=rgb(TALLOW, int(rng.integers(60, 150))), width=2)
    ash_fall(img, rng, 90, rmax=3.0)
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle([20, 20, W - 21, H - 21], outline=rgb(CANDLE, 200), width=3)
    d.rectangle([30, 30, W - 31, H - 31], outline=rgb(CANDLE, 90), width=1)
    try:
        font = ImageFont.load_default(size=110)
        small = ImageFont.load_default(size=34)
    except TypeError:
        font = small = ImageFont.load_default()
    text = "THE SUNK WARD"
    tw = d.textlength(text, font=font)
    d.text(((W - tw) / 2 + 4, 84 + 4), text, font=font, fill=rgb("#000000", 210))
    d.text(((W - tw) / 2, 84), text, font=font, fill=rgb(LINEN))
    sub = "The Young Vanguard"
    sw = d.textlength(sub, font=small)
    d.text(((W - sw) / 2 + 2, 232), sub, font=small, fill=rgb("#000000", 200))
    d.text(((W - sw) / 2, 230), sub, font=small, fill=rgb(CANDLE))
    save(img, "game", "backgrounds/title/sunk_ward.png", out)


def g_menu_sunless(out: Out) -> None:
    # the game's main menu (MENU2): the Shroud, a cathedral and the ward's roofs, falling ash
    W, H = 1920, 1080
    rng, img = st_canvas(W, H, VOID, mix(VOID, VIOLET, 0.25), "menu_sunless", 8)
    shroud_sky(img, rng, 760)
    skyline(img, rng, 780, mix(VOID, BASALT, 0.55), 60, 160, 0.06, wmin=80, wmax=200)
    cathedral(img, 520, 800, 1.35, "#0c0d13", TALLOW, 55)
    skyline(img, rng, 900, "#0b0c11", 40, 120, 0.08, wmin=100, wmax=220)
    skyline(img, rng, 1010, VOID, 20, 70, 0.05, wmin=120, wmax=260)
    ash_fall(img, rng, 300)
    save(vignette(img, 0.7), "game", "backgrounds/menu/main_menu.png", out)


# The Sunk Ward battle zones (grounds from HOOKS1_TASKS 2.3) in the STAGE1 7.3 format; briefs from CAST1_ASSETS N10
ZONE_GROUND.update({"alley": "stone", "rooftops": "stone", "canal": "bridge", "foundry": "floor", "tallow_vats": "floor",
                    "undercroft": "stone", "watchpost": "floor"})


def bd_base(bottom: str) -> Image.Image:
    # the sky gradient in four flat steps, so the 16-colour pass keeps its slots for the scenery's lights
    return quant16(bd_gradient(BD_TOP, rgb(bottom)[:3]), 4)


def bd_alley():
    # Cinder Alley: soot-black brick tenements, linen on lines across the lane, tallow lamps
    rng = np.random.default_rng(seed_of("bdalley"))
    img = bd_base(mix(VOID, VIOLET, 0.14))
    d = ImageDraw.Draw(img)
    x = -10
    while x < BD_W:
        w = int(rng.integers(36, 64))
        top = int(rng.integers(3, 14))
        col = mix(BASALT, OAK, (0.0, 0.25, 0.5)[int(rng.integers(0, 3))])
        d.polygon([(x, 47), (x, top + 6), (x + w // 2, top), (x + w, top + 6), (x + w, 47)], fill=rgb(col), outline=rgb(VOID))
        for wy in range(top + 10, 42, 9):
            for wx in range(x + 5, x + w - 6, 10):
                d.rectangle([wx, wy, wx + 3, wy + 4], fill=rgb(TALLOW) if rng.random() < 0.22 else rgb(VOID))
        x += w + int(rng.integers(4, 10))
    for lx0 in range(20, BD_W, 96):   # washing lines with linen, a few across the lane
        lx1 = lx0 + 60
        pts = [(x, 18 + 5 * math.sin((x - lx0) / (lx1 - lx0) * math.pi)) for x in range(lx0, lx1 + 1, 3)]
        d.line(pts, fill=rgb(CINDER))
        for k in range(lx0 + 6, lx1 - 4, 11):
            if rng.random() < 0.7:
                yy = int(18 + 5 * math.sin((k - lx0) / (lx1 - lx0) * math.pi))
                d.rectangle([k, yy, k + 4, yy + int(rng.integers(4, 8))], fill=rgb(mix(LINEN, CINDER, 0.35)), outline=rgb(VOID))
    for lx in (48, 176, 304):   # tallow lamps on brackets
        d.line([(lx, 30), (lx + 6, 30)], fill=rgb(IRON))
        d.rectangle([lx + 5, 30, lx + 8, 35], fill=rgb(CANDLE), outline=rgb(VOID))
    return img


def bd_rooftops():
    # Market Rooftops: wet slates, chimneys with smoke, meat hooks on a line
    rng = np.random.default_rng(seed_of("bdrooftops"))
    img = bd_base(mix(VOID, VIOLET, 0.16))
    d = ImageDraw.Draw(img)
    for base, col, hi in ((26, mix(BASALT, SLATE, 0.5), None), (40, mix(SLATE, IRON, 0.3), mix(SLATE, PSTEEL, 0.4))):
        x = -int(rng.integers(0, 30))
        while x < BD_W:
            w = int(rng.integers(30, 56))
            pk = base - int(rng.integers(6, 14))
            d.polygon([(x, 47), (x, base), (x + w // 2, pk), (x + w, base), (x + w, 47)], fill=rgb(col), outline=rgb(VOID))
            if hi:
                d.line([(x + 2, base - 1), (x + w // 2, pk + 1)], fill=rgb(hi))
                for sy in range(base + 3, 47, 3):
                    d.line([(x + 1, sy), (x + w - 1, sy)], fill=rgb(mix(col, VOID, 0.3)))
            if rng.random() < 0.55:
                cx = x + int(rng.integers(4, w - 8))
                ch = int(rng.integers(6, 12))
                d.rectangle([cx, pk - ch + 4, cx + 5, base], fill=rgb(mix(OAK, CRIMSON, 0.2)), outline=rgb(VOID))
                for k in range(3):   # a smoke wisp drifting right
                    sx, sy = cx + 2 + k * 3, pk - ch + 1 - k * 4
                    d.ellipse([sx - 2 - k, sy - 2, sx + 2 + k, sy + 1], fill=rgb(mix(CINDER, VOID, 0.45 + 0.1 * k)))
            x += w - 2
    d.line([(x, 12 + 6 * math.sin(x / BD_W * math.pi)) for x in range(0, BD_W, 4)], fill=rgb(IRON))
    for k in range(20, BD_W, 28):
        yy = int(12 + 6 * math.sin(k / BD_W * math.pi))
        d.line([(k, yy), (k, yy + 4)], fill=rgb(PSTEEL))
        d.arc([k - 2, yy + 3, k + 2, yy + 7], 0, 200, fill=rgb(PSTEEL))
    return img


def bd_canal():
    # Canal Bridge: an iron truss bridge over black water, the mortuary cart on the deck
    rng = np.random.default_rng(seed_of("bdcanal"))
    img = bd_base(mix(VOID, VIOLET, 0.12))
    d = ImageDraw.Draw(img)
    x = 0
    while x < BD_W:   # the far bank
        w = int(rng.integers(24, 48))
        top = int(rng.integers(8, 18))
        d.rectangle([x, top, x + w, 34], fill=rgb(mix(VOID, BASALT, 0.7)))
        if rng.random() < 0.5:
            d.rectangle([x + 4, top + 4, x + 6, top + 7], fill=rgb(TALLOW))
        x += w + 2
    d.rectangle([0, 34, BD_W, 47], fill=rgb(mix(VOID, CYAN, 0.06)))
    for _ in range(30):
        x, y = int(rng.integers(0, BD_W)), int(rng.integers(36, 47))
        d.line([(x, y), (x + int(rng.integers(3, 10)), y)], fill=rgb(mix(VOID, CYAN, 0.3)) if rng.random() < 0.7 else rgb(TALLOW))
    d.rectangle([0, 26, BD_W, 29], fill=rgb(IRON), outline=rgb(VOID))   # deck and truss
    d.line([(0, 16), (BD_W, 16)], fill=rgb(IRON), width=2)
    for k in range(0, BD_W, 16):
        d.line([(k, 17), (k + 16, 26)], fill=rgb(mix(IRON, VOID, 0.2)))
        d.line([(k, 17), (k, 26)], fill=rgb(mix(IRON, PSTEEL, 0.3)))
    for px in (64, 192, 320):
        d.rectangle([px - 4, 29, px + 4, 47], fill=rgb(mix(BASALT, IRON, 0.3)), outline=rgb(VOID))
        d.arc([px + 4, 28, px + 124, 52], 180, 360, fill=rgb(mix(IRON, VOID, 0.3)), width=2)
    d.rectangle([150, 14, 182, 25], fill=rgb(mix(VOID, OAK, 0.6)), outline=rgb(VOID))   # the mortuary cart
    for bx in range(153, 181, 4):
        d.line([(bx, 15), (bx, 24)], fill=rgb(IRON))
    d.polygon([(148, 14), (166, 9), (184, 14)], fill=rgb(mix(VOID, OAK, 0.6)), outline=rgb(VOID))
    for wx in (156, 176):
        d.ellipse([wx - 3, 22, wx + 3, 28], fill=rgb(OAK), outline=rgb(VOID))
    d.line([(184, 20), (194, 24)], fill=rgb(OAK))
    return img


def bd_foundry():
    # The Fourth Kiln: kiln stacks, the bone-mill wheel, chains, vent grates and slag pools
    img = bd_base(mix(BASALT, OAK, 0.4))
    d = ImageDraw.Draw(img)
    d.rectangle([0, 14, BD_W, 47], fill=rgb(mix(BASALT, OAK, 0.25)))   # the kiln hall wall
    for i, y in enumerate(range(14, 47, 6)):
        d.line([(0, y), (BD_W, y)], fill=rgb(VOID))
        for x in range((i % 2) * 6, BD_W, 12):
            d.line([(x, y), (x, y + 6)], fill=rgb(VOID))
    d.rectangle([0, 10, BD_W, 13], fill=rgb(IRON), outline=rgb(VOID))   # a gantry beam
    for kx in (40, 120):
        d.polygon([(kx - 16, 47), (kx - 9, 2), (kx + 9, 2), (kx + 16, 47)], fill=rgb(mix(BASALT, OAK, 0.3)), outline=rgb(VOID))
        d.polygon(arch_pts(kx - 8, kx + 8, 36, 47, 0.6, 8), fill=rgb(TALLOW), outline=rgb(VOID))
    cx, cy, r = 280, 24, 19   # the bone-mill wheel
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=rgb(VOID), width=5)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=rgb(LINEN), width=3)
    for k in range(8):
        a = k * math.pi / 4
        d.line([(cx, cy), (cx + r * math.cos(a), cy + r * math.sin(a))], fill=rgb(LINEN), width=2)
        d.ellipse([cx + r * math.cos(a) - 2, cy + r * math.sin(a) - 2, cx + r * math.cos(a) + 2, cy + r * math.sin(a) + 2], fill=rgb(LINEN), outline=rgb(VOID))
    d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=rgb(IRON), outline=rgb(VOID))
    for chx in (180, 196, 344, 360):   # chains
        for y in range(1, 30 + (chx % 3) * 4, 3):
            d.point((chx, y), fill=rgb(IRON))
            d.point((chx + (1 if y % 6 else -1), y + 1), fill=rgb(PSTEEL))
    for gx in (200, 344):   # vent grates
        d.rectangle([gx - 10, 34, gx + 10, 44], fill=rgb(ARTERIAL), outline=rgb(VOID))
        for bx in range(gx - 8, gx + 10, 4):
            d.line([(bx, 34), (bx, 44)], fill=rgb(VOID))
    for px in (80, 240):   # slag pools
        d.ellipse([px - 26, 42, px + 26, 49], fill=rgb(TALLOW), outline=rgb(VOID))
        d.ellipse([px - 14, 44, px + 12, 47], fill=rgb(CANDLE))
    return img


def bd_tallow_vats():
    # Tallow Vats: boiling vats with steam, a catwalk with rails, grease runnels
    rng = np.random.default_rng(seed_of("bdtallow"))
    img = bd_base(mix(BASALT, OAK, 0.5))
    d = ImageDraw.Draw(img)
    d.rectangle([0, 18, BD_W, 20], fill=rgb(OAK), outline=rgb(VOID))   # the catwalk
    d.line([(0, 12), (BD_W, 12)], fill=rgb(IRON))
    for k in range(0, BD_W, 12):
        d.line([(k, 12), (k, 18)], fill=rgb(IRON))
    for vx in (48, 144, 240, 336):
        w = 30
        for k in range(3):   # steam wisps rising past the catwalk
            sx, sy = vx - 4 + (k % 2) * 6, 14 - k * 5
            d.ellipse([sx - 3, sy - 2, sx + 3, sy + 2], fill=rgb(mix(LINEN, VOID, 0.5 + 0.12 * k)))
        d.rectangle([vx - w // 2, 26, vx + w // 2, 47], fill=rgb(IRON), outline=rgb(VOID))
        d.line([(vx - w // 2 + 2, 27), (vx - w // 2 + 2, 46)], fill=rgb(mix(IRON, PSTEEL, 0.4)))
        for by in (32, 40):
            d.line([(vx - w // 2, by), (vx + w // 2, by)], fill=rgb(mix(IRON, VOID, 0.4)))
        d.ellipse([vx - w // 2, 22, vx + w // 2, 30], fill=rgb(TALLOW), outline=rgb(VOID))
        d.ellipse([vx - 8, 24, vx + 6, 28], fill=rgb(CANDLE))
        for gx in (vx - 8, vx + 6):   # runnels down the vat sides
            d.line([(gx, 30), (gx + 1, 30 + int(rng.integers(6, 16)))], fill=rgb(BLEATHER), width=2)
    return img


def bd_undercroft():
    # Sluice Undercroft: drowned vault arches on bone pillars, violet mist over the water
    img = bd_base(mix(VOID, VIOLET, 0.12))
    d = ImageDraw.Draw(img)
    d.rectangle([0, 2, BD_W, 47], fill=rgb(mix(BASALT, VOID, 0.2)))
    for k in range(-1, 7):
        x0 = k * 64 + 8
        d.polygon(arch_pts(x0, x0 + 48, 22, 48, 0.75, 10), fill=rgb(mix(VOID, VIOLET, 0.1)), outline=rgb(VOID))
    for k in range(0, 7):
        px = k * 64 + 4
        d.rectangle([px - 4, 4, px + 4, 47], fill=rgb(mix(LINEN, CINDER, 0.5)), outline=rgb(VOID))
        for jy in (10, 20, 30, 40):
            d.ellipse([px - 5, jy - 2, px + 5, jy + 2], fill=rgb(mix(LINEN, CINDER, 0.3)), outline=rgb(VOID))
    d.rectangle([0, 38, BD_W, 47], fill=rgb(mix(VOID, CYAN, 0.08)))
    for x in range(0, BD_W, 9):
        y = 41 + (x // 9) % 3 * 2
        d.line([(x, y), (x + 5, y)], fill=rgb(mix(VOID, CYAN, 0.3)))
    mist = Image.new("RGBA", (BD_W, BD_H), (0, 0, 0, 0))
    md = ImageDraw.Draw(mist)
    for y in range(BD_H - 14, BD_H):
        md.line([(0, y), (BD_W, y)], fill=rgb(ORCHID, int(30 + 90 * (y - (BD_H - 14)) / 14)))
    img.alpha_composite(mist)
    return img


def bd_watchpost():
    # Crucible Watchpost: the Slag Gate between two towers, the winch, braziers
    img = bd_base(mix(BASALT, OAK, 0.3))
    d = ImageDraw.Draw(img)
    wall, low = mix(BASALT, IRON, 0.3), mix(BASALT, IRON, 0.15)
    d.rectangle([0, 22, BD_W, 47], fill=rgb(low), outline=rgb(VOID))
    for x in range(0, BD_W, 8):
        d.rectangle([x, 18, x + 4, 22], fill=rgb(low), outline=rgb(VOID))
    for tx in (148, 220):   # the gate towers
        d.rectangle([tx - 14, 4, tx + 14, 47], fill=rgb(wall), outline=rgb(VOID))
        for x in range(tx - 14, tx + 14, 7):
            d.rectangle([x, 1, x + 4, 4], fill=rgb(wall), outline=rgb(VOID))
        d.rectangle([tx - 2, 12, tx + 2, 18], fill=rgb(TALLOW))
    d.rectangle([162, 14, 206, 47], fill=rgb(wall), outline=rgb(VOID))
    d.polygon(arch_pts(168, 200, 30, 47, 0.6, 8), fill=rgb(mix(VOID, TALLOW, 0.2)), outline=rgb(VOID))
    for x in range(170, 200, 4):   # the portcullis
        d.line([(x, 22), (x, 44)], fill=rgb(IRON))
    for y in range(26, 46, 5):
        d.line([(169, y), (199, y)], fill=rgb(IRON))
    cx, cy, r = 290, 30, 10   # the winch
    d.rectangle([cx - 14, 38, cx + 14, 47], fill=rgb(OAK), outline=rgb(VOID))
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=rgb(VOID), width=4)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=rgb(BLEATHER), width=2)
    for k in range(6):
        a = k * math.pi / 3
        d.line([(cx, cy), (cx + r * math.cos(a), cy + r * math.sin(a))], fill=rgb(BLEATHER))
    d.line([(cx - r, cy), (206, 18)], fill=rgb(IRON))
    for bx in (60, 110, 258, 340):   # braziers
        d.line([(bx - 4, 47), (bx, 38)], fill=rgb(IRON))
        d.line([(bx + 4, 47), (bx, 38)], fill=rgb(IRON))
        d.chord([bx - 6, 32, bx + 6, 42], 0, 180, fill=rgb(IRON), outline=rgb(VOID))
        d.polygon([(bx - 5, 37), (bx - 2, 28), (bx, 32), (bx + 2, 26), (bx + 5, 37)], fill=rgb(TALLOW), outline=rgb(VOID))
        d.polygon([(bx - 2, 37), (bx, 31), (bx + 2, 37)], fill=rgb(CANDLE))
    return img


def g_sunk_backdrops(out: Out) -> None:
    zones = {"alley": bd_alley(), "rooftops": bd_rooftops(), "canal": bd_canal(), "foundry": bd_foundry(),
             "tallow_vats": bd_tallow_vats(), "undercroft": bd_undercroft(), "watchpost": bd_watchpost()}
    for zone, img in zones.items():
        save(quant16(bd_finish(img, zone), keep_top=BD_TOP).convert("RGB"), "game", "backgrounds/battle/%s.png" % zone, out)


def it_scourge(c):
    # a three-tailed flail: barbed wire tails, the handle wrapped in vestment cloth
    c.line([(18, 94), (44, 56)], LINEN, w=10)
    for k in range(4):   # vestment cloth wound in crimson bands
        t = 0.15 + k * 0.22
        px, py = 18 + 26 * t, 94 - 38 * t
        c.line([(px - 5, py - 1), (px + 3, py + 6)], CRIMSON, w=3.0, ol=None)
    c.ell(46, 53, 6, 6, IRON, w=1.8)
    for ex, ey in ((66, 8), (86, 22), (92, 48)):   # three tails of barbed wire, zig-zag wound
        n = 7
        pts = []
        for i in range(n + 1):
            t = i / n
            x, y = 46 + (ex - 46) * t, 53 + (ey - 53) * t
            nx, ny = -(ey - 53), (ex - 46)
            ln = math.hypot(nx, ny)
            off = (3.0 if i % 2 else -3.0) * (0 if i in (0, n) else 1)
            pts.append((x + nx / ln * off, y + ny / ln * off))
        c.line(pts, mix(IRON, PSTEEL, 0.5), w=3.6)
        for x, y in pts[2:-1:2]:
            c.poly([(x - 3, y - 3), (x + 4, y - 1), (x - 1, y + 4)], PSTEEL, w=1.2)
        c.poly([(ex - 5, ey - 2), (ex + 3, ey - 6), (ex + 5, ey + 3), (ex - 2, ey + 5)], PSTEEL, w=1.4)


def it_sprayer(c):
    # a brass pressure tank, a hose and a pole nozzle
    brass, brass_hi = mix(CANDLE, BLEATHER, 0.45), mix(CANDLE, LINEN, 0.3)
    c.rect(14, 34, 46, 90, brass, w=2.6, r=10)
    c.rect(18, 40, 26, 84, brass_hi, ol=None, r=3)
    for y in (48, 76):
        c.line([(14, y), (46, y)], BLEATHER, w=2.4, ol=None)
    c.ell(30, 30, 9, 6, IRON, w=2)
    c.ell(30, 22, 6, 6, PSTEEL, w=1.6)
    c.line([(23, 21), (37, 21)], ARTERIAL, w=1.6, ol=None)
    c.line([(40, 30), (54, 18), (64, 30), (62, 56)], mix(OAK, LINEN, 0.25), w=4.4)
    c.line([(54, 74), (88, 16)], PSTEEL, w=5.5)
    c.poly([(84, 12), (94, 6), (96, 14), (90, 22)], brass, w=1.6)
    c.ell(57, 68, 5, 5, BLEATHER, w=1.4)


# The scourge and sprayer family icons (CAST1_ASSETS N9) over the stand-ins CAST1-A7 copied from kit icons: a drawn
# icon replaces a file only while it is still that byte copy, never other art
FAMILY_ICONS = {"scourge": (it_scourge, "items/mace/base.png"), "sprayer": (it_sprayer, "items/staff/base.png")}


def grid_icon(draw_fn, grid=48, size=96) -> Image.Image:
    # an item icon in the committed ART2 icon look: a 48-cell grid shown at 2x, a stepped teal field, a dark rim and
    # bracketed corners (sampled from kits/rpg/assets/items/*/base.png); the art drawn by draw_fn on the Cv space
    yy, xx = np.mgrid[0:grid, 0:grid]
    dist = np.sqrt(((xx - grid / 2 + 0.5) / (grid / 2)) ** 2 + ((yy - grid / 2 + 0.5) / (grid / 2)) ** 2)
    lo, hi = np.array(rgb("#273b3c")[:3], np.float32), np.array(rgb("#324a4c")[:3], np.float32)
    bg = to_img(lo + (hi - lo) * (np.floor(np.clip(1 - dist, 0, 1) * 4) / 4)[..., None])
    d = ImageDraw.Draw(bg)
    d.rectangle([0, 0, grid - 1, grid - 1], outline=rgb("#1d282a"))
    for fx, fy in ((0, 0), (1, 0), (0, 1), (1, 1)):
        def at(x, y):
            return (grid - 1 - x if fx else x, grid - 1 - y if fy else y)
        for x, y in ((0, 0), (1, 0), (0, 1)):
            d.point(at(x, y), fill=rgb("#d8d8d6"))
        for x, y in ((2, 0), (3, 0), (4, 0), (1, 1), (2, 1), (0, 2), (1, 2), (0, 3), (0, 4)):
            d.point(at(x, y), fill=rgb("#000203"))
    c = Cv(96)
    draw_fn(c)
    inner = grid - 8
    art = c.done().resize((inner, inner), Image.BOX)
    a = np.asarray(art).copy()
    a[..., 3] = np.where(a[..., 3] >= 110, 255, 0)
    bg.alpha_composite(Image.fromarray(a, "RGBA"), (4, 4))
    return bg.resize((size, size), Image.NEAREST)


def g_family_icons(out: Out) -> None:
    for fam, (fn, copied_from) in FAMILY_ICONS.items():
        rel = "items/%s/base.png" % fam
        path = os.path.join(layer_root(out.root, "game"), rel)
        src = os.path.join(layer_root(out.root, "kit"), copied_from)
        stand_in = os.path.exists(path) and os.path.exists(src) and same_bytes(path, src)
        save(grid_icon(fn), "game", rel, Out(out.root, out.layers, True) if stand_in else out)


# ------------------------------------------------------------------------------
# UI frames (LOOK1-G1, spec games/emberwake/docs/specs/LOOK1_TASKS.md G1): the Tallow and Iron nine-slice frames,
# ui/frames/<role>.png + <role>.json, exact pixels at scale 2 (2 art px per logical px) in the art bible's hues plus
# the spec's two inset darks (#050608, #0e1016). Rings count from the outside in art px.
# ------------------------------------------------------------------------------

INSET_DARK, INSET_FILL = "#050608", "#0e1016"
# The ornate frame's kit (LOOK1-A8, the user's pick, on magenta), or --ornate SHEET. The old A6 sheet stays in art_src.
ORNATE_SHEET = os.path.join(WORKSPACE, "art_src", "frame_kit_ref_gen1793628366_2.png")
# Art px are the kit's own px, never resampled here, so the engine's reduction is the only one: ORNATE_SCALE 8 per logical
# px is 4 per physical px at 720p, the arm's section about 8 px there. ORNATE_STUB: art px of arm kept past the corner point,
# the cut close to the ornament (20 px at 720p). ORNATE_LAP: the tip's head laps the stub by 1 px at 720p. ORNATE_ERODE:
# key_magenta's erosion in source px; the kit's ornament is 2-3 px thin, so the default 3 breaks it and 1 takes the fringe.
# ORNATE_MID: edge and centre px between the corners. ORNATE_RIM: rings of the tip's slanted faces, in art px: the spikes'
# dark outline, then their bright gold (decision "Ornate frame kit": one light on tip and spikes).
ORNATE_SCALE, ORNATE_STUB, ORNATE_LAP, ORNATE_ERODE, ORNATE_MID, ORNATE_RIM = 8, 80, 4, 1, 32, (1, 2)
# The kit's layout: the corner in the sheet's top-left quarter, the tip alone in this share of it (left, top, right, bottom);
# the spikes the tip takes its light from: the outermost art px of the corner along each axis, which hold nothing else.
ORNATE_TIP_AREA, ORNATE_SPIKE = (0.8, 0.4, 1.0, 0.6), 30
ORNATE_SIDECAR = {"scale": ORNATE_SCALE, "pad": [2, 2, 2, 2], "reduce": "average", "straddle": True, "sharpen": 0.6}
# `ornate_full`, the kit's edge as a whole frame (user 2026-11-02): the same corner without tips, its arm kept ORNATE_FULL_STUB
# past the corner point, whole filigree included, fading over its last ORNATE_FULL_FADE px into the kit's plain edge. That edge,
# ORNATE_FULL_SEG px of its middle made to repeat, tiles along every side at the corner's own art scale. The fade keeps the
# edge's section at the join; an even count of logical px in the slice keeps edge and corner on one pixel grid at whole views.
ORNATE_FULL_STUB, ORNATE_FULL_FADE, ORNATE_FULL_SEG = 128, 16, 64


def luma(px):
    return px[..., :3].astype(float) @ np.array([0.3, 0.59, 0.11])


def ornate_lit_tip(head: Image.Image, corner: Image.Image) -> Image.Image:
    # the head keeps its shape and takes the light of the corner's spikes: each pixel the spike colour of its own brightness
    # rank, then on the slanted faces (not the back, which meets the stub) the spikes' dark outline and bright gold
    spikes = np.concatenate([np.array(corner)[:ORNATE_SPIKE].reshape(-1, 4), np.array(corner)[:, :ORNATE_SPIKE].reshape(-1, 4)])
    spikes = spikes[spikes[..., 3] > 128]
    spikes = spikes[np.argsort(luma(spikes), kind="stable")][..., :3]
    a = np.array(head)
    shown = a[..., 3] > 0
    ranks = luma(a[shown]).argsort(kind="stable").argsort(kind="stable") / max(shown.sum() - 1, 1)
    a[..., :3][shown] = spikes[(ranks * (len(spikes) - 1)).astype(int)]
    tenth = max(len(spikes) // 10, 1)
    dark, bright = spikes[:tenth].mean(axis=0).astype(np.uint8), spikes[-tenth:].mean(axis=0).astype(np.uint8)
    # past the back the shape runs on, so the back is no edge
    outline, glow = ORNATE_RIM
    back = sum(ORNATE_RIM) + 1
    inside = np.pad(a[..., 3] > 128, ((0, 0), (back, 0)), mode="edge")
    for ring in range(1, outline + glow + 1):
        smaller = np.array(Image.fromarray(inside.astype(np.uint8) * 255, "L").filter(ImageFilter.MinFilter(3))) > 0
        a[(inside & ~smaller)[:, back:], :3] = dark if ring <= outline else bright
        inside = smaller
    return Image.fromarray(a, "RGBA")


def ornate_inner(q: Image.Image, x: int, y: int):
    # the tightest rect corner past the art (`corner_inner`): the column from which on the art stays above some row, the
    # one where the two insets from the anchor are most even, so screens' content starts clear of the corner
    drawn = np.array(q)[..., 3] > 0

    def inset(col):
        below = np.nonzero(drawn[:, col:].any(axis=1))[0]
        return [col - x, int(below.max()) + 1 - y if len(below) else 0]
    return min((inset(col) for col in range(x, q.width)), key=max)


def ornate_corner_piece(src: Image.Image):
    # the kit's top-left corner, and the middle of its arms' whole section across each axis
    w, h = src.size
    corner = largest_piece(src.crop((0, 0, w // 2, h // 2)))
    corner = corner.crop(corner.getbbox())
    opaque = np.array(corner)[..., 3] > 0
    across = opaque[:, int(corner.width * 0.92)].nonzero()[0]
    along = opaque[int(corner.height * 0.92), :].nonzero()[0]
    return corner, (int(along[0]) + int(along[-1]) + 1) // 2, (int(across[0]) + int(across[-1]) + 1) // 2


def ornate_kit(sheet):
    # the kit's top-left corner, cut ORNATE_STUB past the corner point, with only the head of the kit's tip on each stub,
    # pointing along its edge. Returns the corner on a transparent square, the art px on the panel's corner point (the
    # arms' centres, moved to a whole logical px so the reduction keeps its grid), and the corner's reach into the panel.
    src = key_magenta(Image.open(sheet), ORNATE_ERODE)
    w, h = src.size
    corner, ax, ay = ornate_corner_piece(src)
    area = ORNATE_TIP_AREA
    tip = largest_piece(src.crop((int(w * area[0]), int(h * area[1]), int(w * area[2]), int(h * area[3]))))
    tip = tip.crop(tip.getbbox())
    # the head: from the tip's widest column in its front half on
    shown = np.array(tip)[..., 3] > 0
    half = tip.width // 2
    head = tip.crop((half + int(np.argmax(shown.sum(axis=0)[half:])), 0, tip.width, tip.height))
    head = ornate_lit_tip(head.crop(head.getbbox()), corner)
    # whole logical px for the anchor, an odd count of them for the slice: the line then lands on whole px at whole views
    grid = ORNATE_SCALE
    x, y = (ax + grid - 1) // grid * grid, (ay + grid - 1) // grid * grid
    side = max(x, y) + ORNATE_STUB - ORNATE_LAP + head.width
    side = ((side + grid - 1) // grid | 1) * grid
    q = Image.new("RGBA", (side, side))
    q.alpha_composite(corner.crop((0, 0, ax + ORNATE_STUB, ay + ORNATE_STUB)), (x - ax, y - ay))
    q.alpha_composite(head, (x + ORNATE_STUB - ORNATE_LAP, y - head.height // 2))
    q.alpha_composite(head.transpose(Image.ROTATE_270), (x - head.height // 2, y + ORNATE_STUB - ORNATE_LAP))
    return q, [x, y], ornate_inner(q, x, y)


def ornate_rail(src: Image.Image) -> np.ndarray:
    # the kit's plain edge, its section's rows by ORNATE_FULL_SEG columns from its middle; the last columns fade into the
    # first, so the piece repeats without a seam
    w, h = src.size
    edge = largest_piece(src.crop((0, int(h * ORNATE_TIP_AREA[1]), int(w * ORNATE_TIP_AREA[0]), int(h * ORNATE_TIP_AREA[3]))))
    edge = np.array(edge.crop(edge.getbbox()))
    mid, fade, seg = edge.shape[1] // 2, ORNATE_FULL_FADE, ORNATE_FULL_SEG
    rows = np.nonzero(edge[:, mid, 3] > 128)[0]
    piece = edge[rows[0]:rows[-1] + 1, mid - (seg + fade) // 2:mid - (seg + fade) // 2 + seg + fade].astype(float)
    t = ((np.arange(fade) + 0.5) / fade)[None, :, None]
    out = piece[:, :seg].copy()
    out[:, :fade] = piece[:, seg:] * (1 - t) + piece[:, :fade] * t
    return np.round(out).astype(np.uint8)


def ornate_spine(section: np.ndarray, rail: np.ndarray) -> int:
    # where along `section` (one brightness per art px, across an arm) the rail's profile fits best: the px on its middle
    want = luma(rail).mean(axis=1)
    n = len(want)
    return n // 2 + min(range(len(section) - n + 1), key=lambda o: float(np.abs(section[o:o + n] - want).sum()))


def ornate_full_kit(sheet):
    # the kit's corner without tips, its arms running into the plain edge: the corner on a transparent square, the art px on
    # the panel's corner point (the edge's spine, so that the edge sits centred on the boundary), the corner's reach into the
    # panel and the repeating edge
    src = key_magenta(Image.open(sheet), ORNATE_ERODE)
    corner, ax, ay = ornate_corner_piece(src)
    rail = ornate_rail(src)
    stub, fade, seg = ORNATE_FULL_STUB, ORNATE_FULL_FADE, ORNATE_FULL_SEG
    bright = luma(np.array(corner)) * (np.array(corner)[..., 3] > 128)
    ay = ornate_spine(bright[:, ax + stub:ax + stub + 32].mean(axis=1), rail)
    ax = ornate_spine(bright[ay + stub:ay + stub + 32, :].mean(axis=0), rail)
    # whole counts of two logical px for the anchor and the slice: corner and edge share one pixel grid at views of 1.5 too
    grid = 2 * ORNATE_SCALE
    x, y = (ax + grid - 1) // grid * grid, (ay + grid - 1) // grid * grid
    side = (max(x, y) + stub + fade + grid - 1) // grid * grid
    q = Image.new("RGBA", (side, side))
    q.alpha_composite(corner.crop((0, 0, ax + stub, ay + stub)), (x - ax, y - ay))
    a = np.array(q).astype(float)
    half, reach = rail.shape[0] // 2, rail.shape[0]
    for cross, start, at in ((y, x, lambda c, l: (c, l)), (x, y, lambda c, l: (l, c))):
        for along in range(start + stub - fade, side):
            t = min((along - (start + stub - fade) + 0.5) / fade, 1.0)
            for off in range(-reach, reach):
                i = off + half
                # the piece's phase ends on its last column at the corner's end, where the engine starts the band's first
                over = rail[i, (along - side) % seg].astype(float) if 0 <= i < rail.shape[0] else np.zeros(4)
                under = a[at(cross + off, along)]
                mix = over.copy() if under[3] == 0 else under.copy() if over[3] == 0 else under + (over - under) * t
                mix[3] = under[3] + (over[3] - under[3]) * t
                a[at(cross + off, along)] = mix
    q = Image.fromarray(np.round(a).astype(np.uint8), "RGBA")
    return q, [x, y], ornate_inner(q, x, y), rail


def frame_line() -> str:
    # the one shade of every frame's thin accent line: the theme role `frame_line`, so frames and theme cannot part
    with open(os.path.join(WORKSPACE, "games", GAME, "assets", "ui", "theme.json"), encoding="utf-8") as f:
        return json.load(f)["colors"]["frame_line"]


def fr_ring(w: int, h: int, x: int, y: int) -> int:
    return min(x, y, w - 1 - x, h - 1 - y)


def fr_box(w: int, h: int, rings, centre: str, centre_a: int = 255) -> Image.Image:
    # rings: one colour per ring from the outside in; everything inside them is the centre
    img = Image.new("RGBA", (w, h))
    px = img.load()
    for y in range(h):
        for x in range(w):
            r = fr_ring(w, h, x, y)
            px[x, y] = rgb(rings[r]) if r < len(rings) else rgb(centre, centre_a)
    return img


def fr_rivets(img: Image.Image) -> None:
    # a 4 x 4 Cinder Ash rivet with a 2 x 2 Pewter top-left, 2 px in from each corner
    w, h = img.size
    for ox, oy in ((2, 2), (w - 6, 2), (2, h - 6), (w - 6, h - 6)):
        for y in range(4):
            for x in range(4):
                img.putpixel((ox + x, oy + y), rgb(PSTEEL if x < 2 and y < 2 else CINDER))


def fr_lit(img: Image.Image) -> None:
    # ring 2's top row in Wet Slate: a lit edge
    w, _ = img.size
    for x in range(2, w - 2):
        img.putpixel((x, 2), rgb(SLATE))


def fr_inset() -> Image.Image:
    # a sunken well: ring 0-1 dark on the top and left, Wet Slate on the bottom and right (split on the diagonal)
    w = h = 16
    img = Image.new("RGBA", (w, h))
    for y in range(h):
        for x in range(w):
            if fr_ring(w, h, x, y) >= 2:
                img.putpixel((x, y), rgb(INSET_FILL, 0xe0))
            elif x < 2 and y < h - 2 or y < 2 and x < w - 2 or (x < 2 or y < 2) and x + y < w - 1:
                img.putpixel((x, y), rgb(INSET_DARK))
            else:
                img.putpixel((x, y), rgb(SLATE))
    return img


def key_magenta(img: Image.Image, erode: int = 3) -> Image.Image:
    # a pixel is magenta when R-G and B-G both exceed 35 (the model's soft edges, not only the flat ground); the alpha
    # is then eroded by `erode` px at source size, which takes the pink fringe the soft edges leave. It also erases
    # #9333ea and #e11d48, so it is only for sheets whose prompt bans them.
    a = np.array(img.convert("RGBA"))
    r, g, b = a[..., 0].astype(int), a[..., 1].astype(int), a[..., 2].astype(int)
    a[..., 3] = np.where((r - g > 35) & (b - g > 35), 0, a[..., 3])
    alpha = Image.fromarray(a[..., 3], "L").filter(ImageFilter.MinFilter(2 * erode + 1))
    a[..., 3] = np.array(alpha)
    a[a[..., 3] == 0] = 0
    return Image.fromarray(a, "RGBA")


def largest_piece(img: Image.Image) -> Image.Image:
    # only the largest 8-connected opaque piece of `img`
    a = np.array(img)
    h, w = a.shape[:2]
    seen = a[..., 3] == 0
    best = []
    for sy, sx in zip(*np.nonzero(~seen)):
        if seen[sy, sx]:
            continue
        seen[sy, sx] = True
        piece, stack = [], [(int(sy), int(sx))]
        while stack:
            y, x = stack.pop()
            piece.append((y, x))
            for ny in range(max(y - 1, 0), min(y + 2, h)):
                for nx in range(max(x - 1, 0), min(x + 2, w)):
                    if not seen[ny, nx]:
                        seen[ny, nx] = True
                        stack.append((ny, nx))
        if len(piece) > len(best):
            best = piece
    out = np.zeros_like(a)
    for y, x in best:
        out[y, x] = a[y, x]
    return Image.fromarray(out, "RGBA")


def fr_corners(px: np.ndarray, q: Image.Image) -> Image.Image:
    # the corner `q` over `px`, mirrored for the other three
    n = px.shape[0]
    img = Image.fromarray(px, "RGBA")
    far = n - q.width
    img.paste(q, (0, 0))
    img.paste(q.transpose(Image.FLIP_LEFT_RIGHT), (far, 0))
    img.paste(q.transpose(Image.FLIP_TOP_BOTTOM), (0, far))
    img.paste(q.transpose(Image.ROTATE_180), (far, far))
    return img


def fr_ornate(q: Image.Image) -> Image.Image:
    # see-through but for the centre (Void Abyss at 94 %), the line and the corners. The line is 1 logical px wide, centred in
    # its slice, which the engine centres on the panel's boundary; nothing else on the edge.
    c, n = q.width, 2 * q.width + ORNATE_MID
    px = np.zeros((n, n, 4), dtype=np.uint8)
    px[c:n - c, c:n - c] = rgb(VOID, 0xf0)
    lo, hi, col = c // 2 - ORNATE_SCALE // 2, c // 2 + ORNATE_SCALE // 2, rgb(frame_line())
    px[lo:hi, c:n - c] = col
    px[n - hi:n - lo, c:n - c] = col
    px[c:n - c, lo:hi] = col
    px[c:n - c, n - hi:n - lo] = col
    return fr_corners(px, q)


def fr_ornate_full(q: Image.Image, rail: np.ndarray) -> Image.Image:
    # as `fr_ornate` with the kit's edge for the line: outer rows outward on every side, centred in its slice
    c, n = q.width, 2 * q.width + rail.shape[1]
    px = np.zeros((n, n, 4), dtype=np.uint8)
    px[c:n - c, c:n - c] = rgb(VOID, 0xf0)
    lo, hi = c // 2 - rail.shape[0] // 2, c // 2 - rail.shape[0] // 2 + rail.shape[0]
    px[lo:hi, c:n - c] = rail
    px[n - hi:n - lo, c:n - c] = rail[::-1]
    px[c:n - c, lo:hi] = rail.swapaxes(0, 1)
    px[c:n - c, n - hi:n - lo] = rail.swapaxes(0, 1)[:, ::-1]
    return fr_corners(px, q)


def ui_frames():
    # role -> (image, sidecar)
    def btn(edge, fill, lit):
        img = fr_box(16, 16, [edge, edge], fill)
        if lit:
            fr_lit(img)
        return img
    panel = fr_box(24, 24, [IRON, IRON, BASALT, BASALT], VOID, 0xf0)
    fr_rivets(panel)
    s4, s8 = {"slice": [4, 4, 4, 4]}, {"slice": [8, 8, 8, 8]}
    out = {
        "panel": (panel, s8),
        "tip": (fr_box(16, 16, [IRON, IRON, BASALT, BASALT], VOID, 0xf0), s4),
        "inset": (fr_inset(), s4),
        "parchment": (fr_box(16, 16, [OAK, OAK], LINEN), dict(s4, ink="ink")),
        "button": (btn(CINDER, BASALT, True), s4),
        "button_hot": (btn(TALLOW, SLATE, True), s4),
        "button_down": (btn(TALLOW, OAK, True), s4),
        "button_off": (btn(IRON, INSET_FILL, False), s4),
        "card": (btn(IRON, BASALT, False), s4),
        "card_hot": (btn(TALLOW, SLATE, False), s4),
        "card_on": (btn(TALLOW, OAK, False), s4),
    }
    try:
        q, anchor, inner = ornate_kit(ORNATE_SHEET)
        out["ornate"] = (fr_ornate(q), {"slice": [q.width] * 4, "corner_anchor": anchor, "corner_inner": inner, **ORNATE_SIDECAR})
        q, anchor, inner, rail = ornate_full_kit(ORNATE_SHEET)
        out["ornate_full"] = (fr_ornate_full(q, rail), {"slice": [q.width] * 4, "edges": "tile", "corner_anchor": anchor, "corner_inner": inner, **ORNATE_SIDECAR})
    except (FileNotFoundError, UnidentifiedImageError):  # no file, or an unsmudged LFS pointer
        print("ornate: no sheet at %s, its files are left as they are (--ornate SHEET)" % ORNATE_SHEET)
    out["tab"] = out["button"]
    out["tab_on"] = out["button_down"]
    return out


def g_ui_frames(out: Out) -> None:
    for role, (img, side) in sorted(ui_frames().items()):
        save(img, "game", "ui/frames/%s.png" % role, out)
        path = out.target("game", "ui/frames/%s.json" % role)
        if path:
            with open(path, "w", encoding="utf-8", newline="\n") as f:
                f.write(json.dumps(dict(side, scale=side.get("scale", 2))) + "\n")
            print("wrote game:ui/frames/%s.json" % role)


GROUPS = {
    "bodies": g_bodies, "portraits": g_portraits, "items": g_items, "skills": g_skills, "statuses": g_statuses,
    "terrain": g_terrain, "overlays": g_overlays, "region": g_region, "markers": g_markers, "pins": g_pins,
    "chips": g_chips, "world_map": g_world_map, "backdrops": g_backdrops, "menu": g_menu,
    "stage_tiles": g_stage_tiles, "battle_backdrops": g_battle_backdrops,
    "story": g_story, "world_card": g_world_card, "menu_sunless": g_menu_sunless, "sunk_backdrops": g_sunk_backdrops,
    "family_icons": g_family_icons, "ui_frames": g_ui_frames,
}


# ------------------------------------------------------------------------------
# Regeneration check (ASSETORG2-T): paths and pixel sizes of a regenerated tree against the committed one
# ------------------------------------------------------------------------------

SPOT_BODIES = ["kit:units/warrior/base", "kit:units/mage/base", "kit:units/brawler/goblin", "game:units/warrior/grakka_ashmaw"]


def generated_files(root: str) -> dict:
    # "<layer>:<rel>" -> what must match: a PNG's pixel size, a clips.json's frame and clip ids with frame counts.
    found = {}
    for layer in LAYERS:
        base = layer_root(root, layer)
        for dp, _, fn in os.walk(base):
            for f in fn:
                p = os.path.join(dp, f)
                rel = os.path.relpath(p, base).replace(os.sep, "/")
                if f.endswith(".png"):
                    with Image.open(p) as im:
                        found[layer + ":" + rel] = im.size
                elif f == "clips.json":
                    with open(p, encoding="utf-8") as fh:
                        c = json.load(fh)
                    found[layer + ":" + rel] = (tuple(c["frame"]), tuple((k["id"], k["frames"]) for k in c["clips"]))
    return found


def same_bytes(a: str, b: str) -> bool:
    with open(a, "rb") as fa, open(b, "rb") as fb:
        return fa.read() == fb.read()


def same_pixels(a: str, b: str) -> bool:
    with Image.open(a) as ia, Image.open(b) as ib:
        pa, pb = ia.convert("RGBA"), ib.convert("RGBA")
        return pa.size == pb.size and pa.tobytes() == pb.tobytes()


def spot(regen: str, folder: str) -> None:
    # committed | regenerated, each body's sheet at 3x on the rig's preview background
    os.makedirs(folder, exist_ok=True)
    for key in SPOT_BODIES:
        layer, rel = key.split(":")
        pics = []
        for root in (WORKSPACE, regen):
            p = os.path.join(layer_root(root, layer), rel, "sheet.png")
            pics.append(Image.open(p).convert("RGBA") if os.path.exists(p) else Image.new("RGBA", (104, 104), (90, 0, 0, 255)))
        w = max(p.width for p in pics) * 3
        h = max(p.height for p in pics) * 3
        board = Image.new("RGBA", (w * 2 + 24, h + 16), rgb("#2e342c"))
        for i, p in enumerate(pics):
            board.alpha_composite(p.resize((p.width * 3, p.height * 3), Image.NEAREST), (8 + i * (w + 8), 8))
        path = os.path.join(folder, "spot_%s.png" % rel.replace("units/", "").replace("/", "_"))
        board.save(path)
        print("spot", path)


def compare(regen: str) -> int:
    want, got = generated_files(WORKSPACE), generated_files(regen)
    missing = sorted(set(want) - set(got))
    extra = sorted(set(got) - set(want))
    both = sorted(set(want) & set(got))
    sizes = [k for k in both if want[k] != got[k]]
    pngs = [k for k in both if k.endswith(".png")]
    pairs = [(os.path.join(layer_root(WORKSPACE, k.split(":")[0]), k.split(":", 1)[1]),
              os.path.join(layer_root(regen, k.split(":")[0]), k.split(":", 1)[1])) for k in pngs]
    identical = sum(1 for a, b in pairs if same_bytes(a, b))
    pixels = sum(1 for a, b in pairs if same_pixels(a, b))
    for k in missing:
        print("missing (committed, not regenerated):", k)
    for k in extra:
        print("extra (regenerated, not committed):", k)
    for k in sizes:
        print("differs:", k, "committed", want[k], "regenerated", got[k])
    print("committed %d, regenerated %d: paths %s (%d missing, %d extra), sizes %s (%d differ); of %d PNGs %d pixel-identical, "
          "%d byte-identical (the rest: ART2 art over the placeholders, or drift)"
          % (len(want), len(got), "match" if not missing and not extra else "DIFFER", len(missing), len(extra),
             "match" if not sizes else "DIFFER", len(sizes), len(pngs), pixels, identical))
    return 1 if missing or extra or sizes else 0


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("groups", nargs="*")
    ap.add_argument("--layer", action="append", choices=list(LAYERS), help="write only these layers (default all)")
    ap.add_argument("--root", default=WORKSPACE, help="workspace root holding engine/assets, kits/rpg/assets, games/emberwake/assets")
    ap.add_argument("--overwrite", action="store_true", help="replace existing files (default: keep them)")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--compare", metavar="DIR", help="check a regenerated workspace root against the committed files")
    ap.add_argument("--spot", metavar="OUT", help="with --compare: write committed | regenerated body sheets to OUT")
    ap.add_argument("--ornate", metavar="SHEET", help="ui_frames: another frame kit than the picked one")
    a = ap.parse_args()
    if a.list:
        print(" ".join(GROUPS))
        return
    if a.compare:
        if a.spot:
            spot(a.compare, a.spot)
        sys.exit(compare(a.compare))
    global ORNATE_SHEET
    ORNATE_SHEET = a.ornate or ORNATE_SHEET
    out = Out(a.root, a.layer or list(LAYERS), a.overwrite)
    for g in a.groups or list(GROUPS):
        if g not in GROUPS:
            sys.exit("unknown group " + g)
        GROUPS[g](out)


if __name__ == "__main__":
    main()
