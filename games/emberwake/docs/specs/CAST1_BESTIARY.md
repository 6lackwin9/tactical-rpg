# CAST1 bestiary: the Act 1 regular enemies

Status: proposed (2026-10-25; rewritten to the HOOKS1 hooks by CAST1-R the same day); part of [CAST1.md](CAST1.md).
Lore: `systems/bestiary_castellan_watch.md`, `bestiary_iron_inquisition.md`, `bestiary_tallow_guild.md`,
`bestiary_aberrations.md` (names canon, numbers proposal). Bosses: [CAST1_BOSSES.md](CAST1_BOSSES.md). Hook fields as
`docs/specs/HOOKS1_TASKS.md` 1; notation as CAST1_FOUNDERS 1. All numbers FP.

## 1. Rules for enemy rows

- Enemies have no AP and book `recovery_blocks` after each impact (RULESET1 T1-T4, T11, rule `enemy_moves`). Their
  skills are rows of their own (a shared row would give founders the recovery number they never use, harmless but
  confusing). `level` on enemy skills is 1 unless noted, so every listed unit owns them.
- They pick intent and target by the RULESET1 AI table (intent 9): role, `target_pref`, `reckless`, `flee_ratio`,
  `defend_below` steer it, with HOOKS1 11's `tactics` and the prefs `SOFTEST` and `TOUGHEST`; their `deck.order`
  fills the round. Active Guard and Flank Cut are basics they own too.
- Each unit row: side enemy, `stat_mode` derived, `spend` {}, `skill_cap` 0, `heal_below` 0.7, `pouch` ["none"],
  `extra_skills` = its deck actives, `art` and `palette` per CAST1_ASSETS. Each has a world character (role enemy) of
  the same id with its trait and drill passives (CAST1.md 3.2). Every unit built in `sunk_ward` has an empty class
  path (HQ5 a), so it owns only its `extra_skills` and the basics.
- `armor`: 25, 35 or 50 for armoured units (Q-49 test scale), else -1 (gear mitigation + STR / 4). `poise`: -1
  (derived) unless a number is given.
- AI (G29 through HOOKS1 11): bracing after an ally is hit and holding a chokepoint are the Footman's tactics; rushing
  the back line is the Flagellant's SOFTEST; flushing cover is the Purifier's. Not built: seeking raised tiles (G8),
  pack hunting (a later tactic `pack`, HOOKS1 AT4), targeting a unit in windup (HQ2 a: not adopted; the Crossbowman
  keeps WEAKEST).

## 2. Units (`units.json`, CAST1-A4) FP

| Id | Name | Class | Tier | Lvl | Role, targets | `tactics` | Reckless, flee, defend | Gear (main, off, body) | `armor` | `poise` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `castellan_footman` | Castellan Footman | warrior | standard | 2 | AGGRESSOR, NEAREST | [`brace_after_hit`, `hold_line`] | no, 0.25, 0.4 | `watch_hookbill`, `watch_heater`, `watch_cuirass` | 25 | 24 |
| `watch_crossbowman` | Watch Crossbowman | ranger | standard | 2 | SNIPER, WEAKEST | [] | no, 0.25, 0.4 | `stirrup_arbalest`, none, `watch_gambeson` | -1 | -1 |
| `bailiff_enforcer` | Bailiff Enforcer | warrior | elite | 3 | AGGRESSOR, TOUGHEST | [] | no, 0.0, 0.3 | `bailiff_maul`, none, `watch_cuirass` | 35 | -1 |
| `mortuary_flagellant` | Mortuary Flagellant | rogue | minion | 1 | SKIRMISHER, SOFTEST | [] | yes, 0.0, 0.0 | `penitent_scourge`, none, none | -1 | -1 |
| `inquisitorial_purifier` | Purifier | mage | elite | 3 | SNIPER, NEAREST | [`flush_cover`] | no, 0.2, 0.4 | `pyre_nozzle`, none, `warded_apron` | 25 | -1 |
| `tallow_dredger` | Tallow Dredger | rogue | standard | 2 | SKIRMISHER, WEAKEST | [] | no, 0.3, 0.4 | `boning_knife`, none, `skinners_apron` | -1 | -1 |
| `render_beast` | Render-Beast | rogue | minion | 2 | SKIRMISHER, WEAKEST | [] | yes, 0.0, 0.0 | none, none, none | -1 | -1 |
| `ash_crawler` | Ash-Crawler | rogue | minion | 2 | SKIRMISHER, NEAREST | [] | yes, 0.0, 0.0 | none, none, none | -1 | -1 |
| `calcified_husk` | Calcified Husk | warrior | standard | 3 | AGGRESSOR, NEAREST | [] | yes, 0.0, 0.0 | `bone_club`, none, none | 35 | -1 |

- `name` "Purifier" is the short form of Inquisitorial Purifier (the description holds the full name).
- The Footman's fixed `poise` 24 is the tutorial number: Pommel Strike's stagger 12 doubled in a windup is 24, so one
  timed Snap breaks a Footman's Heavy thrust (0a's lesson, CAST1 T10). Derived it would be about 50 (CONS 19 at level
  2 x 2 + cuirass 8 + shield 5).
- The Footman carries the one-handed `watch_hookbill` (axe family, reach 1) and a heater shield (CQ13 a): 0a stays a
  reach-1 duel against Kaelen's reach 2.
- `none` gear values are the unit gear token `none` (no item).

## 3. Skills (`skills.json`, CAST1-A3) FP

All `learned_only` true, one rank, `family` = the id, `round_intent` as listed; "Rec." is `recovery_blocks`. Forced
moves on these rows set no `collide_*` key (the schema defaults) unless listed.

| Id | Name | Kind | Intent | Blocks | Rec. | Power | Stagger | Reach | Status, flags and hooks | Unit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `heavy_bill_thrust` | Heavy Thrust | ATTACK | assault | 3 | 1 | 1.5 | 10 | weapon | - | Footman, Vane |
| `billhook_hook` | Billhook Hook | ATTACK | assault | 2 | 1 | 1.0 | 8 | weapon | `forced_move` pull 1, `when` moved, `stop_moves` | Footman |
| `aimed_bolt` | Aimed Bolt | ATTACK | assault | 2 | 2 | 1.0 | 5 | weapon | `weapon_types` [crossbow]; expose potency 80, magnitude 2, 1 round | Crossbowman |
| `buckler_jab` | Buckler Jab | ATTACK | assault | 1 | 1 | 0.4 | 4 | 1-1 | `forced_move` push 1 | Crossbowman |
| `crushing_judgment` | Judgment | ATTACK | assault | 4 | 1 | 2.2 | 20 | 1-1 | level 2; area SWEEP; `unbreakable`; stun potency 50, 1 round | Bailiff Enforcer |
| `iron_shackle_drag` | Shackle Drag | ATTACK | assault | 2 | 1 | 0.5 | 6 | 1-3 | root potency 80, 1 round; `forced_move` pull 2 | Bailiff Enforcer |
| `scourge_lash` | Scourge Lash | ATTACK | assault | 1 | 1 | 0.5 | 5 | weapon (1-2) | `weapon_types` [scourge]; bleed potency 70, 2 rounds | Flagellant |
| `martyrs_rush` | Martyr's Rush | ATTACK | maneuver | 1 | 1 | 0.9 | 6 | weapon | `dash_tiles` 3; `hp_cost` 10 | Flagellant |
| `pyre_nozzle_sweep` | Nozzle Sweep | ATTACK | assault | 2 | 2 | 0.7 | 4 | 2-3 | `weapon_types` [sprayer]; area BURST; burn potency 80, magnitude 5, 2 rounds; `hazard` {`phosphorus_fire`, SKILL, target, 2} | Purifier |
| `brand_of_the_heretic` | Heretic's Brand | ATTACK | assault | 1 | 0 | 0.5 | 4 | 1-1 | expose potency 80, magnitude 4, 2 rounds; `riders` [{}: `end_guard`] | Purifier |
| `scalding_grease_pot` | Grease Pot | ATTACK | assault | 2 | 1 | 0.4 | 0 | 2-3 | area PLUS; slow potency 80, 2 rounds; `hazard` {`grease`, SKILL, target, 2} | Dredger |
| `boning_knife_flurry` | Knife Flurry | ATTACK | maneuver | 1 | 0 | 0.55 | 4 | 1-1 | `dash_tiles` 2; bleed potency 60, 1 round; exploit `flanked` x1.3 | Dredger |
| `throat_clamp_pounce` | Throat-Clamp | ATTACK | maneuver | 1 | 1 | 0.8 | 10 | 1-1 | `dash_tiles` 2; root potency 80, 1 round | Render-Beast |
| `burrow_ambush` | Burrow Ambush | ATTACK | maneuver | 1 | 1 | 0.5 | 4 | 1-1 | `dash_tiles` 3; `stagger` (status) potency 60 | Ash-Crawler |
| `caustic_vomit` | Caustic Vomit | ATTACK | assault | 2 | 1 | 0.5 | 0 | 1-3 | poison potency 70, magnitude 4, 2 rounds | Ash-Crawler |
| `bone_club_slam` | Bone-Club Slam | ATTACK | assault | 2 | 1 | 1.1 | 12 | 1-1 | - | Calcified Husk |

"weapon" reach = the wielder's band (the scourge 1-2 by RULESET1 K1, the crossbow's ranged band). Short forms:
"Judgment" (Crushing Judgment), "Shackle Drag" (Iron Shackle Drag), "Nozzle Sweep" (Pyre-Nozzle Sweep), "Heretic's
Brand" (Brand of the Heretic), "Grease Pot" (Scalding Grease Pot), "Knife Flurry" (Boning Knife Flurry),
"Throat-Clamp" (Throat-Clamp Pounce), "Heavy Thrust" (Footman's Heavy Bill Thrust).

- Billhook Hook pulls only a target that moved this round and cancels its later walks (HOOKS1 FM1, FM7); Shackle
  Drag pulls 2 toward the Bailiff (a pull stops at distance 1, FM2).
- Buckler Jab (new, the Crossbowman's lore jab): a Snap push to win back range; the description names it.
- Martyr's Rush costs 10 HP at its start (HOOKS1 HC1); the AI refuses it at 10 HP or less.
- Heretic's Brand ends the target's Active Guard and held guard on landing (HOOKS1 RD1, `end_guard`).
- Nozzle Sweep leaves Phosphorus Fire and Grease Pot leaves Scalding Grease on their area for 2 rounds (HOOKS1_TASKS
  2.1).

Deferred lore effects: the Grease Pot's Flammable (G17); the Burrow Ambush's tunnelling through rubble and its pierce
(G8; pierce is Act 2 on enemies of the Low Sleep, Q-65); Caustic Vomit's second status expose 4 (G9); Crushing
Judgment's painted cone decal (RULESET1 badges show the impact).

## 4. Decks (`deck` on the unit rows, CAST1-A4)

| Unit | `active` | `passive`, `keystone` | `order` |
| --- | --- | --- | --- |
| `castellan_footman` | `heavy_bill_thrust`, `billhook_hook` | -, - | assault [`heavy_bill_thrust`, `billhook_hook`]; guard [`active_guard`] |
| `watch_crossbowman` | `aimed_bolt`, `buckler_jab` | -, - | assault [`aimed_bolt`, `buckler_jab`, `strike`]; guard [`active_guard`] |
| `bailiff_enforcer` | `crushing_judgment`, `iron_shackle_drag` | -, - | assault [`crushing_judgment`, `iron_shackle_drag`]; guard [`active_guard`] |
| `mortuary_flagellant` | `scourge_lash`, `martyrs_rush` | -, - | assault [`scourge_lash`]; maneuver [`martyrs_rush`, `flank_cut`] |
| `inquisitorial_purifier` | `pyre_nozzle_sweep`, `brand_of_the_heretic` | -, - | assault [`pyre_nozzle_sweep`, `brand_of_the_heretic`]; guard [`active_guard`] |
| `tallow_dredger` | `scalding_grease_pot`, `boning_knife_flurry` | -, - | assault [`scalding_grease_pot`]; maneuver [`boning_knife_flurry`, `flank_cut`]; guard [`active_guard`] |
| `render_beast` | `throat_clamp_pounce` | -, - | maneuver [`throat_clamp_pounce`]; assault [`strike`] |
| `ash_crawler` | `burrow_ambush`, `caustic_vomit` | -, - | maneuver [`burrow_ambush`]; assault [`caustic_vomit`, `strike`] |
| `calcified_husk` | `bone_club_slam` | -, - | assault [`bone_club_slam`, `strike`] |

## 5. Traits and drills (world characters, CAST1-A2 and A5)

| Character (= base unit) | Name | Passives (`slot` "") | Lore trait and what is deferred |
| --- | --- | --- | --- |
| `castellan_footman` | Castellan Footman | - | Shield Wall Bracing: the `brace_after_hit` tactic and Active Guard |
| `footman_patrol` | Castellan Footman | - | prologue variant: `tuning` {"hp_pct": -30, "power": -2} |
| `footman_shieldbearer` | Shield-Bearer | - | Vane's wall wave: `tuning` {"hp_pct": -20} |
| `watch_crossbowman` | Watch Crossbowman | - | Buckler Jab is its skill (3) |
| `bailiff_enforcer` | Bailiff Enforcer | `unyielding_authority`: `stagger_taken` mult 0.0, max_blocks 1; `forced_move` `immune` always | Unyielding Authority (none deferred) |
| `mortuary_flagellant` | Mortuary Flagellant | `scourge_drill`; `fanatical_agony`: trigger `hp_below` (`pct` 50), `limit` battle: grant `haste` rounds -1; `crit` 20 | Fanatical Agony (none deferred) |
| `inquisitorial_purifier` | Purifier | `sprayer_drill`, `heat_tempered_carapace`: resist burn 75, poison 50 | Heat-Tempered Carapace (none deferred) |
| `tallow_dredger` | Tallow Dredger | - | - |
| `render_beast` | Render-Beast | - | Tenacious Chase: following a target that moves in its windup (G5, no hook) |
| `ash_crawler` | Ash-Crawler | - | - |
| `calcified_husk` | Calcified Husk | `calcified_frame`: resist bleed 50, poison 50 | Osseous Shrapnel: a hit to every adjacent unit on its break (G5, no hook) |

`footman_patrol` and `footman_shieldbearer` share the Footman's base unit, so they carry its `tactics`.

Passive rows (`passives.json`): `unyielding_authority` "Unyielding Authority", `fanatical_agony` "Fanatical Agony",
`heat_tempered_carapace` "Heat-Tempered Carapace" (short "Heat-Tempered"), `calcified_frame` "Calcified Frame";
`slot` "".

## 6. Loot tables (unit `loot`, REWARD1; chances in %)

| Unit | Rows |
| --- | --- |
| `castellan_footman` | `watch_hookbill` 4, `sulfur_poultice` 6 |
| `watch_crossbowman` | `stirrup_arbalest` 4, `sulfur_poultice` 4 |
| `bailiff_enforcer` | `bailiff_maul` 5, `brimstone_draught` 8 |
| `mortuary_flagellant` | `sulfur_poultice` 5 |
| `inquisitorial_purifier` | `pyre_nozzle` 4, `brimstone_draught` 6 |
| `tallow_dredger` | `caustic_gall_sac` 15, `boning_knife` 4 |
| `render_beast` | `render_beast_tendon` 20, `stalker_optic_nerve` 10 |
| `ash_crawler` | `caustic_gall_sac` 15 |
| `calcified_husk` | `calcified_bone_shard` 25 |

## 7. Not in Act 1 data

The Inquisitor Proctor and the Guild Carver are lore templates for Quell, Nemeth, Orlov and Grendel; the Marrow Horror
first appears in Act 2 (its Strain riders are Act 2, Q-65). Their unit rows come with the Act 2 bestiary (Q-76).
