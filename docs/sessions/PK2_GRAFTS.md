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

- **Advances** (CANON 2.3, and only these): a `calcify` **mark** from a Flesh Breach (PK1; the stage takes hold from the next round, saved at settle unless undone by `unmark`); **Tier III graft surgery** (+1 at install, section 3), the Inoculated keystone included (Q-64).
- **Undo**: Quenched Cinder Salt and the priest's Inoculated keystone in battle (PK1 `unmark`, before settle); **bone-scraping** at the Slab after Aline's quest: -1 stage and -10% max HP for good (section 4).
- **Kit pieces**: a member grant `forbid_hands: 2` (Stage IV: equipping refuses two-handed items; an equipped two-hander goes to the stash at settle) and `pierce_melee` (melee and bare-handed hits pierce); a member **max HP offset** (permanent, from bone-scraping and excision).
- **Living Martyrs** (Stage IV volunteers, Q-37) and the crypts are PK3; this track gives them their counter.

## 3. Grafts: body slots (kit hook (e))

- **Kit**: a second slot set, **body slots**, separate from gear (`LOADOUT_SLOTS` untouched, a new member key `body`: slot -> item id), with its own effect list (body items may grant a meter and a floor; gear still may not, `Grant::gear_ok`).
- **Game data**: 4 slots, Cranial, Thoracic, Forearms, Visceral (CANON 2.1). A **graft** is an item of a new item type `graft` with `body_slot`, `tier` and grants: its **marrow-exclusive skill** (`grant_skill`, a skill with `channel.only`, single intent, budgeted like a dual-intent Cold Steel heavy, ACTS23 5.1) and the **Strain meter** (holding any graft makes the unit a Strain carrier).
- **Three surgery tiers** (user 2026-11-10). **A graft teaches the monster's own skill** (user 2026-11-10, replacing the per-class Inoculated branches): each aberrant or elite enemy that drops an organ names the skill it teaches (its signature move as a marrow-only skill), usable by any class; the tier follows the source:

| Tier | Organs | Teaches | Extra cost |
| --- | --- | --- | --- |
| I | standard organs from ordinary aberrations (Stalker Optic Nerve, Caustic Gall-Bladder) | that monster's skill (T1 budget) | none beyond the floor |
| II | elite organs from elite aberrations and Inquisitors (Inquisitor Marrow Valve) | that elite's skill (T2 budget) | none beyond the floor |
| III (Apex) | Apex organs from bosses and Marrow Horrors (Titan Marrow Core, a unique named organ); Aline offers Apex grafting after mission 2.4; **any Tier III surgery needs at least 3 grafts already** (user 2026-11-10) | that boss's skill (T3 budget), or the class's **Inoculated keystone** | +1 Calcification stage at install |

- **The Inoculated keystone is a Tier III graft** (Q-64: there is no other way to get it): made at the Slab from an Apex organ, **one per unit**, and like every Tier III surgery only for a unit with **at least 3 grafts** (user 2026-11-10). The keystone graft **occupies a body slot** (it counts as a graft) and puts its skill in the deck's one **keystone slot**. It goes into the **Cranial** slot (class pieces only), so it needs the class ritual too. Two ways in (user 2026-11-10): **add** it into an empty Cranial slot as the 4th graft (all skills kept plus the keystone, floor 50), or **overwrite** a class skill already in Cranial (that skill is lost, the count is unchanged). Each class has its own Inoculated keystone (the priest's carries the in-combat Flesh Breach undo, PK1).
- **Class-specific Inoculated skills** (user 2026-11-10): a few per class; the **class ritual** (a quest) unlocks them as Slab surgery options (a recipe that needs the ritual flag plus an organ) and opens the Cranial slot, where they are installed (they count toward the floor). Most Inoculated skills still come from the aberrations themselves.
- **Slot roles** (user 2026-11-10, replacing "generic slots"): the 4 body slots split into **who you are** and **what you hunted**:

| Slot | Opens | Holds | Flavour |
| --- | --- | --- | --- |
| **Cranial** | after the operative's **class ritual** | **class pieces only**: the class-specific Inoculated skills and the class's Inoculated keystone | class identity |
| Forearms | from Act 2 (when grafting starts, after the Crucible) | any monster organ | strikes, claws |
| Thoracic | from Act 2 | any monster organ | hide, breath, endurance |
| Visceral | from Act 2 | any monster organ | blood, bile, sustain |

  **Surgery materials by slot** (user 2026-11-10): a surgery uses two loot pieces: the **core** (the monster's core organ, which teaches its skill and sets the tier; any core fits any monster slot) and a **slot material** matching the body part it goes into, so every part a monster drops counts:

| Slot | Slot material (examples) |
| --- | --- |
| Forearms | claws, talons, bone spurs |
| Thoracic | hide, chitin plates, scale |
| Visceral | bile, gall sacs, blood sacs |
| Cranial | nerve and eye tissue (the class ritual's rite) |

  The three monster slots have no hard organ restriction (their roles are flavour for which organs read best there). A graft's power depends on its organ's tier, not its slot; all four count toward the floor ("graft count"). So a unit with three monster grafts that wants its keystone does the class ritual and grafts the keystone into Cranial as its 4th graft (floor 50), or, with Cranial already holding a class skill, overwrites that class skill (count unchanged).
- **A fallen named founder's Apex organ** teaches a marrow version of that founder's signature skill (Design default, following "the monster's own skill").
- **The floor follows the number of grafts, steepening** (user 2026-11-10; replaces "+10 per graft" and the keystone's "at least 30"): grafts counted are the body slots in use (the keystone graft is one of them, so at most 4); the floor is set from the count, not summed per item (kit: a count-to-floor table on the meter grant; the per-member offset of `FB-167` Q6 stays):

| Grafts | 0 | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- | --- |
| Floor | none (not a Strain carrier) | 5 | 15 | 30 (starts in Acute Resonance) | 50 (starts in Flesh-Tension, the cap) |

- **Inoculated skills come only from surgery** (user 2026-11-10): each surgery teaches its organ's skill, so a unit's Inoculated skills are exactly the grafts in its body (and its keystone).
- **The surgery process**: harvest an organ (loot; tier by source) -> at the Slab pick the core, the slot and its slot material, pay Scrap, pick the operative -> a monster organ goes into a free Forearms, Thoracic or Visceral slot; class skills and the keystone go into Cranial after the class ritual (any Tier III surgery needs 3 grafts first) -> the operative gains the organ's skill, becomes a Strain carrier and moves up the floor table; Tier III adds +1 Calcification; the operative starts the next mission bandaged (flavour). No surgery on anyone under 18 (CANON 2.4).
- **Replacing a graft** (user 2026-11-10): one surgery overwrites an occupied slot: the new organ goes in, the **old organ is destroyed** (no refund), **no HP cost** (the slot stays filled), the graft count and floor stay the same, and a Tier III organ still adds +1 Calcification; so a slot can be upgraded Tier I -> II -> III. **Excision** (leaving the slot empty) keeps its -15% max HP for good and also destroys the organ.
- **Losing a grafted unit** (user 2026-11-10): installed organs are lost with the unit. A **fallen body yields one organ** (`classes_and_skill_trees.md` "On death"): a named founder gives a unique named Apex organ, a recruit a standard class organ, added to the stash when the cell holds the field at the battle's end (a won battle, or a lost one where the fallen are recovered under the roster rules); **captured** units and **Stage V** losses leave no organ (no body), only their keepsake (PK3).
- **When the class ritual happens** (user 2026-11-10): for a **founder**, their Act 2 **loyalty mission** (CANON G-16) is their class ritual; a **recruit** does a shorter per-class **rite at the Slab** (a recipe with a rare material, available once a founder of that class has done their ritual, or from mid-Act 2 if none has).
- **Aline's marrow dose** (Act 1 interlude; user 2026-11-10, replacing the special case written earlier the same day): **the dose is a standard graft**, the Inoculation tutorial. The founder who takes it uses its marrow skill in the interlude fight (ENC-EVT-05, granted for that battle as encounter data); from Act 2 that founder simply **starts with one Tier I graft** installed in a monster slot (that same skill, as a marrow-only skill), so they are a Strain carrier like anyone with one graft (floor 5, one marrow active), and the graft and the Slab's surgeries become visible through them. No other rule. The earlier "starts Act 2 at Calcification I" (a planner pick in CANON 3.2) is dropped, since a Tier I graft does not calcify (Design default; the user may restore it). **Calcification tutorial** (user 2026-11-10): a scripted mission early in Act 2 (Aline's lesson): in a controlled fight the dosed founder's graft forces a Flesh Breach (an encounter phase effect that sets the subject's Strain to 100 at a round end, so the crisis table is skipped for a scripted `mark`), Aline explains Calcification and **hands over one Quenched Cinder Salt**, so the player chooses: use it (the stage is not saved, the boost lasts the fight) or keep the Salt and accept Stage I. **If the dosed founder was captured at the Crucible**, the subject is a **guest youth rescued at the Crucible** whom the Inquisition had dosed, with one graft, breaching on cue; that youth can then become the first recruit (Q-93).
 **If the dosed founder is the one left at the Crucible winch** (user 2026-11-10): allowed, and it pays off: at 2.4 Lazar's records note the captive "already carried the marrow", Aline has a guilt line, and at node 20 the corrupted founder fights stronger (their old skills as marrow versions, Strain and crises of their own); the player loses the Act 2 head start, which is that pairing's cost (story and encounter data, PK8).
- **Excising Cranial pieces** (Design default): as any excision, -15% max HP and the piece destroyed; removing the keystone frees Cranial.
- **Marrow-active limit** (PK1 `marrow_actives`): one marrow active per installed graft, up to the 3 actives.
- **Per-member floor offset** for the Rock Salt flush (section 4): `floor_offset` on the member, applied to the next battle, then cleared.

## 4. Recipes: the exchange service (kit hook (f))

- **Kit**: catalog kind `recipes` and an NPC service `recipe` (beside turn_in, offer, board, shop, recruit; `explore.rs:315`) shown on the SERVICES1 `ServiceScreen`. A recipe row: `id`, `name`, `npc` (who offers it), `requires` (flags, quest steps, act), `cost` (purse amounts and stash items), `target` (`member`, or `weapon` for section 7), and `effects` from a member-effect list: `install` (an item into a body slot), `remove` (out of a body slot), `track` (add to a unit track), `max_hp_pct` (permanent offset), `floor_offset`, `repair`, `reforge`, `assign_technique`, `clear_technique`, `doctrine` (a delta PK4 reads; reserved now).
- **Game data at the Slab (Aline)**: graft surgery per core and slot (core + the slot's material + Scrap; into an empty or an occupied slot, the replaced organ destroyed; Tier III, the keystone included, also `track +1`; `doctrine` toward Flesh-Broker); **excision** (remove a graft, organ destroyed, `max_hp_pct -15`); **Rock Salt flush** (Rock Salt, `floor_offset -10` for the next battle; toward Preservationist); **bone-scraping** (after Aline's quest; `track -1`, `max_hp_pct -10`).
- **At the Hearth (Bram)**: technique assignment (section 5), repair and reforge (section 7), all toward Preservationist.

## 5. Weapon techniques (ACTS23 5.2, user rulings)

- **Stored per member, per weapon family** (member key `techniques`: family -> skill ids); active whenever that family is the active weapon; no per-item state.
- **Slots** = the family's mastery level (proficiency 1-3, `stats.rs:222` `level_of`) **+ 1 while the wielded weapon instance is masterwork** (section 6): up to 4.
- **Source: human enemies drop them** (user 2026-11-10; the mirror of aberration cores): Inquisitors, Castellans and other human elites drop **technique manuals** (a new item type `manual`: its weapon family and the technique skill it teaches); at the Hearth Bram teaches a manual to one operative (it joins that operative's known techniques for that family; the manual is used up). A technique is single intent, budgeted like a dual-intent Cold Steel heavy, and usable **only in Cold Steel rounds** (refused in a PK1 Channel round).
- **Assignment** at the Hearth through recipes (`assign_technique`, Scrap and materials) from the operative's known techniques; clearing a slot is free, assigning again pays again.
- **Mastery grows by use, plus Hearth drills** (user 2026-11-10): a member key `mastery_xp` per weapon family; each battle in which the operative used that family (an attack with it) adds mastery XP at settle; **Bram's drills** (a Hearth recipe, Scrap) add more; thresholds lift the family's proficiency level (the class proficiency stays the floor) up to 3. Default: growth runs **from Act 2** (the same `durability.from_act` switch), so Act 1 play and its goldens are unchanged.
- **Cold Steel keystone: mastery 3 + Bram's forge-rite** (user 2026-11-10; the mirror of "3 grafts + the class ritual"): an operative with mastery 3 in a family can have Bram **forge the keystone into a masterwork weapon of that family** (a Hearth recipe with a rare material). The forge-rite opens for the whole cell through Bram's quest "Cold Slag, Black Iron" (Act 2, after Lucan's rescue, Q-27); a founder's loyalty mission also opens it for that founder (the same mission that is their class ritual, so one mission opens both paths and the player picks the build). The keystone sits in the deck's one keystone slot like the Inoculated keystone; a unit holds one keystone, of either theme.

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
| `PK2-S2` (PR5-9) | Body slots, item type `graft`, the floor from the graft count (5 / 15 / 30 / 50, the keystone graft in a body slot), the 3-graft gate for Tier III, `floor_offset`, the marrow-active limit from grafts | unit tests; checks_golden only if a new item type joins a listed message (named) |
| `PK2-S3` (PR5-10) | Recipe catalog and service, member effects | unit tests; 0 golden diffs |
| `PK2-S4` (PR5-12) | Techniques per family, slots from mastery (+masterwork), Cold Steel-round gate (after PR5-4) | unit tests; 0 golden diffs |
| `PK2-S5a` (PR5-13a) | Weapon instances behind `durability.from_act` | unit tests incl. the act-line conversion and an Act 1 save loading unchanged; **0 journey diffs** |
| `PK2-S5b` (PR5-13b) | Durability spend, dulled penalty, repair and reforge | unit tests; 0 golden diffs |
| `PK2-S6` | Data through `aegis content` (track, slots, organs and grafts per class, Slab and Hearth recipes, techniques per family, durability values) and a balance run (Cold Steel vs marrow, the dulled alternative) | data checks clean; sim report |

## 9. Open points with defaults (logged in `docs/DESIGN_REVIEW.md`)

User 2026-11-10: defaults 1, 2 (durability numbers), 3, 5 and 6 accepted; 4 was replaced by the tier and floor rulings in section 3.

1. Reassigning a technique slot: clearing free, assigning pays again.
2. Durability numbers: attack 1, technique 2, dulled -50% power; per-family max to the balance lane.
3. Loot weapons drop at full durability.
4. ~~Graft tiers +10 floor each~~: decided by the user (section 3: three organ tiers, the floor by graft count).
5. A two-hander equipped at Stage IV goes to the stash at settle.
6. Technique list size: 3-4 per weapon family at first.
