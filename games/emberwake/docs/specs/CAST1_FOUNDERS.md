# CAST1 founders: the six Act 1 kits

Status: proposed (2026-10-25); part of [CAST1.md](CAST1.md) (conventions 3, open questions 9). Lore: the class files
`games/emberwake/docs/lore/systems/class_*.md` (Cold Steel branches and founders' Act 1 kits are canon; the Act 1
stand-ins below replace effects that need hooks no spec provides, CAST1_GAPS). All numbers FP.

## 1. Shared rules for the six kits

- Each founder is a world character (role party) on its own catalog unit `founder_<id>` (class = the base class, side
  party, tier hero, level 1, `stat_mode` derived, `spend` {}). The unit carries the gear, the pouch, `extra_skills`
  (the founder's CAST1 skills) and the `deck`; the character carries the trait and the passives (`passives`, source
  granted, RULESET1 D4), the drill discipline (`disciplines`), the title (class display name stand-in, G1) and
  `gone_flags` `[{"key": "sunk_ward.<id>_captured", "value": true}]` (read by the Crucible's keepsake hook).
- Skill rows: `learned_only` true, one rank (tier 1), `family` = the id; `level` is the unlock level (CQ3); every row
  sets `round_intent`. Party units book no recovery (RULESET1 T4), so founder-only rows leave `recovery_blocks` 0.
- The basics (`move`, `rest`, `strike`, `offhand_jab`, `active_guard`, `flank_cut`) are open to every founder. With
  no Support skill a founder still has Support through pouch items (RULESET1 IN5).
- The shared class path stays as it is (Ashlands families such as Taunt Shout are owned too); the deck's three active
  slots hold the CAST1 families. A world class overlay (G1) would drop the Ashlands path later.
- Order lists below are the starting `deck.order` (intent -> families, highest priority first). At level 1 the deck's
  `active` holds the level-1 skills; a locked family is not slottable until its level, and the player slots it then.

## 2. Kaelen, the Vanguard (`founder_kaelen`, class `warrior`)

Gear: main `notched_billhook` (polearm, two-handed, reach 1-2); body `castellan_breastplate`; pouch
`sulfur_poultice` x1. Discipline `castellan_drill` (passive `polearm_drill`). Title "Vanguard".

Skills (FP):

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status and flags | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `pommel_strike` | Pommel Strike | ATTACK | assault | 1 | 1 | 2 | 0.7 | 12 | 1-1 | - | Snap follow-up after any opener; the windup breaker |
| `intercepting_hook` | Intercepting Hook | ATTACK | maneuver | 1 | 2 | 3 | 0.8 | 10 | band | `weapon_types` [polearm]; `dash_tiles` 2; root potency 75, 1 round; exploit `flanked` x1.2 | Engage opener on a foe already on an ally |
| `billhook_sunder` | Billhook Sunder | ATTACK | assault | 2 | 3 | 4 | 1.8 | 18 | band | `weapon_types` [polearm]; expose potency 90, magnitude 8, 3 rounds | Heavy opener, then Pommel in b4-b6 |
| `seventh_ward_bulwark` | Ward Bulwark | DEFEND | guard | 3 | 3 | 3 | - | 0 | self | mitigation 0.5; grant `taunt` 2 rounds; `extra_grants` fortify magnitude 6, 2 rounds | Guard opener, then Active Guard |

- Billhook Sunder is the variant of Sunder and Pin (Heavy 3, one block faster): Kaelen owns it, not the base row. The
  lore's root (a second status) and "disarm on Clerisy or Inquisition targets" (a faction condition) are deferred
  (G9, G10). Its full name "Notched Billhook Sunder" goes in the description.
- Intercepting Hook stand-in: the intercept and the 1-tile pull wait for forced movement and reactions (G3, G5); the
  Engage carried move onto a foe engaged with an ally (a flank: stagger x1.5, no riposte) carries the idea.
- Seventh Ward Bulwark stand-in (`name` "Ward Bulwark", full name in the description): the 3-tile barrier, the line
  damage and "untargetable by ranged" wait for barriers and line of sight (G6); the taunt draws the attacks onto
  Kaelen meanwhile. Its tower shield against the two-handed billhook: CAST1_GAPS C4.

Passives, keystone and trait (FP):

| Id | Name | `slot` | Effects | Lore effect deferred |
| --- | --- | --- | --- | --- |
| `braced_poise` | Braced Poise | passive | stat `max_poise` +20 | immunity to forced movement while guarding (G3) |
| `kinetic_leverage` | Kinetic Leverage | passive | resist `stagger` 30, `stun` 30 | 1 AP back after a guarded hit (G5) |
| `unbroken_phalanx` | Unbroken Phalanx | keystone | `stagger_taken` mult 0.5, max_blocks 0 | crit immunity, unbreakable while guarding (G5, G11) |
| `castellans_penance` | Castellan's Penance | "" | stat CONS +2 | redirect 35% of a hit on an adjacent ally to himself (G5) |

Deck at level 1: active [`pommel_strike`, `intercepting_hook`]; passive [`braced_poise`, `kinetic_leverage`]; keystone
[`unbroken_phalanx`]. Full deck from level 3: active [`pommel_strike`, `billhook_sunder`, `seventh_ward_bulwark`] or
the player's pick.

| Intent | Order (level 1) | Order (full) |
| --- | --- | --- |
| assault | `pommel_strike`, `strike` | `billhook_sunder`, `pommel_strike`, `strike` |
| guard | `active_guard` | `seventh_ward_bulwark`, `active_guard` |
| maneuver | `intercepting_hook`, `flank_cut` | `flank_cut` (or `intercepting_hook` when slotted) |
| support | - (pouch) | - (pouch) |

## 3. Vesper, the Gutter-Stalker (`founder_vesper`, class `rogue`)

Gear: main `curfew_bodkin` (dagger); body `soot_leathers`; pouch `sulfur_poultice`, `pitch_fire_bomb`. No discipline
(daggers are a rogue proficiency). Title "Gutter-Stalker".

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status and flags | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `hamstring_sever` | Hamstring Sever | ATTACK | maneuver | 1 | 1 | 2 | 0.65 | 6 | 1-1 | `weapon_types` [dagger]; `dash_tiles` 3; slow potency 75, 2 rounds; exploit `flanked` x1.3 | Engage or Disengage opener; Snap repeats |
| `ash_cloud_fade` | Ash-Cloud Fade | DEFEND | guard | 1 | 1 | 2 | - | 0 | self | mitigation 0.2; grant `stealth` 2 rounds | Guard opener, then Active Guard |
| `throat_carver` | Throat-Carver | ATTACK | assault | 2 | 2 | 4 | 2.2 | 8 | 1-1 | `weapon_types` [dagger]; exploit `stagger` x1.5, `root` x1.5 | the finisher on a staggered or pinned foe |
| `orlovs_retribution` | Retribution | ATTACK | maneuver | 2 | 2 | 3 | 2.0 | 8 | 1-1 | `weapon_types` [dagger]; `dash_tiles` 2; exploit `stagger` x1.5, `root` x1.5 | Disengage: strike, then vanish 2 tiles |

- Ash-Cloud and Fade stand-in: the 3x3 smoke zone (a ground hazard a skill leaves, G4) waits; stealth (RULESET1 GD5)
  is the Act 1 smoke: a stealthed Vesper who moved cannot be hit by locked attacks. The 0b lesson "smoke, root,
  disengage" uses this, Kaelen's Intercepting Hook (root) and Hamstring Sever in Disengage.
- Throat-Carver's Art: the execute below 35% and the 2 AP refund wait (G7, G5); the exploit on Staggered and Rooted
  carries "strike the staggered and the pinned". Name short form "Throat-Carver".
- Orlov's Retribution (variant of Throat-Carver's Art, one AP cheaper): not owned at the start; it is the vendetta
  reward at M4 (CQ10; loot row `loot_orlovs_retribution`, CAST1_GEAR 8). Stand-in: the teleport on a kill (G2)
  becomes a Maneuver Disengage carried retreat on every use. Short form "Retribution".

| Id | Name | `slot` | Effects | Lore effect deferred |
| --- | --- | --- | --- | --- |
| `prowlers_cadence` | Prowler's Cadence | passive | resist `slow` 40, `root` 40 | no extra move cost in cover, mist and rubble (G8) |
| `anatomical_flaw` | Anatomical Flaw | passive | `skill_potency` family `throat_carver` +0.15, family `hamstring_sever` +0.15 | ignore armour and +25% crit on stunned, staggered or rooted targets (G11) |
| `ghost_of_the_undercroft` | Ghost of the Undercroft | keystone | `foresight` (no range); `ap_regen` +1 | Q-58 a: rebuilt on foresight and stealth |
| `cinder_childs_reflex` | Cinder-Child's Reflex | "" | stat DEX +2 | +1 tile on the next move after slipping an attack (G5) |

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
x2. Discipline `scriptorium_rites` (passive `censer_drill`). Title "Heretic Acolyte".

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Heal | Reach | Status and flags | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `brimstone_cauterize` | Cauterize | SUPPORT | support | 1 | 1 | 2 | base 10, KNOW x1.2 | 0-2 | ALLY_OR_SELF; cleanse `bleed`, `poison`, `burn` | Snap follow-up after Tourniquet |
| `tourniquet_tonic` | Tourniquet | SUPPORT | support | 1 | 2 | 3 | base 25, KNOW x2.0 | 0-2 | ALLY_OR_SELF; grant `haste` 1 round | the big heal opener |
| `resilience_ward` | Resilience Ward | SUPPORT | support | 2 | 3 | 4 | none | 0-3 | ALLY_OR_SELF; grant `steadfast` 1 round; `extra_grants` fortify magnitude 8, 1 round | before a heavy enemy round |

- Short forms: "Cauterize" (Brimstone Cauterize), "Tourniquet" (Tourniquet and Tonic). At KNOW 15: Cauterize heals
  28, Tourniquet 55.
- Tourniquet and Tonic stand-in: "an ally broken this round holds Active Guard" waits for a break trigger (G5).
- Resilience Ward stand-in: one ally, not the squad within 4 tiles (area support, G12); steadfast is "cannot be
  broken", fortify 8 stands in for +30% mitigation.
- Maren's Assault is the basic Strike with the censer; her Inoculated variant and signature are Act 2.

| Id | Name | `slot` | Effects | Lore effect deferred |
| --- | --- | --- | --- | --- |
| `anatomical_precision` | Anatomical Precision | passive | stat `max_poise` +8; resist `stun` 25 | an aura: allies within 2 take 15% less stagger (G12) |
| `triage_efficiency` | Triage Efficiency | passive | `ap_regen` +1 | 1 AP back when healing an ally below 30% (G5) |
| `pure_hippocratic_oath` | Hippocratic Oath | keystone | `skill_potency` group `heal` +0.4 | human enemies cannot crit her (G11) |
| `copyist_of_st_lyras` | Copyist of St. Lyra's | "" | `skill_potency` group `heal` +0.1 | the Bile Phial counter: a cleanse bottles a phial, a free Snap coats an ally's weapon, +8 damage (G14, Q-61 b) |

Deck at level 1: active [`brimstone_cauterize`, `tourniquet_tonic`]; passive [`anatomical_precision`,
`triage_efficiency`]; keystone [`pure_hippocratic_oath`].

| Intent | Order (level 1) | Order (full) |
| --- | --- | --- |
| support | `tourniquet_tonic`, `brimstone_cauterize` | `resilience_ward`, `tourniquet_tonic`, `brimstone_cauterize` |
| assault | `strike` | `strike` |
| guard | `active_guard` | `active_guard` |
| maneuver | `flank_cut` | `flank_cut` |

## 5. Hollis, the Arbalestier (`founder_hollis`, class `ranger`)

Gear: main `winch_arbalest` (crossbow, range 4); body `poachers_brigandine`; pouch `sulfur_poultice`. No discipline
(crossbow is a ranger proficiency). Title "Arbalestier".

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status and flags | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `poachers_quarrel` | Barbed Quarrel | ATTACK | assault | 1 | 2 | 2 | 0.95 | 8 | weapon band | `weapon_types` [crossbow]; bleed potency 70, 2 rounds | Standard; repeats |
| `tripwire_snare` | Tripwire Snare | ATTACK | assault | 1 | 1 | 3 | 0.35 | 4 | 1-3 | root potency 85, 1 round | Snap pin, then the quarrels |
| `heavy_winch_sunder` | Winch Sunder | ATTACK | assault | 2 | 4 | 5 | 2.1 | 12 | weapon band | `weapon_types` [crossbow]; `sure_break`; expose potency 90, magnitude 6, 2 rounds | Heavy opener into a telegraphed windup |

- Poacher's Barbed Quarrel is the variant of Notched Quarrel (Hollis owns it, not the base): range 5 in the lore comes
  from the arbalest's range 4 plus PERC; the tearing barbs on a moving target wait (G15: a bleed on moving); bleed
  stands in. Short form "Barbed Quarrel".
- Tripwire Snare stand-in: the hidden trap tile (G4) becomes a thrown wire snare (Snap, reach 1-3) that pins: the 0c
  lesson "ranged pins".
- Heavy Winch Sunder: "the target's windup breaks outright" is `sure_break` (RULESET1 P14), AP 4 + 1; the knockback
  waits (G3). Short form "Winch Sunder".

| Id | Name | `slot` | Effects | Lore effect deferred |
| --- | --- | --- | --- | --- |
| `high_ground_traverse` | High-Ground Traverse | passive | stat PERC +2 | +1 range from rubble or roof tiles (G8) |
| `cover_breaker` | Cover Breaker | passive | `status_potency` +15 | halve the target's cover; +25% crit and +15 stagger on a windup (G11) |
| `dead_eye_discipline` | Dead-Eye Discipline | keystone | `ap_regen` +1 | a ranged kill refunds 2 AP and reveals the next round (G5) |
| `crag_hawkeye` | Crag Hawkeye | "" | `foresight` range 4 | ignores partial cover (G8); Q-58 a: the reveal is his |

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
"Slag-Brute". Joins at M3 as a guest, then as a member (bunk 5); join level: CQ14.

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status and flags | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `shackle_pummel_slam` | Shackle Slam | ATTACK | assault | 1 | 1 | 3 | 0.95 | 18 | 1-1 | `weapon_types` [fist, unarmed]; `sure_break` | Snap breaker after any opener |
| `slag_slam_throw` | Slag Slam | ATTACK | assault | 1 | 2 | 3 | 1.2 | 12 | 1-1 | `weapon_types` [fist, unarmed]; area LINE | Standard; hits the foe and the two behind |
| `foundry_shatter` | Foundry Shatter | ATTACK | assault | 2 | 4 | 4 | 1.0 | 15 | 1-1 | area SWEEP; stun potency 60, 1 round | Heavy opener on a cluster |

- Shackle-Pummel Slam is the variant of Jaw-Breaker (Dunstan owns it): the wall-slam doubling and the knockback wait
  (G3); `sure_break` stands in for "the target's remaining blocks are cancelled as a break". Short form "Shackle Slam".
- Slag Slam and Throw stand-in: the throw (G3) becomes a LINE hit on the target and the two tiles behind it.
- Foundry Shatter: rubble from the blow waits (G16); the aimed 3-tile area is SWEEP (facing is battle facing only).

| Id | Name | `slot` | Effects | Lore effect deferred |
| --- | --- | --- | --- | --- |
| `hypertrophy` | Hypertrophy | passive | stat CONS +3 | +20% max HP exactly; knockbacks 1 tile further (G3) |
| `anvil_posture` | Anvil Posture | passive | `stagger_taken` mult 0.0, max_blocks 1 | melee blunt damage -25% (G11) |
| `unbroken_iron` | Unbroken Iron | keystone | `ap_regen` +1; stat `max_poise` +10 | below 30% HP: 2 AP back and fortify 8 (G5) |
| `foundry_wired_spite` | Foundry-Wired Spite | "" | resist `stun` 50; stat `max_poise` +10 | below 50% HP unbreakable; Slag Fury stacks (G5) |

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
`pitch_fire_bomb`, `sulfur_poultice`. Discipline `still_craft` (passive `sprayer_drill`). Title "Ichor Alchemist".
Joins at M4 as a guest, then as a member (bunk 6); not in the slice.

| Id | Name | Kind | Intent | Lvl | Blocks | AP | Power | Stagger | Reach | Status and flags | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `phosphorus_pot` | Phosphorus Pot | ATTACK | assault | 1 | 2 | 2 | 0.5 | 0 | 2-4 | area BURST; burn potency 70, magnitude 4, 2 rounds | Standard area opener |
| `magnesium_flash_bomb` | Flash-Bomb | ATTACK | assault | 1 | 1 | 3 | 0.2 | 10 | 2-3 | area BURST; `stagger` (the status) potency 60 | Snap; the Staggered status costs blocks next round |
| `choke_gas` | Choke-Gas | ATTACK | assault | 2 | 3 | 4 | 0.4 | 0 | 2-4 | area BURST; poison potency 80, magnitude 6, 3 rounds | Heavy zone stand-in |

- Phosphorus Pot stand-in: the 3x3 fire zone for 2 rounds (G4) becomes a burst with burn; Combustion Formula's
  terrain bonus waits (G8).
- Magnesium Flash-Bomb stand-in: Blind (a range cap, Q-56 a) is a new status (G17); the Staggered status (lost
  blocks next round) stands in. Short form "Flash-Bomb".
- Suffocating Choke-Gas stand-in: the 4x4 zone with AP loss (G4, G17) becomes a 3x3 poison burst. Short form
  "Choke-Gas".

| Id | Name | `slot` | Effects | Lore effect deferred |
| --- | --- | --- | --- | --- |
| `combustion_formula` | Combustion Formula | passive | resist `burn` 50 | +25% to units on tar, oil or wooden rubble (G8) |
| `catalytic_munitions` | Catalytic Munitions | passive | `status_potency` +15 | burning or blinded foes exposed 4 for the battle (G5) |
| `master_alchemist` | Master Alchemist | keystone | `skill_potency` +0.2 on `phosphorus_pot`, `magnesium_flash_bomb`, `choke_gas` | +1 radius and 1 AP less on flasks (G12) |
| `volatile_reaction_catalyst` | Volatile Catalyst | "" | stat INT +2 | zones last a round longer; two zones overlapping detonate (G4) |

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
| Kaelen | Pommel Strike, Strike | Active Guard | poultice | Intercepting Hook, Flank Cut |
| Vesper | Strike | Ash-Cloud Fade, Active Guard | poultice | Hamstring Sever, Flank Cut, fire bomb |
| Maren | Strike | Active Guard | Tourniquet, Cauterize, poultice x2 | Flank Cut |
| Hollis | Barbed Quarrel, Tripwire Snare, Strike | Active Guard | poultice | Flank Cut |

Every founder has a non-basic pick in at least two intents at level 1 (OI1: the feel run measures the share of rounds
where an intent had no active).
