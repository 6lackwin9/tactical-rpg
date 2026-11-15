# REVEAL1: Enterable Indoor Rooms, Roof Fade, Interior Walls & Map Boundaries

Archived 2026-11-15 (CTX-CLEANUP1-DESIGN), task done/superseded: kept verbatim for history. Do not build from this file.

Status: **approved** (user 2026-11-14 via `/art-ui-session` `AO-06`; ready for Build `PR5-54` ahead of `SUNKWARD-MAPS1` `PR5-55`).
Depends on: `PROPS1-L1..L2` (built), `REGIONGROUND1-E1..L` (built).
Layers: kit (`rpgkit_data`, `rpgkit_content` map/prop/zone checks & schema), game (`emberwake_gui/src/{region,props,ground}.rs`, `grounds.json`).

## Goal

Let large multi-screen exploration maps (`48-64 x 30-40`, e.g. Junction 7, Crossroads, Fourth Kiln, Tallow Vats, Crucible Watchpost) include **enterable furnished buildings** (armories, workshops, stores, hovels, shrines, guardrooms) on the same seamless grid without separate sub-map loads, while centring compact 1-screen maps (`sluice_undercroft`, `drowned_runoff_grate`, `corvos_gate`, `slag_gate`) inside themed **boundary** surrounds.

## 1. Indoor Zones & Roof-Layer Props

1. **`zone_legend` Indoor Metadata (`maps.json`)**:
   - Extend `zone_legend.<char>` entries using object form `{id: string, indoor?: bool, wall_set?: string}` only (since `maps.json` `zone_legend` already uses object form; `omit_default` keeps existing maps byte-identical, `FB-271`).
   - When `indoor: true`, `wall_set` optionally names an interior wall art set (resolved at `tilesets/<wall_set>/wall|wall_face`, `FB-271`) used to render wall tiles bordering or enclosing that zone while the roof is faded (`wall_set` on an outdoor zone emits an `R2` warning).
2. **Prop `layer: "roof"` & `roof_of` (`PROPS1` extension)**:
   - Extend prop `layer` options from `["sort", "ground"]` to `["sort", "ground", "roof"]`, and add optional string key `roof_of` (`""` default, `omit_default`, valid only when `layer == "roof"`).
   - **Placement rules (`check::map` P9/P11 + R1-R3)**:
     - Unlike `sort` props (which require `#` wall tiles under their entire footprint), a `roof` prop covers an enterable building's wall ring **and** open `.` interior/doorway floor cells (`w x h` up to `16 x 16`, art scale `32 * w` by `32 * h`..`32 * h + 128` px).
     - `R1`: A `roof` prop must name a `roof_of` zone ID that exists in the map's `zone_legend` with `indoor: true` (and `roof_of` is refused on `sort`/`ground` props).
     - `R2`: Every cell belonging to an `indoor: true` zone must be covered by the footprint of at least one `roof` prop with `roof_of == zone.id`.
3. **Drawing & Smooth Fade (`emberwake_gui/src/{props,region}.rs`)**:
   - **Outside**: When the party's current cell is **not** in `zone_id` (`roof_of`), the `roof` prop draws in an overhead pass **above** ground, interior props, markers, and characters (hiding the furnished interior beneath the building exterior/roof; placeholder fallback when PNG is missing draws a muted roof block with eaves rim).
   - **Inside**: While the party stands on any cell of `zone_id` (or its doorway cell tagged to that zone), all `roof` props with `roof_of == zone_id` **fade out** (`alpha -> 0`, instant under `reduce_motion`, ~`180 ms` transition otherwise), revealing the `floorboards` ground, furnished interior `sort`/`ground` props, `npc_<zone>` / chest markers, and the zone's interior `wall_set` on adjacent `#` ring walls.

## 2. `floorboards` Ground Row & Location `boundary` Art

1. **`floorboards` Ground (`grounds.json`)**:
   - Seed `floorboards` (`name: "Floorboards"`, `order: 72`, `kind: "land"`, `indoor: true`, `colour: "#3a2c22"`, `wear: 0.06`, `strays: false`, `tone: 0.08`, `patches: 0.0`, class `built`, gain `0.45`) via `aegis content` so indoor homes, shops, and workshops render warm timber floors (`tilesets/<set>/ground/<id>/v0.png`; `PR5-54` placed a `packed_dirt` copy placeholder moved to `kits/rpg/assets/tilesets/common/ground/floorboards/v0.png` in `PR5-56`, queued for Flow art in `AI-13`, `FB-271`).
2. **Location & Map `boundary` Field (`worlds/<id>/locations.json`, `maps.json`)**:
   - Use optional `boundary` (`string`, default `""`, `omit_default` in `kits/rpg/schema.json`) on maps and world locations (`drain_brick`, `blind_tenements`, `gorge_rock` for Sunk Ward; location `boundary` already existed prior to `REVEAL1` and falls back to the map's `boundary`, `FB-271`).
   - In `emberwake_gui/src/region.rs`, when the camera centres a map smaller than the viewport (or draws the margin beyond map edges), resolve `tilesets/<boundary>/` and `props/<set>/boundary_<b>/` dressing (falling back to the location's `tileset`, world lineage, then today's wall margin) across `game -> kit` layers; `aegis content check` emits a non-blocking asset warning when a named `boundary` set has no art yet.

## 3. Acceptance Criteria

1. Existing `maps.json` and `locations.json` resave byte-identical when no `indoor`, `roof`, or `boundary` fields are set; `0` combat/sim/journey golden diffs.
2. Unit/content tests cover `zone_legend` object normalization, `R1`/`R2` roof-coverage checks, `roof` prop pass order outside vs faded inside an `indoor` zone (with `reduce_motion` instant snap), interior `wall_set` swap, `floorboards` ground composition, and `boundary` margin fallback.
