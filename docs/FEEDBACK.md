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
| W2-4 (R2-W2) | done (slices 1-7) | `develop` | - |
| W2-5 (R2-W2) | done: slices 1-4 (FB-127..129, 131); open FB-128, FB-130 | `develop` | - |
| R2-W2b CLASS-BASICS | done (FB-132, FB-134, FB-135) | `develop` | - |
| R2 release | released: `main` @3cc73f2b ([#56](https://github.com/6lackwin9/tactical-rpg/pull/56), merge commit; tree = develop @f5c6568a) | `main` | - |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (FB-145) | `develop` | Next: R4-2 `R4-CLOSE`. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-144 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-145. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-145: R4-1 slice 2 landed; user ruling: one tier_mult replaces tier_hp and tier_power; duel bands out of reach

- **Spec**: `docs/READY.md` row R4-1 slice 2, FB-144; user ruling 2026-11-09 given in the Build Session: "the tier scaling should be a single tier_mult instead of tier_hp or tier_power ... roughly 0.4-0.6, 0.8-1.0, 1.25-1.5, 2.0-3.0 in order", numbers left to Build
- **Kind**: sim
- **Status**: open
- **Finding**: Slice 2 is on `develop`. FB-144 is built as ruled: the 4 `unit_templates` rows (seed `r41s2`; shares round back to the integer points at levels 1-10), `template: "class"` on `calcified_amalgam` and `inquisitorial_purifier` (the second caster: a mage SNIPER that would take archer shares), stats floored at 6, gear requirements (`Equipment::meets_requirements` / `wield_penalty`; the spec's `gear::unfit_penalty` does not exist) and unlock KNOW on unscaled stats, no double tier factor in attack power (the weapon part takes it, the scaled stat part does not), sim / progression / Forge read through the template. Then the user's ruling replaced FB-144's `tier_power` scaling: `tiers.json` rows are `{id, mult, skill_cap}`, hero 1.0, **minion 0.4, standard 0.825, elite 1.25, boss 2.0** (before: hp 0.4/0.7/1.25/1.5, power 0.6/0.8/1.0/1.1; the FB-144 text's 0.6/1.0/1.15/1.3 never matched the table). Templated units: stats `max(6, roundi(stat * tier_mult))`, HP `roundi(60 * tier_mult) + CONS*8 + STR*2`; other units: HP `(60 + CONS*8 + STR*2) * tier_mult`, attack power base stage `* tier_mult`. Formula input `tier_mult`; `tier_hp` / `tier_power` kept as deprecated aliases (no shipped row uses them). Re-recorded: `rules_golden`, `timeline_golden`, `sim_golden`, `phasefold_golden`; the frozen `timeline_kit.json` gained the 4 template rows (W2-5 precedent). Not moved: `checks_golden`, journey, `rules_contract_golden`, `rng_hash`, store resave. Heroes, party and fixed-stat units are unchanged in every golden. Samples (L1 unless noted, HP / melee): Footman 147 -> 186 / 31; Enforcer (elite) 263 -> 281 / 52; Captain (boss) 315 -> 452 / 105; goblin brawler (minion L3) 108 -> 104 / 21; warrior class L5 boss 501 -> 636 HP, melee 111. Test-assumption fixes, not engine bugs: `emberwake_content/tests/ruleset_switches.rs` replays `r41s2.json`; four `emberwake_gui` battle tests pick a step whose target still stands or wait for the attacker's slide (FX1 4.2).
  - Tuning (sim feel timeline seeds=40, party 5, campaign 5; candidates min/std/elite/boss -> feel total, party line/elite/boss %, campaign clears arms/gate): midpoints 0.5/0.9/1.4/2.5 -> 31.5, 75/75/25, 0/2, 1/2; 0.5/0.85/1.3/2.0 -> 34.5, 75/100/75; 0.4/0.8/1.25/2.0 -> 60.5, 100/100/75, 2/2, 2/2; 0.5/0.825/1.25/2.0 -> 61, 92/92/83, 1/6, 4/6; **picked 0.4/0.825/1.25/2.0 -> 61, 92/92/92, 3/6, 5/6**. Every tier sits at the low end of its range.
  - Out of reach under the ruled ranges: the R2-CLOSE feel bands (76% overall, duels 62.5-65%). `tl_duel_armour` (L1 hero vs boss) is 0% at any boss value >= 2.0, `tl_duel_windup` (vs elite) peaks at 17.5% at 1.25, so feel tops out near 61%. Party elite and boss win rates sit above their 50-75 / 30-50 bands at every value that keeps the campaign winnable. Standard 0.825 sits just below a cliff (the reach duel is 100% at 0.825, 5% at 0.85).
  - The floor of 6 flattens minions at 0.4: most minion stats hit 6, so low-level minions all start at 84 HP in phasefold.
  - Generic sim class foes (`sime_<class>_<tier>`) and rules-golden class rows now play their role's template (a mage foe takes SNIPER shares), since only shipped units carry `template: "class"`.
  - Open builder readings: weapon threshold tiers (`Equipment::tier_met`) read scaled stats; a `derived_stats` formula using a stat and `tier_mult` would scale a templated unit twice (no shipped row does).
- **Recommendation**: Design records the ruling (decision row, GAME_BRIEF, STRUCTURE / PORT_NOTES `tiers.json` rows `{id, mult, skill_cap}`, HISTORY: "`tier_hp` and `tier_power` merged into one `tier_mult` (user 2026-11-09); slice 2's `tier_power` scaling and FB-78's separate `tier_hp` superseded; re-recorded rules, timeline, sim, phasefold goldens"). For R4-CLOSE's balance pass, decide: (1) re-target the elite/boss duel bands, or treat L1 solo-vs-elite/boss as intended losses; (2) the party elite/boss bands; (3) a lower floor for minions, or minion 0.5 for variety; (4) whether weapon threshold tiers read unscaled stats; (5) `template: "class"` on generic sim caster foes. Kit README already carries the slice-2 paragraph.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
