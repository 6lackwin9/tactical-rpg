# INTENT2 skills: every skill and consumable under the proposed framework

Status: proposed, nothing decided. Generated 2026-11-05 by a read-only script from the `skills.json` and `items.json`
catalogs. The framework and the rules named in flags are in [INTENT2.md](../../../../docs/specs/INTENT2.md). Input to
SKILLPASS1, not a replacement for it. The proposed intent is the test applied to today's fields.

## Summary

Skills checked: 129 (the catalog holds 129). Consumables checked: 6 (every item whose family has `item_type` CONSUMABLE; 84 items in all).

| Intent | Today | Proposed |
| --- | --- | --- |
| assault | 88 | 62 |
| field | 0 | 23 |
| guard | 11 | 16 |
| support | 14 | 12 |
| maneuver | 11 | 11 |
| neutral | 5 | 5 |

Skills that change intent: 28: assault to field 23, assault to guard 3, support to guard 2.

Allowed intents (IR14-IR16): 12 skills list two: 8 SUPPORT with `target_mode` ALLY_OR_SELF (support, guard) and 4 thrown area skills (field, assault); the other 117 list one. Allowed lists the primary first; the row a skill is picked from sets its behaviour.

Flags: 31 `rule` (fits with a named rule), 9 `feel` (fits, but would feel wrong or needs tuning), 10 `unclear` (the test gives no one clear answer). A row with no flag fits cleanly.

Why words: `foe` aimed at an enemy unit; `tile` at the ground (Field); `self` lands only on you; `ring` around you, no direction; `arc` one side of you, with a direction; `friend` a chosen friend; `place` the point is where you end up; `none` a system command.

## Proposed assault (62)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `strike` | ATTACK | assault | - | assault | assault | foe |  |
| `offhand_jab` | ATTACK | assault | - | assault | assault | foe |  |
| `shield_bash` | ATTACK | assault | - | assault | assault | foe |  |
| `hamstring` | ATTACK | assault | - | assault | assault | foe |  |
| `sunder` | ATTACK | assault | - | assault | assault | foe |  |
| `reckless_swing` | ATTACK | assault | - | assault | assault | foe |  |
| `aimed_shot` | ATTACK | assault | - | assault | assault | foe |  |
| `pinning_shot` | ATTACK | assault | - | assault | assault | foe |  |
| `arcane_bolt` | ATTACK | assault | - | assault | assault | foe |  |
| `hex` | ATTACK | assault | - | assault | assault | foe |  |
| `haymaker` | ATTACK | assault | - | assault | assault | foe |  |
| `body_blow` | ATTACK | assault | - | assault | assault | foe |  |
| `quick_stab` | ATTACK | assault | - | assault | assault | foe |  |
| `lunge` | ATTACK | assault | - | assault | assault | foe | unclear: dash and hit; Assault by decision "Gap-close" |
| `bleeding_cut` | ATTACK | assault | - | assault | assault | foe |  |
| `barbed_arrow` | ATTACK | assault | - | assault | assault | foe |  |
| `pounce` | ATTACK | assault | - | assault | assault | foe | unclear: dash and hit; Assault by decision "Gap-close" |
| `smite` | ATTACK | assault | - | assault | assault | foe |  |
| `deadeye` | ATTACK | assault | - | assault | assault | foe |  |
| `judgement` | ATTACK | assault | - | assault | assault | foe |  |
| `charge_in` | ATTACK | assault | - | assault | assault | foe | unclear: dash and hit; Assault by decision "Gap-close" |
| `lacerate` | ATTACK | assault | - | assault | assault | foe |  |
| `assassinate` | ATTACK | assault | - | assault | assault | foe |  |
| `intimidate` | ATTACK | assault | - | assault | assault | foe | rule: a no-damage attack stays Assault |
| `crippling_shot` | ATTACK | assault | - | assault | assault | foe |  |
| `rend_armor` | ATTACK | assault | - | assault | assault | foe |  |
| `envenom` | ATTACK | assault | - | assault | assault | foe |  |
| `feint` | ATTACK | assault | - | assault | assault | foe |  |
| `crushing_blow` | ATTACK | assault | - | assault | assault | foe |  |
| `opportunist` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_snap_jab` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_thrust` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_heavy_cleave` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_shoulder_check` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_jaw_breaker` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_marrow_spike` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_hook_thrust` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_snap_shot` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_aimed_shot` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_overhead_smash` | ATTACK | assault | - | assault | assault | foe |  |
| `tl_crushing_judgment` | ATTACK | assault | - | assault | assault | foe |  |
| `pommel_strike` | ATTACK | assault | - | assault | assault | foe |  |
| `billhook_sunder` | ATTACK | assault | - | assault | assault | foe |  |
| `throat_carver` | ATTACK | assault | - | assault | assault | foe |  |
| `poachers_quarrel` | ATTACK | assault | - | assault | assault | foe |  |
| `heavy_winch_sunder` | ATTACK | assault | - | assault | assault | foe |  |
| `shackle_pummel_slam` | ATTACK | assault | - | assault | assault | foe |  |
| `slag_slam_throw` | ATTACK | assault | - | assault | assault | foe |  |
| `heavy_bill_thrust` | ATTACK | assault | - | assault | assault | foe |  |
| `billhook_hook` | ATTACK | assault | - | assault | assault | foe |  |
| `aimed_bolt` | ATTACK | assault | - | assault | assault | foe |  |
| `buckler_jab` | ATTACK | assault | - | assault | assault | foe |  |
| `iron_shackle_drag` | ATTACK | assault | - | assault | assault | foe |  |
| `scourge_lash` | ATTACK | assault | - | assault | assault | foe |  |
| `brand_of_the_heretic` | ATTACK | assault | - | assault | assault | foe | feel: no recovery, 6 a round in Assault |
| `caustic_vomit` | ATTACK | assault | - | assault | assault | foe |  |
| `bone_club_slam` | ATTACK | assault | - | assault | assault | foe |  |
| `sanctified_vivisection` | ATTACK | assault | - | assault | assault | foe |  |
| `coal_shovel_jab` | ATTACK | assault | - | assault | assault | foe |  |
| `flensing_hook_drag` | ATTACK | assault | - | assault | assault | foe |  |
| `sanctified_decapitation` | ATTACK | assault | - | assault | assault | foe |  |
| `prelates_censure` | ATTACK | assault | - | assault | assault | foe |  |

## Proposed field (23)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `shockwave` | ATTACK | assault | LINE | field | field | tile | rule: drawn from you, thrown off = miss |
| `volley` | ATTACK | assault | PLUS | field | field | tile | feel: ranger loses its Volley-then-shots mix |
| `firebomb` | ATTACK | assault | BURST | field | field, assault | tile | rule: both rows, Assault row smaller (IR17) |
| `arc_lance` | ATTACK | assault | LINE | field | field | tile | rule: drawn from you, thrown off = miss |
| `inferno` | ATTACK | assault | BURST | field | field, assault | tile | rule: both rows, Assault row smaller (IR17) |
| `glacial_burst` | ATTACK | assault | BURST | field | field, assault | tile | rule: both rows, Assault row smaller (IR17) |
| `tripwire_snare` | ATTACK | assault | one tile | field | field | tile | feel: then Rest unless the pouch fills; wasted on a unit target today |
| `foundry_shatter` | ATTACK | assault | SWEEP to ARC | field | field | arc | rule: arc; never fires today |
| `phosphorus_pot` | ATTACK | assault | BURST | field | field | tile |  |
| `magnesium_flash_bomb` | ATTACK | assault | BURST | field | field, assault | tile | rule: both rows, Assault row smaller (IR17) |
| `choke_gas` | ATTACK | assault | BURST | field | field | tile |  |
| `crushing_judgment` | ATTACK | assault | SWEEP to ARC | field | field | arc | rule: arc; never fires today |
| `pyre_nozzle_sweep` | ATTACK | assault | BURST | field | field | tile | feel: Purifier loses its Nozzle-then-Brand mix |
| `scalding_grease_pot` | ATTACK | assault | PLUS | field | field | tile | feel: Dredger has no Assault skill left |
| `gilded_billhook_sunder` | ATTACK | assault | SWEEP to ARC | field | field | arc | rule: arc; never fires today; Assault list left empty |
| `execution_wall` | ATTACK | assault | LINE | field | field | tile | rule: drawn from you, thrown off = miss |
| `transfusion_pulse` | ATTACK | assault | BURST | field | field | tile | unclear: reach 1, reads as around you |
| `pneumatic_hammer_strike` | ATTACK | assault | SWEEP to ARC | field | field | arc | rule: arc; never fires today |
| `bone_mill_pulverizer` | ATTACK | assault | BURST | field | field | tile | unclear: reach 1, reads as around you |
| `pressurized_lard_spray` | ATTACK | assault | BURST | field | field | tile | feel: Orlov alternates with Hook Drag; Field only, a boss telegraphs |
| `scalding_cataclysm` | ATTACK | assault | BURST | field | field | tile | unclear: reach 1, reads as around you |
| `ossified_spine_volley` | ATTACK | assault | BURST | field | field | tile | unclear: reach 1, reads as around you |
| `ossification_bloom` | ATTACK | assault | BURST | field | field | tile | unclear: reach 0, may centre on itself |

## Proposed guard (16)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `guard` | DEFEND | guard | - | guard | guard | self |  |
| `focus` | FOCUS | support | - | guard | guard | self | feel: offence buff under Guard; must outlast its round |
| `taunt_shout` | SUPPORT | support | - | guard | guard | self | feel: was Support; fits Guard |
| `riposte` | DEFEND | guard | - | guard | guard | self |  |
| `sweep_kick` | ATTACK | assault | SWEEP to RING | guard | guard | ring | rule: ring, foes only; never fires today |
| `cyclone_kick` | ATTACK | assault | SWEEP to RING | guard | guard | ring | rule: ring, foes only; never fires today |
| `shield_wall` | DEFEND | guard | - | guard | guard | self |  |
| `active_guard` | DEFEND | guard | - | guard | guard | self |  |
| `tl_brace_counter` | DEFEND | guard | - | guard | guard | self |  |
| `tl_fade` | DEFEND | guard | - | guard | guard | self | rule: bound buff ends on leaving Guard |
| `intercepting_hook` | DEFEND | guard | - | guard | guard | self | unclear: guards any friend within 1, aimed at none |
| `seventh_ward_bulwark` | DEFEND | guard | - | guard | guard | self |  |
| `ash_cloud_fade` | DEFEND | guard | - | guard | guard | self | rule: hazard stays when you walk |
| `traitors_parry` | DEFEND | guard | - | guard | guard | self |  |
| `inquisitorial_parry` | DEFEND | guard | - | guard | guard | self |  |
| `tendril_flail` | ATTACK | assault | SWEEP to RING | guard | guard | ring | feel: reckless unit needs the Guard exemption; never fires today |

## Proposed support (12)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `mend` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `rally` | SUPPORT | support | - | support | support | friend |  |
| `purify` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `greater_mend` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `renewal` | SUPPORT | support | - | support | support | friend |  |
| `aegis` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `rejuvenate` | SUPPORT | support | - | support | support | friend |  |
| `tl_field_dressing` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `brimstone_cauterize` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `tourniquet_tonic` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |
| `bile_coat` | SUPPORT | support | - | support | support | friend |  |
| `resilience_ward` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Guard round, on an ally Support |

## Proposed maneuver (11)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `backflip` | DISENGAGE | maneuver | - | maneuver | maneuver | place | rule: aims at a foe or a tile |
| `leap_back` | DISENGAGE | maneuver | - | maneuver | maneuver | place | rule: aims at a foe or a tile |
| `flank_cut` | ATTACK | maneuver | at flank | maneuver | maneuver | place |  |
| `tl_hamstring_cut` | ATTACK | maneuver | at behind | maneuver | maneuver | place |  |
| `tl_slip_strike` | ATTACK | maneuver | at flank | maneuver | maneuver | place |  |
| `hamstring_sever` | ATTACK | maneuver | - | maneuver | maneuver | place | rule: no `position` yet, the transition reads flank |
| `orlovs_retribution` | ATTACK | maneuver | - | maneuver | maneuver | place | unclear: the dash after the strike is a retreat |
| `martyrs_rush` | ATTACK | maneuver | - | maneuver | maneuver | place | rule: no `position` yet, the transition reads flank |
| `boning_knife_flurry` | ATTACK | maneuver | - | maneuver | maneuver | place | rule: no `position` yet, the transition reads flank |
| `throat_clamp_pounce` | ATTACK | maneuver | - | maneuver | maneuver | place | rule: no `position` yet, the transition reads flank |
| `burrow_ambush` | ATTACK | maneuver | - | maneuver | maneuver | place | rule: no `position` yet, the transition reads flank |

## Proposed neutral (5)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `move` | MOVE | neutral | - | neutral | neutral | none |  |
| `switch_target` | TARGET | neutral | - | neutral | neutral | none |  |
| `staggered` | STATUS | neutral | - | neutral | neutral | none |  |
| `stunned` | STATUS | neutral | - | neutral | neutral | none |  |
| `rest` | REST | neutral | - | neutral | neutral | none |  |

## Consumables (6)

Blocks, AP and stack are today's; the proposed values are in INTENT2 section 6.

| Id | Type | Effect | Reach | Blocks | AP | `max_stack` | Now | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `con_heal_potion` | HEALTH_POTION | heal 90 | 1 | 1 | 1 | 5 | support | support, guard | friend | feel: strong, longest block cost |
| `con_focus_elixir` | FOCUS_INCENSE | STR +50 % for 2 rounds | 0 | 1 | 1 | 5 | support | guard | self | feel: offence buff under Guard, as Focus |
| `con_fire_bomb` | THROWABLE_BOMB | damage 45 | 3 | 1 | 1 | 5 | maneuver | field, assault | foe or tile | rule: tile row is new; override: one foe in Assault |
| `sulfur_poultice` | HEALTH_POTION | heal 60 | 1 | 1 | 1 | 5 | support | support, guard | friend |  |
| `pitch_fire_bomb` | THROWABLE_BOMB | damage 40 | 3 | 1 | 1 | 5 | maneuver | field, assault | foe or tile | rule: tile row is new; override: one foe in Assault |
| `brimstone_draught` | FOCUS_INCENSE | CONS +30 % for 3 rounds | 0 | 1 | 1 | 5 | support | guard | self | rule: code gives attack, not CONS |
