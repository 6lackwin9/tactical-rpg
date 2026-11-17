# Transitions

Battle entry and exit art for TRANSITIONS1 (Art/UI request AI-18), from `art_src/transitions/make_transitions.py`
(deterministic). Final names come from Build row `PR7-14`; replacements land in place.

| File | Use | Status |
| --- | --- | --- |
| `ash_wipe_mask.png` | 1280 x 720 greyscale threshold map: at wipe progress t (0..1) cover every pixel below t * 255; soot closes in from the edges to the centre with a ragged front | `[PROVISIONAL]` |
| `ash_flecks.png` | 1280 x 720 ember and ash specks, drawn only along the wipe's front (mask within a few steps of t) | `[PROVISIONAL]` |
| `objective_banner.png` | 960 x 132 fight banner frame (name, objective and enemy count drawn by the UI) | `[PROVISIONAL]` |
| `victory_banner.png` | 720 x 168 Victory banner frame (the word drawn by the UI) | `[PROVISIONAL]` |

Banners use the locked F2 frame (STYLE_BIBLE 2): basalt panel, 2 px iron line `#4b5563`, 2 px oxblood top `#991b1b`,
ends fading into the stage.
