# Lore modules

Status: canon (user 2026-10-25). Game-agnostic. How a game's lore and events are written as independent pieces that
can be reviewed and decided on their own, before anyone fleshes out the broader narrative. A game's modules live in
`games/<id>/docs/lore/modules/`; its story guide ([story_guide.md](story_guide.md)) links them once they connect.

## Why modules

Questions about lore are the last in the focus order ([consistency_pass.md](consistency_pass.md)): systems first, then
art that needs a new engine type, then lore. Writing each piece of lore as a self-contained module lets the user look
at one event, character beat, object or place without first settling the whole story. Later the narrative is fleshed
out by connecting unconnected modules to the game. The core story (the spine the game needs to be built at all) is a
separate track and never waits on modules.

## One module

One file per module, `LM-nn_<slug>.md`, at most about 80 lines:

| Field | Holds |
| --- | --- |
| Id and title | `LM-nn` (stable, never reused) and a short name |
| Kind | event, character beat, object, place, faction beat, rumour |
| Status | `seed` (an idea), `written` (complete on its own), `approved` (the user said yes), `connected` (placed in the narrative) |
| Pitch | One or two sentences a reader understands with no other doc |
| What it is | The module in full: who, what, where, what happens; written so it stands alone |
| What the player sees and does | Scene, dialogue, fight, item, choice or map marker, on systems that exist |
| Systems it uses | Existing engine and kit hooks only; a module that needs a new system raises a `1 system` question instead |
| Assets | Reused assets, or briefs for new ones (the naming and asset brief rules) |
| Hooks | Possible connections to the canon (characters, places, acts) as options, never as canon |
| Answers | The open questions or story gaps it would close, by id |

## Rules

- A module adds no canon until it is `approved`; connecting it to the narrative is a later, separate step.
- Modules do not depend on each other; when two must, merge them or connect them later.
- Names follow the game's naming rules and go into its registry only when the module is approved.
- An index (`modules/README.md`) lists every module with its kind, status and the ids it answers.
