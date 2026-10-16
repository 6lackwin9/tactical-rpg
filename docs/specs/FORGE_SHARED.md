# FORGE_SHARED: shared Content Forge rules (slimmed from FORGE1_SKILLS and its matrix)

Status: done (built in SKILL1-L / SKILL1-A; these rules are live and every later Forge pass, FORGE1-ITEMS / ITEM1 and FORGE1-CLASSES / CLASS1, builds on them)
Source: copied verbatim on 2026-10-17 (CLN-5) from `docs/specs/FORGE1_SKILLS.md` sections 4.1, 7 (two rows), 11 and 12 and from `docs/specs/FORGE1_SKILLS_MATRIX.md` appendices G and M. The full skill-tab spec and the rest of the matrix are archived as `docs/archive/specs/FORGE1_SKILLS.md` and `docs/archive/specs/FORGE1_SKILLS_MATRIX.md`.

Section numbers are kept so that code comments and other specs that cite "FORGE1 4.1", "FORGE1 11", "FORGE1 11.2", "FORGE1 11.3", "FORGE1 section 12", "FORGE1 section 12.5", "FORGE1 appendix G" or "FORGE1_SKILLS 12.3" resolve here. Citations of the skill-only parts (sections 2, 3, 4, 5, 6, 8, 9, appendices A-F and H-L) resolve in the archived full spec.

| Section | Topic | Where |
| --- | --- | --- |
| 2, 3 | skill storage (S3) and ranks | archive |
| 4 | skill kinds and the weapon gate | archive |
| 4.1 | `weapon_hand` | here |
| 5, 6 | description tokens, guard rails and UI (skills) | archive |
| 7 | schema contract (shared field keys and types here; the skill rows in the archive) | here (excerpt) |
| 8, 9, 10, 13 | SKILL1-A assets, milestones, out of scope, decided list | archive |
| 11 (11.1, 11.1b, 11.2, 11.3) | grade, requires, basics, ownership | here |
| 12 (12.1-12.5) | origin, key fields, delete/promote, seeds, SAFE DELETE | here |
| appendix G | typed controls | here |
| appendix M | origin key fields (skills) and origin tests | here |

## 4.1 `weapon_hand` (replaces the bool `offhand`; user answer 2026-10-16, strict hand typing)

| Item | Contract |
| --- | --- |
| Field | `weapon_hand`, enum `main` (default; the active main weapon), `off` (the off-hand weapon, or bare hands when the main weapon is two-handed; STR+DEX unarmed scaling as today), `both` (dual-wield skill using both weapons). Section `damage`, ATTACK only, base-only (identical across ranks); a non-default value on another kind is an error |
| Validation | `off` and `both` need `dash_tiles` = 0 (the old offhand rule). `both` is rejected while `SkillDef.BOTH_ENABLED` is false (SKILL1 ships false; the Forge lists it disabled, tooltip "engine pending"). Warning: `weapon_types` names a type whose family has no `off` hand (`off`) or lacks `off` or a main hand (`both`) |
| Engine surface in SKILL1 | `SkillDef.weapon_hand` (MAIN, OFF, BOTH); `SkillDef.offhand` stays a derived getter (`weapon_hand == OFF`) and the flat row keeps a derived `offhand` key, so the ~12 engine readers (planner, AI, `DamageCalculator.expected`, dojo, preview) are untouched; `Combatant.wielded_type(hand := MAIN)` returns that hand's weapon type (`unarmed` for bare hands) |
| Gate (`weapon_types`, `min_proficiency`) | the checked weapon is the one the skill uses: `main` = main-hand weapon (today), `off` = off-hand weapon type or `unarmed` (proficiency `fist`), `both` = BOTH weapons must satisfy the type list and the proficiency minimum. `offhand_jab` has no gate, so nothing shipped changes |
| Migration | stored `offhand: true` -> `weapon_hand: off`; false/absent -> omitted. Shipped: one row, `offhand_jab` (grep of `data/`). `normalize` accepts the legacy bool for one release; SKILL1-A step 1 rewrites the file |
| Hand categories (round 4) | Off-category items are shield, orb and tome (support items, ITEMS 8.1); dagger, shortsword and knuckles are main-hand weapons. So under the strict engine rule a `weapon_hand: off` skill (offhand_jab) swings with bare hands (STR+DEX) whenever the off hand holds shield, orb, tome or nothing, and `weapon_types` on an `off` skill can only be satisfied by `unarmed` (the 4.1 warning fires for any other type). Only the dojo's legacy flag `Equipment.allow_plain_dual_wield` puts a plain one-hander in the off hand, where `off` and `both` then use it |
| `both` offer rule | `can_use` false ("Needs two weapons") unless the unit holds a main and an off WEAPON (a shield does not count) at cast time; since off-category items are support items, this is true only under the legacy flag. Engine follow-up, NOT SKILL1 |
| `both` damage (recommended; engine follow-up) | TWO hits through the existing paths: main hit = `compute(offhand=false)`, off hit = `compute(offhand=true)` (own weapon, scaling, crit and guard roll), each at `power_mult` x `DUAL_HIT_FACTOR` (engine constant, start 0.6, tuned by the balance lane). `DamageCalculator.compute` unchanged; the dual-strike combo (`DojoSession` ~250) is the template. Rejected: main hit plus a fraction of the off weapon (loses the off weapon's scaling and type, needs a new compute parameter) |

Later state (CLN-3, not part of the copied text): the derived `offhand` / `advanced` getters and the legacy-key normaliser are scheduled for removal (DECISIONS "Cleanup pass 1 answers", `docs/specs/CLEANUP.md#CLN-3`).

## 7. Schema contract (excerpt: the shared field keys and types)

| Item | Contract |
| --- | --- |
| New schema field keys | `section` (string list), `group` (Forge section id), `rank` (bool, rank-modifiable), `dyn_min` / `dyn_max` / `enabled_when` (named rules resolved in `SkillFormModel`), `filters` (per kind) |
| New field types | `ranks` (list of Dictionary), `multi_pick` (list of string with `options` / `options_from`), `template` (text with tokens and `SkillText` validation); `strings` stays for other kinds |

The skill-specific rows (grade, requires, versions, `SkillRanks`, `SkillText`) are in the archived full spec, section 7.

## 11. Grade and requirements (`grade`, `requires`; replaces `advanced` and `evolves_from`, round 4)

Every skill stays its own family (DECISIONS "Skill model v2"); the hierarchy only gates which owned skills give access to learning another. Ranks and ids are unchanged.

| Field | Contract |
| --- | --- |
| `grade` | enum `novice` (default), `advanced`, `expert`; key field; base-only (identical across ranks); not on engine rows. `expert` is RESERVED: the enum value exists and passes validation, no expert skill ships (it is the worked example of the hierarchy). Forge lists it but the picker note says "no expert skills yet" |
| `requires` | list of head skill ids, default `[]`; key field; base-only. ALL entries must be owned to own or learn the skill. Per entry: exists, is a head, not `system`, not `basic`, not itself, no duplicate, and its `grade` is STRICTLY LOWER than this skill's grade. So a novice skill cannot have `requires`; an advanced skill may require novices; an expert may require novices and advanced |

Cycles: impossible by construction (every edge goes to a strictly lower grade), so no cycle search exists; the lower-grade check is the only graph rule. A required skill with dependents cannot be deleted (`references_to` counts `requires`); a grade change is refused while it would break the strictly-lower rule for any dependent or requirement (names the skill).

Complexity judgement: NOT too convoluted for SKILL1. It is already the minimal form (one enum, one id list, one validator rule, one loop in the ownership check), and it is smaller than the `evolves_from` draft (no cycle search, no one-step rule). Fallback if commit 2 runs long: ship the `requires` control as a plain filtered checklist; schema, validation and ownership stay. Deferred add-on system: expert content, graph view, OR-requirements, per-rank requirements. Replaced ideas are logged in HISTORY.md.

### 11.1 Shipped pairings (SKILL1-A step 2)

| Skill (grade advanced) | `requires` |
| --- | --- |
| greater_mend | [mend] |
| deadeye | [aimed_shot] |
| judgement | [smite] |
| charge_in | [lunge] |
| lacerate | [bleeding_cut] |
| assassinate | [quick_stab] |
| cyclone_kick | [sweep_kick] |

`renewal`, `inferno`, `glacial_burst` and `shield_wall` are grade advanced with `requires` empty: no ownership requirement applies. Setting one later is a normal key-field edit.

### 11.1b SK2 rules that read the grade

1. Class path floor: a path step whose skill has `grade` >= advanced needs level >= `StatRules.ADVANCED_MIN_LEVEL` (5; expert uses the same floor until an expert skill exists).
2. The KNOW unlock reduction (SK2) applies to skills with `grade` >= advanced (code: `grade != novice` replaces the `advanced` read).
3. The Forge filter chip "advanced" becomes a grade chip (novice / advanced / expert).

### 11.2 Basic skills

Basics are NOT valid in a class path and exist only in the skills tab. Every unit owns all basic skills (`SkillDef.basic`) from level 1 automatically.

1. `ContentFactory.unit` grants every basic via `SkillCatalog` (the basics list) instead of relying on class path rows; ids are de-duplicated against `extra_skills`. Basics need no unlock level and cannot be removed from a unit.
2. `_class_errors` rejects a basic skill in `skill_path` (message names class and skill).
3. The shipped class paths contain `strike` / `guard` rows; SKILL1-A step 3 removes them. The default loadout and AI read owned families, so behaviour is unchanged (acceptance: class, arena, planning and progression suites pass).
4. Basics are origin system (section 12).

### 11.3 Ownership requires every required skill

A unit may only own or learn a skill if it owns ALL families in its `requires`. Empty `requires` = no requirement.

1. Class path validation: a step with non-empty `requires` needs EVERY required family in the SAME path at an unlock level <= the step's level (equal allowed). Message names class, skill and the missing or too-late requirement.
2. Unit validation: a skill in `extra_skills` or `loadout` needs every required family in the class path (any level) or in `extra_skills`.
3. Engine: `Combatant.can_own_family(family_id) -> bool` (true when `requires` is empty or `owns_skill(r)` for every r). `learn_skill` refuses (no-op plus `push_warning`) when false; passive grants (the `passives.gd` call to `learn_skill`) go through the same check, so a grant with unmet requirements does nothing.
4. Saves keep what they already own (no stripping); the check applies to new learning only.
5. Forge control: `requires` is a multi-select family picker (`multi_pick`, `options_from` skills) filtered to heads of a lower grade than the edited skill (non-system, non-basic, not itself); disabled with a reason while `grade` is novice; shown as "Requires: Mend, Smite". The class path editor flags the same rule.

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

Shipped assets are system. Their KEY fields are read-only in the Forge; non-key fields (numbers, text) stay editable. The schema marks them `key: true`. Skill key fields (table in appendix M): `id`, `kind`, rank structure (number of ranks), hierarchy (`grade`, `requires`), plus the identity flags `basic`, `system`, `origin`. Classes and items define their lists in their own specs. Purpose of the lock: system assets are used by tests and are balance-tuning targets, so the lock marks what must not drift; to change a key field, clone the asset (12.3) or edit inside the system-edit scope.

UI: a locked control is shown disabled with the reason ("System asset: key field locked. Create your own copy to change it") as tooltip and a lock icon; the ranks section hides add and remove rank for system skills. Enforcement is not UI-only: `upsert_skill` and `upsert` compare the key fields of an existing system asset with the stored one and return an error naming the field (nothing changes).

### 12.3 Delete and promote

1. DELETE is allowed only for `origin = user`. `ContentCatalog.remove` and `remove_skill` refuse system assets with the reason, in addition to the existing refusals (referenced, `basic`, `REQUIRED_IDS`). The Forge disables Delete with the reason as tooltip. Basics are origin system, so the basics-cannot-be-deleted rule is now a special case of this rule (the explicit `basic` refusal stays as a second guard).
2. A user asset that a game test uses is converted with the explicit action "Promote to system" (`ContentCatalog.promote(kind, id) -> String`, empty on success): sets `origin` to system, saves, and delete is locked while it stays system. A confirmation names the consequences (key fields locked, delete locked). Promote IS undoable (round 4): `ContentCatalog.demote(kind, id) -> String` (Forge action "Demote to user") sets `origin` back to user. Refused for a basic, a `REQUIRED_IDS` skill, an engine row, and any asset the SAFE DELETE reference scan (12.5) finds in tests or seeds (the refusal lists them), so demote cannot be used to delete test or tuning targets by the back door. The confirmation names the consequence (delete becomes possible, key fields unlock).
3. Clone (copy to a new id) creates a user asset, giving a way to change key fields of a system asset's copy.
4. Deleting a system asset is never done from the Forge; it goes through SAFE DELETE (12.5).

### 12.4 Interaction with `upsert` / `remove` and seeds

1. `ContentCatalog.upsert` and `upsert_many` of an existing system asset: key-field changes are errors, other changes apply. A new id gets `user`. Tests and runtime fixtures that upsert new ids therefore stay removable by `remove`.
2. Seeds and migrations that must change a key field of a system asset (SKILL1-A step 2) wrap the calls in `begin_system_edit()` / `end_system_edit()`. Only seeds and tests call them, never the Forge; the scope resets on `reload_from_disk`.
3. `remove()` of a system asset is refused even inside the system-edit scope unless the scope's `allow_remove` flag is set (seed cleanups only).
4. `references_to` is unchanged; a user asset referencing a system asset is normal, the reverse (a system asset referencing a user asset) makes the user asset not deletable until promoted or unreferenced and a warning on `validate_all`.

### 12.5 SAFE DELETE (system assets of every kind)

Deleting a system asset is rare and risky (tests, seeds, balance sims read them), so it is a process, not a button. Tool: `tools/safe_delete.gd` (headless script, `tools/fc.sh safe_delete <kind> <id> [--apply]`), run by an agent or by hand; the Forge only shows "Use SAFE DELETE (or clone)" on the disabled Delete.

1. Scan (always, read-only): for the id (and every rank id for skills) list references in: tests (`scripts/tests/*`, `scenes/dojo/*` text search), seeds (`data/seeds/*`), class `skill_path` / gear / pouch, unit `extra_skills` / `loadout` / gear, passives, other skills' `requires`, item `family`, named items, world files (`ContentCatalog.references_to` plus a text grep for the rest). Output: the list with file and line.
2. Refuse (exit non-zero, nothing changes) while any reference exists, printing the list. Also refused for basics, `REQUIRED_IDS` and engine rows.
3. With zero references and `--apply`: removes through `ContentCatalog.remove` inside the system-edit scope with `allow_remove`, saves, runs `validate_all`, prints the result. The planner runs the test suite afterwards.
4. Alternative for the user who only wants a variant: clone (12.3) and edit the clone.
5. Test: a fixture system asset with a seed reference is refused with that reference listed; an unreferenced one is removed.

Tests: appendix M (`origin_test_suite.gd`).

Later state (CLN-2, not part of the copied text): executed seeds leave `data/seeds/` after their task merges, so the scan of step 1 no longer counts merged seeds as references (HISTORY 2026-10-17 seeds row).

## Appendix G. Typed controls (from FORGE1_SKILLS_MATRIX)

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

## Appendix M. Origin: key fields and tests (skills; classes and items add their own lists)

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
