# Design review (2026-10-18)

For Tushar. I read `origin/develop` (head 184f09c, "CAMP1-A1/A2 and TRAV2-A landed"). Sources: `games/emberwake/docs/GAME_BRIEF.md`, `games/emberwake/docs/ROADMAP.md`, `docs/TASKS.md`, `docs/HISTORY.md`, `docs/decisions/*.md`, `games/emberwake/docs/BALANCE_REPORT.md`, the specs (CAMP1, TRAV2, AUTO1, BAL3, S9-S12 rules, UI menus), the roadmap review of 2026-10-17 and UI_REVIEW1. I also checked a few claims against `data/` and `scripts/`. Nothing in the repo was changed.

Sizes: S = one small task, M = one logic task plus one data task, L = a milestone of several tasks.

## 1. Where we are

The engine is ahead of the game. Almost every system a tactical RPG needs exists and is tested: planned combat with stances and AP, six classes with 57 skill families, gear with tiers, statuses, terrain and combos, quests, region maps, travel with supplies, dialogue, shops, reputation, saves and menus. The Ashlands now has a full story arc (CAMP1, "The Long Dry") and real battles from the map (BATTLE1). What is thin is the part a player feels between fights and inside them: fights are easy, they give only a little XP, nothing carries over from one fight to the next, and several systems (potions, supplies, the priest) have little or no job in the actual campaign.

| System | State today | Verdict |
| --- | --- | --- |
| Combat core | 12x9 grid, 6-block simultaneous rounds, stances, AP, 14 statuses, terrain and combos on develop (ROADMAP S1, S12) | Strong engine; thin in use (see 2) |
| Enemy AI | Kiting, heals, skill use, BAL3-AI pressure levers; autoplay for the party reuses it (AUTO1) | Strong |
| Classes and skills | Six classes, 57 families with ranks and level unlocks; duel warrior 90, priest 3 (`games/emberwake/docs/BALANCE_REPORT.md` section 5) | Strong content, unbalanced |
| Progression | XP curve, level-up screen, spend pool; party starts at level 5 (`data/catalog/units.json`) | Thin: one or two level-ups per campaign |
| Gear | Families, tiers, effects, gear budget; 5 loot entries and 1 named item in the Ashlands (`data/worlds/ashlands/loot.json`, `items.json`) | Strong system, thin loot |
| World and travel | 4 Ashlands locations plus Slag Gate, 3 routes, fog, world clock, supplies (TRAV1, TRAV2) | Strong system, small map |
| Story and dialogue | 6 quests, 4 dialogue trees, one real choice (steel: gate or arms), boss finale (CAMP1) | Thin but complete |
| Economy | Gold from quests, chests and selling only; no kill gold (decision "BATTLE1-L battle link") | Thin, partly broken (see 2) |
| UI | Menus, party, journal, results done; UIR-1 to UIR-7 and combat layout C approved | In progress |
| Audio and art | Pipeline done, placeholder sounds and art (S14) | Thin (expected now) |
| Tooling | Sim with party, sweep, role, duel modes; playtest recorder and replay; web build | Strong |

## 2. What's working and what's thin

### Working well

- **The planning core is distinctive.** Six simultaneous blocks, stance rock-paper-scissors (1.25 / 0.75) and AP debt give real "read the enemy" decisions. Keep it as the heart of the game.
- **Data-first content.** Every number is data (HISTORY "Riposte AP ... as code constants" row), so tuning and new content never need code.
- **The story arc has a real choice with mechanical teeth.** Barring the gate removes the finale wave and lowers prices; arming the party adds gear (CAMP1 section 4).
- **Autoplay is the right comfort feature** for easy fights (AUTO1), as long as fights are not designed around it.

### Thin, and the gaps that hurt a first Ashlands run

1. **Fights are free, so between fights nothing matters.** The party is fully healed after every battle (M3), a loss wakes the party fully healed "with no further cost" (BATTLE1-L), and a downed hero still gets full XP. There is no attrition, no risk and no reason to retreat, rest or prepare.
2. **Potions and bombs do nothing in a real battle.** Hest sells `con_heal_potion`, `con_focus_elixir` and `con_fire_bomb` (`data/worlds/ashlands/shops.json`) and chests give potions, but the battle pouch is only usable in the Dojo (`scripts/dojo/dojo_session.gd` `use_pouch_item`); `CombatPlan` and `CombatSession` have no item action. With the full heal after battle, potions have no use at all. This is the clearest case of the design working against the fun: the player buys things that cannot be used.
3. **Supplies have no decision behind them.** At 0 supplies the road fight chance is x1.5 (TRAV2-L 2.2). Because fights cost nothing (point 1) and give little (point 4), running out is a mild nuisance, never a choice. Supplies are only ever spent, never used for something good.
4. **Progression is nearly flat.** The party starts at level 5; Ashlands mobs are level 3 minions. A minion kill gives about 9 XP per hero ((10 + 15) x 0.5 x 0.7), so a two-scout fight gives about 18 and level 6 needs 220, roughly 12 fights. The boss gives about 230. My estimate: one or two level-ups in the whole campaign, and the tier 3 skills unlock at levels 7 and 9.
5. **Most fights are too easy and look alike.** Line, elite and boss encounters clear 100% at levels 3, 5 and 7, up to two enemy levels higher (BALANCE_REPORT headline 1). Every fight is "kill everything"; the only twist so far is one reinforcement wave.
6. **The party the player has is not the party the sim tests.** The Ashlands start party is warrior, ranger, rogue, mage (`data/worlds/ashlands/world.json` `start_party`). The balance default comp is warrior, ranger, priest, mage. The player never meets a priest or brawler in play, and there is no way to recruit one (no recruit effect in dialogue). Much of BAL3's support work tunes a hero the player cannot field.
7. **The combo and status systems are barely used.** Only `feint` (flow), `crushing_blow` (stance break) and `opportunist` (exploit) carry combo fields in `skills.json`, and the sim saw zero poison or burn damage in party fights (BALANCE_REPORT "Observation").
8. **Feedback is thin.** The results screen shows XP only (no gold, no loot from fights). In planning, enemy intel is static (role, strike power, flee threshold in `combat_inspector_panel.gd`), so in a simultaneous game the player plans partly blind.
9. **The heroes have no names.** The four starters are the catalog units "Warrior", "Ranger", "Rogue", "Mage"; the named characters (Mara, Tev) are NPCs.

## 3. Improvements on top of current systems

### 3.1 Use items in battle

- What: a "Use item" step in the planner for the battle pouch (potion, focus elixir, fire bomb), 1 block, uses `Inventory.use_pouch_item` and `Consumable.apply_to_combatant`; pouch filled from the stash on the gear tab.
- Why: makes shop and chest rewards meaningful and gives the four-hero party without a priest an emergency heal.
- Size: M. Builds on: `scripts/core/inventory/*`, `CombatPlan`, AUTO1 (autoplay must not drink potions on its own unless a tactic allows it).
- Risk: AI and sim need one rule (enemies with pouches); undo snapshots must cover the pouch.

### 3.2 Fewer, meaner fights in the Ashlands

- What: a data pass on the Ashlands only. Step counter 12..20 raised to about 20..30, mob levels from 3 to 4 (party level minus 1), one more enemy per mob group, elite chance kept.
- Why: fewer filler fights, each one worth some thought and some XP; also eases point 4 (a level 4 minion vs a level 5 hero gives about 13 XP instead of 9).
- Size: S (asset lane). Builds on: S9 baseline (these are tuning numbers, not rules), EXP1-A zone bindings.
- Risk: none structural; verify with 3.3.

### 3.3 Campaign-true balance run

- What: extend CAMP1-B into a "campaign sim": party mode with the real start party (warrior, ranger, rogue, mage) at level 5 against every Ashlands encounter, both steel choices.
- Why: today's numbers describe a party the player never has (point 6).
- Size: S (balance lane; `tools/fc.sh sim party` with a comp argument already exists).
- Risk: none; it is a report.

### 3.4 Recruit the priest and the brawler

- What: a dialogue effect `recruit` (adds a unit to the roster) and two recruitable Ashlands characters, for example a field priest at Warden Keep after the oath and a pit brawler at Slag Gate. The party screen already has Active / Reserve.
- Why: party building becomes a choice (pick 4 of 6), a second run plays differently, and the priest work in BAL3 starts to matter.
- Size: M (one effect key plus save field already covered by the roster; two characters and dialogues as data). Builds on: `PartyState.add_member`, DLG1 node effects, UIR-3.
- Risk: the start-party decision ("starting party fixed per world") stays; this only adds members later.

### 3.5 Named heroes

- What: give the four starters names, a portrait key and one background line (world characters with role `party`), and let `start_party` point at them.
- Why: "Bryn is at 22% HP" (the AUTO1 example) only works if Bryn exists; story beats can address the heroes.
- Size: S (data, plus a small manifest check if `start_party` must accept character ids).
- Risk: low.

### 3.6 Enemy intent tells

- What: during planning, show each enemy's chosen target (an arrow) and the kind of its first action (attack, guard, move), from the enemy draft that already exists. Optional depth: a party member with PERC 14+ also sees the stance of that first action.
- Why: simultaneous rounds with deterministic hits reward reading the enemy; today the player guesses. This is the biggest lever for moment-to-moment decisions.
- Size: M. Builds on: `TacticalAI` drafts, `combat_inspector_panel.gd`, combat layout C hover card.
- Risk: a taste decision. It changes the feel toward "Into the Breach". Enemies must not re-plan after seeing the player, or the tells lie.

### 3.7 Battle report on the results screen

- What: per hero damage dealt, damage taken, heals, statuses applied and combos triggered, plus one "best moment" line.
- Why: players learn what worked; combos and statuses become visible.
- Size: S-M. Builds on: the BAL2 per-unit metrics collector, `BattleResultsModel`, UIR-4.
- Risk: low.

### 3.8 Combo and status content pass

- What: after G5 is frozen, give each class one flow move, one exploit and one status setup that pairs with another class (for example warrior `sunder` expose, rogue exploit vs expose).
- Why: the combo engine exists but almost nothing uses it (point 7); cross-class setups are the fun of a party game.
- Size: M (asset lane). Builds on: S12C-A, S12S-A.
- Risk: a balance rerun afterwards (G5 finish line already asks for it).

## 4. Alternative systems not yet considered

### 4.1 Battle objectives and deployment variants

- Pitch: encounters get an `objective` (rout, hold a tile for N rounds, protect an NPC, reach the exit, slay the chief within N rounds) and a `deployment` (standard, ambush with enemies on both flanks, surrounded, defend the centre).
- Hooks: encounter data already has exits, waves, `max_rounds` and outcomes won / fled / lost (EXP1 X1); `CombatSession` checks one more win rule.
- Size: M.
- Why: line, elite and boss fights all clear 100% and play the same (point 5); objectives change what a good plan is without touching the damage formula.

### 4.2 Injuries and making camp

- Pitch: HP still refills after every battle (M3 stays). A hero who is downed gets an "injury" (a temporary passive, for example -2 to a stat) until the party rests. Rest = an inn service at a SAFE location for gold, or "Make camp" on the world map for 2 supplies.
- Hooks: temporary passives are already allowed for events ("Passives lifetime"); world-map view (TRAV2 readouts); service lines (DLG1).
- Size: M.
- Why: gives fights a cost, gives supplies and gold a positive use, makes potions and a priest valuable.
- Note: this layers a cost on top of "wake with no further cost". The S9 rules left that door open ("C can be added once S11 gold exists"), but it needs your explicit yes.

### 4.3 Enemy morale

- Pitch: when the boss or elite falls, or half the pack is down, minions break: they flee for the exit or surrender (the fight ends as a win).
- Hooks: `AiProfile.flee_ratio` and the exit-tile flee path already exist; one pack-level trigger.
- Size: S-M.
- Why: kill the chief first becomes a real tactic, mop-up rounds disappear (pacing), and goblins feel like goblins.

### 4.4 High ground

- Pitch: a terrain `height` field (0 or 1). Ranged units on high ground get +1 reach; ranged hits from low to high get half cover.
- Hooks: S12T terrain table and cover input (inputs only, the formula is untouched).
- Size: M.
- Why: positional choices on the 12x9 grid. Wait until G5 has a balance rerun.

### 4.5 Warden Keep upgrades

- Pitch: the keep is the party's base. Quest rewards and gold unlock three upgrades: forge (Hest stocks a better tier), infirmary (free rest, see 4.2), training yard (respec, see 5.2).
- Hooks: flags, `requires_flags` on shop stock (CAMP1 already does this for the arms choice), REP1 bands.
- Size: M, mostly data.
- Why: a visible home that grows; gold gets a goal beyond potions.

### 4.6 Contracts board

- Pitch: a board at the keep offers repeatable contracts built from the lore-free quest templates (clear a zone, escort, hunt a roamer) for gold and supply rewards.
- Hooks: `QuestCatalog` templates, world binding, `QuestRunner`, the seeded `GameRolls` stream.
- Size: M.
- Why: something to do after the story; a gold source that is authored, not per kill.

### 4.7 Difficulty options

- Pitch: on New Game choose Story, Normal or Hard (Hard: mobs +1 level, elite chance doubled, injuries on), plus an Ironman switch (autosave only).
- Hooks: `NewGame`, encounter level offset at build time, the save header.
- Size: S-M.
- Why: replay value, and a cheap answer to "too easy" while BAL3 finishes.

Considered and not proposed: crafting from materials (a new economy for a one-region campaign, L for little gain), hero bonds (L, better after more story exists), skill-created terrain (S12 Q-T3 B; keep for after G5).

## 5. Dropped ideas, revisited

Each item names the original reason it was dropped and why the new form avoids it. HISTORY rows about tooling, data formats, schema shapes or workflow (hard-coded content, fixed-power goblins, stat totals, proficiency lists, per-world tiers, vacated tiles, tier scaling of authored power, build orders, riposte constants, item and skill schema rows, seeds, the legacy AI path, `.context` notes, trailers, variant recordings) have no good gameplay revival and are skipped. The archived decision rows (`docs/decisions/archive.md`) are all superseded by rows that kept their intent, so none needs a revival.

### 5.1 MasterySystem (HISTORY, 2026-10) as deeds

- Dropped because: mastery lived on the unit and class; traits should come from lore sources.
- New form: a world kind `deeds`. A deed counts a feat through int flags (REP1 already has int flags with comparisons), for example "30 kills with a dagger" or "win 5 fights without anyone downed". When reached, the deed grants a passive, so the source is the deed, not the class.
- Size: M. Why now: levels are nearly flat in the Ashlands (point 4), so progression needs a second track.

### 5.2 Free level-up allocation (HISTORY, 2026-10) as paid respec

- Dropped because: units needed an exact, enforced spend total.
- New form: the total stays exact; a trainer (or the training yard, 4.5) lets you re-spend the same pool for gold.
- Size: S. Why now: spend is permanent in game (UI2-L), and "choices are flavour" ("Level-up spend") argues for forgiveness.

### 5.3 Shaman, berserker and scout roster (HISTORY, 2026-10-15) as specializations

- Dropped because: the old roster had no support role and was melee heavy.
- New form: the roster stays six. The reserved `expert` grade becomes a one-time choice between two expert families per class at level 6 or 7, for example warrior "Berserker" (rage, gives up guard) or "Bulwark"; priest "Shaman" (regen fields) or "Inquisitor".
- Size: L (12 families of content). Why now: a build decision per hero, and replay.

### 5.4 Off-hand shortsword, knuckles and dual wield (HISTORY, 2026-10-16) as off-hand skill items

- Dropped because: mixed hand categories broke bands and slot rules; off-hand slots are for support items.
- New form: off-hand items stay one category (`off`) but may grant one skill while equipped, for example a parrying dagger granting "Parry" (a defend) or throwing knives granting a short ranged jab. Passive templates already have a `skill` effect; item effects would allow it.
- Size: S-M. Why now: loot is thin (5 entries); items that change how you play beat items with +2 power. The decision "FORGE1 round 5" left "a future system may use the off hand" open.

### 5.5 9x9 grid with a centre ring (HISTORY, GRID12 2026-10-17) as deployment variants

- Dropped because: side against side reads better for a standard encounter.
- New form: standard fights keep left vs right. Non-standard ones (4.1) use other layouts, and "defend the centre" brings the ring back as an objective tile.
- Size: part of 4.1.

### 5.6 Hand-scripted combat layer (HISTORY, before 2026-10) as boss phases

- Dropped because: scripted commands could not express planning, AP and AI.
- New form: no scripted commands. An encounter gets `phases`: a trigger (a unit below X% HP, or round N) and data effects (spawn a wave, show a line, grant a status, switch the AI profile). The planner and AI run as normal.
- Size: M. Why now: Grakka's finale is the climax of the only campaign; a two-phase boss ("Grakka tears off his crown and the shamans start chanting") is what players remember. Waves already prove the trigger path.

### 5.7 Kill gold (S11 rules Q4 option A, not chosen) as an encounter purse

- Not chosen because: authored income kept the reward budget the only economy lever.
- New form: still authored. Each encounter may carry a `purse` (gold range, optional loot entry), shown on the results screen.
- Size: S. Why now: fights give only XP, and 4.2 and 4.5 add things to spend gold on.

### 5.8 Supplies weaken at 0 (S10 rules Q6 option C, not chosen) as a positive use

- Not chosen because: a combat penalty contradicts "no fail state".
- New form: no penalty. Supplies buy rest (4.2 Make camp). Folded into 4.2.

### 5.9 Bestiary (UI menus Q-M3 option C, deferred)

- Deferred because: quests only for the first journal slice.
- New form: a bestiary tab that fills as you fight; after the second fight against a unit type its skills and stance habits show in the hover card.
- Size: S-M. Why now: it is the cheap half of 3.6 and rewards fighting.

### 5.10 The deserter hook (CAMP1 Q1, not chosen) as a second choice

- Not chosen because: the steel choice was the better first hook, not because the deserter was bad.
- New form: add it beside the steel: jail him (Warden rep) or free him (a back route that skips the wastes track road fights).
- Size: S (one character, one dialogue). Why now: one choice per campaign is little replay; two give four combinations.

Not revived: reputation ripple (S11 Q7 B, still hard to author), rolled dialogue checks (S11 Q6 C, randomness the brainstorm avoided), mob level scaling (S9 Q5 B, kills the progression feel), silence and blind (S12 Q-S1 C, high AI risk), save only on safe tiles (S9 Q6 B).

## 6. Decision list

### Top 5 picks (for the first playable Ashlands build)

1. Campaign-true balance run (3.3): measure the party the player actually has before tuning anything else.
2. Use items in battle (3.1): today the shop sells things the player cannot use.
3. Fewer, meaner fights (3.2): one data pass that fixes pacing and XP together.
4. Battle objectives and deployment variants (4.1, 5.5): fights stop all looking the same.
5. Boss phases for Grakka (5.6): the finale is the moment the build is judged on.

### All decisions

1. Campaign-true balance run: approve / skip  (recommend: approve) - small, and it makes every later balance number about the real party.
2. Use items in battle: approve / skip  (recommend: approve) - shop and chest rewards are dead without it.
3. Fewer, meaner fights: approve / skip  (recommend: approve) - pure data, fixes filler fights and slow XP.
4. Recruit the priest and the brawler: approve / skip  (recommend: approve) - party choice and replay; makes BAL3 support work count.
5. Named heroes: approve / skip  (recommend: approve) - cheap, and story and autoplay banners need names.
6. Enemy intent tells: approve / skip  (recommend: approve) - the biggest lever on in-fight decisions; spec first, it changes the feel.
7. Battle report on the results screen: approve / skip  (recommend: approve) - shows players what worked, reuses sim metrics.
8. Combo and status content pass: approve / skip  (recommend: approve) - after G5; the engine is built and unused.
9. Battle objectives and deployment variants: approve / skip  (recommend: approve) - variety without touching the formula.
10. Injuries and making camp: approve / skip  (recommend: approve) - gives fights a cost and supplies a use; touches the "no further cost" default.
11. Enemy morale: approve / skip  (recommend: approve) - small, shortens mop-up, rewards focusing the chief.
12. High ground: approve / skip  (recommend: skip) - good later, but only after the G5 terrain rerun.
13. Warden Keep upgrades: approve / skip  (recommend: approve) - mostly data, gives gold a goal.
14. Contracts board: approve / skip  (recommend: skip) - post-story content; wait until the campaign is fun.
15. Difficulty options: approve / skip  (recommend: approve) - cheap replay value and a stopgap for easy fights.
16. Deeds (mastery revived as a source): approve / skip  (recommend: approve) - a second progression track while levels are flat.
17. Paid respec: approve / skip  (recommend: approve) - small, removes fear of permanent spend.
18. Specializations via the expert grade: approve / skip  (recommend: skip) - strong idea but L; revisit after G3.
19. Off-hand skill items: approve / skip  (recommend: approve) - makes loot interesting with little engine work.
20. Boss phases: approve / skip  (recommend: approve) - makes the finale memorable; waves already prove the path.
21. Encounter purse: approve / skip  (recommend: approve) - authored, boundable fight rewards.
22. Bestiary: approve / skip  (recommend: skip) - nice, but do 3.6 first.
23. Deserter as a second story choice: approve / skip  (recommend: approve) - small data task, doubles story replay.

## Questions for you

1. Intent tells (6): how much should the player see? Target only, target plus action kind, or nothing (keep the blind read)?
2. Injuries (10): are you happy to add a cost to defeat and to being downed, given the current "no further cost" default?
3. Recruits (4): should the priest and brawler join through the story (fixed), or should the player pick two of several at the keep?
