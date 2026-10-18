# Balance report (written by the balance agent; read by the brainstorm agent)

## Proposed trio bands 2026-10-18 (awaiting approval)

Tushar chose "Re-set targets" (17:03): new clear bands for the generated L5 trio sweep (warrior + two, 10 comps, 10 seeds), set from BAL5-R, with no game change. They replace only the sweep clear bands of "BAL2 role answers" (2); story bands (stand, HEAL1 arms and gate), role KPIs and the PACE1 round bands stay as they are. Each band is centred on today's mean and wide enough that a small lever moves a cell in or out.

| Tier | Old band (4 heroes) | Today (trio mean) | Proposed trio band |
| --- | --- | --- | --- |
| mob | >= 90 | 91 | >= 85 |
| line | 70-90 | 26 | 20-45 |
| wave | 55-80 | 28 | 20-45 |
| elite | 50-75 | 38 | 30-55 |
| boss | 30-50 | 26 (best 50, worst 10) | 20-40 |

Sweep limits stay (swap delta within +-10, every class in >= 2 of the top 5), except "no composition below 50% on line", which becomes "no trio below 10% on line". With these bands every tier mean passes today, so no lever fires (the per-trio line minimum is checked at the next sweep); B3 (boss HP) keeps its own triggers (boss above 60, or PACE1 boss median under 7 rounds).

## MORALE1-B 2026-10-18

Measured build: cloud branch claude/project-thread-6a605c = develop 69c82e0 (MORALE1-A morale data on `ash_scouts` and `ash_raiders`, MORALE1-S sim mapping). Level 5, boss phase on, jitter on, 10 seeds. "Before" for the party rows is the same sim on the parent 233dd98 (no morale data, same code otherwise), run in a throwaway worktree, so only the morale rows differ. All 31 party runs: 0 PLAN FAIL, 0 METRIC FAIL. Party wall about 6 min (4 processes). Logs: scratchpad `morale1b/`.

Commands: `tools/fc.sh sim party 5 10 comp=<warrior[,a[,b]]> enc=world:ash_scouts|ash_raiders` (solo; five duos; ten trios) and `tools/fc.sh sim campaign 5 10 heal=none` / `heal=25`. The solo before cell (20 clear, 4.5 rounds, party hp 0.17) reproduces R1b exactly.

Headline (clear % / rounds / party HP at fight end; HP is a fraction of max):

| Cell | Before | Now | Band |
| --- | --- | --- | --- |
| solo warrior vs ash_scouts | 20 / 4.5 / 0.17 | 100 / 1.8 / 0.72 | mob >= 90: PASS |
| duo warrior+ranger vs ash_raiders | 100 / 3.6 / 0.54 | 100 / 1.7 / 0.71 | |
| duo +priest | 90 / 4.6 / 0.76 | 100 / 3.4 / 0.75 | |
| duo +brawler | 100 / 3.1 / 0.71 | 100 / 2.0 / 0.73 | |
| duo +rogue | 100 / 3.1 / 0.72 | 100 / 1.9 / 0.80 | |
| duo +mage | 100 / 2.9 / 0.54 | 100 / 1.4 / 0.71 | |
| ten trios | all 100 / 2.0-2.8 / 0.79-0.91 | all 100 / 1.1-1.7 / 0.84-0.91 | |

Trio detail (rounds before / now, end HP now): ranger+priest 2.1 / 1.1 (0.89), ranger+rogue 2.0 / 1.3 (0.90), ranger+brawler 2.1 / 1.4 (0.86), ranger+mage 2.0 / 1.2 (0.89), priest+rogue 2.6 / 1.6 (0.90), priest+brawler 2.8 / 1.7 (0.89), priest+mage 2.6 / 1.2 (0.84), brawler+rogue 2.4 / 1.6 (0.91), brawler+mage 2.5 / 1.7 (0.84), rogue+mage 2.2 / 1.2 (0.87).

Reading: the solo band is met (20 to 100). The clear rate did not move on ash_raiders (it was already 90-100) but every comp now ends faster (0.7 to 1.9 rounds) and the lowest-HP duos (ranger, mage) end with 0.17 more HP. The sim records no start-of-fight HP (every fight starts at full HP); the columns above are end-of-fight HP.

Campaign (`campaign 5 10 heal=none` and `heal=25`): both runs hit the 900 s sim timeout (`[sim] TIMEOUT after 900 s`) with 14 of 20 trio-steel rows done, so only 7 trios finished (ranger+priest, ranger+brawler, ranger+rogue, ranger+mage, priest+brawler, priest+rogue, priest+mage); brawler+rogue, brawler+mage and rogue+mage are missing. No PLAN or METRIC totals were printed. Compared with BAL4-ATTR v4 restricted to the same seven trios (battles) and to all ten (HP, v4 only has all-trio HP):

| Cell | v4 (same 7 trios) | Now (7 trios) |
| --- | --- | --- |
| heal=none mean battles (arms and gate) | 3.51 | 3.58 |
| heal=25 mean battles | 3.61 | 3.80 |
| heal=none party HP after B2 / B3 | 74 / 69 (10 trios) | 77 / 71 |
| heal=25 party HP after B2 / B3 | 82 / 85 (10 trios) | 84 / 88 |

The 7-trio subset is not the 10-trio set, and v4 is an older build (BAL5 and LEVERS1 changes in between), so the shifts (+0.07 and +0.19 battles, +2 to +3 HP) are small and not attributable to morale; B2 and B3 contain no morale rows (`ash_raiders` is the extra battle only in `extra=` chains, not in the default chain). Expect no campaign change from MORALE1-A on this chain; the missing three trios would need a rerun with a higher SIM_TIMEOUT or `comp=` per trio.

Per-run morale counts (units fled while Shaken or Wavering, units surrendered, breaks from crit losses alone): not measured by the sim. A small sim task would add per-fight counters to the party and campaign output: `fled_shaken`, `fled_wavering`, `surrendered`, `break_by_crit_only` (a break where no kill or exit contributed), `break_round`, and units alive at the break.

DR1 checks:

- Does ash_raiders fall under 2 rounds for every comp? Trios: yes (all ten 1.1-1.7). Duos: no (priest 3.4, brawler 2.0; ranger 1.7, rogue 1.9, mage 1.4). So the condition "under 2 for every comp" is not met for duos; no asset lever (`morale_leader` "" or `morale_break` 40) is triggered, but trios are close to it.
- Do flee cascades end fights before real fighting? Cannot be told from the output (no flee counts). Indirect evidence says no: duo fights last 1.4-3.4 rounds and end HP is 0.71-0.80, trios 1.1-1.7 rounds with 0.84-0.91 HP; the shortest cell (ranger+priest trio, 1.1 rounds) is the nearest to a cascade. FLEE_WAVERING 0.20 is not indicated by these numbers.

Open questions:

- Is 1.1-1.7 rounds for trios acceptable as a "lesson" fight, or should `morale_break` go up to 40 (asset lever) for trios only?
- The priest duo stays at 3.4 rounds: the morale break fires late there (unprobed).
- Rerun the campaign with a larger timeout to cover all ten trios.
- Add the morale counters above to the sim before MORALE1 is called balance-ready.

## BAL5-R (final) 2026-10-18

Branch claude/bal5-r (develop 744f175: BAL5-AI, S2 support_attrition scenario and POTLEDGER, BAL5-L extra_grants, BAL5-A shield_wall fortify 4/6/8 and potion price 30, STATCRED1-L). Same commands and seeds as BAL5-R1b (level 5, boss phase on, jitter on; role 3 seeds, sweep and campaign 10). 0 PLAN FAIL and 0 METRIC FAIL in all 8 runs, including the potion arm (the POTLEDGER fix works). Wall about 47 min (4 processes in parallel). Logs: scratchpad `bal5r/`. STATCRED1-L moved the buffs columns (buff / debuff rounds per fight now credit the applier); no band uses them. Potion price is 30 now (it does not touch the sim pouch).

Headline: every cell except the support role is identical to R1b to the digit (tank, AoE, both sweeps, the stand and both campaign arms), so the shield_wall fortify grant moved nothing (see the open question below). The support role moved only because S2 changed its foes: priest survivors 0.89 -> 0.61, priest down round 1.72 -> 1.22, surv vs brawler -0.61 -> -1.06, heal share still 2. The potion arm's METRIC FAIL went 1104 -> 0 (POTLEDGER). The R1b gaps stand.

Before (R1b) / after:

| Cell | R1b | Now | Band |
| --- | --- | --- | --- |
| tank (warrior row) clear | 89 [67..100] | 89 [67..100] | 50-85 |
| tank allies hp_end won / all | 30 / 26 | 30 / 26 | >= 40 / >= 15 |
| tank survival | 61 | 61 [33..100] | >= 35 (spec) |
| priest heal share | 2 [0..14] | 2 [0..15] | >= 20 (interim 5) |
| priest survivors / down round (S2 foes) | 0.89 / 1.72 | 0.61 / 1.22 | n/a |
| priest down before round 3 | 83% | 83% (alive 17%) | alive >= 60% |
| support priest clear | 50 | 50 | 30-85 |
| AoE foes/cast brawler / mage | 0.79 / 1.74 | 0.79 / 1.74 | >= 1.5 / >= 1.0 (role table header says >= 2) |
| AoE allies/cast brawler / mage | 0.10 / 0.38 | 0.10 / 0.38 | <= 0.3 (mage over) |
| swarm clear brawler / mage | 61 / 83 | 61 / 83 | >= 70 |
| boss sweep mean (best / worst trio) | 26 (50 / 10) | 26 (50 / 10) | 30-50 |
| full L5 sweep mob / line / wave / elite / boss | 91 / 26 / 28 / 38 / 26 | 91 / 26 / 28 / 38 / 26 | 90 / 70-90 / 55-80 / 50-75 / 30-50 |
| HEAL1 arms / gate, no potion | 63 / 97 | 63 / 97 | arms 30-55, gate >= 75 |
| HEAL1 arms / gate, potion arm | 82 / 97 | 82 / 97 | same |
| ashmaw stand mean | 73 | 73 | 30-50 |

Band confirmation:

| Band | Result | Pass |
| --- | --- | --- |
| tank allies hp_end won >= 40 | 30 | FAIL |
| tank allies hp_end all >= 15 | 26 | PASS |
| tank survival >= 35 | 61 | PASS |
| tank clear 50-85 | 89 | FAIL (just over) |
| support heal share >= 20 | 2 | FAIL |
| support heal share interim >= 5 | 2 | FAIL |
| priest alive at round 3 >= 60% | 17% | FAIL |
| support priest clear 30-85 | 50 | PASS |
| aoe brawler foes/cast >= 1.5 | 0.79 | FAIL |
| aoe mage foes/cast >= 1.0 | 1.74 | PASS |
| aoe allies/cast <= 0.3 | mage 0.38 | FAIL (mage) |
| swarm clear >= 70 | brawler 61, mage 83 | brawler FAIL |
| boss sweep 30-50 | 26 | FAIL (4 under) |
| full sweep mob >= 90 | 91 | PASS |
| line 70-90 / wave 55-80 / elite 50-75 | 26 / 28 / 38 | all FAIL |
| HEAL1 arms 30-55 (no potion) | 63 | FAIL (8 over) |
| HEAL1 gate >= 75 | 97 | PASS |
| potion arm 0 METRIC FAIL | 0 | PASS |
| stand 30-50 | 73 | FAIL (23 over) |

Ashmaw stand per trio (clear of 10): ranger+priest 10, ranger+mage 10, ranger+rogue 9, ranger+brawler 9, brawler+mage 9, priest+mage 7, rogue+mage 7, brawler+rogue 6, priest+brawler 3, priest+rogue 3. Swap deltas ranger +36.7, mage +15.8, priest -25.8, rogue -17.5. HEAL1 arms per trio with potions: all 10 for ranger+brawler/rogue/mage, priest+mage, brawler+mage, rogue+mage; priest+rogue 1, priest+brawler 5, brawler+rogue 7, ranger+priest 9. Without potions: priest+rogue 1, priest+brawler 4, rogue+mage 4, brawler+mage 6.

Probe answer (2026-10-18, after BAL5-R): the extra grant works (`plan_entry.gd:91`, `defend_command.gd:30`; fortify 4 active for a round after a cast), but the warrior almost never casts shield_wall. In sweeps and campaigns it is not in the default loadout at all (first 5 class-path skills, `content_factory.gd:362`: taunt_shout, shield_bash, riposte, intimidate, sunder; shield_wall is 6th). In the role tank loadout it was cast 6 times in 18 fights, all in rounds 4-5, because the AI uses a plain DEFEND skill only as a low-HP guard at 40% HP or less (`tactical_ai.gd:1103-1118`); 4 of 6 came in the last round. So T3 cannot close the tank gap as built; the next step is a design call (AI guard use or loadout), routed to Design with LEVERS1.

Open question (needs a probe before anything is tuned): the tank loadout includes shield_wall and BAL5-A gave it a fortify extra grant, yet tank hp_end, survival and clear did not move by a single point. Either the sim tank rarely casts shield_wall, or the extra grant does not reach the sim fight, or the effect is below the noise of the 3 seeds. A cast-count probe (`tools/fc.sh run`) would tell which.

Remaining gaps and the spec's next step:

- Tank won-fight allies hp_end 30 (target 40): BAL5_LEVERS T3 has now fired and not closed it; next step is T4 (bodyguard stance), which is an option-build for the design review.
- Priest heal share 2 and down before round 3 in 83%: S2 fired; S3 goes to the design review (with the LEVERS1 priest Ward).
- Brawler AoE 0.79 foes/cast and swarm clear 61: A2 missed; the AoE follow-up goes to the design review (LEVERS1 brawler spin). Mage allies/cast 0.38 is over 0.3.
- Boss sweep 26 and line/wave/elite 26/28/38 far under band: B3 does not fire (it needs boss above 60); the line, wave and elite gaps are trio-era numbers that have no lever in BAL5_LEVERS, so they need a band recalibration or a new lever decision.
- HEAL1 arms 63 without potions (above 55) and stand 73 (above 50): F2 fires; the spec's next lever is the stand magnitudes. Both ignore potion price 30 (the sim pouch is fixed).
- priest+rogue arms 1 of 10 even with potions, priest+brawler stand 3: priest trios stay the worst.

## BAL5-R1b (BAL5-AI rerun) 2026-10-18

Branch claude/bal5-ai-rerun (develop a8dcf23: BAL5-AI T2/S1/A2 AI levers 7ceddcb and the enemy re-draft fix f65f0bb). Same commands and seeds as BAL5-R1 (level 5, boss phase on, jitter on), plus the R1b additions (HEAL1 chain, potion arm, solo scouts). 0 PLAN FAIL everywhere; 0 METRIC FAIL except the potion arm (see below). Wall about 62 min (up to 4 processes in parallel). Logs: scratchpad `rerun/`. The stand was run as `sweep 5 10 size=3 lead=warrior world=ashlands enc=ashmaw_stand` (one row per trio, same cells as R1's party runs).

Before (R1) / after:

| Cell | R1 | Now | Band |
| --- | --- | --- | --- |
| boss sweep mean, L5 | 21 | 26 | 30-50 |
| best / worst trio (boss) | 40 / 0 (ranger-rogue / priest-brawler) | 50 / 10 (brawler-rogue / priest-brawler) | |
| full L5 sweep: mob / line / wave / elite / boss | 76 / 32 / 27 / 33 / 21 | 91 / 26 / 28 / 38 / 26 | 90 / 70-90 / 55-80 / 50-75 / 30-50 |
| tank (warrior row) clear | 33 (R1 mean) | 89 [67..100] | 50-85 |
| tank allies hp_end won / all | 9 / 6 | 30 / 26 (warrior); brawler row 9 / 6 | >= 40 / >= 15 |
| tank survival | 33 | 61 [33..100] (warrior) | >= 60 (spec 35) |
| priest heal share of party HP | 1 | 2 [0..14] | >= 20 (interim 5) |
| priest down before round 3 | 78% | 83% (cell "down<r3") | alive at r3 >= 60% |
| support priest clear | 50 | 50 | 30-85 |
| AoE foes/cast brawler / mage | 1.09 / 1.51 | 0.79 [0.62..1.11] / 1.74 | >= 1.5 / >= 1.0 |
| AoE allies/cast brawler / mage / ranger | 0.04 / 0.14 / 0.22 | 0.10 / 0.38 / 0.04 | <= 0.3 |
| swarm clear brawler / mage | 56 / 72 | 61 / 83 | >= 70 |
| ashmaw stand clear mean (10 trios) | 66 | 73 | 30-50 |
| solo warrior vs ash_scouts | 20 (RECRUIT1-B) | 20 (hp score 0.37, 4.5 rounds) | >= 90 |

Caveat on R1 tank numbers: R1's tank cell (33 clear, 9 / 6, survival 33) matches the brawler row of today's tank table (33, 9 / 6, 33), not the warrior row, so the warrior-row before/after above may be comparing different rows. Today's warrior row is the primary tank.

HEAL1 chain (`campaign 5 10 heal=rally rest=3 extra=4:ash_raiders`, 994 fights; potion arm adds `pouch=con_heal_potion,con_heal_potion,con_heal_potion`, 994 fights). Finale clears of 100 per steel; falls are counted from the fell_at numbers (CAMPLABEL1 label bug ignored):

| Arm | HEAL1-B | Now (no potion) | Potion arm | Pass |
| --- | --- | --- | --- | --- |
| arms | 46 | 63 | 82 | 30-55 |
| gate | 95 | 97 | 97 | >= 75 |
| falls at fell_at 4 / 2 (both steels) | arms 53 + gate 4 | 38 / 2 | 19 / 2 | |

Without potions arms rose 46 to 63 (above 55): the re-draft fix and AI levers made the chain easier, since the rules are the same. Trios still stuck on arms: priest+rogue 1 of 10 (9 falls at battle 4), priest+brawler 4, rogue+mage 4. Potions lift arms to 82 and lift every trio except priest+rogue (1) and brawler+rogue (7). Potion-arm METRIC FAIL: 1104 of 37972 checks, all hp / other_gain mismatches (potion healing is not in the metric ledger); it is a tooling gap, not a game failure, but potion arm numbers are otherwise valid (0 PLAN FAIL).

Trigger table:

| Trigger | Fires? | Measured value | Threshold |
| --- | --- | --- | --- |
| T3 second grant (extra_grants) | YES on won-fight allies hp_end | warrior won-fight allies hp_end 30; tank survival 61 | hp_end < 40 or survival < 35: hp_end part fails, survival passes |
| S2 support_attrition change | YES | priest heal share 2 after S1; priest still down before r3 in 83% | under 5 |
| A2 result | brawler fails, mage passes | brawler 0.79 foes/cast (fell from 1.09), mage 1.74 | brawler >= 1.5, mage >= 1.0 |
| B3 boss HP 1.5 -> 1.8 | no | L5 boss mean 26 | fires only above 60 |
| PACE1 boss median fight < 7 rounds | not measurable | sweep and party output carry no boss round count (only the role and party-mode "rounds" columns) | median < 7 |
| Potion price 15 -> 30 (BAL5-A) | YES | arms finale 82 with potions | arms above 55 |
| ASHPACE1 scout cut | cut is supported | solo warrior vs ash_scouts 20 (not 90+) | cut only if not already 90+ |
| F2 stand | YES on its face | stand mean 73 at full HP (66 in R1) | trio clear > 50 |

Re-draft fix: big shifts against R1 are mob 76 to 91, stand 66 to 73, the HEAL1 arms 46 to 63 and tank survival; line and wave did not move. The AI levers and the re-draft fix were not separated (one change set), so attribution is a guess; a run on f65f0bb alone would split them.

Issues for the brainstorm agent:

- T3: warrior survival passes but allies still finish at 30 percent HP in won fights (target 40); the S1/T2 levers did not close it.
- Priest is still the worst class: down before r3 in 83%, heal share 2; S2 fires. Priest trios remain worst on boss and arms (priest+rogue arms 1 of 10 even with potions).
- Brawler AoE fell to 0.79 foes/cast after A2 (the mage rose to 1.74); the brawler cast count may have dropped from the AI changes rather than risen. Brawler is the open A2 miss.
- Potions make arms 82: the price lever (15 to 30) fires; a rerun after it is needed. Potion healing should be added to the metric ledger.
- Solo ash_scouts is still 20 (Goblin Archer deals 224 with 100 percent rng); ASHPACE1's scout cut is justified.
- Boss round counts are not printed by sweep: PACE1's median trigger needs a sim metric.
- Stand 73 and arms-without-potions 63 both exceed their targets; F2 fires.

## HEAL1-B campaign with HEAL1 rules 2026-10-18

Branch claude/heal1-b (develop f3d8ac2, with HEAL1-L, HEAL1-A and the HEAL1-S sim keys). Command `tools/fc.sh sim campaign 5 10 heal=rally rest=3 extra=4:ash_raiders` (level 5, all 10 trios, 10 seeds, both steels, objectives and phases on), against the baseline `campaign 5 10` (heal=none, no rest, no extra) run in parallel. 994 / 784 fights, 0 PLAN FAIL and 0 METRIC FAIL in both. Wall about 29 min (both in parallel, CPU shared with a test run). Logs: scratchpad `heal1b/`.

Finale clear rate (full chain cleared, of 100 per steel):

| Arm | Baseline (none) | HEAL1 rules | Pass |
| --- | --- | --- | --- |
| arms | 23 | 46 | 30-55: PASS |
| gate | 68 | 95 | >= 75: PASS |

Mean battles survived: arms 3.15 -> 3.44, gate 3.60 -> 3.93. Falls: arms 73 at the stand (baseline) vs 53 at the `ash_raiders` filler plus the stand fight (HEAL1 run, the log labels every one "battle 4 (ash_raiders)"; no fall is labelled with the stand), gate 28 vs 4, 1 early fall (B2) per steel in both. So with the rally rules the filler fight before the stand is where arms runs end; the stand itself is not what kills them. Caveat: the log's "survived n/5" with 4 hp entries and the label were not probed further.

Clear per trio, arms / gate of 10 (baseline in brackets):

| Trio | Arms | Gate |
| --- | --- | --- |
| ranger, priest | 8 (6) | 10 (10) |
| ranger, brawler | 8 (2) | 10 (6) |
| ranger, rogue | 6 (3) | 10 (6) |
| ranger, mage | 8 (4) | 10 (8) |
| priest, brawler | 2 (0) | 9 (9) |
| priest, rogue | 1 (2) | 9 (8) |
| priest, mage | 6 (6) | 10 (9) |
| brawler, rogue | 1 (0) | 10 (6) |
| brawler, mage | 5 (0) | 9 (2) |
| rogue, mage | 1 (0) | 8 (4) |

Verdict: PASS on both arms (46, inside 30-55) and gate (95, above 75). The spec trigger lists no lever: neither "above 55" (BAL5 F2) nor "below 30" (RALLY_PCT 25 -> 50) fires. Note the spread: ranger trios clear 6-8 of 10 on arms, priest+rogue, brawler+rogue and rogue+mage 1 of 10, so the mean hides trios that never clear the arms path (a trio-balance matter, not a HEAL1 one). Not run: other levels, heal=25 or full on the same chain.

## BAL5-R1 trio re-measure 2026-10-18

Branch claude/bal5-r1 (develop c78b8ba, BAL5-SIM tooling; not on main yet). Trios are warrior plus two recruits, boss phase on (default), jitter on, 0 PLAN FAIL and 0 METRIC FAIL in every run. Cells: sweep and party 10 seeds, role and tier 3 seeds as the spec says (the role cell is a mean over 6 variants, ranges in the raw logs). Logs: scratchpad `r1/`. Wall about 2 h (4 processes in parallel; role L5 about 1 h, sweeps 20-60 min). Not run (time): role at L3 and L7, tier and party at other levels; the triggers below are all decided on L5 numbers, which is where the spec reads them.

Trio sweep, boss only (`sweep <lvl> 10 size=3 lead=warrior enc=boss`), boss clear % per trio:

| Trio | L3 | L5 | L7 |
| --- | --- | --- | --- |
| ranger, rogue | 60 | 40 | 30 |
| ranger, brawler | 20 | 30 | 60 |
| ranger, mage | 50 | 30 | 40 |
| ranger, priest | 30 | 10 | 0 |
| brawler, rogue | 20 | 30 | 40 |
| brawler, mage | 20 | 20 | 20 |
| priest, mage | 10 | 20 | 30 |
| priest, rogue | 10 | 10 | 30 |
| priest, brawler | 0 | 0 | 20 |
| rogue, mage | 70 | 20 | 20 |
| mean (band 30-50) | 29 | 21 | 29 |

All three means are just under the band; the gap flipped from far too easy (80-100 with 4 heroes, no phase) to slightly too hard. Trios below 20 (the spec floor): 5 at L3, 5 at L5, 3 at L7 of 10; every priest trio is at or below 30 at L5. Class swap delta: priest is -18 (L3), -14 (L5 all-encounter), -15 (L7), the worst class each time.

L5 full sweep (5 encounters, mean over trios): mob 76 (band >= 90), line 32 (70-90), wave 27 (55-80), elite 33 (50-75), boss 21 (30-50). With a trio the default L5 line and elite are now far below band, not above it (section 7 closed: the 4-hero excess was party size).

Role L5 (mean over variants):

| Role | Metric | Measured | Band |
| --- | --- | --- | --- |
| tank (warrior) | clear | 33 | 30-85 |
| tank | allies hp_end won-fights / all | 9 / 6 | >= 40 / >= 15 |
| tank | survival | 33 | >= 35 |
| tank | dmg_taken share, rank | 45, rank 1 | rank 1 |
| support (priest) | heal share of party HP | 1 | >= 20 (interim >= 5) |
| support | priest down before round 3 | 78% | alive at r3 >= 60% (here 22%) |
| support | priest clear | 50 | 30-85 |
| aoe | swarm clear (brawler / mage) | 56 / 72 | >= 70 |
| aoe | foes/cast brawler (rank 2) / mage (rank 1) | 1.09 / 1.51 | brawler rank 1 and >= 1.5; mage >= 1.0 |
| aoe | allies/cast brawler / mage / ranger | 0.04 / 0.14 / 0.22 | <= 0.3 |

Solo vs a brawler foe (`tier 5 3 tier_foe=brawler`, 1v1 win %): boss warrior 0, ranger 67, priest 0, brawler 0, rogue 0, mage 50 (band 10-20). Elite: warrior 67, ranger 67, mage 67, others 0.

Ashmaw stand (`party 5 10 world=ashlands enc=ashmaw_stand`, full HP, level 5, the BOSS1-A phase is already in the data): clear per trio ranger-priest 90, ranger-brawler 90, ranger-mage 80, brawler-mage 80, ranger-rogue 60, priest-brawler 60, priest-mage 60, rogue-mage 60, brawler-rogue 50, priest-rogue 30; mean 66 (band 30-50). No before-BOSS1-A run was taken, and the barred stand and arrival-HP path were not run.

Trigger table (BAL5_LEVERS sections 1-7):

| Trigger | Fires? | Measured value | Threshold |
| --- | --- | --- | --- |
| B1 trio re-measure (sec 1) | done | boss mean 29 / 21 / 29 at L3 / L5 / L7 | band 30-50 |
| B2 sim boss phase | already on | phase on by default in these runs | n/a |
| B3 boss HP 1.5 -> 1.8 | no | L5 boss mean 21 | fires only if above 60 |
| B4 boss double action | no (design review) | n/a | n/a |
| T1 won-fight split | done in tooling | won 9 vs all 6 | n/a |
| T2 early taunt | YES | won-fight allies hp_end 9; survival 33 | below 40 or below 35 at L5 |
| T3 second grant | not yet (only after T2) | n/a | still failing after T2 |
| S1 healer self-care | YES | priest down before round 3 in 78% of L5 support fights | >= 50% |
| S2 support_attrition change | not yet (only after S1) | heal share 1 | under 5 after S1 |
| A1 trio swarm | tooling state not checked | roles above were run with the sim default | with Q1 |
| A2 friendly-fire tolerance | YES | brawler foes/cast 1.09; mage 1.51 | brawler < 1.5 or mage < 1.0 |
| A3 mage AoE at L3 | no | needs Q4 yes; L3 role not run | Q4 yes |
| D1 / D2 (sec 5) | D2 measurable now | solo boss: ranger 67, mage 50, warrior 0, others 0 | 10-20 per class |
| F1 / F2 / F3 (sec 6) | F2 trigger: YES on its face | stand clear mean 66 (full HP, phase present) | trio clear > 50 after BOSS1-A |
| Sec 7 line / elite | no longer a lever | line 32, elite 33 at L5 | 70-90 / 50-75 |

Issues for the brainstorm agent:

- Boss band is now slightly low (21-29 vs 30-50) and trio spread is huge (0 to 70); priest trios cannot win bosses. B3 does not fire; consider the opposite check (no change) before any tier edit.
- Tank survival 33 and won-fight allies hp_end 9: T2 fires. The mage-slot comparison also shows the warrior leaves allies near 6% in all variants.
- Priest dies before round 3 in 78% of fights: S1 fires; heal share 1 confirms it never casts enough.
- Brawler AoE 1.09 foes/cast against 1.5: A2 fires; mage 1.51 and friendly fire are fine.
- Solo vs a brawler boss: ranger 67 and mage 50 are far above 10-20 while five classes are at 0-ish; D2 band needs the user (Q3).
- Ashmaw stand is still easy at full HP (66 vs 30-50); the arrival-HP (HEAL1) and barred runs are still missing, as is the pre-BOSS1-A comparison.
- Not measured: role at L3 and L7, sweep elite at L3/L7.

## BAL4-ATTR campaign attrition 2026-10-18 (rerun v4)

Headline v3 (before encounter objectives/phases were applied in the sim): means 3.08 / 3.41 battles (arms / gate, no heal), 3.51 / 3.98 full, 3.26 / 3.73 at 25, 3.38 / 3.82 at 50; arms clears 18 / 51 / 30 / 40 of 100 and gate 51 / 98 / 77 / 84. Older runs: 2.04 / 3.54 / 2.70 / 2.94 (first run, older Ashlands data).

What changed in v4: branch claude/bal-probes (develop 15850e0 + sim change 3deb570): the campaign mode now plays each encounter with its authored objective, deployment, phases and guests (v3 played the units only). So `courtyard_skirmish` is a hold, `wagon_raid` is an ambush layout with the guest Wenn, and the stands carry their `slay` objective and boss phases (1 in `ashmaw_stand`, 2 in the barred one). Same method otherwise: `tools/fc.sh sim campaign 5 10 heal=<mode>`, party level 5, warrior plus 2 recruits, all 10 trios, 10 seeds, steel arms / gate, 784 / 800 / 792 / 796 fights (none / full / 25 / 50), 0 PLAN FAIL and 0 METRIC FAIL in all four. Four runs in parallel, about 47 min wall.

Mean battles survived out of 4, cells arms / gate (full chain cleared of 10 seeds, arms / gate):

| Trio | none | full | 25 | 50 |
| --- | --- | --- | --- | --- |
| ranger, priest | 3.6 / 4.0 (6 / 10) | 3.6 / 4.0 (6 / 10) | 3.6 / 3.9 (6 / 9) | 3.6 / 3.9 (6 / 9) |
| ranger, brawler | 3.2 / 3.6 (2 / 6) | 3.6 / 4.0 (6 / 10) | 3.5 / 3.9 (5 / 9) | 3.5 / 3.9 (5 / 9) |
| ranger, rogue | 3.3 / 3.6 (3 / 6) | 3.5 / 3.9 (5 / 9) | 3.5 / 3.8 (5 / 8) | 3.5 / 3.9 (5 / 9) |
| ranger, mage | 3.4 / 3.8 (4 / 8) | 3.8 / 4.0 (8 / 10) | 3.7 / 4.0 (7 / 10) | 3.8 / 4.0 (8 / 10) |
| priest, brawler | 2.8 / 3.7 (0 / 9) | 3.1 / 4.0 (1 / 10) | 2.9 / 3.6 (1 / 8) | 3.1 / 3.9 (1 / 9) |
| priest, rogue | 3.2 / 3.8 (2 / 8) | 3.2 / 3.9 (2 / 9) | 3.2 / 3.8 (2 / 8) | 3.2 / 3.8 (2 / 8) |
| priest, mage | 3.4 / 3.7 (6 / 9) | 3.7 / 4.0 (7 / 10) | 3.4 / 3.7 (6 / 9) | 3.4 / 3.7 (6 / 9) |
| brawler, rogue | 2.8 / 3.4 (0 / 6) | 3.4 / 3.9 (4 / 9) | 3.4 / 3.9 (4 / 9) | 3.3 / 3.8 (3 / 8) |
| brawler, mage | 2.8 / 3.0 (0 / 2) | 3.9 / 4.0 (9 / 10) | 3.2 / 3.4 (2 / 4) | 3.5 / 3.6 (5 / 6) |
| rogue, mage | 3.0 / 3.4 (0 / 4) | 3.7 / 4.0 (7 / 10) | 3.4 / 3.7 (4 / 7) | 3.7 / 3.9 (7 / 9) |
| mean | 3.15 / 3.60 (23 / 68 of 100) | 3.55 / 3.97 (55 / 97 of 100) | 3.38 / 3.77 (42 / 81 of 100) | 3.46 / 3.84 (48 / 86 of 100) |

Mean over trios and both steels: none 3.38 (v3 3.25), full 3.76 (3.76), 25 3.58 (3.51), 50 3.65 (3.61).

Mean party HP% after each battle (all 10 trios, runs that reached the battle; later values are survivor-biased):

| Heal | B1 | B2 | B3 | B4 arms | B4 gate |
| --- | --- | --- | --- | --- | --- |
| none | 89 | 74 | 69 | 9 | 38 |
| full | 89 | 83 | 92 | 25 | 73 |
| 25 | 89 | 82 | 85 | 20 | 61 |
| 50 | 89 | 82 | 89 | 22 | 66 |

Falls (runs, of 200 per mode): at B4 arms / gate none 73 / 28, full 45 / 3, 25 56 / 17, 50 51 / 13; B2 falls 8 / 0 / 4 / 2 (same few seeds counted per steel); no B1 or B3 falls in any mode.

v4 against v3:

- Slightly easier, not a change of picture: B3 is lighter (party HP after it 69 vs 61 with no heal; `wagon_raid` now ends in about 2 rounds; cause not probed) and the stands are cleared a little more often (arms 23 vs 18 and gate 68 vs 51 of 100 with no heal). With full heal nothing moves (55 / 97 vs 51 / 98).
- The conclusions of v3 stand: B4 is the fall point everywhere, the arms stand is lost in about half of full-heal runs (45 of 100 chains), the gate stand is almost always cleared with heal; with no heal the party enters B4 at 69% and loses it (arms 23 of 100 cleared).
- Heal 25 / 50 close part of the gap: arms 42 / 48, gate 81 / 86 of 100. Downed heroes stay down in 25 / 50, as in v3.
- Weakest trios: priest+brawler (arms 0 of 10 with no heal, 1 with full heal; 3.1 battles) and brawler+mage / brawler+rogue with no heal; best: ranger+mage and brawler+mage with full heal (9 of 10). Brawler+mage depends on heal more than any other trio (2.8 / 3.0 none to 3.9 / 4.0 full).

Method: `tools/fc.sh sim party 5 10 comp=<comp> enc=world:<id>,...` on the same branch, level 5, authored foes, 10 seeds, 0 PLAN FAIL, 0 METRIC FAIL (30 fights per run). Targets: BAL2 role answers (2): mob band >= 90 (solo), line band 70-90 (pass_ambush).

| Fight | Comp | Clear % | Rounds | Verdict |
| --- | --- | --- | --- | --- |
| courtyard_skirmish (hold, 2 raiders) | warrior | 100 | 2.8 | in band (mob >= 90) |
| ash_scouts (2) | warrior | 20 | 4.5 | far below mob band |
| ash_straggler (1) | warrior | 100 | 1.9 | in band |
| pass_ambush (line band 70-90) | warrior + ranger | 100 | 4.6 | above band by 10 |
| pass_ambush | warrior + priest | 30 | 5.0 | below band by 40 |
| pass_ambush | warrior + brawler | 100 | 3.7 | above band by 10 |
| pass_ambush | warrior + rogue | 70 | 3.7 | at band floor |
| pass_ambush | warrior + mage | 70 | 4.0 | at band floor |
| ash_raiders (3) | ranger / priest / brawler / rogue / mage | 90 / 100 / 100 / 100 / 80 | 3.1 / 4.4 / 2.7 / 3.2 / 3.5 | all >= 80 |
| ash_patrol (2) | all five | 100 each | 2.2-4.0 | all clear |

Interpretation: the Courtyard and the straggler hold, but the solo warrior loses `ash_scouts` four times in five (hp score 0.37; cause not probed) and the Cinder Gate ambush is a coin flip for the priest duo (30) while the other four duos are 70-100; `ash_raiders` (3 foes) and `ash_patrol` are soft for every duo. Per the spec the asset fix for `ash_scouts` is one enemy fewer, for the priest duo the cause was not probed. Not tuned (DR1 (3)).

## OBJ1-B objectives 2026-10-18

Same method; trios are warrior plus 2 recruits (10 comps), 10 seeds each. Objectives: `courtyard_skirmish` hold 3 rounds or rout, standard deployment; `wagon_raid` rout, ambush deployment, guest Wenn (survivors read 4.0/3 because the guest counts); both stands `slay` (Grakka) with phases (1 and 2).

| Fight | Band | Result (clear %) | Verdict |
| --- | --- | --- | --- |
| courtyard_skirmish, solo warrior | mob >= 90 | 100 | in band |
| wagon_raid, 10 trios | line 70-90 | 100 for all 10 (rounds 1.7-2.1, party hp 0.88-0.95) | above band by 10+ |
| ashmaw_stand, 10 trios | boss 30-50 | mean 60 (range 30-100) | above band by 10 |
| ashmaw_stand_barred, 10 trios | boss 30-50 | mean 97 (range 90-100) | far above band |

Interpretation: `wagon_raid` is too easy for every trio (the ambush layout and the guest do not make it harder; cause not probed); the stand is close to the band (the softest trios are priest+rogue 30, priest+brawler 40, 50 for brawler+rogue, mage+priest, mage+rogue; the strongest brawler+ranger 100, brawler+mage and priest+ranger 80), and the barred stand is almost a free win as designed. No tuning (DR1 (3)); the spec's fix for a too-hard wagon_raid does not apply since it is too easy.

## BOSS1-B boss phases 2026-10-18

The sim cannot switch phases off without a code or data change, and editing data was not allowed, so "before" uses the CAMP1-B section (archive; four-hero default comp, phases absent, both stands 100) and the BAL4-ATTR v3 B4 numbers; "after" is the OBJ1-B table above (phases on, 10 trios of three).

| Stand | Before (CAMP1-B, 4 heroes; v3 B4) | After (10 trios, phases on) | Boss band 30-50 |
| --- | --- | --- | --- |
| ashmaw_stand | 100 clear, hp 0.63 (v3 arms B4: 18 of 100 chains cleared with no heal) | 60 clear, party hp 0.12-0.49, rounds 4.8-7.2 | 10 above |
| ashmaw_stand_barred | 100 clear, hp 0.93 | 97 clear, party hp 0.54-0.86, rounds 3.8-5.6 | 47 above |

Interpretation: the stand moves toward the band (the before numbers used a 4-hero party and are not a like-for-like comparison, so this is not a measured effect of the phase); the barred stand stays a near-certain win even with two phases, so its phases do not make it a boss fight. Not tuned.

## Issues for the brainstorm agent (this run)

1. `ash_scouts` solo warrior clears only 20 (target >= 90): one enemy fewer or a weaker foe is the spec lever.
2. `pass_ambush` with the priest duo 30 versus 100 for ranger / brawler: a damage-poor pair is lost at the line band; check whether a recruit's class should lower foe count.
3. `wagon_raid` 100 for every trio against a 70-90 band; `ashmaw_stand_barred` 97 against 30-50.
4. Attrition: B4 stays the only real threat in the chain (see v4); a campaign-mode flag to switch phases off would make BOSS1-B a true before/after (sim change, not requested).
5. Party size: campaign is now three heroes; the BAL3 party bands were set for four.

Older runs (BAL3-R final rerun, BAL3-R0, CAMP1-B and earlier): `docs/archive/BALANCE_REPORT_2026-10.md` (BAL3-R archived 2026-10-18).
