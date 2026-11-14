# ASSETORG4: Generic Kit Asset Library & Modular Portrait Overlay FX

Status: **approved** (user 2026-11-14, `AO-04(4)` & `FB-227`).
Depends on: `ASSETORG2` (built), `PR5-29b` (`LAYER-ENG-FX`, built), `PR5-47` (`MAP-HOOKS1`, built).
Layers: engine (`aegis_core`, `aegis_ui` modular portrait FX toggle), kit (`kits/rpg/assets/**`, `rpgkit_content`), game (`games/emberwake/assets/**`, `emberwake_gui/src/portraits.rs`, `.face.json` sidecars).

## Goal

Promote every atomic, lore-free visual and audio asset that passes the **Rename & Composition Test** into `kits/rpg/assets/` so a second RPG built on `kits/rpg/` inherits a complete, ready-to-compose library of tilesets, props, landmarks, character visual templates, icons, UI chips, backdrops, and audio without referencing Emberwake, while keeping world-bound compositions and lore bindings in `games/<id>/`. In the same pass, finish migrating `.face.json` sidecars to `PR5-29b`'s generic primitive names and make portrait idle/corruption overlay FX a **modular opt-in component** per game.

## 1. The Rename & Composition Test (`P-UI-04`)

> **Rule**: If an individual asset can be reused in another dark-fantasy / tactical RPG simply by giving it a lore-free catalog/file key (and optional style/tag metadata), it belongs in **`kits/rpg/assets/`**. If an asset is a painted multi-place composition of one specific campaign geography, a bespoke story CG scene, game branding, or the data glue binding kit visuals to named lore characters/places, it stays in **`games/<id>/`**.

### 1.1 Moves to `kits/rpg/assets/` (Lore-Free Keys + Optional `catalog_tags.json` Metadata)

| Group | Kit Path | Notes |
| --- | --- | --- |
| **Tilesets & Grounds** | `kits/rpg/assets/tilesets/<style>/ground/<id>/v0.png` (and wall/boundary sets) | Material/style-named sets (e.g. `basalt_waste`, `soot_masonry`, `common`) rather than campaign proper nouns |
| **Map & Stage Props** | `kits/rpg/assets/props/<style>/<art>/v<n>.png` | Reusable architectural, industrial, camp, street, and interior furniture props (`common`, `industrial_brick`, `tenement`, `interior`) |
| **World-Map Landmarks** | `kits/rpg/assets/map/landmarks/<key>.png` | Reusable landmark icons/cuts (+ style variants) resolved via `landmarks.<id>.art` in world-map sidecar or `location.landmark` (`game -> kit` before `<local_id>.png`) |
| **Character Visual Templates** | `kits/rpg/assets/units/<archetype>/<variant>/` + `kits/rpg/assets/portraits/templates/<key>.png`, `faces/<key>/*.png`, `full_body/<key>.png` | Complete visual bundle (unit sprite sheet + `clips.json` + palettes + base portrait + expression frames + `.face.json` + full-body VN sprite) under generic archetype keys; `characters.json` / `units.json` bind named lore characters to a template key (game-layer overrides at the same path still win) |
| **Item, Skill & Status Icons** | `kits/rpg/assets/items/<family>/`, `skills/`, `statuses/` | Generic gear, consumable, salvage, skill, and status icons; only strictly named one-off character relics (e.g. `keepsake_<founder>.png`) stay in `games/emberwake/assets/` unless given generic relic keys |
| **UI Chips, Badges & Markers** | `kits/rpg/assets/ui/{hud,markers,pins,intents,badges,chips,services}/` | Generic HUD resource/clock/alert icons (`scrap`, `tallow`/`lantern`, `alert`, `doom_clock` under generic keys mapped by `world.json`/`meters.json`), map markers, and service badges |
| **Battle Backdrops & Audio** | `kits/rpg/assets/backgrounds/battle/`, `audio/{sfx,music}/` | Generic zone battle backdrops, combat/UI SFX, and mood music (`combat`, `boss`, `town`, `shop`, `danger`, `travel`) |

### 1.2 Stays in `games/emberwake/`

- **World-bound multi-asset scenes**: Painted campaign act maps (`assets/backgrounds/world_map/<region>.png` + `<region>.json` layout sidecars), bespoke narrative CGs (`assets/backgrounds/story/vg_*.png`), game crest/icon/title art, and `ui/theme.json` overrides.
- **Game data & bindings**: `data/maps/maps.json`, `data/worlds/<id>/**` (`characters.json` binding lore IDs to kit visual templates, `locations.json`, `encounters.json`, `quests.json`), and bespoke game-only keepsake/story relics.
- **Retired**: `rpgkit_content::art::location_background` (`backgrounds/location/<id>.png`) remains retired in favour of composed `REGIONGROUND1` + `PROPS1` exploration maps (`AO-04(4)`).

## 2. `.face.json` Primitive Migration & Modular Portrait Overlay FX (`FB-227`)

1. **Sidecar Migration**: Migrate all shipped `.face.json` files (and `tools/portraits/face.py` output defaults) from legacy Emberwake-era overlay aliases (`veins`, `eye_glow`, `ash`, `smoke`) to `PR5-29b`'s canonical engine primitives (`lines`, `point_glow`, `particles_rise`, `drift`), keeping the read-time aliases in `aegis_core::assets::parse_face` for external/archived fixtures.
2. **Modular Opt-In Portrait Overlay FX**:
   - Make `PortraitFx` idle/state overlay rendering (`glow`, `lines`, `point_glow`, `particles_rise`, `drift`) a **modular opt-in component** controlled by game configuration (`game.json` `portrait_overlays: bool`, default `true` when present or enabled by the game's `Faces` binding, plus a player-facing `Settings` / accessibility toggle alongside `reduce_motion`).
   - Base portrait life (blink, breath, expression frames) works cleanly with `portrait_overlays: false`, so a non-corruption RPG mounting `kits/rpg/` gets expressive character portraits out of the box without supernatural/corruption particle overlays unless opted in.

## 3. Acceptance Criteria

1. `tools/fc.sh layers` (`G3` lore-word scan included), `aegis content check`, `the_assets_follow_the_layered_tree`, and all workspace tests pass with `0` errors and `0` visual regressions in `aegis scenario shots`.
2. Every migrated `.face.json` uses canonical primitive names (`lines`, `point_glow`, `particles_rise`, `drift`) and passes `face_fixtures` + `tools/portraits/test_face.py`.
3. Disabling `portrait_overlays` cleanly suppresses overlay primitives across talk boxes, vignettes, party cards, and timeline icons while preserving base portraits and expression swaps.
