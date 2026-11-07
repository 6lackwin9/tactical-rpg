# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4. |
| R1 path (READY R1-8..R1-9) | W2-3 slice 1 done; slice 2 (`S2.2-S3` pips on) in review | `claude/s22s3-pips-on` | Then slice 3, AUDIT-R1 (AUDIT-01..06), the R1 release PR. |
| READY rows 14-20 | queued (post-R1) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-84 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-85. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-85: S1.2-S4 (W2-3 slice 1) built: Guard Option 1C; four named re-records; three readings to confirm

- **Spec**: `docs/sessions/S1_STATUS.md` (guard, Option 1C, section 3, acceptance test 2); FB-48 (d)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: A Defend's guard holds from its wind-up start to block 6 of the round in every ruleset (`trigger::hold_defend` on `HeldGuard`); `Unit.guard` = max(running Defend, held guard, `guard` status), never summed; a Poise break or the blocks a stun locks end it (break removes the status too). Multi-round guards (row-only skill key `guard_rounds` 2-5) queue the 18th status `guard` (Guarded, `modifier`, 10-90 %) for their later rounds. Fractions stay per skill on `mitigation` (shipped 0.05-0.55; 0 = none); ruleset numbers `guard_common_mitigation` 0.35 / `guard_multi_round_mitigation` 0.20 are defaults for a row without `mitigation`; cap stays 0.9; the held guard is the skill's authored fraction, unclamped; a later-round status is capped at 90 % and not queued below 10 %. The AI skips a redundant Defend while a `guard` status holds. No shipped multi-round guard (`brace` is INTENT2-A, W2-4). Named re-records, each traced to the 1C hold (a probe with it off matched the old goldens): `timeline_golden` (3 of 15 fights), `sim_golden` (11 of 22 runs), PHASEFOLD (18 of 72 fights), `rules_contract_golden` (+2 tunable, +2 promoted). Tests: `guard_timing.rs` (6), `guard_rows.rs` (1), `intent_ai.rs` (+1).
- **Recommendation**: Confirm: (1) later rounds use the skill's own fraction (the loader fills `mitigation` with the schema default 0.5, so the 0.35 / 0.20 defaults never reach shipped rows), or always `guard_multi_round_mitigation`? (2) a break removes the queued later-round status too? (3) a schema entry for `guard_rounds` with INTENT2-A (moves `checks_golden`). (4) `guard` is now a buff that encounter start buffs, phase effects and hook `grant` accept (10-90): intended? (5) a stun and a stance break now also end trigger-held guards (triggers ruleset); (6) the round-intent AI may still plan two Guards in one round. Placeholder steel tint for the `guard` look. Docs delta: GAME_BRIEF guard line as above; HISTORY: "S1.2-S4: Guard Option 1C; re-recorded on purpose: `timeline_golden`, `sim_golden`, PHASEFOLD, `rules_contract_golden`"; STRUCTURE: the two tests.

### FB-86: S2.2-S3 (W2-3 slice 2) built: the timeline plays Poise pips; one fixture key; three named re-records

- **Spec**: `docs/sessions/S2_POISE.md`; FB-49 (c), FB-71
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `s22s3` sets `poise_mode: pips` on the shipped timeline ruleset; the frozen fixture `timeline_kit.json` (which the timeline and sim goldens read) gets the same one key, set by script (only that line changed). FB-71 done: under pips the value mirrors the pips (`value = pips x T`, so a full bar can read above Max Poise, 65 against 64), Rest restores pips only (1 per 2 blocks), the windup bonus pip count is stored on the command (`Command::poise_bonus_pips`) and may exceed `poise_pips`, the Rest preview (`RestGain.pips`), battle marks, tips, forecasts ("Pips 5 -> 4", "under T 12"), the party card and the Dojo show pips (screenshot sent to the user). `sim feel` now counts the blocks a unit starts guarded (`Resolver.guarded`, Guard 1C), not the Defend's span. Named re-records: `sim_golden` (run 21 feel guard-block counts, the feel fix), then with the flip `timeline_golden` (15 of 15 fights; first: "Stagger 30 (poise 64 -> 34)" becomes "(poise 65 -> 39)") and `sim_golden` (run 21 `tl_skirmish` capped becomes won). Break consequences unchanged until slice 3. Tests: `poise_pips.rs` (+3), `poise_pips_timeline.rs` (5), a GUI pips test.
- **Recommendation**: Confirm the mirrored value may read above Max Poise (or hold it at max). Note for HISTORY / GAME_BRIEF: under pips the `stagger_dealt` metric ("poise taken") sums the mirrored drop; the feel report's "guard blocks" now counts every guard source (Defend, held guard, `guard` status, triggers) in every ruleset; the retired known-issues row 46 (shared target dir) could be reinstated for the cloud `target/`. Known-issues row to add (skill, user approval): "a repo copy built into the same `CARGO_TARGET_DIR` leaves test binaries whose `CARGO_MANIFEST_DIR` points at the copy, so main-tree runs read the copy's fixtures; use a separate target folder, or touch the sources and rebuild" (it hid a failing `ruleset_switches.rs` test here, since fixed). Docs delta: GAME_BRIEF poise line: the timeline plays pips since S2.2-S3; the value mirrors the pips; Rest restores pips; the view shows pips; HISTORY: "S2.2-S3: `sim feel` guard blocks follow Guard 1C; the timeline ruleset (shipped and frozen fixture) plays pips; re-recorded on purpose: `sim_golden`, `timeline_golden`"; STRUCTURE: `poise::mirror`, `rest_pip_gain`, `Resolver.guarded`, test `poise_pips_timeline.rs`, GUI `pip_view`.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-06 are closed and moved to the archive (2026-11-07). New notes start at UN-07.
