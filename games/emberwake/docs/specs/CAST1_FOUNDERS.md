# CAST1 founders: the six Act 1 kits

Status: proposed (2026-10-25; rewritten to the HOOKS1 hooks by CAST1-R the same day); part of [CAST1.md](CAST1.md)
(conventions 3, open questions 9). Lore: the class files `games/emberwake/docs/lore/systems/class_*.md` (Cold Steel
branches and founders' Act 1 kits are canon). Hook fields follow `docs/specs/HOOKS1_TASKS.md` 1 (schema) and the
rows of its section 6; a lore effect no spec covers is listed as deferred with its gap (CAST1_GAPS). All numbers FP.

## 1. Shared rules for the six kits

- Each founder is a world character (role party) on its own catalog unit `founder_<id>` (class = the base class, side
  party, tier hero, level 1, `stat_mode` derived, `spend` {}). The unit carries the gear, the pouch, `extra_skills`
  (the founder's CAST1 skills) and the `deck`; the character carries the trait and the passives (`passives`, source
  granted, RULESET1 D4), the drill discipline (`disciplines`), the `title` and the `keepsake` (`keepsake_<id>`,
  CAST1_GEAR 7; ROSTER1 KS2, RQ7 a). No `gone_flags`: a captured founder reads as `roster.<id>` = `captured`
  (ROSTER1, e.g. `{"key": "roster.kaelen", "value": "captured"}`).
- Class names come from the `sunk_ward` manifest's `classes` overlay (HOOKS1 12; rows in HOOKS1_TASKS 2.4: `warrior`
  Vanguard, `rogue` Gutter-Stalker, `priest` Heretic Acolyte, `ranger` Arbalestier, `brawler` Slag-Brute, `mage` Ichor
  Alchemist). The character `title` is "" or an epithet (Dunstan "Slag-Jaw"), never the class name.
- Each class's `skill_path` is {} in the Sunk Ward (HQ5 a): a founder owns its `extra_skills` and the ruleset's basics
  (RULESET1 9), no Ashlands family. Proficiency for a family the class lacks stays a drill passive (CQ1 a, HQ6 a).
- Skill rows: `learned_only` true, one rank (tier 1), `family` = the id; `level` is the unlock level (CQ3); every row
  sets `round_intent`. Party units book no recovery (RULESET1 T4), so founder-only rows leave `recovery_blocks` 0.
- The basics (`move`, `rest`, `strike`, `offhand_jab`, `active_guard`, `flank_cut`) are open to every founder. With
  no Support skill a founder still has Support through pouch items (RULESET1 IN5).
- Notation: "push 2 (x0.5, st 6)" = `forced_move` `{"kind": "push", "tiles": 2, "collide_mult": 0.5,
  "collide_stagger": 6}` (`from` source, `when` always unless named); "trigger `on` (`when`): effects" = the passive
  effect `{"type": "trigger", "on", "when", "effects", "limit"}`, `limit` round unless named, `to` self unless named.
- Order lists below are the starting `deck.order` (intent -> families, highest priority first). At level 1 the deck's
  `active` holds the level-1 skills; a locked family is not slottable until its level, and the player slots it then.
- Dunstan and Elyse join through the dialogue's `recruit` with `level` `average` (ROSTER1 BK5; CQ14 a), written by
  the world's dialogue data, not CAST1; a fifth member needs ROSTER1-L's bunks (BK1).

## 2. Kaelen, the Vanguard (`founder_kaelen`, class `warrior`)

Gear: main `notched_billhook` (polearm, two-handed, reach 1-2); body `castellan_breastplate`; pouch
`sulfur_poultice` x1. Discipline `castellan_drill` (passive `polearm_drill`). Title "".

Skills (FP):

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status, flags and hooks | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `pommel_strike` | Pommel Strike | ATTACK | assault | 1 | 1 | 2 | 0.7 | 12 | 1-1 | - | Snap follow-up after any opener; the windup breaker |
| `intercepting_hook` | Intercepting Hook | DEFEND | guard | 1 | 2 | 3 | - | 0 | self | mitigation 0.3; `triggers` [`ally_hit` (`range` 1, `melee`): `redirect` 100; `move` pull 1 `to` other] | Guard opener: takes the melee hit meant for an adjacent ally, hooks the attacker in |
| `billhook_sunder` | Billhook Sunder | ATTACK | assault | 2 | 3 | 4 | 1.8 | 18 | band | `weapon_types` [polearm]; expose potency 90, magnitude 8, 3 rounds | Heavy opener, then Pommel in b4-b6 |
| `seventh_ward_bulwark` | Ward Bulwark | DEFEND | guard | 3 | 3 | 3 | - | 0 | self | mitigation 0.5; grant `taunt` 2 rounds; `extra_grants` fortify magnitude 6, 2 rounds | Guard opener, then Active Guard |

- Intercepting Hook (HQ1 a): while the Defend runs, the first melee hit a foe lands on a friend within 1 tile lands
  whole on Kaelen (HOOKS1 8.4, intercept: damage, status roll, stagger, events), and the attacker is pulled 1 tile
  toward him (FM9). `limit` round. No `weapon_types` (a Guard row). 0b teaches it.
- Billhook Sunder is the variant of Sunder and Pin (Heavy 3, one block faster): Kaelen owns it, not the base row.
  Deferred: the lore's root (a second status, G9) and "disarm on Clerisy or Inquisition targets" (G10). Its full name
  "Notched Billhook Sunder" goes in the description.
- Seventh Ward Bulwark (`name` "Ward Bulwark", full name in the description; level 3, not in the slice's first
  fights): the 3-tile barrier, the line damage and "untargetable by ranged" wait for barriers and line of sight (G6);
  the taunt draws the attacks onto Kaelen meanwhile. No tower shield: he plants the billhook haft and his pauldron
  (CQ16 a).

Passives, keystone and trait (FP):

| Id | Name | `slot` | Effects | Lore effect still deferred |
| --- | --- | --- | --- | --- |
| `braced_poise` | Braced Poise | passive | stat `max_poise` +20; `forced_move` `immune` guarding | - |
| `kinetic_leverage` | Kinetic Leverage | passive | trigger `guarded_hit`: `ap` 1 | - |
| `unbroken_phalanx` | Unbroken Phalanx | keystone | trigger `guard_start`: grant `steadfast` 1 round, `until` `guard_end` | crit immunity while guarding (G11) |
| `castellans_penance` | Castellan's Penance | "" | trigger `ally_hit` (`range` 1): `guard` mitigation 0.5, blocks 1; `redirect` 35 | - |

Penance and the Hook both answer `ally_hit`; which one takes a hit while the Hook runs is CQ18 (HOOKS1 8.4 allows
one redirect a hit).

Deck at level 1: active [`pommel_strike`, `intercepting_hook`]; passive [`braced_poise`, `kinetic_leverage`]; keystone
[`unbroken_phalanx`]. Full deck from level 3: active [`pommel_strike`, `billhook_sunder`, `seventh_ward_bulwark`] or
the player's pick.

| Intent | Order (level 1) | Order (full) |
| --- | --- | --- |
| assault | `pommel_strike`, `strike` | `billhook_sunder`, `pommel_strike`, `strike` |
| guard | `intercepting_hook`, `active_guard` | `seventh_ward_bulwark`, `intercepting_hook` (when slotted), `active_guard` |
| maneuver | `flank_cut` | `flank_cut` |
| support | - (pouch) | - (pouch) |

## 3. Vesper, the Gutter-Stalker (`founder_vesper`, class `rogue`)

Gear: main `curfew_bodkin` (dagger); body `soot_leathers`; pouch `sulfur_poultice`, `pitch_fire_bomb`. No discipline
(daggers are a rogue proficiency). Title "".

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status, flags and hooks | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `hamstring_sever` | Hamstring Sever | ATTACK | maneuver | 1 | 1 | 2 | 0.65 | 6 | 1-1 | `weapon_types` [dagger]; `dash_tiles` 3; slow potency 75, 2 rounds; exploit `flanked` x1.3 | Engage or Disengage opener; Snap repeats |
| `ash_cloud_fade` | Ash-Cloud Fade | DEFEND | guard | 1 | 1 | 2 | - | 0 | self | mitigation 0.2; grant `stealth` 2 rounds; `hazard` {`smoke`, BURST, self, 2} | Guard opener: smoke over the 3x3 around her |
| `throat_carver` | Throat-Carver | ATTACK | assault | 2 | 2 | 4 | 2.2 | 8 | 1-1 | `weapon_types` [dagger]; exploit `stagger` x1.5, `root` x1.5 | the finisher on a staggered or pinned foe |
| `orlovs_retribution` | Retribution | ATTACK | maneuver | 2 | 2 | 3 | 2.0 | 8 | 1-1 | `weapon_types` [dagger]; `dash_tiles` 2; exploit `stagger` x1.5, `root` x1.5 | Disengage: strike, then slip 2 tiles |

- Ash-Cloud and Fade: the Defend places Soot Smoke at its start (HOOKS1 GH2; row `smoke`: cover 0.3 against ranged
  hits for every unit in it, 2 rounds) and grants stealth (RULESET1 GD5). No jump (Q-54). 0b's lesson: hide in the
  smoke.
- Throat-Carver's Art: the execute below 35% and its 2 AP back wait (G7); the exploit on Staggered and Rooted carries
  "strike the staggered and the pinned". Name short form "Throat-Carver".
- Orlov's Retribution (variant of Throat-Carver's Art, one AP cheaper): not owned at the start; it is the vendetta
  reward at M4 (CQ10 a; loot row `loot_orlovs_retribution`, CAST1_GEAR 8). Deferred: the teleport on a kill (G2); the
  Disengage carried retreat on every use carries it. Short form "Retribution".

| Id | Name | `slot` | Effects | Lore effect still deferred |
| --- | --- | --- | --- | --- |
| `prowlers_cadence` | Prowler's Cadence | passive | resist `slow` 40, `root` 40 | no extra move cost in cover, mist and rubble (G8) |
| `anatomical_flaw` | Anatomical Flaw | passive | `skill_potency` family `throat_carver` +0.15, family `hamstring_sever` +0.15 | ignore armour and +25% crit on stunned, staggered or rooted targets (G11) |
| `ghost_of_the_undercroft` | Ghost of the Undercroft | keystone | `foresight` (no range); `ap_regen` +1 | Q-58 a: rebuilt on foresight and stealth |
| `cinder_childs_reflex` | Cinder-Child's Reflex | "" | trigger `evaded`: grant `haste` 1 round | - |

Deck at level 1: active [`hamstring_sever`, `ash_cloud_fade`]; passive [`prowlers_cadence`, `anatomical_flaw`];
keystone [`ghost_of_the_undercroft`].

| Intent | Order (level 1) | Order (full) |
| --- | --- | --- |
| assault | `strike` | `throat_carver`, `strike` |
| guard | `ash_cloud_fade`, `active_guard` | `ash_cloud_fade`, `active_guard` |
| maneuver | `hamstring_sever`, `flank_cut` | `orlovs_retribution`, `hamstring_sever`, `flank_cut` |
| support | - (pouch) | - (pouch) |

## 4. Maren, the Heretic Acolyte (`founder_maren`, class `priest`)

Gear: main `bronze_censer` (censer); off `copied_psalter` (tome); body `embalmers_habit`; pouch `sulfur_poultice`
x2. Discipline `scriptorium_rites` (passive `censer_drill`). Title "".

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Heal | Reach | Status, flags and hooks | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `brimstone_cauterize` | Cauterize | SUPPORT | support | 1 | 1 | 2 | base 10, KNOW x1.2 | 0-2 | ALLY_OR_SELF; cleanse `bleed`, `poison`, `burn` | Snap follow-up after Tourniquet; a cleanse bottles a phial |
| `tourniquet_tonic` | Tourniquet | SUPPORT | support | 1 | 2 | 3 | base 25, KNOW x2.0 | 0-2 | ALLY_OR_SELF; grant `haste` 1 round; `riders` [`other_broken`: `guard` mitigation 0.5, blocks 0] | the big heal opener |
| `bile_coat` | Bile Coat | SUPPORT | support | 1 | 1 | 0 | none | 1-2 | ALLY; `meters` [`bile_phial` -1]; grant `empower` magnitude 15, 1 round | free Snap: coats an ally's weapon |
| `resilience_ward` | Resilience Ward | SUPPORT | support | 2 | 3 | 4 | none | 0-3 | ALLY_OR_SELF; grant `steadfast` 1 round; `extra_grants` fortify magnitude 8, 1 round | before a heavy enemy round |

- Short forms: "Cauterize" (Brimstone Cauterize), "Tourniquet" (Tourniquet and Tonic). At KNOW 15: Cauterize heals
  28, Tourniquet 55.
- Tourniquet and Tonic: an ally broken this round also holds a guard of 0.5 for the rest of the round (HOOKS1 RD1,
  8.3).
- Bile Coat (HQ4 a) is a meter skill: listed in the `bile_phial` row's `skills` (HOOKS1_TASKS 2.2), usable while
  Maren holds a phial, outside the deck (HOOKS1 UM6); not in `extra_skills` and not slottable.
- Resilience Ward: one ally, not the squad within 4 tiles (area support, G12); steadfast is "cannot be broken",
  fortify 8 stands in for +30% mitigation.
- Maren's Assault is the basic Strike with the censer; her Inoculated variant and signature are Act 2.

| Id | Name | `slot` | Effects | Lore effect still deferred |
| --- | --- | --- | --- | --- |
| `anatomical_precision` | Anatomical Precision | passive | stat `max_poise` +8; resist `stun` 25 | an aura: allies within 2 take 15% less stagger (G12) |
| `triage_efficiency` | Triage Efficiency | passive | trigger `heal_given` (`other_hp_below` 30): `ap` 1 | - |
| `pure_hippocratic_oath` | Hippocratic Oath | keystone | `skill_potency` group `heal` +0.4 | human enemies cannot crit her (G11) |
| `copyist_of_st_lyras` | Copyist of St. Lyra's | "" | `meter` `bile_phial`; trigger `cleanse_given`: `meter` `bile_phial` +1, `limit` none | - |

The Bile Phial (Q-61 b): 0-3 pips, 0 at each battle's start (HOOKS1 9; row `bile_phial`).

Deck at level 1: active [`brimstone_cauterize`, `tourniquet_tonic`]; passive [`anatomical_precision`,
`triage_efficiency`]; keystone [`pure_hippocratic_oath`].

| Intent | Order (level 1) | Order (full) |
| --- | --- | --- |
| support | `tourniquet_tonic`, `brimstone_cauterize` (then `bile_coat`, FU4) | `resilience_ward`, `tourniquet_tonic`, `brimstone_cauterize` (then `bile_coat`) |
| assault | `strike` | `strike` |
| guard | `active_guard` | `active_guard` |
| maneuver | `flank_cut` | `flank_cut` |

## 5. Hollis, the Arbalestier (`founder_hollis`, class `ranger`)

Gear: main `winch_arbalest` (crossbow, range 4); body `poachers_brigandine`; pouch `sulfur_poultice`. No discipline
(crossbow is a ranger proficiency). Title "".

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status, flags and hooks | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `poachers_quarrel` | Barbed Quarrel | ATTACK | assault | 1 | 2 | 2 | 0.95 | 8 | weapon band | `weapon_types` [crossbow]; bleed potency 70, 2 rounds | Standard; repeats |
| `tripwire_snare` | Tripwire Snare | ATTACK | assault | 1 | 1 | 3 | 0 | 0 | 1-3 | placement: `hazard` {`tripwire`, NONE, target, 3}; aims at a tile; no status | Snap: wire a tile a foe will cross |
| `heavy_winch_sunder` | Winch Sunder | ATTACK | assault | 2 | 4 | 5 | 2.1 | 12 | weapon band | `weapon_types` [crossbow]; `sure_break`; expose potency 90, magnitude 6, 2 rounds; push 2 (x0.5, st 6) | Heavy opener into a telegraphed windup |

- Poacher's Barbed Quarrel is the variant of Notched Quarrel (Hollis owns it, not the base): range 5 in the lore comes
  from the arbalest's range 4 plus PERC. Deferred: the tearing barbs on a moving target (G15); bleed stands in. Short
  form "Barbed Quarrel".
- Tripwire Snare (HQ12 a): an Assault aimed at a tile (HOOKS1 GH3, RULESET1_INTENT 2). The `tripwire` row is hidden
  from the enemy AI; the first foe to enter takes 15, rolls root 90 and stops its walk; it lasts 3 rounds. 0c's lesson.
- Heavy Winch Sunder: "the target's windup breaks outright" is `sure_break` (RULESET1 P14), AP 4 + 1; the knockback
  is the push (a wall or a unit behind takes the collision, HOOKS1 FM4). Short form "Winch Sunder".

| Id | Name | `slot` | Effects | Lore effect still deferred |
| --- | --- | --- | --- | --- |
| `high_ground_traverse` | High-Ground Traverse | passive | stat PERC +2 | +1 range from rubble or roof tiles (G8) |
| `cover_breaker` | Cover Breaker | passive | `status_potency` +15 | halve the target's cover; +25% crit and +15 stagger on a windup (G11) |
| `dead_eye_discipline` | Dead-Eye Discipline | keystone | trigger `kill` (`ranged`): `ap` 2; `reveal` 1 | - |
| `crag_hawkeye` | Crag Hawkeye | "" | `foresight` range 4 | ignores partial cover (G8); Q-58 a: the reveal is his |

Dead-Eye's `reveal` shows him the enemies' next-round plan (HQ7 a; RULESET1 V2).

Deck at level 1: active [`poachers_quarrel`, `tripwire_snare`]; passive [`high_ground_traverse`, `cover_breaker`];
keystone [`dead_eye_discipline`].

| Intent | Order (level 1) | Order (full) |
| --- | --- | --- |
| assault | `tripwire_snare`, `poachers_quarrel`, `strike` | `heavy_winch_sunder`, `tripwire_snare`, `poachers_quarrel` |
| guard | `active_guard` | `active_guard` |
| maneuver | `flank_cut` | `flank_cut` |
| support | - (pouch) | - (pouch) |

## 6. Dunstan "Slag-Jaw", the Slag-Brute (`founder_dunstan`, class `brawler`)

Gear: main `penal_shackles` (fist); body `penal_foundry_apron`; pouch `sulfur_poultice`. No discipline. Title
"Slag-Jaw". Joins at M3 as a guest, then as a member (bunk 5) at the cell's average level (1).

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status, flags and hooks | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `shackle_pummel_slam` | Shackle Slam | ATTACK | assault | 1 | 1 | 2 | 0.95 | 18 | 1-1 | `weapon_types` [fist, unarmed]; push 1 (x0.95, `collide_break`) | Snap: slam a foe into a wall or a unit to break it |
| `slag_slam_throw` | Slag Slam | ATTACK | assault | 1 | 2 | 3 | 1.2 | 12 | 1-1 | `weapon_types` [fist, unarmed]; single target; push 2 (x1.2, st 12) | Standard: throw a foe into its friends |
| `foundry_shatter` | Foundry Shatter | ATTACK | assault | 2 | 4 | 4 | 1.0 | 15 | 1-1 | area SWEEP; stun potency 60, 1 round | Heavy opener on a cluster |

- Shackle-Pummel Slam (HQ14 a), the variant of Jaw-Breaker: the wall-slam is the collision's second hit (x0.95,
  about double) and its break (`collide_break`); no `sure_break`, AP 2. Short form "Shackle Slam".
- Slag Slam and Throw: the throw is the push; the unit it lands against takes the same collision (HQ11 a). A push into
  M3's slag pools is a collision, not a fall (HQ9 a).
- Foundry Shatter: rubble from the blow waits (G16); the aimed 3-tile area is SWEEP (facing is battle facing only).

| Id | Name | `slot` | Effects | Lore effect still deferred |
| --- | --- | --- | --- | --- |
| `hypertrophy` | Hypertrophy | passive | stat CONS +3; `forced_move` `bonus_tiles` 1 | "+20% max HP exactly" (CONS +3 gives +24) |
| `anvil_posture` | Anvil Posture | passive | `stagger_taken` mult 0.0, max_blocks 1 | melee blunt damage -25% (G11) |
| `unbroken_iron` | Unbroken Iron | keystone | trigger `hp_below` (`pct` 30), `limit` battle: `ap` 2; grant `fortify` magnitude 8, 2 rounds | - |
| `foundry_wired_spite` | Foundry-Wired Spite | "" | `meter` `slag_fury`; trigger `hp_below` (`pct` 50), `limit` battle: grant `steadfast` rounds -1; `resist` `stun` 100; trigger `hit_taken`, `limit` none: `meter` `slag_fury` +1 | - |

Slag Fury: 0-5 pips, empower 5 per pip while held (HOOKS1_TASKS 2.2, row `slag_fury`). With Hypertrophy his pushes
go 1 tile further (Shackle Slam 2, Slag Slam 3).

Deck at join: active [`shackle_pummel_slam`, `slag_slam_throw`, `foundry_shatter` when level 2]; passive
[`hypertrophy`, `anvil_posture`]; keystone [`unbroken_iron`].

| Intent | Order |
| --- | --- |
| assault | `foundry_shatter`, `slag_slam_throw`, `shackle_pummel_slam`, `strike` |
| guard | `active_guard` |
| maneuver | `flank_cut` |
| support | - (pouch) |

## 7. Elyse, the Ichor Alchemist (`founder_elyse`, class `mage`)

Gear: main `brass_vitriol_sprayer` (sprayer, two-handed, range 3); body `tanners_rubbered_coat`; pouch
`pitch_fire_bomb`, `sulfur_poultice`. Discipline `still_craft` (passive `sprayer_drill`). Title "".
Joins at M4 as a guest, then as a member (bunk 6) at the cell's average level; not in the slice (M2b).

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status, flags and hooks | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `phosphorus_pot` | Phosphorus Pot | ATTACK | assault | 1 | 2 | 2 | 0.5 | 0 | 2-4 | area BURST; burn potency 70, magnitude 4, 2 rounds; `hazard` {`pitch_fire`, SKILL, target, 2} | Standard area opener; the fire stays |
| `magnesium_flash_bomb` | Flash-Bomb | ATTACK | assault | 1 | 1 | 3 | 0.2 | 10 | 2-3 | area BURST; `stagger` (the status) potency 60 | Snap; the Staggered status costs blocks next round |
| `choke_gas` | Choke-Gas | ATTACK | assault | 2 | 3 | 4 | 0.4 | 0 | 2-4 | area BURST; poison potency 80, magnitude 6, 3 rounds; `hazard` {`choke_gas`, SKILL, target, 2} | Heavy: a lasting gas zone |

- Phosphorus Pot: the 3x3 fire zone for 2 rounds is the `pitch_fire` hazard; Combustion Formula's terrain bonus
  waits (G8).
- Magnesium Flash-Bomb: Blind (a range cap, Q-56 a) is a new status (G17); the Staggered status (lost blocks next
  round) stands in. Short form "Flash-Bomb".
- Suffocating Choke-Gas: the zone is the `choke_gas` hazard on the 3x3 burst (the lore's 4x4 is no area shape); its
  AP loss waits (G17). Short form "Choke-Gas".

| Id | Name | `slot` | Effects | Lore effect still deferred |
| --- | --- | --- | --- | --- |
| `combustion_formula` | Combustion Formula | passive | resist `burn` 50 | +25% to units on tar, oil or wooden rubble (G8) |
| `catalytic_munitions` | Catalytic Munitions | passive | trigger `hit_dealt` (`other_status` burn), `limit` none: `inflict` expose potency 90, magnitude 4, 3 rounds, `to` other | blinded foes (G17); "for the battle" is 3 rounds |
| `master_alchemist` | Master Alchemist | keystone | `skill_potency` +0.2 on `phosphorus_pot`, `magnesium_flash_bomb`, `choke_gas` | +1 radius and 1 AP less on flasks (G12) |
| `volatile_reaction_catalyst` | Volatile Catalyst | "" | stat INT +2 | zones last a round longer; overlapping zones detonate (HOOKS1 14: out of scope) |

Deck at join: active [`phosphorus_pot`, `magnesium_flash_bomb`, `choke_gas` when level 2]; passive
[`combustion_formula`, `catalytic_munitions`]; keystone [`master_alchemist`].

| Intent | Order |
| --- | --- |
| assault | `choke_gas`, `phosphorus_pot`, `magnesium_flash_bomb`, `strike` |
| guard | `active_guard` |
| maneuver | - (pouch: `pitch_fire_bomb`, Utility) |
| support | - (pouch) |

## 8. Coverage check (level 1, the slice's four)

| Founder | Assault | Guard | Support | Maneuver |
| --- | --- | --- | --- | --- |
| Kaelen | Pommel Strike, Strike | Intercepting Hook, Active Guard | poultice | Flank Cut |
| Vesper | Strike | Ash-Cloud Fade, Active Guard | poultice | Hamstring Sever, Flank Cut, fire bomb |
| Maren | Strike | Active Guard | Tourniquet, Cauterize, Bile Coat (with a phial), poultice x2 | Flank Cut |
| Hollis | Barbed Quarrel, Tripwire Snare, Strike | Active Guard | poultice | Flank Cut |

Every intent has at least a basic or a pouch item for every founder; Kaelen and Vesper have a non-basic pick in two
intents at level 1, Maren (support) and Hollis (assault) in one (OI1: the feel run measures the share of rounds where
an intent had no active).
