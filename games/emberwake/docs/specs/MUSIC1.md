# MUSIC1: music tracks and the music player

Approved by the user 2026-10-18 01:02 UTC ("Approve" on the five-track card in the Assets & Audio thread). Replaces the code-built menu placeholder UI/UX planned. Extends S14 (`games/emberwake/docs/specs/S14_RULES.md`): music was the "later" slice of the first audio decision.

## 1. Tracks (MUSIC1-A, MUSIC2-A)

Original loops synthesized from notes by `tools/gen_music.py` (numpy + ffmpeg, no samples), Ogg Vorbis q5, 44.1 kHz stereo, in `res://assets/audio/music/<key>.ogg`, one manifest row each (`music/<key>/stream`, `origin: user`, note `generated loop (MUSIC1)`). Each track renders as one circular buffer, so the loop has no seam.

| Key | Plays on | Mood | Length |
| --- | --- | --- | --- |
| `menu_theme` | Main Menu and its screens | calm A minor, plucked melody over a drone, frame drum on the second pass | 60 s |
| `ashlands` | Ashlands region and world maps | dry and sparse, D Phrygian dominant, wind bed, hand drum | 75 s |
| `iron_reaches` | Iron Reaches maps | heavy E minor, anvil strikes, low toms, horn pads | 72 s |
| `tidewrack` | Tidewrack Coast maps | 6/8 G mixolydian whistle tune, sea-swell bed | 72 s |
| `combat` | every battle | 120 BPM A minor, drums, pulsing bass, riff | 48 s |
| `town` | towns and fortresses (MUSIC2) | warm C major, lute arpeggios, whistle tune | 65 s |
| `danger` | camps, ruins, dungeons (MUSIC2) | 70 BPM B minor, drone, heartbeat drum, wind | 55 s |
| `shop` | while a shop is open (MUSIC2) | cosy F major waltz | 58 s |
| `travel` | world map (MUSIC2) | D mixolydian march, horn tune | 63 s |
| `boss` | fights with a boss-tier unit (MUSIC2) | 140 BPM D minor, toms, anvil, horn calls | 55 s |

## 2. Rules (MUSIC1-L)

1. `MusicDirector` (`scripts/audio/music_director.gd`) plays one track at a time on the Music bus. Scenes do not play streams; they post requests: `MusicDirector.request(owner: Node, key: String, priority: int)` (same owner replaces its request) and `MusicDirector.release(owner)`. Priorities: `MENU = 0`, `FIELD = 1`, `COMBAT = 2`.
2. The wanted track is the request with the highest priority whose owner is valid and inside the tree and, when the owner is a CanvasItem, visible in the tree (ties: the newest request). A request whose key does not resolve to a music file is ignored. No live request means silence. The choice is a pure static function over the requests, so tests run headless.
3. Field key for a location: the first music key that resolves among the location's region id, that region's parent chain, the game world id, then the world's lineage (nearest ancestor first). A single region gets its own track later by adding a manifest row named after the region id, no code change.
4. Hooks: the main menu view requests `menu_theme` (MENU) once in `_ready`; the region map view and the world map view request the field key (FIELD) for their game's location whenever it changes (cached per world + location); `BattleScreen.open` requests `combat` (COMBAT) with the screen as owner. The results popup keeps the combat track because the arena stays mounted under it; closing the battle falls back to the field track; leaving the game falls back to the menu theme.
5. Playback: two `AudioStreamPlayer`s on the `Music` bus under a lazily created node added (deferred) to the tree root, so it survives scene changes. A change of track crossfades over 1 s (old player fades out and stops, new one fades in from the start of its loop); the first track after silence fades in over 2 s; a wanted key equal to the playing key does nothing. Streams loop. Player volume `MUSIC_DB = -6.0` under the Music bus volume from Settings. Headless (`AudioDirector.enabled()` false) creates no players but still tracks the wanted key.
6. Tests in `audio_test_suite`: priority and tie choice, invisible and freed owners ignored, unresolved keys ignored, release, field key fallback (region, parent, world, lineage), manifest rows for the five keys resolve to files.
7. Scope change (user, 2026-10-18, via the Assets & Audio thread): requests name a scene CONTEXT, not a key, so later tracks are data-only. `MusicDirector.request(owner, context, priority, world_id = "", location_id = "")`; `request_game(owner, context, gs)` fills world and location from a game state and uses the context's priority. Contexts and priorities (constants in `MusicDirector`):

   | Context | Priority | Parent context | Last step of its own chain |
   | --- | --- | --- | --- |
   | `menu` | MENU 0 | none | `menu_theme` |
   | `field` | FIELD 1 | none | the plain scope keys (region, parents, world, lineage), tried before `field` alone |
   | `travel` | FIELD 1 | `field` | none |
   | `town` | FIELD 1 | `field` | none |
   | `shop` | FIELD 1 | `town` | none |
   | `danger` | FIELD 1 | `field` | none |
   | `combat` | COMBAT 2 | none | none (`combat` alone is its own step) |
   | `encounter` | COMBAT 2 | `combat` | none |
   | `boss` | COMBAT 2 | `combat` | none |

   Key chain for a context, most specific first: the scopes of rule 3 (region id, its parent chain, the world id, the lineage nearest first) each as `<scope>_<context>`, then for `field` the plain scope keys, then `<context>` alone, then for `menu` `menu_theme`; if none resolves, repeat with the parent context. A generic `field` row therefore never overrides `ashlands` and the other plain keys. Examples: a boss fight in the Ashlands with only `combat` shipped plays `combat`; a row `ashlands_boss` would win. Wired today: the main menu requests `menu`; the region map `shop` while its shop panel is open, else its location's context by kind (rule 8), and releases its request while its embedded world map is open (so `travel` plays); the world map `travel`; `BattleScreen.open` `boss` when an enemy or wave unit has the `boss` tier, else `combat`. `encounter` has no caller yet. Decided 2026-10-18 (coordinator): field order as above; the game world (not the location's world) stays the world scope for connected-world locations.
8. Kind table (MUSIC2-L, user 2026-10-18 "make them"): `MusicDirector.KIND_CONTEXT` maps the current location's `kind` to the region map context: `town` and `fortress` -> `town`; `ruin`, `camp` and `dungeon` -> `danger`; any other kind (`gate`, `wild`, none) -> `field` (`kind_context(kind)`, `field_context(gs)`). `region_context(gs, shop_open)` returns `shop` while the shop panel is open (shop -> town -> field), else the kind context; the region map's `_refresh` runs on shop open and close. A new kind or remap is one table row.
