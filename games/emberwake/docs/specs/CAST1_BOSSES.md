# CAST1 bosses: Vane, Quell, Grendel, Orlov, the Calcified Amalgam, Nemeth

Status: locked for Act 1 authoring (`2026-11-11`; updated for `CL-01`, `CL-08`, `CL-11`, `BIGUNIT1`, and shipped `talk` cues); part of [CAST1.md](CAST1.md).
Lore: [CANON.md](../lore/CANON.md) 3.2, [STORY_GUIDE.md](../lore/STORY_GUIDE.md) (a boss = a template, an innate trait, an altered maneuver and a capstone a phase
unlocks). Rules as CAST1_BESTIARY 1 (no AP, recovery blocks, the AI table, one world character per unit). Phases use
shipped `rpgkit_battle` effects (`line`, `talk`, `wave`, `status`, `ai`, `enrage`, `deck`, `kill`), `ENCOUNTER2`'s `unkillable` and
`timeout_outcome`, and `BIGUNIT1`'s `footprint: 2` (`calcified_amalgam`). `HOOKS1-Q`'s unbuilt `spare_once`/`spared` is retired
in favour of `hp_below: 35` + `regen` on Proctor Quell so M2 avoids immortality/burst-heal stalls (`P-COMBAT-03`). Hook notation as CAST1_FOUNDERS 1. All numbers FP.

Capstones (G19 through HOOKS1 10.1): each capstone is in the boss unit's `extra_skills` from the start but not in its
deck's `active`; the phase that unlocks it runs the `deck` effect (`unit` the boss, `add` the capstone, `order` a new
assault list with the capstone first), which slots it from the boss's next planning (DU1). Every boss deck below
leaves one active slot free for it (DU2: within `deck_slots`).

## 1. Units (`units.json`, CAST1-A4) FP

All tier boss, `flee_ratio` 0.0, `reckless` false unless noted, `pouch` ["none"], `extra_skills` = deck actives plus
the capstone.

| Id | Name | Class | Lvl | Role, targets | Gear (main, body) | `armor` | `poise` | `power` | Character `tuning` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `captain_vane` | Captain Vane | warrior | 3 | AGGRESSOR, NEAREST | `vanes_bill`, `watch_cuirass` | 50 | 60 | 0 | {} |
| `proctor_quell` | Proctor Quell | warrior | 3 | AGGRESSOR, WEAKEST | `quells_bone_saws`, none | 25 | 50 | 0 | {} |
| `overseer_grendel` | Overseer Grendel | brawler | 4 | AGGRESSOR, NEAREST | `foundry_sledge`, `skinners_apron` | 35 | 70 | 0 | {} |
| `guild_master_orlov` | Orlov | brawler | 4 | AGGRESSOR, WEAKEST | `flensing_hooks`, `skinners_apron` | 35 | 75 | 0 | {"hp_pct": 20} |
| `calcified_amalgam` | The Amalgam | mage | 5 | AGGRESSOR, NEAREST; reckless | none, none | 50 | 90 | 40 | {"hp_pct": 60} |
| `inquisitor_prelate_nemeth` | Nemeth | warrior | 6 | AGGRESSOR, TOUGHEST | `prelates_blade`, `watch_cuirass` | 50 | 80 | 0 | {} |

Short names: "Orlov" (Guild-Master Orlov), "The Amalgam" (the Calcified Amalgam), "Nemeth" (Inquisitor-Prelate
Nemeth); the full names go in the character `name` where the UI has room (the character `title` holds the rank).
The Amalgam's `power` 40 is an authored strike power (an unarmed mage would hit for STR + DEX); its HP comes from the
boss tier plus `hp_pct` 60 on its character; its armour 50 is the top of the Q-49 test scale (25-50, CAST1_GAPS C12).
Nemeth's TOUGHEST (HOOKS1 AT1) sends him at the sturdiest melee founder, the one most likely to hold the winch.

## 2. Skills (`skills.json`, CAST1-A3) FP

| Id | Name | Kind | Intent | Blocks | Rec. | Power | Stagger | Reach | Status, flags and hooks | Boss |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `gilded_billhook_sunder` | Gilded Sunder | ATTACK | assault | 3 | 2 | 2.0 | 15 | weapon | `weapon_types` [polearm]; area SWEEP; expose potency 90, magnitude 8, 2 rounds; push 1 (x0.5, st 6) | Vane |
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
| `scalding_cataclysm` | Cataclysm | ATTACK | assault | 3 | 2 | 1.8 | 15 | 1-2 | area BURST; bleed potency 80, 2 rounds; `forced_move` pull 1 | Orlov (capstone) |
| `ossified_spine_volley` | Spine Volley | ATTACK | assault | 2 | 2 | 1.0 | 10 | 1-3 | area BURST; bleed potency 70, 2 rounds | the Amalgam |
| `tendril_flail` | Tendril Flail | ATTACK | assault | 2 | 1 | 0.9 | 10 | 1-1 | area SWEEP | the Amalgam |
| `ossification_bloom` | Bone Bloom | ATTACK | assault | 4 | 1 | 1.5 | 15 | 0-3 | area BURST; `unbreakable`; root potency 80, 1 round | the Amalgam (capstone) |
| `sanctified_decapitation` | Decapitation | ATTACK | assault | 3 | 2 | 2.4 | 15 | 1-1 | `unbreakable`; `ignore_guard` | Nemeth |
| `prelates_censure` | Censure | ATTACK | assault | 2 | 1 | 1.0 | 10 | 1-1 | weaken potency 80, magnitude 20, 2 rounds | Nemeth |

All `learned_only`, one rank, level 1 (the bosses own them at any level), `round_intent` as listed. Full lore names
in the descriptions: Gilded Billhook Sunder, Seventh Ward Execution Wall, Sanctified Vivisection, Sacrificial
Transfusion Pulse, Pneumatic Hammer Strike, Bone-Mill Pulverizer, Pressurized Lard Spray, Flensing Hook Drag,
Scalding Flensing Cataclysm, Ossified Spine Volley, Marrow Tendril Flail, Catastrophic Ossification Bloom, Sanctified
Decapitation, the Prelate's Censure (new: Nemeth had no maneuver, Q-74).

- Hammer Strike's knock-down of adjacent units is the Staggered status (Q-56: the lore effect, not a stand-in).
- Decapitation "ignores Active Guard": its hits skip the guard term and P6's guard factor; cover still counts (HOOKS1
  IG1).

## 3. Decks, traits and phases (`talk` reactivity per `CL-11`)

Every Act 1 boss encounter pairs mechanical phase triggers with 1-2 line character-reactive `talk` cues (`{"type": "talk", "dialogue": "dlg_...", "speaker": "<unit_id>"}`), skipped cleanly if the speaker is not alive on the board.

### 3.1 Captain Vane (M1, `enc_evt_01`; Tier 1)

- Deck: active [`gilded_billhook_sunder`, `traitors_parry`]; order assault [`gilded_billhook_sunder`], guard
  [`traitors_parry`, `active_guard`]. He also owns `heavy_bill_thrust` (unslotted, not used) and `execution_wall`.
- Trait `traitors_discipline` "Traitor's Discipline" (character passive, `slot` ""): resist `stagger` 50, `stun` 50.
  Gilded Sunder's knockback is its push. Drill: `polearm_drill`.
- Phases:
  - `vane_taunt` `{"kind": "round", "round": 1}`: `talk` (`kaelen` -> `dlg_talk_m1_vane`, calling out Vane's candle-gold Bastion bribe-shield and Order 44).
  - `execution_wall` trigger `{"kind": "hp_below", "unit": "captain_vane", "pct": 50, "or_round": 3}`, effects
    line `"Lock shields! Seventh Ward, with me!"`, wave [`footman_shieldbearer`, `footman_shieldbearer`] label
    `"Shield-bearers lock the wall"`, enrage `captain_vane` 15, deck `{"unit": "captain_vane", "add":
    ["execution_wall"], "order": {"assault": ["execution_wall", "gilded_billhook_sunder"]}}`.
  - `vane_withdraws` (`CL-01`): `hp_below` 25, effects line `"This isn't over, deserter."`, `talk` (`captain_vane` -> `dlg_talk_m1_retreat`), ai `{"unit": "captain_vane", "flee_ratio": 1.0}`.
- Reward: `vanes_gilded_pavise` is the objective reward (paid on a win whether he fled or fell); Vane's final reckoning is Kaelen's (or Berek's inherited, `CL-09`) Act 2 quest.

### 3.2 Proctor Quell (M2, `enc_evt_02`; Tier 1): the brass marrow valve

- Deck: active [`sanctified_vivisection`, `inquisitorial_parry`]; order assault [`sanctified_vivisection`], guard
  [`inquisitorial_parry`, `active_guard`]; capstone `transfusion_pulse` (unlocked by the valve).
- Character passives: `bone_saw_drill`; `marrow_valve` "Marrow Valve" (`slot` ""): resist `bleed` 25; its inspect description
  is the pre-fight tell: `"An illegal brass marrow valve in his neck: when cut low, he floods his veins for one frantic surge."`
- The valve (`CL-08`, `P-COMBAT-03` — zero immortality/burst-heal stall): phase `marrow_valve` trigger
  `{"kind": "hp_below", "unit": "proctor_quell", "pct": 35, "or_round": 4}`, effects line `"The brass valve in Quell's neck hisses open—raw condensate floods his veins!"`, `talk` (`maren` -> `dlg_talk_m2_valve`, naming the silver saw Quell used on her Scriptorium mentor), status `{"unit": "proctor_quell", "status": "regen", "rounds": 3, "magnitude": 15}`, enrage 15, deck `{"unit": "proctor_quell", "add": ["transfusion_pulse"], "order": {"assault": ["transfusion_pulse", "sanctified_vivisection"]}}`. No Strain in Act 1; Quell fights to the death (`CL-01`).
- Opening & cull phases:
  - `quell_taunt` `{"kind": "round", "round": 1}`: `talk` (`maren` -> `dlg_talk_m2_cages`).
  - `cull_1` `{"kind": "round", "round": 3}`: line `"A flagellant raises the scourge over the first cage."`, kill `{"units": ["tithe_captive_a"], "by": "mortuary_flagellant", "within": 0}`.
  - `cull_2` `{"kind": "round", "round": 5}`: line `"Another cage falls silent."`, kill `{"units": ["tithe_captive_b"], "by": "mortuary_flagellant", "within": 0}`.
  Each `tithe_captive` (`18-24` Unsealed) surviving to the win adds `+1` `saved_youths` and `+1` `Rock Salt` (`CAST1_ENCOUNTERS` 3).

### 3.3 Overseer Grendel (M3, `enc_evt_03`; Tier 2)

- Deck: active [`pneumatic_hammer_strike`, `coal_shovel_jab`]; order assault [`pneumatic_hammer_strike`,
  `coal_shovel_jab`], guard [`active_guard`]; capstone `bone_mill_pulverizer`.
- Trait `molten_slag_tempering` "Slag Tempering" (`slot` ""): resist `burn` 100; `stagger_taken` mult 0.75,
  max_blocks 0.
- Phases:
  - `mill_oath` `{"kind": "round", "round": 1}`: `talk` (`dunstan` -> `dlg_talk_m3_grendel`, swearing to smash the chute that ground his Spent father's bones).
  - `pulverizer` `{"kind": "hp_below", "unit": "overseer_grendel", "pct": 50, "or_round": 4}`: line `"Feed them to the mill!"`, enrage 20, deck `{"unit": "overseer_grendel", "add": ["bone_mill_pulverizer"], "order": {"assault": ["bone_mill_pulverizer", "pneumatic_hammer_strike", "coal_shovel_jab"]}}`.
  The vents are the encounter's `arena_hazards` row `slag_vents` (`CAST1_ENCOUNTERS` 2). Grendel dies in `M3` (`CL-01`); Dunstan is an AI guest here and joins post-fight.

### 3.4 Guild-Master Orlov (M4, `enc_evt_04`; Tier 2): the vendetta

- Deck: active [`pressurized_lard_spray`, `flensing_hook_drag`]; order assault [`pressurized_lard_spray`,
  `flensing_hook_drag`], guard [`active_guard`]; capstone `scalding_cataclysm`.
- Trait `vat_masters_grease` "Vat-Master's Grease" (`slot` ""): resist `burn` 100, `poison` 50; the arena's `toxic_bog` tiles around his start stand in for his grease floor.
- Phases:
  - `orlov_taunt` `{"kind": "round", "round": 1}`: `talk` (`vesper` -> `dlg_talk_m4_orlov`, calling out the tallow tithe that killed her ash-sweeper brother, answered by guest `elyse` on `dlg_talk_m4_elyse`).
  - `cataclysm` `hp_below` 40 (`or_round: 4`): line `"Into the vats, the lot of you!"`, enrage 20, deck `{"unit": "guild_master_orlov", "add": ["scalding_cataclysm"], "order": {"assault": ["scalding_cataclysm", "pressurized_lard_spray", "flensing_hook_drag"]}}`.
  - `orlov_withdraws` `hp_below` 20 (`CL-01`): line `"Back to the upper vats—seal the sluices!"`, `talk` (`guild_master_orlov` -> `dlg_talk_m4_retreat`), `flee_ratio` `1.0` on `guild_master_orlov` so he retreats into Upper Cinderreach for Vesper's (or Talia's inherited, `CL-09`) Act 2 reckoning.
- Vendetta (`CL-01`, `PR5-34`): `objective_reward` grants `loot_flensing_talon` (`orlovs_flensing_talon`, dropped as he flees) and the M4 turn-in awards `loot_orlovs_retribution` + `+5` `Warded Tallow` whether Orlov fled or fell before fleeing.

### 3.5 The Calcified Amalgam (Causal Interlude, `enc_evt_05`)

- Deck: active [`ossified_spine_volley`, `tendril_flail`]; order assault [`ossified_spine_volley`, `tendril_flail`];
  capstone `ossification_bloom`.
- Trait: `amalgam_carapace` "Bone Carapace" (`slot` ""): resist `bleed` 75, `poison` 75, `root` 50.
- Footprint (`BIGUNIT1` `PR5-26a`): `footprint: 2` (`2x2`, `96x96` sprite frame, `AI-08`), flanked by `ash_crawler` x3 and `calcified_husk` x2 driven up into the Sluice Undercroft by the `M1–M4` surface marrow back-surge (`CL-08`).
- Phases:
  - `surge_breach` `{"kind": "round", "round": 1}`: `talk` (`kaelen` -> `dlg_talk_int_surge`, reacting to the ruptured drain-vault and `Sabina`'s emergency dose).
  - `bells` `{"kind": "round", "round": 2}`: line `"The cathedral bells toll. The Amalgam shudders in answer."` (camera shake cue).
  - `bloom` `hp_below` 30: line `"Bone spires split the floor."`, enrage 25, deck `{"unit": "calcified_amalgam", "add": ["ossification_bloom"], "order": {"assault": ["ossification_bloom", "ossified_spine_volley", "tendril_flail"]}}`.

### 3.6 Inquisitor-Prelate Nemeth (M5, `enc_evt_06`): the Crucible

- Deck: active [`sanctified_decapitation`, `prelates_censure`, `inquisitorial_parry`]; order assault
  [`sanctified_decapitation`, `prelates_censure`], guard [`inquisitorial_parry`, `active_guard`]. No capstone in
  Act 1.
- Trait `marrow_infused` "Marrow-Infused" (`slot` ""): `stagger_taken` mult 0.5, max_blocks 0; resist `stun` 75,
  `root` 75. With the encounter's `unkillable` (ENCOUNTER2 S1) he cannot be finished: `"Cold steel cannot finish him."`
- Phases:
  - arrival `gate_reinforcements` `{"kind": "arrival", "round": 3, "block": 1}` (shifted to `round: 4` in `enc_evt_06_overlook` when Hollis completed *"The Crag-Sight Overlook"*, `sunk_ward.hollis_overlook`, `CL-08`): wave [`castellan_footman`, `castellan_footman`] label `"The watchpost empties"`.
  - `infusion` `hp_below` `inquisitor_prelate_nemeth` 50 or round 4: line `"Cold steel cannot finish me."`, `talk` (`inquisitor_prelate_nemeth` -> `dlg_talk_m5_cold_steel`, answered by `kaelen`/`hollis` on `dlg_talk_m5_winch`), status `steadfast` 2 rounds on Nemeth, enrage 20.
  - `gate_closing` round 6: line `"The gate! Now, or never!"`.
- The escape, the winch and the capture are the encounter's (`stay_behind`, `party_min`, `CAST1_ENCOUNTERS`); Nemeth's
  Warded Cassock drops at node `20` (`3.4` Silent Ossuary, `CANON` 5.1).

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
