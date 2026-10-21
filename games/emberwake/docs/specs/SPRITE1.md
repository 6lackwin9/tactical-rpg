# SPRITE1: Pixel-art sprites and battle animation

Status update (2026-10-20): SPRITE1-L, SPRITE1-U and SPRITE1-A2 approved (user 2026-10-20). The status line below is the original.

Status update (2026-10-22): SPRITE1-A2, SPRITE1-L and SPRITE1-U done. The attack clip by skill and weapon is the
`SKILL_CLIPS` / `FIST_WEAPONS` table in `emberwake_gui/src/battle.rs` (a skill's own rig clip, else jab or kick with
fists, else `attack`); `walk` slides the token to its new tile (`walk_ms`, 0 = off); `idle_focus` is not played yet.

Status update (2026-10-22, user): SPRITE1-U drops the side-coloured disc behind the token: a flat foot ellipse in the
side's colour (ally, foe) under every unit, a 1px gold outline around the selected unit's sprite and a faint white
outline on hover. Battle first; the region map's sprites come with the playtest UI work (PLAYUI1).

Status update (2026-10-22): SPRITE1-S2 done: section 4 gives the sheet layout on ASSETORG2 (one sheet per body, no strips,
no manifest).

Status: SPRITE1-A (static idle sprites for the party) landed; SPRITE1-L, SPRITE1-U and SPRITE1-A2 below are a proposal awaiting the user's approval.
Depends on: the sprite rig in `tools/sprites/` (rig and class clips signed off for the brawler base 2026-10-18; other classes follow the same rules), `AssetRegistry` / `AssetCatalog` (S14).
Lane: asset, then logic, then logic (UI). Size M.
Touches: `tools/sprites/`, `assets/art/unit/`, `data/assets/manifest.json`, `scripts/art/{asset_catalog,asset_registry}.gd`, the battle token node in `scripts/ui/combat/` (UI/UX picks the file), tests.

## 1. What exists (SPRITE1-A, done)

Six party classes and the eight named party characters that use them show a pixel-art token and portrait (the idle frame). Goblins, bosses and NPCs still use the generated placeholders. Manifest `frames` must be 1 (`AssetCatalog.validate_entry`: "frames must be 1 until sprite strips are built").

## 2. Proposal

Animations the rig already produces per class: `idle`, `walk`, `hurt`, `death`, one attack clip per weapon family (warrior thrust and shield bash, ranger draw and release, brawler jab, haymaker, kick, sweep, priest smash, rogue double stab, mage bolt), and the stance clips `defend`, `idle_guarded`, `hurt_guarded`.

- **SPRITE1-A2 (asset):** export each clip as one horizontal strip `assets/art/unit/<key>/<clip>.png` (frames of 48x48, nearest filter; the unit's folder, ASSETORG1) plus per-frame durations in a small JSON next to it (`<key>/<clip>.json`), and one manifest row per strip.
- **SPRITE1-L (logic):** lift the `frames` rule: a strip row has `frames` = the strip's frame count; `AssetRegistry` gets `strip(key, clip)` returning the texture, frame size and durations (never null: falls back to the static token). Variants gain the clip names. Tests: manifest validation, strip slicing, fallback.
- **SPRITE1-U (UI/UX):** the battle token plays clips: `idle` always, `walk` while moving, `hurt` on a hit (`hurt_guarded` when the target defended), `death` on defeat, an attack clip chosen by the skill's kind and weapon family (table in the task), the `defend` clip and `idle_guarded` while a defensive stance is active. Respect the animation speed and instant settings (`FxSettings`). No engine or rules change; clips are presentation only.

## 3. Defaults and open questions

- Default order: A2, then L, then U. Static sprites stay until U lands.
- Goblin enemies, bosses and NPCs have no sprites yet; they would need new art (a separate approval).
- Two characters of one class look the same for now (a palette variant per character is a later art task).
- Skill effects layered on the clips (the "effects layer") and extra stances (taunt, hex, rally) are not part of this proposal.

## 4. Sheet layout (SPRITE1-S2, 2026-10-22, on ASSETORG2)

ASSETORG2 (approved 2026-10-20, built 2026-10-22) replaced the per-clip strips and the asset manifest of section 2: a
body is one sheet plus its clip list (`docs/specs/ASSETORG2.md` section 4.1). This section supersedes the file layout
of SPRITE1-A2 and the manifest work of SPRITE1-L; section 2's clips and SPRITE1-U's behaviour stand.

- **SPRITE1-A2 (asset):** `tools/sprites/export_units.py --clips all` (the writer exists since ASSETORG2-T) writes each
  class's clips into the class body:
  `kits/rpg/assets/units/<class>/base/sheet.png` (one row per clip in `clips.json` order, frames left to right, each
  48x48, nearest filter, transparent) and `clips.json` (`frame` [48, 48]; `palette` = the rig's colour slots, so a
  palette file can recolour the body; `clips` = `idle`, `walk`, `hurt`, `death`, the class's attack clips, `defend`,
  `idle_guarded`, `hurt_guarded`, each with the rig's per-frame `ms`; `loop` true for `idle`, `walk` and
  `idle_guarded`, false for the rest). The 104x104 one-frame placeholder sheets are replaced. The token is the `idle`
  clip's first frame (the engine scales it, ASSETORG2 4.1); there are no token files. Portraits no longer come from
  the rig (ART2 portraits stay).
- **SPRITE1-L (logic, engine):** the sheet format and its checks exist (`aegis_core::assets::Clips`,
  `aegis_ui::art::body_frame`, ASSETORG2-E). L adds playback: the frame of a clip at a time (the `ms` run, looping
  or holding the last frame) and the fallback to `idle` when a body lacks a clip. No manifest rows (the manifest is
  retired). Tests: frame at time for looping and one-shot clips, the idle fallback.
- **SPRITE1-U (UI/UX):** as section 2, reading clips from the unit's body (its palette applied).
- Order: ASSETORG2-T (the tools write the layered tree) before A2; then L, then U.
- Touches (replaces line 8): `tools/sprites/`, `kits/rpg/assets/units/<class>/base/`, `engine/crates/aegis_ui/src/art.rs`
  (playback), `games/emberwake/crates/emberwake_gui/src/battle.rs` (the token plays clips), tests.
