# `docs/archive/ARCHIVE_INDEX`: Completed Specs, Queues & Historical Archives

Maintained by `/design-session` (`CTX-CLEANUP1-FINAL`, 2026-11-16). Agents must **not** read archived specs when building; current contracts live in [GAME_BRIEF.md](../../games/emberwake/docs/GAME_BRIEF.md), [DESIGN_REVIEW.md](../DESIGN_REVIEW.md), [READY.md](../READY.md), [DECISIONS.md](../DECISIONS.md), and active files under `docs/specs/` / `docs/sessions/`.

## 1. Wave 1 Completed Session Specs (`docs/archive/sessions/`)

Moved from `docs/sessions/` on `2026-11-16` (`PR5-51a..62c`):

| File | Scope & Landed Commits | Current Owner Doc |
| --- | --- | --- |
| [U3_GEAR_REWORK.md](sessions/U3_GEAR_REWORK.md) | `GEAR-REWORK1`: 15 weapons, 7 off-hands, 3 armor weights, strict `Prof 0`/`required_stats` locks, `PR5-51a` (`456b0ee1`), `PR5-51b` (`30415b91`), `PR5-57` (`3d9ca3cb`), `PR5-60` (`22e6ce6c`, `a47d4042`) | [GAME_BRIEF.md](../../games/emberwake/docs/GAME_BRIEF.md), [HISTORY.md](../HISTORY.md), [DESIGN_REVIEW.md](../DESIGN_REVIEW.md) |
| [U2_CLASSPASS_TECHNIQUES.md](sessions/U2_CLASSPASS_TECHNIQUES.md) | `CLASSPASS1` index, 7 off-hand skills (`catalyst_pulse`), `FB-268(a)` 3-step orb fix, slices `PR5-62a` (`6e662c53`), `PR5-62b` (`a1b1efb1`), `PR5-62c` (`48f24342`) | [DESIGN_REVIEW.md](../DESIGN_REVIEW.md), [GAME_BRIEF.md](../../games/emberwake/docs/GAME_BRIEF.md), [HISTORY.md](../HISTORY.md) |
| [U2_CLASS_DECKS.md](sessions/U2_CLASS_DECKS.md) | 6 class decks (`1` basic + `3` flat actives + `2` `ClassDef::passives`), `riveted_gauntlets`, `twin_light_tempo`, disposition of `47` flattened/retired `ranks` (`PR5-62a..b`) | [GAME_BRIEF.md](../../games/emberwake/docs/GAME_BRIEF.md), [DESIGN_REVIEW.md](../DESIGN_REVIEW.md) |
| [U2_WEAPON_TECHNIQUES.md](sessions/U2_WEAPON_TECHNIQUES.md) | 15 weapon families × 5 single-rank skills (`3` Mastery + `MW` + `Cold Steel Keystone` = `75`), retiring `6` `tq_*` stubs (`PR5-62c` `48f24342`) | [GAME_BRIEF.md](../../games/emberwake/docs/GAME_BRIEF.md), [DESIGN_REVIEW.md](../DESIGN_REVIEW.md) |
| [U2_GRAFTS_SKILLS.md](sessions/U2_GRAFTS_SKILLS.md) | `PK1/PK2` Monster Ossuary pins (`6`), Class Cranial Rituals (`6`), Inoculated Keystones (`6` incl. `graft_purge`/`purge_marrow`), `30` surgery recipes (`PR5-62a`, `PR5-62c` `48f24342`) | [GAME_BRIEF.md](../../games/emberwake/docs/GAME_BRIEF.md), [DESIGN_REVIEW.md](../DESIGN_REVIEW.md) |

## 2. Queue, Feedback, Log & Report Archives (`docs/archive/`)

| File | Archived Scope |
| --- | --- |
| [READY_DONE_ARCHIVE.md](READY_DONE_ARCHIVE.md) | Completed `READY.md` §1 slices `U1..PR5-62c` (`113` rows through `PR5-62a` `6e662c53`, `PR5-62b` `a1b1efb1`, `PR5-62c` `48f24342`) |
| [READY_ANSWERS_ARCHIVE.md](READY_ANSWERS_ARCHIVE.md) | Closed `READY.md` §2 audits (`1..8`), §3 `PR4-1`/`PR4-2` notes, §4 answers (`FB-33..277`, `AO-01..10`) |
| [FEEDBACK_CLOSED.md](FEEDBACK_CLOSED.md) | Closed `FEEDBACK.md` entries `FB-01..277` and `UN-01..12` |
| [HISTORY_2026_10_11A.md](HISTORY_2026_10_11A.md) | Pre-`PR5-40` `docs/HISTORY.md` rows through `2026-11-12` |
| [DESIGN_REVIEW_2026_11A.md](DESIGN_REVIEW_2026_11A.md) | Settled `DESIGN_REVIEW.md` `2026-11-11..14` calls and Reviewed history |
| [GAME_BRIEF_NOTES_2026_11A.md](GAME_BRIEF_NOTES_2026_11A.md) | Pre-cleanup per-slice and Godot-transition notes from `GAME_BRIEF.md` |
| [STRUCTURE_NOTES_2026_11A.md](STRUCTURE_NOTES_2026_11A.md) | Verbose per-PR crate/test changelogs from `STRUCTURE.md` |
| [BALANCE_REPORT_2026-11A.md](BALANCE_REPORT_2026-11A.md) / [BALANCE_REPORT_2026-10.md](BALANCE_REPORT_2026-10.md) | Pre-`U3_GEAR_REWORK` (`PR5-27..BOSS1-B`) and October 2026 balance runs |
| [CLEANUP_PASS1.md](CLEANUP_PASS1.md) / [CLEANUP_PASS2_2026_11.md](CLEANUP_PASS2_2026_11.md) | Completed Pass 1–3 cleanup backlog tables |
| [WORLDBUILDING_2026_10.md](WORLDBUILDING_2026_10.md) | Pre-`CANON.md` 2026-10-21 worldbuilding snapshot |
| [README.md](README.md) | Full archive folder directory, including legacy Godot files, `specs/`, `specs/emberwake/`, and `handover/` |
