# AUTO1: Battle autoplay with a party tactic

Status: approved (user 2026-10-18, all recommendations: Auto, safety stops On, remembered Yes; tactic per row from the combat UI note; decision "AUTO1 battle autoplay" in docs/decisions/combat.md)
Depends on: nothing new (uses `PlanPresets`, `TacticalAI` with the BAL3-AI levers, `CombatSession`)
Lane: logic (no data)
Touches: `scripts/combat/session/combat_session.gd`, `scripts/combat/ai/plan_presets.gd`, `scripts/ui/timeline_dock_ui.gd` (row dropdowns), `scripts/ui/combat/combat_timeline_panel.gd`, `scripts/ui/combat/combat_playback_panel.gd`, `scripts/game_state/game_state.gd` (tactics per save, kept in the save like other party state; save version per the builder), a new test suite

## Goal

Easy fights should not need the same clicks every round. The player picks a party tactic once, turns Autoplay on, and the AI drafts every hero's sequence each round, confirms it and plays the round, round after round. The player can stop it at any time, edit the timeline and carry on.

## What exists today (no change to the AI itself)

- Every round the player can already open **Draft ▾** and pick one of seven presets for one hero or the whole party (`PlanPresets`: Balanced, All-out Attack, Focus Weakest, Guard & Counter, Kite at Range, Economy, Fall Back). A preset adjusts the hero's own `AiProfile` (its class `ai_role`) and `TacticalAI.build_steps` drafts the sequence, the same code the enemies use, BAL3-AI levers included (Focus only while safe, fast option when about to die, last-slot heal, taunt commit).
- What makes it tedious is the loop around it: Draft, Confirm, Play, Next Round, every round.

## Rules

1. **Tactic choices.** The six combat presets: Balanced (default), All-out Attack, Focus Weakest, Guard & Counter, Kite at Range, Economy. Fall Back is not offered (autoplay never retreats on its own).
2. **Tactic dropdown on each timeline row** (user 2026-10-18 00:16). Every hero's timeline row starts with a dropdown: the six tactics plus **Custom**. A party dropdown above the rows sets every hero at once (for example the party on All-out, then the healer's row set to Balanced). Picking a tactic redrafts that hero's row at once.
3. **Custom.** As soon as the player edits a drafted row (adds, removes, moves or clears a step), its dropdown shows **Custom**. Picking a tactic again throws the edits away and redrafts. A Custom edit lasts for that round only: next round the row goes back to its tactic. Choosing Custom from the dropdown on purpose means "I plan this hero myself": autoplay then stops in planning every round with the other rows drafted and this row empty.
4. **Autoplay loop.** With Autoplay on, each round: at round start the session runs the party draft (`plan_apply_preset_all`, then each overridden hero with `plan_apply_preset`), confirms it, plays the blocks at the normal animation speed, then advances to the next round. Enemies plan as now.
5. **Editing.** Pressing Space or the Autoplay button pauses: the current round finishes playing and the next round stops in planning with the AI draft already on the timeline. The player edits it like any plan and presses Confirm; Autoplay stays paused until it is turned on again. Edits last for that round only; the next autoplayed round is drafted fresh.
6. **Safety stops.** Autoplay pauses itself (as in rule 5) when a hero is downed, or when a hero ends a round at or below 30% HP. A short banner says why ("Autoplay paused: Bryn is at 22% HP").
7. **Invalid draft.** If a draft ever fails `plan_is_valid` (PLAN1 makes this rare), autoplay pauses on that round with the broken steps highlighted, instead of skipping the hero.
8. **Remembered.** The party tactic, each hero's tactic (Custom included) and the Autoplay on/off switch are kept per save, so the next battle starts with the same choice. Autoplay starts off in a new game.
9. **Controls.** In the plan row: a `▶ Autoplay` toggle next to the party tactic dropdown; the old Draft ▾ menu is replaced by the dropdowns. Hotkey: A toggles Autoplay. The combat-screen layout (where the timeline sits) is UI/UX's mockup work; this spec fixes only what the dropdowns do.
10. **Playtests and replays.** Autoplayed rounds are recorded as normal confirmed plans with `source: auto`, so replays and take-over work unchanged.

## Acceptance checks (tests)

- Autoplay with Balanced finishes a sample easy encounter with no player input and the same result as the sim party preset on the same seed.
- Space mid-round: that round completes, the next stops in planning with a valid draft; Confirm plays it; the round after stays manual.
- A row's dropdown drafts that hero with its own tactic; an edit flips it to Custom; picking a tactic again redrafts; a deliberate Custom row stops autoplay each round.
- Safety stop fires on a downed hero and on the 30% line, not otherwise.
- The tactic and switch survive save and load.

## Out of scope

- New AI behaviour or new presets (a later spec if a tactic is missing).
- Auto-skipping animations (the existing animation speed / instant setting covers it).
- Autoplay for enemies or for exploration.

## Decisions for the user (answered 2026-10-18: all recommendations)

| # | Question | Options | Recommendation |
| --- | --- | --- | --- |
| Q1 | Does autoplay confirm rounds on its own? | **Auto** (rounds run until you pause or a safety stop) / **Confirm** (each round is drafted, you press Enter to play it) | Auto |
| Q2 | Safety stops | **On** (downed hero or a hero at 30% HP or less) / **Off** (only your pause stops it) | On |
| Q3 | Remember between battles | **Yes** (per save) / **No** (off every battle) | Yes |
