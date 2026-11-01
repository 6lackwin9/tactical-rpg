# The layer rule

Status: canon (user 2026-10-25). Every feature of a game sits in exactly one layer. Ask the three questions in order.

| Question | Layer | Examples |
| --- | --- | --- |
| Is it a technology any game needs? | `[engine]` (`engine/crates/aegis_*`) | Rendering, input, the screen stack, saves, asset layers, the timeline widget, audio, the agent link |
| Is it a shape any story in this genre could fill? | `[kit hook]` (`kits/<kit>/`) | A campaign meter with thresholds and triggers; unit lifecycle (downed, wounded, captured, dead, retired); hub facilities with levels; a resource purse; exchange recipes (crafting, surgery, trade); ruleset options; event triggers; a mission board; a vignette player |
| Is it named by the story? | `[data]` or `[game code]` (`games/<id>/`) | Data fills kit hooks (a doom clock is a meter, a workshop is a facility with recipes). Game code only for a mechanic nothing else could use, with a one-line reason |

## How to use it

- Tag every feature line in a design doc with its layer: `[engine]`, `[kit hook]`, `[data]` or `[game code]`.
- A missing kit hook becomes one scoped kit task, built once and generically; the next game gets it free.
- Test a hook against a second game: if the kit's sample game could use it too, it is a kit hook; if only one story can,
  it is game code.
- Prefer a data stand-in first (for example crafting as dialogue choices that take and give items) and build the kit
  hook when a playtest shows the stand-in hurts.
- Measure throughput per layer: hours spent on data, kit hooks and game code per milestone.
- The layer check (`tools/fc.sh layers`, in every gate) enforces it: engine and kit code name no game (G1) and have no
  path into `games/` (G2); kit code (`src/`, `schema.json`, `kit.json`) names no lore word of a game (G3: its world ids and
  `games/<id>/lore_words.txt`). A hit is fixed in the code; there is no allow marker.
- Kit code names a game-system word (the currency, the upkeep, the intents) only by a neutral term id with a plain
  default, read through the kit's one resolver; the game names the term in its data (a world block or its `terms`
  table), never in kit code (TERMS1; the RPG kit's register is `rpgkit_data::terms`).
- Where a kit's code and schema live, and how to extract a kit from a game: [kit_layout.md](kit_layout.md).
