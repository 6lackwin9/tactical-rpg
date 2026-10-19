# HEAL1: Between-battle healing (replaces the full heal after every battle)

Status: approved (user 2026-10-18 08:08 UTC: all recommendations, Q1 C, Q2 25%, Q3 world map only for 3 supplies, Q4 not in HEAL1, Q5 trigger on HEAL1-B; the defeat rule was changed by the user, see rule 3.3.3). The direction comes from decision world-campaign "DR1 design review answers (user 2026-10-18)" (1)-(3) and `docs/specs/DESIGN_REVIEW1.md` 1.1 and 4.2
Depends on: BAL4-ATTR (done; v4 numbers below), ITEMUSE1-L (merged, save v8), TRAV2-L (merged, supplies)
Lane: logic first (HEAL1-L, schema-first), then a small asset slice (HEAL1-A), UI (HEAL1-U), sim and balance check (HEAL1-S, HEAL1-B)
Touches: see section 9 (per task)

## Goal

Fights should leave a mark without punishing defeat. Today every game battle starts at full HP, because no HP is stored (`BattleBridge` header "Full heal is implicit: M3 stores no HP"; `PartyBuilder.build` sets `c.hp = c.max_hp`; `ExploreRunner.wake` "Fully healed (no HP is stored)"). HEAL1 keeps each hero's HP from one battle to the next, gives a small recovery after each win, and fully restores the party when it rests at a safe place or makes camp for supplies. A defeat always heals fully, but it now costs something: a forced rest that uses up the camp cost in supplies (the party wakes at the last checkpoint), or, when the party does not have enough supplies, a trip back to town (rule 3.3.3). There are no injuries and no penalty for being downed.

Decision history:

- By the user's word (2026-10-18 08:08 UTC: "losing should make you go back to town ... or losing to last checkpoint and using up supplies (forced rest) ... makes supplies useful in case of bad fight apart from travel"), the defeat rule here partly supersedes the "no loss penalty" part of DR1 (2) and the S9 baseline "wake ... fully healed with no further cost". The new defeat cost is supplies or the walk back from town; the reason is that supplies should matter beyond travel.
- HISTORY "Injuries and a defeat cost" (2026-10-18) stays rejected for injuries: no hero loses stats, XP, gold or items, and nobody sits out. Only its "defeat cost" half is revived, in this new form and for the user's new reason.
- Resting and making camp come from the other half of review 4.2 and 5.8 ("Supplies buy rest"), which the user did not reject. Downed heroes get no malus here: they get up after the battle like everyone else (rule 3.2).

## 1. Numbers that shape it (BAL4-ATTR v4, `docs/BALANCE_REPORT.md`)

v4 sim chain: warrior plus two recruits (10 trios), level 5, 10 seeds, the four story fights in order (B1 `courtyard_skirmish`, B2 `pass_ambush`, B3 `wagon_raid`, B4 `ashmaw_stand` for arms or `ashmaw_stand_barred` for gate), objectives and phases on. In the sim, `heal=25` / `50` adds that percent of max HP to heroes still standing; downed heroes stay down; only `full` revives (`tools/sim/sim_campaign.gd` `apply_heal`).

Finale clear rate = chains cleared / runs that reached B4 (B2 falls removed; B1 and B3 never fell):

| Between-battle heal | Party HP after B3 (before heal) | Finale clear, arms (stand) | Finale clear, gate (barred) | Chains cleared, arms / gate (of 100) |
| --- | --- | --- | --- | --- |
| none | 69 | 24 (23 of 96) | 71 (68 of 96) | 23 / 68 |
| 25 | 85 | 43 (42 of 98) | 83 (81 of 98) | 42 / 81 |
| 50 | 89 | 48 (48 of 99) | 87 (86 of 99) | 48 / 86 |
| full (today, M3) | 92 | 55 (55 of 100) | 97 (97 of 100) | 55 / 97 |

Reference: a fresh trio against the stand clears 60 (OBJ1-B, boss band 30-50) and the barred stand 97 (built as a near-certain win, BOSS1-B). The trios that suffer most without healing are brawler+mage (arms chains 0 of 10 with no heal, 9 with full) and priest+brawler (0 / 1).

Two things the sim leaves out, and both matter for any model here:

- The game has more fights than the four story fights: step-counter mobs in the courtyard, gate and camp zones (`ash_scouts`, `ash_straggler`, `ash_raiders`, elite `ash_patrol`), roamers and road fights on `old_road`, `high_road` and `wastes_track`. Those come on top of the chain.
- The party goes back to Warden Keep between story beats (courtyard, gate, oath, steel choice), and B3 starts right after Slag Gate. Only the finale has a long march before it (keep, `high_road`, `wastes_track`, the camp zone, then the stand).

## 2. Options

| | A. Rally only | B. Rest only | C. Rally and rest (recommended) |
| --- | --- | --- | --- |
| Rule | HP carries over; +25% max HP after each win (downed heroes get up at 25%); no rest; only a defeat restores fully | HP carries over; no recovery after a battle (a downed hero gets up at 1 HP); full rest free at rest places (Warden Keep, Slag Gate) and by making camp for supplies | HP carries over; +25% max HP after each win (downed heroes get up at 25%); full rest free at rest places and by making camp for 3 supplies |
| v4 level it maps to | 25 for every stretch (no reset ever) | none within a stretch, full after a rest | 25 within a stretch, full after a rest |
| Expected finale clear (arms / gate) | about 43 / 83 if the march is light; drifts lower over a long campaign | 30-50 / 75-90 without camping (between none and full, set by the march), 55 / 97 if the party camps before the stand | 45-55 / 85-97 without camping (25 column plus revive, only the march stretch counts), 55 / 97 if the party camps before the stand |
| Gameplay | A slow drain with no way to top up: a hurt party has to fight its way back up, and the only full heal is losing (a perverse incentive) | Most tension: sorties become runs, supplies and potions matter, the priest is worth a slot; heroes at 1 HP read like the rejected injury ("bad until the party rests") | Sorties matter (the march to Grakka is the real test), supplies get a positive use, nobody is crippled by being downed; gentler than B |
| Cost | Smallest: stored HP, one rule after battles | Stored HP, rest places (location field), camp action and button | As B plus the rally rule (one constant) |
| Risk | Death spiral over 10+ fights; players lose on purpose to heal | brawler+mage and priest+brawler trios lean on full heal (v4) and suffer most; frustration after a costly win | Can still feel soft if players camp before every fight (the camp cost is the lever) |

Recommendation (chosen by the user 2026-10-18): C with rally 25 and camp cost 3 supplies; the defeat rule is the user's (3.3.3), not the row-A/B/C "defeat restores fully for free" assumed above. The finale lands near the top of the boss band (30-50) without touching Grakka, the trios or the encounters (DR1 (3)); the gate finale stays the easy branch it was built to be. HEAL1-B measures it (section 8) before any BAL5 F2 data change.

## 3. Rules (logic lane, written for option C; A and B drop the parts noted)

### 3.1 Stored HP

1. Each party member stores `hp_lost` (HP missing, 0 = full). Current HP is always `clampi(max_hp - hp_lost, 1, max_hp)`, where `max_hp` comes from the member's current stats and gear (`PartyBuilder.build`). Storing what is missing keeps a wound the same size through a level-up or a gear change, and a stored state never holds a downed hero.
2. Only game battles use it: `GameEncounter` sets each member's start HP from rule 1 after `PartyBuilder.build`. `PartyBuilder.build` itself still returns full HP, so previews, the Dojo, the arena, sims and tests do not change.
3. Guests (OBJ1) and reserve members are never written by a battle. A reserve member keeps its `hp_lost` until a rest.

### 3.2 After a battle (`HealRules.hp_after(hp, max_hp, outcome) -> int`, pure, shared with the sim)

| Outcome | HP of each member who fought (A and C) | Option B |
| --- | --- | --- |
| won | `mini(max_hp, hp + ceili(max_hp * RALLY_PCT / 100.0))`; a downed hero (0) gets up at `ceili(max_hp * RALLY_PCT / 100.0)` | `maxi(hp, 1)` |
| fled (withdraw or round limit, X1) | `maxi(hp, 1)`: no rally, a downed hero gets up at 1 HP | same |
| lost | every roster member is fully restored (`hp_lost` 0); the defeat cost of rule 3.3.3 applies | same |

`RALLY_PCT` = 25. The rally comes only after a win, so fleeing is never a way to heal. XP is unchanged (downed members still get full XP, M3).

### 3.3 Rest (full restore of every roster member, `HealRules.rest(party) -> int` = HP restored in total)

1. Rest place: a location with `rest: true` (field in section 4.2). Entering one (`ExploreRunner.enter`: arriving by travel, through a region exit, or waking after a defeat) rests the party and records the location as `explore.last_rest` (qualified id, set even when nobody is hurt). Rest is free, and if nobody is hurt no HP changes. Resting is idempotent, so re-entering on load is harmless.
2. Make camp: `HealRules.make_camp(gs) -> {ok, errors, spent}` spends `CAMP_COST` = 3 supplies and rests the party. It is offered on the world map only, with the party at a node and no trip running. It is refused, with the state unchanged, when nobody is hurt ("Everyone is rested."), when supplies are below 3 ("Making camp needs 3 supplies."), or while a battle, dialogue or trip is running. Making camp does not advance `gs.time` and rolls no fight. It autosaves like an arrival.
3. Defeat (user 2026-10-18 08:08). `ExploreRunner.wake(gs, at)` stays the single defeat entry: its three callers are `ExploreRunner.after_fight`, `TravelRunner` road fights and `DialogueRunner.after_fight`. It calls `HealRules.on_defeat(gs, checkpoint)`, where `checkpoint` is today's wake target: `at` or the current location when that location has no map or a SAFE spawn, else `explore.last_safe`, else "" (the start location, `begin`). In order:
   - (a) The checkpoint is itself a rest place (`rest: true`): the party is fully healed there for free and spends no supplies (mode `rest_place`).
   - (b) Otherwise, with supplies >= `CAMP_COST` (3): spend 3 as a forced rest, fully heal every roster member, wake at the checkpoint (mode `forced_rest`).
   - (c) Otherwise (supplies 0-2): fully heal every roster member, keep the supplies, and send the party back to town, meaning the last rest place it entered (`explore.last_rest`, 4.1). When that is empty or no longer resolves, use the start location through `begin(gs)`, the same as today's last fallback (mode `back_to_town`).

   A short fall (1-2 supplies) is kept, not spent. The walk back is the cost already, and the party needs those supplies for the road home. Spending them would put it at 0, where road fights are 1.5x as likely (TRAV2), on top of the walk.

   `on_defeat` returns `{mode, spent, to}` and `wake` adds the event `{"kind": "defeat", "mode", "spent", "location": to}` to its events. XP, gold, items, flags and quests are untouched, as today.

Why 3 supplies: one leg costs 2-4 (`old_road` 2, `high_road` 4, `wastes_track` 3), Hest sells supplies at 2 gold (6 gold per camp, under one potion at 15), and the party starts with 10. The march to Grakka costs 7, so one camp on the way is affordable, but camping before every fight is not.

Option A drops 3.3.1-3.3.2. Option B keeps 3.3 and drops the rally.

### 3.4 Order of the write-back

`BattleScreen._write_back` (runs on Continue for every game fight: map, road and dialogue) calls `GameEncounter.write_back_hp(outcome)` next to `write_back_pouches()`. For won and fled it applies 3.2 to each member who fought; for lost it writes nothing, because `ExploreRunner.wake` (3.3.3) heals the roster and applies the defeat cost after the outcome is settled. The dev outcome buttons have no combatants: won and fled leave HP as stored, and lost goes through `wake` like a real defeat, with the same supplies or back-to-town cost. The results popup's Replay plays the session's own round snapshots (`session.select_round(1)`), so it shows the battle's start HP and does not write again (`_settled`).

### 3.5 Edge cases

| Case | Rule |
| --- | --- |
| Recruit joins (`recruit` effect, RECRUIT1) | New `UnitState` with `hp_lost` 0: joins at full HP whatever the party's state |
| New game, old saves | `hp_lost` 0 everywhere: full HP (migration 4.1) |
| Level-up or gear change between fights | `hp_lost` unchanged; current HP follows the new max (rule 3.1.1, at least 1) |
| Downed heroes after a win | Get up at 25% (C/A) or 1 HP (B); none sits out the next fight |
| Hold / protect / slay wins with downed heroes | Same as any win |
| Party swap (hurt hero to reserve) | Keeps its `hp_lost`; healed only by a rest or a defeat |
| Autoplay (AUTO1) | No change. The 30% safety stop fires on crossings only (`CombatAutoplay._hp_at_start`), so a hero who starts below 30% stops autoplay only if downed; party tactics still never drink potions (ITEMUSE1 rule 8) |
| Potions | Used in battle only (ITEMUSE1); their healing now carries over. No out-of-battle drinking (Q4) |
| Defeat exploit (lose on purpose to heal) | No longer free: a defeat costs the same 3 supplies as making camp (and loses the party's place in the location), or, with fewer than 3, the walk back from town; it is never cheaper than camping |
| Defeat with supplies exactly 3 | Spends all 3 (forced rest); the party is at 0 supplies, so road fights are 1.5x as likely until it buys or finds more |
| Defeat in another world (`connects`, e.g. a road fight from Slag Gate) | Same rule; `last_rest` is a qualified id (for example `iron_reaches:slag_gate`) and `enter` handles the crossing as today (`home_world`) |
| Defeat before any rest place was entered (old save, empty `last_rest`) | Back to town = the start location (`begin`) |
| Playtest recordings made before HEAL1 | Their party loads at full HP (missing `hp_lost`), which matches how they were recorded |

## 4. Schema and save contract (schema-first: yes; commit 1 of HEAL1-L releases HEAL1-A)

### 4.1 `UnitState.hp_lost`, `explore.last_rest` and save version 9

| Name | Value | Notes |
| --- | --- | --- |
| `var hp_lost: int` | 0 | HP missing; `to_dict` key `hp_lost`; `from_dict`: missing reads 0, a non-whole value loads as -1; `duplicate_state` and `equals` include it |
| `explore.last_rest: String` | "" | qualified id of the last rest place entered (rule 3.3.1); added to `GameState.EXPLORE_KEYS` and `default_explore()`; "" = the start location (`begin`); an id that no longer resolves is read as "" (no load error) |

`UnitState.validate`: `hp_lost < 0` gives the error "hp_lost must be 0 or more.". There is no upper bound (max HP is derived; rule 3.1.1 clamps). `SaveGame.VERSION` 9; step `_v8_to_v9` sets `hp_lost: 0` on every party member and `explore.last_rest: ""` when `explore` is an object, nothing else. `tools/gen_save_fixtures.gd`: the played state gets one member with `hp_lost` 37 and `explore.last_rest` `ashlands:warden_keep`; the new `scripts/tests/fixtures/saves/save_v9.json` is committed, and older fixtures must come out byte-identical.

### 4.2 World location field `rest`

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| rest | bool | false | bool (schema type check). Help: "The party rests here on arrival: everyone's HP is restored." |

An explicit field, not the location `kind`: Slag Gate (`iron_reaches`) has kind `gate`, the same as the Cinder Gate battlefield. Inherited and overridden like every location field.

### 4.3 `HealRules` (new `scripts/progression/heal_rules.gd`, `RefCounted`, static)

| Member | Contract |
| --- | --- |
| `RALLY_PCT: int = 25` | rule 3.2 (B: not used) |
| `CAMP_COST: int = 3` | rule 3.3.2 |
| `hp_after(hp, max_hp, outcome) -> int` | rule 3.2 for one hero; `outcome` won / fled (lost returns `max_hp`) |
| `current_hp(state: UnitState) -> Dictionary` | `{hp, max}` via `PartyPreview.derived` max HP and rule 3.1.1 (UI read-out) |
| `hurt(party) -> bool` | any roster member with `hp_lost > 0` |
| `rest(party) -> int` | rule 3.3; total HP restored (0 = nobody hurt) |
| `can_camp(gs) -> String` | "" or the refusal text (3.3.2) |
| `make_camp(gs) -> Dictionary` | `{ok, errors, spent}`; all-or-nothing |
| `on_defeat(gs, checkpoint: String) -> Dictionary` | rule 3.3.3: heals the roster, spends 0 or 3 supplies, returns `{mode (rest_place / forced_rest / back_to_town), spent, to (qualified location id, "" = start location)}`; `ExploreRunner.wake` enters `to` |

Events: `ExploreRunner.enter` adds `{"kind": "rested", "location": qid, "restored": n}` when `rest` restored HP above 0; `ExploreRunner.wake` adds `{"kind": "defeat", "mode", "spent", "location"}` (rule 3.3.3).

## 5. UI touchpoints (HEAL1-U)

| Where | What |
| --- | --- |
| Battle results popup (`BattleResultsModel.rows`, `battle_results_view.gd`) | per hero `HP 142 / 210` after the rally, with "(+53 rallied)" on a win; on a fled battle with someone hurt, one line "Rest at a safe place or make camp to recover fully."; on a loss the outcome note (`BattleResultsModel` "lost", today "The party woke at the last safe place, fully healed. No XP.") says which defeat mode happened, from the `defeat` event passed in the results args ({checkpoint} and {town} are location names): `forced_rest` "Forced rest: -3 supplies. The party wakes at {checkpoint}, fully healed. No XP."; `back_to_town` "Not enough supplies to make camp. Back to {town}, fully healed. No XP."; `rest_place` "The party wakes at {town}, fully healed. No XP." |
| Hub popup party rows (`HubModel.party`) | `HP cur / max` per member, hurt rows tinted with the existing warning colour |
| World map side panel (`world_map_view.gd`) | button "Make camp (3 supplies)" under the supplies readout, disabled with the refusal text as its tooltip (3.3.2); note "The party makes camp. Everyone is rested (supplies -3)." |
| Region map log (`region_map_view.gd` `_note`) | on a `rested` event: "The party rests at Warden Keep. Everyone is rested." |
| Battle cards | no change (they already show start HP below max) |

1280x720 at all four text sizes for the results popup and the world map panel.

## 6. Assets (HEAL1-A, `WorldCatalog.upsert` through a seed `data/seeds/heal1.gd`)

| World | Location | Field | Value |
| --- | --- | --- | --- |
| ashlands | warden_keep | rest | true |
| iron_reaches | slag_gate | rest | true |

No other location rests (`ruined_courtyard`, `cinder_gate`, `ashmaw_camp` stay false). No price, supply or encounter change (DR1 (3)).

## 7. Acceptance checks (rule-derived, headless)

1. A won battle where a 200-HP hero ends at 80 stores `hp_lost` 70 (80 + 50 = 130); the next battle starts that hero at 130.
2. A hero downed in a won battle stores `hp_lost` 150 (starts the next battle at 50); downed in a fled battle, `hp_lost` 199 (1 HP); a standing hero at 80 in a fled battle stays at 80.
3. A lost battle leaves every roster member at `hp_lost` 0, including the reserve; the dev "won" button changes nothing; the dev "lost" button applies the same defeat rule as check 13.
13. Defeat in the `cinder_gate` ambush (safe spawn) with 5 supplies: mode `forced_rest`, supplies 2, the party wakes at `ashlands:cinder_gate`, nobody hurt. The same defeat with 2 supplies: mode `back_to_town`, supplies stay 2, the party is at `explore.last_rest` (`ashlands:warden_keep`), nobody hurt. With 0 supplies and `last_rest` "": the start location.
14. A road-fight defeat whose wake target is a rest place (for example `from` = `warden_keep`) spends no supplies (mode `rest_place`). Losing the `wagon_raid` with 1 supply after visiting Slag Gate sends the party to `iron_reaches:slag_gate`.
15. Entering `warden_keep` sets `explore.last_rest` even with nobody hurt; entering `cinder_gate` does not change it. A v8 save loads with `last_rest` "".
16. Every `wake` caller (`ExploreRunner.after_fight`, `TravelRunner` road fight, `DialogueRunner.after_fight`) applies checks 13-14; XP, gold, items, flags and quests are unchanged by a defeat.
4. A guest and a reserve member are never written by a battle.
5. Entering `warden_keep` with a hurt party gives a `rested` event and `hp_lost` 0 for all; entering with nobody hurt gives no event; entering `cinder_gate` changes nothing.
6. `make_camp` with 5 supplies and a hurt party leaves 2 supplies and nobody hurt; with 2 supplies, or with nobody hurt, it is refused and the state is unchanged; `gs.time` is unchanged.
7. A level-up that raises max HP from 200 to 210 with `hp_lost` 70 starts the hero at 140; a gear change that drops max HP to 60 with `hp_lost` 70 starts at 1.
8. A recruited hero joins with `hp_lost` 0; `PartyBuilder.build` and `PartyPreview.derived` still return full HP.
9. A v8 save loads as v9 with `hp_lost` 0 on every member and `explore.last_rest` ""; a negative or non-whole `hp_lost` is refused by `validate`; `save_v9.json` round-trips.
10. The results Replay of a won battle shows round 1 at the start HP and leaves the stored HP as written once.
11. Autoplay on a hero who starts at 20% HP does not safety-stop in round 1 unless that hero goes down.
12. Every existing suite and sim number is unchanged (no game battle in them reads `hp_lost` except the new checks).

Tests: new `scripts/tests/heal_test_suite.gd` (checks 1-8, 10-11, 13-16) + one line in `suite_registry.gd`; `save_test_suite.gd` (check 9).

## 8. Balance check (HEAL1-S sim, HEAL1-B run)

Sim keys in campaign mode (`tools/sim/sim_args.gd`, `sim_campaign.gd`):

- `heal=rally`: calls `HealRules.hp_after(hp, max, "won")` per hero (downed heroes get up). Existing `none`, `full` and `<pct>` keep their current numbers.
- `rest=<b,b>`: 1-based story battles that start after a full rest (default none; HEAL1-B uses `rest=3`, Slag Gate before `wagon_raid`).
- `extra=<b>:<enc>[,...]`: an extra fight before story battle b to model the march (HEAL1-B: `extra=4:ash_raiders`; a loss in it ends the run as a fall at b).
- The defeat cost (rule 3.3.3) is not modelled: the campaign sim ends a run at its first loss (`fell_at`) and never plays on after a defeat, so supplies and back-to-town do not change its numbers. No sim key is needed for it.

HEAL1-B runs `campaign 5 10 heal=rally rest=3 extra=4:ash_raiders` (all trios, both steels) and reports the finale clear rate as in section 1. Pass: arms 30-55 and gate 75 or higher. Above 55, the next lever is BAL5 F2 (guard at party level, inside ASHPACE1). Below 30, the first lever is `RALLY_PCT` 25 -> 50 (data-free, one constant), not encounter data. Run it after ASHPACE1 lands if that is near, because ASHPACE1 changes the filler fights.

## 9. Proposed task split

| Id | Title | Lane (thread) | Depends on | Touches |
| --- | --- | --- | --- | --- |
| HEAL1-L | Stored HP, rally, rest places, make camp, defeat cost (forced rest or back to town) (rules only); commit 1 = schema slice (hp_lost, explore.last_rest, save v9, location `rest`) | logic (Gameplay) | HEAL1 spec approved; not parallel with BREPORT1-L on `battle_screen.gd` | `scripts/progression/{unit_state.gd, heal_rules.gd (new)}`, `scripts/game_state/{game_state,save_game,save_migrations}.gd` (`EXPLORE_KEYS`, `default_explore`), `tools/gen_save_fixtures.gd` + new `scripts/tests/fixtures/saves/save_v9.json`, `scripts/world/world_schema.gd` (`_location_fields`), `scripts/exploration/{game_encounter,explore_runner}.gd` (`enter`, `wake`), `scripts/exploration/battle_bridge.gd` (header comment only), `scripts/ui/overworld/battle_screen.gd` (`_write_back`, one call), new `scripts/tests/heal_test_suite.gd`, `suite_registry.gd`, `save_test_suite.gd`, `scripts/progression/README.md` |
| HEAL1-A | `rest: true` on warden_keep and slag_gate | asset (Assets & Audio) | HEAL1-L commit 1 merged | `data/worlds/ashlands/locations.json`, `data/worlds/iron_reaches/locations.json` via new `data/seeds/heal1.gd` |
| HEAL1-U | HP on results and hub, defeat-mode line on the results popup, Make camp button, rest and camp notes (1280x720, four text sizes) | logic (UI/UX) | HEAL1-L, UIR-2, UIR-4; after BREPORT1-U (same results view) | `scripts/ui/menus/{battle_results_model,battle_results_view,hub_model,hub_view}.gd`, `scripts/ui/overworld/{world_map_view,region_map_view}.gd` (pass the `defeat` event into the results args), `scripts/ui/menus/menu_router.gd` (results args), the results and menus suites |
| HEAL1-S | Sim keys `heal=rally`, `rest=`, `extra=` | logic (Balance) | HEAL1-L, BAL5-SIM (same files) | `tools/sim/{sim_args,sim_campaign}.gd`, `scripts/tests/bal4_campaign_test_suite.gd` |
| HEAL1-B | Campaign run with HEAL1 rules; finale clear vs section 8 (report only) | balance (Balance) | HEAL1-S, HEAL1-A on main (ASHPACE1-A if close) | `docs/BALANCE_REPORT.md` |

Docs delta for the planner: GAME_BRIEF "Progression and saves" (hp_lost, explore.last_rest, HealRules, save v9) and "Region maps (EXP1)" (defeat rule), plus the `BattleBridge` / `wake` comments. Partly superseded by the HEAL1 decision row: progression-save "M3" ("full heal after battle"); world-campaign "BATTLE1-L battle link" ("no HP carry-over", "defeat ... with no further cost"), "S9-S11 baseline answers" ("defeat = wake ... fully healed") and "DR1 design review answers" (2) ("no loss penalty"; injuries stay rejected). README progression and exploration paragraphs.

## Out of scope

Injuries, stat maluses, XP, gold or item loss, or a downed hero sitting out (HISTORY 2026-10-18); any defeat cost beyond rule 3.3.3; inn prices or gold for resting; out-of-battle potion use (Q4); priest or skill healing outside battle; rest that advances the world clock; reserve members recovering over time; changes to Grakka, tiers, encounters, potion strength or supply prices (DR1 (3), BAL5 owns tuning); KEEP1's infirmary (a later upgrade can reuse `HealRules.rest`); difficulty options (DIFF1 may later scale `RALLY_PCT`).

## Open questions

All answered (user 2026-10-18 08:08 UTC, recommendations taken). The defeat rule was added by the user, not asked: see 3.3.3. Kept for the record:

1. Healing model? A rally only / B rest only / **C rally and rest (recommended)**: the finale lands near the boss band without data changes and supplies get a use; B is harsher and close to the rejected injury feel; A has no top-up and makes losing the best heal.
2. Rally size? **25% (recommended)**, the v4 column that puts the arms finale at 43-55 / 50% (48-55, softer) / 0% (that is option B).
3. Make camp: **world map only, 3 supplies, full rest (recommended)** / anywhere outside a danger zone, including region maps (camping right before the stand makes the finale the full column, 55) / no camping, rest places only (supplies stay travel-only).
4. Potions outside battle: **not in HEAL1 (recommended)**, pack them and drink in block 1 (1 AP), which keeps one item path / a "Drink" action on the hub or Gear tab now (more UI; potions become the cheap camp).
5. BAL5 F2 trigger: **read HEAL1-B's finale number instead of the fresh-trio probe (60) (recommended)**, so the guard at party level only lands if the finale is still above 55 with attrition / keep F2 on the fresh probe (it fires now and the finale gets harder twice).
