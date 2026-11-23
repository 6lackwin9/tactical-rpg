# Needs your eyes

What only you can judge, with what to do. The planner keeps this current: new items go in when they land, and an item leaves when you've answered. Say "open the build" and the planner launches the latest one.

## For the next session (Design keeps this list; updated 2026-11-21 22:50)

Parked for your approval (golden re-records need your word):

- [ ] Auto-plan fixes that move the timeline and sim goldens (`PR7-41` item 6, `PT-22`, patch
  `/mnt/project-files/pr7-41/pt22_regrant_retarget.patch`): (a) auto-fill stops re-granting a multi-round self status
  the unit already holds (ends Vesper's Fade loop; 6 of 18 timeline fights move); (b) a hero whose target is down or
  out of reach retargets the nearest reachable foe instead of resting (8 fights move, sim golden run 21, one repro
  re-recorded). Recommendation: approve both for heroes; enemies keep resting (TG4 unchanged).
- [ ] Planner fixes from the Crucible hand play (`PR7-60`, `FB-403`), behind a test-only switch until you approve: a
  pick that clashes with an earlier hero's route is refused instead of dropping it, and Withdraw no longer spends AP
  below zero. They move `sim_golden`, `phasefold` and `timeline_golden`. Recommendation: approve (both were bugs a player
  could hit); take it together with the auto-plan fixes above.
- [ ] Fight length after the enrage and caps landed (`PR7-55`, `FB-377`; what-if sims `FB-381`, `BALANCE_REPORT.md`
  "2026-11-19 ENGINE-26"): (a) random fights end in 1 round against your 3; more foe HP does not help (still 1 round
  at +100 %); one more foe gives 2 rounds (99 % won), two more give 3 (98 % won). Recommendation: two more foes per
  random encounter, for your 3 rounds (re-records the sim golden). (b) Elites take 3-5 rounds and about 2.5-4 minutes by
  the estimate; 7-8 minutes at 5 rounds would mean about 19-22 s of planning per hero per round (the estimate uses
  10 s). Recommendation: keep the targets and measure your planning time at the next PC playtest. (c) Bosses (band
  20-40 % won): the Bone-Mill Raid 28 % is in band; the Sweep 53, the Tallow Vats 56, the Fissure 56 and the Cages 84
  are above; both Crucible fights are never won by the sim. An agent played it by hand at level 7 (`FB-401`): base fight lost 2, fled 2,
  won 1 (round 6, no downs); the Overlook fled 1, won 1; Auto alone loses. Winnable, with no margin: one founder down
  ends the win. Two escapers instead of three (changes canon) would have turned 3 of the 6 losses into wins. The play
  also found planner bugs (a hand move can't step onto an exit, a later pick drops an earlier one, the keeper defaults
  to the founder farthest from the winch), now fixed (`PR7-60`). Rerun by hand after the fixes (`FB-405`): the base fight
  is still won (round 6); the Overlook is lost 3 of 3, because Withdraw can no longer run into AP debt. Recommendation:
  keep "one holds, three get out" and that AP rule, and ease the Overlook itself (exits nearer, or one more founder AP
  there); then the other bosses after you play one.
- [ ] Act 1 pacing, from an agent's play of Act 1 (`FB-441`, 2026-11-22; run log and shots in project files
  `build-tools/act1_agent_run/`): the party reached level 7 after the first four quests (rated 4-5) and won every fight
  up to the Bone-Mill boss, which then beat it three times on the auto plan at level 7-8. Tithing fell from 42 to 17
  days by the Vats (a day per road and per rest, about 4 days per lost boss trip), and nothing says what happens at 0;
  Tallow fell from 10 to 1 in the first week. Queued without asking: bug fixes, retries that reroll, and an agent
  trying the boss by hand (`ENGINE-34`). Recommendation: decide the pacing after your own playtest; meanwhile the
  Tithing HUD says what happens at 0. The Night of the Red Pen resolving in one log line is on the story list for after
  your story playtest.
  Update 2026-11-22 10:30 (`FB-392`): played by hand, the Bone-Mill boss was won first try at level 7 (the auto plan
  still loses it); the Crucible was lost 5 of 5 at level 8 (the suggested winch holder draws both Purifiers and a
  Footman, and the gate shuts when the holder falls). Bugs found there are queued; balance waits for you.
  Update 2026-11-22 18:30 (`FB-406`): after the fixes, an agent won the Crucible by hand on its third try at level 8
  and reached the end of Act 1 with 19 Tithing days left (Kaelen held the winch and was captured).
- [ ] Battle rule fixes that change the sim golden (`FB-459`, tools lane, all tried and reverted until you say):
  (a) reinforcements stop landing on tiles your planned moves claim (PT-71); (b) a move cut short never ends a block
  farther from its goal (PT-70); (c) path claims: today a planned move claims every tile it walks for the whole round,
  so a 1-tile gap passes one hero a round; recommended: a tile is claimed only in the block it is walked, so a friend
  can follow a block later (PT-72); the same rule would let a route pass a foe's start tile once that foe has
  left (PT-87, `FB-479`); (d) the auto plan learns escape objectives (keeps the holder on the winch, sends
  escapers to exits), which changes the sim's planner too (PT-74). Recommendation: approve all four as one intended
  re-record of `sim_golden`; they also bear on the Crucible losses.
- [ ] Diorama battle stage in the game (your 21:46 go-ahead): the first slice landed (`PR7-64`); Fable's in-game review
  (https://claude.ai/artifact/GfoCKjXQnVeTbFBBshuRe2) found the UI chrome on contract and the stage about half way to
  the mock. Queued as provisional, to confirm: amber light pools under the lanterns with a soft vignette; cover props
  (crates, pillars, lamps) drawn on blocked cells inside the arena, display only; a faint grid at rest (Settings can turn
  it off). Update 2026-11-22 01:00: all of it has landed (`PR7-67`, `PR7-68`, darker floor); the Act 1 story staging is
  in (`PR7-66`); every Act 1 foe and boss now has a painted sheet (`PR7-65`, landed). Fable's second pass (same report,
  new section at the top) says the open alley now matches the mock closely. Queued as provisional in `PR7-74`, to
  confirm: each zone (foundry, watchpost, undercroft, closed alleys) gets its own floor and wall or a tint; the tallow
  vats stop using the lava art; painted units keep their size (about 1.15 cells) and a unit standing in front of the
  selected one or its target turns see-through, instead of shrinking everyone to 1 cell; the boss-fight foe card docks
  away from the party.
- [ ] Style 2 look for the exploration maps: every Sunk Ward map, the hub included, now uses the Style 2 grounds
  (before and after in project files `art-ui/explore_style2/grounds_before_after.png`). Art/UI is holding Style 2 walls
  and the Grate's props until you say whether the grounds look right, so one call covers every map. Recommendation:
  keep them.
- [ ] Later acts, mocked as you allowed at 21:47 (placeholders, no data, nothing locked): Act 2's first region (the Watch
  Ramparts, High Bastion) has a region-map reference and a battle stage mock, in a colder grey-violet palette with cyan
  lamps and warm light only at the breach (project files `art-ui/act2_mock/`), plus a stage set cut from it. Look when
  convenient; nothing waits on it.
- [ ] Enemy short names for the foe plates (`PR7-40`): Brawler, Berserker, Footman, Enforcer, Husk, Archer,
  Crossbowman, Purifier, Shaman, Skirmisher (also the Tallow Dredger), Scout, Flagellant, Render-Beast, Ash-Crawler;
  bosses by name.
- [ ] Pacing (`PR7-42`): patrols every 40-60 steps, at most 2 per visit, no respawn until the next day; Cages offered
  before The Sweep; Warded Tallow at the Candle-Maker for 5 Scrap; Make camp needs Tallow.
- [x] Bunks (`PR7-43`): the hall's plank floor under the bunks with low timber screens and a step; the class blanket Superseded: you signed off take 8 (2026-11-19), built as `PR7-53`.
  colours.
- [ ] Mocks to sign off (images in the project files `art-ui/`; scripts on PR #67):
  battle take 22, the boss fight (`battle_ux14/`, contract section 9); battle take 23, the full timeline view (T)
  and playback controls; the screens for the victory report, story, character, shop, notice board and saves
  (`screens_ai21/`). Build rows follow sign-off.
- [ ] Party token size on maps (`FB-354`, ART-19): tokens are about 40 px beside 80-90 px props. Options:
  (A) scale tokens to 32 px (blurs pixels), (B) shrink tall props, (C) leave, (D) redraw tokens at 32 px. Mock ready in
  `art-ui/token_scale/` (`hub_sheet.png`, `region_sheet.png`, 2x zooms beside them). Recommendation: D (crisp pixels,
  party at the bunks' scale; A looks uneven at 2x, B makes hub beds look toy-like).
- [x] Status shown where it acts (`docs/specs/STATUSVIS1.md`): you approved take 33 with the take 34 changes
  (19:39); built as `PR7-57`. A later kit row redraws the legacy status icons (`AI-24`).
- [x] Boss UI mock (`docs/specs/BOSSUI1.md`): signed off through takes 26-32 (poise A; hero plus "Others", every member on hover);
  built as `PR7-54` (take 31 shape). Once it lands, play a boss fight and check the feel of the boss intent rows. Your 19:15 note:
  if the numbers feel too many in play, the fallback is to drop the per-block cells and keep only the totals (selected
  hero, Others, party breakdown), with the skill strip above giving the timing.
- [ ] Boss poise golden call (`PR7-54` data step, BU9): boss poise tuned so a same-level party breaks it first in
  round 2-3, then about every 3 rounds; it moves the boss fights in the timeline and sim goldens. Recommendation:
  approve once the sim numbers are in.
- [ ] Image queue ready: say "process image queue" in Gemini (bunk props, class glyphs, bed keepsakes, battle
  strips, Junction 7 houses; refs in `image_pipeline/refs/active`).
- [ ] Gamepad on the PC (`FB-327`): repeat timing, dead zone, which button is Back (south Enter, east Back, west
  Space, north Tab, bumpers page, Start Esc).

## Battle view: closes M1

Scene Runner > Arena > `tl_flank`, with **Ruleset set to Timeline** on the Arena card (otherwise it may run on Classic, with no capsules or coloured strip). Plan a round, then confirm it.

- [ ] Plan strip: are the block colours, centred icons and names readable? Icons are generic for now, as `tl_` skills have no art of their own.
- [ ] No first-move chips on enemies, only the intent marker. Is that enough?
- [ ] Hold Alt: bars, arrows and hazard marks show. Is the calm view without Alt right?
- [ ] AP gauge beside the strip: do the capsules read?
- [ ] Dash: use Flank Cut or Hamstring Cut. Is any animation weak, in any direction?
- [ ] Round flow: after a round plays, the next starts by itself with a Round N ribbon. Settings > Gameplay has "Auto-confirm next round". Does the pace feel right?
- [ ] Party cards: AP pips drain as moves play and refill at round start; numbers beside the bars; 5 poise pips. Readable?
- [ ] Rings: the intent ring replaces the side ring; the letter shows on Alt. In a test fight both sides share sprites, so judge it with real factions in mind.
- [ ] Skill menu: click a unit or tile. It opens beside it, icons only, with rows the target does not allow greyed. Does it read? It can cover a nearby unit.
- [ ] Move in the menu, after a skill: click a tile. The plate names the blocks it takes back, only Rest follows, up to 3 move blocks. Is the reach, up to 12 tiles for fast units, what you want?
- [ ] Skill menu from a strip block (`PR7-40` part 3): a pick always lands after your last pick, and the title says which block ("Block 4 vs Brute"), even when you clicked a later empty block. Is that right, or should it land at the clicked block?
- [ ] Anything else looks wrong? If nothing does, the clarity pass closes.

## Walking and turns

Scene Runner > Exploration, on Warden Keep.

- [ ] Walk speed: is 200 ms right? F11 cycles 90, 110, 130, 160, 200.
- [ ] Run: hold Shift, 130 ms. Right speed? Any hitch when pressing or releasing it?
- [ ] Turns: F5 cycles the corner: cut (today), finish the leg, quick finish. Turn from up or down into left or right. Which feels right? The planner's pick to try first is quick finish.
- [ ] F10 cycles the turn beat and equal speed. Does either help?
- [ ] F7 flips Tilted and Square. Which view do you want? Decides EXPLOREVIEW1-R.
- [ ] Zoom, F6: does the whole scene hold up at 1.5x and 2x?
- [ ] NPCs face down, and turn to you while talking. Look right?
- [ ] Sound feel (`FB-504`, 2026-11-23; menu sounds, map footsteps, a 0.4 s battle music fade-in and warmed stings arrive with `ENGINE-56`, all provisional): do menus feel answered, are the footsteps too much, does a fight start punchy enough? Run with `feel=1` once: the overlay shows your PC's real audio buffer (Windows may refuse our 512 frames).
- [ ] Walk feel, measured 2026-11-23 (`FB-493`; every input moves on the next frame, no step after release, even 200 ms beat): (a) full speed from the first step and a dead stop, no ease in or out; keep? (b) a turn pressed mid-walk steps at once (snappy, can make a short step) instead of waiting for the beat; keep? (c) the pad has no run button (Shift only); give it a shoulder button? (d) four directions only, no diagonals; keep? (e) the camera is locked on the party, no soft follow; keep? My recommendation: keep all five as they are and judge them in your hands-on playtest; (c) is the one I'd add (a shoulder for run), if you agree.

## Windows and menus

- [ ] Settings > Display: Borderless and Fullscreen toggles. F9 cycles 720p, 900p, 1080p, now real pixels. Anything odd?
- [ ] Main menu: now on the new grid, and meant to look exactly as before. Spot anything?
- [ ] Theme colours and lighter dim text: anything hard to read?
- [ ] Game icon: at 16 px it's cropped to the eye. Fine in the title bar and taskbar?
