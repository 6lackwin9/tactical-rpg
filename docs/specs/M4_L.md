# M4-L: Quest and flag runtime (S8), structured quest schema

Status: proposed (spec written 2026-10-17; awaiting the user's answers below and approval)
Depends on: M3 (done: `GameState.flags`, opaque `GameState.quests`, `SaveGame` v1, `SaveMigrations`)
Lane: logic (one branch; commit 1 is the schema slice that releases M4-A)
Touches: new `scripts/quests/{quest_catalog,quest_rules,quest_runner,reward_grant}.gd` (+ its `README.md`), new `data/quests/quests.json` (empty envelope only), `scripts/world/world_schema.gd` (`_quest_fields`), `scripts/world/world_catalog.gd` (`_quest_errors`, `_json_refs_to` quest part), `scripts/game_state/{game_state,save_game,save_migrations}.gd`, new `scripts/tests/quest_test_suite.gd` + one line in `suite_registry.gd`, `scripts/tests/save_test_suite.gd` (the opaque-blob fixture). Reads `ContentCatalog`, `PartyState`, `UnitState`, `LevelUp` (no edits). No overlap with the cleanup batch (CLN-9 edits `content_catalog.gd`, which M4 does not touch).

## Open questions for the user

The rest of this spec follows the recommendation (R) of each question. HISTORY check: nothing below revives a deprecated idea; the event-bus quest option (P-S8 C, archived ROADMAP) stays not adopted; passives stay source-granted (classes-units "Passives"), quests become one of the sources.

1. Quest structure: where does gameplay live?
   - A (R) Base template + world binding. `data/quests/quests.json` holds lore-free templates (objective kinds in order, rewards); a world quest names a `template` and binds each objective to a world target through its existing `steps` (text, location, encounter, flag). Same split as base unit + world character: two worlds using one template give the same rewards. Cost: one small catalog; existing world steps already fit. Risk: rewards cannot be tuned per world (by design; a new template is the escape hatch).
   - B World-only: add `objectives` and `rewards` to world quests, no base templates. Cheapest; breaks the "base decides gameplay" layer rule and leaves `data/quests/` unused.
   - C Base-only full quests, worlds only rename. Templates would need world targets, which are not lore-free.
2. Flag scope.
   - A (R) One flat registry per save (the existing `GameState.flags`, key `[a-z0-9_.]`, bool/int/String). A save holds one campaign, so flat = per campaign. Convention (not enforced): authored keys start with the authoring world id (`ashlands.pass_cleared`). Cost: none in storage.
   - B Per-world dictionaries: breaks M3's flag schema (migration) and cross-world quests (`iron_for_embers` ends in Ashlands).
   - C A plus quest-local scratch flags: no current use.
3. Reward types and recipient.
   - (R) Types: `xp` (full amount to every active member, same rule as encounter XP), `items` (base item id -> count, into the party stash), `passives` and `skills` (base ids, to ONE recipient chosen at completion; empty = first active member), plus world `complete_flags`. No gold (no currency exists; S11).
   - Alternatives: passives/skills to every roster member (4x the power per reward), or fixed to the leader (no player choice).
   - Reward budget (the S8 freeze item, archived Q4): (R) soft warnings like `GearBudget`: more than 1 passive, more than 1 skill, `xp` above 300, more than 6 items in total. Hard limits only for malformed data. Confirm or give caps.
4. Failure and branching.
   - A (R) No failure, no in-quest branches. A lost encounter just does not advance (retry); the player may abandon (back to available, progress lost). Campaign branching comes from `complete_flags` + `requires_flags` across quests.
   - B A plus fail conditions (a flag value or a lost encounter sets state `failed`). Small, but needs a rule for re-offering.
   - C Alternate outcomes inside one quest with separate rewards. Needs S11 dialogue to choose; defer.
5. How encounters and locations complete objectives.
   - A (R) Explicit runtime calls from the future game loop (`QuestRunner.on_encounter_won`, `on_location_reached`); no `CombatSession` dependency; only the CURRENT objective of an ACTIVE quest advances (no retroactive credit). Flag objectives are state and are checked whenever quests refresh, so they can be met before the quest starts.
   - B Hook `CombatSession` signals directly: couples combat to S8, harder headless tests.
   - C Encounters set flags and every objective is a flag: flag clutter, loses ids for validation and AssetScan-style checks.
6. Completion.
   - (R) All objectives done = state `ready`; rewards are granted by an explicit `complete(..., recipient_id)` (a turn-in without dialogue, so S11 can later call it from a giver). Alternative: auto-complete on the last objective (then the recipient cannot be chosen).
7. Save format.
   - (R) `SaveGame.VERSION` 2: the `quests` blob becomes structured and validated; a built-in 1 -> 2 migration keeps well-formed entries and drops the rest with warnings. Alternative: keep v1 and an opaque blob (no load validation).
8. Template storage API.
   - (R) Own `QuestCatalog` in `scripts/quests/` for `data/quests/quests.json` (the path STRUCTURE reserves), same contract as `ContentCatalog.upsert`/`save`. Alternative: a `quest_templates` kind in `ContentCatalog`: collides with CLN-9 (validator split in flight) and ITEM1/CLASS1 on the hub file.

## Goal

Give quests a runtime: lore-free quest templates (ordered objectives + rewards), world quests that bind them to world targets, a state machine over `GameState` (start, advance on events and flags, ready, complete with rewards, abandon), reward granting as a passive/skill/item SOURCE, and a structured, validated, migrated quest blob in the save.

## Schema contract (logic lane, schema-first: yes)

### Quest templates: `data/quests/quests.json`

Envelope `{"kind": "quest_templates", "version": 1, "entries": [...]}`. Commit 1 ships the file with `entries: []`.

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| id | String | required | `JsonUtil.id_ok`, unique in the file |
| name | String | required | non-empty after strip; lore-free label ("Reach and clear") |
| description | String | "" | free text, lore-free |
| objectives | Array[Dictionary] | required | 1..`QuestRules.MAX_OBJECTIVES` (8); each `{"kind": K}` with K in `QuestRules.OBJECTIVE_KINDS` = `reach_location`, `win_encounter`, `flag`; any other key is an error |
| rewards | Dictionary | {} | keys only from `xp`, `items`, `passives`, `skills` (unknown key = error) |
| rewards.xp | int | 0 | 0..`QuestRules.XP_MAX` (2000) |
| rewards.items | Dictionary | {} | key = existing `ContentCatalog` item id; value int 1..99; at most 8 keys |
| rewards.passives | Array[String] | [] | each an existing catalog passive; no duplicates |
| rewards.skills | Array[String] | [] | each an existing skill family head (`ContentCatalog.skill_entry(head)` non-empty); not a basic (`SkillCatalog.is_basic_family`); no duplicates |
| tags | Array[String] | [] | strings |

Soft warnings (`QuestCatalog.warnings_for(entry)` / `warnings_all()`, never block saving; constants in `QuestRules`): `rewards.passives.size() > SOFT_MAX_PASSIVES` (1), `rewards.skills.size() > SOFT_MAX_SKILLS` (1), `rewards.xp > SOFT_MAX_XP` (300), sum of item counts `> SOFT_MAX_ITEMS` (6). Starting values; the balance pass may move them.

JSON numbers are read through `JsonUtil.normalize` (whole floats become ints); a non-whole number where an int is required is an error.

### World quests: `world_schema.gd` `_quest_fields` (additive)

Existing fields stay (`id`, `name`, `description`, `giver`, `faction`, `region`, `requires`, `steps`, `reward` text, `tags`). New fields:

| Field | Type (schema) | Default | Validation (`world_catalog._quest_errors`) |
| --- | --- | --- | --- |
| template | `string` | "" | "" = lore-only quest (the runtime ignores it); otherwise an existing `QuestCatalog` id |
| requires_flags | `json` | [] | Array of conditions `{"key": K, "value": V}`; K passes `GameState.flag_key_ok`; V bool/int/String, default true when missing |
| complete_flags | `json` | [] | Array of `{"key": K, "value": V}`, same rules; set in order on completion |
| auto_start | `bool` | false | true needs a non-empty `template` |

Step bindings (the existing `steps` json, one step per template objective when `template` is set):

| Objective kind | Step keys used | Rule |
| --- | --- | --- |
| reach_location | `location` (required) | resolves as a world location from the quest's world (qualified refs allowed, as today) |
| win_encounter | `encounter` (required), `location` (optional, display) | resolves as a world encounter |
| flag | `flag` (required), `value` (optional, default true) | `flag` passes `flag_key_ok`; `value` bool/int/String |

Every step keeps the required `text`. With a template: `steps.size()` must equal `objectives.size()`; a missing required binding is an error naming the step and kind; keys not used by the step's kind are ignored. Without a template the old rules apply unchanged. `requires` may not form a cycle (the quest itself reached through `requires` chains, resolved from the quest's world) = error. `_json_refs_to` keeps listing step `location`/`encounter` refs (rename and delete scans stay correct); flags are not references.

### Quest state blob: `GameState.quests`

Key = the resolved quest entry's qualified id `_qid` (`ashlands:hold_the_courtyard`), so an inherited quest is the same key in every descendant world and a world override gets its own key. A missing key = not started.

| Field | Type | Allowed | Notes |
| --- | --- | --- | --- |
| state | String | `active`, `ready`, `completed` | `QuestRules.STORED_STATES` |
| objective | int | 0..`MAX_OBJECTIVES` | index of the current objective; equals the objective count when `ready` or `completed` |

No other keys. `QuestRules.validate_blob(blob) -> Array[String]` checks: key contains exactly one `:` and both parts pass `id_ok`; value is a Dictionary with exactly these two keys and allowed values. `GameState.validate` calls it (replacing the opaque `JsonUtil.check_safe(quests, ...)`), so `SaveGame.write` refuses a bad blob. Catalog facts (unknown quest, index above the template's count) are NOT save errors: `QuestRunner.check(gs)` reports them as warnings and changes nothing (worlds may be edited between sessions).

### Save version 2

- `SaveGame.VERSION` = 2.
- `SaveMigrations` gets a built-in step 1 -> 2: keep each `quests` entry that passes `validate_blob` for that single entry, drop the others; `migrate` appends one warning `dropped invalid quest entry '<key>'` per dropped key (diff of the `quests` keys before/after the step). `flags` unchanged (already valid in v1).
- `SaveMigrations.reset()` restores the built-in table (tests that register extra steps still clear only their own); `register` may still override a step in tests.
- A v1 envelope with no `quests` key migrates to `quests: {}`.

## Rules (logic lane)

Statuses as computed by `QuestRunner.status(gs, ref)` (the ref resolves from `gs.world_id`):

| Status | Condition |
| --- | --- |
| invalid | ref does not resolve, or the quest has no template (lore-only) |
| completed / ready / active | the stored `state` |
| available | not stored, every `requires` quest is `completed`, every `requires_flags` condition holds |
| locked | not stored, otherwise |

Condition rule (`QuestRules.condition_met(gs, cond)`): `gs.get_flag(key, false) == value` with strict type equality (`1` does not match `true`; a missing flag reads as `false`). Equality only in M4.

Runtime API (`QuestRunner`, static, `RefCounted`-style; every mutator returns problems or results and never throws):

| Function | Behaviour |
| --- | --- |
| `status(gs, ref) -> String` | table above |
| `quests_with(gs, status) -> Array[String]` | qids of the world's merged quests (`WorldCatalog.merged_entries(world, "quests")`) in that status, catalog order |
| `current_objective(gs, ref) -> Dictionary` | `{index, kind, text, target}` (target = bound location/encounter qid or flag key); {} unless active |
| `start(gs, ref) -> Array[String]` | only from `available`: stores `{state: active, objective: 0}` then `refresh` |
| `abandon(gs, ref) -> Array[String]` | only from `active` or `ready`: erases the key (back to available or locked); rewards were never granted |
| `on_location_reached(gs, location_ref) -> Array[String]` | for each active quest whose current objective is `reach_location` bound to the same resolved location qid: advance one objective; then `refresh`; returns advanced qids |
| `on_encounter_won(gs, encounter_ref) -> Array[String]` | same for `win_encounter` and the resolved encounter qid; a loss is simply not reported |
| `set_flag(gs, key, value) -> String` | `gs.set_flag` then `refresh`; error text as `GameState.set_flag` |
| `refresh(gs) -> Array[String]` | repeat until nothing changes: advance every active quest whose current objective is a met `flag` condition; mark a quest whose index reached the objective count `ready`; start every `auto_start` quest that is `available`. Returns qids that changed. Bounded: at most `quest count x (MAX_OBJECTIVES + 1)` passes |
| `complete(gs, ref, recipient_id := "") -> Dictionary` | only from `ready`: `RewardGrant.apply`, then `complete_flags` in order, then state `completed`, then `refresh`. Returns `{ok, errors, granted, notes}` |
| `check(gs) -> Array[String]` | warnings for unknown qids and indexes beyond the template; no changes |

Advancing: one event advances at most one objective per quest; flag objectives chain inside `refresh`. Location and encounter events give no retroactive credit (an objective that is not current ignores the event). Several active quests may advance on one event.

`RewardGrant.apply(gs, rewards, recipient_id) -> Dictionary` (`{ok, errors, granted, notes}`; validates everything first, changes nothing on error):

- `xp`: `LevelUp.award(member, xp)` for every active member (`party.active_members()`); level-ups follow the M3 rules (auto spend on a copy; the confirmed spend is untouched).
- `items`: `party.add_to_stash(id, count)` per entry.
- `passives`, `skills`: need a recipient. `recipient_id` empty = first active member; a non-empty id must be a roster member (`party.member(id) != null`), else error and nothing is granted. Passive: appended to `UnitState.passives` if absent, else note `already owned`. Skill: appended to `UnitState.learned_skills` unless the family is in `owned_families()`, else note `already owned`. Unlock and loadout rules are the existing ones (a learned family does not enter the loadout by itself).
- `granted` echoes what was applied: `{xp, items, passives, skills, recipient}`.

Rewards are granted once: a second `complete` returns `ok: false` (`quest is not ready`).

### Acceptance checks (rule-derived, `quest_test_suite.gd`; fixtures via a test path for `QuestCatalog` and a temp world root, never shipped data)

1. Template validation: each bad case yields an error (empty objectives, 9 objectives, unknown kind, extra objective key, unknown reward key, xp 2001, xp 1.5, item count 0 and 100, 9 item keys, unknown item/passive/skill, basic skill `strike`, duplicate passive); a valid template has none.
2. Soft warnings: 2 passives, 2 skills, xp 301, items summing to 7 each warn; `validate_all()` stays `{}` for them.
3. World quest: template set + step count mismatch = error; `win_encounter` step without `encounter` = error; `flag` step with key `Bad Key` = error; `auto_start` without template = error; a `requires` cycle A -> B -> A = error; a lore-only quest (old shape) stays valid.
4. Status: no `requires` + no flags = available; unmet `requires_flags` = locked, flips to available after `set_flag`; lore-only = invalid.
5. Flow `[reach_location, win_encounter]`: `on_encounter_won` first does nothing (no retroactive credit); location then encounter -> `ready`; `complete` grants once; the second call fails.
6. Flag objective met before start: after `start`, `refresh` advances it at once; a chain of two met flag objectives advances twice in one refresh.
7. Strict equality: flag `1` does not satisfy value `true`.
8. Rewards: every active member's `total_xp` rises by `xp` exactly; stash counts rise by the item counts; passive and skill land on the recipient only; empty recipient = first active member; unknown recipient = error with state, stash and XP unchanged; an already owned passive/skill is a note, not a duplicate.
9. `complete_flags` are set in order (a later entry with the same key wins) and can make another quest available in the same call.
10. Abandon from active erases the key; `start` again begins at objective 0.
11. `auto_start`: an available auto quest is active after `refresh`; a locked one is not.
12. Inheritance: from a child world, an inherited quest's key is the parent `_qid`; an override in the child gets the child's qid.
13. Save: a state with every stored status round-trips (`SaveGame.write`/`read`, `equals`); a blob entry with `state: "done"` or an extra key makes `validate` fail and `write` refuse; a v1 envelope with one good and one bad entry migrates to v2 keeping the good one with exactly one warning; `reset()` keeps the built-in step.
14. `check` warns for an unknown qid and leaves the state equal.

Update `save_test_suite.gd`: its opaque `quests` fixture (`{"q1": {"stage": 2, ...}}`) becomes a valid structured blob; the JSON-safety case becomes the shape case of check 13.

## Slices (one branch)

- Commit 1 (schema, releases M4-A): `QuestRules` (constants, `validate_blob`, `condition_met`), `QuestCatalog` (load, `get_entry`, `ids`, `upsert(entry, replace_id := "")`, `remove`, `validate_entry`, `validate_all`, `warnings_for`, `warnings_all`, `save`, `use_path` for tests; save-blocked when the file loaded with errors), empty `data/quests/quests.json`, world quest fields and validation, checks 1-3.
- Commit 2 (runtime): `QuestRunner`, `RewardGrant`, `GameState.validate` hook, `SaveGame.VERSION` 2 and the migration, `save_test_suite` update, checks 4-14, `scripts/quests/README.md` and `data/quests/README.md` updated (placeholder text replaced).

## Out of scope

Quest UI (journal, turn-in screen, World Forge preview of templates) and a Forge tab for templates (seeds only); dialogue/NPC turn-in (S11); exploration or travel calling the runtime (S9/S10); `have_item`, kill-count and timed objectives; failure states and in-quest branches (question 4); gold; world named items as rewards; `origin` on templates; AssetScan/safe-delete counting quest template references (until then deleting a referenced passive/item/skill makes `QuestCatalog.validate_all` report it); temporary passives for scripted events (classes-units "Passives lifetime"); balance tuning of reward numbers.
