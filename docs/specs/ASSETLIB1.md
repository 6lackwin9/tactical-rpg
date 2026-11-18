# ASSETLIB1: the kit asset library and its metadata

Status: active (user 2026-11-18: "lets use gemini webapp for now, but note that these will be placeholder, only thing
that is probably ok/approved directly from gemini webapp are assets that are not locked to the game itself and would go
in the kits asset library like tilesets, props, skill and gear icons"; "note the placeholders can still be added to the
kits asset library so that there are placeholder assets sets available for characters and background stripped of the
game reference and just have meta data related to the asset, we might flesh out the library with assets of different
styles as part of building the asset library later"). Format proposed by Art/UI (STYLE_BIBLE section 6, PR #67) and agreed with the tools lane,
accepted by Design the same day; numbers and names marked AL are `[PROVISIONAL]` (`P-ARCH`, lowest undo cost). Layer:
kit (the library folder) and tools (the check, in the gate). Lane: tools.

## 1. What goes in

- AL1. Kit assets are generic: no game names, places, lore words or story beats in files, paths or metadata. Generic
  Gemini art for tilesets, props, skill icons and gear icons may be `final`; characters, backdrops and CGs from Gemini
  enter only as `placeholder` sets.
- AL2. A game keeps its own locked art in `games/<id>/assets/`; it may reference a kit asset by its kit path.

## 2. Layout

- AL3. The library sits beside the live asset root, never inside it: `kits/rpg/library/<style>/<kind>/<name>.png`
  with `<name>.meta.json` beside it. `kits/rpg/assets/` (which games resolve from and the web bundle ships) is
  unchanged. Nothing in the library ships until a game copies a pick into its own asset path or references it.
- AL4. Kinds: `tilesets`, `props`, `skill_icons`, `gear_icons`, `status_icons`, `ui`, `units`, `characters`,
  `portraits`, `backdrops`, `cgs`, `audio`. `style` is a short snake id (for example `tallow_iron`). A style folder
  has `style.meta.json`: `{"style", "label", "palette", "notes"}`.

## 3. Metadata sidecar

- AL5. Fields (all required unless marked):

  ```json
  {"id": "ruined_chapel_night", "kind": "backdrops", "style": "tallow_iron", "size": [640, 360], "palette": "p3",
   "tags": ["ruin", "night", "interior"], "status": "placeholder", "source": "gemini_web", "request": "AI-20",
   "license": "generated for this project (Gemini web), owner-held"}
  ```

  `id` is stable, generic snake_case and unique in the library; `size` is pixels; `frames` `[cols, rows]` only for
  sheets (optional); `palette` a palette id or `null`; `tags` generic words; `status` `placeholder` or `final`;
  `source` `gemini_web`, `flow`, `hand` or `generated`; `request` an `ART_UI_REQUESTS.md` id or `null`; `license` the
  licence or provenance line.

## 4. Check (in the gate)

- AL6. Every `.meta.json` parses; required fields are present; `kind`, `status` and `source` are valid enums; `kind`
  and `style` match the folders; `size` matches the PNG; `id` is unique; every PNG has a sidecar.
- AL7. Names, ids and tags with a word from any game's `lore_words.txt` (`games/*/lore_words.txt`, the list the layers
  check uses) are refused. The check lives in `tools/` so the kit never reads a game.

## 5. Tests

Check fixtures: missing sidecar, bad JSON, wrong kind, size mismatch, bad status, duplicate id, a lore word in a tag. `0` golden diffs.
