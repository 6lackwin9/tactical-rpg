# ASSETORG2: asset layers and reuse groups

Status: approved (user 2026-10-20: the direction, then all eight open questions with the recommended answers)
Depends on: ASSETORG1 (done 2026-10-20, `0c73dea`: one folder per thing, `art/<category>/<id>/<variant>.png`)
Lane: logic, then asset (the data rows), then logic (tools)
Touches: `engine/crates/aegis_core/src/assets.rs` (new), `engine/crates/aegis_ui/src/{art,audio}.rs`,
`engine/crates/aegis_cli` (manifest `kit`), `engine/assets/**`, `kits/rpg/**` (new), `games/emberwake/assets/**`,
`games/emberwake/game.json`, `games/emberwake/schema.json`, `games/emberwake/crates/emberwake_content/src/{art.rs,source.rs,check/*}`,
`games/emberwake/crates/emberwake_gui/src/{main,battle,region,worldmap,gallery}.rs`, `games/emberwake/data/` (art fields;
`assets/manifest.json` removed), `tools/gen_art.py`, `tools/sprites/`

Replaces decision "Art folder per entity (user 2026-10-20)" with "Asset layers and reuse groups (user 2026-10-20)"
(`docs/decisions/workflow.md`) and the per-entity layout ASSETORG1 built.

## 1. Goal

Assets are grouped by what works together and what is reused, and every lookup falls back through three layers: the
game (named assets only), the RPG kit (reusable gear, skill, status and unit art) and the engine (a generic baseline),
then a placeholder drawn by code. A unit's body is its class body (a variant such as goblin inside the class), tweaked
per character by a palette; the token is the body's idle frame; portraits are separate art, unique for heroes,
important NPCs, elites and bosses and shared by mobs. Items and skills fall back to a family or kind base icon until
they get unique art.

**User decision on the kit layer.** SPLIT1 principle 2 ("extract, do not invent") says a kit is built when a second
game appears. On 2026-10-20 the user directed an RPG kit layer for assets now. This spec builds the kit for assets
only: `kits/rpg/` holds files and a manifest, no crates. Code still follows principle 2: the RPG lookup chains
(section 4) live in the game until a second game needs them; the engine gets only the generic layered loader.

## 2. Layers and resolution

| Layer | Root | Holds |
| --- | --- | --- |
| Game | `games/<id>/assets/` | named assets only: named characters' bodies and portraits, named items, skill families, world tilesets, backgrounds, world music |
| Kit | `kits/<kit>/assets/` | reusable RPG art: class bodies and variants, common portraits, item family bases, skill kind icons, statuses, map markers, RPG UI chips, combat sounds, mood music |
| Engine | `engine/assets/` | the baseline any game can use: the UI font, the common terrain tileset, interface sounds |

- The `art/` level of today's tree goes: categories sit directly under each root (`units/`, `items/`, ...), audio
  under `audio/`.
- `game.json` field `kit` (exists, today `""`) names the kit: `"rpg"`. Allowed: `""` (no kit) or the id of a folder
  `kits/<kit>/` holding `kit.json`. `aegis content manifest` refuses any other value.
- `kits/rpg/kit.json` (new, written through `aegis content manifest --kit` like `game.json`):
  `{"id": "rpg", "title": "RPG kit", "version": "0.1.0", "engine": ">=0.1"}`. `id` equals the folder name
  (snake_case, 2-24 characters); `version` is semver; `engine` a semver range.
- Roots are found from the game folder: the workspace root is the first ancestor of `games/<id>/` holding
  `engine/assets/`; the kit root is `<workspace>/kits/<kit>/assets/`. A missing kit folder is a start notice, not a
  crash (lookups skip the layer).
- **Resolution order** for a relative path `rel`: game, kit, engine; the first layer holding the file wins. When no
  candidate of a chain exists, the caller draws its placeholder shape (today's behaviour on a missing file).
- A game may override any kit or engine file by putting a file at the same `rel` in its own root.

## 3. Folder tree

```text
engine/assets/
  fonts/OpenSans-SemiBold.ttf, OpenSans-OFL.txt
  tilesets/common/<tile>/v<n>.png            generic terrain, v0..v(n-1)
  audio/sfx/<key>.wav                        interface sounds
  ui/                                        reserved for toolkit glyphs (empty today)
kits/rpg/
  kit.json  README.md
  assets/
    units/<class>/<variant>/sheet.png        one sheet per body (variant "base" = the class default)
    units/<class>/<variant>/clips.json       frame size, clips, palette slots
    units/<class>/<variant>/palettes/<id>.json
    portraits/common/<class>_<variant>.png   shared portraits (mobs, minor NPCs)
    items/<family>/base.png                  family default
    items/<family>/<icon>.png                generic variants (greatsword, amulet, bomb)
    items/any/base.png, items/any/<item_type>.png
    skills/kinds/<kind>.png                  one per skill kind
    statuses/<status>.png
    map/markers/<marker>.png
    ui/{pins,badges,chips,intents}/<id>.png
    audio/sfx/<key>.wav, audio/music/<key>.ogg
games/<id>/assets/
  units/<class>/<variant>/...                named bodies; palettes of named characters
  portraits/{heroes,npcs,elites,bosses}/<id>.png   (common/ only to override the kit)
  portraits/full_body/<id>.png               full-body story stage sprites (FB-140)
  items/<family>/<icon>.png                  named item icons
  skills/<family>.png                        skill family icons
  tilesets/<set>/<tile>/v<n>.png             world tilesets (set = world id)
  backgrounds/{menu,title,world_map,location}/<id>.png
  audio/music/<key>.ogg, audio/sfx/<key>.wav
```

Layout test (replaces ASSETORG1's depth test): every file under each root matches one pattern above (plus
`README.md` files); anything else fails the test with its path.

## 4. Lookup rules

### 4.1 Bodies, palettes, tokens

- **Body key**: `<class>` (means `<class>/base`) or `<class>/<variant>`; `<class>` is a folder under `units/`, not
  necessarily a data class (`dummy` is a prop body).
- A unit's body: character `art`, else its base unit's `art`, else its class's `art`, else the class id. A body key
  that is missing in every layer falls back to `<class>/base`, then the placeholder.
- **Sheet** (engine format, `aegis_core::assets::Clips`): `sheet.png` holds one row per clip in `clips.json` order,
  frames left to right, every frame `frame` pixels. `clips.json`:

```json
{"frame": [48, 48], "palette": {"main": "#a83028", "accent": "#a83028", "trim": "#e0b43c"},
 "clips": [{"id": "idle", "frames": 8, "ms": [120, 120, 120, 120, 120, 120, 120, 120], "loop": true}]}
```

- Validation: `frame` w and h 8..256; `clips` non-empty, ids unique snake_case, `idle` present; `frames` 1..64; `ms`
  has `frames` entries, each 16..2000; `loop` bool (default true); the sheet is at least (max frames x w) by
  (clips x h); `palette` (optional, default `{}`) maps slot names to `#rrggbb`.
- **Token** = clip `idle`, frame 0, scaled by k = floor(size / max(w, h)) with nearest filter (downsampled when
  k = 0), centred horizontally and bottom-aligned in the size square; the team rim is still drawn by code. There are
  no token files.
- **Palette**: character or unit field `palette` names `<body>/palettes/<id>.json`, searched game, kit, engine:
  `{"slots": {"main": "#5a3a30"}}`. Each named slot's colour in the body's `clips.json` palette is replaced by the
  new colour on every pixel of exactly that colour (slots sharing one colour change together). At most 16 slots.

### 4.2 Portraits

- **Portrait tier**: field `portrait_tier` when set, else auto. Auto for a character: role `party` = hero, `npc` =
  npc, `enemy` = by its base unit's tier (`boss` = boss, `elite` = elite, else common). Auto for a catalog unit:
  tier `boss` = boss, `elite` = elite, else common (catalog hero units are class templates).
- **Unique tiers** (hero, npc, elite, boss): `portraits/<heroes|npcs|elites|bosses>/<id>.png` (id = character id,
  else unit id), then the common chain.
- **Common chain**: `portraits/common/<class>_<variant>.png` of the unit's body, then `<class>_base.png`, then the
  placeholder. Mobs and minor NPCs (friendly or hostile) share these by class and race.

### 4.3 Items

For an item with `icon_id` K and family F (`folder` = the family's `art`, else F):

1. K set: `items/<folder>/<K>.png`.
2. `items/<folder>/base.png`.
3. `items/any/<item_type lowercase>.png`, then `items/any/base.png`.
4. Placeholder.

### 4.4 Skills and statuses

- Skill of family F and kind K: `skills/<F>.png`, then `skills/kinds/<K lowercase>.png`, then the placeholder.
- Status S: `statuses/<S>.png`, then the placeholder.

### 4.5 Tiles, backgrounds, UI and audio

- Tile T of set chain C: for each set in C, the first layer holding `tilesets/<set>/<T>/v0.png` gives the tile; its
  variant count n = the run of `v0..v(n-1)` files in that layer; variant = (x x 7 + y x 13) mod n (Euclidean).
- Set chains: the region map uses the world lineage (child first) then `common`; battles use `common` only until a map
  names a set (open question 5). Region map characters: `.` floor, `#` wall, `~` shallow_water (others: colour).
- Backgrounds: world map `backgrounds/world_map/<world>.png` along the lineage; title `backgrounds/title/<world>.png`;
  menu `backgrounds/menu/main_menu.png`; location `backgrounds/location/<location>.png`.
- UI: pins `ui/pins/<location kind>.png`, intent chips `ui/intents/<kind>.png` (`idle` uses `rest`), badges
  `ui/badges/<id>.png`, chips `ui/chips/<id>.png`.
- Audio keeps its relative paths (`sfx/hit.wav`, `music/town.ogg`) under `audio/`, resolved across the layers;
  MusicDirector's candidate order (MUSIC1) is unchanged.

## 5. Schema contract (logic lane, schema-first: yes)

| Kind | Field | Type | Default | Rule |
| --- | --- | --- | --- | --- |
| classes | `art` (exists) | enum, `options_from: "body"` | class id | a body key in the index; values today (`warrior` ...) stay valid |
| units | `art` (exists) | enum, `options_from: "body_inherit"` | `""` | `""` inherits the class; else a body key |
| characters (world) | `art` (exists) | enum, `options_from: "body_inherit"` | `""` | `""` inherits the base unit; else a body key |
| units, characters | `palette` (new) | string | `""` | `""` none; else an id with a file in the body's `palettes/` |
| units, characters | `portrait_tier` (new) | enum `["", "hero", "npc", "elite", "boss", "common"]` | `""` | `""` = auto (4.2) |
| item_families | `art` (exists) | string | `""` | `""` = the family id; else a folder under `items/` |
| items | `icon_id` (exists) | string | `""` | a file `items/<folder>/<icon_id>.png` in a layer |

- `options_from: "body"` lists the index's bodies (`<class>` for base, `<class>/<variant>` otherwise) across the layers;
  `"body_inherit"` adds `""`. They replace `"art"` and `"art_inherit"`.
- `art`, `palette` and `portrait_tier` are identity fields (`"identity": true`, SPLIT1 6).
- Save version: unchanged (art is not saved).

## 6. Content checks (emberwake_content, on `aegis_core::assets`)

| Id | Level | Rule | Message |
| --- | --- | --- | --- |
| A1 | error | a class, unit or character `art` that is not a body | `Art '<key>' isn't a body (units/<class>/<variant>/ in the game, kit or engine).` |
| A2 | error | `icon_id` not found by 4.3 step 1 | `Icon '<key>' isn't in items/<folder>/ (game, kit or engine).` |
| A3 | warning | unique tier without its portrait file | `<name> (<tier>) has no portrait at portraits/<folder>/<id>.png; it shows the common portrait <class>_<variant>.` |
| A4 | warning | `palette` file missing, or a slot not in the body's palette | `Palette '<id>' of <name>: <problem>.` |
| A5 | warning | a body used by a common-tier unit or character without `portraits/common/<class>_<variant>.png` | `Body '<key>' has no common portrait; <name> shows <class>_base.` |
| A6 | error | a `clips.json` or sheet that fails 4.1 validation | `Body '<key>': <problem>.` |

Expected after migration: no A1, A2, A4, A6; A3 ten times (the eight heroes, Gruk (if open question 1 is taken),
`goblin_warlord`); A5 none.

## 7. The asset manifest

`games/emberwake/data/assets/manifest.json` (Godot `res://` rows, 320 entries, not read by the engine) is retired:
the folders are the truth. Sizes come from the PNG header, frames, durations and palette slots from `clips.json`,
tile variants from the files, music candidates from the files (as today). The rows' `origin` lock and `note` text
go; credits and generator notes live in each root's `README.md`. `source::art_keys` and the template key lists go
with it. Godot's copy stays in `legacy/data/`.

## 8. Tools

- `tools/gen_art.py --layer engine|kit|game [group ...]` writes placeholders into the right root by section 3 (default
  all layers); it never writes a path that a rig export owns (`tools/sprites/` bodies). Bodies are written as a
  one-frame `sheet.png` (the 104x104 figure) plus `clips.json` (`frame` [104, 104], one `idle` clip, `ms` [1000]).
  Portraits are written separately to `portraits/`. Skill kind icons use the kind's glyph and colour: attack slash,
  defend shield, focus eye, support plus, disengage chev_up, rest moon, move boot, target swap, status dizzy.
- `tools/sprites/export_units.py` writes `kits/rpg/assets/units/<class>/base/sheet.png` and `clips.json` (with the
  rig's `pal` as `palette` slots) and no portraits. Until SPRITE1-A2 the sheet holds `idle` only; SPRITE1-A2 adds the
  other clips to the same sheet.
- Acceptance: running both tools into an empty folder reproduces every generated file of the three roots byte for
  byte (so the move in section 9 is reproducible).

## 8b. Tools as built (ASSETORG2-T, 2026-10-22)

- `gen_art.py` and `export_units.py` take `--root DIR` (a workspace root to write into); `gen_art.py` takes `--layer`.
- `gen_art.py` keeps an existing file unless `--overwrite` (the ART2 portraits and icons from `art_src/` would
  otherwise be replaced by placeholders); `export_units.py` always rewrites the six class bodies.
- Palette slots in a class body's `clips.json` = the rig's base colours present in the sheet (no `_lo` / `_hi` shades).
- Acceptance as built (user 2026-10-22: paths and pixel sizes, not bytes): an empty-folder regeneration gives the same
  307 paths (285 PNGs, 22 `clips.json`) and the same sizes; 180 of the 285 PNGs are pixel-identical; the rest are the
  ART2 art (104) and the priest body (1: the rig's raised mace, ART2 pick B3, never re-exported; SPRITE1-A2 re-exports
  it).

## 9. Move table (296 PNGs on develop today; E = `engine/assets/`, K = `kits/rpg/assets/`, G = `games/emberwake/assets/`)

Paths on the left are under `games/emberwake/assets/art/`. Moves are `git mv` (history kept).

| Today | Count | Goes to |
| --- | --- | --- |
| `unit/{warrior,ranger,brawler,priest,rogue,mage}/token.png` | 6 | K `units/<class>/base/sheet.png` + `clips.json` (frame 104) |
| `unit/<those six>/portrait.png` | 6 | K `portraits/common/<class>_base.png` |
| `unit/goblin_brawler`, `skirmisher`, `scout`, `archer`, `shaman`, `berserker` tokens | 6 | K `units/brawler/goblin`, `rogue/goblin`, `rogue/goblin_scout`, `ranger/goblin`, `priest/goblin`, `brawler/goblin_berserker` (sheet + clips) |
| the same six portraits | 6 | K `portraits/common/<class>_<variant>.png` (same mapping) |
| `unit/dummy/{token,portrait}.png` | 2 | K `units/dummy/base/` sheet; `portraits/common/dummy_base.png` |
| `unit/{hest_the_quartermaster,brask_toller,wenn_tallow}` | 6 | G `units/warrior/hest_the_quartermaster`, `warrior/brask_toller`, `ranger/wenn_tallow`; portraits G `portraits/npcs/<id>.png` |
| `unit/{skarn_ironjaw,vex_cutpurse,nok_flamebow,bolg_redhand,ash_warchief}` | 10 | G `units/brawler/skarn_ironjaw`, `rogue/vex_cutpurse`, `ranger/nok_flamebow`, `brawler/bolg_redhand`, `brawler/ash_warchief`; portraits G `portraits/elites/<id>.png` |
| `unit/grakka_ashmaw` | 2 | G `units/warrior/grakka_ashmaw`; G `portraits/bosses/grakka_ashmaw.png` |
| `unit/{mara_voss,bryn_hale,tev_quill,sella_marsh,kit_vane,oren_dusk,alda_wren,dagny_coal}` | 16 | **delete**: pixel-identical to their class |
| `unit/gruk_smokecaller` | 2 | **delete**: pixel-identical to `shaman` |
| `unit/ash_{brawler,skirmisher,archer,shaman,berserker}` | 10 | **delete**: near-copies of the goblin variants (open question 3) |
| `item/{sword,axe,hammer,dagger,bow,staff,wand,tome,shield,plate,leather}/icon.png` | 11 | K `items/<same>/base.png` |
| `item/knuckles`, `item/robe` | 2 | K `items/fist/base.png`, `items/cloth/base.png` |
| `item/greatsword`, `item/shortsword` | 2 | K `items/sword/greatsword.png`, `items/sword/shortsword.png` |
| `item/ring`, `item/amulet` | 2 | K `items/accessory/base.png`, `items/accessory/amulet.png` |
| `item/potion`, `item/elixir`, `item/bomb` | 3 | K `items/consumable/base.png`, `consumable/elixir.png`, `consumable/bomb.png` |
| `item/weapon`, `item/misc` | 2 | K `items/any/weapon.png`, `items/any/base.png` |
| `item/titan_warhammer`, `item/runed_staff`, `item/ember_brigandine` | 3 | G `items/hammer/titan_warhammer.png`, `items/staff/runed_staff.png`, `items/leather/ember_brigandine.png` |
| `skill/{strike,guard,focus,mend,backflip,rest,move,switch_target,staggered}` | 9 | K `skills/kinds/{attack,defend,focus,support,disengage,rest,move,target,status}.png` |
| `skill/{reckless_swing,shield_wall,greater_mend,rejuvenate,leap_back}` | 5 | **delete**: pixel-identical to their kind icon |
| the other 43 `skill/<family>/icon.png` | 43 | G `skills/<family>.png` |
| `status/<id>/icon.png` | 14 | K `statuses/<id>.png` |
| `tile/<terrain>/v<n>.png` (17 terrains x 4) | 68 | E `tilesets/common/<terrain>/v<n>.png` |
| `tile/overlay_<marker>/v0.png` | 10 | K `map/markers/<marker>.png` |
| `ui/region_<tile>_<n>/icon.png` | 18 | G `tilesets/ashlands/<id>/v<n>.png`: ash_floor = `floor`, basalt_wall = `wall`, water = `shallow_water`; road, lava, rubble, scrub, door keep their names |
| `ui/{main_menu,ashlands_title,ashlands_world_map}/background.png` | 3 | G `backgrounds/menu/main_menu.png`, `title/ashlands.png`, `world_map/ashlands.png` |
| `ui/{warden_keep,ruined_courtyard,cinder_gate,ashmaw_camp}/background.png` | 4 | G `backgrounds/location/<id>.png` |
| `ui/pin_<kind>`, `ui/intent_<kind>` | 20 | K `ui/pins/<kind>.png`, `ui/intents/<kind>.png` |
| `ui/badge_{quest,ready,foresight}`, `ui/chip_{slay,guest}` | 5 | K `ui/badges/<id>.png`, `ui/chips/<id>.png` |

Totals: engine 68, kit 106, game 89, deleted 33 (all checked with md5 on develop `a295018`). Pixel-identical skill
pairs that stay (each family keeps its placeholder until unique art): aimed_shot/deadeye, arcane_bolt/smite,
assassinate/hex, barbed_arrow/volley, bleeding_cut/envenom, body_blow/haymaker, charge_in/lunge,
hamstring/pinning_shot, lacerate/pounce, rend_armor/sunder, arc_lance/offhand_jab/quick_stab (open question 6).

Audio and fonts: `audio/sfx/ui_{click,back,confirm,error}.wav` to E `audio/sfx/`; the other 10 sfx to K
`audio/sfx/`; music `combat`, `boss`, `town`, `shop`, `danger`, `travel` to K `audio/music/`; `ashlands`,
`iron_reaches`, `tidewrack`, `menu_theme` stay in G. `games/emberwake/assets/fonts/` (the Godot web build's fallback
fonts, unused by the engine; `legacy/assets/fonts/` keeps a copy) is deleted.

## 10. Data rows (asset lane, through `aegis content`, after the schema slice)

| Kind | Id | Field | Value |
| --- | --- | --- | --- |
| units | goblin_brawler / goblin_skirmisher / goblin_archer / goblin_shaman / goblin_berserker / fleeing_scout | art | `brawler/goblin` / `rogue/goblin` / `ranger/goblin` / `priest/goblin` / `brawler/goblin_berserker` / `rogue/goblin_scout` |
| units | ambush_berserker, goblin_warlord | art | `brawler/goblin_berserker` |
| units | training_dummy | art | `dummy` (unchanged) |
| characters | the eight party characters, gruk_smokecaller, ash_brawler, ash_skirmisher, ash_archer, ash_shaman, ash_berserker | art | `""` (inherit) |
| characters | hest_the_quartermaster, brask_toller, wenn_tallow | art | `warrior/hest_the_quartermaster`, `warrior/brask_toller`, `ranger/wenn_tallow` |
| characters | skarn_ironjaw, vex_cutpurse, nok_flamebow, bolg_redhand, ash_warchief, grakka_ashmaw | art | `brawler/skarn_ironjaw`, `rogue/vex_cutpurse`, `ranger/nok_flamebow`, `brawler/bolg_redhand`, `brawler/ash_warchief`, `warrior/grakka_ashmaw` |
| characters | skarn_ironjaw, vex_cutpurse, nok_flamebow, gruk_smokecaller, bolg_redhand, ash_warchief | portrait_tier | `elite` (open question 1) |
| items | wpn_claymore, acc_swift_amulet, con_focus_elixir, con_fire_bomb | icon_id | `greatsword`, `amulet`, `elixir`, `bomb` |
| item_families | mace, orb, crossbow | art | `hammer`, `tome`, `bow` (the Godot icon rule, until those families get bases) |

These keep every icon, token and portrait the player sees today (the Godot item icon rule becomes data).

## 11. Tasks and order

| Id | Title | Layer | Lane | Depends on |
| --- | --- | --- | --- | --- |
| ASSETORG2-E | Layered asset loader: `aegis_core::assets` (roots from game.json and kit.json, `find`, `variants`, `list`, `Clips` and palette parse and validation), `aegis_ui::art` over the layers (image, sheet frame, token from idle, palette swap, tiles by file count), `aegis_ui::audio` over the layers, manifest `kit` and `kit.json` in `aegis content manifest`; tests | engine | logic | - |
| ASSETORG2-G | Game on the new tree: chains of section 4 in `emberwake_content/src/art.rs`, call sites (battle, region, worldmap, gallery), schema fields and options of section 5, checks A1-A6, the manifest and `art_keys` retired, parity retirements (section 12), layout test; every PNG moved inside `games/emberwake/assets/` to its section 3 path (kit- and engine-bound ones included, still in the game root), deletions, one-frame `clips.json` files | game | logic | ASSETORG2-E |
| ASSETORG2-D | Data rows of section 10 | game | asset | ASSETORG2-G schema slice; runs on G's branch and lands with G (G's checks fail on today's art values) |
| ASSETORG2-K | The RPG kit: `kits/rpg/kit.json` and README, `git mv` of the kit-bound files to `kits/rpg/assets/`, `game.json` `kit: "rpg"` | kit | logic | ASSETORG2-G, -D |
| ASSETORG2-B | Engine baseline: `git mv` of the common tileset and interface sounds to `engine/assets/`, its README | engine | logic | ASSETORG2-K (same folders; one at a time) |
| ASSETORG2-T | Tools: `gen_art.py --layer`, `export_units.py` sheets with palette slots, byte-identical regeneration check, READMEs | engine + kit + game | logic | ASSETORG2-B |
| SPRITE1-S2 | SPRITE1 spec update: one sheet per body (section 4.1) instead of per-clip strips; L reads clips from the sheet | game | design | ASSETORG2 approved |
| SPRITE1-A2 | (existing row) export every clip into the class sheets | game | asset | ASSETORG2-T, SPRITE1-S2 |
| SPRITE1-L | (existing row) clip playback on the sheet format (frame at time, loop, fall back to idle) | engine | logic | ASSETORG2-E, SPRITE1-A2 |
| SPRITE1-U | (existing row) battle tokens play the clips | game | logic | SPRITE1-L |

Each step keeps the build drawing the same pictures: G moves files inside the game root first, so K and B are pure
moves the layered loader already finds. ASSETORG1 is the predecessor (done); its TASKS dependency on SPRITE1-A2 is
replaced by ASSETORG2-T.

## 12. Parity retired (named in G's commit and a HISTORY row)

- `checks_parity.rs`: "art keys = AssetRegistry.keys" (`source::art_keys` against the Godot manifest).
- Forge check messages `Art '<k>' isn't a known token/portrait key.` and `Icon '<k>' isn't a known item icon key.`
  (replaced by A1 and A2) and the `art` / `art_inherit` option lists.

## 13. Out of scope

New art (unique portraits, per-character palettes, mace, orb and crossbow bases, goblin rig variants), the clips
beyond `idle` (SPRITE1-A2), skill effect layers, cross-game references, kit crates or kit rules, item rarity frames,
Godot (`legacy/` keeps its own assets and manifest). Proposed follow-ups: PORTRAIT1-A (unique portraits for the eight
heroes, Gruk and the boss unit, 240x240, Style A), PALETTE1-A (party palettes so Mara and Bryn, Tev and Sella differ
from their class), ICONS1-A (kit bases for mace, orb, crossbow, material, quest).

## 14. Open questions (recommended default first)

1. Named goblin lieutenants (Skarn, Vex, Nok, Gruk, Bolg) and the Ash Warchief: unique portraits as elites
   (`portrait_tier: elite`, default; their unique placeholder portraits stay in use) or mobs (common portraits)?
2. Named NPC and lieutenant bodies: keep their unique placeholder bodies as game variants (default; nothing changes
   on screen) or drop them to the class body plus a palette now?
3. The five `ash_*` mob characters' placeholders (near-copies of the goblin variants): delete (default) or make them
   the kit goblin variants instead of the catalog ones?
4. Variant name of the class default: `base` (default, as approved) or `human`?
5. Battle tiles: `common` only (default; today's look) until a map names a set, or the world's set too?
6. Skill families sharing one placeholder glyph (11 groups, 12 files): keep per family (default) or delete and show
   the kind icon?
7. Kit id and path: `rpg`, `kits/rpg/` (default)?
8. Statuses, intent chips, pins and badges in the kit (default) or in the game?

## 9b. Move table as built (ASSETORG2-G, 2026-10-22, after ART2)

ART2 (2026-10-21) changed the tree after section 9 was written: 36 portraits (18 named, 18 generic), 57 skill icons and
16 item icons (8 new family folders). The move was recomputed from the files present by `tools/assetorg2_move.py`
(`plan` prints the table, `apply` moves), which checks that every PNG is moved or deleted once, that no two land on
one path, and that every deletion is pixel-identical to the file that stays (or is an approved Q3 placeholder).
Result: 304 PNGs, 285 moves (114 to kit folders, 103 game, 68 engine) and 19 deletions; 22 bodies with a one-frame
`clips.json` (frame [104, 104]). Differences from section 9, all keeping what the player sees:

- The eight party characters' and Gruk's portraits are now unique (ART2): they move to `portraits/heroes/<id>.png` and
  `portraits/elites/gruk_smokecaller.png`; only their tokens (identical to their class's or the shaman's) are deleted.
- The `ash_*` mob characters: tokens deleted (unchanged near-copies, Q3); their portraits are identical to the goblin
  variants' shared portraits and are deleted too.
- No skill icon is deleted: after ART2 every family icon is unique. The nine kind icons move to `skills/kinds/`.
- Items: `fist`, `cloth`, `mace`, `orb`, `crossbow`, `material`, `quest` have ART2 base icons (`items/<family>/base.png`);
  `knuckles` and `robe` become variants (`items/fist/knuckles.png`, `items/cloth/robe.png`); `battle_axe` goes to
  `items/axe/battle_axe.png`. The section 10 rows giving mace, orb and crossbow another family's folder are dropped,
  and `wpn_battle_axe` gets `icon_id` `battle_axe`.
- Unit folders are named by their ASSETORG1 art keys (`archer`, `skirmisher`, `shaman`, `berserker`, `scout`,
  `goblin_brawler`), mapped to the section 9 bodies (`ranger/goblin`, `rogue/goblin`, `priest/goblin`,
  `brawler/goblin_berserker`, `rogue/goblin_scout`, `brawler/goblin`).
- The asset checks skip the generic enum check of `options_from` `body` / `body_inherit` fields (A1 checks them with its
  own message); no schema row carries an `identity` flag (nothing reads one).
- Palettes reach battles through `UnitDef.palette` and `Session::palettes` (party heroes: none yet).

