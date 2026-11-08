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
| R4-1 `U2.3-TIERS` | slice 1 done (inert, 0 golden diffs; FB-144); slice 2 (template rows, golden re-records) waits on FB-144 rulings | `develop` | Then R4-2. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-141 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-142. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-142: W2-6 landed (closes R3 Wave 2); docs delta and one nit

- **Spec**: `docs/READY.md` row W2-6 (W.2-S2..S3, E1.2-S1, E2.2-S2, T1.2-S2..S3), FB-139 A, FB-140 A, FB-141
- **Kind**: docs-delta
- **Status**: open
- **Finding**: W2-6 is squashed onto `develop` (branches `claude/w26`, `-a`, `-b`, `-c`). The full gate passed with 0 warnings, `change-reviewer` found no blockers, and no golden moved: no shipped unit sets `growth_per_stage` yet, so `journey_golden` and `checks_golden` are byte-identical. The landing added the FB-140 arm `["portraits", "full_body", f] => png(f)` to `layout_ok` in `emberwake_gui/src/region.rs` and dropped a stray empty `emberwake_gui/src/lib.rs`. Nit, left as is: `rpgkit_world/src/battle.rs` `roster_at` reads a missing `tier` as `standard` for the XP weight, while `stage_level` reads it as `hero` (the schema default), so a tierless row is never scaled but is weighted as standard. That default predates W2-6, and shipped rows always carry `tier`.
- **Recommendation**: Design applies this docs delta.
  - `engine/README.md`: `aegis_content::store::set_manifest`, `Hooks::manifest_keys` / `manifest_problems` (kit-registered `game.json` extra keys; an unregistered key is reported, `null` clears one), `Manifest::set_with` / `extra`.
  - `kits/rpg/README.md`: `battle::begin_world(_with)`, `WorldFight`, `world_stage`, `foe_level` / `roster_at` / `enemy_levels_at` (`growth_per_stage`, enemies only, capped at `MAX_LEVEL`); `lower_start_ap` moved from the GUI into the kit; unit field `growth_per_stage`; dialogue choice `roll` effect (`dialogue` roll stream, `dialogue_roll_die` on `world.json`); `STANDING_RATINGS` and `member_stat` for dialogue stat reads.
  - `GAME_BRIEF` / `STRUCTURE`: SERVICES1 (`talk::Services`, `ServiceScreen` on a plain backdrop; lore pages stay in `RegionMap`); `portraits/full_body/<id>.png` in the asset tree (already in ASSETORG2).
  - `docs/HISTORY.md`: a new `dialogue` roll stream in `GameRolls` (rolls on other streams are unchanged).
  - Decide on the nit if you want: make `roster_at`'s tier default `hero` to match, or keep it.

### FB-143: R3-CLOSE backend audit: no release blocker; five dormant-feature follow-ups and big-picture notes

- **Spec**: `docs/READY.md` row R3-CLOSE (`audit-build` over `origin/main..origin/develop` @e6450e22, code commit W2-6); rows 20, W2-6; FB-139..142
- **Kind**: feasibility
- **Status**: open
- **Finding**: Released as #57 (@8d863f58). No golden or fixture changed in the range, layers are clean, and the schema gains only `units.growth_per_stage` and `world.dialogue_roll_die`. Every W2-6 feature is dormant in shipped data, so none of the follow-ups below can reach a player yet.
  - AUDIT-01 (low): rows without `tier` get two defaults: `stage_level` (`rpgkit_world/src/battle.rs:86`) and `UnitDef` read `hero`, while `roster_at`'s XP weight (`battle.rs:215`) and `WorldRoster::tier` (`battle.rs:118`, morale) read `standard`. Fix: one shared default (`hero`, the schema's).
  - AUDIT-02 (medium, balance): 5 shipped enemies have an explicit `spend`. A stage-raised level adds class growth but no spend points (`rpgkit_rules/src/unit.rs:1251` uses the fixed `e.spend`), while XP and purse weight rise in full. Fix: auto-spend the extra pool, or warn when `growth_per_stage > 0` sits on a row with `spend`.
  - AUDIT-03 (low): a `roll` choice on a dialogue loop can be repicked until it succeeds (T_STORY 1.1 anti-save-scum). Fix: extend `tree_warnings` (`check/dialogue.rs:725`) to unguarded `roll` on loops.
  - AUDIT-04 (low): the whole-folder check (`rpgkit_content/src/store.rs:580`) does not report an unregistered game.json key, which `store::set_manifest` does, so a typo passes `checks` and then blocks the next manifest write.
  - AUDIT-05 (low): `stat_kind` (`check/dialogue.rs:108`) checks derived-stat names against the base catalog, while `member_stat` (`dialogue.rs:185`) evaluates with the world's ruleset and silently reads 0 for an unknown name.
  - Unlogged minor rulings for Design to record: `dialogue_roll_die` schema default is `-1` (inherit from the nearest parent world; effective default 20); in `set_manifest`, `null` removes an extra key and an unregistered key in the file blocks every write; plain `Manifest::set` still keeps unregistered keys (tests only); `PurseCaps::problems` now sees the normalized manifest, so a whole float such as `99.0` is accepted.
  - Big picture: (1) R3 features are code-complete but dormant: no golden exercises stage scaling, rolls or purse caps until data sets them; an asset row seeding `growth_per_stage` on a few campaign enemies and one `roll` dialogue, with a journey that crosses an act, would make the release gate meaningful. (2) The sim (`rpgkit_sim/src/world.rs`, `runner.rs:413`) and host (`rpgkit_host/src/lib.rs:231`) always fight at stage 0; a `--stage N` sim option is needed before E/I balance. (3) Stage is the raw `campaign.act`, while checkpoints default `act` to 1 (`checkpoint.rs:47`): `start_act: 1` scales enemies from the first fight; consider `act - start_act` or state the rule. (4) "Enemy" means `tier != hero`, not `side == enemy`, which is fragile for rivals. (5) `ServiceScreen` is reachable only from the UI Gallery. (6) `apply` recomputes the stage at settle time instead of carrying `WorldFight.stage` in the `Tally`.
- **Recommendation**: queue AUDIT-01..05 as one small kit row in a later wave (AUDIT-02 before any data sets `growth_per_stage`); decide (1)-(3) before W/E/T balance; record the unlogged rulings.

### FB-144: R4-1 U2.3-TIERS slice 1 landed inert; slice 2 needs five rulings and the template values

- **Spec**: `docs/READY.md` row R4-1, FB-78; `docs/sessions/U2_CLASS.md` item 2; `I_AI.md` line 20
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Slice 1 is on `develop`: a kit content kind `unit_templates` (`data/catalog/unit_templates.json`, one row per `ai_role`: `id`, `name`, `role`, `base_shares`, `growth_shares`, `origin`; checked like class share rows incl. 6..18 at level 1; editable through `aegis content upsert unit_templates`). `rpgkit_rules::unit::template_for` picks the row for a derived, non-party unit whose tier is not hero (role: unit `ai_role`, else class's); the class keeps gear, path, basics and loadout; a templated unit's stats are `roundi(stat * tier_hp)` and HP `roundi(60 * tier_hp) + CONS*8 + STR*2`. No rows ship, so every golden is byte-identical (gate: 2673 tests, 0 warnings; change-reviewer clean). Build's choice, not in the spec: a content kind (asset-lane editable) rather than a crate-embedded table like `tiers.json`; no unit-row template override; the "growth curve" read as the template's `growth_shares` on the shared budget slope (62 + 5 per level). Slice 2 (rows shipped) moves `rules_golden`, `sim_golden`, the timeline and PHASEFOLD goldens and the journey (maybe `checks_golden`), with wide divergence: max HP mostly within +-5% (minions -4..+3, standard -13..+9, elite -22..+21, boss -20..+29) but `calcified_amalgam` +202 HP (276 -> 478), and a level-3 minion's DEX 23 -> 9, STR 21 -> 8 (scaled minion stats 2-9).
- **Recommendation**: rule on each before slice 2:
  1. Template values. Proposed (count-weighted average of today's non-hero derived units' class shares per role, base x62 / growth x5, largest remainder; all pass 6..18): AGGRESSOR base STR 13, DEX 9, INT 7, CONS 15, PERC 9, KNOW 9, growth STR 2, DEX 1, CONS 2 (18 units: warrior 7, brawler 6, tl_drill 4, mage 1); SKIRMISHER = rogue (base 9/16/9/8/12/8, growth STR 1, DEX 2, CONS 1, PERC 1); SNIPER base 7/14/11/9/13/8, growth STR 1, DEX 2, CONS 1, PERC 1 (ranger 3, mage 1); SUPPORT = priest (base 6/8/11/12/10/15, growth INT 1, CONS 1, PERC 1, KNOW 2). Alternative: the dominant class per role (warrior, rogue, ranger, priest).
  2. Double offence scaling: attack power already takes `tier_power` (minion 0.6); scaling STR/DEX by `tier_hp` (0.4) as well cuts minion offence twice. Scale non-HP stats by `tier_power`, by `tier_hp`, or not at all?
  3. Low stats: scaled minion stats fall to 2-4, breaking DEX move rate, KNOW unlocks and gear requirements. A floor (1? 6?), or scale only some stats?
  4. Role vs archetype: an AGGRESSOR mage (`calcified_amalgam`) takes brute stats. Add a unit-row `template` override (default "" = by role), or role+archetype rows?
  5. Gear modifiers are not tier-scaled (only base stats): intended? And should "non-hero" mean the tier or the side (a named enemy on the hero tier keeps class stats)?
  - Slice 2 watch: `rpgkit_sim/src/variants.rs:70`, `rpgkit_world/src/progression.rs:112` and any Forge enemy preview still read class shares.
  - Docs delta: STRUCTURE `rpgkit_data/src/` add `unit_template.rs` (`UnitTemplate`, `UNIT_TEMPLATES_KIND`, `UnitTemplate::over`; `Catalog.unit_templates` keyed by role); `rpgkit_content/src/` add `check/unit_template.rs` and `source.rs` reading `unit_templates.json` (`UNIT_TEMPLATES_KIND`), tests `tests/unit_template_checks.rs`; `rpgkit_rules` tests `tests/unit_templates.rs`; `games/emberwake/data/catalog/` add `unit_templates.json` (kind `unit_templates`, absent until slice 2). GAME_BRIEF unit rules: the templated-unit rule above. PORT_NOTES: `unit_templates` is engine-only (no Godot original). STRUCTURE is stale on `tiers.json`: it is embedded in `rpgkit_rules/src/tiers.json`, not `data/catalog`. EFFICIENCY repeat finding: a new numeric `pub const` in `rpgkit_rules/src` must go in `LOCKED` / `PROMOTED` or stay private (BASERULES1 scan). HISTORY when slice 2 lands: enemy stats delinked from hero classes; goldens re-recorded.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
