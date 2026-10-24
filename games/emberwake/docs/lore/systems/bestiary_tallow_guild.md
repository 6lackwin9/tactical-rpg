# Bestiary: the Tallow Guild and the Fourth Kiln Syndicate

Status: proposal (maneuvers and tuning targets); names and roles canon. Index, conversion rules and gaps:
[enemy bestiary](enemy_bestiary_and_ai.md). Every unit row is `[data]`; AI lines are `[kit hook]` (AI profile).

The Guild is the corpse-rendering cartel of the tallow vats; the Syndicate is its foundry and bone-mill arm at the
Fourth Kiln. Butchery tools, scalding grease, trained beasts.

| Unit | Tier | Class (proposal) | Bulk | Armour rating | Poise | Mobility | Role |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Tallow Dredger | base (standard) | rogue | 0.8 | 2 | 25 | quick | skirmisher, grease and knife |
| Render-Beast | base (minion) | rogue (beast body) | 0.6 | 0 | 20 | fast | pack hunter, Snap pounce |
| Guild Carver | elite | brawler | 1.6 | 6 | 50 | normal | hook and cleave |
| Guild-Master Orlov | boss | brawler | 3.2 | 8 | 80 | normal | "the Ash-Crawl Butcher" |
| Overseer Grendel | boss | brawler | 3.5 | 8 | 85 | normal | the bone-mill slaver (Syndicate) |

## Tallow Dredger (base)

- Look: skinner's apron slick with yellow corpse grease, copper goggles, wide boning knife, ceramic pots of boiling
  lard.
- Scalding Grease Pot: Standard (2 blocks), range 3, a 2 x 2 grease puddle for 2 rounds: units in it are slowed and
  Flammable (+50% burn damage; new status `[kit hook]`). The puddle is a ground hazard a skill leaves (index gap 2).
- Boning Knife Flurry: Snap (1 block), reach 1, light hit, bleed.
- AI `[kit hook]`: throws grease in front of advancing melee operatives, then flanks.

## Render-Beast (base)

- Look: starved, hairless, mastiff-sized, bone ridges along the spine, jaws wired with iron; fed on human flesh.
- Innate trait, Tenacious Chase: if its target moves during its windup, it follows up to 2 tiles to stay in contact
  `[kit hook]` new.
- Throat-Clamp Pounce: Snap (1 block), a 2-tile leap (`dash_tiles` exists), medium hit, poise damage 20, Pinned (root on
  both until the beast takes damage; the release rule is new).
- AI `[kit hook]`: hunts in pairs; skips the front line to pin isolated ranged operatives (Hollis, Elyse).
- Ownership: also the Inquisition's Lockdown hunting packs (index gap 7).

## Guild Carver (elite)

- Look: obese, heavily muscled slaughterhouse foreman, spiked iron belly plate, a pair of serrated meat hooks.
- Innate trait, Meat-Hook Pull: a hook hit pulls the target 1 tile closer (index gap 2; the old "evasion to 0%" is
  dropped with the evasion rolls).
- Flensing Hook Drag: Standard (2 blocks), range 3, medium hit, drags the target next to the Carver (index gap 2),
  expose 6.
- Butcher's Cleave: Heavy (3 blocks) + 2 recovery, reach 1, heavy hit, poise damage 35; double damage on a target below
  40% HP `[kit hook]` new (an execute rider).
- AI `[kit hook]`: drags back-line operatives into reach, then cleaves.

## Guild-Master Orlov (boss, ENC-EVT-04)

Owner of the Sunk Ward's rendering pipelines; his Guild paid the informant who sold out Vesper's runner ring
([cast registry](../background/cast_registry.md)). Iron-plated butcher's harness, a steam-pressured lard cannon, barbed
flensing hooks. Boss of Act 1 M4 "The Tallow Vats". Template: Guild Carver.

- Innate trait, Vat-Master's Caustic Grease: the floor within 2 tiles of Orlov is boiling fat; an operative moving
  there is slowed and becomes Flammable (aura and hazard: index gap 2; the old "Poise check or slip" is a roll, index
  gap 3).
- Altered maneuver, Pressurized Lard Spray (from Flensing Hook Drag): Standard (2 blocks) + 2 recovery, range 4,
  3-tile cone, medium hit, burn; the floor burns for 2 rounds. The old Blind rider is dropped (index gap 3).
- Capstone, Scalding Flensing Cataclysm: a boss phase at 40% HP `[kit hook]` exists (BOSS1); the old second trigger
  "when Vesper attacks him" is new (a per-hero trigger). Heavy (3 blocks) + 2 recovery, reach 2, every unit in reach
  is dragged in (index gap 2), heavy hit, expose 8, bleed and burn.
- Drops: see [Act 1 encounters](bestiary_encounters_act1.md) ENC-EVT-04.

## Overseer Grendel (boss, ENC-EVT-03)

The Fourth Kiln bone-mill slaver who branded Dunstan "Slag-Jaw" ([cast registry](../background/cast_registry.md)).
Iron-spiked coal shovel and a pneumatic slag hammer. Boss of Act 1 M3 "The Bone-Mill Raid". Template: Guild Carver as a
foundry stoker (brawler).

- Innate trait, Molten Slag Tempering: his hits set burnable terrain alight and expose the target 2 more per hit; no
  poise damage reaches him while he stands on burning or rubble tiles (terrain-conditional rule `[kit hook]` new).
- Altered maneuver, Pneumatic Hammer Strike (from Butcher's Cleave): Heavy (3 blocks) + 2 recovery, reach 1, heavy hit,
  poise damage 40 to the target and a shockwave that knocks down every adjacent unit (index gap 2).
- Capstone, Bone-Mill Pulverizer: a boss phase at 50% HP `[kit hook]` exists. Heavy (4 blocks) + 1 recovery, range 3,
  a 3 x 3 demolition: heavy hit on the units there, and the tiles become impassable rubble for the rest of the fight
  (terrain a skill changes: index gap 2).
- Drops: see [Act 1 encounters](bestiary_encounters_act1.md) ENC-EVT-03.
