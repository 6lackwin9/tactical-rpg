# Balance report (written by the balance agent; read by the brainstorm agent)

## Archive Index

| Archive | Scope | Summary |
| --- | --- | --- |
| [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md) | `PR5-27` .. `R1-BAL`, `INPUT1-MB`, `RULESET1`, `REWARD1-B`, `BAL4/5` (`2026-10-18 .. 2026-11-10`) | All pre-`U3_GEAR_REWORK` Channel Duel, `R4-CLOSE` (`61.0%` feel / `92%` party), `R1..R4`, and classic Ashlands balance runs |
| [BALANCE_REPORT_2026-10.md](../../../docs/archive/BALANCE_REPORT_2026-10.md) | `2026-10-17` initial runs | Earliest October 2026 baseline snapshot |

## 2026-11-20 PR7-60 ESCAPE-PLAN1 the Crucible replayed by hand after the planner fixes (`FB-405`)

Report only. The level-7 saves of `ENGINE-30` on the `PR7-60` build: a hand move may end on an open exit, a pick never
drops another hero's, hand moves pass through friends, the hold pick suggests the nearest hero (Vesper here) and can
change until round 1 is confirmed, and a hero's Withdraw spends no AP it lacks. Three tries of each fight, every
escape picked by hand onto an exit tile except in the Auto tries. Logs and shots: project files `tools-lane/pr760/`.

| Try | Fight | Plan | Result | Downs (round, cause) | Out |
| --- | --- | --- | --- | --- | --- |
| base 1 | `enc_evt_06` | `ENGINE-30`'s winning plan; Maren stops beside the exit in round 3 (AP 5 of 6 for the exit) | **won, round 6** | none (Vesper ends 25/222) | Maren r4, Hollis r4, Kaelen r6 |
| base 2 | `enc_evt_06` | Kaelen holds and rushes the top lane through Vesper's tile | lost | Hollis r3 (Decapitation, 305); a Purifier walls Kaelen off the winch | none by then |
| base 3 | `enc_evt_06` | Auto every round, Vesper the suggested holder | fled, round 5 | Maren r2, Hollis r4 (Decapitation, 348), Vesper r5 | Kaelen r2 |
| Overlook 1 | `enc_evt_06_overlook` | `ENGINE-30`'s winning Overlook plan; Maren stops beside the exit in round 3 | lost | Kaelen r5 (Purifier 2 blocks the lower exits, then Nemeth 174 crit) | Maren r4, Hollis r4 |
| Overlook 2 | `enc_evt_06_overlook` | Maren rests in rounds 1-2 to keep AP and leaves in round 3; Kaelen breaks through the gap | fled, round 6 | Vesper r6, Kaelen r6 (walking the top lane) | Maren r3, Hollis r4 |
| Overlook 3 | `enc_evt_06_overlook` | Auto every round, Vesper the suggested holder | fled, round 4 | Maren r2, Vesper r4 | Kaelen r2, Hollis r4 |

What changed against `ENGINE-30`:

1. Hand exit picks work: every hand-planned escape left through the exit tile picked (seven escapes), with no Auto or
   preset, and the refused picks named their reason (`AP 5 < 6`, `Claimed by Vesper's move`, `No path` onto a
   friend's start tile). No pick dropped another hero's.
2. Withdraw within AP costs a round. `ENGINE-30`'s round-3 auto Withdraw spent Maren to -1 AP to leave; now she stops
   one tile short and leaves in round 4. In the base fight the plan still wins (round 6). In the Overlook the extra
   round lets Purifier 2 follow her to the lower exits and wall Kaelen off; `ENGINE-30`'s Overlook win does not repeat.
3. Auto still loses both fights: Kaelen, now free of the formation, walks out in round 2 and leaves the others to fall.
4. The suggested holder (Vesper) is the one the wins used; the Change chip let the pick move from Kaelen to Vesper.

Answer: **a careful player still wins the base fight at level 7 with no margin; the Overlook is lost on this build in
three tries**, chiefly because a hero's Withdraw can no longer run AP into debt. `objective_count` 2 would most likely
have turned Overlook 1 into a win (two out in round 4, Vesper still holding when Kaelen fell in round 5); Overlook 2 and
3 lost the holder in the round the last hero fell, so the rule's verdict there is not certain; the base losses had at
most one out. A lever for the Overlook alone is AP: Maren with 6 AP in round 3 leaves on time (Overlook 2 did, then
lost Kaelen on the long walk).

## 2026-11-20 ENGINE-31 SIM-JITTER1 start jitter on and off (`FB-401`)

Report only: no code, data or golden change. The feel sim already jitters start tiles by default (`jitter=on`: each
unit up to one tile inside its half, and the party order, seeded per fight), and the `ENGINE-26` what-ifs ran with it:
its authored row is reproduced exactly here. So this run compares the published numbers (jitter on) with fixed starts
(`jitter=off`): all 24 Sunk Ward fights, 32 seeds, `aegis sim feel world=sunk_ward pace=on`, the `ENGINE-26` what-if
keys. "Moves" means won % by 10 points or more, or the median rounds won by 1 or more.

| Run | Fights that move (of 24) | Pace on (won %, median rounds by tier) | Pace off |
| --- | --- | --- | --- |
| As authored | 10 | mob 95.8/4, elite 96.9/4, boss 39.7/9 | mob 88.5/3, elite 100/3.5, boss 31.7/10 |
| Foe HP +25 % | 10 | mob 63.5/3, elite 90.6/4, boss 25.9/9 | mob 69.8/4, elite 98.4/5, boss 19.6/10 |
| Foe HP +50 % | 11 | mob 60.4/4, elite 76.6/5, boss 12.9/10 | mob 66.7/4, elite 88.3/6, boss 15.2/10 |
| Foe HP +100 % | 8 | mob 43.8/3, elite 51.6/7, boss 6.3/11 | mob 41.7/3, elite 28.9/7, boss 5.8/9 |
| One more foe | 16 | random 99.4/2, elite 61.7/5, boss 23.2/9.5 | random 100/1, elite 56.3/6, boss 7.6/8 |
| One more foe, HP +25 % | 11 | random 100/2, elite 37.5/6, boss 13.4/10 | random 100/2, elite 46.1/6, boss 10.7/9 |
| One more foe, HP +50 % | 7 | random 100/2, elite 25.8/7, boss 8.5/10 | random 100/2, elite 26.6/6, boss 2.2/10 |
| One more foe, HP +100 % | 12 | random 98.4/2, elite 2.3/8, boss 3.1/12 | random 100/3, elite 9.4/7.5, boss 1.3/11 |
| Two more foes | 11 | random 98.1/3, elite 18/7, boss 15.2/9 | random 100/3, elite 16.4/8, boss 10.3/9 |
| Every foe +1 level | 9 | mob 75/3, elite 90.6/4, boss 34.8/8.5 | mob 75/3, elite 100/4, boss 33/11 |

As authored, the fights that move (on to off): the prologue's first fight 87.5 to 65.6 %; the Sweep 53.1 to 18.8; the
Crossroads Cages 84.4 to 31.3 (median 7 to 8.5); the Bone-Mill Raid 28.1 to 12.5; the Tallow Vats 56.3 to 71.9; the
Awakening Fissure 56.3 to 87.5; and four fights by one round only (the prologue's second fight, the Proctor Wardens,
the Kiln Haulers, the Guild Bruisers). Under "one more foe" every random fight's median drops from 2 to 1 with fixed
starts, so `ENGINE-26`'s finding (one more foe reaches the 2-round target) holds only with jitter.

Findings:

1. Fixed starts move a third to two thirds of the fights, bosses most (up to 53 points), because 32 seeds of one start
   are close to one fight (`ENGINE-29`). The jittered numbers are the ones to trust.
2. **Jitter is already the default** of every sim mode, the what-ifs and the campaign run included (`SimArgs` `jitter: true`), so
   nothing needs to change and no golden is touched. The goldens and contract tests that pin `jitter=off` do it on
   purpose, for byte-stable output (`emberwake_sim` `sim_golden`, `timeline_golden`, the host's `auto_driver` test,
   `rpgkit_sim` `feel` tests). The fixed starts in `FB-389` came from a hand-built test session, not from the sim.

## 2026-11-20 ENGINE-30 AGENT-CRUCIBLE1 the Crucible played by hand at level 7 (`FB-401`)

Report only: no code, data or golden change. Played through the real game with `tools/fc.sh playtest`, each round
planned by hand with the battle's own picks (round intent picks, Auto-fill off, Auto pressed only where noted). The
saves were built from the Act 1 story test's state after the Interlude: M5 taken, the party standing at the Crucible
Watchpost, the four founders at level 7 (the level `ENGINE-28` finds at M5), full HP, no Warded Tallow (the fight opens
at AP 8/10). The fight is deterministic for the same picks, so each try is a different plan, and later tries replay the
rounds that worked. Run logs and the shots of each deciding round are in project files `tools-lane/engine30/<try>/`.

| Try | Fight | Result | Downs (round, cause) | Escape |
| --- | --- | --- | --- | --- |
| base 1 | `enc_evt_06` | lost (wipe, round 6) | Kaelen r3 in the gap (round 2 took him 326 to 48, mostly Decapitation); then all | gate open r1 (Vesper); none out |
| base 2 | `enc_evt_06` | lost (wipe, round 6) | Vesper r3 at the winch (Heavy Thrust and two crossbows) | gate open r1; none out |
| base 3 | `enc_evt_06` | fled (2 of 3 out) | Kaelen r3 (Decapitation, 232) | Maren r3, Hollis after (Auto) |
| base 4 | `enc_evt_06` | fled (2 of 3 out) | Kaelen r3 (his hit to break it was dodged) | Maren r3, Hollis after (Auto) |
| base 5 | `enc_evt_06` | **won, round 6 block 1** | none (Vesper ends 33/222) | Maren r3, Hollis r4, Kaelen r6 |
| Overlook 1 | `enc_evt_06_overlook` | fled (2 of 3 out) | Vesper r5 at the winch (five Heretic's Brands, a crossbow) | Maren r3, Hollis r4; Kaelen walled in r5 |
| Overlook 2 | `enc_evt_06_overlook` | **won, round 5 block 1** | none (Vesper ends 108/222) | Maren r3, Hollis r4, Kaelen r5 |
| Overlook 3 | `enc_evt_06_overlook` | fled (1 of 3 out) | Vesper r5, Maren r6, Kaelen r7 | Auto every round (Vesper keeper): Hollis r3 |

The winning plan (base 5; Overlook 2 differs only in round 3's tile):

1. Round 1, hold back: Vesper picked to hold the winch; Kaelen Hold Line in front, Vesper and Maren guard, Hollis
   Heavy Winch Sunder kills Crossbowman 2 (142 on 136). The foes walk west and leave both outer lanes.
2. Round 2: Vesper up the top lane to (7,1), Maren down the bottom lane to (7,7), Kaelen one Pommel Strike then Rest
   (AP for later), Hollis shoots. Nemeth's Decapitation on Maren whiffs because she walked off its tile.
3. Round 3: Vesper onto the winch (gate open in block 1); Maren withdraws (Auto, block 2); Hollis to (9,2); Kaelen
   makes a two-block move, (2,5) in base 5 and (4,7) in Overlook 2, so Decapitation lands on an empty tile.
4. Rounds 4-6: Auto for the escapers (only the auto-plan can step onto an exit); Kaelen walks the bottom lane.

Answer: **a careful player can win both fights at level 7, with no margin.** Every win needs all four founders
standing (one holds, three must leave), Vesper survives about three rounds on the winch, and Nemeth's Decapitation
(232-278 in one hit) must be dodged by moving off its tile; a hit timed to break it missed because Nemeth stepped first.
The base fight's round-3 wave lands on (7,3) and (7,4), right outside the middle gap. Pressing Auto every round
loses (Overlook 3), as in the `ENGINE-29` sims.

Which `FB-399` (b) change turns a loss around: **`objective_count` 2.** Three of the six losses ended "fled" with two
founders out and would have been wins (base 3 and 4, Overlook 1; Overlook 3 had one out). Exits nearer the party would shorten
Kaelen's walk (Overlook 1 was lost with Kaelen walled in on the bottom lane, seven tiles from an exit) but not save a founder who falls in
round 3 (base 1 to 4). Moving the base wave to round 4, as in the Overlook, removes the wall outside the gap but the
Overlook itself still lost two of three.

Found while playing (for Design; not fixed here):

1. A hand pick cannot step onto an open exit ("That tile can't be moved to"): `move_route` and `apply_goto` refuse
   exit tiles, so only the auto-plan's escape row (Withdraw) leaves. A player has to know to press Auto or a preset
   for each escaper.
2. A later hero's pick silently drops an earlier hero's picks when the routes cross ("rebuild ... every later member";
   the dropped member's route goes invalid, "Too far"), and a tile an ally starts or ends a block on blocks routes in
   that block. In round 1 Kaelen is boxed in by the formation and cannot plan any route.
3. The keeper pick is final and its default (`highest_hp`) is Kaelen, the one founder who cannot reach the winch in
   round 1.
4. The Withdraw draft may spend AP below zero (-1 to -3); hand moves cannot.
5. Results: the benched Dunstan gets +10 XP and the captured keeper none; the pinned foe card covers the winch corner.

## 2026-11-20 ENGINE-29 ESCAPE-PLAY1 the auto driver's Crucible play by level (`FB-399`)

Report only: no code, data or golden change (two trial fixes were measured and reverted). Founders raised to each level
by `WorldRoster::ally_levels`, the world's timeline ruleset, `auto_draft` balanced, 12 rounds.

| Fight | Start tiles | L3 | L4 | L5 | L6 | L7 | L8 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `enc_evt_06_overlook` | jittered (the sims' default), 64 seeds | 0 | 0 | 0 | 1 | 3 | 4 |
| `enc_evt_06` | jittered, 64 seeds | 0 | 0 | 3 | 3 | 3 | 2 |
| `enc_evt_06_overlook` | authored (`jitter=off`), 64 seeds | 0 | 41 | 54 | 0 | 0 | 0 |
| `enc_evt_06` | authored, 64 seeds | 0 | 0 | 0 | 0 | 0 | 0 |

Findings:

1. **The level effect is an artifact of the fixed start.** With the authored tiles the seeds barely change the fight,
   so 64 seeds are close to one fight: at levels 4 and 5 that fight happens to win, from level 6 it does not (Hollis,
   a little tougher, gets out a round sooner, so the foes turn on Maren, the slowest escaper, alone at the lane's end).
   With start tiles jittered by one tile the Overlook is 0-6 % at every level from 3 to 8 and the base 0-5 %: the
   Crucible is a near loss for the auto driver at any level, and the `FB-389` figures at levels 4-5 (75-84 %) were the
   same fixed-start luck.
2. **No escape bug in the driver.** Every non-keeper plans Withdraw from the round the keeper can reach the winch
   (round 2); the routes run around the enemy line to the column-11 exits; Maren's route takes her south first, so she
   spends rounds 2-4 inside the foes' reach. Round 1 is the tactic's normal draft (the escapers engage the nearest
   foes) because the gate is still shut.
3. **Two trial fixes made it worse** (shared auto-plan, test-only switch, reverted): escapers Withdraw from round 1
   (0/32 at every level, authored start) and escapers Guard in place until the keeper is in reach (0/32 everywhere).
   The round-1 engage is what wins the lucky starts.

So the Crucible's difficulty is in the fight (seven foes and the round-3 wave between the party and the exits, all
three non-holders needed out), not in the driver. A sim cannot rate fix (b) of `FB-389` (wave on round 4) above 0-6 %
for the auto driver; it needs a human playtest or a data change of the escape itself (e.g. `objective_count` 2, or
exits nearer the party's side).

## 2026-11-20 ENGINE-28 CAMPAIGN-SIM1 Sunk Ward Act 1 at the party's real levels (`FB-395`)

Report only: no data, rule or golden change. New kit sim run `tools/fc.sh sim campaign world=sunk_ward seeds=32`
(`rpgkit_sim::world_campaign`): the story fights in story order (the `new_game` sequence, then the main quests, the
interlude before M5 because it sets the flag M5 needs), each with the party it fields, XP paid by the kit's kill XP at
each member's own level and levels raised by the class's default growth. Every fight counts as won for the XP (a lost
story fight is retried in play) and starts at full HP. `side=2` (the default, the patrol `visit_cap`) adds two roamer or
scavenger fights before each quest fight, round robin over the paced bindings, repeats at `repeat_xp_pct`. Each fight is
also fought at the authored level 1 (the feel sim's view). Auto driver, the world's timeline ruleset, jitter on.

| Fight | Levels on entry, side 2 | Won % (level 1) | Median rounds | Minutes | HP left | Levels on entry, side 0 | Won %, side 0 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `enc_pro_0a` | Kaelen 1 | 87.5 (87.5) | 6 | 1.8 | 82 % | 1 | 87.5 |
| `enc_pro_0b` | 2, Vesper 1 | 100 (100) | 3 | 1.4 | 90 % | 2, 1 | 100 |
| `enc_pro_0c` | 2, 1, 1, 1 | 100 (100) | 4 | 3.2 | 95 % | 2, 1, 1, 1 | 100 |
| `enc_evt_01` M1 | 4, 3, 3, 3 | 81.3 (53.1) | 8 | 6.5 | 37 % | 3, 3, 2, 2 | 59.4 |
| `enc_evt_02` M2 | 5 | 96.9 (84.4) | 5 | 4.0 | 74 % | 4 | 93.8 |
| `enc_evt_03` M3 | 6 | 65.6 (28.1) | 6 | 4.8 | 26 % | 5 | 53.1 |
| `enc_evt_04` M4 | 6 | 100 (56.3) | 6 | 4.8 | 73 % | 6 | 100 |
| `enc_evt_05` interlude | 7 | 100 (56.3) | 6 | 4.8 | 86 % | 6 | 100 |
| `enc_evt_06` M5 Crucible | 7 | 3.1 (0) | 4 | 3.2 | 8 % | 7 | 3.1 |
| `enc_evt_06_overlook` | 7 | 9.4 (0) | 6 | 4.8 | 12 % | 7 | not run |

Founders end Act 1 at level 8 with side fights, 7 without. Every side fight is won 100 % (1-3 rounds); story XP is most
of the gain (M5 alone pays 240 a founder), so `side=` moves a founder about one level by M3 and none by M5.

Findings:

1. **The founders reach M5 at level 7**, not the level 1 the feel sim uses. M1 to M4 and the interlude are fair at
   those levels (M3 is the hard one, 66 %; M1 81 %).
2. **The Crucible stays unwinnable at the real level: 3 % (base) and 9 % (Overlook) with jitter, 0 % both without.**
3. **The auto driver's Crucible result is not monotonic in level.** Without jitter the Overlook wins 24/32 at level 4,
   27/32 at level 5 (the ENGINE-27 numbers) and 0/32 at levels 6 and 7: from level 6 Maren falls in round 4 and Kaelen
   in round 7 in every seed (holder Kaelen throughout; no skill unlocks at 6). So the auto driver's escape play, not
   the fight's numbers, decides the Crucible above level 5; fix (b) of `FB-389` cannot be judged by the sim alone.

## 2026-11-20 ENGINE-27 CRUCIBLE-CHECK1 why both Crucible fights are won 0 % (`FB-389`)

Report only: no data, rule or golden change. Replayed `enc_evt_06` and `enc_evt_06_overlook` round by round (the feel
sim's loop: `auto_draft` balanced, the world's timeline ruleset, 32 seeds each, a scratch test not kept) with the gate,
the holder, every unit's HP and tile, and the outcome printed each round.

**No rule bug.** Every part of the fight works as specified: the holder is picked by `highest_hp` (Kaelen) and walks
to the winch (10, 1) by round 2; the gate opens on round 2; the `arrival` wave lands on round 3 (round 4 in the
Overlook variant) at (6, 4) and (7, 4); `infusion` fires on round 4; Nemeth is never finished; heroes withdraw through
the open gate. The objective is `escape` with `objective_count` 3 and four founders (`party_min` 4), so the three who
are not holding must all get out: **any founder down makes the win impossible** (the fight then ends `fled` or `lost`).

| Party level (authored 1) | Base: won | Base: round-1 founder deaths | Overlook: won | With `objective_count` 2: base / Overlook |
| --- | --- | --- | --- | --- |
| +0 (as the feel sim runs it) | 0/32 | 24 | 0/32 | 0 / 0 |
| +1 | 0/32 | 19 | 0/32 | 0 / 0 |
| +2 | 0/32 | 19 | 0/32 | 0 / 0 |
| +3 | 0/32 (32 fled) | 4 | 24/32 | 30 / 28 |
| +4 | 0/32 (32 fled) | 2 | 27/32 | 32 / 31 |

Two causes, in order:

1. **Party level.** The feel sim fights every world encounter with the founders at their authored level 1; the
   Crucible's foes are level 2-6 (Nemeth 6). Up to level 3 a founder falls in round 1 in most seeds (the line starts
   two tiles from seven foes), which ends the escape before it starts. The other Act 1 bosses are winnable at level 1;
   the Crucible is the one fight tuned for a levelled party (`FB-253`: tuned for human play).
2. **The round-3 wave in the escape lane.** At level 4 few founders fall in round 1, but the base fight still wins
   0/32: the escapers cross to the exits at column 11 through the enemy line, the two wave footmen arrive on round 3
   in that lane, and one escaper falls on the way in every seed. The Overlook variant (wave on round 4) wins 24/32 at
   the same level.

Smallest fixes (data; they wait for the user with the other boss calls): (a) state the party level the Crucible is
tuned for and let the feel sim fight it there (at level 4 the Overlook is 75 %, the base 0 %); (b) the base wave on
round 4 as in the Overlook (the base then plays like the Overlook); (c) `objective_count` 2 (94 % at level 4, but
CANON's "one holds, three get out" changes). Open question: no sim models the founders' real level at M5 (there is
no Act 1 XP campaign); below level 4 the fight stays unwinnable for the auto driver whatever the wave does.

## 2026-11-19 ENGINE-26 BALANCE-WHATIF1 what-if runs (`FB-381`)

Report only: no data or golden change; the user picks the data change. Sunk Ward, timeline rules with the ENRAGE2
enrage, the authored parties, 32 seeds per fight, `aegis sim feel pace=on` with the new what-if keys `foe_hp=`,
`foes=` and `foe_level=` (the foes as changed for the run only). Minutes are the EN7b estimate (10 s planning per hero
per round plus 5 s, 0.42 s playback per block, 0.9 s between rounds).

**1. Random fights** (the 10 escape fights together, 320 fights; target median 2-3 rounds with won % at least 85):

| Change | Won % | Escaped % | Won median rounds | Minutes |
| --- | --- | --- | --- | --- |
| As authored | 100 | 0 | 1 | 0.8 |
| Foe HP +25 % | 100 | 0 | 1 | 0.8 |
| Foe HP +50 % | 100 | 0 | 1 | 0.8 |
| Foe HP +100 % | 100 | 0 | 1 | 0.8 |
| One more foe | 99.4 | 0.6 | 2 | 1.6 |
| One more foe, HP +25 % | 100 | 0 | 2 | 1.6 |
| One more foe, HP +50 % | 100 | 0 | 2 | 1.6 |
| One more foe, HP +100 % | 98.4 | 1.6 | 2 | 1.6 |
| Two more foes | 98.1 | 1.9 | 3 | 2.4 |

HP alone does not move the median (foe HP x4 first reaches 2 rounds, x10 reaches 4 with 22 % escaping). The smallest
change that reaches the target is **one more foe** per random fight (median 2); two more reach median 3.

**2. Elite minutes** (four heroes and 6 blocks in every elite fight):

| Fight | Won % | Won median rounds | Minutes now |
| --- | --- | --- | --- |
| Corvo's Gate | 93.8 | 5 | 4.0 |
| Proctor Wardens | 100 | 3 | 2.4 |
| Kiln Haulers | 96.9 | 3 | 2.4 |
| Guild Bruisers | 96.9 | 4 | 3.2 |

At the elite round target of 5, today's estimate reads 4.0 minutes. Five rounds read 7 minutes at **18.9 s** of planning
per hero per round and 8 minutes at **21.9 s** (today's estimate: 10 s). Whether either fits real play is for the
user's playtest.

**3. Bosses** at the authored level and with every foe one level up (`foe_level=1`), beside the approved boss band of
20-40 % won:

| Fight | Won % (authored) | Median rounds won | Won % (+1 level) | Median rounds won |
| --- | --- | --- | --- | --- |
| The Sweep | 53.1 | 10 | 50.0 | 10 |
| The Crossroads Cages | 84.4 | 7 | 81.3 | 8 |
| The Bone-Mill Raid | 28.1 | 7 | 25.0 | 6.5 |
| The Tallow Vats | 56.3 | 9 | 40.6 | 8 |
| The Awakening Fissure | 56.3 | 10 | 46.9 | 10 |
| The Crucible | 0 | - | 0 | - |
| The Crucible (Overlook) | 0 | - | 0 | - |

Only the Bone-Mill Raid sits in the band as authored; +1 level brings the Tallow Vats to its top edge (40.6 %). The
Sweep, the Cages and the Fissure stay above it at +1 level, and both Crucible fights are never won at either level
(lost 28 of 32 as authored).

## 2026-11-19 PR7-55 ENRAGE2 + PR7-56 PHASEBREAK1 boss run (`FB-377`)

- Run: `aegis sim feel enc=<all 24 Sunk Ward encounters> seeds=32 pace=on`, rule off (`tune=enrage_from_round:0`) and
  on (the shipped timeline numbers: from round 6, minion and standard 10 %, elite 15 %, boss 20 %, 4 steps), both with
  the phase break and the new caps (mob 6, elite 8, boss 12, random Sunk Ward fights 5 with the foes escaping).
- Enrage, off to on (won %): The Sweep 56 to 53, The Cages 84 to 84, Bone-Mill 28 to 28, Vats 62 to 56, The Awakening
  Fissure 66 to 56 (the largest drop, inside the 10-point band), Crucible 0 to 0 (tuned for human play,
  `FB-253`); every fight won in 5 rounds or fewer is unchanged; Vats and the Fissure median 0.5 rounds shorter. The
  steps stay as specified (no tuning needed inside the band).
- Pace by tier, rule on (EN6, EN7, EN7b; minutes = won median rounds x (10 s per hero + 5 s + 6 blocks x 0.42 s +
  0.9 s)): mob (prologue 3) won 95.8 %, capped 4.2 %, median 4, 1.8 min; random (10) won 100 %, capped 0 %, median
  1, 0.8 min; elite (corvo gate and 3 elite roamers) won 96.9 %, capped 2.3 %, median 4, 3.2 min; boss (7) won
  39.7 %, capped 5.4 %, median 9, 7.3 min. Caps: mob and elite under the 5 % target; boss median inside 8-11.
- Misses, not tuned (EN8 raises lethality, which would make them shorter still; listed for Design): random fights
  end in 1 round (PACE1 mob band 2-4); elites take about 3 minutes against the 7-8 target (at 5 rounds the EN7b
  estimate is about 4 minutes, so the elite minutes target and the round target disagree); boss win rate is low at the
  authored levels (Bone-Mill 28 %).
- Stun and stagger (blocks lost per fight; SV5): mob 0 and 1.7, random 0 and 0, elite 0.8 and 0.8, boss 0.2 and 1.5.

## 2026-11-19 PR7-42 pacing run (`FB-348`)

- Feel (10 seeds, authored party), unchanged by PR7-42: Rooftop 100%, Corvo gate 90%, The Sweep 30%, The Cages 80%,
  Bone-Mill 20%, Vats 50%, The Crucible 0%.
- `sim bands` already out on develop: duel ranger 63, priest 32, brawler 64; party line clear 75 (band 80-95).
  Tracked for `PR7-20` (BALANCE-TUNE1).

## Current Baselines & Open Issues (`U3_GEAR_REWORK`, `PR5-51b` .. `PR5-57`)

- **`PR5-51b` (`30415b91`, `FB-268`) `sim party` / `sim feel` sweep**: `sim party` boss clear jumped to **`80%`** (target band **`30-50%`**), driven primarily by `mage` `orb` `power_stat_override` (`INT` substitution lifting boss damage `214 -> 396`), while `warrior` sustained tank pressure and output dip; `sim feel` `Crucible` (`world:sunk_ward/enc_evt_06`) remains **`0/10`** auto-sim wins and stays tuned for human play per `FB-253`.
- **Action queued in `PR5-57` (`GEAR-REWORK1-FOLLOWUP`)**: add `counts=on` `RULE_COUNTERS` (`ammo_empower`, `reload`, `knife_throw`, `orb_override`), audit `orb` scaling, and retune `sim party` boss clear back into **`30-50%`** + check `warrior` tank/power balance without weakening human `Crucible` (`FB-253`).
- **Pre-rework `PK1` & `R4-CLOSE` baselines (reference in [BALANCE_REPORT_2026-11A.md](../../../docs/archive/BALANCE_REPORT_2026-11A.md))**: `PR5-27` (`2026-11-10`) Channel Duel `37.5%` won with `Vent -50 Strain / 10% HP` (`channel_power = 1.25`, `channel_ai_edge = 1.2`); `R4-CLOSE` (`2026-11-08`) `sim feel` `61.0%` overall, `sim party` `100/92/100/92/92%` (post-boss HP `0.31`), campaign `arms` `3/6`, `gate` `5/6`, `goblin_skirmish` `17.5%` (20/40 capped, held in `POST-R4-CLEANUP`).
