# CAST1 bosses: Vane, Quell, Grendel, Orlov, the Calcified Amalgam, Nemeth

Status: proposed (2026-10-25); part of [CAST1.md](CAST1.md). Lore: the bestiary faction files (a boss = a template, an
innate trait, an altered maneuver and a capstone a phase unlocks). Rules as CAST1_BESTIARY 1 (no AP, recovery
blocks, the AI table, one world character per unit). Phases use today's effects (line, wave, status, ai, enrage;
BOSS1, PACE1, PHASEFOLD) and ENCOUNTER2's `kill`, `unkillable` and `timeout_outcome`. All numbers FP.

Capstone note: no phase effect can add a skill or reorder a deck today, so each capstone is in the boss's deck from
the start, last in its order (used when nothing earlier fits), and the phase adds the line and an enrage. The lore's
"a phase unlocks the capstone" needs G19 (ENCOUNTER2-X's swap into a phase-two unit, or a phase `order` effect).

## 1. Units (`units.json`, CAST1-A4) FP

All tier boss, `flee_ratio` 0.0, `reckless` false unless noted, `pouch` ["none"], `extra_skills` = deck actives.

| Id | Name | Class | Lvl | Role, targets | Gear (main, body) | `armor` | `poise` | `power` | Character `tuning` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `captain_vane` | Captain Vane | warrior | 3 | AGGRESSOR, NEAREST | `vanes_bill`, `watch_cuirass` | 50 | 60 | 0 | {} |
| `proctor_quell` | Proctor Quell | warrior | 3 | AGGRESSOR, WEAKEST | `quells_bone_saws`, none | 25 | 50 | 0 | {} |
| `overseer_grendel` | Overseer Grendel | brawler | 4 | AGGRESSOR, NEAREST | `foundry_sledge`, `skinners_apron` | 35 | 70 | 0 | {} |
| `guild_master_orlov` | Orlov | brawler | 4 | AGGRESSOR, WEAKEST | `flensing_hooks`, `skinners_apron` | 35 | 75 | 0 | {"hp_pct": 20} |
| `calcified_amalgam` | The Amalgam | mage | 5 | AGGRESSOR, NEAREST; reckless | none, none | 50 | 90 | 40 | {"hp_pct": 60} |
| `inquisitor_prelate_nemeth` | Nemeth | warrior | 6 | AGGRESSOR, NEAREST | `prelates_blade`, `watch_cuirass` | 50 | 80 | 0 | {} |

Short names: "Orlov" (Guild-Master Orlov), "The Amalgam" (the Calcified Amalgam), "Nemeth" (Inquisitor-Prelate
Nemeth); the full names go in the character `name` where the UI has room (the character `title` holds the rank).
The Amalgam's `power` 40 is an authored strike power (an unarmed mage would hit for STR + DEX); its HP comes from the
boss tier plus `hp_pct` 60 on its character.

## 2. Skills (`skills.json`, CAST1-A3) FP

| Id | Name | Kind | Intent | Blocks | Rec. | Power | Stagger | Reach | Status and flags | Boss |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `gilded_billhook_sunder` | Gilded Sunder | ATTACK | assault | 3 | 2 | 2.0 | 15 | weapon | `weapon_types` [polearm]; area SWEEP; expose potency 90, magnitude 8, 2 rounds | Vane |
| `execution_wall` | Execution Wall | ATTACK | assault | 4 | 1 | 1.4 | 12 | 1-1 | area LINE; root potency 80, 1 round | Vane (capstone) |
| `traitors_parry` | Traitor's Parry | DEFEND | guard | 1 | 0 | - | 0 | self | mitigation 0.2; grant `riposte` 1 round | Vane |
| `sanctified_vivisection` | Vivisection | ATTACK | assault | 2 | 2 | 1.4 | 12 | 1-1 | `weapon_types` [bone_saw]; `armor_piercing` (CQ7); bleed potency 80, 2 rounds | Quell |
| `inquisitorial_parry` | Inquisitor's Parry | DEFEND | guard | 1 | 0 | - | 0 | self | mitigation 0.2; grant `riposte` 1 round | Quell, Nemeth |
| `transfusion_pulse` | Transfusion Pulse | ATTACK | assault | 1 | 2 | 0.9 | 6 | 1-2 | area BURST | Quell (capstone) |
| `pneumatic_hammer_strike` | Hammer Strike | ATTACK | assault | 3 | 2 | 2.2 | 20 | 1-1 | area SWEEP; `stagger` (status) potency 60 | Grendel |
| `coal_shovel_jab` | Shovel Jab | ATTACK | assault | 1 | 1 | 0.6 | 8 | 1-1 | burn potency 60, magnitude 3, 2 rounds | Grendel |
| `bone_mill_pulverizer` | Pulverizer | ATTACK | assault | 4 | 1 | 1.6 | 15 | 1-3 | area BURST; `unbreakable` | Grendel (capstone) |
| `pressurized_lard_spray` | Lard Spray | ATTACK | assault | 2 | 2 | 0.9 | 6 | 2-4 | area BURST; burn potency 80, magnitude 5, 2 rounds | Orlov |
| `flensing_hook_drag` | Hook Drag | ATTACK | assault | 2 | 1 | 0.9 | 8 | 1-3 | expose potency 80, magnitude 6, 2 rounds | Orlov |
| `scalding_cataclysm` | Cataclysm | ATTACK | assault | 3 | 2 | 1.8 | 15 | 1-2 | area BURST; bleed potency 80, 2 rounds | Orlov (capstone) |
| `ossified_spine_volley` | Spine Volley | ATTACK | assault | 2 | 2 | 1.0 | 10 | 1-3 | area BURST; bleed potency 70, 2 rounds | the Amalgam |
| `tendril_flail` | Tendril Flail | ATTACK | assault | 2 | 1 | 0.9 | 10 | 1-1 | area SWEEP | the Amalgam |
| `ossification_bloom` | Bone Bloom | ATTACK | assault | 4 | 1 | 1.5 | 15 | 0-3 | area BURST; `unbreakable`; root potency 80, 1 round | the Amalgam (capstone) |
| `sanctified_decapitation` | Decapitation | ATTACK | assault | 3 | 2 | 2.4 | 15 | 1-1 | `unbreakable` | Nemeth |
| `prelates_censure` | Censure | ATTACK | assault | 2 | 1 | 1.0 | 10 | 1-1 | weaken potency 80, magnitude 20, 2 rounds | Nemeth |

All `learned_only`, one rank, level 1 (the bosses own them at any level), `round_intent` as listed. Full lore names
in the descriptions: Gilded Billhook Sunder, Seventh Ward Execution Wall, Sanctified Vivisection, Sacrificial
Transfusion Pulse, Pneumatic Hammer Strike, Bone-Mill Pulverizer, Pressurized Lard Spray, Flensing Hook Drag,
Scalding Flensing Cataclysm, Ossified Spine Volley, Marrow Tendril Flail, Catastrophic Ossification Bloom, Sanctified
Decapitation, the Prelate's Censure (new: Nemeth had no maneuver, Q-74).

## 3. Decks, traits and phases

### 3.1 Captain Vane (M1, `enc_evt_01`)

- Deck: active [`gilded_billhook_sunder`, `execution_wall`, `traitors_parry`]; order assault
  [`gilded_billhook_sunder`, `execution_wall`], guard [`traitors_parry`, `active_guard`]. He also owns
  `heavy_bill_thrust` (unslotted, not used).
- Trait `traitors_discipline` "Traitor's Discipline" (character passive, `slot` ""): resist `stagger` 50, `stun` 50.
  Deferred: the aura (Watch allies within 3 tiles +10 poise, no poise damage from Snaps, G12); "his counter is always
  ready against Kaelen" (a per-target condition, G10); Gilded Sunder's knockback into rubble (G3). Drill:
  `polearm_drill`.
- Phases: `execution_wall` trigger `{"kind": "hp_below", "unit": "captain_vane", "pct": 50, "or_round": 3}`, effects
  line "Lock shields! Seventh Ward, with me!", wave [`footman_shieldbearer`, `footman_shieldbearer`] label
  "Shield-bearers lock the wall", enrage `captain_vane` 15. `vane_withdraws` (only under CQ5 a): `hp_below` 25,
  effects line "This isn't over, deserter.", ai `{"unit": "captain_vane", "flee_ratio": 1.0}`.
- Reward: the pavise is the objective reward (paid on a win whether he fled or fell).

### 3.2 Proctor Quell (M2, `enc_evt_02`): the marrow valve

- Deck: active [`sanctified_vivisection`, `inquisitorial_parry`, `transfusion_pulse`]; order assault
  [`sanctified_vivisection`, `transfusion_pulse`], guard [`inquisitorial_parry`, `active_guard`].
- Character passives: `bone_saw_drill`; `marrow_valve` "Marrow Valve" (`slot` ""): resist `bleed` 25; its description
  is the tell the player can read before the fight: "An illegal marrow valve in his neck. Once, it will not let him
  fall." The inspect card shows it.
- The valve (Q-74: a cheat-death boss phase, once, a heal and a tell; no Strain in Act 1). Target design, when G18
  lands: Quell is spared once (the first lethal blow leaves 1 HP), a phase on that event fires at the next block start
  with a line, a heal of 40% of max HP, `regen` 3 rounds, enrage 15, then he is mortal. Act 1 stand-in on today's
  effects: phase `marrow_valve` trigger `{"kind": "hp_below", "unit": "proctor_quell", "pct": 30}`, effects line "The
  valve in Quell's neck hisses open. He rises, flushed and grinning.", status `{"unit": "proctor_quell", "status":
  "regen", "rounds": 3, "magnitude": 10}`, status `fortify` 2 rounds magnitude 6, enrage 15. The stand-in fires
  before death, so a big hit can skip it; G18 closes that.
- Deferred: Zealous Marrow Graft (Strain, Act 2, Q-65); Arterial Rupture on Vivisection (bleed stands in, G15); the
  pulse's +25 Strain to every operative (Act 2).
- The cull (ENCOUNTER2 EQ8 a): phases `cull_1` `{"kind": "round", "round": 3}`: line "A flagellant raises the
  scourge over the first cage.", kill `{"units": ["tithe_captive_a"], "by": "mortuary_flagellant", "within": 0}`;
  `cull_2` round 5: line "Another cage falls silent.", kill [`tithe_captive_b`]. With the valve, three phases.

### 3.3 Overseer Grendel (M3, `enc_evt_03`, in the slice)

- Deck: active [`pneumatic_hammer_strike`, `coal_shovel_jab`, `bone_mill_pulverizer`]; order assault
  [`pneumatic_hammer_strike`, `coal_shovel_jab`, `bone_mill_pulverizer`], guard [`active_guard`].
- Trait `molten_slag_tempering` "Slag Tempering" (`slot` ""): resist `burn` 100; `stagger_taken` mult 0.75,
  max_blocks 0. Deferred: no poise damage while on burning or rubble tiles (G8); hits setting terrain alight (G16);
  the shockwave's knock-down of adjacent units (G3; SWEEP and the Staggered status stand in); the Pulverizer's rubble
  (G16).
- Phases: `pulverizer` `hp_below` `overseer_grendel` 50 or round 4: line "Feed them to the mill!", enrage 20.
  `vents` `{"kind": "round", "round": 3}`: line "The slag vents roar." (the scheduled vent bursts are G24; the arena's
  burning tiles stand in).
- Dunstan is a guest in this fight (CAST1_ENCOUNTERS).

### 3.4 Guild-Master Orlov (M4, `enc_evt_04`): the vendetta

- Deck: active [`pressurized_lard_spray`, `flensing_hook_drag`, `scalding_cataclysm`]; order assault
  [`pressurized_lard_spray`, `flensing_hook_drag`, `scalding_cataclysm`], guard [`active_guard`].
- Trait `vat_masters_grease` "Vat-Master's Grease" (`slot` ""): resist `burn` 100, `poison` 50. Deferred: the
  boiling-fat aura that slows and makes Flammable (G12, G17); the arena's grease tiles (`toxic_bog`) around his start
  stand in. The cataclysm's drag-in (G3) and second status expose 8 (G9).
- Phases: `cataclysm` `hp_below` 40: line "Into the vats, the lot of you!", enrage 20. The lore's second trigger "when
  Vesper attacks him" is a per-hero trigger (G10).
- Vendetta (CQ10 a): key loot on Orlov's kill pays `loot_flensing_talon` and `loot_orlovs_retribution`.

### 3.5 The Calcified Amalgam (interlude, `enc_evt_05`)

- Deck: active [`ossified_spine_volley`, `tendril_flail`, `ossification_bloom`]; order assault
  [`ossified_spine_volley`, `tendril_flail`, `ossification_bloom`].
- Trait: none in Act 1 (Resonant Titan Nexus reads Strain and Calcification: idle until Act 2, Q-65).
  `amalgam_carapace` "Bone Carapace" (`slot` ""): resist `bleed` 75, `poison` 75, `root` 50.
- Staging until BIGUNIT1 (Q-79; CQ15): a 1-cell unit on a placeholder body (CAST1_ASSETS); its bulk is HP, armour 50,
  poise 90 and two Calcified Husks beside it. When BIGUNIT1 lands, the same unit row gains the 2 x 2 footprint and the
  96 x 96 sheet; no other data changes. The Spine Volley hits its own allies too (areas strike every unit).
- Phases: `bells` `{"kind": "round", "round": 2}`: line "The cathedral bells toll. The Amalgam shudders in answer."
  (FX1's camera shake cue). `bloom` `hp_below` 30: line "Bone spires split the floor.", enrage 25. The lore's warded
  safe tiles (G8) and the second trigger "on a poise break" (G5) wait.

### 3.6 Inquisitor-Prelate Nemeth (M5, `enc_evt_06`): the Crucible

- Deck: active [`sanctified_decapitation`, `prelates_censure`, `inquisitorial_parry`]; order assault
  [`sanctified_decapitation`, `prelates_censure`], guard [`inquisitorial_parry`, `active_guard`].
- Trait `marrow_infused` "Marrow-Infused" (`slot` ""): `stagger_taken` mult 0.5, max_blocks 0; resist `stun` 75,
  `root` 75. With the encounter's `unkillable` (ENCOUNTER2 S1) he cannot be finished: "cold steel cannot finish him".
  His infusion is crude marrow (CANON 3.6) but carries no Strain in Act 1.
- Phases: arrival `gate_reinforcements` `{"kind": "arrival", "round": 3, "block": 1}`: wave [`castellan_footman`,
  `castellan_footman`] label "The watchpost empties"; `infusion` `hp_below` `inquisitor_prelate_nemeth` 50 or round 4:
  line "Cold steel cannot finish me.", status `steadfast` 2 rounds on Nemeth, enrage 20; `gate_closing` round 6: line
  "The gate! Now, or never!".
- The escape, the winch and the capture are the encounter's (`stay_behind`, CAST1_ENCOUNTERS); Nemeth's Warded
  Cassock is not paid here (it drops at node 20, CANON 3.6).

## 4. Loot (unit `loot` tables; relics are paid by the encounters)

| Unit | Rows |
| --- | --- |
| `captain_vane` | `vanes_bill` 25, `sulfur_poultice` 20 |
| `proctor_quell` | `brimstone_draught` 30 |
| `overseer_grendel` | `calcified_bone_shard` 40, `foundry_sledge` 10 |
| `guild_master_orlov` | `caustic_gall_sac` 40 |
| `calcified_amalgam` | `calcified_bone_shard` 60 |
| `inquisitor_prelate_nemeth` | none (he is never defeated in Act 1) |

Passive rows for the traits (`passives.json`, `slot` ""): `traitors_discipline`, `marrow_valve`,
`molten_slag_tempering`, `vat_masters_grease`, `amalgam_carapace`, `marrow_infused`.
