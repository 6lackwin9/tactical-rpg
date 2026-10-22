# Worldbook (2026-10-21): writes two review documents straight from the game data, so they never drift from it.
#   games/emberwake/docs/WORLDBUILDING.md  the worlds, regions, places, factions, characters, questlines, story fights,
#                                          dialogues, lore and the art each one uses
#   docs/RPG_KIT_CATALOG.md                every class, skill, passive, status, item family, item, terrain and unit,
#                                          sorted into reusable RPG kit material and Emberwake-only named content
# The hand-written parts (the story overview, the open threads) live in this file's INTRO text and are kept on every run.
# Usage: python tools/worldbook.py   (from the repo root)
import json, pathlib, re

ROOT = pathlib.Path(__file__).resolve().parents[1]
GAME = ROOT / "games/emberwake"
DATA = GAME / "data"
ASSETS = GAME / "assets"  # the game layer (ASSETORG2); kit and engine layers sit beside it


def entries(path):
    p = DATA / path
    if not p.exists():
        return []
    d = json.loads(p.read_text(encoding="utf-8"))
    return d.get("entries", d) if isinstance(d, dict) else d


def doc(path):
    p = DATA / path
    return json.loads(p.read_text(encoding="utf-8")) if p.exists() else {}


def cell(s):
    return str(s).replace("|", "/").replace("\n", " ").strip() or "-"


def plain(text, name=""):
    """Text with its template tokens readable: {name} becomes the entry's name, numbers and fields become N."""
    text = (text or "").replace("{name}", name or "it")
    return re.sub(r"\{[a-zA-Z_:0-9.]+\}", "N", text).strip()


def flags(fl):
    return ", ".join(f"`{f['key']}`" + ("" if f.get("value", True) is True else f"={f['value']}") for f in fl or []) or "-"


def art_of(rel):
    """Whether the game, kit or engine asset layer holds `rel` (ASSETORG2)."""
    return any((root / rel).exists() for root in (ASSETS, GAME.parent.parent / "kits" / "rpg" / "assets", GAME.parent.parent / "engine" / "assets"))


def world_ids():
    return sorted(p.name for p in (DATA / "worlds").iterdir() if p.is_dir())


# ---- WORLDBUILDING -------------------------------------------------------------------------------------------------

def world_section(wid, units):
    w = doc(f"worlds/{wid}/world.json")
    R = lambda f: entries(f"worlds/{wid}/{f}.json")
    regions, locs, facs, chars = R("regions"), R("locations"), R("factions"), R("characters")
    quests, lore, dlgs, encs, routes = R("quests"), R("lore"), R("dialogues"), R("encounters"), R("routes")
    shops, loot, disc, items = R("shops"), R("loot"), R("disciplines"), R("items")
    name = {c["id"]: c.get("name", c["id"]) for c in chars}
    out = [f"## {w.get('name', wid)} (`{wid}`)", "", f"*{w.get('tagline', '')}*", "", w.get("description", ""), ""]
    meta = []
    if w.get("extends"): meta.append(f"extends {', '.join(w['extends'])}")
    if w.get("connects"): meta.append(f"connects to {', '.join(w['connects'])}")
    if w.get("start_location"): meta.append(f"the game starts at `{w['start_location']}` with {', '.join(name.get(p, p) for p in w.get('start_party', []))}, {w.get('start_gold', 0)} gold, {w.get('start_supplies', 0)} supplies, a party of up to {w.get('party_max', '?')}")
    if meta: out += ["- " + m for m in meta] + [""]
    if regions:
        out += ["### Regions", "", "| Region | Climate | Held by | Description |", "| --- | --- | --- | --- |"]
        out += [f"| {cell(r['name'])} (`{r['id']}`) | {cell(r.get('climate'))} | {cell(r.get('faction'))} | {cell(r.get('description'))} |" for r in regions] + [""]
    if facs:
        out += ["### Factions", "", "| Faction | Motto | Relations | Description |", "| --- | --- | --- | --- |"]
        out += [f"| {cell(f['name'])} (`{f['id']}`) | {cell(f.get('motto'))} | {cell(', '.join(f'{k}: {v}' for k, v in f.get('relations', {}).items()))} | {cell(f.get('description'))} |" for f in facs] + [""]
    if locs:
        out += ["### Places", ""]
        for l in locs:
            npcs = [name.get(m.get("character"), m.get("character")) for m in l.get("markers", []) if m.get("kind") == "npc"]
            kinds = sorted({m.get("kind") for m in l.get("markers", [])} - {"npc"})
            bg = "yes" if art_of(f"backgrounds/location/{l['id']}.png") else "no"
            out += [f"**{l['name']}** (`{l['id']}`, {l.get('kind', '')}, region `{l.get('region', '')}`). {l.get('description', '')}", "",
                    f"- People here: {', '.join(npcs) or '-'}; markers: {', '.join(kinds) or '-'}; exits: {', '.join(e.get('to', '') for e in l.get('exits', [])) or '-'}",
                    f"- Fights here: {', '.join(e if isinstance(e, str) else str(e) for e in l.get('encounters', [])) or '-'}; rest place: {'yes' if l.get('rest') else 'no'}; map `{l.get('map', '')}`; background art: {bg}", ""]
    if routes:
        out += ["### Roads", "", "| Road | From | To | Time | Road fights | Scripted |", "| --- | --- | --- | --- | --- | --- |"]
        for r in routes:
            pool = ", ".join(f"{e[0]} ({e[1]})" for e in r.get("encounters", [])) or "-"
            scr = "; ".join(f"{s.get('encounter')} unless {flags(s.get('unless_flags'))}" for s in r.get("scripted", [])) or "-"
            out.append(f"| {cell(r['name'])} | {r['from']} | {r['to']} | {r.get('time', '')} | {cell(pool)} ({r.get('encounter_pct', 0)}%) | {cell(scr)} |")
        out.append("")
    if chars:
        out += ["### Characters", ""]
        for c in chars:
            u = units.get(c.get("base_unit"), {})
            role = {"party": "playable hero", "npc": "important NPC", "enemy": "enemy"}.get(c.get("role"), c.get("role", ""))
            body = c.get("art") or u.get("art") or u.get("class_id", "?")
            own = next((f"portraits/{f}/{c['id']}.png" for f in ("heroes", "npcs", "elites", "bosses") if art_of(f"portraits/{f}/{c['id']}.png")), "")
            portrait = f"unique (`{own}`)" if own else "shared (the body's common portrait)"
            line = [f"**{c['name']}**" + (f", {c['title']}" if c.get("title") else "") + f" (`{c['id']}`)", "",
                    f"- {role}; {u.get('class_id', '?')} class on unit `{c.get('base_unit', '')}` (tier {u.get('tier', '?')}, level {u.get('level', '?')}); faction `{c.get('faction') or '-'}`; home `{c.get('home') or '-'}`"]
            if c.get("background"): line.append(f"- Background: {c['background']}")
            if c.get("personality"): line.append(f"- Personality: {c['personality']}")
            extra = []
            if c.get("disciplines"): extra.append("disciplines " + ", ".join(c["disciplines"]))
            if c.get("named_items"): extra.append("named items " + ", ".join(c["named_items"]))
            if c.get("dialogue"): extra.append(f"dialogue `{c['dialogue']}`")
            if c.get("gone_flags"): extra.append("leaves the story on " + flags(c["gone_flags"]))
            if extra: line.append("- " + "; ".join(extra))
            line.append(f"- Art: body `{body}` ({'its own' if c.get('art') else 'the base unit' + chr(39) + 's'}), portrait {portrait}")
            out += line + [""]
    if quests:
        out += ["### Questlines", ""]
        for q in quests:
            steps = "; ".join(f"{i + 1}. {s.get('text', '')}" + (f" (fight `{s['encounter']}`)" if s.get("encounter") else "") for i, s in enumerate(q.get("steps", [])))
            out += [f"**{q['name']}** (`{q['id']}`, given by {name.get(q.get('giver'), q.get('giver') or '-')}, region `{q.get('region', '')}`)", "",
                    q.get("description", ""), "",
                    f"- Steps: {steps or '-'}",
                    f"- Needs: quests {', '.join(q.get('requires', [])) or '-'}; flags {flags(q.get('requires_flags'))}; starts {'by itself' if q.get('auto_start') else 'from the giver'}",
                    f"- Reward: {q.get('reward', '') or '-'}; sets {flags(q.get('complete_flags'))}", ""]
    if encs:
        out += ["### Story fights", "", "| Fight | Where | With the party | Against | Objective | Notes |", "| --- | --- | --- | --- | --- | --- |"]
        for e in encs:
            notes = []
            # PHASEFOLD: the reinforcement wave is an "arrival" phase (rows not yet re-saved still hold wave_units)
            arrival = [p for p in e.get("phases", []) if isinstance(p, dict) and (p.get("trigger") or {}).get("kind") == "arrival"]
            boss = len(e.get("phases", [])) - len(arrival)
            if boss: notes.append(f"{boss} phases")
            if e.get("wave_units") or arrival: notes.append("reinforcements")
            if e.get("morale"): notes.append("morale")
            if e.get("guests"): notes.append("guests " + ", ".join(name.get(g, g) for g in e["guests"]))
            obj = e.get("objective") or "defeat"
            out.append(f"| {cell(e['name'])} (`{e['id']}`) | {e.get('location') or e.get('zone', '')} | {cell(', '.join(name.get(p, p) for p in e.get('party', [])))} | {cell(', '.join(name.get(x, x) for x in e.get('enemies', [])))} | {cell(obj + (': ' + e['objective_text'] if e.get('objective_text') else ''))} | {cell('; '.join(notes))} |")
        out += [""]
        for e in encs:
            if e.get("intro"): out += [f"- *{e['name']}*: {e['intro']}"]
        out.append("")
    if dlgs:
        out += ["### Dialogues", ""]
        for d in dlgs:
            first = next((n for n in d.get("nodes", []) if n.get("id") == d.get("start")), (d.get("nodes") or [{}])[0])
            out += [f"**{d['name']}** (`{d['id']}`, {name.get(d.get('speaker'), d.get('speaker', ''))}, {len(d.get('nodes', []))} nodes): {first.get('text', '')}", ""]
            for n in d.get("nodes", []):
                if n is first: continue
                out.append(f"- `{n['id']}`: {n.get('text', '')}")
            out.append("")
    if lore:
        out += ["### Lore", ""] + [f"**{l['name']}** ({l.get('category', '')}): {l.get('text', '')}" + "\n" for l in lore]
    if disc:
        out += ["### Disciplines", "", "| Discipline | Faction | Passives | Description |", "| --- | --- | --- | --- |"]
        out += [f"| {cell(d['name'])} | {cell(d.get('faction'))} | {cell(', '.join(d.get('passives', [])))} | {cell(d.get('description'))} |" for d in disc] + [""]
    if shops:
        out += ["### Shops", ""]
        for s in shops:
            out.append(f"- **{s['name']}** ({name.get(s.get('keeper'), s.get('keeper'))}): " + ", ".join(i["item"] for i in s.get("stock", [])))
        out.append("")
    if loot or items:
        out += ["### Named rewards and items", ""]
        out += [f"- **{l['name']}** ({l.get('reward_kind')} `{l.get('ref')}`, {l.get('rarity', '')}): {l.get('description', '')}" for l in loot]
        out += [f"- **{i['name']}** (item `{i['id']}`): {plain(i.get('description'))}" for i in items]
        out.append("")
    return out


INTRO = """# Worldbuilding: the worlds of Emberwake as they stand

Status: review copy for the user (2026-10-21), written from the game data by `tools/worldbook.py`; the story overview
and the open threads below are hand-written. Proposed changes go to the design lane, then into the data through
`aegis content`; re-run the tool afterwards. Art that no longer fits the story is revisited once the story is set.

## The story so far

Emberwake opens in **the Ashlands**, a burnt frontier on the edge of an old empire. Every dry season, the Long Dry,
the wells of the cinder wastes fail and the **Ashen Horde**, goblin warbands from the wastes, march on the passes. The
**Ember Wardens**, a poor but fiercely loyal border order ("Keep one fire lit."), hold **Warden Keep** and the
**Cinder Pass** against them.

The player starts at Warden Keep as **Bryn Hale**, a recruit whose father's post at Emberfall burned, in a party of up
to three. The main questline, given by Captain **Mara Voss**:

1. **Hold the Courtyard** (starts by itself): the Long Dry has come early; drive Skarn Ironjaw and Vex the Cutpurse
   out of the **Ruined Courtyard** while the keep's gates are shored up. Muster Sergeant **Brask Toller**'s duty roll
   opens: the first recruits join.
2. **Watch at the Cinder Gate**: the warband left scouts in the pass; Skarn and Vex ambush the party at the **Cinder
   Gate**, with reinforcements.
3. **The Warden's Oath** (from the scout **Tev Quill**): once the pass is clear, the party swears the Wardens' oath at
   the keep; more of Toller's roll opens (the recruits: Tev, Sella, Dagny, Alda, Kit, Oren).
4. **Steel for the Gate** (starts by itself after the oath): the Cinder Gate will not hold another Long Dry on timber.
   The party takes the south road into the **Iron Reaches** to **Slag Gate**, where the Forge Guild's **Orsa Tallow**
   sells the Wardens steel at cost, and brings it home on her nephew **Wenn Tallow**'s wagon (the Horde raids it on
   the road). **The choice:** the steel goes to the gate (the gorge is barred) or to arms (Hest's counter sells
   broadswords and claymores).
5. **Keep One Fire Lit**: **Grakka Ashmaw**, chieftain of the **Ashen Horde**, has united the warbands (the rumour of
   the Ashmaw banner: one skull per warband). The party finds the **Ashmaw Camp** and breaks him. If the steel barred
   the gate, his flankers never arrive and he fights alone, throwing off his crown of rusted nails at half strength.

Side work: **Sweep the Ruins** for Hest the Quartermaster once the Wardens trust the party (reputation 30). Road
fights on the old road and the gorge; the Horde's named fighters (Skarn, Vex, Nok, Gruk, Bolg and the patrol
Warchief) are the faces of its warbands. **Grakka at the fire** is a parley before the last fight.

Beyond the Ashlands:

- **The Iron Reaches** (connected, not inherited): the mining highland of the **Forge Guild** ("Strike true."), allies
  of the Wardens. One place (Slag Gate), Orsa Tallow, and her own quest (Iron for the Embers), hidden for now.
- **Tidewrack Coast** (inherits the Ashlands and re-skins it, "the same war, wetter"): its founders were Wardens who
  followed the smoke to the sea; the order became the **Harbour Guard** ("Light the lamp."), the Horde's descendants
  sail as **Reef Raiders**. Two places, Lampwarden **Ilse Marrow** (Mara's counterpart, plays exactly like her),
  the scout **Pell Driftwood**, five raiders led by **Captain Dredge**, and one fight (Clash at Wreck Strand).

## Open threads and things to settle

- Man or woman is not in the data for Tev, Sella, Alda, Kit, Hest, the Horde Warchief, Nok and Vex; the portraits use
  best guesses (Tev, Kit and the Warchief men; the others women). Settle them in the characters' text.
- Orsa's own quest stays hidden until the Iron Reaches campaign (decision "Orsa quest stays hidden").
- The deserter as a second story choice (DESERT1) and the Warden Keep upgrades (KEEP1) are approved ideas without a
  spec yet; both change this story.
- Rewards are moving to per-enemy (REWARD1): the named lieutenants and Grakka will carry key loot.
- The generic portraits (Warden soldier, archer, brawler, acolyte, cutpurse, hedge-mage, five Horde goblins) are
  faceless on purpose; any of them can be promoted to a named character as the world fills out.
- The Iron Reaches and Tidewrack characters (Orsa, Ilse, Pell, the Reef Raiders) have no portraits yet: the one world
  first rule kept the art session to the Ashlands.
- The "Arena party" column of the story fights is the encounter's authored party (Mara and Tev) for the Scene
  Runner's arena; in the campaign the player's own party fights.

## Contents
"""


def write_worldbuilding():
    units = {u["id"]: u for u in entries("catalog/units.json")}
    body = []
    toc = []
    for wid in world_ids():
        w = doc(f"worlds/{wid}/world.json")
        toc.append(f"- [{w.get('name', wid)}](#{re.sub(r'[^a-z0-9 -]', '', (w.get('name', wid) + ' ' + wid).lower()).replace(' ', '-')})")
        body += world_section(wid, units)
    # the story templates every world's quests use
    tmpl = entries("quests/quests.json")
    body += ["## Quest templates (shared by every world)", "", "| Template | Objectives | Default reward |", "| --- | --- | --- |"]
    body += [f"| {cell(t['name'])} (`{t['id']}`) | {cell(', '.join(o.get('kind', '') for o in t.get('objectives', [])))} | {cell(json.dumps(t.get('rewards', {})))} |" for t in tmpl] + [""]
    text = INTRO + "\n" + "\n".join(toc) + "\n\n" + "\n".join(body)
    text = re.sub(r"\n{3,}", "\n\n", text).rstrip() + "\n"
    (GAME / "docs/WORLDBUILDING.md").write_text(text, encoding="utf-8", newline="\n")
    return len(text.split("\n"))


# ---- RPG KIT CATALOG -----------------------------------------------------------------------------------------------

# Proper names of Emberwake's worlds, matched as whole words (so "Shield Bash" and the ash-wood "Ash Wand" stay kit).
NAMED = re.compile(r"\b(ember\w*|ashen|ashlands|ashmaw|grakka|warden\w*|slag|cinder|tidewrack|iron reaches)\b", re.I)

STATUSES = [("stagger", "debuff", "Staggered", "loses a block"), ("stun", "debuff", "Stunned", "loses blocks"), ("root", "debuff", "Rooted", "cannot move"),
            ("bleed", "debuff", "Bleeding", "damage over time"), ("weaken", "debuff", "Weakened", "less outgoing power"), ("slow", "debuff", "Slowed", "fewer move tiles"),
            ("expose", "debuff", "Exposed", "less armour"), ("poison", "debuff", "Poisoned", "stacking damage over time"), ("burn", "debuff", "Burning", "damage over time"),
            ("haste", "buff", "Hasted", "more move tiles"), ("taunt", "buff", "Taunting", "foes in reach must target this unit"), ("riposte", "buff", "Riposte", "counters adjacent melee"),
            ("fortify", "buff", "Fortified", "more armour"), ("regen", "buff", "Regenerating", "heals each round"), ("empower", "buff", "Empowered", "more outgoing power")]

KIT_INTRO = """# RPG kit catalog: what Emberwake has that any RPG can reuse

Status: review copy for the user (2026-10-21), written from the game data by `tools/worldbook.py`. ASSETORG2 builds the
RPG kit layer (`kits/rpg/`) for assets; this list is the content and art behind it. **Kit** means generic: a class,
skill, item family, status or unit template any fantasy RPG could ship. **Game** means named or bound to Emberwake's
worlds (Ember-, Ash-, Grakka, Wardens). A kit row's art is reusable with it; game rows stay in `games/emberwake/`.
Code follows SPLIT1 principle 2 (extract when a second game needs it); this catalog only marks what would move.

Counts: {counts}.
"""


def kit_layer(id_, name=""):
    words = lambda s: (s or "").replace("_", " ").replace("'", "").replace("-", " ")
    return "game" if NAMED.search(words(id_)) or NAMED.search(words(name)) else "kit"


def write_kit():
    classes, skills = entries("catalog/classes.json"), entries("catalog/skills.json")
    passives, fams = entries("catalog/passives.json"), entries("catalog/item_families.json")
    items, terrain, units = entries("catalog/items.json"), entries("catalog/terrain.json"), entries("catalog/units.json")
    world_items = [(w, i) for w in world_ids() for i in entries(f"worlds/{w}/items.json")]
    owner = {}
    for c in classes:
        for s, lvl in c.get("skill_path", {}).items():
            owner.setdefault(s, []).append(f"{c['id']} L{lvl}")
    out = []
    out += ["## Classes", "", "| Class | Role | Weapons (proficiency) | Skill path | Art | Layer |", "| --- | --- | --- | --- | --- | --- |"]
    for c in classes:
        prof = ", ".join(f"{k} {v.get('level')}" for k, v in c.get("proficiency", {}).items())
        out.append(f"| {cell(c['name'])} | {cell(c.get('description'))} | {cell(prof)} | {len(c.get('skill_path', {}))} skills | body and clips on the sprite rig; generic portrait | kit |")
    out += ["", "## Skills", "", "| Skill | Kind | Classes (unlock level) | What it does | Icon | Layer |", "| --- | --- | --- | --- | --- | --- |"]
    for s in skills:
        out.append(f"| {cell(s['name'])} (`{s['id']}`) | {s.get('kind', '')} | {cell(', '.join(owner.get(s['id'], [])) or 'everyone')} | {cell(plain(s.get('description'), s.get('name')))} | {'yes' if art_of(f"skills/{s['id']}.png") else ('kind icon' if art_of(f"skills/kinds/{s.get('kind', '').lower()}.png") else 'no')} | {kit_layer(s['id'], s.get('name'))} |")
    out += ["", "Fallback icons per kind (from the ART2 sheets, in `art_src/skills_4.png`): any attack, ranged attack, spell, support, defence, disengage, status.", ""]
    out += ["## Statuses", "", "| Status | Type | Shown as | Effect | Icon | Layer |", "| --- | --- | --- | --- | --- | --- |"]
    out += [f"| `{s}` | {t} | {d} | {e} | {'yes' if art_of(f'statuses/{s}.png') else 'no'} | kit |" for s, t, d, e in STATUSES] + [""]
    out += ["## Passives", "", "| Passive | Effect | Layer |", "| --- | --- | --- |"]
    out += [f"| {cell(p['name'])} (`{p['id']}`) | {cell(plain(p.get('description'), p.get('name')))} | {kit_layer(p['id'], p.get('name'))} |" for p in passives] + [""]
    out += ["## Item families", "", "| Family | Hands | Icon | Layer |", "| --- | --- | --- | --- |"]
    for f in fams:
        out.append(f"| {cell(f['name'])} (`{f['id']}`) | {cell(', '.join(f.get('hands') or []) or '-')} | {'yes' if art_of(f"items/{f.get('art') or f['id']}/base.png") else 'no'} | kit |")
    out += ["", "## Items", "", "| Item | Family | What it is | Layer |", "| --- | --- | --- | --- |"]
    out += [f"| {cell(i['name'])} (`{i['id']}`) | {i.get('family') or '-'} | {cell(plain(i.get('description'), i.get('name'))[:140])} | {kit_layer(i['id'], i.get('name'))} |" for i in items]
    out += [f"| {cell(i['name'])} (`{i['id']}`, world {w}) | {i.get('family') or '-'} | {cell(plain(i.get('description'), i.get('name'))[:140])} | game |" for w, i in world_items] + [""]
    out += ["## Terrain", "", "| Terrain | Move cost | Cover | Hazard | Tile art | Layer |", "| --- | --- | --- | --- | --- | --- |"]
    for t in terrain:
        hz = f"{t['hazard_status']} ({t.get('hazard_on', '')})" if t.get("hazard_status") else "-"
        out.append(f"| `{t['id']}` | {t.get('move_cost')} | {t.get('cover_guard')} | {hz} | {'yes' if art_of(f"tilesets/common/{t['id']}/v0.png") else 'no'} | kit (engine baseline for the common set) |")
    out += ["", "## Unit templates", "", "| Unit | Side | Class | Tier | Level | Layer |", "| --- | --- | --- | --- | --- | --- |"]
    out += [f"| {cell(u['name'])} (`{u['id']}`) | {u.get('side')} | {u.get('class_id')} | {u.get('tier', '')} | {u.get('level')} | {kit_layer(u['id'], u.get('name'))} |" for u in units] + [""]
    counts = (f"{len(classes)} classes, {len(skills)} skills, {len(STATUSES)} statuses, {len(passives)} passives, "
              f"{len(fams)} item families, {len(items) + len(world_items)} items, {len(terrain)} terrain types, {len(units)} unit templates")
    text = KIT_INTRO.format(counts=counts) + "\n" + "\n".join(out)
    text = re.sub(r"\n{3,}", "\n\n", text).rstrip() + "\n"
    (ROOT / "docs/RPG_KIT_CATALOG.md").write_text(text, encoding="utf-8", newline="\n")
    return len(text.split("\n"))


if __name__ == "__main__":
    print("WORLDBUILDING.md", write_worldbuilding(), "lines")
    print("RPG_KIT_CATALOG.md", write_kit(), "lines")
