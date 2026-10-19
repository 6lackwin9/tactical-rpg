# REP1-L: Faction reputation, condition operators and price bands (S11-B)

Status: approved (user 2026-10-17) (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)")
Depends on: DLG1-L merged (dialogue effects, `ShopRules`); after TRAV1-L in the queue (shared `world_schema.gd` / `world_catalog.gd`)
Lane: logic (one branch; commit 1 is the schema slice that releases REP1-A)
Touches: `scripts/quests/quest_rules.gd` (operators in `condition_errors` / `condition_met`, rep value rule, `reputation` validation), `scripts/quests/quest_runner.gd` (`complete` applies `reputation`), `scripts/world/world_schema.gd` (`_quest_fields` `reputation`), `scripts/world/world_catalog.gd` (`_json_refs_to` quest part), new `scripts/dialogue/rep_rules.gd`, `scripts/dialogue/{dialogue_rules,dialogue_runner,shop_rules}.gd` (`add_flag`, multipliers, hostile closes), new `scripts/tests/reputation_test_suite.gd` + one line in `suite_registry.gd`, `scripts/ui/overworld/{dialogue_panel,shop_panel}.gd` (band display). No save version bump: reputation lives in the flat flag registry and no new `GameState` key is added.

Rules source: `games/emberwake/docs/specs/S11_RULES.md` 2.6 (Q5 A: int flags and operators; Q7 A: no ripple).

## Goal

Reputation per faction as int flags `rep.<faction qid dotted>` in -100..100, changed by dialogue `add_flag` effects and a world quest field `reputation` applied on completion; comparison operators in the ONE shared condition grammar (so quests, variants, loot, markers, routes, gone flags and dialogue all gain rep gating); five display bands; shop price factors and a hostile closure by the keeper's faction.

## 1. Schema contract (schema-first: yes)

### 1.1 Condition operator (`QuestRules.condition_errors` / `condition_met`, used by every condition list)

Condition keys become `key`, `value`, `op` (still nothing else).

| Key | Allowed | Default | Validation (message) |
| --- | --- | --- | --- |
| op | `==`, `!=`, `>=`, `<=`, `>`, `<` (`QuestRules.OPS`) | `==` | unknown = error ("%s %d: op '%s' must be one of == != >= <= > <."); `>=` `<=` `>` `<` need a whole-number `value` ("%s %d: op '%s' needs a whole-number value.") |

Dialogue `flag` conditions accept `op` the same way (they delegate to `QuestRules`).

### 1.2 Reputation keys and values (`RepRules`, `scripts/dialogue/rep_rules.gd`)

- `RepRules.PREFIX` = `rep.`; `RepRules.key(faction_qid)` = `rep.` + qid with `:` as `.` (`rep.ashlands.ember_wardens`). `RepRules.MIN` -100, `MAX` 100.
- Rep value rule `RepRules.value_error(key, value) -> String`: for keys starting `rep.`, the value must be an int in -100..100 ("Reputation '%s' must be a whole number from -100 to 100."). Applied to: quest `complete_flags`, dialogue `set_flag`, marker and scripted `sets_flags`, and to condition values compared with an ordering op on a `rep.` key (a bound outside -100..100 is an error, it could never be decided otherwise).
- A `rep.` key must name a faction resolvable from the entry's world (warning, not error, so a renamed faction reports instead of blocking: "Reputation key '%s' names no faction of this world.").

### 1.3 Dialogue effect `add_flag`

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| key | String | required | `flag_key_ok` |
| delta | int | required | -100..100, not 0 |
| min | int or absent | absent | when both present, `min <= max` |
| max | int or absent | absent | |

Allowed in node and choice effects (it cannot fail on a valid state, see 2.2).

### 1.4 World quest field `reputation`

| Field | Type (schema) | Default | Validation (`QuestRules.world_quest_errors`) |
| --- | --- | --- | --- |
| reputation | json | {} | object faction ref -> int delta -50..50, not 0; max 4 keys; every ref resolves from the quest's world ("Reputation: unknown faction '%s'."); needs a template when non-empty ("Reputation needs a template.") |

Not in `QuestRules.VARIANT_FIELDS` (variants cannot change it; see Open questions). `_json_refs_to` lists the faction keys so a faction rename rewrites them.

## 2. Rules (logic lane)

### 2.1 Condition evaluation (`QuestRules.condition_met`)

- `==` (default) and `!=`: the M4 rule unchanged (strict type equality, a missing flag reads `false`); `!=` is its negation.
- Ordering ops: the flag value must be an int (a missing flag reads 0); a non-int flag makes the condition false (never an error at runtime).
- This is the only evaluation path, so `requires_flags`, `when_flags` / `unless_flags`, loot `requires_flags`, markers, routes, scripted events, `gone_flags`, shops and dialogue all inherit it unchanged.

### 2.2 Changing reputation

- `add_flag`: current = the flag (missing = 0); a non-int existing flag refuses the choice (all-or-nothing pre-check, "Flag '%s' is not a number."); new = clamp(current + delta, `min`, `max`) when given; keys with the `rep.` prefix are always clamped to -100..100 as well; set via `QuestRunner.set_flag`.
- Quest `reputation`: `QuestRunner.complete` applies each delta after the rewards, the loot claim and `complete_flags`, in key order, as an `add_flag` on `RepRules.key(resolved faction qid)` with the rep clamp; then `refresh` runs once. Completing again is refused (M4), so a delta applies once. The applied deltas are echoed in `granted.reputation`.
- No ripple through faction `relations` (Q7 A).

### 2.3 Bands (`RepRules.BANDS`, display and prices)

| Band | Range | Shop | Buy factor `rb` | Sell factor `rs` |
| --- | --- | --- | --- | --- |
| hostile | <= -50 | closed | - | - |
| unfriendly | -49..-10 | open | 115 | 85 |
| neutral | -9..9 | open | 100 | 100 |
| friendly | 10..49 | open | 95 | 105 |
| honoured | >= 50 | open | 90 | 110 |

`RepRules.standing(gs, faction_ref) -> int` (flag or 0), `band(value) -> String`, `buy_pct(band)`, `sell_pct(band)`. Shops use the keeper character's faction (resolved qid); a keeper without a faction is neutral. `ShopRules` replaces `rb` / `rs` = 100 from DLG1 with these factors in the same integer formulas; fixed stock `price` and loot prices ignore them; a hostile keeper's shop is closed (no offers, no buying, no selling, its service choice is not generated). Numbers are starting values (tuned after the balance pass).

### 2.4 Display

The dialogue and shop panels show the keeper faction's band name; a choice gated on a `rep.` condition with an ordering op gets the generated prefix `[<Faction name> <band of the bound>]`, for example `[Ember Wardens friendly]` for `>= 10` (same mechanism as the DLG1 stat prefix).

## 3. Acceptance checks (`reputation_test_suite.gd`; forked world root, never shipped data)

Checks 1-4 ship with commit 1, 5-13 with commit 2.

1. Operators: op `=~`, `>=` with value `true`, `<` with "10", extra key: errors; `!=` with a String is valid; existing conditions without `op` validate unchanged (assertion counts of `quest_test_suite` unchanged).
2. Rep values: `complete_flags` setting `rep.x` to 101 or `"high"`, a dialogue `set_flag` to -101, a condition `rep.x >= 150`: errors; a `rep.` key naming no faction: warning.
3. `add_flag`: delta 0 and 101, min 5 > max 1, unknown key: errors.
4. Quest `reputation`: unknown faction, delta 0 and 51, 5 keys, non-empty without a template: errors; renaming the faction rewrites the key.
5. Evaluation: missing flag with `>= 0` holds and with `> 0` fails; String flag with `>=` fails; `!=` mirrors `==` for every type; `1 == true` still fails.
6. Shared grammar: a quest with `requires_flags [{rep.f >= 10}]` is locked at 9 and available at 10; a variant with `when_flags [{rep.f <= -10}]` applies at -10; a marker with `unless_flags [{rep.f < 0}]` disappears at -1.
7. `add_flag`: from missing to +15; clamps at `max`; `rep.` keys clamp at 100 and -100 without `min`/`max`; a non-int flag refuses the whole choice and changes nothing.
8. Quest completion: deltas apply once after `complete_flags`; a quest available only at rep 20 becomes available in the same `complete` call that raises rep to 20; a second `complete` changes nothing.
9. Bands: -50 hostile, -49 unfriendly, -10 unfriendly, -9 neutral, 9 neutral, 10 friendly, 49 friendly, 50 honoured.
10. Prices: value 40 item at `buy_mult 1.0` costs 46 / 40 / 38 / 36 for unfriendly / neutral / friendly / honoured; sell at 0.5 pays 17 / 20 / 21 / 22; fixed price unchanged.
11. Hostile: the keeper's shop is closed, its service is not generated, `buy` is refused; at -49 it reopens.
12. No ripple: +30 to faction A leaves its `relations` allies and enemies at 0.
13. Saves: rep flags round-trip through `SaveGame` with no version change; a v5 save loads unchanged.

## 4. Commit slices (one branch)

1. Schema (releases REP1-A): `QuestRules.OPS` and operator validation, rep value rule and warning, `add_flag` validation, quest `reputation` field + validation + ref scan, `RepRules` constants and band functions, checks 1-4.
2. Runtime: operator evaluation, `add_flag`, `complete` applying `reputation`, shop factors and hostile closure, panel band display and rep prefix, checks 5-13, `scripts/quests/README.md` and `scripts/dialogue/README.md` updated.

## 5. Proposed STRUCTURE.md rows (planner adds)

- `scripts/dialogue/rep_rules.gd` (S11, REP1) inside the existing `scripts/dialogue/` row; `scripts/tests/reputation_test_suite.gd` (S11).

## Out of scope

Ripple through relations (a later rule on top of Q7 A), faction membership passives, reputation decay over `time`, kill-based reputation, variants overriding `reputation`, a reputation journal screen (UI3), shipped data (REP1-A).

## Open questions

- R1. Should quest variants be able to override `reputation` (for example a different delta when the giver changed after an NPC death)? R: not in REP1 (keeps the field a fixed consequence of the quest); add `reputation` to `VARIANT_FIELDS` later if the story pass needs it.
