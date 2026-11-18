# PLAYTEST_BRIEF1: systems playtest by an agent session (draft for the user)

Status: **draft, shown to the user before the session starts** (user 2026-11-18: "add the full playtest but have it be
conducted by a Fable session, which is supposed to give feedback on the core systems"). Runs on the harness from READY
`PR7-22` (`AGENT-PLAYTEST1`) against `develop`. Format: **hybrid**, a must-cover checklist first, then an open roam.

## 1. Session rules

- Play the real game through the harness (virtual display, agent link inputs), from a fresh save, as a player would:
  no data edits, no code reading to decide what to do, no debug keys. Reading the game's own tooltips and help is fine.
- Take a screenshot and a log line at every judged step; quote the screen, the numbers and the save day.
- Write every finding as one `PT-nn` note: where (screen, node, fight), what happened, what you expected, severity
  (`blocker`, `major`, `minor`, `polish`), and a repro (save, steps, seed). Bugs and design opinions are separate notes.
- Do not fix anything. The report goes to Design, which triages it per `docs/workflow/cloud.md` "Playtest intake".
- Budget: Act 1 in full, then Act 2 as far as content exists; stop and report if blocked for more than three tries.

## 2. Must-cover checklist (judge each, pass / issue / not reached)

| # | System | Judge |
| --- | --- | --- |
| 1 | Onboarding and the opening | Is the first ten minutes clear: what to do, what the controls are, why the fight matters? |
| 2 | Combat core | Turn and timeline flow, AP, reach and range, hit and damage readability, forecasts that match results |
| 3 | Enemy intent and tells | Can you predict the enemy turn from the tells; do intents match what then happens? |
| 4 | Poise, stagger, statuses | Are breaks, bleed, root, weaken etc. visible, explained and worth playing around? |
| 5 | Morale and Composure | Do Shaken and its bands show and matter; any unfair spirals? |
| 6 | Strain, Channel, grafts (Act 2) | Is the risk legible; do Calcification stages and Vent feel like choices? |
| 7 | Skills, decks, keystones | Do builds differ; is the deck editor clear; does the level 6 keystone arrive and matter? |
| 8 | Items, gear, wear and mastery | Equip flow, durability and Dulled, mastery gains, relic signatures, results lines |
| 9 | Progression and economy | XP pace, Scrap income vs costs, shop prices, supplies; anything grindy or trivial? |
| 10 | Recruits, roster, crypts, Martyrs | Rescue and swear-in, temperaments, bunk cap, Martyr fuse and the Retire / Last Stand prompt |
| 11 | Hub, bunks, talks, facilities | Bunk states readable, talks unlock and play, lead passing, facility upgrades understandable |
| 12 | Maps and travel | Region and world map movement, labels, zoom, travel days and countdowns |
| 13 | Objectives and special fights | Rout, retreat, sieges, big 2 x 2 units, captive and swap phases, timeouts |
| 14 | Story flow and transitions | Vignette to battle and battle to hub beats, pacing, skip, Reduce motion |
| 15 | Saves | Save, load, autosave, a save made mid-campaign still loads after a restart |
| 16 | Settings | The approved settings screen (`PR7-21`): staging, revert, close guard, every row takes effect |
| 17 | Balance feel | Which fights felt too easy or too hard; compare with the sim bands (`tools/fc.sh sim bands`) |
| 18 | Text and lore | Placeholder text, wrong names, canon slips (`CANON.md` for reference after a finding, not before) |

## 3. Open roam

After the checklist, play on freely for the rest of the budget: try odd orders, retreat, lose a fight on purpose,
spend everything, ignore the story. Report anything confusing, broken, boring or delightful, with the same note format.

## 4. Report

One file `playtests/agent/<date>/REPORT.md` (summary, the checklist table filled in, the top five issues, then every
`PT-nn` note) plus the screenshots folder; the session posts its path to the Design thread.
