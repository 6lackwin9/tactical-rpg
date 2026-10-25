# Roster: lifecycle, expansion and bunks

Status: canon for Act 1 (founders, bunks, squad size, downed and wounded, the Crucible capture; [CANON](../CANON.md)
1, 2.4); proposal for Act 2 on (recruit paths, permadeath, retirement, quarantine). Names and ages:
[cast registry](../background/cast_registry.md). Related: [safehouse economy](safehouse_economy_and_triage.md),
[magic and corruption](magic_and_corruption.md).

The cell lives in a cramped storm drain, the Sluice Undercroft. Roster play is liberation, transformation, triage and
loss, not hiring: no menu sells recruits.

## 1. The founders

Six founders in the full Act 1; the first playable slice has five (Elyse joins later). Classes map onto the engine's
six classes `[data]`.

| Founder | Class (engine) | Joins | Bunk |
| --- | --- | --- | --- |
| Kaelen | Vanguard (`warrior`) | prologue 0a | 1-4 |
| Vesper | Gutter-Stalker (`rogue`) | prologue 0b | 1-4 |
| Maren | Heretic Acolyte (`priest`) | prologue 0c | 1-4 |
| Hollis | Arbalestier (`ranger`) | prologue 0c | 1-4 |
| Dunstan "Slag-Jaw" | Slag-Brute (`brawler`) | Act 1 M3, the Bone-Mill Raid | 5 |
| Elyse | Ichor Alchemist (`mage`) | Act 1 M4, the Tallow Vats (not in the first slice) | 6 |

Their visible taints (Kaelen's spurs, Hollis's eye, Maren's throat veins, Vesper's smoke fingertips) are backstory and
Act 2 hooks: no Strain, no Calcification and no countdown on any founder in Act 1.

## 2. Ages

Playable operatives and recruits are 18-24. The Unsealed are everyone under 25; the Spent (25 and over) are compliant
after decades of communion draughts and tallow fumes. Children (Pip, Lucan) are stakes and non-combat NPCs only. No
combat graft surgery on anyone under 18. The few adults who kept their minds (Bram, Sister Aline, Bailiff Corvo) are
non-combat hub or quest NPCs. The old "15-24" cohort is retired.

## 3. Squad and bunks

| Rule | Value | Layer |
| --- | --- | --- |
| Squad | up to 4 deployed (`PARTY_MAX` 4; world `party_max` 1..4); the Crucible requires four | `[kit hook]` (exists) |
| Bunks | 6 at the start of Act 1, 4 filled after the prologue; Act 1 holds only the six founders, and the bunk the Crucible empties takes the first procedural recruit at the start of Act 2 (user 2026-10-25, Q-38); 8 through Bram's "Expanded Timber Frames" project, chosen by the player (Act 2 on, a Scrap cost from the balance lane); later hubs inherit the count | `[kit hook]` |
| Full bunks | a new recruit needs a free bunk: dismiss or retire someone first (Act 2 on) | `[kit hook]` |
| Guests | take no bunk until sworn in; they lodge at the Water-Lock Basin | `[kit hook]` |

Engine today: the roster cap is the active cap. RECRUIT1 refuses a recruit when the roster already holds `party_max`
members (`emberwake_world::dialogue`, "The party is full"), so a 6-bunk roster with a 4-hero squad needs a separate
roster cap (the bunk count) `[kit hook]` new. Picking the four is the existing active toggle on the Character screen.

## 4. Lifecycle states `[kit hook]`

The kit shape is downed / wounded / captured / dead / retired; the story decides which are on in each act.

| State | Act 1 | Act 2 on (proposal) | Engine today |
| --- | --- | --- | --- |
| Downed | a founder at 0 HP is downed for the rest of the battle | same | exists: downed heroes get up at the win rally (+25% max HP, HEAL1) |
| Wounded | after the battle a downed founder is wounded: deploys at -20% max HP until a rest (user 2026-10-25, Q-31; a baseline the balance lane revisits) | same | new |
| Captured | only the Crucible's winch founder, taken alive by the Inquisition | capture in battle switches on | new |
| Dead | none | permadeath switches on; a crisis Biological Collapse (bleed 3 rounds, armour 0) can kill a low-HP operative | new |
| Retired | none | a Stage IV operative can be retired to the Undercroft staff, freeing a bunk (any bonus: open) | new |
| Terminal Drift | none | Calcification Stage V: lost (walks into the catacombs or turns hostile) | new (Calcification `[kit hook]`) |

Two story rules sit on top `[data]`:

- The Crucible sacrifice (CANON 2.4): the one scripted Act 1 loss, chosen from the four deployed; deployment is the
  decision. The founder is captured, not killed; the keepsake (an ACC heirloom, planner pick) stays with the cell.
- The corrupted-founder boss is an optional Act 3 node (ENC-ACT3-04), only for a captured founder (the Crucible's, or one
  captured in Act 2). The Crucible's founder is carried off by Nemeth, delivered to Lazar at 2.4 (a rescue is closed in
  practice) and at node 20 fights at Nemeth's side, then breaks and kills him ([CANON](../CANON.md) 3.6).

## 5. How operatives join

Act 1: only the scripted founder joins above (dialogue effect `recruit`, RECRUIT1, joins at the highest active XP)
`[data]`; Act 1 holds only the six founders (Q-38). Recruits switch on in Act 2 (CANON 3.3): the first procedural
recruit takes the bunk the Crucible emptied, at the start of Act 2 (how they arrive, the first wagon or a join at the
act transition, is not written). Act 2 paths (proposal):

| Path | How | Layer |
| --- | --- | --- |
| Culling-wagon breaks | ENC-ACT2-02 at the Culling Stockades (repeatable): free 1-2 recruits, 18-24, of a class the encounter sets | `[data]` encounter + recruit; random class is `[kit hook]` new |
| Defectors and runaways | deserting Watch guards (Vanguards), runaway sweeps (Gutter-Stalkers), escaped bone-mill hands (Slag-Brutes) answer the cell's ciphers at the Water-Lock Basin | `[data]` dialogue and quests |
| Black-market extraction | after Tess's quest frees Pip, Pip's runner network smuggles marked youths out of the workhouses for Scrap or Rock Salt (amounts open) | `[data]` dialogue with `take` |
| Guests sworn in | a guest aged 18-24 can be sworn in and then takes a bunk | `[kit hook]` |

## 6. How operatives leave

| Path | Act | Notes |
| --- | --- | --- |
| The Crucible capture | 1 | scripted, see 4 |
| Death in battle | 2 on | permadeath |
| Capture in battle | 2 on | the rescue of a founder captured in Act 2: CANON open question (Q-33; the Crucible's captive is closed in practice by the 2.4 sighting) |
| Terminal Drift | 2 on | Stage V |
| Retirement | 2 on | frees a bunk |
| Dismissal | 2 on | frees a bunk; nothing else defined |

Calcification undo (CANON 2.3): Aline's bone-scraping, unlocked by her quest: -1 stage and -10% max HP for good.
Repeated scraping can leave an operative too frail for the front line.

## 7. Quarantine

A Stage IV operative (The Shakes) sleeps in the Isolation Crypts, which sit in the Dry Hollows (CANON 2.3, glossary).
The old Composure penalty on bunkmates was dropped while Composure was undefined; Composure is now a derived stat, the
corruption resist ([CANON](../CANON.md) 2.1), so the penalty could return as one of its readers, but no doc restores
it (open, with the Crypts move in [six founders](../narrative/six_founders_and_phase_progression.md) 5.3). Whether a
crypt counts against the bunk total: Open, see gaps.

## 8. Upkeep

The cell pays 1 Warded Tallow per day of travel or rest, for the whole cell; at 0 Tallow every battle starts at -2 AP
until restocked (CANON 2.5). The old per-operative, per-leg, Tallow-and-Salt and "+10% Strain for idle reserves"
versions are dropped.

## 9. Gaps

1. Wounded: what it does and how it heals. Decided (2026-10-25, user, Q-31): -20% max HP until a rest, a baseline the
   balance lane revisits.
2. Isolation Crypts: does a crypt use a bunk, and how many crypts are there.
3. Retirement bonus: the old "+5% forge speed" is the only example; whether retirement grants anything.
4. Recruit class and level: random or set by the encounter; RECRUIT1 joins at the highest active XP today.
5. Dismissal: does a dismissed operative leave for good, and with their gear.
6. Founder deaths from Act 2: which later beats (personal quests, ending voices, the corrupted-founder node) must
   cope with a dead founder.
