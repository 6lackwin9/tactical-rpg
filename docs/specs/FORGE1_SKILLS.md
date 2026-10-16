# FORGE1_SKILLS: Skill tab of the Content Forge (family as template, ranks from modifiers, dynamic descriptions)

Status: proposed (revised with DECISIONS "FORGE1 skill answers 2" and "FORGE1 class answers"; ready for approval; supersedes the skills part of `FORGE1_DESIGN.md`, now archived in `docs/archive/FORGE1_DESIGN_SKILLS.md`)
Depends on: SK2 (done; `advanced`, `unlock_offset`, `upsert_many` exist)
Lane: logic (SKILL1-L, schema commit first), then one small asset task (SKILL1-A, migration seed)
Appendix: `docs/specs/FORGE1_SKILLS_MATRIX.md` (field by kind matrix, per-field guard rails, typed controls, form sections, filters, tests K, tokens L, origin key fields M). Section letters A-M below refer to it.
Touches (when approved): see section 9 for the file list per commit; data: `data/catalog/skills.json` (127 rows to 49 entries), class paths (basics removed), seed `data/seeds/skill1_ranks.gd`. The classes and items passes reference section 12 (Shared Forge rules); their editors are NOT in this spec.

HISTORY.md check: nothing here re-proposes a deprecated idea. Related rows kept: "Hard-coded classes, items, skills in code -> JSON catalog + Forge", DECISIONS "Skill model v2" (advanced skills are separate families), "Tier numbers" (every rank keeps explicit numbers), "Basics v3" (basics are single-tier), "Content edits through `ContentCatalog.upsert`". One earlier recommendation is replaced on its original reason: `FORGE1_DESIGN.md` 1.9 chose flat storage plus a grouped view (S1) because a nested file looked too costly; section 2 gets the nested file cheaply (flat rows in memory), so S1 is superseded. The description token system was listed out of scope in the first draft; the user asked for it (DECISIONS "FORGE1 skills answers").

## Goal

A skill is ONE thing to author: a family template (identity, cost, targeting, gating, one kind block, one description with tokens) plus up to two rank modifiers. The engine keeps seeing per-rank `SkillDef` objects with the existing ids (`mend`, `mend_2`, `mend_3`), so saves, passives, class paths, the planner and about 60 test/seed call sites keep working. The Forge offers only allowed values and shows every other validator finding as a reason next to a disabled Save.

## 1. Decisions applied (user, 2026-10-16)

| # | Decision | Where it lands |
| --- | --- | --- |
| 1 | Storage S3: ranked entries on disk, flat rows in memory. The Forge edits the SKILL FAMILY (template) as the primary object; ranked skills sit in their own section (modifier table, per-rank resolved panels, read-only) | sections 2, 3, appendix I |
| 2 | The weapon-gate section applies to ATTACK, DEFEND and SUPPORT (engine `can_use` is kind-blind already). `weapon_types` without `min_proficiency` stays one directional validation rule (the types would be inert). `min_proficiency` without `weapon_types` is legal and has no rule (appendix B finding 1) | section 4, appendix B, F |
| 3 | Basics have exactly one rank (validator rule; all six shipped basics comply; the legacy 5-tier allowance for basics ends) | section 3 |
| 4 | Rank names are DERIVED from the template name and rank index: Mend, Mend II, Mend III. No authored rank name field. Ids stay `head`, `head_2`, `head_3` | section 3 |
| 5 | Dynamic descriptions: one template description with tokens resolved per rank | section 5 |
| 6 | The family is the template; each rank is derived from it by that rank's modifiers, resolved against the base (no cascade) | section 3 |
| 7 | Caps: name 32, description 240 | appendix F |
| 8 | `evolves_from` accepted: optional link advanced -> base; owning an advanced family requires owning its base | section 11 |
| 9 | Basic skills are not valid in class paths; every unit owns all basics from level 1 automatically | section 11.2 |
| 10 | Every Forge kind gets `origin` (system / user): key fields of system assets are locked, delete only for user assets, explicit promote | section 12 |

## 2. Storage (S3) and API

Ranked at rest and in the Forge, flat in memory. The loader runs `SkillRanks.expand` and the saver `SkillRanks.collapse`; `entries("skills")`, `SkillCatalog`, the engine, tests and seeds keep seeing flat rows.

| Option | What it is | Verdict |
| --- | --- | --- |
| S1 | 127 flat rows on disk, Forge grouped view with fan-out writes | superseded: core fields stay duplicated 3x in the file |
| S2 | the store itself holds ranked entries | rejected: rewrite of every reader, 64 call sites in 16 files |
| S3 | ranked entries on disk and in the API, flat rows in memory | accepted: one new pure helper, a loader/saver branch, 4 API functions |

Entry on disk (key order follows the Forge sections; `family` and `tier` are not stored; there is no `name` or `description` per rank):

```json
{
  "id": "mend",
  "name": "Mend",
  "kind": "SUPPORT",
  "level": 2,
  "blocks": 2,
  "description": "{name}: heals {heal}. A friendly unit within {reach}, or yourself.",
  "learned_only": true,
  "target_mode": "ALLY_OR_SELF",
  "heal_stat": "KNOW",
  "heal_base": 8,
  "heal_mult": 1.5,
  "ranks": [
    {"level": 3, "heal_base": 10, "heal_mult": 1.8},
    {"level": 4, "heal_base": 12, "heal_mult": 2.1}
  ]
}
```

Expansion rules (`SkillRanks.expand(entry) -> Array[Dictionary]`):

1. Rank 1 is the base block. Rank n (n >= 2) is the base overlaid by `ranks[n-2]` (modifiers resolved against the BASE, never the previous rank).
2. `id` = head for n = 1, else `<head>_<n>`. `family` = "" for rank 1 and the head id otherwise; `tier` = n (exactly what `ContentFactory.skill_to_entry` shapes today).
3. `name` = base name for n = 1, else `<name> <roman n>` (`SkillCatalog.roman`). `description` = the template rendered for that rank (section 5), stored in the flat row as plain text.
4. Rank count is `ranks.size() + 1`, 1 to `StatRules.MAX_CLASS_TIER` (3); basic and system entries have 1.

API (`ContentCatalog`, skills only; names are the contract):

| Function | Behaviour |
| --- | --- |
| `entries("skills")` | unchanged: flat resolved rows in rank-major order (all rank-1 rows in entry order, then rank 2, then rank 3), so `SkillCatalog.families()` order equals entry order |
| `skill_entries() -> Array[Dictionary]` / `skill_entry(head_id) -> Dictionary` | ranked entries built by `collapse` from the current rows (what the Forge lists and `save` writes); empty dict if unknown |
| `upsert_skill(entry, replace_id := "") -> Array[String]` | normalise, expand, validate the whole set (rank rows plus ranked and text rules) against the store minus the old family; any error means nothing changes; else atomic replace. A head rename renames rank ids and rewrites references (class `skill_path`, passive `skill` / `family`, unit `extra_skills` / `loadout`, `evolves_from`, `<old>_<n>` references) |
| `remove_skill(head_id) -> String` | refuses `system`, `basic`, `SkillKinds.REQUIRED_IDS` and referenced heads (counted for every rank id); otherwise removes all rank rows |
| `upsert("skills", flat_row)`, `remove("skills", id)` | unchanged legacy path for tests and seeds; `_family_errors` gains "non-rank fields identical across ranks", the id convention and contiguous tiers |

File format: `skills.json` is already `{"kind", "version", "entries"}` at version 1. The new file is version 2 (`ContentCatalog.VERSIONS`, default 1 for other kinds). The loader decides by shape: version 2 entries expand; version 1 flat rows load as today. A version 2 file containing `family`, `tier`, or a per-rank `name` / `description` key is a load error (the literal-description escape hatch of section 5 is the one exception). `save` always writes version 2 and refuses, naming family and field, when rows cannot be folded (a base-only field differs between ranks, ids break `<head>_<n>`, tiers not contiguous from 1, a rank without a head). Rollback is a `git revert` of the data file; the loader still reads version 1.

Migration order: SKILL1-L commit 1 lands the helpers, loader/saver and API with the shipped file still version 1 (expanding a flat file is the identity); SKILL1-A runs the seed (section 8). Existing tests and seeds (about 64 `upsert` / `normalize` / `ContentFactory.skill` call sites in 16 files) are not rewritten. New tests may use `SkillRanks.entry(base: Dictionary, modifiers: Array[Dictionary]) -> Dictionary`.

Risks: rank-major row order differs from today's file order (consumers sort by tier or level; arena, ai_skills and planning suites gate it); the new "non-rank fields identical" rule may reject a fixture that varies a base-only field (builder runs the 12 suites that touch skills); two representations are covered by the round-trip test.

## 3. Ranks: the family is the template, modifiers derive each rank

The Forge ranks section shows, for a family with N ranks, one column per rank: rank 1 (the template values, read-only here), then each modifier column. Under the table, one read-only panel per rank shows the RESOLVED row (cost, stance, all effective numbers, rendered name and description). Authoring happens in the template sections and in the modifier cells; the resolved panels are never edited. Actions: add rank (up to 3), remove top rank, reset cell to base, copy previous rank's modifiers (copies values into this rank's own modifiers; no link remains).

Modifier table. A rank modifier is an ABSOLUTE replacement value for one field, stored as a key in `ranks[i]`:

| Field | Rank-modifiable | Value form | Range (kind-visible only) | Shipped use |
| --- | --- | --- | --- | --- |
| level | yes | absolute int | 1-5 | 39 families |
| blocks | yes | absolute int | 1-6 | 1 |
| unlock_offset | yes (rank 2+) | absolute int, -1 = auto | -1 or 0-8 | SK2: riposte |
| power_mult | yes | absolute float | 0.0-6.0 | 27 |
| dash_tiles | yes | absolute int | 0-6 | 2 |
| reach_min, reach_max | yes | absolute int | 0-8, min <= max | 1 |
| status_potency, status_magnitude, status_rounds | yes | absolute int | per appendix F | 17, 1, 8 |
| heal_base, heal_mult | yes | absolute int / float | 0-500, 0.0-10.0 | 3 |
| grant_status | yes | absolute enum | kind-filtered buffs | 1 |
| cleanse_statuses | yes | absolute list | debuffs or ["all"] | 2 |
| mitigation | yes | absolute float | 0.0-0.95 | 2 |
| boost, boost_rounds | yes | absolute float / int | 1.0-5.0, 1-10 | 1 |
| jump_tiles | yes | absolute int | 1-6 | 1 |
| recover_ap | yes | absolute int | 0-12 | 0 (engine row only) |
| name, description | no | derived (name) / template (description) | n/a | n/a |
| everything else (`kind`, `basic`, `learned_only`, `advanced`, `evolves_from`, `origin`, `system`, `stance`, `weapon_types`, `min_proficiency`, `area`, `weapon_hand`, `status_effect`, `target_mode`, `heal_stat`) | no (base-only) | n/a | n/a | identical across ranks in every shipped family |

Why absolute values and not multiplier or offset:

1. DECISIONS "Tier numbers": every rank keeps explicit numbers. An absolute cell is exactly the number the player sees and the validator can range-check it without arithmetic.
2. Shipped progressions are not uniform: riposte mitigation is 0.15 / 0.05 / 0.00, power and potency steps differ per family, level steps differ. A multiplier or offset would need per-family rounding and clamping rules and would drift from the displayed number.
3. Several modifiable fields cannot be scaled: integers with small ranges (rounds, tiles), enums (`grant_status`) and lists (`cleanse_statuses`).
4. Resolving against the base (no cascade) keeps a rank edit local: editing rank 2 never changes rank 3. Seven shipped cells repeat a rank 2 value in rank 3; they simply stay as explicit modifiers.
5. A UI-only step helper ("+10% each rank") may be added later; it would write absolute values and needs no schema change.

Rules:

1. A modifier is valid only for a field visible in the kind's sections and inside the field's range; a base-only or unknown key, a `name`, or a `description` key is an error. A modifier equal to the base value is dropped on save (soft warning in the Forge).
2. `unlock_offset` default -1 means derived (`TIER_UNLOCK_STEP * (tier - 1)`); rank 1 must be absent or 0. `advanced` is stored once on the base, so the SK2 "mixed advanced values within a family" error cannot occur for ranked entries.
3. Basic and system entries have exactly 1 rank: a `ranks` key on them is an error, and `_family_errors` caps basic families at 1 tier (was `StatRules.MAX_TIER` 5). The ranks section shows "single rank" for them.
4. Rank names: the derived name `<name> <roman n>` must be at most 32 characters (the name cap applies to the longest derived name). There is no rank name field and no override.
5. `family` and `tier` are derived keys of the flat row; `SkillDef.family` and `family_id()` are unchanged for the engine.

## 4. Kinds and the weapon gate

`SkillKinds` (new, `RefCounted`, static) is one table keyed by `SkillDef.Kind`: `label`, `authorable` (ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE; REST/MOVE/TARGET/STATUS are engine-only), `sections`, `stance` (OFFENSIVE, DEFENSIVE, TACTICAL, KEEP), `ap` (LEVEL, PER_TILE, FREE), `cast` (FOE, SELF, ALLY, TILE), `icon`. `SkillDef.stance`, `keeps_stance`, `switch_cost`, `ap_cost`, `icon` and the self-kind loop in `combat_plan.target_options` read it; behaviour is identical (stance, AP and planner tests pass unchanged). `SkillDef` stays flat: about 166 reads in 17 non-test files make nesting costly for no behaviour change.

Schema fields get `section` (list; `status_rounds` is in two) and `group` (Forge section). Sections: `damage` (power_mult, weapon_hand, dash_tiles), `inflict` (status_effect, status_potency, status_magnitude, status_rounds), `reach` (reach_min, reach_max), `area`, `support` (target_mode, heal_stat, heal_base, heal_mult, cleanse_statuses), `grant` (grant_status, status_rounds), `guard` (mitigation), `focus` (boost, boost_rounds), `jump` (jump_tiles), `rest` (recover_ap), `gate` (weapon_types, min_proficiency). Kind map: ATTACK = damage, inflict, reach, area, gate; SUPPORT = reach, support, grant, gate; DEFEND = grant, guard, gate; FOCUS = focus; DISENGAGE = jump; REST = rest. The `when` lists are generated from it, and one loop ("field set but kind has no section for it") replaces about 10 hand-written "belongs to" errors.

Gate change: `gate` joins DEFEND and SUPPORT. No engine change (`Combatant.can_use` is kind-blind). No shipped DEFEND or SUPPORT row sets a gate, so no data changes. Planner, AI and preview already go through `can_use` / the "Needs ..." text; the test in appendix K proves a gated DEFEND fixture is unusable without the weapon. `forge_preview` renders common lines (cost, stance via `stance_after`, rank list, gate) once and one function per section.

### 4.1 `weapon_hand` (replaces the bool `offhand`; user answer 2026-10-16, strict hand typing)

| Item | Contract |
| --- | --- |
| Field | `weapon_hand`, enum `main` (default; the active main weapon), `off` (the off-hand weapon, or bare hands when the main weapon is two-handed; STR+DEX unarmed scaling as today), `both` (dual-wield skill using both weapons). Section `damage`, ATTACK only, base-only (identical across ranks); a non-default value on another kind is an error |
| Validation | `off` and `both` need `dash_tiles` = 0 (the old offhand rule). `both` is rejected while `SkillDef.BOTH_ENABLED` is false (SKILL1 ships false; the Forge lists it disabled, tooltip "engine pending"). Warning: `weapon_types` names a type whose family has no `off` hand (`off`) or lacks `off` or a main hand (`both`) |
| Engine surface in SKILL1 | `SkillDef.weapon_hand` (MAIN, OFF, BOTH); `SkillDef.offhand` stays a derived getter (`weapon_hand == OFF`) and the flat row keeps a derived `offhand` key, so the ~12 engine readers (planner, AI, `DamageCalculator.expected`, dojo, preview) are untouched; `Combatant.wielded_type(hand := MAIN)` returns that hand's weapon type (`unarmed` for bare hands) |
| Gate (`weapon_types`, `min_proficiency`) | the checked weapon is the one the skill uses: `main` = main-hand weapon (today), `off` = off-hand weapon type or `unarmed` (proficiency `fist`), `both` = BOTH weapons must satisfy the type list and the proficiency minimum. `offhand_jab` has no gate, so nothing shipped changes |
| Migration | stored `offhand: true` -> `weapon_hand: off`; false/absent -> omitted. Shipped: one row, `offhand_jab` (grep of `data/`). `normalize` accepts the legacy bool for one release; SKILL1-A step 1 rewrites the file |
| `both` offer rule | `can_use` false ("Needs two weapons") unless the unit holds a main and an off WEAPON (a shield does not count) at cast time. Engine follow-up, NOT SKILL1 |
| `both` damage (recommended; engine follow-up) | TWO hits through the existing paths: main hit = `compute(offhand=false)`, off hit = `compute(offhand=true)` (own weapon, scaling, crit and guard roll), each at `power_mult` x `DUAL_HIT_FACTOR` (engine constant, start 0.6, tuned by the balance lane). `DamageCalculator.compute` unchanged; the dual-strike combo (`DojoSession` ~250) is the template. Rejected: main hit plus a fraction of the off weapon (loses the off weapon's scaling and type, needs a new compute parameter) |

## 5. Dynamic descriptions

The family carries ONE description: a template with `{token}` placeholders. At expansion each rank's flat row gets the template rendered with that rank's EFFECTIVE values (base overlaid by modifiers), so UI and engine keep reading `description` as plain text and nothing else changes.

### 5.1 Tokens

The token table (token, source field, valid kinds, example) is appendix L: `name ap blocks power dash reach weapons proficiency status potency magnitude rounds heal cleanse mitigation boost jump recover unlock`. Values are the effective field values of the rank being rendered.

Family unlock level lives in class paths, not in the skill, so the template cannot print an absolute character level (out of scope).

### 5.2 Formatting rules

1. Percent tokens (`power`, `potency`, `magnitude`, `mitigation`): value x 100 rounded to an integer, suffix `%`. `power_mult` 2.0 renders `200%`, mitigation 0.35 renders `35%`.
2. Multipliers (`boost`): `x` plus the value with at least 1 and at most 2 decimals (`x1.5`, `x1.25`). `heal_mult` uses the same decimal rule without `x` (`1.0 x KNOW` stays `1.0 x KNOW`, matching today's text).
3. Counted tokens carry their unit with plural: `1 tile` / `3 tiles`, `1 round` / `2 rounds`, `1 block` / `2 blocks`, `AP` is never pluralised.
4. `:n` suffix returns the bare number without the unit: `{rounds:n}` renders `2`, `{dash:n}` renders `4`. Only counted tokens accept it.
5. `{heal}` drops the zero part: base only renders `12`; multiplier only renders `1.5 x INT`.
6. Literal braces are written `{{` and `}}`.
7. The rendered text is not re-parsed (a status name containing braces cannot nest).

### 5.3 Validation

Errors (block Save; `field` = `description`, `rank` named):

1. Unknown token (`{poer}`); the message lists the valid tokens for the kind.
2. Token whose field the kind lacks (`{heal}` on ATTACK, `{potency}` on DEFEND, `{weapons}` on FOCUS).
3. Token that resolves to nothing for some rank (`{dash}` with `dash_tiles` 0 in rank 2, `{status}` with no status, `{magnitude}` without weaken); the message names the rank.
4. Unbalanced or empty braces, a stray closing brace, or `:x` with an unknown suffix.
5. Template longer than 240, or any rendered rank text longer than 240; empty text is allowed.

Warnings (never block):

1. Literal digits in the template outside tokens ("heals 10"): the number can drift from the fields. The message quotes the digits and suggests the matching token when one of the rank-1 effective values equals it.
2. A literal per-rank description (section 5.5).

### 5.4 Live preview

The description editor shows the template with a token palette (only tokens valid for the current kind; click inserts) and a preview line per rank: rank name, rendered text, character count. Errors are shown under the editor and as a cell-level problem in the Save header. The preview updates on every edit of the template, the kind, or any modifier. `SkillText.render(template, kind, effective_row, rank) -> String` and `SkillText.validate(template, entry) -> Array[Dictionary]` are pure static functions in `scripts/content/skill_text.gd`; the Forge, the expander and the validator all call them, so they cannot drift.

### 5.5 Migrating the 127 hand-typed descriptions

Today 39 families have a different description per rank. `data/seeds/skill1_ranks.gd` converts each family:

1. Build candidate tokens for rank 1 from the effective values of each field the kind can token.
2. In the rank-1 description replace each formatted value that occurs exactly once (whole-word match) with its token; the rank name prefix becomes `{name}`.
3. Render the candidate for every rank and compare with the stored text of that rank exactly. If all ranks match, adopt the template.
4. If not, retry replacing only values that differ between ranks, and verify again.
5. If it still fails (hand-edited wording differs between ranks), keep the rank-1 description as a literal template and store the differing ranks as a literal per-rank `description` key in `ranks[i]`. This is the ONLY allowed per-rank description, it is flagged with the literal-override warning, and the seed prints each family that needed it so the user can rewrite them over time.
6. A literal rank-1 description with digits is legal and warns (literal numbers allowed, flagged).

The seed prints counts (templated fully, templated by varying fields only, literal fallback) and fails loudly if any expanded text differs from the stored text. Nothing the player reads changes during migration.

## 6. Guard rails and UI (see appendix F-H)

Typed controls only (appendix G): no free text except id, name and description template; checklists replace the comma text boxes for `weapon_types` and `cleanse_statuses`; dropdowns never offer values the validator rejects (the `grant_status` list is filtered by kind); spin boxes have dynamic limits. Save and Apply stay disabled while `ContentCatalog.validate_skill_entry(entry)` returns problems; each is shown with its field and rank. Soft warnings come from `warnings_for("skills", entry)`. Search, filters and basic-skill protection: appendix J.

## 7. Schema contract (logic lane; schema first)

| Item | Contract |
| --- | --- |
| New schema field keys | `section` (string list), `group` (Forge section id), `rank` (bool, rank-modifiable), `dyn_min` / `dyn_max` / `enabled_when` (named rules resolved in `SkillFormModel`), `filters` (per kind) |
| New field types | `ranks` (list of Dictionary), `multi_pick` (list of string with `options` / `options_from`), `template` (text with tokens and `SkillText` validation); `strings` stays for other kinds |
| New / changed skill fields | `evolves_from` (ref `skills`, default "", section 11); `origin` (enum system / user, default system, section 12; every kind); `description` becomes a template; `weapon_types` / `min_proficiency` visible for ATTACK, DEFEND, SUPPORT; `name` max 32, `description` max 240 |
| Stored keys removed | `family`, `tier` for ranked entries (legacy flat rows still carry them) |
| `ContentCatalog` | `VERSIONS` (skills 2); ranked API (section 2); `validate_skill_entry`; `_family_errors` gains non-rank fields identical, id convention, contiguous tiers, rank cap (basic 1, class 3) |
| `weapon_hand` | section 4.1 (replaces `offhand`) |
| `SkillKinds` | section 4; `REQUIRED_IDS` invariant in `validate_all` |
| `SkillRanks` | pure static: `expand(entry) -> Array[Dictionary]`, `collapse(rows) -> Dictionary` (entries or error list), `resolve(entry, rank) -> Dictionary`, `entry(base, modifiers)`, `rank_fields(kind) -> Array[String]` |
| `SkillText` | pure static: `render`, `validate`, `tokens_for(kind) -> Array[String]`, `format_*` helpers (section 5) |
| Validation messages | new texts name field and rank; texts that existing tests match stay unchanged |

## 8. Assets (SKILL1-A)

| Step | Content |
| --- | --- |
| 1 | Run `data/seeds/skill1_ranks.gd` (idempotent): snapshot all flat rows, run section 5.5 conversion, `ContentCatalog.save("skills")`, `reload_from_disk()`, compare every expanded row field by field (including name and description). Expected: rows 127, entries 49, 39 entries with `ranks` of length 2, 10 without, `validate_all()` empty; print template/literal counts and any name or description over the caps (must be none; if some exist the seed lists them and stops) |
| 2 | Set `evolves_from` for the seven pairings of section 11.1 through `upsert_skill` inside `begin_system_edit` / `end_system_edit` (key field of a system asset) |
| 3 | Remove every basic skill row (`strike`, `guard`, and any other basic) from all class `skill_path`s (section 11.2); print each removal |
| 4 | Audit every class path and unit `extra_skills` for an advanced family without its base (section 11.3); print violators and stop, the user decides the fix |
| 5 | `offhand_jab`: `offhand: true` -> `weapon_hand: off` (4.1). No numbers, names or ids change; `origin` is absent everywhere (= system) |

## 9. Milestones (coarse)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| SKILL1-L | logic | Commit 1 (schema first, also the shared infrastructure for classes and items): `SkillKinds`, section/`rank` schema keys, `SkillRanks`, `SkillText`, version map, loader/saver, ranked API, extended `_family_errors`, DEFEND/SUPPORT gate, basics one rank, caps, `validate_skill_entry`, `REQUIRED_IDS`; `evolves_from` and the ownership rules (11); basics auto-grant and class-path rules (11.2); shared Forge rules `origin` (12); the typed controls (`forge_form.gd`, appendix G); tests. Commit 2 (Forge UI for skills): family-first form, ranks section, description editor with live preview, filter bar, disabled Save with reasons, `SkillFormModel`. SKILL1-A is released after commit 1 | SK2 (done) |
| SKILL1-A | asset | Section 8 | SKILL1-L commit 1 |

Commit 1 files: `scripts/combat/planning/skill_def.gd`, `skill_catalog.gd` (basics list); new `scripts/content/skill_kinds.gd`, `skill_ranks.gd`, `skill_text.gd`; `content_schema.gd` (`origin` on every kind, `evolves_from`, section keys, typed field types); `content_catalog.gd` (loader/saver, ranked API, origin enforcement, `promote`, system-edit scope, family and class-path rules); `content_factory.gd` (basics grant); `scripts/core/combat/combatant.gd` (`can_own_family`, `learn_skill`); `scripts/core/specs/passives.gd` (grant refusal); `scripts/ui/forge/forge_form.gd` (typed controls, locked fields); `scripts/ui/forge/content_forge.gd` (Delete disabled, Promote button); tests: new `skill_ranks`, `skill_text`, `origin` suites plus edits listed in appendix K, registered in `scripts/tests/suite_registry.gd`. Commit 2: `forge_preview.gd`, `content_forge.gd`, new `scripts/ui/forge/skill_form_model.gd`, `combat_plan.gd` (one loop), `skill_form_model` suite.

Ordering: SK2-L and SK2-A are merged, so SKILL1-L can start once approved. It edits hub files (listed above), so do not run it in parallel with another task touching them. Rules that SKILL1-L commit 1 makes errors (basic in a class path, advanced without base) fail `validate_all` on the shipped data until SKILL1-A runs; the branch merges only after both. Parallel-safe: balance sims, the M4 quest runtime, items and classes design passes (docs only).

## 10. Out of scope

Items, classes, units, passives editors and the effect-list editor; engine kind merging; changing skill ids, rank ids, or any number; rewriting descriptions beyond the mechanical conversion; absolute character-level text in descriptions; multiplier/offset modifiers; making `SkillDef` nested; `dojo_session` effect duplication; new gameplay content; balance; any implementation. Tests: appendix K.

## 11. Family link (`evolves_from`) and ownership (accepted)

`evolves_from` is an optional key field (ref to a head skill, default "") that links an advanced family to the base family it upgrades. The engine now reads it for the ownership gate; ranks and ids are unchanged, and advanced skills stay separate families (DECISIONS "Skill model v2").

Field rules: non-empty only when `advanced` is true; the target is a head skill that is not `system`, not `basic`, not `advanced` (one step only), not itself; no cycle; Forge picker filtered accordingly. A base with dependents cannot be deleted (`references_to` counts the field).

### 11.1 Accepted pairings (SKILL1-A step 2)

| Advanced family | Base |
| --- | --- |
| greater_mend | mend |
| deadeye | aimed_shot |
| judgement | smite |
| charge_in | lunge |
| lacerate | bleeding_cut |
| assassinate | quick_stab |
| cyclone_kick | sweep_kick |

`renewal`, `inferno`, `glacial_burst` and `shield_wall` have no base for now: `evolves_from` stays empty and no ownership requirement applies. Setting one later is a normal key-field edit.

### 11.2 Basic skills

Basics are NOT valid in a class path and exist only in the skills tab. Every unit owns all basic skills (`SkillDef.basic`) from level 1 automatically.

1. `ContentFactory.unit` grants every basic via `SkillCatalog` (the basics list) instead of relying on class path rows; ids are de-duplicated against `extra_skills`. Basics need no unlock level and cannot be removed from a unit.
2. `_class_errors` rejects a basic skill in `skill_path` (message names class and skill).
3. The shipped class paths contain `strike` / `guard` rows; SKILL1-A step 3 removes them. The default loadout and AI read owned families, so behaviour is unchanged (acceptance: class, arena, planning and progression suites pass).
4. Basics are origin system (section 12).

### 11.3 Ownership requires the base

A unit may only own or learn an advanced family if it owns the base family (`evolves_from`). A family with an empty `evolves_from` has no requirement.

1. Class path validation: an advanced step with a non-empty `evolves_from` requires the base family in the SAME path at an unlock level <= the advanced level (equal is allowed). Message names class, advanced family and base.
2. Unit validation: an advanced family in `extra_skills` or `loadout` needs the base in the class path (at any level) or in `extra_skills`.
3. Engine: `Combatant.can_own_family(family_id) -> bool` (true if not advanced, `evolves_from` empty, or `owns_skill(base)`). `learn_skill` refuses (no-op plus `push_warning`) when it is false; passive grants (`passives.gd` line that calls `learn_skill`) go through the same check, so an advanced grant without the base does nothing.
4. Saves keep what they already own (no stripping); the check applies to new learning only.
5. Forge: the skill form shows "Requires: Mend" next to `evolves_from`; the class path editor (classes pass) filters or flags the same rule.

## 12. Shared Forge rules (the classes and items passes reference this section)

### 12.1 Origin

Every Forge kind (skills, classes, items, passives, units, and so on) gets one common schema field.

| Item | Contract |
| --- | --- |
| Field | `origin`, enum `system` or `user`, default `system` |
| Storage | `to_storage` omits the default, so only user assets carry `"origin": "user"`; ranked skills store it once on the entry and expansion copies it to every rank row (base-only field) |
| Migration | no data change: every existing row has no `origin` and loads as `system` |
| New assets | `upsert` / `upsert_skill` of an id that does not exist stores `user`; a new asset cannot be created as `system` outside the system-edit scope |
| Validation | unknown value is an error; `validate_all` requires every basic, every `SkillKinds.REQUIRED_IDS` skill and every `system: true` engine row to be origin system |

### 12.2 Key fields (locked on system assets)

Shipped assets are system. Their KEY fields are read-only in the Forge; non-key fields (numbers, text) stay editable. The schema marks them `key: true`. Skill key fields (table in appendix M): `id`, `kind`, rank structure (number of ranks), family link (`evolves_from`, `advanced`), plus the identity flags `basic`, `system`, `origin`. Classes and items define their lists in their own specs.

UI: a locked control is shown disabled with the reason ("System asset: key field locked. Create your own copy to change it") as tooltip and a lock icon; the ranks section hides add and remove rank for system skills. Enforcement is not UI-only: `upsert_skill` and `upsert` compare the key fields of an existing system asset with the stored one and return an error naming the field (nothing changes).

### 12.3 Delete and promote

1. DELETE is allowed only for `origin = user`. `ContentCatalog.remove` and `remove_skill` refuse system assets with the reason, in addition to the existing refusals (referenced, `basic`, `REQUIRED_IDS`). The Forge disables Delete with the reason as tooltip. Basics are origin system, so the basics-cannot-be-deleted rule is now a special case of this rule (the explicit `basic` refusal stays as a second guard).
2. A user asset that a game test uses is converted with the explicit action "Promote to system" (`ContentCatalog.promote(kind, id) -> String`, empty on success): sets `origin` to system, saves, and delete is locked from then on. A confirmation names the consequences (key fields locked, delete locked). Promote is one-way inside the Forge; a demote is only possible by editing the file (git) or in the system-edit scope.
3. Duplicate (copy to a new id) creates a user asset, giving a way to change key fields of a system asset's copy.

### 12.4 Interaction with `upsert` / `remove` and seeds

1. `ContentCatalog.upsert` and `upsert_many` of an existing system asset: key-field changes are errors, other changes apply. A new id gets `user`. Tests and runtime fixtures that upsert new ids therefore stay removable by `remove`.
2. Seeds and migrations that must change a key field of a system asset (SKILL1-A step 2) wrap the calls in `begin_system_edit()` / `end_system_edit()`. Only seeds and tests call them, never the Forge; the scope resets on `reload_from_disk`.
3. `remove()` of a system asset is refused even inside the system-edit scope unless the scope's `allow_remove` flag is set (seed cleanups only).
4. `references_to` is unchanged; a user asset referencing a system asset is normal, the reverse (a system asset referencing a user asset) makes the user asset not deletable until promoted or unreferenced and a warning on `validate_all`.

Tests: appendix M (`origin_test_suite.gd`).

## 13. Decided (all earlier questions resolved, user 2026-10-16)

1. The Forge cannot create new basic skills: the `basic` box is disabled on new skills (tooltip); revisit after the balance pass.
2. Promote to system is NOT undoable in the Forge (file edit or system-edit scope only).
3. `evolves_from` base must be non-advanced (11); `advanced` is a key field (12.2).
4. Strict hand typing: `offhand` becomes `weapon_hand` (4.1); `both` is an engine-lane follow-up outside SKILL1 and ITEM1.
