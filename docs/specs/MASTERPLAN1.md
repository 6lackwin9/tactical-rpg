# MASTERPLAN1: the master plan of blocks, sub-blocks and their connections

Status: active (2026-11-06; section 5 questions closed 2026-11-07). Lane: design. Touches: this file and [MASTERPLAN1_ROWS.md](MASTERPLAN1_ROWS.md) (every open task row mapped to the plan task that owns it). Build queue: `docs/READY.md`.

Decisions: "Master plan first; a task list that stops growing", "Link a change to its task; a block is a behaviour library", "Bottom up from building blocks", "System workflow, refined", "Behaviours fixed, stats as data; bugs after lock". Starting point: `docs/specs/SILOS1.md`; base rules: `BASERULES1.md` and companions; intents: `INTENT2.md`.

Marks: libraries, counts and special cases are **verified** in code constants and shipped data by grep and a read-only script (kit code at `67d1045e`, which differs from `origin/develop` only in docs). Stages, join sizes not in SILOS1, merges and the order are **reasoning**.

## 1. Blocks and sub-blocks

Stage (decision "System workflow, refined"): 1 design, 2 build or reuse, 3 iterate as data, 4 finalize, 5 limit test, 6 locked. **B** = built code exists to reuse. No sub-block has a finalized ruleset, so none is past 3. Layer: all kit logic with game data, unless named.

| Id | Sub-block: owns | Behaviour library today | Special-cased in code (level 3 cannot cover) | Stage |
| --- | --- | --- | --- | --- |
| **Unit** | | | | |
| U1 | Stats and derived: what a unit's numbers are | 6 stats (`STATS`); 8 derived numbers (max HP, speed, move rate, armour rating, crit, resist, max poise, composure), 2 as data rows (DERIVE1 formula), 6 in code | Each stat read by name at about 30 sites; PERC only adds reach at 18+, KNOW only heals and unlocks; no rule reads composure | 1, B |
| U2 | Class and loadout: a unit's template and what it fields | Base stats 6-18 summing to 62, growth 5 (max 3 a stat), proficiency, `skill_path`, default gear and pouch, AI role; 5 tiers scale HP and power; deck (slots, basics, order lists); 7 classes, 48 units | Kit falls back to class `warrior` (`unit.rs:924`, `lib.rs:98`); the sim fixes 6 class ids | 1, B |
| U3 | Gear and carry: what a unit wears | 6 slots, 3 hands; an item is family + modifiers (`adjust`, `stat_bonus`, `requirement`, `effect`) + tiers; proficiency 0-3; wield penalty; carry and burden; budget 23 ± 20 % | Families named in code: `fist`, `bow`/`crossbow`/`sling` (always ranged), `shield`, `tome`, `staff`, `wand`; `block_cost` never read | 1, B |
| U4 | Passives and sources: permanent modifiers and who grants them | 13 effect types over 55 passives (resist 21, trigger 13, stat 9, proficiency 8, skill_potency 6, stagger_taken 5, foresight 4, unlock 3, forced_move 3, ap_regen 2, meter 2, skill 1, status_potency 1); sources: character, faction `grants`, discipline, named item; drills. **Affiliation**: faction id; `grants` empty in all 9 shipped factions; rep bands read by dialogue and shop; `relations` checked, never read. **Background**: text only | None found | 1, B |
| **Action** (a skill or a consumable) | | | | |
| A1 | Value and cost: what it is worth and costs | 9 kinds (5 authored); `power_mult`, heal base, mult and stat; `ap_cost` (39 of 129 skills, else level), blocks, recovery, `hp_cost`, meter costs; gates `weapon_types`, `min_proficiency`; items `use_blocks`, `use_reach`, `max_stack` | Consumables by `consumable_type` (3 kinds); item damage and heal skip the hit; `buff_stat` unread, so `buff_pct` always boosts attack; stance fields inert with stances off | 1, B |
| A2 | Resolution: value + attacker + defender to a number | One formula `damage::compute`: attack power, skill mult, crit x1.5, guard + cover capped 0.9, armour r/(r + 50), pierce, ignore guard, riposte 0.5, tier power; heal amount; `damage_model flat` designed | Constants in code (`ARMOR_K`, `GUARD_CAP`, `CRIT_MULT`, `COUNTER_MULT`); guard only while a Defend runs | 1, B |
| A3 | Shape and reach: which cells, how far | PLUS, BURST, LINE, SWEEP; `reach_min`, `reach_max`, `target_mode`, `position`; RING, ARC, CONE, `affects` designed (INTENT2-K2) | SWEEP never fires (INTENT2 section 9) | 2, B |
| A4 | Effects and reactions: what it does besides damage | 21 skill sections (inflict, grant, cleanse, guard, focus, jump, dash, exploit, forced move, hazard, riders, triggers...); triggers: 15 events, 10 conditions, 14 effects; exploits flank, behind, `hp_below` | None by id, but a status is applied in 5 vocabularies: skill fields, trigger `inflict` and `grant`, hazard rows, arena hazard effects, phase `status` | 1, B |
| **Status** | | | | |
| S1 | Status class: timed conditions | 17 rows. Properties: buff, resistable, rounds, 4 stacking modes (refresh, extend, stack potency, none), cap, magnitude range and unit, grant kind, intent-bound, lands next round or now. Behaviours: tick damage (bleed 6 % max HP, poison flat, burn % max HP), tick heal (regen), skill mult (weaken, empower), armour (fortify, expose), move rate (haste, slow), blocked blocks (stagger 1 and 4, stun 1 and 2), no moves (root), AI pull (taunt), counter (riposte), hidden (stealth), unbreakable (steadfast). Chance = potency - INT x 2.5 - resist | **17 of 17 coded by id**, in a code table and per-id reads in 7 files (`status.rs`, `unit.rs`, `resolver.rs`, `damage.rs`, `poise.rs`, `ai.rs`, `round_intent.rs`): no status picks from a set, so a new one needs code | 1, B |
| S2 | Pools: values that fill, drain and fire at a band | Poise: max from a derived row, stagger spent (wind-up x2, guard x0.5, repeat x0.5, flank x1.5), break at 0, refill each round. Unit meters (2 rows): gauge or countdown, bands with 6 effect kinds, decay, skill costs | Poise is one fixed rule (`poise.rs`) while meters are rows; two meters on one status is a known limit | 1, B |
| **Fight flow** (planner's "Round and timing", plus encounters) | | | | |
| F1 | Round and blocks: when things land | 6 blocks, simultaneous resolution, initiative priority x 1000 + speed, wind-up, impact, recovery, attack lock (none, melee, all) | Priority by command kind (Defend 1, Target 2) | 2, B |
| F2 | AP and rest: what a unit can afford | Pool 12, start 10, regen 4; debt allowed, Stagger at 3, Stun at 5; Rest 0.5 a block, cap 2; `enemy_moves` | Debt statuses named in code | 2, B |
| F3 | Intent and planning: what a round aims at | 5 intents (Channel hidden), follow-ups, fill, ghost, chase (classic, every skill), Switch step, order lists; Field and Stand anchors designed (INTENT2) | None found | 2, B |
| F4 | Encounter rules: a fight's setup, script and end | Board, units, objectives, phases with 9 effect kinds (line, wave, status, ai, enrage, deck, shake, kill, talk), round scripts, stay modes, timeout, flee; 11 catalog encounters | None found | 2, B |
| **Space** | | | | |
| P1 | Board, terrain and hazards: the cells | 17 terrain rows (`cover_guard`, `move_cost`, hazard status); 8 code zones plus data zones; 6 hazard rows (the same fields plus duration, trap, hidden, affects, stop moves); arena hazards | Walkable fixed per terrain index (`grid.rs:68`) | 1, B |
| P2 | Movement and position | Move rate from speed (cap 4), Move blocks 2 and 3, path, dash, jump, forced move (push, pull, collision), flank and behind, reach band, `melee_diagonal`, 8-way facing. No line of sight exists | None found | 2, B |
| **AI** | | | | |
| I1 | Decisions and tactics: how a unit picks | 8 rows (flee, support, guard, brace, engage, assault, advance, idle guard); `heal_below`, `defend_below`, `flee_ratio`, `reckless`; 5 target prefs; 3 tactics; `ai_role` | Reads 8 statuses by id (taunt, riposte, regen, root, stun, slow, expose, fortify) | 1, B |
| **World** | | | | |
| W1 | Region map and interaction | 7 marker kinds, touch or proximity, flag gates, 5 services, leader, camera | None found | 3, B (game screen) |
| W2 | World map and travel | Nodes, legs, travel modes, scripted road fights, routes | None found | 2, B |
| W3 | Campaign clock: time and pressure | Gauge and countdown meters, bands, rules with 9 effects, checkpoints, triggers on 5 causes, rest, defeat (wake, retreat), upkeep | None found | 2, B |
| **Economy and progression** | | | | |
| E1 | Levels and XP | XP curve (data), levels 1-10, spend pool 4 + level - 1, growth, skill path unlocks | None found | 2, B |
| E2 | Purse, shop, stash, rewards | Purse, stock, prices, rep, 6 buy types, stash, loot rows, rewards (xp, items, passives, skills, gold) | `SUPPLIES_MAX` 99 | 2, B |
| E3 | Roster | Members, wounds (2 hurt and 4 loss modes), 3 housing modes, departures (5 fates, 7 causes), keepsakes, recruit levels | None found | 2, B |
| **Story** | | | | |
| T1 | Dialogue, quests and flags | 5 condition kinds, 9 effects; flags with 6 operators; quests with steps, variants, rewards, links; mission board | Effects interpreted by direct call (SILOS1 T7: leave) | 2, B |
| T2 | Sequences and vignettes | Cards (4 styles), sequences (5 step kinds), story fights (retry, abort, continue), 5 shake presets | None found | 2, B (engine player) |

**Shells** (serve the blocks; no stages): X1 screens and UI kit; X2 board and map rendering (stage, grounds, walls, props, tokens); X3 art pipeline; X4 FX and audio; X5 content store, checks and the rule register (`REGISTER`, 18 rules, `aegis rules`); X6 saves; X7 sim and tools (sim, Arena, Dojo, Gallery); X8 agent link, scenarios and integration.

**What a level-3 test cannot cover today**: every status (S1), the 3 consumable types (A1), stats by name (U1), the named gear families (U3), walkability (P1), the class fallback (U2), the AI's status reads (I1). Each needs a level-4 test per entry until its sub-block moves it into the library.

## 2. Connections

Size from SILOS1 where it has one, else **R**. All built unless marked **designed**.

| Id | From -> to | What crosses | Size | Pipeline |
| --- | --- | --- | --- | --- |
| C01 | U1 -> U3 | Requirements, scaling stat, burden to speed, carry from STR and CONS | easy R | `gear.rs`, `carry.rs` |
| C02 | U1 -> E1 | Growth a level, spend pool, unlock thresholds (KNOW 16, 20) | easy R | `stats.rs` unit build |
| C03 | U2 -> U3 | Proficiency level and scaling, default kit, `can_wear`, drills | easy R | `Proficiency::with_drills` |
| C04 | U4 -> U1, S1, A4 | Passive effects from 4 sources, once each | medium R | `character_passives` |
| C05 | U1-U4 -> battle unit | Class, spend, growth, gear, passives into one unit | easy | `build_unit_in`, `Prebuilt` |
| C06 | U1 -> A2 | Attack power, armour rating, crit, HP, resist | medium R | `compute`, `heal_amount`, `apply_chance`; `--contract` golden **designed** |
| C07 | A1 -> A2 | Skill value; heal and item value | easy R | **designed**: `heal_model`, `item_value` (BASERULES1-H) |
| C08 | A4 -> S1 | Status id, potency, rounds, magnitude | medium R | `try_inflict`, `grant_now`, from the 5 vocabularies |
| C09 | S1 -> A2, P2 | Skill mult, armour, move rate; no re-derive mid-fight | easy R | `unit.rs` per-id reads |
| C10 | S1 <-> F1, F2 | Blocked blocks, AP debt to Stagger and Stun, roll-over | medium R | `roll_over`, `debt_status` |
| C11 | A1 -> F1, F2 | Blocks, AP cost, recovery, wind-up | easy R | `plan.rs` to `resolver.rs` |
| C12 | A3 -> P2 | Anchor by intent, direction, band, `affects` | medium R | **designed**: INTENT2-K2 |
| C13 | P2 -> A2 | Cover on ranged hits, flank exploit and power | easy R | `command.rs`, `round_intent.rs` |
| C14 | P1 -> S1 | Terrain and hazard statuses on enter or round end | easy R | `ground.rs` |
| C15 | A1 -> S2 -> S1 | Stagger spends poise; a break is the Staggered status | medium R | `poise.rs` |
| C16 | fight -> I1 -> F3 | Units, statuses, intents; the AI plans through the same commands | medium R | `ai.rs`, `Host` |
| C17 | F4 -> F1, I1, T1 | Units, board, objectives, phase effects, battle talk | medium R | `encounter.rs` |
| C18 | E3, W -> battle | Party, ruleset, grants, wounds, upkeep AP, zone, world stage for mob growth (`U2`) | hard | **designed**: WORLDFIGHT1 `begin_world` (passes world stage to `C05`) |
| C19 | battle -> E3, E2 | Tally, party first, then rewards | medium | `settle_party`; **designed**: `finish_world` |
| C20 | W3 <-> battle | Fight and days asks, `battle_mods`, retreat, run-over | hard | `campaign`, asked from `region`, `worldmap` |
| C21 | W1, W2, battle -> W3 | A cause ticks the clock | easy | `tick(Cause)` |
| C22 | W1 -> T1 -> E2 | Interact, service reply, open shop or board | easy | `explore`, `dialogue`; screen seam **designed**: SERVICES1 |
| C23 | W2 <-> W1 | Current node, exit targets, enter with anchor, rest | medium | `travel`, `explore::enter` |
| C24 | T2 -> battle | Story fight source, retry or abort | medium | `FIGHT_SOURCE` |
| C25 | T1 -> W1, W2, E2, T2 | Flags gate markers, routes, stock, cards; rep bands; `stat` conditions read U1 | medium R | the flag store |
| C26 | E1 -> U1, U2 | XP, level, unlocks; rest reads travel and purse | easy | `progression.rs` |
| C27 | all -> X6 | `GameState` blocks, three save gates | easy, not trivial | `aegis_save`, `save.rs` |
| C28 | battle -> X4 | Step events, screen id | easy | `fx.rs`, `music.rs` |

**Pipeline plan.** Unit seams C01-C05 close with the Unit block. `U1.2` owns standing derived stats (Pools + Ratings) and the 4-stage modifier schema; `A2.2` evaluates the 4-stage pipeline dynamically per hit/heal and triggers stat re-derivation on pool band crossings or status changes. Static units (`U2`) replace `BASERULES1-F` (`feel_flat`) as the flat-value combat model. World seams C18-C26 come after. C27 and C28 follow their ends. C08 is unified at `.1` design (one status/effect application schema across all vocabularies).

## 3. The fixed task list

- **Each sub-block, six tasks**: `<id>.1` design, `.2` build or reuse, `.3` iterate as data, `.4` finalize, `.5` limit test, `.6` locked. **Each connection, one pipeline task** `Cnn`, built after both ends reach `.5`. **Each shell, one standing task** `Xn`; its rows are sub-tasks.
- **Total: 25 x 6 + 28 + 8 = 186 tasks**, fixed. Open rows map onto them: [MASTERPLAN1_ROWS.md](MASTERPLAN1_ROWS.md).
- **Testing ladder per stage**: level 1 (a behaviour is there) at `.2`; level 2 (one entry shows all its behaviours) at `.3`; level 3 (a rule on any entry in the silo, such as any status against the resist stat) at `.5`; level 4 (a specific entry) only for the special cases in section 1.

What is not a new task:

1. A **bug** (design and code disagree) is fixed inside the task that owns the behaviour, also after lock.
2. A **tweak** is an iteration of `.3`, or data tuning after `.6`.
3. A **new entry** that picks known behaviours (a status from the status class) is data under `.3`.
4. A **new behaviour or idea** goes to the parking list (the parked rows and the library's candidates, `BASERULES1_LIBRARY.md` section 6). It enters only after data, a sim or a mock shows it improves gameplay, and then reopens its sub-block's `.1`: no new task.
5. **Every change names the task that owns it** (`S1.3`, `C18`, `X1`); "none" flags a plan gap to the planner.

## 4. Order

By dependency, bottom up: **U1** (everything reads it), then **U3, U2, U4**; **S1, S2** (read resist and poise from U1); **A1-A4** (effects pick from S1; resolution reads U1); **F1-F4, P1, P2, I1** (combat may be designed beside A, on the flat model); then **E1-E3, W1-W3, T1, T2**, built and at stage 2 or 3, mostly a design pass that names each ruleset. Shells serve whoever is active; final screens stay last.

| Session | Sub-blocks | The user decides |
| --- | --- | --- |
| 1 | U1 | What each stat signifies and feeds (PERC and KNOW, BASERULES1 Q1); the derived roles and their off states; the formula grammar frozen (Q2); stats as data now or beside K1 (Q5); a 3-stat pack as the proof |
| 2 | U2, U3, U4 | Class shape (budget 62, growth 5, tiers); gear behaviours kept or dropped (`block_cost`, wield penalty, budget); the passive effect list; affiliation and background (Q2 below); OFFHAND1 and temperaments in or parked |
| 3 | S1, S2 | The status class: behaviours and properties, each of the 17 as a pick, so STATUSROWS1 takes that shape; one resist stat or several; poise as a pool beside meters (POISE1); the Stand guard as a status (feeds A2) |

## 5. Answered questions (closed 2026-11-07)

1. **The block list, five changes to the planner's.** **Answered: accept all five** (9 blocks, 25 sub-blocks): (a) Consumables in Action (`A1`/`A3`); (b) Fight flow (`F1-F4`) takes encounter rules; (c) Space parks line of sight and merges hazards into terrain (`P1`); (d) AI is one sub-block (`I1`); (e) Dialogue and quests are one (`T1`, SILOS1 Q1).
2. **Affiliation and background.** **Answered: accept recommendation.** Both live in `U4`, keep faction `grants` and rep bands, background stays text-only; temperament (`ROSTER1-T`) enters `U4.1`.
3. **Poise.** **Answered (session U1, 2026-11-06):** Poise is a unified Pool in `U1` (max and regen derived in `U1`), its dynamic meter/damage runs in combat (`S2`/`A2`), and its break at 0 is a status (`S1`).
4. **Shells.** **Answered: accept recommendation.** One standing task each (`X1-X8`), no stages; readability sub-tasks tag the sub-block they unblock, and final screens wait for the blocks.
5. **Builds during sessions 1-3.** **Answered: accept recommendation.** Only behaviour-neutral builds listed in `docs/READY.md` (`TESTCLEAN1`, `SKILLS1`, `KITIDS1`, `BASERULES1-M`) run while Block `U` design finishes; `WORLDFIGHT1` (`C18`) holds until its ends near lock.
6. **Approved but unbuilt rows** (`KEEP1`, `DEEDS1`, `RESPEC1`, `OFFHAND1`, `ROSTER1-T`). **Answered: accept recommendation.** Each enters its sub-block's `.1` session as a candidate behaviour that must show its gameplay case; "spec, later" rows stay parked.
