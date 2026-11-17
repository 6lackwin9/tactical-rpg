# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to PR6-3, PR6-5 (excl. PR6-4; `FB-01`..`FB-281`, `UN-01`..`UN-12`) | released or done | `main` / `develop` (`f132eabc`) | Archived in `docs/archive/FEEDBACK_CLOSED.md` (moved 2026-11-17) |
| `PR6-4` (`PK3-RECRUITS-KEEPSAKES`) | done with parked clauses (`FB-282`, `FB-283`, `FB-284`) | `develop` (squash `PR6-4`) | Kit recruits/pool/crypts/fuse/keepsakes + `sunk_ward` seed `pr6_4.json`; temperaments, Martyr fuse data and the Retire/Last Stand prompt parked |

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

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
