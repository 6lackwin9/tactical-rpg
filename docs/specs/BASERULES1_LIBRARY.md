# BASERULES1 library: every rule option, what works together, what was dropped

Companion of [BASERULES1.md](BASERULES1.md). Marks: **[V]** verified in code or a test; **[R]** reasoning. "Own test" names a file in
`rpgkit_battle/tests/`; "golden" is the timeline golden, which fights the full `timeline` row on a frozen catalog.

## 1. Rules in the register

Each is on when listed in the row's `rules` (`ruleset.rs:64`). Option means an `adjust` refuses it; Extension switches one by one. `classic` lists none and
`timeline` lists all 19 **[V]**.

| Rule | Kind | What it does on | Off | Really tested |
| --- | --- | --- | --- | --- |
| `round_intent` | Option | One intent and target a round, follow-ups, combined commands, facing, flanks drawn, intent-bound buffs | Classic planning | `round_intent.rs`, golden |
| `field_intent` | Option | `Field` round intent and `IR21` derivation of allowed intents (`intent::allowed`); requires `round_intent` | Legacy single intent (`IN4`) | `field_intent.rs`, `ir21_intent.rs` |
| `enemy_moves` | Option | Enemies pay no AP, book `recovery_blocks` | Everyone pays | `timeline_timing.rs`, also without intent |
| `deck` | Option | Basics open, slots, slottable passives, order lists | 5 + 3 loadout | `deck.rs`, also without intent |
| `poise` | Extension | Max Poise, stagger, breaks, windup factor, Rest poise | None | `poise.rs`, alone in `battle_talk.rs` |
| `ap_cost` | Extension | Skill field replaces level as AP base | AP = level | `timeline_timing.rs` |
| `pierce` | Extension | `armor_piercing` drops the armour term | Ignored | `poise.rs` (`-pierce`) |
| `reach` | Extension | Family rule: melee band 1-2 | Reach 1 | `reach.rs`, alone and with intent |
| `flank` | Extension | `flanked` exploit, power, no riposte | No bonus | `round_intent.rs` (`-flank`) |
| `plan_badges` | Extension | Impact badges, intents in tells, reveal flag, foresight | Classic tells | `plan_badges.rs` |
| `forced_move` | Extension | Push, pull, collision | Ignored | `forced_move.rs` |
| `ground_hazards` | Extension | Skill and item hazards on tiles | Ignored, stripped from the catalog copy | `ground_hazards.rs` |
| `arena_hazards` | Extension | Scheduled hazards | Off | `arena_hazards.rs` |
| `triggers` | Extension | 15 events, riders, held guard, intercept | Off | `triggers.rs`, `riders.rs` |
| `unit_meters` | Extension | Meters, bands, meter costs | Off | `meters.rs` |
| `hp_cost` | Extension | Skill HP cost | Ignored | `riders.rs` |
| `ignore_guard` | Extension | Skill drops guard, keeps cover | Ignored | `riders.rs`, `poise.rs` |
| `ai_tactics` | Extension | SOFTEST, TOUGHEST, unit tactics | Role preference | `tactics.rs` |
| `execute` | Extension | `hp_below` exploit | Never matches | `execute.rs` |

## 2. Row fields and values

| Field | Values (default) | Effect | Who sets | Tested |
| --- | --- | --- | --- | --- |
| `rest_ap` | "" , `on_use`, `round_end` | When Rest pays. "" by the rules | classic `on_use`, timeline `round_end` | `ruleset_switches.rs`, each both ways |
| `chase` | "", `classic`, `every_skill` | Which skills follow 1 tile | classic, every_skill | Same |
| `root_stops_chase` | "", `on`, `off` | Root stops a chase | off, on | Same |
| `switch_target_step` | "", `on`, `off` | Inserts Switch Target on a new target | on, off | Same |
| `stances` | "", `on`, `off` | Whole stance machinery | on, off | Same, plan entry only: no fight on intent + stances |
| `melee_diagonal` | `none`, `maneuver`, `all` | Which hits count a diagonal | none, maneuver | `maneuver.rs`; `all` by Arena and sim key |
| `flank_protected_cost`, `_ap` | `full`, `half`, `fixed`, `none`; 0-6 | A failed flank's price | full | `maneuver.rs` |
| `move_after_blocks` | 3 | Blocks of a Move after a skill. Read from the row but has no schema entry, so no range or form **[V]**: BASERULES1-M adds it | none | `picks.rs`, `maneuver.rs` |
| `intents` | subset of 5 | Rows offered; needs `round_intent` (check) | 4 | `rulesets.rs` |
| `basics` | skill families | Open skills; needed with `deck` (check) | timeline 6 | `deck.rs` |
| `deck_slots` | active 1-8, passive 0-4, keystone 0-2 | Slots | 3, 2, 1 | `deck.rs` |
| `poise_stat` | derived id | Max Poise row; needed with `poise` (check) | `max_poise` | `poise.rs` |
| `stagger_*`, `flank_*`, `break_chance_*`, `rest_*` | numbers in schema range, `tune=` | First three need `poise`; flank needs `flank`; rest pays only with `round_end` | timeline | Sim key tests, not each number |

Built-in fallback: `Ruleset::classic` (CLN-44). Sim and tools: `rules=+x,-y`, `ruleset=`, `melee_diagonal=`, `tune=`, Arena checkboxes.

## 3. Options coming

| Option | Values | From | Requires | Note |
| --- | --- | --- | --- | --- |
| `attack_lock` | `none`, `melee`, `all` | ATTACKLOCK1, in lane 1 now; row field, read whatever the rules | Nothing to run; bands from `reach` | Default none |
| `shape_anchors` | on | K2 | `field_intent` | Direction, RING, ARC, `affects` |
| `field_fill_once` | on | K1 | `field_intent` | |
| `splash_mult`, `assault_shape_blocks`, `field_bonus_values` | numbers | K1, K2 | `shape_anchors` | Skill mult factors |
| `pouch_stacks`, `thrown_miss_spends`, `item_ap` | on; 0 or 1 | K3 | None | Independent of intent |
| `guard_model`, `guard_start` | section 3 of the main spec | BASERULES1-G | `status` needs a guard status row | Conflicts with nothing |
| `damage_model`, `flat_*` | `formula`, `flat` | BASERULES1-F | None | `flat` makes `pierce` and crit triggers inert |
| `heal_model`, `item_value` | `authored`, `value`; `flat`, `value` | BASERULES1-H | None | |
| `armor_form` | `ratio` | Not built: named only to show a future form | | |

## 4. Compatibility

Evidence: **code** names the read, **test** a test on that set, **R** reasoning. Findings: 18 rules, of which only `round_intent` and `flank`, `deck` and
`basics`, `poise` and `poise_stat` have a real dependency; the hooks are independent by design.

| Option | Requires | Conflicts or inert with | Works well with | Evidence |
| --- | --- | --- | --- | --- |
| `round_intent` | The by-rules five: Rest round end, chase every skill, root on, no Switch step, no stances (default) | `stances on`, `switch_target_step on`: runs, but stance switch AP and riposte's stance rule cut against intent-bound buffs and one target | Everything below | code `by_rules`; test plan only; conflict R |
| `flank` | `round_intent` | **Silently inert without it** (`is_flank` starts `if !on(w)`; `round_intent.rs:488`) | `poise` (stagger x1.5), `reach` | code **[V]**; `["poise","flank"]` tested only for reading |
| `flank_protected_*`, `move_after_blocks`, `intents` | `round_intent` | Inert without; `intents` refused by check | | code, check |
| `melee_diagonal maneuver` | Reads as set; `position_by` needs `round_intent` | Inert for positions without it | `flank` | code; R |
| `enemy_moves` | Nothing | Without `ap_cost` still fine | `ap_cost`, `poise` (recovery is not windup) | test `[ENEMY_MOVES, AP_COST]` |
| `deck` | `basics` (error), `deck_slots` (error) | Order lists per intent read only with `round_intent` | `round_intent` | check; test `[DECK]`, `[DECK, POISE]` |
| `poise` | `poise_stat` row in this ruleset (error) | `stagger_*` inert without | `enemy_moves`, `flank`, `triggers` | check; test `[POISE]` |
| `ap_cost` | Skill rows' field | | | test |
| `pierce`, `execute`, `hp_cost`, `ignore_guard` | Skill data | `pierce` inert under `flat` | | own tests |
| `reach` | Family `engine_rules` | Bounds "engaged" for flanks | `flank`, `attack_lock` | test `[REACH]` |
| `plan_badges` | Tells work alone; intents in tells need `round_intent` | Foresight, `plan_reveal_flag` need it | `round_intent` | code; test with intent only; alone untested |
| `triggers` | | `broken` trigger needs `poise` (check, encounter) | `poise` | check |
| `unit_meters` | | Channel needs a meter spec | `deck` (unlock) | test `[UNIT_METERS]` |
| `ground_hazards`, `arena_hazards`, `forced_move` | Data | Collision stagger inert without `poise` | `ai_tactics` | own tests |
| `ai_tactics` | | `flush_cover`'s tile pick needs `round_intent` | Hazards | code |
| Encounter `script` | `round_intent` | Classic ignores it with one log cell | | brief, test `script_round.rs` |
| `attack_lock` | Read as set | Under `chase classic`, untested | `reach`; Guard's value moves **[R]** | lane 1 diff; `attack_lock.rs` |
| `stances on` | | `flat` omits the matchup | | R |

**Not tested together and worth a named test** (M): intent + stances in a fight; `plan_badges` alone; `attack_lock` under `chase classic`. **Known to break:** `flank`
without `round_intent` does nothing and says nothing **[V]**; every other unmet pair above is inert rather than wrong, which is the failure to warn about.

Pairs covered by tests **[V]**: `[]`, `[ROUND_INTENT]`, `[ROUND_INTENT, FLANK]`, `[ROUND_INTENT, AP_COST]`, `[ROUND_INTENT, AP_COST, FLANK]`, `[ROUND_INTENT, DECK]`,
`[ROUND_INTENT, REACH]`, `[ROUND_INTENT, PLAN_BADGES]`, `[ROUND_INTENT, POISE, PIERCE]`, `[ROUND_INTENT, ENEMY_MOVES, AP_COST]`, `[ROUND_INTENT, AP_COST, POISE, DECK]`,
`[ENEMY_MOVES, AP_COST]`, `[DECK]`, `[DECK, POISE]`, `[POISE]`, `[POISE, FLANK]` (reading), `[REACH]`, `[UNIT_METERS]`, `[TRIGGERS]`, `[AI_TACTICS]`, `[GROUND_HAZARDS]`, plus all 18.

## 5. Bundles

Must ship together (a bundle breaks if split) versus works well (a suggestion).

| Bundle | Holds | Kind |
| --- | --- | --- |
| `legacy_core` | No rules; stances on, chase classic, Rest on use, Switch step, diagonal none | Must ship together (the classic row) |
| `intent_core` | `round_intent` + the by-rules five, `intents` | Must ship together; default by `by_rules` |
| `field` | `field_intent`, `shape_anchors`, `field_fill_once`, `splash_mult`, `assault_shape_blocks`, `field_bonus_values` | Must ship together; needs `intent_core` |
| `stand` | `guard_model status` and its status row | Must ship together; needs `intent_core` for the row, not for the term |
| `windup` | `enemy_moves`, `ap_cost`, `poise` + `poise_stat` | Works well |
| `positioning` | `flank`, `reach`, `melee_diagonal maneuver`, `attack_lock melee` | Works well; `flank` needs `intent_core` |
| `build` | `deck`, `basics`, `deck_slots` | Must ship together |
| `tells` | `plan_badges` | Works well with `intent_core` |
| `hooks` | The 9 extensions | Each alone |
| `pouch` | `pouch_stacks`, `item_ap`, `thrown_miss_spends` | Works well; no dependency |
| `feel_flat` | `damage_model flat` on any set | For silo tests; never picked by a shipped world |

This game's set: values file, section 3.

## 6. Dropped ideas as options

Sources: `docs/HISTORY.md`, `LEVERS1-K`, `docs/decisions/archive.md`. One line each; only what is real. **Kept** means reachable today by a row or data.

| Idea | Verdict | Cost and note |
| --- | --- | --- |
| Stances, matchup, switch AP, `ends_in_stance`, `disrupt` | Kept: `stances` | - |
| Rest on use; classic chase; Switch step | Kept: row fields (CLN-45) | - |
| 5 + 3 loadout, class path default | Kept: `deck` off | - |
| AP = skill level | Kept: `ap_cost` off | - |
| Square grid, 10-block round | Kept: board `rows`, `total_blocks` | - |
| Fixed 1.5 scaling; allow-list proficiency | Kept: class data can express both | - |
| Riposte both tier models | Kept: data | - |
| Bodyguard (T4) | Kept: `triggers` intercept | Data |
| Temporary injury | Kept: roster `wounded` | - |
| Ward: absorb pool (LEVERS1-K) | **Candidate kit option** | S after STATUSROWS1; a status row plus one read after the formula; STATUSPASS1 should weigh it, as new reason: Stand's status pass |
| Self-centred area, `area_origin` (LEVERS1-K) | **Mostly superseded** by RING and Stand's anchor (IR5, IR6) | Close the row when K2 lands; a centred BURST from Stand needs no field |
| Taunt guard AI rule, Shield Wall (LEVERS1-K) | **Candidate** | S; an AI row on `taunt` grants; INTENT2-AI's "taunt rules" may already take it: check there |
| Boss acts twice | **Candidate** | M; extra actions in a block; not adopted on a KPI, not on principle |
| Accuracy and evasion rolls, d100 crisis | **Idea only** | M; a new term and roll, so a new option; this game keeps the resolver deterministic |
| Mutual trades | **Idea only** | S; `break_cancels_same_block` as a value; no one needs it |
| Fixed tick anchors for speed tiers | **Idea only** | M; contradicted many skills; reason still holds here |
| Attack lock at planning | **Idea only** | S; a fourth `attack_lock` value; the reason against (breaks chases) holds for this game |
| Unequal class totals | **Becomes free** with BASERULES1-V (`class_budget` 0 = off) | - |
| Free level-up points | **Becomes free** with V (`free_per_level`, growth 0) | - |
| Composure as a stat | **Becomes free** with P | The not-adopted lore reason stays for this game |
| Elemental damage | **Candidate** | M; `damage_type` exists on items; needs a new term and per-type resist, so an option |
| Off-hand attack items | Approved already: OFFHAND1 | Spec needed |
| Heroes choosing rank at planning | **Idea only** | S; enemies still do |
| Mastery on the unit | Stays dropped: drills as passives replaced it | Superseded |
| Enemy tiers per world | Stays dropped: game principle; tier rows are per game | |
| Tier scaling authored `power` | Stays dropped: a bug | |
| Maneuver at half AP and double tiles; `round_stance` names; Manual mode; first RULESET1 draft ids; Area as its own intent; `smart_skills` | Stay dropped: never built, renamed or superseded | |

**Where this lives.** A table "Option candidates" in `engine/docs/authoring/rule_library.md`, kit-agnostic rows only; game reasons stay in `HISTORY.md`. A new drop: when the planner adds a
`HISTORY` row for a mechanic, it adds one line with its verdict and cost, or "stays dropped: reason". The design lane reads it when it specs a system. `HISTORY.md` itself is unchanged here.

## 7. How the engine enforces it

Register metadata, in `ruleset.rs`, beside `REGISTER`:

```rust
pub struct Rule { id, kind, requires: &[&str], conflicts: &[&str], bundle: &str, system: &str, tested: Tested }
pub struct FieldRule { key, needs: &[&str] } // a number or switch and the rule that reads it
```

- **The check** (`check/ruleset.rs` `row_errors`, `warnings`): an unmet `requires` is an error when the rule would be silently inert; a set field whose `needs` is unmet
  is a warning; a `conflicts` pair is a warning, an error only when known to break; an untested pair prints "untested together".
- **Bundles** are metadata plus a list of tested sets; a test runs a short fight on each. `Ruleset::adjust` keeps working for experiments and prints the same warnings.
- **Additive:** new rule off when unlisted, new field defaults to today, metadata refuses only what a row sets. Shipped rows and frozen catalogs are never re-read differently.
- **No row special by id:** `is_classic()` has no reader outside tests **[V]**; it should stay so.
- **Manifest:** `game.json` `ruleset`, `test_rulesets`. Worlds may name only those; a world naming a test row prints a load note. Rows may be many, the picked set is one.

**Game-agnostic version.** `engine/docs/authoring/rule_library.md`, by a builder or the planner as a docs delta (the design lane cannot write under `engine/docs/`): options are
additive and off by default; each has requires, conflicts, bundle and tested; a game picks one set; test on a frozen catalog; value versus form versus new rule (the stop rule);
the candidates table of section 6.
