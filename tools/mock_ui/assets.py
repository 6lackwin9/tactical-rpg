#!/usr/bin/env python3
"""
Layered asset bundler for the Emberwake UI prototype (`tools/mock_ui`).

Reads directly from the repository's 3-layer asset hierarchy:
  1. `games/emberwake/assets`
  2. `kits/rpg/assets`
  3. `engine/assets`

Extracts:
  - Embedded TTF fonts (`AlegreyaSC-Bold`, `OpenSans-SemiBold`, `JetBrainsMono-Bold`)
  - 1280x720 and 640x360 backgrounds (including `backgrounds/menu/title.png`)
  - Reduced Corner-Only Ornate Frame (`ui/frames/ornate.png` + `ornate.json`)
  - Full Ornate Frame with Borders & Medallions (`ui/frames/ornate_full.png` + `ornate_full.json`)
  - UI icons, pins, markers, badges, and intent glyphs
  - Hero, NPC, Elite, Boss, and Common portraits
  - Palette-swapped unit animation frames (`idle0`, `idle1`, `atk`) from `sheet.png` + `clips.json`
  - Skill and status icons
"""
from __future__ import annotations

import base64
import io
import json
from pathlib import Path
from typing import Any

from PIL import Image, ImageFilter

REPO_ROOT = Path(__file__).resolve().parents[2]
GAME_ASSETS = REPO_ROOT / "games" / "emberwake" / "assets"
KIT_ASSETS = REPO_ROOT / "kits" / "rpg" / "assets"
ENGINE_ASSETS = REPO_ROOT / "engine" / "assets"


def find_asset(rel: str) -> Path:
    for base in (GAME_ASSETS, KIT_ASSETS, ENGINE_ASSETS):
        p = base / rel
        if p.is_file():
            return p
    raise FileNotFoundError(f"Asset not found in game/kit/engine layers: {rel}")


def img_to_data_uri(img: Image.Image, fmt: str = "PNG", quantize_colors: int = 48) -> str:
    buf = io.BytesIO()
    if fmt.upper() == "JPEG":
        img.convert("RGB").save(buf, format="JPEG", quality=76, optimize=True)
        return "data:image/jpeg;base64," + base64.b64encode(buf.getvalue()).decode("ascii")
    out_img = img
    if quantize_colors > 0 and img.mode == "RGBA" and (img.width * img.height >= 2048):
        try:
            out_img = img.quantize(colors=quantize_colors, method=Image.Quantize.FASTOCTREE)
        except Exception:
            out_img = img
    out_img.save(buf, format="PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode("ascii")


def png_to_data_uri(img: Image.Image, quantize_colors: int = 48) -> str:
    return img_to_data_uri(img, "PNG", quantize_colors=quantize_colors)


def load_scaled_uri(
    rel: str,
    size: tuple[int, int] | None = None,
    smooth_downsample: bool = True,
    fmt: str = "PNG",
) -> str:
    p = find_asset(rel)
    im = Image.open(p).convert("RGBA")
    if size is not None and im.size != size:
        if smooth_downsample and (size[0] < im.size[0] or size[1] < im.size[1]):
            im = im.resize(size, Image.Resampling.BOX)
        else:
            im = im.resize(size, Image.Resampling.NEAREST)
    return img_to_data_uri(im, fmt)


def font_to_data_uri(rel: str) -> str:
    p = find_asset(rel)
    raw = p.read_bytes()
    return "data:font/ttf;base64," + base64.b64encode(raw).decode("ascii")


def extract_ornate_corner_pieces() -> dict[str, str]:
    """
    Extracts the straddling 9-slice pieces of `ui/frames/ornate.png`
    (`slice: [200, 200, 200, 200]`, `corner_anchor: [72, 80]`, `scale: 8`, `sharpen: 0.6`)
    at 2x physical resolution (50x50 physical px corners, 8px border bands).
    Used for small popups and sub-cards.
    """
    p = find_asset("ui/frames/ornate.png")
    im = Image.open(p).convert("RGBA")
    im108 = im.resize((108, 108), Image.Resampling.BOX)
    sharp = im108.filter(ImageFilter.UnsharpMask(radius=1.0, percent=60, threshold=0))
    tl = sharp.crop((0, 0, 50, 50))
    tr = sharp.crop((58, 0, 108, 50))
    bl = sharp.crop((0, 58, 50, 108))
    br = sharp.crop((58, 58, 108, 108))
    top_band = im108.crop((50, 0, 58, 50))
    bot_band = im108.crop((50, 58, 58, 108))
    left_band = im108.crop((0, 50, 50, 58))
    right_band = im108.crop((58, 50, 108, 58))
    return {
        "ornate_tl": png_to_data_uri(tl),
        "ornate_tr": png_to_data_uri(tr),
        "ornate_bl": png_to_data_uri(bl),
        "ornate_br": png_to_data_uri(br),
        "ornate_corner": png_to_data_uri(tl),
        "ornate_top": png_to_data_uri(top_band),
        "ornate_bot": png_to_data_uri(bot_band),
        "ornate_left": png_to_data_uri(left_band),
        "ornate_right": png_to_data_uri(right_band),
    }


def extract_ornate_full_pieces() -> dict[str, str]:
    """
    Extracts the straddling 9-slice pieces of `ui/frames/ornate_full.png`
    (`slice: [224, 224, 224, 224]`, `edges: "tile"`, `corner_anchor: [80, 80]`, `scale: 8`, `sharpen: 0.6`)
    at 2x physical resolution (56x56 physical px corners, 16px border strips).
    Used for large screens and full/large modals.
    """
    p = find_asset("ui/frames/ornate_full.png")
    im = Image.open(p).convert("RGBA")
    im128 = im.resize((128, 128), Image.Resampling.BOX)
    sharp = im128.filter(ImageFilter.UnsharpMask(radius=1.0, percent=60, threshold=0))
    tl = sharp.crop((0, 0, 56, 56))
    tr = sharp.crop((72, 0, 128, 56))
    bl = sharp.crop((0, 72, 56, 128))
    br = sharp.crop((72, 72, 128, 128))
    top_strip = sharp.crop((56, 0, 72, 56))
    bot_strip = sharp.crop((56, 72, 72, 128))
    left_strip = sharp.crop((0, 56, 56, 72))
    right_strip = sharp.crop((72, 56, 128, 72))
    return {
        "ornate_full_tl": png_to_data_uri(tl),
        "ornate_full_tr": png_to_data_uri(tr),
        "ornate_full_bl": png_to_data_uri(bl),
        "ornate_full_br": png_to_data_uri(br),
        "ornate_full_top": png_to_data_uri(top_strip),
        "ornate_full_bot": png_to_data_uri(bot_strip),
        "ornate_full_left": png_to_data_uri(left_strip),
        "ornate_full_right": png_to_data_uri(right_strip),
    }


def hex_to_rgb(h: str) -> tuple[int, int, int]:
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16))


def extract_unit_frames(body_key: str, palette_id: str = "") -> dict[str, str]:
    """
    Loads `units/<body_key>/clips.json` and `sheet.png` from layered assets,
    applies `palettes/<palette_id>.json` like `aegis_ui::art::Art::load_body`,
    and extracts [idle0, idle1, atk] as PNG data URIs.
    """
    clips_path = find_asset(f"units/{body_key}/clips.json")
    sheet_path = find_asset(f"units/{body_key}/sheet.png")
    clips_data = json.loads(clips_path.read_text(encoding="utf-8"))
    fw, fh = clips_data["frame"]
    base_pal = {k: hex_to_rgb(v) for k, v in clips_data.get("palette", {}).items()}

    sheet = Image.open(sheet_path).convert("RGBA")
    if palette_id:
        try:
            pal_path = find_asset(f"units/{body_key}/palettes/{palette_id}.json")
            slots = json.loads(pal_path.read_text(encoding="utf-8")).get("slots", {})
            swap_map = {
                base_pal[slot_name]: hex_to_rgb(target_hex)
                for slot_name, target_hex in slots.items()
                if slot_name in base_pal
            }
            if swap_map:
                px = list(sheet.getdata())
                sheet.putdata([
                    (*swap_map[(r, g, b)], a) if a > 0 and (r, g, b) in swap_map else (r, g, b, a)
                    for (r, g, b, a) in px
                ])
        except FileNotFoundError:
            pass

    row_map = {c["id"]: (idx, c) for idx, c in enumerate(clips_data["clips"])}
    idle_row, idle_clip = row_map.get("idle", (0, clips_data["clips"][0]))
    atk_row, atk_clip = row_map.get("attack", (idle_row, idle_clip))

    def crop_frame(row: int, col: int) -> str:
        box = (col * fw, row * fh, (col + 1) * fw, (row + 1) * fh)
        return png_to_data_uri(sheet.crop(box))

    f0 = crop_frame(idle_row, 0)
    f1 = crop_frame(idle_row, min(2, idle_clip["frames"] - 1))
    atk_col = atk_clip.get("impact") or min(2, atk_clip["frames"] - 1)
    fatk = crop_frame(atk_row, atk_col)
    return {"idle0": f0, "idle1": f1, "atk": fatk}


def split_vtuber_layers(char_key: str, rel_path: str) -> dict[str, Any]:
    """
    Splits a 340x340 full-body sprite into Live2D / VTuber-style parts:
      - `body`: Torso, head, legs, and coat (receives continuous mesh strip deformation:
                knee flex, hip drop, spine lean, chest breathing, head counter-tilt, coat pendulum).
      - `weapon`: Weapon & weapon-hand layer (rotates as a rigid body around `pivot = [px, py]`
                  so polearms/blades never bend when knees flex or when drawing steel).
    """
    p = find_asset(rel_path)
    im = Image.open(p).convert("RGBA")
    if im.size != (340, 340):
        im = im.resize((340, 340), Image.Resampling.NEAREST)

    w, h = im.size
    src_px = im.load()
    body_im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    weap_im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    body_px = body_im.load()
    weap_px = weap_im.load()

    pivot = [180, 152]
    native_left = char_key in ("vane", "watchman", "bailiff")

    for y in range(h):
        for x in range(w):
            rgba = src_px[x, y]
            if rgba[3] == 0:
                continue
            is_weap = False
            if char_key in ("kaelen", "kaelen_side"):
                pivot = [220, 156]
                if y < 138:
                    xc = 205
                elif y < 172:
                    xc = 215
                else:
                    xc = int(round(210 - (y - 172) * 0.17))
                is_weap = x >= xc and y <= 309
            elif char_key == "kaelen_front":
                pivot = [228, 156]
                if y < 138:
                    xc = 210
                elif y < 175:
                    xc = 220
                else:
                    xc = 201
                is_weap = x >= xc
            elif char_key == "kaelen_back":
                pivot = [110, 156]
                if y < 138:
                    xc = 118
                elif y < 175:
                    xc = 108
                else:
                    xc = 136
                is_weap = x <= xc
            elif char_key in ("watchman", "bailiff"):
                pivot = [124, 155]
                if y < 135:
                    xc = 135
                elif y < 175:
                    xc = 122
                else:
                    xc = int(120 + (y - 175) * 0.11)
                is_weap = x <= xc
            elif char_key == "vesper":
                pivot = [190, 165]
                is_weap = (x >= 200 and 120 <= y <= 230) or (x <= 116 and 155 <= y <= 225)
            elif char_key == "maren":
                pivot = [192, 155]
                is_weap = x >= 204 and 125 <= y <= 265
            elif char_key == "vane":
                pivot = [132, 162]
                is_weap = x <= 122 and 140 <= y <= 305
            elif char_key == "sabina":
                pivot = [192, 160]
                is_weap = x >= 200 and 120 <= y <= 245

            if is_weap:
                weap_px[x, y] = rgba
            else:
                body_px[x, y] = rgba

    return {
        "body": png_to_data_uri(body_im),
        "weapon": png_to_data_uri(weap_im),
        "pivot": pivot,
        "nativeLeft": native_left,
    }


def build_asset_bundle() -> dict[str, Any]:
    ui_dict: dict[str, str] = {
        "logo_title": load_scaled_uri("ui/logo_title.png", (470, 124)),
        "logo_crest": load_scaled_uri("ui/logo_crest.png", (96, 96)),
        "panel": load_scaled_uri("ui/frames/panel.png"),
        "card": load_scaled_uri("ui/frames/card.png"),
        "card_hot": load_scaled_uri("ui/frames/card_hot.png"),
        "card_on": load_scaled_uri("ui/frames/card_on.png"),
        "button": load_scaled_uri("ui/frames/button.png"),
        "button_hot": load_scaled_uri("ui/frames/button_hot.png"),
        "button_down": load_scaled_uri("ui/frames/button_down.png"),
        "button_off": load_scaled_uri("ui/frames/button_off.png"),
        "inset": load_scaled_uri("ui/frames/inset.png"),
        "tab": load_scaled_uri("ui/frames/tab.png"),
        "tab_on": load_scaled_uri("ui/frames/tab_on.png"),
        "lock": load_scaled_uri("ui/lock.png", (16, 16), False),
        "unlock": load_scaled_uri("ui/unlock.png", (16, 16), False),
        "delete": load_scaled_uri("ui/delete.png", (16, 16), False),
        "chevron": load_scaled_uri("ui/chevron.png", (16, 16), False),
        "gold": load_scaled_uri("ui/hud/gold.png", (16, 16), False),
        "supplies": load_scaled_uri("ui/hud/supplies.png", (16, 16), False),
        "party": load_scaled_uri("ui/hud/party.png", (16, 16), False),
        "journal": load_scaled_uri("ui/hud/journal.png", (16, 16), False),
        "stash": load_scaled_uri("ui/hud/stash.png", (16, 16), False),
        "menu": load_scaled_uri("ui/hud/menu.png", (16, 16), False),
        "help": load_scaled_uri("ui/help.png", (16, 16), False),
        "pin_here": load_scaled_uri("ui/pins/here.png", (16, 16), False),
        "pin_town": load_scaled_uri("ui/pins/town.png", (16, 16), False),
        "pin_gate": load_scaled_uri("ui/pins/gate.png", (16, 16), False),
        "pin_ruin": load_scaled_uri("ui/pins/ruin.png", (16, 16), False),
        "pin_fortress": load_scaled_uri("ui/pins/fortress.png", (16, 16), False),
        "pin_camp": load_scaled_uri("ui/pins/camp.png", (16, 16), False),
        "badge_quest": load_scaled_uri("ui/badges/quest.png", (12, 12), False),
        "badge_foresight": load_scaled_uri("ui/badges/foresight.png", (12, 12), False),
        "marker_npc": load_scaled_uri("ui/markers/npc.png", (16, 16), False),
        "marker_enc": load_scaled_uri("ui/markers/encounter.png", (16, 16), False),
        "marker_exit": load_scaled_uri("ui/markers/world_map.png", (16, 16), False),
        "marker_chest": load_scaled_uri("ui/markers/chest.png", (16, 16), False),
        "marker_lore": load_scaled_uri("ui/markers/lore.png", (16, 16), False),
        "service_shop": load_scaled_uri("ui/services/shop.png", (12, 12), False),
        "service_recruit": load_scaled_uri("ui/services/recruit.png", (12, 12), False),
        "intent_attack": load_scaled_uri("ui/intents/attack.png", (12, 12), False),
        "intent_area": load_scaled_uri("ui/intents/area.png", (12, 12), False),
        "intent_guard": load_scaled_uri("ui/intents/guard.png", (12, 12), False),
        "intent_move": load_scaled_uri("ui/intents/move.png", (12, 12), False),
        "intent_rest": load_scaled_uri("ui/intents/rest.png", (12, 16), False),
        "intent_support": load_scaled_uri("ui/intents/support.png", (12, 12), False),
    }
    ui_dict.update(extract_ornate_corner_pieces())
    ui_dict.update(extract_ornate_full_pieces())

    assets: dict[str, Any] = {
        "fonts": {
            "display": font_to_data_uri("fonts/AlegreyaSC-Bold.ttf"),
            "body": font_to_data_uri("fonts/OpenSans-SemiBold.ttf"),
            "mono": font_to_data_uri("fonts/JetBrainsMono-Bold.ttf"),
        },
        "bg": {
            "title": load_scaled_uri("backgrounds/menu/title.png", (1280, 720), fmt="JPEG"),
            "vn_alley_stage": load_scaled_uri("backgrounds/story/vn_alley_stage.png", (1280, 720), fmt="JPEG"),
            "vn_rooftop_stage": load_scaled_uri("backgrounds/story/vn_rooftop_stage.png", (1280, 720), fmt="JPEG"),
            "vn_bridge_stage": load_scaled_uri("backgrounds/story/vn_bridge_stage.png", (1280, 720), fmt="JPEG"),
            "vn_hearth_stage": load_scaled_uri("backgrounds/story/vn_hearth_stage.png", (1280, 720), fmt="JPEG"),
            "sunk_ward": load_scaled_uri("backgrounds/title/sunk_ward.png", (500, 316), fmt="JPEG"),
            "intro_1": load_scaled_uri("backgrounds/story/intro_1.png", (640, 360), fmt="JPEG"),
            "intro_2": load_scaled_uri("backgrounds/story/intro_2.png", (640, 360), fmt="JPEG"),
            "hearth": load_scaled_uri("backgrounds/story/hearth.png", (640, 360), fmt="JPEG"),
            "meat_market": load_scaled_uri("backgrounds/story/meat_market.png", (640, 360), fmt="JPEG"),
            "alley": load_scaled_uri("backgrounds/battle/alley.png", (384, 52)),
            "rooftops": load_scaled_uri("backgrounds/battle/rooftops.png", (384, 52)),
            "canal": load_scaled_uri("backgrounds/battle/canal.png", (384, 52)),
            "foundry": load_scaled_uri("backgrounds/battle/foundry.png", (384, 52)),
            "watchpost": load_scaled_uri("backgrounds/battle/watchpost.png", (384, 52)),
        },
        "ui": ui_dict,
        "portraits": {
            "kaelen": load_scaled_uri("portraits/heroes/kaelen.png", (120, 120)),
            "vesper": load_scaled_uri("portraits/heroes/vesper.png", (120, 120)),
            "maren": load_scaled_uri("portraits/heroes/maren.png", (120, 120)),
            "hollis": load_scaled_uri("portraits/heroes/hollis.png", (120, 120)),
            "dunstan": load_scaled_uri("portraits/heroes/dunstan.png", (120, 120)),
            "elyse": load_scaled_uri("portraits/heroes/elyse.png", (120, 120)),
            "sabina": load_scaled_uri("portraits/npcs/sabina_kress.png", (120, 120)),
            "berek": load_scaled_uri("portraits/npcs/berek_dorn.png", (120, 120)),
            "talia": load_scaled_uri("portraits/npcs/talia_rusk.png", (120, 120)),
            "vane": load_scaled_uri("portraits/bosses/captain_vane.png", (120, 120)),
            "quell": load_scaled_uri("portraits/bosses/proctor_quell.png", (120, 120)),
            "orlov": load_scaled_uri("portraits/bosses/guild_master_orlov.png", (120, 120)),
            "nemeth": load_scaled_uri("portraits/bosses/inquisitor_prelate_nemeth.png", (120, 120)),
            "purifier": load_scaled_uri("portraits/elites/inquisitorial_purifier.png", (120, 120)),
            "husk": load_scaled_uri("portraits/elites/calcified_husk.png", (120, 120)),
            "watchman": load_scaled_uri("portraits/common/warrior_base.png", (120, 120)),
            "crossbow": load_scaled_uri("portraits/common/ranger_base.png", (120, 120)),
            "bailiff": load_scaled_uri("portraits/common/warrior_base.png", (120, 120)),
            "flagellant": load_scaled_uri("portraits/common/brawler_base.png", (120, 120)),
            "flayer": load_scaled_uri("portraits/npcs/wenn_tallow.png", (120, 120)),
            "crawler": load_scaled_uri("portraits/common/crawler_ash_crawler.png", (120, 120)),
            "beast": load_scaled_uri("portraits/common/beast_render_beast.png", (120, 120)),
        },
        "fullBody": {},
        "vtuberRig": {
            "kaelen": split_vtuber_layers("kaelen", "portraits/full_body/kaelen.png"),
            "kaelen_side": split_vtuber_layers("kaelen_side", "portraits/full_body/kaelen_side.png"),
            "kaelen_front": split_vtuber_layers("kaelen_front", "portraits/full_body/kaelen_front.png"),
            "kaelen_back": split_vtuber_layers("kaelen_back", "portraits/full_body/kaelen_back.png"),
            "vane": split_vtuber_layers("vane", "portraits/full_body/vane.png"),
            "vesper": split_vtuber_layers("vesper", "portraits/full_body/vesper.png"),
            "maren": split_vtuber_layers("maren", "portraits/full_body/maren.png"),
            "sabina": split_vtuber_layers("sabina", "portraits/full_body/sabina_kress.png"),
            "watchman": split_vtuber_layers("watchman", "portraits/full_body/watchman.png"),
        },
        "partsAtlas": {
            "kaelen": load_scaled_uri("portraits/full_body/kaelen_parts_atlas.png", (495, 276), True),
        },
        "poseSheets": {
            "kaelen": {
                "relaxed": load_scaled_uri("portraits/full_body/kaelen_pose_relaxed.png", (340, 340), True),
                "sneaking": load_scaled_uri("portraits/full_body/kaelen_pose_sneaking.png", (340, 340), True),
                "weapons_drawn": load_scaled_uri("portraits/full_body/kaelen_pose_drawn.png", (340, 340), True),
            },
        },
        "units3View": {
            "kaelen": {
                "side": load_scaled_uri("portraits/full_body/kaelen_side.png", (64, 64), False),
                "front": load_scaled_uri("portraits/full_body/kaelen_front.png", (64, 64), False),
                "back": load_scaled_uri("portraits/full_body/kaelen_back.png", (64, 64), False),
            },
        },
        "faces": {},
        "units": {
            "kaelen": extract_unit_frames("warrior/2h_axe", "kaelen"),
            "vesper": extract_unit_frames("rogue/base", "vesper"),
            "maren": extract_unit_frames("priest/base", "maren"),
            "hollis": extract_unit_frames("ranger/base", "hollis"),
            "dunstan": extract_unit_frames("brawler/base", "dunstan"),
            "elyse": extract_unit_frames("ranger/wenn_tallow", "elyse"),
            "sabina": extract_unit_frames("priest/base", "maren"),
            "berek": extract_unit_frames("warrior/2h_hammer", "grendel"),
            "talia": extract_unit_frames("rogue/base", "captive"),
            "watchman": extract_unit_frames("warrior/1h_buckler", "castellan_watch"),
            "crossbow": extract_unit_frames("ranger/base", "castellan_watch"),
            "bailiff": extract_unit_frames("warrior/2h_hammer", "bailiff"),
            "purifier": extract_unit_frames("priest/base", "purifier"),
            "flagellant": extract_unit_frames("brawler/base", "flagellant"),
            "flayer": extract_unit_frames("ranger/wenn_tallow", "tallow_guild"),
            "husk": extract_unit_frames("brawler/base", "calcified"),
            "vane": extract_unit_frames("warrior/2h_axe", "vane"),
            "quell": extract_unit_frames("warrior/2h_sword", "quell"),
            "grendel": extract_unit_frames("warrior/2h_hammer", "grendel"),
            "orlov": extract_unit_frames("brawler/base", "orlov"),
            "nemeth": extract_unit_frames("warrior/2h_sword", "nemeth"),
            "beast": extract_unit_frames("beast/render_beast", ""),
            "crawler": extract_unit_frames("crawler/ash_crawler", ""),
        },
        "skills": {},
        "statuses": {},
    }

    # Bundle mood expression portraits (`PORTRAITFX1` faces/<id>/<mood>.png)
    face_sources = [
        ("kaelen", "portraits/heroes/faces/kaelen"),
        ("vesper", "portraits/heroes/faces/vesper"),
        ("maren", "portraits/heroes/faces/maren"),
        ("hollis", "portraits/heroes/faces/hollis"),
        ("dunstan", "portraits/heroes/faces/dunstan"),
        ("elyse", "portraits/heroes/faces/elyse"),
        ("sabina", "portraits/npcs/faces/sabina_kress"),
        ("berek", "portraits/npcs/faces/berek_dorn"),
        ("talia", "portraits/npcs/faces/talia_rusk"),
        ("vane", "portraits/bosses/faces/captain_vane"),
        ("quell", "portraits/bosses/faces/proctor_quell"),
        ("orlov", "portraits/bosses/faces/guild_master_orlov"),
        ("nemeth", "portraits/bosses/faces/inquisitor_prelate_nemeth"),
    ]
    for char_key, rel_dir in face_sources:
        assets["faces"][char_key] = {}
        for mood in ("neutral", "angry", "determined", "hurt", "afraid", "sad", "surprised", "happy"):
            try:
                assets["faces"][char_key][mood] = load_scaled_uri(f"{rel_dir}/{mood}.png", (120, 120))
            except FileNotFoundError:
                pass

    skill_names = [
        "move", "rest", "strike", "active_guard", "disengage",
        "shield_bash", "flank_cut", "field_mend", "quick_shot", "offhand_jab", "arcane_bolt",
        "billhook_hook", "billhook_sunder", "cleave", "iron_brace", "sweeping_heave",
        "shadow_step", "throat_slit", "ash_cloud_fade", "smoke_bomb", "bleeding_cut",
        "cauterize", "volatile_flask", "tallow_leech", "Purge", "mend",
        "pinning_bolt", "overwatch_line", "ricochet_bolt", "caltrop_scatter", "aimed_bolt",
        "furnace_haymaker", "slag_vent_slam", "cinder_shove", "stagger_blow",
        "arc_lance", "static_tether", "galvanic_surge", "ward_pulse",
    ]
    for sk in skill_names:
        try:
            assets["skills"][sk] = load_scaled_uri(f"skills/{sk}.png", (16, 16))
        except FileNotFoundError:
            pass

    for st in ["bleed", "burn", "poison", "slow", "stun", "vulnerable", "weaken", "haste", "regen", "focus", "shielded"]:
        try:
            assets["statuses"][st] = load_scaled_uri(f"statuses/{st}.png", (12, 12))
        except FileNotFoundError:
            pass

    return assets


def load_or_build_bundle(cache_path: Path | None = None, rebuild: bool = False) -> dict[str, Any]:
    if cache_path and cache_path.is_file() and not rebuild:
        return json.loads(cache_path.read_text(encoding="utf-8"))
    bundle = build_asset_bundle()
    if cache_path:
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        cache_path.write_text(json.dumps(bundle), encoding="utf-8")
    return bundle
