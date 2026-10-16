# UI3-L: journal (quests only) over the M4 runtime

Status: spec (design-brainstorm 2026-10-17; DECISIONS progression-save "UI menus baseline answers (user 2026-10-17)": journal = quests only for now; docs/specs/UI_MENUS_RULES.md 2.7; awaiting approval)
Depends on: UI1-L (router, `CurrentGame`, hub), M4-L (merged: `QuestRunner`, `QuestRules`, loot)
Lane: logic only (no asset part)
Touches: new `scripts/ui/menus/{journal_model,journal_view}.gd`, `scripts/ui/menus/hub_model.gd` (`open("journal")` wiring), new `scripts/tests/journal_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`

## Goal

A read-only journal that lists the current world's quests the player knows about, grouped by status, with the resolved (variant-applied) text, the current objective, rewards and the loot the quest still offers. All values come from `QuestRunner` reads on every refresh (M4 rule: variants, availability and offers re-resolve from flags and `loot_claimed` on every read; nothing is cached or saved).

## Schema contract (logic lane, schema first: no; no data or save change)

### `JournalModel`

| Member | Value |
| --- | --- |
| `groups() -> Array[Dictionary]` | `[{status, label, quests: [qid]}]` in the order below; empty groups omitted |
| `entry(qid) -> Dictionary` | `{qid, name, description, status, giver, turn_in, objective, objectives_done, objectives_total, rewards, offers, picks_allowed, links}` (2.1) |
| `selected: String` | qid shown in the detail pane ("" = first of the first group) |
| `refresh()` | re-reads everything from `CurrentGame.state` |

Groups (labels are the player-facing text):

| `QuestRunner.status` | Group label | Shown |
| --- | --- | --- |
| `ready` | Ready to turn in | always |
| `active` | Active | always |
| `blocked` | On hold | always (a started quest whose variant made it unavailable; M4 `blocked`) |
| `completed` | Completed | always |
| `available`, `locked`, `unavailable`, `invalid` | - | never (not known to the player; givers offer them through dialogue, DLG1) |

Within a group: the world's quest catalog order (`QuestRunner.quests_with`).

### 2.1 Entry fields

| Field | Source |
| --- | --- |
| `name`, `description` | `QuestRunner.resolved(gs, qid)` keys `name`, `description` (variant-applied) |
| `giver`, `turn_in` | resolved `giver` / `turn_in` character refs -> `WorldCatalog.resolve(gs.world_id, "characters", ref)["name"]`; "" when empty or unresolved |
| `objective` | `QuestRunner.current_objective(gs, qid)["text"]` (active only, else "") |
| `objectives_done`, `objectives_total` | stored objective index (`gs.quests[qid].objective`; `objectives_total` for completed) and the resolved objective count |
| `rewards` | the resolved generic rewards as display lines: XP, item names (catalog `name`), passive and skill names, gold when DLG1 is merged |
| `offers`, `picks_allowed` | `QuestRunner.offers(gs, qid)` mapped to loot display names; `QuestRunner.picks_allowed` (only for `active` and `ready`) |
| `links` | resolved `links` with the linked quest's name when that quest is in a shown group, else omitted (no spoilers) |

## Rules (logic lane)

1. Read only: the journal calls no mutator (`start`, `abandon`, `complete`, `set_flag`, `refresh` of `QuestRunner` are never called; `JournalModel.refresh` only re-reads).
2. Freshness: `refresh()` runs when the journal opens and after any router `pop` back to it; a flag change elsewhere (for example an NPC death variant) shows on the next open with the changed giver, text or group.
3. No spoilers: quests never started are hidden (group table); objective texts after the current one are not shown; a `blocked` entry shows its resolved description and "On hold" only.
4. Missing data never errors: an unresolved character, item or loot id shows its raw id in brackets (`[mara_voss]`) and `QuestRunner.check` warnings are not shown to the player.
5. Keyboard: arrows move the selection across groups, `ui_cancel` pops.

## Acceptance checks (rule-derived; `journal_test_suite.gd`, sandbox world and quest catalogs via `TestHelpers.fork_worlds` and `QuestCatalog.use_path`)

1. A fresh state lists no quests; after `QuestRunner.start` of one quest it appears under Active with objective 1's text; after its objectives are met, under Ready to turn in; after `complete`, under Completed with `objectives_done == objectives_total`.
2. An `available` and a `locked` quest of the world never appear.
3. Setting the flag of a death variant (M4-A `mara_fallen`) on a started quest changes the shown giver or moves it to On hold, matching `QuestRunner.status` and `resolved` for that state (rule-derived, no literal text).
4. `offers` equals `QuestRunner.offers` mapped to names; after the loot entry is claimed by another quest it disappears from this entry.
5. `JournalModel` never changes the state: `CurrentGame.state.to_dict()` is equal before and after `refresh`, `groups` and `entry` calls.
6. An unresolved giver id shows as `[id]` without error.
7. Existing quest and save suites pass unchanged.

## Out of scope

Lore codex (needs a lore-unlock field first), bestiary, quest tracking markers on the map (EXP1 scene), turning in or abandoning from the journal (dialogue, DLG1), quest notifications or toasts, filters and search.

## Open questions

None (UI menus answers: journal = quests only).
