# Emberwake: A Scarlet Lullaby - lore library

Status: canon (index, 2026-10-25). The title is **Emberwake: A Scarlet Lullaby** (user 2026-10-25); the codename,
crates and paths stay `emberwake`. "The Low Sleep" stays the in-world name of the titan's slumber.

This library is the top-down design brainstorm for the game's new setting: theme first, then narrative and world, then
systems. It is design input, not code or data. [CANON.md](CANON.md) is the single source the other docs follow; where a
doc disagrees with it, CANON wins.

## 1. How to use this library

1. The lore is the game's design input. It changes nothing in the build by itself: a feature becomes a spec and a task
   row through the planner, and builders work from the spec.
2. [CLAUDE.md](../../../../CLAUDE.md) governs process (roles, lanes, gates, commits, the content tool, markdown rules).
3. Settled decisions go to `games/emberwake/docs/decisions/lore.md`; the planner writes it. The brainstorm's own log
   is [worldbuilding_decisions.md](specs/worldbuilding_decisions.md) (DEC-LORE rows).
4. `legacy/` stays frozen. The Ashlands content (`games/emberwake/data/worlds/ashlands/`) stays as the kit's sample and
   parity world on the "classic" combat ruleset; the new campaign is new worlds in `games/emberwake` (first world
   `sunk_ward`) on the "timeline" ruleset.
5. Layer tags: every system or feature line says where its change lands (where the hooks live today:
   [rpg_kit_and_system_architecture.md](specs/rpg_kit_and_system_architecture.md)):

   - `[engine]`: technology any game needs (rendering, screens, saves, assets, the timeline widget);
   - `[kit hook]`: a shape any story in the genre could fill (campaign meters with thresholds and triggers, the unit
     lifecycle, hub facilities with levels, a resource purse, exchange recipes, ruleset options, event triggers, a
     mission board, a vignette player);
   - `[data]`: named by the story and filling a kit hook (most of the game);
   - `[game code]`: a mechanic named by the story that no other game could use.

6. Open questions: every gap and open question is in [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) with a stable id
   (Q-01 ...), options, a recommendation, where it lives and who decides; its section 1 lists the questions to decide
   first, its section 2 the planner picks to confirm. Domain docs keep their own "Gaps" sections as pointers.
7. Status words: each doc has a `Status:` line: `canon` (consistent with CANON.md), `proposal` (Acts 2-3 and later
   systems, not locked), `archive` (kept for history, never a source) or `open` (the open-questions list).
8. Writing rules: repo-relative links, plain statements (no marketing tone), markdownlint-clean
   (`bash tools/fc.sh docs`), at most about 300 lines / 25 KB per doc (split by domain behind a short index). History is
   append-only: superseded text is marked or moved to `archive/`, never deleted.

## 2. Folder layout

```text
games/emberwake/docs/lore/
  README.md        this index
  CANON.md         the numbers sheet, campaign spine, endings, glossary, open questions
  OPEN_QUESTIONS.md  every open question with an id (index: decide first, planner picks)
  STORY_GUIDE.md   the story, who's who, story elements, reveal timeline and story gaps in one place
  questions/       the open questions by area (two parts)
  background/      cast and places, setting, factions, geography, the three district maps, NPCs, the central truth
  narrative/       acts, prologue, founders, choices, fail states, onboarding
  systems/         combat, corruption, classes (index + parts), loadouts, rules, balance, roster, loop, screens,
                   bestiary (index + parts), hub, items
  specs/           decision log, kit and system architecture, systems roadmap, art and visual style, naming guide
  archive/         superseded sources (master_specification.md)
```

## 3. Catalog

### 3.1 Canon and specs

| Doc | Status | What it holds |
| --- | --- | --- |
| [CANON.md](CANON.md) | canon | The cell, the numbers sheet (combat, Strain, Calcification, roster, resources, meters), the campaign spine, endings, glossary, open questions |
| [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) | open | Index of every open question (Q-01..Q-91): decide first, planner picks to confirm (P-01..P-29) |
| [STORY_GUIDE.md](STORY_GUIDE.md) | canon summary | The story in one page, belief against truth, the acts, Nemeth traced, story elements, reveal timeline, story gaps G-01..G-30 |
| - part: [open_world_and_campaign.md](questions/open_world_and_campaign.md) | open | Story and world, campaign and meters, roster and lifecycle, hub and economy (Q-01..Q-46) |
| - part: [open_systems_and_scope.md](questions/open_systems_and_scope.md) | open | Combat and rules, Strain and Calcification, items, enemies, art, slice scope, process (Q-47..Q-91) |
| [worldbuilding_decisions.md](specs/worldbuilding_decisions.md) | canon | The brainstorm's decision log: DEC-LORE-01..16 (2026-10-24, superseded parts marked) and the 2026-10-25 decisions from DEC-LORE-17 on |
| [rpg_kit_and_system_architecture.md](specs/rpg_kit_and_system_architecture.md) | canon | The layer rule; which existing system carries each lore feature, which kit hooks to add, what is game code; corrected module names |
| [full_game_systems_roadmap.md](specs/full_game_systems_roadmap.md) | canon (Acts 2-3 sections proposal) | The first playable slice "Sunk Ward, first night", milestones M0-M4 with layer tags, the Acts 2-3 systems |
| [art_and_visual_style.md](specs/art_and_visual_style.md) | canon | One palette (16 hues, steps, skin ramp), the STAGE1 option B board, sprite bounds, decals, animation, UI and audio direction |
| [naming_and_asset_guide.md](specs/naming_and_asset_guide.md) | canon | How named things are named (ids, asset briefs, faction and role sounds, distinctness, one source of truth) |
| [master_specification.md](archive/master_specification.md) | archive | The raw design export of 2026-10-24; superseded by the domain docs and CANON.md |

### 3.2 Background (`background/`)

| Doc | Status | What it holds |
| --- | --- | --- |
| [cast_registry.md](background/cast_registry.md) | canon | Every named character and place: role, age, one-line background, first appearance, former names |
| [central_truth.md](background/central_truth.md) | canon (Option A) | Spoiler sheet: what the Red Tithing really does and how the endings relate (Option A chosen, Q-01) |
| [who_is_who.md](background/who_is_who.md) | canon summary | Every named character's want, first appearance, arc per act and fate (part of the story guide) |
| [tithing_options.md](background/tithing_options.md) | proposal | The Great Tithing's story context: what commoners know, why now, precedent (options for Q-04 / Q-16) |
| [cosmology_and_setting.md](background/cosmology_and_setting.md) | canon | The Low Sleep titan, the Shroud, the Stagnation of Mortality, the lexicon |
| [factions_and_sociology.md](background/factions_and_sociology.md) | canon | Sovereign Clerisy, Iron Inquisition, Castellan Watch, Tallow Guild, Fourth Kiln Syndicate, the Spent, the Severed Root, the doctrines |
| [world_geography_and_hub.md](background/world_geography_and_hub.md) | canon | Cinderreach's three strata, the Sluice Undercroft and its facilities, the hazard list; forward bases as proposal |
| [sunk_ward_campaign_map.md](background/sunk_ward_campaign_map.md) | canon | Act 1: the Sunk Ward node graph, route rules, the Alert meter, missions M1-M5 and the interlude (`ENC-EVT-01..06`) |
| [upper_bastion_campaign_map.md](background/upper_bastion_campaign_map.md) | proposal | Act 2: the High Bastion graph, rooftop and street routes, culling wagons, `ENC-ACT2-01..06` |
| [mantle_fissure_campaign_map.md](background/mantle_fissure_campaign_map.md) | proposal | Act 3: the Mantle Fissure graph, the hard Doom Clock, tremors, `ENC-ACT3-01..06`, the endings' stage |
| [npcs_and_questlines.md](background/npcs_and_questlines.md) | canon | The hub cast's quests and the city's cast (regime, guild, neutrals, commoners) |

### 3.3 Narrative (`narrative/`)

| Doc | Status | What it holds |
| --- | --- | --- |
| [campaign_narrative_structure.md](narrative/campaign_narrative_structure.md) | canon | The three acts: The Young Vanguard, The First Rupture, The Awakening Abyss |
| [new_game_onboarding_and_intro.md](narrative/new_game_onboarding_and_intro.md) | canon | The intro vignette, the cold open in the rain, prologue missions 0a-0c |
| [prologue_and_starter_squad_lore.md](narrative/prologue_and_starter_squad_lore.md) | canon | Dossiers of Kaelen, Vesper, Maren and Hollis; the hearth scene |
| [recruits_and_prologue_architecture.md](narrative/recruits_and_prologue_architecture.md) | canon | How the prologue converges on the hub; guests; Living Martyrs (Act 2 proposal) |
| [six_founders_and_phase_progression.md](narrative/six_founders_and_phase_progression.md) | canon | The six founders and where each joins in Act 1; the first grafting teaser; the doctrine divide |
| [narrative_choices_and_dilemmas.md](narrative/narrative_choices_and_dilemmas.md) | canon | The Crucible sacrifice and the later dilemmas (doctrine, Living Martyrs, siege priority, the Tithing) |
| [narrative_agency_and_fail_states.md](narrative/narrative_agency_and_fail_states.md) | canon | The ensemble rule, downed and wounded founders, the Extinguished failure screen |

### 3.4 Systems (`systems/`)

The two large docs are split by domain: the class trees and the bestiary are short indexes linking their parts
(`class_*.md`, `bestiary_*.md`, listed under their index below); link the index, not a part.

| Doc | Status | What it holds |
| --- | --- | --- |
| [combat_and_timeline.md](systems/combat_and_timeline.md) | canon | The "timeline" ruleset: 6 blocks per round, speed tiers as relative windups, poise and breaks, Active Guard, impact badges |
| [magic_and_corruption.md](systems/magic_and_corruption.md) | canon | Cold Steel and Inoculated, Strain brackets, the 1d3 crisis, Calcification stages I-V |
| [classes_and_skill_trees.md](systems/classes_and_skill_trees.md) | canon | Index of the six classes' Cold Steel and Inoculated trees and the founders' signatures (split into parts) |
| - part: [class_vanguard.md](systems/class_vanguard.md) | canon (Inoculated proposal) | The Vanguard (`warrior`) and Kaelen |
| - part: [class_gutter_stalker.md](systems/class_gutter_stalker.md) | canon (Inoculated proposal) | The Gutter-Stalker (`rogue`) and Vesper |
| - part: [class_arbalestier.md](systems/class_arbalestier.md) | canon (Inoculated proposal) | The Arbalestier (`ranger`) and Hollis |
| - part: [class_slag_brute.md](systems/class_slag_brute.md) | canon (Inoculated proposal) | The Slag-Brute (`brawler`) and Dunstan |
| - part: [class_heretic_acolyte.md](systems/class_heretic_acolyte.md) | canon (Inoculated proposal) | The Heretic Acolyte (`priest`) and Maren |
| - part: [class_ichor_alchemist.md](systems/class_ichor_alchemist.md) | canon (Inoculated proposal) | The Ichor Alchemist (`mage`) and Elyse |
| [skill_acquisition_and_loadout_systems.md](systems/skill_acquisition_and_loadout_systems.md) | canon | Learning skills at the Hearth and the Slab; the 8-slot deck |
| [rules_engine_architecture.md](systems/rules_engine_architecture.md) | canon | The battle math against the real rules: armour curve, Strain gain, poise, interrupts |
| [risk_reward_and_corruption_balance.md](systems/risk_reward_and_corruption_balance.md) | canon | The balance axes: AP density, reliability against burst, Strain surfing, roster lifespan |
| [roster_expansion_and_lifecycle.md](systems/roster_expansion_and_lifecycle.md) | canon | Bunks 6 to 8, recruits, downed / wounded / captured / dead / retired |
| [gameplay_loop_and_ui_prototype.md](systems/gameplay_loop_and_ui_prototype.md) | canon | The expedition loop, the combat loop, the Dojo prototype |
| [screen_architecture_and_separation_plan.md](systems/screen_architecture_and_separation_plan.md) | canon | Which screens the game needs and which existing screens carry them |
| [enemy_bestiary_and_ai.md](systems/enemy_bestiary_and_ai.md) | canon | Index of the bestiary, named bosses and encounter events (split into parts) |
| - part: [bestiary_castellan_watch.md](systems/bestiary_castellan_watch.md) | proposal (names canon) | Castellan Footman, Watch Crossbowman, Bailiff Enforcer; Captain Vane |
| - part: [bestiary_iron_inquisition.md](systems/bestiary_iron_inquisition.md) | proposal (names canon) | Flagellant, Purifier, Proctor; Proctor Quell, Inquisitor-Prelate Nemeth |
| - part: [bestiary_tallow_guild.md](systems/bestiary_tallow_guild.md) | proposal (names canon) | Dredger, Render-Beast, Guild Carver; Guild-Master Orlov, Overseer Grendel |
| - part: [bestiary_aberrations.md](systems/bestiary_aberrations.md) | proposal (names canon) | Ash-Crawler, Calcified Husk, Marrow Horror; the Calcified Amalgam |
| - part: [bestiary_encounters_act1.md](systems/bestiary_encounters_act1.md) | canon (rosters proposal) | ENC-PRO-0A..0C and ENC-EVT-01..06 |
| [safehouse_economy_and_triage.md](systems/safehouse_economy_and_triage.md) | canon | The Undercroft's facilities, Scrap, Warded Tallow, Rock Salt and organs, hub screens |
| [items_gear_and_progression.md](systems/items_gear_and_progression.md) | canon | Weapon families, armour profiles, pouch consumables, monster parts, named relics |

## 4. Quick reference (matches CANON.md section 2)

| Parameter | Canon value | Layer |
| --- | --- | --- |
| Title | Emberwake: A Scarlet Lullaby (codename `emberwake`) | - |
| Acts | Act 1 The Young Vanguard (the Sunk Ward), Act 2 The First Rupture (the High Bastion, proposal), Act 3 The Awakening Abyss (the Mantle Fissure, proposal) | [data] |
| Ages | Playable operatives 18-24; the Unsealed are everyone under 25; children are stakes, never fighters; no combat graft surgery under 18 | [data] |
| Founders | Six in Act 1 (Kaelen, Vesper, Maren, Hollis, Dunstan, Elyse); the first slice has five (Elyse joins later) | [data] |
| Squad | Up to 4 deployed (`PARTY_MAX` 4) | [kit hook] exists |
| Bunks | 6 at the start of Act 1 (4 filled after the prologue); 8 through Bram's project | [kit hook] |
| Ruleset | "timeline" for the new worlds, "classic" for the Ashlands | [kit hook] |
| Round | 6 blocks, all sides resolved simultaneously; timeline strip at the top of the battle screen | [kit hook] exists, strip [engine] |
| AP | 10 max, +4 per round; an action pays when it starts and lands on its last block | [kit hook] exists |
| AP cost | Skill field `ap_cost`, default = skill level; Cold Steel 2-4, Inoculated 1-2 plus Strain | [data] |
| Speed tiers | Relative windups: Snap 1 block, Standard 2, Heavy 3-4; optional recovery blocks; no fixed tick anchors | [data] (`blocks`), recovery [kit hook] |
| Same-block impacts | Engine initiative order, no mutual trades | [kit hook] exists |
| Poise | CONS x 2 + armour poise; refills each round; a break cancels the windup and the unit's remaining blocks that round; a repeat break costs double | [kit hook] |
| Armour | reduction = rating / (rating + 50); lore armour points x 2 = rating | [kit hook] exists |
| Deck | 8: Move, Active Guard (basics, outside the loadout), 3 class actives, 2 passives, 1 keystone | [kit hook] (passive and keystone slots new, Q-48) |
| Gear | Pouch 3; MAIN, OFF, HEAD, BODY, ACC1, ACC2; graft slots are an Act 2 proposal | [kit hook] exists |
| Strain (Act 2 on) | 0-24 Dormant, 25-49 Acute Resonance, 50-74 Flesh-Tension, 75-99 Aberrant Edge, 100 Rupture; -10 per round without an Inoculated action, -10 more under Active Guard; resets after battle | [kit hook] meter, [data] names |
| Crisis | 1d3 at round end: Unchecked Spasm, Biological Collapse, Flesh Breach | [game code] unless the meter's triggers can express it |
| Calcification (Act 2 on) | I Vein-Blackening, II Calloused Chitin, III Sensory Rot, IV The Shakes, V Terminal Drift; advances through Flesh Breach and Tier III grafts | [kit hook] track, [data] stages |
| Founder at 0 HP | Downed; wounded after the battle in Act 1; permadeath and capture from Act 2; the Crucible is the one scripted Act 1 loss | [kit hook] |
| Resources | Scrap (currency, no gold), Warded Tallow (1 per day of travel or rest; at 0: -2 starting AP), Rock Salt and Aberrant Organs (stash) | [kit hook] purse, [data] names |
| Inquisition Alert | 0-30 Guarded, 31-70 Suspicious, 71-99 Lockdown, 100 Breach; no siege in Act 1; a lost siege costs facilities and a relocation | [kit hook] meter |
| Doom Clock | Visible from Act 1 day 1 at 42 days; soft in Acts 1-2, hard in Act 3 | [kit hook] meter |
| Fail screen | Extinguished: no living operative left, or the Act 3 clock runs out (not an ending) | [kit hook] |
| Endings (proposal) | A Preservation, B Ascendancy, C Red Martyrdom; doctrine gates them | [data] |
| Board and art | STAGE1 option B (tilted 32x24 cells, 40 px units, upright props); 48x48 rig frames, figures up to 32x48; region map square top-down; canvas 640x360 | [engine] + [game code] |
