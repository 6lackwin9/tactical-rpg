# Balance report (written by the balance agent; read by the brainstorm agent)

## BAL3-R0 baseline 2026-10-17 (after 12x9 grid + S12S-L)

Scope: BAL3-R0 (docs/specs/BAL3.md). Checkout: cloud thread, branch claude/balance-kwjk9v at develop 7df8b86 (no sim-relevant change vs main 3942238: 12x9 combat grid, party left / enemies right, and S12S-L statuses). Method as in BAL2-R (archived): `tools/fc.sh sim party|sweep|role|duel|tier`, cap 10, smart_skills on, 3 seeds, spend class, preset balanced (role: spend class/cons/primary x preset balanced/aggressive), no code or data edits. Levels 3 and 5, plus 7 for party and role. About 3,300 fights: **0 `PLAN FAIL` and 0 `METRIC FAIL` in every run**. X = outside the decided target. Raw output: scratchpad `r0/` of the run (not committed). Tooling note: a fresh checkout needs `godot --headless --import` once before `tools/fc.sh sim`; with a stale class cache (new `AssetRegistry` class) the sim hangs silently and prints nothing (cost 40 min here).

### Headline (BAL2-R before -> R0 after)

1. **The grid did not fix party difficulty (I2)**: `line`, `elite`, `boss` still clear 100% for the default party at L3, L5 and L7 (boss 100 up to elevel +2, and with `elite=mage boss=warrior`). Sweep means: L3 wave 76 -> 89 X, elite 78 -> 100 X; L5 line 100 -> 98 X, wave 69 -> 73, elite and boss 100. Default L5 `wave` moved 0 -> 67 (survivors 1.0/4) and L5 loadout=role `elite` is now 67; elevel +1/+2 `wave` 33 / 0 (unchanged).
2. **Caster deaths improved but remain (I3)**: the L5 default mage now deals damage in every fight (before: 0 in `mob` and `wave`; now 115-304) but still takes its full 184 HP in `line` (and in `wave` at elevel +2); the priest still heals 0 in `mob` and `wave` at L5 (L7: 0 in 4 of 5 fights). At L3 the mage still deals 0 in `mob` and `wave`.
3. **Role scenarios moved**: striker L5 rogue clear 11 -> 67 and down-by-round-5 72 (rank 2 behind the ranger, 100); aoe mage foes/cast 1.00 -> 1.78 (L7: 3.75, clear still 0); tank allies hp_end 1 -> 17, taunt/alive 37 -> 50, atk_rcv share 36 -> 38. Still failing: aoe clear 0 for every class at L3/L5/L7, L3 striker 0 for every class, ranged_dive ranger 100 at L5 (72 at L3 and L7), support heal 0 at every level.
4. **Sweep swap deltas now all inside +-10** (L3 rogue +14.7 -> +2.7, brawler -15.3 -> -3.3; L5 max +7.3). Every class is in >= 2 of the top 5; no comp under 50% on `line`.
5. **Duels and tiers are bit-identical to BAL2-R** (no stat or AI change): warrior 90 / 93, priest 3 / 10, mage 20 at L3; solo elite and boss 0 for every class (priest keeps an hp score 36).
6. Variance: only 5 of 45 party cells (9 tables x 5 encounters) are strictly between 0 and 100 at 3 seeds, so the seed-jitter acceptance (>= half) is not met.

### 1. Party mode, default comp warrior,ranger,priest,mage (3 fights per encounter, clear %)

| Encounter | Target | L3 default | L3 loadout=role | L5 default | L5 loadout=role | L5 elevel +1 | L5 elevel +2 | L7 default | L7 loadout=role |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mob | >= 90 | 100 (surv 2.0/4) | 100 (3.3 rds) | 100 (3.0/4) | 100 (4.0/4) | 100 | 100 | 100 (2.0/4) | 100 |
| line | 70-90 | 100 X | 100 X | 100 X | 100 X | 100 X | 100 X | 100 X | 100 X |
| wave | 55-80 | 100 X (3.0/4) | 100 X | 67 (1.0/4, hp 0.07) | 100 X | 33 X (0.7/4) | 0 X | 100 X (1.7/4) | 67 (0.7/4) |
| elite | 50-75 | 100 X | 100 X | 100 X (4.0/4, hp 0.79) | 67 (1.0/4) | 100 X | 100 X | 100 X | 100 X |
| boss | 30-50 | 100 X (2.0 rds) | 100 X | 100 X (3.3/4, hp 0.64) | 100 X | 100 X | 100 X | 100 X (3.0/4) | 100 X |

`elite=mage boss=warrior` at L5: elite 100 (2.0 rounds, 3.0/4 survive), boss 100 (3.3 rounds, 3.7/4, hp 0.78). Per-unit (mean per fight), L5 default: the mage deals 136 (`mob`), 115 (`line`), 301 (`wave`), 304 (`elite`), 224 (`boss`); the priest heals 0 (`mob`), 210 (`line`), 0 (`wave`), 315 (`elite`), 23 (`boss`). L3 default: mage deals 0 in `mob` and `wave` (takes 160 = full HP), priest heals 37-182. L7 default: priest heals 0 in `mob`, `line`, `wave`, `boss` (212 in `elite`); mage deals 131-410. Cells are 0 or 100 in most columns.

### 2. Composition sweep (15 distinct 4-class comps, 3 fights per cell, mean clear % over comps)

| Level | mob (>= 90) | line (70-90) | wave (55-80) | elite (50-75) | boss (30-50) |
| --- | --- | --- | --- | --- | --- |
| L3 | 100 | 100 X | 89 X | 100 X | 100 X |
| L5 | 100 | 98 X | 73 | 100 X | 100 X |

| Class | L3 swap delta / top-5 count | L5 swap delta / top-5 count |
| --- | --- | --- |
| warrior | +6.7 / 4 | +3.3 / 4 |
| ranger | -3.3 / 3 | +3.3 / 4 |
| priest | +0.7 / 3 | -6.7 / 3 |
| brawler | -3.3 / 3 | -6.7 / 2 |
| rogue | +2.7 / 5 | +7.3 / 5 |
| mage | -3.3 / 2 | -0.7 / 2 |

- L5: 8 comps at 100 (top warrior,ranger,priest,rogue hp 0.79); bottom ranger,priest,brawler,mage 73 (`line` 67, `wave` 0), warrior,priest,brawler,rogue 80 (`wave` 0). The spread is `wave` (and one `line` loss).
- L3: 13 comps at 100; ranger,brawler,rogue,mage 87 and ranger,priest,brawler,mage 80 (`wave` 33 / 0). Both lose with a ranger and no warrior.

### 3. Role limit tests, L5 (spend x preset grid, 3 seeds = 18 fights per cell; mean [floor..ceiling])

| Role (scenario) | Primary result | Bands | Other classes |
| --- | --- | --- | --- |
| Tank (tank_hold) | warrior clear 67 [33..100], rank 1 | dmg_taken share 44 ok; atk_rcv share 38 [33..44] X (band 45); taunt/alive 50; allies hp_end 17 [0..33] X (band 40); survival 39 [33..67] X (band 60) | brawler S clear 44, rank 6 X; rogue 78; ranger and mage 0 |
| Ranged (ranged_dive) | ranger clear 100 X, rank 1 | nearest_foe 1.86 [1.43..2.32] X (band 2.5); ranged dmg 100 ok; atk_rcv/rnd 2.35 X (band <= 1.0) | warrior 89, rogue 67, brawler 33, mage S 11 |
| AoE (aoe_swarm) | mage clear 0 X, rank 1; brawler clear 0 X, rank 3 X | foes/cast mage 1.78 [1.33..2.00] X (band 2.0), brawler 0.10 X; allies/cast 0 ok; swarm clear 0 X | every class clears 0 (rogue 22); ranger 0.88 foes/cast |
| Support (support_attrition) | priest clear 44 [0..100] X, rank 3 X | heal 0 X, saves 0 X, survivors vs brawler -0.06 X (baseline 0.83) | warrior 100, ranger 72, rogue 56, brawler 50, mage 17 |
| Striker (striker_priority) | rogue clear 67 [0..100] ok, rank 2 | down by r5 72 ok, prio dmg 65 ok, prio down round 3.94 | ranger 100 (rank 1, round 1.0), brawler S 44; warrior, priest, mage 0 |

### 4. Role limit tests, L3 and L7 (same grid and bands)

| Role / class | L3 clear (50-85) | L3 First KPI and other bands | L7 clear | L7 First KPI and other bands |
| --- | --- | --- | --- | --- |
| Tank / warrior | 44 [0..67] X | dmg_taken share 43 ok, rank 4 X; atk_rcv 50 ok; taunt 19; allies hp_end 3 X; survival 44 X | 44 [33..67] X | share 37 X, rank 4 X; atk_rcv 35 X; taunt 5; allies hp_end 1 X |
| Ranged / ranger | 72 [0..100] ok | nearest_foe 1.55 X, rank 2; ranged dmg 80; atk_rcv 1.93 X | 72 [0..100] ok | nearest_foe 2.06 [1.40..2.97] X, rank 1; ranged dmg 100; atk_rcv 2.60 X |
| AoE / mage | 0 X | foes/cast 0.00 X; swarm clear 0 | 0 X | foes/cast 3.75 [2.50..4.00] ok, rank 1; swarm clear 0 X |
| AoE / brawler | 0 X | foes/cast 0.00 X | 0 X | foes/cast 0.17 X, rank 2 |
| Support / priest | 28 [0..67] X | heal 0 X, saves 0 X; survivors vs brawler 0.00 | 50 [0..100] ok | heal 0 X, saves 0 X; surv vs brawler -0.44 X; cleanse 0.33 |
| Striker / rogue | 0 X | prio down round 11; nobody downs it | 83 [33..100] ok | down by r5 89, prio dmg 61 ok, rank 3 X (brawler 100 clear rank 1, ranger 44) |

### 5. Diagnostics

| Class | Duel L5 (bin/hp, band 30-70) | Duel L3 | Tier L5 elite 1v1 (bin/hp) | Tier L5 boss 1v1 | Tier L3 elite / boss 1v1 |
| --- | --- | --- | --- | --- | --- |
| warrior | 90/89 X | 93/88 X | 0/0 | 0/0 | 0/0 / 0/0 |
| ranger | 53/53 | 67/64 | 0/0 | 0/0 | 0/0 / 0/0 |
| priest | 3/4 X | 10/17 X | 0/36 | 0/33 | 17/44 / 0/37 |
| brawler | 57/57 | 50/51 | 0/0 | 0/0 | 0/0 / 0/0 |
| rogue | 60/60 | 60/60 | 0/0 | 0/0 | 0/0 / 0/0 |
| mage | 37/37 | 20/20 X | 0/0 | 0/0 | 0/0 / 0/0 |

Duel L5 and L3 are identical to BAL2-R. Tier: minion and standard 1v1 100 for every class (priest standard hp 82 / 72); 1v3 0 except priest vs 3 minions 67/64 (L5) and 50/38 (L3); rogue vs minions is now 0 (was 50/50). Rogue elite hp 15 -> 0 at L5.

### BAL3 triggers (spec sections 3.2, 3.3, 4, 5, 6)

Measured on R0 unless the trigger depends on an earlier lever ("conditional"; the number is then the current value and the lever is decided by the build order).

| Section | Lever | Trigger | Measured | Fired |
| --- | --- | --- | --- | --- |
| 3.2 | aoe_swarm scale | primary clear < 50 | mage 0, brawler 0 at L3, L5, L7 | fired |
| 3.2 | striker_priority L3 (elite -> standard) | primary clear < 50 at L3 | rogue 0 (L5 67, L7 83) | fired |
| 3.2 | ranged_dive (third rogue) | primary clear > 85 | ranger 100 at L5 (L3 72, L7 72) | fired (L5 only) |
| 3.3 | line archetype | clear > 90 | party 100 (L3/L5/L7); sweep 100 / 98 | fired |
| 3.3 | elite archetype | clear > 75 | party 100; sweep 100 / 100 | fired |
| 3.3 | boss archetype | clear > 50 | party 100 everywhere; sweep 100 | fired |
| 3.3 | mob, wave (no trigger) | none | mob 100; wave sweep 89 (L3) / 73 (L5) | n/a (L3 wave sweep 89 is above band 80; noted) |
| 4 | B skip Focus under pressure | mage or priest deals/heals 0 in any L5 party fight | priest heals 0 in `mob` and `wave` (L5 default; also elevel +1/+2); mage dealt > 0 in all L5 default fights (L3: 0 in `mob`, `wave`) | fired |
| 4 | C faster tier when likely to die | same | same | fired |
| 4 | Heal reservation | priest heal 0 in support_attrition | heal 0 at L3, L5, L7 | fired |
| 4 | D taunt pull | allies hp_end < 40 in tank_hold | 17 (L5), 3 (L3), 1 (L7) | fired |
| 5 | Caster HP | mage takes its full HP before acting in an L5 party fight | mage takes 184 (full HP) in `line` but deals 115 first; dealt > 0 in all L5 fights | not fired (borderline: full HP lost in `line`, and in `wave` at elevel +2) |
| 5 | Caster armor | still failing after caster HP | depends on the row above | conditional, not fired |
| 5 | Minion skill ceiling | a minion tier III hit takes > 50% of a caster HP | not measurable from the sim output (needs a probe) | not measured |
| 5 | Tank tools | allies hp_end < 40 after lever D | 17 now (before D) | conditional, would fire |
| 5 | Brawler AoE | brawler foes/cast < 1.5 in aoe_swarm | 0.10 (L5), 0.00 (L3), 0.17 (L7) | fired |
| 5 | Mage AoE | mage foes/cast < 2.0 after 3.2 | 1.78 [1.33..2.00] (L5), 0.00 (L3), 3.75 (L7) | fired at L3/L5 (conditional on 3.2) |
| 5 | Support | priest heal 0 after the heal reservation | heal 0 (before the reservation) | conditional, would fire |
| 5 | Duel: warrior | duel L5 > 70 | 90 | fired |
| 5 | Duel: priest | duel L5 < 30 and party support KPIs met | 3; party support KPIs not met (priest heals 0 in L5 `mob`, `wave`) | not fired (second condition fails) |
| 5 | Composition swap | L3 rogue or brawler outside +-10 | rogue +2.7, brawler -3.3 | not fired |
| 5 | S12S magnitudes | any S12S status METRIC FAIL or band miss traced to it | 0 METRIC FAIL in all runs; no band miss traced (dot column 0 in party fights: no burn/poison is exercised yet) | not fired |
| 6 | Solo elite power 1.1 -> 1.0 | solo elite < 30% at equal level (after jitter) | 0 for every class (L3 priest 17); jitter not yet built | fired (waits for 3.1 jitter and Q3) |
| 3.1 | Seed jitter | >= half of party cells strictly in 0-100 at 5 seeds | 5 of 45 party cells strictly between (3 seeds) | fired |

### 6. Targets that fail (summary)

| Target | Result | Gap |
| --- | --- | --- |
| Party line 70-90 | 100 at L3, L5, L7 (sweep 100 / 98) | +10 |
| Party elite 50-75 | default 100 (L5 loadout=role 67); sweep 100 | +25 |
| Party boss 30-50 | 100 everywhere, up to elevel +2 | +50 |
| Party wave 55-80 | default L5 67 ok, L3 100 X, L7 100 X; sweep L3 89 X | comp-dependent |
| Sweep swap delta +-10 | all inside | resolved |
| Tank bands | allies hp_end 17 / 3 / 1 vs 40; atk_rcv share 38 / 50 / 35 vs 45 | -23 |
| Ranged bands | atk_rcv/rnd 2.35 vs <= 1.0; clear 100 (L5) vs <= 85; nearest_foe 1.86 vs 2.5 | +1.35 / +15 / -0.6 |
| AoE bands | foes/cast mage 1.78 (L5) vs 2.0; swarm clear 0 vs 70 | -0.2 / -70 |
| Support bands | heal 0, saves 0 at every level | all |
| Striker bands | L5 ok (67), L7 ok (83), L3 0 | L3 all |
| Duel 30-70 | warrior 90 / 93, priest 3 / 10, mage 20 (L3) | +20 / -27 / -10 |
| Solo elite ~50, boss 10-20 | 0 for every class | -50 / -10 |

### Issues for the brainstorm agent (R0 update)

- I2 stays open: line/elite/boss need the 3.3 archetypes (all three triggers fired); the grid alone changed nothing there.
- I3 improved: the L5 mage always acts, but L3 mage still deals 0 in `mob`/`wave`; the L5 priest heals 0 in `mob`/`wave` (and 4 of 5 fights at L7). AI levers B and C fired.
- I4: aoe_swarm and L3 striker fired; ranged_dive clear 100 at L5 only. New at L7: striker rank 3 (brawler clears 100, rogue 83).
- I5: tank improved (taunt 50, hp_end 17) but still under 40; lever D fired. At L7 tank is worse (taunt 5, hp_end 1, clear 44), a grid or level-scaling effect worth checking in BAL3-AI.
- I6/I7: mage foes/cast is 1.78 (L5) and 3.75 (L7) already; brawler 0.1-0.2; support heal 0 everywhere (reservation lever fired).
- I8/I9: duels and solo tiers unchanged; solo elite lever waits for jitter.
- I10/variance: 5 of 45 party cells are strictly between 0 and 100; jitter (3.1) is needed before any band recalibration.
- Observation: the `dot` column is 0 in every party fight, so burn/poison (S12S) are not exercised by the default loadouts; the "S12S magnitudes" trigger cannot fire until a skill applies them.

Cloud run times (4 cores, up to 4 runs in parallel): role L5 about 35 min, role L3 19 min, role L7 40 min, sweep L3 6 min, sweep L5 11 min, party/duel/tier 1-2 min each; about 1 h 50 min wall including the 40 min lost to the stale-cache hang.

The BAL2-R clean rerun (input to BAL3) is archived, text unchanged, in `docs/archive/BALANCE_REPORT_2026-10.md`.
