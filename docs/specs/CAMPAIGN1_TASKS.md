# CAMPAIGN1 tasks: tests, parity plan, task split, data, open questions

Status: proposed (2026-10-25); the rules are in [CAMPAIGN1.md](CAMPAIGN1.md) (rule ids such as M5 or R4 refer to
it)

## 1. Tests (rule-derived; scratch data folders, never the shipped data)

| Task | Test | Expectation from |
| --- | --- | --- |
| E | write and read a keyed save `act1_ab12cd34`; a slot listing | same envelope back; not among slots 0-5; a bad key refused (F5) |
| S | a meter whose bands do not start at `min`, a rule `at` naming no band, a countdown without `days`, a `unit` scope row | one error each (11.2, M9) |
| S | a checkpoint with both `quest` and `met_when`; a trigger with `encounter` and `on: day` | one error each |
| S | a dialogue choice `meter_add` naming an unknown meter; a condition on `meter.nope` | one error; one error (11.4) |
| S | the Ashlands data | no new error or warning (checks report unchanged) |
| M | a world without `campaign`: travel, rest, battles, dialogue | results, states and saves equal to before (X1, X3) |
| M | `set_flag meter.alert` | refused "reserved key" in every world (11.4) |
| M | gauge 25 + `meter_add` 10 | value 35, band `suspicious`, one `meter_warning` (M2, M5, W1) |
| M | gauge 95 + 10 | clamped 100; the Breach rule fires once; run over with its card (M4, F1) |
| M | the same in act 2 with only the act-1 rule | no run over (M5 `acts`) |
| M | act 1 to 2 with value 12 and floor 30 | 30; then `meter_add` -20 gives 30 (M4) |
| M | a won battle tagged `loud` with input won +8 | +8; lost with no lost input: +0 (M3) |
| M | a rule whose effect re-crosses another rule | both fire once; a self-feeding pair is a check error (X4) |
| M | conditions `meter.alert >= 31`, `meter.alert.band == "lockdown"`, `campaign.act == 1` on a route gate | open or shut as the value moves (X5) |
| M | save round trip with meters, fired rules, pending items | same state; `over` not written; `save::write` while over refused (F3) |
| M | a version-12 save of a campaign world | loads with a fresh campaign block (11.5) |
| C | `clock_start` 42 at time 0, a 3-leg trip of 1 day each | days left 39; the `days` tick ran 3 times (K1, K2) |
| C | a leg from 8 to 6 days left with rules at 7 | the 7-day warning once (K5) |
| C | soft rule at 0 with `restart_days` 14 | effects ran, flag +1, vignette queued, 14 days left (K4) |
| C | hard rule at 0 | run over, the trip stops with status `over` (K4, F1) |
| C | `clock_set` 10 with a label; `clock_stop` | 10 days left and the HUD label; reads `max`, `running` false (K3) |
| C | checkpoint deadline 14, quest done on day 14 | met; on day 15 not done: missed once, `end_run` card (C1, C3) |
| C | the same with `go_on` | effects ran, no run over (C3) |
| C | warn days 7, 3, 1 crossed by one trip | one warning each, in order (C2) |
| C | `per_day` 2, a 3-day leg | 6 upkeep spent; at 0 never refused (P2) |
| C | `at_zero.road_pct` 100 at 0 upkeep; Lockdown `road_pct` 150 | chance unchanged; then x1.5 capped 100 (P3, M7) |
| C | `battle_mods` at 0 upkeep with `start_ap` -2 | party AP -2 at round 1, not below 0; with upkeep 1: 0 (P3) |
| C | `rest` with days 1, cost 1, where `rest_places` | at the hub: full HP, wounded cleared (ROSTER1 hook), day +1, 1 upkeep; on a street node: refused (R1, R2) |
| C | `on_enter` false; enter the hub hurt | no free heal (R1) |
| C | lost battle, `defeat` retreat to the hub, days 1 | at the hub, encounter not done, day +1, upkeep -1, ROSTER1 loss applied, on all three paths (R4) |
| T | trigger `on battle` enc X won with effects and a vignette | effects applied, vignette queued and saved (T1, T2) |
| T | `once` trigger twice true; repeatable trigger true, false, true | fires once; fires twice (T1) |
| T | `when roster.living < 1` with `end` fail | run over (F2) |
| T | `preview_battle` with alert 95 and a loud fight | lists the Breach end; with alert 50: empty (W3) |
| T | `preview_days` passing an `end_run` deadline | lists the bad end (W3) |
| U | the region and world map HUD in the Ashlands | today's ids, texts and order (hud.rs tests unchanged, W4) |
| U | a campaign world HUD | Scrap, Tallow, then the visible meters by order with units and tones (W4) |
| U | run over | only the run-over screen; its three buttons; "Load the act's start" loads the keyed save (F4, F5) |
| SIM | `sim deadlines` on a scratch world with known routes | best and worst days per checkpoint as computed by hand (section 4.3) |

## 2. Parity plan (the Ashlands "classic" stays byte-identical)

- Every new manifest field defaults to today's rule; the Ashlands, Iron Reaches and Tidewrack set none, so X1 keeps
  every new path inert. No recorded trace, golden or expectation is edited.
- `journey`, `world_traces`, `saves`, `trace_parity`, `phasefold`, `checks_parity`, `sim_parity` and the host
  `replay` run unchanged after each task: no new result key, state key or view key without a campaign (X3); the
  travel arithmetic for road chances is today's when no modifier applies; `wake` stays the default defeat.
- `data_source` compares Godot's schema whole: CAMPAIGN1-S strips the three new kinds, the four manifest fields and
  the shop field `supply_steps` (later) through `AFTER_GODOT`, named in the commit and a HISTORY row (as DERIVE1-S).
- The reserved prefixes change `set_flag` only for keys no shipped data or Godot journey uses (grep 2026-10-25).
- Saves: version 13 (next free) adds nothing; every Godot save fixture still loads to the same state.

## 3. Tasks (slice = needed before the vertical slice)

Paths (KIT1_TASKS 5): `rpgkit_*` under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`;
`schema.json` = `kits/rpg/schema.json`; `check/` = `rpgkit_content/src/check/`; `campaign/`, `quest.rs`, `shop.rs`
in `rpgkit_world/src/`.

| Id | Title | Layer | Lane | Depends | Slice | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| CAMPAIGN1-E | Keyed saves: write, read, exists, peek by key outside the slot range (F5) | engine | logic | - | yes | `engine/crates/aegis_save/src/lib.rs`, `engine/crates/aegis_save/tests/keyed.rs` (new) |
| CAMPAIGN1-S | Schema and checks: manifest fields, kinds `meters`, `checkpoints`, `triggers`, the new effect kinds and read keys in the effect and condition checkers, the Forge viewer list, the schema parity strip (11.1-11.4) | kit | logic | - | yes | `kits/rpg/schema.json`, `rpgkit_content/src/{source,store}.rs`, `rpgkit_content/src/check/{world,dialogue,campaign (new)}.rs`, `emberwake_gui/src/forge.rs`, `emberwake_world/tests/data_source.rs` |
| CAMPAIGN1-M | Meter runtime: state and save step, `tick`, gauges, bands, rules, inputs, floors, the act, effects, read keys, run over (X1-X5, M1-M8, F1, F3, 11.3-11.5) | kit | logic | S | yes | `rpgkit_world/src/campaign/{mod,meter}.rs` (new), `rpgkit_world/src/{lib,state,save,quest,dialogue,battle}.rs`, `rpgkit_world/tests/campaign.rs` (new, scratch data), `emberwake_world/tests/saves.rs` (a v12 campaign case) |
| CAMPAIGN1-C | Days, countdowns, checkpoints, purse, rest, defeat: the `days` tick in travel, rest and retreat, the clock effects, `per_day`, `at_zero`, band modifiers on road fights, `battle_mods` (K1-K5, C1-C3, P1-P3, R1-R4) | kit | logic | M | yes | `rpgkit_world/src/campaign/{clock,checkpoint,purse}.rs` (new), `rpgkit_world/src/{travel,progression,explore}.rs`, `rpgkit_world/tests/campaign.rs` |
| CAMPAIGN1-T | Triggers, the pending queue, previews; `dialogue::apply_effects` as a free function shared with triggers (T1-T3, F2, W3) | kit | logic | C | yes | `rpgkit_world/src/campaign/{trigger,preview}.rs` (new), `rpgkit_world/src/{dialogue,explore}.rs`, `rpgkit_world/tests/campaign.rs` |
| CAMPAIGN1-U | Screens: HUD counters and meter chips, notices, fatal asks, the run-over screen, the act-start save, playing the pending queue, the start-AP penalty in the battle bridge, the Travel tip (F4, F5, W1-W6, P3) | game | logic | T, E; STORYUI1 card and vignette player (else plain cards) | yes | `emberwake_gui/src/{hud,worldmap,region,battle,screens,menus}.rs`, `emberwake_gui/src/runover.rs` (new) |
| CAMPAIGN1-SIM | `aegis sim deadlines`: best and worst finish per checkpoint (section 4.3) | kit | logic | S | yes (the balance lane checks the slice's deadlines) | `rpgkit_sim/src/{args,lib}.rs`, `rpgkit_sim/src/deadlines.rs` (new), `rpgkit_sim/tests/deadlines.rs` (new, scratch world) |
| CAMPAIGN1-D | The `sunk_ward` campaign rows (section 4) | game | asset | S; the `sunk_ward` manifest and quests (SUNKWARD1's first step) | yes | `games/emberwake/data/worlds/sunk_ward/{world,meters,checkpoints,triggers,dialogues,encounters}.json` |
| CAMPAIGN1-TM | Travel modes per leg: stealth (+1 day, lower road chance) and forced march (+Alert per leg, a roamer fight at Alert 50+); the mode toggle on the world map (CMQ2) | kit | logic | C, U | CMQ2 | `rpgkit_world/src/travel.rs`, `campaign/meter.rs` (`on: leg` mode filter), `schema.json` (route fields), `emberwake_gui/src/worldmap.rs` |
| CAMPAIGN1-P | Shop `supply_steps` (P4) | kit | logic | S | later (Act 3) | `schema.json`, `rpgkit_world/src/shop.rs`, `check/shop.rs` |
| CAMPAIGN1-DOC | Doctrine: `scope` unit, leans, play inputs, favour conditions (M9) | kit | logic | M; ROSTER1 member records | later (Act 2) | `campaign/meter.rs`, `quest.rs`, `check/campaign.rs`, `emberwake_gui/src/party.rs` |

Order: E and S in parallel; M, C, T in a row (shared `campaign/mod.rs` and `tests/campaign.rs`); U last; SIM after S;
D after S and the manifest. Save bumps run one at a time (CAMPAIGN1-M, ROSTER1, RULESET1-D, ENCOUNTER2-W). Hub-file
overlaps: S with RULESET1-R and DERIVE1-S (`schema.json`, `source.rs`); M and C with ENCOUNTER2-W (`rpgkit_world/src/
{battle,state,save,explore,dialogue,travel}.rs`); U with ENCOUNTER2-U and STAGE1-L (`gui/battle.rs`, one bridge hunk).

## 4. Data (asset lane, `tools/fc.sh content upsert <kind> @file.json world=sunk_ward`, then `content check`)

Numbers marked (B) are placeholders for the balance lane; texts marked (T) are placeholders for the story lane
(story guide G-20, G-23). Card and vignette ids are STORYUI1's rows; quest ids are SUNKWARD1's.

### 4.1 Manifest fields (`world` row of `sunk_ward`)

| Field | Value |
| --- | --- |
| `campaign` | `{"start_act": 0, "act_saves": true}` |
| `purse.currency` | name "Scrap", icon `ui/hud/scrap.png`, tip "Salvaged iron: trade, the forge, bribes." |
| `purse.upkeep` | name "Warded Tallow", short "Tallow", icon `ui/hud/tallow.png`, tip "1 a day on the move or at rest. Without it every fight starts at -2 AP.", `per_day` 1, `at_zero` `{"road_pct": 100, "start_ap": -2}` |
| `start_gold`, `start_supplies` | 60 Scrap (B), 10 Tallow (B) |
| `rest` | `{"days": 1, "cost": 1, "where": "rest_places", "on_enter": false, "needs_upkeep": false}` (CMQ5) |
| `defeat` | `{"mode": "retreat", "to": "sluice_undercroft", "days": 1}` |

The hub location has `rest: true`; the Dry Hollows bunk marker's dialogue offers "Rest a day (1 Tallow)" with the
effect `rest`. Routes set `time` 1.

### 4.2 Meters

| Row | Values |
| --- | --- |
| `alert` | name "Inquisition Alert"; gauge 0-100, start 0, unit `%`, visible false; hud `{"short": "Alert", "icon": "ui/hud/alert.png", "order": 2}` |
| `alert` bands | `guarded` "Guarded" 0 calm; `suspicious` "Suspicious" 31 warn; `lockdown` "Lockdown" 71 danger; `breach` "Breach" 100 fatal |
| `alert` rules | `a_suspicious` at suspicious, warn (T) "The Inquisition is Suspicious: gates are watched. Quiet jobs and bribes at Tess lower Alert."; `a_lockdown` at lockdown, warn (T) "Lockdown: hunter packs roam the main roads."; `a_last` at 90, acts [1], warn (T) "One more loud job and they find the grate. The run would end."; `a_breach_1` at breach, acts [1], effects `[{"effect": "end", "kind": "fail", "card": "extinguished_breach"}]` |
| `alert` inputs | battle tags [loud] won +8 (B); loud lost or fled +4 (B); quiet won +2 (B) |
| `alert` modifiers, floors | lockdown `road_pct` 150 (B); floor from act 2: 30 |
| `doom_clock` | name "The Red Tithing" (CMQ9); countdown, max 99, unit `days`, visible false; hud `{"short": "Tithing", "icon": "ui/hud/doom_clock.png", "order": 1}`; bands `due` 0 danger, `near` 1 warn, `far` 8 calm |
| `doom_clock` rules | `d_week` at 7, warn (T) "Seven days to the Red Tithing."; `d_zero` at 0, acts [1, 2], effects `[{"effect": "add_flag", "key": "lesser_tithings", "delta": 1}]`, then `[{"vignette": "v_lesser_tithing"}]`, `restart_days` 14 (B); the Act 3 hard rule comes with Act 3 data |

Encounter tags (B): the main missions' boss fights `loud`; Vesper's drainage locks and Hollis's vantage points
`quiet`. Bribe (B): a repeatable choice at Tess, `take` 40 Scrap and `meter_add alert -10`, shown while
`meter.alert >= 31`. Suspicious and Lockdown gates on routes and roamer markers use the read keys (SUNKWARD1 data).

### 4.3 Checkpoints, triggers and the worst-case model

| Row | Values |
| --- | --- |
| `cp_cages` (CMQ1) | act 1, quest M2 (The Crossroads Cages), deadline 14 (B), warn [7, 3, 1], `end_run`, card `bad_end_cages`, text (T) |
| `cp_crucible` (CMQ1) | act 1, quest M5 (The Crucible), deadline 38 (B), warn [7, 3, 1], `end_run`, card `bad_end_crucible`, text (T) |
| `t_act1` | on enter `sluice_undercroft`, when `prologue_done`; effects `set_act` 1, `meter_show alert`, `clock_start doom_clock 42`; then vignette `v_act1_title` |
| `t_wiped` | on any, when `roster.living < 1`; effects `end` fail, card `extinguished_wiped` |

Worst case (`sim deadlines`, CAMPAIGN1-SIM): main quests are the act's quests tagged `main`, in data order; each
starts and ends at the `defeat.to` hub, its node is its first step's location. Per mission: travel out, then
`losses` (default 1) times lose, retreat (`defeat.days`), rest (`rest.days`), travel out again; win; travel back
unless it is the act's last; a rest after each win. Best case: travel out and back only. Output per checkpoint:
deadline, best day, worst day, slack. On the full Act 1 graph (all legs 1 day: hub 2 legs to 03, 3 to 04, 4 to 06)
the slice's M2, M3, M5 worst cases are day 10, 22 and 32, hence the placeholders 14 and 38; the sim recomputes them
on SUNKWARD1's slice graph.

## 5. Slice scope versus later

| In the slice | Later |
| --- | --- |
| E, S, M, C, T, U, SIM, D; Alert with Act 1 rules; the Red Tithing countdown, its soft rule and restart; two checkpoints; the three fail conditions as data (the clock's in Act 3); Scrap and Tallow names; -2 AP at zero Tallow; the hub rest; the retreat | Travel modes (CMQ2), searchlight sweeps, the Act 2 siege at Breach and the defend-or-finish dilemma, the 2.4 correction and relabel data, side quests locking at the reveal, doctrine (DOC), Act 3 Tallow prices (P), the Rock Salt flush, the final battle's dream retries, the Act 3 hard clock row |

## 6. Docs delta (for the planner)

- `docs/STRUCTURE.md`: `rpgkit_world/src/campaign/` (meters, clock, checkpoints, purse, triggers, previews; kit,
  story-free), `emberwake_gui/src/runover.rs`, `rpgkit_sim/src/deadlines.rs`, `aegis_save` keyed saves.
- `games/emberwake/docs/GAME_BRIEF.md` "where to look": campaign meters, checkpoints, run over.
- `engine/README.md` and `engine/docs/authoring/campaign_structure.md`: keyed saves; the kit hook that carries
  checkpoints and meters (link this spec); `games/emberwake/docs/SCREENS.md`: the run-over screen.
- `docs/WORKFLOW.md` or the balance skill: `tools/fc.sh sim deadlines`; `decisions/balance.md`: a slack target
  (proposal: worst-case slack of at least 2 days per checkpoint).
- ROADMAP row (`proposed`): CAMPAIGN1, campaign meters, the day clock, checkpoints, run over, purse names, rest and
  retreat (kit), depends on ROSTER1 and STORYUI1 for the slice.

## 7. Open questions for the user (recommended option first)

Planner picks (2026-10-25, the user is away and asked not to wait): CMQ1-CMQ9 as recommended; listed for the user's
confirmation in games/emberwake/docs/lore/OPEN_QUESTIONS.md section 4.

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| CMQ1 | Which Act 1 missions are checkpoints (canon: "certain missions") | (a) M2 the Crossroads Cages and M5 the Crucible, placeholders day 14 and 38; (b) only the Crucible | (a): the cages have a story clock (youths culled), the Crucible closes the act; both are in the slice |
| CMQ2 | Travel modes (CANON: a stealth leg +1 day; sunk_ward 2.1: stealth by default, forced march +5 Alert) | (a) build in the slice (CAMPAIGN1-TM): stealth default, forced march optional; (b) later: slice legs are 1 day, no mode | (a): it is the map's trade between the two meters and the deadlines are tuned on it |
| CMQ3 | What the failure screen offers | (a) Load the act's start, Load a save (slots and the autosave before the fatal step), Main menu; (b) only the act's start and Main menu | (a): the fail is reached only past warnings and an ask; (b) adds roguelite stakes |
| CMQ4 | Does the retreat day after a lost battle cost Tallow | (a) yes, it is a day of travel (1); (b) only the day | (a) |
| CMQ5 | Where can the cell rest | (a) only at rest places (the Dry Hollows); (b) also camp on any district node, 1 day, 1 Tallow | (a): one hub to invest in (Q-23), defeat already sends the cell home |
| CMQ6 | Alert's floor of 30 from Act 2 | (a) a minimum for the rest of the campaign; (b) only Act 2's starting value | (a): "floor" in Q-14 and campaign_structure.md |
| CMQ7 | The act-start save in the load menu | (a) only on the failure screen; (b) also a locked "Act start" row in the load menu | (a) for the slice, (b) later |
| CMQ8 | An ask before any step that would end the run (a loud fight at Breach, a trip past an Act 1 deadline) | (a) yes, No as the default; (b) the threshold warnings only | (a): "only by blatantly ignoring the meter" |
| CMQ9 | The clock's HUD name | (a) "The Red Tithing", relabelled "The Great Tithing" at 2.4; (b) "Doom Clock" | (a): the relabel only reads as a reveal if the HUD names the rite |

Planner notes (not user questions): the "own save format" of the new worlds ("Worlds and reuse") may absorb 11.5's
block if it lands first; STORYUI1 must offer an end card with a button row or CAMPAIGN1-U draws plain cards.

## 8. Canon versus code and lore (found while surveying)

| Canon | Code or doc today | Resolution here |
| --- | --- | --- |
| A rest costs 1 day and 1 Tallow (CANON 2.6, safehouse 2.3) | free heal on entering a rest place (`explore.rs:456-460`); Camp 3 supplies, no time (`progression.rs:229-319`) | manifest `rest` (R1), defaults keep the Ashlands |
| A lost Act 1 battle: retreat home, wounded, a day passes (Q-20) | `wake`: full heal, forced rest or back to town, no time (`explore.rs:528`, `progression.rs:331-347`) | manifest `defeat` retreat (R4) |
| Zero Tallow: -2 AP only (Q-18) | zero supplies: road chance x1.5 (`travel.rs:12,110-124`); no battle AP hook (`gui/battle.rs:575-578`) | `at_zero` (P3) |
| Route `time` 1 = 1 day | `time` is unitless; schema default 3; Ashlands legs 2-4 | X2: only campaign worlds read days; `sunk_ward` sets 1 |
| Scrap and Tallow by name | "Gold" and "Supplies" fixed in `hud.rs:102-103` and in log texts (`region.rs:118-124`) | purse names (P1, W4); the log texts read them too |
| The Extinguished screen | no fail flow, no run-over state anywhere | F1-F5 |
| Act 3 Tallow costs more Scrap (Q-45) | one flat `supply_price` per shop (`shop.rs:107`) | `supply_steps` (P4, later) |
| A stealth leg takes 1 day more (CANON 2.6) | `sunk_ward_campaign_map.md` 2.1 makes stealth the default: every leg 2 days | CMQ2 |
| Q-18 decided (b) | `sunk_ward_campaign_map.md` 2.1 still calls the x1.5 open | stale doc line (lore lane) |
| Q-43 decided (b) | `safehouse_economy_and_triage.md` 1 and 2.1 still call the flush effect open | stale doc lines (lore lane) |
| The recruit cap is the bunks (ROSTER1) | `recruit` refuses at `party_max` (`dialogue.rs:675-681`) | ROSTER1 |
