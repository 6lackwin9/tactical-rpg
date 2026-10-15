# FORGE1_SKILLS: Skill tab of the Content Forge (kind behaviour, ranks in one entry, typed guard rails)

Status: proposed (design only; nothing here is approved; supersedes the skills part of `FORGE1_DESIGN.md`)
Depends on: SK2 (in progress; it adds `advanced` and `unlock_offset` and edits the same hub files, so it merges first)
Lane: logic (SKILL1-L, schema first), then one small asset/migration task (SKILL1-A)
Touches (when approved): `scripts/combat/planning/skill_def.gd`, new `scripts/combat/planning/skill_kinds.gd`, new `scripts/content/skill_ranks.gd`, `scripts/content/content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `scripts/ui/forge/{forge_form,content_forge,forge_preview}.gd`, new `scripts/ui/forge/skill_form_model.gd`, `data/catalog/skills.json` (re-saved, 127 flat rows -> 49 entries), new seed `data/seeds/skill1_ranks.gd`, tests listed in section 9. Items, classes and units are NOT in this spec.

HISTORY.md check: nothing here re-proposes a deprecated idea. Related rows: "Hard-coded classes, items, skills in code -> JSON catalog + Forge" (we keep the catalog), DECISIONS "Skill model v2" (ranks are tiers of one family, advanced skills are separate families: kept), "Tier numbers" (every rank keeps explicit numbers: kept), "Basics v3" (basics stay single-tier: kept), "Content edits through `ContentCatalog.upsert`" (kept; the migration is a seed script). One recommendation CHANGES from `FORGE1_DESIGN.md` section 1.9: that file recommended flat storage plus a grouped view (S1) because a nested file looked too costly. Section 4 shows a cheaper way to get the nested file (flat rows in memory, ranked entries on disk and in the Forge), so S1 is superseded on its original reason.

## Goal

Make a skill ONE thing to author. A skill entry carries its common record once (identity, cost, stance, targeting, gating, progression flags), one behaviour block for its kind, and a `ranks` table of per-rank overrides, so changing a behaviour edits one place instead of three rows. The engine keeps seeing per-rank `SkillDef` objects with the existing ids (`mend`, `mend_2`, `mend_3`), so saves, passives, class paths, the planner and about 60 test/seed call sites keep working. The Forge only offers values the schema allows, and everything else the validator catches is shown as a reason next to a disabled Save.

## 0. What the code does today (checked)

| Fact | Where |
| --- | --- |
| 127 flat rows, 49 families: 39 families with exactly 3 ranks, 10 single rows (strike, offhand_jab, guard, move, switch_target, rest, staggered, stunned, shockwave, reckless_swing). Kinds by row: ATTACK 88, SUPPORT 18, DEFEND 7, DISENGAGE 6, FOCUS 3, STATUS 2, MOVE 1, TARGET 1, REST 1. `skills.json` is 2954 lines | data audit script (node, read only) |
| Every row id follows `<head>` for tier 1 and `<head>_<tier>` for the others; zero exceptions. Every rank name is `<head name> II` / `III`. `learned_only`, `basic`, `kind`, `area`, `stance`, `offhand`, `status_effect`, `target_mode`, `heal_stat`, `weapon_types`, `min_proficiency` never differ between ranks of a family | data audit |
| Fields that DO differ between ranks (families of 39): level 39, description 39, power_mult 27, status_potency 17, status_rounds 8, heal_base 3, heal_mult 3, mitigation 2, cleanse_statuses 2, dash_tiles 2, then reach_min, reach_max, blocks, boost_rounds, grant_status, jump_tiles, status_magnitude 1 each. A ranks table needs 289 override cells in total (78 of them descriptions); 7 of the non-description, non-level cells repeat rank 2 in rank 3 | data audit |
| No data row references a rank id (`_2`, `_3`) outside `skills.json`: class `skill_path`, unit `extra_skills` / `loadout`, passives all name the head id. Rank ids are used by the planner (`SkillCatalog.family_tiers`) and by tests | grep of `data/` |
| `ContentCatalog.upsert` is the single write path; `entries("skills")` hands out flat rows; `references_to` / `_rename_refs` treat `family` as a skill id field; `to_storage` omits fields hidden by `when` | `content_catalog.gd`, `content_schema.gd` |
| `ContentSchema.coerce` clamps numbers silently to min/max, and `forge_form._enum_widget` appends an unknown stored value to the dropdown. Two list fields in skills (`weapon_types`, `cleanse_statuses`) are comma-separated text boxes | `content_schema.gd` 346, `forge_form.gd` 139, 175 |
| `grant_status` dropdown offers every buff (`StatusEffects.buffs()`), but validation accepts only support buffs for SUPPORT and defend+support buffs for DEFEND, so the dropdown can produce an invalid value | `content_schema.gd` 104 vs `content_catalog.gd` 465, 473 |

## 1. Kind versus shared behaviour (answers clarification 1)

### 1.1 Field by kind matrix

Legend: C = common (authored and read for every kind that costs AP/uses a timeline), K = kind-specific (authored and read), D = derived from the kind (not authored), `-` = unused (schema hides it, engine ignores it, validator rejects non-defaults), E = engine fixed value. Engine kinds are REST, MOVE, TARGET, STATUS (all `system` rows except the single `rest` row, which is also `basic`).

| Field | ATTACK | DEFEND | FOCUS | SUPPORT | DISENGAGE | REST | MOVE | TARGET | STATUS | Note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| id, name (`display_name`), description, kind | C | C | C | C | C | C | C | C | C | |
| system | C | C | C | C | C | E | E | E | E | read-only; 5 engine rows |
| level (AP) | C | C | C | C | C | - | - | - | - | MOVE costs 1 per tile, REST/TARGET/STATUS cost 0 (`SkillDef.ap_cost`) |
| blocks | C | C | C | C | C | C | C | C | C | TARGET's command is fixed at 2 blocks (`plan_entry` line 99); STATUS blocks ignore it |
| learned_only, basic | C | C | C | C | C | C | C | C | C | `owns_skill`, `is_always_active` read them for any kind |
| family, tier (SK2: advanced, unlock_offset) | C | C | C | C | C | - | - | - | - | engine rows are single-tier; `advanced` invalid on basic/system |
| stance | K | D | D | D | D | D | D | D | D | ATTACK may override (6 rows DEFENSIVE); engine honours `stance_override` for any kind that does not keep stance, but the schema only exposes it for ATTACK. SUPPORT's `stance_override < 0` branch in `keeps_stance` is unreachable from data |
| weapon_types, min_proficiency | K | - | - | - | - | - | - | - | - | schema is ATTACK-only (DECISIONS "M2C2B-L X1/X2": a scope choice, not a rule); `Combatant.can_use` would enforce them for ANY kind |
| reach_min, reach_max | K | - | - | K | - | - | - | - | - | one concept, two readers: `reach_for` (ATTACK) and `support_reach` (SUPPORT); 33 ATTACK rows and 1 SUPPORT row use it |
| area | K | - | - | - | - | - | - | - | - | 22 rows |
| power_mult | K | - | - | - | - | - | - | - | - | 79 non-default |
| offhand, dash_tiles | K | - | - | - | - | - | - | - | - | offhand 1 row, dash 9 rows |
| status_effect, status_potency, status_magnitude | K | - | - | - | - | - | - | - | - | inflict on hit; 51 rows; magnitude only weaken (3) |
| status_rounds | K | K | - | K | - | - | - | - | - | duration of an inflicted status (ATTACK) or granted buff (SUPPORT, DEFEND): two meanings, one field |
| grant_status | - | K | - | K | - | - | - | - | - | DEFEND 6 rows, SUPPORT 10 |
| target_mode | - | - | - | K | - | - | - | - | - | other kinds are implicitly SELF (or FOE for ATTACK) |
| heal_stat, heal_base, heal_mult | - | - | - | K | - | - | - | - | - | 9 SUPPORT rows heal |
| cleanse_statuses | - | - | - | K | - | - | - | - | - | 6 rows |
| mitigation | - | K | - | - | - | - | - | - | - | 7 rows |
| boost, boost_rounds | - | - | K | - | - | - | - | - | - | `boost` is 1.5 in all 3 FOCUS rows (default), only `boost_rounds` varies |
| jump_tiles | - | - | - | - | K | - | - | - | - | 6 rows |
| recover_ap | - | - | - | - | - | K | - | - | - | 1 row (`rest`, default 2) |

Dead or redundant findings (all verified by grep and the data audit; nothing is unused outright):

1. `weapon_types` alone is inert. `Combatant.can_use` returns true when `min_proficiency <= 0` BEFORE it looks at `weapon_types`, but `forge_preview` shows the same row as "Needs bow". All 61 gated rows set both fields, so the two always travel together. Fix (validation only, no engine change): `weapon_types` non-empty requires `min_proficiency >= 1`.
2. `stance_override` for non-ATTACK kinds and the SUPPORT `keeps_stance` branch are reachable only from hand-built `SkillDef`s (tests), never from data.
3. `level` is unused for 4 engine kinds; `blocks` is unused for TARGET (fixed 2).
4. `system` and `basic` overlap for `move`, `rest`, `switch_target` (both flags set; `is_always_active` is `system OR basic family`). Harmless; keep both (system = engine owns it, basic = counted outside loadout slots).
5. `family` and `tier` are fully derivable from the id (`<head>_<n>`); they exist only to group ranks (section 2).
6. `heal_stat` is coupled to `heal_mult > 0` by two validator lines; it is a dependent field, not an independent one.
7. `status_effect` (inflict, debuff only) and `grant_status` (grant, buff only) are mutually exclusive by kind and differ only in polarity and target. Merging them into one `status` field is possible later (the registry knows polarity) but not worth a storage change now; the Forge can present them as one "Status" section with a polarity-filtered picker.
8. MOVE, TARGET and STATUS are already unauthorable (validator); REST is still listed as authorable in the validator message although only the one `rest` row exists.

### 1.2 Shared behaviour today (where it lives, where it is duplicated)

| Shared behaviour | Where it is implemented today | Duplication |
| --- | --- | --- |
| Identity | `SkillDef` fields; schema fields; `field_errors` | none |
| AP cost | `SkillDef.ap_cost` (special cases REST/TARGET/STATUS/MOVE); Forge preview text "per tile" | preview repeats the MOVE special case |
| Stance | `SkillDef.stance`, `keeps_stance`, `stance_after`, `switch_cost` (four kind lists), `plan_entry` line 105 (`stance_on_start`) | `forge_preview` line 55 re-derives it with `kind != MOVE`, so it shows "Neutral" for SUPPORT and DISENGAGE although the engine keeps the stance |
| Targeting / reach | `reach_for`, `support_reach`, `targets_ally`, `combat_plan.target_options` (self kinds hard-listed: DEFEND, FOCUS, REST, SUPPORT non-ALLY, DISENGAGE), `tactical_ai` buckets | the "who may a skill target" rule is spread over 4 files |
| Status payload | ATTACK: `build_command` sets effect/potency/magnitude/rounds in two branches (area and single); SUPPORT: `SupportRules.apply`; DEFEND: `DefendCommand.on_start` | three delivery paths for one concept |
| Gating (weapon, proficiency, learned, basic, system) | `Combatant.can_use`, `knows_skill`, `owns_skill`, `SkillCatalog.available` | enforcement is already kind-blind; only the schema and validator restrict it to ATTACK |
| Family / rank | `SkillCatalog` family helpers, `ContentCatalog._family_errors`, `_rename_refs`, `forge_preview` sibling list, `PlanOption.group_by_family` | see section 2 |
| Command tail (name, AP, stance switch, stance on start, estimate) | `plan_entry.build_command` lines 102-106 | already shared once; the `match` above it is real per-kind behaviour and stays |
| Authoring visibility | `content_schema._skill_fields`: 4 named `when` lists plus 6 inline ones | the same kind lists as `SkillDef` and the validator |
| "X belongs to Y skills" validation | `_skill_errors`: about 10 hand-written rules (heal, grant, dash, cleanse, jump, magnitude) | restates the schema `when` lists |
| Preview | `forge_preview._skill`: a `match` with 6 kind branches | per-kind text, no section reuse |
| Effects applied outside the command classes | `dojo_session` lines 398-480 re-implement DEFEND, FOCUS, REST effects | out of scope here; noted as a duplicate |

### 1.3 Refactor proposal: a kind descriptor and schema sections (no nesting of `SkillDef`)

1. New `scripts/combat/planning/skill_kinds.gd` (`SkillKinds`, `RefCounted`, static): one table keyed by `SkillDef.Kind` with: `label`, `authorable` (ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE true; REST/MOVE/TARGET/STATUS false), `sections` (list of section ids the kind uses, below), `stance` (OFFENSIVE, DEFENSIVE, TACTICAL, KEEP), `ap` (LEVEL, PER_TILE, FREE), `cast` (FOE, SELF, ALLY, TILE: the planner bucket), `icon`. `SkillDef.stance`, `keeps_stance`, `switch_cost`, `ap_cost`, `icon`, `combat_plan.target_options` self-kind loop and the schema read it. Result: adding a kind or moving a field between kinds is a one-row change plus its command class.
2. Fields get a `section` (and the schema `group` key for the Forge). A field is visible for a kind when its section is in `SkillKinds.sections(kind)`; the `when` lists are generated from it, and the validator rule "field X set but kind has no section for it" replaces the hand-written "belongs to" errors (about 10 lines become one loop). Sections: `damage` (power_mult, offhand, dash_tiles), `inflict` (status_effect, status_potency, status_magnitude, status_rounds), `reach` (reach_min, reach_max), `area` (area), `support` (target_mode, heal_stat, heal_base, heal_mult, cleanse_statuses), `grant` (grant_status, status_rounds), `guard` (mitigation), `focus` (boost, boost_rounds), `jump` (jump_tiles), `rest` (recover_ap), `gate` (weapon_types, min_proficiency). Kind map: ATTACK = damage, inflict, reach, area, gate; SUPPORT = reach, support, grant; DEFEND = grant, guard; FOCUS = focus; DISENGAGE = jump; REST = rest.
3. `status_rounds` belongs to two sections (inflict, grant); a field may list several sections.
4. `SkillDef` stays flat. About 166 reads of its fields exist in 17 non-test files; nesting it would touch all of them for no behaviour change. The "record with sections" is a schema/Forge concept and the on-disk layout (key order follows sections), not an object model.
5. Gating becomes a core section. Today it is shown for ATTACK only. With the descriptor, giving DEFEND or SUPPORT a weapon gate (a shield-gated Shield Wall, a tome-gated heal) is adding `gate` to that kind's list; nothing else changes, because `Combatant.can_use` already enforces it. Not done here (decision: question 3).
6. `forge_preview._skill` keeps one `match` but renders common lines (cost, stance via `stance_after`, rank list, gate) once and then one function per SECTION (so ATTACK and DEFEND share "Status"/"Grant" lines).
7. `build_command` and the command classes stay per kind: that is the behaviour.

### 1.4 Which kinds could merge (count of touch points, non-test code)

| Candidate | What really differs | Touch points to change | Benefit | Recommendation |
| --- | --- | --- | --- | --- |
| DEFEND into FOCUS (one "self stance buff") | different state (`defense_mitigation_ratio` applied from the resolver at block 1 vs `attack_boost_*` plus a StatModifier), stance (Defensive vs Tactical), DEFEND also grants riposte/taunt | DEFEND: 11 sites in 7 files; FOCUS: 9 sites in 7 files, plus `FocusCommand` readers (timeline_dock_ui, timeline_modal, combat_log_builder) and state in combatant, damage_calculator, combat_session and block_snapshot snapshots, unit_inspector, dojo_stage_panel, consumable | one enum value fewer | keep both |
| FOCUS into a SUPPORT self buff (`grant_status: focus` registry buff) | needs a StatusRegistry buff with a multiplier potency channel and a damage-formula hook; breaks the S12 rule that the formula core is untouched | the FOCUS list above plus registry/effects and snapshot undo | fewer kinds | keep (already declined as option B in `FORGE1_DESIGN.md`; no new reason) |
| SUPPORT self buff into DEFEND | `shield_wall` (DEFEND, taunt, -35%) and `taunt_shout` (SUPPORT, SELF, taunt) already overlap: both put a buff on the caster. DEFEND adds Defensive stance and mitigation, SUPPORT keeps the stance and queues the status | SUPPORT 17 sites in 6 files, DEFEND 11 in 7 | authors choose between two kinds for the same buff | keep; the Forge hint on SUPPORT/SELF says "use DEFEND if the skill should also guard and switch stance" |
| DISENGAGE into MOVE | DISENGAGE has its own planner path (landing tile selection), `DisengageCommand`, root refusal, landing rules | DISENGAGE 13 sites in 6 files (combat_plan alone has 7) | none | keep |
| REST authorable | one engine row | 1 validator message | consistency | mark REST engine-only (like MOVE/TARGET/STATUS); costs one validator line and the Forge dropdown filter |

Result: keep all nine kinds, split into authorable (5) and engine (4), and centralise the shared logic through `SkillKinds` (section 1.3). A merge would remove one enum value at the cost of 20+ edits per pair with no authoring benefit, because the descriptor already makes the authoring surface uniform.

## 2. Family and ranks (answers clarification 2)

### 2.1 Every use of `family` today

Categories: R = rank grouping (tiers of one skill), S = slot / ownership identity (what a unit owns or slots), A = advanced-skill linkage, X = other.

| Use | File / function | Category |
| --- | --- | --- |
| Field `family`, `tier` | `skill_def.gd` `family`, `tier`, `family_id()`; schema lines 107-108; `content_factory.skill` / `skill_to_entry` (family "" when equal to the id) | R |
| Family validation: tier limit (basic 5, class 3), unique tier, one kind, same `basic` | `ContentCatalog._family_errors` | R |
| Rename carries rank rows | `ContentCatalog._rename_refs` (skills `family` field) | R |
| Planner groups tiers ("Mend I / II / III") | `SkillCatalog.family_tiers`, `grouped`, `group_label`, `roman`, `PlanOption.group_by_family`, `family()`, `tier()`; `tactical_ai` line 535 sorts by tier | R |
| Family name = lowest tier's name | `SkillCatalog.family_name` | R |
| Skill potency passive and gear effects key on the family id | `Combatant.potency_for(family)`, `potency_for_skill` (`skill.family_id()`), `Passives` `skill_potency` `family` field, validation `skill_exists` | S (the head id is a stable "skill identity" for bonuses) |
| Ownership and loadout slots | `Combatant.owned_families`, `set_loadout`, `knows_skill` (active families), `learn_skill`, `refresh_active`; `ContentFactory.families_of`, `loadout_of`; `UnitState.owned_families`; unit `loadout` (families); `ContentCatalog._unit_errors` loadout checks; `MAX_CLASS_SLOTS` / `MAX_EXTRA_SLOTS` | S |
| Basic family test | `SkillCatalog.is_basic_family`, `is_always_active` | S |
| Class `skill_path` names the tier-1 row of a family (SK2: tier-1 only, one step per family) | `ContentCatalog._class_errors` | S (it already names the head id, which equals the family) |
| Forge preview lists siblings | `forge_preview._skill` lines 56-61 | R |
| SK2 `advanced` flag is family-level and must match across ranks | SK2 `_family_errors` extension | A (stored per row today) |
| SK2 unlock level per family; tier availability = family unlock + `unlock_offset` | `Combatant.tier_unlocked`, `family_unlock_level` | S + R |
| SK2 passive `unlock` `family` | `Passives` | S |
| Test fixtures | `skill_family_test_suite` (`t_fix`, `t_fix_2`, `t_fix_3`), others | R |

Conclusion: nothing about `family` links a basic and an advanced skill (they are separate families, DECISIONS "Skill model v2"). Rank grouping (R) is the only reason a free-text `family` field exists; slot identity (S) needs only the HEAD id, which every consumer already uses. So the field can become derived: `family` = head id, `tier` = rank index.

### 2.2 Proposal: one entry, a base block and a `ranks` table

Entry shape on disk and in the Forge (flat keys in schema order, one nested list):

```json
{
  "id": "mend",
  "name": "Mend",
  "kind": "SUPPORT",
  "level": 2,
  "blocks": 2,
  "description": "Mend: heals 8 + 1.5 x KNOW. A friendly unit within 3 tiles, or yourself.",
  "learned_only": true,
  "target_mode": "ALLY_OR_SELF",
  "heal_stat": "KNOW",
  "heal_base": 8,
  "heal_mult": 1.5,
  "ranks": [
    {"level": 3, "heal_base": 10, "heal_mult": 1.8, "description": "Mend II: heals 10 + 1.8 x KNOW. ..."},
    {"level": 4, "heal_base": 12, "heal_mult": 2.1, "description": "Mend III: heals 12 + 2.1 x KNOW. ..."}
  ]
}
```

Rules of the shape:

1. The top-level fields are rank 1. `ranks` lists the overrides for rank 2, 3, ... (so `ranks.size() + 1` ranks). Overrides are resolved against the base, NOT against the previous rank (editing rank 2 never silently changes rank 3); the Forge table shows the resolved value in every cell, marks overridden cells, and offers "copy previous rank".
2. Resolved row for rank n: base fields, overlaid by `ranks[n-2]`. `id` = head id for n = 1, `<head>_<n>` otherwise. `name` = override or `<name> <roman n>` (matches all 39 shipped families). `description` = override or the base description. `family` = "" for rank 1 and the head id otherwise; `tier` = n. These two keys are no longer stored, they are produced on expansion exactly as `ContentFactory.skill_to_entry` shapes them today.
3. Rank-overridable fields (the `rank: true` schema flag, taken from the data audit plus the obvious numeric amounts): `level`, `blocks`, `description`, `name`, `unlock_offset`, `power_mult`, `dash_tiles`, `reach_min`, `reach_max`, `status_potency`, `status_magnitude`, `status_rounds`, `heal_base`, `heal_mult`, `grant_status`, `cleanse_statuses`, `mitigation`, `boost`, `boost_rounds`, `jump_tiles`, `recover_ap`. Everything else (`kind`, `basic`, `learned_only`, `advanced`, `evolves_from`, `system`, `stance`, `weapon_types`, `min_proficiency`, `area`, `offhand`, `status_effect`, `target_mode`, `heal_stat`) is base-only: it is identical across ranks in every shipped family, and a base-only value in an override is a validation error. This is the same set the earlier design wanted to enforce across flat rows.
4. An override is valid only for fields visible in the kind's sections and within the field's range. An override equal to the base value is dropped when saving (and is a soft warning in the Forge).
5. Rank count: 1 to `StatRules.MAX_CLASS_TIER` (3). Basic and system skills: 1 rank (see question 4 about the legacy basic cap of 5).
6. `family` (Forge, API) becomes a derived read-only key: the head id. `SkillDef.family` and `family_id()` stay unchanged for the engine. `tier` = rank index. `unlock_offset` (SK2) is a per-rank override (default -1 = derived `TIER_UNLOCK_STEP * (tier - 1)`, rank 1 must be 0 or absent). `advanced` (SK2) is stored once on the base, so SK2's "mixed advanced values within a family" error can no longer occur for ranked entries (it stays for legacy flat rows).
7. `evolves_from` (new, optional): ref to another skill entry that is a head, not system, not basic, not itself, and no cycle. Informational ("Greater Mend upgrades Mend": progression and Forge text). The engine does not read it. Advanced skills remain separate families with their own ranks.

### 2.3 What happens to each SK2 rule

| SK2 item | Under this proposal |
| --- | --- |
| `advanced` family-level flag | base-only field on the entry; validation floor (path level >= 5) unchanged and keyed by the head id |
| `unlock_offset` per row | `ranks[i].unlock_offset`; validation (-1 or 0..8, rank 1 absent/0) unchanged |
| Class path names tier-1 rows only | class `skill_path` is unchanged (it already names the head id); the Forge picker lists entries (heads) only |
| SK2 seed: `advanced` on 33 rows, `riposte_2/3` offset 0 | after SK2 is merged the migration folds them into 11 entries with `advanced: true` and `ranks[0].unlock_offset: 0` on riposte |

### 2.4 Storage: how the JSON shape changes (recommended: ranked at rest, flat in memory)

| Option | What it is | Cost | Risk |
| --- | --- | --- | --- |
| S1 (earlier recommendation, `FORGE1_DESIGN.md`) | keep 127 flat rows on disk; Forge shows a grouped view and fan-out writes | schema `group`/`rank`, `upsert_many`, form | core fields stay duplicated 3x in the file; "edit one place" holds only inside the Forge |
| S2 (full) | the store itself holds ranked entries; `entries("skills")` returns ranked entries | rewrite every reader: `SkillCatalog._build`, `forge_preview`, `references_to`, `_rename_refs`, validation, 64 call sites across 16 files (11 test suites, 6 seeds) | high; silent breakage of any missed reader |
| S3 (recommended) | ranked entries on disk and in the Forge/API; the in-memory store stays flat rows, produced by `SkillRanks.expand` at load and folded back by `SkillRanks.collapse` at save | new pure helper `skill_ranks.gd` (expand, collapse, resolve one rank), loader/saver branch for skills, ranked API on `ContentCatalog`, extended `_family_errors`; engine, `SkillCatalog`, tests and seeds untouched | two representations must round-trip; guarded by a round-trip test over all shipped rows |

Recommendation: S3. It gives S2's authoring result (one place in the file, one entry in the Forge, 127 rows -> 49 entries, about half the lines) at close to S1's cost, because everything that reads `entries("skills")` keeps seeing flat rows. S1 is superseded: its reason (the nested file is expensive) no longer holds once the nesting is confined to load/save and the Forge.

API contract (`ContentCatalog`, skills only; names are the contract, bodies are the builder's):

| Function | Behaviour |
| --- | --- |
| `entries("skills")` | unchanged: flat resolved rows, rank-major order (all rank-1 rows in entry order, then all rank-2 rows, then rank-3), so `SkillCatalog.families()` order is the entry order |
| `skill_entries() -> Array[Dictionary]` | ranked entries built by `collapse` from the current rows (what the Forge lists and what `save` writes) |
| `skill_entry(head_id) -> Dictionary` | one ranked entry copy, empty if unknown |
| `upsert_skill(entry, replace_id := "") -> Array[String]` | normalise, expand, validate the whole set (all rank rows plus ranked rules) against the store minus the old family; on any error nothing changes; otherwise replace the old family rows atomically; head rename renames rank ids and rewrites references (class `skill_path`, passive `skill` / `family`, unit `extra_skills` / `loadout`, `evolves_from`, rank-id references `<old>_<n>`) |
| `remove_skill(head_id) -> String` | refuses `system`, `basic` and the required engine skills, and referenced heads (existing `references_to`, counted for every rank id); otherwise removes all rank rows |
| `upsert("skills", flat_row)` / `remove("skills", id)` | unchanged legacy path used by tests and seeds; `_family_errors` gains "non-rank fields identical across ranks" and the id convention |

File format. `skills.json` gets `"version": 2` (per-kind `ContentCatalog.VERSIONS`, default 1 for other kinds). The loader decides by shape: an entry with a `ranks` key, or any entry in a version-2 file, is expanded; an entry in a version-1 file (flat row with `family`/`tier`) loads as before. A version-2 file containing `family` or `tier` keys is a load error. `save` always writes version 2 and refuses (with the family and field named) if rows cannot be folded: a non-rank field differs between ranks, ids break the `<head>_<n>` convention, tiers are not contiguous from 1, or a rank has no head. Normal `upsert` already blocks all four, so this only guards hand-edited data. Rollback is a `git revert` of the data file; the loader still reads version 1.

Migration plan:

1. SK2-L and SK2-A merge first.
2. SKILL1-L lands the helper, loader/saver and API with the shipped file still version 1 (expansion of a flat file is the identity).
3. SKILL1-A runs `data/seeds/skill1_ranks.gd` (idempotent): load, snapshot all rows, `ContentCatalog.save("skills")`, `reload_from_disk()`, compare every row field by field, print counts (rows 127, entries 49, ranks per entry) and fail loudly on any difference; then optionally set `evolves_from` (section 7).
4. Tests and seeds that build flat rows (about 64 `upsert`/`normalize`/`ContentFactory.skill` call sites in 16 files: content 6, combat_extensions 11, skill_family 16, engine_pieces 5, riposte 5, weaken 5, foundation 2, others 1 each; seeds `m2c2b_lib` 5 and `c1a1` 1) keep working unchanged, because the in-memory model and the legacy `upsert` path do not change. A small helper `SkillRanks.entry(head: Dictionary, overrides: Array[Dictionary]) -> Dictionary` is added for NEW tests that want a ranked entry; existing fixtures are not rewritten.
5. Risks: (a) rank-major order differs from today's file order for `SkillCatalog.available()` iteration; consumers sort by tier or level (planner, AI `_ordered`), and `families()` order is unchanged; the arena, ai_skills and planning suites are the gate and any order-dependent assertion is derived instead of hard-coded; (b) the new "non-rank fields identical" rule can reject a test fixture that varies a base-only field across ranks (none of the 12 suites that touch `"skills"` do so in the code read; the builder confirms by running them); (c) two representations: covered by the round-trip test.

Cost versus S1: S3 adds about one new file (`skill_ranks.gd`, pure functions), a loader/saver branch and 4 API functions to the hub files; S1 would add a fan-out writer plus `upsert_many` and still leave 127 rows on disk. The extra cost of S3 is the collapse (diff against the base) and its refusals; the gain is a smaller file, one-place edits for scripts and seeds too, and removal of `family`/`tier` from the file. Explicit recommendation: S3.

## 3. Guard rails and UI (answers clarification 3)

### 3.1 Per-field table

"Control" names the typed control in 3.2. "Rank" = may be overridden per rank. Ranges come from the current schema unless marked NEW (a rule added by this spec). Every free-text field is limited to id, name and description.

| Field | Type | Allowed values / range | Default | Control | Visible when | Rank |
| --- | --- | --- | --- | --- | --- | --- |
| id | id | `^[a-z][a-z0-9_]*$`, unique; rank ids `<id>_<n>` are reserved | "" | id edit with live character filter and uniqueness check | always (read-only after create for basics/system) | no (derived) |
| name | string | non-empty, 1-32 chars (NEW cap) | "New Skill" | line edit + counter | always | optional override |
| description | text | 0-240 chars (NEW soft cap, warning) | "" | multi-line + counter | always | yes |
| kind | enum | authorable: ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE; engine kinds shown read-only | ATTACK | dropdown; switching kind resets hidden-section fields to defaults (existing `to_storage` behaviour) and applies `kind_defaults` | always | no |
| system | bool | read-only | false | label | never editable | no |
| basic | bool | locked after create; basic implies 1 rank | false | checkbox | locked on existing | no |
| learned_only | bool | | false | checkbox | always | no |
| advanced (SK2) | bool | not with basic or system | false | checkbox, disabled for basic | not basic | no |
| evolves_from | ref | head skill, not system, not basic, not self, no cycle | "" | ref picker filtered (kind skills, rank 1, non-system, non-basic) | always | no |
| level (AP) | int | 1-5 (`SkillDef.MIN_LEVEL..MAX_LEVEL`) | 1 | spinbox 1-5 | kinds with AP | yes |
| blocks | int | 1-6 | 1 | spinbox 1-6 | always | yes |
| stance | enum | AUTO, OFFENSIVE, TACTICAL, DEFENSIVE, NEUTRAL | AUTO | dropdown | ATTACK (others show derived text) | no |
| weapon_types | list of enum | subset of `SkillDef.weapon_type_tokens()`, no duplicates | [] | multi_pick checklist | gate (ATTACK) | no |
| min_proficiency | int | 0 to `Proficiency.MAX_LEVEL` (3); at least 1 when `weapon_types` is non-empty (NEW) | 0 | spinbox, minimum becomes 1 once a type is ticked | gate | no |
| power_mult | float | 0.0-6.0 step 0.05; 0 only with a status and no area | 1.0 | spinbox | ATTACK | yes |
| area | enum | NONE, PLUS, BURST, LINE, SWEEP; not with power 0 | NONE | dropdown; invalid options disabled | ATTACK | no |
| offhand | bool | not with dash_tiles > 0 | false | checkbox, disabled when dash > 0 | ATTACK | no |
| dash_tiles | int | 0-6; 0 with an area or offhand | 0 | spinbox, maximum 0 while area/offhand set | ATTACK | yes |
| reach_min | int | 0-8, at most reach_max; 0 when reach_max is 0 | 0 | spinbox, maximum = reach_max | ATTACK, SUPPORT | yes |
| reach_max | int | 0-8; at most 1 for LINE/SWEEP | 0 | spinbox, maximum 1 for LINE/SWEEP | ATTACK, SUPPORT | yes |
| status_effect | enum | "" or one of `StatusEffects.debuffs()` | "" | dropdown from the registry (debuffs only; already so) | ATTACK | no |
| status_potency | int | 1-200 | 0 | spinbox, minimum 1 while a status is set | status_effect set | yes |
| status_magnitude | int | 1 to `StatusRegistry.cap_of(weaken)` (25); only weaken | 0 | spinbox with that range | status_effect is weaken | yes |
| status_rounds | int | 1-5 | 1 | spinbox | status_effect or grant_status set | yes |
| target_mode | enum | SELF, ALLY, ALLY_OR_SELF; SELF forced when grant is taunt | ALLY_OR_SELF | dropdown, options disabled by the taunt rule | SUPPORT | no |
| heal_stat | enum | "", INT, KNOW (`SupportRules.HEAL_STATS`); non-empty iff heal_mult > 0 | "" | dropdown, enabled once heal_mult > 0 | SUPPORT | no |
| heal_base | int | 0-500 | 0 | spinbox | SUPPORT | yes |
| heal_mult | float | 0.0-10.0 step 0.05 | 0.0 | spinbox | SUPPORT | yes |
| grant_status | enum | SUPPORT: `StatusEffects.support_buffs()`; DEFEND: `defend_buffs()` plus `support_buffs()` (list differs per kind, NEW: the dropdown stops offering invalid buffs) | "" | status picker from the registry, filtered by kind | SUPPORT, DEFEND | yes |
| cleanse_statuses | list of enum | subset of `StatusEffects.debuffs()` or exactly `["all"]` | [] | multi_pick; ticking "all" clears the rest | SUPPORT | yes |
| jump_tiles | int | 1-6 | 0 (draft starts at 1 via `kind_defaults`) | spinbox 1-6 | DISENGAGE | yes |
| mitigation | float | 0.0-0.95 step 0.05 | 0.5 | spinbox | DEFEND | yes |
| boost | float | 1.0-5.0 step 0.05 | 1.5 | spinbox | FOCUS | yes |
| boost_rounds | int | 1-10 | 2 | spinbox | FOCUS | yes |
| recover_ap | int | 0-12 | 2 | spinbox | REST (engine) | yes |
| unlock_offset (SK2) | int | -1 (auto) or 0-8 | -1 | "auto" checkbox + spinbox 0-8 (no -1 shown to the user) | rank rows 2+ | yes |
| ranks | list of override rows | 0 to `MAX_CLASS_TIER - 1` rows; each key rank-overridable, visible for the kind, in range | [] | ranks table | not basic, not system | n/a |

`kind_defaults` is a new per-kind map applied once when the kind changes (jump_tiles 1 for DISENGAGE, heal_stat KNOW and heal_base 1 for SUPPORT, mitigation 0.5 for DEFEND ...), so a fresh draft is not invalid for a reason the user cannot see.

### 3.2 Typed controls needed

| Control | Replaces | Source of allowed values |
| --- | --- | --- |
| Enum dropdown that NEVER shows an unknown stored value as a normal choice (an unknown value appears as a red "(invalid: x)" item and the problem list names it) | `_enum_widget` appending unknown values | schema `options` / `options_from` |
| Status picker (enum with `options_from` `debuff`, `buff_support`, `buff_defend_support`) | `status`, `status_buff` | `StatusEffects` / `StatusRegistry` |
| Int and float spinbox with min / max / step and DYNAMIC limits (limits can be a function of other fields, e.g. reach_min <= reach_max, min 1 when a status is set) | `SpinBox` with static limits | schema `min`, `max`, `step`, new `dyn_min` / `dyn_max` ids resolved in `SkillFormModel` |
| Checkbox with enable rules | CheckBox | schema `enabled_when` |
| Ref picker filtered by catalog kind and field values | `ref` dropdown | `ref`, `ref_filter` (extended to "not in", "not self") |
| Multi-select checklist with an allowed set, exclusive items and no free text | comma `strings` line edit | schema `options` / `options_from` (`weapon_type_tokens`, `debuffs_or_all`) |
| Ranks table editor: one row per rank, columns = the rank-overridable fields visible for the kind, resolved values with overridden cells highlighted, "reset cell to base", "copy previous rank", "add rank" (up to 3) and "remove last rank"; cells use the same typed controls | the future grouped view | `rank: true` flags |
| Derived read-only label | family, tier, rank ids, derived stance | computed |
| Character-filtered id edit with live uniqueness check | `LineEdit` for ids | `is_valid_id` |

No skill field needs a free JSON or dict editor after this change (`weapon_types`, `cleanse_statuses` become checklists; `ranks` is a table). The effect-list editor from the passive grammar belongs to the items/passives pass, not here.

### 3.3 Prevent versus detect

| Class | Examples | Mechanism |
| --- | --- | --- |
| Prevented by the control (cannot be entered) | wrong type, out-of-range number, enum outside its set, duplicate list entries, non-existent ref, invalid id characters, a field of a hidden section, a buff on the wrong kind, `min_proficiency` 0 while a weapon type is ticked, dash with an area, reach_min above reach_max, taunt with target other than SELF, an override of a base-only field | typed controls with dynamic limits, `when`/section visibility, picker option filtering, silent clamping removed from the UI path (the SpinBox cannot exceed its range; `coerce` stays as a safety net for scripts) |
| Detected by the validator (cross-field or whole-entry) | id already used, support skill with no payload, heal_stat set while heal_mult is 0, power 0 without a status, weaken magnitude missing, rank count over the cap, basic with ranks, `advanced` on a basic, `evolves_from` cycle, `unlock_offset` rules, a required engine skill missing or changed, rename collisions, rank id collision with another skill | `ContentCatalog.validate_skill_entry(entry) -> Array[Dictionary]` returning `{"field": key, "rank": n or 0, "message": text}`; `validate_entry` keeps returning plain strings by mapping `message` so every existing caller and test is unchanged |
| Soft warnings | override equal to the base, description over 240 chars, description mentions a number that no longer matches a field (only if question 6 is accepted) | `warnings_for("skills", entry)` (separate from errors, never blocks) |

Presentation: while `validate_skill_entry` is non-empty, the Save and Apply buttons are disabled, a red "Cannot save: N problems" header shows at the top of the preview with one line per reason, the offending field (and rank cell) is outlined red with the same text as its tooltip, and the list row gets the existing warning mark. Keyboard Apply is ignored while blocked. Warnings show in yellow below and do not block.

## 4. Form layout (core versus kind sections)

| Order | Section (Forge) | Kind | Fields |
| --- | --- | --- | --- |
| 1 | Identity | core | id, name, description |
| 2 | Classification | core | kind, basic, learned_only, advanced, evolves_from, system (read-only) |
| 3 | Cost and timing | core | level (AP), blocks, stance |
| 4 | Targeting | core (shown when the kind has a cast target) | target_mode (SUPPORT), reach_min, reach_max, area (ATTACK) |
| 5 | Gating | core | weapon_types, min_proficiency (ATTACK today) |
| 6 | Kind modifier | per kind, one of | ATTACK: damage (power_mult, offhand, dash_tiles) and inflict (status_effect, status_potency, status_magnitude, status_rounds); SUPPORT: heal and cleanse (heal_stat, heal_base, heal_mult, cleanse_statuses) and grant; DEFEND: guard (mitigation) and grant; FOCUS: boost, boost_rounds; DISENGAGE: jump_tiles; REST: recover_ap |
| 7 | Ranks | core | the ranks table (collapsed for single-rank skills; basics/system show "single tier") |
| 8 | Preview and problems | derived | resolved per-rank summary (cost, stance, numbers), "Used by" (classes, units, passives), problem and warning list |

Sections 1-5 and 7 are the common record (centralised once); section 6 is the only kind-specific one, generated from `SkillKinds` sections.

## 5. Search, filter and protection

Skill list (one row per entry, showing name, kind, AP range such as "2-4", rank count, flags):

| Control | Behaviour |
| --- | --- |
| Text search | substring over id, name, description (case-insensitive) |
| Filter chips | kind (multi), rank count (1, 2, 3), basic, learned_only, advanced, system (hidden by default), weapon type, inflicts status, grants status, "has problems" |
| Class usage | any of: on a class path (pick a class), extra skill of a unit, referenced by a passive, `evolves_from` target, unused |
| Sort | name, kind, AP (rank 1), rank count, problems first |

`SkillFilter` is a pure `RefCounted` predicate over entries and `ContentCatalog.references_to`, so it is tested headless. Filters are declared per kind in the schema (`"filters"`) so items and classes can reuse the bar later.

Basic-skill protection: `ContentCatalog.remove_skill` and the legacy `remove` refuse `basic` and `system`, and a new `SkillKinds.REQUIRED_IDS` list (strike, guard, move, rest, switch_target, offhand_jab) is a `validate_all` invariant (missing, wrong kind, or `basic` false is a catalog error), because engine code and tests find them by id or by `SkillCatalog.lowest(kind)`. For existing basics the Forge locks id, kind, basic and the rank table (read-only "single tier"); name, description, numbers and modifier fields stay editable. The Delete button is disabled with the reason as its tooltip. A non-basic head with ranks is deleted as a whole (all ranks), only when no reference exists; a rank can only be removed from the top.

## 6. Schema contract (logic lane; schema first: yes)

| Item | Contract |
| --- | --- |
| New schema field keys | `section` (string list, see 1.3), `group` (Forge section id), `rank` (bool, rank-overridable), `dyn_min` / `dyn_max` / `enabled_when` (named rules resolved in `SkillFormModel`), `filters` (per kind) |
| New field type | `ranks` (list of Dictionary) and `multi_pick` (list of string with `options` or `options_from`); `strings` stays for other kinds |
| New skill fields | `evolves_from` (ref `skills`, default "", validated as in 2.2 rule 7); SK2's `advanced` and `unlock_offset` as specified there |
| Stored keys removed | `family`, `tier` for ranked entries (derived on expansion); legacy flat rows still carry them |
| `ContentCatalog` | `VERSIONS` map (skills 2); ranked API in 2.4; `validate_skill_entry`; `_family_errors` gains: non-rank fields identical across ranks, id convention `<head>_<tier>`, tiers contiguous from 1, rank cap by basic/class |
| `SkillKinds` | table in 1.3; `SkillDef.stance/keeps_stance/switch_cost/ap_cost/icon` and `combat_plan.target_options` read it; behaviour identical (acceptance: existing stance, AP and planner tests pass unchanged) |
| `SkillRanks` | pure static: `expand(entry) -> Array[Dictionary]`, `collapse(rows) -> Dictionary` (entries or an error list), `resolve(entry, rank) -> Dictionary`, `entry(head, overrides)`, `rank_fields(kind) -> Array[String]` |
| Validation messages | new texts name the field and rank; existing message texts that tests match stay as they are |

## 7. Assets (asset lane, SKILL1-A)

| Step | Content |
| --- | --- |
| 1 | Run `data/seeds/skill1_ranks.gd`: re-save `skills.json` as version 2. Expected: 127 rows -> 49 entries; 39 entries with `ranks` of length 2; 10 without; every expanded row identical to its pre-migration row (name, description and all fields), `validate_all()` empty |
| 2 | Optional `evolves_from` for the advanced families, only after the user confirms the pairings. Suggested (judged from descriptions, needs confirmation): greater_mend from mend, renewal from rally, deadeye from aimed_shot, judgement from smite, charge_in from lunge, cyclone_kick from sweep_kick, lacerate from bleeding_cut, assassinate from quick_stab; unclear: shield_wall (riposte? guard?), glacial_burst and inferno (firebomb? arcane_bolt?) |
| 3 | No numbers, names or descriptions change |

## 8. Milestone plan (skill tab only)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| SKILL1-L | logic | Commit 1, schema and engine-side data layer: `SkillKinds`, section/`rank` schema keys, `SkillRanks`, version map, loader/saver, ranked API, extended `_family_errors`, `validate_skill_entry`, required-skills invariant, `evolves_from`, new tests. Commit 2, Forge: typed controls, ranks table, filter bar, disabled Save with reasons, section-based preview. The asset task is released after commit 1 | SK2 merged |
| SKILL1-A | asset | Section 7 (migration seed, optional `evolves_from`) | SKILL1-L commit 1 |

Hub-file conflicts with SK2 (in progress): `skill_def.gd`, `content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `forge_preview.gd`, and SK2 also edits `combatant.gd`, `passives.gd`, `stat_rules.gd`, `unit_state.gd`, `skills.json`, `classes.json`. SKILL1-L edits the first five plus `combat_plan.gd` (one loop), `forge_form.gd`, `content_forge.gd`. They overlap, so run sequentially. Recommended order: SK2-L, SK2-A (merge) -> SKILL1-L commit 1 (+ SKILL1-A) -> SKILL1-L commit 2. Safe in parallel with all of it: balance sims, M4 quest runtime, the items and classes design passes (docs only). If the user prefers not to wait, the only SK2-independent slice is `SkillKinds` plus the rank helper; do not start it while SK2 edits `skill_def.gd`.

## 9. Tests (rule-derived; read counts and ids from the catalog, never hard-code)

| Suite | Checks |
| --- | --- |
| new `skill_ranks_test_suite.gd` | round trip: for every shipped row, `collapse` then `expand` reproduces it field by field; entry count equals the number of distinct families and rank counts sum to the row count; expanded ids are `<head>` and `<head>_<n>`; `family_tiers(head).size() == ranks.size() + 1`; editing a base field in a sandbox changes every rank row, editing an override changes only that rank; a base-only key in an override is rejected; an override equal to the base is dropped on save; head rename renames rank ids and class/passive/unit/`evolves_from` references; `remove_skill` removes all ranks and refuses a referenced head; version-1 and version-2 files both load; a version-2 file with `family` or `tier` is a load error; legacy fixture `upsert` of flat rows (the `t_fix` pattern) still works and saves as one entry |
| `content_test_suite.gd` | new validation cases: non-rank field mismatch across flat ranks, id convention, non-contiguous tiers, `weapon_types` without `min_proficiency`, `evolves_from` rules, required engine skills invariant, kind-section rule replaces the old per-field messages (message texts that existing tests match are kept) |
| `skill_family_test_suite.gd` | family validation unchanged; add a ranked-fixture variant using `SkillRanks.entry` |
| new `skill_form_model_test_suite.gd` | headless `SkillFormModel`: visible fields per kind equal `SkillKinds` sections, dynamic limits (reach, dash, min potency), kind-switch resets hidden fields, disabled-Save reasons list, `SkillFilter` predicates, ranks table resolved cells and override highlighting; one `results.append` line in `suite_registry.gd` |
| `skill_engine`, `foundation`, `riposte`, `arena`, `planning`, `ai_skills`, `world`, `runner`, `progression`, `save` suites | pass unchanged (stance, AP cost, planner and AI behaviour, saves): the acceptance gate for the descriptor refactor and the storage change |

## 10. Out of scope

Items, classes, units, passives editors and the effect-list editor; engine kind merging; changing skill ids, rank ids or any number, name or description; making `SkillDef` nested; changing weapon gating beyond the `weapon_types` needs `min_proficiency` rule; changing `dojo_session` effect duplication; a description token or template system; new gameplay content; balance; any implementation.

## 11. Open questions (for the user)

1. Storage: confirm S3 (ranked entries on disk and in the Forge, flat resolved rows in memory) in place of the earlier S1 recommendation. The alternative S2 (ranked entries in memory too) is far more invasive for the same visible result.
2. `evolves_from`: add the optional link (display only), and if yes confirm the pairings in section 7 step 2; or keep advanced families fully independent as today?
3. Weapon gating for non-ATTACK kinds: keep it ATTACK-only (current decision), or open the `gate` section to DEFEND/SUPPORT (shield-gated Shield Wall, tome-gated heals)? The engine already supports it; it is a gameplay decision, not a technical one.
4. Basics may currently have up to 5 tiers (`StatRules.MAX_TIER`), but "Basics v3" says single-tier. Enforce 1 rank for basics now (all shipped basics comply), or keep the 5-tier allowance?
5. Rank names: derive "Mend II" automatically with an optional override (matches all 39 shipped families), or always authored?
6. Rank descriptions contain hand-typed numbers ("heals 10 + 1.8 x KNOW"), and all 39 ranked families differ in description. Keep per-rank description overrides as now, or add a token/auto-text system later so numbers cannot drift (separate task)?
7. Overrides resolve against the base, not the previous rank (7 shipped cells repeat a value from rank 2 in rank 3). Accept that, or use cascade (rank 3 inherits rank 2), which is shorter but lets a rank 2 edit change rank 3?
8. Name and description length caps (32 and 240) are new limits: accept or choose others?
