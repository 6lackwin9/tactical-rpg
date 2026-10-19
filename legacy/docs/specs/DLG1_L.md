# DLG1-L: Dialogue, quest services, NPC presence, shops and gold (S11)

Status: spec (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)"; this build spec awaits the user's approval)
Depends on: EXP1-L merged (markers, `BattleBridge`, `ExploreRunner.wake`, save v3), M4-L merged
Lane: logic (one branch; commit 1 is the schema slice that releases DLG1-A)
Touches: new `scripts/dialogue/{dialogue_rules,dialogue_runner,service_catalog,shop_rules}.gd` + `README.md`, new `data/dialogue/services.json` (empty envelope) + `data/dialogue/README.md`, `scripts/world/world_schema.gd` (kinds `dialogues`, `shops`; character fields; manifest fields), `scripts/world/world_catalog.gd` (dispatch, `_shape_errors`, `_json_refs_to` dialogue/shop/character parts), `scripts/tests/world_test_suite.gd` (kind list), `scripts/quests/quest_rules.gd` (reward key `gold`), `scripts/quests/reward_grant.gd` (gold, `take`), `scripts/progression/party_state.gd` (gold, `remove_from_stash`), `scripts/game_state/{game_state,save_game,save_migrations,new_game}.gd`, `scripts/exploration/{map_rules,explore_runner}.gd` (npc presence, chest gold), new `scripts/tests/dialogue_test_suite.gd` + one line in `suite_registry.gd`, `scripts/tests/save_test_suite.gd` (version 4), new `scripts/ui/overworld/{dialogue_panel,shop_panel}.gd` + `scripts/ui/overworld/region_map_view.gd` (opens them).

Rules source: `docs/specs/S11_RULES.md` (Q1 A, Q2 A, Q3 A, Q4 B, Q6 A). REP1 adds operators, `add_flag` and price bands; DLG1 conditions are equality only.

## Goal

Talk to world characters: authored trees with gated choices and whitelisted all-or-nothing effects, generated quest services that make M4 `giver` / `turn_in` binding, `gone_flags` presence, shops with derived prices and claim-once loot sales, party gold (quests, chests, selling, dialogue gifts), and save version 4.

## 1. Schema contract (schema-first: yes)

### 1.1 World kind `dialogues` (`data/worlds/<id>/dialogues.json`)

Added to `WorldSchema.KINDS` (inherited, overridable, missing file = empty kind). Validation in `DialogueRules.tree_errors(world_id, e)`.

| Field | Type (schema) | Default | Validation (message) |
| --- | --- | --- | --- |
| id, name | id, string | required, "New Dialogue" | generic rules |
| speaker | ref characters, optional | "" | default speaker of every node ("" = the character talked to) |
| start | string | required | a node id ("Start node '%s' does not exist.") |
| nodes | json | [] | 1..`MAX_NODES` 64 ("A dialogue holds 1..64 nodes.") |
| services | bool | true | true = service choices are appended to the start node, which then needs choices ("With services on, the start node needs choices.") |

Node keys exactly `id`, `speaker`, `text`, `choices`, `effects`, `next`, `end` ("Node '%s': unknown key '%s'."): id `id_ok`, unique; speaker "" or a character ref; text non-empty, at most 1000 chars; exactly one of non-empty `choices`, non-empty `next` (a node id), `end: true` ("Node '%s' needs exactly one of choices, next or end."); `effects` 0..8, only `set_flag` and `give` ("Node effects may only be set_flag or give."). Warning: a node no `to`/`next` reaches from `start`.

Choice keys exactly `text`, `to`, `requires`, `unless`, `effects`, `show_locked`: 1..`MAX_CHOICES` 6 per node; text non-empty; `to` "" (= end) or a node id; `requires` / `unless` = conditions (1.2); effects 0..8 (1.3); `show_locked` bool, default false.

### 1.2 Conditions (`DialogueRules.CONDITION_KINDS`, dialogue choices only)

Each `{"kind": K, ...}`, kind default `flag`; unknown kind or key = error. Markers, quests, loot, routes and `gone_flags` keep the M4 flag-only grammar.

| Kind | Keys (default) | Validation | Holds when |
| --- | --- | --- | --- |
| flag | `key`, `value` (true) | as `QuestRules.condition_errors` | `QuestRules.condition_met` |
| quest | `quest`, `status` | quest ref resolves; status one of `locked`, `available`, `unavailable`, `active`, `ready`, `blocked`, `completed` | `QuestRunner.status` equals it |
| items | `item`, `count` (1) | base item exists; count 1..99 | `party.stash[item] >= count` |
| gold | `min` | 0..9999999 | `party.gold >= min` |
| stat | `stat`, `min` | stat in `ContentSchema.STAT_KEYS`; min 1..30 | the highest effective value (`PartyBuilder.build`, gear and passives included) among active members >= min |

### 1.3 Effects (`DialogueRules.EFFECTS`, `{"effect": E, ...}`)

| Effect | Keys | Validation |
| --- | --- | --- |
| set_flag | `key`, `value` (true) | `flag_key_ok`; value bool/int/String |
| start_quest | `quest` | a quest with a template |
| turn_in | `quest` | a quest with a template; the FIRST effect of its choice; later effects only `set_flag` / `give` ("turn_in must come first; only set_flag or give may follow.") |
| give | `items` ({}), `gold` (0) | items: base id -> 1..99, max 8 keys; gold 0..99999; not both empty |
| take | `items` ({}), `gold` (0) | same |
| open_shop | `shop` | shop ref; the LAST effect |
| fight | `encounter` | encounter ref; the LAST effect; not with `turn_in` or `open_shop` in the same choice |

### 1.4 Characters, manifest, shops, service lines

Character fields (additive): `dialogue` ref dialogues, optional, "" (= services only); `gone_flags` json [] (`QuestRules.condition_errors`; ANY holding = gone).

Manifest fields: `start_gold` int, default -1 (= nearest ancestor's value, else 0), -1..99999; `service_lines` json {} (line id -> text, merged along the lineage key by key, nearest wins; ids and tokens checked as in the service file).

World kind `shops` (`data/worlds/<id>/shops.json`; validation `ShopRules.shop_errors(world_id, e)`):

| Field | Type (schema) | Default | Validation |
| --- | --- | --- | --- |
| id, name | id, string | required, "New Shop" | generic |
| keeper | ref characters | required | resolves ("Choose the shop keeper.") |
| stock | json | [] | max 32; entry exactly `{item, count, price, requires_flags}`; base item exists and is not `QUEST` ("Quest items can't be stocked."); count -1 or 1..99; price -1 or 1..99999; requires_flags conditions; no item twice |
| loot | json | [] | max 8; entry exactly `{loot, price}`; a world loot ref with `reward_kind` item ("Shops sell item loot only."); price 1..99999; no ref twice |
| buy_mult | float | 1.0 | 0.5..3.0, at most two decimals |
| sell_mult | float | 0.5 | 0.0..1.0, at most two decimals; 0 = buys nothing |
| buys | json | ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL"] | each an item type other than `QUEST`, no duplicates |
| requires_flags | json | [] | conditions |

Base service lines: `data/dialogue/services.json`, envelope `{"kind": "dialogue_services", "version": 1, "entries": [{"id", "text"}]}`; `ServiceCatalog` has the `QuestCatalog` contract. Ids exactly `DialogueRules.SERVICE_IDS` = `greeting`, `offer`, `accept`, `decline`, `turn_in`, `shop`, `farewell` (unknown id = error; missing id = warning, the runtime uses `DialogueRules.DEFAULT_LINES`); text non-empty; tokens only `{npc}`, `{quest}`, `{shop}`, `{gold}` ("Unknown token '%s'."). Commit 1 ships `entries: []`.

Quest template reward key `gold` (`QuestRules.REWARD_KEYS`): int 0..`GOLD_MAX` 5000; soft warning above `SOFT_MAX_GOLD` 200. Chest marker key `gold` (`MapRules`): int 0..9999, default 0; a chest with only gold is valid.

Ref scans: dialogue `speaker`, node speakers, condition `quest` refs, effect `quest` / `shop` / `encounter` refs, shop `keeper` and `loot` refs, character `dialogue`.

### 1.5 State and save version 4

- `PartyState.gold` int 0..9999999 (default 0; in `to_dict` / `from_dict` / `validate`: "gold must be 0..9999999."); `PartyState.remove_from_stash(id, count) -> bool` (false and no change when fewer are held).
- `GameState.shops` `{shop qid: {item id: sold count >= 1}}` (added to `KEYS`; validate: qid keys, `id_ok` item ids, ints >= 1).
- `SaveGame.VERSION` = 4; step 3 -> 4 adds `party.gold: 0` and `shops: {}`. `NewGame.create` sets `gold` = the resolved `start_gold`.

## 2. Rules (logic lane)

### 2.1 Presence

`DialogueRules.is_gone(gs, character_ref)`: any `gone_flags` condition holds, re-read on every call (no saved state). A gone character cannot be opened ("%s is gone."), its `npc` markers are absent (`MapRules` presence), its shops are closed, its services are never generated.

### 2.2 Opening and showing (`DialogueRunner`, one instance per conversation, never saved)

`DialogueRunner.open(gs, character_ref) -> DialogueRunner` (`errors` non-empty = not opened). The tree is the character's `dialogue`; "" = a generated root node with the `greeting` line and the service choices only. `view() -> Dictionary`: `{speaker, speaker_name, node, text, choices: [{text, locked}], end}`; tokens replaced (`{npc}` talked-to name, `{quest}` resolved quest name, `{shop}` shop name, `{gold}` party gold). Showing a node runs its `effects` (every time it is shown), except returning to the node a shop was opened from (2.4 step 5).

Visible choices, in order: authored choices whose `requires` all hold and `unless` none hold; failing ones with `show_locked` appear `locked` (not choosable); the rest are hidden. Each `stat` condition in `requires` prefixes the text with `[STAT min]` (example `[KNOW 14] `), generated, never authored. Indexes passed to `choose` refer to the visible list.

### 2.3 Services (appended to the start node, in this order, unless `services: false`)

C = the talked-to character's qid; quests are `QuestRunner.quests_with` order; giver and turn-in refs resolve from `gs.world_id` to qids.

1. `offer` for each quest whose RESOLVED giver is C and status is `available` -> a generated node: text = the resolved description (name when empty), choices `accept` (`start_quest`) and `decline` (back to the start node).
2. `turn_in` for each quest whose resolved effective turn-in (`turn_in`, else `giver`) is C and status is `ready` -> a `turn_in` effect.
3. `shop` for each open shop (2.5) whose keeper is C -> `open_shop`.
4. `farewell` -> end.

Enforcement (also for authored effects): `start_quest` is refused unless the resolved giver is "" or C ("Only %s gives this quest."); `turn_in` is refused unless the effective turn-in is "" or C. `DialogueRules.complete_from_journal(gs, ref, recipient, picks)` completes a `ready` quest only when giver and turn-in both resolve to "" ("This quest is turned in to %s."). `QuestRunner.start` / `complete` stay unrestricted for tests and tools; variants re-route services automatically because only resolved quests are read.

### 2.4 Choosing (`choose(index) -> Dictionary`)

1. A locked or unknown index = error, nothing changes.
2. Pre-check every effect against the current state: `start_quest` needs `available` plus enforcement; `turn_in` needs `ready` plus enforcement; all `take` effects together need the summed items and gold; `open_shop` needs an open shop; `fight` needs a resolving encounter. Any failure = `{ok: false, errors}` and nothing changes (all-or-nothing).
3. With `turn_in`: nothing is applied yet; returns `{pending: "turn_in", quest, offers, picks_allowed}`. `confirm_turn_in(recipient_id, picks)` calls `QuestRunner.complete` (all-or-nothing itself), then the remaining effects, then moves on; `cancel()` keeps the node and changes nothing.
4. Otherwise apply in order: `set_flag` via `QuestRunner.set_flag`; `start_quest` via `QuestRunner.start`; `give` via `RewardGrant.apply(gs, {items, gold}, "")`; `take` via `RewardGrant.take`.
5. Move: `fight` closes the dialogue and returns `{fight: {source: "dialogue", encounter, character}}` (after the battle `DialogueRunner.after_fight(gs, fight, outcome)` = `BattleBridge.apply`, plus `ExploreRunner.wake(gs)` on `lost`); `open_shop` returns `{shop: qid}` and `close_shop()` shows the choice's `to` node (`to` "" = the current node again, WITHOUT re-running its effects: user 2026-10-17, decision "DLG1 shop return"; a different `to` node runs its effects as usual); else the `to` node, "" = end.

`RewardGrant.take(gs, items, gold) -> Array[String]`: validates holdings first ("Not enough %s."), then removes. `RewardGrant.apply` adds `gold` and echoes it in `granted`.

### 2.5 Shops (`ShopRules`, pure)

- Open: shop `requires_flags` hold and the keeper is not gone.
- Integer prices: `bm = roundi(buy_mult x 100)`, `sm = roundi(sell_mult x 100)`, `rb = rs = 100` (REP1 replaces them). Buy = `price` when > 0, else `maxi(1, ceil(value x bm x rb / 10000))`; sell = `floor(value x sm x rs / 10000)`. `value` = the base item's `value`. Loot sells at its fixed `price`.
- `offers(gs, shop_ref) -> Array[Dictionary]` `{kind: "item" | "loot", id, item, count, price, left}`: stock entries whose `requires_flags` hold (`left` -1 = unlimited, else `count - sold`), then loot entries that are unclaimed and whose loot `requires_flags` hold.
- `buy(gs, shop_ref, kind, id, count := 1) -> Dictionary`: refused when closed, `count` above `BUY_MAX` 99 ("Buy at most 99 at a time.", DLG1-F), `left < count` ("Sold out."), gold < price x count ("Not enough gold."), or loot with count != 1; then gold falls, the stash gains the items, limited stock adds to `gs.shops`, loot adds its qid through `RewardGrant.claim_loot` (it leaves every quest pool).
- `sell_list(gs, shop_ref)`: stash items whose type is in `buys`, not `QUEST`, sell price >= 1. `sell(gs, shop_ref, item, count := 1)`: refused otherwise; stash falls, gold rises (above `GOLD_MAX` the rest is dropped and reported: result `gold` = added, plus a note; DLG1-F). Sold items never restock the shop; no restock at all in DLG1.

### 2.6 Authoring lint (`DialogueRules.quest_warnings(world_id)`, warnings only)

For each merged quest with a template, for its resolved-from-base giver G and effective turn-in T: for every condition c in the `gone_flags` of G (or T), warn unless some variant has a `when_flags` entry equal to c (same key and value) whose `set` replaces `giver` (for G; also covers T when `turn_in` is ""), replaces `turn_in` (for T), or sets `available: false`. Message: "Quest '%s': %s '%s' can be gone (%s) and no variant re-routes it."

## 3. Acceptance checks (`dialogue_test_suite.gd`; fixtures in a forked world root and test paths, never shipped data)

Checks 1-6 ship with commit 1, 7-20 with commit 2.

1. Trees: missing start, dangling `to`/`next`, node with choices and `end`, 65 nodes, 7 choices, unknown node key, empty text, node effect `take`, services on with an `end` start node: one error each; an unreachable node warns.
2. Conditions: unknown kind, status `done`, unknown item, count 0, stat `LUCK`, min 31, extra key: errors.
3. Effects: unknown effect, `fight` not last, `open_shop` not last, `fight` + `turn_in`, `turn_in` second, `take` after `turn_in`, give with nothing, count 100, unknown shop: errors.
4. Shops: unknown keeper, QUEST item in stock, count 0, price 0, passive loot entry, `buy_mult` 0.4 and 1.234, `buys` `QUEST`, duplicate item: errors.
5. Template `rewards.gold` 5001 = error, 201 = warning; chest `gold` 10000 = error, a gold-only chest is valid; `gone_flags` with a bad key = error; services file: unknown id error, unknown token error, missing id warning.
6. Lint: giver with `gone_flags` [f] and no variant warns; a variant `when_flags` [f] setting giver clears it; `available: false` clears it; a variant on a different flag does not.
7. Visibility: hidden vs `show_locked`; choosing a locked choice changes nothing; the stat prefix shows; the best active member decides (a benched member's stat does not).
8. All-or-nothing: a choice `[give 5 gold, take 2 potions]` with one potion held changes nothing; two `take` effects are summed.
9. Services: an available quest given by C is offered at C only; a variant changing the giver moves the offer; a ready quest shows "We finished" only at the effective turn-in; Accept starts it; `decline` returns to start.
10. Enforcement: an authored `start_quest` in another NPC's tree is refused; `complete_from_journal` works only with empty giver and turn-in.
11. Turn-in: `pending` lists offers; `confirm_turn_in` with a bad pick changes nothing; a valid one completes and then applies the trailing `set_flag`; `cancel` keeps everything equal.
12. Gone: setting a gone flag closes the conversation on the next `open`, removes the npc marker, closes the keeper's shop and hides services; clearing it restores them.
13. Node effects run each time the node is shown; a repeat-once choice authored with `unless` + `set_flag` shows once.
14. Prices: derived buy `ceil` and sell `floor` on values 15 and 25 at 1.0/0.5 and 1.15/0.85 equivalents; fixed `price` ignores multipliers; minimum buy 1.
15. Stock: a count-2 entry sells twice then "Sold out." and `gs.shops` round-trips; unlimited never runs out; not enough gold changes nothing.
16. Loot sale: buying claims the loot, the quest no longer offers it, the shop no longer lists it; a loot entry with failing `requires_flags` is not listed.
17. Selling: QUEST items and types outside `buys` are not listed; `sell_mult` 0 lists nothing; gold rises by the sell price.
18. Gold: quest `rewards.gold` reaches `party.gold` once; a chest with gold adds it on first open only; `NewGame` uses the inherited `start_gold`.
19. Fight effect: returns the fight and closes; `after_fight(lost)` wakes the party by the EXP1 rule; `won` advances a waiting `win_encounter` quest.
20. Save v4: a v3 envelope migrates to `gold: 0` and `shops: {}`; negative gold, a shops entry with count 0, or a bad shop qid fails `validate` and `write`.

## 4. Commit slices (one branch)

1. Schema (releases DLG1-A): kinds `dialogues` and `shops`, character and manifest fields, `DialogueRules` validation and lint, `ShopRules.shop_errors`, `ServiceCatalog` + empty `services.json`, `rewards.gold` and chest `gold` validation, ref scans, world suite kind list, checks 1-6, READMEs.
2. Runtime: `DialogueRunner`, services, presence (incl. `MapRules` npc absence), `ShopRules` runtime, `PartyState.gold`, `RewardGrant` gold/take, chest gold, save v4 + migration, `NewGame` gold, checks 7-20, save suite update.
3. Panels: `dialogue_panel.gd` (text, choices, locked greyed, turn-in step with recipient picker and loot checkboxes) and `shop_panel.gd` (buy/sell lists, gold), opened by the region map debug scene on npc interaction; a dialogue fight uses the scene's debug outcome buttons.

## 5. Proposed STRUCTURE.md rows (planner adds)

- `scripts/dialogue/` (S11, DLG1): DialogueRules, DialogueRunner, ServiceCatalog, ShopRules (placeholder -> in progress).
- `data/dialogue/` (S11): `services.json` lore-free service lines (the row "dialogue trees" becomes "service lines"; trees are world data in `data/worlds/<id>/dialogues.json`).
- `scripts/tests/dialogue_test_suite.gd` (S11).

## Out of scope

Reputation, operators and `add_flag` (REP1), kill gold, restocking, barter, party recruitment, portraits and voice, rolled checks, passives or skills from dialogue, a journal UI (UI3), timed events, shipped data (DLG1-A).

## Open questions

None (the S11 answers cover every choice; operators wait for REP1).
