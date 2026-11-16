# `PK4_PK5_CAMPAIGN_HUB` — `[PROVISIONAL]` Doctrine, Checkpoints, Hub Facilities & Connected-Room Sieges (`R6`)

- **Status**: **`[PROVISIONAL]`** — Wave 2 `.1` spec for `PK4` (`Doctrine`, `Checkpoints`, `2.4` Great Tithing Reveal, `Act 3 Hard Clock` & `Tallow Scarcity`) and `PK5` (`Hub Facilities x3` & `Connected-Room Sieges`).
- **Parent Plan**: [WAVE2_PLAN.md](WAVE2_PLAN.md) | **Canon & Decisions**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§2.2, §4.1–4.2, `CL-04`, `CL-10`), [ACTS23.md](../specs/ACTS23.md) (§2 #13–18, Q-A5, Q-A7, Q-A8).

---

## Part A — `PK4`: Campaign Pressure, Doctrine & Tallow Scarcity

### 1. Cell Doctrine Gauge & Per-Member Lean (`Q-19`, `Q-19a`, `ACTS23` Q-A5)

- **Cell Meter (`sunk_ward/meters.json`)**: Add `doctrine` (`kind: "gauge"`, `min: -100, max: 100, start: 0`, `visible: true` from `campaign.act >= 2`, `order: 3`). Negative leans **Preservationist** (`Berek`), positive leans **Flesh-Broker** (`Sabina`):

| Range | Band ID | Tone | Campaign & Ending Consequences |
| --- | --- | --- | --- |
| `-100..-41` | `ironclad` | `calm` | Unlocks `Ending A` (`Black-Iron Severance`) climax loyalty lines; `Berek` discount (`-10%` repair) |
| `-40..-16` | `preservationist` | `calm` | `Ending A` eligible; `Berek` favors cell, `Sabina` skeptical in Hub dialogues |
| `-15..+15` | `divided` | `warn` | `Ending A` eligible (`Q-19`: A requires non-Flesh-Broker); `Ending B` locked; `Malachi` neutral |
| `+16..+40` | `pragmatist` | `warn` | `Ending A` still eligible (`< +41`); `Malachi` truce preview lines unlock |
| `+41..+100` | `flesh_broker` | `danger` | **`Ending A` refused (`<= +40` required)**; **`Ending B` (`Living Conduit`) unlocked**; `Malachi` mediates `3.3` |

- **Play-Input Triggers (`rpgkit_world` campaign triggers + `recipe` / `battle` events)**:
  - Add `TRIGGER_ON` causes `"recipe"` (carrying recipe `tags` / `doctrine` delta already reserved in `PK2_GRAFTS.md` §4!) and `"battle"` (`strain_peak`, `strain_end`, `channel_actions`).
  - Provisional Deltas: Hearth `masterwork`/`forge_rite` (`-5` Doctrine), Hearth `repair`/`train_proficiency` (`-2`), Slab `salt_flush` (`-2`), Slab Monster/Cranial Surgery (`+4`), Tier III Apex Surgery (`+8`), Bone-Scraping (`-3`), Battle winning with any hero ending at `>= 50` Strain (`+2` per hero), plus Dialogue choices (`Hub Debates 1–4`).
- **Per-Character Lean (`unit_tracks.lean`)**:
  - Integer unit track (`-3..+3`, step via battle Strain threshold / surgery / story choices) controlling per-Founder Hub Debate lines, `3.3 Ysolde` reaction, and Ending epilogue voice cards.

### 2. Act 2–3 Checkpoints, `2.4` Great Tithing Reveal & Act 3 Hard Clock (`CL-04`)

| Checkpoint / Rule | Act | Deadline / Trigger | Miss / Failure Behaviour (`Q-16`, `Q-17`, `Q-20a`) |
| --- | --- | --- | --- |
| `cp_cloisters_24` | `2` | Day `28` (`warn_days: [7,3,1]`) | **`mode: "go_on"`**: Act 2 never hard-fails via checkpoint; missing adds **`+25` Alert**, `-3 saved_youths`, triggers forced Breach countdown escalation (`Q-20a`). |
| `cp_reliquary_26` | `2` | `Lazar` fall + `10` days | **`mode: "go_on"`**: Missing adds `+1` Sentinel Guard & pre-triggered Bell resonance wave in `2.6`. |
| `clock_great_tithing` | `2->3` | Victory at `2.4 Embalming Cloisters` | **`clock_set {meter: "doom_clock", days: max(current, 18), label: "The Great Tithing"}`** (`Lazar` ledger reveal); side quests gated (`unless_flags`) close cleanly; becomes the Act 3 budget. |
| `act3_hard_clock` | `3` | `doom_clock` reaches `0` (`floor 12` min start on Act 3 entry) | **`d_zero` (`acts: [3]`) fires `end: {kind: "fail", card: "extinguished_tithing"}`** (no `14d` restart loop in Act 3); `3.2` Conduit cut executes `meter_add +2`. |

### 3. Act 3 Warded Tallow Scarcity (`Q-45`)

- Add optional `price_mult_by_act: IndexMap<i32, f64>` on `ShopItem` / `ItemDef` (or `ShopManifest::act_price_scale`) in `kits/rpg/` (`P-ARCH-02` no-op default `1.0`).
- Game data (`shops.json` / `items.json`): set `warded_tallow` Act 3 shop price multiplier to **`2.0x`** (scarce Warded Tallow costs double Scrap under Act 3 blockade pressure).

---

## Part B — `PK5`: Hub Facilities (`5 × 3` Tiers) & Connected-Room Alert Sieges

### 4. Facilities Catalog & Level Upgrades (`ACTS23` Q-A8, `Q-42`, `Q-46`)

- **Kit Hook (`rpgkit_data`, `rpgkit_world`, `emberwake_gui`)**:
  - New world/catalog kind `facilities`: `{id, name, keeper_character, levels: [{level: 1..3, name, scrap_cost, materials: {item: count}, requires_flags, effects}]}`.
  - New NPC service `"facility"` (`explore::SERVICES`: shown on keeper's `ServiceScreen` alongside `shop` / `recipe`), persisting `party.facilities: IndexMap<String, i32>` (`1..3`, default `1` in Act 1/upgraded from Act 2).
- **5 Sunk Ward Facilities (`Act 2+`)**:

| Facility ID | Keeper NPC | Level `1` (Act 1) | Level `2` (Early Act 2) | Level `3` (Post-`2.2` / `2.4`) |
| --- | --- | --- | --- | --- |
| `fac_hearth` | `berek` | Scrap salvage & base repair | Unlocks Masterwork drills & `-15%` repair Scrap | Unlocks Cold Steel Keystone rite discount (`-25%` cost) & `+1` durability grace buffer |
| `fac_slab` | `sabina` | Act 1 triage only | Opens Monster Tiers I–II + Cranial (`pk2.slab_open`) | Post-`2.4` Apex Tier III (`-15%` Rock Salt flush cost, Bone-Scraping access) |
| `fac_board` | `talia` | Mission board (`main`/`side`) | Sewer flank prep (`1` extra ambush reveal / `-10%` Alert gain on quiet runs) | Spy ring briber (`-5` extra Alert per Rock Salt bribe, reveals boss phase count) |
| `fac_bunks` | `berek` | `6` Dry Hollows bunks | **`Expanded Timber Frames` (`+2` bunks -> `8`)** (`-20%` Scrap cost if `lucan_rescued`) | Reinforced Smoke-Seals (`-1` Tallow every 2nd rest -> `0` extra bleed penalty on rest) |
| `fac_crypts` | `sabina` | `0` crypt places | Unlocks **`2` Isolation Crypts** (`crypt_only` Living Martyrs + calmer Stage IV rest) | Sacred Quiet (`Stasis days` divisor `5 -> 7` for Martyrs, `+1` passive Strain floor buffer `-5` post-battle reset) |

### 5. Alert `100` Breach: Connected-Room Hub Sieges (`Acts 2–3`, `Q-22`, `Q-23`, `CL-10`)

- **Trigger Wiring (`sunk_ward/meters.json`)**:
  - Keep `a_breach_1` (`acts: [1]`) triggering `end: fail extinguished_breach`.
  - Add `a_breach_23` (`acts: [2, 3]`, `at: "breach"`) triggering **`start_siege {siege: "sluice_siege_act2" | "sluice_siege_act3"}`**.
- **Kit Siege Runner (`rpgkit_world::siege` / `story.rs` multi-encounter queue, `ACTS23` Q-A7(a))**:
  - New world data block `sieges.json`: ordered list of `2–3` linked hub room encounters (`Drain Grate` -> `Water-Lock Lockgate` -> `Vault Hearth`).
  - **Carry-Over Rule**: Between siege rooms, heroes keep exact `HP`, `downed`/`bleed` status (must heal/revive in fight or between-room triage slot!), `Calcification` stage, and reset `Strain` to their graft floor (normal battle end), with **no rest** until the final room clears (`1` emergency pouch re-stock allowed between rooms from stash).
  - **Room Loss & Facility Damage**: Losing any siege room abandons that perimeter room, **reduces its mapped facility level by `1` (floor `1`)**, resets `Alert` to `50`, and shifts remaining rooms or concludes siege recovery (narrative fallback notes temporary relocation to **Scribe's Crypt** in Act 2 or **Weeping Redoubt** in Act 3). Winning all rooms resets `Alert` to **`30`** (Act 2 floor), pays Siege defense salvage, and advances Doctrine/flags.
  - **Act 2 Commander (`Ruthven`, `CL-10`)**: `sluice_siege_act2` Room 3 fields `Inquisitor-General Ruthven` if `ruthven_alive`; phase rule triggers **`ruthven_withdraws`** at **`<= 25%` HP** (ending battle as Defender Victory so `Ruthven` always commands the `2.6` Reliquary Gate climax!).
