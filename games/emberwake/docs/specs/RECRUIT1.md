# RECRUIT1: Warrior-only start, two story recruits, named heroes

Status: proposed (spec 2026-10-18; approved direction: decision world-campaign "DR1 design review answers (user 2026-10-18)" (4), review `games/emberwake/docs/specs/DESIGN_REVIEW1.md` 3.4 and 3.5)
Depends on: CAMP1-A2 (story data on develop), DLG1-L (choice effects), REP1-L (int flags with `op`)
Lane: both (RECRUIT1-L schema-first, then RECRUIT1-A data, then RECRUIT1-B balance probe)
Touches: L: `scripts/world/{world_schema,world_catalog}.gd` (manifest `party_max`, mixed `start_party`, party-character check), `scripts/game_state/{new_game,game_state}.gd`, `scripts/progression/{party_state,party_builder}.gd`, `scripts/dialogue/{dialogue_rules,dialogue_runner}.gd` (effect `recruit`), `scripts/ui/menus/{party_model,new_game_model}.gd` (names, active count), suites `progression_test_suite`, `dialogue` suites, `menus_test_suite`. A: `data/catalog/units.json` (two rows), `data/worlds/ashlands/{world,characters,dialogues,locations,encounters,quests}.json`, one seed `data/seeds/recruit1.gd`

## Goal

A new Ashlands game starts with one named warrior. He fights the first small fights alone. Twice during "The Long Dry" a muster at Warden Keep lets the player add one recruit of any class other than warrior, so the campaign is played by a party of three whose make-up the player chose. Heroes have names. Old four-hero saves keep working.

## What exists (cited)

- `start_party` is a list of catalog party units, max `PartyState.PARTY_MAX` 4 (`world_catalog.gd` `start_party_errors`); Ashlands: warrior, ranger, rogue, mage (`data/worlds/ashlands/world.json`).
- Catalog party units: warrior, ranger, rogue, mage at level 5 (`data/catalog/units.json`); no priest or brawler party unit.
- `UnitState.lore_ref` exists and `PartyBuilder.build_report` copies it to the Combatant; `GameEncounter.hero_key` uses the member id, so renaming never breaks AUTO1 tactics.
- Choice effects: set_flag, add_flag, start_quest, turn_in, give, take, open_shop, fight (`dialogue_rules.gd` `EFFECTS`); 6 authored choices per node.
- Sim `comp=` takes 1..`PARTY_MAX` class ids (`tools/sim/sim_args.gd`), so solo, duo and trio runs need no tool change.
- BALANCE_REPORT (BAL3-R0) "Tier": minion and standard 1v1 clear 100 for every class; 1v3 minions 0 except priest. So one hero beats one or two weak foes, not three.

## 1. Schema contract (schema-first: yes)

### 1.1 Manifest field `party_max`

| Field | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| party_max | int | -1 | -1 (= nearest parent's) or 1..`PartyState.PARTY_MAX` ("Party max must be -1 (inherited) or 1..4.") |

Resolver `WorldCatalog.party_max(world_id) -> int`: nearest manifest in the lineage with a value >= 1, else `PartyState.PARTY_MAX` (4). A resolved `start_party` longer than it is an error ("Start party has %d heroes; party max is %d."). `PartyState.PARTY_MAX` stays 4 (the sim sweep uses four-class comps).

### 1.2 `start_party` accepts world characters

An entry is either a catalog unit with side party (as today) or a world character id that resolves from the world (`WorldCatalog.resolve(world, "characters", id)`). An id that is both is an error ("'%s' is both a unit and a character."). New check `WorldCatalog.party_character_errors(world_id, id) -> Array[String]` used by `start_party` and by `recruit`:

- role is `party`; `base_unit` is a catalog unit with side party;
- `named_items`, `disciplines` and `tuning` are empty ("Party hero '%s' may not carry named items, disciplines or tuning yet (RECRUIT1).");
- `passives` may be set (copied once, rule 2).

### 1.3 Runtime party cap

`PartyState.max_active: int` (not saved, default `PARTY_MAX`) is set from `WorldCatalog.party_max(gs.world_id)` by `GameState.create` and `GameState.from_dict`. `active_members()` and `validate()` use it instead of the constant. `from_dict` keeps the first `max_active` ids of a longer `active_ids` (the rest stay in the roster as reserve). No save version change: no new saved field.

### 1.4 Dialogue effect `recruit`

| Effect | Keys (default) | Where | Validation (message) |
| --- | --- | --- | --- |
| recruit | character ("") | choices only (not node effects), at most one per choice | ref resolves in the world ("Recruit: unknown character '%s'."); `party_character_errors` empty; `EFFECT_REF_KEYS` gains `character: characters` |

## 2. Rules (logic lane)

1. **Hero from a character.** `UnitState` id = the character id (unqualified, e.g. `bryn_hale`), `base_unit` = its base unit, `lore_ref` = its `_qid`, `passives` = `WorldFactory.character_passives(character, world)` (character passives plus faction grants), empty spend, default gear.
2. **Names.** `PartyBuilder.build_report` sets `Combatant.name` to the character's `name` when `lore_ref` resolves; else the unit name as today. A new `PartyBuilder.display_name(state)` is used by the party, results and new-game screens (no "Warrior" label for a named hero).
3. **New game.** `NewGame.create` builds catalog entries as today and character entries by rule 1, at the base unit's level.
4. **Recruit, precheck (all-or-nothing, `DialogueRunner._precheck`).** Refused with a note when the character is already a member ("%s is already with you."), or when the roster holds `party_max` members or more ("The party is full (%d of %d).").
5. **Recruit, apply.** Add the member (rule 1) with `total_xp` = max(threshold of the base unit's level, highest `total_xp` among active members), so a recruit joins at the warrior's level. If `active_ids` is non-empty, append the new id. Event note "%s joins the party.".
6. **Visibility.** A choice whose `recruit` character is already a member is hidden (same filter as gone characters).
7. **Old saves (Q3 A).** An Ashlands save with four members loads with the first three active and the fourth in reserve; the muster refuses recruits because the roster is full (rule 4). Old members keep no name (empty `lore_ref`).

Acceptance (tests): a new Ashlands game has one member `bryn_hale` named "Bryn Hale"; a recruit choice adds the member at the warrior's XP and hides itself; a third recruit is refused with the full-party note; `party_max` 3 trims a four-id `active_ids` on load and `validate()` is clean; a character with a named item in `start_party` is an error; an id in both units and characters is an error; Tidewrack (extends ashlands) inherits `start_party` and `party_max`.

## 3. Story placement (asset lane)

| Phase | From | Until | Party | Fights in the phase |
| --- | --- | --- | --- | --- |
| Solo | New Game | first recruit | Bryn | courtyard_skirmish, ruined_courtyard step fights |
| Duo | offer 1 (Courtyard held) | second recruit | Bryn + 1 | pass_ambush, cinder_gate step fights |
| Trio | offer 2 (oath sworn) | end | Bryn + 2 | steel trip raid, ashmaw_camp, finale (unchanged) |

Offers never block the story: a player who skips the muster may walk on and lose (the loss cost is lost time, decision DR1 (2)). The `cinder_gate_watch` description points to the muster.

### 3.1 Muster (Q1 A: any remaining class at each offer)

- New character `brask_toller`: "Brask Toller", title "Muster Sergeant of the Wardens", role npc, base_unit warrior, faction ember_wardens, home warden_keep, dialogue `muster_talk`, background "Keeps the duty roll in his head and the names of the dead on his belt."
- Marker at warden_keep: `muster`, kind npc, anchor `npc_d` (free in `walled_town`), character brask_toller.
- Quest edits: `hold_the_courtyard.complete_flags` += `ashlands.muster_1` true; `warden_oath.complete_flags` += `ashlands.muster_2` true.
- Dialogue `muster_talk` (services false). hello: "Toller runs a thumb down the duty roll. "The keep can spare a blade or two, Warden. No more."" Choices:
  1. "Who can ride with me?" requires `ashlands.muster_1` true and `ashlands.recruits` op `<` 1, to `roll`.
  2. "The oath is sworn. Who else?" requires `ashlands.muster_2` true and `ashlands.recruits` op `<` 2, to `roll`.
  3. "Farewell." to "".
- Node `roll`: "Toller nods at the yard. "Pick one. The rest stay on the wall."" Five choices, one per row of 3.2: text = the row's pitch line, effects `recruit <id>` then `add_flag ashlands.recruits +1`, to `joined`; sixth choice "Not yet." to hello.
- Node `joined`: "Toller strikes a line through a name. "Bring them back."", next hello.
- `cinder_gate_watch` description gains the sentence "Take someone from Toller's roll first; the pass is no place to walk alone." (text edit only).

### 3.2 Heroes (characters, ashlands, role party, art "", no named items, disciplines or tuning)

| id | name | base_unit | pitch line (choice text) | background |
| --- | --- | --- | --- | --- |
| bryn_hale | Bryn Hale | warrior | (start hero) | A Warden's son from the burned post at Emberfall; carries his father's shield and his temper. |
| sella_marsh | Sella Marsh | ranger | "Sella Marsh, the archer on the north wall." | Hunts ash-hares in the wastes and counts every arrow twice. |
| kit_vane | Kit Vane | rogue | "Kit Vane, the cutpurse who took the oath." | Stole from the Wardens once, got caught, and stayed. |
| oren_dusk | Oren Dusk | mage | "Oren Dusk, the hedge-mage by the forge." | Reads embers the way Tev reads smoke; nobody asks where he learned fire. |
| alda_wren | Alda Wren | priest | "Sister Alda Wren, the hearth-keeper." | Tends the keep's one fire and every Warden who comes back from the pass. |
| dagny_coal | Dagny Coal | brawler | "Dagny Coal, the pit fighter from Slag Gate." | Fought for coin in the slag pits until the Horde burned them; fists are what she kept. |

`passives` empty for all (INTENT1 may give Bryn one; that spec owns it).

### 3.3 Catalog units (`ContentCatalog.upsert`, copy the `warrior` row's defaults)

| id | name | side | class_id | level | tier | loadout |
| --- | --- | --- | --- | --- | --- | --- |
| priest | Priest | party | priest | 5 | hero | smite, rally, mend, purify, greater_mend |
| brawler | Brawler | party | brawler | 5 | hero | body_blow, lunge, sweep_kick, hamstring, haymaker |

If a family is not unlocked at level 5 (`Combatant.tier_unlocked`), take the next path family by level and report it. This supersedes balance decision "BAL2 role answers" (7) "no new priest/brawler party units" (the user's DR1 (4) makes every class a recruit); the planner marks that item superseded.

### 3.4 Manifest and early encounter sizing

- `world.json`: `start_party` ["bryn_hale"], `party_max` 3.
- Sizing rule: solo fights hold at most 2 minions and no elite; duo fights at most 3 enemies at once and 4 counting waves; trio fights are unchanged here (BAL4-ATTR measures them, ASHPACE1 tunes lightly).

| Encounter or binding | Change |
| --- | --- |
| courtyard_skirmish | enemies [skarn_ironjaw, vex_cutpurse]; description "Bryn against two Horde raiders in the Ruined Courtyard."; intro "The Horde has come early this year. Two raiders drop from the Courtyard wall, blades out." |
| ash_straggler (new) | copy of ash_scouts with enemies [ash_brawler]; name "A Horde Straggler" |
| ruined_courtyard `ruins` binding | encounters [[ash_scouts, 2], [ash_straggler, 3]], elites [], elite_pct 0, steps 12..20 kept |
| ruined_courtyard marker `patrol` | requires_flags += `ashlands.pass_cleared` true (the elite patrol appears after the duo phase) |
| pass_ambush | wave_units [nok_flamebow, gruk_smokecaller] (bolg_redhand removed); description ends "...an archer and a smokecaller fall on them." |
| cinder_gate binding, everything after the oath | unchanged |

If validation refuses an empty `elites` list, the builder keeps `[[ash_patrol, 1]]` with `elite_pct` 0 and reports it.

Acceptance (asset, headless): `validate_all()` clean for both catalogs; New Game party = Bryn; courtyard win opens offer 1 at Toller with five choices; after one pick the roll entry is gone until the oath; after the oath the roll shows the four remaining classes; a third recruit never shows.

## 4. Balance follow-up

- RECRUIT1-B (balance, after RECRUIT1-A on main): solo `comp=warrior` L5 vs courtyard_skirmish, ash_scouts, ash_straggler; duo `comp=warrior,<x>` for the five classes vs pass_ambush, ash_raiders, ash_patrol. Targets from `games/emberwake/docs/decisions/balance.md` "BAL2 role answers" (2): solo fights at the mob band (>= 90), pass_ambush at the line band (70-90). Below target: the asset fix is one enemy fewer, not stat tuning.
- BAL4-ATTR (approved) already runs warrior + two recruits: the 10 trio comps (5 choose 2). Decision "BAL3 answers" Q2 measures party targets "with a party of 4"; the planner should ask whether Ashlands targets move to a party of 3.

## Out of scope

Recruits outside the Ashlands, dismissing or swapping recruits, recruit portraits and art, recruit dialogue or loyalty, named items or disciplines on heroes, a player-chosen hero name, the deserter (DESERT1), any sim tool change.

## Questions for the user

1. How does the player pick a recruit?
   - (recommended) A: any remaining class at each muster (five, then four). Matches DR1 (4) and gives the most replay.
   - B: two fixed offers of two or three candidates (for example ranger or rogue, then priest, brawler or mage). More story control, less choice.
   - C: both picks at the first muster. Simplest, but the duo phase disappears.
2. When do the two offers come?
   - (recommended) A: after the Courtyard is held and after the oath. A short solo phase, a duo Cinder Gate, a trio for the steel trip and finale.
   - B: both before the Cinder Gate. Shortest solo phase, no duo phase, fewer encounter edits.
   - C: after the Cinder Gate and at Slag Gate. Longer solo phase; needs bigger cuts to the pass fights and a cross-world recruit.
3. Old saves with four heroes?
   - (recommended) A: keep them; three active, one in reserve, no muster offers. No data lost and no migration.
   - B: refuse old Ashlands saves with a "start a new game" message. Cleanest, but loses progress.
   - C: let old saves keep four active (exempt from `party_max`). Keeps old play, adds a special case.
