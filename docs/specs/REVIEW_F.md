# REVIEW-F: follow-ups from the retroactive review of the 2026-10-17 merges

Status: approved (user 2026-10-17, F1-F5); Q1-Q3 answered A ("all recs"): Q1 and Q2 join REVIEW-F2, Q3 joins REVIEW-F3
Depends on: see each task
Lane: logic (five tasks, no asset part)
Touches: see each task; F5 overlaps F1-F3 and runs last

## Goal

Thirteen PRs (#4, #6, #8, #9, #11, #13, #14, #16, #17, #19, #20, #21, #23) were merged on 2026-10-17 without a change-reviewer pass. A retroactive pass found five medium bugs and a set of lows. This spec groups them into five small logic tasks. Each finding was checked against current `main` before it went in. Line numbers are from `main` at bb7a367.

Verification notes:

- Every medium finding holds. The design-only medium (new game overwrites the autosave) is not a bug against UI1-L (its rule 4 says to autosave after a new game), so it is Q1 below.
- #6 DLG1-L c3 (the TASKS_DONE.md wipe) is already fixed on branch `claude/project-thread-x7he3n` (REVIEW-STEP) and is left out here.
- The `combat.md:20` pointer nit (#13) is in a planner-owned decisions file. It is listed in the report for the planner and is not in any task here.
- `ai_skills_tactics_test_suite.gd:166,178` (`kits[i] as Array`) is older than these merges. It is folded into F5 because it is the same Variant-cast pattern.

Schema contract (all tasks): no data, schema or save-format change. `SaveGame.VERSION` stays 4.

## REVIEW-F1: save safety (SAVE-IO, UI1-L)

Depends on: none. Coordinate with TRAV1-L, which shares `save_game.gd`, `region_map_view.gd` and `save_test_suite.gd`; run F1 first or rebase it after.
Touches: `scripts/game_state/save_game.gd`, `scripts/ui/overworld/region_map_view.gd`, `scripts/ui/menus/save_slots_model.gd` (comment), `scripts/ui/runner/scene_runner.gd` (description strings), the battle and dialogue entry points that call `CurrentGame.set_busy` (`scripts/ui/overworld/region_map_view.gd`), `scripts/tests/save_io_test_suite.gd`, `scripts/tests/save_test_suite.gd`

Findings and fixes:

1. Medium, `save_game.gd:134-138`. `export_to` deletes an existing file at `path` before the rename, so a failed rename loses the player's previous export. This breaks the tmp-and-rename rule (UI_MENUS_RULES "Writes keep the previous file on failure"). Fix: never delete `path` first. Rename `path` to `path + ".bak"`, rename the tmp file into place, then remove the `.bak`. If the second rename fails, rename the `.bak` back and report `"Can't move the export into place at '<path>' (the previous file was kept)."`.
2. Medium, `region_map_view.gd:17` and `:244-251`. `DEBUG_SLOT = SaveGame.AUTOSAVE_SLOT`. The standalone debug map (scene runner entry, its own "New game" button with `DEBUG_SEED`) writes the player's real autosave (slot 0) on every `autosave` event. Fix: `_autosave()` writes slot 0 only when `gs == CurrentGame.state()` (a game started from the main menu). Otherwise it writes nothing and logs `"Autosave skipped (debug game)."`. Remove `DEBUG_SLOT` and use `SaveGame.autosave(gs)`. Update the header comment on line 12 and the scene runner description (`scene_runner.gd:35`) to match.
3. Medium, `save_game.gd:183-196`. `peek` marks a file `ok` when `version` is missing, 0 or negative. `SaveMigrations.migrate` refuses those files (`save_migrations.gd:76-82`). `newest_slot` can then pick a slot that `read` refuses, and Continue fails on every press. UI1-L rule 3 says peek reads versions 1 to `VERSION`. Fix: when `version` is not a number or is `< 1`, set `error = "unreadable"` and leave `ok = false`.
4. Low, `save_game.gd:63-68` and `:128-133`. `store_string` and `close` results are not checked. Fix: after `close`, check `file.get_error()` (or reopen the tmp file and compare its length). On failure, remove the tmp file and return `"Can't write '<path>'."`.
5. Low, `save_game.gd:127`. `path + ".tmp"` can overwrite a user file with that name. Fix: use `path + ".fc_tmp"`, plus a numeric suffix if that name already exists.
6. Low, `save_game.gd:158`. Import keeps the file's `saved_at` even when it is in the future, so that slot wins `newest_slot` forever. Fix: on import, clamp `saved_at` to `now`.
7. Low, `save_game.gd:152`. Import reads files of any size. Fix: refuse files over 8 MiB with `"File is too large to be a save."` before reading.
8. Low, `current_game.gd:33`. `set_busy` is never called. `SaveSlotsModel` (`save_slots_model.gd:83`) checks `CurrentGame.busy()`, so a save is never refused during a fight or a conversation (UI1-L rule 5). Fix: the region map sets busy to true when a fight is pending or a dialogue or shop opens, and back to false when it settles or closes. The existing `busy()` method (`region_map_view.gd:139`) is the source of truth.
9. Low, `save_slots_model.gd:5` says "slot 0..3", and `scene_runner.gd:25` says "autosave + 3". Fix: change both to "0 = autosave, 1-5 manual".

Acceptance checks:

- An `export_to` over an existing file, with the rename forced to fail (read-only target folder in a sandbox), leaves the old file byte-identical and returns the "previous file was kept" error.
- A normal `export_to` over an existing file replaces it and leaves no `.bak` or `.fc_tmp` file behind.
- A debug-map game (not `CurrentGame`) that fires an `autosave` event leaves slot 0 unchanged (same bytes, or still missing). A main-menu game that fires the same event writes slot 0.
- Version-less, version 0 and version -1 files peek as `ok = false, error = "unreadable"`. `newest_slot` skips them, and `can_continue` is false when only such a file exists.
- An import with `saved_at = now + 1 day` stores `saved_at <= now`. A 9 MiB file is refused and the slot stays untouched.
- With a pending fight or an open dialogue, `SaveSlotsModel` save returns `BUSY`. After the fight settles, save works.

## REVIEW-F2: menu flow (UI2-L, UI3-L)

Depends on: none (no shared Touches with F1). F3 waits for F2 because both touch `menus_test_suite.gd`.
Touches: `scripts/ui/menus/main_menu_view.gd`, `scripts/ui/menus/battle_results_model.gd`, `scripts/ui/menus/battle_results_view.gd`, `scripts/ui/menus/level_up_model.gd`, `scripts/ui/menus/gear_model.gd`, `scripts/ui/menus/loadout_model.gd`, `scripts/ui/menus/party_model.gd`, `scripts/core/specs/character_stats.gd` (one constant), `scripts/ui/menus/journal_view.gd`, `scripts/ui/menus/journal_model.gd`, `scripts/tests/menus_test_suite.gd`, `scripts/tests/party_screen_test_suite.gd`, `scripts/tests/journal_test_suite.gd`

Findings and fixes:

1. Medium, `main_menu_view.gd:316-319`. Esc on the results screen takes the `router.pop()` branch, because the game view is hidden while results show. That skips `continue_on()`, so there is no autosave and no `results_done`, and the hidden game view is orphaned behind the hub. Fix: when the top screen is `MenuRouter.RESULTS`, Esc calls the results model's `continue_on()` (the same as the Continue button). The router's generic pop must never pop RESULTS.
2. Low, `gear_model.gd:66` and `loadout_model.gd:131`. Gear errors and loadout errors are told apart by `e.contains("loadout")`. Fix: `LevelUp` / `Loadout` / `Gear` confirm paths return or tag the source (for example, a `{"errors": [], "loadout_errors": []}` split at the model level). No message text is parsed.
3. Low, `level_up_model.gd:75-84`. `drafted_state()`, `before()` and `after()` dereference `_member()` without a null check, so they crash on a stale member id. Fix: return an empty `Dictionary` (and a null-safe `drafted_state`) when `_member()` is null. `confirm()` returns `"No such party member."`.
4. Low, UI2-L spec rule 4. The new-skills list on the results card (`battle_results_view.gd:24-29`) has no "Open loadout" action when a slot is free. Fix: add the button. It pushes `MenuRouter.LOADOUT` for that member and shows only when that member has a free class or extra slot.
5. Low, `party_model.gd:9`. The message hardcodes "at most 4", while the check on line 66 uses `PartyState.PARTY_MAX`. Fix: build the message from `PartyState.PARTY_MAX`.
6. Low, `party_screen_test_suite.gd:97`. The test uses the HP literal 8, which copies the inline `effective_cons * 8` at `character_stats.gd:146`. Fix: add `const HP_PER_CONS: int = 8` to `CharacterStats` (no value change), use it on line 146 and in the test. Coordinate with CLN-7, which also edits `character_stats.gd`.
7. Low, missing test. There is no save-load round trip for slots 4 and 5. Fix: add a `menus_test_suite` check that saves slots 4 and 5, then reads them back equal.
8. Low, UI3-L `journal_view.gd:33`. `var name` shadows `Node.name`, and the view calls `QuestRunner.resolved` itself. Fix: `JournalModel` rows carry `name` (resolved), and the view reads `row["name"]` into `quest_name`.
9. Low, `journal_test_suite.gd:121` (check 1). Fix: also assert `jm.groups().is_empty()` for a world with no known quests.
10. Low, `journal_test_suite.gd:202` (check 4). Fix: compare the journal's loot ids with `QuestRunner.offers(gs, qid)` before and after the claim, not only a count.

Acceptance checks:

- In the results screen, Esc writes slot 0 (`saved_at` changes), emits `results_done`, and shows the game view again. The router stack has no RESULTS entry afterwards.
- A gear error that contains the word "loadout" is still shown as a gear error.
- `LevelUpModel.new("nobody")` gives `before() == {}` and `after() == {}`, and `confirm()` returns the error without crashing.
- After a level-up that unlocks a tier while a slot is free, the results card has "Open loadout", and pressing it opens the loadout screen for that member. With no free slot, the button is absent.
- Toggling a fifth active member returns a message built from the constant. Changing the constant changes the message.
- Slots 4 and 5 round-trip `to_dict` equal.
- The journal checks pass with the stricter assertions, and no view code calls `QuestRunner`.

## REVIEW-F3: audio and FX (S14-L2, S14-L3)

Depends on: REVIEW-F2 (shared `menus_test_suite.gd`)
Touches: `scripts/audio/audio_director.gd`, `scripts/ui/combat/combat_presenter.gd`, `scripts/ui/combat/fx_settings.gd`, `scripts/game_state/settings_file.gd` and every `SettingsFile.load()` caller (`settings_file.gd:83`, `scripts/tests/audio_test_suite.gd`, `scripts/tests/menus_test_suite.gd`), `scripts/tests/animation_lifecycle_test_suite.gd`

Findings and fixes:

1. Medium, `audio_director.gd:17,34-39`. `active` is one static slot. When the arena's director (`scenes/combat_arena.gd`) exits inside the scene runner, it sets `active = null` while the runner's director is still alive. After that, `SettingsModel.apply` (`settings_model.gd:62`) finds no director and volume changes do nothing. Fix: replace the single slot with a static stack (`Array[AudioDirector]`). `_enter_tree` pushes, `_exit_tree` erases self, and `active` returns the last valid entry.
2. Low, `combat_presenter.gd:68-70`. `fx_played` is emitted for every preset, even when `_apply` skipped it (null actor or target), so a sound plays with no visual. Fix: `_apply` returns `bool` (applied), and the signal is emitted only when it returns true.
3. Low, `combat_presenter.gd:58-61`. The distance is computed from `target_px` even when `target` is null and the entry is not MOVE. Fix: compute it only when `actor != null` and (`target != null` or the entry is MOVE). Otherwise pass -1, so `FxPresets._distance` uses the log coordinates.
4. Low, `fx_settings.gd:31-33`. `load_from` can set `instant = false` in a headless run (for example, a developer's settings.cfg), so tests start animating. Fix: `instant = raw_instant or DisplayServer.get_name() == "headless"`.
5. Low, `settings_file.gd:39`. `static func load()` shadows the global `load()` inside the class. Fix: rename it to `load_file()` and update all callers.

Acceptance checks:

- Runner director A, then arena director B enters and exits: `AudioDirector.active == A`, and `apply_volumes` reaches A. With both gone, `active == null`.
- `play()` with a null target on an ATTACK entry emits `fx_played` only for the presets that ran, and the distance comes from `coord_from` / `coord_to`.
- Headless, `load_from` with `anim/instant = false` leaves `instant == true`. The speed is still read.
- `grep -rn "SettingsFile.load()"` returns nothing, and the settings tests pass.

## REVIEW-F4: web CI hardening (WEB1)

Depends on: none (CI only, runs in parallel with anything)
Touches: `.github/workflows/web.yml`, `export_presets.cfg` (Web preset `exclude_filter`), `docs/workflow/web.md` (butler pin note, as a docs delta for the planner)

Findings and fixes:

1. Medium, `web.yml:112`. Butler is downloaded from `LATEST` with no checksum and then runs with `BUTLER_API_KEY` in its environment. Fix: add the env vars `BUTLER_VERSION` (an exact version) and `BUTLER_SHA256`. Download `.../linux-amd64/${BUTLER_VERSION}/archive/default` and run `echo "$BUTLER_SHA256  /tmp/butler.zip" | sha256sum -c -` before unzipping. The builder records the current stable version and its hash.
2. Low, `web.yml:13-15`. `cancel-in-progress: true` also cancels a running deploy on `main`. Fix: `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`.
3. Low, `web.yml:95-114`. The `itch` job has no timeout. Fix: `timeout-minutes: 15`.
4. Low, `export_presets.cfg:11`. The Web export packs `scripts/tests/*` and `data/seeds/*`. Fix: `exclude_filter="build/*, scripts/tests/*, data/seeds/*"`. If a shipped script preloads anything under `scripts/tests/` (for example the scene runner's tools entry), keep that file and record the exception in the report. Do not break the export.
5. Optional, low. Actions are pinned by tag (`@v4`, `@v3`). Pinning to commit SHAs is allowed if the builder has the SHAs. It is not required.

Acceptance checks:

- `web.yml` contains no `LATEST`, and a wrong `BUTLER_SHA256` makes the itch step fail before `butler push`.
- On a PR, a new push cancels the older run. On `main`, two pushes both finish.
- The exported `index.pck` lists no `res://scripts/tests/` or `res://data/seeds/` paths (check with `--headless --export-pack` plus a listing, or the export log). The Web build still passes the existing export step (all 4 files exist, no `Parse Error`).

## REVIEW-F5: Variant casts and small cleanups

Depends on: REVIEW-F1, REVIEW-F2, REVIEW-F3 (overlapping files). Coordinate with CLN-8 (`TestHelpers`) and REP1-L (`dialogue_runner.gd` header).
Touches: `scripts/game_state/game_state.gd` (new static helper), the 13 files with `as GameState` (`new_game_model`, `save_slots_model`, `current_game`, `save_game`, `quest_runner`, and the tests `journal`, `exploration`, `save_io`, `quest_runtime`, `save`, `party_screen`, `dialogue`, `menus`), `scripts/tests/ai_skills_tactics_test_suite.gd`, `tools/fc.sh` (usage comment), `scripts/dialogue/dialogue_runner.gd` (header comment), `scripts/tests/asset_pipeline_test_suite.gd`, `data/seeds/ui1_start_party.gd` (code, not data), `tools/_render_party_tmp.gd.uid` (delete), `scripts/tests/menus_test_suite.gd` (UI1-A header note)

Findings and fixes:

1. Low, cross-cutting. `<Variant> as GameState` appears 42 times in 13 files. This is the `UNSAFE_CAST` pattern from skill `gdscript-typing` section A. Fix: add `static func of(v: Variant) -> GameState` to `GameState`. It returns `v` when `v is GameState` and null otherwise, following the checked pattern already used in `region_map_view.gd:107-111`. Replace every site.
2. Low, `ai_skills_tactics_test_suite.gd:166,178`. `kits[i] as Array`. Fix: `DictUtil.array_of(kits[i])`. Sweep the touched files for any other `<Variant> as <Type>` and apply the same fix.
3. Low, `tools/fc.sh:18`. The usage text still lists `smart_skills=on|off`, which `balance_sim.gd` no longer reads (CLN-16). Fix: drop the key from the usage text.
4. Low, `dialogue_runner.gd:8`. The comment cites "DLG1 open items" for the shop-return rule. The right row is `world-campaign.md` "DLG1 shop return (user 2026-10-17)". Fix: correct the citation.
5. Low, `asset_pipeline_test_suite.gd:108`. The second half (`skill_family_ids() == _family_list`) is always true, because the call rebuilds and returns that same list. A reload from the same root is also not checked. Fix: record the family list before the reload, then assert that the stamp was cleared and that the rebuilt list equals a fresh computation from the catalog. Add a second reload with the same root and assert the cache rebuilds (stamp set again).
6. Low, `data/seeds/ui1_start_party.gd:12`. The `save_world` Error is printed, but the seed exits 0. Fix: append `"save_world failed: <err>"` to `errors` when it is not OK, so the seed exits 1.
7. Low, `tools/_render_party_tmp.gd.uid`. A stray uid file with no script. Fix: delete it with `git rm`. No script refers to it.
8. Low, process (#19). The asset task UI1-A added a test (`menus_test_suite.gd:68-80`). Fix: keep the test and add a header comment saying the logic lane owns it. The planner records the lane rule (asset tasks propose tests, and logic tasks write them) in the report.

Acceptance checks:

- `grep -rn "as GameState" scripts tools` returns nothing. `GameState.of(null)`, `GameState.of(42)` and `GameState.of({})` return null, and a GameState returns itself.
- The gate is clean with no new `UNSAFE_CAST` warnings.
- The `fc.sh` usage text has no `smart_skills`, and the `dialogue_runner.gd` header cites "DLG1 shop return".
- The asset pipeline check fails when `AssetRegistry` keeps a stale family list (fault injection in a sandbox), and passes otherwise.
- Running the seed with an unwritable worlds root exits 1.
- `tools/_render_party_tmp.gd.uid` no longer exists.

## Out of scope

- Any change to new-game autosave behaviour, level-up confirmation or miss visuals until Q1-Q3 are answered. Each answer becomes a separate task or an addition to F2 or F3.
- The decisions-file pointer at `combat.md:20` (planner).
- Moving tests between suites, or CLN-8 helper consolidation beyond `GameState.of`.

## Questions for the user

Answered 2026-10-17: A for all three (user, "all recs"). Q1 and Q2 are built in REVIEW-F2, Q3 in REVIEW-F3.

Q1. A new game silently replaces the autosave (`new_game_model.gd:35`; UI1-L rule 4 says to autosave after a new game, but it never asks first). A player who starts a new game by mistake loses their most recent progress if they had not saved manually.

- A. Ask `"Starting a new game replaces your autosave (<world>, <location>, <time>). Continue?"` only when slot 0 holds a readable save. Cost: small (one `router.ask` in `NewGameModel.choose`, plus one test).
- B. Before the new autosave, copy the old slot 0 into the first empty manual slot. If no slot is empty, ask as in A. Cost: medium. It fills manual slots without the player's consent.
- C. Keep the current behaviour.
- Recommendation: A. It matches the existing overwrite-confirm rule for manual slots (UI_MENUS_RULES 2.3) and is the cheapest.

Q2. Level-up Confirm is permanent, with no respec (decision "Level-up spend", UI2-L rule 3), but it applies on one click with no question (`level_up_view.gd:35`).

- A. Confirm asks "Spend these points? This is permanent." (the `router.ask` pattern). Cost: small.
- B. No question, but the button reads "Confirm (permanent)" and the stats before and after stay visible. Cost: trivial. Misclicks can still happen.
- C. Keep the current behaviour (Cancel already exists before Confirm).
- Recommendation: A. It is permanent and cheap to guard, and it matches the delete and overwrite confirms.

Q3. A MISS plays the hit shake and flash (`fx_presets.gd:80`: zero damage without a status falls through to `hit`). The log tags a miss `"MISS"` (`combat_log_builder.gd:29,119`).

- A. A new `miss` preset: a short sidestep or fade on the target and float text "Miss", with no shake or flash. Its sound key `sfx/miss` falls back to silence until an asset exists (S14 rule 7). Cost: small (`FxPresets` table plus one branch, one test).
- B. A miss plays only the attack preset and float text "Miss", with no target effect. Cost: trivial.
- C. Keep the current behaviour.
- Recommendation: A. A miss must read differently from a hit, and A gives the asset lane a hook for a miss sound later.
