# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to PR6-3, PR6-5 (excl. PR6-4; `FB-01`..`FB-281`, `UN-01`..`UN-12`) | released or done | `main` / `develop` (`f132eabc`) | Archived in `docs/archive/FEEDBACK_CLOSED.md` (moved 2026-11-17) |
| `PR6-4` (`PK3-RECRUITS-KEEPSAKES`) | done with parked clauses (`FB-282`, `FB-283`, `FB-284`) | `develop` (squash `PR6-4`) | Kit recruits/pool/crypts/fuse/keepsakes + `sunk_ward` seed `pr6_4.json`; temperaments, Martyr fuse data and the Retire/Last Stand prompt parked |
| `PR7-1` (`FIX-TEXT1`) | done | `develop` (squash `PR7-1`) | `sunk_ward` text fixes, seed `pr7_1.json`; new copy `[PROVISIONAL]` in `FB-286` |
| `WEBFIX1` (playtest, user 2026-11-17) | done | `develop` (squash `WEBFIX1`) | Web canvas fills page/fullscreen, web Fullscreen setting works; `FB-287` |
| `PR7-2` (`SPEC-FIX-N1N2`) | done | `develop` (squash `PR7-2`) | Sensory Rot on ranged hits; Cold Steel per-family rite + masterwork; `FB-288` |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-281 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-17). New items start at FB-282. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-282: PR6-4 cohort temperament ids do not exist

- **Spec**: `PK3_ROSTER_LIFECYCLE.md` §3.2, `READY.md` §1 `PR6-4`
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: The cohort table's shared temperament pool names `steady`, `wary`, `grim`, `quiet`, `reckless`, but temperaments are slotless passive ids and none of these exists in `passives.json` (`steady` is already a `unit_meters` id). The six `cohort_*` rows shipped with `temperaments: []` (the check allows it); the assertion is parked as the ignored test `sunk_ward_cohort_templates_draw_from_slotless_temperament_passives` (`emberwake_world/tests/sunk_ward_roster.rs`).
- **Recommendation**: Option A (recommended): name five slotless passives `temper_<x>` with a small stat or resist each, seeded with the pool lists in the table; Build then un-ignores the test. Option B: drop temperaments from cohorts (delete the parked test).

### FB-283: PR6-4 Living Martyr fuse needs numbers, passives and a deploy prompt

- **Spec**: `PK3_ROSTER_LIFECYCLE.md` §3.1, `ROSTER1` §8 (archive), `READY.md` §1 `PR6-4`
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: (a) Tier stacking conflicts: PK3 §3.1 says tiers are cumulative, `ROSTER1` FZ3 says only the highest tier applies; the kit implements PK3 (`FuseDef::passives_at`, `rpgkit_data/src/lib.rs`). (b) The spec gives `max: 6` and tiers `at: 2, 4` but no `per_deploy` or `stasis_days`, no tier passive ids and no `last_passive` ("Calcification V stats"), and no cohort carries a fuse, so no Martyr template is seeded. (c) The kit hooks exist (`roster::fuse_choices`, `fuse_refusal`, `last_stand`, `retire`, `portrait_of`) but `emberwake_gui` has no call site: the battle start does not yet gate on `fuse_refusal` (gating with no UI would block play), and procedural members need `roster::portrait_of` for their portrait.
- **Recommendation**: Confirm cumulative tiers; set `per_deploy: 1`, `stasis_days: 7`; pick tier passives (Option A: reuse `calcify_2`, `calcify_3` and a Stage V passive; Option B: new `martyr_*` passives) and the template that carries the fuse; queue a small UI row for the Retire / Last Stand prompt beside `party_min_refusal`.

### FB-284: PR6-4 crypt count is static; crypt-only refusal text added

- **Spec**: `PK3_ROSTER_LIFECYCLE.md` §3.1, `READY.md` §1 `PR6-4`
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: "`crypts` 0 in Act 1, 2 from the Act 2 facility upgrade or `pk2.slab_open`" has no mechanism (the manifest holds a static int); shipped `crypts: 2` as the READY row states (Martyrs only arrive in Act 2 anyway). A `crypt_only` recruit takes no bunk, so the spec's bunk refusal cannot apply to it; the kit checks crypt places instead with a third text, "Every crypt is taken ({n} of {cap}).".
- **Recommendation**: Option A (recommended): keep static `crypts: 2` and accept the third refusal text. Option B: a kit `crypts_flag` like `bunks_flag` (small schema slice), tied to `PR6-8` facilities.

### FB-285: PR6-4 landing report

- **Spec**: `PK3_ROSTER_LIFECYCLE.md` §§3–4, `READY.md` §1 `PR6-4` (@squash `PR6-4`)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit: world kind `recruits` (`recruits.json`, `WORLD_KINDS` 20); effects `rescue {template|character, base_unit?, count 1..4}` (seeded `recruit` roll stream, no name collisions) and `recruit {pool: id|first, level?}`; `roster::{rescue, swear_in, retire, dismiss}`; `UnitState` `name`/`housing`/`fuse`/`origin` plus `temperament`, `template` (needed for fuse and portrait lookup), `party.pool`, `party.keepsakes` (all skip-if-empty); procedural keepsakes `keepsake@<member>` named "{name}'s {item}", wear-legal accessory, one per template per unit. Data (`seeds/pr6_4.json`): six `orig_*` passives, six `keepsake_tpl_*` items, six `cohort_*` recruit rows with provisional given names (7 each), `sunk_ward` roster `crypts: 2` and `keepsakes.by_class`. Retire, dismiss and last-stand departures use cause `roster`. `units/mage` has no `base` body (only `1h_sprayer`); the mage cohort uses the class-derived `mage` party unit and asset checks pass. Tests: kit roster/campaign/temperament/roster_schema + `sunk_ward_roster` (6 pass, 1 parked by `FB-282`); 0 golden diffs; `campaign_schema` and `story_checks` pinned effect-list text gained `rescue`.
- **Recommendation**: `GAME_BRIEF.md` roster section: add the recruits/pool/crypts/fuse/keepsake line above. `HISTORY.md`: none (no golden moved). `STRUCTURE.md`: `worlds/<id>/recruits.json` beside the other world kinds. Art/UI: cohort names are provisional placeholders for a lore pass.

### FB-286: PR7-1 provisional copy for Design review

- **Spec**: `READY.md` §1 `PR7-1`, [2026-11-17 review](audits/2026-11-17_11d320a_consistency_audit.md) "Lore and data" (@squash `PR7-1`)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `PLACEHOLDER` copy replaced (`[PROVISIONAL]`, from `CANON.md`). Meters: `alert/a_suspicious` "The Inquisition is Suspicious: the ward gates are watched. Quiet jobs and bribes at Talia lower Alert."; `alert/a_lockdown` "Lockdown: Inquisition hunter packs and Castellan Watch sweeps hold the main roads."; `alert/a_last` "One more loud job and the Inquisition finds the grate. The run would end."; `doom_clock/d_week` "Seven days to the Red Tithing.". Faction descriptions: `iron_inquisition` "The church's hunters: bell sweeps, stockades and vivisection slabs; the Castellan Watch is its city arm."; `castellan_watch` "The city watch: curfews, ward gates and Tithing sweeps under Inquisition writ."; `tallow_guild` "Renders the district's Spent dead into Warded Tallow; its mercenaries scour the cellars."; `fourth_kiln_syndicate` "The Guild's foundry arm: casts brass conduits and runs the bone-mill.". The four `q_hook_*` rewards now say Scrap; `q_a1_m5` and `loot_hearth_ingot_cache` say "The Crucible". Faction `motto` fields stay empty. Test `emberwake_world/tests/sunk_ward_text.rs` (3 tests); 0 golden diffs.
- **Recommendation**: Design reviews the copy above and, if wanted, writes faction mottos (still empty) as a data row. `HISTORY.md`: quest `q_a1_m5` "The Drowned Sluice" renamed "The Crucible" (PR7-1). Note: older seeds (`act1fix1.json`, `act1story.json`, `pr6_5_act2_drops_flags.json`) still carry the old text; harmless while `pr7_1.json` applies after them.

### FB-287: WEBFIX1 web display fixes from the user's playtest, and three open asks

- **Spec**: user playtest 2026-11-17 (relayed by the coordinator, "fix ahead of PR7-1"); `WEBREL1`, archived `DISPLAY1` rule 8 (@squash `WEBFIX1`)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Fixed: (1) the web canvas was pinned to 1280 x 720 CSS px (`tools/web/index.html` viewport `width=1280`; `shell.rs` web window `with_inner_size`/`with_min_inner_size(CANVAS_CSS)`), so fullscreen only letterboxed it; now the page owns the canvas size (`shell::web_attributes_of`), the buffer is CSS size x pixel ratio and the layout rescales on `Resized` like a desktop window. (2) web `apply_settings` ignored every display setting; now Settings > Display > Fullscreen uses the browser Fullscreen API (`screen::WebFullscreen::step`, follows Esc, session-only) and the Display tab shows only Fullscreen and Text size on web (`screen::display_keys(web)`). Tests `aegis_ui/tests/web_display.rs` (5), `emberwake_host/tests/web_bundle.rs` (1); Playwright on a real web build at 1280x720, 1600x1000, 2560x1440. Not a web bug: (3) frame art is in the web bundle and loads; the battle, region and hub HUD panels draw flat panels on desktop too (`Painter::panel`/`fill`, e.g. `hub.rs:205,693`). Map zoom exists on web (Gameplay tab); the F6/F7 debug keys never work in web builds (`FB-162`) and no UI-scale setting exists (`LAYOUT720` unbuilt). `tools/web/smoke.js` step (b) "Start reaches the region map" fails because New game now opens a story scene and a battle (stale smoke step, not this change).
- **Recommendation**: Design rows for: (a) HUD panels using the theme frame art (Art/UI per screen; the user's "UI elements missing the frames"); (b) "zoom on web": Option A copy Map zoom onto the Display tab (recommended, smallest), Option B build the `LAYOUT720` UI-scale setting; (c) update `smoke.js` for the new opening; (d) cap the canvas buffer at the GPU texture limit for very large screens. Docs delta: `docs/workflow/web.md` "What differs in the browser": "Display | The canvas fills the page and the screen in fullscreen (itch.io's button or Settings > Display > Fullscreen, session only); the Display tab shows Fullscreen and Text size only."; `engine/README.md`: "Web display (WEBFIX1): the page owns the canvas CSS size (`shell::web_attributes_of`); buffer = CSS size x pixel ratio, rescaled on `Resized`; `screen::WebFullscreen::step` asks for or follows browser fullscreen; `screen::display_keys(web)` lists each platform's Display rows."; `HISTORY.md`: the fixed 1280 x 720 CSS canvas of `WEBREL1-E2` replaced by a canvas that fills the page.

### FB-288: PR7-2 landing report and two small questions

- **Spec**: `READY.md` §1 `PR7-2`, `PK2_GRAFTS.md` §5.2, review `N1`/`N2` (@squash `PR7-2`)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: N1: a `skill_potency` grant of group `ranged` with `min_distance > 0` now matches any ranged hit, the basic bow Strike included (`unit::distance_group_matches`; `potency_for_skill_at`/`power_for_at`/`potency_parts_at` take the hit's `ranged` flag; battle hit and forecast agree). N2 `[PROVISIONAL]`: the Cold Steel keystone needs mastery 3 in the main-hand family, that family's forge rite and a masterwork main hand of it; the rite is recorded per family (`forge_rites: Vec<String>`; the Hearth pane offers a family picker when several are open). Save: new member key `forge_rites` (skip-if-empty); a legacy `forge_rite: true` loads as one rite per family at mastery 3 and resaves as `forge_rites`. 0 golden diffs. Also: `READY.md` §4 rows `FB-282`..`FB-285` use disposition `answered` and follow-up `applied`, which `feedback_check.py archive` rejects (accepted values: `accepted`, `deferred`, `policy`; follow-up must be a §1/§3 id or `none`), so those four stay in this file until the rows are fixed.
- **Recommendation**: `GAME_BRIEF.md`: "Cold Steel keystone: mastery 3 in the wielded family + that family's forge rite (one per family) + a masterwork main hand of that family (PR7-2, [PROVISIONAL])"; "Sensory Rot's -15% beyond 3 tiles applies to every ranged hit, the bow Strike included". `HISTORY.md`: "PR7-2: one forge rite for every family deprecated (spec 5.2: per family, into a masterwork); Sensory Rot ranged-skill-only reading deprecated (N1). No golden re-recorded; a legacy single rite maps to the families at mastery 3 at load, so a family that reaches mastery 3 later needs its own rite." `kits/rpg/README.md`: "`potency_for_skill_at`/`power_for_at`/`potency_parts_at` take the hit's `ranged` flag; `UnitState.forge_rites` (save key `forge_rites`)." Questions: (a) should the rite itself also require a masterwork of the family (today only the keystone checks it)? (b) a `ranged` group grant without a distance gate still follows the skill, not the hit (no shipped data uses one): keep, or follow the hit like N1? Fix the §4 dispositions so Build can archive.

### FB-289: PR7-3 landing report and copy to confirm

- **Spec**: `READY.md` §1 `PR7-3`, review `N3` (@squash `PR7-3`)
- **Kind**: suggestion
- **Status**: open
- **Finding**: The kit already wrote `remains`, `mastery` and `wear` rows into the battle result; the results screen never read them. Now `emberwake_gui/src/hub.rs` draws them after the track lines: remains (gold) "Remains: X from Y, to the stash."; mastery (green) the kit's own note, e.g. "Bryn Hale: hammer mastery +10 (level 1). New level."; wear (dim) "Y's X wore N: M left."; dulled (red) "Y's X is dulled: -50% weapon power until repaired." (the -50% comes from the kit constant). Battle: the unit hover tip gets the dulled line and the party card a red "Dulled" pill (`battle/tips.rs`, `battle/draw.rs`, foe-badge colour, no new theme colour). `rpgkit_host` `state` unit rows gain `wear {slot: {durability, worn, dulled}}` and `attacked [families]`; `outcome_of` carries `wear` only (adding `attacked` there would change Act 1 checkpoint shapes). The UI Gallery "Battle results" sample shows all four lines. No formula changed; 0 golden diffs (`rules_contract_golden` unchanged). Tests: `rpgkit_host/tests/wear_keys.rs` (2), `hub::tests::the_results_draw_remains_mastery_and_wear`, `battle/tests/dulled_tests.rs`. Neither screen is reachable in a real run until Act 2 content (`PR6-12`); shots in project files `pr7-3/`.
- **Recommendation**: Confirm or rewrite the four line texts and the "Dulled" badge; the mastery line shows the family id ("hammer") rather than a display name: Option A (recommended) a display-name lookup in the kit note, Option B keep ids. Docs delta: `kits/rpg/README.md` host command list: "`state` unit rows gain `wear` and `attacked`; `outcome_of` checkpoints carry `wear` (PR7-3)"; `GAME_BRIEF.md` where-to-look: "Graft-era result lines `emberwake_gui/src/hub.rs` `graft_lines`; dulled tip and badge `battle/tips.rs`, `battle/draw.rs`". `HISTORY.md`: none.

### FB-290: PR7-5 landing report and two questions

- **Spec**: `READY.md` §1 `PR7-5`, review part 2 "Engine track" `3` (@squash `PR7-5`)
- **Kind**: suggestion
- **Status**: open
- **Finding**: `tools/fc.sh repro` (a gate step) replays every `games/emberwake/playtests/repro/*.json`; a divergence, refused request or panic fails, an empty folder passes. Seeded with one headless recording (classic `goblin_ambush` seed 3, timeline `goblin_skirmish` seed 7 autoplay). New `aegis record <requests.json | recording.json> out= id=` records (or re-records) a session on the current data. UI bot `aegis_ui::shell::bot` (game-agnostic) runs in-process from `emberwake_gui/tests/ui_bot.rs`: starts `menu`, `arena`, `arena_timeline`, `runner` x seeds 1,2,3 x 40 steps; skips `recordings.copy`, `recordings.open_folder`, `recordings.note_view.copy`, `menu.quit`; no panics found (22 screens reached); `tools/fc.sh bot seeds= steps=` for longer runs. Gate time +~16 s. 0 golden diffs. Deviation: the bot runs inside `cargo test` rather than as its own gate command. A seed repeats the same choice from the same widget list; screens read wall time, so a long run may take a different path with the same seed (the report lists every press).
- **Recommendation**: (a) The 2 inbox recordings (`20261021-193519` diverges at its first checkpoint, `20261022-204440` has a refused request at 17) were recorded on older data; moving them to `repro/` would fail the gate, and re-recording captures today's behaviour, not the reported bug. Option A (recommended): triage them as bugs/declined per intake, and re-record only those that still show the bug. (b) Repros replay on shipped data, so an intended data change that moves a checkpoint fails `repro` until re-recorded and named in the commit; Option A (recommended) keep that (it is the regression signal), Option B pin repros to a fixture data copy. Docs delta: `engine/README.md` "Playtest recordings and replay": add the `aegis record` line and "`tools/fc.sh gate` replays every recording in `games/<game>/playtests/repro/` (`tools/fc.sh repro`)"; crate table `aegis_ui`: "`shell::bot`: seeded in-process UI bot (`Bot {seed, steps, size, skip, keys}`, `bot::run`); the game supplies starts and skips". `STRUCTURE.md`: `shell/bot.rs`; `aegis_cli` `record`; `tools/fc.sh` `repro`, `bot`; `playtests/repro/` replayed by the gate. `cloud.md` "Playtest intake" step 4: re-record with `aegis record` when an intended change moves checkpoints. `HISTORY.md`: none.

### FB-291: PR7-6 balance bands: 4 of 9 rows out of band on develop

- **Spec**: `READY.md` §1 `PR7-6` (@squash `PR7-6`)
- **Kind**: sim
- **Status**: open
- **Finding**: `tools/fc.sh sim bands` (`tools/planner/sim_bands.py`) reads `games/emberwake/tuning.json` `balance_bands` (hand-edited: tuning is not a content kind; the script validates the block), runs `sim duel 5 50 score=binary` and `sim party 5 200 enc=<id>` per band, prints the table and exits 1 when a row is out of band (2 when the block is malformed). About 2 min on the 4-core cloud box; not in the per-task gate; 6 tool tests in `tools/planner/test_sim_bands.py`. Today's develop (level 5, party warrior/ranger/priest/mage): duel warrior 56% in, ranger **63% out**, priest **32% out**, brawler **64% out**, rogue 43% in, mage 42% in (band 40-60, each class's average vs the other five); party clear line **75% out** (80-95), elite 82% in (70-85), boss 34% in (30-50).
- **Recommendation**: Design picks the tuning for ranger, brawler (too strong in duels), priest (too weak) and line encounters (too hard), or widens the `[PROVISIONAL]` bands; a duel band may not suit a support class like priest (Option: exclude priest from duel bands). Questions: should the nightly CI step (`PR7-7`) treat exit 2 as a failure (recommended: any non-zero fails)? Should bands cover a higher level later? Docs delta: `GAME_BRIEF.md` where-to-look: "balance target bands: `games/emberwake/tuning.json` `balance_bands`, checked by `tools/fc.sh sim bands` (about 2 min, exit 1 out of band)"; `STRUCTURE.md`: none (planner scripts folder).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
