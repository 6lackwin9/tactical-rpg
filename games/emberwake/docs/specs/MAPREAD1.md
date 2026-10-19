# MAPREAD1: Readable region and world maps (points of interest easy to spot)

Status update (2026-10-20): approved and built; the rows are done (docs/archive/TASKS_DONE.md). The status line below is the original.

Status: proposed (spec 2026-10-18; approved direction: decision world-campaign "DR1 design review answers (user 2026-10-18)" (5))
Depends on: POI-FIX (e143307, on develop: hint line, bump notes, "Nothing to interact with there.", click to interact); aligns with UI/UX's planned full-screen Ashlands and world maps with placeholder art (no TASKS row yet)
Lane: logic in two parts (MAPREAD1-L1 read-out and data keys, MAPREAD1-L2 drawing) plus a small asset slice (MAPREAD1-A)
Touches: L1: `scripts/exploration/{map_rules,explore_runner}.gd` (marker `label` and `note`, `marker_view`), `scripts/travel/travel_rules.gd` (`view` node extras), `exploration` and `travel` suites. L2: `scripts/ui/overworld/{region_map_view,world_map_view}.gd`, `scripts/art/{tile_art,asset_registry}.gd` (template-backed marker and pin keys), `asset_pipeline_test_suite`, overworld UI checks. A: `data/worlds/ashlands/locations.json`

## Goal

Without real art the maps are grey squares with single letters (`MARKER_GLYPHS` in `region_map_view.gd`: N, C, L, !, R) and plain world-map circles. A player must see at a glance what each point is, what it is called, what to do with it and whether it is used up. Each marker kind gets its own placeholder icon and colour, a name on hover or when faced, a legend of what is on the map, and clear "spent" states. Real art later replaces only the files behind the keys.

## Ownership (recommended)

- MAPREAD1-L2 (drawing) belongs to the **UI/UX thread**, folded into or run right after its full-screen map work: both edit `region_map_view.gd` and `world_map_view.gd`, so one owner avoids a merge fight. The full-screen task owns layout, scale, camera and panels; MAPREAD1 owns what a marker shows.
- MAPREAD1-L1 (read-out, two marker keys) belongs to the **Gameplay thread**, which owns `ExploreRunner` and did POI-FIX. It is small, headless-tested and can land first.
- MAPREAD1-A belongs to **Assets & Audio**.

## 1. Schema contract (L1, schema-first: yes)

### 1.1 Marker keys (`MapRules.MARKER_COMMON`)

| Key | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| label | String | "" | 0..32 characters ("Marker '%s': label is at most 32 characters.") |
| note | String | "" | 0..160 characters; only on kind `point` ("Marker '%s': only points take a note.") |

### 1.2 `ExploreRunner.marker_view(gs, loc, m) -> Dictionary` (read-only)

| Key | Value |
| --- | --- |
| kind | the marker kind |
| state | `present`, `locked` (as `presence`), or `spent`: a chest whose done flag is set, or a point whose `sets_flags` all already hold |
| label | `label` if set, else by kind: npc = character name; chest = "Chest"; lore = lore name; exit = `To %s` (location name) or "World map"; encounter and roamer = encounter name; point = "Something here" |
| verb | npc "Talk"; chest "Open" ("Empty" when spent); lore "Read"; exit "Walk onto it" (`Locked: %s` with the `locked_text`, when locked); encounter "Fight"; roamer "Patrol: fight on contact"; point "Look" ("Nothing more here" when spent) |
| badge | npc only: `quest` when the character offers a quest, `ready` when a quest can be turned in to them (`QuestRunner.offers` and ready flags), else "" |
| icon | asset key, see 2.1 |

### 1.3 `TravelRules.view` node extras

Each known node gains `kind` (the location `kind`: fortress, town, camp, ruin, gate, wild), `region` name and `quest` (true when an active quest step targets the location or one of its encounters or characters). Fogged nodes stay hidden (unchanged).

## 2. Rules

### 2.1 Placeholder icons (L2)

Template-backed tile keys (generated SVG in `TileArt`, listed with `overlay_exit` and `overlay_poi` in `AssetRegistry.TILE_OVERLAY_KEYS`), so they always resolve and a manifest file can replace each one later:

| Marker kind | Key | Shape | Colour (state present) |
| --- | --- | --- | --- |
| npc | overlay_npc | head and shoulders | cyan |
| chest | overlay_chest | box with lid | gold |
| lore | overlay_lore | open book | violet |
| exit | overlay_exit (exists) | arrow out | green; locked orange |
| encounter | overlay_encounter | crossed blades | red |
| roamer | overlay_roamer | blades in a ring | red, pulsing every 1 s |
| point | overlay_poi (exists, the ring) | ring | white |

Spent: the same icon at 40% opacity. World map pins: `pin_fortress`, `pin_town`, `pin_camp`, `pin_ruin`, `pin_gate`, `pin_wild` (simple silhouettes), current location ringed gold with "You are here".

### 2.2 Region map (L2)

1. Every present, locked or spent marker draws its icon over the tile; the letter glyphs go.
2. **Names on demand (Q1 A):** hovering a marker tile, or facing one from an adjacent tile, shows `label · verb` in the hint line under the top bar; npc and exit labels are also drawn above the tile at all times (they are what players look for first).
3. **Badges:** `quest` draws a gold "!" and `ready` a gold "?" on the npc icon.
4. **Legend ("Here" panel):** a list of the location's visible markers (icon, label, state), npcs first, then exits, chests, lore, points, fights; hovering a row pulses its tile. No auto-walk.
5. **Notes:** walking onto a point logs its `note` (else "You look around."), replacing the raw "Flags set: ..." line (POI-FIX handover); a spent point logs "Nothing more here."; E on an opened chest logs "The chest is empty." The existing "Nothing to interact with there." stays.

### 2.3 World map (L2)

1. Pins use the kind icons; names stay beside the pin.
2. **Quest marker (Q2 A):** a gold "!" on a pin whose `quest` is true.
3. **Hover card** on a pin: name, kind, region, visited or not, route time and supply cost from the current location (`TravelRules`), and the quest step names that point there.
4. A small legend of pin kinds and route styles (solid open, dashed closed, arrow one-way) in a corner.

## 3. Acceptance checks (tests)

- L1: `marker_view` labels for every Ashlands marker are non-empty; a chest after opening and the `beacon` point after use report `spent`; Mara reports `quest` while she has an offer; a 33-character label and a note on a chest are validation errors; `TravelRules.view` marks `ashmaw_camp` `quest` while `keep_one_fire_lit` step 1 is active.
- L2: every overlay and pin key resolves through `AssetRegistry.texture` with no fallback; at 1280x720 and all four text sizes, labels and the legend are not clipped; hovering a marker shows its label and verb; the letter glyphs are gone.

## 4. Assets (MAPREAD1-A)

| Location | Marker | label | note |
| --- | --- | --- | --- |
| cinder_gate | beacon | "Signal beacon" | "You light the old Warden beacon. Smoke climbs over the pass." |
| cinder_gate | wastes_road | "World map: the wastes" | - |
| warden_keep | signpost | "World map: the roads" | - |
| warden_keep | hearth | "The keep's hearth" | - |

Other markers take the derived labels (1.2).

## Out of scope

Real art (S14 follow-ups), full-screen layout, camera and zoom (UI/UX full-screen maps), auto-walk or path preview, a minimap, fog changes, new marker kinds, sound cues.

## Questions for the user

1. Marker names on the region map?
   - (recommended) A: npc and exit names always shown; every name on hover or when facing it, plus the legend. The two things players hunt for are always visible, and the map stays uncluttered.
   - B: every marker name always shown. Nothing hidden, but small maps get crowded.
   - C: names only on hover and in the legend. Cleanest map, least discoverable.
2. Show where the active quest leads?
   - (recommended) A: yes, a gold "!" on world-map pins and on npcs with an offer, "?" for a ready turn-in. The early game has few hints and the maps are new to players.
   - B: only in the hover card and journal. Less hand-holding, more searching.
