# DLG1-A: Service lines, Ashlands dialogue, the keep shop and gold (S11)

Status update (2026-10-20): approved and built; the rows are done (docs/archive/TASKS_DONE.md). The status line below is the original.

Status: spec (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)"; this build spec awaits the user's approval)
Depends on: DLG1-L commit 1 merged; EXP1-A merged (npc and chest markers); M4-A merged
Lane: asset (one branch, seed `data/seeds/dlg1a.gd`)
Touches: `data/dialogue/services.json` (via `ServiceCatalog.upsert` + `save`), `data/quests/quests.json` (template `gold` via `QuestCatalog.upsert`), `data/worlds/ashlands/{world,characters,quests,locations}.json`, new `data/worlds/ashlands/{dialogues,shops}.json` (via `WorldCatalog.update_manifest` / `upsert` + `save_world`), `data/seeds/dlg1a.gd`, `data/seeds/README.md`. No code, no tests, no `data/catalog/*` edit.

## Goal

Base service lines; trees for Mara, Tev and Hest (a stat check, quest-status gates, a take/give trade, repeat-once gifts); `gone_flags` matching the M4 NPC-death variants plus the one variant the new lint needs; the keep shop with one loot sale; start gold, quest gold and chest gold.

## 1. Service lines (`data/dialogue/services.json`, this order)

| id | text |
| --- | --- |
| greeting | {npc} looks up. |
| offer | About {quest}... |
| accept | We will do it. |
| decline | Not now. |
| turn_in | We finished: {quest}. |
| shop | Show me your wares. |
| farewell | Farewell. |

## 2. Gold

- Manifest: `WorldCatalog.update_manifest("ashlands", {"start_gold": 50})`.
- Templates (read, merge `rewards.gold`, upsert under the same id): reach_and_clear 30, two_stop_delivery 25, clear_site 40, signal_and_return 20 (all under `SOFT_MAX_GOLD` 200).
- Chest markers (merge the `gold` key into the EXP1-A marker, upsert the location): warden_keep `stores` 10; ruined_courtyard `ruin_cache` 25; cinder_gate `camp_cache` 15.

## 3. Characters (merge into the existing entries)

| Character | dialogue | gone_flags |
| --- | --- | --- |
| mara_voss | mara_talk | [{key ashlands.mara_fallen, value true}] |
| tev_quill | tev_talk | [{key ashlands.tev_fallen, value true}] |
| hest_the_quartermaster | hest_talk | [] |

## 4. Quest variant required by the lint

`hold_the_courtyard` (giver Mara, turn-in = giver) gains `variants` [{id `mara_fallen`, when_flags [{key ashlands.mara_fallen, value true}], unless_flags [], set {giver tev_quill, turn_in tev_quill, description "Mara is gone. Tev holds the line in her place: clear the Courtyard."}}]. `cinder_gate_watch` (mara_fallen re-routes giver and turn-in) and `warden_oath` (tev_fallen sets `available: false`) already pass.

## 5. Dialogues (`data/worlds/ashlands/dialogues.json`)

All three: `services` true, tags []. Conditions without `kind` are flag conditions with value true; `set_flag` values are true.

### 5.1 `mara_talk` (name "Mara at the map table", speaker mara_voss, start `hello`)

| Node | text | choices / next | effects |
| --- | --- | --- | --- |
| hello | Mara looks up from a map pinned flat with knives. "Make it quick." | 1. "What is this place?" -> keep. 2. "These ash drifts are fresh. The Horde is close." requires [stat KNOW 14], unless [ashlands.mara_drifts_told], show_locked true -> drifts. 3. "The Courtyard is ours." requires [quest hold_the_courtyard completed], unless [ashlands.mara_courtyard_thanks] -> proud | |
| keep | "Warden Keep. One fire, four walls and too few of us to hold either." | next hello | |
| drifts | "Sharp eyes. Take this; you will need it before I do." | next hello | set_flag ashlands.mara_drifts_told; give items {con_focus_elixir: 1} |
| proud | "It is. For now. Drink to it later." | next hello | set_flag ashlands.mara_courtyard_thanks; give gold 20 |

### 5.2 `tev_talk` (name "Tev by the arrow bench", speaker tev_quill, start `hello`)

| Node | text | choices / next | effects |
| --- | --- | --- | --- |
| hello | Tev is whittling arrow shafts. "Smoke's wrong today." | 1. "What do you see?" -> smoke. 2. "The pass is clear." requires [quest cinder_gate_watch completed], unless [ashlands.tev_pass_thanks] -> thanks | |
| smoke | "Too much of it, too close, and upwind. Someone wants us looking the wrong way." | next hello | |
| thanks | "Then I owe you a drink and an arrow. Have the arrow money." | next hello | set_flag ashlands.tev_pass_thanks; give gold 15 |

### 5.3 `hest_talk` (name "Hest at the stores", speaker hest_the_quartermaster, start `hello`)

| Node | text | choices / next | effects |
| --- | --- | --- | --- |
| hello | Hest counts arrows without looking up. "Buying or bothering?" | 1. "Trade a fire bomb back for coin?" requires [items con_fire_bomb 1], show_locked true, effects [take items {con_fire_bomb: 1}, give gold 20] -> traded. 2. "What do the stores need?" -> needs | |
| traded | "Fewer bombs in the hall is fine by me." | next hello | |
| needs | "Everything. Bring the potions back unopened and we will get along." | next hello | |

JSON shapes: stat condition `{"kind": "stat", "stat": "KNOW", "min": 14}`; quest condition `{"kind": "quest", "quest": "hold_the_courtyard", "status": "completed"}`; items condition `{"kind": "items", "item": "con_fire_bomb", "count": 1}`; effects `{"effect": "set_flag", "key": "ashlands.mara_drifts_told", "value": true}`, `{"effect": "give", "items": {"con_focus_elixir": 1}}`, `{"effect": "give", "gold": 20}`, `{"effect": "take", "items": {"con_fire_bomb": 1}}`.

## 6. Shop (`data/worlds/ashlands/shops.json`)

`hest_stores`: name "Keep Stores Counter", keeper hest_the_quartermaster, buy_mult 1.0, sell_mult 0.5, buys default, requires_flags [], tags [].

| stock item | count | price | derived buy at 1.0 |
| --- | --- | --- | --- |
| con_heal_potion | -1 | -1 | 15 |
| con_focus_elixir | 5 | -1 | 25 |
| con_fire_bomb | 3 | 40 | 40 (fixed) |
| shd_buckler | 1 | -1 | 30 |
| arm_leather | 1 | -1 | 40 |

Every stock `requires_flags` []. Loot: [{loot keep_ring, price 150}] (listed once `ashlands.courtyard_held` holds, the loot entry's own condition; buying it removes it from the gate-watch pool). Prices are starting values; tuning waits for the balance pass.

## 7. Authoring rules

- Seed order: service lines (`ServiceCatalog.save()`), template gold (`QuestCatalog.save()`), manifest, dialogues, the shop, characters, the `hold_the_courtyard` variant, chest gold on the three locations, then `save_world("ashlands")`.
- Read existing entries with `get_own`, merge, upsert under the same id; stop on the first problem; never patch JSON by hand. Register the seed in `data/seeds/README.md` (task DLG1-A).

## 8. Acceptance checks (asset lane, headless after the seed)

1. `ServiceCatalog.validate_all()` == {} and `warnings_all()` empty; `QuestCatalog.warnings_all()` empty.
2. `WorldCatalog.validate_all()` == {}; `DialogueRules.quest_warnings` empty for ashlands, tidewrack and iron_reaches.
3. Planner smoke script after merge: a new Ashlands game has 50 gold; Mara's start node shows "What is this place?", the KNOW choice (visible or locked by the party's best KNOW) and "Farewell" (no offer: the courtyard quest auto-starts).
4. After winning `courtyard_skirmish`, "We finished: Hold the Courtyard" appears at Mara only; confirming with one pick completes it (+30 gold with the template), Mara then offers `cinder_gate_watch` and Tev offers `warden_oath`.
5. With `ashlands.mara_fallen` set: Mara cannot be opened, her marker is absent, the courtyard turn-in and the gate-watch offer appear at Tev.
6. Hest's shop lists potions (15), elixirs (25, 5 left), bombs (40), buckler (30), leather (40); `keep_ring` (150) appears only after `ashlands.courtyard_held`; buying it adds `ashlands:keep_ring` to `loot_claimed`.
7. The bomb trade with no bomb is shown locked and changes nothing; with one bomb it takes it and gives 20 gold.

## Out of scope

Reputation deltas and rep-gated content (REP1-A), Tidewrack or Iron Reaches dialogue and shops, who sets `mara_fallen` / `tev_fallen` in play (a later story pass), new base items, price tuning.

## Open questions

None.
