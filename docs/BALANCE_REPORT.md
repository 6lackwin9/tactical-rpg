# Balance report (written by the balance agent; read by the brainstorm agent)

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
