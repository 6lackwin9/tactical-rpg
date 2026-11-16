# `PK8_ACT23_CONTENT` — `[PROVISIONAL]` Act 2–3 Content Wiring, Sample World Boundary & `WAVE2-BALANCE` (`PR6-1`, `PR6-5..`)

- **Status**: **`[PROVISIONAL]`** — Wave 2 `.1` content, drop-hook, legacy-world parity, and balance tuning spec.
- **Parent Plan**: [WAVE2_PLAN.md](WAVE2_PLAN.md) | **Canon**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§4.1, §5.1–5.3, `CL-01..11`).

---

## 1. Deferred Act 2 Drops, `MASTERY_BOOK`s & `pk2.*` Quest Setters (`PR6-5`, closing `FB-277`)

`PR5-62c` (`48f24342`) authored the `15` weapon trees, `18` `gp_*` grafts, `30` `sg_*` surgeries, and `hearth_rite`, leaving Act 2 drops/books/`pk2.*` quest flags deferred to Wave 2:

| Hook Group (`FB-277`) | Target Files (`aegis content` seed) | Exact Provisional Wiring (`0` `ashlands`/`iron_reaches`/`tidewrack` edits) |
| --- | --- | --- |
| **`slot_*` Organ & Tissue Drops** | `catalog/encounters.json`, `worlds/sunk_ward/loot.json` | Add weighted/guaranteed drops on Act 1–2 `sunk_ward` foes: `calcified_husk` (`sinew_braid` / `ossified_bellows` + `slot_bone_spur` / `slot_chitin_plate`), `ash_crawler` (`sinew_braid` + `slot_blood_sac` + `caustic_gall_sac`), `tallow_dredger` (`caustic_gall_sac` + `slot_blood_sac`), `render_beast` (`stalker_optic_nerve` + `slot_eye_tissue` + `render_beast_tendon`), `mortuary_flagellant` / `purifier` (`slot_eye_tissue` / `Rock Salt`), `calcified_amalgam` (`titan_marrow_core`). |
| **`black_iron_ingot`** (*Black-Iron Bar*) | `worlds/sunk_ward/{shops,loot}.json` | Stock `2` limited bars at `berek_hearth` gated on `pk2.masterwork_quest` (`80 Scrap` each), plus deterministic chest/clear rewards in `2.2` Fourth Kiln Stockades & `dunstan`/`berek` Foundry raid so Hearth `forge_rite` (`hearth_rite`), `train_third`, and `train_minor` stay bounded without infinite farm loops (`P-PROG-04`). |
| **`15` `MASTERY_BOOK` Items** | `catalog/items.json`, `sunk_ward/{shops,loot}.json` | Author `15` `item_type: "MASTERY_BOOK"` manuals (`book_<family>`, `+150` base XP -> `+225` at `Prof 3`, `65 Scrap`, `stock: 1` per act at `berek_hearth` / `varka_trade` + Act 2 quest/cache drops) covering all `15` main-hand weapon families (`polearm`..`staff`). |
| **`pk2.*` Flag Progression** | `worlds/sunk_ward/{sequences,quests,dialogues}.json` | Keep `seq_after_m5` setting `pk2.slab_open` + `pk2.hearth_open`; set `pk2.masterwork_quest` on `Berek`'s Act 2 *"Cold Slag, Black Iron"* clear (`post-2.2 Lucan`); set `pk2.minor_quest` on mid-Act 2 Foundry blueprint turn-in; set `pk2.apex_open` + `pk2.bone_scraping` on `2.4 Embalming Cloisters` (`Lazar` defeat); set `pk2.rite_<class>` on each Founder Loyalty node (OR **Survivor Inheritance** mentor quest when `roster.departed.<founder>` is true, `CL-09`). |

---

## 2. `sunk_ward` Acts 2–3 Expansion vs. Sample Worlds (`iron_reaches`, `tidewrack`, `ashlands`)

### 2.1 Explicit World Boundary Disposition (`P-ARCH-02`, `ACTS23` Q-A2(a))

- **Canonical Emberwake World (`sunk_ward` ONLY)**: All Wave 2 content (Acts 2 & 3 regions, nodes `08-22`, Bestiary, Loyalty/Inheritance arcs, Act 2–3 checkpoints, Sieges, Endings) lives inside **`games/emberwake/data/worlds/sunk_ward/`** (`act 1 -> 2 -> 3` progression via one hub `sluice_undercroft`).
- **Legacy Sample & Parity Worlds (`ashlands`, `iron_reaches`, `tidewrack`) — `[KEEP-PARITY / CLASSIC-ONLY]`**:
  - `ashlands` (`ashen_marches`, `cinder_pass`), `iron_reaches` (`ore_highlands` -> `slag_gate`, `south_road`, `orsa_tallow`), and `tidewrack` (`tidewrack_coast` -> `gull_harbour`, `wreck_strand`, `strand_clash`) stay strict `classic` ruleset fixtures protecting `journey_golden.json`, `phasefold_golden.json`, `save_aliases.rs`, and cross-world route tests (`ashlands <-> iron_reaches:slag_gate`).
  - Rule checks (`fc.sh content check`) still validate those 3 worlds, so any shared catalog edit in Wave 2 keeps their `loadout` / `item` / `encounter` refs valid (`0` unintended removals, `0` narrative expansion outside `sunk_ward`).

### 2.2 `sunk_ward` Act 2 & Act 3 Node Roadmap (`PK8`, `CANON.md` §2.2 & §4.1)

| Region (`regions.json`) | Act & Unlock | Nodes (`08-22`) & Encounters | Bosses, Mechanics & Unlocks |
| --- | --- | --- | --- |
| **`high_bastion`** (*The High Bastion*) | `Act 2` (`post-M5`, Alert floor `30`) | `08` Watch Ramparts (`2.1`), `09` Gilded Blvd (`2.1b`), `10` Tallow Reservoir (`2.3`), `11` Embalming Cloisters (`2.4`), `12` Culling Stockades (`2.2`), `13` Grand Belltower (`2.5`), `14-15` Reliquary Crypts (`2.6`), + `6` Founder Loyalty / Mentor Inheritance nodes (`CL-02`, `CL-09`), + `Scribe's Crypt` story base | Act 2 Recruit Tutorial (dosed Cohort if Crucible took dosed Founder) + `Calcification` tutorial (`Sabina`'s lesson + `1x` Quenched Cinder Salt); `Lucan` rescue -> `fac_bunks` discount; `Lazar` fall -> **Debate 4** + `pk2.apex_open` + *Great Tithing* reveal; `Penitent Bell-Warden` (`2.5`) silences St. Lyra bells; `Ruthven` (`25%` siege retreat) + `First Apostle Saint-Warden` (`2.6`) -> `Mantle Key`. |
| **`mantle_fissure`** (*The Mantle Fissure*) | `Act 3` (`Mantle Key`, Hard Clock min `12d`) | `16` Breach Lift, `17` Basalt Chasm (`3.1` Survey Outpost / `Malachi`), `18` Marrow Conduits (`3.2`), `19` Chorus Spire (`3.3` opt.), `20` Silent Ossuary (`3.4` opt.), `21` High Altar (`3.5`) -> `22` Dreaming Core (`3.6`), + `Weeping Redoubt` story base | `Malachi` ally intro (`3.1`); `Ulric` kills conduits (`+2d`); `Ysolde` slain or `Malachi`-mediated on Flesh-Broker (`3.3`, halves Resonance); `$crucible_captive` Stage V breach kills `Nemeth` -> `2x2` `swap` boss (`3.4`); `Othemar` falls (`3.5`) -> `camp_prep` Altar lip -> `3.6` `2x2` Dreaming Core (`A / B / C`). |

---

## 3. `WAVE2-BALANCE` Scope & Tuning Plan (`PR6-1`, `FB-276` & `FB-277`)

Wave 1 landed cleanly (`0 METRIC FAIL`, `0 PLAN FAIL`), flagging `4` targeted balance items for `PR6-1` immediately before Act 2 roster/boss tuning:

| Observed Wave 1 Signal (`FB-276` / `FB-277`) | Root-Cause Audit | Exact `PR6-1` Candidate Knobs & Target Band |
| --- | --- | --- |
| **`brawler` `swap_delta: -9.4`, `top5_count: 1` (min `2`)** | `riveted_gauntlets` (`power 13`) + `forearm_braces` (`mit 3`, vs Warrior `13 + 6 mit shield` & `0.75x shield_bash`) paired with low `AP`-to-damage multipliers on `brawler_clinch` (`0.70x / 2 AP`) and `slag_charge` (`1.35x / 3 AP`). | **Data-only knobs**: raise `gauntlet` `base.main_1h.power` (`13 -> 15`) OR bump `brawler_clinch` (`0.70x -> 0.90x`) + `slag_charge` (`1.35x -> 1.50x`) and `forearm_braces` `mit 3 -> 4`. Target: `swap_delta` inside **`[-5.0, +5.0]`**, `top5_count >= 2`. |
| **`sim sweep 5 10`: `line 92–95%` (target `70–90%`)** | `FB-276` left generated `line` `standard` enemy `warrior` without post-`PR5-62c` weapon skills (`hook_drag`). | Equip generated `line` `standard` enemies in `sim.json` / generator cleanly with `M1` family strikes (`hook_drag` on `warrior`, etc.) -> bring `line` cleanly to **`75–85%`**. |
| **`sim party 5 10` `boss 20%` vs `sim party 5 30` `33–37%` / `sim sweep 34%`** | Small-sample (`10`-seed) variance plus `warrior`/`brawler` melee output dip dragging default `4`-hero squad (`warrior, ranger, priest, mage`). | Run `sim party 5 30` AND `sim sweep 5 30` after `brawler` & `line` fixes; tune `@boss` `+40% hp_pct` (`35%..40%`) only if 30-seed boss drifts out of **`30–50%`**, keeping Crucible `enc_evt_06` at **`0%`** win! |
| **`[PROVISIONAL]` Mage AoE (`0.28 / 0.28 / 0.28 / 0.22`)** | Emergency flatten from `0.50x` prevented boss burst spikes, but risks making `cinder_spark` / `glacial_burst` feel flat vs `1H` single-target. | Compare `0.28/0.28/0.28/0.22` vs `0.32/0.30/0.32/0.25` in `aoe_swarm` & `sim party 5 30` (`counts=on`); ratify whichever keeps `mage` `swap_delta` in `[-3, +3]` & `boss` in `30–50%`. |
