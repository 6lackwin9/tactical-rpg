# A game's story guide

Status: canon (user 2026-10-25, asked for with the promote rule). Game-agnostic: this page names no game. The template for the one doc a reader opens to
follow a game's story and cast without reading every domain doc. It lives at `games/<id>/docs/lore/STORY_GUIDE.md`
and belongs to the game's design library ([design_library.md](design_library.md)).

## Rules

- The guide summarises; the canon sheet wins. Its Status line reads "canon summary (date); CANON.md wins on conflict".
- It adds no story. Where the docs are silent it lists a gap; it never fills one.
- Content from `proposal` docs (later acts, later systems) is marked as proposal wherever it appears.
- One row per named character in the cast registry, with a fate per act; group rows by faction or role.
- Every open item links its open-question id; a gap with no id is marked "new" and gets a short option pair and a
  recommendation, collected for the user (the planner adds it to the open-questions list).
- User decisions the guide relies on are listed under the Status line with their question ids and dates.
- Spoilers are expected: the guide uses the central truth once the user has picked it.
- Size: at most about 300 lines / 25 KB. If the who's who would push it over, move it to `background/who_is_who.md`
  with the same Status line and link it.
- Update it when the cast registry, the canon sheet or a decision changes; history stays append-only.

## Sections

| # | Section | Holds |
| --- | --- | --- |
| 1 | The story in one page | Premise, the central truth, act by act, endings and the failure screen |
| 2 | Believed and true | One row per topic the story hides |
| 3 | The acts | Prologue and each act in a short paragraph, with an endings table |
| 4 | Who's who | One row per named character, or a link to `background/who_is_who.md` |
| 5 | Story elements tracker | The recurring things: rites, meters, relics, secrets, places |
| 6 | Reveal timeline | What the player learns at each point, in play order |
| 7 | Gaps | Every undefined arc, motive or fate |

## Columns

| Table | Columns |
| --- | --- |
| Believed and true | Topic; what the player is told or sees first; what is true; where it is learned |
| Endings | Ending; what happens under the central truth; voice; gate |
| Who's who | Name; faction and role; wants; first seen (act and mission); one column per act; fate or open (id) |
| Story elements | Element; what it is; who knows what, when; set up; paid off |
| Reveal timeline | When (mission or node); what the player learns |
| Gaps | Id; gap; open-question id or "new"; options (new only); recommendation |

## Spotlights

When the user asks about one character, add a short traced subsection under the who's who: who they are, every
scene they appear in by act, what is decided, and what is open (with ids).
