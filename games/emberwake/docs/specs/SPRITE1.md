# SPRITE1: Pixel-art sprites and battle animation

Status update (2026-10-20): SPRITE1-L, SPRITE1-U and SPRITE1-A2 approved (user 2026-10-20). The status line below is the original.

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
