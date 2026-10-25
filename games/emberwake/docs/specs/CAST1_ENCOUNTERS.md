# CAST1 encounters: the slice and the rest of Act 1

Status: proposed (2026-10-25); part of [CAST1.md](CAST1.md). Lore: CANON 3.1-3.2,
`systems/bestiary_encounters_act1.md` (rosters proposal; Q-73 a: 0b Mortuary Flagellants, 0c Castellan Footmen and
one Watch Crossbowman), `background/sunk_ward_campaign_map.md`. ENCOUNTER2 provides `rows`, `unkillable`,
`timeout_outcome`, `stay_behind` and `kill`. All numbers FP.

## 1. Shared values (world encounters in `sunk_ward/encounters.json`, CAST1-A6)

`arena` CLOSED; `total_blocks` 6; `ap_regen` 4; `unit_ap` 10 (CANON 2.1); `rng_seed` 1337; `pace` auto; `deployment`
standard; `morale` false unless listed; `start_status` []; `guests`, `protect`, `key_loot` [] unless listed;
`timeout_outcome` "fled" unless listed; `rows` as in section 4. `party` lists the expected founders (previews and
sims); the campaign fights with the active party. `location` ids come from the M0 skeleton (the map sketch's ids:
`alleyway_junction_7`, `ash_crawl_crossroads`, `fourth_kiln`, `tallow_vats`, `crucible_watchpost`,
`sluice_undercroft`; prologue places `alleyway_of_cinders`, `meat_market_rooftops`, `canal_bridge`; the job's
`third_ward_gate`); "" where the skeleton has none yet. Zones are today's code zones as placeholders (new Sunk Ward
zones: G22).

## 2. The slice (Q-84 a: 0a-0c, the hearth, M3, one dilemma, M5)

| Key | `enc_pro_0a` | `enc_pro_0b` | `enc_pro_0c` |
| --- | --- | --- | --- |
| `name` | The Alley | The Rooftop | The Bridge |
| `zone`, `location` | `ruined_courtyard`, `alleyway_of_cinders` | `ruined_courtyard`, `meat_market_rooftops` | `river_crossing`, `canal_bridge` |
| `party` | `kaelen` | `kaelen`, `vesper` | `kaelen`, `vesper`, `maren`, `hollis` |
| `enemies` | `footman_patrol` x2 | `mortuary_flagellant` x3 | `footman_patrol` x3, `watch_crossbowman` |
| `objective` | rout | rout | rout |
| `objective_text` | "Strike while they wind up: a quick Pommel Strike breaks a slow blow." | "Follow the smoke: hide, pin them, slip away." | "Free the cart's captives and hold the bridge." |
| `exits` | [[11, 3], [11, 4]] | [[11, 4]] | [[11, 4]] |
| `max_rounds` | 8 | 8 | 8 |
| `phases` | `tremor` round 2: line "The alley shudders. Kaelen's forearm burns under the linen." | - | - |
| `purse` | {} | {"gold_min": 10, "gold_max": 10, "once": true} | {"gold_min": 15, "gold_max": 15, "once": true} |
| `objective_reward` | {} | {} | {"loot": "loot_poultices", "once": true} |
| Teaches | windup, guard, the Pommel Strike interrupt (T10) | stealth (Ash-Cloud Fade), root (Intercepting Hook), disengage (Hamstring Sever) | ranged pins (Tripwire Snare), field suture (Cauterize) |

- 0a: Captain Vane gives "Order 44" in the pre-fight dialogue and withdraws (he is not on the board). The prologue
  Footmen are `footman_patrol` (hp -30%, power -2) with the tutorial poise 24.
- 0c: Maren and Hollis are freed in the pre-fight dialogue at the cart and fight as members (a guest is AI-driven, so
  the player could not practise their skills); their join flags are the dialogue's (RECRUIT1).

| Key | `enc_job_corvo_gate` (the dilemma's fight, CQ9) | `enc_evt_03` (M3) | `enc_evt_06` (M5) |
| --- | --- | --- | --- |
| `name` | Corvo's Gate | The Bone-Mill Raid | The Crucible |
| `zone`, `location` | `ruined_courtyard`, `third_ward_gate` | `lava_cavern`, `fourth_kiln` | `desert_outpost`, `crucible_watchpost` |
| `party` | the four prologue founders | the four prologue founders | the four deployed (from Kaelen, Vesper, Maren, Hollis, Dunstan) |
| `enemies` | `castellan_footman` x2, `watch_crossbowman`, `bailiff_enforcer` | `overseer_grendel`, `tallow_dredger` x2, `bailiff_enforcer` | `inquisitor_prelate_nemeth`, `inquisitorial_purifier` x2, `castellan_footman` x2, `watch_crossbowman` x2 |
| `guests` | - | `dunstan` | - |
| `objective` | rout | rout | escape, `objective_count` 3 |
| `objective_text` | "Force the Third Ward gate." | "Wreck the mill and free the chain gang." | "Open the Slag Gate: one holds the winch, three get out." |
| `exits` | [[11, 3], [11, 4], [11, 5]] | [[11, 4]] | [[11, 3], [11, 4], [11, 5]] |
| `max_rounds` | 8 | 10 | 8 |
| `phases` | - | `pulverizer`, `vents` (CAST1_BOSSES 3.3) | `gate_reinforcements`, `infusion`, `gate_closing` (3.6) |
| `unkillable`, `timeout_outcome` | -, fled | -, fled | [`inquisitor_prelate_nemeth`], lost |
| `stay_behind` | {} | {} | see below |
| `purse` | {"gold_min": 20, "gold_max": 20, "once": true} | {"gold_min": 50, "gold_max": 50, "once": true} | {} |
| `key_loot` | - | [{"loot": "loot_slag_hammer", "kill": "overseer_grendel"}] | - |
| `objective_reward` | {"loot": "loot_poultices", "once": true} | {"loot": "loot_slag_plating", "once": true} | {} |
| `tags` | ["loud"] | ["main", "loud"] | ["main", "checkpoint"] |

- Corvo's gate (CQ9 a): the board job offers a bribe (Scrap, quiet, a dialogue choice) or this fight (loud: an
  Inquisition Alert input through the meter spec, and Corvo turns cold: flag `sunk_ward.corvo_forced` set by the
  job's dialogue). Corvo is not on the board. Tags `loud` and `quiet` are read by the Alert meter spec (Q-15).
- M3: Dunstan is a guest (AI-driven) so the player sees his kit; he joins after the win (bunk 5, flag
  `sunk_ward.dunstan_joined` by the after-fight dialogue).
- M5 `stay_behind` (ENCOUNTER2 3.1; Q-75a a): `{"mode": "hold", "tiles": [[10, 1]], "tiles_label": "the winch",
  "fallback": "any", "pick_default": "highest_hp", "flag": "sunk_ward.crucible_stayed", "then": "leave_party",
  "result_line": "{name} held the winch."}`; when ROSTER1-L lands, `then` becomes `capture` (ROSTER1's lifecycle) and
  the row adds `party_min` 4 (ROSTER1 BK6; CANON 3.2: the Crucible needs four deployed, G23). Nemeth is `unkillable`,
  so "no active foe" cannot win first.
- The hearth scene and the slice end card are vignettes (not encounters).

## 3. The rest of Act 1 (M2b, Q-86)

| Key | `enc_evt_01` (M1) | `enc_evt_02` (M2) | `enc_evt_04` (M4) | `enc_evt_05` (interlude) |
| --- | --- | --- | --- | --- |
| `name` | The Sweep | The Crossroads Cages | The Tallow Vats | The Awakening Fissure |
| `zone`, `location` | `ruined_courtyard`, `alleyway_junction_7` | `ruined_courtyard`, `ash_crawl_crossroads` | `lava_cavern`, `tallow_vats` | `swamp_bog`, `sluice_undercroft` |
| `enemies` | `captain_vane`, `castellan_footman` x2, `watch_crossbowman` | `proctor_quell`, `inquisitorial_purifier`, `mortuary_flagellant` x2 | `guild_master_orlov`, `render_beast` x2, `tallow_dredger` x2 | `calcified_amalgam`, `ash_crawler` x3, `calcified_husk` x2 |
| `guests`, `protect` | - | `tithe_captive_a`, `tithe_captive_b`; none protected | `elyse`; none | - |
| `objective` | rout | rout | rout | rout |
| `objective_text` | "Break Vane's sweep before the grates are sealed." | "Free the cages before the flagellants cull them." | "Cut through the vats; find the chemist." | "Drive the thing back into the fissure." |
| `exits` | [[11, 4]] | [[11, 4]] | [[11, 4]] | [[0, 4]] |
| `max_rounds` | 8 | 8 | 10 | 10 |
| `morale` | true; `morale_leader` `captain_vane`; `morale_break` 50 | false | false | false |
| `phases` | `execution_wall`, `vane_withdraws` (CQ5) | `cull_1`, `cull_2`, `marrow_valve` | `cataclysm` | `bells`, `bloom` |
| `purse` | 30 | 25 | 100 | 0 |
| `key_loot` | [{"loot": "loot_gate_pass", "kill": "castellan_footman"}] | [{"loot": "loot_marrow_scalpel", "kill": "proctor_quell"}, {"loot": "loot_marrow_valve", "kill": "proctor_quell"}] | [{"loot": "loot_flensing_talon", "kill": "guild_master_orlov"}, {"loot": "loot_orlovs_retribution", "kill": "guild_master_orlov"}] | [{"loot": "loot_marrow_core", "kill": "calcified_amalgam"}] |
| `objective_reward` | {"loot": "loot_gilded_pavise", "once": true} | {"loot": "loot_slab_retorts", "once": true} | {} | {"loot": "loot_dreamers_vertebra", "once": true} |
| `tags` | ["main", "loud"] | ["main", "loud"] | ["main", "loud"] | ["main"] |

Purse values are `{"gold_min": n, "gold_max": n, "once": true}` (Scrap is the purse's currency name, M0 kit).

- M2 captives (CQ6 a): world characters `tithe_captive_a` "Caged Unsealed" and `tithe_captive_b` "Caged Unsealed"
  (role npc) on a catalog unit `tithe_captive` (side party, class rogue, level 1, tier minion, gear all `none`,
  `flee_ratio` 1.0, `deck` {}); adults 18-24 by the tone rule. Captives alive at the win add to the saved counter
  (Q-29: ROSTER1 SY1's int flag `youths_saved`, raised by the after-fight dialogue's `add_flag` by the number alive).
- M4: Elyse is a guest; she joins after the win (bunk 6). Orlov's vendetta: key loot (CQ10 a). The lore's +5 Warded
  Tallow is a purse resource reward no encounter field pays (G25). The grease (`toxic_bog`) tiles of the arena stand
  in for Orlov's boiling-fat aura.
- Interlude: Aline's marrow dose (Q-66 b, a one-fight start buff for the founder the player picks) needs a per-member
  start buff and a "member deployed" dialogue condition (G20). Until then the pre-fight dialogue records the pick in
  `sunk_ward.aline_dose` (the member id; Act 2 reads it for Calcification I) and the fight has no dose buff. The dose's
  first-pass effect when G20 lands: `empower` magnitude 25 for 2 rounds (the lore's "+100% damage that ignores armour"
  needs Act 2 pierce; the balance lane sets it).
- Vane's bounty (a later Vane fight, only if he withdrew) is not Act 1 data.

## 4. Terrain rows (`rows`, ENCOUNTER2 R0)

Glyphs (`grid.rs`): `.` floor, `o` stone, `r` rubble, `#` wall, `=` bridge, `W` deep water, `L` lava, `f` burning
ground, `p` toxic bog. Each block lists y0-y2, y3-y5, y6-y8 side by side; every row is 12 glyphs; exits and the hold tile are walkable.

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
enc_evt_03     y0 ############  y3 #......f...#  y6 #..LL..f...#
               y1 #..LL....r.#  y4 #.r.........  y7 #..LL......#
               y2 #..LL..f...#  y5 #......f..r#  y8 ############
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
walls are the iron mortuary cart; M2's centre walls are the cage wagon and `f` the burning lanterns; M3's lava is the
slag pools and `f` the vent tiles (the scheduled bursts are G24); M4's lava is the boiling vats, `=` the catwalks and
`p` Orlov's grease; the interlude's walls are bone pillars and `p` the violet mist; the Crucible's `f` are braziers,
the wall column at x 6 the inner gate, [10, 1] the winch.

## 5. Board checks for the asset lane

| Check | Expectation |
| --- | --- |
| Rows | 9 strings of 12 known glyphs (ENCOUNTER2 R0 check) |
| Deployment | the party's and the enemies' deployment tiles are walkable: if `content check` or the Arena reports a blocked start tile, move one rubble or wall glyph and note it in the task report |
| Exits | every listed exit is walkable; M5's hold tile [10, 1] is walkable, not an exit |
| Unit count | enemy side plus phase waves at most 12 (M1: 4 + 2; M5: 7 + 2) |
| Phases | at most 4 boss phases plus one arrival (M2: 3; M5: 2 + arrival) |
