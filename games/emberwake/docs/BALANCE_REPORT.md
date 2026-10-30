# Balance report (written by the balance agent; read by the brainstorm agent)

## RULESET1-A3 2026-10-31 (final confirmation after the A3 data; win-rate tuning stopped)

This final run supersedes the round-1 numbers quoted in decision "Timeline test fights after RULESET1-A3": tl_skirmish lands inside its band (69/66 %) and only its capped fights (21/25 %) stay out (decision "RULESET1-A3 loose ends": recorded as known); tl_flank reads 92-98 %; the reach duel 55/48 % at 100 seeds. tl_flank_elite (100 %, won in round 2) has no band and is left as a diagnostic.

Measured build: `C:/dev/tactical-rpg-lane1`, branch `claude/ruleset1a3`, with the uncommitted A3 data (seed `games/emberwake/data/seeds/ruleset1a3.json`). No data, tuning or code changed by the balance run. Jitter on, preset `balanced`, seed i = 11 + 12 i, 0 PLAN FAIL and 0 PLAN SKIP. Commands: `tools/fc.sh sim feel party=both seeds=100` (the five tl_ fights), the same with `enc=tl_flank_elite`, and `battle kit_culling seed=N ruleset=timeline log` for seeds 1-5. Scratch outputs: `ruleset1a3_fin_*.txt` and the round-1 probes `ruleset1a3_bal_*.txt` in the session scratchpad.

Decision "Timeline test fights after RULESET1-A3": win-rate tuning of the tl_ fights stops; they are judged on the mechanic floors; the sure-break floor moves from the reach duel to tl_skirmish; armour, flank and skirmish are known over band and are judged later on CAST1's real encounters.

### A3 changes measured

Crushing Judgment `power_mult` 1.5 to 1.8; `tl_footman`, `tl_footman_b`, `tl_crossbowman` to level 2 with CONS (HP +19-20%); `tl_skirmish` `max_rounds` 10; `tl_footman` assault order back to [tl_snap_jab, tl_hook_thrust] (active [tl_snap_jab, tl_hook_thrust, tl_brace_counter]); `kit_culling` gains a goblin_warlord.

### Final table (100 seeds, won %; capped in brackets)

| Fight | Band | A3 auto | A3 timed | B2 auto / timed | Verdict |
| --- | --- | --- | --- | --- | --- |
| tl_duel_windup | 30-70 timed | 5 | 31 | 5 / 31 | timed on the lower edge (about +-9), unchanged |
| tl_duel_reach | 30-70 | 55 | 48 | 70 / 62 | inside |
| tl_duel_armour | 15-45 | 89 | 89 | 100 / 100 | over by 44 (known) |
| tl_flank | 50-85 | 92 [8] | 98 | 98 / 100 | over by 7-13 (known) |
| tl_skirmish | 40-70 | 69 [21] | 66 [25] | 70 / 72 | inside; the cap rate is the miss |
| tl_flank_elite | none set | 100, median 2 | 100, median 2 | 100 / 100 | no band, unchanged |
| TOTAL (five fights) | | 62.0 | 66.4 | 68.6 / 73 | |

### Floors under the moved floors

| Floor | Result | Met |
| --- | --- | --- |
| Carried moves > 0 in tl_flank | 222 auto / 224 timed | yes |
| Carried moves > 0 in tl_skirmish | 478 / 502 | yes |
| Poise breaks > 0 in tl_duel_windup (timed) | 83 poise breaks, 114 chance breaks timed; auto 0 poise breaks | yes (timed) |
| Sure breaks > 0 in tl_skirmish (moved floor) | 132 auto / 101 timed | yes |
| Sure breaks in tl_duel_reach (old floor, retired) | 0 / 0 | not required any more |
| Capped fights at most 10% | skirmish 21% auto / 25% timed, flank 8% auto; others 0 | NO, tl_skirmish (and tl_flank auto at 8, inside) |
| Enemy rounds without an action of their intent under 15% | 0% in every fight, both drivers (party side: 159-166 no-active rounds, not part of the floor) | yes |

### Cliff finding (Crushing Judgment)

`tl_duel_armour` (Spiker v Captain) is a threshold, not a slope. 40 seeds auto: `power_mult` 1.8 / 1.9 / 2.0 / 2.03 / 2.06 all 87.5%; 2.1 and 2.2 7.5%; 2.6 2.5%; 3.0 0%. No `power_mult` gives the 15-45 band (the step lies between 2.06 and 2.1, a kill threshold on the Spiker). The Captain's armour is irrelevant here (`armor=tl_captain:60/70/80` all 87.5%): the skill is unbreakable and pierces. Pierce is 83% of hits and the fight is over in round 2. A smooth middle would need the Spiker's HP or the kill threshold to move together with the power (not probed).

### Reach duel orders (40 seeds, before the decision)

Footman assault orders: [snap_jab, hook_thrust] 65% both drivers, sure breaks 0 (the shipped order; 55 / 48% at 100 seeds); [thrust, hook_thrust] and [hook_thrust, snap_jab] 100% with 47-70 sure breaks; hook first with CONS 10 or 20 stays 100%. Hook Thrust early gives the Halberdier's sure break (Jaw Breaker) a wind-up to land on every round; Snap Jab first leaves none. That is why the sure-break floor moved to the skirmish.

### kit_culling (timeline, seeds 1-5, goblin_warlord added)

All five seeds ended "won". The first cull (cull_1, trigger round 3) fires in round 3 in all five seeds: the dojo Warrior is at 0 HP in the round-3 status (in seeds 1, 2 and 5 the Warlord had already taken it to 40-104 HP in round 2). The second cull (cull_2, round 5) fires only when the fight lasts that long: seeds 2 and 5 (5 rounds) lose the Ranger 2 in round 5; seeds 1, 3 and 4 (4 rounds) are won before it. The phase text lines are not printed in the log, so the rounds come from the unit status blocks. The Warlord reaches the captives and strikes Warrior 2 for 60 twice in round 2, so the first captive is lost to the Warlord's hits before the cull in most seeds; the cull only finishes it.

### Known over band (decision reason: tl_ fights are mechanic tests, not tuned encounters)

1. tl_duel_armour 89% against 15-45: the cliff above; judged later on CAST1's real encounters.
2. tl_flank 92-98% against 50-85: no single rule carries it (see B2 switches); judged later.
3. tl_skirmish cap rate 21-25% against 10%: stalemates against guarding enemies (0-10 lost); judged later.
4. tl_duel_windup auto 5% (timed 31% is the band's only driver): the Enforcer beats the auto driver, which does not read tells.
5. tl_flank_elite 100% in round 2, no band: looks unintended, flagged.

### Issues for the brainstorm agent

1. Decide whether the capped-fights floor (10%) stays for tl_skirmish or moves with the win-rate bands to CAST1.
2. The cull log prints no phase line; a "[Phase]" line would make the cull round readable.

## RULESET1-B2 2026-10-31 (timeline feel rerun against the bands and floors)

Measured build: `staging` 013cd955 (RULESET1-AI2 on top of BX, G2 and A2), checkout `C:/dev/tactical-rpg-lane1` (branch `claude/ruleset1b2`), no code, data or tuning changed. Jitter is ON (the feel default, decision "RULESET1-G2 answers"); one `jitter=off` run at 40 seeds is the comparison. Preset `balanced`, tactic default, seed i = 11 + 12 i. 0 PLAN FAIL and 0 PLAN SKIP in every run. Scratch outputs: `ruleset1b2_*.txt` in the session scratchpad (`both100`, `elite100`, `jitteroff40`, `noflank40`, `nopierce40`, `nopoise40`, `armor25|35|50`, `tune0.25`, `tune0`).

Commands (all `tools/fc.sh sim feel ... party=both`, which prints an auto block and a timed block): `seeds=100` (the five tl_ fights); `seeds=100 enc=tl_flank_elite`; with `enc=tl_duel_windup,tl_duel_reach,tl_duel_armour,tl_flank,tl_skirmish,tl_flank_elite`: `seeds=40 jitter=off`, `seeds=40 rules=-flank`, `seeds=40 rules=-pierce`, `seeds=40 rules=-poise`; `seeds=40 enc=tl_flank rules=-pierce armor=tl_footman:25` (also `:35`, `:50`); `seeds=40 enc=tl_duel_windup,tl_skirmish tune=break_chance_scale:0.25` (also `:0`).

### Headline (100 seeds, jitter on; won % [lost / capped]); bands from Q1

| Encounter | Band | Auto | Timed | B baseline (auto, 100 seeds) | Verdict |
| --- | --- | --- | --- | --- | --- |
| tl_duel_windup (Vanguard v Enforcer) | 30-70 timed | 5% [95 / 0] | 31% [69 / 0] | 8% | timed inside, 1 point above the lower edge (100 seeds: about +-9); auto far under |
| tl_duel_reach (Halberdier v Footman) | 30-70 | 70% [30 / 0] | 62% [38 / 0] | 57% | inside (auto on the edge) |
| tl_duel_armour (Spiker v Captain) | 15-45 | 100% [0 / 0] | 100% [0 / 0] | 2% | OVER by 55 points, flipped from unwinnable to unlosable |
| tl_flank (Vanguard + Spiker v Footman + Crossbowman) | 50-85 | 98% [1 / 1] | 100% [0 / 0] | 82% | OVER by 13-15 points |
| tl_skirmish (4 v 4) | 40-70 | 70% [0 / 30] | 72% [3 / 25] | 67% | auto on the edge, timed 2 over; the real miss is the cap |
| tl_flank_elite (Vanguard + Spiker v Enforcer) | none set | 100% [0 / 0], median 2 | 100% [0 / 0], median 2 | n/a | no band; two heroes kill an elite in round 2 |
| TOTAL (five fights) | | 68.6% | 73% | 43% | |

Jitter off (40 seeds, same fights, auto / timed): windup 10 / 27.5 (jitter on 5 / 31), reach 80 / 70 (70 / 62), armour 100 / 100, flank 100 / 100 (98 / 100), skirmish 67.5 / 85 (70 / 72), elite 100 / 100. Jitter on lowers reach by 8-10 points and the timed skirmish by 13; no verdict changes bracket. Windup timed at 27.5% (40 seeds, jitter off) is inside the sampling error (+-14) of the band edge: treat it as "at the lower edge", not "met".

### Floors

| Floor | Result | Met |
| --- | --- | --- |
| Carried moves > 0 in tl_flank | 190 auto / 180 timed (party Maneuver share 10.5%) | yes (baseline 0) |
| Carried moves > 0 in tl_skirmish | 479 / 470 (Maneuver 7%) | yes (baseline 0) |
| Poise breaks > 0 in tl_duel_windup (timed) | 83 timed (chance breaks 114); auto 0 poise breaks | yes with the timed player (baseline 0) |
| Sure breaks > 0 in tl_duel_reach (timed) | 0 (auto 0; poise breaks 104, chance 87) | NO (baseline 0) |
| Capped fights <= 10% in every encounter | windup 0, reach 0, armour 0, flank 1 auto / 0 timed, elite 0, skirmish 30% auto / 25% timed | NO, tl_skirmish only (baseline not printed; B's mean rounds 5.8 hinted at a long tail) |
| Enemy rounds with no usable action of their intent < 15% | 0.0% in every encounter, both drivers (baseline 29.8%) | yes (party side: flank 5%, skirmish 3.5%) |

### Mechanic switches (40 seeds, auto / timed won %)

| Run | windup | reach | armour | flank | skirmish | elite |
| --- | --- | --- | --- | --- | --- | --- |
| all rules (100 seeds, jitter on) | 5 / 31 | 70 / 62 | 100 / 100 | 98 / 100 | 70 / 72 | 100 / 100 |
| `rules=-flank` | 10 / 30 | 75 / 72.5 | 100 / 100 | 95 / 100 | 55 / 72.5 | 100 / 100 |
| `rules=-pierce` | 10 / 30 | 75 / 72.5 | 0 / 0 | 97.5 / 100 | 62.5 / 57.5 | 100 / 100 |
| `rules=-poise` | 7.5 / 15 | 100 / 100 | 100 / 100 | 100 / 100 | 80 / 80 | 100 / 100 |

Readings: pierce is the armour duel (0% without it, as in B); poise is what separates a hero from a standard (reach 100% without it); flank moves only the auto skirmish (by 15 points) and does nothing to the elite fight, although 32-33% of its hits are flank hits.

Pierce at 25 / 35 / 50 (tl_flank, `rules=-pierce`, footman armour set by `armor=`): auto 97.5 / 92.5 / 90%, timed 100 / 97.5 / 97.5%; capped fights 1, 3, 3 of 40. Armour moves the result under 8 points even without pierce: tl_flank is decided by reach and flank play, not by armour.

`tune=break_chance_scale` (40 seeds, auto / timed): 0.25 gives windup 12.5 / 12.5 and skirmish 77.5 / 85; 0 gives windup 7.5 / 15 and skirmish 80 / 80 (default 0.5: windup 5 / 31, skirmish 70 / 72). Timed windup reaches its band only WITH the full roll: poise breaks stay at 83 in 100 fights, the 114 chance breaks carry it.

### Comparison with RULESET1-B

- Improved: carried moves 0 to 190 / 479; enemy no-active rounds 29.8% to 0%; windup duel 8% to 31% (timed); reach 57% to 62-70%; all but two floors met.
- Overshot: tl_duel_armour 2% to 100% (Crushing Judgment `power_mult` is now 1.5; the Captain no longer wins in round 2, pierce share 84%); tl_flank 82% to 98-100% (the Spiker now flanks: 24-26% flank hits, carried moves 180-190); skirmish 67% to 70-72% with a 25-30% cap rate.
- Unchanged: sure breaks 0 in the reach duel; the auto driver in the windup duel (5%) cannot beat the Enforcer, so that band holds only for the timed player.

### What still misses, ranked

1. **tl_duel_armour 100% against 15-45.** Evidence: 100 of 100 won in round 2, both drivers, jitter on or off; `rules=-pierce` gives 0%, so the fight is the pierce multiplier. Lever (data): `tl_crushing_judgment` `power_mult` 1.5 up towards 2.0 (2.2 gave 2% in B), the Captain's armour above 50, or `tl_marrow_spike` power 0.9 down. Retest at 10 seeds x 10.
2. **tl_flank 98-100% against 50-85.** Evidence: `-flank` 95 / 100, `-pierce` 97.5 / 100, `-poise` 100: no single rule carries it, so it is the sum; armour 25-50 without pierce stays at 90-100. Lever (data): `tl_footman` / `tl_crossbowman` HP or damage up (the standards deal 12-33 against 210 HP heroes), or Marrow Spike power; a tier multiplier on the standard is the blunt option.
3. **tl_skirmish capped 25-30% against at most 10%.** Evidence: 0 lost, 30 capped auto; party Rest blocks 3753; enemy Guard 18% with 1099 guard blocks; 133 lost attacks. Stalemates against guarding enemies, not losses. Levers (data or tuning): more party damage on the break lines, `defend_below` 0.4 lower on the tl_ units, or `max_rounds` 8 to 10 (changes what "capped" means; design choice).
4. **Sure breaks 0 in tl_duel_reach (floor).** Evidence: `tl_jaw_breaker` (1 block, sure break, stagger 15) is on the Halberdier's deck, yet 0 sure breaks in 200 fights while the skirmish makes 82-107; the Footman's skills (Snap Jab 1 block, Thrust, Brace Counter) leave no windup for a 1-block sure break to land in. Not probed per block (no `battle ... log` run); a probe `aegis battle tl_duel_reach seed=N log` would confirm. Lever (data): a longer-windup skill on the Footman for this duel, or move the floor to the skirmish.
5. **tl_duel_windup timed 31% (lower edge), auto 5%.** Evidence: 83 poise and 114 chance breaks timed; 15% at roll scale 0. Held-back levers L1 (`tl_overhead_smash` `windup_poise` 20 to 10) and L2 (`tl_shoulder_check` `stagger` 10 to 12) would add margin but also push the skirmish and elite fights further over; apply only after a 10 x 100 retest shows the edge is real.
6. **tl_flank_elite has no band: 100% in round 2.** Two heroes kill an elite with 66-69% pierce hits and 17-21 breaks per 100 fights; the Enforcer sits in Guard 46-47% of rounds (190 guard blocks) and is not broken. Flank on a heavy target changes nothing (`-flank` 100 / 100). Looks unintended; flagged, not decided.

### Issues for the brainstorm agent

1. Armour duel and tl_flank are overshot (100%); choose levers 1 and 2 above and retest at 100 seeds.
2. The skirmish cap rate (25-30%) is the only capped-fight miss.
3. The reach-duel sure-break floor: probe it, then fix the data or move the floor.
4. Windup timed sits at the 30% edge; hold L1 and L2 until a larger retest.
5. Give tl_flank_elite a band or drop it from the report.

## RULESET1-B 2026-10-30 (timeline ruleset feel sim)

Measured build: `origin/develop` bbd1733f (detached checkout `C:/dev/tactical-rpg-ruleset1b`), no code, data or tuning changed. Ruleset `timeline` (rules round_intent, enemy_moves, deck, poise, ap_cost, pierce, reach, flank, plan_badges; `break_chance_scale` 0.5, cap 50; stagger windup x2, guard x0.5, repeat x0.5, flank x1.5; Rest 0.5 AP and 5 poise per landed block, cap 2 AP). Raw output files are `ruleset1b_*.txt` in that checkout root (not committed).

Commands (all `tools/fc.sh sim feel`, seed i = 11 + 12 i, preset `balanced`): `seeds=100` (the headline, 500 fights, `ruleset1b_feel100_full.txt`); `seeds=40` (`ruleset1b_feel40.txt`); `seeds=40 jitter=off`; `seeds=40 rules=-pierce`, `rules=-flank`, `rules=-poise`, `rules=-reach`, `rules=-reach,-flank` (`ruleset1b_feel_rules_*.txt`); `seeds=40 preset=aggressive|defensive|withdraw`. Probes: `cargo run -q -p emberwake_host -- battle <enc> seed=N log` (the engine's own auto driver), 40 seeds per encounter (`ruleset1b_battles40.txt`). 0 PLAN FAIL and 0 PLAN SKIP in every run.

Limits: the spec asks for pierce at 25/35/50. Armour is a unit field (footman 25, enforcer 35, captain 50) and `sim feel` has no armour key, so a clean A/B at one armour needs a data change; not made. What the existing data gives instead: pierce on/off per encounter (below) and the formula. `rules=-poise/-pierce/-flank/-reach` are the only switches (round_intent is an option and cannot be switched off one by one). `sim feel` does not print the cap-hit count, only median rounds; the mean rounds below are unit-rounds divided by fights.

### Headline (100 seeds, 500 fights)

| Encounter | Won | Median rounds | Mean rounds | Breaks (poise / chance / sure) | Pierce share | Flank share |
| --- | --- | --- | --- | --- | --- | --- |
| tl_duel_windup (Vanguard v Enforcer, elite) | 8% | 8 | 6.6 | 59 (0 / 59 / 0) | 0% | 0% |
| tl_duel_reach (Halberdier v Footman, standard) | 57% | 4 | 4.9 | 179 (110 / 69 / 0) | 0% | 0% |
| tl_duel_armour (Spiker v Captain, boss) | 2% | 2 | 2.1 | 0 | 82% | 0% |
| tl_flank (Vanguard + Spiker v Footman + Crossbowman) | 82% | 3 | 3.6 | 178 (84 / 94 / 0) | 38% | 25% |
| tl_skirmish (4 v 4) | 67% | 7 | 5.8 | 520 (297 / 144 / 79) | 27% | 19% |
| TOTAL | 43% | 4 | | 936 (491 / 366 / 79) | 29% | 13% |

The 40-seed runs agree within a few points (windup 12.5%, reach 62.5%, armour 5%, flank 77.5%, skirmish 67.5%). Windup's median moves from 6 to 8 between 40 and 100 seeds, so its rounds are bimodal (an early loss or the cap).

### The spec list, item by item

| Item | Number | Reading |
| --- | --- | --- |
| Stagger and poise factors | Windup hits 1948, broken 709 (36%); 936 breaks in all: poise 491, chance 366, sure 79. Windup breaks per windup hit: windup duel 21%, reach 35%, flank 34%, skirmish 43%. `rules=-poise` (40 seeds): total 45% to 60%, reach 62.5% to 100%, flank 77.5% to 95%, skirmish 67.5% to 90%, windup 12.5% to 10% | Poise is the party's lever against standards; the Enforcer duel does not care (issue 2). Lock rate (broken two rounds running) is 1.7% (reach 3.1%, others 0-2%), so the repeat x0.5 factor (P10) is rarely exercised |
| Chance | 1605 rolls, 366 successes (22.8%); 21-25% per encounter. The cap 50 never binds on these stagger values (`floor(100 x taken / poise_before x 0.5)`) | 37% of all breaks are the roll. Windup duel: 282 rolls, 59 breaks, 0 poise breaks in 100 fights, so the Enforcer is only ever broken by chance |
| Sure break | 79, all in tl_skirmish; 0 in tl_duel_reach, the encounter meant to show it | The Jaw-Breaker needs a windup to land in; the Footman's 1-block snap and 2-block thrust seem to give no window the Halberdier reaches, the Enforcer's 3-block smash in the skirmish does. Hypothesis from where the 79 occur, not probed per block |
| Pierce at 25 / 35 / 50 | Not runnable without a data change. Formula `r / (r + 50)`: a plain hit keeps 67% / 59% / 50%, so pierce gives +50% / +69% / +100% damage (RULESET1 A3 says "gains 33-100%", which is the armour cut, not the gain). Pierce share of hits: flank 38%, skirmish 27%, armour 82%. Pierce off (`rules=-pierce`, 40 seeds): armour 5% to 0%, flank 77.5% to 70%, skirmish 67.5% to 50%, total 45% to 39% | Pierce matters most where armour 25-35 meets four units. Open: Marrow Spike (power 0.9, pierced) deals 15 against the Captain but 20 against the Footman in the same driver, which a full pierce should not do; unexplained (a guard or tier term on the Captain is the guess) |
| Rest gain | 5026 party Rest blocks gave 1998 AP: 0.40 AP per block (windup 0.48, reach 0.39, flank 0.35), 0.45 AP per unit-round against the 2 AP cap. The party spends 5.17 AP a round of a 10 AP pool (4 regen) | Real but small; the party leaves about 1.1 Rest blocks per unit-round (19% of its 6 blocks) idle. AP limits nothing (armour duel: 5.45 AP a round) |
| Flanks | 1100 flank hits of 8504 (13%): flank 25%, skirmish 19%. `rules=-flank` (40 seeds): flank 77.5% to 80%, skirmish 67.5% to 67.5%, total 45% to 45.5% | The rule fires but moves no result; `-reach,-flank` also shows 0-3 points |
| Carried moves | 0 of 4392 party unit-rounds | Cause found, below (issue 1) |
| Kiting AP | 32 kite steps for 38 AP in 100 skirmishes (the Archer: "kite: threatened" 28, "in melee" 4); none elsewhere (tl_flank has no archer on the party side) | About one step in three skirmishes at 1.2 AP a step; it decides nothing |
| Intent variety (OI1) | Rounds whose intent had no active: party 274 of 4392 (6.2%): flank 14.3%, skirmish 6.9%, armour 5.8%; enemy 1043 of 3499 (29.8%): windup 45%, reach 45%, flank 12%, skirmish 28%. Party intent share: assault 70.6%, guard 22.2%, support 7.3%, maneuver 0% | Enemies are the thin ones: their decks hold assaults only, but AI row 3 puts them in Guard at 40% HP (a basic Active Guard covers it, then 1676 Rest blocks). Party gaps are authored: the Spiker and Archer have no Guard active, the Vanguard and Halberdier no Support one |
| Lock rate | 1.7% overall | See the stagger row |
| Lost attacks | 63, all in tl_skirmish | Attacks cancelled by a break, 0.6 per skirmish |

### What looks off against RULESET1's targets

RULESET1 states few numbers: "Balance sets the factor" (5.6); a Snap of 12 in windup (24) should break light units while heavies need a second hit, the chance roll or `sure_break`; armour 25/35/50 is the pierce test; OI1 revisits the slot counts after this run. The tier line of the balance brief applies to the rest: standard beats a nothing-special hero 1v1, elite about 50%, boss favoured. Read against them:

- tl_duel_windup 8% is far under "elite about 50%".
- tl_flank 82% is over: two heroes beat two standards, where a standard should beat a hero 1v1.
- tl_duel_reach 57% is a hero win against a standard, mildly over the tier line but inside 40-60.
- tl_duel_armour 2% is a boss duel (boss favoured), but the hero cannot win on any seed (issue 3).
- tl_skirmish 67% is an even-strength match won by the party by a third.
- 5.6's "a Snap breaks light units, heavies need a second hit": a windup Snap never broke the Enforcer or the Captain on stagger alone (0 poise breaks in the windup and armour duels), as designed for heavies; only the 22% chance and the Jaw-Breaker break them.

### Known signals, measured and explained

- **tl_duel_armour is lost in round 2 in every seed.** Engine driver: 40 of 40 lost in round 2; sim: 98 of 100. Cause from the log (seed 11): the Captain lands Crushing Judgment (power x2.2, 4 blocks, unbreakable) once a round for 110 on a 210 HP Spiker, so it dies on the second hit in round 2. The Spiker needs 315 HP of damage at about 15 per spike (80-100 a round, 3.2+ rounds), so it is a 2-round clock against a 3.5-round need. Unbreakable removes the Snap answer; the Spiker has no Guard active (guard intent 5.8%, assault 94%), and the AI does not guard. Pierce off makes it 0%.
- **tl_flank won in round 2 (golden).** Engine driver: 37 of 40 won in round 3 and 3 in round 2; sim median 3, 82%. Seed 11 log: the Spiker kills the Footman (147 HP) by round 2 block 1 with 20-damage pierced spikes while the Vanguard soaks the Crossbowman down to 15 of 210 HP; the Crossbowman dies in round 3. The standards deal too little: a Footman hits for 12-17, the Crossbowman for 17-33, against 210 HP heroes.
- **7 of 15 fights hit the 8-round cap (golden).** In the sim, mean rounds are 6.6 in the windup duel (with early losses at rounds 3-4 that means well over half its fights run to the cap), 4.9 reach, 3.6 flank, 5.8 skirmish, 2.1 armour. The windup duel is a guard stalemate with a weak hero (party Guard 52.8% of rounds, enemy Guard 45.1%, 299 enemy rounds with no active; the hero's 13-damage hits against a 263 HP elite); the skirmish also runs long (engine driver: 2 of 40 "fled" at round 8, 6 wins on round 7). `sim feel` counts a capped fight as not won, without saying so.
- **The AI never picks Maneuver (carried moves 0).** AI table row 4 (`kits/rpg/crates/rpgkit_battle/src/ai.rs:1687`, `engage_pick`) fires only for `Role::Skirmisher`. Every timeline unit is `tl_drill` (AGGRESSOR) or `ranger` (SNIPER) with an empty `ai_role`, so none is a Skirmisher and the Spiker's Maneuver deck (Hamstring Cut, Slip Strike, Flank Cut) is never used. Not a bug: no Skirmisher is authored. The Disengage trigger is gated the same way (`ai.rs:152`).

### Issues for the design lane

1. **Maneuver is unreachable by the AI** (carried moves 0, Maneuver share 0%): the tl_ roster needs an authored Skirmisher (or another trigger) before Engage, carried moves, Hamstring Cut and Disengage (MV4-MV9) can be judged. Evidence: `ai.rs:1687` gates row 4 on `Role::Skirmisher`; `tl_drill` is AGGRESSOR.
2. **The Enforcer duel (tl_duel_windup) does not teach the windup**: 8% wins, the Enforcer is never broken by stagger (0 poise breaks in 100 fights, 59 chance breaks from 282 rolls). The hero hits for 13 against a 263 HP target at armour 35 while the Enforcer deals 29-56 twice a round. Levers: tier multipliers (HP, power), Vanguard damage and Heavy Cleave use (the log shows 13-damage Snaps, rarely the Cleave), stagger values, `windup_poise` 20.
3. **tl_duel_armour has no winning line** (2%, lost in round 2 in 98-100%): the Captain's one unbreakable 110-damage hit a round is a 2-round clock. Levers: the Captain's power, hero HP, a Guard active for the Spiker; and check that Marrow Spike ignores armour (15 against the Captain, 20 against the Footman).
4. **The standards are too weak in tl_flank** (82% in the sim, 40 of 40 in the engine driver). Levers: tier multipliers, enemy skill power.
5. **Enemy turns are thin**: enemy rounds with no active of their intent are 29.8% (45% in the duels); Guard is picked at 40% HP and then rests or uses a basic; enemy Rest blocks 1676. OI1 reads as an enemy-deck problem more than a party one.
6. **Break chance is a third of all breaks** (366 of 936, 22.8% per roll): review the scale (0.5) against the stagger values; it is the only break source in the Enforcer duel. Lock rate 1.7% means P10 is almost never seen.
7. **Flank and reach rules move no win rate** (flank on or off: 0-2.5 points; reach on or off: 0-5 points, in reach and skirmish only) although flank is 13-25% of hits. Probably intended as feel rules, but the x1.5 stagger and flank power then buy nothing measurable; test with the elite or boss as the flanked unit.
8. **The sim and the engine driver disagree**: windup 8-12% (sim) against 0 of 40 (driver), flank 82% against 40 of 40, skirmish 67% against 32% (13 of 40). `preset=aggressive|defensive|withdraw` changes no number (identical to `balanced`) and `jitter=off` does not close the gap. The headline here is the sim's; whichever driver resembles a player should be the one balanced. Not diagnosed.
9. **RULESET1 A3 wording**: pierce "gains 33-100%"; the damage gain is +50% / +69% / +100% (33% / 41% / 50% is the cut).
10. **`sim feel` gaps**: no capped-fight count, no armour or tuning key (so no pierce 25/35/50 run), and `preset=` is inert. Candidates: an `armor=` override and a cap column.

### Compared with the previous run

No earlier feel run exists (RULESET1-G is the tool's first release); the numbers above are the baseline. The golden's signals (armour lost in round 2, flank won in round 2, 7 of 15 at the cap, Maneuver 0) all reproduce and are explained above.

## REWARD1-B 2026-10-22 (rewards per defeated enemy)

Measured build: `main` 18f3893 (REWARD1-L, REWARD1-A and REWARD1-S merged), clean checkout, no code or data changed. Sim: `tools/fc.sh sim party 5 10 comp=<comp> enc=world:<id>,... rewards=on` (10 seeds, authored foe levels, jitter on) and `tools/fc.sh sim campaign 5 3 heal=rally rest=3 extra=4:ash_raiders rewards=on` (the HEAL1 chain, 3 seeds, all ten trios, 300 fights: 0 PLAN FAIL, 0 METRIC FAIL). Rule coverage: `cargo test -p emberwake_world --test rewards` passes (15 of 15).

Read this first (what `rewards=on` is): the sim's gold is an expected value (mean purse x defeated weight share), with no floor, no `once` purse and no `reward_paid` state; key loot counts a kill in a won fight every time, with no claimed-once rule; the drop column is the expected item value (chance x `value` x count, buy value; the shop pays `sell_mult` 0.5 of it). A campaign or party run that repeats an encounter overcounts every once-only source (story purses, the courtyard objective, the Warchief's Axe). The sim also never models the party fleeing (only enemies flee), so "fled" pay below is derived from the rules and from lost fights, not simulated. The run totals below are my own tally from the sim's per-encounter means plus the authored sources, not a sim output. (Do not pass `elevel=0` to a party run: it re-levels the foes to the party level and doubles their XP; all numbers here use the authored levels.)

### Result against the targets

| Target | Number | Verdict |
| --- | --- | --- |
| Gold per run 550-650 | 568-571 with no mob fights, 603-606 at 6 mob wins, 627-630 at 10, 650-653 at 14, 674-677 at 18 | In band up to about 13 mob wins; no recorded run says how many a player gets (see issue 1) |
| Drops per run 3-5 | 2.1 (no mobs), 3.4 (6 mob wins), 4.2 (10), 5.0 (14), 5.8 (18); the Warchief's Axe adds 1 | In band at 6-14 mob wins; below 3 with 0-3 mob wins; 5.2 at 10 wins if the axe counts |
| Sell value 40-70 | 36 (no mobs), 57 (6 wins), 70 (10), 84 (14), 98 (18); the axe (value 57) would add about 28 if sold | In band at 3-10 mob wins; over at 12+ |
| Potions and elixirs at most 2 | expected 0.6-1.5 per run; chance of 3 or more 5% (6 wins), 9% (10), 13% (14) | Pass on average; 1 run in 11 breaks the cap at 10 wins |
| Lost or fled pays at most half of a win | lost fights 5-36% of a win in gold and XP, drop value up to 45% (table below); a flee after most kills can pay more than half | Pass for losses; exception for flees by rule |
| Lost mob fight pays at most 6 gold | scouts at most 3 (1 of 2 foes), raiders at most 6 (2 of 3), straggler 4 (only if its one foe dies in a lost fight) | Pass |
| XP within one level of win-only at the finale | chain XP per hero 353-383 (new) vs 356-383 (win-only, same wins) | Pass, difference at most 5% (about 0.1 level) |
| Flee loop never pays above `gold_max` on a `once` purse | engine tests pass (`a_once_purse_pays_each_foe_once_per_save`); the sim cannot show it | Pass by test only |

### Gold per full Ashlands run

Authored sources are unchanged from PURSE1 section 2 and re-read from data: start gold 100 (`world.json`), quest gold 180 (templates: reach_and_clear 30 x2, clear_site 40 x2, signal_and_return 20 x2), chests 90 (10, 25, 15, 40), dialogue gifts 35 (Mara 20, Tev 15; a bomb sale for 20 is a trade and left out) = 405. Fight sources are sim means for a trio at level 5 (warrior, ranger, priest), all at full clear unless noted:

| Source | Sim gold per win | Notes |
| --- | --- | --- |
| courtyard_skirmish | 17.5 + 10 (objective, once) | purse 15-20 |
| pass_ambush | 27.5 | purse 25-30; wave units count in the divisor |
| wagon_raid | 27.5 | purse 25-30 |
| ashmaw_stand (arms) | 50.3 (defeated 4.2 of 6) | purse 50-60; the unspawned wave and fled foes cost about 4.7 against the 55 mean |
| ashmaw_stand_barred (gate) | 53.5 (defeated 2.5 of 3) | |
| ash_patrol (2 marker roamers, fought once each) | 15.0 each | purse 12-18, not `once` but each marker roamer is fought once |
| ash_scouts / ash_raiders / ash_straggler | 4.5 / 7.0 / 3.0 | mob mean about 5.9 at a 45/55 scouts to raiders mix |

Story fights plus patrols give 164.5 (arms), so a run without mob fights ends at 568 (arms) or 571 (gate): already inside the band, and each mob win adds 5.9 on average. Mob wins per run are the open variable: the road legs give about 1-2 random fights per run (encounter_pct 15, 25, 30 on the first crossing, half on repeats) plus the scripted first raid; the zone steps (`steps_min`/`steps_max` 10-20 per fight in three zones) decide the rest, so 6-14 is a plausible range and the PURSE1 estimate of 10 (+59 gold) sits in the middle. A mob purse mean of 5.9 is about one camp (6 gold), as PURSE1 intended.

### Drops per run

Data: minions 8% per defeated unit (a potion or elixir is 2% of it), the Horde Warchief (`ambush_berserker`) 20% (potion 6%), Grakka 60% (elixir 15%). Expected drops = 0.08 x minion defeats + 0.20 x 2 (one warchief in each marker patrol) + 0.60 (Grakka). A run holds about 14 minion defeats without mob fights (courtyard 2, pass 4, wagon 3, stand 3.2, patrols 2) and about 2.55 more per mob win. The counts above are expectations; the spread at 10 mob wins is 18% of runs under 3 drops and 23% over 5 (58% in 3-5); at 6 wins 31% under 3 and 11% over 5; at 14 wins 10% under and 38% over. The sim's `drops` column per encounter (value, trio, win): courtyard 4.95, pass 10.70, wagon 7.55, stand 30.27 (arms) or 25.79 (gate), patrol 9.26, scouts 6.00, raiders 7.55, straggler 2.10; sell value is half of these.

A drop sells for 14 on average (item values 15-57, halved). The shipped tables hold weapons and armour at 3% each and a potion or elixir at 2% on minions, so about a quarter of drops are consumables and the rest are gear a trio may already own (arm_leather, value 40, is on four of the five minion tables).

### Flee and loss pay against a win

Lost fights are simulated by weak parties (rows with 0% clear are all losses). Pay as a share of the same encounter's win by a trio:

| Encounter | Win (gold / XP per hero / drop value) | Lost fight | Share of a win |
| --- | --- | --- | --- |
| pass_ambush | 27.5 / 36 / 10.70 | solo warrior, 0% clear: 8.3 / 10.8 / 3.04 | 30% / 30% / 28% |
| ashmaw_stand (arms) | 50.3 / 259 / 30.27 | warrior+mage, 0% clear: 6.8 / 23.4 / 7.23 | 14% / 9% / 24% |
| ashmaw_stand (arms) | same | warrior+rogue, 0% clear: 3.7 / 12.6 / 3.83 | 7% / 5% / 13% |
| ash_patrol | 15.0 / 35 / 9.26 | solo warrior, 20% clear, losses derived: about 3.3 / 6.8 / 3.3 | about 22% / 19% / 36% |
| ash_raiders | 7.0 / 27 / 7.55 | solo warrior, 80% clear, losses derived: about 2.3 / 9 / 3.4 (1.0 foe of 3 defeated) | about 33% / 33% / 45% |

All lost fights stay under half of a win for gold and XP; drop value reaches 36-45% (units that surrender or flee still roll the table) and is the closest to the limit. Campaign check: in the 8 of 60 runs where the party fell at the finale (7 arms, 1 gate; all at battle 4) the chain still paid 117-144 XP per hero and 82-90 gold, against 347-383 XP and 124-134 gold when it won (a fall keeps 33-40% of the XP and 65-70% of the gold, because the first three fights are won in both).

Party flight is not in the sim, so the rule gives the bound: a flee pays the defeated weight share of the purse and the kill XP of the defeated, and nothing of key loot or the objective. That is not capped at half. Examples at the current rosters: ash_scouts after 1 of 2 kills 50% (gold 2.3, XP 9), ash_raiders after 2 of 3 kills 67% (gold 4.7, XP 18), ash_patrol after the warchief alone 71% of gold (10.6) and 74% of XP, ashmaw_stand after Grakka alone 76% of gold (about 38 against the 50 win) and 84% of XP (230 of 275), but never the key loot or the courtyard objective. So "at most half" holds for losses and for flights after killing about half the weight, and is exceeded by flights after most of the kills; this follows from the spec's rule (pay what you killed), not from a bug.

### XP pace

New rule against win-only on the same fights (HEAL1 chain, per hero at level 5; XP counts each defeated enemy once, so a win that defeats the whole roster pays the same under both):

| Chain | REWARD1 mean (3 seeds, 10 trios) | Win-only maximum (every roster foe) | Difference |
| --- | --- | --- | --- |
| steel arms | 365 (range 347-383, winning runs) | 383 | minus 5% (waves that never came, fled foes in part of the runs) |
| steel gate | 353 (range 347-356, winning runs) | 356 | minus 1% |

Only runs that fell gain against the old rule (a lost finale pays XP where win-only paid 0 for it). One fight is most of the chain: Grakka is 230 of 275 XP per hero in the stand. Per-win XP at level 5 is courtyard 18, pass 36, wagon 27, patrol 35, scouts 18, raiders 27, straggler 9, stand 244-259; level 5 to 6 needs 220, so quests (380 XP over six) plus the three early fights (81) put a hero near level 6-7 before the finale at 6-14 mob wins, while the campaign sim holds the party at level 5 for all five fights: its finale numbers are conservative for the party level.

### Farm checks

- Once-only sources (story purses, the courtyard objective, key loot): bounded by `reward_paid` and the claimed key loot; 15 engine tests pass, including a once purse paid per roster key per save and key loot needing the kill and the win. The sim cannot test this by design.
- Mob fights (purse not once; XP and drops too): per win 4.5-7.0 gold (about one camp, 6), 18-27 XP per hero at level 5, drop value 6.0-7.55 (sell 3.0-3.8). A mob fight ends with party HP 0.94-0.98 on average (scouts, raiders), so a rest is often not due: a repeated mob win nets about 9 gold-equivalent. Reaching one needs travel or zone steps (10-20 steps per fight; route legs 15-30% on the first crossing, half on repeats). XP falls with party level through the level-delta factor (a level 3 minion is worth 9 at level 5, 7 at level 6, 5 at level 7), so XP grinding self-limits; gold does not. Not a strong exploit, not zero; whether it is acceptable is a design call.
- Flee loop on ash_raiders: not simulated. By the rules the loop pays at most what a win pays (pay follows kills; a mob win adds nothing beyond its kills), so it cannot beat winning per fight, and per round it equals the kill rate. Nothing found that pays above a win.
- Surrender and flight on their own count as defeated (Q8 A): ash_scouts defeats 2.0 of 2.0 with 1.0 surrendered, raiders 1.7 slain and 1.1 surrendered of 3.0, so a morale break pays the full purse and full drop rolls. This matches MORALE1 rule 11 (a break win pays the full rout).

### Comparison with the previous run

First run under REWARD1; no earlier gold, drops or XP measurement exists. The PURSE1 estimates (gold about 600, drops about 4 at about 67 sell) hold: gold 568-571 without mobs and 627 at 10 mob wins, drops 4.2 and sell 70 at 10 mob wins (the top of the 40-70 target). New: the Warchief's Axe is a drop outside that budget.

### Issues for the brainstorm agent

1. Mob wins per run are unmeasured (6-14 moves gold from 603 to 653, drops 3.4 to 5.0, sell 57 to 84). A recorded run (DR2 item 9, approved, not done) or a zone-step model would settle it; above about 13 mob wins gold passes 650, above about 10 sell passes 70. Candidate levers if needed: the mob purse ranges (data), the minion drop chance (8%), `sell_mult`.
2. The Warchief's Axe (sell 28, a fourth drop) is outside the drop budget: with it drops reach 5.2 and sell about 99 at 10 mob wins. Decide whether key loot counts against the 3-5 and 40-70 targets.
3. Drop mix: most drops are gear rows at 3% each, and `arm_leather` (40) is on four of five minion tables; a copy the trio already owns is only a sale. Candidate lever: the table rows per unit.
4. Consumable cap: 1 run in 11 exceeds two potions or elixirs at 10 mob wins (13% at 14); the expected number is 1.1. Raise the cap or lower the 2% rows.
5. Flee pay can exceed half of a win after most kills (up to 84% of the finale XP and 76% of its gold, without the win's key loot or objective). Decide whether "at most half" is a rule or only a lost-fight limit.
6. Mob fights repeat at almost no cost (party HP 0.94-0.98 afterwards; about 9 gold-equivalent and 18-27 XP each at level 5). XP falls off through level delta, gold does not. Decide whether a non-`once` mob purse needs a farm limit or stays an accepted slow income.
7. Sim gaps for a possible follow-up: no party flight (flee pay and flee loops untested), no `once` / claimed state under `rewards=on`, drops in buy value not sell value, `elevel=0` re-levels foes instead of meaning "authored", and the campaign holds the party at level 5 although XP puts the finale at level 6-7.

## Trio bands 2026-10-18 (approved 17:06)

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
