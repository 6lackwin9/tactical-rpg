# ROSTER1 tasks: tests, parity plan, task split, ENCOUNTER2 changes, data, open questions

Status: proposed (2026-10-25); the rules are in [ROSTER1.md](ROSTER1.md) (rule ids such as BK3 refer to it)

## 1. Tests (rule-derived; scratch data folders, never the shipped data)

| Task | Test | Expectation from |
| --- | --- | --- |
| L | world without `roster`, 4 members, `recruit` | refused "The party is full (4 of 4)." (BK3, classic) |
| L | `bunks` 6, 4 members all active, `recruit` | joins, benched (not in `active_ids`) (BK3, BK4) |
| L | `bunks` 6, 6 members / `bunks_flag` value 2, `bunks_max` 8 | refused "Every bunk is taken (6 of 6)." / cap 8 (BK1, BK3) |
| L | no rule row, every outcome with downed and standing heroes | `hp_lost` equals the HEAL1 result of the old GUI path (L3) |
| L | Act 1 row: won rout with a hero downed / lost battle | that hero wounded / every deployed hero wounded (3.2) |
| L | wounded member, max HP 100, `hp_lost` 0 | battle max 80, start 80; ends at 80 untouched: `hp_lost` 20 against 100 (W2) |
| L | downed twice while wounded | one `wounded` entry (W1) |
| L | rest place, Camp, CAMPAIGN1 `rest` effect | `wounded` cleared on every member (W3) |
| L | Act 2 row: fled with a hero downed | captured, in `departed` with cause `downed`, bunk freed, out of `active_ids` (L2) |
| L | depart a member whose character has `keepsake`, row `keepsakes_on` [captured] | stash +1 of that item; the record keeps its id; paid once (KS1, KS2) |
| L | `recruit` of a departed character | refused "Kaelen is gone." (L1) |
| L | `party_min` 4 with 3 active, dialogue `fight` | refused "Take 4 into this fight (3 in the party)." (BK6) |
| L | `recruit` with `level` average, actives at 2, 3, 3 | joins at level 2 (BK5) |
| L | conditions `roster.living`, `roster.kaelen` = captured, `roster.bunks_free` | answered from the state (RK1) |
| L | save round trip with `departed`, `conditions`; a save without them | equal; reads empty; Godot saves still compare equal (11.4) |
| L | checks: keepsake not an accessory, `value` 2, rule rows with two empty `acts` | one error each (11.1-11.3) |
| W | won hold battle, `then` capture | holder in `departed` (captured, cause `stayed`), flag holds its id; others settle by the row |
| B | `bleed_out_rounds` 2, hero downed in round 1 | bleeds out at the end of round 2, fate `dead`, log line (B2) |
| B | `revive` heal on a downed ally with `bleed` 1 / a plain heal | active next block at `max(heal, 1)` / refused (B3) |
| B | `bleed_out_rounds` 0 | `trace_parity` unchanged (B1) |
| T | member with temperament `wary` | passive applied, no deck slot; default intent guard unless the member sets one (TP3, TP4) |
| T | temperament `ai` {defend_below 0.6} | the member's `AiProfile` reads 0.6, other keys from the unit (TP5) |
| P | `rescue` template count 2, same seed twice | same names, ids and origins; no name repeats a member, pool or departed name (PG2) |
| P | swear in before `open_flag` / with no free bunk | refused with the PO3 / BK3 text |
| P | dismiss | departs `dismissed`, gear and pouch in the stash (PO3) |
| K | procedural member departs, `by_class` warrior | instance "Ida's Shield Strap" wearable, not in the sell list, a second of its template refused on one member (KS3) |
| M | `per_deploy` 1, `stasis_days` 5: 2 deployments, 10 days since joining | value 4; no effect lowers it (FZ2) |
| M | value at `max`, the player picks the last fight | deploys with `last_passive`, departs `dead` after the battle (FZ4) |
| D | `tools/fc.sh content check` | clean |

## 2. Parity plan (the Ashlands stay byte-identical)

- Every hook defaults off: no `roster` block in any Ashlands manifest, so bunks = `party_max`, no rule row (classic
  HEAL1), no new member or party key is written, `bleed_out_rounds` 0, no `revive`, no temperament.
- The risky refactor lands first and alone: L's first commit moves the GUI write-back (`gui/battle.rs:744-751`) and
  the hero build (`:576-588`) into `roster::settle_party` and `roster::field_unit` with no rule change; `journey`,
  `world_traces` and the host `replay` prove it.
- `trace_parity`, `world_traces`, `journey`, `phasefold` (golden), `saves`, `checks_parity`, `sim_parity` and
  `replay` run unchanged after each task. No recorded trace, golden or expectation is edited; no check is retired.
- `data_source`: the new keys (`roster`, `keepsake`, `temperament`, `party_min`, `revive`) go into `AFTER_GODOT`;
  the new kinds (`temperaments`, `recruits`) follow DERIVE1-S's strip for `derived_stats`; a HISTORY row each.
- Saves: new keys only when non-empty, so `tests/saves.rs` still finds Godot's state; the shared version bump adds an
  empty step (ROSTER1.md 11.4).

## 3. Tasks (split by layer and lane; one builder run each; status `spec`)

Paths (KIT1_TASKS 5): `rpgkit_*` under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`;
`schema.json` = `kits/rpg/schema.json`; `data_source.rs`, `tests/rewards.rs` = `emberwake_world/tests/` (game).

| Id | Title | Layer | Lane | Depends | Slice | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| ROSTER1-L | Roster core: manifest `roster`, rule rows, `settle_party` (moves the GUI write-back), wounded, bunks, `party_min`, join `level`, `departed` and `depart`, named keepsakes, read keys, save keys (3.1, 3.2, 4, 5.1, KS1-KS2, 9, 11) | kit | logic | ENCOUNTER2-H | yes | `rpgkit_world/src/{roster (new),state,save,progression,dialogue,quest,battle,data,lib}.rs`; `emberwake_gui/src/battle.rs` (start and settle call the world); `rpgkit_content/src/check/{world,dialogue,encounter}.rs`; `schema.json`; `rpgkit_world/tests/roster.rs` (new, scratch data); `emberwake_world/tests/data_source.rs` |
| ENCOUNTER2-W | Revised (section 4): `then` capture and kill through `roster::depart`; no `party.away` | kit | logic | ENCOUNTER2-H, ROSTER1-L | yes | `rpgkit_world/src/{battle,explore,dialogue,travel}.rs`; `tests/rewards.rs`; `emberwake_gui/src/{battle,hub}.rs` |
| ROSTER1-U | Party screen and Results: wounded mark and field HP, "Bunks n/cap", departure and keepsake lines, the `party_min` refusal | game | logic | ROSTER1-L, ENCOUNTER2-W | yes | `emberwake_gui/src/{party,hub}.rs` |
| ROSTER1-D | `sunk_ward` roster rows (5.1); the Crucible (`enc_evt_06`) a check only | game (data) | asset | ROSTER1-L, SUNKWARD1, CAST1-A (keepsake items), CAST1-A6 (the Crucible row) | yes | `data/worlds/sunk_ward/{world,characters,encounters,dialogues}.json` (`encounters`: M2 only) |
| ROSTER1-T | Temperaments: kind, character and member field, passive, default intent, AI overrides (7) | kit | logic | ROSTER1-L, RULESET1-S, RULESET1-AI | if CAST1 gives founders one | `schema.json`; `rpgkit_data/src/lib.rs`; `rpgkit_content/src/{source,store,check/simple}.rs`; `rpgkit_world/src/{roster,new_game,progression,state,save}.rs`; `emberwake_gui/src/{battle,party}.rs`; `data_source.rs` |
| ROSTER1-TA | Temperament samples (5.2) | game (data) | asset | ROSTER1-T | as T | `data/catalog/{temperaments,passives}.json` |
| ROSTER1-B | Bleed-out and revive (3.3) | kit | logic | ENCOUNTER2-H, RULESET1-IN | later (Act 2) | `rpgkit_battle/src/{session,world,skill,command,plan,log}.rs`; `rpgkit_content/src/check/{skill,item}.rs`; `schema.json`; `rpgkit_battle/tests/bleed_out.rs` (new, scratch data) |
| ROSTER1-BU | Bleed counter on inspect and the tell (B5) | game | logic | ROSTER1-B | later | `emberwake_gui/src/{battle,intent_text}.rs` |
| ROSTER1-P | Pool, `rescue`, recruit templates, generation, the `open_flag` gate, swear in, dismiss, retire, housing and crypts (5.2, 5.3) | kit | logic | ROSTER1-L | later (Act 2 start) | `rpgkit_world/src/{roster,dialogue,rolls,state,save,progression}.rs`; `rpgkit_content/src/check/{dialogue,world}.rs` (+ `recruits`); `schema.json`; `data_source.rs` |
| ROSTER1-K | Procedural keepsake instances (KS3) | kit | logic | ROSTER1-P | later | `rpgkit_world/src/{roster,progression,state,data}.rs`; `emberwake_gui/src/party.rs` |
| ROSTER1-KD | Keepsake templates by class (5.3) | game (data) | asset | ROSTER1-K | later | `data/catalog/items.json`; `data/worlds/sunk_ward/world.json` |
| ROSTER1-M | Living Martyr fuse (8) | kit | logic | ROSTER1-P, CAMPAIGN1 (clock), the Calcification spec | later (Act 2) | `rpgkit_world/src/{roster,state,save}.rs` |
| ROSTER1-RU | The Bunks screen (roster management) | game | logic | ROSTER1-P | later (Act 2, Q-96) | `emberwake_gui/src/` (a new hub screen) |

Order: L shares `state.rs`, `save.rs`, `dialogue.rs` and `quest.rs` with CAMPAIGN1's world task and RULESET1-D: run
them in a row, not in parallel. L lands before ENCOUNTER2-W (the slice chain: ENCOUNTER2-H, ROSTER1-L, ENCOUNTER2-W,
ROSTER1-U). ROSTER1-D can fold into SUNKWARD1 if the planner prefers one data run. Hub files: `roster.rs` (new) is
the module every later ROSTER1 task extends.

## 4. How ENCOUNTER2-W changes

- W1 unchanged (`Tally` party fates); ROSTER1-B adds the fate `dead`.
- W2 unchanged (`stay_behind.flag` holds the member id).
- W3 replaced: `then` is `none` (default), `capture` or `kill` (ROSTER1.md 3.2); `capture` and `kill` call
  `roster::depart(member, captured | dead, cause stayed | stood)`. No `party.away` list and no save step of its own:
  departed members live in `party.departed` (ROSTER1-L). Heroes who did not stay settle by `roster::settle_party`
  with the act's rule row instead of "today's rules".
- W4 unchanged; ROSTER1-U adds the departure and keepsake lines after it.
- The row gains the dependency ROSTER1-L and drops `state.rs` and `save.rs` from its Touches.
- Spec text updated by KIT1-P (2026-10-25): ENCOUNTER2.md 3.3 W3, section 6 (`then` row) and 7 ("the unit
  lifecycle hook" is ROSTER1); ENCOUNTER2_TASKS.md 1 (the W test), 2 (the saves bullet), 3 (the W row), 4.2 (`then`
  capture, the id `enc_evt_06`) and 7.5.

## 5. Data (asset lane)

### 5.1 `sunk_ward` (ROSTER1-D; `tools/fc.sh content upsert world world=sunk_ward @file.json` and per kind)

Manifest `roster` (act 0 is the prologue, CAMPAIGN1 `start_act`):

```json
{"bunks": 6, "bunks_flag": "hub_bunks_bonus", "bunks_max": 8, "crypts": 0, "open_flag": "roster_open",
 "rules": [
  {"acts": [0, 1], "hp_after": "<RQ1>", "downed_won": "wounded", "downed_left": "wounded",
   "downed_lost": "wounded", "standing_lost": "wounded", "wounded_max_hp_pct": 20, "keepsakes_on": ["captured"]},
  {"acts": [2, 3], "hp_after": "<RQ1>", "bleed_out_rounds": 3, "downed_won": "wounded",
   "downed_left": "captured", "downed_lost": "captured", "standing_lost": "wounded", "wounded_max_hp_pct": 20,
   "keepsakes_on": ["captured", "dead"]}],
 "keepsakes": {"on": ["captured", "dead"], "by_class": {}, "name": "{name}'s {item}"}}
```

| Where | Rows |
| --- | --- |
| characters (six founders) | `keepsake` `keepsake_<founder>` (CAST1_GEAR 7), only if RQ7 picks the hook |
| `enc_evt_06`, the Crucible (ENCOUNTER2 4.2; CAST1-A6 writes the whole row, CAST1 C16) | check only, no upsert: `content check` clean and the row holds `party_min` 4 and `stay_behind.then` `capture` (was `leave_party`); a missing key goes back to CAST1-A6 |
| the Crucible's end (CAMPAIGN1 trigger or STORYUI1 card) | `set_flag` `roster_open` true |
| M2 (`enc_evt_02`) win | `add_flag` `youths_saved` + the youths freed (SUNKWARD1's count) |
| Bram (Act 2, later) | a choice with `requires` `campaign.act` >= 2 and `hub_bunks_bonus` 0: `take` Scrap (balance lane), `add_flag` `hub_bunks_bonus` +2 |

Act 2 numbers (`bleed_out_rounds` 3, crypts) are placeholders for the balance lane.

### 5.2 Temperament samples (ROSTER1-TA; numbers placeholders for the balance lane)

| Id | Name | Passive (`slot` "") | `default_intent` | `ai` | Description |
| --- | --- | --- | --- | --- | --- |
| `steady` | Steady | `tmp_steady`: resist stagger 20, resist stun 20 | "" | `defend_below` 0.3 | "Keeps their feet when the line buckles." |
| `reckless` | Reckless | `tmp_reckless`: stat STR +1 (RQ9) | `assault` | `reckless` true, `flee_ratio` 0 | "Fights hardest with nothing left to lose." |
| `wary` | Wary | `tmp_wary`: resist bleed 20, resist poison 20 | `guard` | `defend_below` 0.6, `flee_ratio` 0.35 | "Watches the exits before the enemy." |

### 5.3 Keepsake templates by class (ROSTER1-KD)

Each row: family `accessory`, slot ACCESSORY_1, rarity RARE, `value` 0, `special` true.

| Id | Name | Class | Modifiers (below CAST1's named keepsakes) |
| --- | --- | --- | --- |
| `keepsake_tpl_warrior` | Shield Strap | warrior | stat CONS +2; resist stun 10 |
| `keepsake_tpl_ranger` | Fletching Charm | ranger | stat PERC +2; resist root 10 |
| `keepsake_tpl_rogue` | Lockpick Ring | rogue | stat DEX +2; resist slow 10 |
| `keepsake_tpl_priest` | Prayer Cord | priest | stat KNOW +2; resist poison 10 |
| `keepsake_tpl_brawler` | Knuckle Wrap | brawler | stat STR +2; resist stagger 10 |
| `keepsake_tpl_mage` | Ash Vial | mage | stat INT +2; resist burn 10 |

`by_class` in 5.1 then maps each class to its row.

## 6. Docs delta (for the planner)

- `docs/STRUCTURE.md`: `rpgkit_world` row adds `roster.rs` (lifecycle, bunks, pool, keepsakes, temperament);
  content `temperaments`, world kind `recruits`.
- `games/emberwake/docs/GAME_BRIEF.md`: a roster section (rule rows, bunks, wounded, keepsakes) as tasks land.
- CANON 2.4 rows and `lore/systems/roster_expansion_and_lifecycle.md` gaps 2-5 point to ROSTER1.
- `engine/docs/authoring/roster.md` (new, game-agnostic, promote-generic-methods): the lifecycle rule rows, bunks,
  the pool, keepsakes, temperament and the fuse as a kit shape, written when ROSTER1-L lands (in `rpgkit_world`).
- ROADMAP row wanted: ROSTER1, proposed, depends ENCOUNTER2: the unit lifecycle (wounded, captured, dead, retired,
  bleed-out), bunks apart from the party cap, the recruit pool and its flag, keepsakes, temperament, the Martyr fuse;
  spec `docs/specs/ROSTER1.md`.
- HISTORY rows wanted: (1) 2026-10-25, "temporary injury until a rest" (rejected 2026-10-18) returns for the new
  worlds only as `wounded` (Q-31); the Ashlands keep the rejection. (2) ENCOUNTER2's `party.away` and `then:
  leave_party` (never built), replaced by `party.departed` and `then: capture`. (3) The parity strips of section 2.

## 7. Open questions for the user (recommended option first)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| RQ1 | HP between battles in the new worlds (the canon names only wounded) | (a) `full`: everyone home at full HP, wounded is the only carry-over; (b) `carry`: HEAL1's carry-over and rally, wounded on top | (a): one readable cost; Tallow already prices rests; sims start fights at full HP |
| RQ2 | The gear of a captured or dead member | (a) back to the stash; (b) lost with them | (a): the Crucible pick should not favour the worst-geared founder |
| RQ3 | How the downed are revived from Act 2 | (a) a `revive` key on heal skills and pouch items (Maren's suture, a field kit any unit carries through Support); (b) a new basic every unit has | (a): no change to RULESET1's basics; a pouch item gives everyone the option |
| RQ4 | Isolation Crypt places | (a) 2 from Act 2, more with the Crypts' facility levels; (b) unlimited until the facility tree | (a): crypts stay a decision |
| RQ5 | A wounded member downed again | (a) stays one wounded (-20%); (b) stacks to -40% | (a): the baseline Q-31 wording |
| RQ6 | Who counts in `youths_saved` | (a) every freed youth, recruits included; (b) only the under-18s sent to Agnes or Pip | (a): one number for every rescue |
| RQ7 | The Crucible's keepsake | (a) the hook pays it at the capture (KS2); CAST1's six loot rows go; (b) CAST1's loot rows at the Act 2 opening, the hook only from Act 2 | (a): one path, and the cell holds it at once as the canon says |
| RQ8 | The first procedural recruit when several wait in the pool | (a) the player picks one at the act transition; (b) the first rescued joins | (a): the first roster decision of Act 2 |
| RQ9 | Q-40a's "Reckless: more damage at low HP" | (a) an `hp_below` stat context in the effect grammar (small, in ROSTER1-T); (b) a flat stat now | (a): the user's own example |
| RQ10 | CANON 2.4 Hub "the bunk project in the slice" against Q-38 "Act 2 on" | (a) Bram offers it in the slice, locked until Act 2; (b) nothing in the slice | (a): it shows the project and tests the data stand-in |
| RQ11 | From Act 2, members still standing after a lost battle | (a) come home wounded (as Act 1); (b) come home unhurt | (a): a loss still costs, and only the downed are taken |

## 8. Canon, specs and code (found while surveying)

1. CANON 2.4 Hub and Q-42 ("the bunk project in the slice") against Q-38 and "Roster in Act 1" (Act 2 on): RQ10.
2. CAST1_GEAR 7 pays the keepsake through loot rows at the Act 2 opening, while CANON says the cell keeps it at the
   capture: RQ7; under KS2 its rows also need `value` 0 (unsellable). CAST1_FOUNDERS 1 gives founders `gone_flags`
   `sunk_ward.<id>_captured` "read by the keepsake hook", but nothing sets those flags; the read key `roster.<id>`
   = `captured` serves instead (`{"key": "roster.kaelen", "value": "captured"}`).
3. CAST1_FOUNDERS: Dunstan joins at M3 as the fifth member; without BK1 RECRUIT1 refuses him ("The party is full
   (4 of 4).", `dialogue.rs:680`), so ROSTER1-L blocks the slice's M3.
4. `recruit` pushes onto `active_ids` past `max_active` once members may outnumber it (`dialogue.rs:729-731`): BK4.
5. World rules in the GUI: the HP write-back (`gui/battle.rs:744-751`), the hero build and AI profile (`:576-588`),
   the active toggle (`gui/party.rs:371-388`). L moves the first two; the toggle stays (no rule here changes it).
6. Entering a `rest` location heals for free (`explore.rs:456-460`), so a retreat to a rest hub would clear wounded:
   CAMPAIGN1 R1 sets `on_enter` false for `sunk_ward`.
7. HISTORY 2026-10-18 rejected temporary injuries; Q-31 brings them back for the new worlds only (header).
8. ENCOUNTER2 3.3 W3's `party.away` stand-in and `leave_party` (section 4); EQ2 (Act 1: carried out wounded) and
   Q-32 (Act 2: captured) agree with the rule rows.
9. Q-37 replaces the old "+1 Calcification stage per battle" with the fuse (8); the Calcification spec must map the
   fuse's tiers onto stages.
10. "Worlds and reuse" gives the new campaign its own save format and KIT1 6 leaves it to CAMPAIGN1 or ROSTER1;
    both specs instead add optional keys to `fc_save` with one shared bump (ROSTER1.md 11.4). A separate format
    stays open for the planner. KIT1 6 also leaves per-world class display names ("Vanguard" for `warrior`) to the
    roster or campaign spec: ROSTER1 does not cover them (CAST1 G1 uses the character `title` as the stand-in).
