# Balance report (written by the balance agent; read by the brainstorm agent)

## BAL2 rerun 2026-10-17 (worktree `ngp-bal2run`, branch bal/bal2run at main 3155f59: BAL2-L tooling complete + BAL2-A data)

Scope note: ROADMAP S2/S4 are not promoted to `balance-ready`; this run is the BAL2 gate at the user's request. Method: `tools/fc.sh sim party|sweep|role|duel|tier` as in README "Balance rules and sim v2", cap 10, smart_skills on, 3 seeds (seed i = 11 + 12 i), no code or data edits. 0 `METRIC FAIL` in every run (conservation checks hold). X = outside the decided target. Levels 3 and 5 only (L1/L7 not run).

### Headline

1. **The party/sweep numbers measured by the shipped sim are contaminated by a plan-abort defect** (issue 1). In about a quarter of party fights (19 of 75 at L5, 18 of 75 at L3, seed 11, balanced) `session.confirm_plan()` is refused because the preset auto-plan contains an invalid step of one ally (typically "The planned approach tile (x, y) is taken; re-plan this attack", warrior/brawler/rogue approach conflicts; also "Another unit is (or will be) standing there", "can't jump to", "Moving must come first", "Strike needs 2 in a row, 1 left"). The sim loop `break`s on that refusal, so the fight ends early and is scored as an unfinished fight: a "stall" with a healthy party and rounds below the cap. Brawler is in 18 of the 19 aborted L5 fights and warrior in 15 (both melee approachers), which is what produced the baseline's brawler composition penalty. Re-applying the preset (up to 3 times) always produced a valid plan (0 residual repairs). Tables below give both the shipped-sim numbers and a repaired-runner re-run (scratch copy of `tools/` outside the repo, one change: retry `plan_apply_preset_all` before `confirm_plan`; clear invalid steps as last resort, never needed). Treat the repaired numbers as the real balance picture; the shipped-sim numbers show what the gate tool prints today.
2. With the abort removed, party clear rates are **too high, not too low**: `line`, `elite`, `boss` clear 100% (targets 70-90 / 50-75 / 30-50) at L3 and L5, with elite/boss also 100% at elevel +1/+2, elite=mage, boss=warrior. Only `wave` and `mob` land in band.
3. **Mage (AoE) is killed in round 1 before its first cast resolves** in 12 of 17 probed L5 aoe_swarm fights (and deals 0 damage in the default-party mob/wave fights). Completed casts hit 2.0 foes with 0 ally hits (KPI pass); the KPI failure is purely survival. Same early-death pattern costs the priest its heals in the support limit test (heal 0, 0 near-death saves).
4. **Role limit tests**: only Striker (rogue) and Ranged (ranger) rank first; Tank passes its first-KPI ranking (warrior rank 2) but fails 2 of 4 bands; AoE and Support primary classes fail their bands. Scenario clear rates are mis-scaled: aoe_swarm 0% for every class, striker_priority 11% for the rogue, ranged_dive 100% (even warrior), all against the 50-85 target.
5. Duel diagnostic L5 is unchanged from the baseline (warrior 89 X high, priest 4 X low; 4/6 in 30-70).

### 1. Party mode, default comp warrior,ranger,priest,mage (3 fights per encounter, clear %; target band in the header)

| Encounter | Target | L3 default loadout | L3 loadout=role | L5 default loadout | L5 loadout=role |
| --- | --- | --- | --- | --- | --- |
| mob | >= 90 | 100 (surv 2.0/4) | 100 (2.0/4) | 100 (3.0/4) | 0 X shipped / 100 repaired (1.7/4) |
| line | 70-90 | 100 X | 100 X | 100 X | 100 X |
| wave | 55-80 | 100 X | 100 X | 0 X (4 dead, r 7) | 0 X (also repaired) |
| elite | 50-75 | 100 X | 100 X | 100 X | 0 X shipped (abort) / 100 X repaired |
| boss | 30-50 | 100 X | 100 X | 100 X | 100 X |

Repaired runner changes only the loadout=role L5 rows (mob, elite); the other columns had no abort. Survivors/party HP at L5 default: mob 3.0/4 hp 0.44, line 3.7/4 0.81, wave 0/4, elite 4.0/4 0.94, boss 2.7/4 0.48. One comp of fights per cell (3 near-identical seeds), so cells are 0 or 100. Variants (L5, default comp): elevel +1: line 100, wave 67 (0.7/4 survive), elite 100, boss 100; elevel +2: line 100, wave 0, elite 100, boss 100; `elite=mage` 100 (2.0 rounds, 4/4 survive), `boss=warrior` 100 (3.7/4). The boss encounter (boss brawler plus two minions) is not a threat to a level-matched four-person party even two levels up.

### 2. Composition sweep (15 distinct 4-class comps, 3 fights per cell, mean clear % over comps)

| Run | mob (>= 90) | line (70-90) | wave (55-80) | elite (50-75) | boss (30-50) |
| --- | --- | --- | --- | --- | --- |
| L3 shipped sim | 78 X | 73 | 76 | 62 | 64 X |
| L3 repaired runner | 96 | 100 X | 76 | 89 X | 100 X |
| L5 shipped sim | 82 X | 82 | 40 X | 73 | 73 X |
| L5 repaired runner | 98 | 100 X | 56 | 100 X | 100 X |

Sweep limits (swap delta within +-10, no comp below 50% on `line`, every class in at least 2 of the top 5):

| Class | L3 shipped delta / top5 | L3 repaired | L5 shipped | L5 repaired |
| --- | --- | --- | --- | --- |
| warrior | -18.0 X / 4 | +8.0 / 5 | -4.7 / 3 | +4.0 / 5 |
| ranger | +18.0 X / 4 | +2.0 / 3 | +21.3 X / 4 | -4.0 / 3 |
| priest | +26.0 X / 4 | -6.0 / 2 | +9.3 / 3 | -6.0 / 2 |
| brawler | -42.0 X / 0 X | -12.0 X / 2 | -30.7 X / 2 | -2.0 / 2 |
| rogue | +18.0 X / 4 | +8.0 / 5 | -2.7 / 4 | +4.0 / 5 |
| mage | -2.0 / 4 | +0.0 / 3 | +7.3 / 4 | +4.0 / 3 |

On the shipped sim, `line` is below 50% for 4 comps at L3 and 3 at L5 (all contain brawler plus mage/rogue/warrior, i.e. aborts); on the repaired runner no comp is below 50% on `line` (all 100).

Composition ranking, repaired runner (clear %, party HP):

- L5: 1 warrior,brawler,rogue,mage 100 (0.69); 2 warrior,ranger,priest,rogue 100 (0.66); 3 warrior,ranger,brawler,rogue 100; 4 warrior,ranger,rogue,mage 100; 5 warrior,priest,rogue,mage 100; 6 warrior,priest,brawler,mage 100; 7 ranger,priest,brawler,mage 100; 8 ranger,brawler,rogue,mage 93; 9 priest,brawler,rogue,mage 87; 10-15 at 80 (all lose `wave`: ranger,priest,brawler,rogue; warrior,ranger,priest,mage; warrior,ranger,brawler,mage; ranger,priest,rogue,mage; warrior,ranger,priest,brawler; warrior,priest,brawler,rogue). Spread of the comps is only `wave` (and `mob` for two comps).
- L3: top is warrior,priest,rogue,mage 100 (0.76); bottom is ranger,priest,brawler,mage 53, warrior,priest,brawler,rogue 67, warrior,ranger,brawler,mage 80 (`wave` and `elite` losses). Brawler comps sit at ranks 2, 4, 7, 9-15; the two worst comps contain brawler and no rogue/warrior pair.
- Shipped-sim L5 order differs a lot (warrior,ranger,priest,rogue 100 at rank 1, warrior,priest,brawler,rogue 20 at rank 15) because of aborts.

Rogue and warrior appear in all five top comps at both levels; brawler and priest only just meet the 2-of-5 limit.

### 3. Role limit tests, L5 (full variant grid spend x preset, 3 seeds = 18 fights per cell; mean [floor..ceiling]; P primary, S secondary)

Tank (tank_hold): bands dmg_taken share 40-65, attacks_received share >= 45, survival >= 60, reference allies hp_end >= 40; rank on dmg_taken share.

| Class | clear (50-85) | dmg_taken share % | atk_rcv share % | taunt/alive % | allies hp_end % | survival % | rank |
| --- | --- | --- | --- | --- | --- | --- | --- |
| warrior P | 50 [33..67] | 41 [37..44] | 36 [29..44] X | 8 [0..19] | 10 [9..11] X | 50 [33..67] (range overlaps 60) | 2 (top 2 ok) |
| brawler S | 61 [33..100] | 43 [39..46] | 37 | 0 | 5 | 33 | 1 |
| rogue | 56 | 39 | 39 | 0 | 15 | 0 | 3 |
| priest | 33 | 36 | 38 | 0 | 7 | 0 | 4 |
| mage | 6 | 32 | 30 | 0 | 0 | 6 | 5 |
| ranger | 78 | 24 | 39 | 0 | 0 | 50 | 6 |

Warrior holds the line (41% of damage) but draws only 36% of the attacks (band 45) and its reference allies end at 10% HP (band 40): the tank does not protect the back line. Taunt is active in only 8% of its alive blocks; hold-position 0%.

Ranged (ranged_dive): bands nearest_foe >= 2.5, ranged dmg >= 70, atk_rcv per round <= 1.0.

| Class | clear | nearest_foe | ranged dmg % | atk_rcv/rnd | survival % | rank |
| --- | --- | --- | --- | --- | --- | --- |
| ranger P | 100 [100..100] X (too easy) | 2.23 [1.44..3.00] (ceiling reaches 2.5) | 100 | 2.29 [1.33..3.00] X | 83 | 1 |
| mage S | 17 | 1.60 | 33 | 2.39 | 17 | 2 |
| rogue | 22 | 1.20 | 0 | 2.13 | 0 | 3 |
| warrior | 100 | 1.09 | 0 | 2.23 | 100 | 4 |

The ranger gets hit 1.3-3.0 times per round even in its best variant (band <= 1.0): nobody is kited. Warrior clears 100% as well, so this scenario does not stress the ranger.

AoE (aoe_swarm): bands foes/cast >= 2.0, allies/cast <= 0.3, swarm cleared >= 70%.

| Class | clear | foes/cast | allies/cast | swarm clear % | dmg share % | rank |
| --- | --- | --- | --- | --- | --- | --- |
| brawler P | 0 [0..0] X | 0.83 [0.00..1.50] X | 0 | 0 X | 64 | 1 |
| mage P (secondary role of the user's AoE ask) | 0 X | 0.72 [0.00..2.00] (ceiling meets 2.0) | 0 | 0 X | 20 | 2 |
| ranger S | 11 [0..67] | 0.39 [0..2.33] | 0 | 11 | 19 | 3 |
| priest / rogue / warrior | 0 | 0.00 | 0 | 0 | 17 / 58 / 74 | 4 / 5 / 6 |

Every class loses aoe_swarm at L5 (clear 0 except 11 for ranger), so the scenario is unwinnable as set up (6 minions with an aimed-shot ranger and two dashing rogues against test unit plus warrior), independent of the AoE class. Early deaths versus completed casts, mage, L5 aoe_swarm (17 probed variants/seeds, `tools/fc.sh run`-style probe): 12 died in round 1 (block 3 or 4: minion ranger Aimed Shot x2.10 for 78-117 plus two rogue Quick Stabs of 30 each against 184 HP) before the first Glacial Burst resolved (the cast is logged as MISS after the DEFEATED entry); the 5 fights that survived longer (cons spend) cast 1-3 times and hit 2 foes per cast with 0 ally hits. So for completed casts the mage passes (foes/cast 2.0, allies 0.0); the mean 0.72 is the early-death mixture. Brawler: dies round 2-3, area hits at most 1 foe per cast (sweep kick, 3-tile cone) in the probed fights; its foes/cast ceiling is 1.5, below 2.0 even when alive.

Support (support_attrition): bands effective heal >= 20% of party HP, saves >= 0.3, survivors >= +0.5 versus a reference brawler in the slot (baseline 2.00 survivors).

| Class | clear | survivors | heal/party hp % | saves | buff rds | surv vs brawler | rank |
| --- | --- | --- | --- | --- | --- | --- | --- |
| priest P | 39 [0..100] X | 0.61 | 0 X | 0 X | 0.33 | -1.39 X | 3 X (all heal 0, tied) |
| warrior S | 50 | 0.83 | 0 | 0 | 3.5 | -1.17 | 6 X |
| ranger / mage / rogue / brawler | 100 / 56 / 89 / 100 | 2.9 / 1.4 / 2.1 / 2.0 | 0 | 0 | 0 | 0.89 / -0.61 / 0.06 / 0.00 | 4 / 2 / 5 / 1 |

The priest heals nothing in any variant. Probe (L5, 4 spend/preset variants): in round 1 nobody is hurt at plan time, so the priest plans Approach and Strike (melee auto-attack, 34 damage); from round 2 the enemy ranger kills it with one Aimed Shot (101) in block 3, after which its queued Greater Mend lands on itself at +0 HP. In the party default comp (allies take damage early) the same priest heals 163-210 per fight, so the heal logic works when something is hurt at plan time; the limit test never gives it that.

Striker (striker_priority): bands priority down by round 5 >= 60%, priority dmg share >= 50.

| Class | clear | prio down round | down by r5 % | prio dmg % | survival % | rank |
| --- | --- | --- | --- | --- | --- | --- |
| rogue P | 11 [0..67] X | 1.00 | 100 | 67 [64..71] | 0 | 1 |
| brawler S | 78 | 2.83 | 83 | 62 | 28 | 2 |
| priest S | 0 | 5.50 | 61 | 100 | 0 | 3 |
| ranger / warrior / mage | 33 / 0 / 0 | 6.6 / 9.7 / 11.0 | 50 / 17 / 0 | 5 / 0 / 96 | 11 / 0 / 0 | 4 / 5 / 6 |

Rogue reaches and kills the mage priority target in block-1 round 1 in every variant (KPIs pass) and then dies; the scenario is lost (clear 11%, rogue survival 0) because the two protecting standard warriors win the rest.

### 4. Role limit tests, L3 (primary class only; same bands)

| Role / class | clear (50-85) | First KPI and range | Other band checks |
| --- | --- | --- | --- |
| Tank / warrior | 50 [0..100] | dmg_taken share 34 [18..48], rank 1 | atk_rcv share 30 [13..41] X, allies hp_end 27 [4..53] overlaps 40, survival 50 [0..100] |
| Ranged / ranger | 94 [67..100] X | nearest_foe 1.61 [1.17..1.92] X (ceiling < 2.5) | ranged dmg 93 ok, atk_rcv/rnd 2.24 [1.44..3.20] X |
| AoE / mage | 0 X | foes/cast 0.00 X (no cast ever resolved) | swarm clear 0 X; dies in round 1 in every variant |
| AoE / brawler | 0 X | foes/cast 0.40 [0..1.0] X | swarm clear 0 X |
| Support / priest | 0 X | heal 0 X, saves 0 X | survivors 0.0 versus brawler baseline 0.50 (-0.50 X) |
| Striker / rogue | 0 X | prio down round 11 (never) X | down by r5 0 X, prio dmg 0 X |

L3 is worse than L5 for AoE, Support and Striker (nothing reaches its first cast/heal/kill); Tank and Ranged track L5.

### 5. Diagnostics

Duel L5 (hp score, 30-70): warrior 89 X high, ranger 53, priest 4 X low, brawler 57, rogue 60, mage 37; identical to the baseline's L5 column. Tier L5 (bin/hp): minion and standard 1v1 100 for every class (priest standard hp 82); elite 1v1 0 (priest hp 36, rogue 15); boss 1v1 0 (priest 33, rogue 13); 1v3 0 except priest vs minion 17 and rogue vs minion 50; unchanged from the baseline. Solo elite (target ~50%) and boss (10-20%) still read 0; the sims here do not show any solo boss damage (hp margin 0 for most classes).

### 6. Stalls (10-round cap hit with a healthy party)

No true stall exists. Every "healthy party, not cleared" fight in the shipped sim (e.g. party L5 loadout=role elite: 4/4 survive, party HP 0.91, no clear; support limit test ranger/mage rows: 3.00 survivors, 0% clear, rounds 1) is a plan abort, not a cap hit. Abort probe (all 15 comps x 5 encounters, seed 11, balanced): L5 19 aborted + 0 cap stalls, L3 18 aborted + 1 genuine mutual-attrition end (ranger,priest,brawler,rogue vs wave: both sides near 0 HP at the cap). Typical aborting error texts at L5: approach tile taken (14), another unit standing there (2), cannot jump (0 at L5, 1 at L3), moving must come first (1), not enough free blocks (1). Probe of what the AI was doing: warrior,ranger,brawler,mage vs line at L5, round 1: warrior's planned approach tile (4, 4) is also the brawler's planned destination, so its first Approach/attack step is invalid; confirm_plan refuses; the sim ends the fight at round 1 with 0 log entries (rounds 1, both sides at full HP, scored 50). A separate observation from the aoe probes: many AI melee actions end "OUT OF RANGE" (0 damage; e.g. warrior Shield Bash/Intimidate/Quick Stab at targets that moved), i.e. blocks wasted on attacks planned from the round-start positions.

### 7. Targets that fail (summary, repaired runner where noted)

| Target | Result | Gap |
| --- | --- | --- |
| Party mob >= 90 | L3 96, L5 98 | ok |
| Party line 70-90 | 100 | +10 over the top |
| Party wave 55-80 | L3 76, L5 56 | ok (L5 at the low edge; 6 of 15 comps get 0) |
| Party elite 50-75 | L3 89, L5 100 | +14 / +25 |
| Party boss 30-50 | 100 at both levels, all elevel/class variants | +50 |
| Sweep swap delta +-10 | brawler -12 at L3 (repaired); shipped sim: 5 classes out at L3, 2 at L5 | abort artefact mostly |
| Sweep no comp < 50% on line | repaired ok; shipped sim X for 4 (L3) / 3 (L5) comps | abort artefact |
| Sweep every class in 2 of top 5 | repaired ok; shipped L3 brawler 0 X | ok after repair |
| Tank bands | dmg share ok; atk_rcv share 36 vs 45; allies HP 10 vs 40 | -9 / -30 |
| Ranged bands | nearest_foe 2.23 (ceiling 3.0), atk_rcv 2.29 vs <= 1.0; scenario clear 100 vs <= 85 | -1.3 per round; +15 clear |
| AoE bands | foes/cast 0.83 (brawler), 0.72 (mage) vs 2.0; swarm clear 0 vs 70 | -1.2 / -70 |
| Support bands | heal 0 vs 20%, saves 0 vs 0.3, survivors -1.39 vs +0.5 | all |
| Striker bands | first KPI ok; scenario clear 11 vs 50-85 | -39 |
| Duel 30-70 (L5) | warrior 89, priest 4 | +19 / -26 |

Comparison with the baseline (2026-10-17): the duel and tier tables are identical (no data change since). The party and role gates are new. The earlier "brawler is a weak composition member" reading (swap delta -31 to -42 on the shipped sim) is mostly the abort artefact (-2 to -12 repaired).

### Issues for the brainstorm agent

Bug first (tool or AI, not a balance lever):

- I1: **Auto-plan emits invalid plans (preset auto-planner, then `confirm_plan` refused, sim aborts the fight).** About 25% of party fights at L3/L5 with melee approachers. In-game this would be an auto-plan that blocks Confirm. Re-running `plan_apply_preset_all` fixes it in every observed case, which suggests the planner's per-ally reservations (approach tile, destination, block order, 2-block skills) are computed from state that the previous ally's step changes. Needs a logic-lane fix (planner reservation) and/or a sim guard (retry/repair before `confirm_plan`, count and print aborts). Until then the shipped `party`/`sweep` numbers must not gate `balance-ready`. Scratch repaired runner: agent scratchpad `simx/` (not in repo).

Data levers (first):

- I2: **Party encounters are far too easy** (line/elite/boss 100% at L3 and L5, boss even at elevel +2). Candidate levers: generated-encounter compositions (more/higher-tier enemies, a boss with real adds), tier multipliers (`StatRules.TIERS` elite 1.25/1.1, boss 1.5/1.1 already make solo elite/boss unbeatable but not party-threatening), enemy loadouts/gear budget, or party-size scaling. Decide whether the targets (boss 30-50% for a four-person party) are meant at equal level or via a level offset; the `wave` encounter is the only one that bites (0% for 6 of 15 comps).
- I3: **Squishy classes die in round 1 under the dive start** (mage 184 HP L5, EHP 199; priest 208; ranger 200): enemy Aimed Shot x2.10 (78-117) plus two Quick Stabs kill a mage in three blocks. Levers: mage/priest HP or CONS growth, armor on the default caster loadout, Aimed Shot multiplier or minion/standard ranger power, rogue dash reach, start distance; this decides the AoE and Support limit tests.
- I4: **Role scenario scale**: aoe_swarm unwinnable for every class (6 minions), striker_priority lost even though the KPIs pass (11%), ranged_dive 100% even for warrior. These are sim constants (`tools/sim/sim_scenarios.gd`), a tuning decision for the brainstorm lane; the spec asks for 50-85% clear.
- I5: **Tank**: warrior draws 36% of attacks (band 45) and its allies end at 10% HP. Levers: taunt duration/multiplier (taunt active only 8% of alive blocks), shield_wall/intimidate, warrior HP/armor; mage/ranger references have no peel.
- I6: **Brawler as AoE primary**: foes per cast 0.8 (ceiling 1.5); sweep_kick/cyclone_kick geometry (3-tile) or the brawler's AoE skill multipliers/range; only data lever without AI.
- I7: **Priest/Support**: heal 0 in the limit test; party mode heal works (163-210 per fight, overheal 0-70). Levers: mend/rally multipliers are secondary; the problem is survival and reactive timing (see AI rules).
- I8: **Solo tiers**: elite 1v1 0% and boss 1v1 0% (hp margin 0) for every class at every level; elite multipliers (1.25/1.1) look too strong for a ~50% solo elite, as already flagged in the baseline.
- I9: **Duel L5**: warrior 89 and priest 4 out of 30-70, unchanged; warrior EHP 468 vs 199-390 and priest EHP 229 are the standing levers (growth, kite shield/D1 mitigation).
- I10: **Bands**: the spec asks the lane to propose recalibrated role bands once after this run; with the abort bug and mis-scaled scenarios the run is not a sound basis. Proposal: keep the starting bands unchanged until items 1 and 4 are fixed, then recalibrate on a clean rerun.

AI rules (AI2 options, with this run's evidence):

- I11: **B, Focus-skip under pressure**: both the mage and the priest spend R1 blocks on Approach/Strike (priest) or an unresolved Glacial Burst (mage) while being focus-fired; skipping Focus/chains and acting first would raise completed casts. Evidence: 12 of 17 early-death mage fights.
- I12: **C, faster tier when likely to die**: the same fights; the casters die before their first heal/cast. Pairs with a defensive reposition (kite/Leap Back) rather than Approach in the opening block.
- I13: **D, wider taunt trigger**: warrior taunt active in 8% of alive blocks, atk_rcv share 36%; wider trigger (any ally threatened, not only low HP) would raise both tank KPIs.
- I14: Plan-time targeting: many melee blocks end OUT OF RANGE (planned from round-start positions; also the priest R2-R3 Strike chase pattern), and heals are planned only for allies already hurt at round start (a mid-round injury cannot be answered; the priest then self-heals at +0 HP after death). Candidate AI rules: re-evaluate targets by block, reserve a heal block when an ally is below a threshold at plan time.

Notes: parallel probes shared one sandbox once (one probe printed `ok false`); the mage early-death count is 12 of 17 valid probes. Seeds give near-identical fights (cells are 0 or 100), as in earlier runs; a seeded start-position or initiative jitter would make bands measurable.

---

Older runs (BAL2 baseline 2026-10-17, BAL1 2026-10-16, baseline 2026-10-15): `docs/archive/BALANCE_REPORT_2026-10.md`.
