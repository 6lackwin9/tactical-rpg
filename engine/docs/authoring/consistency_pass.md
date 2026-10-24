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

## Rules

- The planner commits; writers never do. One branch for the whole pass.
- History is append-only: superseded decisions are marked, not deleted; raw exports go to `archive/`.
- Design or gameplay choices the brief does not settle go to the user, each with options and a recommendation.
