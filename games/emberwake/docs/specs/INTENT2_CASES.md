# INTENT2 cases: archetypes, worked rounds, winners and losers, item values, feel measures

Sections 3-5 and the item values of [INTENT2.md](../../../../docs/specs/INTENT2.md), which holds the rules (IR ids). Per-skill listing: [INTENT2_SKILLS.md](INTENT2_SKILLS.md). Status: revised to the user's answers of 2026-11-05. `guard` is shown as Stand.

## 3. Archetypes

Verdict: C fits cleanly, R fits with a rule, X counterintuitive. "Test" is a builder's test; "plausible" rows have no catalog skill yet.

| # | Archetype (example) | Intent | In a round | Test | V |
| --- | --- | --- | --- | --- | --- |
| 1 | Melee single (Strike) | Assault | locks the tile at wind-up | target steps off before impact: miss, "Dodged", AP spent | C |
| 2 | Ranged single (Aimed Shot) | Assault | no tile lock | target stays in band on a new tile: hit; leaves band: miss | C |
| 3 | Reach 1-2 (Hook Thrust) | Assault | melee lock, band 2 | target steps inside the band but off the tile: miss | R |
| 4 | Gap-close (Lunge, Pounce, Charge In) | Maneuver | dash in skill (`IR21` step 5) | dash to target, then melee lock; plain Move dodges after the dash | C |
| 5 | Position dash (Flank Cut) | Maneuver | MR1-MR8 | positions full at start: `[Flank Protected]`, AP spent | C |
| 6 | Hit then retreat (Orlov's Retribution) | Maneuver | hits, then a dash away | ends farther from the target than it began | R |
| 7 | Jump (Backflip) | Maneuver | aims at a foe, lands farthest | rooted: refused; follow-ups none, Rest | C |
| 8 | Teleport or mobility item to a tile (plausible) | Maneuver (IR3 a) | tile target, no follow-up can act | Field skill refused after it; 5 Rest blocks | X |
| 9 | Burst, Assault row (Firebomb) | Assault | ranged rule; authored BURST around the target at impact, splash x 0.5, foes only | begins after the target moved out by 1: chases, hits; ally beside it unhurt; target leaves band at impact: miss | R |
| 10 | Burst, Field row (Firebomb) | Field | source fixed, reach + 1 | target walks out: hits nobody, AP spent, marks unchanged; friend in the marks hit | R |
| 11 | Thrown enemy burst (Lard Spray) | Field, Assault | dual by `IR21`: Field row fixes tile source + `reach` bonus; Assault row tracks target (+1 block, half splash) | Field row: target is a tile; Assault row: tracks foe | R |
| 12 | Burst at reach 0 (Bone Bloom) | Field | may centre on the caster, stays if it is pushed | push caster 1: tiles unchanged, caster unhurt | X |
| 13 | Reach 1 burst with pull (Cataclysm) | Field | fixed tiles (`reach_max == 1`), pulls those hit | each unit hit pulled 1 toward the caster | R |
| 14 | Melee line (Execution Wall) | Field | from the source along the direction; `area` makes it 4 | on (6, 4), source (5, 4): (5-2, 4); a foe beside the caster is safe; pushed before impact: `[Area Miss]` | R |
| 15 | Melee aimed swing (Gilded Sunder) | Field | ARC away from the caster, `area` makes it 5, 3 blocks | on (6, 4), source (5, 4): (5, 2-6); at 3 wide, on (5, 4), source (6, 5): the L (6, 4), (6, 5), (5, 5) | R |
| 16 | Direction, straight and snapped (ARC, LINE now; CONE plausible) | Field | away from the caster through the source | caster (5, 4). Source (8, 4), right: LINE (8-10, 4), ARC (8, 3-5), CONE (8, 4) + (9, 3-5). (8, 5), larger axis, right: CONE (8, 5) + (9, 4-6). (6, 7), down: CONE (6, 7) + (5-7, 8). Exact diagonal (7, 6): LINE (7, 6), (8, 7), (9, 8); ARC (7, 5), (7, 6), (6, 6); CONE (7, 6), (8, 6), (7, 7), (8, 7) | R |
| 17 | Planned Move changes the marks (CONE plausible) | Field | direction read from the planned end tile | on (5, 4), source (7, 4): right, (7, 4) + (8, 3-5). Pick Move here to (7, 2) first: down, (7, 4) + (6-8, 5). Undo the Move: right again | R |
| 18 | Direction shape on your own tile; adjacent source | Field | refused; adjacent is the ordinary case | source = caster: grey, with the reason; a reach 0 burst there is fine. ARC, source (6, 4) from (5, 4): (6, 3-5) | R |
| 19 | Thrown direction shape, shoved caster (plausible) | Field | reach above 1 lands; reach 1 misses | CONE reach 3 shoved after planning: tiles unchanged, lands; melee ARC shoved: `[Area Miss]`. Assault row: re-read at impact from the target's tile, foes only | R |
| 20 | Light ring (Sweep Kick) | Stand | RING read at impact, foes only | foe and ally adjacent: foe hit, ally not; caster pushed: ring moves | R |
| 21 | Ring beside Riposte | Stand | both run; Guard skill holds from wind-up start through Block 6 (`S1.1` Option 1C) | round 3: ring + riposte counters while guarded through Block 6 | R |
| 22 | Defend, Riposte, Stealth (Active Guard, Brace Counter, Fade) | Stand | hold, counter, hide | side melee hit answered; moved and stealthed: single-target and thrown miss, Field hits | C |
| 23 | Hazard under you (Ash-Cloud Fade) | Stand | placed at start, stays when you walk | walk out: hazard stays, stealth stays with you | R |
| 24 | Taunt (Taunt Shout) | Stand (was Support) | self-only | taunted enemy's source must cover the taunter | R |
| 25 | Focus | Stand (was Support) | boost this round, attack next | no attack in the same round; boost on in next round's first block | X |
| 26 | Guard on an ally (out of scope for `IR21`; T4 Bodyguard deferred) | Stand, Support | the row picks the anchor | ally steps 1: chases; on yourself: no chase | R |
| 27 | Intercept (Intercepting Hook) | Stand | self target, redirects a hit within 1 | no friend near: plain guard | R |
| 28 | Heal one (Mend) | Support, Stand | ally row or self row | self row keeps Riposte; ally row warns "loses Riposte" | R |
| 29 | Heal beside Riposte (Brace + Mend, learned) | Stand | keeps both; Guard skill holds through Block 6 (`S1.1` Option 1C) | fight at cap with both units in Stand: counted by the feel run | X |
| 30 | Heal around you or an ally (out of scope for `IR21`) | Stand, Support | RING anchored by the row | self row: ring moves with you; ally row: follows the ally | R |
| 31 | Heal on a tile (out of scope for `IR21`) | Field, `affects` friends | ground helps friends | foe on a tile unhealed; needs Area on non-attacks (M) | R |
| 32 | Trap (Tripwire Snare) | Field | one tile, unoccupied, `lingering` | aim at an occupied tile: refused at the pick; then Rest | X |
| 33 | Hazard burst (Phosphorus Pot) | Field | hit, then ground, + 1 round | burn ground stays after the caster leaves | C |
| 34 | Forced move (Slag Slam) | Assault | push leaves the band | next follow-up chases 1 | C |
| 35 | Leech, HP cost, rider, debuff (Martyr's Rush, Brand, Intimidate) | as the aim | effects never change intent | HP cost refused at low HP; a no-damage hit still misses if the target left | C |
| 36 | One-skill enemy (Husk); snap with no recovery (Brand) | Assault | Slam 2 + 1 twice; Brand 6 a round | fill: 2 Slams, no third; Brand: data adds recovery 1 | X |
| 37 | Potion, either row | Support, Stand | item opener, 0 AP | ally out of reach at impact: not used, stack kept; self row beside Active Guard | R |
| 38 | Bomb, either row | Assault, Field | as a thrown skill: authored shape, half splash under the ranged rule; or full on a source tile, reach + 1 | foe leaves reach: miss, spent; foe down first: Rest, kept; tile empty: spent; Kaelen's Field row holds only the bomb | R |
| 39 | Pouch as kit (a healer's bombs) | any | the pouch reaches intents the tree lacks | Maren with bombs opens Field; fill finds no skill: Rest | C |
| 40 | Burst round | as the opener | AP on skills, then items | 0 AP left: a bomb is still offered; no AP debt | C |

## 4. Worked rounds

Six blocks, AP 10, move 2 tiles a block, ATTACKLOCK1 on; item values are section 6's first pass.

1. **Mixed party.** Kaelen, Assault, Footman A: Pommel b1-b5, Rest b6. Maren, Support, Kaelen: Tonic b1-b2, Cauterize b3, Rest b4-b6. Vesper, Maneuver, Footman B: Sever (dash) b1, Sever b2-b5, Rest b6. Hollis, Field, tile (8, 3): Wire b1, Rest b2-b6. Four rings, nothing mixes inside a unit. Hollis is the cost: a wire and five Rest blocks, where he shoots too today. Round 5 is the fix, with no new skill.
2. **A boss's telegraphed Field round.** Orlov's source is (6, 4); Lard Spray (BURST, no direction, 2 blocks, 2 recovery, `reach`: 2-5) marks (5-7, 3-5) at planning. b1-b2 Spray lands b2; b3-b4 recovery; b5-b6 Rest (a second Spray's recovery would pass the round). Kaelen on (6, 4) walks to (6, 6) in b1 (2 AP): safe. Vesper on (7, 5) picks Sever: its dash takes her out free. Maren on (5, 4) keeps healing and takes 0.9 power and burn, her choice with the marks in view. A friend of Orlov's in the marks is hit too. Next round row Z does not repeat: Hook Drag on the first ranked.
   **The same with a swing.** Vane on (6, 4), Kaelen beside him on (6, 3), Hollis on (5, 4). Vane's source is (6, 3): up, Sunder (ARC, `area`: 5 across, 3 blocks, 2 recovery) marks (4-8, 3). A source at (5, 4) would mark (5, 2-6) instead and miss Kaelen. b1-b3 Sunder lands b3, b4-b5 recovery, b6 Rest. Kaelen steps out in b1; or Hollis shoves Vane in b2: off his planned tile, the swing misses.
3. **A Stand round with a ring (`S1.1` Option 1C).** Deck [Brace Counter, Sweep Kick], two footmen and an ally adjacent; a ring has no direction. b1-b2 Brace Counter (guard 0.3, Riposte on, `HeldGuard` active from b1 through b6); b3-b4 Sweep Kick lands b4 on both footmen, not the ally, while guard and Riposte stay active; b5-b6 Sweep Kick or Rest. Footman A's hit lands b2: guarded, countered 0.5. Footman B's lands b4: guarded (through b6 under Option 1C), countered 0.5. Guarded across all 6 blocks after b1 wind-up start and impossible to flank. Strong when surrounded, weak otherwise.
4. **Dodges.** Footman A starts Thrust (3 blocks) on Kaelen in b1: the tile locks at b1's start. Kaelen's Move in b1 lands at b1's end: b3 misses, "Dodged", and he attacks from b2 (a Move must come first, so this is his only time). Had the Thrust started in b2 the chase would re-lock and hit. Footman B starts Thrust on Vesper in b1; she picks Rest, Rest, Sever: her dash at b3's tick leaves the tile, her hit lands, B's misses. Same dodge, no Move and no AP beyond the skill.
5. **Items outside the skill tree.** Kaelen, Field, source (8, 4), no Field skill: Bomb b1, Bomb b2, Rest b3-b6; 0 AP. Vesper, Stand, self: Fade b1, Draught b2 (0 AP), Active Guard b3-b4, Rest b5-b6. Hollis, Support, Vesper: Move b1, Poultice b2-b3, Poultice b4-b5, Rest b6; 120 HP for 2 AP. Maren, hurt, Stand, self: Tonic on herself b1-b2, Active Guard b3-b4, Rest: the skill that was Support on Kaelen in round 1. Hollis's Wire then a Bomb in one Field round mends round 1.
6. **Burst round.** Hollis, AP 3, Assault Footman: Quarrel b1-b2 (2 AP), Strike b3-b4 (1 AP), Bomb b5, Bomb b6 (0 AP): 0.95 and 1.0 power plus 40 and 40 flat base damage (routed through `U1`/`A2` mitigation when item mitigation is on, `FB-21a`), and half on any foe in its PLUS. The Footman leaves reach in b6: the second bomb misses, spent, no refill this fight. That is the balance: the cap, not the cost.
7. **Mid-round failures.** Kaelen's target dies in b2: b3-b6 become Rest (TG4). Elyse's Field on the same tile keeps landing on empty tiles. A bomb at a foe downed before it starts: Rest, stack kept; downed after: spent. A stealthed foe that moved: Strike and bomb miss, Spray hits its tile.
8. **A boss across rounds.** The Amalgam (reckless, no AP). R1 Field Spine Volley (no direction, `reach`: 1-3, dual Field/Assault under `IR21`) on the party's centre: b1-b2, recovery b3-b4, Rest. R2 two adjacent: Stand, Flail (ring, no direction) b1-b2 + 1 recovery, Flail b4-b5 + 1: 0.9 x 2 foes twice. R3 Field. Row Z and the ring row make the pattern with no authored sequence (AI3).
9. **Why a mage picks each row.** Firebomb (0.6, `reach`: Assault 2-2, Field 2-3), mage at (7, 2). Cluster F1 (6, 4), F2 (7, 4), F3 (6, 5), F4 (7, 5), engaged with Kaelen on (5, 4). Field, source (7, 4): all four, Kaelen clear: 0.6 x 4 = 2.4 with status on each, then Glacial Burst on the same source (0.6 x 4): 4.8 and all rooted, if they stay. Assault, F2: 0.6 + 0.3 x 3 = 1.5, status on F2 only, Kaelen safe even if F2 stepped toward him; then Arcane Bolt on F2: 1.1. Total 2.6, of which 1.7 on F2. Two foes: Assault 0.9, Field 1.2. One foe alone: he steps off but stays in band: Field 0, Assault 0.6 + 1.1 = 1.7; he stays: Field 0.6 alone. So Field wins a cluster that holds still, and reach reaches F1 from a tile Assault cannot; Assault wins pursuit, focus and one or two foes. Both are worth a pick, but Assault is the default: a target approaching from far is out of Field's guess.

## 5. Winners and losers

Power in one six-block round from today's order lists, by script: AP 10 for founders, no AP and recovery for enemies, no reach or targets counted (Field and ring numbers multiply by those hit). "Today" drops SWEEP (it never fires).

| Unit | Today | Assault | Field | Stand ring | Judgment and fix |
| --- | --- | --- | --- | --- | --- |
| Vane | nothing: order holds only Sunder | none | Sunder 2.0 | - | wins; add `strike` to his Assault list |
| Bailiff Enforcer | Drag x2, 1.0 | Drag 1.0 | Judgment 2.2 | - | wins |
| Grendel | Jab x3, 1.8 | Jab 1.8 | Hammer 2.2 | - | wins |
| Orlov | Spray 0.9 | Drag x2, 1.8 (or Spray Assault row) | Spray 0.9 | - | alternates (`Spray` is dual Field/Assault under `IR21`, `FB-20`) |
| Amalgam | Volley 1.0 | Spine Volley (Assault row) | Volley 1.0 | Flail x2, 1.8 | wins (`Spine Volley` is dual Field/Assault under `IR21`, `FB-20`; add `strike`) |
| Quell, Nemeth | 1.4, 2.4 | same | dual throws | - | no change (`FB-17`) |
| Purifier | Nozzle + Brand x2, 1.7 | Brand x6, 3.0 | Nozzle 0.7 | - | loses its mix; Brand needs recovery 1 |
| Dredger | Grease x2, 0.8 | Strike x3, 3.0 | Grease 0.4 | - | its Strike out-hits its kit; keep `strike` out of its order and let the feel run decide |
| Hollis | Wire x3 (wasted on a unit target), Strike | Quarrel x3, 2.85 | Wire, then Rest | - | clearer, thin Field; pouch bombs fill |
| Elyse | Pot x3, 1.5 | Strike and Flash | Pot + Flash, 0.7 | - | Flash has both rows; hazards stay on the ground |
| Kaelen, Vesper, Dunstan, Maren | 3.0-3.6 | same | none (Shatter at level 2) | - | no change; Field through the pouch |

By class: the **mage** keeps burst then bolt in the Assault row at half splash, and gains a real Field round: a winner. The **brawler** wins most: rings give it a real Stand round. The **ranger** keeps its Volley-then-shots mix (`Volley` gains an Assault row under `IR21`, reach 2-3 in data, `FB-20`). **Priest** gains a self-heal Stand round; **rogue, warrior** barely move (Taunt and Focus to Stand). Every unit keeps Assault and Stand through basics (`strike`, `active_guard`; `flank_cut` for Maneuver), so no class has a dead intent it needs: Support and Field may be out of reach of a unit with an empty pouch, whose rows simply show no icon. Enemy fixes are data. **New skills needed before the slice: one multi-round guard skill (`brace`, `FB-18`).**

**Does the pouch answer thin rounds?** Access: yes. With a stack any unit can open an intent its tree lacks: no Field or Support basic is needed for the slice. Filling: no. Auto-fill spends no item, so after an item opener the ghost is the intent's skills or Rest; the player can pick the item again. Whole fight: no, and that is right: 3 slots of 2-3 uses carry a missing intent for 1-3 rounds of a fight capped at 8-10 (`max_rounds`), so a pouch is a spike, not an identity. Units that carry none lose nothing. Enemies are not helped. Cost: stash stock, 1 bomb and 2 poultices at the hearth (CAST1_GEAR 6): the balance lane raises it for the slice; the shop's `count` limits stock already.

## 6. Item values

**Verified.** 6 consumables. Each: `use_blocks` 1, `ITEM_AP` 1 (`command.rs:1451`, read in `plan.rs:723`, `round_intent_plan.rs:1674`, host `lib.rs:550`), `max_stack` 5 (no kit code reads it). `item_intent` reads damage-only as Maneuver, else Support; `item_opener` needs a unit, `pick_options` lists no item on a tile; `CmdKind::Item` holds a unit. `offered_in_battle` hides STAMINA_BREW and ANTIDOTE types (no rows). `aims_at_foes` makes a damage-less foe item a friend item: the authored list replaces it. No class, drill or `can_wear` gate: `pouch_put` and `UnitState` validation check only the CONSUMABLE type and "offered". Side bug: `buff_pct` always boosts attack, so Brimstone Draught ("+30 % CONS") gives +30 % attack (`command.rs:688-691`, fixed in `A1.2` by granting an `S1` status via `grant_now`).

First-pass values (the user tunes; `use_blocks` and `max_stack` per item through `aegis content`):

| Item | Effect | Shape | Allowed | Field bonus | `use_blocks` (today 1) | `max_stack` (today 5) | Why |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `con_heal_potion` | heal 90 | - | Support, Stand | - | 3 | 2 | the strongest; time is the price |
| `sulfur_poultice` | heal 60 | - | Support, Stand | - | 2 | 3 | at most 180 a round with nothing else |
| `con_focus_elixir` | attack +50 %, 2 rounds | - | Stand | - | 1 | 2 | must outlast its round |
| `brimstone_draught` | +30 %, 3 rounds | - | Stand | - | 1 | 2 | fix the stat bug first |
| `pitch_fire_bomb` | 40 flat | PLUS | Assault, Field | `reach` | 1 | 2 | burst: 80 in 2 blocks, then gone |
| `con_fire_bomb` | 45 flat | BURST | Assault, Field | `reach` | 2 | 2 | wider, so slower |

## 7. Feel run measures

Field impacts per fight and the share that hit nobody, by side; Assault-row and Field-row picks on dual skills, and foes hit by each; drawn swings that missed from a shove, and swings on nobody; dodges by Move and by Maneuver; boss damage per round against today; ring rounds and their output; capped fights (`tl_skirmish` 24-25 % today); self-heal rounds and rounds with Riposte and a self-heal together; share of rounds per intent; items used per fight by type, thrown misses, stacks empty by round; stash spent per fight; rounds where an intent had no active (OI1); Rest blocks per round, as a plain count; later blocks wasted when a target dies (TG4 stays until this shows a problem).
