# Authoring a game on Aegis, top down

Status: canon (user 2026-10-25). Game-agnostic: this guide names no game. A game's own design library lives in
`games/<id>/docs/lore/` and fills in the templates below; the kit's genre pieces live in `kits/<kit>/`.

Top down means theme and setting first, then narrative and world, then the systems and combat derived from them. The
engine and kit already carry most systems, so a new game is mainly data plus a few kit hooks. These pages keep agents
from re-deriving the method each time.

| Page | Use it to |
| --- | --- |
| [layer_rule.md](layer_rule.md) | Decide whether a feature is engine, kit hook, data or game code |
| [kit_layout.md](kit_layout.md) | Lay out a kit's folders and crates, and extract a kit from a game |
| [design_library.md](design_library.md) | Set up a game's design library (canon sheet, registry, numbers, glossary, truth, gaps) |
| [naming_and_asset_briefs.md](naming_and_asset_briefs.md) | Name things and write the asset briefs art is generated from |
| [story_guide.md](story_guide.md) | Write a game's story guide: plot, who's who with arcs per act, story elements, reveal timeline, gaps |
| [lore_modules.md](lore_modules.md) | Write lore and events as independent modules that can be decided alone and connected later |
| [campaign_structure.md](campaign_structure.md) | Give the player forks and optional missions while checkpoints with deadlines keep the story on time |
| [consistency_pass.md](consistency_pass.md) | Turn a brainstormed design library into canon and collect the gaps |
| [combat_hooks.md](combat_hooks.md) | Use the RPG kit's combat hooks as data: forced moves, hazards, triggers, meters, phase effects, AI tactics |
| [art_bible.md](art_bible.md) | Fix canvas, board, sprite, palette and pipelines before any art is made |
| [milestones.md](milestones.md) | Plan the build (foundation, feel prototype, slice on data, playtest, depth, balance and art) and measure throughput |

## The order of work

1. **Theme and setting**: one page: tone, the central conflict, what the player is and fights against.
2. **Narrative and world**: acts, hub, cast, places, the central truth (a spoiler sheet), endings, fail states.
3. **Systems from the theme**: combat feel, progression, economy, each tagged with its layer (see the layer rule).
4. **Consistency pass**: one canon sheet resolves every contradiction; brainstorm docs become `canon`, `proposal` or
   `archive`.
5. **Gap list**: every undefined thing a player or a system would hit, with 1-2 options and a recommendation; the user
   fills the gaps before implementation.
6. **Vertical slice on data**: the smallest playable path through the first act on today's engine and kit, then the
   kit hooks the playtest shows are missing.

## Rules for agents

- Brainstorm text is a proposal until the consistency pass marks it `canon`. Verify every claim about the engine
  against the code; brainstorm docs often name modules that do not exist.
- Settled decisions go to the game's decisions file and the project index (CLAUDE.md); history stays append-only.
- Ask the user before any design or gameplay decision the canon sheet does not settle.
- Load the skill `game-design-library` before writing or reviewing a game's design library.
- Promote methods: when a spec, guide or process made for one game could serve others, write its game-agnostic
  version here (no game names) and link it from the game's version (user 2026-10-25).
