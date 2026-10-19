# Design review 2 (2026-10-18)

For Tushar. This review covers `develop` a499b3f against DR1 (184f09c, `games/emberwake/docs/specs/DESIGN_REVIEW1.md`). It is due because RECRUIT1-L, INTENT1-L, MAPREAD1-L1, ITEMUSE1-L and MAPREAD1-L2 are now on `develop`. Sources: git log 184f09c..a499b3f (about 40 playable commits), `games/emberwake/docs/BALANCE_REPORT.md` (BAL4-ATTR v4, RECRUIT1-B, OBJ1-B, BOSS1-B), `games/emberwake/docs/ROADMAP.md`, `docs/TASKS.md`, the decisions files, `docs/HISTORY.md`, `games/emberwake/docs/specs/BAL5_LEVERS.md`, the "Candidates" list in `games/emberwake/docs/design/OPTION_BRANCHES.md`, and the frames under `/mnt/project-files/` (intent1-u, itemuse1-u, mapread1, combat-ui2-l3). I also read two data rows (consumables, Ashlands manifest). Nothing in the repo was changed.

Option builds are paused (user, 08:29). Items that are easier to judge by playing are marked **try later as an option build (paused)**. Everything ends in the approval list.

Already approved and queued, so not proposed again below: HEAL1 (HP carry-over, rest places, Make camp, defeat cost), BREPORT1 (battle report), BAL5 (trio re-measure plus AI and data levers on trigger), MORALE1, ASHPACE1, UIR-2..7, INTENT1-U / BOSS1-U / OBJ1-U, the dialogue layout B, the log ticker and quest tracker B picks, and the DR1 spec rows (KEEP1, DIFF1, DEEDS1, RESPEC1, OFFHAND1, PURSE1, DESERT1, COMBO-A2).

## 1. Summary

The DR1 batch landed fast and holds together. The game now has a solo-warrior opening, story recruits, enemy tells, battle items, objectives, a two-phase finale and readable maps. Three things work against the player's first hour:

1. **The first random fight is usually lost.** Bryn alone clears `ash_scouts` 20 times in 100 (target 90 or more). The scouts are 2 of the 5 weights in the Courtyard's step fights during the solo phase.
2. **Battle items are too weak to matter.** A Healing Draught heals 25 HP. Bryn has 334 HP at level 5 and an enemy strike does about 49. A Fire Bomb does 20 damage to goblins with 65-108 HP.
3. **The region maps are bigger than the screen.** Warden Keep is 48x30 tiles, and a 1000x660 view shows about 31x20 of them. The "Here" legend lists markers you cannot see, and nothing points to them.

The rest is either queued already (HEAL1 for fights without a threat, BAL5 for class gaps) or polish for the UI/UX queue.

## 2. Combat rules and AI

Working:

- INTENT1-L (a56dda7) and INTENT1-A (4205bc1). Enemies plan before the party and never re-plan, so a tell is always true. A move tell still names its target ("Move toward Ranger (B1)", `intent1-u/after_planning.png`), so a tell that is only for block 1 is still useful in round 1.
- ITEMUSE1-L (ad83b98, e5653ac). Items are a planner step with reach and undo, and the pouch is written back after the battle.
- OBJ1-L/A (3abdb63, bda1aa0) and BOSS1-L/A (5c85e7e, 1303031). A hold, an ambush with a guest, and a slay finale with phases mean fights are no longer all "kill everything".
- AUTO1-L (f09c933) with the tactic dropdowns (bb98f4f). Autoplay is now a real comfort feature.

Needs improvement:

- **Roles feel thin in play.** BAL5 found that the tank leaves allies at about 6% HP (band 40), the priest heals about 1% of party HP (band 20) and AoE swarm clears are 0-22 (band 70). The queued BAL5 AI and data levers cover the numbers. BAL5 sent four "feel" levers to this review because they are judged by playing:
  - warrior Bodyguard (T4)
  - priest Ward (S3)
  - mage self-centred burst (A4)
  - boss acts twice (B4)
- **A bodyguard or ward changes what a tell means.** For example, a hit aimed at the ally lands on Bryn. Specs must state that the tell still names the planned target.

Options (role feel; **try later as an option build (paused)**):

- A (recommended): try T4 Bodyguard, S3 Ward and A4 as one set: each class gets a move you would plan around. The self-centred burst reuses the existing BURST shape aimed at the caster's own tile, so there is no new area shape; the S12 rule "no new area shapes" still holds. Cost: about 3 small logic commits plus skill rows. The risk is BAL5 numbers moving while BAL5 runs.
- B: wait for BAL5-R1 and build only the lever whose gap is still open after BAL5-AI. Cheaper, but the class feel stays unchanged until then.
- C: skip. Roles stay number-tuned only.

## 3. Party and classes

Working:

- RECRUIT1-L/A (795aca1, 0d6a3ae, 40cd863): Bryn Hale starts alone, the Toller muster offers any remaining class, two recruits at most, and the heroes have names. `PartyBuilder.display_name` puts the names into combat and the menus.
- Recruits join at the higher of their base XP and the party's top XP, so a late pick is never behind.

Needs improvement:

- **The solo opening contradicts its own premise.** DR1 (4) says "he can solo low-level fights". RECRUIT1-B measured `ash_scouts` at 20% for the solo warrior (hp score 0.37), while `courtyard_skirmish` and `ash_straggler` both clear 100. The fight is weighted 2 of 5 in the solo-phase binding (RECRUIT1 3.4). Once HEAL1 lands, each loss also costs 3 supplies.
- **One recruit choice can stall the duo phase.** `pass_ambush` with warrior plus priest clears 30, while the other duos clear 70-100. Among trios, priest plus brawler clears the arms finale 0 of 10 with no heal. BAL5-AI (healer self-care S1) is the queued fix, so watch it rather than add a new item.

Options (`ash_scouts`):

- A (recommended): one scout fewer, the lever RECRUIT1 itself names. It is a one-row data change and the lightest tuning possible (DR1 (3)). Run it as the first change of ASHPACE1.
- B: keep the scouts but move them out of the solo-phase binding until the first recruit. No unit edits, but it needs a flag-gated binding; if bindings cannot gate on a flag, that is a logic change.
- C: leave it. The first random fight stays a likely loss, and HEAL1 makes the loss cost supplies.

## 4. Progression (levels, skills, gear)

No change since DR1. Flat XP, little loot and permanent spend are queued in ASHPACE1, DEEDS1, RESPEC1, OFFHAND1 and PURSE1. ITEMUSE1-U (069ac28) adds a pouch section to the Gear screen. Its plain list layout (`itemuse1-u/after_pouch.png`) moves into the Character popup with UIR-3, which is queued.

## 5. Items and economy

Working:

- ITEMUSE1-A (3048364) gives the three shop items a reach (potion 1, elixir 0 = self, bomb 3).
- "No free class potion" (decision world-campaign): only bought or found items go into a pouch, so items are a purchase decision.

Needs improvement:

- **The numbers predate level 5.** From `data/catalog/items.json`:
  - `con_heal_potion` heals 25 flat. That is 7% of Bryn's 334 HP and 12% of a 200 HP ranger, about half of one enemy strike (~49, forecast in `combat-ui2-l3/after_timeline.png`).
  - `con_fire_bomb` does 20 damage, 18-31% of a goblin with 65-108 HP.
  - The bomb costs 40 against a start gold of 50 (`world.json`).
  - DR1 3.1 made items usable so that shop and chest rewards matter. At these numbers a pouch slot is not worth a planner block. HEAL1 ("potions stay battle-only") makes a good potion more valuable, not less.

Options (**try later as an option build (paused)**: whether a potion saves a hero is a feel question):

- A (recommended): data only. The potion heals 90 (27-45% of an L5 hero), the bomb does 45, start gold rises from 50 to 100. Three upserts, no code. Shop prices stay, so gold still buys only a few.
- B: a rule. A new `heal_pct` field heals a share of max HP (for example 30%), and the bomb gets a PLUS area. It scales with levels and is a real bomb, but it needs a logic task (consumable fields, item command, validation).
- C: wait for HEAL1-B and BREPORT1 numbers on how often items are used. No work now, but items stay dead in the meantime.

## 6. World map, travel and quests

Working:

- MAPVIEW1 (cf333ab), MAPREAD1-L1/L2/A (bf162bb, 73f31fa, 092c1d6): full-screen maps, marker icons, NPC and exit names always shown, quest badges.
- The world map hover card shows trip time, supplies and quest (`mapread1/after_world.png`).
- WORLDMAP-ENTER (0bd96e3): click your own pin to enter.
- QUESTX and EXPWORLD (2d9db38, 74f4748): quests keep tracking across connected worlds.

Needs improvement:

- **Off-screen markers.** The Warden Keep frame (`mapread1/after_region.png`) shows 3 of the 7 "Here" entries. Hest and the chest are off-screen, there is no pointer, and the middle of the view is empty grass.
- **The party token reads worst.** It is a plain yellow pin, while NPCs have portraits.
- **World map framing.** The legend box covers the Warden Keep silhouette, and grey letterbox bars sit above and below the backdrop (`after_world.png`).

Options (**try later as an option build (paused)**: layout feel):

- A (recommended): edge arrows for every "Here" entry outside the view. Clicking or hovering a row pans the camera there; this is not auto-walk, so it keeps MAPREAD1's "No auto-walk". The party token uses Bryn's portrait token. Small changes in `region_map_view.gd` and `map_art.gd`. Cameras belong to UI/UX, and minimap and camera were out of MAPREAD1's scope, not rejected.
- B: smaller maps. Assets cut the Ashlands maps to fit the view (about 31x20). Data only, but it undoes the 48x30 choice from MAPREAD1-A and crowds the markers.
- C: leave it for real art (S14). Least work, but the user's DR1 (5) complaint ("empty and confusing") stays half-fixed.

Conflicts: HEAL1-U and UIR-6 also edit `region_map_view.gd` / `world_map_view.gd`, so land this after them or inside UIR-6.

## 7. Story and dialogue

Working:

- RECRUIT1-A: five named story recruits and the Toller muster.
- OBJ1-A: Wenn Tallow as a guest in the wagon raid.
- BOSS1-A: the Crown of Nails phase line on Grakka.

No other change since DR1. The dialogue layout (pick B) and DESERT1 are queued. No new item.

## 8. Battles: pacing and difficulty

Working:

- BAL4-ATTR v4 shows a curve. B1-B3 never down the party. The finale is the threat: the arms stand is lost in 45 of 100 chains even with a full heal, the gate stand in 3. The steel choice reads in the numbers, with the barred stand at 97% clear, which is intended as the reward.
- OBJ1-B: the solo `courtyard_skirmish` hold is in band (100).

Needs improvement:

- **The middle of the chain is flat.** There are no B1 or B3 falls in any heal mode, and `wagon_raid` is cleared 100 by all 10 trios in 1.7-2.1 rounds (band 70-90). HEAL1 is queued to give this stretch a cost through HP carry-over.
- **The finale is lopsided by steel choice.** The arms stand clears 60 (band 30-50), the barred stand 97. BAL5 levers B1-B3 (trio re-measure, sim phase, boss HP 1.8 on trigger) are queued. B4, "boss acts twice", was sent here.

Options (`wagon_raid`):

- A: one more raider, so the ambush puts two behind the party as the OBJ1 acceptance assumed (Gameplay handover notes only one today). More B3 HP drain and more finale falls under HEAL1.
- B: make Wenn `protect` (the battle is lost if he falls). This changes what a good plan is, not the numbers, but adds some risk.
- C (recommended): leave it until HEAL1-B. With HP carried over, an easy B3 is the breather before the fall point; HEAL1-B's pass line (arms finale 30-55) already decides whether B3 needs teeth.

Options (boss acts twice, B4; **try later as an option build (paused)**):

- A: a boss-tier unit drafts a second short sequence each round, with its own AP pool, resolved in the same blocks. A new boss feel, but it touches enemy scheduling and INTENT1 tells (two records). It conflicts with BAL5-A boss HP, so pick one.
- B (recommended): keep BAL5's order (B1, B2, then B3 on trigger) and revisit B4 only if the trio sweep boss mean stays above 60 at L5. Measure first (DR1 (3)).
- C: drop B4.

## 9. UI and readability

Working:

- COMBAT-UI2-L1/L2/L3 (7f66209, 1c6fea4, bb98f4f): layout C with the grid as the largest area, a hover card that pins, and tactics in the timeline row (`combat-ui2-l3/after_timeline.png`).
- UIR-1 (3e4f6d3): shared frame and No-first focus.
- BATTLE-FLOW (608d45a): transition and victory popup.
- POI-FIX (e143307) and NOTE-BUTTON (68f5ac3).

Needs improvement (frames from INTENT1-U, in flight on the UI/UX thread, not yet on `develop`):

- **The pinned hover card covers about a third of the grid.** It is about 270x560 px over a 768x576 grid (`intent1-u/after_planning.png`). The "Threat intel" block (4 lines) plus Intent plus "Versus your hero" pushes it to full height. At the largest text size the "Versus" block drops off (`after_xl.png`).
- The hostile cards' "Next: Move toward Ranger (..." line is cut at 1280 px.
- Two hostiles listed as active read "Arriving" (Brawler, Skirmisher in `after_planning.png`). `IntentRules` uses ARRIVING when a unit has no scheduled action, so this is either a test fixture or a real empty enemy plan. UI/UX should check it before INTENT1-U lands.

Options (hover card):

- A (recommended): collapse "Threat intel" into one line (role, strike, flee threshold) with a toggle for the full block. The card shrinks by about 4 lines and nothing is lost. Small, in `unit_inspector_ui.gd`, as a UI/UX row after INTENT1-U.
- B: a pinned card docks into the right column in place of the Forecast. It frees the grid but hides the forecast while pinned; a layout change to decision COMBAT-UI2 C.
- C: leave it.

## 10. Audio and art

Working:

- MUSIC1/MUSIC2 (a836119, 5a1f677): music by scene context.
- ART1-A/ART2-A (98d6da5, 0218602): Ashlands placeholder tokens, portraits, chips, ring tile, miss sound.
- SOUND-PREVIEW (bf53760).

No other change needed. The party token is covered in section 6.

## 11. Saves and settings

No change since DR1 beyond working additions: CONTROLS-HELP (3a2ede5: Controls tab, Settings during play, Mute), save v7 (autoplay) and v8 (pouch) with migrations. HEAL1 brings v9.

## 12. Playtest notes

- `playtests/` holds only the README: no user recording or inbox note since DR1. The last user evidence is the DR1 answers and the `battle-flow/before_*_user.png` screenshots.
- The loop the user has not played yet: solo Bryn, Toller muster, duo Cinder Gate, trio finale.

Options:

- A (recommended): one recorded run on `develop` from New Game to the Cinder Gate, with F8 notes where it feels off. This checks items 1-3 of the summary by hand before HEAL1 changes the loop.
- B: wait for HEAL1 and play once then. One run instead of two, but the first-hour issues stay unconfirmed.
- C: rely on sims only.

## 13. Approval list

One line per item: approve / skip / pick A, B, C. "(paused)" means try later as an option build once option builds resume. Until then, an approval of a paused item means "build it the normal way" only if you say so.

1. `ash_scouts` solo fix (section 3): pick A one scout fewer as the first change of ASHPACE1 / B gate scouts until the first recruit / C leave. **Recommend A.**
2. Battle item strength (section 5, paused): pick A data (potion 90, bomb 45, start gold 100) / B `heal_pct` rule plus bomb area / C wait for HEAL1-B. **Recommend A**, as a normal asset task now if you do not want to wait for option builds.
3. Region map navigation (section 6, paused): pick A edge arrows, pan on row click, Bryn's token / B smaller maps / C wait for art. **Recommend A**, landing with or after UIR-6.
4. Role feel levers T4 Bodyguard, S3 Ward, A4 self-centred burst (section 2, paused): pick A try all three / B only the gap BAL5 leaves open / C skip. **Recommend A** once builds resume, B if not.
5. Boss acts twice, B4 (section 8, paused): pick A build / B only if BAL5's boss trigger fires / C drop. **Recommend B.**
6. `wagon_raid` too easy (section 8): pick A one more raider / B protect Wenn / C leave until HEAL1-B. **Recommend C.**
7. Combat hover card size (section 9): pick A collapse threat intel / B dock into the right column / C leave. **Recommend A** (UI/UX row after INTENT1-U).
8. "Arriving" on active hostiles (section 9): approve as a check inside INTENT1-U (no new row) / skip. **Recommend approve.**
9. Playtest pass (section 12): pick A record New Game to the Cinder Gate now / B after HEAL1 / C sims only. **Recommend A.**

## Questions for you

1. Item 2: should a potion heal a fixed amount (simple, the same at every level) or a share of max HP (scales, needs a rule)?
2. Item 4: should Bodyguard be a skill the warrior plans each round (costs AP and a block), or a stance-like effect that is always on while he guards?
3. Item 5, if ever built: only authored bosses (Grakka), or every boss-tier unit?
