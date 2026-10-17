# BAL3: data and AI levers for the BAL2-R issues I2-I14

Status: spec (written 2026-10-17 by the BAL3 cloud thread; awaiting the user's approval; nothing here is tuned yet)
Input: `docs/BALANCE_REPORT.md` "BAL2-R clean rerun 2026-10-17" (issues I2-I14; I1 resolved by PLAN1). Its rechecks already include S12S-L commit 1 (registry schema and six status rows, no behaviour).
Decisions this spec builds on (not re-opened): `docs/decisions/balance.md` rows "Solo difficulty", "BAL2 answers" (6) and (7) tier values, "BAL2 role answers" (1)-(7), "BAL2 sim follow-ups", "AI2 diagnosis" options A-E, "BAL3 plan".

## 1. Sequencing (hard rule)

No lever in this spec is applied before both of these are merged on `main`:

1. **The 12x9 combat grid task** (user 2026-10-17: grid 12x9, party on the left, enemies on the right, no centre-tile marker). It changes start distances and lanes, which by itself moves I3 (caster round-1 deaths), I4 (scenario scale) and I5 (tank peel).
2. **S12S-L commits 2-3** (slow, expose, fortify, regen, poison, burn behaviour, round-end order, AI use). Their magnitudes are placeholders "for the balance pass" (`docs/specs/S12S_L.md`), and fortify / regen are new tank and support tools.

Then the order is fixed: **BAL3-R0 baseline rerun -> BAL3-S sim -> BAL3-AI -> BAL3-A data -> BAL3-R final rerun**. Data goes last so its numbers are tuned against the final sim and AI. Every lever below carries a **trigger**: it is applied only if the R0 baseline still shows the issue; levers whose trigger is not met are dropped and logged in the report.

Run cost (cloud, 4 parallel): role L5 grid ~15 min, role L3 ~8 min, sweep L5 ~6 min, party / duel / tier under 1 min each. R0 and R are each about 35 min. L7 is added to the party and role runs because S12S-A families arrive at level 6.

## 2. Tasks (rows the planner adds to `docs/TASKS.md` on approval)

| Id | Lane | Depends on | Scope | Touches |
| --- | --- | --- | --- | --- |
| BAL3-R0 | balance | 12x9 grid task, S12S-L (commits 2-3) | Baseline rerun on that main, same method as BAL2-R plus L7 party/role; marks which triggers below fire | docs/BALANCE_REPORT.md |
| BAL3-S | logic | BAL3-R0 | Sim only: seeded jitter, scenario scale, encounter archetypes (sections 3.1-3.3) | tools/sim/*, bal2_rules_test_suite |
| BAL3-AI | logic | BAL3-S | AI levers (section 4) | scripts/combat/ai/tactical_ai.gd, ai_profile.gd, ai_skills(_tactics)_test_suite |
| BAL3-A | asset | BAL3-AI | Data levers (section 5) through `ContentCatalog.upsert`, seed `data/seeds/bal3.gd` | classes.json, skills.json, items.json, units.json, data/seeds/* |
| BAL3-R | balance | BAL3-A | Final rerun; then the one-time band recalibration (I10) | docs/BALANCE_REPORT.md, docs/decisions/balance.md (proposed rows only) |

## 3. Sim levers (BAL3-S, tools only, no game rule changes)

### 3.1 Seeded variance (report note, prerequisite for I8 and every band)

Cells are 0 or 100 because three seeds give near-identical fights, so no band is measurable.

- Lever: a seeded start jitter in `SimVariants`: each unit's start tile shifted by up to 1 tile inside its own half (never onto another unit or blocked tile), plus a seeded order of the party list. Default on; `jitter=off` restores today's layout for A/B.
- Seeds stay 3 for party/duel/tier; role runs keep the spend x preset grid.
- Acceptance: on the R0 data, at least half of the party cells fall strictly between 0 and 100 at 5 seeds; `PLAN FAIL` and `METRIC FAIL` stay 0.

### 3.2 Role scenario scale (I4; target 50-85% clear for the primary)

Constants in `tools/sim/sim_scenarios.gd` `SCENARIOS`. Starting values; the builder runs the role grid once per change and keeps the first value inside 50-85.

| Scenario | Today | Trigger (R0) | Starting change |
| --- | --- | --- | --- |
| aoe_swarm | 6 minions in a line; every class 0% at L5 | primary clear < 50 | enemies start clumped (2x3 block, 1 tile apart) and the minion ranger is replaced by a second minion brawler (no Aimed Shot one-shot on the caster) |
| striker_priority | L3: nobody downs the priority mage (elite) | primary clear < 50 at L3 | priority target tier elite -> standard at L3 only (L5 already meets its KPIs) |
| ranged_dive | ranger and warrior clear 100 | primary clear > 85 | add a third standard rogue (divers now outnumber the party) |
| tank_hold, support_attrition | unchanged | - | scale left alone; their gaps are AI and data (sections 4, 5) |

### 3.3 Encounter archetypes (I2; party targets mob >= 90, line 70-90, wave 55-80, elite 50-75, boss 30-50)

The party bands are judged on the sim's `GENERATED` encounters, which are also the authoring template for world encounters. Today an encounter of 3 tiered foes faces 4 heroes. Threat per unit = tier HP x tier power: hero 1.00, minion 0.24, standard 0.56, elite 1.375, boss 1.65. Party of 4 = 4.00.

| Archetype | Today (threat, bodies) | Trigger (R0) | Starting composition (threat, bodies) |
| --- | --- | --- | --- |
| mob | 5 minions (1.20, 5) | none (meets target) | unchanged |
| line | 3 standard (1.68, 3) | clear > 90 | warrior, brawler, ranger, rogue standard (2.24, 4) |
| wave | 3 standard + 3 minion wave (2.40, 6) | none (sweep in band) | unchanged |
| elite | elite + 2 standard (2.50, 3) | clear > 75 | elite + rogue, mage, warrior standard (3.06, 4) |
| boss | boss + 2 minion (2.13, 3) | clear > 50 | boss + ranger, priest standard + 2 minion adds arriving in round 2 as a wave (3.25, 5) |

- Measured at equal level (`elevel` 0) with a party of 4 (question Q2).
- Tier values stay as decided (elite 1.25 / 1.1, boss 1.5 / 1.1). The lever is bodies and threat, not multipliers.
- Once accepted, the final compositions become an "encounter budget" table in `docs/GAME_BRIEF.md` for world authors (planner docs delta); shipped world encounters are not retuned in BAL3.

## 4. AI levers (BAL3-AI; engine, with behaviour tests)

Options from the "AI2 diagnosis" row. Each is a rule, not a number, so it is tested in `ai_skills_tactics_test_suite.gd` with a fixed board.

| Lever | Issue | Trigger (R0) | Rule |
| --- | --- | --- | --- |
| B: skip Focus under pressure | I3, I11 | mage or priest deals or heals 0 in any L5 party fight | a unit drafts Focus before a cast only if the estimated incoming damage before that cast resolves is below 50% of its current HP; otherwise the cast goes first |
| C: faster tier when likely to die | I3, I12 | same | if the estimated incoming damage this round is >= current HP, prefer the highest-damage option that resolves by block 2 (lower tier or plain strike) over a slower higher tier |
| Heal reservation | I7, I14 | priest heal 0 in support_attrition | a SUPPORT unit keeps one heal block (lowest heal tier that fits the AP) open in rounds where any ally is projected below `heal_below` by the end of the round, not only at plan time; `heal_below` stays 0.7 |
| D: taunt pull | I5, I13 | allies hp_end < 40 in tank_hold | a taunted foe whose route to the taunter is at most its move for the round treats the taunter as reachable (today only foes already in reach switch); the taunt trigger distance stays 3 |

Deferred (not in BAL3): full per-block re-targeting (I14 first half). It reworks plan drafting, and the heal reservation covers the measured gap. Revisit only if BAL3-R still shows support heal 0.

## 5. Data levers (BAL3-A; starting values, applied only when their trigger fires on the rerun after BAL3-AI)

Changes go through `ContentCatalog.upsert` in `data/seeds/bal3.gd`. One lever per row; the asset builder applies them in order and stops at the first row that brings its metric into band.

| Lever | Issue | Trigger | Starting value |
| --- | --- | --- | --- |
| Caster HP | I3 | mage still takes its full HP before acting in an L5 party fight | mage base CONS 8 -> 10 (from INT 17 -> 15, budget 62 kept); priest base CONS 12 -> 13 (from INT 11 -> 10) |
| Caster armor | I3 | still failing after caster HP | mitigation +3 on the mage and priest default body armor rows |
| Minion skill ceiling | I3, I4 | a minion's tier III hit takes over 50% of a caster's HP | minion units use AP ceiling 2 (tier I-II only) through `AiProfile.skill_cap` for the minion tier |
| Tank tools | I5 | allies hp_end < 40 after lever D | warrior role loadout gets `shield_wall` (taunt + mitigation) in place of `riposte`; fortify (S12S) magnitude on shield_wall set from its placeholder |
| Brawler AoE | I6 | brawler foes/cast < 1.5 in aoe_swarm | `sweep_kick` power_mult 0.7/0.9/1.05 -> 0.8/1.0/1.15 and `cyclone_kick` in the AoE role loadout |
| Mage AoE | I6 | mage foes/cast < 2.0 after 3.2 | `firebomb` reach_max 2 -> 3 (it can open from the back row) |
| Support | I7 | priest heal 0 after the heal reservation | `mend` heal_base 8/10/12 -> 10/12/14; `renewal` regen (S12S) magnitude set from its placeholder |
| Duel: warrior | I9 | warrior duel L5 > 70 | per the "BAL2 role answers" (6) rule: find the GearBudget component (kite shield mitigation) and lower it first; no formula change |
| Duel: priest | I9 | priest duel L5 < 30 and party support KPIs met | smite power_mult +0.1 per tier; nothing else (duels are a diagnostic) |
| Composition swap | sweep +-10 | L3 rogue or brawler outside +-10 | brawler first: `body_blow` power_mult +0.1 per tier; rogue: no change unless still above +10 after the other rows |
| S12S magnitudes | all | any S12S status shows a METRIC FAIL or a band miss traced to it | set inside its registry range, smallest change first |

## 6. Solo tiers (I8)

Solo elite and boss read 0% for every class, partly because the deterministic sim turns any stat edge into 0 or 100. Order: measure with jitter (3.1) on BAL3-R0 data first. If solo elite is still under 30% at equal level, the single follow-up is elite power 1.1 -> 1.0 (HP stays 1.25). That changes the "BAL2 answers" (6) tier value, so it waits for question Q3; boss values stay (solo boss 10-20% per "Solo difficulty").

## 7. Bands (I10)

Role bands stay unchanged through BAL3-R0, BAL3-S and BAL3-AI. BAL3-R proposes the one-time recalibration ("BAL2 role answers" (1)) from the final run; the proposal goes to the user as decision rows, never applied silently.

## 8. Acceptance (BAL3-R)

- Party, default comp and sweep, L3/L5/L7: every archetype inside its band; sweep swap delta within +-10; no comp below 50% on line; every class in >= 2 of the top 5.
- Role L5: each primary ranks first on its first KPI and clears 50-85; L3 within 15 points of that.
- Mage deals damage and priest heals in every L5 party fight.
- Duels inside 30-70 or explained in the report as a diagnostic outlier; solo elite and boss per section 6.
- 0 `PLAN FAIL`, 0 `METRIC FAIL`; `check_project.ps1 -Fast -Quiet` (or `tools/fc.sh gate`) and `tools/fc.sh docs` clean on every BAL3 PR.

## 9. Questions for the user

- **Q1. Approve this plan and its order** (R0 baseline after the 12x9 grid and S12S merge, then sim, AI, data, final rerun; per-block re-targeting deferred). Rec: yes.
- **Q2. Party targets are measured at equal level with a party of 4.** Rec: yes (the boss 30-50% band means an equal-level boss fight).
- **Q3. If solo elite is still under 30% after jitter, lower elite power 1.1 -> 1.0** (HP stays 1.25; boss unchanged). Rec: yes, only if the trigger fires.

## 10. Docs delta (planner applies on approval)

- `docs/TASKS.md`: BAL3 row -> done (spec written); add the five rows of section 2 in `spec` status, then `approved` on the user's word.
- `docs/ROADMAP.md`: none until BAL3-R (S2 / S4 promotion to `balance-ready` is decided there).
- `docs/decisions/balance.md`: the Q1-Q3 answers as one row.
