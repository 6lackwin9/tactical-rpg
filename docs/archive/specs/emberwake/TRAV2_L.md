# TRAV2-L: Travel supplies (S10)

Status: approved (user 2026-10-17 23:25, card 'Both') (rules approved: DECISIONS world-campaign "Travel supplies" and "S9-S11 baseline answers (user 2026-10-17)", S10 Q6 A)
Depends on: TRAV1-L merged (`TravelRunner`, `TravelRules.leg_chance`, save v5), DLG1-L merged (`ShopRules.offers` / `buy`, chest gold in `ExploreRunner._open_chest`), REP1-L merged (shared `shop_rules.gd`)
Lane: logic (one branch; commit 1 is the schema slice that releases TRAV2-A)
Touches: `scripts/progression/party_state.gd` (`supplies`), `scripts/game_state/{save_game,save_migrations,new_game}.gd`, `tools/gen_save_fixtures.gd` + new `scripts/tests/fixtures/saves/save_v6.json`, `scripts/world/world_schema.gd` (`_shop_fields` `supply_price`, manifest `start_supplies`), `scripts/world/world_catalog.gd` (`manifest_errors`), `scripts/exploration/map_rules.gd` (chest `supplies`), `scripts/exploration/explore_runner.gd` (`_open_chest`), `scripts/dialogue/shop_rules.gd` (validation, `offers`, `buy`), `scripts/travel/{travel_rules,travel_runner}.gd` + `README.md`, `scripts/ui/overworld/{world_map_view,shop_panel,region_map_view}.gd` (readouts), `scripts/tests/{travel_test_suite,save_test_suite}.gd`. No new suite and no new script file.

Rules source: `games/emberwake/docs/specs/S10_RULES.md` 2.6 and Q6 A (single counter 0..99, a leg costs `route.time`, shops and chests give supplies, travel is never refused, x1.5 road chance at 0, capped 100).

## Goal

One party counter `PartyState.supplies` (0..99) that every travel leg drains by its route `time`, refilled by buying from shops and opening chests. Running out never blocks travel: it only makes road fights more likely (x1.5, capped at 100) while the counter is 0. Save version 6 adds the counter; old saves start with the new-game value.

## 1. Schema contract (schema-first: yes)

### 1.1 Party state (`PartyState`)

| Name | Value | Notes |
| --- | --- | --- |
| `SUPPLIES_MAX` | 99 | hard cap |
| `SUPPLIES_START` | 10 | new-game fallback and migration value (Defaults picked 1, 2) |
| `var supplies: int` | 0 | `to_dict` key `supplies`; `from_dict`: missing reads 0, a non-whole value loads as -1 so `validate` refuses it (same as `gold`) |
| `add_supplies(n) -> int` | | adds `maxi(n, 0)`, clamps to 0..`SUPPLIES_MAX`, returns the amount actually added |
| `spend_supplies(n) -> int` | | subtracts `mini(maxi(n, 0), supplies)`, returns the amount actually spent (never below 0) |

Validation (`PartyState.validate`): `supplies` outside 0..99 = error ("supplies must be 0..99."). Supplies are a party counter, not a stash item (no `ContentCatalog` entry, no item art, not sellable).

### 1.2 Save version 6 (`SaveGame`, `SaveMigrations`)

`SaveGame.VERSION` = 6; step 5 -> 6 (`_v5_to_v6`) sets `party.supplies = PartyState.SUPPLIES_START` when `party` is an object, nothing else. `GameState.KEYS` is unchanged (the counter lives in `party`). `tools/gen_save_fixtures.gd`: the played state gets `party.supplies = 7`; the v5 fixture is the v6 dict without `party.supplies`; regenerate, commit the new `save_v6.json` (older fixtures must come out byte-identical).

### 1.3 World manifest field `start_supplies` (`_manifest_fields`, `WorldCatalog.manifest_errors`)

| Field | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| start_supplies | int | -1 | -1 (= the nearest parent world's) or 0..99 ("Start supplies must be -1 (inherited) or 0..99.") |

Resolver `TravelRules.start_supplies(world_id) -> int`: the nearest manifest along `WorldCatalog.lineage` (the world first) with `start_supplies >= 0`, else `PartyState.SUPPLIES_START` (mirrors `DialogueRules.start_gold`, whose fallback is 0). `NewGame.create` sets `state.party.supplies` from it right after `party.gold`.

### 1.4 Shop field `supply_price` (world kind `shops`, `_shop_fields`, `ShopRules.shop_errors`)

| Field | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| supply_price | int | 0 | 0 (= the shop sells no supplies) or 1..`ShopRules.SUPPLY_PRICE_MAX` 999 ("Supply price must be 0 (not sold) or 1..999.") |

Gold per supply. A fixed price: like a fixed stock `price`, it ignores `buy_mult` and the REP1 band factors (Defaults picked 3). Unlimited (shops never run out); not tracked in `gs.shops`. Existing shops load with 0 and are unchanged.

### 1.5 Chest marker key `supplies` (`MapRules.MARKER_KINDS["chest"]`, `_chest_errors`)

| Key | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| supplies | int | 0 | 0..`MapRules.CHEST_SUPPLIES_MAX` 99 ("Marker '%s': supplies must be a whole number 0..99.") |

The "holds nothing" rule counts it: a chest with only `supplies > 0` is valid; items, loot and gold empty and supplies 0 stays the error "Chest '%s' holds nothing.". Existing chests load with 0 (marker defaults merge).

No ref scans (no new refs). No quest reward or dialogue effect key (Q1).

## 2. Rules (logic lane)

### 2.1 Spending per leg (`TravelRunner.travel`)

For each leg, in this order (TRAV1-L 2.3 step 1 becomes 1a-1c):

1. a. `starved` = `gs.party.supplies == 0` (read BEFORE paying this leg; Defaults picked 5).
   b. `gs.time += route.time` (unchanged).
   c. `spent += gs.party.spend_supplies(route.time)` (clamps at 0; travel is never refused for lack of supplies).
2. Scripted check (unchanged: scripted road events ignore supplies).
3. The road roll uses `TravelRules.leg_chance(gs, route, starved)`.

A refused trip (no node, no path) spends nothing. A leg whose road fight is fled or lost keeps its supplies spent (like its time); a won fight completes the leg without a second charge. `travel()` results (fight, arrived, stopped) gain `supplies_spent` (int, this call) and `starved_legs` (int, legs rolled at 0).

### 2.2 Starved chance (`TravelRules`)

- `STARVED_PCT` = 150; `starved_chance(chance: int) -> int` = `mini(PCT_MAX, chance * STARVED_PCT / 100)` (integer, rounded down).
- `leg_chance(gs, route, starved: bool = false) -> int`: the TRAV1 value (first traversal `encounter_pct`, later the resolved repeat chance), then `starved_chance` when `starved`. Applies to the first and the repeat chance alike; `elite_pct` and the pick are unchanged; a 0 chance stays 0.
- Determinism: starving changes only the threshold, never the number of draws (`rolls.travel` rises by the same count for an equal trip).

Reference values: 15 -> 22, 7 -> 10, 25 -> 37, 12 -> 18, 20 -> 30, 10 -> 15, 41 -> 61, 67 -> 100, 100 -> 100.

### 2.3 Buying supplies (`ShopRules.offers` / `buy`)

- `offers`: when the shop is open and `supply_price > 0`, one row is appended after the loot rows: `{kind: "supplies", id: "supplies", item: "", count: 1, price: supply_price, left: SUPPLIES_MAX - gs.party.supplies}` (`left` = room; 0 = full). A closed or hostile shop offers none (REP1 rule unchanged).
- `buy(gs, shop, "supplies", "supplies", count)`: the existing checks (open, `count` 1..`BUY_MAX`, offered) plus: `count > left` refuses ("You can carry at most 99 supplies."); `price x count > gold` refuses ("Not enough gold."). All-or-nothing; then `gold -= total`, `add_supplies(count)`; no stash change, no `gs.shops` entry. Returns `{ok, errors, price (total), supplies: count}` (`items` absent).

### 2.4 Chest supplies (`ExploreRunner._open_chest`)

After `RewardGrant.apply` succeeds (supplies are not a reward key): `added = gs.party.add_supplies(m.supplies)`; the result gains `supplies` (= `added`); when `added < m.supplies` it gains `notes` ["Supplies are capped at 99 (%d left behind)."] (the chest is still marked done; Defaults picked 6). A refused grant adds no supplies.

### 2.5 Readouts (cheap, no new scene)

- World map (`WorldMapView._refresh`): the status line gains `supplies N/99`; while 0, an extra line "Out of supplies: road fights are more likely." The arrival note becomes "Arrived at %s (time %d, supplies -%d)." from `supplies_spent`. `TravelRules.view(gs)` gains `supplies` (int).
- Shop panel: the gold label reads "Gold: %d  Supplies: %d/99"; the supplies offer renders as "Supplies x1 - %d gold" and "Supplies x5 - %d gold" (count `mini(5, left)`; both disabled when `left == 0` or gold is short; label "(full)" when `left == 0`). `buy_one` passes the count.
- Region map chest note: "Opened: ..." gains " +%d supplies" when `supplies > 0`, plus the cap note.

## 3. Acceptance checks (`travel_test_suite.gd`; forked world root, never shipped data)

Checks 1-5 ship with commit 1, 6-14 with commit 2. `save_test_suite` keeps its v4 -> v5 check as a two-step check with explicit target 5 and its `SaveGame.VERSION == 5` assertion becomes `>= 5`.

1. Party: `supplies` -1 and 100 fail `validate`; 0 and 99 pass; a String in a saved dict loads as -1 and fails; `add_supplies(50)` from 60 returns 39 and leaves 99; `spend_supplies(5)` from 3 returns 3 and leaves 0; negative arguments change nothing.
2. Save v6: a v5 envelope migrates to `party.supplies` 10 and nothing else differs (deep-equal with the expectation); `write` refuses a state with supplies 100; `save_v6.json` loads and round-trips; v1..v5 fixtures still load.
3. Manifest: `start_supplies` -2, 100 and "ten": one error each; a child with -1 inherits the parent's 4; a world with no value anywhere starts at 10; `NewGame.create` uses the resolved value.
4. Shop: `supply_price` -1, 1000 and 2.5: one error each; 0 and 999 valid; an existing shop without the field validates unchanged (offer counts of the dialogue suite unchanged).
5. Chest: `supplies` -1 and 100: errors; a chest with only `supplies 3` is valid; a chest with nothing (supplies 0) still errors.
6. Spending: a trip of legs 2 + 3 from 10 leaves 5 and reports `supplies_spent` 5; from 4 leaves 0 and reports 4; travel from 0 is not refused; a refused trip changes nothing.
7. Starved roll: with supplies 0 at the leg start, `encounter_pct 20` rolls at 30; a traversed route with `repeat_pct -1` and `encounter_pct 25` rolls at 18; `encounter_pct 67` rolls at 100; `encounter_pct 0` never fights; elite chance unchanged.
8. Leg-start rule: from 2 supplies, a 2-leg trip of 3 + 3 rolls leg 1 at the normal chance and leg 2 starved; `starved_legs` = 1.
9. Determinism: two equal states at 0 supplies travelling the same trip give equal fights; `rolls.travel` after a starved trip equals that of the same trip with supplies (same draws).
10. Scripted: a scripted road event fires the same at 0 supplies; fled keeps the leg's supplies spent; won does not charge the leg twice.
11. Shop buy: price 2, buying 5 from 10 gold and 3 supplies leaves 0 gold and 8 supplies; buying 3 at 97 supplies refuses "You can carry at most 99 supplies." with nothing changed; unaffordable refuses; a shop with `supply_price 0` lists no supplies row; a closed shop refuses; `gs.shops` and the stash are unchanged.
12. Band independence: at `friendly` and `unfriendly` the supplies price stays `supply_price`.
13. Chest: opening a chest with supplies 6 at 5 gives 11 and reports 6; at 97 it reports 2 with the cap note and is done; opening again gives nothing.
14. View: `TravelRules.view(gs).supplies` equals `gs.party.supplies`.

## 4. Commit slices (one branch)

1. Schema (releases TRAV2-A): `PartyState` field, constants, add/spend, validate, to/from dict; save v6 + `_v5_to_v6` + fixture generator + `save_v6.json`; manifest `start_supplies` + validation + `TravelRules.start_supplies` + `NewGame`; shop `supply_price` and chest `supplies` fields + validation; checks 1-5; save suite update; `scripts/game_state/README.md` (v6).
2. Behaviour: per-leg spending and the starved chance in `TravelRunner` / `TravelRules`, shop offer and buy, chest grant, the three readouts, `view` key, checks 6-14, `scripts/travel/README.md`, `scripts/dialogue/README.md`, `scripts/exploration/README.md` updated.

TRAV2-A may start after commit 1 merges; its data merges only after commit 2 (so a supplies price or chest value never sits in data without effect, same as the DLG1-A rule).

## 5. Docs delta (builder reports, planner applies)

- README.md: travel paragraph gains "supplies (0..99): each leg costs its route time; buy at shops, find in chests; at 0 road fights are x1.5 likely; travel is never refused"; save version 6.
- GAME_BRIEF S10 line: `PartyState.supplies`, save v6. STRUCTURE.md: no new rows.

## Out of scope

Supplies from quest rewards, dialogue `give` / `take` or selling; per-route supply costs other than `time`; supplies as a stash item; rations per member, food types, spoilage over `time`; any fail state, refusal or combat debuff at 0 (Q6 C was not chosen); a trip-cost preview before travelling; reputation factors on the supply price; shipped data (TRAV2-A); balance sims (the sim does not cover travel).

## Open questions

- Q1 (decided: not in TRAV2, user 2026-10-17 23:33, DECISIONS world-campaign "Supplies sources (TRAV2)"). Should quests (reward key `supplies`) and dialogue `give` effects also grant supplies? R: not in TRAV2 (the decision names shops and chests only); add a `supplies` reward key later if the CAMP1 story pass wants a quartermaster gift.

## Defaults picked (not user-confirmed)

1. A new game starts with 10 supplies (manifest `start_supplies`, fallback `PartyState.SUPPLIES_START` 10): enough for the Ashlands round trip keep - courtyard (4) plus keep - Cinder Gate one way (4), or one way to the Slag Gate (6) with 4 left.
2. Old saves migrate to the same 10 (gentler than 0, which would start every migrated party starved).
3. Supplies are bought through a shop field `supply_price` (fixed gold per unit, unlimited, ignores bands), not a stock entry (stock entries must be catalog items) and not a dialogue service (no new service line or choice).
4. Supplies always show on the world map and in the shop panel; at 0 the world map adds the warning line "Out of supplies: road fights are more likely."
5. "While supplies are 0" is read at the start of each leg (before paying it): a leg the party could partly pay rolls at the normal chance.
6. Chest supplies above the cap are lost with a note; the chest still counts as opened.
7. The x1.5 rounds down (`chance * 150 / 100`), so it never adds a fight the rule did not ask for.
