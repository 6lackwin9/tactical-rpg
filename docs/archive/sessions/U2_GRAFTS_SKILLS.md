# `U2_GRAFTS_SKILLS` — `[PROVISIONAL]` Monster Growth Pins, Cranial Rituals & Inoculated Keystones (`PK1`/`PK2`)

Archived 2026-11-16 (`CTX-CLEANUP1-FINAL`), implemented (`PR5-62a` `6e662c53`, `PR5-62c` `48f24342`): kept for history. Do not build from this file.

- **Parent Index**: [U2_CLASSPASS_TECHNIQUES.md](U2_CLASSPASS_TECHNIQUES.md) (`[PROVISIONAL]` User Rule 5).
- **Build Target**: `PR5-62a` (`crit`, `"channel"` potency group, `SkillReach.ranged`) -> `PR5-62c` (`TECHNIQUES-GRAFTS-DATA`).

---

## 1. Channel & Surgery Rules (`PK1`, `PK2`, `CANON.md` §4.2 & §5.3)

- **`Channel` Rules**: `0 AP` round toggle (`x1.25` base marrow amplification on eligible attacks, `Strain` floor `[0, 5, 15, 30, 50]` across `0..4` installed grafts, `Rest -> Feed`, `Vent` at `>= 50` Strain, Rupture at `100` -> `Unchecked Spasm`, `Biological Collapse`, `Flesh Breach` `mark: "calcify"`).
- **Single-Intent Guard (`check/hooks.rs:441`)**: Every `channel.only: true` skill must derive a single IR21 intent (`assault`, `maneuver`, `guard`, or `field` via `BURST` + `hazard`). All AoE `channel.only: true` skills below include a placed `hazard` (`field`) or stay `RING` (`guard`) / unshaped single-target (`assault`), while `purge_marrow` sets `channel: {"none": true}` (`SUPPORT` `[support, guard]` stays callable anytime).
- **Surgery Slots (`Chirurgeon Sabina`, post-`M2` `Slab Retorts`)**: `forearms`, `thoracic`, and `visceral` accept any Monster Ossuary Growth pin (`body_role: "monster"`); `cranial` accepts a Class Cranial Ritual (`tier: 1`) or Inoculated Keystone (`tier: 3`, `keystone: true`, `+1 Calcification`, requires `3` installed grafts + `pk2.apex_open` + `pk2.rite_<class>`).

---

## 2. Monster Ossuary-Growth Grafts (`forearms` / `thoracic` / `visceral`)

Every `CANON.md` §5.3 ossuary growth and Act 1–2 aberration drop maps into a `Slab`-refined pin (`3` slot recipes each: `forearms + slot_bone_spur`, `thoracic + slot_chitin_plate`, `visceral + slot_blood_sac`):

| Graft ID & Pin Name | Tier | Material & Mob Source (`CANON.md` §5.3) | `body_effects` (Passive + `unlock`) | Channel Skill ID & Specs (`channel.only: true`) |
| --- | --- | --- | --- | --- |
| `graft_optic` (*Amber Tapetum Pin*) | `1` | `stalker_optic_nerve` (`Render-Beast`) | `stat: PERC +1`, `unlock: "marrow_lash"` | `marrow_lash` (*Marrow Lash*): `ATTACK` -> `assault`, `1 AP, 2 blk`, `1–2`, `1.45x`, `stag 10`, `armor_piercing: true`, **`Strain 15`**. |
| `graft_sinew` (*Black-Sinew Pin*) | `1` | `sinew_braid` (`Calcified Husk`, `Ash-Crawler`) | `resist: root +20`, `unlock: "marrow_snare"` | `marrow_snare` (*Sinew Grasp*): `ATTACK` -> `assault`, `2 AP, 2 blk`, `1–3`, `1.20x`, `stag 10`, `armor_piercing: true`, `forced_move: {kind: "pull", tiles: 1}`, `root` (`pot 85, 1` rnd), **`Strain 15`**. |
| `graft_bellows` (*Ossified Bellows Pin*) | `1` | `ossified_bellows` (`Calcified Husk`, Act 2 Horrors) | `stat: max_poise +10`, `unlock: "marrow_shriek"` | `marrow_shriek` (*Bellows Shriek*): `ATTACK` (`RING`) -> `guard`, `2 AP, 2 blk`, `1 / RING`, `1.05x`, `stag 16`, `armor_piercing: true`, `forced_move: {kind: "push", tiles: 1}`, **`Strain 20`**. |
| `graft_bezoar` (*Vitriol Bezoar Pin*) | `2` | `caustic_gall_sac` (`Tallow Dredger`, `Ash-Crawler`, `Orlov`) | `resist: poison +40`, `resist: burn +40`, `unlock: "marrow_spray"` | `marrow_spray` (*Vitriol Gorge*): `ATTACK` (`BURST`+`haz`) -> `field`, `2 AP, 2 blk`, `2 / BURST`, `1.15x`, `stag 8`, `armor_piercing: true`, `poison` (`pot 90, mag 6, 3` rnds), `hazard: {id: "choke_gas", shape: "SKILL", at: "target", rounds: 2}`, **`Strain 25`**. |
| `graft_valve` (*Siphon-Valve Pin*) | `2` | `inquisitor_marrow_valve` (`Proctor Quell` / Proctors) | `trigger: kill -> {"type": "meter", "meter": "strain", "delta": -10, "to": "self"}`, `unlock: "marrow_bolt"` | `marrow_bolt` (*Marrow Bolt*): `ATTACK` -> `assault`, `2 AP, 2 blk`, `2–4`, `1.55x`, `stag 12`, `armor_piercing: true`, **`Strain 25`**. |
| `graft_core` (*Dreamer's Geode Pin*) | `3` | `titan_marrow_core` (`Calcified Amalgam`, `Living Martyrs`, Horrors) | `status_potency +8`, `crit +8` (`PR5-62a`), `unlock: "marrow_rend"` (`+1 Calcification`) | `marrow_rend` (*Marrow Rend*): `ATTACK` -> `assault`, `2 AP, 2 blk`, `1–2`, `2.05x`, `stag 22`, `armor_piercing: true`, `ignore_guard: true`, **`Strain 35`**. |

---

## 3. 6 Class Cranial Ritual Grafts (`slot: "cranial"`, `body_role: "class"`, `tier: 1`)

Installed after unlocking `pk2.slab_open` + class personal quest/rite flag `pk2.rite_<class>` (`1x stalker_optic_nerve` + `1x slot_eye_tissue` + `40 Scrap`):

| Class | Graft ID & Name | Body Passive Bonus | Ritual Skill ID & Name | Kind -> IR21 | AP / Blk | Exact Specs (`channel.only: true`) |
| --- | --- | --- | --- | --- | --- | --- |
| `warrior` | `graft_war_ritual` (*Cranial Iron-Vein*) | `stat: max_poise +10` | `marrow_bulwark` (*Ossified Wall*) | `DEFEND` -> `guard` | `2 / 2` | `mitigation: 0.50`, `guard_rounds: 1`, `grant_status: "fortify"`, `status_magnitude: 8`, `status_rounds: 2`, NO `windup_poise`/`unbreakable`, **`Strain 20`**. |
| `brawler` | `graft_bra_ritual` (*Kiln-Cranial Spur*) | `forced_move: {bonus_tiles: 1}` | `marrow_pummel` (*Calcified Ram*) | `ATTACK` -> `assault` | `2 / 2` | `1`, `1.50x`, `stag 24`, `armor_piercing: true`, `forced_move: {kind: "push", tiles: 2, collide_break: true}`, **`Strain 20`**. |
| `ranger` | `graft_rng_ritual` (*Ocular Marrow-Sight*) | `stat: PERC +1` | `marrow_sight` (*Tapetum Lance*) | `ATTACK` -> `assault` | `2 / 2` | `2–6`, `1.60x`, `stag 10`, `armor_piercing: true`, `ignore_guard: true`, `expose -10` (`pot 95, 2` rnds), **`Strain 20`**. |
| `rogue` | `graft_rog_ritual` (*Shade-Nerve Loop*) | `crit +5` (`PR5-62a`) | `marrow_step` (*Fade-Marrow Cut*) | `ATTACK` -> `maneuver` | `2 / 1` | `reach_min: 0, reach_max: 3, dash_tiles: 0, position: "behind"`, `1.45x`, `stag 8`, `armor_piercing: true`, `bleed` (`pot 95, 3` rnds), **`Strain 20`**. |
| `priest` | `graft_litany` (*Litany Graft*) | `status_potency +5` | `marrow_litany` (*Marrow Litany*) | `ATTACK` -> `assault` | `2 / 2` | `1–2`, `1.40x`, `stag 10`, `armor_piercing: true`, `weaken -20%` (`pot 85, 2` rnds), **`Strain 20`**. |
| `mage` | `graft_mag_ritual` (*Retort Cortex Pin*) | `stat: INT +1` | `marrow_caustic` (*Resonant Bloom*) | `ATTACK` (`BURST`+`haz`) -> `field` | `2 / 2` | `2–4 / BURST`, `1.10x`, `stag 10`, `armor_piercing: true`, `burn` (`pot 90, mag 5, 2` rnds), `hazard: {id: "pitch_fire", shape: "SKILL", at: "target", rounds: 2}`, **`Strain 25`**. |

---

## 4. 6 Class Inoculated Keystones (`slot: "cranial"`, `body_role: "class"`, `tier: 3`, `keystone: true`, `+1 Calcification`)

Requires `pk2.apex_open` + `pk2.rite_<class>` + **`3` installed grafts** + `1x titan_marrow_core` + `1x slot_eye_tissue` + `150 Scrap`. Occupies the deck's single `keystone` slot (mutually exclusive with a Cold Steel weapon `keystone`).

| Class | Graft ID & Name | Body Keystone Bonus | Skill ID & Name | Kind -> IR21 | AP / Blk | Exact Specs (`uses_per_battle: 1`) |
| --- | --- | --- | --- | --- | --- | --- |
| `warrior` | `graft_ks_war` (*Stone-Marrow Crown*) | `guarded_hit` (`limit: "round"`) -> `damage: 18` + `stagger: 18` (`to: "other"`) | `ks_iron_marrow` (*Marrow Phalanx*) | `DEFEND` -> `guard` | `3 / 2` | `mit 0.60`, `guard_rounds: 2`, `grant_status: "fortify"`, `mag 10, 2` rnds, `extra_grants: [{"status": "steadfast", "rounds": 1, "magnitude": 0}]`, NO `windup_poise`/`unbreakable`, `Strain 30`, `channel.only: true`. |
| `brawler` | `graft_ks_bra` (*Spinal Kiln-Core*) | `skill_potency: {group: "channel", value: 0.15}` (`PR5-62a`) | `ks_bone_eruption` (*Ossuary Erupt*) | `ATTACK` (`RING`) -> `guard` | `3 / 2` | `1 / RING`, `2.00x`, `stag 30`, `armor_piercing: true`, `sure_break: true`, `forced_move: {kind: "push", tiles: 2}`, `Strain 35`, `channel.only: true`. |
| `ranger` | `graft_ks_rng` (*Choir-Lens Keystone*) | `skill_reach: {kind: "ATTACK", reach: 1, ranged: true}` (`PR5-62a`) | `ks_marrow_needle` (*Marrow Needle*) | `ATTACK` -> `assault` | `3 / 2` | `2–6`, `2.25x`, `stag 18`, `armor_piercing: true`, `ignore_guard: true`, `sure_break: true`, `Strain 35`, `channel.only: true`. |
| `rogue` | `graft_ks_rog` (*Apex Nerve-Spire*) | `crit +10` (`PR5-62a`) | `ks_nerve_sever` (*Spinal Excision*) | `ATTACK` -> `assault` | `2 / 1` | `1`, `2.35x`, `armor_piercing: true`, `ignore_guard: true`, `exploit`: `bleed 1.4x`, `stagger 1.4x`, `Strain 30`, `channel.only: true`. |
| `priest` | `graft_purge` (*Inoculated Keystone*) | `skill_potency: {group: "heal", value: 0.15}` | `purge_marrow` (*Purge Marrow*) | `SUPPORT` -> `[support, guard]` | `1 / 1` | `0–2`, **`channel: {"none": true}`** (callable anytime), `unmark: {mark: "calcify"}`, cleanses `["stun", "weaken"]`; **calms Stage V `hostile_all` ally -> Stage IV without aborting mission**. |
| `mage` | `graft_ks_mag` (*Abyssal Crown Pin*) | `hit_dealt` (`limit: "round"`) -> `{"type": "meter", "meter": "strain", "delta": -5, "to": "self"}` | `ks_geode_nova` (*Siphon Nova*) | `ATTACK` (`BURST`+`haz`) -> `field` | `3 / 3` | `2–4 / BURST`, `1.85x`, `stag 20`, `armor_piercing: true`, `burn` (`pot 95, mag 6, 3` rnds), `hazard: {id: "phosphorus_fire", shape: "SKILL", at: "target", rounds: 2}`, `Strain 35`, `channel.only: true`. |
