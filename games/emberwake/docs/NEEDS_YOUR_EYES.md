# Needs your eyes

What only you can judge, with what to do. The planner keeps this current: new items go in when they land, and an item leaves when you've answered. Say "open the build" and the planner launches the latest one.

## For the next session (Design keeps this list; updated 2026-11-20 04:25)

Parked for your approval (golden re-records need your word):

- [ ] Auto-plan fixes that move the timeline and sim goldens (`PR7-41` item 6, `PT-22`, patch
  `/mnt/project-files/pr7-41/pt22_regrant_retarget.patch`): (a) auto-fill stops re-granting a multi-round self status
  the unit already holds (ends Vesper's Fade loop; 6 of 18 timeline fights move); (b) a hero whose target is down or
  out of reach retargets the nearest reachable foe instead of resting (8 fights move, sim golden run 21, one repro
  re-recorded). Recommendation: approve both for heroes; enemies keep resting (TG4 unchanged).
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
  to the founder farthest from the winch), now queued (`PR7-60`). Recommendation: keep "one holds, three get out";
  fix the planner, rerun the hand play, and only then weigh a data change; then the other bosses after you play one.
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

## Windows and menus

- [ ] Settings > Display: Borderless and Fullscreen toggles. F9 cycles 720p, 900p, 1080p, now real pixels. Anything odd?
- [ ] Main menu: now on the new grid, and meant to look exactly as before. Spot anything?
- [ ] Theme colours and lighter dim text: anything hard to read?
- [ ] Game icon: at 16 px it's cropped to the eye. Fine in the title bar and taskbar?
