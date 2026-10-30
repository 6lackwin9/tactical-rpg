# ROSTER1: the unit lifecycle, bunks, recruits, keepsakes and temperament (RPG kit hooks)

Status: proposed (spec 2026-10-25, from CANON 2.4 and the rows "Founder death and the Crucible", "Wounded", "Roster
in Act 1", "Losses: choice, then fate", "Roster group (1.3)" and "Round intent (renamed)" (Q-37a, Q-40a) in
`games/emberwake/docs/decisions/lore.md`; Q-29, Q-31, Q-32, Q-34, Q-36, Q-37, Q-37a, Q-38, Q-40, Q-40a, Q-93)
Depends on: ENCOUNTER2-H (party fates) for the core; RULESET1-S (`default_intent`, passive `slot`) for temperament;
CAMPAIGN1 (written in parallel: acts, the day clock, rests, the retreat, read keys) by reference; KIT1-N
Lane: logic first (schema, world rules, battle rules), then asset (the `sunk_ward` rows); tests, parity plan, task
split, data, ENCOUNTER2 changes, docs delta and open questions: [ROSTER1_TASKS.md](ROSTER1_TASKS.md)
Touches (by task, see the task file): under `kits/rpg/crates/`: `rpgkit_world/src/{roster (new),state,save,
progression,dialogue,quest,battle,new_game,data,lib}.rs`, `rpgkit_battle/src/{session,world,skill,command,log,
intent}.rs`, `rpgkit_data/src/lib.rs`, `rpgkit_content/src/{source,store,check/*}.rs`, `kits/rpg/schema.json`;
`games/emberwake/crates/emberwake_gui/src/{battle,party,hub}.rs`; `games/emberwake/data/**` (asset lane). Kit line
numbers are pre-KIT1 (KIT1-M moved the files byte-identical; `rpgkit_*` = `kits/rpg/crates/rpgkit_*`).

History: `docs/HISTORY.md` 2026-10-18 rejected "a temporary injury until the party rests" (a penalty for losing
discouraged risk). Wounded is that idea; the reason changed for the new worlds only: the user chose it (Q-31) as the
cost that replaces death in Act 1. It is a rule row, off by default, so the Ashlands keep the rejection.

## 1. Goal

One kit shape for a party member's life across battles: ready, carrying conditions (wounded), downed and bleeding
out in battle, then captured, dead, retired, dismissed or lost. A rule row per act says which transitions are on, so
Act 1 has no death or capture and the Ashlands keep HEAL1 exactly. Beside it: bunks apart from `party_max`, a pool of
rescues gated by a story flag, keepsakes from the captured and the fallen, a temperament per member and a burn-down
counter for Living Martyrs. No `[engine]` part: every rule is genre logic (`[kit hook]`) filled by `[data]`.

## 2. Survey: the roster today (`claude/lore-canon` at 153ec78)

| Area | Where | What it does |
| --- | --- | --- |
| Caps | `rpgkit_world/src/data.rs:17`, `:165-173` | `PARTY_MAX` 4; `party_max(world)` = nearest manifest value >= 1 |
| Party | `rpgkit_world/src/state.rs:291-305`, `:317-327`, `:372-375` | `members`, `active_ids` (empty = the first `max_active`), `max_active` (never saved), stash, gold, supplies |
| Member | `state.rs:33-51` | `UnitState`: id, base unit, `lore_ref`, XP, spend, skills, loadout, equipment (catalog ids), pouch, `hp_lost` (HEAL1) |
| Recruit | `dialogue.rs:673-682`, `:724-734`; `new_game.rs:10-21`; `data.rs:188-207` | effect `recruit`: refused at `members.len() >= party_max` ("The party is full (n of cap)."); joins at the highest active XP; pushed onto `active_ids` when that list is non-empty (`:729-731`, can pass `max_active` once members may outnumber it); party heroes may not carry named items |
| HP between fights | `progression.rs:228-279`, `:342-347`; `explore.rs:456-460`, `:528-551` | HEAL1: carry-over through `hp_lost`; a downed hero rallies to 25% on a win, 1 HP on a flee; a lost fight wakes everyone healed; entering a `rest` location heals all |
| Write-back | `emberwake_gui/src/battle.rs:578`, `:744-751` | the GUI builds heroes at their stored HP and writes `hp_lost` back on won and fled: a world rule living in the GUI |
| Results | `rpgkit_world/src/battle.rs:179-214` | `Tally` holds enemy fates only; ENCOUNTER2-H adds `party_fates` (escaped, stayed, stood, downed, standing) |
| Downed | `rpgkit_battle/src/world.rs:142-148`; `grid.rs:154`; `metrics.rs:430-432` | HP 0 = not alive; heals refuse a downed unit; dead units keep their tile; no revive, no bleed-out |
| Hero AI | `emberwake_gui/src/battle.rs:588`; `rpgkit_battle/src/world.rs:31-42` | `AiProfile::for_unit` from the base unit: role, `defend_below`, `target_pref`, `reckless`, `heal_below`, `flee_ratio` |
| Accessories | `rpgkit_data/src/lib.rs:272-340`; `rpgkit_content/src/source.rs:588-600`; `rpgkit_rules/src/unit.rs:254`, `:278-341` | items are catalog rows; `effect` modifiers use the passive grammar (`stat`, `resist`, `ap_regen`, `skill_potency`, `status_potency`, `spell_reach`); worn by catalog id (`progression.rs:176-182`) |
| Selling | `rpgkit_world/src/shop.rs:183-201` | only stash items with a sell price >= 1 are offered: `value` 0 cannot be sold |
| Active toggle | `emberwake_gui/src/party.rs:371-388`, `:852` | "In party (n/max)" check box; the rule lives in the GUI |
| Saves | `save.rs:17`, `:115-199`; `tests/saves.rs:41-47`; `tests/data_source.rs:25-34` | v12; Godot saves must give a `to_value()` equal to Godot's state (only `reward_paid` stripped); new schema keys go into `AFTER_GODOT`. No crate names `captured`, `wounded`, `bunk`, `keepsake` or `temperament` |

## 3. The member lifecycle `[kit hook]` (module `rpgkit_world::roster`, ROSTER1-L)

### 3.1 States

| State | Where kept | Deploys | Takes a bunk | Notes |
| --- | --- | --- | --- | --- |
| ready | `party.members` | yes (party screen toggle) | yes (`housing` `bunk` or `crypt`) | may carry conditions (4) |
| downed | battle only | - | - | HP 0; with bleed-out on, a counter runs (3.3) |
| captured | `party.departed` | no | no | left behind or taken; the Crucible's winch holder |
| dead | `party.departed` | no | no | bled out, or a scripted death |
| retired, dismissed | `party.departed` | no | no | Act 2 roster management (5.2); no retirement bonus (Q-40) |
| lost | `party.departed` | no | no | reserved for Calcification Stage V (the Calcification spec) |
| pooled | `party.pool` | no | no | a rescue waiting to be sworn in (5.2) |

- L1. Departed members keep their whole `UnitState` (choices, lean, gear record) so endings can count the dead and
  retired (Q-19a, CAMPAIGN1 M9) and node 20 can name the captured founder. Member ids are never reused: `recruit`
  of a departed character is refused ("`{name}` is gone.").
- L2. `roster::depart(d, gs, member, life, cause)`: removes the member from `members` and `active_ids`, frees its
  bunk, returns its gear and pouch per RQ2, records `{member, life, cause, encounter, day}` in `party.departed` (day
  = `gs.time`), then pays the keepsake (6). One call path for every departure.

### 3.2 Battle end: fates to states (`roster::settle_party`, replaces the GUI write-back)

The world settles the party once per battle, in the world crate, on every runner (`explore`, `travel`, `dialogue`
`after_fight`); the GUI only passes the session's fates. Fates per hero (guests excluded): ENCOUNTER2 S3's
`escaped`, `stayed`, `stood`, `downed`, `standing`, plus `dead` (bled out, 3.3). "The field is held" = outcome
`won` and the objective is not `escape`.

| Fate | Outcome | Rule key | Allowed values |
| --- | --- | --- | --- |
| stayed, stood | any | encounter `stay_behind.then` | `none` (then as `downed` if at 0 HP, else as `escaped`), `capture`, `kill` |
| dead | any | - | dead |
| downed | won, field held | `downed_won` | `classic`, `wounded` |
| downed | won by escape, or fled | `downed_left` | `classic`, `wounded`, `captured`, `dead` |
| downed | lost | `downed_lost` | `classic`, `wounded`, `captured`, `dead` |
| standing, escaped | lost | `standing_lost` | `classic`, `wounded` |
| standing, escaped | won, fled | - | HP only (`hp_after`) |

- L3. `classic` is today's HEAL1 result for that case (rally 25% on a win, 1 HP on a flee, the wake heal on a loss),
  so a world with no rule row settles exactly as now.
- L4. HP of members who come home: `hp_after` `carry` (HEAL1 `hp_after`, `hp_lost` against the unwounded max) or
  `full` (every member back to full; wounded is then the only carry-over). Which one the new worlds use: RQ1.
- L5. Order: `then`, then the table, then HP, then the keepsakes, then CAMPAIGN1's battle tick (its retreat R4
  calls `settle_party` with outcome `lost` before it moves the cell).
- L6. The pick of the rule row: `roster.rules` is a list; the row used is the first whose `acts` is empty or holds
  `campaign.act` (CAMPAIGN1 11.4; 0 when the world has no campaign). No row = classic.

### 3.3 Bleed-out and revive in battle `[kit hook]` (ROSTER1-B, Act 2)

- B1. `Session.bleed_out_rounds` (0 = off, today): the world passes the active rule row's value; sims, the Dojo and
  the host default to 0.
- B2. A hero (not a guest) dropping to 0 HP gets `bleed` = N. At each round end, after the status steps, every downed
  hero's `bleed` drops by 1; at 0 it bleeds out: fate `dead`, log "`{name}` bleeds out.", never revivable; it keeps
  its tile. Enemies never bleed out (their fates are unchanged).
- B3. Revive (RQ3): a skill heal or an item with `revive: true` may target a downed, not bled-out, ally in its reach;
  it heals `max(heal, 1)`, clears `bleed`, and the unit is active again from the next block (its plan was dropped
  when it fell). A heal without `revive` still refuses a downed unit (`world.rs:142-148`). Under the timeline ruleset
  a downed ally is a valid Support target only for `revive` skills and items (an addition to RULESET1's target rule).
- B4. Verdicts are unchanged: a bled-out hero is not alive, as a downed one is today.
- B5. Inspect and the downed hero's tell show "Bleeds out in `{n}`" (game screen task ROSTER1-BU).

## 4. Conditions: wounded `[kit hook]` (ROSTER1-L)

- W1. A member carries `conditions`, a list of ids; ROSTER1 defines one, `wounded`. Applying it twice keeps one
  (RQ5).
- W2. Field HP: at battle build a wounded member's max HP is `max - roundi(max × wounded_max_hp_pct / 100)` (row key,
  default 20, Q-31) and its start HP is `min(start_hp(max, hp_lost), that max)`; HP is written back against the
  unwounded max. `roster::field_unit(d, gs, member)` builds it; the battle screen calls it instead of
  `progression::build` plus `start_hp` (`gui/battle.rs:576-578`).
- W3. A rest clears `wounded` for every member: `progression::rest` (`progression.rs:266`) calls
  `roster::on_rest`, so every rest path clears it (rest places, Camp, CAMPAIGN1 R2). The lost-battle retreat is not
  a rest (CAMPAIGN1 R4 and `sunk_ward`'s `rest.on_enter` false keep Q-20's "all wounded" true on arrival).
- W4. Wounded members deploy (Q-31); no condition blocks deployment in this spec.

## 5. Bunks, the pool and recruits

### 5.1 Bunks `[kit hook]` (ROSTER1-L)

- BK1. Bunk cap = `bunks` + the int flag `bunks_flag` (clamped at 0), at most `bunks_max`. `bunks` 0 (default) =
  `party_max`: today's rule. The 8-bunk project is data first (layer rule): a Bram dialogue choice that takes Scrap
  and runs `add_flag` on `bunks_flag` (+2), gated on `campaign.act >= 2` (Q-38); the facility level hook replaces
  it later.
- BK2. Bunk use = members with `housing` `bunk` (default) or `crypt` (Q-36: a crypt place keeps the bunk); `crypt_only`
  (Living Martyrs) uses a crypt place and no bunk. Crypt places: `crypts` (default 0, RQ4).
- BK3. `recruit` refuses when bunk use >= cap: with `bunks` 0 the RECRUIT1 text "The party is full (`{n}` of
  `{cap}`)."; else "Every bunk is taken (`{n}` of `{cap}`).". Pool entries and hub guests take no bunk.
- BK4. A new member joins `active_ids` only when that list is non-empty and shorter than `max_active`; else it waits
  on the bench (fixes `dialogue.rs:729-731` once bunks exceed `party_max`).
- BK5. Join level (CAST1 CQ14 and its gap G13): `recruit` key `level`: `highest` (default, RECRUIT1), `average` (the
  active members' mean level, rounded down) or 1-20.
- BK6. Minimum party: encounter key `party_min` (0-4, default 0). The dialogue `fight` precheck refuses "Take
  `{n}` into this fight (`{k}` in the party)."; the battle start refuses with the same text (the Crucible: 4).
- BK7. The resolved roster settings (cap, open flag, crypts) sit on `PartyState` like `max_active` (never saved), so
  read keys need no `Data`.

### 5.2 The pool and recruits `[kit hook]` (ROSTER1-P, Act 2 start)

- PO1. Effect `rescue {template | character, base_unit, count 1-4}`: adds pool entries; a template entry is generated
  at once (5.3), a `character` entry is a named rescue or hub guest. Under-18 rescues are not pool entries: data
  counts them in the saved counter (9).
- PO2. Swearing in (effect `recruit {pool: id | "first"}`, or `roster::swear_in`): needs `open_flag` true (or ""),
  a free bunk (BK3) and, for a character, `party_character_errors` clean (role `party`; Q-39 decides the guests).
- PO3. Roster management beyond party selection (swear in, dismiss, retire, housing) refuses until `open_flag`
  holds: "The cell takes no one new yet." Dismiss departs `dismissed`, retire departs `retired`; gear to the stash
  (Q-40). The Crucible sets the flag (Q-93); scripted founders' `recruit` (Dunstan, Elyse) is never gated.
- PO4. The first procedural recruit (Q-93 (a)): the act transition event runs `recruit {pool: "first"}`, or `rescue`
  then `recruit` when nobody waits; who picks among several: RQ8.

### 5.3 Procedural recruits `[kit hook]`, `[data]` templates (ROSTER1-P)

- PG1. World kind `recruits` (templates): `base_units` (the class set by the encounter, Q-40), `names`, `origins`
  (passive ids: "a small stat or skill", Q-40), `temperaments`, `level` (as BK5), `fuse` (8).
- PG2. Generation draws from a new seeded `rolls` stream `recruit` (`rolls.rs`): base unit (or the effect's
  `base_unit`), a name not held by any member, pool entry or departed member, an origin, a temperament. Id
  `recruit_{n}` (n = 1 + the highest used). The display name is saved on the member (`name`); `display_name` reads
  it first.

## 6. Keepsakes `[kit hook]`, `[data]` items

- KS1. When a member departs with a life listed in the row's `keepsakes.on` (default empty; the new worlds:
  `captured`, `dead`, Q-34), the cell receives its keepsake once (the departed record keeps the paid id).
- KS2. Named (ROSTER1-L): the member's character has `keepsake`, a catalog item id; the stash gets one. The rows
  and names are CAST1's (`games/emberwake/docs/specs/CAST1_GEAR.md` 7, `keepsake_<founder>`; Q-35 names there). The
  content check: an `ACCESSORY`, `value` 0 (never sold, `shop.rs:197`), `special` true.
- KS3. Procedural (ROSTER1-K): `keepsakes.by_class` maps a class id to a template item; the cell gets an instance
  `{id: "keepsake@{member}", item, name}` in `party.keepsakes`, named by `keepsakes.name` ("`{name}`'s `{item}`").
  Equipment holds the instance id; `roster::item_def` resolves it (the template row renamed, as `named_item` does,
  `world/battle.rs:44-71`) for builds, `can_wear`, validation and the party screen. Instances are never in the stash,
  never sold, never dropped; two keepsakes of one template on one member count as a duplicate accessory.
- KS4. Strength (Q-34 (a)): a named keepsake carries about 50% of the founder's trait and is far above shop
  accessories (CAST1_GEAR 7); procedural templates sit below named ones; numbers by the balance lane.
- KS5. The Crucible's keepsake: CAST1 pays it through loot rows at the Act 2 opening; KS2 pays it at the capture.
  One path must win: RQ7.

## 7. Temperament `[kit hook]`, `[data]` rows (ROSTER1-T; Q-40a (c))

- TP1. Content kind `temperaments`: a slotless passive, a default intent and AI preferences (schema 11.5).
- TP2. A member's temperament comes from its character's `temperament` (founders, named recruits) or its recruit
  template (procedural) and is saved on the member.
- TP3. The passive is source-granted (`slot` ""), always on, in no deck slot (RULESET1 D3, D4).
- TP4. Default intent (RULESET1 TQ23 (a)): the member's own `default_intent`, else the temperament's, else the unit's,
  else none.
- TP5. AI: `roster::ai_profile(d, member)` = the base unit's `AiProfile` with the temperament's `ai` keys over it;
  party autoplay and RULESET1's AI5 table read it (`gui/battle.rs:588` calls the world).
- TP6. Q-40a's "Reckless: more damage at low HP" needs a stat context the grammar lacks (only `weapon` today,
  `stats.rs:105`): RQ9.

## 8. Living Martyrs: the fuse hook `[kit hook]` (ROSTER1-M, Act 2, design only)

- FZ1. A recruit template's `fuse {per_deploy, stasis_days, max, tiers: [{at, passive}], last_passive}` is copied
  onto the member as `fuse {value, deploys, since}` (`since` = the join day).
- FZ2. `value = deploys × per_deploy + floor((gs.time - since) / stasis_days)`: it ticks on every deployment and in
  stasis on the Doom Clock's days (Q-37a (a)); nothing lowers it and nothing moves it to another member (Q-37).
- FZ3. The highest tier with `at <= value` applies its passive (the bonus scales); tiers map onto Calcification
  stages when the Calcification spec lands.
- FZ4. At `max`: before its next deployment the player picks retire (departs `retired`) or a last fight: it deploys
  with `last_passive` (Stage V) and departs `dead` after that battle, whatever the outcome (Q-37).
- FZ5. Martyrs live in the Isolation Crypts with no bunk (`housing` `crypt_only`, Q-36).

## 9. The saved counter and read keys

- SY1. The saved counter (Q-29) is the int flag `youths_saved`, raised by the existing `add_flag` effect on the
  rescue's win; scenes and endings read it with flag conditions. No code; a CAMPAIGN1 meter only if a HUD ever
  shows it. Who counts: RQ6.
- RK1. Read keys (answered by `quest::condition_met`, `quest.rs:18`, beside CAMPAIGN1 11.4; `set_flag` already
  refuses the `roster.` prefix there): `roster.living` (members: bunks plus crypts; CAMPAIGN1 F2's wiped-cell
  check), `roster.bunks`, `roster.bunks_free`, `roster.pool`, `roster.open` (bool), `roster.departed.{life}`
  (count), `roster.{member id}` (its state: `ready` or a departed life; "" when unknown). Member ids equal to
  those key words are refused.

## 10. What the slice needs

| Piece | Slice | Task |
| --- | --- | --- |
| Bunks apart from `party_max` (Dunstan joins as the fifth member) | yes | L, D |
| Wounded: Act 1 downed and lost battles, cleared by a rest | yes | L, U, D |
| The Crucible holder captured (`then: capture`), the bunk freed | yes | L, ENCOUNTER2-W, D |
| The Crucible's keepsake | yes (RQ7 picks the path) | L or CAST1 loot rows |
| `party_min` 4 for the Crucible; `open_flag` set at its end (read later) | yes | L, D |
| Saved counter `youths_saved` (M2) | yes (data) | SUNKWARD1 or D |
| Temperament | only if CAST1 gives founders temperaments | T, TA |
| Bleed-out, revive, the pool, procedural recruits and keepsakes, crypts, the Martyr fuse, the Bunks screen | later (Act 2) | B, BU, P, K, M, RU |

## 11. Schema contract (schema-first: yes)

### 11.1 World manifest `roster` (json, default {}; resolved from the nearest manifest in the lineage with one)

| Key | Type | Default | Validation (error text) |
| --- | --- | --- | --- |
| `bunks` | int | 0 | 0 or 1-12 ("Bunks must be 0 (party max) or 1..12.") |
| `bunks_flag` | string | "" | a flag key, not `roster.`, `meter.` or `campaign.` |
| `bunks_max` | int | 12 | `bunks`..12 |
| `crypts` | int | 0 | 0-12 |
| `open_flag` | string | "" | a flag key |
| `rules` | json | [] | 0-4 rows (11.2) |
| `keepsakes` | json | {} | `on` (lives from `captured`, `dead`, `lost`), `by_class` (class id -> item id), `name` (1-40 chars, tokens `{name}`, `{item}`) |

### 11.2 Rule rows (`roster.rules[]`)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `acts` | [int] | [] | each 0-9; at most one row with empty `acts`, and it is last |
| `bleed_out_rounds` | int | 0 | 0-6 |
| `hp_after` | enum | `carry` | `carry`, `full` |
| `downed_won` | enum | `classic` | `classic`, `wounded` |
| `downed_left`, `downed_lost` | enum | `classic` | `classic`, `wounded`, `captured`, `dead` |
| `standing_lost` | enum | `classic` | `classic`, `wounded` |
| `wounded_max_hp_pct` | int | 20 | 1-90 |
| `keepsakes_on` | [enum] | [] | overrides `keepsakes.on` for this row when set |

### 11.3 Other fields

| Kind, key | Type | Default | Validation |
| --- | --- | --- | --- |
| `world.characters.keepsake` | ref items | "" | an `ACCESSORY` with `value` 0 and `special` true; role `party` only |
| `world.characters.temperament` | ref temperaments | "" | resolves (ROSTER1-T) |
| encounters `party_min` | int | 0 | 0-4 and at most the world's `party_max` |
| encounters `stay_behind.then` | enum | `none` | `none`, `capture`, `kill` (replaces ENCOUNTER2's `leave_party`) |
| skills (heal) and items `revive` | bool | false | only on a heal (ROSTER1-B) |
| effect `recruit` | + `level`, `pool` | `highest`, "" | `level` as BK5; `pool` a pool id or `first`, not with `character` |
| effect `rescue` | `template` or `character`, `base_unit`, `count` | -, "", 1 | exactly one of the first two; `count` 1-4 (ROSTER1-P) |
| `world.recruits` (new kind) | `base_units`, `names`, `origins`, `temperaments`, `level`, `fuse` | | refs resolve; `names` 1+; party base units only (ROSTER1-P) |

### 11.4 Save keys (written only when non-empty, so Godot saves still compare equal)

| Where | Key | Shape |
| --- | --- | --- |
| member | `conditions`, `temperament`, `name`, `housing` (only when not `bunk`), `fuse` | [string], string, string, enum, `{value, deploys, since}` |
| party | `departed` | `[{member, life, cause, encounter, day, keepsake}]`; `cause` in `stayed`, `stood`, `downed`, `bled_out`, `lost_battle`, `story`, `roster` |
| party | `pool`, `keepsakes` | `[{id, character, base_unit, name, origin, temperament, since}]`, `[{id, item, name, from}]` |

Validation (`PartyState::validate`): ids unique across members, pool and departed; lives and conditions known;
bunk and crypt use within their caps after load (a warning, never a refusal); each worn keepsake instance exists and
is worn once. Version: the shared unreleased bump (CAMPAIGN1 11.5 names v13; one bump for CAMPAIGN1, ROSTER1,
RULESET1-D and ENCOUNTER2-W, whichever lands first adds the empty step).

### 11.5 Content kind `temperaments` (`data/catalog/temperaments.json`, ROSTER1-T)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `id`, `name`, `description` | id, string, text | | as passives |
| `passive` | ref passives | "" | `slot` "" |
| `default_intent` | enum | "" | "", `assault`, `guard`, `support`, `maneuver` |
| `ai` | json | {} | keys `target_pref`, `defend_below`, `heal_below`, `flee_ratio`, `reckless`, ranges as the unit fields |

Content tool path: `tools/fc.sh content upsert <kind> @file.json` (`world=<id>` for world kinds), or a seed through
`tools/fc.sh content apply`; then `tools/fc.sh content check`; the Content Forge edits the json fields raw.

## 12. Out of scope

The Bunks and Facility screens and facility levels (Act 2, Q-96, Q-42); hub guests' perks; the Calcification track
and Stage V drift (the Calcification spec); Strain; the Apex organ harvest from Martyrs; the Transcendent Relic's
keepsake upgrade (Act 3, an exchange recipe); grief traits (Q-40: none); the corrupted-founder encounters (ENCOUNTER2
4.3, EQ7); rescuing a captured founder (Q-33: never); the doctrine lean (CAMPAIGN1 M9); balance numbers (keepsakes,
bleed-out rounds, wounded percent after the baseline).

## As built (ROSTER1-L slice 1, 2026-10-31)

- Slice order: the schema lands first (11.1-11.3 and RK1's keys, checks and typed readers, no behaviour), so the asset
  lane can write ROSTER1-D; the refactor of `ROSTER1_TASKS.md` 2 ("L's first commit") is slice 2, the first code
  slice, still alone and with no rule change.
- `world.characters.keepsake` refs a catalog item (`base:items`); the item rule (an `ACCESSORY`, `value` 0, `special`)
  also applies to each `keepsakes.by_class` item. Like every world entry rule, a bad keepsake is a content-check
  error, not a refused write.
- `party_min` carries the schema range 0..4, which clamps on normalize; the check adds "at most the world's party max".
- `bleed_out_rounds` above 0 is accepted and stays inert until ROSTER1-B.
- `roster.<member>` in a condition must name a character of the world, a party-side unit or `recruit_<n>`; a
  character whose id is a roster word (`living`, `bunks`, `bunks_free`, `pool`, `open`, `departed`) is refused.
- One "only the last row may have no acts" error covers both 11.2 cases (two open rows, an open row before another).

## As built (ROSTER1-L slice 2, 2026-10-31)

- `field_unit(d, member, world)` takes the battle's world (the played world, or the connected world whose encounter
  is fought, as the battle screen picked it before) and no game state yet; the wounded build (W2) adds what it needs
  in slice 3. `settle_party(d, gs, outcome, ends)` takes `HeroEnd`s from `hero_ends(session, members)`, which pair
  the fielded members with the session's allies in order (guests follow and have none) and carry ENCOUNTER2 S3's
  fate; the pouch write-back (ITEMUSE1 rule 7) moved with the HP write-back.
- The battle screen still calls `settle_party` just before the runner's `after_fight` (L5's order: the settle comes
  before CAMPAIGN1's tick); moving the call into the runners comes with the rule rows (slice 3) or ENCOUNTER2-W.
- No parity suite builds saved members for a battle (`journey` settles without one, `world_traces` and `replay` play
  the encounter's own party), so `rpgkit_world/tests/roster.rs` keeps the old screen build and write-back as its
  reference; `journey`, `world_traces` and the saves suites stay green.
