# `U2_CLASSPASS_TECHNIQUES` — `[PROVISIONAL]` Class Decks, Off-Hand Skills, 15 Weapon Mastery Trees & Graft Progression (`CLASSPASS1`)

- **Status**: **`[PROVISIONAL]`** — Ready for user playtest sign-off per User Rule 5 (`U2_CLASSPASS_TECHNIQUES`).
- **Baseline**: `origin/develop` (`a14c9d8e`) + `@design-triage` queued precursor `PR5-57` (`GEAR-REWORK1-FOLLOWUP`, `U3_GEAR_REWORK.md` §7).
- **Governance & Inputs**: `docs/PRINCIPLES.md` (`P-ARCH-01..06`, `P-SYS-04`), `docs/sessions/U3_GEAR_REWORK.md`, `docs/sessions/PK1_CORRUPTION.md`, `docs/sessions/PK2_GRAFTS.md`, `games/emberwake/docs/lore/CANON.md` (§4.2, §5.3, §6.1), `docs/FEEDBACK.md` (`FB-231(b)`, `FB-265(f)`, `FB-267(3)`, `FB-268(a-c)`).
- **Formatting Note**: Every skill ID satisfies `ID_MAX = 24` (`[a-z0-9_]`), every display name satisfies `NAME_MAX = 24`, every skill intent follows `rpgkit_rules::intent::derive` (`IR21`), and every status uses only the 18 engine status IDs in `statuses.json`.

---

## 0. Architectural Invariants & Boundary Summary

1. **Zero Skill Duplication Across Progression Lanes (`duplicate_skill_errors`)**: Every skill ID belongs to **at most one** bucket: Class Basic (`basics`), Class Deck (`skill_path`), Off-Hand Signature Button (`off_hand_skill`), Main-Hand Weapon Family (`techniques` / `item.technique` / `keystone`), or Slab Graft (`cranial` / `growth`). No Class Deck skill carries `weapon_types` that overlap weapon Mastery Techniques, so switching between `main`, `secondary`, and `third` weapons never breaks a character's class identity (`P-ARCH-01`).
2. **Flat Single-Rank Skills (`FB-231(b)` & `FB-265(f)`)**: All 29 legacy `ranks` arrays and `_2`/`_3` scaling tiers are eliminated. Class identity skills unlock whole at `L1 / L2 / L4`, while offensive weapon specialization comes cleanly from **Mastery `1/2/3`** + **Masterwork Reforge (`4th`)** + **Cold Steel Keystone** (`PK2_GRAFTS.md` §5) or **Slab Channel Grafts** (`PK1`/`PK2`).
3. **Kit vs. Game Layer Separation (`G3` / `fc.sh layers`)**: All code hooks required in `PR5-57a` (`dual_wield`, `stat_override_factor`, `duplicate_skill_errors` extension) remain 100% story-free and carry zero Emberwake proper nouns (`games/emberwake/lore_words.txt`). Sunk Ward world overlays, Slab implants, and Cold Steel items use `[LOCKED]` `CANON.md` terminology and zero §6.1 retired words.

---

## 1. Part 1 — 6 Class Skill Decks (`CLASSPASS1`)

### 1.1 Class Deck Matrix (Basic + 3 Flattened Actives + 2 Passives)

Each class keeps **1 Class Basic**, **3 single-rank active class-identity skills (`L1 / L2 / L4`)**, and **2 Class Passives (`passive` deck slots)** matching the Founder deck cadence (`kaelen`, `dunstan`, `hollis`, `vesper`, `maren`, `elyse`). Founder units keep their named Sunk Ward signature overrides/variants in `units.json` while sharing the exact structural chassis below.

| Class (`sunk_ward`) | Starter Loadout Fix | Basic (`L1`) | Active 1 (`L1`) | Active 2 (`L2`) | Active 3 (`L4`) | Passive 1 & Passive 2 |
| --- | --- | --- | --- | --- | --- | --- |
| **Warrior** (*Vanguard*) | `wpn_iron_billhook`, `shd_kite`, `arm_plate` | `hold_line` (*Hold Line*) | `taunt_shout` (*Taunt Shout*) | `intimidate` (*Dread Roar*) | `intercept_guard` (*Phalanx Cover*) | `vanguard_poise`, `iron_counter` |
| **Brawler** (*Slag-Brute*) | **`riveted_gauntlets`** (`gauntlet` Prof 3, replaces `wpn_knuckles`), `forearm_braces`, `arm_plate` | `sweep_kick` (*Sweep Kick*) | `lunge` (*Reckless Lunge*) | `brawler_clinch` (*Collar Clinch*) | `slag_charge` (*Ramming Charge*) | `iron_shoulders`, `kiln_poise` |
| **Ranger** (*Arbalestier*) | `stirrup_arbalest`, `hip_quiver`, `arm_leather` | `disengage_step` (*Disengage Step*) | `mark_quarry` (*Spotter's Mark*) | `snare_wire` (*Snare Wire*) | `focus` (*Steady Aim*) | `keen_elevation`, `patient_sight` |
| **Rogue** (*Gutter-Stalker*) | `wpn_stiletto`, `ash_cloak`, `arm_leather` | `flank_cut` (*Flank Cut*) | `backflip` (*Backflip*) | `soot_feint` (*Slip & Feint*) | `opportunist` (*Exploit Opening*) | **`twin_light_tempo`**, `gutter_reflex` |
| **Priest** (*Heretic Acolyte*) | `bronze_censer`, `wpn_tome`, `arm_cleric_vestment` | `tend_wounds` (*Tend Wounds*) | `purify` (*Cauterize*) | `rally` (*Urgent Triage*) | `aegis` (*Ward Litany*) | `triage_discipline`, `surgical_precision` |
| **Mage** (*Ichor Alchemist*) | `brass_vitriol_sprayer`, `wpn_orb`, `arm_mage_robe` | `cinder_spark` (*Cinder Spark*) | `firebomb` (*Pitch Firebomb*) | `glacial_burst` (*Alum Frost-Pot*) | `choke_cloud` (*Choke-Vapor Bomb*) | `reactive_reagents`, `catalytic_reserve` |

#### Starter Gear Alignment Fix (`brawler`)

- Currently `classes.json` spawns `brawler` with `wpn_knuckles` (`fist`, Prof 2 = `1.0x`), despite `gauntlet` being `brawler`'s `main` (`Prof 3 = 1.25x`) proficiency. Add `COMMON` starter gauntlet **`riveted_gauntlets`** (*Riveted Gauntlets*, family `gauntlet`, `power: 13`, `burden: 1`, `STR >= 10`) so every class spawns holding its `main` Prof 3 main-hand and off-hand (`founder_dunstan` keeps `penal_shackles` `+2` power).

### 1.2 Exact Specs for New & Flattened Class Skills

All class skills set `"ranks": []`, `"learned_only": true` (except `basics`), `"weapon_types": []`, and clean `timeline` AP/block budgets (`Snap = 1`, `Standard = 2`, `Heavy = 3–4`).

| Class | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | Exact Numbers & Effects |
| --- | --- | --- | --- | --- | --- | --- |
| **Warrior** | `hold_line` | Hold Line | `DEFEND` -> `guard` | `1 / 2` | `self` | `-25%` dmg (`mitigation: 0.25`), `guard_rounds: 1`, `+10` `windup_poise`, `fortify +4` (`1` rnd). Replaces `brace` in `warrior.basics`. |
| **Warrior** | `taunt_shout` | Taunt Shout | `SUPPORT` (`SELF`) -> `guard` | `1 / 1` | `self` | `taunt` `2` rnds, `+10` `windup_poise`. Flattened. |
| **Warrior** | `intimidate` | Dread Roar | `ATTACK` -> `assault` | `1 / 1` | `1–2` | `0.0x` dmg, `stagger 8`, `weaken` `-20%` (`potency 80`, `2` rnds). Flattened. |
| **Warrior** | `intercept_guard` | Phalanx Cover | `DEFEND` -> `guard` | `3 / 2` | `self` | `-35%` dmg, `steadfast` `1` rnd, `triggers`: `ally_hit` (`range: 1, melee: true`, `limit: "round"`) -> `redirect 100%` + `fortify +6` (`2` rnds). |
| **Brawler** | `sweep_kick` | Sweep Kick | `ATTACK` (`RING`) -> `guard` | `1 / 2` | `1` / `RING` | `0.75x` dmg, `stagger 10`. Works bare-handed, `gauntlet`, `fist`, or `hammer`. Flattened. |
| **Brawler** | `lunge` | Reckless Lunge | `ATTACK` -> `maneuver` | `2 / 2` | `1` / `dash 3` | `1.15x` dmg, `stagger 10`, `dash_tiles: 3`. Flattened. |
| **Brawler** | `brawler_clinch` | Collar Clinch | `ATTACK` -> `assault` | `2 / 2` | `1` | `0.70x` dmg, `stagger 16`, `slow` (`potency 80`, `2` rnds), `sure_break: true`. |
| **Brawler** | `slag_charge` | Ramming Charge | `ATTACK` -> `maneuver` | `3 / 3` | `1` / `dash 4` | `1.35x` dmg, `stagger 18`, `dash_tiles: 4`, `forced_move: push 1` (`collide_mult: 0.6, collide_stagger: 12`). |
| **Ranger** | `disengage_step` | Disengage Step | `DISENGAGE` -> `maneuver` | `1 / 1` | `jump 2` | Jumps `2` tiles away from nearest threat. Unchanged basic. |
| **Ranger** | `mark_quarry` | Spotter's Mark | `ATTACK` -> `assault` | `1 / 1` | `2–5` | `0.0x` dmg, `expose -6` (`potency 90`, `2` rnds), `stagger 4`, `riders`: `reveal` round plan (`1` rnd). |
| **Ranger** | `snare_wire` | Snare Wire | `ATTACK` -> `field` | `2 / 1` | `1–3` | `0.0x` dmg, places `tripwire` `hazard` (`shape: "NONE", at: "target", rounds: 3`). |
| **Ranger** | `focus` | Steady Aim | `FOCUS` -> `guard` | `2 / 2` | `self` | `boost: 0.5` (`+50%` next attack), `boost_rounds: 2`. Flattened. |
| **Rogue** | `flank_cut` | Flank Cut | `ATTACK` -> `maneuver` | `2 / 1` | `dash 2` / `flank` | `0.60x` dmg, `stagger 8`, `dash_tiles: 2`, `position: "flank"`. Unchanged basic. |
| **Rogue** | `backflip` | Backflip | `DISENGAGE` -> `maneuver` | `1 / 1` | `jump 3` | Jumps `3` tiles away over units. Flattened. |
| **Rogue** | `soot_feint` | Slip & Feint | `ATTACK` -> `maneuver` | `1 / 1` | `dash 2` / `behind` | `0.65x` dmg, `slow` (`potency 75`, `2` rnds), `ends_in_stance: "DEFENSIVE"`. |
| **Rogue** | `opportunist` | Exploit Opening | `ATTACK` -> `assault` | `2 / 1` | `1` | `0.90x` dmg, `exploit`: `expose 1.3x`, `slow 1.25x`, `stagger 1.3x`, `bleed 1.25x`. Flattened, weapon-agnostic. |
| **Priest** | `tend_wounds` | Tend Wounds | `SUPPORT` -> `[support, guard]` | `2 / 2` | `0–2` | Heals `12 + 1.0 * KNOW`. Flattened baseline basic. |
| **Priest** | `purify` | Cauterize | `SUPPORT` -> `[support, guard]` | `1 / 1` | `0–3` | Heals `8 + 0.8 * KNOW`; cleanses `["bleed", "poison", "burn", "stun"]`. Flattened. |
| **Priest** | `rally` | Urgent Triage | `SUPPORT` (`ALLY`) -> `support` | `2 / 2` | `1–3` | Heals `14 + 1.4 * KNOW`; grants `haste` (`1` rnd); `rider` `{other_broken: true} -> guard 0.4`. Flattened. |
| **Priest** | `aegis` | Ward Litany | `SUPPORT` -> `[support, guard]` | `3 / 2` | `0–3` | Grants `fortify +8` (`2` rnds) and `steadfast` (`1` rnd). Flattened. |
| **Mage** | `cinder_spark` | Cinder Spark | `ATTACK` (`PLUS`) -> `[field, assault]` | `2 / 2` | `2–3` / `PLUS` | `0.45x` dmg in cross area, `stagger 4`. |
| **Mage** | `firebomb` | Pitch Firebomb | `ATTACK` (`BURST`+`haz`) -> `field` | `2 / 2` | `2–4` / `BURST` | `0.50x` dmg, `burn` (`potency 70, mag 4, 2` rnds), places `pitch_fire` `hazard` (`2` rnds). Flattened (`0.90x` rank removed). |
| **Mage** | `glacial_burst` | Alum Frost-Pot | `ATTACK` (`BURST`) -> `[field, assault]` | `2 / 2` | `2–3` / `BURST` | `0.50x` dmg, `root` (`potency 75, 1` rnd), `stagger 8`. Flattened (`0.90x` rank removed). |
| **Mage** | `choke_cloud` | Choke-Vapor Bomb | `ATTACK` (`BURST`+`haz`) -> `field` | `3 / 3` | `2–4` / `BURST` | `0.40x` dmg, `poison` (`potency 80, mag 5, 3` rnds), places `choke_gas` `hazard` (`2` rnds). |

### 1.3 Class Passives & Rogue `1h_light + 1h_light` Dual-Wield Passive

Every class defines two `"slot": "passive"` rows (added to `passives.json`; Sunk Ward founders retain their existing bespoke passives alongside or superseding these cleanly):

| Passive ID | Name | Class | Exact `effects` Spec |
| --- | --- | --- | --- |
| `vanguard_poise` | Braced Stand | `warrior` | `[{"type": "stat", "stat": "max_poise", "value": 15}, {"type": "forced_move", "immune": "guarding"}]` |
| `iron_counter` | Iron Counter | `warrior` | `[{"type": "trigger", "on": "guarded_hit", "limit": "round", "effects": [{"type": "ap", "value": 1}]}]` |
| `iron_shoulders` | Iron Shoulders | `brawler` | `[{"type": "stat", "stat": "CONS", "value": 2}, {"type": "forced_move", "bonus_tiles": 1}]` |
| `kiln_poise` | Kiln Endurance | `brawler` | `[{"type": "stagger_taken", "mult": 0, "max_blocks": 1}, {"type": "trigger", "on": "hp_below", "when": {"pct": 50}, "effects": [{"type": "grant", "status": "empower", "magnitude": 10, "rounds": 2}], "limit": "battle"}]` |
| `keen_elevation` | Long Sight | `ranger` | `[{"type": "stat", "stat": "PERC", "value": 2}, {"type": "skill_reach", "kind": "ATTACK", "reach": 1, "ranged": true}]` |
| `patient_sight` | Patient Sight | `ranger` | `[{"type": "status_potency", "value": 15}, {"type": "trigger", "on": "break_dealt", "when": {"ranged": true}, "limit": "round", "effects": [{"type": "ap", "value": 1}]}]` |
| **`twin_light_tempo`** | **Twin-Blade Rhythm** | **`rogue`** | **Dual-Wield `1h_light + 1h_light` Passive (`PR5-57a` kit hook `dual_wield: "1h_light"`)**: see full spec below. |
| `gutter_reflex` | Gutter Reflex | `rogue` | `[{"type": "resist", "status": "slow", "value": 40}, {"type": "resist", "status": "root", "value": 40}, {"type": "trigger", "on": "evaded", "limit": "round", "effects": [{"type": "grant", "status": "haste", "rounds": 1}]}]` |
| `triage_discipline` | Triage Cadence | `priest` | `[{"type": "stat", "stat": "max_poise", "value": 8}, {"type": "trigger", "on": "heal_given", "when": {"other_hp_below": 35}, "limit": "round", "effects": [{"type": "ap", "value": 1}]}]` |
| `surgical_precision` | Surgical Hand | `priest` | `[{"type": "skill_potency", "group": "heal", "value": 0.15}, {"type": "status_potency", "value": 10}]` |
| `reactive_reagents` | Reactive Reagents | `mage` | `[{"type": "resist", "status": "burn", "value": 50}, {"type": "resist", "status": "poison", "value": 50}, {"type": "trigger", "on": "hit_dealt", "when": {"other_status": "burn"}, "limit": "none", "effects": [{"type": "inflict", "status": "expose", "potency": 85, "magnitude": 5, "rounds": 2, "to": "other"}]}]` |
| `catalytic_reserve` | Catalytic Reserve | `mage` | `[{"type": "stat", "stat": "INT", "value": 1}, {"type": "skill_potency", "family": "firebomb", "value": 0.1}, {"type": "skill_potency", "family": "glacial_burst", "value": 0.1}, {"type": "skill_potency", "family": "choke_cloud", "value": 0.1}]` |

#### Rogue `1h_light + 1h_light` Dual-Wield Passive Spec (`twin_light_tempo`)

- **Design Trade-off (`U3_GEAR_REWORK` §3.2)**: A Rogue equipping **`ash_cloak`** (`off_only`) gets `+1 PERC` and emergency `fade_step`; swapping the off-hand for a second **`1h_light` weapon** (`dagger`, `knife`, or `fist`) unlocks `offhand_jab` (`1 AP, 1 blk`, reaches `off_hand` weapon's `range_band`) and activates `twin_light_tempo`:
  1. `{"type": "crit", "value": 8, "when": {"dual_wield": "1h_light"}}` (`+8%` Crit Chance while both hands hold `1h_light` weapons).
  2. `{"type": "skill_potency", "family": "offhand_jab", "value": 0.5, "when": {"dual_wield": "1h_light"}}` (lifts `offhand_jab` from `0.50x` to `0.75x` off-hand weapon power — competitive with a `1 AP` `quick_stab`!).
  3. `{"type": "trigger", "on": "hit_dealt", "when": {"dual_wield": "1h_light", "skills": ["offhand_jab"]}, "limit": "round", "effects": [{"type": "ap", "value": 1}, {"type": "inflict", "status": "bleed", "potency": 70, "rounds": 2}]}` (`offhand_jab` inflicts Bleed and refunds `+1 AP` once per round when both hands hold `1h_light`).
- **Required Story-Free Kit Hook (`PR5-57a`)**:
  - Add `"dual_wield": "<slot_rule>"` (matching `Unit::has_dual_wield(slot_rule) -> bool`, true iff `main_hand.slot_rule == rule && off_hand.slot_rule == rule`) to `WHEN_KEYS` in `rpgkit_data` / `rpgkit_rules::trigger` AND allow the same `when` filter on passive `crit` / `skill_potency` grant rows (or add `dual_wield` to grant condition context alongside `weapon` context). Zero class IDs or Emberwake lore terms touch the kit (`P-ARCH-01`, `fc.sh layers` compliant).

### 1.4 Disposition Table of All 29 Existing Multi-Rank & Overlapping Class Skills (`FB-231(b)`)

| Old Multi-Rank Skill(s) | Final Disposition | Target Owner / Notes |
| --- | --- | --- |
| `shield_bash`, `riposte`, `brace` | **Removed from `warrior`** | Exclusive Off-Hand Signature Buttons (`shield`, `buckler`, `braces`). `warrior` gains `hold_line` + `intercept_guard`. |
| `taunt_shout`, `intimidate`, `lunge`, `focus`, `backflip`, `opportunist`, `purify`, `rally`, `aegis`, `firebomb`, `glacial_burst`, `sweep_kick` | **Flattened in Class Deck** | `ranks` deleted (`12` skills flattened directly to single-rank stats in §1.2). |
| `sunder` (`greatsword`), `rend_armor` (`sword`), `crushing_blow` (`hammer`), `cyclone_kick` & `haymaker` (`gauntlet`), `body_blow` (`fist`), `aimed_shot` & `crippling_shot` & `deadeye` (`crossbow`), `pinning_shot` & `barbed_arrow` & `volley` (`bow`), `bleeding_cut` & `envenom` (`knife`), `quick_stab` & `lacerate` & `assassinate` (`dagger`), `smite` & `judgement` (`censer`), `arcane_bolt` & `arc_lance` & `hex` & `inferno` (`staff`) | **Migrated to Weapon Mastery** | Removed from `classes.json` (`skill_path` cleared); `ranks` deleted; promoted cleanly to `M1–M3 / MW` skills in Part 3 (`0` `duplicate_skill_errors`). |
| `shield_wall`, `charge_in`, `hamstring`, `leap_back`, `pounce`, `feint`, `mend`, `greater_mend`, `renewal`, `rejuvenate` | **Replaced / Retired in `CLASSPASS1`** | Consolidated (`intercept_guard`, `slag_charge`, `knife:hamstring_cut`, `disengage_step`, `soot_feint`, `purify` + `rally` + `tend_wounds`). Any legacy `ashlands` encounter entries referencing these keep flat rank-1 values (all `ranks` arrays removed so `FB-231(b)` holds catalog-wide). |

---

## 2. Part 2 — 7 Off-Hand Signature Button Skills (`U3_GEAR_REWORK` §4, `FB-267(3)`) & `orb` Balance Fix (`FB-268(a)`)

### 2.1 Complete Spec for All 7 `off_only` Family Skills

Equiping an off-hand item grants its family's `off_hand_skill` while worn (`Unit::gear_skills`) and surfaces it on the dedicated battle **Off-Hand Button** (`offhand_use`). Note `offhand_jab` remains exclusively for `1h_light` off-hand weapons (`dagger`, `knife`, `fist`).

| Family (`slot_rule: off_only`) | Skill ID & Name | Kind -> IR21 | AP / Blk | Reach | Exact Specs & Numbers (`ranks: []`) | `FB-267(3)` / Notes |
| --- | --- | --- | --- | --- | --- | --- |
| **`shield`** (`mit 6, bur 2`) | `shield_bash` (*Shield Bash*) | `ATTACK` -> `assault` | `2 / 2` | `1` | `0.75x` power (`weapon_hand: ""` scales STR main-hand), `stagger: 16`, `stagger` status (`potency 75, 1` rnd), `windup_poise: 8`, `forced_move: {kind: "push", tiles: 1, collide_mult: 0.5, collide_stagger: 8}`. | Removed from `warrior`; bound to `shield.off_hand_skill`. |
| **`buckler`** (`mit 4, bur 0`) | `riposte` (*Riposte*) | `DEFEND` -> `guard` | `2 / 2` | `self` | `mitigation: 0.15`, `guard_rounds: 1`, `grant_status: "riposte"` (`status_rounds: 2`, counters surviving adjacent melee hits for `50%` attack power `1/attacker/round`). | Removed from `warrior`; bound to `buckler.off_hand_skill`. |
| **`braces`** (`mit 3, bur 1`) | `brace` (*Brace*) | `DEFEND` -> `guard` | `2 / 3` | `self` | `mitigation: 0.25`, `guard_rounds: 2`, `windup_poise: 12` (two-round guard + poise buffer beats generic `Active Guard` on multi-round sustain). | Removed from `warrior.basics`; bound to `braces.off_hand_skill`. |
| **`ammo`** (`pwr 0, bur 0`) | `reload` (*Reload*) | `FOCUS` -> `guard` | `1 / 1` | `self` | Refills `ammo_pips` meter to max (`3`); gated open only at `0` pips (`offhand::reload_problem`). `empower_next`: `{meter: "ammo_pips", cost: 1, power_mult: 1.25, throw_skill: "knife_throw"}`. | Ratifies `PR5-51b` (`FB-265(a)` + `FB-267(3)`). `round_intent` removed in favor of IR21 (`guard`). |
| **`cloak`** (`pwr 0, bur 0`) | `fade_step` (*Fade-Step*) | `DISENGAGE` -> `maneuver` | `1 / 1` | `jump 1` | `jump_tiles: 1` away from nearest threat, `ends_in_stance: "DEFENSIVE"` (clean `1`-tile emergency break; inferior to Rogue's `3`-tile `backflip`). | Ratifies `PR5-51b` Build numbers. |
| **`orb`** (`power_stat_override: INT`) | **`catalyst_pulse`** (*Catalyst Pulse*) | `FOCUS` -> `guard` | `2 / 1` | `self` | `boost: 0.25` (`+25%` damage on caster's next attack/spell for `boost_rounds: 2`), `windup_poise: 8`. Replaces unconditional passive `wpn_orb` damage bonus with deliberate `2 AP / 1 block` setup! | Closes open `FB-267(3)` spec gap (`orb.off_hand_skill`). |
| **`tome`** (`pwr 0, bur 0`) | `cite_passage` (*Cite Passage*) | `SUPPORT` -> `[support, guard]` | `1 / 1` | `0–2` | `target_mode: ""` (`ALLY_OR_SELF`, fixing `PR5-51b`'s `ALLY`-only mismatch against its description), grants `fortify +4` (`status_rounds: 2`). | Ratifies & fixes `PR5-51b` `cite_passage`. |

- Also retain `knife_throw` (*Throw*, `ATTACK` -> `assault`, `1 AP, 1 blk`, `reach 2-3`, `1.0x`, `weapon_types: ["knife"], min_proficiency: 1`), triggered alongside `ammo` + `1h_light` `knife`.

### 2.2 `orb` `INT` Weapon-Power Scaling & `FB-268(a)` Boss-Damage Tuning Fix

**Root-Cause Finding (`30415b91` + `damage.rs:157-170`)**: In `PR5-51b`, `mage` L5 boss damage spiked `214 -> 396` (`50% -> 80%` win) from three compounding changes:

1. Giving `mage` starter `wpn_orb` added an unconditional **`+10% spell potency`** AND **`+1 SPELL reach`** (`staff`/`sprayer` skills fired on Turn 1 without moving).
2. `Prof 3` (`PROF_GEAR_MULT = 1.25`) boosted `brass_vitriol_sprayer`'s `15` base power (`1H`!) to `19` alongside rank-3 spell multipliers (`arcane_bolt 1.65x`, `firebomb/glacial_burst 0.90x`).
3. Furthermore, `orb.power_stat_override: "INT"` in `damage::attack_power` (`damage.rs:160-166`) currently swaps a physical main-hand weapon's `scaling_stat` to `INT` at **100% (`1.0x`)**, letting `INT`-stackers wield high-base physical `1h_heavy` weapons (`bone_saw 14`, `sword 14`) at full martial stat scaling.

**Complete 3-Step Tuning Fix**:

1. **Orb Focus Rebalance (`Data` — `PR5-57` / `PR5-57b`)**: Remove unconditional `skill_potency: {group: "spell", value: 0.1}` from `wpn_orb` (keep `+1` reach via `skill_reach: {reach: 1, intent: "field"}` or `kind: "SPELL"` with `sprayer` carrying `spell_source`), moving burst payoff into active `2 AP / 1 block` **`catalyst_pulse` (`+25%` Focus)**. Lower `sprayer` `base.main_1h.power` from **`15 -> 12`** (`bands: 8..20`, so `Prof 3` `mage` hits `12 * 1.25 = 15`, preserving `2H staff`'s `24` raw power lead over `1H sprayer + orb`).
2. **Class Spell Flattening (`Data` — `CLASSPASS1`)**: Flattened `firebomb` (`0.50x`) and `glacial_burst` (`0.50x`) drop L5 AoE spam from `0.90x` back to tactical control/burn/root levels (`~44%` cut on L5 AoE damage before boss resists).
3. **Kit `power_stat_override` Scaling Cap (`PR5-57a` — story-free)**: In `rpgkit_rules::damage::attack_power`, when `stat_override` fires (off-hand `power_stat_override` replaces a weapon whose native `scaling_stat` differs from `over`), scale `unit_power` by **`stat_override_factor = 0.75`** (`Ruleset`/`constants`, default `0.75` under strict gear). Native `INT` weapons (`sprayer`, `censer`) keep `1.0x`, while hybrid `orb + bone_saw/sword` gains `75% INT` scaling instead of `100%`, eliminating non-caster melee scaling exploits while keeping hybrid builds viable (`counts=on` metric `orb_override` verifies usage).

---

## 3. Part 3 — 15 Main-Hand Weapon Families × 5 Skills (`75` Skills Total)

### 3.1 Family Architecture & Progression Rules (`rpgkit_data::mastery`)

- Every main-hand `WEAPON` family defines **3 Mastery Techniques** (`techniques: [{"skill", "grip"}, ...]`, unlocked at Mastery `1 / 2 / 3` via `100 / 300 / 600` XP in families at **`Proficiency >= 2`**), **1 Cold Steel Keystone** (`keystone`, unlocked at Mastery `3` + `Berek`'s `forge_rite`, sitting in the deck's single `keystone` slot unless an Inoculated Keystone holds it), and **1 canonical Masterwork `4th` Technique** (`item.technique` on reinforced/reforged instances of weapons in that family via `hearth_reforge`).
- All 6 placeholder `tq_*` test techniques (`tq_polearm_hook`, `tq_crossbow_volley`, `tq_fist_breaker`, `tq_dagger_flurry`, `tq_censer_swing`, `tq_sprayer_jet`) are retired/replaced cleanly below (`seed` removes `tq_*` after updating `item_families.json`).
- **Act 2 Durability Hook**: `1` durability per attack, `2` per Technique, `0` = `dulled` (`0.5x` power).

### 3.2 Overview Matrix Across All 15 Families

| # | Family (`slot_rule`) | Primary Stat / Reach / Grip | M1 (`Mastery 1`) | M2 (`Mastery 2`) | M3 (`Mastery 3`) | MW (`Masterwork 4th`) | KS (`Cold Steel Keystone`) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `polearm` (`1h_heavy`) | `STR` / `1–2` / `1h` | `hook_drag` | `pike_skew` | `check_advance` | `halberd_sweep` | `polearm_cordon` |
| 2 | `greatsword` (`2h_solo`) | `STR` / `1` / `2h` | `sunder` | `wide_cleave` | `rending_arc` | `gallow_sweep` | `gsword_sever` |
| 3 | `sword` (`1h_heavy`) | `STR` / `1` / `1h` | `pommel_check` | `rend_armor` | `bind_parry` | `hookbill_draw` | `sword_duelist` |
| 4 | `hammer` (`2h_solo`) | `STR` / `1` / `2h` | `crushing_blow` | `earth_shaker` | `skull_ring` | `slag_driver` | `hammer_shatter` |
| 5 | `gauntlet` (`1h_heavy`) | `STR` / `1` / `1h` | `knuckle_jolt` | `cyclone_kick` | `haymaker` | `foundry_drive` | `gauntlet_breaker` |
| 6 | `fist` (`1h_light`) | `STR` / `1` / `1h` | `body_blow` | `palm_check` | `liver_hook` | `rapid_combo` | `fist_tempest` |
| 7 | `crossbow` (`2h_rig`) | `DEX` / `2–4` / `2h` | `aimed_shot` | `crippling_shot` | `deadeye` | `bodkin_punch` | `crossbow_siege` |
| 8 | `bow` (`2h_rig`) | `DEX` / `2–4` / `2h` | `pinning_shot` | `barbed_arrow` | `volley` | `wing_shot` | `bow_rain` |
| 9 | `knife` (`1h_light`) | `DEX` / `1–2` / `1h` | `hamstring_cut` | `bleeding_cut` | `envenom` | `gut_hook` | `knife_sever` |
| 10 | `dagger` (`1h_light`) | `DEX` / `1` / `1h` | `quick_stab` | `lacerate` | `assassinate` | `bodkin_twist` | `dagger_shadow` |
| 11 | `censer` (`1h_heavy`) | `INT/STR` / `1` / `1h` | `smite` | `chime_swing` | `judgement` | `anoint_flare` | `censer_pyre` |
| 12 | `bone_saw` (`1h_heavy`) | `DEX/STR` / `1` / `1h` | `amputate_cut` | `nerve_nick` | `rib_spread` | `vivisect` | `saw_dissect` |
| 13 | `scourge` (`1h_heavy`) | `DEX/STR` / `1–2` / `1h` | `snap_lash` | `hook_snatch` | `flay_arc` | `penance_bite` | `scourge_flay` |
| 14 | `sprayer` (`1h_heavy`) | `INT` / `2–3` / `1h` | `caustic_spritz` | `pitch_cone` | `pressure_stream` | `flash_nozzle` | `sprayer_deluge` |
| 15 | `staff` (`2h_solo`) | `INT` / `2` / `2h` | `arcane_bolt` | `arc_lance` | `hex` | `inferno` | `staff_cataclysm` |

### 3.3 Exact Skill Definitions Across All 15 Families (`75` Single-Rank Skills)

Every technique sets `"weapon_types": ["<family>"]`, `"min_proficiency": 1` (gating via Mastery/Masterwork unlocks rather than redundant class checks), `"learned_only": true`, `"ranks": []`, and `uses_per_battle: 1` on Cold Steel Keystones (`KS`).

#### Families 1–5 (Polearm, Greatsword, Sword, Hammer, Gauntlet)

| Fam / Tier | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | `power_mult` / `stagger` / Effects |
| --- | --- | --- | --- | --- | --- | --- |
| `polearm` M1 | `hook_drag` | Billhook Drag | `ATTACK` -> `assault` | `2 / 2` | `1–2` | `1.05x`, `stag 10`, `forced_move: pull 1`. Replaces `tq_polearm_hook`. |
| `polearm` M2 | `pike_skew` | Skewer Thrust | `ATTACK` (`LINE`) -> `[field, assault]` | `2 / 2` | `1` / `LINE` | `0.95x`, `stag 8`, `armor_piercing: true`. |
| `polearm` M3 | `check_advance` | Halt Advance | `ATTACK` -> `assault` | `3 / 3` | `1–2` | `1.45x`, `stag 18`, `root` (`pot 85, 1` rnd), `sure_break: true`. |
| `polearm` MW | `halberd_sweep` | Crescent Halberd | `ATTACK` (`ARC`) -> `[field, assault]` | `3 / 2` | `1–2` / `ARC` | `1.20x`, `stag 14`, `forced_move: push 1`. |
| `polearm` KS | `polearm_cordon` | Castellan Cordon | `DEFEND` -> `guard` | `3 / 3` | `self` | `mit 0.40`, `guard_rounds: 2`, `windup_poise: 20`, `unbreakable: true`, `ally_hit` (`range: 2, melee`) -> `redirect 100%` + `pull 1` (`1/rnd`), `uses: 1`. |
| `greatsword` M1 | `sunder` | Sunder Chop | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.30x`, `stag 12`, `expose -6` (`pot 75, 2` rnds). |
| `greatsword` M2 | `wide_cleave` | Broad Cleave | `ATTACK` (`ARC`) -> `[field, assault]` | `3 / 2` | `1` / `ARC` | `1.15x`, `stag 10`. |
| `greatsword` M3 | `rending_arc` | Armor-Splitter | `ATTACK` -> `assault` | `3 / 3` | `1` | `1.70x`, `stag 14`, `expose -10` (`pot 85, 3` rnds), `windup_poise: 12`. |
| `greatsword` MW | `gallow_sweep` | Gallows Sweep | `ATTACK` (`ARC`) -> `[field, assault]` | `4 / 3` | `1` / `ARC` | `1.50x`, `stag 16`, `exploit`: `stagger 1.35x`, `expose 1.25x`. |
| `greatsword` KS | `gsword_sever` | Steel Severance | `ATTACK` -> `assault` | `4 / 4` | `1` | `2.30x`, `stag 22`, `unbreakable: true`, `armor_piercing: true`, `uses: 1`. |
| `sword` M1 | `pommel_check` | Pommel Check | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.70x`, `stag 12`, `sure_break: true`. |
| `sword` M2 | `rend_armor` | Half-Sword Thrust | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.15x`, `stag 8`, `expose -8` (`pot 80, 2` rnds). |
| `sword` M3 | `bind_parry` | Bind & Counter | `DEFEND` -> `guard` | `2 / 2` | `self` | `mit 0.30`, `guard_rounds: 1`, `riposte` (`2` rnds), `windup_poise: 10`. |
| `sword` MW | `hookbill_draw` | Hookbill Draw | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.25x`, `stag 8`, `bleed` (`pot 80, 2` rnds), `forced_move: pull 1`. |
| `sword` KS | `sword_duelist` | Master-Cut | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.95x`, `stag 16`, `ignore_guard: true`, `ends_in_stance: "DEFENSIVE"`, `uses: 1`. |
| `hammer` M1 | `crushing_blow` | Maul Smash | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.25x`, `stag 16`, `disrupt: "STANCE_BREAK"`. |
| `hammer` M2 | `earth_shaker` | Kiln Shockwave | `ATTACK` (`LINE`) -> `[field, assault]` | `3 / 3` | `1` / `LINE` | `1.15x`, `stag 18`, `forced_move: push 1`. |
| `hammer` M3 | `skull_ring` | Bell-Ringer | `ATTACK` -> `assault` | `3 / 3` | `1` | `1.75x`, `stag 24`, `stun` (`pot 65, 1` rnd), `windup_poise: 15`. |
| `hammer` MW | `slag_driver` | Piston Slam | `ATTACK` -> `assault` | `4 / 3` | `1` | `1.90x`, `stag 20`, `forced_move: push 2` (`collide_mult: 0.8, collide_stagger: 12`). |
| `hammer` KS | `hammer_shatter` | Sledge Judgment | `ATTACK` -> `assault` | `4 / 4` | `1` | `2.20x`, `stag 30`, `unbreakable: true`, `sure_break: true`, `forced_move: push 2`, `uses: 1`. |
| `gauntlet` M1 | `knuckle_jolt` | Shackle Jolt | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.85x`, `stag 14`, `forced_move: push 1`. |
| `gauntlet` M2 | `cyclone_kick` | Foundry Spin | `ATTACK` (`RING`) -> `guard` | `3 / 2` | `1` / `RING` | `1.00x`, `stag 14` (hits every adjacent enemy). |
| `gauntlet` M3 | `haymaker` | Haymaker | `ATTACK` -> `assault` | `3 / 3` | `1` | `1.85x`, `stag 20`, `stun` (`pot 60, 1` rnd), `windup_poise: 10`. |
| `gauntlet` MW | `foundry_drive` | Cestus Throw | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.30x`, `stag 16`, `forced_move: push 2` (`collide_mult: 1.0, collide_break: true`). |
| `gauntlet` KS | `gauntlet_breaker` | Anvil-Fist | `ATTACK` -> `assault` | `4 / 3` | `1` | `2.10x`, `stag 28`, `unbreakable: true`, `ignore_guard: true`, `forced_move: push 2` (`collide_break: true`), `uses: 1`. |

#### Families 6–10 (Fist, Crossbow, Bow, Knife, Dagger)

| Fam / Tier | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | `power_mult` / `stagger` / Effects |
| --- | --- | --- | --- | --- | --- | --- |
| `fist` M1 | `body_blow` | Rib Shot | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.90x`, `stag 14`, `weaken -15%` (`pot 75, 2` rnds). Replaces `tq_fist_breaker`. |
| `fist` M2 | `palm_check` | Palm Thrust | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.05x`, `stag 16`, `sure_break: true`. |
| `fist` M3 | `liver_hook` | Solar-Plexus Hook | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.45x`, `stag 22`, `stun` (`pot 55, 1` rnd). |
| `fist` MW | `rapid_combo` | Four-Knuckle Combo | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.55x`, `stag 16`, `exploit`: `stagger 1.35x`. |
| `fist` KS | `fist_tempest` | Chain-Breaker | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.90x`, `stag 25`, `sure_break: true`, `recover_ap: 1`, `uses: 1`. |
| `crossbow` M1 | `aimed_shot` | Winch Bolt | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `1.45x`, `stag 8`. |
| `crossbow` M2 | `crippling_shot` | Tendon Bolt | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `1.05x`, `stag 8`, `slow` (`pot 85, 2` rnds). |
| `crossbow` M3 | `deadeye` | Deadeye Quarrel | `ATTACK` -> `assault` | `3 / 3` | `2–5` | `1.95x`, `stag 14`, `sure_break: true`. Replaces `tq_crossbow_volley`. |
| `crossbow` MW | `bodkin_punch` | Bodkin Punch | `ATTACK` -> `assault` | `3 / 2` | `2–4` | `1.55x`, `stag 10`, `armor_piercing: true`, `ignore_guard: true`. |
| `crossbow` KS | `crossbow_siege` | Siege-Winch Shot | `ATTACK` -> `assault` | `4 / 4` | `2–6` | `2.25x`, `stag 18`, `armor_piercing: true`, `sure_break: true`, `forced_move: push 2`, `uses: 1`. |
| `bow` M1 | `pinning_shot` | Pinning Arrow | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `1.00x`, `stag 6`, `root` (`pot 85, 2` rnds). |
| `bow` M2 | `barbed_arrow` | Barbed Shaft | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `1.10x`, `stag 6`, `bleed` (`pot 80, 3` rnds). |
| `bow` M3 | `volley` | Arrow Volley | `ATTACK` (`PLUS`) -> `[field, assault]` | `3 / 2` | `2–4` / `PLUS` | `0.95x`, `stag 8`. |
| `bow` MW | `wing_shot` | Snapshot Loose | `ATTACK` -> `assault` | `1 / 1` | `2–4` | `0.90x`, `stag 10`, `sure_break: true`. |
| `bow` KS | `bow_rain` | Black-Shaft Hail | `ATTACK` (`BURST`) -> `[field, assault]` | `4 / 3` | `2–5` / `BURST` | `1.15x`, `stag 12`, `bleed` (`pot 85, 2` rnds), `uses: 1`. |
| `knife` M1 | `hamstring_cut` | Hamstring Slash | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.90x`, `stag 6`, `root` (`pot 85, 1` rnd). |
| `knife` M2 | `bleeding_cut` | Slitter's Cut | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.10x`, `stag 6`, `bleed` (`pot 85, 3` rnds). |
| `knife` M3 | `envenom` | Bile-Dipped Edge | `ATTACK` -> `assault` | `2 / 1` | `1–2` | `0.85x`, `stag 4`, `poison` (`pot 85, mag 6, 3` rnds). |
| `knife` MW | `gut_hook` | Flensing Curve | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.45x`, `stag 8`, `exploit`: `bleed 1.4x`. |
| `knife` KS | `knife_sever` | Tendon Sever | `ATTACK` -> `assault` | `3 / 2` | `1–2` | `1.80x`, `stag 10`, `root` (`pot 95, 2` rnds), `exploit`: `bleed 1.35x`, `slow 1.25x`, `uses: 1`. |
| `dagger` M1 | `quick_stab` | Curfew Thrust | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.85x`, `stag 6`. |
| `dagger` M2 | `lacerate` | Artery Flurry | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.30x`, `stag 6`, `bleed` (`pot 95, 3` rnds). Replaces `tq_dagger_flurry`. |
| `dagger` M3 | `assassinate` | Marrow-Gap Thrust | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.85x`, `stag 10`, `armor_piercing: true`, `exploit`: `stagger 1.35x`, `root 1.35x`. |
| `dagger` MW | `bodkin_twist` | Bodkin Twist | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.35x`, `stag 6`, `ignore_guard: true`, `expose -8` (`pot 85, 2` rnds). |
| `dagger` KS | `dagger_shadow` | Shadow-Gap Kill | `ATTACK` -> `maneuver` | `3 / 1` | `1` / `dash 3` | `dash_tiles: 3, position: "behind"`, `2.00x`, `armor_piercing: true`, `exploit`: `hp_below 35: 1.5x`, `uses: 1`. |

#### Families 11–15 (Censer, Bone-Saw, Scourge, Sprayer, Staff)

| Fam / Tier | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | `power_mult` / `stagger` / Effects |
| --- | --- | --- | --- | --- | --- | --- |
| `censer` M1 | `smite` | Incense Smite | `ATTACK` -> `assault` | `2 / 2` | `1–2` | `1.10x`, `stag 8`. |
| `censer` M2 | `chime_swing` | Brimstone Swing | `ATTACK` (`ARC`) -> `[field, assault]` | `2 / 2` | `1` / `ARC` | `1.05x`, `stag 10`, `burn` (`pot 75, mag 3, 2` rnds). Replaces `tq_censer_swing`. |
| `censer` M3 | `judgement` | Rite of Ashes | `ATTACK` -> `assault` | `3 / 3` | `1–3` | `1.60x`, `stag 14`, `weaken -20%` (`pot 85, 2` rnds). |
| `censer` MW | `anoint_flare` | Sanctified Flare | `ATTACK` (`RING`) -> `guard` | `3 / 2` | `1` / `RING` | `1.10x`, `stag 14`, `burn` (`pot 80, mag 4, 2` rnds). |
| `censer` KS | `censer_pyre` | Censer Pyre | `ATTACK` (`BURST`+`haz`) -> `field` | `3 / 3` | `1–3` / `BURST` | `1.45x`, `stag 14`, `burn` (`pot 90, mag 5, 3` rnds), `hazard: pitch_fire` (`2` rnds), `uses: 1`. |
| `bone_saw` M1 | `amputate_cut` | Serrated Sweep | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.10x`, `stag 6`, `bleed` (`pot 85, 3` rnds). |
| `bone_saw` M2 | `nerve_nick` | Nerve Section | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.15x`, `stag 10`, `weaken -20%` (`pot 85, 2` rnds). |
| `bone_saw` M3 | `rib_spread` | Rib-Spreader | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.55x`, `stag 12`, `expose -10` (`pot 90, 3` rnds), `exploit`: `bleed 1.3x`. |
| `bone_saw` MW | `vivisect` | Prosector Vivisect | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.65x`, `stag 10`, `armor_piercing: true`, `bleed` (`pot 95, 3` rnds), `hit_dealt` -> `-10` `strain` (`1/rnd`). |
| `bone_saw` KS | `saw_dissect` | Anatomical Sever | `ATTACK` -> `assault` | `4 / 3` | `1` | `2.05x`, `stag 18`, `armor_piercing: true`, `bleed` (`pot 100, 3` rnds), `sure_break: true`, `uses: 1`. |
| `scourge` M1 | `snap_lash` | Barbed Lash | `ATTACK` -> `assault` | `1 / 1` | `1–2` | `0.85x`, `stag 6`, `bleed` (`pot 75, 2` rnds). |
| `scourge` M2 | `hook_snatch` | Wire-Snatch | `ATTACK` -> `assault` | `2 / 2` | `1–2` | `1.00x`, `stag 12`, `forced_move: pull 1`. |
| `scourge` M3 | `flay_arc` | Sweeping Lash | `ATTACK` (`ARC`) -> `[field, assault]` | `3 / 2` | `1–2` / `ARC` | `1.15x`, `stag 10`, `bleed` (`pot 85, 2` rnds). |
| `scourge` MW | `penance_bite` | Cat-o'-Nine-Iron | `ATTACK` -> `assault` | `3 / 2` | `1–2` | `1.45x`, `stag 14`, `expose -8` (`pot 85, 2` rnds). |
| `scourge` KS | `scourge_flay` | Iron Flaying | `ATTACK` (`LINE`) -> `[field, assault]` | `3 / 3` | `1–2` / `LINE` | `1.70x`, `stag 14`, `bleed` (`pot 95, 3` rnds), `forced_move: pull 1`, `uses: 1`. |
| `sprayer` M1 | `caustic_spritz` | Vitriol Jet | `ATTACK` -> `assault` | `2 / 1` | `2–3` | `1.05x`, `stag 4`, `expose -6` (`pot 80, 2` rnds). Replaces `tq_sprayer_jet`. |
| `sprayer` M2 | `pitch_cone` | Pyre-Nozzle Cone | `ATTACK` (`CONE`) -> `[field, assault]` | `2 / 2` | `2` / `CONE` | `0.95x`, `stag 8`, `burn` (`pot 75, mag 4, 2` rnds). |
| `sprayer` M3 | `pressure_stream` | Pressure Stream | `ATTACK` (`LINE`) -> `[field, assault]` | `3 / 2` | `2–3` / `LINE` | `1.20x`, `stag 12`, `forced_move: push 1`. |
| `sprayer` MW | `flash_nozzle` | Ignition Mist | `ATTACK` (`BURST`) -> `[field, assault]` | `3 / 2` | `2–4` / `BURST` | `1.05x`, `stag 8`, `burn` (`pot 85, mag 5, 2` rnds), `exploit`: `expose 1.3x`. |
| `sprayer` KS | `sprayer_deluge` | Vitriol Deluge | `ATTACK` (`CONE`+`haz`) -> `field` | `4 / 3` | `2–3` / `CONE` | `1.55x`, `stag 14`, `burn` (`pot 90, mag 6, 3` rnds), `hazard: pitch_fire` (`2` rnds), `uses: 1`. |
| `staff` M1 | `arcane_bolt` | Marrow-Force Bolt | `ATTACK` -> `assault` | `2 / 2` | `2–3` | `1.25x`, `stag 8` (`2H` exclusive high-coefficient bolt). |
| `staff` M2 | `arc_lance` | Line Discharge | `ATTACK` (`LINE`) -> `[field, assault]` | `2 / 2` | `1` / `LINE` | `1.10x`, `stag 10`. |
| `staff` M3 | `hex` | Warding Hex | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `0.85x`, `stag 18`, `weaken -20%` (`pot 90, 2` rnds), `sure_break: true`. |
| `staff` MW | `inferno` | Conflagration | `ATTACK` (`BURST`) -> `[field, assault]` | `3 / 3` | `2–4` / `BURST` | `1.05x`, `stag 10`, `burn` (`pot 85, mag 5, 2` rnds). |
| `staff` KS | `staff_cataclysm` | Abyssal Conduit | `ATTACK` (`BURST`) -> `[field, assault]` | `4 / 4` | `2–4` / `BURST` | `1.65x`, `stag 22`, `armor_piercing: true`, `uses: 1`. |

- **Masterwork Item Seeding (`items.json`)**: Every main-hand weapon in `items.json` sets `"technique": {"skill": "<MW skill of its family>", "grip": "1h"|"2h"}` (for instance `notched_billhook`/`vanes_bill` -> `halberd_sweep`, `penal_shackles` -> `foundry_drive`, `winch_arbalest`/`stirrup_arbalest` -> `bodkin_punch`, `quells_marrow_scalpel`/`quells_bone_saws` -> `vivisect`, `brass_vitriol_sprayer`/`pyre_nozzle` -> `flash_nozzle`, etc.). Reforging any worn/plain weapon via `Berek`'s `hearth_reforge` sets `masterwork: true` and immediately opens its `4th` technique (`weapon_instances.rs`).

---

## 4. Part 4 — Channel / Graft Progression (`PK1` / `PK2`)

Per `PK1_CORRUPTION.md`, `PK2_GRAFTS.md`, and `CANON.md` §4.2 & §5.3:

- **`Channel` Rules**: `0 AP` round toggle (`x1.25` base marrow amplification on eligible attacks, `Strain` floor `[0, 5, 15, 30, 50]` by installed graft count `0..4`, `Rest -> Feed`, `Vent` at `>= 50` Strain, Rupture at `100` -> `Unchecked Spasm`, `Biological Collapse`, `Flesh Breach` `mark: "calcify"`).
- **Surgery Rules (`Chirurgeon Sabina`, post-`M2` `Slab Retorts`)**: `forearms`, `thoracic`, and `visceral` accept any Monster Ossuary Growth pin (`body_role: "monster"`); `cranial` accepts a Class Cranial Ritual (`tier: 1`) or Inoculated Keystone (`tier: 3`, `keystone: true`, `+1 Calcification`, requires `3` installed grafts + `pk2.apex_open` + `pk2.rite_<class>`).

### 4.1 Monster Ossuary-Growth Grafts (`forearms` / `thoracic` / `visceral`)

Every `CANON.md` §5.3 ossuary growth and Act 1–2 aberration drop maps cleanly into a `Slab`-refined pin (`3` slot surgery recipes each: `forearms + slot_bone_spur`, `thoracic + slot_chitin_plate`, `visceral + slot_blood_sac`):

| Graft ID & Pin Name | Tier | Material & Mob Source (`CANON.md` §5.3) | `body_effects` (Passive + `unlock`) | Channel Skill ID & Specs (`channel.only: true`) |
| --- | --- | --- | --- | --- |
| `graft_optic` (*Amber Tapetum Pin*) | `1` | `stalker_optic_nerve` (`Render-Beast`) | `stat: PERC +1`, `unlock: "marrow_lash"` | `marrow_lash` (*Marrow Lash*): `ATTACK` `assault`, `1 AP, 2 blk`, `1–2`, `1.45x`, `stag 10`, `armor_piercing: true`, **`Strain 15`**. |
| `graft_sinew` (*Black-Sinew Pin*) | `1` | `sinew_braid` (`Calcified Husk`, `Ash-Crawler`) | `resist: root +20`, `unlock: "marrow_snare"` | `marrow_snare` (*Sinew Grasp*): `ATTACK` `assault`, `2 AP, 2 blk`, `1–3`, `1.20x`, `stag 10`, `armor_piercing: true`, `pull 1`, `root` (`85, 1` rnd), **`Strain 15`**. |
| `graft_bellows` (*Ossified Bellows Pin*) | `1` | `ossified_bellows` (`Calcified Husk`, Act 2 Horrors) | `stat: max_poise +10`, `unlock: "marrow_shriek"` | `marrow_shriek` (*Bellows Shriek*): `ATTACK` (`RING`) -> `guard`, `2 AP, 2 blk`, `1 / RING`, `1.05x`, `stag 16`, `armor_piercing: true`, `push 1`, **`Strain 20`**. |
| `graft_bezoar` (*Vitriol Bezoar Pin*) | `2` | `caustic_gall_sac` (`Tallow Dredger`, `Ash-Crawler`, `Orlov`) | `resist: poison +40`, `resist: burn +40`, `unlock: "marrow_spray"` | `marrow_spray` (*Vitriol Gorge*): `ATTACK` (`CONE`) -> `[field, assault]`, `2 AP, 2 blk`, `2 / CONE`, `1.15x`, `armor_piercing: true`, `poison` (`90, mag 6, 3` rnds), `expose -6`, **`Strain 25`**. |
| `graft_valve` (*Siphon-Valve Pin*) | `2` | `inquisitor_marrow_valve` (`Proctor Quell` / Proctors) | `trigger: kill -> -10 strain`, `unlock: "marrow_bolt"` | `marrow_bolt` (*Marrow Bolt*): `ATTACK` `assault`, `2 AP, 2 blk`, `2–4`, `1.55x`, `stag 12`, `armor_piercing: true`, **`Strain 25`**. |
| `graft_core` (*Dreamer's Geode Pin*) | `3` | `titan_marrow_core` (`Calcified Amalgam`, `Living Martyrs`, Horrors) | `status_potency +8`, `crit +8`, `unlock: "marrow_rend"` (`+1 Calcification`) | `marrow_rend` (*Marrow Rend*): `ATTACK` `assault`, `2 AP, 2 blk`, `1–2`, `2.05x`, `stag 22`, `armor_piercing: true`, `ignore_guard: true`, **`Strain 35`**. |

### 4.2 6 Class Cranial Ritual Grafts (`slot: "cranial"`, `body_role: "class"`, `tier: 1`)

Installed after unlocking `pk2.slab_open` + class personal quest/rite flag `pk2.rite_<class>` (`1x stalker_optic_nerve` + `1x slot_eye_tissue` + `40 Scrap`):

| Class | Graft ID & Name | Body Passive Bonus | Ritual Skill ID & Name | Kind -> IR21 | AP / Blk | Exact Specs (`channel.only: true`) |
| --- | --- | --- | --- | --- | --- | --- |
| `warrior` | `graft_war_ritual` (*Cranial Iron-Vein*) | `max_poise +10` | `marrow_bulwark` (*Ossified Wall*) | `DEFEND` -> `guard` | `2 / 2` | `mit 0.45`, `guard_rounds: 1`, `windup_poise: 25`, `unbreakable: true`, **`Strain 20`**. |
| `brawler` | `graft_bra_ritual` (*Kiln-Cranial Spur*) | `forced_move: {bonus_tiles: 1}` | `marrow_pummel` (*Calcified Ram*) | `ATTACK` -> `assault` | `2 / 2` | `1`, `1.50x`, `stag 24`, `armor_piercing: true`, `push 2` (`collide_break: true`), **`Strain 20`**. |
| `ranger` | `graft_rng_ritual` (*Ocular Marrow-Sight*) | `stat: PERC +1` | `marrow_sight` (*Tapetum Lance*) | `ATTACK` -> `assault` | `2 / 2` | `2–6`, `1.60x`, `stag 10`, `armor_piercing: true`, `ignore_guard: true`, `expose -10` (`95, 2` rnds), **`Strain 20`**. |
| `rogue` | `graft_rog_ritual` (*Shade-Nerve Loop*) | `crit +5` | `marrow_step` (*Fade-Marrow Cut*) | `ATTACK` -> `maneuver` | `2 / 1` | `dash 3, behind`, `1.45x`, `stag 8`, `armor_piercing: true`, `bleed` (`95, 3` rnds), **`Strain 20`**. |
| `priest` | `graft_litany` (*Litany Graft*) | `status_potency +5` | `marrow_litany` (*Marrow Litany*) | `ATTACK` -> `assault` | `2 / 2` | `1–2`, `1.40x`, `stag 10`, `armor_piercing: true`, `weaken -20%` (`85, 2` rnds), **`Strain 20`**. |
| `mage` | `graft_mag_ritual` (*Retort Cortex Pin*) | `stat: INT +1` | `marrow_caustic` (*Resonant Bloom*) | `ATTACK` (`BURST`) -> `[field, assault]` | `2 / 2` | `2–4 / BURST`, `1.10x`, `stag 10`, `armor_piercing: true`, `burn` (`90, mag 5, 2` rnds), **`Strain 25`**. |

### 4.3 6 Class Inoculated Keystones (`slot: "cranial"`, `body_role: "class"`, `tier: 3`, `keystone: true`, `+1 Calcification`)

Requires `pk2.apex_open` + `pk2.rite_<class>` + **`3` installed grafts** + `1x titan_marrow_core` + `1x slot_eye_tissue` + `150 Scrap`. Occupies the deck's single `keystone` slot (mutually exclusive with Cold Steel weapon `keystone`). Note `purge_marrow` (`priest`) is deliberately callable even outside Channel (`channel.only: false`) so a Priest can emergency-calm a Stage V ally at any time (`uses_per_battle: 1`).

| Class | Graft ID & Name | Body Keystone Bonus | Skill ID & Name | Kind -> IR21 | AP / Blk | Exact Specs (`uses_per_battle: 1`) |
| --- | --- | --- | --- | --- | --- | --- |
| `warrior` | `graft_ks_war` (*Stone-Marrow Crown*) | `guarded_hit` (`limit: "round"`) -> `18` dmg + `18` stag (`"to": "other"`) | `ks_iron_marrow` (*Marrow Phalanx*) | `DEFEND` -> `guard` | `3 / 2` | `mit 0.60`, `guard_rounds: 2`, `windup_poise: 30`, `unbreakable: true`, `taunt` `2` rnds, `Strain 30`. |
| `brawler` | `graft_ks_bra` (*Spinal Kiln-Core*) | `+15%` Channel skill potency | `ks_bone_eruption` (*Ossuary Erupt*) | `ATTACK` (`RING`) -> `guard` | `3 / 2` | `1 / RING`, `2.00x`, `stag 30`, `armor_piercing: true`, `sure_break: true`, `push 2`, `Strain 35`. |
| `ranger` | `graft_ks_rng` (*Choir-Lens Keystone*) | `skill_reach` (`kind: "ATTACK", reach: 1, ranged: true`) | `ks_needle_volley` (*Marrow Needle*) | `ATTACK` (`LINE`) -> `[field, assault]` | `3 / 2` | `2–6 / LINE`, `2.15x`, `stag 18`, `armor_piercing: true`, `ignore_guard: true`, `Strain 35`. |
| `rogue` | `graft_ks_rog` (*Apex Nerve-Spire*) | `crit +10` | `ks_nerve_sever` (*Spinal Excision*) | `ATTACK` -> `assault` | `2 / 1` | `1`, `2.35x`, `armor_piercing: true`, `ignore_guard: true`, `exploit`: `bleed 1.4x`, `stagger 1.4x`, `Strain 30`. |
| `priest` | `graft_purge` (*Inoculated Keystone*) | `skill_potency: heal +0.15` | `purge_marrow` (*Purge Marrow*) | `SUPPORT` -> `[support, guard]` | `1 / 1` | `0–2`, `unmark: {mark: "calcify"}`, cleanses `["stun", "weaken"]`; **calms Stage V `hostile_all` ally -> Stage IV without aborting mission**. |
| `mage` | `graft_ks_mag` (*Abyssal Crown Pin*) | `hit_dealt` (`limit: "round"`) -> `-5 strain` | `ks_geode_nova` (*Siphon Nova*) | `ATTACK` (`BURST`) -> `[field, assault]` | `3 / 3` | `2–4 / BURST`, `1.85x`, `stag 20`, `armor_piercing: true`, `burn` (`95, mag 6, 3` rnds), `Strain 35`. |

---

## 5. Part 5 — Build Slices (`PR5-57a..c`) & `[PROVISIONAL]` Playtest Sign-Off Checklist

> **Coordinator Alignment Note**: `@design-triage` queued `PR5-57` (`GEAR-REWORK1-FOLLOWUP`), `PR5-58` (`SUNKWARD-MAPS2`), and `PR5-59` (`REGIONVIEW2`) in `docs/READY.md` Section 1. The three `CLASSPASS1` slices below (`PR5-57a..c`, or `PR5-60a..c` if sequenced after `PR5-59`) build cleanly atop `PR5-57` (`GEAR-REWORK1-FOLLOWUP`).

### 5.1 Ready-to-Queue Slice Definitions

| ID | Block / Goal | Kind | Deps | Scope & Key Deliverables (`fc.sh` gates) |
| --- | --- | --- | --- | --- |
| **`PR5-57a`** | `CLASSPASS-KIT`: dual-wield `1h_light`, orb override cap & dup checks | `kit` | `PR5-57` | (1) Add story-free `when` key `"dual_wield": "<slot_rule>"` in `rpgkit_data` / `rpgkit_rules::trigger` + passive grant condition check (`Unit::has_dual_wield`), plus optional `"ranged": bool` filter on `SkillReach` grants. (2) Add `stat_override_factor` (`0.75` default on non-matching native `scaling_stat`) in `rpgkit_rules::damage::attack_power`. (3) Extend `rpgkit_rules::unit::duplicate_skill_errors` to guard `class.basics` and `off_hand_skill` vs `class.skill_path` and `masterable_families`. (4) Retire `+level` class rank scaling lookups if any linger (`FB-265(f)`). Zero Emberwake lore words (`fc.sh layers` green). |
| **`PR5-57b`** | `CLASSPASS-DATA`: `6` decks, `7` off-hands, `75` techniques/keystones, `18` grafts via `aegis content` | `asset` | `PR5-57a` | Author idempotent seed(s) updating `classes.json` (`riveted_gauntlets` starter, flattened decks + `2` passives/class, `basics`), `item_families.json` (`shield_bash`/`riposte`/`brace`/`catalyst_pulse`, `sprayer` base power `15 -> 12` + `spell_source`, remove `wpn_orb` passive `skill_potency`, populate `15 × 3` `techniques` + `15` `keystone`s, retire `tq_*`), `skills.json` (`0` `ranks` arrays, all `24` class + `7` off-hand + `75` technique/keystone + `18` graft skills), `items.json` (`riveted_gauntlets`, `weapon.technique` Masterwork bindings, `graft_*` pins + `sinew_braid`/`ossified_bellows` drops on `units.json`), `passives.json`, `recipes.json` (`3x` Monster slot recipes + `6x2` Cranial recipes), and `sunk_ward` overlays. Re-freeze `timeline_kit.json` and re-record named goldens (`rules_golden`, `timeline_golden`, `checks_golden`, `sim_golden`, `strict_loadouts.rs`). |
| **`PR5-57c`** | `CLASSPASS-BAL`: `sim feel`, `sim party`, `sunk_ward` & Act 2 Channel verification | `logic` / `balance` | `PR5-57b` | Run `fc.sh gate`, `sim feel` (`tl_*` fixtures), `sim party` (all `6` classes at `L1–L5` verifying boss clear in `30–50%`, mage L5 boss dmg normalized from `396` into `200–250`, Rogue `dagger+dagger` `twin_light_tempo` vs `dagger+cloak` parity via `counts=on`), `world:sunk_ward/*` Act 1/2 sweep (`Channel`, Mastery `1/2/3`, Masterwork `4th`, Forge Rite Keystones, `purge_marrow` Stage V calm test). |

### 5.2 `[PROVISIONAL]` Playtest Review Checklist (User Rule 5)

- [ ] **Rogue Dual-Wield vs. Cloak Trade-off**: Confirm sacrificing `ash_cloak` (`Fade-Step`, `+1 PERC`) for two `1h_light` weapons + `twin_light_tempo` (`0.75x` Bleed `offhand_jab`, `+1 AP/rnd`, `+8%` crit) feels crisp at `M1–M4` without eclipsing `Backflip`/`Fade-Step` survival.
- [ ] **Orb vs. 2H Staff & `FB-268(a)` Boss Target**: Confirm `sprayer (12)` + `wpn_orb` (`+1` reach, `2 AP` `catalyst_pulse` `+25%` Focus) differentiates cleanly below `2H staff (24)` burst and holds `sim party` boss win in `30–50%`.
- [ ] **Off-Hand Signatures (`shield`/`buckler`/`braces`)**: Confirm moving `shield_bash`, `riposte`, and `brace` exclusively to off-hand slots gives Warriors and Brawlers clear shield/buckler/braces gear identity with zero dead class slots.
- [ ] **Mastery Pace (`100 / 300 / 600` XP) & Masterwork `4th` Value**: Confirm unlocking M1/M2/M3 across Act 2 (`10` use XP, `150%` on `Prof 3`, `Berek` drills & books) feels rewarding without starving `secondary`/`third` swaps (`Berek` `train_secondary`/`third`).
- [ ] **Cranial Ritual & Stage V `purge_marrow`**: Confirm `1/battle` `purge_marrow` (`unmark: calcify`) rescues Stage V `hostile_all` crises meaningfully while still enforcing high tension around Tier III Calcification costs.
