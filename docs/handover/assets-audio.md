# Assets & Audio handover

Written 2026-10-18 16:35 UTC by the eighth Assets & Audio session (wind-down; the project moves to "interactive" mode, see `docs/workflow/cloud.md` and the modes skill). Read this first. Nothing is in flight: no open branch work, no running builds. A fresh session starts only on a request from Tushar or the coordinator and does one thing at a time.

## Queue (all approved, none started)

All three are data tasks and share the ashlands encounter rows, so run one at a time:

1. PACE1-A: Grakka two-act data on both stands (`docs/specs/PACE1.md`; schema on develop since 39daec9; field notes in README "Fight pacing schema (PACE1-L stage 1)"; `enrage` phase effect, `or_round` trigger, encounter `pace`; no start_status data).
2. PURSE1-A: Ashlands purse data on 9 encounters plus the new `warchiefs_axe` loot on `ash_patrol` (`docs/specs/PURSE1.md`; schema on develop since 02980d3; README "Encounter purse schema").
3. ASHPACE1: `ash_scouts` one scout fewer unless MORALE1-B already lifts the solo warrior to 90+ (MORALE1-A landed 69c82e0, so check MORALE1-B first; the task row says a spec is needed).

## Sprite art (heavy review session, base signed off; rest is a background task)

Tushar's rule (2026-10-18 12:00): art is one heavy review session; once the base is signed off the rest runs in the background. The brawler base was signed off 2026-10-18 (11:45, "all recs" 12:14). Preferences are in `docs/preferences/assets-world.md`.

- Generators: `rig.py` and `classes.py` in `/mnt/project-files/sprite-sample/` (procedural Python/PIL, 48x48 frames facing right). Copy both to a scratch dir and run `python3 classes.py cls <class>`; output is per-clip GIFs, a sheet and a preview. Outputs sit in `/mnt/project-files/sprite-sample/classes/` (including `brawler_set.gif`/`.png`).
- All six classes (warrior, ranger, brawler, priest, rogue, mage) already have idle, walk, attack, hurt, death plus stance clips (defend, guarded idle, guarded hit) from the earlier rounds. Only the brawler got the signed-off polish (L-shaped alternating fists, relaxed walk, face-height jab, limp death).
- Not started: the background pass over the other five classes (apply the same polish rules: hunch in defensive stances, death from idle, limp when unarmed, readable arm colours), then one set shown to Tushar for a final look.
- Not started and not approved: in-game wiring (frame format, animation-per-skill-kind table, effect and stance overlays, a UI/UX logic task). It needs a one-card spec and Tushar's approval before any repo work. Also offered, not approved: cyclone kick spin, more stances (taunt, hex, rally), a skill-effects layer.
- Style rules: pixel retro Style A (48 px, ~16 colours, full dark outline); class sets outfit and palette, weapon type sets animation; defensive stances hunch; warrior attack = shield lower, guard and bash = hunched behind the shield.

## Landed on `develop` (this session)

- ITEMPOW1-A (02c6bf4): Healing Draught heals 90, Fire Bomb 45, Ashlands start gold 100; the gear-equip pouch test and a dojo assert now read the item values. Seed `data/seeds/itempow1.gd`.
- ASSETS-PREFS (b7e5e1e): sprite notes in `docs/preferences/assets-world.md`.
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
