# RPG kit and system architecture: what carries the lore

Status: canon (2026-10-25). The reuse assessment of the lore against the code as it is on `develop` (checked against
[CLAUDE.md](../../../../../CLAUDE.md), [docs/STRUCTURE.md](../../../../../docs/STRUCTURE.md),
[engine/README.md](../../../../../engine/README.md), [GAME_BRIEF.md](../../GAME_BRIEF.md) and the crates). Numbers:
[CANON.md](../CANON.md). It replaces the 2026-10-24 version, which named modules that do not exist (section 6).

## 1. The layer rule (user 2026-10-25)

| Tag | Meaning | Test | Where it lives today |
| --- | --- | --- | --- |
| `[engine]` | Technology any game needs: rendering, screens, saves, assets, audio, the timeline widget | Would a puzzle game or a racing game need it? | `engine/crates/aegis_*` (never names a game) |
| `[kit hook]` | A shape any story in the genre could fill: meters with thresholds and triggers, the unit lifecycle, hub facilities with levels, a resource purse, exchange recipes, ruleset options, event triggers, a mission board, a vignette player | Could another tactical RPG use it unchanged with its own data? | Story-free, data-driven code in the kit crates `kits/rpg/crates/rpgkit_*` (rules, battle, world, content, data, host, sim) with the field tables in `kits/rpg/schema.json`; the `emberwake_*` crates are façades over them plus game code and screens (KIT1, 2026-10-25) |
| `[data]` | Named by the story, filling a kit hook | Is it a name, a number, a line of text or a map? | `games/emberwake/data/` through `aegis content` (world `sunk_ward`) |
| `[game code]` | A story mechanic nothing else could use | Can no kit hook express it? | The game crates, kept small and named as game code |

Two consequences: (1) a kit hook is written with genre names (`meter`, `stage track`, `facility`), never story names
(`strain`, `calcification`, `slab`); the story names arrive as data. (2) The Ashlands keep running on the same hooks
with the "classic" options, so the parity suites stay the guard rail for every hook added.

Reuse rules (user 2026-10-25, Q-52, Q-55, Q-56): the engine is reused with the Ashlands as the baseline; a rule
incompatible with the classic rules is built as an option, a compatible one as an extension that can be disabled; bug
fixes go to the core and may need an Ashlands or kit revisit. Every engine and kit system the vertical slice needs is
built before it (M0-M1), so the slice is only narrative choices and catalog data; anything existing systems do not
cover becomes a reusable engine or kit system, and only game-specific parts stay in the game.

## 2. What exists today

| System | Real carrier | Tag |
| --- | --- | --- |
| Window, 640x360 logical canvas, screen stack, settings, agent link | `aegis_ui::shell`, `aegis_ui::screen::Screen`, `aegis_ui::bridge` | `[engine]` |
| Timeline widget (ruler of blocks, playhead, tracks, segments with hover tips) | `aegis_ui::timeline::Timeline` (TIMELINE1) | `[engine]` |
| Feet-sorted drawing of props and units | `aegis_ui::depth::DepthList` (PROPS1-E); the battle stage B is approved (STAGE1-E engine, STAGE1-L game) | `[engine]`, `[game code]` |
| Saves | `aegis_save` (slots, envelope, migrations) with the game's format in `emberwake_world::save` (version 12) | `[engine]` |
| Content store, world lineage, upsert | `aegis_content`; the game's rules in `emberwake_content` (`source`, `store`, `check/`) | `[engine]`, `[kit hook]` |
| Asset layers (game, kit, engine) | `aegis_core::assets`, `aegis_ui::art`, `kits/rpg/assets/`; lookup chains in `emberwake_content::art` | `[engine]`, kit assets |
| Audio and music | `aegis_ui::audio`; track choice `emberwake_gui` `music.rs` (MUSIC1) | `[engine]`, `[kit hook]` |
| Stats, classes, proficiency, passives | `emberwake_rules` `stats.rs` (`StatRules`: STR DEX INT CONS PERC KNOW, budget 62), `unit.rs`; rows in `emberwake_data` (`ClassDef`, `UnitDef`, `SkillDef`, `ItemDef`, `PassiveDef`) | `[kit hook]` |
| Gear and pouch | `emberwake_rules::gear`: six slots MAIN, OFF, HEAD, BODY, ACC1, ACC2; pouch 3; two-handed rule | `[kit hook]` |
| Damage | `emberwake_rules::damage` (`DamageCalculator`): armour reduction = rating / (rating + 50), guard capped at `GUARD_CAP` 0.9 | `[kit hook]` |
| Statuses | `emberwake_rules::status`: fifteen statuses incl. stagger, stun, bleed, slow, haste, expose, fortify | `[kit hook]` |
| Battle round | `emberwake_battle::resolver`: 6 blocks per round, all sides resolved simultaneously; an action pays AP and enters its stance when it starts and lands at the end of its last block, highest initiative first; a scheduled action can be cancelled (`ScheduledAction.cancelled`: today when its unit drops); guard breaks end running guards | `[kit hook]` |
| AP, skill length, loadout | 10 AP, +4 per round, skill AP = skill level; skill field `blocks` (1-6, "the effect lands at the end of the last block"); loadout 5 class families + 3 extra (`StatRules` `MAX_CLASS_SLOTS`, `MAX_EXTRA_SLOTS`); basics (strike, guard, move, rest, switch_target, offhand_jab) always owned | `[kit hook]` |
| Stances | Offensive > Tactical > Defensive, 1 AP switch (the "classic" ruleset) | `[kit hook]` |
| Planning, AI, tells | `plan.rs`, `ai.rs` (`TacticalAI`), `intent.rs` (enemy tells while planning, `Targets you in B<n>`) | `[kit hook]` |
| Encounter rules | `emberwake_battle::encounter`: objectives (rout, hold, escape, slay, protect, guests), deployment, boss phases, the arrival wave (reinforcements), morale; `session.rs` (`CombatSession`, guests, the round archive) | `[kit hook]` |
| Hazards and cover | skill fields `hazard_*`, `command.rs` (hazard, cover), terrain catalog | `[kit hook]` |
| Sandbox | the Gear Dojo (`emberwake_battle::dojo`, `emberwake_gui` `dojo.rs`) | `[kit hook]` |
| Balance sim | `emberwake_sim` (`tools/fc.sh sim`) | `[kit hook]` |
| Game state | `emberwake_world::state`: `GameState` (flags, quests, `time`, position, rolls, shops, purses), `PartyState` (gold, supplies, stash, roster, active ids), `UnitState` (XP, spend, skills, loadout, pouch, `hp_lost`) | `[kit hook]` |
| Travel and the world map | `emberwake_world::travel` (node graph, routes with `time`, supplies spent per time unit, encounter chance x1.5 when out of supplies), `emberwake_gui` `worldmap.rs` | `[kit hook]` |
| Region map | `emberwake_world::explore` (markers, roamers, chests, exits, defeat and wake), `emberwake_gui` `region.rs` (square top-down, 16 px tiles) | `[kit hook]` |
| Quests, dialogue, reputation | `quest.rs` (templates, flags, loot pools), `dialogue.rs` (conditions flag with reputation keys, quest, items, gold, stat, no `status` condition; effects set_flag, add_flag, give, take, recruit, open_shop, fight, start_quest, turn_in), faction standing in flags `rep.<world>.<faction>` (RepRules, bands) | `[kit hook]` |
| Shops, purse, rewards | `emberwake_world::shop` and `emberwake_gui` `shop.rs` `Trade`; `purse.rs` (PURSE1); kill XP, purse shares and loot tables (REWARD1, `rolls.rs`) | `[kit hook]` |
| Recruits and healing | RECRUIT1 (dialogue `recruit`, `party_max`); HEAL1 (stored HP, rest places, camp for 3 supplies, a win rallies +25%, defeat wakes at the last rest) | `[kit hook]` |
| Menus and party | `emberwake_gui` `hub.rs` `Hub` (the in-game menu: party, journal, save, load, settings, quit), `party.rs` `Character`, `menus.rs` | `[kit hook]` |
| Dev tools | `forge.rs` (the dev-only Content and World Forge), `runner.rs` (Scene Runner), `gallery.rs` (UI Gallery) | dev |

## 3. Lore feature by carrier

"Exists" = works today with data; "extend" = an existing system gains an option; "new" = a hook to add. Acts 2-3
features are `proposal`.

### 3.1 Combat (the "timeline" ruleset)

| Lore feature | Carried by | Work | Tag |
| --- | --- | --- | --- |
| Ruleset switch "timeline" / "classic" | session start reads an option; classic = today's behaviour | new: a world field (the Ashlands "classic", `sunk_ward` "timeline") with a Dojo toggle (Q-52) | `[kit hook]` |
| 6 blocks, simultaneous, AP 10 +4 | resolver | exists | `[kit hook]` |
| Timeline strip at the top of the battle screen | `aegis_ui::timeline` in the COMBAT-UI3 layout A | exists | `[engine]` |
| Speed tiers Snap 1, Standard 2, Heavy 3-4 | skill `blocks` | exists: set `blocks` per skill | `[data]` |
| Recovery blocks after the impact | resolver schedules start..end | new: blocks after the effect that keep the unit busy; 0 by default, set per skill | `[kit hook]` |
| Chained commands (Q-54) | resolver schedules each action on its own | new: a follow-up skill can start in the blocks a previous action leaves, otherwise the unit spends them in recovery | `[kit hook]` |
| `ap_cost` decoupled from level | skill AP = level | extend: field `ap_cost`, default = level | `[kit hook]`, `[data]` |
| Same-block order, no mutual trades | highest initiative first; a downed unit's actions are cancelled | exists | `[kit hook]` |
| Poise meter and break | `ScheduledAction.cancelled`, the stagger status, guard breaks | new: a per-unit poise meter (formula, refill each round), a break cancels the windup and the remaining blocks that round, doubled cost for a repeat break | `[kit hook]`, numbers `[data]` |
| Stagger multipliers (windup x2, Active Guard x0.5) | - | new, with the poise meter | `[kit hook]` |
| Stagger intent (Q-50) | seeded `rolls` | new: rescaled stagger values, more poise or an unbreakable flag on heavy enemy windups, an optional seeded break chance and a guaranteed break at a cost (AP, a skill or Strain); design-lane spec, then a sim | `[kit hook]`, numbers `[data]` |
| Composure, the corruption resist (Q-47a) | stats are the six base stats only | new: data-defined derived stats computed by a formula evaluator from base stats plus gear, status and Calcification modifiers; shown on the unit panel (`docs/specs/DERIVE1.md`, being written) | `[engine]` evaluator, `[kit hook]` derived stats, `[data]` the Composure row |
| Active Guard | DEFEND skills, the guard ratio, `GUARD_CAP` | extend: halves incoming stagger; venting is a meter trigger | `[kit hook]`, `[data]` |
| Stances off | stance rules | extend: an option of the ruleset | `[kit hook]` |
| Armour rescale | `DamageCalculator` curve | exists: lore armour points x 2 = rating | `[data]` |
| Inoculated attacks ignore armour | round-end damage already ignores armour; the damage type `pierce` is a type, not a bypass | new: a skill flag that bypasses armour | `[kit hook]`, `[data]` |
| Deck 8 slots (Move, Active Guard, 3 actives, 2 passives, 1 keystone) | loadout 5 class + 3 extra; basics always owned; passives come from sources, not slots | extend: slot kinds base, active, passive, keystone, working like today's skill slots (Q-48) | `[kit hook]` |
| Advance and Disengage | `dash_tiles` (gap-close first in the sequence), DISENGAGE skills (`jump_tiles`) | exists for one move per round | `[kit hook]`, `[data]` |
| Impact badges on the strip | strip segments and intent tells | extend: a badge on the block where an impact lands | `[engine]` |
| Hit-stop, swing trails, impact sparks | FX1 (approved) | new under FX1 | `[engine]`, `[game code]` |
| Telegraphed ground zones and hazards | hazards, cover, terrain, intent tells | exists; new zone kinds as data | `[data]` |
| Named bosses, waves, morale | encounter phases, the arrival wave, morale | exists | `[data]` |
| A phase where an ally boss turns on everyone (node 20: the corrupted founder breaks, kills Nemeth and his guard, then fights the cell; CANON 3.6) | `hp_below` with `or_round`, line, status, ai and enrage effects; a unit swap for the larger body | new (small): a phase effect that kills named units, and a "hostile to all sides" AI or faction switch | `[kit hook]` |
| Maneuver hooks the slice needs (forced movement, ground hazards a skill leaves, scheduled arena hazards; effects that leaned on cut rolls) | statuses, hazards, phases | new: reusable kit or engine systems before the slice, starting with forced movement and ground hazards (Q-55, Q-56); cover and line of sight after testing (Q-54) | `[kit hook]` or `[engine]` |
| Cut (DEC-LORE-37) | evasion and accuracy rolls, facing cones, backstab, stealth cones, mutual trades, the multi-track ghost scrubber | none | - |

### 3.2 Corruption (Act 2 on)

| Lore feature | Carried by | Work | Tag |
| --- | --- | --- | --- |
| Strain 0-100 with brackets | - | new: a per-unit battle meter with thresholds; gain on action start (skill field), decay per round with conditions, floor, reset after battle | `[kit hook]`, names and effects `[data]` |
| Bracket effects (crit, move, damage, armour, recoil) | statuses and passives | extend: a threshold grants a status or passive | `[data]` |
| Crisis 1d3 at round end | seeded `rolls` | new, unless the meter's triggers can roll a table | `[game code]` or `[kit hook]` |
| Calcification stages I-V | - | new: a per-unit stage track on `UnitState`, advanced by triggers, stage effects as passives | `[kit hook]`, `[data]` |
| Grafts (4 slots), apex organs | gear slots | new (proposal): a second slot set or gear rows on the stage track | `[kit hook]` |
| Slab services: flush, bone-scraping, surgery | shops (`Trade`) | new: exchange recipes at a facility (materials in, effect out) | `[kit hook]`, `[data]` |

### 3.3 Campaign, hub and roster

| Lore feature | Carried by | Work | Tag |
| --- | --- | --- | --- |
| World `sunk_ward` (district graph, locations, factions) | world folders, `travel`, `worldmap.rs` | exists | `[data]` |
| Stealth and speed routes | routes with `time`, events, fights | exists: two routes per leg | `[data]` |
| Time costs (route `time` 1 = 1 day; a rest 1 day) | `GameState.time`, route `time` | extend: a rest advances time | `[kit hook]` |
| Scrap as the currency | `PartyState.gold`, shops, purse, rewards | extend: the currency's display name per world | `[kit hook]`, `[data]` |
| Warded Tallow upkeep | supplies (per time unit, camp cost, `start_supplies`) | extend: the name per world; the 0-stock effect becomes an option (today +50% encounter chance; Emberwake -2 starting AP) | `[kit hook]`, `[data]` |
| Rock Salt, Aberrant Organs | stash items | exists | `[data]` |
| Inquisition Alert, Doom Clock, doctrine | numeric flags and reputation bands | new: a generic meter system with thresholds, triggers, inputs (mission tags, bribes, story amounts), a warning at each threshold and effects such as Lockdown (visible, soft or hard; Q-15) | `[engine]` if game-agnostic, else `[kit hook]`; `[data]` |
| Tess's dead-drop board | quests and dialogue `start_quest` | new: a mission board listing a giver's offers | `[kit hook]`, `[data]` |
| The Undercroft hub and facilities with levels | a world location on the region map, NPC markers, shops | new: facilities with levels and projects (Bram's 8-bunk frames) | `[kit hook]`, `[data]` |
| Bunks 6 to 8 | roster unbounded, `party_max` 4 active | new: a roster cap raised by a facility level (Act 1: six founders only; the 8-bunk level from Act 2, Q-38) | `[kit hook]` |
| Downed, wounded, captured, dead, retired | downed in battle; HEAL1 rally and wake | new: the unit lifecycle after battle (states, effects, recovery) | `[kit hook]` |
| Founders join | RECRUIT1 `recruit` effect, world characters | exists | `[data]` |
| Class names per world (Vanguard over `warrior`) | one shared class set | extend: base classes live in the kit with a display name per world; the base set may grow so other games' classes port and are re-themed (Q-53) | `[kit hook]`, `[data]` |
| Prologue missions 0a-0c, guests | encounters with `guests`, `deployment`, objectives | exists | `[data]` |
| Intro vignette, hearth scene, title cards | dialogue | new: a vignette player (images and lines, skippable) | `[kit hook]`, `[data]` |
| Scripted beats (hearth after 0c, the interlude) | flags, quests, dialogue | new: event triggers (on flag or arrival, play a scene or fight) | `[kit hook]` |
| The Crucible sacrifice | deployment of four, `escape` / `protect` objectives | new: pick a deployed unit after the fight, apply "captured" | `[kit hook]`, `[data]` |
| Extinguished failure screen | defeat wakes at the last rest | new: campaign fail conditions (the Act 1 Breach first, Q-13) | `[kit hook]` |
| Endings A-C | flags, dialogue | vignette player | `[data]` |
| Sieges, procedural recruits, Living Martyrs, bells, blackouts (Acts 2-3, proposal) | encounters at the hub, RECRUIT1, the lifecycle, meter triggers | new hooks named in [full_game_systems_roadmap.md](full_game_systems_roadmap.md) | `[kit hook]`, `[data]` |

## 4. Kit hooks to add, in build order

Every hook the slice needs lands in M0-M1, before the slice (Q-55, Q-56); M2 adds only data.

1. M0: the ruleset as a world field with a Dojo toggle; base classes with per-world display names; data-defined
   derived stats on a formula evaluator (Composure, DERIVE1); deck slot kinds; the vignette player, event triggers,
   mission board, the generic meter system (Alert inputs, warnings and Lockdown; the Doom Clock), the unit lifecycle
   (downed, wounded, captured), the roster cap, the currency and upkeep names and the 0-stock option, rest time, the
   campaign fail conditions (the Act 1 Breach).
2. M1, in the Dojo: the "timeline" rules with poise and break, stagger intent (rescale, break chance, guaranteed break
   at a cost), recovery blocks and chained follow-ups, `ap_cost`, the armour-bypass flag, stances off; the maneuver
   hooks the slice's encounters use (forced movement and ground hazards first); impact badges on the strip `[engine]`;
   the sim's ruleset option.
3. M3: per-unit meter, per-unit stage track, exchange recipes, more facility levels (the 8-bunk project), the
   no-operative fail condition.
4. Act 3 data: for the node 20 battle, a phase effect that kills named units and a "hostile to all sides" AI or
   faction switch (both small; may ride with earlier kit work).

Milestones: [full_game_systems_roadmap.md](full_game_systems_roadmap.md).

## 5. Game code (kept small)

| Candidate | Why it might be game code | Default |
| --- | --- | --- |
| The crisis table (Unchecked Spasm hits the nearest unit, Collapse sets AP 0 and armour 0, Breach adds a stage) | Spasm is a forced action, which no status expresses today | Kit hook if the meter's triggers can fire a forced strike; else game code |
| Strain decay rules (no Inoculated action, Active Guard vent) | conditions on what a unit did this round | Kit hook: meter decay with conditions |

Everything else in the lore is data on kit hooks.

## 6. Wrong names in the 2026-10-24 docs

| Written | Real |
| --- | --- |
| `engine/combat`, `Resolver::step_tick`, sequential initiative | `emberwake_battle::resolver` (game crate), 6 simultaneous blocks per round |
| `ClassCatalog`, `FactionCatalog`, `emberwake_data::models::*` | `emberwake_data::{ClassDef, UnitDef, SkillDef, ItemDef, PassiveDef}` and `Catalog`; factions are a world kind |
| `catalogs/enemies.json` | `games/emberwake/data/catalog/units.json` and `encounters.json`; world `encounters` |
| `ResourceState` | HP and AP on the battle unit; `PartyState` gold and supplies |
| `LocationKind::Hub`, `RegionKind` | world `locations` and `regions` rows; `hub.rs` `Hub` is the in-game menu |
| `DistrictMap` | `worldmap.rs` + `emberwake_world::travel` (node graph); `region.rs` (tile walk) |
| `emberwake_world::{reputation, crisis, campaign, recruitment, safehouse}`, `emberwake_rules::grafts` | none exist; reputation is RepRules over flags; the rest are new hooks (section 4) |
| `BattleScreen` | `emberwake_gui` `battle.rs` |
| The Content Forge as a game screen | `forge.rs` is dev-only |

## 7. Gaps

1. Where kit code lives. Answered (user 2026-10-25): kit extraction happens in M0; the earlier recommendation to defer
   it is withdrawn (the planner's record holds the scope; [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md) section 3). Done in KIT1
   (2026-10-25): the kit crates `kits/rpg/crates/rpgkit_*`.
2. Where the ruleset switch is set. Options: (a) the world manifest (Ashlands classic, `sunk_ward` timeline); (b) per
   encounter. Recommendation: (a), with a Dojo toggle. Decided (2026-10-25, user, Q-52): (a), a world field with a Dojo
   toggle, plus the reuse rules of section 1.
3. Deck slots: the lore's 8 include 2 passives and a keystone, the engine's 8 are skill families, and passives come from
   sources. Options: (a) passives and keystone become loadout slot kinds; (b) keep the engine's 5 + 3 skills and grant
   passives from sources. Decided (2026-10-25, user, Q-48): (a), working like today's skill slots, bucketed by kind.
4. Class names per world (Vanguard for warrior). Options: (a) new class rows for the new worlds; (b) a world-level
   display name over the shared classes. Decided (2026-10-25, user, Q-53): (b); the base classes live in the kit (not
   the engine) and may grow so other games' classes port directly and are re-themed.
5. Composure (named by Calcification I and the recruit quirks) is not one of the six stats. Decided (2026-10-25, user,
   Q-47 and Q-47a): not a seventh base stat; Composure is a derived stat, the corruption resist, computed by its own
   formula from base stats plus gear, status and Calcification modifiers (CANON 2.1); the engine gains a formula
   evaluator and the kit data-defined derived stats (`docs/specs/DERIVE1.md`).
6. The balance sim cannot run the timeline ruleset yet. Recommendation: the sim takes the ruleset option in M1 so the
   balance lane can measure it (listed with Q-52's options; in the M1 list).
