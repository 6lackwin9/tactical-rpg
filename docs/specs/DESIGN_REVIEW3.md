# Design review 3 (2026-10-18)

For Tushar. This is a delta review: it covers `develop` a499b3f..8e2530a, which is 13 playable commits since review 2: ITEMUSE1-A2, HEAL1-L (2 commits), INTENT1-U plus its follow-up, BAL5-SIM/PH, HEAL1-S, BOSS1-U, BREPORT1-L, MAPKEYS1-U, UIR-2, BAL5-AI and ITEMPOW1-A. Sources: the commit messages and diffs, `docs/BALANCE_REPORT.md` (HEAL1-B, BAL5-R1), the frames `boss1-u/after_banner.png`, `uir2/menu_popup_normal.png`, `mapkeys1/after_labels.png` and `mapkeys1/before_labels_user.png` (your screenshot), and four small read-only code probes. `playtests/` still holds only the README. Nothing in the repo was changed. I do not re-propose anything that is already decided or queued (MORALE1, PACE1, PURSE1, COMBAT-UI3, MAPNAV1, HEAL1-U/A/B, DR2-LEVERS, ASHPACE1).

## Summary

The batch works. HP now carries over between battles, enemies show readable tells and no longer skip turns, the boss announces its phases, there is a real game menu, and battle items are worth carrying. Little needs fixing, but this batch makes the current balance numbers unreliable in three ways, and two leftover bugs show up in the shipped game:

1. Every balance number on file was measured before two fixes that change fights. Enemies used to skip turns (fixed in f65f0bb), and the party AI got better (BAL5-AI). Before the fix, enemies sometimes idled, so the fights were easier than they should have been.
2. The HEAL1-B finding "runs end at the filler fight, not the stand" comes from a mislabelled log line.
3. The new potion is cheap and has no stock limit, and no run has measured it together with the HEAL1 healing rules.
4. The battle report credits status effects to the wrong hero.
5. The region map in the real game still shows a debug bar, and its "New game" button wipes your run in one click.

## 1. Combat rules and AI

Working:

- INTENT1-U (8d4971d) and its follow-up (f65f0bb). Review 2 item 8 asked why some enemies read "Arriving", and the answer was a real bug. A packmate's tile claim broke earlier drafts, so those enemies did nothing. Now they re-draft, and an enemy that truly waits says "Holding".
- BAL5-AI (7ceddcb): early taunt with walk-in, the healer heals itself first, and area attacks accept one ally hit when they catch 3 or more foes. Each lever has a counter-case test.
- BOSS1-U (c14582e): a 2 s banner with the phase name and line, which any key skips (`boss1-u/after_banner.png`). It reads well.

Needs improvement (evidence):

- **The balance numbers predate both fixes.** BAL5-R1 ran on c78b8ba and HEAL1-B on f3d8ac2. Neither base contains f65f0bb or 7ceddcb (checked with `git merge-base`). The fix makes enemies stronger and BAL5-AI makes the party stronger, and nobody has measured the net effect. Three queued decisions depend on these numbers:
  - ASHPACE1's "one scout fewer" (solo 20/100, RECRUIT1-B)
  - the HEAL1-B PASS (arms 46, gate 95)
  - the BAL5 triggers that BAL5-L, BAL5-A and DR2-LEVERS wait on

Options:

- A (recommended): widen the BAL5-AI rerun (the run BAL5-L, BAL5-A and DR2-LEVERS already wait on). Besides the L5 trio sweep and role cells, it also repeats the HEAL1-B chain (`campaign 5 10 heal=rally rest=3 extra=4:ash_raiders`) and the solo `ash_scouts` probe. The potion arm in section 3 is added too. One run, so every later decision starts from today's rules. Cost: about 1 h more of Balance time.
- B: rerun only the trio sweep as planned, and let MORALE1-B and PACE1-B re-measure the campaign and the scouts later. Cheaper, but ASHPACE1 and MORALE1-A would tune against the old numbers.
- C: keep the old numbers. ASHPACE1 may remove a scout the fixed enemies no longer need removed, or not enough.

## 2. Battles: pacing and difficulty

Working:

- HEAL1-L (44e0641, 0832b03) and HEAL1-S (47ef3b4): stored HP, a 25% rally after a win, rest places, Make camp, and a defeat cost without injuries. HEAL1-B passes on both steels: arms 23 -> 46 (band 30-55), gate 68 -> 95 (75 or more). This is the "fights carry a cost" goal from DR1, met with one rule set and no unit edits.

Needs improvement (evidence):

- **The fall label is wrong when a filler fight is inserted.** In `tools/sim/sim_campaign.gd:193`, `fell_at` counts story battles, and the extra fight belongs to the battle after it. The label then reads `fights[fell_at - 1]`, and that list includes the extra fight. So with `extra=4:ash_raiders`, a fall at the stand is printed as "battle 4 (ash_raiders)".
- The report already flagged "no fall is labelled with the stand" as not probed. So the HEAL1-B claim that runs end at the filler fight, not at the stand, is unproven. It matters for ASHPACE1 ("fewer fights"): cutting a filler fight helps only if the filler is what kills runs.

Options:

- A (recommended): a small sim fix before the section 1 run, as a HEAL1-S follow-up. Report falls in the filler and in the story fight separately, with a rule-derived check in `bal4_campaign_test_suite`. Logic lane (Balance thread), about 30 min.
- B: Balance works it out by hand from the HP entries per run. No code, but slow and easy to get wrong.
- C: leave it, and ASHPACE1 picks fights to cut without knowing where runs end.

## 3. Items and economy

Working:

- ITEMUSE1-A2 (fda40cc) and ITEMPOW1-A (02c6bf4): no free class potions; the potion heals 90, the bomb does 45, and you start with 100 gold. Review 2 item 2 is done as approved. A chest now gives a potion (your screenshot), so items show up in the first ten minutes.

Needs improvement (evidence):

- **The potion economy has not been measured with HEAL1.** The potion's value is 15 and its price is -1 (value at buy_mult 1.0), so it costs 15 gold, with stock -1 (no limit) at Hest (`shops.json`).
  - Start gold buys 6 potions. A trio carries 3 each, so 9 potions, about 810 HP of healing, into every fight.
  - The sim has item use off by default, so HEAL1-B's arms 46 has no potions in it.
  - PURSE1 (spec proposed) will add fight gold on top. If potions make the arms finale easy, the HP carry-over that HEAL1 just built loses its bite.

Options:

- A (recommended): measure first. Add a potion arm to the section 1 run: the same chain with `pouch=con_heal_potion,con_heal_potion,con_heal_potion`. The AI uses potions, so this is an upper bound. Trigger: if the arms clear goes above 55, raise the potion price from 15 to 30 (one shop row), then rerun. Light touch, per DR1 (3).
- B: limit stock now: 4 potions per shop visit. A direct cap, but it needs a restock rule the shop does not have yet (logic plus data).
- C: leave it. Potions become the player's way to buy down difficulty, and DIFF1 covers difficulty later.

## 4. Battle report (data)

Working: BREPORT1-L (d0219fa) moves the sim's counting into shipped code, so the results screen and the sim cannot disagree. Sim output is byte-identical.

Needs improvement (evidence):

- **Status effects are credited to the wrong hero.** The commit message says: "statuses are credited to the next log entry's unit (also in buffs_applied); fixing it changes sim numbers, sent to Balance".
- No TASKS row and no Balance handover line tracks this.
- BREPORT1-U will show these counts as the Effects column and its tip (BREPORT1 rule 5: "Bleed x2, Slow, Haste on allies x1"). The player would see the priest's Haste listed under the next hero to act.

Options:

- A (recommended): a small logic row before BREPORT1-U fixes the status source in `battle_metrics.gd`. It adds a test where hero A applies a status and hero B acts next. Balance notes that the status-count KPIs shift. No band uses these counts as a pass line, so no tuning follows.
- B: BREPORT1-U ships with the Effects column hidden until the fix lands.
- C: ship it as is.

## 5. Maps and menus

Working:

- UIR-2 (52dda97): Esc opens a game menu over the paused map, with party HP shown (HEAL1 is already visible there), Save and Load, and a proper New Game with a destructive question (`uir2/menu_popup_normal.png`).
- MAPKEYS1-U (126f56f): WASD, held-key walking, a gliding token, and labels that no longer overlap. "Tev Quill" and "Brask Toller" collided in your screenshot and are separate in `after_labels.png`.

Needs improvement (evidence; a real problem, not a taste call):

- **The region map's debug bar is in the real game.** Your own screenshot and the UIR-2 frame show the bar. It holds:
  - a world drop-down
  - a "New game" button
  - the status line "ashlands:warden_keep (20, 15) steps 0 / 16 rolls { }"
- The source is the EXP1 debug scene (`region_map_view.gd:129-135, 937`). `start_game` (line 535) swaps in a debug state with no question asked: DEBUG_SEED and the first two catalog party units, a warrior and a ranger, not Bryn alone. So one stray click ends a run. Now that UIR-2 has a real New Game, the bar does nothing useful.
- The chest note prints a raw dictionary (line 667: `Opened: { "con_heal_potion": 1 } +4 supplies`). MAPLOG1-U's ticker will show it. Both break the preferences "dev-only UI only in test scenes" and "hide engine detail".

Options:

- A (recommended): a small UI/UX row ahead of MAPLOG1-U. Show the world picker, New game and the debug status only in the debug scene, using a flag like the existing `dev_outcomes`. In play, the bar shows the place name and supplies. The chest note names the items ("Found: Healing Draught, 4 supplies").
- B: keep the bar and put UIR-2's destructive question on its New game button. Safe, but the debug text stays.
- C: leave it for your UI review. The one-click wipe stays until then.

## 6. Playtest notes

There are no new recordings. Review 2 item 9 (one recorded run from New Game to the Cinder Gate) is approved but not done yet. The batch above (HP carry-over, potions, the drafting fix) is exactly what that run should feel, so it is worth doing once sections 1-3 have numbers. No new item.

## Approval list

One line per item: approve / skip / pick A, B, C.

1. Re-baseline after the drafting fix and BAL5-AI (section 1): pick A widen the BAL5-AI rerun (trio sweep + HEAL1 chain + solo `ash_scouts` + potion arm) / B trio sweep only / C keep the old numbers. **Recommend A.**
2. Campaign sim fall label (section 2): pick A fix before the run / B read it by hand / C leave. **Recommend A.**
3. Potion economy (section 3): pick A measure in run 1, price 15 -> 30 only if arms > 55 / B stock cap 4 per visit now / C leave to DIFF1. **Recommend A.**
4. Battle report status credit (section 4): pick A fix before BREPORT1-U / B hide Effects until fixed / C ship as is. **Recommend A.**
5. Region map debug bar and raw chest note (section 5): pick A debug-scene only, place and supplies in play, item names in notes / B a question on New game only / C leave for the UI review. **Recommend A.**

Wanted rows if approved (the planner writes them):

- BAL5-R1b (balance; after item 2)
- CAMPLABEL1-S (logic, Balance thread; `tools/sim/sim_campaign.gd`, `bal4_campaign_test_suite`)
- STATCRED1-L (logic, Gameplay; `scripts/combat/session/battle_metrics.gd`, `battle_report_test_suite`; before BREPORT1-U)
- MAPDEV1-U (logic, UI/UX; `region_map_view.gd`; before MAPLOG1-U, not in parallel with other `region_map_view.gd` rows)
- the potion price lever goes into BAL5-A as a trigger line
