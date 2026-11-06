# BASERULES1: the base rules pass, before the battle work

Status: **partially superseded (2026-11-07)**.

- **Active**: Sections 1-2 and 6 for `BASERULES1-M` (rule metadata register, `check/ruleset.rs`, `aegis rules --contract`, queued in `docs/READY.md`) and `BASERULES1_LIBRARY.md` as the candidate behaviour reference.
- **Superseded**: Sections 3-5 and 7-8 (`BASERULES1-V`, `-P`, `-PP`, `-F`, `-G`, `-H`, `-D`, `-B`) are superseded by the bottom-up session specs `docs/sessions/U1_STATS.md` and `docs/sessions/U2_CLASS.md` (and upcoming `U3`/`U4`/`A2` sessions). `BASERULES1-F` (`feel_flat`) is retired in favour of `U2` Static units. **Do not build `BASERULES1-V/P/F/G/H` from this file.**

Depends on: rows follow ATTACKLOCK1 where they touch the same files.
Lane: logic, asset, balance (section 7).
Touches: per row, section 7.
Companions: [BASERULES1_LIBRARY.md](BASERULES1_LIBRARY.md) (every rule option, compatibility, bundles, dropped ideas);
[BASERULES1_BOUND.md](BASERULES1_BOUND.md) (the audit table, stats as data, skill values, the tweak report, the proof);
`games/emberwake/docs/specs/BASERULES1_VALUES.md` (this game's units, numbers and picked set).

Decisions: "Order of work: engine and kit first", "Engine options are additive; a game picks one set".
Marks: **[V]** verified in code or data at `db55a63e`; **[R]** reasoning. Stat numbers come from a Python replica of the rules
over shipped rows (it leaves out passives and drills, and no build was run). Nothing in `HISTORY.md` is re-proposed unless library section 6 says what changed.

## 1. The foundation as it is

### 1.1 Primary stats

Six ids, `rpgkit_data::STATS` (`lib.rs:21`). What reads each **[V]**:

| Stat | Reads | Where |
| --- | --- | --- |
| STR | HP +2 a point; armour +1 per 4; weapon scaling; unarmed power; carry x3; item requirements and tiers | `stats.rs:259,267`, `damage.rs:73`, `carry.rs:48` |
| DEX | Speed +1 (move rate, initiative); crit +1 % above 10; weapon scaling; unarmed power | `stats.rs:263,276`, `damage.rs:73` |
| INT | Status resist 2.5 a point; weapon scaling | `stats.rs:280` |
| CONS | HP +8; Max Poise x2; Composure +1; carry +1 | `stats.rs:259`, `derived_stats.json`, `carry.rs:48` |
| PERC | At 18 or more, ranged weapons reach 1 more | `stats.rs:271` |
| KNOW | Heals (`heal_stat`, every shipped heal); family unlock 1 level sooner at 16 and 20; Composure x2; one dialogue check | `command.rs` `heal_amount`, `stats.rs:48`, `dialogue.rs:187` |

### 1.2 Derived numbers

| Number | Rule | Where |
| --- | --- | --- |
| Max HP | `60 + CONS x 8 + STR x 2`, x tier | `stats.rs:259`, `unit.rs:407` |
| Speed | `max(1, 8 + DEX - burden)`, minus carry overload, floor 1 | `stats.rs:263`, `unit.rs:409` |
| Move rate | `clamp(floor((speed + 2) / 5), 1, 4)`, +1 haste, -1 slow, floor 1 | `unit.rs:370` |
| Armour rating | Worn mitigation + `round(STR x 0.25)` + fortify - expose, floor 0; a unit's `armor` replaces the first two | `damage.rs:123-134` |
| Crit | `5 + (DEX above 10) + (training - 1) + trigger`; x1.5 | `damage.rs:141`, `stats.rs:276` |
| Status resist | `INT x 2.5` + per-status resist, as a float | `status.rs:210` |
| Max Poise | Row `max_poise`: `CONS x 2 + gear_mitigation`, or the unit's `poise` | data, `poise.rs:93` |
| Composure | Row `composure`: `KNOW x 2 + CONS`. Only portrait cues read it; no rule does | data |

DERIVE1 (`clamp(roundi((formula + flat) x (1 + percent)), min, max)`) is live for those last two rows only; the rest is code.

### 1.3 Level and growth

Levels 1-10. A class row: base stats 6-18 summing to 62, growth summing to 5 a level with at most 3 a stat, enforced by
`check/unit.rs:13-16`. A unit: class base + spend (pool `4 + level - 1`) + growth x (level - 1). Tiers scale HP and power:
minion 0.4/0.6, standard 0.7/0.8, hero 1/1, elite 1.25/1, boss 1.5/1.1 (`stats.rs:29`). The XP curve is already data. **[V]**

### 1.4 Damage and mitigation

`damage::compute` is the one formula. **[V]**

```text
Attack Power = roundi(((base + tier + training) x (1 - wield penalty) + stat x scaling) x tier power), min 1
               (no weapon: STR + DEX, plus 15 + training with trained fists; an authored `power` replaces it all)
Skill mult   = power_mult x potency x weaken x empower x exploit x flank
Hit          = Attack Power x Focus x Skill mult x Stance x Crit
               x (1 - min(0.9, Guard + Cover)) x (1 - rating / (rating + 50)), rounded, min 1
```

Worked hit: weapon 18, training 2, STR 13, scaling 1.5: `roundi(20 + 19.5)` = 40. Rating 25, 1.0 skill: `40 x 2/3` = 26.67, so 27. Under a 0.3 guard: 19. A crit: 40.

- **Guard** is a per-block field, zeroed in `prepare_block` and raised by any running Defend from its start (`resolver.rs:114,125`).
- **Cover** is terrain `cover_guard` (0.3, 0.15, 0.15), ranged hits only, added to Guard under the 0.9 cap (`command.rs:1229`).
- **Pierce** drops the armour term under its rule. **Ignore guard** drops Guard, keeps cover.
- **Riposte**: 0.5 x weaken x empower (`COUNTER_MULT`).
- **Crit** draws one RNG number per hit, wanted or not (`command.rs:733`). That draw order is part of every trace.
- **Resistances** apply to statuses only. `damage_type` reaches only effects (`playback.rs`).

### 1.5 Poise and stagger

A skill's `stagger`, x windup 2 x guard 0.5 (while a Defend runs) x repeat 0.5 x flank 1.5 x passive, spends poise; at 0 the unit
breaks. Refill each round. Rule `poise` only; POISE1 revisits it. The Staggered status is separate: it blocks blocks 1 and 4. **[V]**

### 1.6 AP and movement

`unit_ap` 12, `start_ap` 10 where set, `ap_regen` 4 (encounter fields). AP = `ap_cost` under its rule (set on 39 of 129 skills), else
level clamped 1-5. Move 1 AP a tile. Items 1 AP (`ITEM_AP`; INTENT2 makes it 0). Rest under the round intent: 0.5 a block, cap 2, at
round end. Overspending is allowed and queues Stagger at 3 and Stun at 5 of debt (`status.rs:68`). Enemies pay nothing under
`enemy_moves`. **[V]**

A Move opener takes up to `MAX_MOVE_BLOCKS` 2 (`plan.rs:23`); a Move after a skill up to `move_after_blocks` 3, both at move rate
tiles a block and 1 AP a tile, so AP is the real cap. Initiative in a block is `priority x 1000 + speed` (Defend 1, Target 2, the
rest 0; `command.rs:276`), so speed decides who lands first in a block. **[V]**

### 1.7 Gear, proficiency and carry

- **Slots** (`gear.rs:13`): main, off, head, body, two accessories. **Hands**: `main_1h`, `main_2h`, `off`.
- **An item** is a family row (`base`, `bands`) plus modifiers (`adjust`, `requirement`, `stat_bonus`, `effect`) and tiers.
- **Wield penalty**: 15 % of weapon power per point short, cap 60 %, after a discount of 2 per training level (`gear.rs:116`).
- **Budget** (M2d): one weighted number per item, class loadouts near 23 +- 20 %, warning only (`budget.rs:12-21`).
- **Proficiency**: per class `{gear: {level 0-3, scaling 1.0-2.0}}`. Level gives +1 power each, -2 requirement each, crit and
  burden from level 2. Level 0 cannot be worn. Drills add levels (UNITDRILL1); `wears_untrained` excuses single items.
- **Carry**: party-wide capacity `30 + sum(STR x 3 + CONS)`; over it, every member loses 1 speed per 10 weight (`carry.rs`). **Burden** is per unit.

### 1.8 Items, classes and the statuses damage needs

- **Consumables**: 6 rows; a pouch of 3 ids, stacks with INTENT2-K3; flat `heal`, flat `damage`, `buff_pct`; a synthetic SUPPORT skill (`skill.rs:212`).
- **A class** holds base stats, growth, proficiency, `skill_path`, a default gear set and pouch, an AI role and art. No passives.
- **Statuses in the formula**: weaken and empower on skill mult, fortify and expose as flat rating, haste and slow on move rate.
  Bleed, poison, burn and regen go round the formula. Chance = potency - resist. STATUSPASS1 owns the rest.

## 2. What is wrong or weak, ranked

Numbers per `+4` on a stat (the spend pool of a level 1 unit) are in the values file, section 2. **[V]** unless marked.

1. **Part of the budget buys little.** +4 PERC: 0 damage, 0 survivability, range +1 for 7 of 47 units. +4 KNOW: 0 and 0 for every unit;
   it pays only with a heal skill or an unlock. Only 1 of 47 units reaches PERC 18. Every class pays 9-14 points for PERC at level 1, and
   5 of 7 pay 1 of 5 growth a level for it. "Equal budget 62" (decision "Class balance") is equal in points, not in effect.
2. **Stand has no place in the formula.** `guard` is a per-block field from a running Defend, starting at the Defend's start, so a
   mitigation status for one or more rounds (decisions "What Stand is", "Stand answers") has nowhere to live. `stagger_guard_mult`
   also reads "a Defend runs". See section 3 and BASERULES1-G.
3. **Move rate saturates.** 26 of 47 units are at the cap of 4 tiles a block, 3 at 2. +4 DEX steps only 20 of 47. A 3-block Move at
   the cap is 12 tiles on a 12 x 9 board, limited only by AP. Any Move of 2 or more tiles beats a 1-tile chase, so under the attack
   lock speed does not decide who can dodge; what it decides is the same-block order: a Walk and a swing share priority 0, so the
   faster unit lands first. **[R]** on lane 1's uncommitted build. Balance should measure this before INTENT2.
4. **Items go round the rules.** `damage` uses `w.damage(target, n, true)`: no armour, guard, cover or crit. `buff_stat` has no reader,
   so `buff_pct` always boosts attack (`command.rs:635`), and `apply_attack_boost` overwrites, so a weaker buff can replace a stronger
   Focus. Reading the stat as named would not help: a CONS buff would not raise max HP, as stats are not re-derived mid-battle
   (decision "HOOKS1-M and deck answers"). So: an item grants a status (BASERULES1-H and K3).
5. **Priced but never read.** `Equipment::effective_block_cost` has no caller outside `gear.rs`. So gear `block_cost` (2 items), the
   wield penalty's extra block and the level 3 training perk do nothing in battle, while the budget credits `-3` for each extra block
   and the brief lists the perk.
6. **Stance leftovers.** Under `stances` off every unit stays Neutral, `stance_factor` is 1.0 and the four fields are inert:
   `shield_bash`, `intimidate` (`stance`) lose nothing. `feint` (`ends_in_stance`, 0.7 power) and `crushing_blow` (`disrupt`) lost the
   price they were priced for: at rank 1, Crushing Blow (1.0, 2 blocks, AP 2) is worse than Strike (1.0, 2 blocks, AP 1). Their
   text tokens (`{flow}`, `{disrupt}`) and the Move and Rest descriptions still speak of stances. No check warns.
7. **Hard-coded values** a second game would want to change: section 4 and the audit. The sharpest: `ARMOR_K`, `GUARD_CAP`, `CRIT_MULT`,
   `COUNTER_MULT`, tier scales, the budget 62, and family names in code (`gear.rs:10,210`, `damage.rs:20`).
8. **Numbers a rule ignores, smaller cases.** An authored `armor` (14 of 47) replaces gear for armour, yet `gear_mitigation` still feeds
   Max Poise. A drill trains at scaling 1.25 while an entry-less level 0 reads 1.5 (`stats.rs:172-175`).
9. **One stat answers every status.** At median potency 70, INT 6 lands 55 %, INT 17 27.5 %, a level 5 INT class 2.5 %. For STATUSPASS1.
10. **Skills do not pay for their riders.** Median power per block is 0.57 with no rider and 0.47-0.55 with 1-4. There is no skill
    budget at all (`check/skill.rs` has none). Section 5.

## 3. What the framework decisions need from base rules

| Decision | Needs | Row |
| --- | --- | --- |
| What Stand is; Stand answers | A defined guard term: `guard = max(running Defend, mitigation status, held guard)`, starting at impact; poise's guard factor reads the same; riposte stays bound to its intent | G |
| Skills hold baseline data | The row holds value, base rules hold the number (section 5) | H, B |
| Attack lock at wind-up | Nothing in the formula; speed's meaning in the same block should be stated and measured | Balance |
| Intent framework answers | A place for `splash_mult` and the `power` bonus (both are skill mult factors); a thrown item through the same term (Q3) | K2, K3, H |
| Consumable stacks | Weight stays per copy (`carry.rs:61`); a status grant replaces `buff_stat` | K3, H |
| Move after a skill | A named value for the opener's 2 blocks beside `move_after_blocks` 3, and a move rate row balance can tune | V, P |

Rule for the guard term, with fields and acceptance: bound file 7. Two row fields, `guard_model` and `guard_start`, defaults today's.

## 4. The bound

The test: **what could a second game want to change without a builder?** Each build states what is tweakable and what is locked. A
separate behaviour is a new option that expands the engine. Systems iterate in silos and integrate once each is well defined.

- **Data sets values and picks among forms the rule engine already has. A new kind of rule stays code**, as a new option with its own
  task row.
- **No new language.** One already exists: `aegis_core::formula` (DERIVE1-E, 706 lines; `+ - * /`, `min`, `max`, `clamp`, `floor`,
  `ceil`, `round`; 256 characters, 64 tokens). LAYOUTDATA1's "no arithmetic" cannot apply literally. Weighted terms alone cannot reproduce
  crit, move rate or armour byte for byte (they need `max`, `floor`, `round`). So: the grammar is frozen, used only by derived rows,
  and has no conditions, no reads of a target, an event or a block (Q2).
- **Sends a case back to code:** a new function or input in the grammar; a condition; a reading of anything but the unit's own stats,
  gear and level; a new term, order or roll in the hit; a new role; a new kind of status behaviour.
- **The test on the damage formula.** Values that leave code: `ARMOR_K`, `GUARD_CAP`, `CRIT_MULT`, unarmed base, tier scales. Inputs that
  become role rows: crit rate, armour base, unarmed power. What stays code: the term order, the two-unit reading, the RNG draw,
  armour's `r / (r + k)` form. A game wanting subtractive armour gets option `armor_form` from a builder. Result: 5 values and 3 inputs
  leave, no structure leaves.

Stats as data (bound file 2): stat ids and names are rows; a **role** on a derived row says what it answers (`max_hp`, `speed`, `move_rate`, ...);
code reads roles, and one kit file holds the default pack, as `Ruleset::classic` is a built-in fallback (CLN-44). About 30 sites close the set today.
The store, modifiers, saves, the AI and dialogue conditions are already open.

Proof: rename all six stats and fight the timeline test fights on a frozen catalog with no Rust edit: the trace must equal the golden step
for step. Then an 8-stat pack runs clean. Size S after BASERULES1-P. It does not prove UI fit, balance, or a stat with a new kind of rule.

Against over-generalising: generalise only what this game, its test bed or the top-down test game needs, or what costs almost nothing. Left
as code on purpose: bound file 1 and 6.

### 4.1 The tweak report (each build states it)

`aegis rules` (`tools/fc.sh rules`), generated from `kits/rpg/schema.json`, the register's metadata and the role table: per system, each value
with range and default, each option with values, requirements and bundle, then what is locked and why. Locked is a kit table `LOCKED`; a test scans
`rpgkit_rules/src` and fails on a numeric `pub const` neither promoted nor listed, so it cannot go stale. Size S. Output shape: bound file 4.

### 4.2 The contract other silos read

**"Defining what a skill does should not require you to know how the damage for the skill is calculated, so we handle that in the base
rules, while for combat we use the direct data for skill value, then the integration connects the two; the balance already works in a
range so it can assign an appropriate value range for the skills, and from base rules we calculate the number and tweak the formula to
bring it in the balance range."** (user, 2026-11-05)

Three layers: **a skill row holds an abstract value** (`power_mult` is close); **base rules turn value plus attacker and defender into a
number**; **balance sets ranges** (per block, per AP, per level) and tunes the formula to land in them.

| Silo | May rely on | Must not read | Free to change inside its bound |
| --- | --- | --- | --- |
| Base rules gives | `compute(attacker, target, value)`: a hit of at least 1, one RNG draw per hit; `heal_amount`; `max_hp`, `speed`, `move_rate`, `armor_rating`, `crit`, status chance; `can_wear`, slots, reach, pouch counts | - | Stat pack, role rows, values, tiers, budgets, gear numbers |
| Battle | The above; skill values | Stat ids, formula constants | Skill rows, timing, intents, AI |
| Exploration | Dialogue `stat` conditions by id, carry penalty, stash and pouch | The hit, roles | Maps, travel, quests |
| Party, progression | Level, spend, growth, `can_wear`, rows with `show_on` | Formula constants | Screens, XP data |

A **contract change** is a change to the signature above, the RNG draw order, the role list or a role's meaning. `aegis rules --contract`
prints that surface; a small golden holds it. A change fails the test, and like every golden is re-recorded only when intended, named in
the commit and a `HISTORY` row, and the planner tells the other silos. Changes inside a bound never touch it.

## 5. Skill values

Full audit in the bound file 3. What moves: `heal_base`, `heal_mult`, `heal_stat` become a heal value (option `heal_model`); flat item
`damage` and `heal` become a value (option `item_value`); `level` as AP gives way to `ap_cost` in data. What stays baseline: `power_mult`,
`blocks`, `ap_cost`, `stagger`, `mitigation`, `status_potency`, magnitudes, `recovery_blocks`. Flags that pick a form stay: `armor_piercing`,
`ignore_guard`. Gates stay: `weapon_types`, `min_proficiency`.

**Silo test option** `damage_model`: `formula` (default) or `flat`, with `flat_damage`, `flat_hp`, `flat_speed`. Under `flat`,
`Hit = flat_damage x Skill mult x (1 - min(cap, Guard + Cover))`: no stats, armour or crit. Battle's feel, dodge and timing can then be tested with
no dependency on stats or gear, and it stays in the library. The sim already takes any row. Size S. Bound file 3.2.

**Value bands** (BASERULES1-B): kind `value_bands`, a row `{id, rulesets, where, field, per, min, max}`; the check warns on a skill outside its band;
`special` with a note excuses it, as M2d does. Balance owns the numbers. Bound file 3.3.

**The limit.** Tuning the formula moves every skill together. It cannot fix one outlier or one class. So per skill stay value, blocks, AP, status and
riders; per unit stay gear and tier. A formula value is a shipped data row, and `rules_golden`, `checks_golden`, `sim_golden`, the timeline golden and
the journey load a frozen catalog **[V]**, so tuning it moves none of them; PHASEFOLD reads shipped data and may move, an intended re-record. Changing a
kit default moves them all. `tune=` lets balance try a value with no edit (`args.rs:457`).

## 6. How the engine enforces it

Full design: library section 7. In short: `REGISTER` entries gain `requires`, `conflicts`, `bundle`, `system`, `tested`; row fields gain `needs`; `check/ruleset.rs`
errors on a rule that would be silently inert (`flank` without `round_intent`) and warns on the rest; `adjust` warns and never refuses, as the Arena and sim exist to
try things. **Additive by construction:** a new rule is off when unlisted, a new field defaults to today's behaviour, metadata refuses only what a row sets, so a new
option cannot change an existing game; `rules_golden`, the timeline golden and a row-less `Ruleset::classic` hold it.

**One set, how many rows?** A game's manifest names one: `game.json` `ruleset` (the picked row) and `test_rulesets` (rows allowed in worlds and tools for testing). Several
rows stay, as the Arena, Dojo and sim need them and each is a working template of the library. Q4.

## 7. Active build slice (`BASERULES1-M`) and superseded rows

| Id | Title | Layer | Lane | After | Size | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| BASERULES1-M | Register metadata, `check/ruleset.rs`, bundle tests, `aegis rules` with `LOCKED` and `--contract` | kit | logic | ATTACKLOCK1 | S | `ruleset.rs`, `check/ruleset.rs`, `rpgkit_host`, `tools/fc.sh`, tests |

Rows `BASERULES1-V`, `-D`, `-G`, `-F`, `-P`, `-PP`, `-H`, `-B`, `-A`, `-BN`, and `-U` are superseded by the bottom-up session specs (`docs/sessions/U1_STATS.md`, `docs/sessions/U2_CLASS.md`, and upcoming `U3`/`U4`/`S1`/`A2` sessions). `BASERULES1-DOM` (dominance audit) runs at `U1.5`.

## 8. Out of scope and evidence

Out of scope: tuning, status rules (`S1`), skill rows (`A1-A4`), the 8-stat pack as a design.

Verified: file and line references; the stat table by grep; row counts by script; dead fields by grep (no reader); goldens' sources from their headers.
