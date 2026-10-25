# Enemy bestiary and AI (index)

Status: canon for the names, factions and the Act 1 boss line-up ([CANON](../CANON.md) 3.1-3.2); proposal for every
maneuver and number (tuning targets for the balance lane, not stat blocks). Split by faction on 2026-10-25; the old
single file is in git history.

| File | Holds |
| --- | --- |
| [Castellan Watch](bestiary_castellan_watch.md) | Castellan Footman, Watch Crossbowman, Bailiff Enforcer; boss Captain Vane |
| [Iron Inquisition](bestiary_iron_inquisition.md) | Mortuary Flagellant, Inquisitorial Purifier, Inquisitor Proctor; bosses Proctor Quell, Inquisitor-Prelate Nemeth |
| [Tallow Guild and Fourth Kiln Syndicate](bestiary_tallow_guild.md) | Tallow Dredger, Render-Beast, Guild Carver; bosses Guild-Master Orlov, Overseer Grendel |
| [Low Sleep aberrations](bestiary_aberrations.md) | Ash-Crawler, Calcified Husk, Marrow Horror; boss the Calcified Amalgam |
| [Act 1 encounters](bestiary_encounters_act1.md) | ENC-PRO-0A..0C and ENC-EVT-01..06 as the Act 1 missions use them |

Factions follow the glossary: the Castellan Watch is the Inquisition's city arm, the Fourth Kiln Syndicate the Tallow
Guild's foundry arm. The Sovereign Clerisy fields no troops of its own in Act 1; the Awakening Chorus (Act 3) has no
units yet (gap 6).

## 1. Design rules

1. Three tiers per faction: base, elite, named boss. A boss is an authored variant of a base or elite template with one
   innate trait, one altered maneuver and one capstone that a boss phase unlocks.
2. Enemies obey the player's rules (ruleset option "timeline", CANON 2.1): 6 blocks per round resolved simultaneously,
   AP 10 max +4 per round, an action pays AP when it starts and lands on its last block, same-block impacts in the
   engine's initiative order, poise breaks cancel the action in windup and the rest of the unit's round.
3. Speed tiers are relative windups: Snap 1 block, Standard 2, Heavy 3-4, plus recovery blocks where stated. The old
   tick anchors ("Impact T4") are gone; an old "impact T4" heavy became Heavy 3, an "impact T5" heavy Heavy 4, so the
   order between maneuvers is kept.
4. Cut everywhere (CANON 2.1): evasion and accuracy rolls, facing and rear arcs, stealth and sight cones, the d100
   crisis, mutual trades, stances. Old text that leaned on them is restated or listed as a gap.
5. "Turn" means round.

## 2. Numbers as tuning targets `[data]`

An enemy is a unit row: class + level + side + tier + tuning deltas (`hp_pct`, `power`, `armor`, `speed`, `ap`) in
`games/emberwake/data/catalog/units.json` (GAME_BRIEF "Core rules"). A named boss is a world character on such a unit
(world layer, `WorldFactory` character units). The faction files therefore give targets, and the balance lane turns
them into deltas:

| Target | Meaning | Conversion from the old lore numbers |
| --- | --- | --- |
| Class, tier | One of the six engine classes (warrior, ranger, brawler, priest, rogue, mage) and an engine tier (minion, standard, elite, boss; elite 1.25 HP, boss 1.5 HP / 1.1 power) | Base = standard (minion for swarm units), elite = elite, boss = boss (proposal) |
| Bulk | Total HP after tier, as a multiple of a Castellan Footman | Old HP / 65 |
| Armour rating | Engine rating on the curve rating / (rating + 50) | Old armour points x 2 (CANON 2.1) |
| Poise | Target for the poise formula (CONS x 2 + armour poise; coefficients open) | Old poise value kept as the target |
| Mobility | low / normal / quick / fast, set through the `speed` delta | Old tiles per round 2 / 3 / 4 / 5-6 |
| Hit | light / medium / heavy, set through `power` | Old damage 10-16 / 18-28 / 30-50 |
| Poise damage | Stagger a hit deals to poise | Old "+N Stagger" kept as the target |

## 3. What the engine already has `[kit hook]`

| Need | Exists today | Notes |
| --- | --- | --- |
| Units, tiers, tuning deltas | `units.json`, `StatRules.TIERS` | `[data]` fills them |
| Named bosses | world characters, named items with `grants` | `[data]` |
| Encounters | catalog `encounters.json` and world encounters | `[data]` |
| Objectives | rout, hold, escape, slay, protect, guests (OBJ1) | the Crucible is an escape |
| Boss capstones | boss phases (BOSS1): HP and round triggers (`or_round`, PACE1), at most 4 per fight | a capstone "at 50% HP or round 3" is one phase |
| Reinforcements | the arrival phase (PHASEFOLD: one wave, round and block) | |
| Morale | MORALE1 (break, surrender, leader) | |
| Rewards | REWARD1 loot table per base unit, `key_loot`, `objective_reward`, purse | organs and relics drop here |
| AI | `emberwake_battle::ai` profiles and intents (INTENT1 tells, chips and arrows) | heuristics beyond today's profiles are new `[kit hook]` |
| Statuses | 15 in the status registry (stagger, stun, root, bleed, weaken, slow, expose, poison, burn, haste, taunt, regen, fortify, empower, riposte) | table below |

Lore status to engine status:

| Lore word | Engine | Layer |
| --- | --- | --- |
| Stunned, Rooted, Bleed, Burn | stun, root, bleed, burn | exists |
| Exposed (-N armour) | expose (lore points x 2) | exists |
| Pinned (both units held) | root on both (proposal) | exists |
| Parry and riposte | riposte (the defensive "stance" wording is dropped) | exists |
| Arterial Rupture (bleed on moving) | new | `[kit hook]` |
| Blind | dropped as written (no accuracy rolls); gap 3 | - |
| Flammable (+50% fire taken) | new | `[kit hook]` |
| Nightmare Hallucination (may strike an ally) | new; its 50% roll is gap 3 | `[kit hook]` |
| Calcified legs (held in stone 1 round) | root (proposal) | exists |

## 4. Act 2-3 adversaries (proposal, no blocks yet)

From [CANON](../CANON.md) 3.4-3.5; none has a maneuver set yet (gap 6).

| Encounter | Adversary | Faction |
| --- | --- | --- |
| ENC-ACT2-01 | Inquisitor-General Ruthven's checkpoint (whether he fights: CANON open question) | Iron Inquisition |
| ENC-ACT2-02 | culling wagon escort (repeatable) | Iron Inquisition |
| ENC-ACT2-04 | Chief Anatomist Lazar | the regime (cast registry; which arm is open) |
| ENC-ACT2-05 | Bell-Warden Colossus | open |
| ENC-ACT2-06 | Reliquary Guardian | open |
| ENC-ACT3-01 | Chasm Gatekeepers | open |
| ENC-ACT3-02 | High Castellan Ulric | Iron Inquisition (a reconstructed inquisitor, cast registry) |
| ENC-ACT3-03 | Cantor Ysolde | Awakening Chorus |
| ENC-ACT3-04 | the corrupted founder (optional), in two phases (CANON 3.6): phase 1 at Inquisitor-Prelate Nemeth's side with his guard, using the founder's old signature moves, twisted; then the founder goes insane, kills Nemeth and his guard and fights the cell alone as a monstrosity (a unit swap for the larger body, art Q-79); needs a phase effect that kills named units and a "hostile to all sides" AI or faction switch `[kit hook]` new | phase 1: Iron Inquisition (Nemeth's side); phase 2: hostile to all |
| ENC-ACT3-05 | Arch-Prelate Othemar | Sovereign Clerisy |
| ENC-ACT3-06 | the titan | the Low Sleep |

Render-Beasts also hunt for Inquisitor-General Ruthven at Lockdown ([cast registry](../background/cast_registry.md));
they are listed with the Tallow Guild, which breeds them in this doc (whose beasts they are: gap 7).

## 5. Gaps (bestiary-wide)

Each gap is in [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md) with options and an id (Q-60, Q-55, Q-56, Q-65, Q-73, Q-76,
Q-11); the faction files point here.

1. Board size: the engine board is 12 x 9 cells (`grid.rs` `GRID_W`/`GRID_H`, STAGE1 geometry); the old encounter
   layouts ask for 12 x 10 up to 18 x 12.
2. Combat hooks the maneuvers use that the engine lacks: forced movement (pull, push, knockback, drag), ground hazards a
   skill leaves behind, auras (effects in a radius), triggers on a poise break, terrain a skill destroys, scheduled
   arena hazards (vents on fixed blocks). Decided (2026-10-25, user, Q-55): every hook the slice uses is built as a
   reusable kit or engine system before the slice, starting with forced movement and ground hazards; the rest when the
   data that uses them is planned.
3. Rolls and checks the canon cut: the Flagellant crit rider stays (crits exist), but "Poise check or slip", "DEX check
   or slip", Blind and the 50% hallucination roll need a non-roll restatement. Decided (2026-10-25, user, Q-56): each is
   rebuilt on existing levers (a range cap, poise damage, AP loss; numbers by the balance lane); anything existing
   systems do not cover becomes a reusable engine or kit system first.
4. Strain riders in Act 1: Act 1 has no Strain (CANON 2.2), yet Quell, the Marrow Horror and the Amalgam push or read
   Strain in Act 1 fights.
5. Unit names in the prologue: 0a uses Castellan Footmen (fixed 2026-10-25); 0b's ambush squad and 0c's "3
   Inquisitors + a crossbowman" still name no bestiary unit (Q-73).
6. Act 2-3 adversaries and Awakening Chorus troops have no maneuvers.
7. Render-Beast ownership (Guild or Inquisition).
