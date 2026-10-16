# M4-L: Quest and flag runtime (S8), structured quest schema

Status: proposed (the 8 base answers are in, DECISIONS world-campaign "M4 quest answers (user 2026-10-17)"; the metadata/variant questions V1-V5 below are open; approval by the user)
Depends on: M3 (done: `GameState.flags`, opaque `GameState.quests`, `SaveGame` v1, `SaveMigrations`)
Lane: logic (one branch; commit 1 is the schema slice that releases M4-A)
Touches: new `scripts/quests/{quest_catalog,quest_rules,quest_runner,reward_grant}.gd` (+ its `README.md`), new `data/quests/quests.json` (empty envelope only), `scripts/world/world_schema.gd` (`_quest_fields`), `scripts/world/world_catalog.gd` (`_quest_errors`, `_json_refs_to` quest part), `scripts/game_state/{game_state,save_game,save_migrations}.gd`, new `scripts/tests/quest_test_suite.gd` + one line in `suite_registry.gd`, `scripts/tests/save_test_suite.gd` (the opaque-blob fixture). Reads `ContentCatalog`, `PartyState`, `UnitState`, `LevelUp` (no edits). No overlap with the cleanup batch (CLN-9 edits `content_catalog.gd`, which M4 does not touch).

## Answered (user 2026-10-17)

All eight recommendations accepted: (1) lore-free templates in `data/quests/` + world binding; (2) one flat flag registry per save; (3) xp to every active member, items to the stash, passive/skill to one chosen recipient, soft reward caps (1 passive, 1 skill, xp 300, 6 items); (4) no failure, no in-quest branching, campaign linkage through flags; (5) explicit game-loop objective calls, no retroactive credit; (6) `ready` + explicit `complete` (turn-in); (7) save v2 with a 1 -> 2 migration; (8) a separate `QuestCatalog`. Clarification: quests carry linking METADATA that FLAGS can alter (an NPC dies: other giver, other target, other reward, or the quest is gone). HISTORY check: nothing revives a deprecated idea; the event-bus quest option (archived P-S8 C) stays not adopted; passives stay source-granted.

## Open questions for the user (metadata and variants; the spec follows R)

- V1 Variant resolution. R: ORDERED MERGE: every matching variant applies in list order, a later one replaces the same field. Independent consequences compose (Mara dead AND keep burned = two variants, not a third combined one), and since every overridable field validates on its own, any subset of variants is valid without enumerating combinations. First-match needs one variant per flag combination (2^n authoring) and silently hides later ones.
- V2 Active quest when its variant flips. R: bindings, giver, turn-in, description and rewards re-resolve on every read (objective index and completed objectives stay); `available: false` makes an active/ready quest `blocked` (computed, not saved: events do not advance it, `complete` refuses, `abandon` works) and it un-blocks if the flags flip back. Simpler alternative: `available` only affects not-started quests (an active quest always finishes). Auto-abandon is rejected (loses progress on a flag the player may not see).
- V3 Reward change. R: a variant names another TEMPLATE (`rewards_from`) whose rewards replace the quest's; numbers stay in the base layer. Alternative: an inline `rewards` dict in world data (gameplay numbers in the lore layer, against the layer rule).
- V4 Enforce giver/turn-in in M4? R: no, metadata only (S11 dialogue reads the resolved `giver`/`turn_in`); `complete` stays a plain call.
- V5 NPC death as a character field (`gone_when` on characters that auto-blocks every quest naming them)? R: not now; authors write the variant per quest (explicit, testable); revisit with S11.

## Goal

Give quests a runtime: lore-free quest templates (ordered objectives + rewards), world quests that bind them to world targets and carry linking metadata, flag-driven variants resolved at read time, a state machine over `GameState` (start, advance on events and flags, ready, complete with rewards, abandon), reward granting as a passive/skill/item SOURCE, and a structured, validated, migrated quest blob in the save.

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
| turn_in | `ref` characters, optional | "" | "" = same as `giver`; else resolves as a world character |
| requires_flags | `json` | [] | Array of conditions `{"key": K, "value": V}`; K passes `GameState.flag_key_ok`; V bool/int/String, default true when missing |
| complete_flags | `json` | [] | Array of `{"key": K, "value": V}`, same rules; set in order on completion |
| auto_start | `bool` | false | true needs a non-empty `template` |
| links | `json` | {} | metadata block below |
| variants | `json` | [] | flag-driven overrides below; needs a non-empty `template` when non-empty |

Step bindings (the existing `steps` json, one step per template objective when `template` is set):

| Objective kind | Step keys used | Rule |
| --- | --- | --- |
| reach_location | `location` (required) | resolves as a world location from the quest's world (qualified refs allowed, as today) |
| win_encounter | `encounter` (required), `location` (optional, display) | resolves as a world encounter |
| flag | `flag` (required), `value` (optional, default true) | `flag` passes `flag_key_ok`; `value` bool/int/String |

Every step keeps the required `text`. With a template: `steps.size()` must equal `objectives.size()`; a missing required binding is an error naming the step and kind; keys not used by the step's kind are ignored. Without a template the old rules apply unchanged. `requires` may not form a cycle (resolved from the quest's world) = error.

### Links (metadata; no gameplay effect, gating stays in `requires` / `requires_flags`)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| follows | Array[String] quest refs | [] | each resolves; not the quest itself |
| leads_to | Array[String] quest refs | [] | each resolves; not the quest itself |
| related_quests | Array[String] quest refs | [] | each resolves |
| related_characters | Array[String] character refs | [] | each resolves |
| related_locations | Array[String] location refs | [] | each resolves |
| related_factions | Array[String] faction refs | [] | each resolves |

Unknown keys = error. Soft warning (`WorldCatalog`-side helper `QuestRules.link_warnings(world_id)`, used by tests and the Forge later): A `leads_to` B without B `follows` A (or the reverse). `_json_refs_to` lists every ref in `links` and in variant `set` (giver, turn_in, step location/encounter, links) so rename and delete scans stay correct.

### Variants (flag-driven overrides, ordered merge, V1)

`variants` = Array (max `QuestRules.MAX_VARIANTS` 8) of:

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| id | String | required | `id_ok`, unique inside the quest (used in messages and tests) |
| when_flags | Array of conditions | [] | condition rules as `requires_flags`; ALL must hold |
| unless_flags | Array of conditions | [] | same; NONE may hold |
| set | Dictionary | required, non-empty | keys only from the whitelist below |

At least one of `when_flags` / `unless_flags` is non-empty (an unconditional variant is an error: edit the quest instead).

Whitelist of `set` keys (`QuestRules.VARIANT_FIELDS`); every value validates exactly like the base field, so any subset of variants yields a valid quest:

| set key | Value | Effect |
| --- | --- | --- |
| giver | character ref or "" | replaces `giver` |
| turn_in | character ref or "" | replaces `turn_in` |
| description | String | replaces `description` |
| reward | String | replaces the reward text |
| rewards_from | QuestCatalog id | the rewards of that template replace the quest template's rewards (V3); the soft caps apply to it as to any template |
| steps | Dictionary `"<index>"` -> partial step | per step: replaces only the given keys among `text` and the binding keys of THAT step's objective kind (`location`/`encounter`/`flag`/`value`); index 0..count-1; a required binding may be replaced, never emptied |
| links | links block | replaces the whole block |
| available | bool | false = the quest is unavailable (V2) |

Not overridable (so no override can branch a quest or change its shape): `id`, `name`, `template`, objective kinds and count, `requires`, `requires_flags`, `complete_flags`, `auto_start`, `faction`, `region`, `variants`. Any other `set` key = error.

### Quest state blob: `GameState.quests`

Key = the resolved quest entry's qualified id `_qid` (`ashlands:hold_the_courtyard`), so an inherited quest is the same key in every descendant world and a world override gets its own key. A missing key = not started.

| Field | Type | Allowed | Notes |
| --- | --- | --- | --- |
| state | String | `active`, `ready`, `completed` | `QuestRules.STORED_STATES` |
| objective | int | 0..`MAX_OBJECTIVES` | index of the current objective; equals the objective count when `ready` or `completed` |

No other keys: variants are NOT saved (they re-resolve from flags on every read). `QuestRules.validate_blob(blob) -> Array[String]` checks: key contains exactly one `:` and both parts pass `id_ok`; value is a Dictionary with exactly these two keys and allowed values. `GameState.validate` calls it (replacing the opaque `JsonUtil.check_safe(quests, ...)`), so `SaveGame.write` refuses a bad blob. Catalog facts (unknown quest, index above the template's count) are NOT save errors: `QuestRunner.check(gs)` reports them as warnings and changes nothing.

### Save version 2

- `SaveGame.VERSION` = 2.
- `SaveMigrations` gets a built-in step 1 -> 2: keep each `quests` entry that passes `validate_blob` for that single entry, drop the others; `migrate` appends one warning `dropped invalid quest entry '<key>'` per dropped key (diff of the `quests` keys before/after the step). `flags` unchanged.
- `SaveMigrations.reset()` restores the built-in table (tests that register extra steps still clear only their own); `register` may still override a step in tests.
- A v1 envelope with no `quests` key migrates to `quests: {}`.

## Rules (logic lane)

Resolution (`QuestRules.resolve(gs, entry) -> Dictionary`, pure, deterministic: input = the world entry + `gs.flags`, no RNG, no clock): start from the entry with the template's objectives and rewards attached; for each variant in list order whose `when_flags` all hold and `unless_flags` none hold, apply its `set` (field replace; `steps` per index and key; `rewards_from` swaps the rewards); add `available` (default true) and `applied_variants` (ids in order). Every runtime read below uses the resolved quest.

Statuses as computed by `QuestRunner.status(gs, ref)` (the ref resolves from `gs.world_id`):

| Status | Condition |
| --- | --- |
| invalid | ref does not resolve, or the quest has no template (lore-only) |
| completed | stored `completed` (variants no longer matter: rewards are given) |
| blocked | stored `active`/`ready` and resolved `available` is false |
| ready / active | the stored `state` |
| unavailable | not stored and resolved `available` is false |
| available | not stored, every `requires` quest is `completed`, every `requires_flags` condition holds |
| locked | not stored, otherwise |

Condition rule (`QuestRules.condition_met(gs, cond)`): `gs.get_flag(key, false) == value` with strict type equality (`1` does not match `true`; a missing flag reads as `false`). Equality only.

Runtime API (`QuestRunner`, static; every mutator returns problems or results and never throws):

| Function | Behaviour |
| --- | --- |
| `status(gs, ref) -> String` | table above |
| `resolved(gs, ref) -> Dictionary` | the resolved quest ({} if invalid); for UI and S11 (giver, turn_in, links, texts) |
| `quests_with(gs, status) -> Array[String]` | qids of the world's merged quests (`WorldCatalog.merged_entries(world, "quests")`) in that status, catalog order |
| `current_objective(gs, ref) -> Dictionary` | `{index, kind, text, target}` from the resolved steps (target = location/encounter qid or flag key); {} unless active |
| `start(gs, ref) -> Array[String]` | only from `available`: stores `{state: active, objective: 0}` then `refresh` |
| `abandon(gs, ref) -> Array[String]` | only from `active`, `ready` or `blocked`: erases the key; rewards were never granted |
| `on_location_reached(gs, location_ref) -> Array[String]` | for each `active` (not blocked) quest whose current objective is `reach_location` bound (after resolution) to the same location qid: advance one objective; then `refresh`; returns advanced qids |
| `on_encounter_won(gs, encounter_ref) -> Array[String]` | same for `win_encounter` and the encounter qid; a loss is simply not reported |
| `set_flag(gs, key, value) -> String` | `gs.set_flag` then `refresh`; error text as `GameState.set_flag` |
| `refresh(gs) -> Array[String]` | repeat until nothing changes: advance every `active` quest whose current objective is a met `flag` condition; mark a quest whose index reached the objective count `ready`; start every `auto_start` quest that is `available`. Returns qids that changed. Bounded: at most `quest count x (MAX_OBJECTIVES + 1)` passes (variants change no stored data, so they cannot loop) |
| `complete(gs, ref, recipient_id := "") -> Dictionary` | only from `ready` (a `blocked` quest refuses): `RewardGrant.apply` with the RESOLVED rewards, then `complete_flags` in order, then state `completed`, then `refresh`. Returns `{ok, errors, granted, notes}` |
| `check(gs) -> Array[String]` | warnings for unknown qids and indexes beyond the template; no changes |

Advancing: one event advances at most one objective per quest; flag objectives chain inside `refresh`. No retroactive credit for location/encounter events. A variant that rebinds the CURRENT objective changes the target the next event must match; completed objectives stay completed (V2).

`RewardGrant.apply(gs, rewards, recipient_id) -> Dictionary` (`{ok, errors, granted, notes}`; validates first, changes nothing on error):

- `xp`: `LevelUp.award(member, xp)` for every active member; level-ups follow the M3 rules.
- `items`: `party.add_to_stash(id, count)` per entry.
- `passives`, `skills`: one recipient; `recipient_id` empty = first active member; a non-empty id must be a roster member, else error and nothing is granted. Passive: appended to `UnitState.passives` if absent, else note `already owned`. Skill: appended to `UnitState.learned_skills` unless the family is in `owned_families()`, else note `already owned`.
- `granted` echoes `{xp, items, passives, skills, recipient}`.

Rewards are granted once: a second `complete` returns `ok: false` (`quest is not ready`).

### Acceptance checks (rule-derived, `quest_test_suite.gd`; fixtures via a test path for `QuestCatalog` and a temp world root, never shipped data)

Checks 1-4 ship with commit 1 (schema), checks 5-15 with commit 2 (runtime).

1. Template validation: each bad case yields an error (empty objectives, 9 objectives, unknown kind, extra objective key, unknown reward key, xp 2001, xp 1.5, item count 0 and 100, 9 item keys, unknown item/passive/skill, basic `strike`, duplicate passive); a valid template has none. Soft warnings: 2 passives, 2 skills, xp 301, 7 items each warn; `validate_all()` stays `{}`.
2. World quest: step count mismatch, `win_encounter` step without `encounter`, flag key `Bad Key`, `auto_start` without template, a `requires` cycle, `turn_in` to an unknown character: each an error; a lore-only quest (old shape) stays valid.
3. Links: unknown key, unresolved ref, `follows` itself: errors; one-sided `leads_to` is a warning only.
4. Variants: unconditional variant, duplicate variant id, `set` key `template` / `requires` / `complete_flags` / `objectives`, step index out of range, `encounter` set on a `reach_location` step, emptied required binding, unknown `rewards_from`, 9 variants, variants without template: each an error; a valid variant set has none.
5. Status: no `requires` + no flags = available; unmet `requires_flags` = locked, flips after `set_flag`; lore-only = invalid.
6. Flow `[reach_location, win_encounter]`: encounter first does nothing; location then encounter -> `ready`; `complete` grants once; the second call fails.
7. Flag objective met before start advances on `start`; two met flag objectives chain in one refresh; flag `1` does not satisfy `true`.
8. Rewards: each active member's `total_xp` rises by `xp`; stash counts rise; passive/skill on the recipient only; empty recipient = first active member; unknown recipient = error with nothing changed; already owned = note.
9. `complete_flags` set in order (later same key wins) and can make another quest available in the same call; abandon then `start` restarts at objective 0; an `auto_start` available quest is active after `refresh`, a locked one is not.
10. Variants resolve: no flags = base values; one matching variant changes `giver`; two matching variants setting the same field = the later one wins, different fields = both apply; `unless_flags` suppresses; `applied_variants` lists ids in order; same flags twice = equal results (deterministic).
11. Rebinding: an active quest whose current objective is rebound by a flag ignores the old target and advances on the new one; a completed earlier objective is not undone.
12. `available: false`: not started -> `unavailable` (cannot start); active -> `blocked` (events ignored, `complete` refuses, `abandon` works); clearing the flag -> active again with the same objective index; the saved blob is byte-equal before and after the flips.
13. `rewards_from`: `complete` under the variant grants the other template's rewards.
14. Inheritance: from a child world, an inherited quest's key is the parent `_qid`; an override in the child gets the child's qid.
15. Save: every stored status round-trips; a blob entry with `state: "done"` or an extra key fails `validate` and `write`; a v1 envelope with one good and one bad entry migrates keeping the good one with one warning; `reset()` keeps the built-in step; `check` warns for an unknown qid and leaves the state equal.

Update `save_test_suite.gd`: its opaque `quests` fixture becomes a valid structured blob; its JSON-safety case becomes the shape case of check 15.

## Slices (one branch)

- Commit 1 (schema, releases M4-A): `QuestRules` (constants incl. `VARIANT_FIELDS`, `MAX_VARIANTS`, `validate_blob`, `condition_met`, variant and links validation helpers), `QuestCatalog` (load, `get_entry`, `ids`, `upsert(entry, replace_id := "")`, `remove`, `validate_entry`, `validate_all`, `warnings_for`, `warnings_all`, `save`, `use_path`; save-blocked when the file loaded with errors), empty `data/quests/quests.json`, world quest fields (incl. `turn_in`, `links`, `variants`) with validation and ref scans, checks 1-4.
- Commit 2 (runtime): `QuestRules.resolve`, `QuestRunner`, `RewardGrant`, `GameState.validate` hook, `SaveGame.VERSION` 2 and the migration, `save_test_suite` update, checks 5-15, `scripts/quests/README.md` and `data/quests/README.md` updated.

## Out of scope

Quest UI and a Forge tab for templates (seeds only); dialogue/NPC turn-in enforcement (S11, V4); character-level death flags (V5); exploration or travel calling the runtime (S9/S10); `have_item`, kill-count and timed objectives; failure states and in-quest branches; gold; world named items as rewards; `origin` on templates; AssetScan/safe-delete counting quest template references (until then `QuestCatalog.validate_all` reports a deleted reference); temporary passives for scripted events; balance tuning of reward numbers.
