# Milestones for a new game (template)

Status: canon (user 2026-10-25). The default plan for building a game top down on Aegis. Game-agnostic.

| Milestone | Scope | Exit |
| --- | --- | --- |
| M0 Foundation | The game's folder, manifest and save format; the design library locked (consistency pass, gaps filled); kit pieces the game needs moved out of earlier games | The empty game boots to its menu; gate and parity green; decisions written |
| M1 Feel prototype | The core loop's signature mechanic in the Dojo or arena (for a tactics game: combat feel) as a ruleset option | Recorded golden traces; a sim report on the new rules; earlier games unchanged |
| M2 Slice on data | The smallest playable path through the first act, from the very start (level 1, the onboarding), built only as narrative choices and catalog data: every engine and kit system it needs is built in M0-M1 first | New game to the end of the slice is playable and recorded; it is fun from the first fight |
| Playtest | The user plays the slice | Notes routed; the kit hooks the stand-ins showed are needed become tasks |
| M3 Depth | The systems that give the game its identity (progression, corruption, economy), built as kit hooks or game code | Rules tests and a sim report |
| M4 Balance and art | Balance runs; the art pass in the game's palette | Bands met; art from briefs, no redraws |

## Rules (user 2026-10-25)

- The slice needs no system design: anything the existing engine and kit do not cover is fleshed out as a reusable
  engine or kit system before the slice; only game-specific parts stay in the game.
- Reuse the engine with the earlier game as the baseline: a rule incompatible with the earlier rules is built as an
  option; a compatible one as an extension that can be disabled. Bug fixes go to the core and may need a revisit of
  the earlier game and the kit.
- The slice starts where a new player starts, so it also tests the onboarding.

## Measuring throughput

Log hours per milestone, split by layer ([layer_rule.md](layer_rule.md)): data, kit hooks, game code, engine. The share
of data shows how much of the game the engine and kit already carried.
