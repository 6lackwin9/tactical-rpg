# The consistency pass

Status: canon (user 2026-10-25). How a brainstormed design library becomes canon. Game-agnostic.

## Steps

1. **Review** (read-only, parallel by area: narrative and world, combat and rules, loop and economy and screens,
   architecture and roadmap). Each reviewer compares the docs with the real code and lists: what exists, what extends,
   what is new (with size), internal contradictions (file and section), wrong engine claims, gaps, and questions with a
   recommendation. Rank by time to playable: data, small extension, new code.
2. **User decisions**: the planner condenses the reviews into a short list of decisions with recommendations; the user
   answers.
3. **Canon brief** (planner): the process rules for the library, every user decision, naming rules, the central-truth
   task and the rule "collect gaps, do not invent". Plus a contradictions checklist from the reviews.
4. **Canon sheet first** (one writer): `CANON.md`, the names registry, the central-truth proposal and a mechanical
   rename list (`old -> new`, with context-sensitive lines flagged).
5. **Domain rewrites** (parallel writers, disjoint files): each follows the canon sheet, applies the renames, resolves
   its contradictions, tags layers, fixes lint and size, and ends every doc with its gaps. Writers report fixes the canon
   sheet needs instead of editing it.
6. **Final check** (one writer): cross-file consistency (renames everywhere, links resolve, numbers match the canon
   sheet), applies the canon-sheet fixes that need no decision, and writes one deduplicated open-questions doc.
7. **Gap fill**: the user answers the open questions; the world and systems are locked; implementation starts.

## Focus order for the open questions (user 2026-10-25)

Tag every open question with a focus and work them in this order:

1. `1.n <system>`: design and system questions, split into one group per system so they can be taken one system at
   a time (for a tactics RPG: combat, campaign meters and fail states, roster and lifecycle, hub and economy, slice
   scope and process, later-act systems, enemy and item content), ordered by what the slice needs first.
2. `2 art type`: art and asset questions that cannot reuse existing assets and need their own type in the engine;
   `2 art` for the rest (process, reuse).
3. `3 lore`: lore and events, each written as an independent lore module ([lore_modules.md](lore_modules.md)) that
   can be read and decided on its own; the narrative is fleshed out later by connecting modules.

Separate track, `core story`: the core narrative the game needs to be built at all (the spine, the slice's story
beats, the world logic the systems read). It is handled on its own, not mixed into the lore modules.

## Rules

- The planner commits; writers never do. One branch for the whole pass.
- History is append-only: superseded decisions are marked, not deleted; raw exports go to `archive/`.
- Design or gameplay choices the brief does not settle go to the user, each with options and a recommendation.
