# Classes and skill trees (index)

Status: canon for the six classes, the Cold Steel branches and the founders' Act 1 kits; proposal for the Inoculated
branches and every Inoculated founder skill (Act 2 on). Class trees are `[data]` on the kit's skill hooks.

Numbers follow [CANON](../CANON.md) section 2.1; rules in [combat and the timeline](combat_and_timeline.md); how skills
are learned and slotted in [skill acquisition](skill_acquisition_and_loadout_systems.md); names and backgrounds in the
[cast registry](../background/cast_registry.md).

## 1. The six classes

| Engine class | Class | Role | Founder | File |
| --- | --- | --- | --- | --- |
| `warrior` | The Vanguard (garrison deserter) | tank, poise breaker, chokepoints | Kaelen | [Vanguard](class_vanguard.md) |
| `rogue` | The Gutter-Stalker (curfew runner) | skirmisher, executions, mobility | Vesper | [Gutter-Stalker](class_gutter_stalker.md) |
| `ranger` | The Arbalestier (crag poacher) | ranged interrupts, pins, marks | Hollis | [Arbalestier](class_arbalestier.md) |
| `brawler` | The Slag-Brute (bone-mill stoker) | stagger engine, forced movement, line breaker | Dunstan "Slag-Jaw" | [Slag-Brute](class_slag_brute.md) |
| `priest` | The Heretic Acolyte (defrocked scribe) | field surgery, Strain siphon (Act 2) | Maren | [Heretic Acolyte](class_heretic_acolyte.md) |
| `mage` | The Ichor Alchemist (hedge chemist) | fire and smoke zones, mutagens (Act 2) | Elyse | [Ichor Alchemist](class_ichor_alchemist.md) |

Act 1 has six founders; the first playable slice has five (Elyse joins in Act 1 mission 4). Act 2 recruits (18-24,
freed from culling wagons) use the base trees.

## 2. Two branches per class

- **Cold Steel** (Act 1 on): trained skill and forged steel. Higher AP (2-4 per action), no Strain, no lasting cost;
  answers armour with expose and poise breaks; owns Active Guard play.
- **The Bitter Inoculation** (Act 2 on): grafted titan marrow. Low AP (1-2) plus Strain 15-35 per action; attacks
  pierce armour; grafts raise the dormant floor; Tier III grafts advance Calcification.

Each branch has T1 active and passive, T2 active and passive, a T3 active and a keystone. Inoculated keystones lock
the dormant floor at 30; the older drafts' "every battle ended in the Aberrant Edge adds Calcification" cost on them
is retired (CANON 2.3), so their lasting cost is open (see gaps).

## 3. Conventions in the class files

- **Timing**: Snap = 1 block, Standard = 2, Heavy = 3-4; "+1 rec" = one recovery block. Converted from the old tick
  anchors: Snap 1, Standard 2, Heavy 3 (old impact on T4) or 4 (old impact on T5-6); an old Heavy recovery window
  becomes 1 recovery block; Snap and Standard recovery windows (tier-wide anchors) are dropped.
- **AP** is the skill's `ap_cost`; **Strain** is points (0-100), not percent.
- **Armour** is the engine rating (lore points x 2): "expose N" and "fortify N" are the engine's flat statuses.
- **Stagger** is the skill's base poise damage; the windup x2 is a general rule, never repeated in a skill.
- **Damage and healing** figures are the lore's flat values; data uses attack power x `power_mult` and heal =
  `heal_base` + stat x `heal_mult` (GAME_BRIEF). The conversion is a gap.
- **Profiles**: HP and poise come from the engine class's stats (HP = 60 + CONS x 8 + STR x 2; poise = CONS x 2 +
  armour poise); the lore's flat values (120 HP, poise 60, base armour 4) are retired; the class files keep the
  relative words only.
- **Statuses**: the engine has fifteen (stagger, stun, root, bleed, weaken, haste, taunt, riposte, slow, expose,
  fortify, regen, poison, burn, empower). Corrosion, blind, mark, frenzy (confusion), mutilated and disarmed are new
  rows or map onto one of these (open, decided with the data).
- **Cut riders** (evasion, accuracy, backstab, facing, stealth) are removed; where a skill depended on one the file
  says "replacement open".

## 4. Founders

Each founder has an innate trait, an authored variant of a base skill and an exclusive signature skill, all
character-locked: `[data]` as the world character's passive (source-granted) and `learned_only` skills given through
the character's `extra_skills`.

| Founder | Innate trait | Variant (modifies) | Signature | Act 1 kit |
| --- | --- | --- | --- | --- |
| Kaelen | Castellan's Penance | Notched Billhook Sunder (Sunder and Pin) | Seventh Ward Bulwark | all three |
| Vesper | Gutter-Runner's Reflex | Orlov's Retribution (Throat-Carver's Art) | Umbral Flensing | trait, variant |
| Hollis | Crag Hawkeye | Poacher's Barbed Quarrel (Notched Quarrel) | Predator Reticle Pulse | trait, variant |
| Dunstan "Slag-Jaw" | Foundry-Wired Spite | Shackle-Pummel Slam (Jaw-Breaker) | Molten Marrow Eruption | trait, variant |
| Maren | Copyist of St. Lyra's | Black-Ichor Transfusion (Ichor Thread Suture) | Martyr's Shroud | trait only |
| Elyse | Volatile Reaction Catalyst | Pressurized Vitriol Siphon (Caustic Vitriol) | Spore-Bloom of the Sleeper | trait only |

Inoculated variants and signatures (Strain cost listed) wait for Act 2. Founders are clean mortal humans on Day 1;
personal physical taints manifest only on the Act 2 Inoculated path (`CANON.md` 3.1, 3.3).

| | Founders | Act 2 recruits |
| --- | --- | --- |
| Source | authored dossiers and quests | freed from culling wagons |
| Look | unique sprites and taints | modular heads, aprons and rags |
| Innate trait | exclusive | rolled from a recruit trait table |
| Skills | base trees + variant + signature | base trees |
| Death | Act 1: downed, then wounded; Act 2 on: permadeath, a story beat | permadeath; the cell recruits again |
| On death (Act 2 proposal) | yields a unique named Apex organ | yields a standard class organ |

## 5. Data binding example `[data]`

Field names follow the proposals in [rules engine architecture](rules_engine_architecture.md) section 3.1; how a skill
is character-locked follows section 4 above.

```json
{
  "id": "kaelen_seventh_ward_bulwark",
  "name": "Seventh Ward Bulwark",
  "kind": "DEFEND",
  "level": 3,
  "learned_only": true,
  "blocks": 3,
  "recovery_blocks": 1,
  "ap_cost": 3,
  "strain_cost": 0,
  "stagger": 35,
  "armor_piercing": false,
  "area": "LINE",
  "power_mult": 1.2,
  "description": "Kaelen plants his tower shield: a 3-tile barrier for 2 rounds."
}
```

## 6. Gaps (open)

| Gap | Options | Recommendation |
| --- | --- | --- |
| Lasting cost of the Inoculated keystones (their Calcification-per-battle cost is retired) | A: the floor lock at 30 is the whole cost; B: installing a keystone is Tier III surgery and adds a Calcification stage | B: it reuses CANON's own Tier III rule |
| Flat damage and heal numbers | A: convert each to `power_mult` against a reference attack power at the asset pass; B: keep flat values through a new field | A |
| Flat poise bonuses on passives (Braced Poise +20) beside the CANON formula | A: allowed as a third term; B: passives add armour poise instead | A, stated in the poise spec |
| Skills whose identity was a cut rider (Ghost of the Undercroft, Viscous Shadow, Gutter-Runner's Reflex) | A: rebuild on engine hooks (`exploit` bonuses against statuses, chase, intent foresight); B: replace the skill | A, per class file |
| Enemy Strain (Miasma and Spore-Bloom add Strain to enemies) | A: aberrant enemies carry the meter, humans ignore it; B: no enemy Strain, swap the rider | A, Act 2 bestiary |
| Martyr's Shroud costs 3 AP, above the Inoculated 1-2 guideline | keep as a signature exception, or 2 AP | keep (`ap_cost` is decoupled) |
| Ocular Lock's mark: +35% (old tree table) or +30% crit (old text) | 30 (matches Predator Reticle Pulse) | 30 |
