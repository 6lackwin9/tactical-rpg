# INTENT1: Enemy intent tells

Status: proposed (spec 2026-10-18; approved direction: decision combat "DR1 enemy intent tells (user 2026-10-18)", review `docs/specs/DESIGN_REVIEW1.md` 3.6)
Depends on: COMBAT-UI2-L1 (layout C columns) and COMBAT-UI2-L2 (hover card) for the UI part; the logic part needs nothing new
Lane: logic (INTENT1-L engine read-out and perk; INTENT1-U UI on the UI/UX thread), plus a tiny asset slice (INTENT1-A: one passive, one character edit)
Touches: L: `scripts/combat/session/combat_session.gd` (`enemy_intents`), new `scripts/combat/session/intent_rules.gd`, `scripts/core/specs/passives.gd` (effect type `foresight`), `scripts/core/combat/combatant.gd` (`has_foresight`), new `scripts/tests/intent_test_suite.gd`. U: `scripts/ui/combat/combat_grid_view.gd` (arrows, chips), hostile cards in the right column, `scripts/ui/unit_inspector_ui.gd` (hover card), `scripts/ui/timeline_dock_ui.gd` (masked banner), `combat_ui_framework_test_suite`. A: `data/catalog/passives.json`, `data/worlds/ashlands/characters.json` (bryn_hale, RECRUIT1)

## Goal

While planning, the player sees for every enemy what it does in its first block: the kind of action and its target. A perk lets its holder read the whole round plan of each enemy that is coming for the holder. Simultaneous rounds become a reading game without the enemy ever lying.

## How intent is computed (no AI change)

- Enemy plans already exist before the player plans: `CombatSession._begin_round` calls `encounter.schedule_round`, which drafts every enemy with `TacticalAI.build_steps` into one shared `CombatPlan` and commits it to `resolver.scheduled_actions` (`combat_encounter.gd` `schedule_enemy_ai`). The party plan is committed later, at Confirm.
- Enemies never re-plan after seeing the party plan, so a tell is always the plan that resolves. `TacticalAI` and `CombatPlan` use no random numbers, so `reset_round` (which re-runs `_begin_round`) shows the same tells.
- Units that arrive in a wave mid-round have no scheduled action that round and show "Arriving" instead of a tell.

## 1. Schema contract

- Passive effect type `foresight` (no params), added to `Passives.TYPES` and the effect table; `Combatant.has_foresight()` is true when any applied passive carries it. Allowed on passive templates only (not in the item effects grammar).
- `IntentRules.kind_of(cmd: BattleCommand) -> String`: AttackCommand `attack`, AreaAttackCommand `area`, SupportCommand `support`, DefendCommand `guard`, FocusCommand `focus`, RestCommand `rest`, DisengageCommand `retreat`, WalkCommand `move` (`flee` when its goal is an exit tile), TargetCommand `switch`, StatusBlockCommand `stunned`, anything else `act`.
- `CombatSession.enemy_intents() -> Array[Dictionary]`, read-only, only while `is_planning()`; one record per active enemy on the grid:

| Key | Type | Meaning |
| --- | --- | --- |
| unit | Combatant | the enemy |
| kind | String | `kind_of` of its first scheduled action (lowest `start_block`), or `arriving` when it has none |
| target | Combatant or null | attack and support: the command target; move: `WalkCommand.approach_target`; else null |
| cell | Vector2i | area: the aim cell; move without a target: the goal; else (-1, -1) |
| blocks | Vector2i | first action's start and end block |
| skill | String | display name of that action |
| revealed | bool | true when rule 3 holds |
| plan | Array[Dictionary] | only when revealed: every scheduled action in block order, each {kind, target, cell, blocks, skill}; else [] |

## 2. Rules

1. **Always shown.** Every enemy's first-block tell (kind, target or cell) is shown in planning, for every party, without a perk.
2. **Perk.** The `foresight` effect comes from a passive (passives come only from sources, never classes: GAME_BRIEF "Passives"). Shipped source (Q1 A): passive `wardens_eye` in Bryn Hale's character `passives`, copied into his `UnitState.passives` at New Game (RECRUIT1 rule 1).
3. **Who is revealed (Q2 A).** An enemy is revealed when at least one party unit that `has_foresight()` and is active is targeted by any of that enemy's scheduled actions this round: a single-target attack or debuff on the holder, or an area action whose planned cells include the holder's round-start tile. Taunted enemies count like any other.
4. **What is revealed.** The revealed enemy's whole round: every action, its blocks, skill and target, as its timeline track. Stance and AP are not part of the tell beyond what the track already shows.
5. **Display only.** Intents never change planning, AI drafts (party or enemy), resolution, recordings or the sim. Autoplay ignores them.
6. **Truth check.** A tell names the plan, not the outcome: a cancelled action (target down, stunned, rooted) resolves differently, and the log already says why.

## 3. UI hooks (layout C, `docs/specs/COMBAT_UI2.md`)

- **Grid:** in planning, each enemy token gets a small kind chip (sword attack, burst area, cross support, shield guard, star focus, boot move or flee, arrow retreat); an arrow from the enemy to its target token, or to its aim cell for area and move. Arrows use the hostile colour, dashed for `move`; hidden during resolution and replay. Hotkey `I` toggles tells (on by default; remembered in settings).
- **Hostile cards (right column):** the behaviour line becomes "Next: Shield Bash on Bryn (B1)" or "Next: Move toward Sella (B1-2)"; a revealed enemy shows an eye badge.
- **Hover card (COMBAT-UI2-L2):** the enemy section adds "Intent" with the first action; a revealed enemy lists its full plan, one line per action ("B3 Shield Bash on Bryn"), headed "Read by Bryn (Warden's Eye)".
- **Timeline row:** selecting an enemy keeps the masked banner ("Hostiles plan in secret", `timeline_dock_ui.gd`) but shows the block 1 tell in it; a revealed enemy's track is drawn unmasked like a hero track, read-only.
- **Forecast:** when the selected hero is the target of a hovered enemy's tell, the forecast line adds "Targets you in B1".

## 4. Acceptance checks (tests)

- For a fixed encounter and seed, each `enemy_intents()` record matches `resolver.scheduled_actions` (kind, target, blocks) at round start; editing, clearing or redrafting the party plan leaves the records unchanged.
- `reset_round` gives identical intents.
- An enemy whose only action is an Engage shows `move` with the attack target as `target`.
- With no `foresight` holder, every `plan` is empty; with Bryn holding it, an enemy attacking Bryn in block 4 only is revealed, one attacking only Sella is not, an area whose cells cover Bryn's tile is revealed.
- A wave unit arriving mid-round reports `arriving`.
- Combat results of a recorded replay are identical with tells on and off.
- UI (U): at 1280x720 all four text sizes, chips and arrows never cover the hover card; `I` hides and shows them.

## 5. Assets (INTENT1-A)

| Kind | id | Data |
| --- | --- | --- |
| passive (catalog) | wardens_eye | name "Warden's Eye", description "Reads the foe coming for you: shows the whole round plan of every enemy that targets this hero.", effects [{"type": "foresight"}] |
| character edit (ashlands) | bryn_hale | passives ["wardens_eye"] (needs RECRUIT1-A) |

## Out of scope

The review's PERC 14+ stance tell (the user chose a perk instead), enemies that react to the player's plan, intent for allies, bestiary (skipped in DR1), sound cues.

## Questions for the user

1. Who holds the perk?
   - (recommended) A: Bryn, the warrior, from the start (passive `wardens_eye`). The tank pulls enemies with taunt, so he reads exactly the foes he draws; while he is alone he reads every foe, which teaches the system.
   - B: a reward of the oath quest (`warden_oath`), the player picks which hero gets it. A story beat, but the first hours have no perk.
   - C: any hero with PERC 14 or more (stat rule, the review's idea). No source needed, but it is a class-stat perk, against the rule that passives come from sources.
2. What counts as "targeting the perk holder"?
   - (recommended) A: any action of that enemy this round aims at the holder (attack, debuff or an area over the holder's tile). Matches "enemies targeting the perk holder" for the whole round.
   - B: only the first-block target. Simpler to read (follow the arrow), but misses enemies that walk first and strike later.
