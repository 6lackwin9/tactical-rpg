# `U2_CLASSPASS_TECHNIQUES` — `[PROVISIONAL]` Progression Spec Index, Off-Hand Signatures & Build Slices (`CLASSPASS1`)

- **Status**: **`[PROVISIONAL]`** — Ready for user playtest sign-off per User Rule 5.
- **Baseline**: `origin/develop` (`a14c9d8e`) + `@design-triage` queued precursors `PR5-57..61` (`U3_GEAR_REWORK.md` §7).
- **Domain Siblings** (split `< 25 KB / < 300 lines` each):
  - [U2_CLASS_DECKS.md](U2_CLASS_DECKS.md) — Part 1: `6` Class Decks (`CLASSPASS1`), passives, `twin_light_tempo`, `47` retired `ranks` dispositions
  - [U2_WEAPON_TECHNIQUES.md](U2_WEAPON_TECHNIQUES.md) — Part 3: `15` Main-Hand Weapon Families × `5` skills (`3` Mastery + `1` Masterwork `4th` + `1` Cold Steel Keystone = `75` skills)
  - [U2_GRAFTS_SKILLS.md](U2_GRAFTS_SKILLS.md) — Part 4: `PK1`/`PK2` Monster Ossuary Growths (`6` Tier I–III), `6` Class Cranial Rituals, `6` Class Inoculated Keystones
- **Inputs & Gates**: `docs/PRINCIPLES.md` (`P-ARCH-01..06`, `P-SYS-04`), `U3_GEAR_REWORK.md`, `PK1_CORRUPTION.md`, `PK2_GRAFTS.md`, `CANON.md` (§4.2, §5.3, §6.1), `docs/FEEDBACK.md` (`FB-231(b)`, `FB-265(f)`, `FB-267(3)`, `FB-268(a-c)`).
- **Naming & Validator Guard**: Every skill ID <= `24` chars (`[a-z0-9_]`), display name <= `24` chars, IR21 intents derived from `rpgkit_rules::intent::derive`, areas strictly within `NONE/PLUS/BURST/LINE/CONE/RING/ARC`, and statuses strictly within `statuses.json`.

---

## 0. Architectural Invariants

1. **Zero Skill Duplication Across Lanes (`duplicate_skill_errors`)**: Every skill ID belongs to **one** bucket only: Class Basic (`basics`), Class Deck (`skill_path`), Off-Hand Button (`off_hand_skill`), Main-Hand Family (`techniques` / `weapon.technique` / `keystone`), or Slab Graft (`cranial` / `growth`). Class skills use `"weapon_types": []` so switching `main`/`secondary`/`third` weapons preserves full class identity (`P-ARCH-01`).
2. **Flat Single-Rank Skills (`FB-231(b)`, `FB-265(f)`)**: All **`47`** `ranks` arrays in `catalog/skills.json` are eliminated. Classes unlock whole `L1/L2/L4` identity skills; combat scaling shifts to **Mastery `1/2/3`**, **Masterwork Reforge (`4th`)**, **Cold Steel Keystones**, and **Slab Channel Grafts**.
3. **Kit vs. Game Separation (`fc.sh layers`)**: All `PR5-62a` code hooks remain story-free with zero `games/emberwake/lore_words.txt` proper nouns; Emberwake names and `CANON.md` §5.3 terms stay in data/world overlays.

---

## 1. Part 2 — 7 Off-Hand Signature Skills (`U3_GEAR_REWORK` §4, `FB-267(3)`) & `FB-268(a)` Orb Fix

### 1.1 Exact Specs for All 7 `off_only` Family Skills

Equipping an off-hand item grants its family's `off_hand_skill` (`Unit::gear_skills`) on the battle Off-Hand Button (`offhand_use`). `1h_light` off-hand weapons (`dagger`, `knife`, `fist`) instead unlock `offhand_jab`.

| Family (`slot_rule: off_only`) | Skill ID & Name | Kind -> IR21 | AP / Blk | Reach | Exact Specs (`ranks: []`, `weapon_types: ["<family>"]`, `min_proficiency: 1`) |
| --- | --- | --- | --- | --- | --- |
| **`shield`** (`mit 6, bur 2`) | `shield_bash` (*Shield Bash*) | `ATTACK` -> `assault` | `2 / 2` | `1` | `0.75x` power (`weapon_hand: ""` scales STR main-hand), `stagger: 16`, `stagger` status (`pot 75, 1` rnd), `forced_move: {kind: "push", tiles: 1, collide_mult: 0.5, collide_stagger: 8}`. Removed from `warrior`. |
| **`buckler`** (`mit 3, bur 0`) | `riposte` (*Counter-Guard*) | `DEFEND` -> `guard` | `2 / 1` | `self` | `mitigation: 0.30`, `guard_rounds: 1`, `grant_status: "riposte"`, `status_rounds: 2`, NO `ends_in_stance`, NO `windup_poise`. Removed from `warrior`. |
| **`braces`** (`mit 4, bur 1`) | `brace` (*Iron Brace*) | `DEFEND` -> `guard` | `1 / 1` | `self` | `mitigation: 0.45`, `guard_rounds: 1`, `grant_status: "fortify"`, `status_magnitude: 6`, `status_rounds: 1` (NO `windup_poise`: disallowed on `DEFEND`). Removed from `warrior`. |
| **`ammo`** | `reload` (*Press-Reload*) | `FOCUS` -> `guard` | `1 / 1` | `self` | `boost: 0.35`, `boost_rounds: 2`, `ends_in_stance: "OFFENSIVE"`; bolts/arrows stay `1 AP` free-flow reload + shot empower. |
| **`cloak`** | `fade_step` (*Fade-Step*) | `DISENGAGE` -> `maneuver` | `1 / 1` | `jump 2` | `jump_tiles: 2` away from nearest enemy (NO `ends_in_stance`: `DISENGAGE` lacks `flow`). Paired with `ash_cloak` `+1 PERC`. |
| **`orb`** (`INT >= 12`) | **`catalyst_pulse`** (*Catalyst Pulse*) | `FOCUS` -> `guard` | `2 / 1` | `self` | `boost: 0.25` (`+25%` next attack/spell damage), `boost_rounds: 2`, `grant_status: "haste"`, `status_rounds: 1`. Replaces unconditional orb `+10%` passive damage. |
| **`tome`** (`KNOW >= 12`) | `cite_passage` (*Cite Passage*) | `SUPPORT` -> `support` | `2 / 2` | `1–3` | **`target_mode: "ALLY"`**, heals `10 + 1.0 * KNOW`, cleanses `["bleed", "burn", "poison"]`, `grant_status: "fortify"`, `status_magnitude: 6`, `status_rounds: 2`. |

> **Validator Alignment**: Removing `windup_poise` from `brace`/`riposte` (`DEFEND` lacks `timing`) and `ends_in_stance` from `fade_step` (`DISENGAGE` lacks `flow`) passes `check/skill.rs:56, 59, 769-770` cleanly without widening engine validators. `cite_passage` explicitly sets `target_mode: "ALLY"` (single-intent `support` at range `1–3`).

### 1.2 Root-Cause & 3-Part Fix for `FB-268(a)` (`mage` L5 Boss Spike)

**Root-Cause Finding (`30415b91` + `damage.rs:157-170` + `source.rs:703-712`)**: In `PR5-51b`, `mage` L5 boss damage spiked `214 -> 396` (`50% -> 80%` win) from three compounding interactions:

1. Giving `mage` `wpn_orb` added unconditional `+10% spell potency` AND `+1 SPELL reach` while `arcane_bolt`/`firebomb`/`glacial_burst` still carried `["staff", "sprayer"]` `weapon_types` (`source::mark_spell_sources` marks `spell_source = true` only when `weapon_types` matches a `spell_source` family).
2. **`sprayer` + `orb` Double-Dip**: `brass_vitriol_sprayer` (`Prof 3` `1.25x` on power `15 -> 19`) was tuned for `1H` raw damage alongside `wpn_orb`'s unconditional passive `+10%` and Turn-1 reach.
3. **`orb.power_stat_override: "INT"` (`damage.rs:160-166`)**: Reassigns physical weapon power to `INT` at full `1.0x` uncapped ratio.

**Complete 3-Step Fix (`PR5-57` -> `PR5-62a`/`PR5-62b`)**:

1. **Orb Focus & Field Reach (`PR5-62b` refining `PR5-57`)**: Remove unconditional `skill_potency: {group: "spell", value: 0.1}` from `wpn_orb`; set `wpn_orb`'s reach passive to `{"type": "skill_reach", "reach": 1, "intent": "field"}` so it aids class `field` bombs (`weapon_types: []` makes `spell_source = false` on class spells per `source.rs:703-712`) plus `sprayer`/`staff` `field` skills, while moving orb burst into `2 AP` `catalyst_pulse` (`+25%` Focus). Lower `sprayer` `base.main_1h.power` from **`15 -> 12`** (`bands: 8..20`, `Prof 3` = `15`), preserving `2H staff`'s `24` raw power lead.
2. **Kit `stat_override_factor` (`PR5-62a`)**: Add `stat_override_factor: f64` (`0.75` default when `orb.power_stat_override` overrides a weapon whose native `scaling_stat != override`) in `rpgkit_rules::damage::attack_power` so Mage `orb + mace/dagger` hybrids stay viable without overtaking pure Martial weapon scaling or `staff` nukes.
3. **Class Deck Flatten (`PR5-62b`)**: Flattening L5 rank-3 spell multipliers (`firebomb`/`glacial_burst` `0.90x -> 0.50x`, moving `arcane_bolt` `1.65x` into `staff` M1 at `1.25x`) eliminates the L5 scalar jump and restores `sim party` boss clear into the **`30–50%`** (`200–250` L5 boss dmg) target.

---

## 2. Part 5 — Ready-to-Queue Build Slices (`PR5-62a..c`) & `[PROVISIONAL]` Review Checklist

> **Queue Alignment (`@coordinator` #19)**: `PR5-57..61` (`GEAR-REWORK1-FOLLOWUP`, `SUNKWARD-MAPS2`, `REGIONVIEW2`, `RIG-GEAR1`, `STORYMUSIC-WIRE1`) are already allocated in `docs/READY.md`. The slices below are numbered **`PR5-62a..c`**; **`PR5-62b`** explicitly builds atop `PR5-57`'s interim `wpn_orb` / `catalyst_pulse` / boss-tuning work.

### 2.1 Ready-to-Queue Slice Definitions

| ID | Block / Goal | Kind | Deps | Scope & Key Deliverables (`fc.sh` gates) |
| --- | --- | --- | --- | --- |
| **`PR5-62a`** | `CLASSPASS1-KIT`: passives, `dual_wield`, `crit`, `channel` group, `ranged` reach & dup guard | `kit` | `PR5-57` | (1) Add `ClassDef::passives: Vec<String>` (`rpgkit_data/src/lib.rs`, `check/class.rs`, derived-unit deck hook in `rpgkit_rules::unit`). (2) Add story-free `when` key `"dual_wield": "<slot_rule>"` (`Unit::has_dual_wield`) + conditional `when` on passive grant rows. (3) Add `"crit"` to `simple::TYPES` and wire non-gear `Target::Crit` in `rpgkit_rules::unit`. (4) Add `"channel"` to `simple::GROUPS` matching `skill.channel.only` in `unit::skill_in_group`. (5) Add optional `"ranged": bool` to `SKILL_REACH_KEYS` / `SkillReach` (`check/simple.rs`, `grants.rs`, `unit::skill_reach_matches`). (6) Add `stat_override_factor` (`0.75` default) in `damage::attack_power`. (7) Extend `unit::duplicate_skill_errors` across `class.basics` and `off_hand_skill` vs `class.skill_path` & `masterable_families`. Zero lore words (`fc.sh layers` green). |
| **`PR5-62b`** | `CLASSPASS1-DATA`: `6` class decks, `7` off-hands, `47` flattened ranks & archetype loadouts | `asset` + `balance` | `PR5-57`, `PR5-62a` | Builds atop `PR5-57`'s interim `wpn_orb`/`catalyst_pulse`/boss pass via idempotent `aegis content` seed: update `classes.json` (`riveted_gauntlets` starter, `1` basic + `3` flat actives + `2` `passives`/class), `item_families.json` (`shield_bash`/`riposte`/`brace`/`catalyst_pulse`, `sprayer` base power `15 -> 12`), `items.json` (`riveted_gauntlets`, `wpn_orb` drop `skill_potency` + use `skill_reach {intent: "field", reach: 1}`), `skills.json` (remove all `47` `ranks` arrays, finalize `24` class + `7` off-hand skills), `passives.json` (`12` class passives incl. `twin_light_tempo`), and `units.json` archetype `loadout` arrays (`warrior`, `brawler`, `ranger`, `rogue`, `priest`, `mage`, `mara_voss` + `strict_loadouts.rs`). Re-freeze `timeline_kit.json`, re-record named goldens cleanly, verify `fc.sh gate`, `sim feel`, `sim party` (`30–50%` boss, Mage L5 `200–250`, Rogue `dual_wield` parity via `counts=on`). |
| **`PR5-62c`** | `TECHNIQUES-GRAFTS-DATA`: `75` weapon skills, `18` `PK1/PK2` grafts & Act 1/2 sweep | `asset` + `balance` | `PR5-62b` | Author idempotent `aegis content` seed populating all `15 × (3 Mastery + 1 MW + 1 KS)` = `75` single-rank skills (retiring `6` `tq_*` stubs), `items.json` `weapon.technique` Masterwork bindings + Act 2 durability hook, and Part 4 (`6` Monster Ossuary Growth pins I–III + `6` Class Cranial Rituals + `6` Class Inoculated Keystones incl. `graft_purge`/`purge_marrow`, `recipes.json` surgery recipes, `sinew_braid`/`ossified_bellows` drops on `units.json`, `sunk_ward` overlays). Verify `fc.sh gate`, `sim feel`, `sim party`, and `world:sunk_ward/*` Act 1/2 sweep (`Channel`, Mastery `1/2/3`, Masterwork `4th`, Forge Rites, `purge_marrow` Stage V calm rescue). |

### 2.2 `[PROVISIONAL]` Playtest Review Checklist (User Rule 5)

- [ ] **Rogue Dual-Wield vs. Cloak Trade-off**: Confirm trading `ash_cloak` (`Fade-Step`, `+1 PERC`) for two `1h_light` blades + `twin_light_tempo` (`0.75x` Bleed `offhand_jab`, `+1 AP/rnd`, `+8%` crit) feels rewarding across `M1–M4` without crowding out `Backflip` survival.
- [ ] **Orb vs. 2H Staff & `FB-268(a)` Boss Target**: Confirm `sprayer (12)` + `wpn_orb` (`+1` `field` reach, `2 AP` `catalyst_pulse` `+25%` Focus) sits cleanly below `2H staff (24)` nukes and holds `sim party` L5 boss wins inside `30–50%`.
- [ ] **Off-Hand Signatures (`shield`/`buckler`/`braces`)**: Confirm moving `shield_bash`, `riposte`, and `brace` exclusively to off-hand slots sharpens Martial gear decisions with zero dead Warrior class slots.
- [ ] **Mastery Pace (`100 / 300 / 600` XP) & Masterwork `4th`**: Confirm unlocking `M1/M2/M3` in Act 2 (`10` use XP, `150%` on `Prof 3`, `Berek` drills/books) paces cleanly alongside `Berek`'s `train_secondary`/`third` swaps.
- [ ] **Cranial Ritual & Stage V `purge_marrow`**: Confirm `1/battle` `purge_marrow` (`unmark: calcify`, `channel.none: true`) rescues Stage V `hostile_all` crises meaningfully while keeping Tier III Calcification consequential.
