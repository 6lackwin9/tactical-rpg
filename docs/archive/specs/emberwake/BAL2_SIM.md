# BAL2-SIM: balance sim contract (scores, variants, metrics, party, role limit tests, sweep)

Status: spec (design-brainstorm 2026-10-17; part of task BAL2-L commits 1a and 1b; targets set by DECISIONS "BAL2 role answers (user 2026-10-17)"; awaiting approval)
Depends on: nothing (tools only); its numbers are meaningful after BAL2-L commit 2 (range fix)
Lane: logic (tools only)
Touches: `tools/balance_sim.gd`, new `tools/sim/sim_args.gd`, `tools/sim/sim_metrics.gd`, `tools/sim/sim_scenarios.gd`, `tools/sim/sim_roles.gd` (helpers via `preload`, no `class_name`), `tools/fc.sh`

User direction (2026-10-17): duels are rare in RPGs; balance is benchmarked mostly by party play and by ROLE. Commit 1a keeps and repairs the duel tooling (diagnostic); commit 1b adds the metrics collector shared by party mode, role limit tests and the composition sweep.

## 1. Command line (commit 1a)

`tools/fc.sh sim <mode> [level] [seeds] [key=value ...]`. Positional `mode`, `level` (default 5), `seeds` (default 3; seed i = 11 + 12 i) stay backward compatible; key=value tokens may follow in any order. An unknown key or value prints the usage and exits without fighting.

| Key | Values | Default | Meaning |
| --- | --- | --- | --- |
| `cap` | 1..30 | 10 | Round cap (encounter `max_rounds` and the loop) |
| `smart_skills` (removed by CLN-16, 2026-10-17: now an unknown key) | `on`, `off` | `on` | `off` set `AiProfile.smart_skills = false` on every profile in `session.encounter.ai_profiles` after `session.start()`; prints `smart_skills off: <n> profiles` |
| `loadout` | `default`, `newest`, `role` | `default` | Party/left-side units only. `newest`: section 3.1; `role`: the class's primary role loadout (section 6.3) |
| `spend` | `class`, `cons`, `primary`, `mixed`, `all` | `class` | Section 3.2; `all` runs class, cons and primary and averages |
| `preset` | any `PlanPresets.IDS`, or `both` | `balanced` | Preset for every unit; `both` averages balanced and aggressive |
| `distance` | `default`, 2..6 | `default` | Section 3.3 |
| `score` | `both`, `hp`, `binary` | `both` | Which score columns print |
| `quick` | flag | off | 1 seed, variants `all`/`both` collapse to their first value |
| `metrics` | `on`, `off` | `off` in duel/tier/levels/group, `on` in party/role/sweep | Print the per-unit KPI block (section 4) |

`tools/fc.sh`: the sim timeout becomes `${SIM_TIMEOUT:-900}`; the usage block lists modes and keys. Every table starts with one header line that echoes the effective arguments.

## 2. Scores

For a fight, `l` and `r` = remaining HP / max HP summed over each side (inactive units count 0 HP; wave units count from the start).

- Binary (today's rule): `r == 0 and l > 0` -> 1; `l == 0 and r > 0` -> 0; else 1 if `l > r + 0.02`, 0 if `r > l + 0.02`, else 0.5.
- HP margin ("hp"): `r == 0 and l > 0` -> 1; `l == 0 and r > 0` -> 0; both 0 -> 0.5; else `clamp(0.5 + (l - r) / 2, 0, 1)`.
- Cells print `bin/hp` in percent (e.g. `100/78`); row averages print both. The duel sanity band 30-70 (section 8.3) is judged on the hp score.
- Duel, tier, levels, group: every matchup runs every seed with sides swapped (as today). Party, role and sweep fights are asymmetric: seeds only, no swap.

## 3. Variants and diagnostic modes (commit 1a)

### 3.1 `loadout=newest`

From the built unit's owned class-path families (unreduced path levels): families listed at path level 1 first (path order); then the rest by path level descending, ties in path order; area families (base row `area` not empty) move after every non-area family; take the first `StatRules.MAX_CLASS_SLOTS`. The sim writes the list as the sandbox unit's `loadout` field via `ContentCatalog.upsert` (validation applies). Check on main c3520b4 data, ranger L5: `[barbed_arrow, deadeye, focus, aimed_shot, pinning_shot]`; after BAL2-A (aimed_shot at L1): `[aimed_shot, barbed_arrow, deadeye, focus, pinning_shot]`.

### 3.2 `spend`

`pool = StatRules.spend_pool(level)`. `class` = `{}` (class weights, today). `cons` = `{"CONS": pool}`. `primary` = `{S: pool}` with S the stat of the largest class growth (ties: first key in the class `growth` dict). `mixed` = `{S: floor(pool / 2), "CONS": pool - floor(pool / 2)}` (merged if S is CONS). If an upsert refuses a variant, the sim prints the error and skips that variant.

### 3.3 `distance`

After `session.start()` and before the first plan, each right-side unit (enemies) is moved with `CombatGrid.remove_combatant` / `place_combatant` to the walkable, free, non-exit tile at grid distance N from the nearest left-side unit that is closest to its current tile (ties: lowest y, then x). A unit with no such tile stays; the header line reports `distance N: <k> units not moved`. If the grid API cannot do this without engine changes, print `distance unsupported` and report it (no engine change).

### 3.4 Modes

| Mode | Fights | Output |
| --- | --- | --- |
| `duel [level]` | class round robin, hero vs hero | matrix `bin/hp`, row averages |
| `tier [level]` | hero vs its class at minion/standard/elite/boss, 1v1 and 1v3 | `bin/hp` per cell |
| `levels` | class at L vs L+2 for L = 1, 3, 5, 7 | `bin/hp` per cell |
| `group [level]` | per class: hero L vs 3 minions of its class at max(1, L-2); vs 3 standard at max(1, L-2); vs 3 minions at L (expected loss, DECISIONS "Solo difficulty") | `bin/hp` per cell |
| `stats [level or all]` | no fights; levels 1, 3, 5, 7 when `all` | per class: HP, armor rating, reduction %, EHP = round(HP / (1 - rating / (rating + 50))), melee power `power_of(c, false)`, ranged power `power_of(c, true)`, attack range, speed, crit %, active class families |

`stats` check on main c3520b4: warrior L5 HP 370, armor 31, EHP 597; priest L7 range 2 (1 after BAL2-L commit 2).

## 4. Metrics collector (commit 1b, `tools/sim/sim_metrics.gd`)

One record per unit per fight, filled by reading `session.log_entries` after every `session.step_next()` (new entries since the last index) and by sampling every unit after each step (HP, grid tile, statuses). Units are matched by display name (`ActionLogEntry.actor_name` / `target_name`, unique per encounter). If a field cannot be derived, it prints `n/a` and the builder lists it.

| Field | Definition |
| --- | --- |
| `dmg_dealt` | Sum of `damage` of this unit's ATTACK entries and its riposte reactions on foes |
| `dmg_dealt_ranged` | Part of `dmg_dealt` where the attacker was not adjacent (grid distance > 1) to the target at the hit |
| `dmg_taken` | HP lost to foe hits (overkill excluded) |
| `dot_taken` | HP lost at round end (bleed; since S12S-L also poison and burn) |
| `dot_dealt` | Added by S12S-L: round-end damage from statuses this unit applied, credited to the status source |
| `regen_received` | Added by S12S-L: HP restored to this unit by regen at round end (also counted in `heal_received`; the caster's `heal_done`) |
| `attacks_received` | Foe ATTACK entries that targeted this unit (area hits count once per unit hit) |
| `heal_done` / `overheal` | HP restored to allies or self, capped at their missing HP / the capped-off rest |
| `heal_received` | HP restored to this unit |
| `cleanses` | Statuses removed by this unit's skills |
| `buffs_applied`, `buff_rounds` | Successful buff grants (haste, taunt, riposte, ...) and the sum of their rounds |
| `debuffs_applied`, `debuff_rounds` | Successful debuff applications on foes (stagger, stun, root, bleed, weaken) and the sum of their rounds |
| `area_casts`, `area_foes_hit`, `area_allies_hit` | Area attack casts by this unit and units hit per side |
| `kills` | Foes whose HP reached 0 from this unit's hit |
| `blocks_alive`, `survived`, `hp_end_frac`, `min_hp_frac` | Blocks while active, alive at the end, HP / max at the end, lowest HP fraction |
| `near_death_saves` | Allies of this unit that fell below 25% HP, then received this unit's heal and ended the fight alive |
| `nearest_foe_dist` | Mean grid distance to the nearest active foe, sampled once per block while alive |
| `blocks_adjacent` | Blocks with at least one foe adjacent |
| `hold_blocks` | Blocks within 1 tile of its starting tile |
| `taunt_blocks` | Blocks this unit carried the `taunt` status |
| `priority_dmg` | Damage dealt to the scenario's priority target (section 6.4; 0 elsewhere) |

Fight record: `cleared` (every foe, waves included, inactive before the cap), `rounds`, `party_hp_frac`, `enemy_hp_frac`, `hp_score`, `survivors`, `priority_down_round` (cap + 1 if never).

Conservation checks (assert in every party/role fight; print `METRIC FAIL <unit> <field>` on a miss): per unit `hp_start - dmg_taken - dot_taken + heal_received == hp_end` (HP clamped at 0..max); sum of party `dmg_dealt` == sum of enemy `dmg_taken`, and the reverse.

## 5. Party mode (commit 1b)

`party [level] comp=<c1,c2,...> enc=<list> elevel=<+n|-n|catalog>`.

- `comp`: 1 to `PartyState.PARTY_MAX` (4) class ids, duplicates allowed; more prints `party size > PARTY_MAX (4)` and exits. Default: `warrior,ranger,priest,mage`. Sandbox units `simp_<class>_<i>`, side party, hero, `flee_ratio` 0, with the `loadout`/`spend` arguments.
- `enc`: generated ids below, catalog encounter ids (`goblin_skirmish`, `veterans_drill`, `goblin_ambush`: enemies and waves from the catalog, party replaced), `generated` (all five) or `all`. Default `generated`.
- `elevel`: generated enemies are at party level + n (default +0); catalog encounters keep their authored levels unless `elevel` is given explicitly.

Generated encounters (sandbox units `sime_<class>_<tier>_<i>`, default class gear and loadout, zone `stone_arena`, CLOSED arena, `total_blocks` 6, `unit_ap` 12, `ap_regen` 4):

| Id | Enemies | Wave (round 2, block 1) |
| --- | --- | --- |
| `mob` | minion brawler, rogue, ranger, priest, brawler | - |
| `line` | standard warrior, brawler, ranger | - |
| `wave` | standard brawler, rogue, ranger | minion brawler, rogue, priest |
| `elite` | elite warrior, standard rogue, standard mage | - |
| `boss` | boss brawler, minion ranger, minion priest | - |

Arguments `elite=<class>` and `boss=<class>` replace the elite/boss class. Output per (comp, encounter): `clear %`, mean `rounds` of cleared fights, mean `survivors`/party size, mean `party_hp_frac`, mean `hp_score`, then the per-unit KPI block if `metrics=on`.

## 6. Roles and limit tests (commit 1b, `role` mode)

### 6.1 Roles and KPIs

| Role | Job | KPIs (section 4 fields) |
| --- | --- | --- |
| Tank | Holds the line, draws and absorbs foe attacks | `dmg_taken` share of the party; `attacks_received` share; `taunt_blocks / blocks_alive`; reference allies' mean `hp_end_frac`; `hold_blocks / blocks_alive`; survival |
| Ranged attacker | Keeps distance and pokes | mean `nearest_foe_dist`; `dmg_dealt_ranged / dmg_dealt`; `attacks_received` per round; `dmg_dealt` per round; survival |
| AoE attacker | Handles swarms | `area_foes_hit / area_casts`; `area_allies_hit / area_casts`; rounds to clear the swarm; `dmg_dealt` share of the party |
| Support | Keeps party HP and buffs up; assist value | `heal_done` (effective) / party max HP; `near_death_saves`; `cleanses`; `buff_rounds`; `debuff_rounds` (assist); party survivors |
| Melee striker | Reaches and removes a priority target | `priority_down_round`; `priority_dmg / dmg_dealt`; `dmg_dealt` per round; survival |

The striker role is added because the roster has two melee damage classes (rogue, brawler) that none of the four requested roles measures.

### 6.2 Class to role map (archetype, DECISIONS roster rows; rogue striker and brawler melee AoE confirmed by the user 2026-10-17)

| Class | Primary | Secondary |
| --- | --- | --- |
| warrior | Tank | Support (assist: intimidate weaken, taunt) |
| ranger | Ranged attacker | AoE attacker (chip: volley) |
| priest | Support | Melee striker (battle priest: smite, judgement) |
| brawler | AoE attacker (melee: sweeps, user 2026-10-17 "engaging several foes") | Tank, Melee striker |
| rogue | Melee striker | none |
| mage | AoE attacker | Ranged attacker |

### 6.3 Role loadouts (sim constants in `tools/sim/sim_roles.gd`; BAL2-A writes the primary L5 list on the shipped party units)

At L5 (L7 deltas in brackets). At L1 and L3 a list is cut to the owned families and filled by the default rule up to 5. A class without a loadout for a role uses its primary loadout (that measures it off-role).

| Class | Role | Families |
| --- | --- | --- |
| warrior | Tank | taunt_shout, shield_bash, riposte, intimidate, shield_wall |
| warrior | Support | taunt_shout, intimidate, shield_bash, sunder, riposte |
| ranger | Ranged | aimed_shot, pinning_shot, leap_back, barbed_arrow, deadeye |
| ranger | AoE | aimed_shot, pinning_shot, leap_back, volley, deadeye |
| priest | Support | rally, mend, purify, greater_mend, renewal |
| priest | Striker | smite, rally, hex, mend, greater_mend [L7: judgement replaces mend] |
| brawler | AoE (also used for Tank) | body_blow, lunge, sweep_kick, cyclone_kick, haymaker |
| brawler | Striker | body_blow, lunge, charge_in, hamstring, haymaker |
| rogue | Striker | quick_stab, pounce, lacerate, backflip, hamstring [L7: assassinate replaces hamstring] |
| mage | AoE | arcane_bolt, arc_lance, firebomb, glacial_burst, focus [L7: inferno replaces focus] |
| mage | Ranged | arcane_bolt, arc_lance, hex, focus, firebomb |

Every advanced family's `requires` (deadeye: aimed_shot, greater_mend: mend, cyclone_kick: sweep_kick, charge_in: lunge, lacerate: bleeding_cut, judgement: smite) is on the same class path, so the lists validate.

### 6.4 Limit-test scenarios

`role [level] role=<tank|ranged|aoe|support|striker|all> class=<id|all>`. The test unit takes slot 1 with the role's loadout; reference allies are hero units at the same level with default loadouts. Enemies at the same level.

| Scenario | Party | Enemies | Stress |
| --- | --- | --- | --- |
| `tank_hold` | test + ranger + mage (fragile references) | standard brawler, rogue, brawler, rogue | focused melee pack diving the back line |
| `ranged_dive` | test + warrior | standard rogue, rogue, brawler | divers that bypass the tank |
| `aoe_swarm` | test + warrior (BAL5-SIM A1: + priest, a trio) | minion brawler, rogue, ranger, brawler, rogue, priest | six weak foes |
| `support_attrition` | test + warrior + brawler | standard ranger, ranger, mage, rogue (BAL5 S2, trigger fired in BAL5-R1b: ranger, brawler, mage, rogue) | sustained ranged damage on the party |
| `striker_priority` | test + warrior | elite mage (priority target) + standard warrior, warrior (screen) | reach and remove a protected target |

Each class runs every scenario under the variant grid spend {class, cons, primary} x preset {balanced, aggressive}, 3 seeds each (`quick`: one variant, 1 seed). Output per (class, scenario): each KPI as `mean [floor..ceiling]` over the variants (the class's potential range in that role) and the scenario's clear % and survivors.

## 7. Composition sweep (commit 1b, secondary check)

`sweep [level] enc=<list> dupes=<off|on> sample=<n>`: all 15 distinct 4-class compositions (default), or with `dupes=on` the 126 multisets (`sample=n` draws n of them with RNG seed 7). Each composition runs `party` against `enc` (default `generated`). Output: compositions ranked by mean clear % (ties: mean `party_hp_frac`); per class the mean swap delta = mean over every composition C that holds class Y but not X of `clear(C - Y + X) - clear(C)`; and per class the count of appearances in the top 5. Levels run as separate processes (sandbox per process id), e.g. one `sweep 1`, `sweep 3`, `sweep 5`, `sweep 7` each.

## 8. Targets (user 2026-10-17, DECISIONS "BAL2 role answers"; the BAL2 balance run reports against these)

### 8.1 Role KPI targets: absolute starting bands plus ranking

- Ranking: in its limit test a primary class ranks in the top 2 of six on the role's first KPI (the first one in each row below); a secondary class ranks above every off-role class on it.
- Bands: the primary class's `[floor..ceiling]` range must overlap the band. The bands are starting values: the balance lane recalibrates them ONCE after the first BAL2 run (proposal to the user in `games/emberwake/docs/BALANCE_REPORT.md`), then they are fixed.

| Role | Band (primary class, equal level) |
| --- | --- |
| Tank | `dmg_taken` share 40-65%; `attacks_received` share >= 45%; survives >= 60%; reference allies `hp_end_frac` >= 40% |
| Ranged | `nearest_foe_dist` >= 2.5; ranged damage share >= 70%; `attacks_received` <= 1.0 per round |
| AoE | foes per area cast >= 2.0 in `aoe_swarm`; ally hits <= 0.3 per cast; swarm cleared in >= 70% of fights |
| Support | effective heal >= 20% of party max HP per fight; `near_death_saves` >= 0.3 per fight; survivors >= +0.5 versus the same scenario with a reference brawler in the support slot |
| Striker | priority target down by round 5 in >= 60% of fights; `priority_dmg` share >= 50% |
| Every scenario | the primary class clears 50-85% (a limit test should be hard but winnable) |

### 8.2 Party encounter and sweep targets

- Clear rate at equal level, mean over the 15 distinct 4-class compositions: `mob` >= 90%, `line` 70-90%, `wave` 55-80%, `elite` 50-75%, `boss` 30-50% (party play encouraged; the solo boss 10-20% of DECISIONS "Solo difficulty" stays in `tier`).
- Sweep limits: every class's swap delta within +-10 points; no composition below 50% clear on `line`; every class appears in >= 2 of the top 5 compositions.

### 8.3 Duel: diagnostic with a sanity band

Duels are a diagnostic, no longer the gate (replaces the 40-60 bracket of decision 11): every class's duel row average (hp score) stays within 30-70% at L1/3/5/7, so no class is hopeless or unbeatable 1v1 (solo exploration still has 1v1 moments). Party and role targets are the gate for `balance-ready`.

## 9. Acceptance checks (rule-derived)

1. `tools/fc.sh sim duel 5` without keys prints a header with cap 10, smart_skills on (dropped from the header by CLN-16), loadout default, spend class, preset balanced, distance default, 3 seeds, and `bin/hp` cells.
2. Regression: on the same commit, `duel 5 cap=6 score=binary` equals the old sim's table (run the old script first; report both).
3. Determinism: the same command twice gives identical output (every mode).
4. HP score: a KO win prints 100, a KO loss 0; a timeout with `l` 0.8 and `r` 0.4 scores 70.
5. `smart_skills=off` prints `smart_skills off: <n> profiles` with n = units in the fight (key removed by CLN-16).
6. `loadout=newest` and `spend=*` produce the lists and stats of sections 3.1 and 3.2 (`stats` shows them).
7. Conservation checks of section 4 hold in every fight of `party 5` and `role 5 quick` (no `METRIC FAIL` line).
8. `party 5 comp=warrior,ranger,priest,mage,rogue` prints the PARTY_MAX error and runs no fight.
9. `role 5 role=aoe class=mage` prints `area_casts` > 0 and `area_foes_hit / area_casts` >= 1.0 (an area cast hits at least its aimed foe).
10. `sweep 5 quick` ranks 15 compositions and prints a swap delta for all six classes.
11. Every report table marks a value outside its section 8 target with `X` (role bands and ranking, party clear bands, sweep limits, duel 30-70), so the balance lane reads pass/fail directly.

## 10. Out of scope

Engine or AI changes for metrics; new encounter data in `data/catalog` (generated encounters live in the sim); tuning numbers (BAL2-A and the BAL2 balance run); UI.

## 11. Open questions

None open. Answered 2026-10-17 (DECISIONS "BAL2 role answers"): role bands plus ranking (8.1), party clear bands and sweep limits (8.2), duel as a 30-70 diagnostic (8.3), rogue = melee striker, brawler primary = melee AoE (6.2).
