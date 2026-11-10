# Design session PK2: Calcification, grafts, weapon techniques and durability (`PK2.1`, 2026-11-10)

Status: `.1` design written by the Design Session (2026-11-10) from `docs/specs/ACTS23.md` sections 4, 5.1 and 5.2, CANON 2.2-2.5 and Build's `ACTS23-R5` feasibility (`FB-167`, which sized every hook here); open points in section 9 take the listed defaults unless the user changes them. Release `R5` with PK1 and PK3.
Layers: kit (all hooks story-free: "unit track", "body slots", "recipes", "techniques", "weapon instances", "durability"), game (data: Calcification stages, graft slots and organs, recipes at the Slab and the Hearth, techniques per family, durability numbers).
Extends: `E3` roster, `U3` gear, `U4` grants, `W1` services, `E2` purse and stash.

## 1. Two branches, two currencies (the frame)

| | Inoculation (marrow) | Cold Steel |
| --- | --- | --- |
| Power-up slots | 4 graft body slots (section 3) | technique slots per weapon family (section 5) |
| Where | Aline's Slab (surgery recipes) | Bram's Hearth (assignment, repair and reforge recipes) |
| In battle | marrow versions, graft skills (Channel rounds, PK1) | techniques (Cold Steel rounds only) |
| Running cost | Strain (PK1) | weapon durability (section 7) |
| Lasting cost | Strain floor, Calcification (section 2) | resources to repair and reforge |
| Keystone theme | corruption (Inoculated keystones) | weapons (Cold Steel keystones) |
| From | Act 2 | Act 2 (nothing here exists in Act 1) |

## 2. Calcification: the unit track (kit hook (d))

- **Kit**: a per-member **unit track** (unit-scope campaign meter, the refused CAMPAIGN1-DOC, `check/campaign.rs:287`): an integer stage per member, saved as an additive member key (no save VERSION bump). A track row (game data `unit_tracks`) names its stages; each stage carries a **slotless passive** (granted while the member is at that stage) and an optional **departure** (`lost` at the last stage).
- **Game data**: `calcification`, stages I-V (CANON 2.3):

| Stage | Name | Passive |
| --- | --- | --- |
| I | Vein-Blackening | -5 Composure, +5% debuff resistance |
| II | Calloused Chitin | +2 armour rating, -1 tile per move block |
| III | Sensory Rot | immune to fear and morale debuffs; -15% ranged damage beyond 3 tiles |
| IV | The Shakes | cannot equip two-handed weapons (bows and arbalests included); melee and bare-handed attacks pierce armour; sleeps in the Isolation Crypts (housing, PK3) |
| V | Terminal Drift | lost (departure `lost`, cause `calcified`) when the battle settles; in battle the unit turns hostile at once (PK1 2.1) |

- **Advances** (CANON 2.3, and only these): a `calcify` **mark** from a Flesh Breach (PK1; the stage takes hold from the next round, saved at settle unless undone by `unmark`); **Tier III graft surgery** (+1 at install, section 3); installing an **Inoculated keystone** (Tier III surgery, +1, Q-64).
- **Undo**: Quenched Cinder Salt and the priest's Inoculated keystone in battle (PK1 `unmark`, before settle); **bone-scraping** at the Slab after Aline's quest: -1 stage and -10% max HP for good (section 4).
- **Kit pieces**: a member grant `forbid_hands: 2` (Stage IV: equipping refuses two-handed items; an equipped two-hander goes to the stash at settle) and `pierce_melee` (melee and bare-handed hits pierce); a member **max HP offset** (permanent, from bone-scraping and excision).
- **Living Martyrs** (Stage IV volunteers, Q-37) and the crypts are PK3; this track gives them their counter.

## 3. Grafts: body slots (kit hook (e))

- **Kit**: a second slot set, **body slots**, separate from gear (`LOADOUT_SLOTS` untouched, a new member key `body`: slot -> item id), with its own effect list (body items may grant a meter and a floor; gear still may not, `Grant::gear_ok`).
- **Game data**: 4 slots, Cranial, Thoracic, Forearms, Visceral (CANON 2.1). A **graft** is an item of a new item type `graft` with `body_slot`, `tier` (I, II, III) and grants:
  - its **marrow-exclusive skill** (`grant_skill`, a skill with `channel.only`, single intent, budgeted like a dual-intent Cold Steel heavy, ACTS23 5.1);
  - the **Strain meter** (holding any graft makes the unit a Strain carrier);
  - an **additive floor** step (`meter_floor`, body-only; default +10 per graft, per-organ in data, Q-44; floors from body items add up, capped at 50, CANON 2.2; `FB-167` Q6).
- **Tier III** (Apex organs, for example the Titan Marrow Core) adds +1 Calcification stage at install.
- **Inoculated keystone**: slotted in the one keystone slot by surgery (Tier III, +1 stage); while slotted the Strain floor is at least 30 ("keystones lock it at 30").
- **Marrow-active limit** (PK1 `marrow_actives`): one marrow active per installed graft, up to the 3 actives.
- **Per-member floor offset** for the Rock Salt flush (section 4): `floor_offset` on the member, applied to the next battle, then cleared.

## 4. Recipes: the exchange service (kit hook (f))

- **Kit**: catalog kind `recipes` and an NPC service `recipe` (beside turn_in, offer, board, shop, recruit; `explore.rs:315`) shown on the SERVICES1 `ServiceScreen`. A recipe row: `id`, `name`, `npc` (who offers it), `requires` (flags, quest steps, act), `cost` (purse amounts and stash items), `target` (`member`, or `weapon` for section 7), and `effects` from a member-effect list: `install` (an item into a body slot), `remove` (out of a body slot), `track` (add to a unit track), `max_hp_pct` (permanent offset), `floor_offset`, `repair`, `reforge`, `assign_technique`, `clear_technique`, `doctrine` (a delta PK4 reads; reserved now).
- **Game data at the Slab (Aline)**: graft surgery per organ (organ item + Scrap; Tier III also `track +1`; `doctrine` toward Flesh-Broker); **excision** (remove a graft, `max_hp_pct -15`); **Rock Salt flush** (Rock Salt, `floor_offset -10` for the next battle; toward Preservationist); **bone-scraping** (after Aline's quest; `track -1`, `max_hp_pct -10`).
- **At the Hearth (Bram)**: technique assignment (section 5), repair and reforge (section 7), all toward Preservationist.

## 5. Weapon techniques (ACTS23 5.2, user rulings)

- **Stored per member, per weapon family** (member key `techniques`: family -> skill ids); active whenever that family is the active weapon; no per-item state.
- **Slots** = the family's mastery level (proficiency 1-3, `stats.rs:222` `level_of`) **+ 1 while the wielded weapon instance is masterwork** (section 6): up to 4.
- **Source**: each weapon family row lists its technique skills (`techniques`, game data); a technique is single intent, budgeted like a dual-intent Cold Steel heavy, and usable **only in Cold Steel rounds** (refused in a PK1 Channel round).
- **Assignment** at the Hearth through recipes (`assign_technique`, Scrap and materials); clearing a slot is free, assigning again pays again (default, section 9).

## 6. Weapon instances (kit, the largest piece)

- **Weapons only** become instances (main-hand and off-hand weapons; armour, accessories and consumables stay plain ids): `{uid, item, durability, masterwork}`.
- **Switched on by act** (`FB-167` Q7 (a)): manifest `durability.from_act` (game data: 2) plus kit rule `durability`. At the act line every owned weapon (equipped or in the stash) becomes an instance at full durability; later shop buys and loot create instances. Act 1 saves and data are untouched; the keys are additive, so **no save VERSION bump**.
- Equipment and the stash refer to a weapon instance by `uid` once it exists; the catalog row stays the template (the content writer is unchanged).

## 7. Durability (ACTS23 5.2, user rulings)

- **Max durability** per family (data, overridable per item); a fresh instance starts full.
- **Costs** (from Act 2): every Cold Steel **attack** skill used with the weapon costs durability (default 1 per use); **techniques cost more** (default 2); **basics, support skills, items and every marrow action cost none** (marrow pays Strain).
- **At 0** the weapon is **dulled**: still usable, techniques included, with one heavy penalty (default weapon power -50%); no brackets in between. **Sim alternative** for PK2 balance: a dulled weapon also locks its techniques.
- **Repair** (Hearth recipe, Scrap and materials, cost scaling with the missing durability) restores it; **reforge** (rarer material) restores it and can make it **masterwork** (+1 technique slot).
- The battle tracks spending per instance and returns it through `HeroEnd` at settle.

## 8. Slices (proposed, after Build's feasibility numbering)

| Slice | Scope | Acceptance |
| --- | --- | --- |
| `PK2-S1` (Build's PR5-8) | Unit track, stage passives, `forbid_hands`, `pierce_melee`, max HP offset, the `mark` settle (with `unmark`), Stage V departure | unit tests; **0 golden diffs** (no data) |
| `PK2-S2` (PR5-9) | Body slots, item type `graft`, additive body-only `meter_floor` (cap 50), the keystone floor 30, `floor_offset`, the marrow-active limit from grafts | unit tests; checks_golden only if a new item type joins a listed message (named) |
| `PK2-S3` (PR5-10) | Recipe catalog and service, member effects | unit tests; 0 golden diffs |
| `PK2-S4` (PR5-12) | Techniques per family, slots from mastery (+masterwork), Cold Steel-round gate (after PR5-4) | unit tests; 0 golden diffs |
| `PK2-S5a` (PR5-13a) | Weapon instances behind `durability.from_act` | unit tests incl. the act-line conversion and an Act 1 save loading unchanged; **0 journey diffs** |
| `PK2-S5b` (PR5-13b) | Durability spend, dulled penalty, repair and reforge | unit tests; 0 golden diffs |
| `PK2-S6` | Data through `aegis content` (track, slots, organs and grafts per class, Slab and Hearth recipes, techniques per family, durability values) and a balance run (Cold Steel vs marrow, the dulled alternative) | data checks clean; sim report |

## 9. Open points with defaults (logged in `docs/DESIGN_REVIEW.md`)

1. Reassigning a technique slot: clearing free, assigning pays again.
2. Durability numbers: attack 1, technique 2, dulled -50% power; per-family max to the balance lane.
3. Loot weapons drop at full durability.
4. Graft tiers: Tier I +10 floor; Tier II +10 floor and a stronger skill; Tier III +10 floor, the strongest skill, +1 Calcification.
5. A two-hander equipped at Stage IV goes to the stash at settle.
6. Technique list size: 3-4 per weapon family at first.
