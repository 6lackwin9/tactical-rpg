# Ready Queue (Design Session writes, Build Session reads)

Single active queue for the Build Session under [WORKFLOW.md](WORKFLOW.md) (`2026-11-07`):

- **Ownership**: Design writes `READY.md` and shared specs; Build runs `ready-to-build` / `feasibility-check` slices and writes updates/deltas exclusively to [FEEDBACK.md](FEEDBACK.md) (archiving closed items in [FEEDBACK_CLOSED.md](archive/FEEDBACK_CLOSED.md)).
- **Build Lease Guard**: Before resuming or pushing, run `python tools/planner/feedback_check.py lease check`; halt immediately on exit `1`.
- **Art/UI Ownership & Placeholders**: `ART_UI_REQUESTS.md`, `tools/gen_music.py`, audio files, and `assets/**/README.md` are Art/UI-owned. Missing art/audio uses a working placeholder copied to the final path and logged in `FEEDBACK.md` so Art/UI replaces it in place.
- Do not build directly from legacy docs unless queued here.

## 1. Ready to Build (`ready-to-build`)

Execute these slices in order. Existing engine goldens must remain unchanged except where a named golden re-record is specified. `R1`..`R4`, `PR4-1..PR4-9`, and `PR5-1..PR5-62c` (`113` slices through `48f24342`) are `(done)` and archived in [READY_DONE_ARCHIVE.md](archive/READY_DONE_ARCHIVE.md). No open Wave 1 ready-to-build slices remain.

| Order | Task ID | Masterplan | Layer | Summary & Spec | Acceptance Test |
| --- | --- | --- | --- | --- | --- |

## 2. Feasibility Check (`feasibility-check`)

All `25` sub-blocks across all `9` blocks of `MASTERPLAN1`, `WEBREL1` (`FB-147`), and `ACTS23-R5` (`FB-167`) are audited (`FB-01`..`FB-59`, `FB-147`, `FB-167`). Closed Section 2 rows (`1..8`) are archived in [READY_ANSWERS_ARCHIVE.md](archive/READY_ANSWERS_ARCHIVE.md).

## 3. Held / Post-Release Pre-Staged Queue (`Post-R4` Optional Extensions)

Post-Act-1 or `.1`-gated slices: Build takes no action until promoted to Section 1. Pre-split `Original row:` notes for `PR4-1`/`PR4-2` live in [READY_ANSWERS_ARCHIVE.md](archive/READY_ANSWERS_ARCHIVE.md) §3.

| Wave 2 Order | Task ID | Masterplan | Summary & Dependencies | Named Golden Re-record |
| --- | --- | --- | --- | --- |
| Post-R4 | `BASERULES1-PP` | `U1.2`, `S1.2` | Parked (`FB-79`): optional 3-stat world mode merging `STR+CONS`, `DEX+PERC`, `INT+KNOW` (plus `Composure` `shaken` at 0 / Rest `+5`). | `rules_golden`, `timeline_golden`, `sim_golden` |
| Post-R4 | `R3-STAGE-SEED` | `W.2`, `E1.2`, `T1.2` | **Split (`PR4-1` shipped `stage=N`, `AUDIT-07`)**: holds campaign `growth_per_stage` values and interactive-story `roll` choice. | `journey_golden`, `sim_golden` |
| Post-R4 | `POST-R4-CLEANUP` | `U1.2`, `U2.3`, `C27`, `C28` | **Split (`PR4-2` shipped items 1,3,4)**: holds item (2), playthrough review of `goblin_skirmish` (`AUDIT-09`) and 5 explicit-`spend` enemies (`ash_stalker`, `cinder_hound`, `ember_wraith`, `gilded_Automaton_Sentry`, `sunken_dredger`). | `timeline_golden`, `rules_golden`, `sim_golden`, `phasefold_golden` |
| Post-R4 | `ACTS23` | `S2`, `U3`, `E3`, `W3`, `T1`, `T2`, `F`, `P` | **Answered (`2026-11-10`, Channel fuel row, 5.1 Option B)**: [ACTS23.md](specs/ACTS23.md) proposes `PK1`..`PK8` (`R5`..`R7`); needs per-package `.1` sessions before queueing. | - |
| R5 | `PK3` slices (`PR5-11`, `PR5-14`, `PR5-15`, `FB-167`) | `E3`, `F` | **Held until PK3 `.1` session**: `PR5-11` bleed-out/revive, `PR5-14` recruit generator, `PR5-15` procedural keepsakes. (`PK2` queued in Section 1.) | - |
| Later | `INTEG-ASSETS1` | `X3`, `X8` | **Held**: post-R5-R7 integration pass (`integration` agent + `aegis scenario`) verifying replaced Art/UI assets against sizes, clip/frame counts, loop tags, palettes, names. | none expected |
| Later | `PK2-STAGE-PASSIVES` | `U4`, `S2` | **Held (`FB-221(a)`, `PK2_GRAFTS.md` 2)**: author `calcify_stage_2`/`3` passives on `unit_tracks.calcification` during `PR6`/`CAST2`. | none expected |
| Later | `ENGINE-PURITY-2` | `X6` | **Held (`FB-227(c-e)`, `P-ARCH-01`)**: optional cleanup for non-turn-based second game (`TimelineStyle.hatched`, `bridge.rs`, `aegis_ui::timeline` labels). | none expected |
| Later | `RULESET-CLEANUP` | `R1` | **Parked (`FB-269(a),(c)`, `P-ARCH-02`)**: optional typed Forge `modes` editor & retiring remaining `classic` legacy seeds (`cln44.json`, `cln45.json`, `attacklock1.json`, `p12s1.json`). | none expected |
| Later | `VFX-DATA2` | `FX1` | **Parked (`FB-270(b),(c)`, `P-UI-02`)**: neutral-look tint/glow (`buff` vs `heal`) & shared generic `fall_dust`/`apply_burst` presets. | none expected |
| Later | `ASSETORG4-2` | `A11` | **Parked (`FB-273(c)`, `AO-08`, [ASSETORG4.md](archive/specs/ASSETORG4.md) §1.1)**: finish moving kit-owned assets (`items/<weight>` rename `plate`/`leather`/`cloth` -> `heavy`/`medium`/`light`, generic textures/SFX) to `kits/rpg/assets/` or `engine/`. | none expected |
| Later | `WAVE2-BALANCE` | `U2`, `E2` | **Tracked (`FB-277`, `PR5-62c` `48f24342`)**: Wave 2 balance pass covering `sim party 5 10` (`boss 20%` vs `sim party 5 30` `33%` / `sim sweep 5 10` `34%`), `brawler` swap delta (`-9.4`), `line 92-95%` (post-Act-1), and `[PROVISIONAL]` Mage AoE scalars (`0.28/0.28/0.28/0.22`). | `sim_golden`, `rules_golden` |

## 4. Feedback Responses (Design answers to `docs/FEEDBACK.md`)

`FB-01`..`FB-277` and `UN-01`..`UN-12` live in [FEEDBACK_CLOSED.md](archive/FEEDBACK_CLOSED.md); `FB-33`..`FB-277` and `AO-01`..`AO-10` answers are archived in [READY_ANSWERS_ARCHIVE.md](archive/READY_ANSWERS_ARCHIVE.md). Zero open Build items remain (`48f24342`).

| Feedback ID | Disposition | Follow-up | Design Decision / Response | Date |
| --- | --- | --- | --- | --- |
