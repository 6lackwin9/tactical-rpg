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
7. Status words and maturity tags: each doc has a `Status:` line (`canon`, `proposal`, `archive` or `open`), and
   [CANON.md](CANON.md) tags individual elements as `[LOCKED]` (narrative spine + gameplay-linked mechanics),
   `[PROVISIONAL]` (buildable Design defaults awaiting user lock), `[PLACEHOLDER]` (unfinalized cast/faction backstories,
   names, voices and visual look & feel kept data-driven) or `[MISSING]` (unauthored slots).
8. Writing rules: repo-relative links, plain statements (no marketing tone), markdownlint-clean
   (`bash tools/fc.sh docs`), at most about 300 lines / 25 KB per doc (split by domain behind a short index). History is
   append-only: superseded text is marked or moved to `archive/`, never deleted.

## 2. Folder layout

```text
games/emberwake/docs/lore/
  README.md        this index
  CANON.md         master worldbuilding, history timeline, factions, strata, cast dossiers, systems, named items/organs, glossary & Lore->Asset pipeline
  OPEN_QUESTIONS.md  every open question with an id (index: decide first, planner picks)
  STORY_GUIDE.md   the story, who's who, story elements, reveal timeline and story gaps in one place
  modules/         lore modules: independent lore and events, decided alone, connected later (index README.md)
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
| [CANON.md](CANON.md) | master blueprint (`[LOCKED]` / `[PROVISIONAL]` / `[PLACEHOLDER]`) | Cosmology, chronological history -> mission linkage, factions, strata/hub, full cast dossiers, campaign spine, systems, named relics/keepsakes/organs, glossary, `CL-01..07` & Lore -> Asset pipeline |
| [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) | open | Index of every open question (Q-01..Q-91): decide first, planner picks to confirm (P-01..P-29) |
| [STORY_GUIDE.md](STORY_GUIDE.md) | `[LOCKED]` spine / `[PLACEHOLDER]` copy | The story in one page, belief against truth, the acts, Nemeth traced, story elements, reveal timeline, story gaps G-01..G-33 |
| [modules/README.md](modules/README.md) | open | Lore modules: the format, and the seeds LM-01..LM-17 from the `3 lore` questions and story gaps |
| - part: [open_world_and_campaign.md](questions/open_world_and_campaign.md) | open | Story and world, campaign and meters, roster and lifecycle, hub and economy (Q-01..Q-46) |
| - part: [open_systems_and_scope.md](questions/open_systems_and_scope.md) | open | Combat and rules, Strain and Calcification, items, enemies, art, slice scope, process (Q-47..Q-91) |
| [worldbuilding_decisions.md](specs/worldbuilding_decisions.md) | canon | The brainstorm's decision log: DEC-LORE-01..16 (2026-10-24, superseded parts marked) and the 2026-10-25 decisions from DEC-LORE-17 on |
| [rpg_kit_and_system_architecture.md](specs/rpg_kit_and_system_architecture.md) | canon | The layer rule; which existing system carries each lore feature, which kit hooks to add, what is game code; corrected module names |
| [full_game_systems_roadmap.md](specs/full_game_systems_roadmap.md) | canon (Acts 2-3 sections proposal) | The first playable slice "Sunk Ward, first night", milestones M0-M4 with layer tags, the Acts 2-3 systems |
| [art_and_visual_style.md](specs/art_and_visual_style.md) | `[LOCKED]` geometry / `[PLACEHOLDER]` aesthetics | One palette (16 hues, steps, skin ramp), the STAGE1 option B board, sprite bounds, decals, animation, UI and audio direction |
| [naming_and_asset_guide.md](specs/naming_and_asset_guide.md) | canon | How named things are named (ids, asset briefs, faction and role sounds, distinctness, one source of truth) |
| [master_specification.md](archive/master_specification.md) | archive | The raw design export of 2026-10-24; superseded by the domain docs and CANON.md |

### 3.2 Background (`background/`)

| Doc | Status | What it holds |
| --- | --- | --- |
| [cast_registry.md](background/cast_registry.md) | `[LOCKED]` roles / `[PLACEHOLDER]` names & bios | Every named character and place: role, age, one-line background, first appearance, former names |
| [central_truth.md](background/central_truth.md) | `[LOCKED]` (Option A) | Spoiler sheet: what the Red Tithing really does, the precedent and the Clerisy's founding, and how the endings relate (Option A chosen, Q-01) |
| [who_is_who.md](background/who_is_who.md) | `[LOCKED]` spine / `[PLACEHOLDER]` wants | Every named character's want, first appearance, arc per act and fate (part of the story guide) |
| [tithing_options.md](background/tithing_options.md) | `[LOCKED]` (user 2026-10-25): option B with b2 | The Great Tithing's story context: what commoners know, why now, precedent |
| [cosmology_and_setting.md](background/cosmology_and_setting.md) | `[LOCKED]` rules / `[PLACEHOLDER]` lexicon | The Low Sleep titan, the Shroud, the Stagnation of Mortality, the lexicon |
| [factions_and_sociology.md](background/factions_and_sociology.md) | `[LOCKED]` roles / `[PLACEHOLDER]` aesthetics | Sovereign Clerisy, Iron Inquisition, Castellan Watch, Tallow Guild, Fourth Kiln Syndicate, the Spent, the Severed Root, the doctrines |
| [world_geography_and_hub.md](background/world_geography_and_hub.md) | `[LOCKED]` strata & hub / `[PROVISIONAL]` hazards | Cinderreach's three strata, the Sluice Undercroft and its facilities, the hazard list; forward bases as story-only |
| [sunk_ward_campaign_map.md](background/sunk_ward_campaign_map.md) | canon (`[PLACEHOLDER]` copy) | Act 1: the Sunk Ward node graph, route rules, the Alert meter, missions M1-M5 and the interlude (`ENC-EVT-01..06`) |
| [upper_bastion_campaign_map.md](background/upper_bastion_campaign_map.md) | `[PROVISIONAL]` (`2.4/2.6` `[LOCKED]`) | Act 2: the High Bastion graph, rooftop and street routes, culling wagons, `ENC-ACT2-01..06` |
| [mantle_fissure_campaign_map.md](background/mantle_fissure_campaign_map.md) | `[PROVISIONAL]` (`3.2/3.4-3.6` `[LOCKED]`) | Act 3: the Mantle Fissure graph, the hard Doom Clock, tremors, `ENC-ACT3-01..06`, the endings' stage |
| [npcs_and_questlines.md](background/npcs_and_questlines.md) | `[LOCKED]` facilities / `[PLACEHOLDER]` personas | The hub cast's quests and the city's cast (regime, guild, neutrals, commoners) |

### 3.3 Narrative (`narrative/`)

| Doc | Status | What it holds |
| --- | --- | --- |
| [campaign_narrative_structure.md](narrative/campaign_narrative_structure.md) | `[LOCKED]` spine | The three acts: The Young Vanguard, The First Rupture, The Awakening Abyss |
| [new_game_onboarding_and_intro.md](narrative/new_game_onboarding_and_intro.md) | `[LOCKED]` flow / `[PLACEHOLDER]` copy | The intro vignette, the cold open in the rain, prologue missions 0a-0c |
| [prologue_and_starter_squad_lore.md](narrative/prologue_and_starter_squad_lore.md) | `[LOCKED]` spine / `[PLACEHOLDER]` dossiers | Dossiers of Kaelen, Vesper, Maren and Hollis; the hearth scene |
| [recruits_and_prologue_architecture.md](narrative/recruits_and_prologue_architecture.md) | `[LOCKED]` lifecycle | How the prologue converges on the hub; guests; Living Martyrs |
| [six_founders_and_phase_progression.md](narrative/six_founders_and_phase_progression.md) | `[LOCKED]` progression / `[PLACEHOLDER]` bios | The six founders and where each joins in Act 1; the first grafting teaser; the doctrine divide |
| [narrative_choices_and_dilemmas.md](narrative/narrative_choices_and_dilemmas.md) | `[LOCKED]` gates / `[PROVISIONAL]` numbers | The Crucible sacrifice and the later dilemmas (doctrine, Living Martyrs, siege priority, the Tithing) |
| [narrative_agency_and_fail_states.md](narrative/narrative_agency_and_fail_states.md) | `[LOCKED]` rules | The ensemble rule, downed and wounded founders, the Extinguished failure screen |

### 3.4 Systems (`systems/`)

The two large docs are split by domain: the class trees and the bestiary are short indexes linking their parts
(`class_*.md`, `bestiary_*.md`, listed under their index below); link the index, not a part.

| Doc | Status | What it holds |
| --- | --- | --- |
| [combat_and_timeline.md](systems/combat_and_timeline.md) | `[LOCKED]` ruleset | The "timeline" ruleset: 6 blocks per round, speed tiers as relative windups, poise and breaks, Active Guard, impact badges |
| [magic_and_corruption.md](systems/magic_and_corruption.md) | `[LOCKED]` mechanics | Cold Steel and Inoculated, Strain brackets, the 1d3 crisis, Calcification stages I-V |
| [classes_and_skill_trees.md](systems/classes_and_skill_trees.md) | `[LOCKED]` roles / `[PROVISIONAL]` trees | Index of the six classes' Cold Steel and Inoculated trees and the founders' signatures (split into parts) |
| - part: [class_vanguard.md](systems/class_vanguard.md) | `[LOCKED]` kit / `[PLACEHOLDER]` founder bio | The Vanguard (`warrior`) and Kaelen |
| - part: [class_gutter_stalker.md](systems/class_gutter_stalker.md) | `[LOCKED]` kit / `[PLACEHOLDER]` founder bio | The Gutter-Stalker (`rogue`) and Vesper |
| - part: [class_arbalestier.md](systems/class_arbalestier.md) | `[LOCKED]` kit / `[PLACEHOLDER]` founder bio | The Arbalestier (`ranger`) and Hollis |
| - part: [class_slag_brute.md](systems/class_slag_brute.md) | `[LOCKED]` kit / `[PLACEHOLDER]` founder bio | The Slag-Brute (`brawler`) and Dunstan |
| - part: [class_heretic_acolyte.md](systems/class_heretic_acolyte.md) | `[LOCKED]` kit / `[PLACEHOLDER]` founder bio | The Heretic Acolyte (`priest`) and Maren |
| - part: [class_ichor_alchemist.md](systems/class_ichor_alchemist.md) | `[LOCKED]` kit / `[PLACEHOLDER]` founder bio | The Ichor Alchemist (`mage`) and Elyse |
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

## 4. Quick reference (matches CANON.md sections 1-3)

| Parameter | Canon value | Layer |
| --- | --- | --- |
| Title | Emberwake: A Scarlet Lullaby (`[LOCKED]`; codename `emberwake`) | - |
| Acts | Act 1 The Young Vanguard (Sunk Ward, `[LOCKED]`), Act 2 The First Rupture (High Bastion, spine `[LOCKED]`, lineup `[PROVISIONAL]`), Act 3 The Awakening Abyss (Mantle Fissure, spine `[LOCKED]`, lineup `[PROVISIONAL]`) | `[data]` |
| Ages | Playable operatives 18-24 (`[LOCKED]`); Unsealed under 25; children are stakes, never fighters; no combat graft surgery under 18 | `[data]` |
| Founders & cast | 6 Founder class roles/weapon lineups `[LOCKED]` (`kaelen`, `vesper`, `maren`, `hollis`, `dunstan`, `elyse`; `elyse` joins at M4); all names, ages, backstories, voices and look & feel are `[PLACEHOLDER]` | `[data]` |
| Squad & bunks | Up to 4 deployed (`PARTY_MAX` 4); 6 bunks in Act 1 (4 filled after prologue), 8 through Act 2 Hearth project | `[kit hook]` |
| Ruleset | `timeline` for `sunk_ward`, `classic` for Ashlands | `[kit hook]` |
| Round & AP | 6 simultaneous blocks, timeline strip at top; 12 max AP (`start_ap: 10` on timeline fights), +4 regen/round; Rest banks +0.5 AP/block (cap +2) + 5 Poise/block | `[kit hook]` + `[engine]` strip |
| Intents & Channel | 5 Round Intents (`Assault`, `Guard`, `Support`, `Maneuver`, `Field`) + Act 2 `0 AP` converting `Channel` toggle for Strain carriers | `[kit hook]` + `[data]` |
| Poise | `CONS * 2 + gear poise + passives`, quantized into 5 discrete pips; break cancels windup + remaining blocks, drops guard, queues `stagger` (locks blocks 1 & 4 next round) | `[kit hook]` |
| Armour & Guard | `Active Guard` 35% DR (`Brace` 20% / 2 rounds), 0.90 cap with cover; armour `rating / (rating + 50)`, binary marrow `pierce` | `[kit hook]` |
| Deck & gear | 3 actives + 2 passives + 1 keystone (5 universal basics + 1 class basic free); Pouch 3; 6 gear slots + 4 Act 2 `body` graft slots; Act 2 weapon instances with durability & mastery techniques | `[kit hook]` |
| Strain (Act 2 on) | 0-24 Dormant, 25-49 Acute Resonance, 50-74 Flesh-Tension, 75-99 Aberrant Edge, 100 Rupture; -10 decay without marrow (-10 more under Active Guard) to graft-count floor (`0/5/15/30/50`) | `[kit hook]` + `[data]` |
| Crisis | Seeded 1d3 `roll_table` at round end (resets Strain to 50): Unchecked Spasm, Biological Collapse, Flesh Breach | `[kit hook]` + `[data]` |
| Calcification (Act 2 on) | Cumulative `unit_tracks` I-V (Vein-Blackening .. Terminal Drift); advances via Flesh Breach or Tier III grafts; undone in battle by `unmark`, at hub by bone-scraping | `[kit hook]` + `[data]` |
| Founder at 0 HP | Downed; wounded (-20% max HP until rest) in Act 1; bleed-out, capture and permadeath from Act 2; M5 Crucible is the one scripted Act 1 winch capture | `[kit hook]` |
| Resources | Scrap (currency + common salvage), Warded Tallow (1/day; at 0: -2 starting AP), Rock Salt, Quenched Cinder Salt, Aberrant Organs, Fine/Themed Salvage | `[kit hook]` + `[data]` |
| Campaign meters | Inquisition Alert (0-100, Act 2+ floor 30; Act 1 Breach ends run, Act 2-3 Breach starts hub room-chain siege), Doom Clock (42d, soft Acts 1-2, hard Act 3), Doctrine (`-100..+100`) | `[kit hook]` |
| Endings & fail | Endings A Preservation, B Ascendancy, C Red Martyrdom (`[LOCKED]` gates, `[MISSING]` prose); `Extinguished` failure screen | `[kit hook]` + `[data]` |
| Board and art | `[LOCKED]` geometry: `640x360` canvas, `STAGE1` `32x24` cells (`12x9`), `40 px` figures in `48x48` frames (`96x96` on `2x2` footprints); `[PLACEHOLDER]` palettes, portraits and costumes | `[engine]` + `[data]` |
