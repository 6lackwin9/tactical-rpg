# REP1-A: Sample reputation deltas, a rep-gated choice and a rep-gated quest (S11-B)

Status: approved (user 2026-10-17) (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)")
Depends on: REP1-L commit 1 merged; DLG1-A merged (Hest's tree and shop); EXP1-A merged (encounter `ash_raiders`)
Lane: asset (one branch, seed `data/seeds/rep1a.gd`)
Touches: `data/worlds/ashlands/{quests,dialogues}.json`, `data/worlds/iron_reaches/quests.json` (via `WorldCatalog.upsert` + `save_world`), `data/seeds/rep1a.gd`, `data/seeds/README.md`. No code, no tests.

## Goal

Give the sample quests reputation consequences, gate one dialogue choice and one new quest on Ember Wardens standing, and so show the shop price band moving as the party helps the Wardens.

## 1. Quest `reputation` (merge into the existing entries)

| Quest | reputation |
| --- | --- |
| ashlands:hold_the_courtyard | {ember_wardens: 10} |
| ashlands:cinder_gate_watch | {ember_wardens: 10, ashen_horde: -10} |
| ashlands:warden_oath | {ember_wardens: 15} |
| iron_reaches:iron_for_embers | {forge_guild: 10} |

Running total for the Wardens: 10 after the courtyard (friendly), 20 after the gate watch, 35 after the oath.

## 2. Rep-gated dialogue choice (`hest_talk`)

Append to node `hello` as choice 3: text "Any spare stores for friends of the Wardens?"; requires [{"key": "rep.ashlands.ember_wardens", "op": ">=", "value": 10}]; unless [{"key": "ashlands.hest_spare_given", "value": true}]; show_locked true; effects [{"effect": "set_flag", "key": "ashlands.hest_spare_given", "value": true}, {"effect": "give", "items": {"con_heal_potion": 2}}]; to `spare`.

New node `spare`: text "\"For you? Two, and don't tell the others.\""; next `hello`.

## 3. Rep-gated quest (`ashlands:ember_patrol`, new)

| Field | Value |
| --- | --- |
| name | Sweep the Ruins |
| description | Hest wants the ruins swept of raiders before the next supply run. |
| giver | hest_the_quartermaster |
| turn_in | "" (= giver) |
| faction | ember_wardens |
| region | cinder_pass |
| requires | [] |
| template | clear_site |
| steps | [{text "Break a raiding party in the ruins", encounter ash_raiders}] |
| requires_flags | [{"key": "rep.ashlands.ember_wardens", "op": ">=", "value": 30}] |
| complete_flags | [{"key": "ashlands.ruins_swept", "value": true}] |
| auto_start | false |
| reward (text) | Hest's gratitude and a pick from the stores. |
| links | {related_characters [hest_the_quartermaster], related_locations [ruined_courtyard, cinder_gate], related_factions [ember_wardens, ashen_horde]} |
| variants | [] |
| loot_pool | [keep_stores] |
| loot_picks | 1 |
| reputation | {ember_wardens: 5} |
| tags | [] |

`ash_raiders` comes up from the step counter (both ruin bindings) and the high road, so the objective is met by an ordinary random fight. Budget: clear_site (1 potion, 40 gold) + keep_stores (3 potions) = 4 items, inside the soft caps.

## 4. Authoring rules

- Seed order: the four quest merges (`get_own`, merge `reputation`, upsert under the same id), the new quest, then Hest's tree (read, add the choice and node, upsert), then `save_world("ashlands")` and `save_world("iron_reaches")`.
- Stop on the first problem; never patch JSON by hand. Register the seed in `data/seeds/README.md` (task REP1-A).

## 5. Acceptance checks (asset lane, headless after the seed)

1. `WorldCatalog.validate_all()` == {}; `QuestRules.reward_warnings`, `QuestRules.link_warnings` and `DialogueRules.quest_warnings` empty for ashlands and iron_reaches; no `rep.` key warning.
2. Planner smoke script after merge: a new game has Ember Wardens standing 0 (neutral); Hest's spare-stores choice is shown locked; `ember_patrol` is locked.
3. Completing `hold_the_courtyard` sets `rep.ashlands.ember_wardens` to 10: the spare-stores choice opens once (2 potions), and Hest's derived prices fall to potion 15, elixir 24, buckler 29, leather 38 (friendly, 95 percent, rounded up); the fixed-price bombs stay 40.
4. Completing `cinder_gate_watch` gives Wardens 20 and `rep.ashlands.ashen_horde` -10; completing `warden_oath` gives 35 and makes `ember_patrol` available at Hest.
5. Winning an `ash_raiders` fight with `ember_patrol` active makes it ready; completing it gives 40 and sets `ashlands.ruins_swept`.

## Out of scope

Reputation in Tidewrack or Iron Reaches dialogue, hostile-faction content, kill-based reputation, price tuning.

## Open questions

None.
