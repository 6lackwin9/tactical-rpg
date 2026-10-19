# Assets & Audio handover

Written 2026-10-18 16:35 UTC by the eighth Assets & Audio session, updated 18:55 UTC after SPRITE1-A (wind-down; the project is in "interactive" mode, see `docs/workflow/cloud.md` and the modes skill). Read this first. Nothing is in flight: no open branch work, no running builds. A fresh session starts only on a request from Tushar or the coordinator and does one thing at a time.

## Queue (all approved, none started)

All three are data tasks and share the ashlands encounter rows, so run one at a time:

1. PACE1-A: Grakka two-act data on both stands (`docs/specs/PACE1.md`; schema on develop since 39daec9; field notes in README "Fight pacing schema (PACE1-L stage 1)"; `enrage` phase effect, `or_round` trigger, encounter `pace`; no start_status data).
2. PURSE1-A: Ashlands purse data on 9 encounters plus the new `warchiefs_axe` loot on `ash_patrol` (`docs/specs/PURSE1.md`; schema on develop since 02980d3; README "Encounter purse schema").
3. ASHPACE1: `ash_scouts` one scout fewer unless MORALE1-B already lifts the solo warrior to 90+ (MORALE1-A landed 69c82e0, so check MORALE1-B first; the task row says a spec is needed).

## Sprite art

Tushar's rule (2026-10-18 12:00): art is one heavy review session; once the base is signed off the rest runs in the background. Brawler base signed off 2026-10-18. Preferences: `docs/preferences/assets-world.md`.

- In the build (SPRITE1-A): the six party classes and eight named party characters (Mara, Bryn, Tev, Sella, Dagny, Alda, Kit, Oren) use pixel-art tokens and portraits from the idle frame. Source: `tools/sprites/` (`rig.py`, `classes.py`, `export_units.py`; run `python3 tools/sprites/export_units.py`, then `godot --headless --import`, then seed `data/seeds/sprite1_a.gd`). `tools/gen_art.py` skips those keys. Goblins, bosses and NPCs are still generated placeholders. Tushar said he will ask for tweaks.
- Proposed, not approved, not started: animation playback in battle. Spec `docs/specs/SPRITE1.md`, TASKS rows SPRITE1-A2 (export strips), SPRITE1-L (asset pipeline strips, lifts `frames` = 1), SPRITE1-U (UI/UX playback). Tushar to approve; he checks and releases to `main` himself (2026-10-18 18:51).
- Not started: the polish pass for the other five classes (same rules as the brawler: hunch in defensive stances, death from idle, limp when unarmed, readable arm colours), goblin sprites, palette variants for two characters of one class.
- Older generator copies and GIF outputs: `/mnt/project-files/sprite-sample/` (the repo copy in `tools/sprites/` is the source now).
- Style rules: pixel retro Style A (48 px, ~16 colours, full dark outline); class sets outfit and palette, weapon type sets animation; defensive stances hunch; warrior attack = shield lower, guard and bash = hunched behind the shield.

## Landed on `develop` (this session)

- ITEMPOW1-A (02c6bf4): Healing Draught heals 90, Fire Bomb 45, Ashlands start gold 100; the gear-equip pouch test and a dojo assert now read the item values. Seed `data/seeds/itempow1.gd`.
- ASSETS-PREFS (b7e5e1e): sprite notes in `docs/preferences/assets-world.md`.
- SPRITE1-A (this commit's squash on develop): pixel-art tokens and portraits for the party, rig in `tools/sprites/`, spec SPRITE1.
- MORALE1-A (69c82e0): morale on `ash_scouts` and `ash_raiders` (leader `ash_shaman`). Seed `data/seeds/morale1.gd`. Balance's MORALE1-B is unblocked. Not checked in a real battle (the spec's "shaman falls first, fight won with the line logged").

## Standing rules and notes

- Build only when the coordinator gives the rotating slot; hand it back with a one-line message when done. Mode follows `docs/workflow/planner-ops` "Current mode" (economy until 18:45 UTC on 2026-10-18, then see the modes skill; interactive replaces parallel chains).
- Landing: `tools/fc.sh gate auto` (per-system gate) plus `tools/fc.sh docs`; catalog data runs every system. Squash commit with `Review:`, `Gate:`, `AgentMap:`, `Done:`, `Timing:` (with `type=`, `tokens=`, `model=sonnet` while the Sonnet trial runs) and `Blocked-by:`. After landing, message Platform "landed <id> <sha> model=<m>"; Merge & Release no longer needs it (cutback f1db72d).
- Never touch the `engine` branch (push, merge, rebase, delete) until Tushar approves. Never force-push develop or main.
- Asset-lane work goes to the `asset-builder` agent, review to `change-reviewer`; the thread writes only planner docs.
- `ContentCatalog.save_all()` and `WorldCatalog.save_world()` backfill new schema defaults into rows (MORALE1-A touched all nine ashlands encounters this way); `data/catalog/encounters.json` gets renormalised by catalog seeds, so `git checkout` it after a seed.
- Shipped rows: upsert inside `begin_system_edit()`/`end_system_edit()`; run seeds with `tools/fc.sh run <seed>`; commit the seed's `.uid`.
- Gate in the background with `SUITE_TIMEOUT=1200`; the arena suite is slow.
- Branches safe to delete at the next cleanup: `claude/project-thread-f7wktp` (landed), plus the older ones listed in git history of this file.
- Live peers (session ids change on rotation; ask the coordinator): coordinator session_01XdQe1nMY9pgmknr8u2XHAz, Platform session_01F3J8BGWSSBbqckdqifYoJF, Gameplay session_01448UbhXvTzeBA1Brw4buLy.
