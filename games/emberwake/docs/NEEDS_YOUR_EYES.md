# Needs your eyes

What only you can judge, with what to do. The planner keeps this current: new items go in when they land, and an item leaves when you've answered. Say "open the build" and the planner launches the latest one.

## For the next session (Design keeps this list; updated 2026-11-19 15:35)

Parked for your approval (golden re-records need your word):

- [ ] Auto-plan fixes that move the timeline and sim goldens (`PR7-41` item 6, `PT-22`, patch
  `/mnt/project-files/pr7-41/pt22_regrant_retarget.patch`): (a) auto-fill stops re-granting a multi-round self status
  the unit already holds (ends Vesper's Fade loop; 6 of 18 timeline fights move); (b) a hero whose target is down or
  out of reach retargets the nearest reachable foe instead of resting (8 fights move, sim golden run 21, one repro
  re-recorded). Recommendation: approve both for heroes; enemies keep resting (TG4 unchanged).

Mocks waiting for you, then provisional calls Design made while you were away, to confirm or change:

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
- [ ] Status shown where it acts (`docs/specs/STATUSVIS1.md`, your note 2026-11-19 18:05): Design mapped each
  status kind to its place (locks on the timeline, ticks on the HP bar, attack and armour changes in the numbers, move
  changes on the grid) and the plate order (statuses with no other home first). Mock ready: battle take 25
  (`art-ui/battle_ux14/battle_take25_status_where_2x.png`), one frame per kind. Recommendation: approve.
- [x] Boss UI mock (`docs/specs/BOSSUI1.md`): signed off through takes 26-30 (poise A; hero plus "Others", every member on hover);
  built as `PR7-54` (take 31 shape). Once it lands, play a boss fight and check the feel of the boss intent rows.
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
