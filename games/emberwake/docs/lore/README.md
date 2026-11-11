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
7. Status words and maturity tags: each doc has a `Status:` line, and [CANON.md](CANON.md) tags elements as `[LOCKED]`
   (macro aesthetic, lexicon, history, factions, all cast backstories & visual hooks, 3-tier taint, Endings `A/B/C`,
   relics/ossuary growths/consumables, and gameplay-linked mechanics — locked `2026-11-10`), `[PROVISIONAL]` (unreviewed
   tuning numbers & Reversible Design Calls in `docs/DESIGN_REVIEW.md`), or `[MISSING]` (unauthored content/art files
   queued in `READY.md` & `ART_UI_REQUESTS.md`).
8. Writing rules: repo-relative links, plain statements (no marketing tone), markdownlint-clean
   (`bash tools/fc.sh docs`), at most about 300 lines / 25 KB per sibling doc (split by domain behind a short index).
   History is append-only: superseded text is marked or moved to `archive/`, never deleted.

## 2. Folder layout

```text
games/emberwake/docs/lore/
  README.md        this index
  CANON.md         master worldbuilding, history timeline, factions, strata, cast dossiers, visual hooks, systems, ossuary items & endings
  OPEN_QUESTIONS.md  open combat/tuning/scope question index (lore questions Q-01..Q-40 locked 2026-11-10)
  STORY_GUIDE.md   the story, belief vs. truth, acts, endings, and resolved story ledger G-01..G-33
  modules/         lore modules: independent lore seeds LM-01..LM-17
  questions/       the open questions by area (two parts)
  background/      cast & cohorts, setting, factions, geography, district maps, NPCs, central truth
  narrative/       acts, prologue, founders, choices, cohorts/martyrs, fail states, onboarding
  systems/         combat, corruption, classes (index + parts), loadouts, rules, balance, roster, loop, screens,
                   bestiary (index + parts), hub, items/ossuary growths
  specs/           decision log, kit and system architecture, systems roadmap, art and visual style, naming guide
  archive/         superseded sources (master_specification.md)
```

## 3. Catalog

### 3.1 Canon and specs

| Doc | Status | What it holds |
| --- | --- | --- |
| [CANON.md](CANON.md) | **`[LOCKED]` master blueprint (`2026-11-10`)** | Cosmology, chronological history -> mission linkage, 6 factions & rim-lights, strata/hub, full cast & visual briefs, 3-tier taint, systems, relics/ossuary growths/consumables, Endings `A/B/C`, `CL-01..07` & Asset pipeline |
| [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) | open (tuning/scope) | Index of open questions (`Q-01..Q-91`; lore items locked in `CANON.md`) |
| [STORY_GUIDE.md](STORY_GUIDE.md) | **`[LOCKED]` (`2026-11-10`)** | The story in one page, belief vs. truth, acts, Endings `A/B/C`, and closed story ledger `G-01..G-33` |
| [modules/README.md](modules/README.md) | open | Lore module seeds `LM-01..LM-17` |
| - part: [open_world_and_campaign.md](questions/open_world_and_campaign.md) | open (historical log) | World/campaign/roster/hub question archive (`Q-01..Q-46`) |
| - part: [open_systems_and_scope.md](questions/open_systems_and_scope.md) | open | Combat/rules, Strain/Calcification, items, enemies, art, slice scope (`Q-47..Q-91`) |
| [worldbuilding_decisions.md](specs/worldbuilding_decisions.md) | canon | Brainstorm decision log (`DEC-LORE-01..17+`) |
| [rpg_kit_and_system_architecture.md](specs/rpg_kit_and_system_architecture.md) | canon | Layer boundaries (`engine -> kit -> data -> game code`) |
| [full_game_systems_roadmap.md](specs/full_game_systems_roadmap.md) | canon | Playable slice milestones `M0-M4` & Acts 2-3 systems |
| [art_and_visual_style.md](specs/art_and_visual_style.md) | **`[LOCKED]`** | Soot-Gothic + Early-Industrial Ossuary aesthetic, 16-hue palette, `STAGE1` `32x24` board, sprite bounds & rim-lights |
| [naming_and_asset_guide.md](specs/naming_and_asset_guide.md) | canon | Naming conventions, IDs, silhouette distinctness, single source of truth |
| [master_specification.md](archive/master_specification.md) | archive | Superseded raw export (`2026-10-24`) |

### 3.2 Background (`background/`)

| Doc | Status | What it holds |
| --- | --- | --- |
| [cast_registry.md](background/cast_registry.md) | **`[LOCKED]` (`2026-11-10`)** | Every named character, `18-24` Act 2 cohort, `25+` guest, and place + retired names |
| [central_truth.md](background/central_truth.md) | **`[LOCKED]` (`Option A` + waking crisis)** | Spoiler sheet: Lullaby harvest, First Extinguishers, caldera-waking collapse, and Endings `A/B/C` |
| [who_is_who.md](background/who_is_who.md) | **`[LOCKED]` (`2026-11-10`)** | Every character's motivation, Act 1-3 arc, `M5` winch/keepsake, and fate |
| [tithing_options.md](background/tithing_options.md) | **`[LOCKED]` (`B+b2`)** | Great Tithing precedent & Extinguishing mystery |
| [cosmology_and_setting.md](background/cosmology_and_setting.md) | **`[LOCKED]`** | Low Sleep titan, unbroken Shroud, Broken Rot, Unsealed (`under 25`) vs. Spent (`25+`) |
| [factions_and_sociology.md](background/factions_and_sociology.md) | **`[LOCKED]`** | All 6 factions, monopolies, marrow access, and visual rim-lights |
| [world_geography_and_hub.md](background/world_geography_and_hub.md) | **`[LOCKED]`** | Three strata, Sluice Undercroft (`Sabina`, `Berek`, `Talia`, `Ivo`, `Widow Varka`), hazards (`CL-05`) |
| [sunk_ward_campaign_map.md](background/sunk_ward_campaign_map.md) | **`[LOCKED]`** | Act 1 Sunk Ward graph, Alert, `M1-M5` & `Interlude` |
| [upper_bastion_campaign_map.md](background/upper_bastion_campaign_map.md) | **`[LOCKED]` spine** | Act 2 High Bastion graph, curfew bells, `2.1-2.6` & 6 dual-rituals |
| [mantle_fissure_campaign_map.md](background/mantle_fissure_campaign_map.md) | **`[LOCKED]` spine** | Act 3 Mantle Fissure graph, hard clock (`min 12d`), Mantle Resonance, `3.1-3.6` |
| [npcs_and_questlines.md](background/npcs_and_questlines.md) | **`[LOCKED]` (`2026-11-10`)** | Hub mentors (`Sabina`, `Berek`, `Talia`/`Ivo`/`Widow Varka`), regime bosses & street neutrals |

### 3.3 Narrative (`narrative/`)

| Doc | Status | What it holds |
| --- | --- | --- |
| [campaign_narrative_structure.md](narrative/campaign_narrative_structure.md) | **`[LOCKED]`** | Three-act structure: *Young Vanguard*, *First Rupture*, *Awakening Abyss* |
| [new_game_onboarding_and_intro.md](narrative/new_game_onboarding_and_intro.md) | **`[LOCKED]` spine** | Four-slide vignette, rain cold open, prologue `0a-0c` |
| [prologue_and_starter_squad_lore.md](narrative/prologue_and_starter_squad_lore.md) | **`[LOCKED]` (`2026-11-10`)** | Clean Day 1 human starter 4 dossiers (`Kaelen`, `Vesper`, `Maren`, `Hollis`) & Hearth scene |
| [recruits_and_prologue_architecture.md](narrative/recruits_and_prologue_architecture.md) | **`[LOCKED]` (`2026-11-10`)** | Prologue lessons, 6 visored/cowled `18-24` Act 2 cohorts, `25+` Basin guests (`Ferrand`, `Clemence`), Living Martyrs |
| [six_founders_and_phase_progression.md](narrative/six_founders_and_phase_progression.md) | **`[LOCKED]` (`2026-11-10`)** | All 6 Founders, 3-tier taint visuals, Doctrines, and `CL-05` stratum hazards |
| [narrative_choices_and_dilemmas.md](narrative/narrative_choices_and_dilemmas.md) | **`[LOCKED]` (`2026-11-10`)** | All 6 `M5` winch beats & keepsakes, 6 dual-rituals, Martyrs, `Ruthven` sieges, Endings `A/B/C` |
| [narrative_agency_and_fail_states.md](narrative/narrative_agency_and_fail_states.md) | **`[LOCKED]`** | Ensemble rule, wounded/captured/lost lifecycle, `Extinguished` screen |

### 3.4 Systems (`systems/`)

The class trees and bestiary are short indexes linking their parts (`class_*.md`, `bestiary_*.md`); link the index, not a part.

| Doc | Status | What it holds |
| --- | --- | --- |
| [combat_and_timeline.md](systems/combat_and_timeline.md) | **`[LOCKED]`** | `timeline` ruleset: 6 blocks, windups, 5-pip Poise & breaks, `Active Guard`, Intents |
| [magic_and_corruption.md](systems/magic_and_corruption.md) | **`[LOCKED]`** | Cold Steel vs. Inoculated (`Channel`), Strain `0..100`, `1d3` Rupture, Calcification `I..V` |
| [classes_and_skill_trees.md](systems/classes_and_skill_trees.md) | **`[LOCKED]` roles / `[PROVISIONAL]` trees** | Index of the 6 classes (`class_vanguard.md` .. `class_ichor_alchemist.md`; `CANON.md` Sec. 3.1 governs founder bios) |
| [skill_acquisition_and_loadout_systems.md](systems/skill_acquisition_and_loadout_systems.md) | canon | Hearth & Slab skill acquisition and deck slots |
| [rules_engine_architecture.md](systems/rules_engine_architecture.md) | canon | Armour curve, Strain gain, Poise, interrupts |
| [risk_reward_and_corruption_balance.md](systems/risk_reward_and_corruption_balance.md) | canon | AP density, Strain surfing, roster lifespan |
| [roster_expansion_and_lifecycle.md](systems/roster_expansion_and_lifecycle.md) | canon | Bunks `6 -> 8`, cohorts, downed/wounded/captured/lost |
| [gameplay_loop_and_ui_prototype.md](systems/gameplay_loop_and_ui_prototype.md) | canon | Expedition loop & combat loop |
| [screen_architecture_and_separation_plan.md](systems/screen_architecture_and_separation_plan.md) | canon | Screen ownership across engine, kit, and game shell |
| [enemy_bestiary_and_ai.md](systems/enemy_bestiary_and_ai.md) | **`[LOCKED]` roster** | Index of the 9 regular mobs, unit-bosses & 11 named bosses (`CAST1_BESTIARY.md`, `CAST1_BOSSES.md`) |
| [safehouse_economy_and_triage.md](systems/safehouse_economy_and_triage.md) | **`[LOCKED]`** | Sluice Undercroft rooms (`Sabina`, `Berek`, `Talia`, `Widow Varka`), Scrap, Warded Tallow, Rock Salt |
| [items_gear_and_progression.md](systems/items_gear_and_progression.md) | **`[LOCKED]` (`2026-11-10`)** | `3/2/2` families, Tactile Consumables, Ossuary Growths (`Slab Retorts`) vs. Hearth Salvage, Relics & Keepsakes |

## 4. Quick reference (matches CANON.md sections 1-6)

| Parameter | Locked Canon Value (`2026-11-10`) | Layer |
| --- | --- | --- |
| Title & aesthetic | **Emberwake: A Scarlet Lullaby** (`emberwake`); **Soot-Gothic + Early-Industrial Ossuary** (no gunpowder firearms; unbroken Shroud) | - |
| Acts | Act 1 *The Young Vanguard* (`Sunk Ward`), Act 2 *The First Rupture* (`High Bastion`), Act 3 *The Awakening Abyss* (`Mantle Fissure`) | `[data]` |
| Ages & cohorts | Operatives `18-24` static (`CL-07`); `6` barefaced Day 1 human Founders + `6` visored/cowled Act 2 cohorts (`1` sheltered by `Talia` at Act 2 start); `25+` guests (`Ferrand` `40`, `Herbalist Clemence` `36`) stay bunk-free; `under 18` = non-combat stakes only | `[data]` |
| Hub cast | **Sluice Undercroft**: `Chirurgeon Sabina` (`31`, Slab), `Berek "Cinder-Lung" Dorn` (`33`, Hearth), `Talia Rusk` (`29`) + `Ivo` (`12`) + `Widow Varka` (`42`, Basin `Trade` shop) | `[data]` |
| Squad & bunks | `PARTY_MAX = 4`; `6` Dry Hollows bunks in Act 1 (`8` via `Berek` in Act 2) + `2` Isolation Crypt stasis slots for Living Martyrs | `[kit hook]` |
| Ruleset & AP | `timeline` (`sunk_ward`), `classic` (`ashlands`); `6` blocks, `12` max AP (`10` start, `+4`/round; `Rest` `+0.5` AP/block cap `+2`, `+5` Poise/block); `5` Intents + Act 2 `0 AP` `Channel` | `[kit hook]` + `[engine]` |
| Poise & Guard | `CONS * 2 + gear + passives` (`5` pips; break cancels windup & queues `stagger` on blocks `1 & 4`); `Active Guard` `35%` DR (`Brace` `20%`), `0.90` cap; armour `rating / (rating + 50)` | `[kit hook]` |
| Corruption & 3-tier taint | Tier 1 clean Day 1 human -> Tier 2 `portrait_fx` + Founder Inoculated taints (`Strain 0..100`, `1d3` Rupture, `Calcification I..V`, `Slab Retorts` `4` bone-pin slots) -> Tier 3 `3.4` `2x2` Saint-Weapon | `[engine]` + `[kit hook]` + `[data]` |
| Resources & items | `Scrap`, `Warded Tallow` (`1/d`), `Rock Salt`, `Quenched Cinder Salt`, **Ossuary Growths** (`Dreamer's Marrow Geode`, `Black-Sinew Braid`, `Ossified Bellows`, `Vitriol Bezoar`, `Amber Tapetum`), **Hearth Salvage** (`Brass Marrow Valve`, `Calcified Bone Shard`, `Render-Beast Tendon`) | `[kit hook]` + `[data]` |
| Meters & clocks | Alert (`0-100`, Act 2+ floor `30`, Breach = Act 1 fail / Act 2 `Ruthven` siege), Doom Clock (`42d`; soft `0d` lesser tithing `14d` `+10` Alert claims `Ivo`/`Ida`; `2.4` `max(days, 18)`; Act 3 hard `min 12d`), Doctrine (`-100..+100`) | `[kit hook]` |
| Endings & fail | **A `Black-Iron Severance`** (`Berek`), **B `The Living Conduit`** (`Sabina`), **C `Stone Martyrdom`** (always open); `Extinguished` fail screen | `[kit hook]` + `[data]` |
| Board & art | `640x360` canvas, `STAGE1` `32x24` cells (`12x9`), `40 px` in `48x48` (`96x96` on `2x2`), `120px` portraits (`AI-04..06`) | `[engine]` + `[data]` |
