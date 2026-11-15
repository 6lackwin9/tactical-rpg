# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to PR4 rows (R1 to R4 releases, the Post-R4 queue PR4-1..PR4-9) | released or done | `main` / `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-1 to PR5-6, PR5-8, PR5-16, PR5-17, PR5-20 to PR5-25 | done (FB-168 to FB-189, FB-194 to FB-196, answered) | `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-7, PR5-9 to PR5-13b, PR5-18, PR5-19, PR5-26 to PR5-35 | done, released to `main` in #62 (FB-190 to FB-234, answered @cf3d93b4) | `main` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-11) |
| PR5-36 to PR5-48 | done (FB-235 to FB-259, answered through @35c365b9) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md` |
| PR5-49 `MAP-POLISH1` | done (FB-260; FB-261 questions): region `map_reveal` with grey locked places, sidecar beside the drawn background, location `art`, saved Legend, Act 1 `map_pos` = sidecar, editor end anchors and pins; 0 golden diffs | `develop` | - |
| PR5-50 `KIT-HOOKS-CLEANUP` | done (FB-262; FB-263 questions): `skill_reach`, `friendly` / `hostile` sides, preset and training checks; `checks_golden` re-recorded (3 lines) | `develop` | - |
| PR5-51a `GEAR-REWORK1-KIT` | done (FB-264; FB-265 questions and notes for 51b): Switch Weapon retired, role-gated strict gear rule, slot rules, off-hand mechanics and button, knife Throw; 0 golden diffs | `develop` | - |
| PR5-51b `GEAR-REWORK1-DATA` | done (FB-266; FB-267 limit-blocker: row vs spec, soft penalty, provisional numbers; FB-268 sweep): gear families, roles, off-hands, named goldens re-recorded | `develop` | - |
| PR5-52 `RULESET-MODES1` | done (FB-269): one shipped ruleset (`timeline`), `classic` a kit preset, typed `modes`; `rules_contract_golden` re-recorded | `develop` | - |
| PR5-53 `VFX-DATA1` | done (FB-270): `fx_presets` catalog (42 presets), `fx.rs` with no per-preset tables, frozen parity fixture; 0 golden diffs | `develop` | - |
| PR5-54 `REVEAL1` | next | - | - |
| PR5-55, PR5-56 | queued, in order (user 2026-11-14) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-261 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-14). New items start at FB-262. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-262: PR5-50 `KIT-HOOKS-CLEANUP` landed

- **Spec**: READY Section 1 row PR5-50; FB-171, FB-189, FB-204, FB-212 (the `PR5-50` squash commit on `develop`, 2026-11-14)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-263: PR5-50 readings and questions

- **Spec**: READY row PR5-50 (@ the `PR5-50` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (a) The row asks for a warning when a `train_proficiency` targets "a family that 0 classes hold at minor proficiency (1)"; the effect names a class role, not a family (`{role, to}`), so the warning is per role at level `to - 1` and covers every role, and since no shipped class sets a proficiency `role` yet it raises 3 warnings (`train_secondary`, `train_third`, `train_minor`) until PR5-51b adds the roles. (b) `kind: SPELL` is not a skill kind in code; it reads as the old spell group (ATTACK with a spell source), which is what reproduces `spell_reach`. (c) The constant `W_SPELL_REACH` keeps its name (it now weighs every `skill_reach` tile) so `rules_contract_golden` stays unchanged.
- **Recommendation**: (a) keep every role (recommended; PR5-51b's roles clear the warnings) or limit to `minor`; (c) rename to `W_SKILL_REACH` with a named 1-line `rules_contract_golden` re-record in a later row if wanted; (d) aliasing the unit-row `side` field to `friendly` / `hostile` too is a larger change (many `== "party"` checks) left out.

### FB-264: PR5-51a `GEAR-REWORK1-KIT` landed

- **Spec**: READY Section 1 row PR5-51a; `docs/sessions/U3_GEAR_REWORK.md` §1-6 (the `PR5-51a` squash commit on `develop`, 2026-11-15)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-265: PR5-51a questions and notes for PR5-51b

- **Spec**: READY rows PR5-51a / PR5-51b (@ the `PR5-51a` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (a) Reload is open only at 0 ammo (the row's "Reload basic when 0"), checked against the current meter, so "shoot, then Reload" in one round at 1 pip is refused at planning time; the AI never chooses to Reload early. (b) The planner's damage preview shows the un-empowered number (the empower applies when the attack starts). (c) The Throw goes to any `1h_light` main (knife, dagger, fist) beside ammo; data can make it knife-only (`weapon_types: ["knife"]` with `min_proficiency >= 1` on the throw skill). (d) §1's retirement of the soft `untrained_penalty` / `unfit_penalty` is not done yet: `PROF_GEAR_MULT[0] = 1.0` keeps the soft penalty for gear worn while locked (e.g. enemy loadouts). (e) For 51b: the strict lock is enforced in party / save paths and the Dojo, not in battle builds or the content loadout check, so strict-class units or templates could wear locked gear under the old soft penalty; `keepsake_kaelen` / `keepsake_vesper` (3 bonuses each) will fail the accessory limit once roles exist; the two Dojo presets above need new pairs. (f) Not built (not in the clause text): retiring `+level` training power under the strict rule.
- **Recommendation**: (a) keep (recommended) or "any time below max", plus plan-aware meter checks (`SimState::meter`) in a later kit row; (b) add the empowered number to the preview later if wanted; (c) knife-only through data in 51b (recommended); (d)-(e) 51b: a strict-mode loadout content check and `*_drill` passives for enemies, retire the soft penalty with the data, trim the two keepsakes, fix the presets; (f) decide with the 51b numbers.

### FB-266: PR5-51b `GEAR-REWORK1-DATA` landed

- **Spec**: READY Section 1 row PR5-51b; `docs/sessions/U3_GEAR_REWORK.md` §2-6 (the `PR5-51b` squash commit on `develop`, 2026-11-15)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-267: PR5-51b conflicts, limits and provisional numbers

- **Spec**: READY row PR5-51b vs `U3_GEAR_REWORK.md` §1-5
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: (1) **Row vs spec**: the row's summary lists warrior `flail 1`, rogue `scourge 2`, a `2h_rig` `sprayer`, and keeps `axe` / `mace` among the 1H heavy families; the locked §2-3 matrix has warrior minor `hammer`, rogue third `bow` / minor `fist`, a 1H sprayer nozzle, no `flail`, and retires `axe` / `mace` / `wand`. The row says "per §3.1/§3.2", so the data follows the spec. (2) **Soft penalty not fully retired** (§1): catalog units have no passives field, so the five enemies whose drills live on their `sunk_ward` characters (`calcified_husk`, `guild_master_orlov`, `overseer_grendel`, `mortuary_flagellant`, `proctor_quell`) still wear locked gear when fought from the catalog; dropping the soft-penalty fallback would give them full power (`PROF_GEAR_MULT[0]` = 1.0), so it stays for gear worn while locked and for role-less classes (`tl_*`); the content loadout check does not error on it. (3) **Numbers chosen by Build** (no spec values): `reload`, `fade_step`, `cite_passage` rows (`reload` FOCUS; `fade_step` jump 1; `cite_passage` fortify +3 for 2 rounds, range 0-2), ammo `power_mult` 1.25, `buckler` mitigation 4 / burden 0, `braces` 3 / 1, the four new item names (`hip_quiver`, `fuel_canister`, `ash_cloak`, `forearm_braces`), the keepsake trims (Kaelen's stun resist, Vesper's root resist), wands -> sprayer (a mage's spell sourcing may change), `penal_foundry_apron` -> heavy (mitigation 6 -> 10, burden 2 -> 4). No `off_hand_skill` on `orb`, `shield`, `buckler`, `braces`: spec §4 names Shield Bash, Riposte, Brace and Catalyst Pulse, but `shield_bash` / `riposte` / `brace` already sit in class paths (the duplicate-skill guard) and **Catalyst Pulse has no skill row** (an open spec gap). Also chosen by Build: the `knife` family's reach band 1-2 and the `knife_throw` row (1 block, 1 AP, reach 2-3, knife-only through `weapon_types` + `min_proficiency` 1). (4) `deserters_kettle_helm` and `nemeths_warded_cassock` (named in the row) do not exist in the catalog. (5) **Enemy and preset loadouts the strict rule refuses** (pinned in `emberwake_world/tests/strict_loadouts.rs`; heroes, founders, start parties and class defaults are all legal): `shd_buckler` and `arm_leather` require DEX 10 and a warrior has 9 even with the Prof 1 discount, so these warrior NPCs wear a locked piece: ashlands `mara_voss` (`ember_brigandine`), `hest_the_quartermaster` (`hest_buckler`), `brask_toller` (`brask_buckler`), iron_reaches `orsa_tallow` (`orsa_buckler`), tidewrack `ilse_marrow` (`arm_leather`), catalog `aberrant_test`; Dojo presets no Dojo hero (warrior, ranger) can wear whole: `dual_wield` (daggers; warrior DEX on leather), `duelist` (off-hand dagger), `arcanist` (wand -> sprayer, tome), `titan_deficit` (warhammer STR 22 vs 19; its stat-deficit showcase is now a hard lock, so the piece is skipped). The kit content check does not run `wear_lock`, so `content check` stays clean on these.
- **Recommendation**: (1) confirm the spec matrix (recommended) or correct it. (2) Option A (recommended): a kit row adding unit-level drill passives (or the check resolving a unit's world character), then drop the soft fallback, add a strict loadout content error and retire the `untrained_*` ruleset fields (a contract change); Option B: keep the fallback for catalog-only fights. (3) confirm or give numbers. (4) author those items or drop them from the row. (5) per piece: lower the DEX 10 requirement, retag / swap the gear, or give the NPC a drill (moves goldens: a named re-record); presets: add Dojo heroes for the other classes, a preset class field, or keep them as skipped-piece showcases; and a content check running `wear_lock` on loadouts.

### FB-268: PR5-51b gear rework sanity sweep

- **Spec**: READY row PR5-51b (3); FB-248, FB-253
- **Kind**: sim
- **Status**: open
- **Finding**: Pre = 456b0ee1 (PR5-51a kit, old data), post = the PR5-51b data, 10 seeds; 0 PLAN FAIL / SKIP everywhere; every class still attacks. `sim feel` (the 5 `tl_*` fixtures): 62% -> 64% won, `tl_duel_windup` 2/8 and `tl_duel_armour` 0/10 unchanged (old fixture behaviour). `sim party` (warrior, ranger, priest, mage, level 5), clear %: mob 100 -> 100, line 90 -> 70, wave 100 -> 100 (above its 55-80 band, as before), elite 100 -> 90 (above 50-75), boss 50 -> 80 (above 30-50). Damage dealt per fight: mage up 30-85% (boss 214 -> 396; the orb's INT override is the likely cause), ranger down 10-25% (still top or second), warrior down on 4 of 5 (mob -28%, boss -16%) while damage taken drops on 4 of 5 (elite -23%), priest damage taken roughly halves on line / wave. Act 1 world sweep (won / lost / capped / fled): prologue 10/10 each; Corvo's Gate 5/0/2/3 -> 8/0/1/1; M1 1/4/3/2 -> 2/2/4/2; M2 5/1/4/0 -> 6/1/3/0; M3 1/4/1/4 -> 2/4/0/4; M4 2/4/2/2 -> 4/2/2/2; M5 Fissure 0/0/5/5 -> 3/1/4/2; Crucible 0/6/2/2 -> 0/5/3/2; overlook 0/7/2/1 -> 0/6/2/2; total 40% -> 50% won. Not observed: ammo empower, Reload (only in `tl_skirmish`), the knife Throw and the orb have no `counts=on` counters and barely appear in the world fights.
- **Recommendation**: Options for Design: (a) boss above its band (80%): trim the mage's boss damage or the party-vs-boss multiplier, or boss HP +30-40%; (b) warrior still takes the most damage: a small armour / HP bump or some melee power back; (c) the Crucible stays at 0 wins (pre-existing, FB-253: human-play verification); (d) a small kit row adding `counts=on` counters for empowered shots, Reload, Throw and orb swings so the next sweep can see them.

### FB-269: PR5-52 `RULESET-MODES1` landed

- **Spec**: READY Section 1 row PR5-52; FB-223, `P-ARCH-02` (the `PR5-52` squash commit on `develop`, 2026-11-15)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `games/emberwake/data/catalog/rulesets.json` ships only `timeline` (seed `seeds/rulesetmodes1.json`: remove `classic` `if_exists`, upsert `timeline` in the `modes` form); `classic` is the kit built-in preset (FEEDBACK kit README line), equal to the old row field for field (tested), so Ashlands, Iron Reaches and Tidewrack (no `ruleset` named, so classic) fight exactly as before; no world, scenario, playtest, `sim.json` or `game.json` named `classic`; `aegis` / sim `ruleset=classic` still resolve; CLN-44's `rulesets:missing` warning and classic load note retired; Dojo / Arena pickers list `known_ids` (Timeline, Classic). (2) Typed `modes` (`poise_mode` -> `modes.poise`, the others keep their names), flat keys as load aliases, the writer stores under `modes`; the check refuses lists / unknown values / disagreeing aliases (mutually exclusive). There is no `intent_mode` field; the exclusive set is `poise_mode`, `attack_lock`, `melee_diagonal`, `flank_protected_cost`, `rest_ap`, `chase`. **`rules_contract_golden` re-recorded** (named by the row): the flat lines and row-only `poise_mode` replaced by `modes` + 6 `modes.*` lines; `rest_ap_*` and pip `needs=` renamed. `checks_golden`, `journey_golden` and every battle / sim golden unchanged. Tests: kit `ruleset_modes.rs` (5), `rules_report` `the_contract_lists_the_modes`; game `classic_ruleset.rs` `the_game_ships_one_ruleset_and_classic_is_the_kit_preset`; `ruleset_switches.rs` replay ends with `rulesetmodes1.json`; renames in battle / sim / GUI tests. Gate 3394 passed, 0 failed.
- **Recommendation**: HISTORY: "PR5-52 (FB-223, `P-ARCH-02`): the game ships one ruleset row (`timeline`); `classic` moved to the kit built-in preset; CLN-44's `rulesets:missing` warning and classic load note retired; seed `rulesetmodes1.json`." "PR5-52: mutually exclusive switches moved under a typed `modes` (`poise_mode` -> `modes.poise`); flat keys are load aliases; tokens `modes.<key>=<v>`; `rules_contract_golden` re-recorded." GAME_BRIEF / engine README rules: "`classic` is the kit's built-in preset; a game ships one ruleset row; exclusive switches live in `modes`." STRUCTURE: `rpgkit_content/tests/ruleset_modes.rs`; seed `rulesetmodes1.json`. Decisions ("Ruleset placement"): classic is a kit preset, not a row. Open: (a) a typed Forge editor for `modes` (today a JSON editor; the engine schema has no object-of-enums type); (b) Dojo / Arena pickers still offer Classic (dev tools) — keep or list only the game's row; (c) retire the old classic-row seeds (`cln44`, `cln45`, `attacklock1`, `p12s1`; re-applying one brings the row back) in a cleanup pass.

### FB-270: PR5-53 `VFX-DATA1` landed

- **Spec**: READY Section 1 row PR5-53; FB-231 (d), `P-ARCH-03` (the `PR5-53` squash commit on `develop`, 2026-11-15)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit catalog kind `fx_presets` (FEEDBACK kit README line); seed `seeds/vfxdata1.json` (42 presets: 25 action looks incl. `reach_bolt` and `flask`, 17 status looks) applied to `data/catalog/fx_presets.json`. `games/emberwake/crates/emberwake_gui/src/fx.rs` holds no per-preset table (removed `BLOWS`, `SHOTS`, `STRIKES`, `SUPPORTS`, `STATUS_LOOKS`, reach bolt, flask and the hues only they used); generic values stay in code (hit flash, crit / hit-for-0 colours, fall dust, sidestep / overshoot, volley / wave gaps, status apply burst, head anchors). A look key with no row draws its role's neutral look (`NEUTRAL_BLOW`, `NEUTRAL_SHOT` / strike, `NEUTRAL_SUPPORT`, `NEUTRAL_STATUS`); shipped data never reaches them (content check `data ok`, `fx_warnings`). The status table stays global (`install_status_looks`; threading it through every token / board call is invasive), with pure `status_look_in` / `marks_in` beside it. Parity: the old tables were frozen into `emberwake_gui/tests/fixtures/fx_library.json` before deletion; `fx::tests::the_data_draws_every_frozen_library_look_exactly` pins the shipped `fx_presets.json` (and the seed) to it field for field, incl. delivery / impact / aim effects under crit, miss and hit-for-0 and the status table order (a swapped colour fails it). Deviation: parameters live in one shared table keyed by look key instead of fields on the skill / item family / status rows (keys are shared across many rows, and the kit's `statuses.json` cannot be written through `aegis content`). Tests: kit `fx_preset_checks.rs` (6), `looks.rs` (+2); GUI parity, neutral-look and status-order tests; three assertions changed for the new fallback (an unknown key now draws the neutral look instead of slash / mend). Gate 3408 passed, 0 failed; 0 golden diffs.
- **Recommendation**: GAME_BRIEF ("where to look"): "battle VFX parameters: `data/catalog/fx_presets.json` (kind `fx_presets`); `fx.rs` holds no per-preset table; a key without a row draws its role's neutral look and `aegis content check` warns." STRUCTURE: `data/catalog/fx_presets.json`; `emberwake_gui/tests/fixtures/fx_library.json` (the frozen effect library, re-recorded only for an intended look change); seed `vfxdata1.json`; kit `check/fx_preset.rs`, `tests/fx_preset_checks.rs`. HISTORY: "PR5-53: the `fx.rs` look tables moved into `fx_presets` data (frozen in `fx_library.json`); an unknown look key now draws a neutral look per role instead of the default key's." Open: (a) the shared table vs per-row overrides; (b) the four neutral look designs (subtle placeholders); (c) generic values (fall dust, apply burst) could become presets later.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
