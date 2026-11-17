# `PK6_PK7_BOSSES_ENDINGS` — `[PROVISIONAL]` Boss Tech, Resonance, Dream Retries & Endings A/B/C (`R7`)

- **Status**: **`[PROVISIONAL]`** — `ACTS23-R7` feasibility audited and rulings **`B1` Option B** + **`B7`** integrated and **accepted by the user 2026-11-17** (decision "Act 2-3 feasibility rulings B1-B7"); queued in `READY.md` §1 as `PR6-7`, `PR6-10`, `PR6-11`.
- **Parent Plan**: [WAVE2_PLAN.md](WAVE2_PLAN.md) | **Canon & Decisions**: [CANON.md](../../games/emberwake/docs/lore/CANON.md) (§3.2–3.3, §4.1, `CL-05`, `CL-09..11`), [ACTS23.md](../specs/ACTS23.md) (§2 #19–24, `FB-170`, `FB-171`).

---

## Part A — `PK6`: Boss Mechanics & Act 3 Battle Extensions

### 1. Battle-Start & Hazard Meter Resonance (`CL-05`, `ACTS23` §2 #21, `B7`)

- **Battle-Start Regional Strain (`world` / `region` / `encounter` hook)**:
  - Add optional `start_meter: [{meter, value, requires_calcified?: bool, unless_flag?}]` on `regions.json` / `encounters.json`.
  - Applied in `rpgkit_world::battle` after `graft_floor` (`battle::meter::battle_start` on `meter >= 1`, `0` for `Stage 0` Cold Steel purists!):
    - **Act 2 `high_bastion` (St. Lyra's Curfew Bells)**: `+10` start Strain on Calcified operatives (`calcification >= 1`) until `pk6.belltower_cleared` (`2.5 Penitent Bell-Warden`).
    - **Act 3 `mantle_fissure` (Mantle Resonance)**: `+15` start Strain on Calcified operatives (`halved -> +8` when `pk6.chorus_halved` after `3.3 Ysolde`).
- **Arena & Ground Hazard Meter Gate (`rpgkit_battle::arena.rs` & `ground.rs`, `B7`)**:
  - Extend `ArenaHazard` / ground hazard `effect` with optional **`when_meter: {id, at_least}`** (`when_meter: {id: "strain", at_least: 50}`), **`every_rounds`** periodic cadence, **`meter: {meter: "strain", delta: N}`**, and the **`ground.rs:408`** meter-only outcome fix:
    - Act 2 `Bell Toll` / Act 3 `Mantle Pulse` (`every_rounds: 2`, firing on even rounds `round % 2 == 0`): hits any unit at `strain >= 50` for `10% max HP` resonance damage + `+10 Strain` (giving `Vent` at `50+` crisp counterplay before pulse rounds!).

### 2. Node `20` (`3.4` Silent Ossuary) Crucible Victim Climax (`FB-170` / `FB-171`, `Q-94`, `B1` Option B, `B7`)

Reuses already-landed **`PHASEFX1`** (`PR5-2`: `side: hostile_all` and `swap` in place) with two `[PROVISIONAL]` rulings (**`B1` Option B** + **`B7`**):

- **Crucible Lookup Spawn (`B7`: lore-free `kits/rpg/` token **`"$captive_unit"`**)**:
  - Reads `GameState.captive` snapshot (built from `party.departed` where `life == "captured"` and `cause == "stayed"`, with deterministic static fallback for standalone battle tests so `kits/rpg/` holds zero lore terms).
  - Builds Phase 1 unit (`Saint-Weapon <Name>`) inheriting that captured Founder's class visuals/skills converted to marrow versions + `Strain` meter (`+25%` Phase-2 stats if `sunk_ward.sabina_dose_to == departed.id`, honoring `PK2_GRAFTS.md` §3!).
- **`B1` Option B — `1x1` Cell + Oversized `2x2` Sprite Art (`BIGUNIT1` Parked in `READY.md` §3)**:
  - Keep `1x1` cell occupancy in `rpgkit_battle` so `PHASEFX1` (`swap`, `hostile_all`, pathing, flank checks) works untouched; render `saint_aberration_<class>` (`Node 20`) and `Dreaming Core` (`Node 22`) visually as oversized `2x2`-footprint sprites.
- **Encounter Sequence (`enc_34_silent_ossuary`, fighting `Nemeth` + captured `Founder`)**:
  1. **Phase 1**: Party vs `Nemeth` + `Phase 1 Captive` + `2` Inquisition guards.
  2. **Phase 2 Trigger (`Nemeth` `hp_below: 50` or `round: 4`)**: `PHASEFX1` triggers **`side: hostile_all`** on `"$captive_unit"`, then **`swap` `{to: "saint_aberration_<class>", hp: "full"}`** (`1x1` occupancy w/ oversized `2x2` sprite art), and `talk` cue (`"It broke the yoke!"`) — the broken Saint-Weapon attacks `Nemeth` and party alike (`Nemeth` falls or turns vulnerable; slaying both drops `Nemeth's Warded Cassock` + `Transcendent Relic` upgrading the fallen founder's `keepsake_<id>`!).

### 3. `3.5 High Altar -> 3.6 Dreaming Core`: Altar-Lip Camp Prep & Dream-Echo Retry (`CL-11`, `Q-21a`, `B1` Option B)

- **`camp_prep` Story Step (`rpgkit_world::story.rs` + `emberwake_gui/src/story.rs`)**:
  - Warned modal before advancing `3.5 High Altar` (`Othemar`) into `3.6 Dreaming Core`: `"Beyond the cracked Altar there is no return to the Sluice. Prepare gear, grafts, and pouches?"`
  - Opens a one-time **`StoryCampPrep`** screen (lets player edit party active 4, repair/equip gear/pouches, slot passives, save, and take `1` free non-clock-ticking camp rest without unlocking world map travel).
- **Dream Retry Cost & Fallen Echoes (`story::Step::Fight` extensions)**:
  - Add optional `retry_day_cost: i32` (default `0`; `2` on `3.6`) and `echo_fallen: bool` (default `false`; `true` on `3.6`).
  - Losing `3.6` heals party to full (`retry`), advances `campaign.day` by `retry_day_cost` (`2` days via `campaign::tick(days(n, rest))` — if Doom Clock hits `0`, `act3_hard_clock` fires `extinguished_waking_collapse` `end: fail`!), and on next attempt injects up to `2` **`dream_echo`** shadow units mirroring `party.departed` (`captured`, `dead`, `lost`) alongside `Dreaming Core` (`1x1` cell + `2x2` oversize art)!

---

## Part B — `PK7`: Campaign Endings (`A / B / C`) & Epilogue Screen

### 4. Kit Ending Hook (`rpgkit_data::END_KINDS`, `campaign`, `story`)

- Extend `END_KINDS` (`lib.rs:1381`) from `["fail", "bad_end"]` to **`["fail", "bad_end", "ending"]`**, adding optional `ending: String` (`id` of row in new world kind **`endings.json`**) on `"end"` effect / dialogue choice / story node.
- **`endings.json` Row Schema**: `{id, title, tone, art, requires_flags, requires_meter: [{id, op, value}], epilogue_paragraphs, world_outcomes: [{flag, text_if_true, text_if_false}], member_fates_template}` (`Shroud` remains unbroken across all endings per `CANON.md` §1.1 `[LOCKED]`).
- When `{effect: "end", kind: "ending", ending}` runs, `Campaign::over` records `"completed": true` + winning `ending` ID, opening the **`EpilogueScreen`** (replacing gameplay screens cleanly alongside `runover.rs`).

### 5. Locked Emberwake Endings (`3.6` Climax, `CANON.md` §4.1 & `Q-19`)

| Ending ID | Name & Gate (`CANON.md` §4.1) | Choice Owner & Narrative Outcome (`[LOCKED]` Shroud Never Breaks) | Epilogue Slides Rendered (`emberwake_gui`) |
| --- | --- | --- | --- |
| `ending_a_severance` | **Black-Iron Severance** (`doctrine <= 40` / Preservationist–Pragmatist) | `Berek` drives black-iron spikes into the waking Core's brainstem. City survives; geothermal warmth fails, raw marrow dries up, and all installed bone-pins petrify harmlessly. | Doctrine slide + Hub casts (`Berek`, `Sabina`, `Talia`, `Ivo`) + `saved_youths` tally + per-operative outcome card (`party.members` + `departed`, Final Calcification stage, `keepsakes`). |
| `ending_b_conduit` | **The Living Conduit** (`doctrine >= 41` / Flesh-Broker only) | `Sabina` binds grafted wardens into the Core's crown to lull the titan back to sleep without child Tithings — keeping geothermal warmth and marrow alive at grave moral/corruption cost. | Same 4-section Epilogue structure highlighting Conduit throne cost, `Sabina`'s ascension/vigil, and Martyr fates. |
| `ending_c_martyrdom` | **Stone Martyrdom** (always available fallback) | Squad ossifies the wounded Core from within, petrifying both the Core and the active strike team into black-stone pillars propping up Cinderreach (`departed` cause `"story"`). | Same 4-section Epilogue honoring the fallen active 4 alongside any Sluice survivors (`Berek`, `Talia`, `Ivo`, benched recruits). |

- **`Extinguished` Fail Screen Integration (`runover.rs`)**: Ensure all 3 loss paths (`extinguished_breach` Act 1 Alert `100`, `extinguished_tithing` Act 3 Clock `0`, `extinguished_waking_collapse` `3.6` retry clock `0`) keep clear cause cards and return cleanly to Act Start load / Save load / Main Menu.
