# `U2_WEAPON_TECHNIQUES` — `[PROVISIONAL]` 15 Main-Hand Weapon Families × 5 Skills (`75` Skills)

- **Parent Index**: [U2_CLASSPASS_TECHNIQUES.md](U2_CLASSPASS_TECHNIQUES.md) (`[PROVISIONAL]` User Rule 5).
- **Build Target**: `PR5-62c` (`TECHNIQUES-GRAFTS-DATA`).

---

## 1. Architecture & Shared Rules (`PK2_GRAFTS.md` §5, `check/item.rs:522`)

- **5 Skills per Family (`15 × 5 = 75`)**: `3` Mastery Techniques (`techniques: [M1@100, M2@300, M3@600]` XP) + `1` **Masterwork Reforge Technique** (`weapon.technique` opened when `masterwork: true` at Hearth `hearth_reforge`) + `1` **Cold Steel Keystone** (`family.keystone: "<skill_id>"`, `SkillDef` in `skills.json` with `uses_per_battle: 1`, mutually exclusive with a Cranial Inoculated Keystone).
- **Zero Duplicate Skills**: All `6` `tq_*` stubs (`tq_polearm_hook`, `tq_crossbow_volley`, `tq_fist_breaker`, `tq_dagger_flurry`, `tq_censer_swing`, `tq_sprayer_jet`) are retired cleanly after remapping `item_families.json`.
- **Single-Rank & Prof-1 Technique Gate**: Every technique sets `"weapon_types": ["<family>"]`, `"min_proficiency": 1`, `"learned_only": true`, `"ranks": []` (access gated by Mastery / Masterwork / Cold Steel Keystone while equipping `secondary` / `third` families stays smooth). `LINE`/`ARC`/`CONE`/`RING` keep `reach_max: 1` (`check/skill.rs:871`), `DEFEND` omits `windup_poise`/`unbreakable` (`check/skill.rs:56`), and `position` skills set `reach_min: 0` + `dash_tiles: 0` (`check/skill.rs:1068, 1089`).
- **Act 2 Durability Hook**: `1` durability loss per basic attack, `2` per Technique, `0` = `dulled` (`0.5x` power).

---

## 2. Overview Matrix Across All 15 Families

| # | Family (`slot_rule`) | Primary Stat / Reach | M1 (`Mastery 1`) | M2 (`Mastery 2`) | M3 (`Mastery 3`) | MW (`Masterwork 4th`) | KS (`Cold Steel Keystone`) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `polearm` (`1h_heavy`) | `STR` / `1–2` | `hook_drag` | `pike_skew` | `check_advance` | `halberd_sweep` | `polearm_cordon` |
| 2 | `greatsword` (`2h_solo`) | `STR` / `1` | `sunder` | `wide_cleave` | `rending_arc` | `gallow_sweep` | `gsword_sever` |
| 3 | `sword` (`1h_heavy`) | `STR` / `1` | `pommel_check` | `rend_armor` | `bind_parry` | `hookbill_draw` | `sword_duelist` |
| 4 | `hammer` (`2h_solo`) | `STR` / `1` | `crushing_blow` | `earth_shaker` | `skull_ring` | `slag_driver` | `hammer_shatter` |
| 5 | `gauntlet` (`1h_heavy`) | `STR` / `1` | `knuckle_jolt` | `cyclone_kick` | `haymaker` | `foundry_drive` | `gauntlet_breaker` |
| 6 | `fist` (`1h_light`) | `STR` / `1` | `body_blow` | `palm_check` | `liver_hook` | `rapid_combo` | `fist_tempest` |
| 7 | `crossbow` (`2h_rig`) | `DEX` / `2–4` | `aimed_shot` | `crippling_shot` | `deadeye` | `bodkin_punch` | `crossbow_siege` |
| 8 | `bow` (`2h_rig`) | `DEX` / `2–4` | `pinning_shot` | `barbed_arrow` | `volley` | `wing_shot` | `bow_rain` |
| 9 | `knife` (`1h_light`) | `DEX` / `1–2` | `hamstring_cut` | `bleeding_cut` | `envenom` | `gut_hook` | `knife_sever` |
| 10 | `dagger` (`1h_light`) | `DEX` / `1` | `quick_stab` | `lacerate` | `assassinate` | `bodkin_twist` | `dagger_shadow` |
| 11 | `censer` (`1h_heavy`) | `INT/STR` / `1` | `smite` | `chime_swing` | `judgement` | `anoint_flare` | `censer_pyre` |
| 12 | `bone_saw` (`1h_heavy`) | `DEX/STR` / `1` | `amputate_cut` | `nerve_nick` | `rib_spread` | `vivisect` | `saw_dissect` |
| 13 | `scourge` (`1h_heavy`) | `DEX/STR` / `1–2` | `snap_lash` | `hook_snatch` | `flay_arc` | `penance_bite` | `scourge_flay` |
| 14 | `sprayer` (`1h_heavy`) | `INT` / `2–3` | `caustic_spritz` | `pitch_cone` | `pressure_stream` | `flash_nozzle` | `sprayer_deluge` |
| 15 | `staff` (`2h_solo`) | `INT` / `2` | `arcane_bolt` | `arc_lance` | `hex` | `inferno` | `staff_cataclysm` |

---

## 3. Exact Skill Definitions (`75` Single-Rank Skills)

### Families 1–5 (`polearm`, `greatsword`, `sword`, `hammer`, `gauntlet`)

| Fam / Tier | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | `power_mult` / `stagger` / Effects |
| --- | --- | --- | --- | --- | --- | --- |
| `polearm` M1 | `hook_drag` | Billhook Drag | `ATTACK` -> `assault` | `2 / 2` | `1–2` | `1.05x`, `stag 10`, `forced_move: {kind: "pull", tiles: 1}`. Replaces `tq_polearm_hook`. |
| `polearm` M2 | `pike_skew` | Skewer Thrust | `ATTACK` (`LINE`) -> `[field, assault]` | `2 / 2` | `1` / `LINE` | `0.95x`, `stag 8`, `armor_piercing: true`. |
| `polearm` M3 | `check_advance` | Halt Advance | `ATTACK` -> `assault` | `3 / 3` | `1–2` | `1.45x`, `stag 18`, `root` (`pot 85, 1` rnd), `sure_break: true`. |
| `polearm` MW | `halberd_sweep` | Crescent Halberd | `ATTACK` (`ARC`) -> `[field, assault]` | `3 / 2` | `1` / `ARC` | `1.30x`, `stag 14`, `forced_move: {kind: "push", tiles: 1}` (`reach_max: 1` `ARC`). |
| `polearm` KS | `polearm_cordon` | Castellan Cordon | `DEFEND` -> `guard` | `3 / 3` | `self` | `mit 0.40`, `guard_rounds: 2`, `grant_status: "steadfast"`, `1` rnd, `triggers`: `ally_hit` (`range: 2, melee: true, limit: "round"`) -> `{"type": "redirect", "pct": 100}` + `{"type": "move", "kind": "pull", "tiles": 1, "to": "other"}`, `uses: 1`. |
| `greatsword` M1 | `sunder` | Sunder Chop | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.30x`, `stag 12`, `expose -6` (`pot 75, 2` rnds). |
| `greatsword` M2 | `wide_cleave` | Broad Cleave | `ATTACK` (`ARC`) -> `[field, assault]` | `3 / 2` | `1` / `ARC` | `1.15x`, `stag 10`. |
| `greatsword` M3 | `rending_arc` | Armor-Splitter | `ATTACK` -> `assault` | `3 / 3` | `1` | `1.70x`, `stag 14`, `expose -10` (`pot 85, 3` rnds), `windup_poise: 12`. |
| `greatsword` MW | `gallow_sweep` | Gallows Sweep | `ATTACK` (`ARC`) -> `[field, assault]` | `4 / 3` | `1` / `ARC` | `1.50x`, `stag 16`, `exploit`: `stagger 1.35x`, `expose 1.25x`. |
| `greatsword` KS | `gsword_sever` | Steel Severance | `ATTACK` -> `assault` | `4 / 4` | `1` | `2.30x`, `stag 22`, `unbreakable: true`, `armor_piercing: true`, `uses: 1`. |
| `sword` M1 | `pommel_check` | Pommel Check | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.70x`, `stag 12`, `sure_break: true`. |
| `sword` M2 | `rend_armor` | Half-Sword Thrust | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.15x`, `stag 8`, `expose -8` (`pot 80, 2` rnds). |
| `sword` M3 | `bind_parry` | Bind & Counter | `DEFEND` -> `guard` | `2 / 2` | `self` | `mit 0.30`, `guard_rounds: 1`, `grant_status: "riposte"`, `status_rounds: 2` (NO `ends_in_stance`, NO `windup_poise`). |
| `sword` MW | `hookbill_draw` | Hookbill Draw | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.25x`, `stag 8`, `bleed` (`pot 80, 2` rnds), `forced_move: {kind: "pull", tiles: 1}`. |
| `sword` KS | `sword_duelist` | Master-Cut | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.95x`, `stag 16`, `ignore_guard: true`, `ends_in_stance: "DEFENSIVE"`, `uses: 1`. |
| `hammer` M1 | `crushing_blow` | Maul Smash | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.25x`, `stag 16`, `disrupt: "STANCE_BREAK"`. |
| `hammer` M2 | `earth_shaker` | Kiln Shockwave | `ATTACK` (`LINE`) -> `[field, assault]` | `3 / 3` | `1` / `LINE` | `1.15x`, `stag 18`, `forced_move: {kind: "push", tiles: 1}`. |
| `hammer` M3 | `skull_ring` | Bell-Ringer | `ATTACK` -> `assault` | `3 / 3` | `1` | `1.75x`, `stag 24`, `stun` (`pot 65, 1` rnd), `windup_poise: 15`. |
| `hammer` MW | `slag_driver` | Piston Slam | `ATTACK` -> `assault` | `4 / 3` | `1` | `1.90x`, `stag 20`, `forced_move: {kind: "push", tiles: 2, collide_mult: 0.8, collide_stagger: 12}`. |
| `hammer` KS | `hammer_shatter` | Sledge Judgment | `ATTACK` -> `assault` | `4 / 4` | `1` | `2.20x`, `stag 30`, `unbreakable: true`, `sure_break: true`, `forced_move: {kind: "push", tiles: 2}`, `uses: 1`. |
| `gauntlet` M1 | `knuckle_jolt` | Shackle Jolt | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.85x`, `stag 14`, `forced_move: {kind: "push", tiles: 1}`. |
| `gauntlet` M2 | `cyclone_kick` | Foundry Spin | `ATTACK` (`RING`) -> `guard` | `3 / 2` | `1` / `RING` | `1.00x`, `stag 14` (hits every adjacent enemy). |
| `gauntlet` M3 | `haymaker` | Haymaker | `ATTACK` -> `assault` | `3 / 3` | `1` | `1.85x`, `stag 20`, `stun` (`pot 60, 1` rnd), `windup_poise: 10`. |
| `gauntlet` MW | `foundry_drive` | Cestus Throw | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.30x`, `stag 16`, `forced_move: {kind: "push", tiles: 2, collide_mult: 1.0, collide_break: true}`. |
| `gauntlet` KS | `gauntlet_breaker` | Anvil-Fist | `ATTACK` -> `assault` | `4 / 3` | `1` | `2.10x`, `stag 28`, `unbreakable: true`, `ignore_guard: true`, `forced_move: {kind: "push", tiles: 2, collide_break: true}`, `uses: 1`. |

### Families 6–10 (`fist`, `crossbow`, `bow`, `knife`, `dagger`)

| Fam / Tier | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | `power_mult` / `stagger` / Effects |
| --- | --- | --- | --- | --- | --- | --- |
| `fist` M1 | `body_blow` | Rib Shot | `ATTACK` -> `assault` | `1 / 1` | `1` | `0.90x`, `stag 14`, `weaken -15%` (`pot 75, 2` rnds). Replaces `tq_fist_breaker`. |
| `fist` M2 | `palm_check` | Palm Thrust | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.05x`, `stag 16`, `sure_break: true`. |
| `fist` M3 | `liver_hook` | Solar-Plexus Hook | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.45x`, `stag 22`, `stun` (`pot 55, 1` rnd). |
| `fist` MW | `rapid_combo` | Four-Knuckle Combo | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.55x`, `stag 16`, `exploit`: `stagger 1.35x`. |
| `fist` KS | `fist_tempest` | Chain-Breaker | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.90x`, `stag 25`, `sure_break: true`, `riders`: `[{"when": {}, "effects": [{"type": "ap", "value": 1, "to": "self"}]}]`, `uses: 1`. |
| `crossbow` M1 | `aimed_shot` | Winch Bolt | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `1.45x`, `stag 8`. |
| `crossbow` M2 | `crippling_shot` | Tendon Bolt | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `1.05x`, `stag 8`, `slow` (`pot 85, 2` rnds). |
| `crossbow` M3 | `deadeye` | Deadeye Quarrel | `ATTACK` -> `assault` | `3 / 3` | `2–5` | `1.95x`, `stag 14`, `sure_break: true`. Replaces `tq_crossbow_volley`. |
| `crossbow` MW | `bodkin_punch` | Bodkin Punch | `ATTACK` -> `assault` | `3 / 2` | `2–4` | `1.55x`, `stag 10`, `armor_piercing: true`, `ignore_guard: true`. |
| `crossbow` KS | `crossbow_siege` | Siege-Winch Shot | `ATTACK` -> `assault` | `4 / 4` | `2–6` | `2.25x`, `stag 18`, `armor_piercing: true`, `sure_break: true`, `forced_move: {kind: "push", tiles: 2}`, `uses: 1`. |
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
| `dagger` KS | `dagger_shadow` | Shadow-Gap Kill | `ATTACK` -> `maneuver` | `3 / 1` | `reach_max: 3` / `behind` | `reach_min: 0, reach_max: 3, dash_tiles: 0, position: "behind"`, `2.00x`, `armor_piercing: true`, `exploit`: `{hp_below: 35, power_mult: 1.5}`, `uses: 1`. |

### Families 11–15 (`censer`, `bone_saw`, `scourge`, `sprayer`, `staff`)

| Fam / Tier | Skill ID | Name | Kind -> IR21 | AP / Blk | Reach / Area | `power_mult` / `stagger` / Effects |
| --- | --- | --- | --- | --- | --- | --- |
| `censer` M1 | `smite` | Incense Smite | `ATTACK` -> `assault` | `2 / 2` | `1–2` | `1.10x`, `stag 8`. |
| `censer` M2 | `chime_swing` | Brimstone Swing | `ATTACK` (`ARC`) -> `[field, assault]` | `2 / 2` | `1` / `ARC` | `1.05x`, `stag 10`, `burn` (`pot 75, mag 3, 2` rnds). Replaces `tq_censer_swing`. |
| `censer` M3 | `judgement` | Rite of Ashes | `ATTACK` -> `assault` | `3 / 3` | `1–3` | `1.60x`, `stag 14`, `weaken -20%` (`pot 85, 2` rnds). |
| `censer` MW | `anoint_flare` | Sanctified Flare | `ATTACK` (`RING`) -> `guard` | `3 / 2` | `1` / `RING` | `1.10x`, `stag 14`, `burn` (`pot 80, mag 4, 2` rnds). |
| `censer` KS | `censer_pyre` | Censer Pyre | `ATTACK` (`BURST`+`haz`) -> `field` | `3 / 3` | `1–3` / `BURST` | `1.45x`, `stag 14`, `burn` (`pot 90, mag 5, 3` rnds), `hazard: {id: "pitch_fire", shape: "SKILL", at: "target", rounds: 2}`, `uses: 1`. |
| `bone_saw` M1 | `amputate_cut` | Serrated Sweep | `ATTACK` -> `assault` | `2 / 1` | `1` | `1.10x`, `stag 6`, `bleed` (`pot 85, 3` rnds). |
| `bone_saw` M2 | `nerve_nick` | Nerve Section | `ATTACK` -> `assault` | `2 / 2` | `1` | `1.15x`, `stag 10`, `weaken -20%` (`pot 85, 2` rnds). |
| `bone_saw` M3 | `rib_spread` | Rib-Spreader | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.55x`, `stag 12`, `expose -10` (`pot 90, 3` rnds), `exploit`: `bleed 1.3x`. |
| `bone_saw` MW | `vivisect` | Prosector Vivisect | `ATTACK` -> `assault` | `3 / 2` | `1` | `1.65x`, `stag 10`, `armor_piercing: true`, `bleed` (`pot 95, 3` rnds), `riders`: `[{"when": {"meter": {"id": "strain", "at_least": 1}}, "effects": [{"type": "meter", "meter": "strain", "delta": -10, "to": "self"}]}]`. |
| `bone_saw` KS | `saw_dissect` | Anatomical Sever | `ATTACK` -> `assault` | `4 / 3` | `1` | `2.05x`, `stag 18`, `armor_piercing: true`, `bleed` (`pot 100, 3` rnds), `sure_break: true`, `uses: 1`. |
| `scourge` M1 | `snap_lash` | Barbed Lash | `ATTACK` -> `assault` | `1 / 1` | `1–2` | `0.85x`, `stag 6`, `bleed` (`pot 75, 2` rnds). |
| `scourge` M2 | `hook_snatch` | Wire-Snatch | `ATTACK` -> `assault` | `2 / 2` | `1–2` | `1.00x`, `stag 12`, `forced_move: {kind: "pull", tiles: 1}`. |
| `scourge` M3 | `flay_arc` | Sweeping Lash | `ATTACK` (`ARC`) -> `[field, assault]` | `3 / 2` | `1` / `ARC` | `1.25x`, `stag 10`, `bleed` (`pot 85, 2` rnds) (`reach_max: 1` `ARC`). |
| `scourge` MW | `penance_bite` | Cat-o'-Nine-Iron | `ATTACK` -> `assault` | `3 / 2` | `1–2` | `1.45x`, `stag 14`, `expose -8` (`pot 85, 2` rnds). |
| `scourge` KS | `scourge_flay` | Iron Flaying | `ATTACK` (`LINE`) -> `[field, assault]` | `3 / 3` | `1` / `LINE` | `1.70x`, `stag 14`, `bleed` (`pot 95, 3` rnds), `forced_move: {kind: "pull", tiles: 1}`, `uses: 1` (`reach_max: 1` `LINE`). |
| `sprayer` M1 | `caustic_spritz` | Vitriol Jet | `ATTACK` -> `assault` | `2 / 1` | `2–3` | `1.05x`, `stag 4`, `expose -6` (`pot 80, 2` rnds). Replaces `tq_sprayer_jet`. |
| `sprayer` M2 | `pitch_cone` | Pyre-Nozzle Cone | `ATTACK` (`CONE`) -> `[field, assault]` | `2 / 2` | `1` / `CONE` | `1.05x`, `stag 8`, `burn` (`pot 75, mag 4, 2` rnds) (`reach_max: 1` self-origin `CONE`). |
| `sprayer` M3 | `pressure_stream` | Pressure Stream | `ATTACK` -> `assault` | `3 / 2` | `2–3` | `1.35x`, `stag 16`, `sure_break: true`, `forced_move: {kind: "push", tiles: 2}` (`area: "NONE"` ranged jet). |
| `sprayer` MW | `flash_nozzle` | Ignition Mist | `ATTACK` (`BURST`) -> `[field, assault]` | `3 / 2` | `2–4` / `BURST` | `1.05x`, `stag 8`, `burn` (`pot 85, mag 5, 2` rnds), `exploit`: `expose 1.3x`. |
| `sprayer` KS | `sprayer_deluge` | Vitriol Deluge | `ATTACK` (`BURST`+`haz`) -> `field` | `4 / 3` | `2–3` / `BURST` | `1.55x`, `stag 14`, `burn` (`pot 90, mag 6, 3` rnds), `hazard: {id: "pitch_fire", shape: "SKILL", at: "target", rounds: 2}`, `uses: 1` (`BURST` hazard valid in `hooks_schema`). |
| `staff` M1 | `arcane_bolt` | Marrow-Force Bolt | `ATTACK` -> `assault` | `2 / 2` | `2–3` | `1.25x`, `stag 8` (`2H`-exclusive high-coefficient bolt). |
| `staff` M2 | `arc_lance` | Line Discharge | `ATTACK` (`LINE`) -> `[field, assault]` | `2 / 2` | `1` / `LINE` | `1.10x`, `stag 10`. |
| `staff` M3 | `hex` | Warding Hex | `ATTACK` -> `assault` | `2 / 2` | `2–4` | `0.85x`, `stag 18`, `weaken -20%` (`pot 90, 2` rnds), `sure_break: true`. |
| `staff` MW | `inferno` | Conflagration | `ATTACK` (`BURST`) -> `[field, assault]` | `3 / 3` | `2–4` / `BURST` | `1.05x`, `stag 10`, `burn` (`pot 85, mag 5, 2` rnds). |
| `staff` KS | `staff_cataclysm` | Abyssal Conduit | `ATTACK` (`BURST`) -> `[field, assault]` | `4 / 4` | `2–4` / `BURST` | `1.65x`, `stag 22`, `armor_piercing: true`, `uses: 1`. |

- **Masterwork Bindings (`items.json`)**: Every main-hand weapon sets `"technique": {"skill": "<MW skill of its family>", "grip": "1h"|"2h"}` (for instance `notched_billhook`/`vanes_bill` -> `halberd_sweep`, `penal_shackles` -> `foundry_drive`, `winch_arbalest`/`stirrup_arbalest` -> `bodkin_punch`, `quells_marrow_scalpel`/`quells_bone_saws` -> `vivisect`, `brass_vitriol_sprayer`/`pyre_nozzle` -> `flash_nozzle`). Reforging via `Berek`'s `hearth_reforge` sets `masterwork: true` (`weapon_instances.rs`).
