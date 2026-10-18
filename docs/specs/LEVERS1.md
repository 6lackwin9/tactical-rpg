# LEVERS1: role-feel levers left open after BAL5-AI

Status: approved (user 2026-10-18 14:26 "all recs": Q1 A, Q2 A, Q3 A; spec 2026-10-18, task DR2-LEVERS; input: `docs/BALANCE_REPORT.md` section "BAL5-R1b (BAL5-AI rerun)", `docs/specs/BAL5_LEVERS.md` triggers, `docs/specs/DESIGN_REVIEW2.md` items 4 and 5, both answered B)
Depends on: BAL5-L (shares `skill_fields.gd`, `SkillValidator`, the `StatusEffects` grant path); BAL5-A (shares `skills.json`); Questions for Tushar 1-3 (answered: all A)
Lane: logic, asset, UI, balance (split in section 6)
Touches: see the task rows in section 6

## Goal

DR2 item 4 B: build only the role-feel lever whose gap is still open after BAL5-AI. DR2 item 5 B: build "boss acts twice" only if BAL5's boss trigger fires. This spec reads the BAL5-R1b numbers, decides each lever in or out, and specs the ones that are in. It does not fix a gap twice: where an approved BAL5 lever already targets a KPI, the feel lever stays out until that lever has been measured.

## 1. Verdicts

| Lever | Gap (BAL5-R1b, L5) | Queued BAL5 work on the same KPI | Verdict | Reason |
| --- | --- | --- | --- | --- |
| T4 Bodyguard (warrior) | won-fight allies hp_end 30 (band >= 40); survival 61 passes | T3 `extra_grants` (BAL5-L) + shield_wall fortify 4 / 6 / 8 (BAL5-A), approved and next in the Balance queue | **out (deferred)** | T3 targets this exact KPI and has not been measured; Bodyguard on top would double-fix and blur which lever worked. Re-entry trigger: BAL5-R still shows warrior-row won-fight allies hp_end < 40. |
| S3 Ward (priest) | priest down before round 3 in 83% (band: alive at r3 >= 60%); heal share 2 (interim 5) | S1 already applied (no effect); S2 is a sim scenario edit only, it changes the measurement, not the game | **in** (subject to Q1) | The only game-side lever left on the worst class; priest trios are worst on boss and arms (priest+rogue arms 1 of 10 even with potions). |
| A4 self-centred burst | brawler 0.79 foes/cast (band >= 1.5); mage 1.74 passes | A2 applied (brawler fell from 1.09); A3 is mage-only and does not fire (mage >= 1.0) | **in, for the brawler** (subject to Q2) | BAL5 named it a mage lever, but the mage passes; the brawler is the open miss and has nothing queued. |
| B4 boss acts twice | boss sweep mean 26 (band 30-50) | B3 (boss HP 1.8) fires only above 60 | **out** | The trigger did not fire: the boss is below the band, so a stronger boss goes the wrong way. PACE1's enrage now gives bosses their phase-2 feel. Re-entry only if a later trio sweep shows the L5 boss mean above 60 after B3. |

Note for the planner (no lever here): the boss mean 26 is under its band; BAL5-R should say whether B2 / PACE1 made the generated boss too hard before anyone tunes it.

## 2. Schema contract (logic lane)

Schema-first: yes. Both pieces are additive; existing rows stay valid with the defaults.

### 2.1 Skill field `area_origin` (LEVERS1-L1)

| Field | Type | Allowed | Default | Validation (message) |
| --- | --- | --- | --- | --- |
| `area_origin` | enum | `TARGET`, `SELF` | `TARGET` | `SELF` needs `area` BURST or PLUS ("A self-centred area needs BURST or PLUS."); `SELF` needs `reach_min` 0, `reach_max` 0 and `dash_tiles` 0 ("A self-centred area has no reach or gap-close."); per rank, as other area fields |

Forge help text: "TARGET: the area is aimed at a tile in reach. SELF: the area is centred on you (you are not hit)."

### 2.2 Status `ward` (LEVERS1-L2)

New `StatusRegistry` row (the fifteenth status), added by the registry header checklist:

| Key | Value |
| --- | --- |
| id | `ward` |
| kind | buff (not resistable; `cleanse` never removes it) |
| default rounds | 1 |
| stacking | potency: max wins (a bigger ward replaces a smaller one, rounds refresh); never additive |
| magnitude | `magnitude_min` 1, `magnitude_max` 120, unit `flat` (HP absorbed) |
| timing | granted immediately when the casting skill resolves (like riposte, `StatusEffects.grant_now`), not queued for next round |

No new skill field: a ward skill is a SUPPORT skill with `grant_status` `ward` and a per-rank `status_magnitude`. New validation: a skill with `grant_status` `ward` needs `status_magnitude` >= 1 ("Ward needs a status_magnitude (HP absorbed).") and `heal_base` 0, `heal_mult` 0 ("A ward skill does not heal; use a separate heal skill.").

## 3. Rules (logic lane)

### 3.1 Self-centred area (L1)

1. Cells: `area_cells(origin, origin)` minus the caster's own tile, where origin is the caster's tile when the command resolves (after any earlier move in its sequence). BURST hits the 8 neighbours, PLUS the 4 orthogonal ones.
2. Friendly fire as every other area (decision combat "M2C2B-L X1/X2"): allies on those tiles are hit and roll the status (Q3 may change this).
3. Planner: the step needs no target pick; the preview draws the cells around the caster's planned tile at the cast's start block. Undo as any attack step.
4. AI: candidate origins are the caster's tile at the cast block and, when the draft still has its move before the cast, every tile its move reaches (same approach rules and PLAN1 claims). Pick the origin with the most foes hit; ties: fewer allies hit, then the shorter move. Cast only under the existing area rules: foes >= `area_min_foes` with no ally hit, or the BAL5 A2 tolerance (one ally, >= 3 foes, estimated damage < 25% of its current HP).
5. Tells (INTENT1): an enemy casting it shows the area kind with itself as the anchor; no shipped enemy has such a skill now.
6. Acceptance (tests): a brawler with 3 adjacent foes casts it from its tile and hits all 3, never itself; a brawler 2 tiles from a cluster of 3 moves next to them then casts; with 2 foes and an adjacent ally at 90% HP it does not cast; with 3 foes and that ally it casts (A2); validation rejects `SELF` with LINE, SWEEP, a reach above 0 or a dash.

### 3.2 Ward (L2)

1. Absorb: after `DamageCalculator.compute` (formula unchanged, decision combat "Combat extensions (S12)"), each hit on a warded unit first reduces the ward pool; the rest goes to HP. Applies to attack, area and riposte-counter hits. Round-end damage (bleed, poison, burn) is not absorbed. At pool 0 the status ends. The pool lives in the unit's status magnitude and is snapshotted for undo like other statuses.
2. Timing: active from the end of the block in which the ward skill resolves; hits in that same block are not absorbed. Lasts its `status_rounds` (the cast round counts, as riposte).
3. Tells stay true: a ward never changes who an enemy targets or what the tell says (DR2 section 2). Enemy target ranking ignores wards (same answer as cover, decision combat "S12T-L rule 8c").
4. AI cast rule (SUPPORT units, counts toward `MAX_HEALS` 2 per round): ward the friend (caster included) with the highest projected `incoming_damage` this round when that damage is >= 40% of the friend's current HP and the friend has no ward; a ward outranks a heal while the friend is above `heal_below`. S1 self-care extension: when the caster's own missing HP is smaller than its ward amount it wards itself instead of healing. `incoming_damage` subtracts a target's current ward pool.
5. Metrics: `BattleMetrics` records `ward_absorbed` per hit, credited to the ward's source (`status_sources`). The sim support KPI becomes support share = (heal_done + ward_absorbed) / party max HP (same band, >= 20, interim 5). The battle report adds ward absorbed into the caster's Healed column (one support number for the player; BREPORT1 column rules unchanged).
6. Sim loadout: `tools/sim/sim_roles.gd` priest support loadout gains `ward` in place of its last family.
7. Acceptance (tests): a 40-point ward takes a 30 hit (pool 10, HP unchanged), then a 25 hit (ward ends, HP -15); bleed at round end is not absorbed; a hit in the cast block is not absorbed; a bigger ward replaces a smaller one, a smaller one does not; undo restores the pool; an AI priest facing a projected 60%-of-HP hit on the mage wards the mage; a priest at full HP projected to lose 50% wards itself; `ward_absorbed` is credited to the caster.

## 4. Assets (asset lane)

Through `ContentCatalog.upsert`, seed `data/seeds/levers1.gd`, after BAL5-A has landed its `skills.json` rows.

### 4.1 Cyclone Kick becomes the spin (A1, needs L1 and Q2 A)

| Field | Now | New |
| --- | --- | --- |
| `area` | SWEEP | BURST |
| `area_origin` | (TARGET) | SELF |
| `power_mult` base / rank L4 / rank L5 | 0.9 / 1.1 / 1.3 | 0.7 / 0.85 / 1.0 |
| `status_potency` stagger base / L4 / L5 | 50 / 55 / 60 | 40 / 45 / 50 |
| level, blocks, grade, `requires`, path level 5 | unchanged | unchanged |
| description | | "Spin and kick everyone next to you, friends included; may stagger them." |

Reason for the cut: 8 tiles instead of 3; per-foe power stays above Firebomb (0.6 / 0.75 / 0.9) because the brawler takes melee risk. `sweep_kick` (the aimed L) is unchanged.

### 4.2 Ward (A2, needs L2 and Q1 A or B)

| Field | Value |
| --- | --- |
| id / name | `ward` / Ward |
| kind | SUPPORT |
| level / blocks | 2 / 1 (fast, so it lands before most hits) |
| `target_mode` | ALLY_OR_SELF (Q1 B: SELF) |
| reach | 0 (support default 0-3) |
| `heal_base` / `heal_mult` | 0 / 0.0 |
| `grant_status` | ward |
| `status_magnitude` base / rank L3 / rank L4 | 30 / 40 / 50 |
| `status_rounds` base / L3 / L4 | 1 / 1 / 2 |
| `learned_only` | true |
| description | "{name}: shields an ally (or you) from the next {magnitude} damage until the end of the round." (rank L4: "for {rounds} rounds") |
| priest `skill_path` | `ward: 3`, placed after `mend` |

Reference: an L5 enemy strike is about 49 (DR2 section 1), so rank III soaks one strike. Side effect: the priest's default loadout at L5 becomes smite, rally, mend, ward, purify (hex leaves the default five; the player can swap it back).

## 5. Order, interaction and acceptance (balance lane, LEVERS1-B)

Order: BAL5-L, BAL5-A and BAL5-R first (T3 and the data levers measured alone); the BAL5 S2 scenario edit lands before the Ward is measured, so the support cells are read on the S2 scenario both before and after. Then L1 / L2, A1 / A2, and one -B run that records the BAL5-R numbers as "before".

| Cell (L5 and L7, trio method, BAL5 seeds) | Pass |
| --- | --- |
| brawler AoE foes/cast | >= 1.5 and brawler rank 1 on foes/cast |
| AoE allies/cast brawler | <= 0.3 |
| swarm clear brawler | >= 70 |
| mage foes/cast | within 0.2 of its BAL5-R value (no regression) |
| priest alive at round 3 (S2 scenario) | >= 60% |
| support share (heal + ward absorbed) | >= 5 interim (band 20) |
| support priest clear | 30-85 |
| boss sweep mean | within 5 points of BAL5-R |
| HEAL1 arms finale, no potions | rises by at most 10 points over BAL5-R |
| PLAN FAIL / METRIC FAIL | 0 / 0 |

One data step each if missed, then report and stop (DR1 (3)): priest alive at r3 under 60 -> ward magnitudes +10 per rank; arms finale up more than 10 -> ward top rank 50 -> 40; brawler foes/cast under 1.5 is an AI or geometry finding, reported with casts per fight, no data step.

## 6. Task rows (for the planner)

| Id | Title | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| LEVERS1-L1 | Self-centred area (`area_origin`): field, validation, cells, planner step, AI origin pick, tests | logic | BAL5-L | approved | docs/specs/LEVERS1.md 2.1, 3.1 | scripts/content/skill_fields.gd, scripts/content/validators/skill_validator.gd, scripts/combat/planning/skill_def.gd, combat_plan.gd, scripts/combat/ai/tactical_ai.gd, ai_skills_tactics_test_suite, skill tests |
| LEVERS1-L2 | Ward status: registry row, absorb after the formula, AI cast rule, `ward_absorbed` metric, sim priest loadout, tests | logic | LEVERS1-L1 (shares tactical_ai.gd), BAL5-L | approved | docs/specs/LEVERS1.md 2.2, 3.2 | scripts/combat/status/*, damage application in scripts/combat/actions/*, scripts/content/validators/skill_validator.gd, tactical_ai.gd, scripts/combat/session/battle_metrics.gd, battle_report.gd, tools/sim/sim_roles.gd, status and AI test suites |
| LEVERS1-A1 | Cyclone Kick becomes the spin | asset | LEVERS1-L1, BAL5-A | approved | docs/specs/LEVERS1.md 4.1 | data/catalog/skills.json, data/seeds/levers1.gd |
| LEVERS1-A2 | Ward skill and priest path | asset | LEVERS1-L2, BAL5-A | approved | docs/specs/LEVERS1.md 4.2 | data/catalog/skills.json, data/catalog/classes.json, data/seeds/levers1.gd |
| LEVERS1-U | Ward on the unit card and HP bar (shield segment), forecast line "Ward absorbs N", log line; spin preview around the caster | UI | LEVERS1-L2, LEVERS1-L1 | approved | docs/specs/LEVERS1.md 3.1 (3), 3.2 | scripts/ui/combat/*, scripts/ui/roster_panel_ui.gd |
| LEVERS1-B | Measure both levers against section 5 | balance | LEVERS1-A1, LEVERS1-A2, BAL5-R, BAL5-S2 (done, b9fcc80) | approved | docs/specs/LEVERS1.md 5 | docs/BALANCE_REPORT.md |

ROADMAP row (proposed): "LEVERS1 role-feel levers: brawler spin, priest Ward (DR2 4B)", status `proposed`.

HISTORY rows (deferred, not dropped): T4 Bodyguard deferred 2026-10-18, reason: T3 extra_grants targets the same KPI and is not yet measured; B4 boss acts twice not built 2026-10-18, reason: BAL5 boss trigger did not fire (L5 mean 26, trigger above 60) and PACE1 enrage covers the phase-2 feel.

## Out of scope

Bodyguard (T4) and boss acts twice (B4), until their re-entry triggers; heal power (tried, BAL3); mage kit changes (A3 does not fire); `sweep_kick`; new area shapes (the spin reuses BURST and PLUS); enemy units with ward or the spin; the boss band miss (BAL5-R).

## Open questions

Questions for Tushar (plain terms; answered 2026-10-18 14:26 "all recs": 1 A, 2 A, 3 A):

1. **Priest Ward.** An earlier rule says the priest gets no protective buffs for allies (Aegis, an armor buff at level 6, already bends it). The data says priests still go down in the first two rounds in most fights. A: allow Ward, a shield she puts on herself or one ally that soaks the next hit. B: Ward only on herself. C: no Ward; move Aegis earlier to level 3 instead (data only, weaker). **Recommend A**: it is the one move you would plan around the enemy tells.
2. **Who spins.** The design review named a mage burst around the caster, but the mage already hits enough enemies; the brawler does not. A: the brawler's Cyclone Kick becomes a spin that hits everyone next to him. B: keep Cyclone Kick and add the spin as a new brawler move. C: give it to the mage as first named. **Recommend A**: no extra skill to learn, and the name already says spin.
3. **Does the spin hit friends?** A: yes, like every other area attack, so where you stand matters. B: no, enemies only. **Recommend A**: it keeps one rule for all area attacks.
