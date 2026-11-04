# INTENT2 behaviour matrix

Section 2 of [INTENT2.md](INTENT2.md), which holds the rules (IR ids). `guard` is shown as Stand.

## 2. Behaviour matrix

Table A: target and timing.

| Intent | Valid round target | Shape anchored on | Moves before, after | Chase | Commits | The other side avoids it by | Target gone or moved |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Assault | an enemy | its tile at impact | approach, kite (AS1-3), after-move | 1 tile when out of range (MR16); items none | melee: its tile at wind-up start, after the chase; ranged and thrown: in band at impact, shape read then | leaving the locked tile or the band, hiding, breaking the wind-up | down first: Rest (TG4); left: miss, AP and stack spent |
| Field | a source tile in the arena | it, fixed at planning; a direction shape expands away from the caster (IR7) | one regular move first (it changes a direction), or Step here after | none | planning: source, direction and impact tiles | walking out of the marks; shoving the caster of a drawn shape | never fails; empty tiles `[Area Miss]`; hazards skip unwalkable tiles, traps occupied ones (GH2) |
| Stand | you | your tile at impact, no direction | Move here first, or Step here after | none | impact | leaving your neighbours before impact | cannot fail |
| Support | another friend | its tile at impact | approach into reach (SP1) | 1 tile (MR16); items none | in reach at impact | killing or pulling the friend away | down first: Rest; out of reach at impact: not landed, AP spent, item kept |
| Maneuver | a position by an enemy, or away from it, or a tile (a jump) | none | the dash or jump rides in the skill; regular move only beyond range (MR7) | inside the dash (MR8) | destination at the wind-up tick; then the melee lock | filling the position tiles; leaving after the dash | `[Flank Protected]`, `[Dash Blocked]` as MR8; down first: Rest |

Table B: interplay and display.

| Intent | Friendly fire | Riposte | Stealth | Flank | Follow-up fill | Items | Player sees in planning | AI picks it when |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Assault | none (IR9) | melee at distance 1 answered | hidden target: the whole command misses | hits flank (MV3); engaged, so it can be flanked | FU2 on the unit | a bomb on a foe | intent ring, target arrow, locked tile (ATTACKLOCK1) | row 5, as today |
| Field | `affects` | a melee area hit is answered as today (`command.rs:494`) | hits hidden units | not engaged, cannot be flanked; its hits flank as today | same source, each family once, then Rest | a bomb on a tile | source and impact tiles from the first pick; enemy Field rounds marked too | new row Z |
| Stand | none (IR9) | your own Riposte holds beside a ring or heal (IN6) | a ring does not end it | not engaged, so never flanked and Riposte always answers | Stand order; a guard only while a Defend runs | elixirs; a potion on yourself | ring preview on neighbours while hovered | rows 3 and brace; row 2 on itself; new ring row |
| Support | friends only | none | a hidden friend can still be helped | none | same friend (SP2) | a potion on an ally | arrow to the friend | row 2 on another |
| Maneuver | none | a flank hit gets none (MV3) | no dash at a hidden target (GD5) | the point of it | MR6, decision "Auto-fill keeps a flank" | a mobility item (planned) | position plates (MR13) | row 4 |

AI rows. Row Z, before row 5: the unit has a Field skill; its best source (extends `flush_pick`) covers at least `area_threshold` foes, for an elite or boss at least the first ranked one, passes `friendly_fire_ok`, and the unit did not use Field last round (so it alternates; AI3's order lists stay). Enemies plan first, so a Move and the impact tiles it gives are visible while the player plans. Ring row, before row 4: two or more foes adjacent and a ring usable. Reckless drops Defend skills, not the Stand intent, or a reckless boss never flails. A taunted unit picks only a source whose impact tiles cover the taunter. Enemies pay no AP: recovery stops repetition, as a recovery past the round is refused. Enemies use no items (`pouch` ["none"], `use_items` off, sim only).

Tips and forecasts. A dual tip names both rows: "On an ally: Support. On yourself: Stand, keeps Riposte." IF3's "loses Riposte" reads the row. Forecasts read the planned command; the strip colours by the round's intent. Skill text that opens "Support, Standard:" drops the intent word on dual rows.
