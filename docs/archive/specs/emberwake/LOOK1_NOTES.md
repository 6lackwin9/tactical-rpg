# LOOK1 Slices, Decided/Rejected & Resolved Questions

Archived 2026-11-15 (CTX-CLEANUP1-DESIGN), resolved slices and Q1-Q11/open-question notes moved verbatim from [LOOK1.md](../../../../games/emberwake/docs/specs/LOOK1.md).

## 9. Slices (briefs in `LOOK1_TASKS.md`)

| Id | Title | Layer | Lane | Depends on |
| --- | --- | --- | --- | --- |
| E1 | `aegis_ui::theme`: roles, layered `ui/theme.json`, faces, nine-slice frames, layout values; Theme page | engine | logic | S |
| F | Display font pick board and licence record | game (art) | art | - |
| G1 | The Tallow and Iron theme file, `ui_frames` art, the picked font | game | logic | E1, F |
| G2 | `emberwake_gui` reads the theme (constants and chrome literals) | game | logic | G1; a flush point after CLN-52 |
| ME | Engine panes `Backdrop`, `MenuColumn`, `TabRail`, `SlotCard`, themed ask | engine | logic | E1 |
| M1 | Title / main menu | game | logic | ME, G1 |
| M2 | New game, load / save, settings rail, game menu, `ornate` one-time cards | game | logic | M1, G2 |
| E2 | `aegis_ui::autotile` and the Gallery Tiles pane | engine | logic | S |
| K | `tileset`, `stage_set`, lookups, asset checks | kit | logic | S |
| T | `tools/tiles/`: cut, seam check and fix, transition composer, boards | game (tools) | logic | S |
| L1 | Battle stage: the world's stage set, transitions, wall faces | game | logic | E2, K |
| L2 | Region map: the bound sets, transitions, wall caps and faces in the picked view | game | logic | L1, EXPLOREVIEW1-R |
| A1-A7 | Art batches of section 7 | game (art) | art | T; A1-A3 after EXPLOREVIEW1-R; A6 after F |
| R | Place the picks; `tileset` and `stage_set` rows through `aegis content` | game (data) | asset | K, T, the picks; SUNKWARD1 for location rows |

First slice (Q2): E1, F, G1, ME, M1, E2, K, T, L1, A4-A6, R (stage set and title only). Then G2 at its flush point,
M2, EXPLOREVIEW1's review, A1-A3, L2, R (region sets), A7. E1, E2, K and T share no files and run in parallel.

## 10. Decided and rejected

Decided: decision "LOOK1 answers" (user 2026-11-02), Q1-Q11 all as recommended; sections 2-9 are written to it.
Rejected (not adopted; the planner adds the HISTORY rows): Cathedral Reliquary everywhere (Q1 b: clutter on the
640 x 360 battle HUD and small popups at Extra large text); retint only (Q1 c: smooth vector panels around hard pixel
art); menus only or tiles only first (Q2 b, c); the Ashlands first (Q3 b); region art before the view test (Q4 b);
retuning the engine constants (Q5 b: every game would inherit Emberwake's colours); a pixel font everywhere (Q6 b:
only whole multiples look right, breaking the four text sizes) or OpenSans alone (Q6 c); 128 x 96 stage tiles (Q7 b:
finer than the units on them); Gemini transition sheets (Q8 b: seams and drift) or none (Q8 c); a `ground` char layer
for more region kinds (Q9 b); Salt Cyan for good (Q10 b); a stage set per zone (Q11 b, about 35 more images).

## 11. Open questions (recommended first)

1. Badge numbers (the bible's "bold monospace"): (a) add two OFL mono candidates (JetBrains Mono Bold, IBM Plex Mono
   Bold) to the font board, shipped only if picked; (b) badges stay in the body face. Recommend (a).
2. The title screen subtitle (`game.json` `subtitle`, today "A small campaign of tactical battles."): (a) "A Scarlet
   Lullaby" (decision "Title"); (b) keep it. Recommend (a).
