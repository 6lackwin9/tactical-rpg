# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | done (FB-95) | `develop` | 3 parallel lanes + FB-94 additions; checks_golden re-recorded. |
| R1.1-2 `C02-A` | done (FB-96) | `develop` | asset + logic lanes in parallel; phasefold re-recorded. |
| R1.1-3 `R1-BAL` | done (FB-99, FB-100, FB-103) | `develop` | Phase R1.1 complete. |
| READY row 14 `A2.2-S1/S2` | done (FB-102) | `develop` | - |
| READY row 16 `A4.2-K1a` + `A3.2-S1` | done (FB-105; `field_intent` gate moved to W2-4 per FB-106) | `develop` | R2-W1 batch 1 complete. |
| READY row 19 `I1.2-S1` | done (FB-104) | `develop` | - |
| READY row 15 `A1.2-S1/S2` | done (FB-107) | `develop` | - |
| READY row 17 `F1.2-S1` + `F2.2-S1` | done (FB-108) | `develop` | - |
| READY row 18 `P1.2-S1` + `P2.2-S1` | done (FB-109) | `develop` | R2 Wave 1 complete. |
| READY row 20 (R3-W1) | done (FB-111) | `develop` | R3 Wave 1 complete. |
| W2-4 (R2-W2) | slice 2 done (FB-112); slice 1 (`field_intent`) building | worktree | Slices: field_intent; FB-107 text; K2 snap + retire SWEEP; INTENT2-A seed; A4.2 EffectSpec; K1b Field planner + UI; K3 items + INTENT2-U. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-111 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-112. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-112: W2-4 slice 2 (FB-107) landed: stances-off skill text drops the stance token and its separator

- **Spec**: `docs/READY.md` row W2-4 (the FB-107 clause); `docs/sessions/A1_VALUE.md` acceptance 2
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `source::render(.., stances = false)` drops a run of empty stance tokens (`{flow}`, `{disrupt}`) with one separator (the preceding `, ` / `; `, else the following one), plus a space left before punctuation; `"power, {flow}. Needs"` reads `"power. Needs"`. New `source::skill_description(row, stances)`: stored text with stances on; with stances off the rank's template re-rendered when it uses a stance token and the stored text is its normal render (a literal text is kept, the same literal-vs-rendered rule the store and checks use). Ruleset-aware callers pass the ruleset's `stances()`: `rpgkit_host` `skills_of` (feeds the battle hover tips; an absent description is now `""`, not `null`), the battle pick tip, the party Skills (3 sites) and Deck tabs; the catalog writer, the store comparison and the checks keep `true`, so shipped data and the resave are byte-identical. Under the shipped timeline ruleset the two skills that use the tokens show neither the token text nor a leftover comma; classic shows them as stored. `rpgkit_host` now depends on `rpgkit_content` (no cycle). 0 golden diffs. Tests: `stance_tokens.rs` (+3, 1 updated), `emberwake_world/tests/skill_text.rs` (1).
- **Recommendation**: kit README / GAME_BRIEF content: the stances-off separator rule and `source::skill_description`; which callers pass `stances`. STRUCTURE: test `emberwake_world/tests/skill_text.rs`. HISTORY: "W2-4 slice 2 (FB-107): stances-off skill text no longer leaves `power, .`; no golden moved". Notes for later (no shipped template is affected): `"a {flow}, b"` reads `"a b"`, a leading token leaves a lowercase start, and only `,` / `;` count as separators; item rows from the host can still send `null` descriptions; `skill_description` could move to a lighter crate than `rpgkit_content`.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
