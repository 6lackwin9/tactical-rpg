# Balance report (written by the balance agent; read by the brainstorm agent)

## BAL4-ATTR campaign attrition 2026-10-18 (rerun v3)

Superseded: the first run (older Ashlands data, before RECRUIT1-A / OBJ1-A / BOSS1-A) had headline means 2.04 / 3.54 / 2.70 / 2.94 battles (heal none / full / 25 / 50); its tables are replaced by this section.

Method: `tools/fc.sh sim campaign 5 10 heal=<mode>` on branch claude/bal4-attr at 1959db8 (develop merged). Data state: after RECRUIT1-A (Bryn the named warrior with his perk), OBJ1-A (objectives), BOSS1-A (boss phases, ambush layout); OBJ1-L objective wins count. Story order, arms steel: B1 `courtyard_skirmish`, B2 `pass_ambush`, B3 `wagon_raid`, B4 `ashmaw_stand`; gate steel ends with `ashmaw_stand_barred`. Party level 5, the warrior plus 2 recruits, all 10 trios, 10 seeds each, foes at authored levels, each fight at its encounter's `max_rounds`. HP carries over, statuses reset, downed heroes stay down (only `heal=full` revives); a run ends at the first uncleared battle. Heal between battles: none; full (today's rule); 25 / 50 (standing heroes heal that share of max HP). 0 PLAN FAIL and 0 METRIC FAIL in all four runs (780 / 800 / 792 / 796 fights). Four runs in parallel with SIM_TIMEOUT=3300 took 1,955 s wall. Level 4 and 6 not run.

Mean battles survived out of 4, cells arms / gate (full chain cleared of 10 seeds, arms / gate):

| Trio | none | full | 25 | 50 |
| --- | --- | --- | --- | --- |
| ranger, priest | 3.6 / 3.9 (6 / 9) | 3.6 / 4.0 (6 / 10) | 3.4 / 3.9 (4 / 9) | 3.5 / 3.9 (5 / 9) |
| ranger, brawler | 3.1 / 3.4 (1 / 4) | 3.7 / 4.0 (7 / 10) | 3.6 / 3.9 (6 / 9) | 3.5 / 3.9 (5 / 9) |
| ranger, rogue | 3.1 / 3.6 (1 / 6) | 3.3 / 3.9 (3 / 9) | 3.2 / 3.8 (2 / 8) | 3.3 / 3.9 (3 / 9) |
| ranger, mage | 3.3 / 3.5 (3 / 5) | 3.8 / 4.0 (8 / 10) | 3.7 / 4.0 (7 / 10) | 3.8 / 4.0 (8 / 10) |
| priest, brawler | 3.0 / 3.4 (2 / 6) | 3.2 / 4.0 (2 / 10) | 3.0 / 3.5 (2 / 7) | 3.2 / 3.8 (2 / 8) |
| priest, rogue | 3.1 / 3.7 (1 / 7) | 3.2 / 4.0 (2 / 10) | 3.1 / 3.8 (1 / 8) | 3.2 / 3.8 (2 / 8) |
| priest, mage | 3.2 / 3.6 (4 / 8) | 3.5 / 4.0 (5 / 10) | 3.0 / 3.7 (2 / 9) | 3.2 / 3.7 (4 / 9) |
| brawler, rogue | 2.8 / 3.1 (0 / 3) | 3.2 / 3.9 (2 / 9) | 3.1 / 3.6 (1 / 6) | 3.2 / 3.8 (2 / 8) |
| brawler, mage | 2.6 / 2.7 (0 / 1) | 3.9 / 4.0 (9 / 10) | 3.1 / 3.3 (1 / 3) | 3.3 / 3.5 (3 / 5) |
| rogue, mage | 3.0 / 3.2 (0 / 2) | 3.7 / 4.0 (7 / 10) | 3.4 / 3.8 (4 / 8) | 3.6 / 3.9 (6 / 9) |
| mean | 3.08 / 3.41 (18 / 51 of 100) | 3.51 / 3.98 (51 / 98) | 3.26 / 3.73 (30 / 77) | 3.38 / 3.82 (40 / 84) |

Mean over trios and both steels: none 3.25, full 3.76, 25 3.51, 50 3.61.

Mean party HP% after each battle (all 10 trios, runs that reached the battle; later values are survivor-biased):

| Heal | B1 | B2 | B3 | B4 arms | B4 gate |
| --- | --- | --- | --- | --- | --- |
| none | 90 | 74 | 61 | 10 | 33 |
| full | 90 | 84 | 87 | 24 | 75 |
| 25 | 90 | 82 | 79 | 15 | 55 |
| 50 | 90 | 82 | 83 | 19 | 62 |

Where runs fall: B4 is the fall point in almost every case under every heal mode (arms: 5-10 of 10 per trio for 9 of 10 trios with no heal; gate: 1-8). Falls before B4 are rare: B2 x1 in four trios (priest+brawler, priest+mage, brawler+rogue, brawler+mage), and brawler+mage with no heal falls at B3 twice. No trio falls at B1.

Per-battle drain, no heal (percentage points of party HP): B1 10, B2 16, B3 13, B4 wipes or leaves 10 (arms) / 33 (gate). Weakest trios without heal: brawler+mage (HP 83 / 59 / 35 after B1-B3, 2.6 survived) and rogue+mage (87 / 62 / 37). Healthiest: ranger+priest (96 / 90 / 84).

Ranking by battles survived, no heal: ranger+priest 3.8; ranger+rogue, ranger+mage, priest+rogue, priest+mage 3.4; ranger+brawler, priest+brawler 3.2; rogue+mage 3.1; brawler+rogue 3.0; brawler+mage 2.6. With full heal: brawler+mage 4.0; ranger+mage, rogue+mage 3.9; ranger+brawler 3.9; brawler+rogue last at 3.5.

Findings for the HEAL1 spec (context: the user wants fights threatening through attrition, no loss penalty, little tuning):

- With no heal the party now lasts 3.25 of 4 battles on average (3.08 arms, 3.41 gate): B1-B3 cost only about 40 points of party HP in total (90 / 74 / 61 left), so the first three fights alone do not wipe parties. Attrition shows up only at B4, which a no-heal party reaches at 61% HP and then loses (arms 18 of 100 cleared, gate 51 of 100).
- Heal level changes the outcome mostly through B4 entry HP: 61% (none), 79% (25), 83% (50), 87% (full). The arms clear rate rises 18 / 30 / 40 / 51 and the gate clear rate 51 / 77 / 84 / 98 of 100 across none / 25 / 50 / full.
- Even with full heal the arms finale is lost in about half of runs (51 of 100 cleared; worst trios ranger+rogue 3, priest+brawler 2, priest+rogue 2, brawler+rogue 2 of 10), while the gate finale is almost always cleared (98 of 100). The heal rule matters less than which B4 the player faces.
- A heal of 50 gives most trios a survivable chain on the gate steel (84 of 100) but not on the arms steel (40 of 100); 25 to 50 adds only 0.10 battles on average. Downed heroes stay down in 25 / 50 (no revive tested), which is the likely reason B4 end-HP stays low.
- Differences between trios are small compared with the old data: at heal 50 all trios survive 3.4-3.9; brawler+mage is the weakest in every mode (B3 HP 35 with no heal). Priest+rogue, an outlier in the first run, is now an ordinary trio.

## BAL3-R final rerun 2026-10-18

Scope: BAL3-R (docs/specs/BAL3.md sections 7 and 8). Checkout: cloud thread, branch claude/bal3-r-final at develop a3b7c35 (BAL3-R0, S, AI, A and CAMP1-A1/A2 landed; develop instead of main is agreed). Method as in BAL3-R0: `tools/fc.sh sim party|sweep|role|duel|tier`, cap 10, smart_skills, spend class, preset balanced (role: spend class/cons/primary x preset balanced/aggressive), jitter on (new). **Party and sweep use 5 seeds (R0: 3)**, role 3 seeds, duel/tier 3 seeds; so R0 -> R deltas mix the BAL3 levers with the seed count. About 3,000 fights. **0 `METRIC FAIL` everywhere; 2 `PLAN FAIL` (1 in sweep L3, 1 in sweep L5, both path reach under jitter), 0 in all other runs.** Sweep L7 was not completed (hit a 3,000 s timeout, run in parallel with others). X = outside the decided target. Raw output: scratchpad `bal3r/` (not committed).

### What each BAL3 step changed

| Step | Change | Effect measured here |
| --- | --- | --- |
| BAL3-S | Seeded start jitter (default on); aoe_swarm as a 2x3 block with a third brawler minion; striker_priority priority mage is standard tier at L3; line = 4 standard, elite = elite + 3 standard, boss = boss + 2 standard + 2-minion wave | Cells are no longer 0/100 only (22 of 40 party cells strictly between, R0 5 of 45); line/elite/boss drop from 100 to 80-100; L3 striker rogue 0 -> 67 |
| BAL3-AI | Lever B (skip Focus under pressure), C (faster tier when likely to die, all units), heal reservation, D (taunt commit), minion skill ceiling 2 | Mage deals and priest heals in every cell mean (priest heal min 23, L5 elevel +2 `line`); tank allies hp_end moved 17 -> 6 (worse), support heal 0 -> 1-5% |
| BAL3-A | sweep_kick power 0.8/1.0/1.15; elite power 1.1 -> 1.0; sim tank loadout drops riposte | Brawler aoe foes/cast 0.1 -> 1.3-1.8; solo elite still 0 for 5 classes (priest 33 -> 50); tank hp_end unchanged. Tried and gated off (no effect): firebomb reach 3, mend 10/12/14 and renewal, kite shield mitigation |

### Headline (R0 -> R)

1. **Party archetypes moved into or near band at L3 and L5 default but not everywhere**: L3 default line/wave/elite now 80/80/60 (R0 100/100/100) and only boss stays 100 X. L5 default 100/100/80/100/80 (mob/line/wave/elite/boss) vs R0 100/100/67/100/100: `line` and `elite` still 100 X. L7 default wave 100 X, elite 80 X, boss 80 X.
2. **Boss is the largest open gap**: 80-100% in every default and role cell and sweep 92 / 89 (L3 / L5) vs band 30-50. Even elevel +2 gives boss 80.
3. **The scenario bands got worse in several places**: L5 tank allies hp_end 17 -> 6, tank warrior clear 67 -> 33, ranged clear 100 -> 39, striker rogue clear 67 -> 44, support priest surv vs brawler -0.06 -> -1.00 (the brawler baseline itself rose 0.83 -> 1.83 survivors). The ranger/rogue/warrior reference classes no longer win the scenarios for free; the primaries now rank first on their first KPI in all five L5 roles (aoe: brawler 1, mage 3) but clear below the 50-85 band in four of five.
4. **Duels barely changed**: warrior 93 / 90, priest 0-4, ranger L3 80 (new X high, R0 67), mage L3 33 (now in band).
5. Sweep swap deltas stay inside +-10 (L3 max 5.2, L5 max 7.6).

### 1. Party mode, default comp warrior,ranger,priest,mage (5 seeds; R0 3 seeds in brackets)

| Encounter | Target | L3 default | L3 role | L5 default | L5 role | L5 elevel +1 | L5 elevel +2 | L7 default | L7 role |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mob | >= 90 | 100 [100] | 100 [100] | 100 [100] | 100 [100] | 100 | 100 | 100 [100] | 100 [100] |
| line | 70-90 | 80 [100 X] | 80 [100 X] | 100 X [100 X] | 80 [100 X] | 60 X | 40 X | 80 [100 X] | 100 X [100 X] |
| wave | 55-80 | 80 [100 X] | 80 [100 X] | 80 [67] | 40 X [100 X] | 100 X [33 X] | 80 [0 X] | 100 X [100 X] | 100 X [67] |
| elite | 50-75 | 60 [100 X] | 60 [100 X] | 100 X [100 X] | 80 X [67] | 80 X [100 X] | 80 X [100 X] | 80 X [100 X] | 80 X [100 X] |
| boss | 30-50 | 100 X [100 X] | 100 X [100 X] | 80 X [100 X] | 100 X [100 X] | 60 X [100 X] | 80 X [100 X] | 80 X [100 X] | 100 X [100 X] |

- Survivors / party hp (default L5): mob 3.8/4 0.74, line 2.0/4 0.34, wave 2.6/4 0.46, elite 3.0/4 0.62, boss 2.8/4 0.57. L3 boss takes 7.0 rounds (survivors 2.0/4).
- Mage dealt and priest healed (cell means) in every one of the 40 cells; smallest priest heal 23 (L5 elevel +2 `line`) and 31 (L7 `line`). Per-fight minima are not printed, so "every fight" is shown on means only.
- Cells strictly between 0 and 100: 22 of 40 (R0 5 of 45); the seed-jitter acceptance (>= half) is met.
- Elevel scaling works for `line` (100 -> 60 -> 40); `boss` (80 / 60 / 80), `wave` and `elite` do not respond monotonically.

### 2. Composition sweep (15 comps, 5 fights per cell, mean clear % over comps)

| Level | mob (>= 90) | line (70-90) | wave (55-80) | elite (50-75) | boss (30-50) |
| --- | --- | --- | --- | --- | --- |
| L3 | 99 [100] | 80 [100 X] | 85 X [89 X] | 69 [100 X] | 92 X [100 X] |
| L5 | 100 [100] | 88 [98 X] | 80 [73] | 72 [100 X] | 89 X [100 X] |
| L7 | not run (timeout) | | | | |

| Class | L3 swap delta / top-5 | L5 swap delta / top-5 |
| --- | --- | --- |
| warrior | +1.6 / 4 [+6.7 / 4] | -2.0 / 3 [+3.3 / 4] |
| ranger | +5.2 / 4 [-3.3 / 3] | +7.6 / 4 [+3.3 / 4] |
| priest | -4.4 / 2 [+0.7 / 3] | -6.8 / 3 [-6.7 / 3] |
| brawler | -0.8 / 4 [-3.3 / 3] | -3.2 / 3 [-6.7 / 2] |
| rogue | -3.2 / 2 [+2.7 / 5] | -0.8 / 4 [+7.3 / 5] |
| mage | +1.6 / 4 [-3.3 / 2] | +5.2 / 3 [-0.7 / 2] |

L3 best warrior,ranger,brawler,mage 96, worst priest,brawler,rogue,mage 76; one comp under 50% on `line`: ranger,priest,rogue,mage 40. L5 best 96 (two comps), worst warrior,priest,brawler,rogue 64 (`wave` 40, `line` 60, `elite` 40).

### 3. Role limit tests, L5 (mean [floor..ceiling]; R0 in brackets)

| Role | Primary result | Bands |
| --- | --- | --- |
| Tank | warrior clear 33 [0..67] X (R0 67), rank 1; brawler S rank 2 (R0 6) | dmg_taken share 45 ok; atk_rcv share 42 X (38); taunt/alive 24 (50); allies hp_end 6 X (17); survival 33 X (39) |
| Ranged | ranger clear 39 X (100), rank 1 | nearest_foe 1.60 X (1.86); ranged dmg 90 ok; atk_rcv/rnd 2.24 X (2.35) |
| AoE | brawler clear 11 X, foes/cast 1.56 X (0.10), rank 1; mage clear 6 X, foes/cast 0.93 X (1.78), rank 3 X | allies/cast mage 0.51 X, brawler 0.02 ok; swarm clear 11 / 6 vs 70 |
| Support | priest clear 50 ok (44), rank 1 (3) | heal 1% [0..2] X (0); saves 0 X; surv vs brawler -1.00 X (-0.06; baseline brawler 1.83) |
| Striker | rogue clear 44 X (67), rank 1 (2) | down by r5 67 ok (72); prio dmg 54 ok (65); prio down round 4.83 |

### 4. Role limit tests, L3 and L7

| Role / class | L3 clear (50-85) | L3 bands | L7 clear | L7 bands |
| --- | --- | --- | --- | --- |
| Tank / warrior | 50 [0..100] ok (44) | share 41 ok; atk_rcv 36 X (50); taunt 22; hp_end 10 X (3); survival 33 X | 39 [33..67] X (44) | share 39 X, rank 2 X (rank 4); atk_rcv 27 X (35); taunt 4; hp_end 9 X (1) |
| Ranged / ranger | 28 X (72), rank 1 (2) | nearest_foe 1.58 X; atk_rcv 2.19 X; ranged dmg 40 X (80) | 50 [33..100] ok (72) | nearest_foe 1.94 X; atk_rcv 1.94 X; ranged dmg 89 ok |
| AoE / mage | 0 X | foes/cast 0.42 X (0.00), rank 2 | 0 X | foes/cast 1.12 X (3.75), rank 3 X |
| AoE / brawler | 22 X (0) | foes/cast 1.29 X (0.00), rank 1 | 22 X (0) | foes/cast 1.81 X (0.17), rank 1 |
| Support / priest | 22 X (28) | heal 0 X; saves 0 X; surv vs brawler -0.61 X | 83 ok (50) | heal 5 [0..12] X; saves 0 X; cleanse 0.78; surv vs brawler -0.83 X |
| Striker / rogue | 67 ok (0), rank 2 (mage rank 1) | down by r5 83; prio dmg 39 X | 61 ok (83), rank 1 (3) | down by r5 67 ok; prio dmg 45 X |

### 5. Diagnostics (duel 3 seeds, band 30-70; R0 in brackets)

| Class | Duel L5 (bin/hp) | Duel L3 | Tier L5 elite 1v1 | Tier L5 boss 1v1 |
| --- | --- | --- | --- | --- |
| warrior | 93/89 X [90/89] | 90/93 X [93/88] | 0/0 | 0/0 |
| ranger | 60/60 [53/53] | 80/77 X [67/64] | 0/0 | 0/0 |
| priest | 0/4 X [3/4] | 0/0 X [10/17] | 50/50 [0/36] | 67/50 [0/33] |
| brawler | 57/57 [57/57] | 57/57 [50/51] | 0/0 | 0/0 |
| rogue | 50/50 [60/60] | 40/40 [60/60] | 0/0 | 0/0 |
| mage | 40/40 [37/37] | 33/33 [20/20] | 0/0 | 0/0 |

Tier L5: minion and standard 1v1 100 for every class (priest standard 67/55); 1v3 minion ranger 17, priest 67/53, brawler 33, others 0.

### Section 8 acceptance checklist

| Criterion | Result | Numbers |
| --- | --- | --- |
| Party default + sweep L3/L5/L7: every archetype inside band | not met | Default L5 line 100, elite 100, boss 80; L3 boss 100; L7 wave 100, elite 80, boss 80. Sweep L3 wave 85, boss 92; L5 boss 89; L7 not run |
| Sweep swap delta within +-10 | met | max 5.2 (L3), 7.6 (L5) |
| No comp below 50% on `line` | not met (L3) | L3 ranger,priest,rogue,mage 40; L5 min 60 |
| Every class in >= 2 of the top 5 | met | L3 priest and rogue 2, all others 3-4; L5 all >= 3 |
| Role L5: primary ranks first on first KPI | met except aoe mage | warrior, ranger, rogue, priest, brawler rank 1; mage rank 3 |
| Role L5: primary clears 50-85 | not met (1 of 5) | support 50; tank 33, ranged 39, aoe 11 / 6, striker 44 |
| Role L3 within 15 points of L5 | not met | ranged 28, aoe 22 / 0, support 22, tank 50, striker 67 |
| Mage deals and priest heals in every L5 party fight | met on means | all cells > 0 (priest min 23); per-fight not printed |
| Duels 30-70 or explained | partly | warrior 93 / 90, priest 0-4, ranger L3 80 outside; explained below as diagnostics (priest is a support, warrior has no sim answer yet) |
| Solo elite / boss per section 6 | not met | solo elite 0 for 5 classes (priest 50); solo boss 0 (priest 67) |
| 0 `PLAN FAIL`, 0 `METRIC FAIL` | not met on PLAN | METRIC 0; PLAN 2 (sweep L3: simp_rogue_3 no path to a minion priest; sweep L5: simp_brawler_2 too far, 9 tiles), both only with jitter, 0 in all 17 other runs |

### Known open gaps (confirmed or refuted)

| Gap | Result |
| --- | --- |
| aoe_swarm clear low | confirmed: brawler 22 / 11 / 22, mage 0 / 6 / 0 (L3 / L5 / L7) |
| support_attrition heal about 1% | confirmed: 0 / 1 / 5 (L3 / L5 / L7), band 20; saves 0 everywhere |
| Tank allies hp_end about 6 | confirmed: 10 / 6 / 9, band 40 |
| Duel warrior 93, priest 0 | confirmed: L5 93 / 0 (hp 4), L3 90 / 0 |
| Solo elite 0 for 5 classes | confirmed (priest 50); jitter and elite power 1.0 did not move it |
| Party line / elite / boss above band | confirmed for boss everywhere and L5 default line / elite; refuted for L3 line / elite, L7 default line (80) and sweep L3/L5 line and elite (in band) |
| New: ranged clear and rogue / tank clears collapsed | the R0 gift (reference class wins by default) is gone; the scenario is now harder than its 50-85 band |

### Proposed band recalibration (proposed, needs user approval; no decision rows edited)

Basis: R = this run (L3 / L5 / L7). Rule used: recalibrate a band only where the metric stayed outside it across all three BAL3 lever families (sim, AI, data); keep it where a lever can still reach it.

| Band | Current | Measured | Proposed | Why |
| --- | --- | --- | --- | --- |
| Tank allies hp_end | >= 40 | 10 / 6 / 9 | >= 15 | never above 17 in any run; the taunt/shield_wall schema gap blocks the lever |
| Tank atk_rcv share | >= 45 | 36 / 42 / 27 | >= 35 | warrior takes 40-45% of damage; share moves with the foe count |
| Tank survival | >= 60 | 33 / 33 / 33 | >= 35 | same |
| Role primary clear | 50-85 | tank 50 / 33 / 39, ranged 28 / 39 / 50, striker 67 / 44 / 61, support 22 / 50 / 83 | 30-85 | limit tests are meant to be hard; only aoe (0-22) stays out and keeps its lever |
| Ranged nearest_foe | >= 2.5 | 1.58 / 1.60 / 1.94 (ceiling 2.62) | >= 1.8 | the jitter reduces dive distance; 2.5 reached only in the best variants |
| Ranged atk_rcv/rnd | <= 1.0 | 2.19 / 2.24 / 1.94 | <= 2.0 | no variant under 1.38 |
| AoE foes/cast | >= 2.0 | brawler 1.29 / 1.56 / 1.81, mage 0.42 / 0.93 / 1.12 | brawler >= 1.5; mage >= 1.0 | mage firebomb reach could not be raised (no effect) |
| AoE swarm clear | >= 70 | brawler 22 / 11 / 22, mage 0 / 6 / 0 | keep 70 (lever open) | a lever family is still available (scenario size, mage AoE) |
| Support heal / party hp | >= 20% | 0 / 1 / 5 (ceiling 12) | keep (lever open), interim >= 5 | mend and renewal increases had no effect; needs a skill or AI change |
| Support surv vs brawler | >= 0.5 | -0.61 / -1.00 / -0.83 (baseline brawler 0.83-2.50) | redefine, drop the band | a brawler in the slot is always the better survivor; the metric cannot be met by a healer |
| Striker prio dmg | >= 50 | 39 / 54 / 45 | >= 40 | within noise of the band |
| Duel 30-70 | all classes | warrior 90-93, priest 0, ranger L3 80 | priest exempt (support); warrior stays flagged | priest is not a duelist; warrior needs a lever |
| Party boss | 30-50 | 80-100 everywhere (sweep 92 / 89) | keep 30-50, no recalibration | bosses are the largest gap and need a tier lever, not a wider band |
| Party wave | 55-80 | sweep 85 / 80 | 55-85 | one point over at L3 |
| Solo elite / boss | ~50 / 10-20 | 0 / 0 (priest 50 / 67) | drop the solo bands, keep the party bands | a solo hero is not meant to beat an elite at equal level; confirm with the user |

### Issues for the brainstorm agent

- **Boss tier**: 80-100% party clear at every level and elevel +2 is still 80. No BAL3 lever touched boss HP or power. Candidate levers: boss HP/power multipliers (StatRules tier values), boss skill set, wave size.
- **line / elite at L5 default**: still 100. Candidates: elite tier multipliers, archetype composition (the sim lever S is already spent), enemy AI.
- **Support heal 1-5%**: heal reservation, mend and renewal tuning all failed to move it. Candidates: heal skill power, a second heal slot, priest attribute scaling, full per-block re-targeting (deferred in BAL3).
- **Tank allies hp_end 6**: lever D (taunt commit) and the sim loadout did not help. Schema gap: shield_wall has one grant slot and taunt uses it, so a fortify grant is impossible. Candidates: a second grant slot, taunt radius/duration, warrior armor or HP.
- **aoe_swarm clear 0-22**: brawler sweep_kick changed foes/cast 0.1 -> 1.6 but clears stayed below 25; firebomb reach 3 had no effect on the mage (0.4-1.1). Candidates: AoE shape and reach in the skill schema, swarm size, mage AI target choice.
- **Duel warrior 90-93**: kite shield mitigation had no effect. Candidates: warrior stats or gear budget, the duel AI (kiting classes).
- **Priest solo 0**: not a duelist; decide whether the duel band applies.
- **Solo elite / boss 0**: decide whether the solo bands exist (see proposal).
- **PLAN FAIL under jitter**: 2 of ~375 fights per sweep (unreachable target after jitter and block movement). Not a balance issue; the sim planner should treat them as skipped fights.
- **Sim tooling**: `enc=` only sees `data/catalog` encounters, not `data/worlds/<world>` ones (CAMP1-B needed a scratch wrapper); a catalog copy of a goblin at L5 fails spend validation (`simc_goblin_skirmisher`: spend 6, a level 5 unit spends 8), so level-matched elevel on world finales is impossible for some units.
- Sweep L7 did not finish inside 3,000 s with other runs in parallel; it needs a dedicated run.

Cloud run times (4 cores): role L3 / L5 / L7 about 25-40 min in parallel, sweep L3 / L5 about 20 min each under load, party/duel/tier 1-4 min each; about 1 h 45 min wall.

## CAMP1-B Ashmaw finale sim 2026-10-18

Archived verbatim in `docs/archive/BALANCE_REPORT_2026-10.md` (above BAL3-R0).
