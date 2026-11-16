# `PK3_ROSTER_LIFECYCLE` — `[PROVISIONAL]` Stage II–III Passives, Bleed-Out/Revive, Recruits & Keepsakes (`PR6-2..PR6-4`)

- **Status**: **`[PROVISIONAL]`** — Wave 2 `.1` spec extending `ROSTER1` (`docs/archive/specs/ROSTER1.md`) and `PK2_GRAFTS.md` (§2) for ready-to-build slices `PR6-2`, `PR6-3`, and `PR6-4`.
- **Parent Plan**: [WAVE2_PLAN.md](WAVE2_PLAN.md) | **Canon Authority**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§2.2, §3.2, §4.2, `CL-07`).

---

## 1. `PK2-STAGE-PASSIVES`: `calcify_2` & `calcify_3` (`PR6-2`)

In `games/emberwake/data/catalog/unit_tracks.json`, `calcification` stages `1` (`calcify_1`) and `4` (`calcify_4`) carry passives, while stages `2` (*Calloused Chitin*) and `3` (*Sensory Rot*) are blank (`FB-221(a)`). Both stages become live before Act 2 Calcification builds ship (`PR6-2`).

### 1.1 Story-Free Kit Hooks (`kits/rpg/`, `0` Lore Words)

- **`Target::Armor` (`stat: "armor"` or effect `{"type": "armor", "value": N}` -> prefer extending `simple::TYPES` / `STATS` cleanly with `"armor"` / `"move_rate"` as flat `Stat` or dedicated `Target::Armor` + `Target::MoveRate`)**:
  - `Target::Armor`: summed in `rpgkit_rules::damage::base_armor_rating(u)` alongside `gear` and `mitigation_rating`, clamped `>= 0`.
  - `Target::MoveRate`: summed in `rpgkit_rules::unit::Unit::move_per_block(&self)` as `(self.move_rate + self.passive_move_rate + hasted - slowed).max(1)`.
- **`"ranged"` Group + Optional `min_distance` Filter on `skill_potency`**:
  - Add `"ranged"` to `rpgkit_content::check::simple::GROUPS` (`["spell", "heal", "channel", "ranged"]`).
  - `rpgkit_rules::unit::skill_in_group(skill, "ranged")` matches `skill_is_ranged` (already factored in `PR5-62a`).
  - Add optional whole key `min_distance` (`0..12`, default `0`) on `skill_potency`; when `min_distance > 0`, `damage::compute` applies the `skill_potency` value only when hit tile Manhattan/Chebyshev range `>= min_distance` (`min_distance: 4` = `"beyond 3 tiles"`).

### 1.2 Catalog Passive Data (`passives.json` + `unit_tracks.json`)

| Stage | Passive ID | Name (`PK2_GRAFTS.md` §2) | Exact `effects` Spec (`slot: ""`) |
| --- | --- | --- | --- |
| `2` | `calcify_2` | Calloused Chitin | `[{"type": "stat", "stat": "armor", "value": 2}, {"type": "stat", "stat": "move_rate", "value": -1}]` (`move_per_block` floor `1`) |
| `3` | `calcify_3` | Sensory Rot | `[{"type": "resist", "status": "weaken", "value": 100}, {"type": "resist", "status": "stun", "value": 100}, {"type": "skill_potency", "group": "ranged", "min_distance": 4, "value": -0.15}]` |

---

## 2. `PR5-11` (`PR6-3`): Bleed-Out & Revive in Battle (`Act 2+`)

Existing schema `RosterRule::bleed_out_rounds` (`rpgkit_data/src/lib.rs:1672`, checked `0..6` in `check/world.rs:843`) is parsed on `sunk_ward` Acts `2..3` (`roster.rules`) but not yet wired in `rpgkit_battle`.

### 2.1 Rules & Edge Cases (`[kit hook]` `rpgkit_battle`, `emberwake_gui`)

- **Battle Init (`Session::bleed_out_rounds`)**: World battle launcher (`rpgkit_world::battle`) passes active rule row's `bleed_out_rounds`; `Dojo`, `sim`, and `ashlands`/`iron_reaches`/`tidewrack` default to `0` (`0` golden diffs).
- **Downed Bleed Timer**: When `bleed_out_rounds > 0`, any deployed party `member` (never guests, never enemies) dropping to `0 HP` enters `downed` with `bleed_remaining = bleed_out_rounds` instead of immediate non-lethal stasis. At each `round_end` after status ticks, `bleed_remaining` decrements by `1`; reaching `0` transitions fate to `"dead"` (permanently non-revivable in this battle, battle log `"{name} bleeds out."`, departed `cause: "bled_out"` on settle).
- **Occupied Cell Resolution on Revive (`FB-167` Q9 Fix)**:
  - A `SUPPORT` heal skill (`rally` `revive: true`) or healing consumable (`smelling_salts` `revive: true`) may target a `downed` (`bleed_remaining > 0`) ally cell even if another living unit currently stands on that tile.
  - On resolve: if the downed ally's tile is free, revive in place; if occupied, search Chebyshev radius `1..=2` around the downed cell (nearest passable unoccupied floor tile sorted by deterministic distance -> `(y, x)` tie-break) and place the revived ally there. If all cells within radius `2` are blocked/impassable, planner/execution refuses cleanly with `"Space is blocked"`.
  - Revived unit clears `bleed_remaining`, gains `max(heal_amount, 1)` HP (`clamp(1, max_hp)`), resumes normal turns from the next block (`AP` preserved from round start less already spent; prior cancelled plan stays cleared).
- **Inspect & Tell UI (`emberwake_gui/src/battle/`)**: Downed party unit tooltip/banner shows `"Bleeds out in {n} round(s)"` cleanly (`dim`/`danger` tone when `n == 1`).

### 2.2 Act 2 Game Data (`[data]` via `aegis content`, `[PROVISIONAL]`)

- `sunk_ward/world.json`: keep Act `0..1` row `bleed_out_rounds: 0`; set Act `2..3` row `bleed_out_rounds: 3`, `downed_won: "wounded"`, `downed_left: "captured"`, `downed_lost: "dead"`, `standing_lost: "wounded"`, `keepsakes_on: ["captured", "dead"]`.
- `skills.json`: add `"revive": true` to Priest `L2` skill `rally` (*Urgent Triage*, `SUPPORT ALLY`, `2 AP / 2 blk`, `1–3`) — Act 1 behaviour unchanged (`0` diffs), Act 2 gives Priest vital rescue agency (`P-COMBAT-04`).
- `items.json` / `shops.json`: add **`smelling_salts`** (*Brimstone Smelling Salts*, `COMMON` pouch `consumable`, `ap_cost: 1, blocks: 1, reach: 1`, `heal: 15, revive: true`, `15 Scrap`), stocked at `sabina_slab` and `varka_trade` from Act 2 (`pk2.slab_open`).

---

## 3. `PR5-14` (`PR6-4` Pt. 1): Procedural Recruit Generator, Pool, Bunks & Living Martyrs

### 3.1 Kit Additions (`rpgkit_data`, `rpgkit_world`, `rpgkit_content`)

- **Member & Party Fields (additive save keys under unreleased v13, `0` Godot golden impact)**:
  - `UnitState`: add optional `name: String`, `housing: "bunk" | "crypt" | "crypt_only"` (default `"bunk"`), `fuse: Option<MartyrFuse { value: i32, deploys: i32, since: i32 }>`, and `origin: String` (slotless trait passive ID).
  - `PartyState`: add `pool: Vec<PoolMember>` (`{id, character, base_unit, name, origin, temperament, since}`), tracked by `roster.pool` read key (`roster.rs:723`).
- **Effects `rescue` & `recruit` Extension**:
  - `rescue {template | character, base_unit?, count}` generates/pushes pool entries using seeded `rolls` stream `"recruit"` (guarantees names never collide with `members`, `pool`, or `departed`).
  - `recruit {pool: "<id>" | "first", level?}` swears a pooled recruit into `party.members` (refused if `!roster.open` `"The cell takes no one new yet."` or `bunks_used >= bunk_cap` `"Every bunk is taken."`). Also adds `roster::retire(id)` / `roster::dismiss(id)` (`depart` returning gear to stash).
- **World Kind `world.recruits` & `Isolation Crypts` Housing**:
  - Schema `recruits.json`: `{id, class_id, base_units, names, age_band, art, origins, temperaments, level, fuse?}`.
  - `roster.crypts` (`sunk_ward`: `0` in Act 1 -> `2` from Act 2 facility upgrade or `pk2.slab_open`): operatives with `housing: "crypt_only"` (`Living Martyrs`) occupy crypt slots and take `0` bunks; Stage IV operatives with `housing: "crypt"` sleep in crypts while retaining their bunk (`PK2_GRAFTS` §2, `ROSTER1` §5.1).
  - **Living Martyr Fuse (`ROSTER1` §8)**: `fuse.value = deploys * per_deploy + floor((day - since) / stasis_days)`. Tiers grant cumulative passives (`at: 2, 4`). At `value >= max` (`6`), next deployment prompts **Retire (`departed` `"retired"`)** or **Last Stand (`deploys` with `last_passive` Calcification V stats, `depart` `"dead"` at battle settle)**.

### 3.2 Act 2 Sunk Ward Cohort Templates (`[PROVISIONAL]`, `CANON.md` §3.2 & `CL-07`)

All Act 2 procedural recruits are **`18–24`** static Unsealed operatives wearing visors/cowls/scarves/goggles (never cloning barefaced Founders):

| Recruit Template ID | Cohort Archetype (`CANON.md` §3.2) | Class / Base Unit | Origin Passive (`slot: ""`, `~+1` stat flavor) | Shared Temperament Pool |
| --- | --- | --- | --- | --- |
| `cohort_levy` | Turncoat Watch Levy | `warrior` | `orig_watch_shield` (`max_poise +6`) | `steady`, `wary`, `grim` |
| `cohort_bolt` | Outwall Bolt-Man | `ranger` | `orig_crag_eye` (`PERC +1`) | `wary`, `steady`, `quiet` |
| `cohort_flue` | Ash-Crawl Flue-Runner | `rogue` | `orig_soot_step` (`resist slow +20`) | `reckless`, `quiet`, `wary` |
| `cohort_novice` | Apostate Charnel-Novice | `priest` | `orig_mortuary_calm` (`KNOW +1`) | `steady`, `grim`, `wary` |
| `cohort_breaker` | Kiln Chain-Breaker | `brawler` | `orig_slag_grip` (`STR +1`) | `reckless`, `grim`, `steady` |
| `cohort_vat` | Vat-Still Saboteur | `mage` | `orig_alum_nose` (`INT +1`) | `quiet`, `reckless`, `steady` |

---

## 4. `PR5-15` (`PR6-4` Pt. 2): Procedural Keepsakes (`keepsakes.by_class`)

- **Kit (`rpgkit_world`)**: Enable `RosterManifest.keepsakes_by_class` and `keepsake_name` (`"{name}'s {item}"`, `lib.rs:1726-1728`). When a procedural member departs with a `life` matching `keepsakes.on`, mint an instance `{id: "keepsake@<member>", item: template_id, name}` into `party.keepsakes` instead of stash items. `roster::item_def` resolves it as a wear-legal `ACCESSORY` (`value: 0`, `special: true`, non-sellable, duplicate-template guard `1` per unit).
- **Catalog Data (`items.json` + `sunk_ward/world.json`, `ROSTER1` §5.3)**: Seed `6` keepsake templates (`~50%` of named Founder keepsakes) bound to `sunk_ward.roster.keepsakes.by_class`:
  - `warrior`: `keepsake_tpl_warrior` (*Shield Strap*: `CONS +1`, `resist stun +10`)
  - `ranger`: `keepsake_tpl_ranger` (*Fletching Charm*: `PERC +1`, `resist root +10`)
  - `rogue`: `keepsake_tpl_rogue` (*Lockpick Ring*: `DEX +1`, `resist slow +10`)
  - `priest`: `keepsake_tpl_priest` (*Prayer Cord*: `KNOW +1`, `resist poison +10`)
  - `brawler`: `keepsake_tpl_brawler` (*Knuckle Wrap*: `STR +1`, `resist stagger +10`)
  - `mage`: `keepsake_tpl_mage` (*Ash Vial*: `INT +1`, `resist burn +10`)
