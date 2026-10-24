# Worldbuilding decisions log

Status: canon (2026-10-25). The brainstorm's own decision log: DEC-LORE-01..16 were logged on 2026-10-24 from the
user's design session with Gemini and are kept as written, with superseded parts marked "Superseded (2026-10-25)";
DEC-LORE-17 on record the user's 2026-10-25 decisions (the lore consistency pass). Settled rows are copied by the
planner into `games/emberwake/docs/decisions/lore.md`, which is the project's record. Names follow the
[cast registry](../background/cast_registry.md) (the 2026-10-25 renames are applied below: Bran -> Hollis, Torin ->
Dunstan, Inquisitor-General Vaelen -> Ruthven, Madame Vespera -> Sallow). Numbers: [CANON.md](../CANON.md).

## 1. The setting pivot (2026-10-24)

| Vector | Legacy setting (the Ashlands) | New setting (the Low Sleep) |
| --- | --- | --- |
| Genre | Frontier fantasy | Medieval feudalism and eldritch cosmic horror |
| Atmosphere | Frontier border war, volcanic ash, dry season | Bruised-violet gloom (the Shroud), dead stars, warded tallow lanterns, cathedral bells |
| Adversary | The Ashen Horde (goblin warbands) | The Sovereign Clerisy (the Red Tithing) and the Iron Inquisition |
| Player faction | Ember Wardens holding Warden Keep | The Severed Root, cells of Unsealed youths (ages: DEC-LORE-22) |
| Natural law | Ordinary biology | The Stagnation of Mortality: flesh ossifies into black bone; the dead do not rot without rock-salt cremation |
| Power | Elemental and divine spells | Cold Steel (no Strain) against the Bitter Inoculation (grafts, Strain, Calcification) |
| Combat | Block timeline with stances (the "classic" ruleset) | The "timeline" ruleset (DEC-LORE-19) |
| Hub | Warden Keep | The Sluice Undercroft beneath Lower Cinderreach |
| Opening | The Ashlands (Cinder Pass, Ruined Courtyard) | The Sunk Ward (Ash-Crawl Tenements, Tallow Vats, Crucible Watchpost, Slag Gate) |

Superseded (2026-10-25): the pivot does not replace the Ashlands; they stay as the kit's sample and parity world
(DEC-LORE-18).

## 2. Decisions DEC-LORE-01..16 (2026-10-24)

### DEC-LORE-01: The cosmic leviathan ("The Low Sleep")

- Decision: the continent rests on the slumbering mass of a cosmic titan.
- Consequence: earthquakes are its tremors; vents emit mutagenic black marrow; deep mining distorts space. The Clerisy
  keeps power by sacrificing youths (the Red Tithing) to lull the titan back to sleep. Open (2026-10-25): what the
  Tithing really does is a proposal in [central_truth.md](../background/central_truth.md); the Clerisy's claim stands.

### DEC-LORE-02: Generational divide and the permadeath clock

- Decision: people aged 25+ (the Spent) calcify and become docile, dependent on communion draughts and tallow fumes.
- Consequence: the resistance is the Unsealed, ages 15-24; heretical power advances Calcification through five stages.
  Superseded (2026-10-25): operatives are 18-24 and the Unsealed are everyone under 25 (DEC-LORE-22).

### DEC-LORE-03: Cold Steel against the Bitter Inoculation

- Decision: every class has two opposed branches: Cold Steel (no Strain, timeline positioning and Active Guard, no
  corruption) and Inoculated (cheap AP, burst and armour-piercing damage, Strain, crisis risk, Calcification).
- Superseded in part (2026-10-25): AP cost is decoupled from tier and branch (`ap_cost`, DEC-LORE-37); "high AP" for
  Cold Steel reads as "2-4 AP".

### DEC-LORE-04: The 6-tick simultaneous timeline

- Decision: actions resolve across a 6-block round for all sides at once, not in sequential turns.
- Consequence: windups can be staggered and interrupted; movement through Advance (move, then act) or Disengage (act,
  then move). Superseded (2026-10-25): the fixed impact anchors "Snap T2, Standard T4, Heavy T5-6" are replaced by
  relative windups (DEC-LORE-20). The engine already resolves 6 blocks simultaneously (it was never sequential).

### DEC-LORE-05: Hub and campaign geography

- Decision: the haven is the Sluice Undercroft with four wings: the Apothecary's Slab (Sister Aline: surgery, Strain
  flushes, grafts), the Tinker's Hearth (Bram: Cold Steel forging, stagger tuning, demolition), the Water-Lock Basin
  (storage, dead-drops, Tess's board) and the Dry Hollows (bunks, the Isolation Crypts).
- Decision: the Act 1 theatre is the Sunk Ward: Ash-Crawl Tenements, Tallow Vats, Crucible Watchpost, Slag Gate.

### DEC-LORE-06: Codebase and file isolation

- Decision: engine and game files stay untouched during the worldbuilding phase; all lore lives in
  `games/emberwake/lore/`.
- Superseded (2026-10-25): the library moved to `games/emberwake/docs/lore/` with the game's design docs and follows
  CLAUDE.md (DEC-LORE-42). Code and data still change only through specs and tasks.

### DEC-LORE-07: The three-act dramatic arc

- Decision: Act 1 The Young Vanguard (about 4-5 h: Cold Steel only, the founders, personal quests, the Crucible
  sacrifice); Act 2 The First Rupture (about 7-9 h: Inoculation, Strain and Calcification, recruits from culling
  wagons, the doctrine split); Act 3 The Awakening Abyss (about 5-6 h: bells, tallow shortages, Alert peaks, the Doom
  Clock). Superseded in part (2026-10-25): "Phase" reads "Act" (DEC-LORE-38); the Doom Clock shows from Act 1 day 1
  (DEC-LORE-27).

### DEC-LORE-08: Six founders and the bunk count

- Decision: the Undercroft starts with 6 bunks, one per base class.
- Consequence: Kaelen (Vanguard), Vesper (Gutter-Stalker), Maren (Heretic Acolyte), Hollis (Arbalestier), Dunstan
  "Slag-Jaw" (Slag-Brute) and Elyse (Ichor Alchemist) are the Act 1 founders. Refined (2026-10-25): the first playable
  slice has five (DEC-LORE-21).

### DEC-LORE-09: Episodic prologue and in-medias-res start

- Decision: New Game opens with a short illustrated vignette, then Kaelen in a rain-slick Seventh Ward alley.
- Consequence: three prologue steps (Kaelen's flight, Vesper's rooftop ambush, Maren and Hollis freed from an iron
  mortuary cart), then the hearth scene. Confirmed (2026-10-25) as missions 0a-0c (DEC-LORE-33).

### DEC-LORE-10: Resilient ensemble and fail states

- Decision: no single death ends the game; the cell outlives any member.
- Consequence (2026-10-24): Game Over on (1) total annihilation, (2) a siege breach at Alert 100%, (3) the Doom Clock
  expiring; captured Stage III-IV operatives can be deployed as Living Martyrs. Superseded (2026-10-25): (2) is not a
  Game Over (DEC-LORE-26); (3) fails only in Act 3 (DEC-LORE-27); the screen is "Extinguished" (DEC-LORE-28); Living
  Martyrs are an Act 2 proposal.

### DEC-LORE-11: Stances dropped

- Decision: the four stances and their switch cost are dropped for the timeline phases (windup, impact, recovery,
  Active Guard), poise and the Strain brackets.
- Refined (2026-10-25): stances are a ruleset option, off in "timeline" and on in "classic" for the Ashlands
  (DEC-LORE-19); the Flesh-Tension "-20 Stance Stability" reads "stagger taken x2".

### DEC-LORE-12: Rules baselines

- Decision: maximum reuse of the existing rules and battle crates; three preferred baselines, open to playtest: Strain
  added when an action starts; Inoculated attacks ignore armour (binary); speed-tier impact mapping.
- Superseded (2026-10-25): the "~85% reuse" figure is withdrawn (it was never measured; the reuse assessment is
  [rpg_kit_and_system_architecture.md](rpg_kit_and_system_architecture.md)); the impact mapping is replaced by relative
  windups (DEC-LORE-20). The first two baselines stand.

### DEC-LORE-13: Title

- Decision (2026-10-24): "Emberwake: The Low Sleep"; `emberwake` stays the codename, crate prefix and path. "Ember" is
  the brief warmth of doomed youth; "Wake" is both a vigil over a dying world and the titan's stirring.
- Superseded (2026-10-25): the subtitle is "A Scarlet Lullaby" (user 2026-10-25, DEC-LORE-17).

### DEC-LORE-14: Class trees and founder specialisations

- Decision: all six classes have a Cold Steel and an Inoculated tree; each founder adds an innate trait, an authored
  skill variant and a signature capstone; procedural recruits use the base trees. Detail:
  [classes_and_skill_trees.md](../systems/classes_and_skill_trees.md). Refined (2026-10-25): the deck is 8 slots
  (DEC-LORE-37).

### DEC-LORE-15: Equipment families, consumables and cast

- Decision: 8 weapon families (4 Cold Steel, 4 Inoculated), a battle pouch for Strain and poise consumables, 9 named
  relics, a monster-part harvest economy; a city cast beyond the hub (Arch-Prelate Othemar, Inquisitor-General Ruthven,
  Madame Sallow, Bailiff Corvo, Old Mother Agnes, Brother Malachi, Lucan). Detail:
  [items_gear_and_progression.md](../systems/items_gear_and_progression.md),
  [npcs_and_questlines.md](../background/npcs_and_questlines.md).
- Superseded in part (2026-10-25): the pouch holds 3 (the engine's pouch), not 4.

### DEC-LORE-16: Three-tier campaign map

- Decision: Act 1 the Sunk Ward (Lower Cinderreach, 8 nodes, ends at the Slag Gate; hub the Sluice Undercroft, 6
  bunks); Act 2 the High Bastion (Upper Cinderreach, 8 nodes, ends at the Reliquary Crypts; relay the Scribe's Crypt, 8
  bunks); Act 3 the Mantle Fissure (8 nodes, the Doom Clock, ends at the Dreaming Core; base the Weeping Redoubt).
- Superseded in part (2026-10-25): one hub through the campaign is preferred; the Scribe's Crypt and the Weeping
  Redoubt are proposal forward bases (DEC-LORE-39); the 8th bunk comes from Bram's project (DEC-LORE-32).

## 3. Decisions of 2026-10-25 (user, the lore consistency pass)

| Id | Decision |
| --- | --- |
| DEC-LORE-17 | Title "Emberwake: A Scarlet Lullaby" (the Red Tithing is one lullaby among many, sung in blood each generation); supersedes DEC-LORE-13's subtitle. Codename and paths stay `emberwake` |
| DEC-LORE-18 | The engine and kit are reused. The Ashlands stay as the kit's sample and parity world; the Low Sleep campaign is new worlds in `games/emberwake` (first world `sunk_ward`). Answers open query 2 of the 2026-10-24 log |
| DEC-LORE-19 | The new combat rules are an evolution of the existing resolver as a switchable ruleset option, "timeline" against "classic"; classic stays for the Ashlands. Answers open query 1 |
| DEC-LORE-20 | Speed tiers are relative windups: the impact lands at the end of the action's own blocks (Snap 1, Standard 2, Heavy 3-4); a maneuver may add recovery blocks; no fixed tick anchors |
| DEC-LORE-21 | Six founders in the full Act 1; the first playable slice has five (the Ichor Alchemist, Elyse, joins later). "The Closed Four" is retired |
| DEC-LORE-22 | Playable operatives are 18-24; the Unsealed are everyone under 25; the Tithing takes Unsealed youths of any age; children (Pip, Lucan) are stakes and non-combat NPCs; no combat graft surgery on anyone under 18 |
| DEC-LORE-23 | 0 HP = downed; after a battle a downed founder is wounded in Act 1; permadeath and capture start in Act 2; the Crucible sacrifice is the one scripted Act 1 loss |
| DEC-LORE-24 | No gold: Scrap is the currency; Warded Tallow is upkeep (1 per day of travel or rest; replaces supplies); Rock Salt and the monster organs are stash materials; 0 Tallow = -2 starting AP until restocked; other upkeep versions dropped |
| DEC-LORE-25 | Visual target STAGE1 option B (tilted 32x24 board, zone backdrops, 40 px units, upright props sorted with the units); "32x48 sprite" = figure bounds in the rig's 48x48 frames; the region map stays square top-down; the art bible's hex values win; a skin ramp is added; 16 master hues plus at most one dark and one light step each |
| DEC-LORE-26 | A siege is never a Game Over: a lost siege costs facilities and forces a relocation; no siege in Act 1 |
| DEC-LORE-27 | Doom Clock: a visible calendar from day 1 (the Tithing in six weeks); soft in Acts 1-2, hard in Act 3; travel costs days through the existing travel clock |
| DEC-LORE-28 | Endings: A Preservation (the titan put back into a deep coma), B Ascendancy (merge with the titan's marrow), C Red Martyrdom (the cell gives itself to break the Tithing); "Extinguished" is the failure screen, not an ending; doctrine gates which endings are open |
| DEC-LORE-29 | The corrupted-founder boss is optional and exists only for a captured founder (the Crucible's, or one captured in Act 2) |
| DEC-LORE-30 | Missions are picked on Tess's dead-drop board (a hub NPC offering quests) plus node travel on the district map |
| DEC-LORE-31 | The Crucible sacrifice is chosen from the four founders deployed there (deployment is the decision) |
| DEC-LORE-32 | Bunks: 6 at the start of Act 1 (Bram's "four dry bunks" is fixed); 8 through a Bram forge project the player chooses; later hubs inherit the count; up to 4 deployed |
| DEC-LORE-33 | Prologue = missions 0a-0c (alley, rooftop, bridge): Kaelen alone, Vesper joins on the rooftop, Maren and Hollis are freed from an iron mortuary cart at the bridge; then the hearth scene; then Act 1 mission 1. Proctor Quell's crossroads fight (ENC-EVT-02) is its own Act 1 mission |
| DEC-LORE-34 | Dunstan joins in the bone-mill raid (`sunk_ward` M3); Elyse at the Tallow Vats with Orlov (M4); the first grafting teaser is the ENC-EVT-05 Amalgam interlude in the hub (the "elite Castellan in mission 5" variant is dropped) |
| DEC-LORE-35 | The founders' visible taints are backstory and Act 2 hooks only in Act 1 (no mechanics, 0% corruption): Kaelen's spurs come from an involuntary Inquisition graft trial; Hollis's eye is a crude field graft he regrets |
| DEC-LORE-36 | Strain: brackets 0-24 Dormant, 25-49 Acute Resonance, 50-74 Flesh-Tension, 75-99 Aberrant Edge, 100 Rupture; the crisis is the 3-outcome table rolled at round end; -10 per round without an Inoculated action, Active Guard vents 10 more; reset to the floor after battle. Calcification advances through Flesh Breach and Tier III grafts only; stages I-V with one name set |
| DEC-LORE-37 | Combat rules: a stagger cancels the action and the unit's remaining blocks that round; poise from a formula (CONS-based plus armour), refilled each round, a repeat break in the same or next round costs double; the engine's initiative order (no mutual trades); the hyperbolic armour curve with the lore's armour rescaled to it; deck = 2 base moves + 3 class actives + 2 passives + 1 keystone (8 slots); AP cost decoupled from tier (`ap_cost`, default = level); timeline strip at the top. Cut: evasion and accuracy rolls, facing, backstab and stealth cones, the d100 crisis, mutual trades, the multi-track ghost scrubber (impact badges on the existing strip first) |
| DEC-LORE-38 | One phase name set: Act 1 "The Young Vanguard", Act 2 "The First Rupture", Act 3 "The Awakening Abyss" |
| DEC-LORE-39 | One hub through the campaign (the Sluice, renamed per act only if the story needs a forward base); the Scribe's Crypt and the Weeping Redoubt stay proposal forward bases |
| DEC-LORE-40 | Names: every on-stage character has a distinct first letter and sound within the founders and hub cast and no near-collision anywhere; the registry ([cast_registry.md](../background/cast_registry.md)) is canon. Duplicate backgrounds split: Aline keeps the dissection hall and the draught discovery, Maren is a defrocked scriptorium scribe; Vesper is a curfew runner and thief, Tess the cell's courier; Bram from the Syndicate forge, Dunstan from the bone-mill, Lucan from the Tallow Vats. Adults who avoided communion keep their minds but cannot fight (Bram, Aline, Corvo) |
| DEC-LORE-41 | Layer rule: engine = technology any game needs (rendering, screens, saves, assets, the timeline widget); kit = a shape any story in the genre could fill (campaign meters with thresholds and triggers, unit lifecycle, hub facilities with levels, resource purse, exchange recipes, ruleset options, event triggers, mission board, vignette player); game = named by the story, mostly data filling kit hooks, game code only for mechanics nothing else could use. Every lore system line carries a tag `[engine]`, `[kit hook]`, `[data]` or `[game code]` |
| DEC-LORE-42 | The library lives in `games/emberwake/docs/lore/` and follows CLAUDE.md: repo-relative links, a `Status:` line per doc (canon, proposal, archive), markdownlint-clean, at most about 300 lines / 25 KB per doc; CANON.md wins over the domain docs; the raw export moved to `archive/`; supersedes DEC-LORE-06 |

## 4. Engine mapping (corrected 2026-10-25)

The 2026-10-24 matrix named types that do not exist (`ClassCatalog`, `PartyState` as a 4-person cell, `ResourceState`,
`LocationKind::Hub`, `RegionKind`, `FactionCatalog`) and called the combat sequential. The real carriers:

| Lore concept | Carried by today | Change |
| --- | --- | --- |
| Six classes | `games/emberwake/data/catalog/classes.json`, typed `emberwake_data::ClassDef`; warrior, ranger, rogue, brawler, priest, mage | `[data]`: Vanguard, Arbalestier, Gutter-Stalker, Slag-Brute, Heretic Acolyte, Ichor Alchemist as the new worlds' names (how a world renames a class is open) |
| The cell | `emberwake_world::state::PartyState` (roster unbounded, `PARTY_MAX` 4 active), founders as world characters (RECRUIT1) | `[data]` the founders; `[kit hook]` the bunk cap |
| HP, AP, Strain, Calcification | HP and AP on the battle unit (`emberwake_battle`), stored HP `UnitState.hp_lost` (HEAL1) | `[kit hook]` a per-unit battle meter (Strain) and a per-unit stage track (Calcification) |
| Hub | a world location the party walks (`emberwake_gui` `region.rs`); `hub.rs` `Hub` is the in-game menu, not a place | `[data]` the Undercroft location; `[kit hook]` facilities with levels |
| Regions and districts | world `regions` and `locations` in `games/emberwake/data/worlds/<id>/`, the node graph in `worldmap.rs` + `emberwake_world::travel` | `[data]` world `sunk_ward` |
| Factions | world kind `factions`, standing in flags `rep.<world>.<faction>` (RepRules) | `[data]` one row per faction of the CANON glossary (ids open) |
| Combat | `emberwake_battle::resolver` (6 blocks per round, simultaneous, AP paid at start, effect on the last block, highest initiative first), drawn by `aegis_ui::timeline` | `[kit hook]` the "timeline" ruleset option |

## 5. Open queries of 2026-10-24 (answered)

1. Engine migration for combat: answered 2026-10-25 by the user: the timeline is an evolution of the existing resolver
   as a switchable ruleset (DEC-LORE-19). (The query named `engine/combat`, which does not exist; combat is
   `games/emberwake/crates/emberwake_battle`.)
2. The Ashlands data: answered 2026-10-25 by the user: the Ashlands stay as the kit's sample and parity world; the new
   campaign is new worlds, first `sunk_ward` (DEC-LORE-18).
