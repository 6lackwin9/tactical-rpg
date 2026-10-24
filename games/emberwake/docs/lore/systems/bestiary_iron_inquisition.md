# Bestiary: the Iron Inquisition

Status: proposal (maneuvers and tuning targets); names and roles canon. Index, conversion rules and gaps:
[enemy bestiary](enemy_bestiary_and_ai.md). Every unit row is `[data]`; AI lines are `[kit hook]` (AI profile).

The church's enforcement wing: fanatical speed, self-mutilation, phosphorus fire and stolen marrow organs.

| Unit | Tier | Class (proposal) | Bulk | Armour rating | Poise | Mobility | Role |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Mortuary Flagellant | base (minion) | rogue | 0.7 | 0 | 20 | fast | swarmer, Snap bleed flail |
| Inquisitorial Purifier | elite | mage | 1.2 | 4 | 40 | normal | area denial, phosphorus |
| Inquisitor Proctor | elite | warrior (two-handed sword) | 1.9 | 8 | 60 | quick | duelist and officer |
| Proctor Quell | boss | warrior | 2.5 | 6 | 65 | quick | "the Scalpel of St. Lyra's" |
| Inquisitor-Prelate Nemeth | boss | warrior | open | open | open | open | the Crucible's commander; node 20 phase 1 |

## Mortuary Flagellant (base)

- Look: emaciated penitent, bare-chested under blood-stained mortuary vestments, barbed wire round the forearms,
  three-tailed scourge.
- Innate trait, Fanatical Agony: below 50% HP, +1 tile per move block and +20% crit chance (crits exist; the HP
  threshold trigger is `[kit hook]` new).
- Scourge Lash: Snap (1 block), reach 2, light hit, poise damage 10, bleed 2 rounds.
- Martyr's Rush: Snap (1 block), costs the Flagellant 10 HP, a 3-tile leap then a medium hit (dash exists:
  `dash_tiles`; the HP cost is new).
- AI `[kit hook]`: ignores its own safety, rushes the low-armour back line (Arbalestier, Alchemist, Acolyte) past the
  tanks (today's `reckless` profile never flees; target choice is new).

## Inquisitorial Purifier (elite)

- Look: warded iron apron, bronze bird-beak gas mask, pressurised sulfur canister on the back, pole-mounted phosphorus
  nozzle.
- Innate trait, Heat-Tempered Carapace: resists burn and poison (`resist` exists).
- Pyre-Nozzle Sweep: Standard (2 blocks) + 2 recovery, range 3, a 3 x 3 area painted on the ground during the windup;
  it leaves Phosphorus Fire on those tiles for 2 rounds (medium hit per round and burn; a ground hazard a skill leaves:
  index gap 2).
- Brand of the Heretic: Snap (1 block), reach 1, light hit, expose 4, ends the target's Active Guard.
- AI `[kit hook]`: coats clusters and covered tiles in fire to flush operatives into the open.

## Inquisitor Proctor (elite)

- Look: mirror-polished plate over a crimson cassock, a long executioner's broadsword in both hands, a tallow vial
  set in the throat collar.
- Innate trait, Cold Inquest: an operative that starts a round within 2 tiles loses 1 AP (aura: index gap 2).
- Sanctified Decapitation: Heavy (3 blocks) + 2 recovery, reach 1, heavy hit, poise damage 30, ignores Active Guard.
- Inquisitorial Parry: Snap (1 block), self: riposte for the round (the engine `riposte` status: counters adjacent
  melee hits it survives at 50% power; the lore's full parry and its "before T4" window are dropped).
- AI `[kit hook]`: duels the strongest melee operative (Kaelen or Dunstan); alternates the parry and the telegraphed
  heavy so the player has to read the strip.

## Proctor Quell (boss, ENC-EVT-02)

The Inquisitor vivisector who denounced Maren and oversaw her mentor's autopsy; an illegal marrow valve in his neck
([cast registry](../background/cast_registry.md)). White linen cassock stiff with black marrow, twin bone-saws. Boss of
Act 1 M2 "The Crossroads Cages". Template: Inquisitor Proctor.

- Innate trait, Zealous Marrow Graft: Quell gains Strain when he takes damage; at 50+ (Flesh-Tension) his attacks
  pierce armour. Act 1 has no Strain (index gap 4).
- Altered maneuver, Sanctified Vivisection (from Sanctified Decapitation): Standard (2 blocks) + 2 recovery, reach 1,
  medium hit that pierces armour, poise damage 25, Arterial Rupture (bleed whenever the victim moves; new status
  `[kit hook]`).
- Capstone, Sacrificial Transfusion Pulse: a boss phase when he reaches 100 Strain or would die `[kit hook]` (BOSS1
  has HP triggers; a Strain trigger and a cheat-death trigger are new). Snap (1 block), radius 3: medium hit on every
  hero, +25 Strain into every operative (Act 2 only, index gap 4), then he heals 50 HP (a share of his max HP once
  tuned).
- Drops: see [Act 1 encounters](bestiary_encounters_act1.md) ENC-EVT-02.

## Inquisitor-Prelate Nemeth (boss, ENC-EVT-06 and ENC-ACT3-04)

The marrow-infused commander of the Crucible Watchpost "whom cold steel cannot finish"; he takes the founder who holds
the Slag Gate winch alive ([cast registry](../background/cast_registry.md); renamed from "High Inquisitor Bastion").
His infusion is aberrant marrow from older, weaker stock, which wears the body down ([CANON](../CANON.md) 3.6).
Template: Inquisitor Proctor. No trait, altered maneuver or capstone is written yet (Q-74).

- ENC-EVT-06, the Crucible (Act 1 M5): the cell does not defeat him; the objective is escape, and he escapes with the
  captured founder (user 2026-10-25, Q-25 a).
- ENC-ACT3-04, the Silent Ossuary (Act 3 node 20, optional; CANON 3.6): phase 1, Nemeth and his guard with the
  corrupted founder fighting at his side; when the founder breaks (an `hp_below` trigger on the founder with a round
  fallback, or earlier if Nemeth falls low, refinement), the founder kills Nemeth and his guard (a phase effect that
  kills named units `[kit hook]` new). Drop: Nemeth's Warded Cassock, where he dies
  ([items](items_gear_and_progression.md) 7).
- The corrupted founder: built from the founder's loadout with their signature moves twisted; in phase 2 a "hostile to
  all sides" AI or faction switch `[kit hook]` new and a unit swap for the monstrosity's larger body (art Q-79); see
  the [index](enemy_bestiary_and_ai.md) section 4.
