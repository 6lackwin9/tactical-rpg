# CAST1 encounters: the slice and the rest of Act 1

Status: locked for Act 1 authoring (`2026-11-11`; updated for `CL-08`, `CL-11`, `PR5-34`, and shipped `rpgkit_battle` `talk` cues); part of [CAST1.md](CAST1.md).
Lore: [CANON.md](../lore/CANON.md) 3.1-3.2, 4.1, [STORY_GUIDE.md](../lore/STORY_GUIDE.md) (`0b` Mortuary Flagellants, `0c`
Castellan Footmen and one Watch Crossbowman), `background/sunk_ward_campaign_map.md`. `ENCOUNTER2` provides `rows`,
`unkillable`, `timeout_outcome`, `stay_behind`, `kill`, and phase `talk` (`{"type": "talk", "dialogue": "dlg_...", "speaker": "..."}`);
`HOOKS1` provides the data `zone` rows (13) and `arena_hazards` (7) (`HOOKS1-Q`'s unbuilt `spare_once` is retired in favour of
`hp_below: 35` + `regen` per `P-COMBAT-03`); `ROSTER1` provides `party_min` (BK6) and `then` `capture`. All numbers FP.

## 1. Shared values (world encounters in `sunk_ward/encounters.json`, CAST1-A6)

`arena` CLOSED; `total_blocks` 6; `ap_regen` 4; `unit_ap` 10 (CANON 4.2); `rng_seed` 1337; `pace` auto; `deployment`
standard; `morale` false unless listed; `start_status` []; `guests`, `protect`, `key_loot`, `arena_hazards` [] unless
listed; `timeout_outcome` "fled" unless listed; `rows` as in section 4 (the encounter's rows win over the zone's,
HOOKS1 ZN3). `party` lists the expected founders (previews and sims); the campaign fights with the active party.
`location` ids come from the M0 skeleton (`alleyway_junction_7`, `ash_crawl_crossroads`,
`fourth_kiln`, `tallow_vats`, `crucible_watchpost`, `sluice_undercroft`; prologue places `alleyway_of_cinders`,
`meat_market_rooftops`, `canal_bridge`; the job's `third_ward_gate`); "" where the skeleton has none yet.

- `zone`: the `zones` rows of HOOKS1_TASKS 2.3 (`alley`, `rooftops`, `canal`, `foundry`, `tallow_vats`,
  `undercroft`, `watchpost`); no backdrop file in the slice, so STAGE1's gradient draws from the row's `ground`
  (backdrop art: CAST1_ASSETS N10, asked first).
- `tags`: `loud` and `quiet` are read by CAMPAIGN1's Alert inputs (CAMPAIGN1_TASKS 4.2: a `loud` win +8, a `loud`
  loss or flight +4, a `quiet` win +2); `main` and `checkpoint` are labels.
- **Act 1 2-Tier Sunk Ward Diamond (`CL-08`)**: After reaching the Sluice Hearth (`0c` -> **Hub Debate 1**), **Tier 1** opens `M1` (*The Sweep*) and `M2` (*The Crossroads Cages*) in either order. Completing both unlocks **Tier 2**—`M3` (*The Bone-Mill Raid*, recruiting `dunstan`) and `M4` (*The Tallow Vats*, recruiting `elyse`) in either order—so whichever recruit is rescued first can deploy in the sibling raid and run their Act 1 side-hook before shutting down all four surface valves forces the **causal `Interlude` marrow back-surge** (`enc_evt_05`) and **`M5` Crucible** (`enc_evt_06`).

## 2. Prologue, Corvo's Gate, M3 & M5 (`enc_pro_0a..0c`, `enc_job_corvo_gate`, `enc_evt_03`, `enc_evt_06`)

| Key | `enc_pro_0a` | `enc_pro_0b` | `enc_pro_0c` |
| --- | --- | --- | --- |
| `name` | The Alley | The Rooftop | The Bridge |
| `zone`, `location` | `alley`, `alleyway_of_cinders` | `rooftops`, `meat_market_rooftops` | `canal`, `canal_bridge` |
| `party` | `kaelen` | `kaelen`, `vesper` | `kaelen`, `vesper`, `maren`, `hollis` |
| `enemies` | `footman_patrol` x2 | `mortuary_flagellant` x3 | `footman_patrol` x3, `watch_crossbowman` |
| `objective` | rout | rout | rout |
| `objective_text` | "Strike while they wind up: a quick Pommel Strike breaks a slow blow." | "Follow the smoke: hide, guard each other, slip away." | "Free the cart's captives and hold the bridge." |
| `exits` | [[11, 3], [11, 4]] | [[11, 4]] | [[11, 4]] |
| `max_rounds` | 8 | 8 | 8 |
| `phases` | `tremor` round 2: line `"The alley shudders as the distant curfew bell tolls. Watch whistles shrill down the lane."` | `chimney_call` round 2: `talk` (`vesper` -> `dlg_talk_0b_smoke`) | `bridge_call` round 2: `talk` (`maren` / `hollis` -> `dlg_talk_0c_cart`) |
| `purse` | {} | {"gold_min": 10, "gold_max": 10, "once": true} | {"gold_min": 15, "gold_max": 15, "once": true} |
| `objective_reward` | {} | {} | {"loot": "loot_poultices", "once": true} |
| Teaches | windup, guard, the Pommel Strike interrupt (T10) | smoke and stealth (Ash-Cloud Fade), the intercept (Intercepting Hook against the Flagellants' rush), disengage (Hamstring Sever) | traps (Tripwire Snare), field suture and the Bile Phial (Cauterize, Bile Coat), the canal as a wall for pushes |

- 0a: Captain Vane gives "Order 44" in the pre-fight dialogue and withdraws (he is not on the board). All Founders start
  Act 1 100% un-implanted (`CANON` 3.3, `CL-08`: Kaelen hides only a faded Lullaby wrist-number under bloodied linen, never
  a glowing forearm spur). The prologue Footmen are `footman_patrol` (hp -30%, power -2) with the tutorial poise 24.
- 0b & 0c `talk` cues (`CL-11`): `rpgkit_battle`'s shipped `talk` phase effect (`{"type": "talk", "dialogue": "dlg_...", "speaker": "..."}`)
  surfaces 1-2 line character-reactive mid-battle callouts when the speaker is alive on the board (silently skipped if absent).
- 0c: Maren and Hollis are freed in the pre-fight dialogue at the cart and fight as members (a guest is AI-driven, so
  the player could not practise their skills); their join flags are the dialogue's (RECRUIT1). A push into the canal
  is a collision, not a fall (HQ9 a).

| Key | `enc_job_corvo_gate` (the dilemma's fight, CQ9) | `enc_evt_03` (M3) | `enc_evt_06` (M5) |
| --- | --- | --- | --- |
| `name` | Corvo's Gate | The Bone-Mill Raid | The Crucible |
| `zone`, `location` | `alley`, `third_ward_gate` | `foundry`, `fourth_kiln` | `watchpost`, `crucible_watchpost` |
| `party` | active party (`4` deployed) | active party (`4` deployed; includes `elyse` if `M4` ran first) | active party (`4` deployed from `6` Founders) |
| `party_min` | 0 | 0 | 4 (ROSTER1 BK6) |
| `enemies` | `castellan_footman` x2, `watch_crossbowman`, `bailiff_enforcer` | `overseer_grendel`, `tallow_dredger` x2, `bailiff_enforcer` | `inquisitor_prelate_nemeth`, `inquisitorial_purifier` x2, `castellan_footman` x2, `watch_crossbowman` x2 |
| `guests` | - | `dunstan` | - |
| `objective` | rout | rout | escape, `objective_count` 3 |
| `objective_text` | "Force the Third Ward gate." | "Wreck the mill and free the chain gang." | "Open the Slag Gate: one holds the winch, three get out." |
| `exits` | [[11, 3], [11, 4], [11, 5]] | [[11, 4]] | [[11, 3], [11, 4], [11, 5]] |
| `max_rounds` | 8 | 10 | 8 |
| `phases` | `corvo_call` (`talk` R2) | `mill_oath` (`talk` R1 `dunstan`), `pulverizer` (CAST1_BOSSES 3.3) | `gate_reinforcements` (`R3`, or `R4` if `sunk_ward.hollis_overlook`), `infusion`, `gate_closing` (3.6) |
| `arena_hazards` | - | [`slag_vents`] (below) | - |
| `unkillable`, `timeout_outcome` | -, fled | -, fled | [`inquisitor_prelate_nemeth`], lost |
| `stay_behind` | {} | {} | see below |
| `purse` | {"gold_min": 20, "gold_max": 20, "once": true} | {"gold_min": 50, "gold_max": 50, "once": true} | {} |
| `key_loot` | - | [{"loot": "loot_slag_hammer", "kill": "overseer_grendel"}] | - |
| `objective_reward` | {"loot": "loot_poultices", "once": true} | {"loot": "loot_slag_plating", "once": true} | {} |
| `tags` | ["loud"] | ["main", "loud"] | ["main", "checkpoint"] |

- Corvo's gate (CQ9 a): the board job offers a bribe (Scrap/Salt, quiet, a dialogue choice) or this fight (loud: the
  Alert input above, and Bailiff Corvo turns cold: flag `sunk_ward.corvo_forced` set by the job's dialogue, risking his
  `16`-yo sister `Ida` on a lesser tithing). Corvo is not on the board.
- M3: Dunstan is a guest (AI-driven) so the player sees his kit; he joins after the win (bunk 5 or 6, ROSTER1 BK1; join
  level `average`, BK5; flag `sunk_ward.dunstan_joined` by the after-fight dialogue's `recruit`), unlocking his Act 1
  side-hook (*"Chain-Gang Wedges"* -> `+1` `Rock Salt` + `Skerrit, Bone-Wright` barbed cestus/quarrel recipes).
- M3 `arena_hazards` (HOOKS1 7; HQ8 a: vents only, belts later as data):

```json
[{"id": "slag_vents", "name": "Slag Vents", "columns": [7, 8], "rounds": [], "blocks": [3, 6],
  "effect": {"damage_pct": 12, "stagger": 10, "status": "burn", "potency": 60, "magnitude": 3, "status_rounds": 1},
  "line": "The slag vents roar."}]
```

- The vents strike every unit on columns 7 and 8 at the end of blocks 3 and 6 of every round (both sides; a vent into
  a windup can break it, AH2); the battle screen marks the columns each round (AH3), so section 4's M3 board has no
  `f` vent tiles. Grendel's slag pools (`L`) stay blocked tiles: a push into them is a collision (HQ9 a).
- M5 `stay_behind` (ENCOUNTER2 3.1; Q-75a a): `{"mode": "hold", "tiles": [[10, 1]], "tiles_label": "the winch",
  "fallback": "any", "pick_default": "highest_hp", "flag": "sunk_ward.crucible_stayed", "then": "capture",
  "result_line": "{name} held the winch."}`. `then` `capture` departs the founder through ROSTER1's lifecycle
  (ENCOUNTER2-W revised); the founder's character `keepsake` (`keepsake_<id>`) is paid to the stash at once (ROSTER1 KS2).
  `party_min` 4 is CANON 4.1's "the Crucible needs four deployed". Nemeth is `unkillable`, so "no active foe" cannot
  win first. If Hollis completed *"The Crag-Sight Overlook"* (`sunk_ward.hollis_overlook`, `CL-08`), the outer chimney
  counterweight drops onto the Watch muster yard (`enc_evt_06_overlook` quest variant), delaying `gate_reinforcements`
  from Round `3` to Round `4`.
- The hearth scene (**Hub Debate 1**) and post-`M5` wake (**Hub Debate 3**) are sequences/vignettes (not encounters).

## 3. Tier 1 (`M1`, `M2`), Tier 2 (`M4`) & Causal Interlude (`enc_evt_01`, `enc_evt_02`, `enc_evt_04`, `enc_evt_05`)

| Key | `enc_evt_01` (M1, Tier 1) | `enc_evt_02` (M2, Tier 1) | `enc_evt_04` (M4, Tier 2) | `enc_evt_05` (Causal Interlude) |
| --- | --- | --- | --- | --- |
| `name` | The Sweep | The Crossroads Cages | The Tallow Vats | The Awakening Fissure |
| `zone`, `location` | `alley`, `alleyway_junction_7` | `alley`, `ash_crawl_crossroads` | `tallow_vats`, `tallow_vats` | `undercroft`, `sluice_undercroft` |
| `enemies` | `captain_vane`, `castellan_footman` x2, `watch_crossbowman` | `proctor_quell`, `inquisitorial_purifier`, `mortuary_flagellant` x2 | `guild_master_orlov`, `render_beast` x2, `tallow_dredger` x2 | `calcified_amalgam`, `ash_crawler` x3, `calcified_husk` x2 |
| `guests`, `protect` | - | `tithe_captive_a`, `tithe_captive_b`; none protected | `elyse`; none | - |
| `objective` | rout | rout | rout | rout |
| `objective_text` | "Break Vane's sweep before the grates are sealed." | "Free the cages before the flagellants cull them." | "Cut through the vats; find the chemist." | "Drive the marrow surge back into the fissure." |
| `exits` | [[11, 4]] | [[11, 4]] | [[11, 4]] | [[0, 4]] |
| `max_rounds` | 8 | 8 | 10 | 10 |
| `morale` | true; `morale_leader` `captain_vane`; `morale_break` 50 | false | false | false |
| `phases` | `vane_taunt` (`talk` R1 `kaelen`), `execution_wall`, `vane_withdraws` (`CL-01`) | `quell_taunt` (`talk` R1 `maren`), `cull_1`, `cull_2`, `marrow_valve` (`hp_below: 35`) | `orlov_taunt` (`talk` R1 `vesper`), `cataclysm`, `orlov_withdraws` (`CL-01`) | `surge_breach` (`talk` R1), `bells`, `bloom` |
| `purse` | 30 | 25 | 100 | 0 |
| `key_loot` | [{"loot": "loot_gate_pass", "kill": "castellan_footman"}] | [{"loot": "loot_marrow_scalpel", "kill": "proctor_quell"}, {"loot": "loot_marrow_valve", "kill": "proctor_quell"}] | [] | [{"loot": "loot_marrow_core", "kill": "calcified_amalgam"}] |
| `objective_reward` | {"loot": "loot_gilded_pavise", "once": true} | {"loot": "loot_slab_retorts", "once": true} | {"loot": "loot_flensing_talon", "once": true} (plus quest `loot_orlovs_retribution` on M4 completion) | {"loot": "loot_dreamers_vertebra", "once": true} |
| `tags` | ["main", "loud"] | ["main", "loud"] | ["main", "loud"] | ["main"] |

Purse values are `{"gold_min": n, "gold_max": n, "once": true}` (Scrap is the purse's currency name, CAMPAIGN1 P1).

- M2 captives (`CQ6 a`, `CL-08`): world characters `tithe_captive_a` "Caged Unsealed" and `tithe_captive_b` "Caged Unsealed"
  (role npc) on catalog unit `tithe_captive` (side party, class rogue, level 1, tier minion, gear all `none`,
  `flee_ratio` 1.0, `deck` {}); adults `18-24` by the tone rule. Each captive alive at the win (`cull_1` round 3, `cull_2`
  round 5) adds `+1` to `saved_youths` (`youths_saved`) **and awards `+1` `Rock Salt`** in the post-fight dialogue as
  Maren decodes their cage seals (`CL-08`, unlocking *"Liturgical Cipher Rubbings"*). Proctor Quell's brass neck-valve
  triggers cleanly at `hp_below: 35` (`or_round: 4`) without immortality/self-heal stalls (`CAST1_BOSSES` 3.2, `P-COMBAT-03`).
- M4: Elyse is a guest; she joins after the win (bunk 5 or 6, join level `average`), unlocking her Act 1 side-hook
  (*"The Tenement Counter-Still"* -> `+5` `Warded Tallow` + `1` early prototype `Snapped Alum Inhaler`). Orlov's vendetta
  (`CL-01`, `PR5-34`): Orlov retreats at low HP (`orlov_withdraws`, `hp_below` 20) so he survives into Act 2 for Vesper's
  (or Talia's inherited) loyalty quest, leaving behind `loot_flensing_talon` (`objective_reward`) while the M4 quest
  turn-in awards `loot_orlovs_retribution` and `+5` `Warded Tallow`. The grease (`toxic_bog`) tiles stand in for Orlov's
  boiling-fat floor.
- Causal Interlude (`CL-08`, `CL-11`): Shutting down `M1`–`M4`'s Watch grates, titration cages, bone-mill chutes, and
  rendering vats backs raw abyssal condensate down the blocked drain-lines until it ruptures straight into the Sluice
  Undercroft. Pre-fight dialogue records which Founder takes **Chirurgeon Sabina**'s emergency raw marrow dose in
  `sunk_ward.sabina_dose` (carrying Stage I `Vein-Blackening` + free Tier I implant head-start into Act 2) and triggers
  **Hub Debate 2** (`Sabina` vs. `Berek` vs. `Talia`) after the fight.
- Vane's & Orlov's final reckonings are Act 2 personal/mentor-inherited quests (`CL-02`, `CL-09`).

## 4. Terrain rows (`rows`, ENCOUNTER2 R0)

Glyphs (`grid.rs`): `.` floor, `o` stone, `r` rubble, `#` wall, `=` bridge, `W` deep water, `L` lava, `f` burning
ground, `p` toxic bog. Each block lists y0-y2, y3-y5, y6-y8 side by side; every row is 12 glyphs; exits and the hold
tile are walkable.

```text
enc_pro_0a     y0 ############  y3 #...........  y6 #.......r..#
               y1 #..r.......#  y4 #...........  y7 ############
               y2 #......r...#  y5 #...r......#  y8 ############
enc_pro_0b     y0 oooooooooooo  y3 oooooorooo#o  y6 ooroo##ooooo
               y1 oo#ooooo##oo  y4 o##ooooooooo  y7 ooooooorooo#
               y2 oo#ooroooooo  y5 oooooo#ooroo  y8 oooooooooooo
enc_pro_0c     y0 WWWWWWWWWWWW  y3 ======##====  y6 ============
               y1 WWWWWWWWWWWW  y4 ============  y7 WWWWWWWWWWWW
               y2 ============  y5 ============  y8 WWWWWWWWWWWW
enc_job_corvo  y0 ############  y3 #...........  y6 #.r..#..r..#
_gate          y1 #....#.....#  y4 #...........  y7 #....#.....#
               y2 #.r..#..r..#  y5 #...........  y8 ############
enc_evt_01     y0 ####..######  y3 #..........#  y6 #.r...#....#
               y1 #..r..#....#  y4 ..r.....r...  y7 #.....#..r.#
               y2 #.....#..r.#  y5 #..........#  y8 ####..######
enc_evt_02     y0 #....##....#  y3 #....##....#  y6 #...r..r...#
               y1 #.f......f.#  y4 .....##.....  y7 #.f......f.#
               y2 #...r..r...#  y5 #....##....#  y8 #....##....#
enc_evt_03     y0 ############  y3 #..........#  y6 #..LL......#
               y1 #..LL....r.#  y4 #.r.........  y7 #..LL......#
               y2 #..LL......#  y5 #.........r#  y8 ############
enc_evt_04     y0 LLLLLLLLLLLL  y3 ===..LL..pp.  y6 L.r..LL..pp.
               y1 L....==....L  y4 ...........=  y7 L....==....L
               y2 L.r..LL..pp.  y5 ===..LL..pp.  y8 LLLLLLLLLLLL
enc_evt_05     y0 ############  y3 #.....pp...#  y6 #.#.....#..#
               y1 ##...p....##  y4 ...........#  y7 ##....p...##
               y2 #..#.....#.#  y5 #.....pp...#  y8 ############
enc_evt_06     y0 ############  y3 #.....#.....  y6 #..f..#....#
               y1 #..........#  y4 #...........  y7 #..........#
               y2 #..f..#....#  y5 #.....#.....  y8 ############
```

What the glyphs stand for: 0a's rubble are crates (half cover through `cover_guard`); 0b's walls are chimneys; 0c's
walls are the iron mortuary cart and the canal is deep water (a wall for pushes); M2's centre walls are the cage wagon
and `f` the burning lanterns; M3's lava is the slag pools and columns 7-8 the vent line (the arena hazard, no glyph);
M4's lava is the boiling vats, `=` the catwalks and `p` Orlov's grease; the interlude's walls are bone pillars and `p`
the violet mist; the Crucible's `f` are braziers, the wall column at x 6 the inner gate, [10, 1] the winch.

## 5. Board checks for the asset lane

| Check | Expectation |
| --- | --- |
| Rows | 9 strings of 12 known glyphs (ENCOUNTER2 R0 check) |
| Deployment | the party's and the enemies' deployment tiles are walkable: if `content check` or the Arena reports a blocked start tile, move one rubble or wall glyph and note it in the task report |
| Exits | every listed exit is walkable; M5's hold tile [10, 1] is walkable, not an exit |
| Unit count | enemy side plus phase waves at most 12 (M1: 4 + 2; M5: 7 + 2) |
| Phases | at most 4 boss phases plus one arrival (M2: 3; M3: 1; M5: 2 + arrival) |
| Zones | every `zone` is a code zone or a `zones` row (HOOKS1 ZN2); the slice's are all rows |
| Arena hazards | M3's columns 7 and 8 are on the board; blocks 1-6; one row (at most 4, HOOKS1_TASKS 1.6) |
| Crucible | `party_min` 4 is at most the world's `party_max` (ROSTER1 BK6) |
