# Naming and asset briefs (template)

Status: canon (user 2026-10-25). Game-agnostic rules; each game fills in its own naming styles in
`games/<id>/docs/lore/specs/naming_and_asset_guide.md`.

## 1. Names are cheap, ids and briefs are not

- A display name is a data field, changed through `aegis content` at any time.
- Every thing has a stable id that never follows a rename: saves, quests, art paths and seeds refer to the id.
- Every thing has an asset brief, written once and in full, because art is generated from it.

## 2. The asset brief

Every named character, unit type, boss, place and relic has one:

| Field | What it says |
| --- | --- |
| Age and build | Years, height and frame relative to the rig's class body |
| Silhouette | What reads at token size on the battle board and the region map |
| Clothing and materials | Fabric, leather, metal, bone; condition |
| Faction insignia | Sigils, colours, masks; what is defaced or hidden |
| Signature props | Weapon, tool, keepsake; the weapon family it maps to |
| Marks | Scars, augmentations, corruption, and how they change over the acts |
| Palette notes | Which of the game's art-bible hues; signal colours only for mechanics |
| Expression and posture | Stance, gaze, habits (portraits and idle animation) |
| Pipeline | Sprite rig body or variant, portrait pipeline, or image-API sheet (asked each time) |
| Rig limits | Anything the rig cannot draw yet, so it is planned, not redrawn |

## 3. Names carry faction and role

Each game writes a table with one naming style per group, so a player can guess a stranger's side from the name alone.
Fill in every row the game has:

| Group | Style to define |
| --- | --- |
| The player's faction | Sound, length, use of epithets |
| Each opposing faction | Titles, ranks, sound (for example liturgical, military, mercantile) |
| Neutrals and commoners | Everyday names of the setting |
| Monsters | Descriptive kinds, no personal names unless unique |
| Enemy unit types | Rank or role plus function |
| Places | What happens there; how each act or region shows in the name |
| Gear and skills per tradition | One vocabulary per power source or school |
| Relics | Owner plus object, with provenance |

## 4. Distinctness

- No two characters who share a scene share a first letter or a near-sound.
- No name is reused across a character, a place and an item.
- Display names fit the game's labels at every text size (TEXTFIT1): prefer at most 14 characters, and give long titles
  a short form for cards and lists.

## 5. One source of truth

- Characters, places and terms live in one registry with name, id, faction, role and asset brief; the game's glossary
  holds one name per term.
- Content seeds and asset prompts are written from the registry, never from older docs.
