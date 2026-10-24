# Naming and asset guide

Status: canon (user 2026-10-25). Applies to every named thing in *Emberwake: A Scarlet Lullaby*: characters, units,
items, skills, places and terms. The names registry is [cast_registry.md](../background/cast_registry.md); the numbers
and glossary are [CANON.md](../CANON.md). This is the game's filled-in version of the engine's template
[naming_and_asset_briefs.md](../../../../../engine/docs/authoring/naming_and_asset_briefs.md).

## 1. Names are cheap, ids and briefs are not

- A display name is a data field `[data]`, changed through `aegis content` at any time. Renames are expected once the
  game is playable.
- Every thing has a stable id that never follows a rename: saves, quests, art paths and seeds refer to the id.
- Every thing has an asset brief (section 2), written once and in full, because art is generated from it. Effort goes
  into the brief, not the name.

## 2. The asset brief

Every named character, unit type, boss, place and relic has a brief in its registry row or a linked section:

| Field | What it says |
| --- | --- |
| Age and build | Years, height and frame relative to the class body (the rig's proportions) |
| Silhouette | What reads at token size (40 px on the battle board, 16 px on the region map): headgear, weapon shape, cloak |
| Clothing and materials | Fabric, leather, iron, bone; condition (patched, scorched, wax-stiffened) |
| Faction insignia | Sigils, colours, masks, what is defaced or hidden |
| Signature props | Weapon, tool, keepsake; the weapon family it maps to |
| Marks and taints | Scars, grafts, calcification, and how they change in later acts |
| Palette notes | Which art-bible hues and the skin ramp; signal colours only for mechanics |
| Expression and posture | Stance, gaze, habits (for portraits and idle animation) |
| Pipeline | Sprite rig body or variant, portrait (ART2 pipeline), or image-API sheet (asked each time) |
| Rig limits | Anything the rig cannot draw yet (non-human anatomy, extra limbs), so it is planned, not redrawn |

## 3. Names carry faction and role

A player who has never read the lore should be able to guess a stranger's side from the name alone.

| Group | Style | Examples |
| --- | --- | --- |
| Severed Root youths | Short, plain street names; often an earned epithet | Kaelen, Vesper, Hollis, Dunstan "Slag-Jaw" |
| Sovereign Clerisy | Long, liturgical, saintly; always a church title | Arch-Prelate Othemar, Cantor Ysolde |
| Iron Inquisition | Military rank plus a hard one- or two-syllable surname | Captain Vane, Inquisitor-Prelate Nemeth, Proctor Quell |
| Tallow Guild and industry | Trade and guild surnames, often from the work | Foundry-Master Jessop, Madame Sallow |
| Commoners and the Spent | Old, homely names | Old Mother Agnes, Ida |
| Hermits and heretics | Scriptural or apocryphal | Brother Malachi, Brother Rook |
| Aberrations | No personal names: what the thing is | the Amalgam, Render-Beast |
| Enemy unit types | Rank or role plus function | Castellan Footman, Lantern Warden |
| Places | What happens there, industrial-gothic compounds; holy sites take saints' names | Ash-Crawl Tenements, Tallow Vats, St. Lyra's |
| Places by act | The stratum shows in the name | Act 1 Sunk or Low, Act 2 Bastion or High, Act 3 Mantle or Deep |
| Cold Steel gear and skills | Plain smith and soldier words | billhook, pavise, Pommel Strike, Sunder and Pin |
| Inoculated grafts and skills | Anatomical, clinical, body-horror | Calcified Spike, Bile Siphon, marrow gland |
| Relics | Owner plus object, with where it came from | Captain Vane's pavise |

## 4. Distinctness

- No two characters who share a scene (the founders, the hub cast, one act's cast) share a first letter or a
  near-sound.
- No name is reused across a character, a place and an item.
- Names fit the game's labels at every text size (TEXTFIT1): prefer at most 14 characters for a display name, and give
  long titles a short form for cards and lists.

## 5. One source of truth

- Characters, places and terms live in one registry ([cast_registry.md](../background/cast_registry.md)) with name, id,
  faction, role and asset brief; the glossary in [CANON.md](../CANON.md) holds one name per term.
- Content seeds and asset prompts are written from the registry, never from older docs.
