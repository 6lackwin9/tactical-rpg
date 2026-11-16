# `WAVE2_PLAN` — `[PROVISIONAL]` Wave 2 Master Plan (`PK2-STAGE-PASSIVES`, `WAVE2-BALANCE`, `PK3..PK8`)

- **Status**: **`[PROVISIONAL]`** — Ready for Wave 2 Build execution & interactive user review (`docs/PRINCIPLES.md` §0, `ACTS23.md` §4).
- **Baseline**: `origin/main` / `origin/develop` (`43a6cbd8`: `PR5-57..62c` + `CTX-CLEANUP1-FINAL` Wave 1 release).
- **Locked Foundation**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§1–§6 `[LOCKED]`), [ACTS23.md](../specs/ACTS23.md) (Q-A1..Q-A10, Channel §5.1, Techniques §5.2), [PK1_CORRUPTION.md](PK1_CORRUPTION.md), [PK2_GRAFTS.md](PK2_GRAFTS.md), [PRINCIPLES.md](../PRINCIPLES.md).

---

## 1. Wave 2 `.1` Session Index (`<= 300 L / <= 25 KB` per file)

| Domain Spec | Release Wave | Covered Systems & Held Items | Target Queue |
| --- | --- | --- | --- |
| [PK3_ROSTER_LIFECYCLE.md](PK3_ROSTER_LIFECYCLE.md) | `R5-Finish` | `PK2-STAGE-PASSIVES` (`calcify_2`/`calcify_3`), `PR5-11` Bleed-out & Revive (`rally` + `smelling_salts`), `PR5-14` Recruit Generator (`18-24` Act 2 Cohorts, `pool`, `Isolation Crypts`, `Living Martyrs`), `PR5-15` Procedural Keepsakes (`by_class`) | `PR6-2..PR6-4` (`§1`) |
| [PK4_PK5_CAMPAIGN_HUB.md](PK4_PK5_CAMPAIGN_HUB.md) | `R6` | `PK4` Campaign Pressure (`Doctrine` `-100..+100` + play-input triggers, Act 2–3 Checkpoints, `2.4` Great Tithing Reveal, Act 3 Hard Clock + Tallow scarcity); `PK5` Hub Facilities (`5 × 3` tiers) + Connected-Room Alert Sieges (`Ruthven` `25%` retreat) | `§2` Feasibility -> `§1` (`PR6-6..PR6-9`) |
| [PK6_PK7_BOSSES_ENDINGS.md](PK6_PK7_BOSSES_ENDINGS.md) | `R7` | `PK6` Boss & Act 3 Tech (Bell/Mantle Resonance hazard meters, `3.5->3.6` Altar-lip `camp_prep`, Dream-Echo retries w/ day cost, `Node 20` Saint-Weapon `swap`/`side`, `3.5/3.6` multi-phase); `PK7` Endings `A / B / C`, Epilogue & Extinguished screens | `§2` Feasibility -> `§1` (`PR6-10..PR6-13`) |
| [PK8_ACT23_CONTENT.md](PK8_ACT23_CONTENT.md) | `R5-Finish` -> `R7` | Deferred Act 2 drop & `pk2.*` quest hooks (`slot_*`, `black_iron_ingot`, `15` `MASTERY_BOOK`s), `sunk_ward` Act 2 `high_bastion` (`08-15`) & Act 3 `mantle_fissure` (`16-22`) content map, legacy sample world parity (`iron_reaches`, `tidewrack`, `ashlands`), & `WAVE2-BALANCE` (`brawler -9.4`, `sim party`, `line 92-95%`, Mage AoE) | `PR6-1`, `PR6-5` (`§1`) + Act 2/3 waves |

---

## 2. Wave 2 Execution Order & Release Gates

```text
43a6cbd8 (Wave 1 Release)
  ├─► PR6-1  [WAVE2-BALANCE]          (brawler -9.4, line 70-90%, Mage AoE review, sim party 5 30 + sweep)
  └─► PR6-2  [PK2-STAGE-PASSIVES]     (armor / move_rate / ranged min_distance kit grants -> calcify_2, calcify_3)
        └─► PR6-3  [PK3-BLEEDOUT-REVIVE]     (bleed_out_rounds, revive cell-resolve, rally + smelling_salts, HUD tell)
              └─► PR6-4  [PK3-RECRUITS-KEEPSAKES] (pool, Cohorts generator, crypts, Martyrs fuse, by_class keepsakes)
                    └─► PR6-5  [ACT2-DROPS-PK2-FLAGS] (15 MASTERY_BOOK, slot_* drops, black_iron_ingot stock, pk2.* hooks)
                          ├─► R6: PK4 (Doctrine, Checkpoints, Act 3 Clock/Tallow) + PK5 (Facilities x3, Connected Sieges) + Act 2 PK8 (high_bastion 08-15)
                          └─► R7: PK6 (Resonance, Node 20, Altar Camp Prep, Dream-Echoes) + PK7 (Endings A/B/C) + Act 3 PK8 (mantle_fissure 16-22)
```

- **Strict Parity Invariant (`P-ARCH-02`, `P-CAMP-01`)**: `ashlands`, `iron_reaches`, and `tidewrack` stay `classic` ruleset fixtures (`0` Act 2–3 bleed-out, Strain, durability, or Martyr mechanics leak into classic fights; `journey_golden`, `phasefold_golden`, and `rules_contract_golden` change ONLY when a slice explicitly names a golden update in [READY.md](../READY.md)).
- **Dual Layer Split (`P-ARCH-01`, `fc.sh layers`)**: Every new rule/hook added in `PK2-STAGE-PASSIVES` and `PK3..PK7` under `kits/rpg/` and `engine/` uses strictly generic RPG terminology (`unit_tracks`, `bleed_out_rounds`, `revive`, `recruits`, `keepsakes`, `facilities`, `siege`, `ending`, `doctrine` -> numeric faction/alignment gauge) with zero `games/emberwake/lore_words.txt` words.

---

## 3. `[PROVISIONAL]` Decisions Requiring User Playtest / Review Sign-Off

1. **`WAVE2-BALANCE` Targets (`PR6-1`)**: Accept tuning `brawler_clinch` (`0.70x -> 0.90x`), `slag_charge` (`1.35x -> 1.50x`), and `riveted_gauntlets` (`13 -> 15` probe), re-tuning `sim.json` generated `line` (`92–95% -> 70–90%` w/ `hook_drag`), and deciding whether `[PROVISIONAL]` Mage AoE (`0.28/0.28/0.28/0.22`) stays or bumps to `0.32–0.35`.
2. **Revive Design (`PR6-3`)**: Put `revive: true` on Priest `L2` `rally` (*Urgent Triage*, `SUPPORT ALLY 1–3`) + consumable `smelling_salts` (`COMMON` Snap pouch item), relocating revived allies onto nearest open adjacent tile if their downed cell is occupied (`FB-167` Q9).
3. **Campaign & Siege Cadence (`PK4` / `PK5`)**: `2–3` connected room encounters per Hub Breach Siege; Alert breach in Acts 2–3 downgrades damaged facility levels by `1` (floor `1`) rather than ending the campaign; `Warded Tallow` cost doubles (`2x`) in Act 3.
4. **Endings & Legacy Sample Worlds (`PK7` / `PK8`)**: Lock `iron_reaches`, `tidewrack`, and `ashlands` as classic parity/route fixtures (`[KEEP-PARITY]`) while placing `High Bastion` (`08-15`) and `Mantle Fissure` (`16-22`) inside `sunk_ward` (`ACTS23` Q-A2(a)).
