# UI/UX handover

Written 2026-10-18 11:06 UTC by the UI/UX session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md` ("Usage caps": build work in economy mode, standard only for the final change-reviewer pass; runs over ~10 min in the background; rotate at ~120k context).

## Landed this session (develop)

INTENT1-U enemy tells (8d4971d), BOSS1-U phase banner (c14582e), MAPKEYS1-U WASD, held-key walking, glide and label fix (126f56f). No branch in flight; `claude/ui-ux-os35x9` equals `origin/develop`.

## Waiting on the user

- COMBAT-UI3 mockups (Design canvas https://claude.ai/artifact/E75abGsPLH6E6GZfBXVASi: A slim top strip, recommended; B bottom command bar; C plan column). Design holds the pick card and relays the pick. HOVER1-U is absorbed into it.

## Queue (in order)

1. **INTENT1-U follow-up** (no row; Design DR2 item 8): `IntentRules.intent_of` returns kind `arriving` for any enemy with no scheduled action (`intent_rules.gd:130`), so on-grid enemies whose AI drafted nothing read "Arriving" (Brawler, Skirmisher in `/mnt/project-files/intent1-u/after_planning.png`). Find why their plans are empty and give them a correct label (only real wave arrivals say Arriving).
2. **UIR-3** Character popup: Skills tab (descriptions, active rank only), Level up tab rework (user 09:41, his own choices, screenshot `/mnt/project-files/uir-3/before_level_up_user.png`), Gear tab with the pouch section. Merge on top of Gameplay's UIR-2 (in-play Menu gets Party, Save, Load); overlap on `menu_router.gd`.
3. **UIR-6** with the dialogue bottom bar (pick B).
4. COMBAT-UI3 once picked; OBJ1-U (results line on top of BREPORT1-L's `BattleResultsModel` API); MAPLOG1-U; QUESTTRACK1-U; RCLOSE1-U (after UIR-2); MAPNAV1-U (after UIR-6; includes click to move and legend collapse); UIR-4, UIR-7; BREPORT1-U and HEAL1-U after UIR-4 (Gameplay's HEAL1 notes: HP via `HealRules.current_hp`/`max_hp_of`, rename "fully healed" texts).

## Notes

- UI picks from mockups are preferences, not final (user 08:37); his own playtest choices are decided. Keep layouts easy to swap.
- Peers (session ids): Gameplay `session_01448UbhXvTzeBA1Brw4buLy` (doing UIR-2), Platform `session_01XyjA2VuzGTGfemqck5wsKf`, Design `session_01GS8RxAznG7gTrbfCb7w5uB`, Assets `session_01V6GmFsz2ESnDgqza17YhQt`, Balance `session_017JH6C5a2uwCQHAbGFH5whk`. Tell them your new id.
- Gate in the cloud needs `SUITE_TIMEOUT=900` (580 times out). `fc.sh parse` checks nothing on a clean tree; reviewers use `godot --headless --check-only` on the branch's changed files.
- Commit Timing lines carry `type=` and `tokens=<agent>:<k>` per agent kind (7abbb56).
- Renders: Xvfb, helper scripts outside the repo, screenshots to `/mnt/project-files/<task>/`; reply with them and "press Restart runner".
