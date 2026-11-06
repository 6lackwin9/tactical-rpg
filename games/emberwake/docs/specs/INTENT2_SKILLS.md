# INTENT2 skills: every skill and consumable under the proposed framework

Status: revised to `IR21` (`2026-11-07`, `FB-17`, `FB-19`). The id `guard` is the intent shown as Stand. Derived Field bonuses (`IR21`): 11 `reach`, 5 `lingering`, 7 `area`, 0 `power` (`power` is reserved as the future default). Generated from `skills.json` and `items.json` catalogs. The framework and the rules named in flags are in [INTENT2.md](../../../../docs/specs/INTENT2.md). Input to SKILLPASS1, not a replacement for it.

## Summary

Skills checked: 129 (the catalog holds 129). Consumables checked: 6 (every item whose family has `item_type` CONSUMABLE; 84 items in all).

| Intent | Today | Derived (`IR21`) |
| --- | --- | --- |
| assault | 88 | 59 |
| field | 0 | 23 |
| guard | 11 | 16 |
| support | 14 | 12 |
| maneuver | 11 | 14 |
| neutral | 5 | 5 |

Skills that change primary intent: 31 (`lunge`, `pounce`, and `charge_in` move from `assault` to `maneuver` via `dash_tiles` in `IR21`, `FB-17`).

Allowed intents (`IR21` derived): 19 skills list two: 8 `SUPPORT` with `target_mode` `ALLY_OR_SELF` (`support, guard`) and 11 thrown area skills (`field, assault`, including `volley` and 6 enemy bursts); the other 110 list one. Allowed lists the primary first; the row a skill is picked from sets its behaviour.

Flags: 31 `rule` (fits with a named rule), 9 `feel` (fits, but would feel wrong or needs tuning), 10 `unclear` (the test gives no one clear answer). A row with no flag fits cleanly.

Why words: `foe` aimed at an enemy unit; `tile` at the ground (Field); `self` lands only on you; `ring` around you, no direction; `arc` away from you, across the source; `friend` a chosen friend; `place` the point is where you end up; `none` a system command.

## Proposed assault (59)

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
| `bleeding_cut` | ATTACK | assault | - | assault | assault | foe |  |
| `barbed_arrow` | ATTACK | assault | - | assault | assault | foe |  |
| `smite` | ATTACK | assault | - | assault | assault | foe |  |
| `deadeye` | ATTACK | assault | - | assault | assault | foe |  |
| `judgement` | ATTACK | assault | - | assault | assault | foe |  |
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

| Id | Kind | Now | Shape | Primary | Allowed | Bonus | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `shockwave` | ATTACK | assault | LINE | field | field | area | tile | rule: drawn; off the planned tile = miss |
| `volley` | ATTACK | assault | PLUS | field | field, assault | reach | tile | feel: ranger keeps Volley-then-shots mix via Assault half-splash row |
| `firebomb` | ATTACK | assault | BURST | field | field, assault | reach | tile | rule: both rows: Assault half splash, Field its bonus (IR17) |
| `arc_lance` | ATTACK | assault | LINE | field | field | area | tile | rule: drawn; off the planned tile = miss |
| `inferno` | ATTACK | assault | BURST | field | field, assault | reach | tile | rule: both rows: Assault half splash, Field its bonus (IR17) |
| `glacial_burst` | ATTACK | assault | BURST | field | field, assault | reach | tile | rule: both rows: Assault half splash, Field its bonus (IR17) |
| `tripwire_snare` | ATTACK | assault | one tile | field | field | lingering | tile | feel: then Rest unless the pouch fills; wasted on a unit target today |
| `foundry_shatter` | ATTACK | assault | SWEEP to ARC | field | field | area | arc | rule: ARC; never fires today |
| `phosphorus_pot` | ATTACK | assault | BURST | field | field | lingering | tile |  |
| `magnesium_flash_bomb` | ATTACK | assault | BURST | field | field, assault | reach | tile | rule: both rows: Assault half splash, Field its bonus (IR17) |
| `choke_gas` | ATTACK | assault | BURST | field | field | lingering | tile |  |
| `crushing_judgment` | ATTACK | assault | SWEEP to ARC | field | field | area | arc | rule: ARC; never fires today |
| `pyre_nozzle_sweep` | ATTACK | assault | BURST | field | field | lingering | tile | feel: Purifier loses its Nozzle-then-Brand mix |
| `scalding_grease_pot` | ATTACK | assault | PLUS | field | field | lingering | tile | feel: Dredger has no Assault skill left |
| `gilded_billhook_sunder` | ATTACK | assault | SWEEP to ARC | field | field | area | arc | rule: ARC; never fires today; Assault list left empty |
| `execution_wall` | ATTACK | assault | LINE | field | field | area | tile | rule: drawn; off the planned tile = miss |
| `transfusion_pulse` | ATTACK | assault | BURST | field | field, assault | reach | tile | unclear: reach 1, reads as around you (`IR21` dual row) |
| `pneumatic_hammer_strike` | ATTACK | assault | SWEEP to ARC | field | field | area | arc | rule: ARC; never fires today |
| `bone_mill_pulverizer` | ATTACK | assault | BURST | field | field, assault | reach | tile | unclear: reach 1, reads as around you (`IR21` dual row) |
| `pressurized_lard_spray` | ATTACK | assault | BURST | field | field, assault | reach | tile | feel: `IR21` dual row; alternates with Hook Drag |
| `scalding_cataclysm` | ATTACK | assault | BURST | field | field, assault | reach | tile | unclear: reach 1, reads as around you (`IR21` dual row) |
| `ossified_spine_volley` | ATTACK | assault | BURST | field | field, assault | reach | tile | unclear: reach 1, reads as around you (`IR21` dual row) |
| `ossification_bloom` | ATTACK | assault | BURST | field | field, assault | reach | tile | unclear: reach 0, may centre on itself (`IR21` dual row) |

## Proposed stand (id guard) (16)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `guard` | DEFEND | guard | - | guard | guard | self |  |
| `focus` | FOCUS | support | - | guard | guard | self | feel: offence buff under Stand; must outlast its round |
| `taunt_shout` | SUPPORT | support | - | guard | guard | self | feel: was Support; fits Stand |
| `riposte` | DEFEND | guard | - | guard | guard | self |  |
| `sweep_kick` | ATTACK | assault | SWEEP to RING | guard | guard | ring | rule: RING, foes only; never fires today |
| `cyclone_kick` | ATTACK | assault | SWEEP to RING | guard | guard | ring | rule: RING, foes only; never fires today |
| `shield_wall` | DEFEND | guard | - | guard | guard | self |  |
| `active_guard` | DEFEND | guard | - | guard | guard | self |  |
| `tl_brace_counter` | DEFEND | guard | - | guard | guard | self |  |
| `tl_fade` | DEFEND | guard | - | guard | guard | self | rule: bound buff ends on leaving Stand |
| `intercepting_hook` | DEFEND | guard | - | guard | guard | self | unclear: guards any friend within 1, aimed at none |
| `seventh_ward_bulwark` | DEFEND | guard | - | guard | guard | self |  |
| `ash_cloud_fade` | DEFEND | guard | - | guard | guard | self | rule: hazard stays when you walk |
| `traitors_parry` | DEFEND | guard | - | guard | guard | self |  |
| `inquisitorial_parry` | DEFEND | guard | - | guard | guard | self |  |
| `tendril_flail` | ATTACK | assault | SWEEP to RING | guard | guard | ring | feel: reckless unit needs the Stand exemption; never fires today |

## Proposed support (12)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `mend` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `rally` | SUPPORT | support | - | support | support | friend |  |
| `purify` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `greater_mend` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `renewal` | SUPPORT | support | - | support | support | friend |  |
| `aegis` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `rejuvenate` | SUPPORT | support | - | support | support | friend |  |
| `tl_field_dressing` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `brimstone_cauterize` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `tourniquet_tonic` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |
| `bile_coat` | SUPPORT | support | - | support | support | friend |  |
| `resilience_ward` | SUPPORT | support | - | support | support, guard | friend | rule: on yourself a Stand round, on an ally Support |

## Proposed maneuver (14)

| Id | Kind | Now | Shape | Primary | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `backflip` | DISENGAGE | maneuver | - | maneuver | maneuver | place | rule: aims at a foe or a tile |
| `leap_back` | DISENGAGE | maneuver | - | maneuver | maneuver | place | rule: aims at a foe or a tile |
| `lunge` | ATTACK | assault | - | maneuver | maneuver | place | rule: `dash_tiles > 0` derives `maneuver` (`IR21`, `FB-17`) |
| `pounce` | ATTACK | assault | - | maneuver | maneuver | place | rule: `dash_tiles > 0` derives `maneuver` (`IR21`, `FB-17`) |
| `charge_in` | ATTACK | assault | - | maneuver | maneuver | place | rule: `dash_tiles > 0` derives `maneuver` (`IR21`, `FB-17`) |
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

Blocks, AP and stack are today's; the proposed values, shape and bonus are in INTENT2_CASES section 6. No consumable lists Maneuver: none is about mobility.

| Id | Type | Effect | Reach | Blocks | AP | `max_stack` | Now | Allowed | Why | Flag |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `con_heal_potion` | HEALTH_POTION | heal 90 | 1 | 1 | 1 | 5 | support | support, guard | friend | feel: strong, longest block cost |
| `con_focus_elixir` | FOCUS_INCENSE | STR +50 % for 2 rounds | 0 | 1 | 1 | 5 | support | guard | self | feel: offence buff under Stand, as Focus |
| `con_fire_bomb` | THROWABLE_BOMB | damage 45 | 3 | 1 | 1 | 5 | maneuver | field, assault | foe or tile | rule: tile row is new; Assault half splash (IR17) |
| `sulfur_poultice` | HEALTH_POTION | heal 60 | 1 | 1 | 1 | 5 | support | support, guard | friend |  |
| `pitch_fire_bomb` | THROWABLE_BOMB | damage 40 | 3 | 1 | 1 | 5 | maneuver | field, assault | foe or tile | rule: tile row is new; Assault half splash (IR17) |
| `brimstone_draught` | FOCUS_INCENSE | CONS +30 % for 3 rounds | 0 | 1 | 1 | 5 | support | guard | self | rule: code gives attack, not CONS |
