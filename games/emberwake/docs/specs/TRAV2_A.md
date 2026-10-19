# TRAV2-A: Ashlands supply sources and start supplies (S10)

Status: approved (user 2026-10-17 23:25, card 'Both') (rules: `games/emberwake/docs/specs/TRAV2_L.md`)
Depends on: TRAV2-L commit 1 merged (fields `start_supplies`, `supply_price`, chest `supplies`); merges only after TRAV2-L commit 2 (TRAV2_L.md section 4)
Lane: asset (one branch, seed `data/seeds/trav2a.gd`)
Touches: `data/worlds/ashlands/{world,shops,locations}.json` via `WorldCatalog.update_manifest` / `upsert` + `save_world`; `data/seeds/trav2a.gd`, `data/seeds/README.md`. No code, no tests, no `data/catalog/*` edit.

## Goal

Give the sample campaign a supply loop: a new Ashlands game starts with enough supplies for a few legs, the keep quartermaster sells them cheaply, and the three Ashlands chests top them up, so a party that buys nothing still reaches the Cinder Gate and the Slag Gate (and only rolls the x1.5 road chance on the way back from a long trip).

## 1. Numbers and why

Route times (TRAV1-A): old road 2, high road 4, south road 6. Round trips from the keep: courtyard 4, Cinder Gate 8, Slag Gate 12. Gold (DLG1-A): start 50, quest gold 20-40 each, chest gold 10-25, healing potion 15.

| Source | Where | Value | Effect |
| --- | --- | --- | --- |
| Start supplies | manifest `ashlands` `start_supplies` | 10 | one courtyard round trip plus the road to the Cinder Gate, or the road to the Slag Gate with 4 left |
| Shop | `ashlands:hest_stores` `supply_price` | 2 gold each | a Slag Gate round trip (12) costs 24 gold, a bit over one and a half healing potions; a full 99 is out of reach early |
| Chest | `ashlands:warden_keep` marker `stores` | 4 | first refill at home |
| Chest | `ashlands:ruined_courtyard` marker `ruin_cache` | 6 | rewards the first trip out |
| Chest | `ashlands:cinder_gate` marker `camp_cache` | 6 | covers most of the high road back |

Iron Reaches and Tidewrack get no supply source in TRAV2-A (Iron Reaches has no shop and no chest yet; its manifest stays at the -1 default and is not playable on its own).

## 2. Edits

### 2.1 Manifest

`WorldCatalog.update_manifest("ashlands", {"start_supplies": 10})`.

### 2.2 Shop

Read `ashlands` shop `hest_stores` with `get_own`, set `supply_price` 2, upsert under the same id. Stock, loot, `buy_mult`, `sell_mult`, `buys` and `requires_flags` stay as they are.

### 2.3 Chest markers

For each row, read the location with `get_own`, set the `supplies` key on the named chest marker in `markers` (all other marker keys unchanged), upsert under the same id.

| Location | Marker id | supplies |
| --- | --- | --- |
| warden_keep | stores | 4 |
| ruined_courtyard | ruin_cache | 6 |
| cinder_gate | camp_cache | 6 |

## 3. Authoring rules

- Seed order: manifest, shop, the three chest markers, then `save_world("ashlands")`.
- Stop on the first problem (a missing shop or marker id is a stop, never a create); never patch JSON by hand. Register the seed in `data/seeds/README.md` (task TRAV2-A).

## 4. Acceptance checks (asset lane, headless after the seed)

1. `WorldCatalog.validate_all()` == {} (Iron Reaches and Tidewrack included); `TravelRules.warnings("ashlands")` empty.
2. `TravelRules.start_supplies("ashlands")` == 10; `NewGame.create_for_world("ashlands")` gives `party.supplies` 10.
3. `ShopRules.offers(gs, "hest_stores")` on a new Ashlands game ends with `{kind: "supplies", price: 2, left: 89}`; the item and loot rows before it are unchanged.
4. Planner smoke script after merge (TRAV2-L commit 2 merged): a new game opens the keep chest `stores` (supplies 14), travels to the Ruined Courtyard (12; winning the scripted first raid on the old road does not charge the leg again), opens `ruin_cache` (18); buying 5 at Hest costs 10 gold.

## Out of scope

Supply sources outside the Ashlands, new shops or chests, quest or dialogue supply rewards (TRAV2_L.md Q1), route time changes, gold retuning.

## Open questions

None.
