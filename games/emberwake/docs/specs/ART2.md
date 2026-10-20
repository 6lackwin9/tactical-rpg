# ART2: art review of 2026-10-20 (the user's marks)

Status: approved (user 2026-10-20: every asset marked Keep, Redo or Drop on the Emberwake Art Ledger, with notes)
Depends on: SPRITE1 rig (`tools/sprites/`), ASSETORG2 (where the new files land); art tools in Python, portable to Rust (decision in `docs/decisions/workflow.md`)
Lane: asset (art); the rig and icon generators are tools

The art session reviewed all 296 shipped images and the sprite rig's 54 animation clips (regenerated from `tools/sprites/classes.py`; the GIFs were never committed). This file keeps the marks so later sessions do not depend on the review page. Rules the user set for units: the token is the class body in a variant for that character (goblins: a goblin variant on the class rig); the portrait is the character's own, more detailed picture (unique for heroes, important NPCs, leaders, bosses; shared for mobs). Keep is not final: the user judges every kept asset again in the game.

## Counts

| Area | Keep | Redo | Drop |
| --- | --- | --- | --- |
| Animations | 53 | 1 | 0 |
| Units (token and portrait marks) | 5 | 67 | 0 |
| Skills | 0 | 57 | 0 |
| Items | 15 | 8 | 2 |
| Statuses | 8 | 6 | 0 |
| Battle tiles | 23 | 4 | 0 |
| Interface and maps | 44 | 6 | 0 |

## The user's notes

| Asset | Mark | Note |
| --- | --- | --- |
| anim `priest_idle` | redo | Mace should be facing up and not the ground |
| unit `dummy portrait` | redo | detailing in line with rest of the portrait |
| unit `dummy token` | redo | switch to sprite model |
| unit `grakka_ashmaw token` | redo | for goblin create goblin variant on the class rig, same applies for all goblins |
| unit `priest token` | redo | facing up and not the ground |
| skill `aegis` | redo | redo all skill icons as action icons |
| item `axe` | redo | if its two handed use double sided axe |
| item `bomb` | keep | split based on the bomb type, it can use the curent base for variants |
| item `dagger` | redo | have a clear disctinction between dagger and shortsword |
| item `elixir` | keep | split based on the elixir type, it can use the curent base for variants |
| item `hammer` | redo | the top and stick is not aligned |
| item `knuckles` | drop | drop candidate as we have moved brawler to unarmed |
| item `misc` | redo | not sure where it is used |
| item `potion` | keep | split based on the potion type, it can use the curent base for variants |
| item `shortsword` | redo | have a clear disctinction between dagger and shortsword |
| item `staff` | redo | have a clear distinction between wand and staff instead of just the top shape |
| item `wand` | redo | ave a clear distinction between wand and staff instead of just the top shape |
| status `haste` | redo | use up arrows next to clock |
| status `root` | redo | use vines instead |
| status `slow` | redo | add down arrows next to clock |
| status `stagger` | redo | use question mark instead |
| status `stun` | redo | use spiral instead |
| status `taunt` | redo | use shout image |
| tile `bridge` | redo | have wariants in bridge, like slighlty damaged in differet places and keep only 1 clean) |
| tile `grass` | redo | have variant in grass |
| tile `overlay_npc` | redo | distinguish between different types of NPC (vendor NPC which contribute to game system vs Quest giver vs Flavour NPC) |
| tile `tree` | redo | have variants in tress |
| ui `ashlands_title` | redo | make the text more stylistic |
| ui `ashlands_world_map` | redo | the paths need to be refined and not a straight line |
| ui `main_menu` | redo | castle seems off |
| ui `pin_wild` | redo | more variants for trees |
| ui `ruined_courtyard` | redo | background is fine but the ruins need work |
| ui `warden_keep` | redo | the flag and gate need more detailing |

## Redo and drop lists

- **Animations** redo: `priest_idle`
- **Units** redo: `alda_wren portrait`, `alda_wren token`, `archer portrait`, `archer token`, `ash_archer portrait`, `ash_archer token`, `ash_berserker portrait`, `ash_berserker token`, `ash_brawler portrait`, `ash_brawler token`, `ash_shaman portrait`, `ash_shaman token`, `ash_skirmisher portrait`, `ash_skirmisher token`, `ash_warchief portrait`, `ash_warchief token`, `berserker portrait`, `berserker token`, `bolg_redhand portrait`, `bolg_redhand token`, `brask_toller portrait`, `brask_toller token`, `brawler portrait`, `bryn_hale portrait`, `bryn_hale token`, `dagny_coal portrait`, `dagny_coal token`, `dummy portrait`, `dummy token`, `goblin_brawler portrait`, `goblin_brawler token`, `grakka_ashmaw portrait`, `grakka_ashmaw token`, `gruk_smokecaller portrait`, `gruk_smokecaller token`, `hest_the_quartermaster portrait`, `hest_the_quartermaster token`, `kit_vane portrait`, `kit_vane token`, `mage portrait`, `mara_voss portrait`, `mara_voss token`, `nok_flamebow portrait`, `nok_flamebow token`, `oren_dusk portrait`, `oren_dusk token`, `priest portrait`, `priest token`, `ranger portrait`, `rogue portrait`, `scout portrait`, `scout token`, `sella_marsh portrait`, `sella_marsh token`, `shaman portrait`, `shaman token`, `skarn_ironjaw portrait`, `skarn_ironjaw token`, `skirmisher portrait`, `skirmisher token`, `tev_quill portrait`, `tev_quill token`, `vex_cutpurse portrait`, `vex_cutpurse token`, `warrior portrait`, `wenn_tallow portrait`, `wenn_tallow token`
- **Items** redo: `axe`, `dagger`, `hammer`, `misc`, `shortsword`, `staff`, `titan_warhammer`, `wand`; drop: `knuckles`, `weapon`
- **Statuses** redo: `haste`, `root`, `slow`, `stagger`, `stun`, `taunt`
- **Battle tiles** redo: `bridge`, `grass`, `overlay_npc`, `tree`
- **Interface and maps** redo: `ashlands_title`, `ashlands_world_map`, `main_menu`, `pin_wild`, `ruined_courtyard`, `warden_keep`
- **Skills** redo: all 57, as action icons (the user's note on `aegis`).

## Order

1. Rig fixes: the priest's mace points up (idle, token); a goblin variant on each class rig (every goblin token: ash goblins, named goblins, Grakka); the dummy on the sprite model.
2. Character token variants: a class-body variant per named character (outfit, colours, headgear).
3. Portrait style test on one hero at two or three detail levels; the user picks; then every unique portrait and the shared mob portraits.
4. Skill action icons (57), then the base icon per skill kind as the fallback.
5. Items (the notes above; potions, elixirs and bombs split by type on the current bases), statuses (the notes above).
6. Tiles (variants for grass, trees, bridge; NPC overlays by type) and the six interface images.

Every step is shown on a review page before anything lands in the game; new files land in the ASSETORG2 layout.

## Portraits (2026-10-20)

- Route: unique portraits are generated source art (Gemini 3 Pro Image through `tools/portraits/generate.py`, Mara Voss
  as the style reference) run through `tools/portraits/pixelize.py` at the 96 px grid, 4 shades per colour family.
- The cast of 18 is accepted as it stands (user 2026-10-20: "use the current set for now"): Mara Voss (five
  variants), Bryn Hale, Tev Quill, Sella Marsh, Dagny Coal, Alda Wren, Kit Vane, Oren Dusk, Brask Toller, Hest the
  Quartermaster, Wenn Tallow, Grakka Ashmaw, the Horde Warchief, Bolg Redhand, Skarn Ironjaw, Gruk Smokecaller, Nok
  Flamebow, Vex the Cutpurse. Characters may be repurposed once the world is fleshed out. Sources: `art_src/` (LFS).
- Generic portraits: one generated base per mob kind (six human class defaults, five goblin types, the dummy) with
  faces no named character has; further mob variants are recolours of those bases (`tools/portraits/recolor.py`),
  never of a named character's portrait.
- Picks of 2026-10-20 (the Art Picks page): every named and generic portrait kept; Mara Voss's main portrait is the
  prompt image (S), the four variants kept beside it; recolours are good for mob variants; the priest's mace pose B3.
- Icons next: skill and item icon sheets through the API (user 2026-10-20: "go ahead with icons generation").
- Icons (2026-10-21): the 80 icons of the five sheets (57 skills, 7 fallbacks per kind and attack type, 16 item
  family and weapon icons) are kept as the first-pass baseline. All generated art is a baseline for an alpha build that
  other people play; the user gives asset feedback once the full game runs, so no further polish until then.
