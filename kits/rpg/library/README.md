# Kit asset library

Generic art and sound a game on this kit may pick from (spec `docs/specs/ASSETLIB1.md`). It sits beside the live asset
root `kits/rpg/assets/`, never inside it: nothing here ships or resolves until a game copies a pick into its own asset
path or references it by its kit path.

| Path | Holds |
| --- | --- |
| `<style>/style.meta.json` | `{"style", "label", "palette", "notes"}` of one style set (`style` is the folder name, snake_case) |
| `<style>/<kind>/<name>.png` (or `.ogg`, `.wav`, `.mp3` under `audio/`) | One asset |
| `<style>/<kind>/<name>.meta.json` | Its sidecar: `id`, `kind`, `style`, `size` (`[w, h]` px; `null` for a sound), `frames` (`[cols, rows]`, sheets only), `palette`, `tags`, `status` (`placeholder` or `final`), `source` (`gemini_web`, `flow`, `hand`, `generated`), `request`, `license` |

Kinds: `tilesets`, `props`, `skill_icons`, `gear_icons`, `status_icons`, `ui`, `units`, `characters`, `portraits`,
`backdrops`, `cgs`, `audio`. Names, ids and tags are generic: no game name, world id or lore word
(`games/*/lore_words.txt`). `tools/fc.sh assetlib` checks the library and runs in the gate.
