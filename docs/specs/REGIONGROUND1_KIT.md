# REGIONGROUND1: kit part (ground kinds and map fields)

Status: spec (2026-11-02), with the engine part `docs/specs/REGIONGROUND1_ENGINE.md` and the game part
`games/emberwake/docs/specs/REGIONGROUND1.md`. Row: REGIONGROUND1-K. Schema-first: yes.
Depends on: LOOK1-K (`tileset`, A13-A15, done), PROPS1-L1 (`props` on templates, done), HOOKS1-S (catalog row checks, done)
Lane: logic
Touches: section 3.

## 1. Contract

### 1.1 Catalog kind `grounds`

`data/catalog/grounds.json`, `kits/rpg/schema.json` `content.grounds`, following the `zones` kind (HOOKS1). An unknown
key is an error.

| Key | Type | Default | Allowed |
| --- | --- | --- | --- |
| `id` | string | required | `[a-z0-9_]`, 2-24 chars, unique |
| `name` | string | required | at most 32 chars |
| `order` | int | required | 0-999, unique in the catalog; paint order |
| `kind` | string | `land` | `land`, `water` (water is `sunken` and belongs on `~` cells) |
| `colour` | string | required | `#rrggbb`: the flat stand-in with no art, so a layout can be played before its tile set |
| `wear` | number | .10 | 0-.3 |
| `strays` | bool | false | |
| `tone` | number | .13 | 0-.2 |
| `patches` | number | 0 | 0-.3 |
| `marks` | list of strings | `[]` | at most 4 prop art ids |
| `marks_per_100` | int | 0 | 0-10 |

### 1.2 Map template fields (`maps` kind, still version 1)

`source::map_template` and `store::GAME_KINDS` add them after `props`, before `tags`; each is stored even when empty.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `ground` | list of strings | `[]` | one char per cell, beside `rows` and in the form of `zones`; `[]`: the map draws as today |
| `ground_legend` | object | `{}` | char to ground id |
| `ground_patches` | list | `[]` | large patches on purpose: `{id, x, y, w, h, strength}`, strength default .12 |

`rows` alone still decide walking: `explore::is_open` is `.` only, and nothing in `rpgkit_world` or `rpgkit_battle` reads
the new keys. Edited by `aegis content upsert maps`, as `zones` and `props` are.

### 1.3 Checks (errors, under `maps:<id>`; `check/ground.rs`, called from `validate_all`)

| Check | Fails when | Message |
| --- | --- | --- |
| G1 | `ground` empty but a legend or patch is set | `Ground legend and patches need ground rows.` |
| G2 | `ground` not one string per row, one char per cell | `Ground has W x H cells; the map has W2 x H2.` |
| G3 | a legend key not one char of `[a-z0-9]` | `Ground legend key '<k>' must be one lowercase letter or digit.` |
| G4 | a cell's char not in the legend | `Ground (x, y): '<c>' is not in the legend.` |
| G5 | a legend id not a `grounds` row | `Ground legend '<k>': no ground '<id>'.` |
| G6 | more than `GROUND_LEGEND_MAX` 12 entries | `At most 12 grounds per map (got N).` |
| G7 | a `water` ground on a `.` cell, or a land ground on a `~` cell (the picture must not lie about walking; `#` takes any) | `Ground (x, y): '<id>' on a '<c>' cell.` |
| G8 | a placed patch: bad keys or id, outside the map, strength not .02-.25, or its rect and a ring of one cell not all one land ground and free of `#` | `Ground patch '<id>': <reason>.` |

Catalog rows: the 1.1 table's limits, one error each, as HOOKS1-S's row checks do.

A legend entry no cell uses is a warning. `ground_patches` at most 16.

### 1.4 Lookups and the plan (pure; `rpgkit_content`)

- `art::ground_dirs(sets, ground) -> Vec<String>`: `tilesets/<set>/ground/<ground>` over `region_sets(lineage, tileset)`;
  the file is `v0.png` in the first layer that holds it (the assets tree test's `v<n>` pattern), and only `v0` is read:
  a ground has one texture. `art::ground_px()`: 256.
- `ground::plan(template, grounds) -> Option<GroundPlan>`: None when `ground` is empty; else the layer list sorted by
  `order`, `cells` as indices into it, `blocked` from `#` in `rows`, placed patches, and `seed` FNV-1a 64 of the template
  id, so one map has one picture however it is reached.

### 1.5 Asset checks (warnings unless marked; `check/assets.rs`, next free codes)

| Check | When | Message |
| --- | --- | --- |
| A16 | a ground a map uses has no `v0.png` along the chain | `Map '<map>' ground '<id>': no art for world '<w>'; it draws its flat colour.` |
| A17 (error, as A15) | a ground file not 256 x 256 | `Ground art '<rel>' is WxH px; it must be 256 x 256.` |
| A18 | a `marks` art id with no file, or larger than 64 x 64 px | `Ground '<id>' mark '<art>': <reason>.` |
| A19 | a marker's anchor on either of the 2 rows north of a wall cell, unless that cell is on the south row or a side column, or under a standing prop with art (the game's `REGIONGROUND1_SCENE.md` 1.3) | `Map '<map>' marker anchor '<id>' stands behind a wall; it draws its outline.` |

A13 (no `floor/v0.png` in a bound set) also accepts a set with a `ground/` folder. A15's `REGION_FACE_PX` gains (128, 48), the tall face.

## 2. Acceptance checks (rules, not pictures)

`rpgkit_content` tests, as `prop_checks.rs`:

1. Each of G1-G8 and the row limits, one fixture each, with its message; A16-A18 on a fixture layer.
2. A template without the keys normalizes to the three empties in stored order; shipped maps re-save byte for byte.
3. Walking is untouched: on a fixture with ground over every cell, `is_open` and `path_to` results equal the map without
   it; `plan` of `ground: []` is None; `plan`'s order sorts by `order`, not by legend order.
4. Journey golden, store resave and battle traces unchanged (presentation only; no parity check names these keys, and
   none retires).

## 3. Touches

`kits/rpg/schema.json`, `kits/rpg/crates/rpgkit_data/src/lib.rs`, `kits/rpg/crates/rpgkit_content/src/{source.rs, store.rs, art.rs, ground.rs (new), check/{ground.rs (new), mod.rs, assets.rs}}`, its tests, `games/emberwake/data/catalog/grounds.json` (empty kind), `games/emberwake/data/maps/maps.json` (re-saved by the writer: three empty keys per template), test fixtures that list map keys (`emberwake_world/tests/store.rs`, `emberwake_content/tests/prop_checks.rs`, `emberwake_sim/tests/fixtures/sim_data.json` if it compares them).
