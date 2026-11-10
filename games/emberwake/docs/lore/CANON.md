# Emberwake: A Scarlet Lullaby - lore canon

Status: canon (2026-10-25, the consistency pass approved by the user). Title: **Emberwake: A Scarlet Lullaby** (user
2026-10-25; replaces the subtitle "The Low Sleep" of DEC-LORE-13; the article says the Red Tithing is one lullaby among
many, sung in blood each generation; the title need not appear in game text). The codename and paths stay `emberwake`.

This file is the single source the domain docs follow. Where a domain doc disagrees, this file wins; Acts 2-3 and
later systems are `proposal` (not locked), except the systems surveyed in `docs/specs/ACTS23.md`, locked by the user on 2026-11-10 (Q-A1 (b); the Act 2-3 mission lists stay proposal; Channel reworked as a fuel row, ACTS23 section 5.1). "(planner pick)" marks a choice the approved brief did not make (the list
to confirm is in [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) section 2); open questions (section 6) must not be filled with
invented design. Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]` (engine = technology any game needs;
kit = a shape any story in the genre could fill; game = named by the story, mostly data). "(exists)" marks a
genre rule that today's game crates already carry (`emberwake_rules`, `emberwake_battle`, `emberwake_world`): it is a
`[kit hook]`, not `[engine]`, because the engine crates (`aegis_*`) hold no game rules.

- Names, ages, backgrounds, first appearances and place names: [cast registry](background/cast_registry.md).
- What the Tithing really does (spoiler sheet; Option A is canon, Q-01): [central truth](background/central_truth.md).
  The Clerisy feeds the dreaming titan for its own eternal youth (the marrow) and blames the titan for the folk's woes.
- The story and cast in one place (who's who, story elements, reveal timeline): [story guide](STORY_GUIDE.md).

## 1. The cell in one line each

Founders (six in the full Act 1; the first playable slice has five, Elyse joins later): Kaelen (Vanguard), Vesper
(Gutter-Stalker), Maren (Heretic Acolyte), Hollis (Arbalestier), Dunstan "Slag-Jaw" (Slag-Brute), Elyse (Ichor
Alchemist). Hub cast: Sister Aline Vance (Slab), Bram "Iron-Lung" Karr (Hearth), Tess Morn (courier, mission board),
Pip (her brother). Founders' visible taints are backstory and Act 2 hooks only: Act 1 has no Strain, no Calcification
and no countdown on any founder (the hearth line "four months" becomes a non-mechanical hook).

## 2. Numbers sheet

### 2.1 Combat (ruleset option "timeline"; the Ashlands keeps "classic") `[kit hook]`

The ruleset is a world field (the Ashlands "classic", `sunk_ward` "timeline") with a Dojo toggle (user 2026-10-25,
Q-52). The engine is reused with the Ashlands as the baseline: a rule incompatible with the classic rules is built as
an option; a compatible one as an extension that can be disabled; bug fixes go to the core and may need an Ashlands
or kit revisit. Every engine and kit system the slice needs is built before it, so the slice itself is only narrative
choices and catalog data (Q-55, Q-56). Commands that work together chain: a follow-up skill can start in the blocks a
previous action leaves, otherwise the unit spends them in recovery; recovery makes heavy skills a commitment (Q-54).

| Rule | Canon value | Layer |
| --- | --- | --- |
| Round | 6 blocks, all sides resolved simultaneously; timeline strip at the TOP of the battle screen | [kit hook] (exists: `emberwake_battle::resolver`); only the strip widget is [engine] (`aegis_ui::timeline`) |
| AP | 10 max, +4 per round; an action pays AP when it starts and lands on its last block | [kit hook] (exists) |
| AP cost | Decoupled from speed tier: skill field `ap_cost`, default = skill level. Guideline: Cold Steel 2-4 AP, Inoculated 1-2 AP plus Strain | [kit hook] (new field), [data] values |
| Speed tiers | Relative windups: Snap 1 block, Standard 2, Heavy 3-4; the impact lands at the end of the action's own blocks; a maneuver may add recovery blocks. No fixed tick anchors | [data] through the skill field `blocks` (exists); recovery blocks are a new [kit hook] |
| Same-block impacts | The game's initiative order; no mutual trades (a unit stopped earlier in the block does not land) | [kit hook] (exists) |
| Poise | Max Poise = CONS x 2 + armour poise from gear (coefficients planner pick); refills each round; break at 0. CONS runs 6-18 in the stat budget, so CONS x 2 is 12-36 before armour; the lore's stagger values (Pommel Strike 25, Jaw-Breaker 30) were set for flat bars of 30-70 and need a rescale (Q-50) | [kit hook] (new) |
| Stagger intent | A break is a tactical choice to disrupt an enemy action (user 2026-10-25, Q-50 a): the lore's stagger values are rescaled to a fraction of a typical Max Poise and heavy enemy windups get more poise or an unbreakable flag; an optional seeded chance to break adds some randomness, with a guaranteed break at a cost (AP, a skill or Strain; the design lane specs it, then a sim) | [kit hook] |
| Stagger multipliers | x2 if struck in windup, x2 in Flesh-Tension, x0.5 under Active Guard; they multiply (planner pick) | [kit hook] |
| Repeated breaks | A unit's second break in the same round or in the next round costs double stagger | [kit hook] |
| Break effect | Cancels the action in windup and the unit's remaining blocks that round; nothing else (the evasion and +25% damage riders are dropped, planner pick) | [kit hook] |
| Active Guard | Base move: halves incoming stagger; 50% damage reduction through the guard term, capped by `GUARD_CAP` 0.9 (planner pick); vents 10 extra Strain at round end | [kit hook] |
| Armour | The hyperbolic curve, reduction = rating / (rating + 50). Design intent (user 2026-10-25, Q-49): from Act 2, fights are hard unless the cell uses armour pierce and the other Inoculation benefits; the scale is set after a balance review; sims and the RULESET1 test data use armoured units at ratings 25-50 (this replaces the x 2 rule below for armoured units until the review), so Inoculation gives a real advantage the player weighs against its risk. Until then lore armour points x 2 = engine rating (planner pick): Flesh-Tension +2 -> +4 rating, Chitin +1 -> +2, "shreds 4 armour" -> expose 8, boss armour 3-5 -> rating 6-10 | [kit hook] (exists: `emberwake_rules::damage`), [data] ratings |
| Pierce | Inoculated attacks ignore armour (binary, DEC-LORE-12 baseline). Today's damage type `pierce` is a type, not a bypass, so this needs an armour-bypass skill flag | [kit hook] (new flag), [data] (which skills set it) |
| Deck | 6 slots = 3 class actives + 2 passives + 1 keystone, bucketed by kind like today's skill slots (Q-48). The basics (Move, Active Guard, Strike, Rest and the other always-on moves) take no slot and are open to every unit, as in the Ashlands (user 2026-10-25: the two base-move slots are removed) | [kit hook] (new slot type) |
| Composure | A derived stat, not a seventh base stat (user 2026-10-25, Q-47 and Q-47a): the corruption resist, computed by its own formula from base stats plus gear, status and Calcification modifiers; shown on the unit panel; read by every corruption effect (Calcification I, the curfew chimes, the shockwave, nightmares, the Martyr mercy reward, recruit quirks). Formula and numbers: the derived-stat spec and the balance lane | [engine] formula evaluator, [kit hook] data-defined derived stats (new), [data] the Composure row |
| Pouch, gear | Pouch 3; six gear slots MAIN, OFF, HEAD, BODY, ACC1, ACC2. Graft slots 4 (Cranial, Thoracic, Forearms, Visceral) are an Act 2 proposal (2026-11-10: Cranial opens through the class ritual for class pieces; Forearms, Thoracic and Visceral take any monster organ; a graft teaches the monster's own skill; `docs/sessions/PK2_GRAFTS.md` 3) | [kit hook] (exists: `emberwake_rules::gear`); graft slots new |
| Reach | Melee reach 1 by default; a reach-2 rule for polearms, billhooks and scourges (user 2026-10-25, Q-70) | [kit hook] (new) |
| Arenas | A fixed arena size (12 x 9); larger fights are connected rooms: enemies move between rooms, an escape moves the party into another room instead of off the map, and the party may fight in several rooms at once (Q-60; spec ENCOUNTER2) | [kit hook] (new) |
| Stay-behind objectives | A hold-a-tile escape (one unit holds while the rest escape) and an impossible fight (the first unit knocked out makes a heroic stand that holds the boss while the others escape); the Crucible uses the hold-a-tile escape and the player picks who holds; the impossible fight belongs to a later mission: the first loss is a choice, later losses become fate (Q-75, Q-75a; spec ENCOUNTER2) | [kit hook] (escape and hold exist; new variants) |
| Enemy plans | Intent tells plus impact badges by default; the full plan (speed tier, impact block, movement) with Tess's reconnaissance unlock (Q-91) | [kit hook] (tells exist) |
| Round intent | Each member sets one intent for the round (Assault, Guard, Support, Maneuver; Channel from Act 2, reworked 2026-11-10: a 0 AP, 0 block Channel opener starts an all-marrow round in which the intent rows offer marrow versions only with Strain cost, armour pierce and a power bonus; only single-intent skills have a marrow version; the Channel kit is the graft skills, a universal marrow basic, the class basic's marrow version and the actives selected as marrow, `docs/specs/ACTS23.md` 5.1) and uses only skills of that intent; a skill has one intent even with side effects (Shield Bash is Assault); a plain move and Rest are neutral. The intent is what the existing intent tells and threat intel show, for enemies and the party alike. One target per round (self, ally, enemy or tile). Follow-ups: same-intent skills that fit wholly in the blocks left (mostly Snap), in a per-intent priority order set in the loadout; empty blocks are Rest; an auto planner fills the round, and a manual mode follows the same rules. Player units have no recovery blocks (AP limits heavy skills); enemies, with fixed move sets and no AP, keep recovery. An intent persists across rounds and may be changed at round start with no penalty; intent-bound buffs last only while that intent is active, other buffs keep their round durations across intents. Assault attacks the target and keeps it in engaged range while avoiding attacks where it can (moving in first; ranged units kite with an automatic step away, before the attack when the enemy is in melee range, after it otherwise). Guard may take cover (cover reduces damage) or enter stealth (a stealthed unit that moved is out of reach for chase and locked attacks; stealth breaks when the unit leaves Guard). Maneuver: flanking is attacking an enemy already engaged with another character, from outside its line of sight; a maneuver skill carries its own movement, so it costs no move when the target is within the skill's range; kinds Engage (get in position and attack, never face-on), Disengage (attack, then retreat to safety) and Utility (pouch items on the target). Support uses pouch items on self or an ally in range, never on enemies. The intent replaces the old stance damage multipliers and between-round switching under this ruleset (user 2026-10-25; spec RULESET1) | [kit hook] (timeline option) |
| Cut | Evasion and accuracy rolls, facing and stealth cones (facing returns as a derived battle facing for flanks and stealth as a Guard status, both without cones or rolls: row Round intent), the d100 crisis, mutual trades, the multi-track ghost scrubber (start with impact badges on the existing strip), stance damage multipliers (DEC-LORE-11; intents replace them as the round's skill limit, row Round intent) | - |

Board and art: STAGE1 option B, a tilted battle board of cells drawn 32x24 px (12 x 9 cells today; arena sizes:
Q-60) with zone backdrops, 40 px units and upright props sorted with the units; "32x48 sprite" means the figure
bounds inside the rig's 48x48 frames; the region map stays square top-down. Palette: the art bible's hex values, 16
master hues plus at most one dark and one light step each (computed, hand-picked where a step reads badly, Q-77), plus three skin ramps chosen per character in the unit palette files (Q-78); each world picks its units' palette files (Q-80). Large units (the Amalgam, the titan, the node 20 monstrosity) use 96 x 96 frames on 2 x 2 cells (Q-79). A camera shake with limits and a reduce-motion setting serves bell tolls, heavy impacts and tremors (Q-82). Slice placeholders may start now; final art comes in the M4 pass (Q-81).

### 2.2 Strain (Act 2 on; Act 1 has none) `[kit hook]` per-unit meter, `[data]` bracket names

| Strain | Bracket | Effect (engine scale) |
| --- | --- | --- |
| 0-24 | Dormant | none |
| 25-49 | Acute Resonance | +15% crit chance, +1 tile per move block, -25% healing received |
| 50-74 | Flesh-Tension | +25% damage, +4 armour rating, stagger taken x2 |
| 75-99 | Aberrant Edge | +50% damage, recoil 10% of damage dealt (planner pick: the bleed and stagger extras are dropped) |
| 100 | Rupture | crisis roll at round end |

- Gain: an Inoculated skill adds its `strain_cost` (guideline 15-35) when the action starts; hazards and eldritch attacks
  add more.
- Decay: at round end -10 if the unit used no Inoculated action that round, a further -10 if it held Active Guard; never
  below the dormant floor. After battle Strain resets to the floor. Grafts raise the floor (0-50) by how many a unit has (1: 5, 2: 15, 3: 30, 4: 50; an Inoculated keystone graft takes one of the 4 slots; user 2026-11-10, `docs/sessions/PK2_GRAFTS.md` 3).
- Caps (user 2026-10-25): Strain caps at 100 within a round and is set to 50 after a crisis (Q-62; changed from 75 by the user 2026-11-10, so crises do not chain: from 75 one more Channel round reached 100 again); the meter is a
  generic kit meter (Q-63); aberrant enemies carry it from Act 2, humans never; Act 1 has no Strain anywhere (Q-65).
  Installing an Inoculated keystone counts as Tier III surgery (+1 stage, Q-64).
- Crisis `[game code]` unless the kit meter's triggers can express it: 1d3, equal odds, seeded, rolled at round end:
  1 Unchecked Spasm (a full-power strike on the nearest unit, ally or foe, then stunned next round); 2 Biological
  Collapse (0 AP next round, Bleed 3 rounds, armour 0 for the rest of the battle); 3 Flesh Breach (+1 Calcification
  stage).

### 2.3 Calcification (out of combat, permanent, Act 2 on) `[kit hook]` per-unit stage track

Advances only through Flesh Breach and Tier III graft surgery (not by ending a battle in the Edge).

| Stage | Name | Effect |
| --- | --- | --- |
| I | Vein-Blackening | -5 Composure, +5% debuff resistance |
| II | Calloused Chitin | +2 armour rating, -1 tile per move block |
| III | Sensory Rot | immune to fear and morale debuffs; -15% ranged damage beyond 3 tiles (planner pick: accuracy rolls are cut) |
| IV | The Shakes | cannot equip two-handed weapons (bows and arbalests included); melee and bare-handed attacks pierce armour; sleeps in the Isolation Crypts |
| V | Terminal Drift | lost: walks into the catacombs or turns hostile |

The only undo: Aline's bone-scraping (unlocked by her quest), -1 stage and -10% max HP for good.

### 2.4 Roster and lifecycle `[kit hook]` downed / wounded / captured / dead / retired

| Rule | Canon value |
| --- | --- |
| Ages | Playable operatives and recruits 18-24; Unsealed = everyone under 25; children are stakes, never fighters; no combat graft surgery on anyone under 18. The Spent rarely pass 45 (Agnes, 44, is one of the oldest in the Ash-Crawl; Othemar at 64 is the exception, see the central truth) |
| Squad | Up to 4 deployed (`PARTY_MAX` 4, [kit hook] exists). RECRUIT1 refuses a recruit once the roster holds `party_max`, so the bunk count needs a separate roster cap ([kit hook] new) |
| Bunks | 6 at the start of Act 1 (4 filled after the prologue); Act 1 holds only the six founders (user 2026-10-25, Q-38); the bunk the Crucible empties takes the first procedural recruit at the start of Act 2; 8 through Bram's "Expanded Timber Frames" project from Act 2, chosen by the player; later hubs inherit the count. Guests take no bunk until sworn in |
| Hub | One hub; forward bases are story only (Q-23); facility levels: the bunk project in the slice, a level tree per facility from Act 2 (Q-42); facility screens open from their keeper NPC; the Doom Clock and Alert sit on the map HUD only (Q-46) |
| Founder at 0 HP | Downed. Act 1: after the battle a downed founder is wounded (no death, no capture). From Act 2: a downed unit bleeds out after a few rounds unless revived, and on a lost battle the downed are captured; the cell may also leave a unit behind so the rest escape (the left are captured; this is not the whole party fleeing) (user 2026-10-25, Q-32). Wounded: deploys at -20% max HP until a rest (user 2026-10-25, Q-31; a baseline the balance lane revisits);  |
| Scripted loss | The Crucible: one founder, chosen from the four deployed, holds the winch and is taken alive by the Inquisition |
| Corrupted-founder boss | Optional Act 3 node, only for a captured founder (the Crucible's, or one captured in Act 2) |
| Recruits | Act 1 rescues cannot join the roster; after the Crucible sacrifice the rescued become recruitable (the first recruit may be someone rescued at the Crucible itself) and roster management unlocks beyond party selection (user 2026-10-25, Q-38, Q-93) |
| Keepsakes | A captured or fallen unit, named or procedural, leaves a keepsake: an accessory passive carrying the character's name, far stronger than shop accessories, named ones stronger still (a roguelite thread: the perfect loadout is built over time; numbers by the balance lane) (Q-34) |
| Living Martyrs | A fixed Calcification counter that advances on every deployment, cannot be reduced or shared, and scales their bonus; at the threshold they retire or deploy at Stage V for a last fight (an exception to Stage V = lost); something stops hoarding them for bosses (Q-37, Q-37a). They take a crypt place and no bunk (Q-36) |
| Saved youths | A saved counter read by scenes and endings; under-18s go to Agnes's attics or Pip's runner network (Q-29) |

### 2.5 Resources and upkeep `[kit hook]` resource purse, `[data]` names

| Resource | Use |
| --- | --- |
| Scrap | The currency (replaces gold): trade, forge costs, bribes |
| Warded Tallow | Upkeep: 1 per day of travel or rest for the whole cell. At 0: -2 starting AP in every battle until restocked; nothing else (the road-fight penalty stays an Ashlands option, Q-18); Act 3 Tallow costs more Scrap, daily use stays 1 (Q-45) |
| Rock Salt | Stash material: Slab flushes, bribes; a Slab flush lowers the dormant Strain floor by 10 for the next battle; Quenched Cinder Salt is the rare one-charge item that prevents one Calcification advance (Q-43, Q-69; used in battle after a Flesh Breach, user 2026-11-10: the stage is not saved but its effects last the battle, and it calms a unit a Stage V breach turned hostile) |
| Aberrant Organs | Stash materials: graft surgery (Act 2 on) |

No other upkeep (per-operative, per-leg, Tallow-and-Salt and per-rest versions are dropped).

### 2.6 Campaign meters `[kit hook]` meters with thresholds and triggers

| Meter | Canon value |
| --- | --- |
| Inquisition Alert | 0-100%: 0-30 Guarded, 31-70 Suspicious, 71-99 Lockdown, 100 Breach. No siege in Act 1. Act 1 (user 2026-10-25, Q-13): Breach ends the run, but only a player who blatantly ignores the meter gets there (small Act 1 inputs, clear warnings at each threshold; the warning steps are a design-lane spec with Q-15). Acts 2-3: Alert is an active management task; Breach starts a siege of the hub; a lost siege costs facilities and forces a relocation, never a Game Over |
| Doom Clock | The Red Tithing calendar in days, visible from Act 1 day 1 at 42 days ("six weeks"). Story (user 2026-10-25, Q-04 and Q-16; section 3.6): it counts to this year's rite, which the Clerisy turns into a Great Tithing because the cell's raids hasten the titan's waking; 2.4 reveals that and relabels the clock "the Great Tithing"; at 0 in Acts 1-2 the Clerisy holds a lesser tithing and the clock restarts. Soft in Acts 1-2 (story pressure, no fail); hard in Act 3 (fail if it reaches 0 before node 21 falls) |
| Time costs | Route `time` 1 = 1 day on the travel clock ([kit hook] exists: `emberwake_world::travel`, `GameState.time`); a stealth leg takes 1 day more; a rest costs 1 day (planner pick; a rest advancing time is a [kit hook] extension); battles cost no days (planner pick) |
| Doctrine | Preservationist (Bram) to Flesh-Broker (Aline), Act 2 on; gates the endings. Moved by play (grafts and high-Strain battles toward Flesh-Broker, forge projects and salt flushes toward Preservationist) and by narrative choices; some narrative options open only at thresholds; each character's lean is shown beside the cell's doctrine and hub NPC favours read it; the endings also count the choices of deceased and retired members (user 2026-10-25, Q-19, Q-19a) |
| Checkpoints | Each act has checkpoint missions with day deadlines; a missed checkpoint gives the act a bad end because the cell could not prevent the events (Act 1: the run ends on a bad-end card, reload the act's start; from Act 2 the story goes on worse, with lasting costs; Q-20a); deadlines leave enough time to finish the act in the worst case. Every mission converges on the same climax, so optional missions and forks are free as long as checkpoints are met (user 2026-10-25, Q-20, Q-30) |
| Act lines | Alert carries over with a floor of 30 (Q-14); the Great Tithing date found at 2.4 corrects the clock once and the rest becomes the Act 3 budget; some Act 2 side quests lock at that reveal (Q-17) |
| Lost battles | Act 1: the cell retreats to the hub, all wounded, the mission stays, a day passes (Q-20). The final battle may be retried while any operative remains, inside the titan's dream: a beaten cell wakes at the Core's edge, its fallen return in the next attempt as dream-echoes fighting for the titan, and each attempt costs Act 3 days (Q-21, Q-21a). Sieges are fought in the hub as connected rooms (Q-22) |

Fail states: the **Extinguished** screen (a failure screen, not an ending) shows when no living operative remains in
the field or the bunks, when the Act 3 Doom Clock expires, or at an Act 1 Breach (Q-13). Not a fail: a single death, a lost siege, an Act 1-2
clock expiry.

## 3. Campaign spine

### 3.1 Prologue (canon)

Scripted maps outside the district graph; encounter ids `ENC-PRO-0A..0C` are new (planner pick).

| # | Mission | Place | Encounter | Who joins | Teaches |
| --- | --- | --- | --- | --- | --- |
| 0a | The Alley | Alleyway of Cinders, Seventh Ward | ENC-PRO-0A: 2 Castellan Footmen; Captain Vane gives "Order 44" and withdraws | Kaelen (solo) | windup, guard, Pommel Strike interrupt |
| 0b | The Rooftop | Meat Market rooftops | ENC-PRO-0B: an ambush squad | Vesper | smoke zone, root, disengage |
| 0c | The Bridge | Canal bridge, iron mortuary cart | ENC-PRO-0C: 3 Inquisitors + a crossbowman (bestiary unit names: Q-73) | Maren, Hollis (freed from the cart) | ranged pins, field suture |

Then the cell dives through the Drowned Runoff Grate (node 01) into the Sluice Undercroft: the hearth scene with Bram
and Aline, six bunks with four filled, the Doom Clock appears at 42 days, Act 1 title card.

### 3.2 Act 1 "The Young Vanguard": the Sunk Ward (canon)

Missions come from Tess's dead-drop board (a hub NPC offering quests) plus node travel; the board offers the next main
mission when the previous one is done (planner pick).

| # | Mission | Node | Encounter | Joins / loses | Key choice |
| --- | --- | --- | --- | --- | --- |
| M1 | The Sweep | 02 Alleyway Junction 7 | ENC-EVT-01, Captain Vane (boss; breaks the sweep sealing the grates) | - | - (planner pick: the old prologue slot) |
| M2 | The Crossroads Cages | 03 Ash-Crawl Crossroads | ENC-EVT-02, Proctor Quell: an open-air culling station, free the youths before the flagellants kill them; retorts for the Slab (Maren's hook) | - | - |
| M3 | The Bone-Mill Raid | 04 Fourth Kiln | ENC-EVT-03, Overseer Grendel | Dunstan (bunk 5) | - |
| M4 | The Tallow Vats | 05 Tallow Vats | ENC-EVT-04, Guild-Master Orlov (Vesper's vendetta) | Elyse (bunk 6) | - |
| Interlude | The Awakening Fissure | 00 Sluice Undercroft | ENC-EVT-05, the Calcified Amalgam; Aline's single marrow dose (first grafting teaser) | - | Which deployed founder takes the dose; that founder starts Act 2 at Calcification I (planner pick) |
| M5 | The Crucible | 06 Crucible Watchpost to 07 Slag Gate | ENC-EVT-06, Inquisitor-Prelate Nemeth; one founder holds the Slag Gate winch; the cell escapes and Nemeth survives into Act 2 (user 2026-10-25, Q-25); he returns in Acts 2-3 (3.6) | Loses the chosen founder (captured) | Deploy four (the Crucible requires four, planner pick); choose who stays |

Optional Act 1 board work: Kaelen's watchpost raid (forge bellows), Hollis's vantage points over the Crucible, Vesper's
drainage locks (route shortcuts), Tess's "Night of the Red Pen" (rescue Pip), Agnes's "Rafters of Tenement 12", Corvo's
gate, Brother Rook on street nodes. ENC-EVT-05 is not a siege in Act 1.

### 3.3 Act transitions

| Transition | Hub | Alert | Doom Clock | Switches on |
| --- | --- | --- | --- | --- |
| Prologue to Act 1 | The Sluice Undercroft opens (6 bunks) | starts at 0 (planner pick) | appears at 42 days, soft | board, travel, upkeep |
| Act 1 to Act 2 | Stays the Sluice; the Slag Gate breach opens the High Bastion graph (node 08 on); the Scribe's Crypt is a proposal forward base | open (Q-14) | stays soft | Inoculation, Strain, Calcification, grafts, Cold Steel weapon techniques (the grafts' mirror) and weapon durability (every Cold Steel attack skill wears the weapon, basics and support skills do not; marrow skills cost Strain instead; none in Act 1; `docs/specs/ACTS23.md` 5.2), recruits, doctrine, permadeath and capture, sieges, the 8-bunk project; the captured founder's keepsake (an ACC heirloom, planner pick) |
| Act 2 to Act 3 | Stays the Sluice; the Weeping Redoubt is a proposal forward base | carries on | turns hard | a continental wave of bell resonance (resonance itself starts in Act 2 in Upper Cinderreach and 2.5 silences it there, Q-09), Act 3 hazards |

### 3.4 Act 2 "The First Rupture": the High Bastion (proposal)

| # | Node | Encounter | Payoff |
| --- | --- | --- | --- |
| 2.1 | 09 Gilded Boulevard | ENC-ACT2-01, Inquisitor-General Ruthven's checkpoint | Into Upper Cinderreach |
| 2.2 | 12 Culling Stockades | ENC-ACT2-02, culling wagon (repeatable) | Recruits 18-24; Lucan's rescue (planner pick: one wagon) |
| 2.3 | 10 Tallow Reservoir | ENC-ACT2-03 | +20 Warded Tallow, -30 Alert |
| 2.4 | 11 Embalming Cloisters | ENC-ACT2-04, Chief Anatomist Lazar (Aline's quest) | Apex grafting; the Great Tithing's date and scale, and that the cell's raids caused it; the cell sees Nemeth deliver the captured founder to Lazar's anatomists and arrives too late (3.6) |
| 2.5 | 13 Grand Belltower | ENC-ACT2-05, Bell-Warden Colossus | Silences the bells' resonance in Upper Cinderreach (conflicts with 3.3: Q-09) |
| 2.6 | 14 Reliquary Crypts to 15 Grand Spire Gate | ENC-ACT2-06, Reliquary Guardian | Key of the Mantle Chasm; Act 3 |

Key choices: the doctrine split, Living Martyrs, the siege dilemma (defend the hub or finish the mission).

### 3.5 Act 3 "The Awakening Abyss": the Mantle Fissure (proposal)

| # | Node | Encounter | Payoff |
| --- | --- | --- | --- |
| 3.1 | 17 Weeping Basalt Chasm | ENC-ACT3-01, Chasm Gatekeepers | Foothold |
| 3.2 | 18 Marrow Conduits | ENC-ACT3-02, High Castellan Ulric | Conduits cut; Doom Clock +2 days |
| 3.3 | 19 Chorus Spire (optional) | ENC-ACT3-03, Cantor Ysolde | Flesh-Brokers may turn the Chorus against the altar; Malachi mediates (planner pick) |
| 3.4 | 20 Silent Ossuary (optional) | ENC-ACT3-04 in two phases (user 2026-10-25): Nemeth with the corrupted founder at his side, then the founder goes insane and kills everyone, Nemeth included (3.6) | Nemeth's Warded Cassock; mercy; the Transcendent Relic (upgrades the keepsake, planner pick) |
| 3.5 | 21 High Altar | ENC-ACT3-05, Arch-Prelate Othemar | Othemar always falls here (planner pick); the Doom Clock stops |
| 3.6 | 22 Dreaming Core | ENC-ACT3-06, the titan | Endings |

Main path 16-17-18-21-22; 19 and 20 are optional (planner pick).

### 3.6 The Great Tithing and Nemeth (user 2026-10-25; "(refinement)" marks the planner's detail on the user's gist)

The Great Tithing (Q-04, Q-16, Q-12; [tithing_options.md](background/tithing_options.md) option B with b2):

- Founding: the Sovereign Clerisy was founded two centuries ago, in the dark of the Extinguishing, as the order that
  holds the Tithing ritual to pacify the titan; that is its public charter. Its founding act was the first Great
  Tithing, held under cover of the Extinguishing, its dead recorded as the titan's wrath. The founders found that the
  dream's marrow kept them young, and the harvest began (the central truth, Option A). They are "the first
  Extinguishers", entombed in the Reliquary Crypts (2.6), where the record of that night lies (Q-12 option a).
- What the folk know: only the yearly Red Tithing (hundreds of Unsealed). The Great Tithing (thousands) is known to the
  Clerisy's inner circle alone. What caused the Extinguishing stays a mystery (b2).
- Why this year: the cell's own raids hasten the waking. Every tithe it frees and every station it breaks starves the
  dream, the titan stirs, and the Clerisy turns this year's rite into a Great Tithing to drug it back to sleep. The
  more the cell saves, the more the Clerisy needs. (refinement: the decision follows the Act 1 raids, M2 and the
  Crucible; the player learns it at 2.4 from Lazar's records.)
- Reveal path: Act 1, hints in Agnes's tales of the stars and Maren's forbidden texts (Q-07 option a: a chronicle of
  the first Great Tithing); 2.4, the date, the scale and the cause; 2.6, the founders' record in the Reliquary Crypts;
  Act 3, Malachi confirms what the dream is fed for.
- What the Tithing is (user 2026-10-25): not only a killing. The Clerisy raises children as sacrifices for the titan,
  and the marrow it drinks comes from the remains of the sacrificed; the younger the child, the more potent the
  marrow. (refinement:) the tithe-children are raised from infancy in the Clerisy's foundling houses, the Lullaby
  Houses, where they are sung the lullaby each night and the folk see charity; Unsealed youths swept from the wards
  fill out the yearly count. After the rite the tithed are laid in the crypts over the titan's veins, where the dream
  steeps them; Lazar's anatomists render the remains into marrow in the Embalming Cloisters, and the conduits carry it
  up to the cathedral. Without the dream the remains are only flesh, which is why the Clerisy needs the titan asleep
  and dreaming, and why its waking ends the harvest. The old serve too: the Spent are drugged docile and, dead, are
  rendered into tallow by the Guild; the young become marrow, the old become fuel. The Inquisition's infusions
  (Quell's valve, Nemeth's) are aberrant marrow from older, weaker stock, which wears the body down. A Great Tithing
  yields marrow for a generation.
- Tone rule (refinement): children are stakes, never fighters or on-screen victims; the game shows the Lullaby Houses,
  the rite, the pens and the marrow, never violence to a child.
- Endings: A Preservation ends the cycle (no dream to feed); B Ascendancy hands the cell the chronicle and the source,
  so the next Great Tithing would be theirs to call; C Red Martyrdom sates the dream so the cycle never turns again.
  Losing (the Extinguished screen) is a second Extinguishing for the Unsealed.

Core story (user 2026-10-25): only the Clerisy drinks marrow, and the draughts suppress dissent, not competence (Q-03,
Q-06). Two substances: the titan's raw black marrow mutates whoever touches it (veins, Aline's dose, organs), the
rendered marrow is the Clerisy's elixir of youth (G-33); the tithed are steeped in the deep crypts, rendered in Lazar's
Cloisters and piped up to the cathedral, the Marrow Conduits (node 18) being the deep end of that line (G-32). The
player gets a hint and the rendering at 2.4, where Aline recognises her old order's work, and the full truth from
Malachi before node 21, a fixed beat whatever the doctrine (Q-02, G-08, G-31). Bell resonance starts in Act 2 as the
raids hasten the waking (Q-09). If node 20 is skipped, the captured founder stays the saint-aberration, named in the
ending text, and Nemeth survives (G-15); only the Crucible's founder fights at Nemeth's side (Q-94). Each surviving
founder gets an Act 2 loyalty mission (G-16); every ending states Bram's and Aline's fates (G-17). The slice ends on a
card after the Crucible escape (Q-87); Act 1 founder hooks pay off in play (Q-28).

Nemeth (Q-25, G-01, G-02):

- Act 1: escapes the Crucible with the captured founder (3.2).
- Want (refinement): his infusion is the Inquisition's crude aberrant marrow, which wears the body down; he wants the
  Clerisy's pure marrow, rendered from the youngest tithe-children, and a place among the ageless. The captured
  founder is his offering: he gives them to Lazar's anatomists to be made into a saint-weapon against the cell.
- Act 2 sighting (refinement): at 2.4 the cell sees Nemeth deliver the founder to Lazar and arrives too late; this
  also closes a rescue (Q-33 option a).
- Act 3, node 20 the Silent Ossuary, one battle in two phases (user 2026-10-25): (1) the founder, already corrupted
  by Lazar's anatomists, fights at Nemeth's side against the cell; (2) fighting their old teammates breaks the
  founder's mind: they go insane and kill everyone in reach, Nemeth and his guard included, and the cell fights the
  monstrosity alone. The win offers mercy and the Transcendent Relic. Nemeth's Warded Cassock is the fight's reward, paid when the cell wins (loot is settled on a win).
  (refinement:) phase 1 lets the founder use their old signature moves, twisted, and the founder's lines answer the
  cell by name; the break comes on an `hp_below` trigger on the founder with a round fallback, or earlier if Nemeth
  falls low. Build: one encounter with phases ([kit hook] exists: `emberwake_battle::encounter`, `hp_below` with
  `or_round`, line, status, ai and enrage effects); new and small: a phase effect that kills named units, and a
  "hostile to all sides" AI or faction switch for the founder ([kit hook] new); the monstrosity's larger body is a
  unit swap (Q-79 for 96 x 96 art).

## 4. Endings (proposal; meaning set by the central truth)

| Ending | What happens | Voice | Doctrine gate |
| --- | --- | --- | --- |
| A Preservation | Cold Steel: black-iron spikes sever the synapse; the titan sinks into a deep coma; humanity stays mortal and untainted | Bram | Open unless Flesh-Broker (planner pick) |
| B Ascendancy | Transhuman: the cell merges with the titan's marrow and rules as something new | Aline | Flesh-Broker only |
| C Red Martyrdom | The cell gives itself to break the Tithing for good | the cell | Always open (planner pick) |
| Extinguished | Failure screen (annihilation or the Act 3 clock), not an ending | - | - |

## 5. Glossary (one name per thing)

| Canonical term | Meaning | Retired variants |
| --- | --- | --- |
| Act 1 / 2 / 3 | Campaign units; titles The Young Vanguard, The First Rupture, The Awakening Abyss | Phase 1-3, The Iron Vanguard, The Transhuman Rupture, The Blight of the Low Sleep, The Closed Four |
| The Low Sleep | The titan's slumber and, by extension, the titan | (title "Emberwake: The Low Sleep") |
| The Red Tithing | The recurring sacrifice of Unsealed youths of any age, children raised for it in the Lullaby Houses first; their remains become the Clerisy's marrow; the yearly rite is the one the folk know | - |
| The Lullaby Houses | The Clerisy's foundling houses that raise tithe-children (planner refinement, 3.6) | - |
| The Great Tithing | A Tithing of thousands, held once before at the Extinguishing (the Clerisy's founding) and again this year; known to the Clerisy's inner circle until 2.4 | - |
| Lesser tithing | An out-of-season Tithing the Clerisy holds when the soft Doom Clock reaches 0 in Acts 1-2 | - |
| Composure | The derived corruption resist (section 2.1) | - |
| The Unsealed / the Spent | Under 25 / 25 and over (compliant through decades of draughts and tallow fumes) | The Caged Young, The Spent Generation (fine in prose), Docile Masses |
| Sovereign Clerisy, Iron Inquisition, Castellan Watch, Tallow Guild, Fourth Kiln Syndicate, Severed Root, Awakening Chorus | Factions; the Watch is the Inquisition's city arm, the Syndicate the Guild's foundry arm | Hegemony, High Hegemony, Sub-Basalt Cults, Seventh Ward Watch, Castellan Guard |
| Preservationist / Flesh-Broker | The two doctrines | The Purists, The Inoculated (as faction names) |
| Cold Steel / Inoculated | The two skill branches | Martial path, Resonant path |
| Strain brackets | Dormant, Acute Resonance, Flesh-Tension, Aberrant Edge, Rupture | Clear, Quiescence, Twitch, Tremor, Resonance, Rupture Verge, Rupt, CRISIS |
| Calcification stages | Vein-Blackening, Calloused Chitin, Sensory Rot, The Shakes, Terminal Drift | Veins, Brittle, Gargoyle, Martyr, Death/Aberr, calcification "points" |
| Scrap | Currency | Gold, Scrap Iron, black iron slag, Salvaged Slag, salvaged timber and charcoal |
| Warded Tallow | Upkeep fuel | vials, candles, ledgers, High-Grade Warded Tallow |
| Rock Salt | Stash material | Salt Rations, Rock-Salt Rations, Sanctified (Rock) Salt (the pouch item Sanctified Salt Sachet stays an item) |
| Aberrant Organs | Harvested graft materials; Apex organs are Tier III | Harvested Aberrant Organs, Aberrant Marrow Organs |
| The Sluice Undercroft | The hub | - |
| Apothecary's Slab, Tinker's Hearth, Dry Hollows, Water-Lock Basin, Isolation Crypts | Hub facilities (the Basin is the central pool, dead-drop, launch point and guest quarters; the Crypts sit in the Dry Hollows) | Bram's Hearth/Forge, surgical rig, Water-Lock (& Siphon / Gate), Dead-Drop Basin, Central Basin, Quarantine Kiln Cell, Isolation Vaults, Quarantine Crypts |
| Mission board | Tess's dead-drop board | - |
| Doom Clock, Inquisition Alert | Campaign meters | Great Extinguishing (now the Extinguished failure screen) |
| The Extinguishing / Extinguished | The event two centuries ago that put out the sun, moon and stars / the failure screen (an intended echo, kept) | the Great Extinguishing (as a name for the loss) |
| Castellan Footman, Watch Crossbowman | The Castellan Watch's line soldier and crossbow unit | Castellan Warden (prologue), Watch Arbalestier (collided with the Arbalestier class) |

## 6. Open questions (do not fill with invented design)

Every open question, with options, a recommendation, where it lives and who decides, is in
[OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) (stable ids Q-01..Q-91; the questions to decide first in its section 1, the
planner picks to confirm in its section 2). The ones this file names: the bell-resonance contradiction
between 3.3 and 3.4 (Q-09, no pick); Alert across act lines (Q-14); soft-clock expiry in Acts 1-2 and the Act 3 day
budget (Q-16, Q-17); doctrine inputs and thresholds (Q-19); a fight for Inquisitor-General Ruthven (Q-26); the act of
Bram's quest (Q-27); the Act 2 downed rule (Q-32); rescuing a captured founder in Act 2 (Q-33); keepsake numbers
(Q-34); Living Martyrs' "+1 stage per battle" against the Calcification rule of 2.3 (Q-37); costs of the 8-bunk
project (Q-38); the Rock Salt flush effect (Q-43); the armour scale after the balance review (Q-49); the stagger rescale (Q-50); post-crisis Strain (Q-62).

Decided on 2026-10-25 (user; rows in `games/emberwake/docs/decisions/lore.md`): the central truth is Option A (Q-01);
an Act 1 Breach ends the run only for a player who ignores the meter (Q-13); Nemeth escapes the Crucible (Q-25);
wounded = -20% max HP until a rest (Q-31); Composure is not a base stat (Q-47); the deck is bucketed slot kinds (Q-48); the
slice and its dilemma follow the roadmap (Q-83, Q-84, Q-85). Also 2026-10-25: the Great Tithing's story, the
Clerisy's founding and Nemeth's arc (section 3.6; Q-04, Q-12, Q-16, Q-25 follow-up); Composure returns as a derived
stat, the corruption resist (Q-47a).
