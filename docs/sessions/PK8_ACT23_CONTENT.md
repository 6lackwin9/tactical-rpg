# `PK8_ACT23_CONTENT` — `[PROVISIONAL]` Act 2–3 Content Wiring, Sample World Boundary & `WAVE2-BALANCE` (`PR6-1`, `PR6-5..`)

- **Status**: **`[PROVISIONAL]`** — `PR6-1` (`29dfc241`) & `PR6-5` (`f132eabc`) **DONE**; `PR6-12` (`PK8-ACT2-CONTENT`) & `PR6-13` (`PK8-ACT3-CONTENT`) queued in `READY.md` §1 (`8`–`9`).
- **Parent Plan**: [WAVE2_PLAN.md](WAVE2_PLAN.md) | **Canon**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§4.1, §5.1–5.3, `CL-01..11`).

---

## 1. Act 2 Drops, `MASTERY_BOOK`s & `pk2.*` Flags (`PR6-5` — **DONE `f132eabc`**, closing `FB-281`)

`PR5-62c` (`48f24342`) authored the `15` weapon trees, `18` `gp_*` grafts, `30` `sg_*` surgeries, and `hearth_rite`; **`PR6-5` (`f132eabc`, `26`-step `pr6_5_act2_drops_flags.json`, `0`-diff idempotent)** seeded the drops, manuals, shops, and `Act 1` base flags while reserving High Bastion `08-15` setters for `PR6-12`:

| Hook Group (`FB-277` / `FB-281`) | Landed in `PR6-5` (`f132eabc`, `0` `ashlands`/`iron_reaches`/`tidewrack` edits) | Remaining `PR6-12` (`PK8-ACT2-CONTENT`) Work |
| --- | --- | --- |
| **`slot_*`, `rock_salt` & `titan_marrow_core` Drops** | `catalog/units.json`: additive loot (`sum(chance) <= 100`) on `calcified_husk` (`+slot_bone_spur 18%`, `+slot_chitin_plate 15%`), `ash_crawler`/`tallow_dredger` (`+slot_blood_sac 15%`), `render_beast` (`+slot_eye_tissue 15%`), `inquisitorial_purifier`/`mortuary_flagellant` (`+slot_eye_tissue 12%`, `+rock_salt 20%/18%` accepted in Act 1 too), `calcified_amalgam` (`+titan_marrow_core 5%`). | Tune Act 2 `high_bastion` foe loot tables & regional drop pacing as nodes `08-15` land. |
| **`black_iron_ingot`** (*Black-Iron Bar*) | `sunk_ward/shops.json`: `loot_hearth_ingot_cache` (`count: 1`, `80` Scrap, `pk2.hearth_open`) + `black_iron_ingot` (`count: 2`, `80` Scrap, `pk2.masterwork_quest`) at `berek_hearth`. | Place bounded deterministic `black_iron_ingot` chest/clear rewards in `2.2` Fourth Kiln Stockades & Foundry raid (`P-PROG-04`). |
| **`15` `MASTERY_BOOK` Items** | `catalog/items.json` + `item_families.json` (`manual`): `15` `book_<family>` items (`150` base XP, `65` Scrap, `price: -1`), stocked `count: 1` (`8` martial at `berek_hearth` `pk2.hearth_open`, `7` knave/occult at `varka_trade` `pk2.slab_open`). | Add bounded Act 2 exploration cache drops alongside `book_*` shop stock. |
| **`pk2.*` Flag Progression** | `seq_after_m5` preserves `pk2.slab_open` + `pk2.hearth_open`. | Seed `high_bastion` (`08-15`) setters: `pk2.masterwork_quest` (post-`2.2` `Berek`), `pk2.minor_quest` (Foundry blueprint), `pk2.apex_open` + `pk2.bone_scraping` (`2.4` `Lazar` defeat), `pk2.rite_<class>` (Loyalty or Survivor Inheritance, `CL-09`). |

---

## 2. `sunk_ward` Acts 2–3 Expansion vs. Sample Worlds (`iron_reaches`, `tidewrack`, `ashlands`)

### 2.1 Explicit World Boundary Disposition (`P-ARCH-02`, `ACTS23` Q-A2(a))

- **Canonical Emberwake World (`sunk_ward` ONLY)**: All Wave 2 content (Acts 2 & 3 regions, nodes `08-22`, Bestiary, Loyalty/Inheritance arcs, Act 2–3 checkpoints, Sieges, Endings) lives inside **`games/emberwake/data/worlds/sunk_ward/`** (`act 1 -> 2 -> 3` progression via one hub `sluice_undercroft`).
- **Legacy Sample & Parity Worlds (`ashlands`, `iron_reaches`, `tidewrack`) — `[KEEP-PARITY / CLASSIC-ONLY]`** (ruling F7, 2026-11-17: `iron_reaches` and `tidewrack` are parked dev and test worlds, outside Playable Release 1):
  - `ashlands` (`ashen_marches`, `cinder_pass`), `iron_reaches` (`ore_highlands` -> `slag_gate`, `south_road`, `orsa_tallow`), and `tidewrack` (`tidewrack_coast` -> `gull_harbour`, `wreck_strand`, `strand_clash`) stay strict `classic` ruleset fixtures protecting `journey_golden.json`, `phasefold_golden.json`, `save_aliases.rs`, and cross-world route tests (`ashlands <-> iron_reaches:slag_gate`).
  - Rule checks (`fc.sh content check`) still validate those 3 worlds, so any shared catalog edit in Wave 2 keeps their `loadout` / `item` / `encounter` refs valid (`0` unintended removals, `0` narrative expansion outside `sunk_ward`).

### 2.2 `sunk_ward` Act 2 & Act 3 Node Roadmap (`PK8`, `CANON.md` §2.2 & §4.1)

| Region (`regions.json`) | Act & Unlock | Nodes (`08-22`) & Encounters | Bosses, Mechanics & Unlocks |
| --- | --- | --- | --- |
| **`high_bastion`** (*The High Bastion*) | `Act 2` (`post-M5`, Alert floor `30`) | `08` Watch Ramparts (entry, no story label), `09` Gilded Blvd (`2.1`; CANON numbering, ruling F4 2026-11-17), `10` Tallow Reservoir (`2.3`), `11` Embalming Cloisters (`2.4`), `12` Culling Stockades (`2.2`), `13` Grand Belltower (`2.5`), `14-15` Reliquary Crypts (`2.6`), + `6` Founder Loyalty / Mentor Inheritance nodes (`CL-02`, `CL-09`), + `Scribe's Crypt` story base | Act 2 Recruit Tutorial (dosed Cohort if Crucible took dosed Founder) + `Calcification` tutorial (`Sabina`'s lesson + `1x` Quenched Cinder Salt); `Lucan` rescue -> `fac_bunks` discount; `Lazar` fall -> **Debate 4** + `pk2.apex_open` + *Great Tithing* reveal; `Penitent Bell-Warden` (`2.5`) silences St. Lyra bells; `Ruthven` (`25%` siege retreat) + `First Apostle Saint-Warden` (`2.6`) -> `Mantle Key`. |
| **`mantle_fissure`** (*The Mantle Fissure*) | `Act 3` (`Mantle Key`, Hard Clock min `12d`) | `16` Breach Lift, `17` Basalt Chasm (`3.1` Survey Outpost / `Malachi`), `18` Marrow Conduits (`3.2`), `19` Chorus Spire (`3.3` opt.), `20` Silent Ossuary (`3.4` opt.), `21` High Altar (`3.5`) -> `22` Dreaming Core (`3.6`), + `Weeping Redoubt` story base | `Malachi` ally intro (`3.1`); `Ulric` kills conduits (`+2d`); `Ysolde` slain or `Malachi`-mediated on Flesh-Broker (`3.3`, halves Resonance); `$crucible_captive` Stage V breach kills `Nemeth` -> `2x2` `swap` boss (`3.4`); `Othemar` falls (`3.5`) -> `camp_prep` Altar lip -> `3.6` `2x2` Dreaming Core (`A / B / C`). |

---

## 3. `WAVE2-BALANCE` Scope & Ratified Results (`PR6-1` — **DONE `29dfc241`**, closing `FB-278`)

Wave 1 landed cleanly (`0 METRIC FAIL`, `0 PLAN FAIL`), and **`PR6-1` (`29dfc241`, `pr6_1_balance.json`)** resolved all `4` targeted items cleanly (`sim sweep 5 30`: `mob 100%, line 88%, wave 60%, elite 78% X, boss 32%`; `sim party 5 30`: `mob 100%, line 83%, wave 67%, elite 80% X, boss 40%`; Crucible `0%`):

| Observed Wave 1 Signal (`FB-276` / `FB-277`) | Root-Cause Audit | Landed & Ratified `PR6-1` Solution (`29dfc241`, `FB-278`) |
| --- | --- | --- |
| **`brawler` `swap_delta: -9.4`, `top5_count: 1` (min `2`)** | `riveted_gauntlets` (`power 13`) + `forearm_braces` (`mit 3`) paired with low `brawler_clinch` (`0.70x`) and `slag_charge` (`1.35x`). | **Ratified**: `gauntlet` `13 -> 15`, `forearm_braces +1 mit` (`3 -> 4`), `brawler_clinch` `0.70x -> 0.95x`, `slag_charge` `1.35x -> 1.60x` -> **`brawler -2.6 (top5 3)`**. |
| **`sim sweep 5 10`: `line 92–95%` (target `70–90%`)** | Generated `line` `standard` foes lacked post-`PR5-62c` weapon skills. | **Ratified**: Added `Generated.enemy_mastery` (`rpgkit_sim`) and set `sim.json` `line.enemy_mastery: {"warrior": 1, "rogue": 1}` (`hook_drag` + `quick_stab`) -> **`line 88%` sweep / `83%` party**. |
| **`sim party 5 10` `boss 20%` vs `sim party 5 30` `33–37%` / `sim sweep 34%`** | Small-sample (`10`-seed) variance plus `warrior`/`brawler` melee output dip. | **Ratified**: `@boss` `+40% hp_pct` needed no change; `30`-seed sweeps cleanly hit **`boss 32%` sweep / `40%` party** and Crucible `0%` (`elite 78–80%` kept `3-5%` above `50-75%` unless tuned in `PR6-13`). |
| **`[PROVISIONAL]` Mage AoE (`0.28 / 0.28 / 0.28 / 0.22`)** | Emergency flatten kept boss spikes down but risked flat AoE feel vs `1H`. | **Ratified**: Raised to **`cinder_spark 0.32 / firebomb 0.30 / glacial_burst 0.32 / choke_cloud 0.25`** (`mage -0.6 (top5 3)`, `5.89` rnds clear in `aoe_swarm`). |
