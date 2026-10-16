# FORGE1_SKILLS_MATRIX: appendix tables for FORGE1_SKILLS

Status: proposed (appendix of `docs/specs/FORGE1_SKILLS.md`; the main spec holds the decisions, this file holds the field matrices and reference tables)

Legend for matrices: C = common (authored for every kind that uses it), K = kind-specific, D = derived from the kind, `-` = unused (hidden by the schema, rejected by the validator if non-default), E = engine fixed value. Engine kinds are REST, MOVE, TARGET, STATUS.

## A. What the code does today (checked)

| Fact | Where |
| --- | --- |
| 127 flat rows, 49 families: 39 with exactly 3 ranks, 10 single rows (strike, offhand_jab, guard, move, switch_target, rest, staggered, stunned, shockwave, reckless_swing). Kinds by row: ATTACK 88, SUPPORT 18, DEFEND 7, DISENGAGE 6, FOCUS 3, STATUS 2, MOVE 1, TARGET 1, REST 1. `skills.json` is already `{"kind", "version", "entries"}` (version 1) | data audit |
| Every row id is `<head>` for tier 1 and `<head>_<tier>` otherwise; every rank name is `<name> II` / `III`. `learned_only`, `basic`, `kind`, `area`, `stance`, `offhand` (now `weapon_hand`), `status_effect`, `target_mode`, `heal_stat`, `weapon_types`, `min_proficiency` never differ between ranks | data audit |
| Fields that differ between ranks (39 families): level 39, description 39, power_mult 27, status_potency 17, status_rounds 8, heal_base 3, heal_mult 3, mitigation 2, cleanse_statuses 2, dash_tiles 2, then reach_min, reach_max, blocks, boost_rounds, grant_status, jump_tiles, status_magnitude 1 each. 289 override cells in total (78 are descriptions); 7 non-description cells repeat rank 2 in rank 3 | data audit |
| No data row references a rank id outside `skills.json`: class `skill_path`, unit `extra_skills` / `loadout`, passives all name the head id. Rank ids are used by the planner (`SkillCatalog.family_tiers`) and tests | grep of `data/` |
| `ContentCatalog.upsert` is the single write path; `entries("skills")` returns flat rows; `references_to` / `_rename_refs` treat `family` as a skill id field; `to_storage` omits fields hidden by `when`; `upsert_many` exists (atomic set edits) | `content_catalog.gd`, `content_schema.gd` |
| `ContentSchema.coerce` clamps silently; `forge_form._enum_widget` appends an unknown stored value; `weapon_types` and `cleanse_statuses` are comma-separated text boxes | `content_schema.gd`, `forge_form.gd` |
| `grant_status` dropdown offers every buff, validation accepts only support buffs for SUPPORT and defend+support buffs for DEFEND | `content_schema.gd` vs `content_catalog.gd` |
| `Combatant.can_use` is kind-blind: returns true when `min_proficiency <= 0`, otherwise checks `weapon_types` (empty = any wielded type) and the proficiency level. So `min_proficiency` without `weapon_types` is valid ("any weapon at level N"); `weapon_types` without `min_proficiency` is inert | `combatant.gd` 212 |

## B. Field by kind matrix

| Field | ATTACK | DEFEND | FOCUS | SUPPORT | DISENGAGE | REST | MOVE | TARGET | STATUS | Note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| id, name, description, kind | C | C | C | C | C | C | C | C | C | name and description are template-level (main spec sections 3-4) |
| system | C | C | C | C | C | E | E | E | E | read-only; 5 engine rows |
| level (AP) | C | C | C | C | C | - | - | - | - | MOVE costs 1 per tile, REST/TARGET/STATUS cost 0 |
| blocks | C | C | C | C | C | C | C | C | C | TARGET fixed at 2; STATUS ignores it |
| learned_only, basic | C | C | C | C | C | C | C | C | C | |
| grade, requires, unlock_offset | C | C | C | C | C | - | - | - | - | engine rows are single-tier; non-novice `grade` and `requires` invalid on basic/system |
| stance | K | D | D | D | D | D | D | D | D | ATTACK may override (6 rows DEFENSIVE) |
| weapon_types, min_proficiency | K | K | - | K | - | - | - | - | - | DEFEND and SUPPORT opened by this spec (engine already enforces) |
| reach_min, reach_max | K | - | - | K | - | - | - | - | - | |
| area, power_mult, weapon_hand, dash_tiles | K | - | - | - | - | - | - | - | - | |
| status_effect, status_potency, status_magnitude | K | - | - | - | - | - | - | - | - | magnitude only weaken |
| status_rounds | K | K | - | K | - | - | - | - | - | inflicted status (ATTACK) or granted buff (SUPPORT, DEFEND) |
| grant_status | - | K | - | K | - | - | - | - | - | |
| target_mode, heal_stat, heal_base, heal_mult, cleanse_statuses | - | - | - | K | - | - | - | - | - | |
| mitigation | - | K | - | - | - | - | - | - | - | |
| boost, boost_rounds | - | - | K | - | - | - | - | - | - | `boost` 1.5 in all 3 FOCUS rows |
| jump_tiles | - | - | - | - | K | - | - | - | - | |
| recover_ap | - | - | - | - | - | K | - | - | - | |

Findings (verified; nothing is unused outright):

1. `weapon_types` alone is inert (`can_use` returns early on `min_proficiency <= 0`) but `forge_preview` shows "Needs bow". Keep ONE directional validation rule: `weapon_types` non-empty requires `min_proficiency >= 1`. The reverse (`min_proficiency` without `weapon_types`) is legal and meaningful, so no rule exists for it.
2. `stance_override` for non-ATTACK kinds and the SUPPORT `keeps_stance` branch are reachable only from hand-built `SkillDef`s (tests).
3. `level` is unused for 4 engine kinds; `blocks` is unused for TARGET.
4. `system` and `basic` overlap for `move`, `rest`, `switch_target`; harmless, keep both.
5. `family` and `tier` are derivable from the id; with ranked storage they are no longer stored.
6. `heal_stat` is coupled to `heal_mult > 0` (dependent field).
7. `status_effect` (inflict, debuff) and `grant_status` (grant, buff) are exclusive by kind; the Forge shows them as one "Status" section with a polarity-filtered picker. No storage merge.
8. MOVE, TARGET, STATUS are already unauthorable; REST becomes engine-only too.

## C. Shared behaviour today

| Shared behaviour | Where it lives | Duplication |
| --- | --- | --- |
| AP cost | `SkillDef.ap_cost` | preview repeats the MOVE special case |
| Stance | `SkillDef.stance`, `keeps_stance`, `stance_after`, `switch_cost`, `plan_entry` | `forge_preview` re-derives it and shows "Neutral" for SUPPORT/DISENGAGE |
| Targeting / reach | `reach_for`, `support_reach`, `targets_ally`, `combat_plan.target_options`, `tactical_ai` | rule spread over 4 files |
| Status payload | ATTACK `build_command`, SUPPORT `SupportRules.apply`, DEFEND `DefendCommand.on_start` | three delivery paths |
| Gating | `Combatant.can_use`, `knows_skill`, `owns_skill`, `SkillCatalog.available` | engine kind-blind; schema/validator ATTACK-only |
| Family / rank | `SkillCatalog` helpers, `_family_errors`, `_rename_refs`, `forge_preview`, `PlanOption.group_by_family` | see section E |
| Authoring visibility | `content_schema._skill_fields` named and inline `when` lists | same kind lists as `SkillDef` and the validator |
| "X belongs to Y" validation | `_skill_errors`, about 10 hand-written rules | restates the `when` lists |
| Preview | `forge_preview._skill`: 6-branch `match` | per-kind text, no section reuse |
| Effects outside command classes | `dojo_session` lines 398-480 | out of scope; noted |

## D. Kind merge candidates (touch points, non-test code)

| Candidate | What really differs | Touch points | Recommendation |
| --- | --- | --- | --- |
| DEFEND into FOCUS | different state (mitigation ratio vs attack boost), stance, riposte/taunt | DEFEND 11 sites/7 files; FOCUS 9/7 plus readers and snapshots | keep both |
| FOCUS into SUPPORT buff | needs a registry buff with a multiplier channel and a damage-formula hook (breaks S12 "formula core untouched") | FOCUS list plus registry and undo snapshots | keep (declined before, no new reason) |
| SUPPORT self buff into DEFEND | `shield_wall` and `taunt_shout` overlap; DEFEND adds stance and mitigation | SUPPORT 17/6, DEFEND 11/7 | keep; Forge hint on SUPPORT/SELF |
| DISENGAGE into MOVE | own planner path, command, root refusal | 13 sites/6 files | keep |
| REST authorable | one engine row | 1 validator message | mark REST engine-only |

Result: keep all nine kinds, split into authorable (ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE) and engine (REST, MOVE, TARGET, STATUS).

## E. Every use of `family` today

R = rank grouping, S = slot / ownership identity (needs only the head id), A = grade / requirements linkage.

| Use | File / function | Cat. |
| --- | --- | --- |
| Fields `family`, `tier`, `family_id()`; factory writes family "" when equal to the id | `skill_def.gd`, schema, `content_factory` | R |
| Tier limit (basic 5, class 3), unique tier, one kind, same `basic` | `ContentCatalog._family_errors` | R |
| Rename carries rank rows | `_rename_refs` | R |
| Planner groups "Mend I / II / III", family name = lowest tier | `SkillCatalog.family_tiers`, `grouped`, `group_label`, `roman`, `family_name`; `PlanOption.group_by_family`; `tactical_ai` | R |
| Skill potency passives and gear effects key on the family id | `Combatant.potency_for`, `Passives` `skill_potency` | S |
| Ownership and loadout slots | `Combatant.owned_families`, `set_loadout`, `knows_skill`, `learn_skill`; `ContentFactory.families_of`, `loadout_of`; `UnitState`; unit `loadout` | S |
| Class `skill_path` names the head id | `_class_errors` | S |
| SK2 `advanced` bool (becomes `grade`, family-level), unlock level plus `unlock_offset`, `unlock` passive | `_family_errors`, `Combatant.tier_unlocked`, `Passives` | A, S |
| Test fixtures `t_fix`, `t_fix_2`, `t_fix_3` | `skill_family_test_suite` | R |

Conclusion: basic and advanced skills are separate families (DECISIONS "Skill model v2"). `family` exists only to group ranks, so it becomes derived (head id) and `tier` the rank index.

## F. Per-field guard rails

"Rank" = may be changed by a rank modifier (absolute value). NEW = rule added by this spec.

| Field | Type | Allowed values / range | Default | Control | Visible when | Rank |
| --- | --- | --- | --- | --- | --- | --- |
| id | id | `^[a-z][a-z0-9_]*$`, unique; `<id>_<n>` reserved for ranks | "" | id edit, live filter and uniqueness | always (read-only after create for basic/system) | no |
| name | string | 1-32 chars (NEW), derived rank names `<name> II` must also fit 32 | "New Skill" | line edit + counter | always | no (derived) |
| description | template text | 0-240 chars (NEW), resolved text per rank also 0-240; tokens per main spec section 5 | "" | multi-line + counter + token buttons | always | no (tokens) |
| kind | enum | authorable: ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE; engine kinds read-only | ATTACK | dropdown; switching resets hidden-section fields and applies `kind_defaults` | always | no |
| system | bool | read-only | false | label | never | no |
| basic | bool | locked after create; implies exactly 1 rank (NEW); owned by every unit from level 1; NOT valid in a class path; the Forge cannot create new basics (main spec question 1) | false | checkbox | locked on existing | no |
| learned_only | bool | | false | checkbox | always | no |
| grade | enum | `novice`, `advanced`, `expert` (reserved, no expert skill ships); novice only on basic or system; KEY field | novice | dropdown | not basic | no |
| requires | list of ref | head skills, each not system, not basic, not self, no duplicate, grade STRICTLY LOWER than this skill's; ALL required to own/learn; empty for novice; KEY field (main spec section 11) | [] | `multi_pick` filtered to lower grades | grade is not novice | no |
| origin | enum | `system`, `user`; KEY field; set by the Forge (create = user, Promote = system), never typed | system | read-only label + Promote button | always | no |
| level (AP) | int | 1-5 | 1 | spinbox | kinds with AP | yes |
| blocks | int | 1-6 | 1 | spinbox | always | yes |
| stance | enum | AUTO, OFFENSIVE, TACTICAL, DEFENSIVE, NEUTRAL | AUTO | dropdown | ATTACK (others show derived text) | no |
| weapon_types | list of enum | subset of `SkillDef.weapon_type_tokens()`, no duplicates; non-empty requires `min_proficiency >= 1` (NEW) | [] | checklist | ATTACK, DEFEND, SUPPORT (NEW) | no |
| min_proficiency | int | 0-3 (`Proficiency.MAX_LEVEL`) | 0 | spinbox; minimum 1 once a type is ticked | ATTACK, DEFEND, SUPPORT (NEW) | no |
| power_mult | float | 0.0-6.0 step 0.05; 0 only with a status and no area | 1.0 | spinbox | ATTACK | yes |
| area | enum | NONE, PLUS, BURST, LINE, SWEEP; not with power 0 | NONE | dropdown, invalid options disabled | ATTACK | no |
| weapon_hand | enum | main, off, both; off/both not with dash_tiles > 0; both rejected while `SkillDef.BOTH_ENABLED` is false (main spec 4.1); replaces bool `offhand` (true -> off) | main | dropdown, `both` disabled "engine pending" | ATTACK | no |
| dash_tiles | int | 0-6; 0 with an area or weapon_hand off/both | 0 | spinbox, max 0 while area or off/both hand | ATTACK | yes |
| reach_min | int | 0-8, at most reach_max; 0 when reach_max is 0 | 0 | spinbox, max = reach_max | ATTACK, SUPPORT | yes |
| reach_max | int | 0-8; at most 1 for LINE/SWEEP | 0 | spinbox | ATTACK, SUPPORT | yes |
| status_effect | enum | "" or `StatusEffects.debuffs()` | "" | registry dropdown | ATTACK | no |
| status_potency | int | 1-200 | 0 | spinbox, min 1 while a status is set | status_effect set | yes |
| status_magnitude | int | 1-25 (`cap_of(weaken)`), weaken only | 0 | spinbox | status_effect is weaken | yes |
| status_rounds | int | 1-5 | 1 | spinbox | status_effect or grant_status set | yes |
| target_mode | enum | SELF, ALLY, ALLY_OR_SELF; SELF forced when grant is taunt | ALLY_OR_SELF | dropdown with disabled options | SUPPORT | no |
| heal_stat | enum | "", INT, KNOW; non-empty iff heal_mult > 0 | "" | dropdown | SUPPORT | no |
| heal_base | int | 0-500 | 0 | spinbox | SUPPORT | yes |
| heal_mult | float | 0.0-10.0 step 0.05 | 0.0 | spinbox | SUPPORT | yes |
| grant_status | enum | SUPPORT: `support_buffs()`; DEFEND: `defend_buffs()` plus `support_buffs()` (NEW: dropdown stops offering invalid buffs) | "" | registry picker filtered by kind | SUPPORT, DEFEND | yes |
| cleanse_statuses | list of enum | subset of `debuffs()` or exactly `["all"]` | [] | checklist; "all" clears the rest | SUPPORT | yes |
| jump_tiles | int | 1-6 | 0 (draft 1 via `kind_defaults`) | spinbox | DISENGAGE | yes |
| mitigation | float | 0.0-0.95 step 0.05 | 0.5 | spinbox | DEFEND | yes |
| boost | float | 1.0-5.0 step 0.05 | 1.5 | spinbox | FOCUS | yes |
| boost_rounds | int | 1-10 | 2 | spinbox | FOCUS | yes |
| recover_ap | int | 0-12 | 2 | spinbox | REST (engine) | yes |
| unlock_offset | int | -1 (auto) or 0-8; rank 1 absent/0 | -1 | "auto" checkbox + spinbox 0-8 | rank 2+ | yes |
| ranks | list of modifier rows | 0 to 2 rows (`MAX_CLASS_TIER - 1`); keys rank-modifiable, visible for the kind, in range | [] | ranks section | not basic, not system | n/a |

`kind_defaults` is a per-kind map applied once when the kind changes (jump_tiles 1 for DISENGAGE, heal_stat KNOW and heal_base 1 for SUPPORT, mitigation 0.5 for DEFEND), so a fresh draft is not invalid for a hidden reason.

## G. Typed controls

| Control | Replaces | Source |
| --- | --- | --- |
| Enum dropdown that never offers an unknown stored value as a normal choice (red "(invalid: x)" item plus a problem line) | `_enum_widget` appending unknown values | schema `options` / `options_from` |
| Status picker (`debuff`, `buff_support`, `buff_defend_support`) | `status`, `status_buff` | `StatusEffects` / `StatusRegistry` |
| Int/float spinbox with dynamic limits (reach_min <= reach_max, min 1 when a status is set) | static `SpinBox` | schema `min`, `max`, `step`, `dyn_min` / `dyn_max` resolved in `SkillFormModel` |
| Checkbox with enable rules | CheckBox | `enabled_when` |
| Ref picker filtered by kind and field values | `ref` dropdown | `ref_filter` (extended: "not in", "not self") |
| Multi-select checklist, exclusive items, no free text | comma `strings` | `options` / `options_from` (`weapon_type_tokens`, `debuffs_or_all`) |
| Ranks section (main spec section 3) | future grouped view | `rank: true` flags |
| Description editor with token buttons, per-rank live preview | multi-line text | token table |
| Derived read-only label | family, tier, rank ids and names, derived stance | computed |
| Character-filtered id edit with live uniqueness | `LineEdit` | `is_valid_id` |

No free JSON or dict editor remains for skills. The effect-list editor belongs to the items/passives pass.

## H. Prevent versus detect

| Class | Examples | Mechanism |
| --- | --- | --- |
| Prevented by the control | wrong type, out-of-range number, enum outside its set, duplicate list entries, non-existent ref, invalid id characters, hidden-section field, buff on the wrong kind, `min_proficiency` 0 while a weapon type is ticked, dash with an area, reach_min above reach_max, taunt with a non-SELF target, a rank modifier on a base-only field | typed controls, `when`/section visibility, option filtering; `coerce` stays only as a safety net for scripts |
| Detected by the validator | id used, support skill with no payload, heal_stat set while heal_mult 0, power 0 without a status, weaken magnitude missing, rank count over cap, basic with ranks, non-novice `grade` or `requires` on a basic, a required skill of equal or higher grade, `unlock_offset` rules, required engine skill missing, rename collisions, rank id collision, unknown or kind-invalid description token, resolved name/description over cap | `ContentCatalog.validate_skill_entry(entry) -> Array[Dictionary]` returning `{"field", "rank", "message"}`; `validate_entry` maps `message` to plain strings so existing callers and tests are unchanged |
| Soft warnings | modifier equal to the base, literal digits in a description, literal per-rank description override | `warnings_for("skills", entry)`; never blocks |

Presentation: while `validate_skill_entry` is non-empty, Save and Apply are disabled, a red "Cannot save: N problems" header lists each reason, the offending field (and rank cell) is outlined red with the same text as tooltip, and the list row gets the warning mark. Keyboard Apply is ignored while blocked. Warnings show in yellow and do not block.

## I. Form layout

The Forge edits the SKILL FAMILY (the template) as the primary object.

| Order | Section | Kind | Fields |
| --- | --- | --- | --- |
| 1 | Identity | core | id, name, description template |
| 2 | Classification | core | kind, basic, learned_only, grade, requires, system (read-only) |
| 3 | Cost and timing | core | level (AP), blocks, stance (rank 1 values; ranks may modify level and blocks) |
| 4 | Targeting | core when the kind has a cast target | target_mode (SUPPORT), reach_min, reach_max, area (ATTACK) |
| 5 | Gating | core for ATTACK, DEFEND, SUPPORT | weapon_types, min_proficiency |
| 6 | Kind block | per kind | ATTACK: damage (power_mult, weapon_hand, dash_tiles) and inflict (status_effect, status_potency, status_magnitude, status_rounds); SUPPORT: heal and cleanse, grant; DEFEND: guard (mitigation), grant; FOCUS: boost, boost_rounds; DISENGAGE: jump_tiles; REST: recover_ap |
| 7 | Ranks | core | the ranks section (rank table of modifiers, per-rank resolved panel, read-only); basics and system show "single rank" |
| 8 | Preview and problems | derived | resolved per-rank summary (cost, stance, numbers, rendered description), "Used by", problems and warnings |

Sections 1-5 and 7 are authored once; section 6 is the only kind-specific one and is generated from `SkillKinds` sections.

## J. Search, filter and protection

| Control | Behaviour |
| --- | --- |
| Text search | substring over id, name, description template (case-insensitive) |
| Filter chips | kind (multi), rank count, basic, learned_only, grade, system (hidden by default), weapon type, inflicts status, grants status, "has problems" |
| Class usage | on a class path (pick a class), unit extra skill, referenced by a passive, `requires` target, unused |
| Sort | name, kind, AP (rank 1), rank count, problems first |

`SkillFilter` is a pure `RefCounted` predicate over entries and `ContentCatalog.references_to`, tested headless; filters are declared per kind in the schema (`"filters"`).

Basic protection: `remove_skill` and legacy `remove` refuse `basic` and `system`; `SkillKinds.REQUIRED_IDS` (strike, guard, move, rest, switch_target, offhand_jab) is a `validate_all` invariant (missing, wrong kind, or `basic` false is a catalog error). For existing basics (origin system) the Forge locks the key fields of section M (id, kind, basic, ranks section "single rank"); name, description and numbers stay editable. Delete is disabled with the reason as tooltip. A non-basic family is deleted whole, only when no reference exists; a rank can only be removed from the top.

## K. Tests (rule-derived; read counts and ids from the catalog, never hard-code)

| Suite | Checks |
| --- | --- |
| `weapon_hand` (in the skill model suite) | legacy `offhand: true` normalizes to `off` and the derived `offhand` getter is true; `both` is rejected while `BOTH_ENABLED` is false; `off`/`both` with dash_tiles > 0 error; non-ATTACK with non-default errors; `wielded_type(OFF)` is the off weapon type or `unarmed`; gate on an `off` fixture checks the off weapon; shipped `offhand_jab` is `off` and behaves as before (arena, planning suites unchanged) |
| new `skill_ranks_test_suite.gd` | round trip: for every shipped row `collapse` then `expand` reproduces it field by field; entry count equals distinct families and rank counts sum to the row count; ids `<head>`, `<head>_<n>`; `family_tiers(head).size() == ranks.size() + 1`; editing a base field changes every rank row, editing a modifier changes only that rank (no cascade: changing rank 2 leaves rank 3 equal to its own resolved value); a base-only key in a modifier row is rejected; a modifier equal to the base is dropped on save; derived rank names for ranks 1-3 (roman) and the 32-char cap on them; head rename renames rank ids and class/passive/unit/`requires` references; `remove_skill` removes all ranks and refuses a referenced head; basics with ranks rejected; version-1 and version-2 files both load; a version-2 file with `family`, `tier` or a `name` modifier is a load error; legacy flat `upsert` (the `t_fix` pattern) still works and saves as one entry |
| new `skill_text_test_suite.gd` | token resolution for every token against a fixture per kind (formats from main spec section 5); plural handling; `:n` suffix; unknown token error; token invalid for the kind error; token empty for some rank error; `{{`/`}}` escapes; unbalanced brace error; digit-literal warning; resolved length cap; migration generator: for every shipped family, the generated template renders to the stored text of every rank exactly, or the family is reported as literal-fallback (count printed, expected small) |
| `content_test_suite.gd` | non-rank field mismatch across flat ranks, id convention, non-contiguous tiers, `weapon_types` without `min_proficiency`, gate on DEFEND/SUPPORT accepted and enforced by `can_use` (a shield-gated DEFEND fixture is unusable without the weapon), gate on FOCUS/DISENGAGE rejected, `grade` / `requires` rules (equal or higher grade required skill rejected, novice with `requires` rejected, grade change breaking a dependent refused, expert accepted by validation), required-skills invariant, section rule replaces the old per-field messages (texts that existing tests match are kept) |
| new `origin_test_suite.gd` | see section M |
| `skill_family_test_suite.gd` | family validation unchanged; add a ranked-fixture variant using `SkillRanks.entry` |
| new `skill_form_model_test_suite.gd` | headless `SkillFormModel`: visible fields per kind equal `SkillKinds` sections, dynamic limits, kind-switch resets, disabled-Save reasons, `SkillFilter` predicates, ranks panel resolved values and highlighted modifiers, live description preview per rank; one `results.append` line in `suite_registry.gd` |
| `skill_engine`, `foundation`, `riposte`, `arena`, `planning`, `ai_skills`, `world`, `runner`, `progression`, `save` suites | pass unchanged: the acceptance gate for the descriptor refactor and the storage change |

## L. Description tokens (moved from the main spec section 5.1)

Values are the effective field values of the rank being rendered. Sections are from main spec section 4.

| Token | Source | Valid for | Output example |
| --- | --- | --- | --- |
| `{name}` | derived rank name | all | `Mend II` |
| `{ap}` | `level` | kinds with AP | `3 AP` |
| `{blocks}` | `blocks` | all | `2 blocks` |
| `{power}` | `power_mult` x 100 | ATTACK | `200%` |
| `{dash}` | `dash_tiles` | ATTACK | `4 tiles` |
| `{reach}` | `reach_min`, `reach_max` | ATTACK, SUPPORT | `3 tiles`, `2-3 tiles`, `1 tile` |
| `{weapons}` | `weapon_types` joined by `/` | gate kinds | `bow/crossbow` |
| `{proficiency}` | `min_proficiency` | gate kinds | `3` |
| `{status}` | ATTACK: `status_effect`; SUPPORT, DEFEND: `grant_status`; registry display name | ATTACK, SUPPORT, DEFEND | `Stagger`, `Haste` |
| `{potency}` | `status_potency` | ATTACK | `50%` |
| `{magnitude}` | `status_magnitude` | ATTACK with weaken | `20%` |
| `{rounds}` | ATTACK, SUPPORT, DEFEND: `status_rounds`; FOCUS: `boost_rounds` | those kinds | `1 round`, `2 rounds` |
| `{heal}` | `heal_base`, `heal_mult`, `heal_stat` | SUPPORT with a heal | `10 + 1.8 x KNOW`, `12`, `1.5 x INT` |
| `{cleanse}` | `cleanse_statuses` | SUPPORT with a cleanse | `stun`, `stun, bleed`, `all statuses` |
| `{mitigation}` | `mitigation` x 100 (author writes the sign) | DEFEND | `35%` |
| `{boost}` | `boost` | FOCUS | `x1.5` |
| `{jump}` | `jump_tiles` | DISENGAGE | `3 tiles` |
| `{recover}` | `recover_ap` | REST | `2 AP` |
| `{unlock}` | effective `unlock_offset` (auto = `2 x (n - 1)`) | non-basic, non-system | `+4` (`+0` for rank 1) |

## M. Origin: key fields and tests (skills; classes and items add their own lists)

Key fields of a skill (read-only on `origin = system`, with the reason shown):

| Key field | Why locked |
| --- | --- |
| `id` | references from classes, units, passives, saves |
| `kind` | engine dispatch (command class, planner) |
| `basic`, `system`, `origin` | identity and ownership rules |
| rank structure: the number of ranks (no add or remove rank) | rank ids `<head>_<n>` are referenced by saves and the planner |
| hierarchy: `grade` and `requires` | ownership gate (main spec section 11) |

Everything else (name, description template, level, blocks, numbers, modifier values, weapon gate, status, learned_only, stance) stays editable on system assets.

| Test (`origin_test_suite.gd`) | Check |
| --- | --- |
| load default | a file without `origin` loads every row as system; only `"origin": "user"` is stored |
| delete | `remove` / `remove_skill` refuse system with the reason; succeed for user |
| key lock | `upsert_skill` on a system entry changing a key field returns the locked-field error naming it; changing a number succeeds |
| create | an unknown id created through `upsert_skill` stores origin user; `origin: system` for a new id is refused outside the system-edit scope |
| promote / demote | `promote` flips user to system, persists, then delete is refused; promoting a system asset is an error; `demote` flips back and delete works again; `demote` of a basic, engine row or an asset referenced by a test/seed fixture is refused with the references listed |
| safe delete | `tools/safe_delete.gd`: scan lists test, seed, class path, unit loadout, passive and `requires` references; refuses while any exist (nothing changes); removes an unreferenced system fixture only with `--apply` |
| scope | `begin_system_edit` / `end_system_edit` allow key edits for seeds and tests only; the Forge never calls them; the scope is off after `reload_from_disk` |
| basics | all basics and `REQUIRED_IDS` are origin system (`validate_all` invariant) |
| ownership | `Combatant.can_own_family`, `learn_skill` and passive grants refuse a skill when ANY `requires` entry is unowned, allow it when ALL are owned (two-entry fixture), allow skills with empty `requires` |
| class path | a basic in a path is an error; a step whose `requires` are not ALL on the path at a level <= its own is an error; a requirement at equal level is accepted; grade >= advanced below level 5 is an error; the KNOW reduction applies to grade >= advanced |
| basics grant | `ContentFactory.unit` of any class owns every basic from level 1 with no path row; no duplicate if also in `extra_skills` |
