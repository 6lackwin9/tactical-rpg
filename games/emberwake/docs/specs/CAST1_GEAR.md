# CAST1 gear: weapon families, items, drills, relics, keepsakes and loot rows

Status: proposed (2026-10-25; rewritten to the HOOKS1 hooks and ROSTER1's keepsakes by CAST1-R the same day); part
of [CAST1.md](CAST1.md). Lore: `systems/items_gear_and_progression.md` (canon for slots, pouch and resources; rows
are proposals). Q-71 (b): the missing families are data. All numbers FP.

## 1. Weapon families (`item_families.json`, CAST1-A1)

All five are Act 1: the polearm (Kaelen, Vane), the scourge (Mortuary Flagellants), the censer (Maren), the bone-saw
(Quell) and the chemical sprayer (Elyse, the Purifiers). `polearm` is RULESET1-A's row (TQ8); CAST1 uses it unchanged.
The other four need CAST1-S first (weapon type options). Shared values: `item_type` WEAPON, `adjustable` [power,
block_cost, burden] (sprayer adds range), `modifier_kinds` [adjust, stat_bonus, requirement, effect], `max_modifiers`
8, `max_tiers` 3, `off_hand_category` "", `art` = the id, `bonus_value` band 0-4, `mitigation` 0 (band 0-0),
`block_cost` 1 (band 1-3).

| Id | Name | Hands (default) | Scaling | Damage | Base power | Range (band) | Burden (band) | Power band | `engine_rules` | Reach |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `polearm` | Polearm | main_2h | STR | pierce | 26 | 1 (1-1) | 2 (0-4) | 20-32 | [reach] | 1-2 (RULESET1 K1) |
| `scourge` | Scourge | main_1h | DEX, STR | slash | 12 | 1 (1-1) | 0 (0-2) | 8-18 | [reach] | 1-2 |
| `censer` | Censer | main_1h | INT, STR | blunt | 11 | 1 (1-1) | 1 (0-3) | 8-18 | [] | 1 |
| `bone_saw` | Bone-Saw | main_1h | DEX, STR | slash | 14 | 1 (1-1) | 0 (0-2) | 10-20 | [] | 1 |
| `sprayer` | Chemical Sprayer | main_2h | INT | fire | 18 | 3 (2-4) | 1 (0-3) | 14-24 | [always_ranged] | ranged band 2-3 |

Notes: the censer stays reach 1 (only polearm and scourge carry the reach rule, Q-70, TQ8). The sprayer's fire type
is cosmetic in Act 1 (no damage-type resist exists, G11). Billhook = polearm (one family, Q-71's "billhook/polearm").

## 2. Items (`items.json`, CAST1-A1)

Weapons and armour (COMMON unless noted; `starter_stash` false; `scaling_stat` = the family's first; `requirement`
modifiers as listed; `slot` BODY for armour):

| Id | Name | Family, hand | Weight | Value | Modifiers | Effects | Used by |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `notched_billhook` | Notched Billhook | polearm main_2h | 4.0 | 40 | req STR 12 | - | Kaelen |
| `castellan_breastplate` | Dented Cuirass | plate | 8.0 | 60 | req CONS 12 | - | Kaelen |
| `curfew_bodkin` | Curfew Bodkin | dagger main_1h | 0.8 | 35 | power +1; req DEX 11 | - | Vesper |
| `soot_leathers` | Soot Leathers | leather | 2.5 | 30 | req DEX 10 | - | Vesper |
| `bronze_censer` | Bronze Censer | censer main_1h | 2.5 | 40 | req INT 10; stat KNOW +1 | - | Maren |
| `copied_psalter` | Copied Psalter | tome off | 1.0 | 35 | req KNOW 11; stat KNOW +1 | `status_potency` 5 | Maren |
| `embalmers_habit` | Embalmer's Habit | cloth | 1.2 | 40 | - | `skill_potency` group heal 0.1; resist bleed 15 | Maren |
| `winch_arbalest` | Winch Arbalest | crossbow main_2h | 4.5 | 55 | range +1; req DEX 12 | - | Hollis |
| `poachers_brigandine` | Poacher's Coat | leather | 3.0 | 40 | mitigation +1; req DEX 10 | - | Hollis |
| `penal_shackles` | Penal Shackles | fist main_1h | 1.5 | 20 | power +2; req STR 10 | - | Dunstan |
| `penal_foundry_apron` | Foundry Apron | leather | 4.0 | 35 | mitigation +2; burden +1 | resist burn 25 | Dunstan |
| `brass_vitriol_sprayer` | Brass Sprayer | sprayer main_2h | 3.5 | 50 | req INT 12 | - | Elyse |
| `tanners_rubbered_coat` | Rubbered Coat | cloth | 1.5 | 30 | - | resist poison 25 | Elyse |
| `watch_hookbill` | Watch Hookbill | axe main_1h | 2.0 | 30 | req STR 10 | - | Footmen |
| `watch_heater` | Heater Shield | shield off | 3.5 | 35 | mitigation -1; req STR 11 | - | Footmen |
| `watch_cuirass` | Watch Cuirass | plate | 8.0 | 50 | req CONS 12 | - | Footmen, Enforcer |
| `watch_gambeson` | Watch Gambeson | leather | 3.0 | 25 | - | - | Crossbowmen |
| `stirrup_arbalest` | Stirrup Arbalest | crossbow main_2h | 4.0 | 40 | req DEX 11 | - | Crossbowmen |
| `bailiff_maul` | Bailiff's Maul | hammer main_2h | 6.5 | 60 | req STR 13 | - | Bailiff Enforcer |
| `penitent_scourge` | Penitent Scourge | scourge main_1h | 1.5 | 20 | - | - | Flagellants |
| `pyre_nozzle` | Pyre Nozzle | sprayer main_2h | 4.0 | 45 | - | - | Purifiers |
| `warded_apron` | Warded Apron | leather | 4.0 | 40 | mitigation +2 | resist burn 25 | Purifiers |
| `boning_knife` | Boning Knife | dagger main_1h | 0.8 | 20 | - | - | Dredgers |
| `skinners_apron` | Skinner's Apron | leather | 2.5 | 20 | - | - | Dredgers |
| `bone_club` | Bone Club | mace main_1h | 3.0 | 0 | - | - | Calcified Husks |
| `vanes_bill` | Executioner Bill | polearm main_2h | 5.0 | 90 | power +3; req STR 14 | - | Captain Vane (UNCOMMON) |
| `quells_bone_saws` | Twin Bone-Saws | bone_saw main_1h | 1.5 | 70 | power +2 | - | Proctor Quell (UNCOMMON) |
| `foundry_sledge` | Foundry Sledge | hammer main_2h | 7.0 | 70 | req STR 15 | - | Overseer Grendel |
| `flensing_hooks` | Flensing Hooks | axe main_1h | 3.0 | 60 | power +2 | - | Guild-Master Orlov |
| `prelates_blade` | Prelate's Blade | sword main_2h | 4.5 | 120 | power +4; req STR 14 | - | Nemeth (RARE) |
| `slag_plating` | Slag Plating | plate | 9.0 | 90 | mitigation +2; req CONS 13 | resist burn 15 | M3 reward (UNCOMMON) |

Consumables (`consumable` family; `max_stack` 5):

| Id | Name | Type | Numbers | Use | Value |
| --- | --- | --- | --- | --- | --- |
| `sulfur_poultice` | Stitch-Poultice | HEALTH_POTION | heal 60 | blocks 1, reach 1 | 15 |
| `pitch_fire_bomb` | Pitch Fire-Bomb | THROWABLE_BOMB | damage 40, damage type fire | blocks 1, reach 3 | 30 |
| `brimstone_draught` | Brimstone Draught | FOCUS_INCENSE | buff CONS 0.3 for 3 rounds | blocks 1, reach 0 | 25 |

- Sulfur Stitch-Poultice: the lore's cleanse of bleed and poison waits (consumables heal or buff, G17); full name in
  the description. Tallow Burn Salve and the Magnesium Flash-Pot are not made (no item cleanse or item stagger).
- Brimstone Tallow Draught stand-in: "halves poise damage for 3 rounds" becomes CONS +30% (Max Poise reads CONS).

Materials (`material` family) and quest items (`quest` family), value 0 unless noted:

| Id | Name | Family | Drops from | Act 1 use |
| --- | --- | --- | --- | --- |
| `calcified_bone_shard` | Bone Shard | material | Husks (loot 25), M3 | Hearth recipes (later) |
| `render_beast_tendon` | Beast Tendon | material | Render-Beasts (loot 20) | Hearth recipes (later) |
| `caustic_gall_sac` | Gall-Sac | material | Dredgers, Ash-Crawlers (loot 15) | Act 2 |
| `stalker_optic_nerve` | Optic Nerve | material | Render-Beasts (loot 10) | Act 2 |
| `inquisitor_marrow_valve` | Marrow Valve | material | Quell (key loot) | Act 2 graft |
| `titan_marrow_core` | Marrow Core | material | the Amalgam (key loot) | Act 2 Apex graft |
| `slab_retorts` | Slab Retorts | quest | M2 objective reward | opens the Slab's recipes (dialogue condition) |
| `city_gate_pass` | Gate Pass Cipher | quest | M1 objective reward | a route or Corvo dialogue condition |

## 3. Drill passives and disciplines (CAST1-A2, CAST1-A5)

Proficiency for a family a class lacks comes from a source-granted passive (`slot` ""), the `proficiency` effect
(today's grammar; unlisted families use the passive-only scaling 1.25). CQ1 (a); the `sunk_ward` class overlay
renames the classes and empties their paths but adds no proficiency (HOOKS1 12, HQ6 a), so the drills stay.

| Passive id | Name | Effects | Granted by |
| --- | --- | --- | --- |
| `polearm_drill` | Polearm Drill | proficiency polearm +2 | discipline `castellan_drill` (Kaelen); Vane's character |
| `censer_drill` | Censer Rites | proficiency censer +2 | discipline `scriptorium_rites` (Maren) |
| `sprayer_drill` | Still Craft | proficiency sprayer +2 | discipline `still_craft` (Elyse); Purifier characters |
| `scourge_drill` | Penitent Rite | proficiency scourge +2 | Flagellant characters |
| `bone_saw_drill` | Vivisector's Hand | proficiency bone_saw +2 | Quell's character |

Disciplines (`sunk_ward/disciplines.json`): `castellan_drill` "Castellan Drill" [polearm_drill]; `scriptorium_rites`
"Scriptorium Rites" [censer_drill]; `still_craft` "Still Craft" [sprayer_drill]; teacher "", faction "", one-line
descriptions from the lore dossiers.

## 4. Statuses CAST1 uses (all exist or come with RULESET1)

| Status | Row | Used by |
| --- | --- | --- |
| `stagger` (Staggered: lost blocks) | exists | Flash-Bomb, Burrow Ambush, Pneumatic Hammer; Throat-Carver's exploit |
| `stun`, `root`, `slow`, `bleed`, `poison`, `burn`, `expose`, `weaken` | exist | skills (founders 2-7, bestiary, bosses) |
| `taunt`, `haste`, `fortify`, `regen`, `riposte`, `empower` | exist | Bulwark, Tourniquet, Ward, parries, phases |
| `steadfast`, `stealth` | RULESET1-S | Resilience Ward, Ash-Cloud Fade, Nemeth's phase |

The lore statuses with no row (Blind, Flammable, Arterial Rupture, mark, frenzy, AP loss) are G17; CAST1 maps none of
them onto a wrong row.

## 5. Relics (one copy per campaign, CAST1-A1 and A5)

Loot rows reference catalog items, not world named items (G26), so the relics are catalog items (`special` true with
`special_note`, rarity RARE, `starter_stash` false) paid once through `key_loot` or `objective_reward` `once`. Their
effects use item modifiers (a `stat_bonus` may name the derived `max_poise`, DERIVE1-S; a bonus on a derived stat is bounded 1-50 by the kit constant `DERIVED_BONUS_MAX`, CAST1-S2; base stats 1-8; tier bonuses on derived stats stay 1-5) and item effects, the HOOKS1
item `trigger` included (the pavise: HOOKS1_TASKS 1.2, 6).

| Id | Name | Family, hand | Source | Modifiers and effects | Lore effect deferred |
| --- | --- | --- | --- | --- | --- |
| `vanes_gilded_pavise` | Gilded Pavise | shield off | M1 objective reward | mitigation +4; stat `max_poise` +30; req STR 13; effect trigger `guarded_hit`: `damage` 15, `stagger` 20, `to` other | - |
| `quells_marrow_scalpel` | Marrow Scalpel | bone_saw main_1h | M2 key loot (Quell) | power +3; `status_potency` 10 | the hit vents Strain (Act 2), bleed on hit (G9) |
| `grendels_slag_hammer` | Slag-Hammer | hammer main_2h | M3 key loot (Grendel) | power +4; req STR 15 | the shockwave hits the unit behind (G12) |
| `orlovs_flensing_talon` | Flensing Talon | dagger main_1h | M4 key loot (Orlov) | power +3; `skill_potency` family `throat_carver` 0.2, family `orlovs_retribution` 0.2 | execute a bleeding foe below 35% (G7) |
| `dreamers_vertebra` | Dreamer's Vertebra | accessory | interlude key loot | stat KNOW +1 | Strain floor 20, Inoculated +30% (Act 2) |

The pavise is an off-hand: Kaelen can use it only with a one-handed weapon (a build choice; CAST1_GAPS C4).
Description gives the full lore name ("Vane's Gilded Castellan Pavise" and so on).

## 6. Starting stash for the slice (FP; the balance lane sets stocks, Q-41)

Founders arrive with the gear and pouches in CAST1_FOUNDERS. The hub stash at the hearth scene: 2 `sulfur_poultice`,
1 `pitch_fire_bomb`, 1 `brimstone_draught`. Scrap and Warded Tallow: the M0 purse task.

## 7. Keepsakes (the Crucible's captured founder; Q-34, Q-35)

Each is a catalog accessory (family `accessory`, slot ACCESSORY_1, rarity RARE, `special` true, `value` 0: never
sold, as ROSTER1 KS2's check asks) carrying the founder's name, about 50% of the founder's Act 1 trait and far above
shop accessories (ROSTER1 KS4; numbers by the balance lane). Kaelen's and Hollis's names are in the story guide; the
other four are proposals (CQ11 a, planner pick).

| Id | Name | Founder | Modifiers and effects (FP) |
| --- | --- | --- | --- |
| `keepsake_kaelen` | Kaelen's Gouged Sigil | Kaelen | stat CONS +2; resist stagger 20, stun 20 |
| `keepsake_vesper` | Vesper's Runner Whistle (proposal) | Vesper | stat DEX +2; resist slow 25, root 25 |
| `keepsake_maren` | Maren's Copyist Quill (proposal) | Maren | stat KNOW +2; `skill_potency` group heal 0.1 |
| `keepsake_hollis` | Hollis's Focusing Monocle | Hollis | stat PERC +3 |
| `keepsake_dunstan` | Dunstan's Shackle Link (proposal) | Dunstan | stat `max_poise` +12; resist stun 30 |
| `keepsake_elyse` | Elyse's Cracked Retort (proposal) | Elyse | stat INT +2; `status_potency` 10 |

`name` short forms for lists: "Gouged Sigil", "Runner Whistle", "Copyist Quill", "Focusing Monocle", "Shackle Link",
"Cracked Retort". Delivery (ROSTER1 RQ7 a, planner pick): each founder's world character carries `keepsake`
`keepsake_<founder>` (CAST1_FOUNDERS 1); when the Crucible's `stay_behind` captures the founder (`then` `capture`),
ROSTER1-L puts one copy in the stash at once (KS1, KS2; the manifest's `keepsakes.on` lists `captured`, ROSTER1-D).
No loot row pays a keepsake: the six `loot_keepsake_<founder>` rows of the first draft are dropped.

## 8. Loot rows (`sunk_ward/loot.json`, CAST1-A5)

| Id | Name | Kind | Ref | Count | Rarity | Requires flags | Paid by |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `loot_gilded_pavise` | Gilded Pavise | item | `vanes_gilded_pavise` | 1 | unique | - | `enc_evt_01` objective reward |
| `loot_gate_pass` | Gate Pass Cipher | item | `city_gate_pass` | 1 | unique | - | `enc_evt_01` key loot (a footman) |
| `loot_marrow_scalpel` | Marrow Scalpel | item | `quells_marrow_scalpel` | 1 | unique | - | `enc_evt_02` key loot (Quell) |
| `loot_marrow_valve` | Marrow Valve | item | `inquisitor_marrow_valve` | 1 | rare | - | `enc_evt_02` key loot (Quell) |
| `loot_slab_retorts` | Slab Retorts | item | `slab_retorts` | 1 | unique | - | `enc_evt_02` objective reward |
| `loot_slag_hammer` | Slag-Hammer | item | `grendels_slag_hammer` | 1 | unique | - | `enc_evt_03` key loot (Grendel) |
| `loot_slag_plating` | Slag Plating | item | `slag_plating` | 1 | rare | - | `enc_evt_03` objective reward |
| `loot_flensing_talon` | Flensing Talon | item | `orlovs_flensing_talon` | 1 | unique | - | `enc_evt_04` key loot (Orlov) |
| `loot_orlovs_retribution` | Orlov's Retribution | skill | `orlovs_retribution` | 1 | unique | - | `enc_evt_04` key loot (Orlov), CQ10 |
| `loot_marrow_core` | Marrow Core | item | `titan_marrow_core` | 1 | unique | - | `enc_evt_05` key loot (the Amalgam) |
| `loot_dreamers_vertebra` | Dreamer's Vertebra | item | `dreamers_vertebra` | 1 | unique | - | `enc_evt_05` objective reward |
| `loot_poultices` | Stitch-Poultices | item | `sulfur_poultice` | 2 | common | - | `enc_pro_0c`, `enc_job_corvo_gate` objective rewards |

Unit `loot` tables (catalog units, one roll per defeated unit) are in CAST1_BESTIARY and CAST1_BOSSES.
