# BASERULES1 bound: audit, stats as data, skill values, report, proof

Companion of [BASERULES1.md](BASERULES1.md), section 4 (the stop rule, the contract). Parallel: [LAYOUTDATA1.md](LAYOUTDATA1.md). Size limits kept this out of the main file.
Marks: **[V]** read in code at `db55a63e`; **[R]** reasoning.

## 1. Audit: what is fixed in code

Ranked by cost to change later (cost grows with data and systems built on it). Verdict: **Code** genre logic stays; **Value** a rule value in a `base_rules` row; **Pack** role or stat
row; **Option** a named form; **Leaves** game-specific, should not name it in code. The test: what could a second game want to change without a builder?

### Tier 1: most expensive, every system builds on it

| What | Where | Verdict | Why |
| --- | --- | --- | --- |
| Stat ids | `rpgkit_data/lib.rs:21`, `check/simple.rs:81` | Pack | A second game's first question |
| HP, speed, armour base, crit, resist, range, carry, unarmed | `stats.rs:259-280`, `damage.rs:73`, `carry.rs:48` | Pack | Named reads, 11 sites |
| Move rate | `unit.rs:371` | Pack | 21 readers through one method; balance needs to tune it |
| Family names: `bow`, `crossbow`, `sling`; `orb`, `tome`; `fist` | `gear.rs:10,210`, `damage.rs:20` | Leaves: family `engine_rules` | The pattern already exists (`reach`) |
| Guard's lifetime | `resolver.rs:114-125` | Option `guard_model` | Decided |
| The hit's term order and two-unit read | `damage.rs:214` | Code | Stop rule |
| RNG draw per hit | `command.rs:733` | Code | Contract; every trace |

### Tier 2: values, cheap now (BASERULES1-V)

| What | Where | Default | Second game |
| --- | --- | --- | --- |
| `ARMOR_K`, `GUARD_CAP`, `CRIT_MULT` | `damage.rs:12,16,19` | 50, 0.9, 1.5 | Tuning; balance's main dials |
| `UNARMED_BASE_POWER`, `TRAINED_FIST_MIN_LEVEL` | `damage.rs:17-18` | 15, 2 | Unarmed classes |
| Tier scales | `stats.rs:29` | section 1.3 | Yes; tier ids stay code, as `when` and AI read them |
| `CLASS_BUDGET`, stat floor and ceiling, growth total and max | `stats.rs:8`, `check/unit.rs:13-16` | 62, 6, 18, 5, 3 | 0 = off gives unequal classes |
| `SPEND_BASE`, `FREE_PER_LEVEL`, `MAX_LEVEL` | `stats.rs:9-11` | 4, 1, 10 | Level cap and spend |
| Unlock: stat, base, step, max | `stats.rs:15-18` | KNOW, 12, 4, 2 | Stat id goes with Pack |
| Proficiency: max, per-level gains, `drill_scaling`, default scaling | `stats.rs:113-115,178-201` | 3, section 1.7, 1.25, 1.5 | Shape stays code |
| Wield penalty ratio, cap | `gear.rs:116` | 0.15, 0.60 | Tuning |
| `COUNTER_MULT` | `command.rs:17` | 0.5 | HISTORY moved Riposte's other numbers to data for this reason |
| `MAX_MOVE_BLOCKS` | `plan.rs:23` | 2 | Sits beside `move_after_blocks` |
| `ITEM_AP` | `command.rs:1397` | 1 | K3 |
| `SUPPORT_REACH`, `GEAR_REACH_CAP`, `REACH_WEAPON_REACH` | `skill.rs:12`, `unit.rs:11,18` | 3, 1, 2 | Tuning |
| AP clamps; `BASE_AP` | `skill.rs:156-157`, `unit.rs:10` | 1-5, 10 | Unit default AP is already data |
| Budget weights, target, tolerance | `budget.rs:12-21` | M2d | Balance owns them; decided as "tuned at balance" |
| Carry base, ratio, step | `carry.rs:10-12` | 30, 0.75, 10 | Tuning |

### Tier 3: waits for its owner

| What | Where | Verdict | Owner |
| --- | --- | --- | --- |
| Status table, bleed, debt, blocked blocks | `status.rs:47-69,202` | Rows | STATUSROWS1; values with STATUSPASS1 |
| Status names in rules (`weaken`, `haste`, ...) | `unit.rs:366-375`, `damage.rs:133` | Code until STATUSPASS1 says | Roles on status rows then |
| `GROUPS` spell, heal | `unit.rs:12` | Value | Later |
| Poise, flank, rest numbers | `ruleset.rs` | Already data | - |
| Forced tiles, foresight, stagger-taken caps | `forced.rs:26`, `unit.rs:14,16` | Value, low value | Later |
| Morale numbers | `encounter.rs:789-792` | Value | ENCOUNTER2 |

### Left as code on purpose

Hit pipeline; block resolver and `prepare_block`; slots and hands (`gear.rs:13`); proficiency's shape; shapes (`skill.rs:62`); `kind_intents`; AI heuristics (`ai.rs:20-28`,
`plan.rs:407`); grid constants (`grid.rs:9-12`); stance matchup numbers (legacy template, `unit.rs:159`); dojo values. **Engine crates hold none of this**: `aegis_core::formula` knows
names and numbers only.

## 2. Stats as data

### 2.1 Schema contract

Kind `stats` (`catalog/stats.json`, kit default when the file has no rows):

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | id, upper case allowed | required | A formula name, not a function, not a derived id |
| `name`, `short`, `description` | text | required, 1-5 chars for `short`, "" | Screens' label, sheet, tooltip |
| `order` | int | file order | |
| `default` | int | 10 | Value of a stat a class omits (`stats.rs:217`) |
| `scales_weapons`, `heals` | bool | false | Allowed in `scaling_stat`, `scaling_stats`, `heal_stat` |
| `primary` | enum "", `vitality` | "" | Sim's `cons` and `mixed` variants read it |

Derived row gains:

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| `role` | enum "", `max_hp`, `speed`, `move_rate`, `armor_base`, `crit_rate`, `status_resist`, `range_bonus`, `carry`, `unarmed_power` | "" | At most one row per role per ruleset; the row's `rulesets` scopes it |
| `round` | enum `int`, `none` | `int` | `none` returns the float: `status_resist` needs it |

New formula inputs, closed: `burden`, `carry_penalty`. A role with no row reads the default pack.

### 2.2 The default pack

`kits/rpg/data/default_pack.json`, `include_str!` from `rpgkit_data`, the one file that names a stat. Six stats as today, flags `scales_weapons` STR DEX INT, `heals` INT KNOW, `vitality` CONS. Roles, each byte-identical to
its code **[R, checked by `rules_golden`]**:

| Role | Formula | Stays in code |
| --- | --- | --- |
| `max_hp` | `60 + CONS * 8 + STR * 2` | Tier scale, floor 1 |
| `speed` | `max(1, max(1, 8 + DEX - burden) - carry_penalty)` | |
| `move_rate` | `clamp(floor((speed + 2) / 5), 1, 4)` | Haste, slow |
| `armor_base` | `gear_mitigation + round(STR * 0.25)` | Authored `armor`, fortify, expose |
| `crit_rate` | `5 + max(0, DEX - 10)` | Training, trigger |
| `status_resist` | `INT * 2.5`, `round` none | Per-status resist |
| `range_bonus` | `min(1, floor(PERC / 18))` | Only on `base_range` above 1 |
| `carry` | `STR * 3 + CONS` | Base capacity |
| `unarmed_power` | `STR + DEX` | Trained fists |

Also per pack, in `base_rules`: `unlock_stat`, `unfit_melee_stat`, `unfit_ranged_stat` (`damage.rs:96-98`), `default_scaling_stat` (`source.rs:814`, `gear.rs:124`, `lib.rs:473`).

These rows are hidden by `show_on` empty, so no screen moves. A role row's parts include modifiers on its id, so a passive may name `max_hp`: new reach, no shipped row uses it. Live reads stay live: crit is read per hit today, so a unit holds an `Arc` of parsed roles rather than a cached number **[V: `crit_rate` in `damage.rs:143`]**.

### 2.3 What reads stats, what breaks

| Reader | Today | Change | State |
| --- | --- | --- | --- |
| Stored values | `core` is a string map, `stats.rs:208` | Loop `Catalog.stat_ids()` | Open **[V]** |
| Modifiers | `StatModifier.stat` string | None | Open |
| Saves | `spend` a string map, `state.rs:42,353` | None; a removed stat is a data update like DRILLSAVE1 | Open |
| AI | Reads no stat, only numbers (`ai.rs`) | None | Open **[V]** |
| Dialogue | `stat` condition by id, `dialogue.rs:187`; check `check/dialogue.rs:316` | Check reads rows; 1 shipped use | Nearly open |
| Item modifiers, tiers, passives | Strings; checks `item.rs:493,575`, `simple.rs:124`, `mod.rs:102` | Check reads rows | Check only |
| Class checks | `check/unit.rs:33` | Reads rows; messages keep today's text for the default pack | Check only |
| Derived | `derived.rs:249`, `check/derived.rs:173` | Reads rows, 2 new inputs | Close |
| Gear scaling | `check/item.rs:23,1004`, schema options | `scales_weapons` flag | Closed list |
| `heal_stat` | `check/skill.rs:534`, schema options | `heals` flag | Closed list |
| Schema | 6 fields hold `keys` (read by no Rust: grep **[V]**, tool hints only), 4 hold `options` (enforced, `schema.rs:243`) | `options` on the 4 becomes `ref` to `stats`, already a type (`schema.rs:260`); `keys` dropped or generated | Small; **check messages in `checks_golden` may change wording**: keep them |
| Named reads | 11 in `stats.rs`, `damage.rs`, `carry.rs`, `unit.rs:798` | Roles | Closed: the main work |
| Sim | `variants.rs:46-57` names CONS; `primary_stat` reads growth | `vitality` flag | One site |
| UI | `party.rs:293-298` help text; loops `party.rs:1565-1619,2110-2164`, `dojo.rs:409`, 2-column sheet | Help from `description`; `Catalog.stat_ids()`. A 6-row layout may not fit 8 | Game; not proven (below) |
| Text tokens | `{req:STAT}`, `{bon:STAT}`, `check/item.rs:850` | Reads rows | Check only |
| Data | Class `base_stats`, `growth`; unit `spend`; passive `stat` | Change with the pack; asset lane | Data |

**Totals:** about 30 sites. Closed by code but mechanical; nothing in the resolver, planner, AI or saves changes. Roughly 70 % of what touches a stat is already string-keyed data **[R]**.

### 2.4 A more detailed pack

8 stats, none sharing an id with today's, as the proof's asset (first pass; a test pack, not a design):

| Id | Name | Roles | Flags |
| --- | --- | --- | --- |
| MIG | Might | `carry` x3, `unarmed_power`, `armor_base` | `scales_weapons` |
| FIN | Finesse | `crit_rate`, `unarmed_power` | `scales_weapons` |
| QUI | Quickness | `speed` | |
| VIT | Vitality | `max_hp` x8, `carry` | `vitality` |
| WIL | Will | `status_resist` x2.5 | |
| WIT | Wit | unlock | `scales_weapons`, `heals` |
| AWR | Awareness | `range_bonus` | |
| LOR | Lore | none at first | `heals` |

Class rows redistribute to the same budget (8 x 6 floor needs a base budget above 62: set `class_budget` 80, as V allows). Tier ids, gear and skills unchanged but item requirements and scaling.

## 3. Skill values

### 3.1 Fields audit

| Field | Rows | Today | Verdict |
| --- | --- | --- | --- |
| `power_mult` | 86 authored | Multiplies Attack Power | **Baseline value.** Layer 1 |
| `blocks`, `ap_cost`, `recovery_blocks` | all, 39, 41 | Timing and price | Baseline |
| `level` as AP | 90 rows with no `ap_cost` | AP = level | **Leak**: price tied to unlock. Data: set `ap_cost` (BASERULES1-A) |
| `stagger`, `mitigation`, `status_potency`, `status_magnitude`, `status_rounds` | 53, 9, 51, 19 | Read by rule | Baseline: abstract strength |
| `heal_base`, `heal_mult`, `heal_stat` | 6 | `base + roundi(stat x mult)` in the row | **Leak.** Option `heal_model`: `authored` (today) or `value`: `roundi(heal_value x role heal_power)`. Per-row base and mult differ (8 + 1.5, 25 + 2.0, 8 + 0.3), so one form cannot reproduce them: frozen rows stay `authored` |
| `armor_piercing`, `ignore_guard` | 2, 1 | Pick a form | Keep: a flag, no number, no knowledge of the formula beyond its name |
| `weapon_types`, `min_proficiency`, `weapon_hand` | 40, 40, 1 | Gates and hand | Keep: who may use it |
| Weapon scaling (item `scaling_stat`, class `scaling`) | gear | Base rules already | In place |
| Item flat `damage`, `heal` (bomb 45, 40; potion 90, 60) | 4 | Absolute, round every rule | **Leak in reverse.** Option `item_value`: `flat` (today) or `value`: hit by `value x flat_or_item_power`, same term as a skill's. Q3 |
| `buff_pct`, `buff_stat` | 2 | Wrong | Item `grant_status`, `status_magnitude`; `buff_stat` retired after K3 |
| `hp_cost` | 1 | Absolute | Leave; one row |
| Unit `power`, `armor` | 1, 14 | Replace the formula | Not skill data; allowed; report lists them |

### 3.2 `damage_model` flat

Row fields: `damage_model` (`formula`, `flat`), `flat_damage` (default 40), `flat_hp` (0 = the unit's), `flat_speed` (0 = the unit's). A test row, for example `feel_flat`, copied from the picked row.

`flat`: in `damage::compute` an early branch through `HitInput.flat: Option<f64>`: `flat_damage x skill_mult x (1 - min(cap, guard + cover))`, rounded, min 1. Keeps the crit RNG draw, so traces stay comparable; no crit. `flat_hp`, `flat_speed` set at `Session::start` after the build, one place. Weaken, empower, exploit, flank and Field bonuses stay, as they are skill mult. Riposte and collisions go through the same branch.

Does the sim allow it? **[V]** `sim feel ruleset=` takes any row (`args.rs`), `rules=` switches extensions, `tune=` sets numbers, `melee_diagonal=` shows the enum pattern (lane 1 adds `attack_lock=` the same way). All 9 `tl_` units are `derived`, so their stats still set HP and speed unless `flat_*`; that is why those two exist. A catalog-only stand-in (`stat_mode` fixed, authored `power`, `armor` 0) fails: a skill's `min_proficiency` needs gear training, and crit stays. Sites: `land` (2), `roll_hit_with`, `expected`, the collision, Dojo. Size S.

Accept: under `flat` two units with very different stats and gear hit equally for the same skill; a guard reduces it; `sim feel enc=tl_skirmish ruleset=feel_flat` runs with 0 PLAN FAIL.

### 3.3 Value bands as data

Today only gear has a budget: item value by weights, class loadout near a target, warning level, `special` with a note excuses (decision "Gear budget constants (M2d)"; `check/item.rs:931`). Skills have none: `check/skill.rs` warns on text and timeline fields only **[V]**.

Kind `value_bands`:

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | id | required | |
| `rulesets` | refs | [] = all | As derived rows |
| `where` | object: `kind`, `blocks`, `intent` | {} | Closed keys |
| `field` | enum `power_mult`, `stagger`, `mitigation`, `status_potency`, `heal_value` | required | |
| `per` | enum `none`, `block`, `ap`, `level` | `none` | Fixed division; no expressions |
| `min`, `max` | float | required | min not above max |

The check (`warnings_all`, soft, as M2d): `Skill X: power_mult per block 1.10 is over band attack 0.40-0.80 (or mark it special with a note)`. No row, no warning, so frozen catalogs and `checks_golden` do not move.

Why bands first, not a weighted budget as M2d: a band is one number a balance agent can read and set; a skill's weights (riders, status, reach) are unknown today: riders show no price in the data (main spec, weakness 10). A weighted skill value can come later as `per: budget` if the band proves too coarse.

The formula's own range: a **reference hit**: `aegis rules --hits` prints a 1.0 skill from a reference attacker on a reference defender, by level, as a share of HP, from named rows. Balance sets the band and tunes `ARMOR_K`, scaling and role rows to land in it. Size: with BN.

### 3.4 The limit

A formula value moves every skill. It cannot separate one skill, or heavy from light, or change the ratio of skill value to stats without moving them all. So per skill stay value, blocks, AP, status, stagger and riders; per class proficiency scaling and stats; per unit gear and tier. Order for a miss: a whole-range miss is the formula; a family's miss is its band; one skill's miss is its row.

## 4. The tweak report

`aegis rules [system] [--json] [--locked] [--contract] [--hits]`. Text shape:

```text
BATTLE (ruleset timeline, row rulesets.timeline)
  value  stagger_windup_mult  2.0  range 1.0-4.0  needs poise  [set]
  option attack_lock          melee  of none, melee, all  bundle positioning  tested
  rule   flank                on  requires round_intent  bundle positioning
STATS (pack default)
  role   move_rate  clamp(floor((speed + 2) / 5), 1, 4)  row: kit default
LOCKED
  hit term order    damage.rs  a new term is a new option
  gear slots        gear.rs    genre logic
```

Sources: `schema.json` (type, default, min, max, options, help), register metadata, role table, `LOCKED`. Nothing by hand but `LOCKED` and its reason, held by the source-scan test. Run by `tools/fc.sh rules`; printed in each flush note. Size S: about 200 lines in `rpgkit_host` plus tests, with the metadata row.

## 5. The proof

Like the main menu for layouts: objective, small, no Rust after the feature lands.

1. **Rename pack.** A scratch data copy of the frozen catalog with STR to S1, DEX to S2, and so on: `stats` rows, role rows, every class, unit, item and passive reference rewritten by the test. Fight the `tl_` encounters: **the trace must equal the timeline golden step for step**, as no number moved. This is the `shot_diff` of this spec.
2. **8-stat pack** (2.4) on `tl_skirmish`: content check clean, `sim feel seeds=10` with 0 PLAN FAIL. Numbers unjudged.

**Stop rule:** if it needs a Rust edit after P, a new function or input in the grammar, a new role, or a condition: stop and report the exact need, as LAYOUTDATA1 7.1 does. Size S.

**What it does not prove:** that the screen fits 8 stats (the Character popup's 2-column sheet, Dojo); balance of any pack; World Forge widgets; save migration when a shipped pack changes; a stat with a new kind of rule (Luck rolling something); story text naming stats; a Sunk Ward fight (the `tl_` fights are the test).

## 6. Against over-generalising

Only for this game, its test bed, or the top-down test game (memory, "The Low Sleep"), or at almost no cost. Passing: stat ids (the user's ask), roles (DERIVE1 and `poise_stat` already show the pattern), values (one row). Not passing: a damage pipeline as data; a condition language; per-world packs (classes are catalog-wide, so a pack is per game); damage types; tier ids; slots as data; status behaviours as data before STATUSPASS1.

## 7. The guard term (BASERULES1-G)

Behind two new row fields, defaults today's:

- `guard_model`: `block` (today) or `status`. Under `status`, a Defend's impact grants its `mitigation` as a status; `prepare_block` sets
  `guard = max(status magnitude, held guard)`; the status row is `guard_status` (a new row in the status table; not intent-bound).
- `guard_start`: `start` (today) or `impact`. Only read under `block`; `status` starts at impact by its nature.
- Acceptance: a hit in a guard's wind-up is unreduced under `status`; two guards take the higher; `ignore_guard` still bypasses;
  poise's guard factor applies while the status holds; classic and the frozen goldens stay byte-identical.
