# `U2_CLASS_DECKS` — `[PROVISIONAL]` 6 Class Skill Decks, Passives & Legacy Disposition (`CLASSPASS1`)

- **Parent Index**: [U2_CLASSPASS_TECHNIQUES.md](U2_CLASSPASS_TECHNIQUES.md) (`[PROVISIONAL]` User Rule 5).
- **Build Target**: `PR5-62a` (`CLASSPASS1-KIT`) -> `PR5-62b` (`CLASSPASS1-DATA`).

---

## 1. Class Deck Matrix (Basic + 3 Flattened Actives + 2 Passives)

Each class keeps **1 Class Basic**, **3 single-rank active class-identity skills (`L1 / L2 / L4`)**, and **2 Class Passives (`ClassDef::passives`)** matching the Founder deck cadence (`kaelen`, `dunstan`, `hollis`, `vesper`, `maren`, `elyse`). Founder units keep their named Sunk Ward overrides/variants in `units.json` while sharing this chassis.

| Class (`sunk_ward`) | Starter Loadout Fix | Basic (`L1`) | Active 1 (`L1`) | Active 2 (`L2`) | Active 3 (`L4`) | Passives (`2`) |
| --- | --- | --- | --- | --- | --- | --- |
| **Warrior** (*Vanguard*) | `wpn_iron_billhook`, `shd_kite`, `arm_plate` | `hold_line` | `taunt_shout` | `intimidate` | `intercept_guard` | `vanguard_poise`, `iron_counter` |
| **Brawler** (*Slag-Brute*) | **`riveted_gauntlets`** (`gauntlet` Prof 3, replaces `wpn_knuckles`), `forearm_braces`, `arm_plate` | `sweep_kick` | `lunge` | `brawler_clinch` | `slag_charge` | `iron_shoulders`, `kiln_poise` |
| **Ranger** (*Arbalestier*) | `stirrup_arbalest`, `hip_quiver`, `arm_leather` | `disengage_step` | `mark_quarry` | `snare_wire` | `focus` | `keen_elevation`, `patient_sight` |
| **Rogue** (*Gutter-Stalker*) | `wpn_stiletto`, `ash_cloak`, `arm_leather` | `flank_cut` | `backflip` | `soot_feint` | `opportunist` | **`twin_light_tempo`**, `gutter_reflex` |
| **Priest** (*Heretic Acolyte*) | `bronze_censer`, `wpn_tome`, `arm_cleric_vestment` | `tend_wounds` | `purify` | `rally` | `aegis` | `triage_discipline`, `surgical_precision` |
| **Mage** (*Ichor Alchemist*) | `brass_vitriol_sprayer`, `wpn_orb`, `arm_mage_robe` | `cinder_spark` | `firebomb` | `glacial_burst` | `choke_cloud` | `reactive_reagents`, `catalytic_reserve` |

### Starter Gear Alignment Fix (`brawler`)

- Currently `classes.json` spawns `brawler` with `wpn_knuckles` (`fist`, Prof 2 = `1.0x`), despite `gauntlet` being `brawler`'s `main` (`Prof 3 = 1.25x`) proficiency. Add `COMMON` starter gauntlet **`riveted_gauntlets`** (*Riveted Gauntlets*, family `gauntlet`, `power: 13`, `burden: 1`, `STR >= 10`) so every class spawns holding its `main` Prof 3 weapon and off-hand (`founder_dunstan` keeps `penal_shackles` `+2` power).

---

## 2. Exact Specs for All 24 Class Skills

All class skills set `"ranks": []`, `"learned_only": true` (except `basics`), `"weapon_types": []`, and clean `timeline` budgets (`Snap = 1`, `Standard = 2`, `Heavy = 3`). Note `DEFEND` skills omit `windup_poise` and `unbreakable` (`check/skill.rs:56`), `DISENGAGE` skills omit `ends_in_stance` (`check/skill.rs:59`), and `position` maneuvers set `reach_min: 0` and `dash_tiles: 0` (`check/skill.rs:1068, 1089`).

| Class | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | Exact Numbers & Effects |
| --- | --- | --- | --- | --- | --- | --- |
| **Warrior** | `hold_line` | Hold Line | `DEFEND` -> `guard` | `1 / 2` | `self` | `mitigation: 0.25`, `guard_rounds: 1`, `grant_status: "fortify"`, `status_magnitude: 4`, `status_rounds: 1`. Replaces `brace` in `basics`. |
| **Warrior** | `taunt_shout` | Taunt Shout | `SUPPORT` (`SELF`) -> `guard` | `1 / 1` | `self` | `target_mode: "SELF"`, `grant_status: "taunt"`, `status_rounds: 2`, `windup_poise: 10` (`SUPPORT` permits `timing`). Flattened. |
| **Warrior** | `intimidate` | Dread Roar | `ATTACK` -> `assault` | `1 / 1` | `1–2` | `0.0x` dmg, `stagger 8`, `weaken` `-20%` (`pot 80, 2` rnds). Flattened. |
| **Warrior** | `intercept_guard` | Phalanx Cover | `DEFEND` -> `guard` | `3 / 2` | `self` | `mitigation: 0.35`, `guard_rounds: 1`, `grant_status: "steadfast"`, `status_rounds: 1`, `triggers`: `ally_hit` (`range: 1, melee: true, limit: "round"`) -> `{"type": "redirect", "pct": 100}` + `{"type": "grant", "status": "fortify", "magnitude": 6, "rounds": 2, "to": "self"}`. |
| **Brawler** | `sweep_kick` | Sweep Kick | `ATTACK` (`RING`) -> `guard` | `1 / 2` | `1` / `RING` | `0.75x` dmg, `stagger 10`. Weapon-agnostic close sweep (`weapon_types: []`, zero weapon-family overlap). Flattened. |
| **Brawler** | `lunge` | Reckless Lunge | `ATTACK` -> `maneuver` | `2 / 2` | `1` / `dash 3` | `1.15x` dmg, `stagger 10`, `dash_tiles: 3` (`position: ""`). Flattened. |
| **Brawler** | `brawler_clinch` | Collar Clinch | `ATTACK` -> `assault` | `2 / 2` | `1` | `0.70x` dmg, `stagger 16`, `slow` (`pot 80, 2` rnds), `sure_break: true`. |
| **Brawler** | `slag_charge` | Ramming Charge | `ATTACK` -> `maneuver` | `3 / 3` | `1` / `dash 4` | `1.35x` dmg, `stagger 18`, `dash_tiles: 4`, `forced_move: {kind: "push", tiles: 1, collide_mult: 0.6, collide_stagger: 12}`. |
| **Ranger** | `disengage_step` | Disengage Step | `DISENGAGE` -> `maneuver` | `1 / 1` | `jump 2` | `jump_tiles: 2` away from nearest threat. Unchanged basic. |
| **Ranger** | `mark_quarry` | Spotter's Mark | `ATTACK` -> `assault` | `1 / 1` | `2–5` | `0.0x` dmg, `expose -6` (`pot 90, 2` rnds), `stagger 4`, `riders`: `[{"when": {}, "effects": [{"type": "reveal", "rounds": 1}]}]`. |
| **Ranger** | `snare_wire` | Snare Wire | `ATTACK` -> `field` | `2 / 1` | `1–3` | `0.0x` dmg, `hazard: {id: "tripwire", shape: "NONE", at: "target", rounds: 3}`. |
| **Ranger** | `focus` | Steady Aim | `FOCUS` -> `guard` | `2 / 2` | `self` | `boost: 1.5` (`+50%` multiplier on next attack), `boost_rounds: 2`. Flattened. |
| **Rogue** | `flank_cut` | Flank Cut | `ATTACK` -> `maneuver` | `2 / 1` | `reach_max: 2` / `flank` | `0.60x` dmg, `stagger 8`, `reach_min: 0, reach_max: 2, dash_tiles: 0, position: "flank"`. Unchanged basic. |
| **Rogue** | `backflip` | Backflip | `DISENGAGE` -> `maneuver` | `1 / 1` | `jump 3` | `jump_tiles: 3` over units. Flattened. |
| **Rogue** | `soot_feint` | Slip & Feint | `ATTACK` -> `maneuver` | `1 / 1` | `reach_max: 2` / `behind` | `0.65x` dmg, `slow` (`pot 75, 2` rnds), `reach_min: 0, reach_max: 2, dash_tiles: 0, position: "behind"`, `ends_in_stance: "DEFENSIVE"`. |
| **Rogue** | `opportunist` | Exploit Opening | `ATTACK` -> `assault` | `2 / 1` | `1` | `0.90x` dmg, **`exploit` (`2` statuses <= max `2`)**: `slow 1.3x`, `bleed 1.3x` (synergizes directly with `soot_feint` & `twin_light_tempo`). |
| **Priest** | `tend_wounds` | Tend Wounds | `SUPPORT` -> `[support, guard]` | `2 / 2` | `0–2` | Heals `12 + 1.0 * KNOW`. Flattened baseline basic. |
| **Priest** | `purify` | Cauterize | `SUPPORT` -> `[support, guard]` | `1 / 1` | `0–3` | Heals `8 + 0.8 * KNOW`; cleanses `["bleed", "poison", "burn", "stun"]`. Flattened. |
| **Priest** | `rally` | Urgent Triage | `SUPPORT` (`ALLY`) -> `support` | `2 / 2` | `1–3` | `target_mode: "ALLY"`, heals `14 + 1.4 * KNOW`, `grant_status: "haste"`, `status_rounds: 1`, `riders`: `[{"when": {"other_broken": true}, "effects": [{"type": "guard", "mitigation": 0.4, "blocks": 2, "to": "other"}]}]`. Flattened. |
| **Priest** | `aegis` | Ward Litany | `SUPPORT` -> `[support, guard]` | `3 / 2` | `0–3` | `grant_status: "fortify"`, `status_magnitude: 8`, `status_rounds: 2`, `extra_grants: [{"status": "steadfast", "rounds": 1, "magnitude": 0}]`. Flattened. |
| **Mage** | `cinder_spark` | Cinder Spark | `ATTACK` (`PLUS`) -> `[field, assault]` | `2 / 2` | `2–3` / `PLUS` | `0.45x` dmg, `stagger 4`. |
| **Mage** | `firebomb` | Pitch Firebomb | `ATTACK` (`BURST`+`haz`) -> `field` | `2 / 2` | `2–4` / `BURST` | `0.50x` dmg, `burn` (`pot 70, mag 4, 2` rnds), `hazard: {id: "pitch_fire", shape: "SKILL", at: "target", rounds: 2}`. Flattened (`0.90x` rank removed). |
| **Mage** | `glacial_burst` | Alum Frost-Pot | `ATTACK` (`BURST`) -> `[field, assault]` | `2 / 2` | `2–3` / `BURST` | `0.50x` dmg, `root` (`pot 75, 1` rnd), `stagger 8`. Flattened (`0.90x` rank removed). |
| **Mage** | `choke_cloud` | Choke-Vapor Bomb | `ATTACK` (`BURST`+`haz`) -> `field` | `3 / 3` | `2–4` / `BURST` | `0.40x` dmg, `poison` (`pot 80, mag 5, 3` rnds), `hazard: {id: "choke_gas", shape: "SKILL", at: "target", rounds: 2}`. |

---

## 3. Class Passives (`12`) & Rogue `1h_light + 1h_light` Dual-Wield Passive

Every class defines `passives: [<p1>, <p2>]` in `classes.json` (`PR5-62a` adds `ClassDef::passives` + derived-unit deck hook, `"crit"` in `simple::TYPES` / non-gear `Unit`, and `"ranged"` filter on `SkillReach`):

| Passive ID | Name | Class | Exact `effects` Spec (`passives.json`) |
| --- | --- | --- | --- |
| `vanguard_poise` | Braced Stand | `warrior` | `[{"type": "stat", "stat": "max_poise", "value": 15}, {"type": "forced_move", "immune": "guarding"}]` |
| `iron_counter` | Iron Counter | `warrior` | `[{"type": "trigger", "on": "guarded_hit", "limit": "round", "effects": [{"type": "ap", "value": 1, "to": "self"}]}]` |
| `iron_shoulders` | Iron Shoulders | `brawler` | `[{"type": "stat", "stat": "CONS", "value": 2}, {"type": "forced_move", "bonus_tiles": 1}]` |
| `kiln_poise` | Kiln Endurance | `brawler` | `[{"type": "stagger_taken", "mult": 0, "max_blocks": 1}, {"type": "trigger", "on": "hp_below", "when": {"pct": 50}, "effects": [{"type": "grant", "status": "empower", "magnitude": 10, "rounds": 2, "to": "self"}], "limit": "battle"}]` |
| `keen_elevation` | Long Sight | `ranger` | `[{"type": "stat", "stat": "PERC", "value": 2}, {"type": "skill_reach", "kind": "ATTACK", "reach": 1, "ranged": true}]` (`PR5-62a` `SkillReach.ranged`) |
| `patient_sight` | Patient Sight | `ranger` | `[{"type": "status_potency", "value": 15}, {"type": "trigger", "on": "break_dealt", "when": {"ranged": true}, "limit": "round", "effects": [{"type": "ap", "value": 1, "to": "self"}]}]` |
| **`twin_light_tempo`** | **Twin-Blade Rhythm** | **`rogue`** | **Dual-Wield `1h_light + 1h_light` Passive (`PR5-62a` `"dual_wield": "1h_light"` `when` hook)**: see spec below. |
| `gutter_reflex` | Gutter Reflex | `rogue` | `[{"type": "resist", "status": "slow", "value": 40}, {"type": "resist", "status": "root", "value": 40}, {"type": "trigger", "on": "evaded", "limit": "round", "effects": [{"type": "grant", "status": "haste", "rounds": 1, "to": "self"}]}]` |
| `triage_discipline` | Triage Cadence | `priest` | `[{"type": "stat", "stat": "max_poise", "value": 8}, {"type": "trigger", "on": "heal_given", "when": {"other_hp_below": 35}, "limit": "round", "effects": [{"type": "ap", "value": 1, "to": "self"}]}]` |
| `surgical_precision` | Surgical Hand | `priest` | `[{"type": "skill_potency", "group": "heal", "value": 0.15}, {"type": "status_potency", "value": 10}]` |
| `reactive_reagents` | Reactive Reagents | `mage` | `[{"type": "resist", "status": "burn", "value": 50}, {"type": "resist", "status": "poison", "value": 50}, {"type": "trigger", "on": "hit_dealt", "when": {"other_status": "burn"}, "limit": "none", "effects": [{"type": "inflict", "status": "expose", "potency": 85, "magnitude": 5, "rounds": 2, "to": "other"}]}]` |
| `catalytic_reserve` | Catalytic Reserve | `mage` | `[{"type": "stat", "stat": "INT", "value": 1}, {"type": "skill_potency", "family": "firebomb", "value": 0.1}, {"type": "skill_potency", "family": "glacial_burst", "value": 0.1}, {"type": "skill_potency", "family": "choke_cloud", "value": 0.1}]` |

### Rogue `1h_light + 1h_light` Dual-Wield Passive (`twin_light_tempo`)

- **Trade-off (`U3_GEAR_REWORK` §3.2)**: `ash_cloak` (`off_only`) grants `+1 PERC` + `fade_step`; swapping the off-hand for a second **`1h_light`** weapon (`dagger`, `knife`, `fist`) opens `offhand_jab` (`1 AP, 1 blk`) and enables `twin_light_tempo`:
  1. `{"type": "crit", "value": 8, "when": {"dual_wield": "1h_light"}}` (`+8%` Crit Chance when both hands hold `1h_light`).
  2. `{"type": "skill_potency", "family": "offhand_jab", "value": 0.5, "when": {"dual_wield": "1h_light"}}` (lifts `offhand_jab` `0.50x -> 0.75x`).
  3. `{"type": "trigger", "on": "hit_dealt", "when": {"dual_wield": "1h_light", "skills": ["offhand_jab"]}, "limit": "round", "effects": [{"type": "ap", "value": 1, "to": "self"}, {"type": "inflict", "status": "bleed", "potency": 70, "rounds": 2, "to": "other"}]}`.
- **Required Story-Free Kit Hook (`PR5-62a`)**: Add `"dual_wield": "<slot_rule>"` (`Unit::has_dual_wield`) to `WHEN_KEYS`, allow `when` on passive grant rows (`crit`, `skill_potency`), add `"crit"` to `simple::TYPES`, and wire non-gear `Target::Crit` in `rpgkit_rules::unit`. Zero lore words touch `kits/rpg`.

---

## 4. Disposition of All `47` Existing Ranked & Overlapping Class Skills (`FB-231(b)`)

| Old Multi-Rank / Overlapping Skills (`47` `ranks` rows in `skills.json`) | Final Disposition | Target Owner / Notes |
| --- | --- | --- |
| `shield_bash`, `riposte`, `brace` (`3`) | **Removed from `warrior`** | Exclusive Off-Hand Buttons (`shield`, `buckler`, `braces`). `warrior` gains `hold_line` + `intercept_guard`. |
| `taunt_shout`, `intimidate`, `lunge`, `focus`, `backflip`, `opportunist`, `purify`, `rally`, `aegis`, `firebomb`, `glacial_burst`, `sweep_kick` (`12`) | **Flattened in Class Deck** | `ranks` deleted (`[]`), converted to single-rank stats in §2. |
| `sunder` (`greatsword`), `rend_armor` (`sword`), `crushing_blow` (`hammer`), `cyclone_kick` & `haymaker` (`gauntlet`), `body_blow` (`fist`), `aimed_shot` & `crippling_shot` & `deadeye` (`crossbow`), `pinning_shot` & `barbed_arrow` & `volley` (`bow`), `bleeding_cut` & `envenom` (`knife`), `quick_stab` & `lacerate` & `assassinate` (`dagger`), `smite` & `judgement` (`censer`), `arcane_bolt` & `arc_lance` & `hex` & `inferno` (`staff`) (`22`) | **Migrated to Weapon Mastery** | Removed from `classes.json` (`skill_path` cleared); `ranks` deleted (`[]`); promoted cleanly to `M1–M3 / MW` skills in [U2_WEAPON_TECHNIQUES.md](U2_WEAPON_TECHNIQUES.md) (`0` `duplicate_skill_errors`). |
| `shield_wall`, `charge_in`, `hamstring`, `leap_back`, `pounce`, `feint`, `mend`, `greater_mend`, `renewal`, `rejuvenate` (`10`) | **Retired / Consolidated** | Replaced (`intercept_guard`, `slag_charge`, `knife:hamstring_cut`, `disengage_step`, `soot_feint`, `purify` + `rally` + `tend_wounds`). All `47` `ranks` removed (`units.json` skirmish `loadout` updated in `PR5-62b`). |
