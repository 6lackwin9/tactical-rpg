# Agent map (planner-maintained dashboard)

The harness agent list cannot be regrouped, so this file is the readable view: active runs on top, then finished runs grouped by milestone with the most recently completed milestone first. The planner updates it when it dispatches an agent and when one reports. Tokens and minutes are from each agent's own report. Commit hashes are in `git log`.

## Active now

Desktop: the planner keeps this section current. Cloud: the project's thread list is the live view, so this section changes only at a handover between environments (`docs/workflow/README.md`); merged cloud tasks still get a finished-runs line.

Dispatch mode: **standard**. No agents running (wind-down 2026-10-17 for the move to a cloud instance).

Merged 2026-10-17: SKILL1, BAL1, BAL2-L (run 1 + 1b), BAL2-A, cleanup batches 1-2, CLN-1/2/4/5/10/11/15, AI2 (diagnosis), PLAN1, M4-L + M4-A, QUEST2, EXP1-L (commits 1-3) + EXP1-A, ITEM1-L (commits 1-3) + ITEM1-A, CLASS1-L + CLASS1-M, DLG1-L (commits 1-2) + DLG1-A, S14-L1 (commits 1-2). Open next: S14-L1 commit 3, S14-L2/L3, TRAV1, REP1, UI1-3, S12S/T/C, BAL3 spec (from BAL2-R), CLN-12/13/16, CLN-3b.

## Finished, by milestone (newest first)

### S12S-A status carriers (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | Five status families on class paths at level 6 (seed s12s.gd); loadouts, tier gates and duel/role sims unchanged | n/a / 30 | merged via PR |

### DLG1-F dialogue and shop follow-ups (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Buy count cap, sell/give gold-cap notes, confirm_turn_in pre-check, integer price oracle | n/a / 15 | merged via PR |

### BAL2-R clean balance rerun (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| cloud thread (no subagent) | party, sweep, role, duel, tier at L3/L5 on main d6f716c; BALANCE_REPORT rewritten, old rerun archived as superseded | n/a / 30 | PR squash-merged; 0 PLAN FAIL, 0 METRIC FAIL |

### UI1-L menus and save slots (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread, no subagent) | UI3-L: read-only quest journal, journal suite; screenshots 17-19 in project files ui1/ | n/a / 20 | merged via PR |
| planner (cloud thread, no subagent) | SAVE-IO: save export / import through a file dialog, save_io suite | n/a / 15 | merged via PR |
| planner (cloud thread, no subagent) | UI2-L: party, level-up, skills, gear, battle results screens; 5 shared save slots; screenshots 10-16 in project files ui1/ | n/a / 70 | merged via PR |
| planner (cloud thread, no subagent) | UI1-A: ashlands start_party seed (warrior, ranger, rogue, mage) | n/a / 5 | merged via PR |
| planner (cloud thread, no subagent) | start_party, slots 0-3 + peek + autosave, CurrentGame, SettingsFile, MenuRouter and five screen models, main menu scene; screenshots in project files ui1/ | n/a / 60 | merged via PR |

### S14-L3 audio (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Bus layout, SettingsFile audio keys, SoundEvents, SoundBank, AudioDirector, placeholder SFX tool, audio suite | n/a / 25 | merged via PR |

### S14-L2 combat presenter (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | CombatPresenter, FxPresets, FxSettings instant mode, animation lifecycle suite | n/a / 40 | merged via PR |

### Cloud cleanup threads (2026-10-17)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| cloud thread (no subagent) | CLN-12 suite split + CLN-6 ItemCatalog part | n/a | PR #8 squash-merged; PASS lines unchanged |
| cloud thread (no subagent) | CLN-16 remove the `smart_skills = false` AI path, its tests and the sim key | n/a | PR #13 squash-merged |

### S12S-L status expansion (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Commit 1 schema: registry magnitude ranges, six status rows, magnitude validation, sources, StatusExpansion suite | 135k / 16 | PR #15 |
| change-reviewer | Commit 1 review | 63k / 2 | no blockers; popup unit and comment fixed by the thread |
| logic-builder | Commits 2-3: slow, expose/fortify armor, round-end order, granted magnitudes, chips, AI rule 8, sim regen/dot metrics, two new suites | 212k / 28 | arena 472/0, dojo 45/0 |

### DLG1-L commit 3 dialogue and shop panels (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | DialoguePanel, ShopPanel in the region map; shop return without node effects (user decision); DialoguePanelChecks | n/a / 25 | merged via PR |

### S14-F asset pipeline follow-ups (2026-10-17, cloud)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| planner (cloud thread) | Registry family cache cleared on content reload, suite .get() lookups and copy check, spec item refs and origin rules | n/a / 10 | merged via PR |

### SKILL1 Forge skill tab (2026-10-17)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | SKILL1-L (first run stopped at usage limit; resumed run 105k / 2) | n/a + 105k / 2 | merged `611b667` |
| asset-builder | SKILL1-A migration seed | 111k / 11 | merged (planner enabled the 7 accepted pairings) |
| change-reviewer | Review | 83k / 3.5 | 1 high (fixture), 2 medium (warnings_all, lossy save) |
| logic-builder | Review fixes | 140k / 14 | merged |

### BAL1 balance rerun (2026-10-16)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| balance | duel L1/3/5/7, tier, levels, smart_skills A/B | 66k / 22 | report committed `80eeaf0`; open issues await user decisions |

### AI1 AI skill use

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | AI use of disengage, areas, cleanse, taunt, buffs, riposte | 205k / 20 (incl. 1 resumed fix) | merged |
| change-reviewer | Review | 36k / 1 | merged |
| design-brainstorm | SK2 unlock-rule spec | 79k / 3 | spec awaiting approval |

### M2C2B class skill content

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | All six kits, C1-A2, D1-A, party units | 92k / 8 | merged |
| logic-builder | Update 9 shipped test groups for the new kit | 75k / 6 | merged |
| change-reviewer | Review | 56k / 1.5 | merged |

### Model trial 1 (reviewer, inherited vs haiku)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| change-reviewer (haiku) | M2C2B-L static review | 70k / 3.5 | 1 test gap + nits |
| change-reviewer (inherited) | M2C2B-L static review | 28.5k / 1 | clean + nits |

### M3 progression and save

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Party state, XP curve, versioned save | 117k / 12 | merged |
| change-reviewer | Review | 34k / 1 | merged (rename fallback hardened) |

### M2C2B-L area statuses and defend taunt

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | X1 area statuses, X2 defend grants | 69k / 11 | merged |

### R1 test split

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Per-group suites, SuiteRegistry, TestHelpers | 56k / 12 | merged |

### Milestone planning (brainstorm in builder idle time)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | Milestone plan v2 (coarse), AI1 and M3 specs | 91k / 4 | docs, awaiting your answers |

### D1 weaken and riposte

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | D1-L weaken, magnitude, zero-power attacks, riposte data | 154k / 23 | merged (D1-A folds into M2C2B assets) |
| change-reviewer | Review of merged commit | 35k / 6 | clean |

### C1 orb and tome

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| asset-builder | C1-A1 gating, priest proficiency, tome retype | 23k / 2 | merged |
| logic-builder | C1-L orb/tome rules, item effects, gear bonuses | 155k / 14 | merged |
| change-reviewer | Review | 42k / 3 | merged (hand-rule gap accepted) |

### B1 rest to Tactical (first parallel pilot)

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| logic-builder | Rest to Tactical, stance constant | 47k / 10 | merged (no reviewer) |
| design-brainstorm | Specs C1, D1, M2C2B (ran in parallel) | 131k / 7 | docs/specs |

### Slice A foundation

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Basics cleanup, registry, unarmed, hands | 213k / 20 | merged |
| change-reviewer | Review | 51k / 4 | merged |

### M2c-2b design

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm | Skill draft v2 | 51k / 3 | docs |
| design-brainstorm | Revision 3 proposals | 47k / 2.5 | docs |
| design-brainstorm | C1, D1, M2C2B specs | 131k / 7 | docs/specs |

### M2c-2a2 engine pieces

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Cleanse, disengage, sweep, unarmed budget | 188k / 13 | merged |
| change-reviewer | Review | 51k / 3 | merged |

### M2c-2a1 skill families

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Families, tiers, loadout | 126k / 7 | merged |
| change-reviewer | Review | 37k / 2 | found rename hazard (fixed) |

### M2c-1 classes

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Six-class roster data | 109k / 12 | merged |
| design-brainstorm | Skill draft v1 | 41k / 2 | docs |
| change-reviewer | Review | 29k / 2 | merged |

### M2b items and art

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Eight items, placeholder art keys | 68k / 4 | merged (no reviewer) |

### M2d gear budget

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | GearBudget guard | 66k / 8 | merged |
| change-reviewer | Review | 24k / 2 | merged |

### M2a riposte

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Riposte engine | 128k / 9 | merged |
| change-reviewer | Review | 33k / 2 | merged |

### M1 combat extensions

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| builder | Support skills, gap-close, haste/bleed/taunt, AI fixes | 300k / 23 | merged |
| change-reviewer | Review | 30k / 2.5 | found stale reset bug (fixed) |

### Process and roadmap

| Agent | Task | Tokens / min | Outcome |
| --- | --- | --- | --- |
| design-brainstorm (general-purpose) | Roadmap proposals | 56k / 1.5 | Proposed designs in ROADMAP |

## Cost tracking (started 2026-10-16)

What can be measured:

- Subagents: each report gives total tokens, tool calls and minutes (tables above). Before 2026-10-16 every agent ran on the model the main session used (Sonnet 5.5): no agent definition sets a model, so they inherit it. From now on the Model column records the model passed on each dispatch (`sonnet`, `opus`, `haiku`, `fable` or "inherited").
- Main session (planner): I can read a remaining-token counter after each tool call. The delta between two points is logged below per milestone. It is an approximate indicator (it is not split into input, output and cached tokens, and it does not include subagent usage).
- Money: no prices are stored here. If you give a $/million-tokens figure per model, the planner can add a cost column.

| Point | Counter value | Delta since previous | Note |
| --- | --- | --- | --- |
| Baseline (R1, M2C2B-L, M3 running) | about 14,988,000 | - | start of tracking |

Model comparison plan (to pick the right model per task; change one thing at a time): run the same agent type on another model for a comparable task and compare tokens, minutes and defects found.

| Agent type | Suggested trial | What to compare |
| --- | --- | --- |
| change-reviewer | haiku vs inherited | defects found on the same diff, tokens, minutes |
| asset-builder | haiku vs inherited | validate_all clean on first try, spec deviations |
| design-brainstorm | opus vs inherited | spec completeness and number of questions raised |
| logic-builder | keep inherited; try opus on one large task only | tokens, minutes, reviewer findings |

Trial protocol (user decision 2026-10-16: compare tokens and minutes only; cost is tracked in the usage page, token efficiency stands in for cost efficiency):

1. Trial 1 (reviewer): when the next feature branch is ready, run two `change-reviewer` agents with an IDENTICAL prompt on the same worktree, one with `model: haiku` and one inherited (Sonnet 5.5). Record tokens, minutes, tool calls and the list of distinct findings (true defects, false alarms, misses relative to the union).
2. Pick the model per agent type from the results and record it in the agent definition (`model:` frontmatter) and in `docs/DECISIONS.md`.
3. Repeat for `asset-builder` (first data task) and `design-brainstorm` (next spec), one trial per type.

Trial results:

| Trial | Model | Tokens | Minutes | Real defects found | False alarms | Missed (vs union) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 reviewer (M2C2B-L static review) | haiku | 69.6k | 3.5 | 1 test gap (a double status roll would not be detected) + 4 nits (self-referential haste check, unused loop var, help-text wording, trailer ambiguity) | 0 | 0 |
| 1 reviewer (M2C2B-L static review) | Sonnet 5.5 (inherited) | 28.5k | 1.0 | 0 (all PASS) + 4 nits (rng order change, hardcoded [Riposte] log tag, outdated comment, weak seeded-repeat check) | 0 | 1 (the double-roll test gap) |

Trial 1 conclusion (2026-10-16): on a static read-only review the inherited model used 41% of haiku's tokens (28.5k vs 69.6k) and 29% of its time (1 vs 3.5 minutes), because haiku explored far more (23 tool calls vs 4). Haiku found one real test gap the inherited model missed (the area-status test could not detect a double roll); the inherited model found no gap but its other nits were of similar value. Token efficiency favours the inherited model; haiku adds some extra thoroughness at 2.4x the tokens. Decision: keep reviewers on the inherited model; use haiku only when a second, deliberately thorough pass is wanted. Next trials: asset-builder (a small data task) and design-brainstorm (opus vs inherited), one per type. Other reviewer runs for reference (inherited model): M3 34k / 1, C1-L 42k / 3, D1-L 35k / 6.

Trial 2 (2026-10-16): asset-builder on haiku, SK2-A data step (small, well specified): 38.9k tokens, 3 minutes, 12 tool calls. It completed 3 of 4 steps correctly (riposte offsets, warrior path, passive templates), followed the two-attempt rule and stopped on a real schema gap (a single-row upsert cannot change `advanced` across a family) with an exact error and a clear question instead of working around it by deleting rows. The gap was a miss of SK2-L (fixed with `ContentCatalog.upsert_many`). Verdict: haiku is fine for small data steps with a clear spec and strict stop rules (about 40% of the tokens of the last inherited asset run, though that task was larger); keep the inherited model for large content tasks until a comparable-size trial exists.
