"""ASSETORG2-G: the move of games/emberwake/assets/art/ (ASSETORG1, one folder per thing) to the layered tree of
docs/specs/ASSETORG2.md section 3, inside the game root (ASSETORG2-K and -B then move the kit- and engine-bound
folders out). The table is the spec's section 9 as revised after ART2 (section 9b): it is computed here from the
files present and checked (every deletion is pixel-identical to the file that stays, or an approved ash_* placeholder).

  python tools/assetorg2_move.py plan     prints the moves as a markdown table and the counts
  python tools/assetorg2_move.py apply    git mv every file, writes the bodies' clips.json, deletes the rest
"""
import hashlib
import json
import os
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "games", "emberwake", "assets")
ART = os.path.join(ROOT, "art")

CLASSES = ["warrior", "ranger", "brawler", "priest", "rogue", "mage"]
# ASSETORG1 unit folder -> (body key, layer) for the kit's goblin variants and the game's named bodies
VARIANTS = {"goblin_brawler": "brawler/goblin", "skirmisher": "rogue/goblin", "archer": "ranger/goblin", "shaman": "priest/goblin",
            "berserker": "brawler/goblin_berserker", "scout": "rogue/goblin_scout", "dummy": "dummy"}
NAMED = {"hest_the_quartermaster": ("warrior", "npcs"), "brask_toller": ("warrior", "npcs"), "wenn_tallow": ("ranger", "npcs"),
         "skarn_ironjaw": ("brawler", "elites"), "vex_cutpurse": ("rogue", "elites"), "nok_flamebow": ("ranger", "elites"),
         "bolg_redhand": ("brawler", "elites"), "ash_warchief": ("brawler", "elites"), "grakka_ashmaw": ("warrior", "bosses")}
# characters whose token is their class (or variant) token: the token goes, the unique portrait stays
HEROES = {"mara_voss": "warrior", "bryn_hale": "warrior", "tev_quill": "ranger", "sella_marsh": "ranger", "kit_vane": "rogue",
          "oren_dusk": "mage", "alda_wren": "priest", "dagny_coal": "brawler"}
TOKEN_ONLY_COPIES = {"gruk_smokecaller": ("shaman", "elites")}
# approved ASSETORG2 Q3: the ash_* mob placeholders go (tokens near-copies of the goblin variants, portraits identical)
ASH = {"ash_brawler": "goblin_brawler", "ash_skirmisher": "skirmisher", "ash_archer": "archer", "ash_shaman": "shaman", "ash_berserker": "berserker"}
SKILL_KINDS = {"strike": "attack", "guard": "defend", "focus": "focus", "mend": "support", "backflip": "disengage", "rest": "rest",
               "move": "move", "switch_target": "target", "staggered": "status"}
ITEM_BASES = ["sword", "axe", "hammer", "dagger", "bow", "staff", "wand", "tome", "shield", "plate", "leather", "fist", "cloth", "mace", "orb",
              "crossbow", "material", "quest"]
ITEM_VARIANTS = {"knuckles": "fist/knuckles", "robe": "cloth/robe", "greatsword": "sword/greatsword", "shortsword": "sword/shortsword",
                 "battle_axe": "axe/battle_axe", "ring": "accessory/base", "amulet": "accessory/amulet", "potion": "consumable/base",
                 "elixir": "consumable/elixir", "bomb": "consumable/bomb", "weapon": "any/weapon", "misc": "any/base"}
ITEM_GAME = {"titan_warhammer": "hammer/titan_warhammer", "runed_staff": "staff/runed_staff", "ember_brigandine": "leather/ember_brigandine"}
REGION_TILES = {"ash_floor": "floor", "basalt_wall": "wall", "water": "shallow_water"}
LOCATIONS = ["warden_keep", "ruined_courtyard", "cinder_gate", "ashmaw_camp"]
BODY_CLIPS = {"frame": [104, 104], "clips": [{"id": "idle", "frames": 1, "ms": [1000]}]}

# where each new path's folder goes in ASSETORG2-K / -B (engine, kit or game)
def layer_of(dst):
    if dst.startswith(("tilesets/common/",)):
        return "engine"
    if dst.startswith(("units/", "portraits/common/", "items/", "skills/kinds/", "statuses/", "map/", "ui/")):
        named = dst.startswith("units/") and any(f"/{n}/" in dst for n in NAMED)
        game_item = dst.startswith("items/") and any(dst.startswith(f"items/{v}") for v in ITEM_GAME.values())
        return "game" if named or game_item else "kit"
    return "game"


def md5(p):
    return hashlib.md5(open(p, "rb").read()).hexdigest()


def plan():
    moves, deletions, bodies = [], [], []
    a = lambda *p: "/".join(("art",) + p)
    for c in CLASSES:
        moves.append((a("unit", c, "token.png"), f"units/{c}/base/sheet.png"))
        moves.append((a("unit", c, "portrait.png"), f"portraits/common/{c}_base.png"))
        bodies.append(c)
    for src, key in VARIANTS.items():
        cls, var = (key.split("/") + ["base"])[:2]
        moves.append((a("unit", src, "token.png"), f"units/{cls}/{var}/sheet.png"))
        moves.append((a("unit", src, "portrait.png"), f"portraits/common/{cls}_{var}.png"))
        bodies.append(key)
    for src, (cls, tier) in NAMED.items():
        moves.append((a("unit", src, "token.png"), f"units/{cls}/{src}/sheet.png"))
        moves.append((a("unit", src, "portrait.png"), f"portraits/{tier}/{src}.png"))
        bodies.append(f"{cls}/{src}")
    for src, cls in HEROES.items():
        deletions.append((a("unit", src, "token.png"), a("unit", cls, "token.png")))
        moves.append((a("unit", src, "portrait.png"), f"portraits/heroes/{src}.png"))
    for src, (same, tier) in TOKEN_ONLY_COPIES.items():
        deletions.append((a("unit", src, "token.png"), a("unit", same, "token.png")))
        moves.append((a("unit", src, "portrait.png"), f"portraits/{tier}/{src}.png"))
    for src, variant in ASH.items():
        deletions.append((a("unit", src, "token.png"), "approved Q3 placeholder (near-copy of " + variant + ")"))
        deletions.append((a("unit", src, "portrait.png"), a("unit", variant, "portrait.png")))
    for d in sorted(os.listdir(os.path.join(ART, "skill"))):
        if os.path.isdir(os.path.join(ART, "skill", d)):
            moves.append((a("skill", d, "icon.png"), f"skills/kinds/{SKILL_KINDS[d]}.png" if d in SKILL_KINDS else f"skills/{d}.png"))
    for f in ITEM_BASES:
        moves.append((a("item", f, "icon.png"), f"items/{f}/base.png"))
    for src, dst in list(ITEM_VARIANTS.items()) + list(ITEM_GAME.items()):
        moves.append((a("item", src, "icon.png"), f"items/{dst}.png"))
    for d in sorted(os.listdir(os.path.join(ART, "status"))):
        if os.path.isdir(os.path.join(ART, "status", d)):
            moves.append((a("status", d, "icon.png"), f"statuses/{d}.png"))
    for d in sorted(os.listdir(os.path.join(ART, "tile"))):
        p = os.path.join(ART, "tile", d)
        if not os.path.isdir(p):
            continue
        if d.startswith("overlay_"):
            moves.append((a("tile", d, "v0.png"), f"map/markers/{d[len('overlay_'):]}.png"))
        else:
            for v in sorted(os.listdir(p)):
                moves.append((a("tile", d, v), f"tilesets/common/{d}/{v}"))
    for d in sorted(os.listdir(os.path.join(ART, "ui"))):
        if not os.path.isdir(os.path.join(ART, "ui", d)):
            continue
        if d.startswith("region_"):
            name, n = d[len("region_"):].rsplit("_", 1)
            moves.append((a("ui", d, "icon.png"), f"tilesets/ashlands/{REGION_TILES.get(name, name)}/v{n}.png"))
        elif d.startswith("pin_"):
            moves.append((a("ui", d, "icon.png"), f"ui/pins/{d[4:]}.png"))
        elif d.startswith("intent_"):
            moves.append((a("ui", d, "icon.png"), f"ui/intents/{d[7:]}.png"))
        elif d.startswith("badge_"):
            moves.append((a("ui", d, "icon.png"), f"ui/badges/{d[6:]}.png"))
        elif d.startswith("chip_"):
            moves.append((a("ui", d, "icon.png"), f"ui/chips/{d[5:]}.png"))
        elif d == "main_menu":
            moves.append((a("ui", d, "background.png"), "backgrounds/menu/main_menu.png"))
        elif d.endswith("_title"):
            moves.append((a("ui", d, "background.png"), f"backgrounds/title/{d[:-len('_title')]}.png"))
        elif d.endswith("_world_map"):
            moves.append((a("ui", d, "background.png"), f"backgrounds/world_map/{d[:-len('_world_map')]}.png"))
        elif d in LOCATIONS:
            moves.append((a("ui", d, "background.png"), f"backgrounds/location/{d}.png"))
        else:
            raise SystemExit(f"no rule for ui/{d}")
    return moves, deletions, bodies


def check(moves, deletions):
    pngs = set()
    for dp, _, fn in os.walk(ART):
        for f in fn:
            if f.endswith(".png"):
                pngs.add(os.path.relpath(os.path.join(dp, f), ROOT).replace("\\", "/"))
    srcs = [m[0] for m in moves] + [d[0] for d in deletions]
    problems = []
    if len(set(srcs)) != len(srcs):
        problems.append("a file is moved or deleted twice")
    if set(srcs) != pngs:
        problems.append(f"not covered: {sorted(pngs - set(srcs))}; unknown: {sorted(set(srcs) - pngs)}")
    dsts = [m[1] for m in moves]
    if len(set(dsts)) != len(dsts):
        problems.append("two files land on one path")
    for src, same in deletions:
        if same.startswith("art/") and md5(os.path.join(ROOT, src)) != md5(os.path.join(ROOT, same)):
            problems.append(f"{src} is not identical to {same}")
    return problems


def main():
    moves, deletions, bodies = plan()
    problems = check(moves, deletions)
    if problems:
        raise SystemExit("\n".join(problems))
    cmd = sys.argv[1] if len(sys.argv) > 1 else "plan"
    if cmd == "plan":
        counts = {}
        for _, dst in moves:
            counts[layer_of(dst)] = counts.get(layer_of(dst), 0) + 1
        print("| Today (under games/emberwake/assets/) | Goes to (section 3 path; folder's final layer) |")
        print("| --- | --- |")
        for src, dst in moves:
            print(f"| `{src}` | `{dst}` ({layer_of(dst)}) |")
        for src, same in deletions:
            print(f"| `{src}` | **delete** ({'identical to `' + same + '`' if same.startswith('art/') else same}) |")
        print(f"\nmoves {len(moves)} ({counts}), deletions {len(deletions)}, bodies {len(bodies)}")
    elif cmd == "apply":
        for src, dst in moves:
            os.makedirs(os.path.dirname(os.path.join(ROOT, dst)), exist_ok=True)
            subprocess.run(["git", "mv", src, dst], cwd=ROOT, check=True)
        for src, _ in deletions:
            subprocess.run(["git", "rm", "-q", src], cwd=ROOT, check=True)
        for key in bodies:
            cls, var = (key.split("/") + ["base"])[:2]
            with open(os.path.join(ROOT, "units", cls, var, "clips.json"), "w", encoding="utf-8", newline="\n") as f:
                f.write(json.dumps(BODY_CLIPS) + "\n")
        print(f"moved {len(moves)}, deleted {len(deletions)}, wrote {len(bodies)} clips.json")


if __name__ == "__main__":
    main()
