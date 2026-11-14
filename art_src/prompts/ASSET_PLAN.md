# Art asset plan: what is open and in what order (sweep 2026-11-15)

Asked for by Tushar ("go over the existing art inbox and potential other assets that need to be built, and create a
plan and prompt list"). Sources: the Art/UI Inbox (`ART_UI_REQUESTS.md`), Design's queue (`docs/READY.md`, `PR5-49`
to `PR5-56`), the approved props list, and a hash sweep of `games/emberwake/assets` and `kits/rpg/assets` for files
that are byte-identical copies of another (placeholders nobody listed).

**Nothing here runs without the user's go-ahead on the model and the image count** (`P-UI-06`, PRINCIPLES 0.5).
Routes: Flow or the Drive web queue first, local ComfyUI for tiles; every result is viewed against its prompt and
reference before it is kept. Placeholders are replaced in place: the same path, size and format.

**Portraits from the web queue are placeholders (Tushar, 2026-11-15).** Final characters are fleshed out in Flow as
Flow characters, because a Flow character carries over to animations and story CGs. So batches 1, 5, 6 and 10 below
aim at a readable placeholder in place (one review pass, no polishing rounds), and each named character's final art
is a later Flow set: portrait, three-view body, then animations and CGs (`ART_UI_STUDIO.md` 3.6, 3.7). The founders
(batch 5) already have Flow portraits and bodies for four of the six on Drive, so their final art starts there.

## 1. Batches, in the order proposed

| # | Batch | Images | Prompts | Blocked by | Why this order |
| --- | --- | --- | --- | --- | --- |
| 1 | **Act 1 cast portraits**: Grendel (AI-06), Calcified Amalgam and Bailiff Enforcer (AI-08), Varka and Ivo (AI-05), Corvo and Skerrit (new) | 7 portraits, 2 takes each | [ai_portraits.md](ai_portraits.md) | nothing (`AO-07` for the two new ones) | Faces the player meets in Act 1 show a wrong or fallback portrait today; smallest batch with the most visible gain |
| 2 | **Small icons**: 6 keepsakes (AI-07), recipe badge and 4 HUD chips (AI-07, AI-11), 3 family and 2 status icons (new) | 3 sheets | [ai_icons.md](ai_icons.md), [sunk_ward_icons.md](sunk_ward_icons.md) `families_1` | nothing | Three sheets clear two inbox rows and five unlisted copies |
| 3 | **M5 Slag Gate CG** (AI-10) | 1 scene, 2 takes | [ai10_cg.md](ai10_cg.md) | nothing | The Act 1 climax vignette shows the hearth backdrop today |
| 4 | **Story music** (AI-03): `story_opening`, `prologue_tension`, `story_escape`, `story_goodbye` | 4 tracks | `art_src/music/prompts/story_tracks.txt` | nothing (Flow Music, the user's side) | Three of the four keys are already asked for by vignettes and play nothing |
| 5 | **Founders, Day 1** (AI-04): 6 portraits, then 6 expression sheets | 6 + 6 | [ai04_founders.md](ai04_founders.md), [ai_faces.md](ai_faces.md) | nothing | The six leads; larger and needs the closest review (identity lock), so after the quick wins |
| 6 | **Hub NPCs** (AI-05 rest): Sabina, Talia, their face sheets | 2 + 2 | [ai_portraits.md](ai_portraits.md), `ART3_PROMPTS.md`, `PORTRAITFX1_FACES.md` | nothing | Both have usable art today (shared with Varka and Ivo, who get their own in batch 1) |
| 7 | **Skill icons**: 52 that copy another skill's icon | 4 sheets | [sunk_ward_icons.md](sunk_ward_icons.md) `skills_1`-`skills_4` | `AO-07` (1): which are meant to share | Wide but shallow: readable in battle today, only not distinct |
| 8 | **Sunk Ward props**: 84 new arts, the nine region maps | 8 sheets + 8 single buildings | [sunk_ward_props.md](sunk_ward_props.md) S4-S11, S9, S10 | `PR5-54`, `PR5-55` (the maps import on placeholders first) | Largest batch; worth running once the maps are in the game, so each sheet is checked in place |
| 9 | **Sunk Ward floors and walls**: 9 grounds, `floorboards`, 3 boundary sets, 2 interior wall sets | about 25 tiles | `REGIONGROUND1.md` section 7, [sunk_ward_walls.md](sunk_ward_walls.md) | the R1 tuning (Design held it, `AO-04` (1)); `PR5-54` | Tiles are tuned as a set against the props, so after batch 8's first sheets |
| 10 | **Acts 2 and 3 bosses** (AI-06 rest): Jessop, Ruthven, Lazar, Ulric, Ysolde, Othemar, and their face sheets | 6 + 6 | [ai_portraits.md](ai_portraits.md), [ai_faces.md](ai_faces.md) | nothing | Not seen before Act 2; no game data for those acts yet |
| 11 | **Gear rework looks and icons** (AI-01): the one-handed polearm with a shield, new families and off-hands | rig work + 1 icon sheet | written when `PR5-51b` reports its placeholder paths | `PR5-51b` | On hold by Design |

Totals for batches 1 to 3 (the proposed first run): 14 portrait takes, 3 icon sheets, 2 scene takes.

## 2. What the hash sweep found that no list had

- **Portraits**: `bailiff_corvo` and `skerrit_bone_wright` have dialogue and no portrait file (batch 1). The AI-08
  placeholders (`calcified_amalgam`, `bailiff_enforcer`) are not on disk either. `ivo.png` is Talia's picture and
  `widow_varka.png` is Sabina's.
- **Item family icons**: `polearm/base.png` is the kit's battle axe, `censer/base.png` the mace, `bone_saw/base.png`
  the shortsword (batch 2).
- **Status icons**: `steadfast.png` is the kit's fortify; `stealth.png` is the disengage kind icon (batch 2).
- **Skill icons**: 52 of the game's 100 skill icons are copies of another (batch 7).
- **Battle bodies**: `warrior/2h_polearm` is the two-handed axe sheet (AI-01, now waiting on the gear rework).
- **Vignettes**: `vg_m5_gate` uses the hearth stage (AI-10). The three debates and the Interlude surge share
  `hearth.png`, which is correct (they happen at the hearth).
- **Music**: `prologue_tension` is a declared key that nothing asks for yet (`AO-07`).
- Not placeholders: each founder's portrait equals its own `neutral` face, and the UI frame pairs (`button` = `tab`)
  are shared on purpose.

## 3. Not in this plan (parked by the user or by Design)

- **Story and VN backdrops per place, travel and transition animations**: after the exploration maps (Tushar,
  2026-11-14).
- **Combat maps** per place: after the exploration maps; no spec yet.
- **Full-body and keyframe sprites** for story actors: the code-rig animation stands in (Tushar, 2026-11-12).
- **Acts 2 and 3 region maps and props**: no location data yet (`ACTS23`).
- **Rank-and-file enemy portraits** (footmen, crossbowmen, dredgers, beasts): they use their class or family portrait
  by design.
