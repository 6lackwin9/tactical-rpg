# M4-L: Quest and flag runtime (S8), structured quest schema

Status update (2026-10-20): approved and built; the rows are done (docs/archive/TASKS_DONE.md). The status line below is the original.

Status: proposed (design answered: DECISIONS world-campaign "M4 quest answers" and "M4 variant answers + world loot rewards" (user 2026-10-17); one open question L1 below; approval by the user)
Depends on: M3 (done: `GameState.flags`, opaque `GameState.quests`, `SaveGame` v1, `SaveMigrations`)
Lane: logic (one branch; commit 1 is the schema slice that releases M4-A)
Touches: new `scripts/quests/{quest_catalog,quest_rules,quest_runner,reward_grant}.gd` (+ its `README.md`), new `data/quests/quests.json` (empty envelope only), `scripts/world/world_schema.gd` (`_quest_fields`, new kind `loot` in `KINDS` + `_loot_fields`), `scripts/world/world_catalog.gd` (`_quest_errors`, new `_loot_errors`, `_json_refs_to` quest part), `scripts/tests/world_test_suite.gd` (its fixed kind list gains `loot`), `scripts/game_state/{game_state,save_game,save_migrations}.gd`, new `scripts/tests/quest_test_suite.gd` + one line in `suite_registry.gd`, `scripts/tests/save_test_suite.gd` (the opaque-blob fixture). Reads `ContentCatalog`, `PartyState`, `UnitState`, `LevelUp` (no edits). No overlap with the cleanup batch (CLN-9 edits `content_catalog.gd`, which M4 does not touch).

## Answered (user 2026-10-17)

All recommendations of the two question rounds are accepted (DECISIONS rows cited in Status); the loot table is the user's addition. Ordered merge was chosen because independent consequences compose (two flags = two variants, not a combined third) and every overridable field validates alone, so any subset of variants is valid. HISTORY check: nothing revives a deprecated idea; passives stay source-granted (a loot entry is a source).

## Open question for the user

- L1 May the player take FEWER picks than offered (keep a unique for a later quest)? R: yes, `complete` accepts 0..`loot_picks` picks; untaken entries stay in other pools, and the generic rewards are granted either way. This supports "pick quests by what they offer". Alternative: exactly `min(loot_picks, offers)` picks (no deferring).

## Goal

Give quests a runtime: lore-free quest templates (ordered objectives + rewards), world quests that bind them to world targets and carry linking metadata, flag-driven variants resolved at read time, a shared per-world loot table that quests offer as a pick-n choice on top of their generic rewards (claim-once per save), a state machine over `GameState` (start, advance on events and flags, ready, complete with rewards, abandon), reward granting as a passive/skill/item SOURCE, and a structured, validated, migrated quest blob in the save.

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
| loot_pool | `json` | [] | Array of world loot refs (section "World loot table"), each resolves, no duplicates, max `QuestRules.MAX_POOL` 8; needs a template |
| loot_picks | `int` | 1 | 0..`QuestRules.MAX_PICKS` (3); must be <= `loot_pool.size()` when the pool is non-empty; ignored when empty |

Step bindings (the existing `steps` json, one step per template objective when `template` is set):

| Objective kind | Step keys used | Rule |
| --- | --- | --- |
| reach_location | `location` (required) | resolves as a world location from the quest's world (qualified refs allowed, as today) |
| win_encounter | `encounter` (required), `location` (optional, display) | resolves as a world encounter |
| flag | `flag` (required), `value` (optional, default true) | `flag` passes `flag_key_ok`; `value` bool/int/String |

Every step keeps the required `text`. With a template: `steps.size()` must equal `objectives.size()`; a missing required binding is an error naming the step and kind; keys not used by the step's kind are ignored. Without a template the old rules apply unchanged. `requires` may not form a cycle (resolved from the quest's world) = error.

### Links (metadata; no gameplay effect, gating stays in `requires` / `requires_flags`)

Keys (each an Array[String] of refs, default [], every ref must resolve): `follows`, `leads_to`, `related_quests` (quest refs; `follows`/`leads_to` may not name the quest itself), `related_characters`, `related_locations`, `related_factions`. Unknown keys = error. Soft warning (`WorldCatalog`-side helper `QuestRules.link_warnings(world_id)`, used by tests and the Forge later): A `leads_to` B without B `follows` A (or the reverse). `_json_refs_to` lists every ref in `links` and in variant `set` (giver, turn_in, step location/encounter, links) so rename and delete scans stay correct.

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
| loot_pool | loot refs | replaces the pool (same validation) |
| loot_picks | int 0..3 | replaces the pick count (no cross-check with the pool: the runtime caps picks at the number of offers) |

Not overridable (so no override can branch a quest or change its shape): `id`, `name`, `template`, objective kinds and count, `requires`, `requires_flags`, `complete_flags`, `auto_start`, `faction`, `region`, `variants`. Any other `set` key = error.

### World loot table: world kind `loot` (`data/worlds/<id>/loot.json`)

A new world kind in `WorldSchema.KINDS`, inherited and overridable like every world kind (`merged_entries`, `override`, qualified refs). A missing `loot.json` loads as an empty kind (current `_read_kind` behaviour). Every entry is claim-once per save.

| Field | Type (schema) | Default | Validation (`world_catalog._loot_errors`) |
| --- | --- | --- | --- |
| id | `id` | required | unique in the world |
| name | `string` | "New Loot" | non-empty (lore name shown in the offer) |
| description | `text` | "" | free text |
| reward_kind | `enum` item, passive, skill | item | |
| ref | `string` | required | item: existing `ContentCatalog` item; passive: existing passive; skill: existing family head, not a basic (same rules as template rewards, so every pooled entry can be granted) |
| count | `int` | 1 | 1..99 for item; must be 1 for passive/skill |
| rarity | `enum` common, rare, unique | rare | display and sorting only |
| requires_flags | `json` | [] | conditions; an entry whose conditions fail is not offered (it stays in the pool) |
| tags | `tags` | [] | strings |

Soft reward budget (`QuestRules.reward_warnings(world_id) -> Array[String]`, warnings only): per quest, with all variants applied (worst case), warn when the generic rewards plus ONE pooled special of a kind exceed that kind's cap (passives, skills, item counts), and when `loot_picks > SOFT_MAX_PICKS` (1).

### Quest state blob: `GameState.quests`

Key = the resolved quest entry's qualified id `_qid` (`ashlands:hold_the_courtyard`), so an inherited quest is the same key in every descendant world and a world override gets its own key. A missing key = not started.

| Field | Type | Allowed | Notes |
| --- | --- | --- | --- |
| state | String | `active`, `ready`, `completed` | `QuestRules.STORED_STATES` |
| objective | int | 0..`MAX_OBJECTIVES` | index of the current objective; equals the objective count when `ready` or `completed` |

Claimed loot: new `GameState` key `loot_claimed` (added to `GameState.KEYS`, so the migration's unknown-key drop keeps it): Array[String] of qualified loot ids (`ashlands:ember_charm`), unique, each passing the qid format; saved sorted ascending for byte-stable saves. `GameState.validate` checks the shape; unknown ids are a `QuestRunner.check` warning.

No other keys: variants and offers are NOT saved (they re-resolve from flags on every read). `QuestRules.validate_blob(blob) -> Array[String]` checks: key contains exactly one `:` and both parts pass `id_ok`; value is a Dictionary with exactly these two keys and allowed values. `GameState.validate` calls it (replacing the opaque `JsonUtil.check_safe(quests, ...)`), so `SaveGame.write` refuses a bad blob. Catalog facts (unknown quest, index above the template's count) are NOT save errors: `QuestRunner.check(gs)` reports them as warnings and changes nothing.

### Save version 2

- `SaveGame.VERSION` = 2.
- `SaveMigrations` gets a built-in step 1 -> 2: keep each `quests` entry that passes `validate_blob` for that single entry, drop the others; `migrate` appends one warning `dropped invalid quest entry '<key>'` per dropped key (diff of the `quests` keys before/after the step). `flags` unchanged.
- `SaveMigrations.reset()` restores the built-in table (tests that register extra steps still clear only their own); `register` may still override a step in tests.
- A v1 envelope with no `quests` key migrates to `quests: {}`; every v1 envelope gains `loot_claimed: []`.

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
| `offers(gs, ref) -> Array[String]` | loot qids the quest offers now: resolved `loot_pool` minus `gs.loot_claimed` minus entries whose `requires_flags` fail, in pool order; same rule in any status (UI preview), only `complete` claims |
| `picks_allowed(gs, ref) -> int` | `mini(loot_picks, offers.size())` |
| `complete(gs, ref, recipient_id := "", picks: Array[String] = []) -> Dictionary` | only from `ready` (a `blocked` quest refuses). `picks`: unique, each in `offers`, size 0..`picks_allowed` (L1). Grants the RESOLVED generic rewards (always, even with an empty pool or 0 picks) plus each picked loot entry via `RewardGrant.apply`, appends the picks to `loot_claimed`, sets `complete_flags` in order, stores `completed`, then `refresh`. All-or-nothing: any error changes nothing. Returns `{ok, errors, granted, notes}` |
| `check(gs) -> Array[String]` | warnings for unknown quest qids, indexes beyond the template, unknown claimed loot ids; no changes |

Claim rule: a claimed loot id leaves every pool of every quest at once (offers subtract `loot_claimed`); unclaimed entries stay offered by every quest that pools them. Offers are computed on every read from flags + `loot_claimed`, so they are deterministic and never saved.

Advancing: one event advances at most one objective per quest; flag objectives chain inside `refresh`. No retroactive credit for location/encounter events. A variant that rebinds the CURRENT objective changes the target the next event must match; completed objectives stay completed (V2).

`RewardGrant.apply(gs, rewards, recipient_id) -> Dictionary` (`{ok, errors, granted, notes}`; validates first, changes nothing on error):

- `xp`: `LevelUp.award(member, xp)` for every active member; level-ups follow the M3 rules.
- `items`: `party.add_to_stash(id, count)` per entry.
- `passives`, `skills`: one recipient; `recipient_id` empty = first active member; a non-empty id must be a roster member, else error and nothing is granted. Passive: appended to `UnitState.passives` if absent, else note `already owned`. Skill: appended to `UnitState.learned_skills` unless the family is in `owned_families()`, else note `already owned`.
- Loot entries are folded into the same lists before validation (item -> `items` with its count; passive -> `passives`; skill -> `skills`), so the recipient and "already owned" rules apply to them too; an already owned special is still claimed (the player chose it).
- `granted` echoes `{xp, items, passives, skills, loot, recipient}`.

Rewards are granted once: a second `complete` returns `ok: false` (`quest is not ready`).

### Acceptance checks (rule-derived, `quest_test_suite.gd`; fixtures via a test path for `QuestCatalog` and a temp world root, never shipped data)

Checks 1-4 and 16 ship with commit 1 (schema), checks 5-15 and 17 with commit 2 (runtime).

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
15. Save: every stored status round-trips; a blob entry with `state: "done"` or an extra key fails `validate` and `write`; a v1 envelope with one good and one bad entry migrates keeping the good one with one warning; `reset()` keeps the built-in step; `check` warns for an unknown qid and leaves the state equal; `loot_claimed` round-trips sorted, a duplicate or malformed id fails `validate`, a v1 envelope gains `loot_claimed: []`.
16. Loot schema: unknown `ref` per reward kind, basic skill, count 2 on a passive, count 0 on an item, bad rarity: errors; a child world inherits and can override a parent loot entry; `loot_pool` with an unknown or duplicate ref, 9 refs, `loot_picks` 4 or above a non-empty pool size, a pool without template: errors; variant `set` with `loot_pool` / `loot_picks` validates like the base field; `reward_warnings`: generic 1 passive + a passive entry in the pool warns, generic 0 passives + one passive entry does not, `loot_picks` 2 warns.
17. Loot runtime: two quests pooling entry E both offer it; completing the first with pick E grants generic + E and the second no longer offers E (other entries stay); 0 picks grants only the generic rewards and claims nothing; a pick not in `offers`, a duplicate pick or more than `picks_allowed` = error with nothing changed; an entry with failing `requires_flags` is not offered and appears once the flag is set; a variant replacing `loot_pool` changes the offers; empty pool = generic rewards only; same state twice = equal offers.

Update `save_test_suite.gd`: its opaque `quests` fixture becomes a valid structured blob; its JSON-safety case becomes the shape case of check 15.

## Slices (one branch)

- Commit 1 (schema, releases M4-A): `QuestRules` (constants incl. `VARIANT_FIELDS`, `MAX_VARIANTS`, `validate_blob`, `condition_met`, variant and links validation helpers), `QuestCatalog` (load, `get_entry`, `ids`, `upsert(entry, replace_id := "")`, `remove`, `validate_entry`, `validate_all`, `warnings_for`, `warnings_all`, `save`, `use_path`; save-blocked when the file loaded with errors), empty `data/quests/quests.json`, world quest fields (incl. `turn_in`, `links`, `variants`, `loot_pool`, `loot_picks`) with validation and ref scans (loot refs included), world kind `loot` with `_loot_errors`, `reward_warnings`, checks 1-4 and 16.
- Commit 2 (runtime): `QuestRules.resolve`, `QuestRunner` (incl. `offers`, `picks_allowed`), `RewardGrant`, `GameState.loot_claimed` + `validate` hook, `SaveGame.VERSION` 2 and the migration, `save_test_suite` update, checks 5-15 and 17, `scripts/quests/README.md` and `data/quests/README.md` updated.

## Out of scope

Quest UI and a Forge tab for templates (seeds only); dialogue/NPC turn-in enforcement (S11, V4); character-level death flags (V5); exploration or travel calling the runtime (S9/S10); `have_item`, kill-count and timed objectives; failure states and in-quest branches; gold; world named items as loot payloads (the stash holds base item ids only; a later slice); loot from sources other than quests (shops, chests: they would reuse `loot_claimed`); `origin` on templates; AssetScan/safe-delete counting quest and loot references (validation reports a deleted ref); temporary passives for scripted events; balance tuning of reward numbers.
