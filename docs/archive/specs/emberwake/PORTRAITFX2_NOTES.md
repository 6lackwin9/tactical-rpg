# PORTRAITFX2 Answered Design Questions (Q1-Q10)

Archived 2026-11-15 (CTX-CLEANUP1-DESIGN), resolved design questions moved verbatim from [PORTRAITFX2.md](../../../../games/emberwake/docs/specs/PORTRAITFX2.md).

## 13. Open questions (recommended option first)

- Q1. Where the profiles live: (a) on the per-face rows `portrait_expressions`; (b) on each `face` cue row (per
  trigger, but every row repeats the profile). Recommend (a).
- Q2. Formula: (a) parts composed by one function (4.1); (b) a `kind` enum with one fixed formula each. Recommend (a):
  a new face needs no engine change.
- Q3. Heave depth against the 1-3 px rule: (a) stretch up to 4 px at 240 (angry only), offsets at most 3; (b) 3 at most
  (today's breath is already 2.88, so the heave reads only from speed and lean). Recommend (a).
- Q4. The 64 px talk box at 1x is the reduced tier: (a) as PORTRAITFX1 (no cycle, entries halved); (b) cycles at half
  amplitude there too. Recommend (a).
- Q5. Face list home: (a) the ids of `portrait_expressions`, with the kit's four when there are no rows; (b) a `faces`
  list in `game.json`; (c) its own kind. Recommend (a): one place per face.
- Q6. Happy on level-up: (a) held on the roster surfaces while points are unspent; (b) a timed event on the first draw
  after a level gain (a not-saved memory); (c) none. Recommend (a).
- Q7. `round_start` in the timeline ruleset: (a) none; (b) at a unit's own turn. Recommend (a).
- Q8. Colour effects for the new faces: (a) none; (b) Happy a faint Candle Gold tint, Sad a cool tint. Recommend (a);
  the prompt pushes Determined away from Neutral.
- Q9. Sheet 2 reference: (a) the image the founder's first sheet used, cells aligned to its neutral frame; (b) the
  240 px game portrait. Recommend (a): the two sheets match.
- Q10. New faces for speakers without frames: (a) allowed (motion over the single portrait); (b) heroes only.
  Recommend (a).
