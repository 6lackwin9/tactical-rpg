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
| W2-4 (R2-W2) | slices 1, 2, 3A, 5 done (FB-115, FB-112, FB-113, FB-117); K2 phases B-C parked on FB-116 | `claude/w24-k2bc` | Next: slice 6 (K1b Field planner + UI); then 4 (after K2), 7. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-114 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-115. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-115: W2-4 slice 1 (FB-106) landed: rule `field_intent`; `intent::allowed()` gates on it; timeline lists it

- **Spec**: `docs/READY.md` row W2-4 (K1b gate clause); FB-106; `docs/specs/INTENT2.md` (INTENT2-K1), `BASERULES1_LIBRARY.md`
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `ruleset::FIELD_INTENT` registered: an Option (the spec is silent on the kind; Option matches `round_intent`, it changes core planning, and is never in classic), requires `round_intent` (alone it is an error note and a content-check error, as `flank` does), system planning, no bundle yet (the library's `field` bundle holds rules that do not exist yet); placed after `plan_badges`, before the hook rules. `intent::allowed()` gates on `field_intent` (round_intent alone keeps the legacy single intent); nothing calls `allowed()` yet (K1b's planner will). Schema rulesets `rules` options and help list it. Data via seed `games/emberwake/data/seeds/w24s1.json` (aegis content apply): timeline `rules` gains `field_intent` after `plan_badges`; classic stays `[]`. Re-recorded (named): `rules_contract_golden` (+ the option value, + the rule line). Every other golden byte-identical (the frozen `timeline_kit.json` does not need the rule while nothing calls `allowed()`). Tests: `rpgkit_battle/tests/field_intent.rs` (3), `ir21_intent.rs` gate row, `rulesets.rs` every-rule row, `ruleset_switches.rs` replays `w24s1`. A tested set of the 18 pre-`field_intent` timeline rules was kept during the build and is now unused (harmless; remove in a cleanup, which shortens `TESTED_SETS`).
- **Recommendation**: HISTORY: "W2-4 slice 1 (FB-106): rule `field_intent` (option, requires `round_intent`); `intent::allowed()` gated on it; timeline lists it (seed `w24s1`); re-recorded on purpose: `rules_contract_golden`". GAME_BRIEF / BASERULES1 library: `field_intent` moves from coming options to the register. STRUCTURE: test `rpgkit_battle/tests/field_intent.rs`. Confirm Option (not Extension) as the rule kind.

### FB-116: K2 phases B-C move `phasefold_golden` (not named in W2-4): the Goblin Berserker stops planning Sweep Kick

- **Spec**: `docs/READY.md` row W2-4 (named re-records: `checks_golden`, `timeline_golden`, `sim_golden`, `rules_contract_golden`); FB-114 (migrate SWEEP, accept classic kicks firing)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Phases B and C are built on `claude/w24-k2bc` (B: seed `w24k2` migrates the 7 shipped SWEEP skills, `tendril_flail` `round_intent` "", the two FB-114 templates, `cast1a3` follows; C: SWEEP removed from `Area`, ground `Shape`, `HAZARD_SHAPES`, the schema and checks, a stored SWEEP is an unknown-area error; the frozen kit's 6 kick rows are RING; `ir21_intent.rs` asserts 26 = 8 + 18 and the primary totals 59 / 23 / 16 / 12 / 14 / 5 on shipped data directly, with `tendril_flail` now Stand). Re-recorded (named): `sim_golden` (12 of 22 runs, 99 lines; the classic brawler kicks now fire, as FB-114 accepts) and `checks_golden` (2 lines; SWEEP leaves the area enum message); `timeline_golden`, `rules_golden`, journey, contract and resave unchanged. But `phasefold_golden` (reads shipped data, not named) moves in 49 of 72 fights, from the phase B data alone (the same 49 with the phase C code removed): first `catalog:goblin_ambush {} balanced seed 1` round 2, the Goblin Berserker planned "Sweep Kick" x3 while the kick was SWEEP and now plans "Body Blow II" x3, because a RING centred on itself scores nothing with no foe adjacent. Build did not re-record it; full gate fails on that test only.
- **Recommendation**: (A, recommended) accept the move as part of FB-114's migration and name `phasefold_golden` for W2-4 slice 3: re-record it with a HISTORY row (the Berserker's ring kick is only worth using with a foe beside it, so it picks Body Blow II in these openings). (B) treat it as an AI gap: score RING by the foes the user could have adjacent after its move this round (a planner change in `ai.rs`, then re-check the move). Build lands phases B-C as soon as one is chosen; W2-4 slice 5 (A4.2) proceeds meanwhile.

### FB-117: W2-4 slice 5 (A4.2) landed: one `EffectSpec` for the five status vocabularies; no behaviour change

- **Spec**: `docs/READY.md` row W2-4 (`EffectSpec` across 5 vocabularies); `docs/sessions/A4_EFFECTS.md` section 1, acceptance 2; C08, FB-57(5), FB-58
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `rpgkit_rules::status::{EffectSpec { status_id, potency, rounds, magnitude, resistable: Option<bool>, source }, Timing { Now, Next }, apply_effect, grant_timing}`. Converters: skill `inflict` (`EffectSpec::inflict`, next round, KNOW bonus; command.rs Attack / Area, dojo.rs), skill `grant` (`EffectSpec::grant` + `grant_timing`: a DEFEND buff now, otherwise queued; command.rs Defend / Support, `status::grant_extras`), `trigger::effect_of` (inflict potency 100 / rounds 1 next round; grant rounds >= 1, negative = `BATTLE_ROUNDS`, now), `ground::effect_of`, `arena::effect_of`, `encounter::phase_effect` (grant, rounds 2, now, no draw: FB-58). `apply_effect`: an inflict (`resistable` unset) or a rolled spec goes through `try_inflict_with` (one draw when the row is resistable; a dead target takes nothing), an explicit grant (`Some(false)`) is granted now or queued with no draw. Same draw order, timing, defaults and stacking as before: every golden byte-identical. Grants carry `resistable: Some(false)` (never rolled), which matches today; every buff row is non-resistable and every shipped grant names a buff, so the spec's "row default" reading differs only for a future grant of a debuff row. Not among the five (left on their own paths): terrain hazards (W2-5, FB-51d), encounter `start_status`, phase enrage / empower, the Guard hold queue, the poise-break queue. Test: `rpgkit_battle/tests/effect_spec.rs` (7: each vocabulary against today's direct S1 call on twin units and RNGs over 20 seeds, acceptance 2, the dead-target guard).
- **Recommendation**: STRUCTURE: `status.rs` `EffectSpec`, `Timing`, `apply_effect`, `grant_timing`; the four converters; test `effect_spec.rs`. GAME_BRIEF / kit README statuses: "every status application from skills, triggers, ground hazards, arena hazards and encounter phase effects goes through `status::apply_effect` (rolled when `resistable` resolves true; grants and phase effects never roll)". HISTORY: "A4.2: the five status vocabularies share one application path; no golden moved". Confirm: should a grant of a resistable (debuff) row ever roll?

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
