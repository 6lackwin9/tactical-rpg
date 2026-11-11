# Preferences

Tushar's choices that are not formal decisions (user 2026-10-18). Firm, specific rules stay in `docs/decisions/`; cross-cutting heuristics for scoring options and making `[PROVISIONAL]` calls when Tushar is away live in [`docs/PRINCIPLES.md`](../../../docs/PRINCIPLES.md); these files hold the taste and defaults that design, UI and asset work should follow. A UI or asset pick Tushar approves from a set of options counts as a preference, unless the choice came from him (then it is a decision).

- [design.md](design.md): game design (Design thread).
- [ui.md](ui.md): screens, controls, layout (UI/UX thread).
- [assets-content.md](assets-content.md): items, units, stats, text (Assets & Audio thread, content tasks).
- [assets-world.md](assets-world.md): sprites, maps, tiles, audio, art style (Assets & Audio thread, art and audio tasks).

The two asset files are separate work streams: a task reads only the file for its kind, so the `asset-builder` gets just the context it needs.

The owning thread reads its file before proposing or building and adds a line, with the date and the source, whenever Tushar states or approves a new preference. Other threads read them when their work touches that area.
