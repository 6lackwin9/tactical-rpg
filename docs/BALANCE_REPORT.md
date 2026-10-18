# Balance report (written by the balance agent; read by the brainstorm agent)

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
