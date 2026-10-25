# Art bible (template)

Status: canon (user 2026-10-25). What every game's art bible fixes before any art is made. Game-agnostic; the game's
filled-in version lives in its design library (`specs/art_and_visual_style.md`).

| Section | Fix |
| --- | --- |
| Canvas | The logical canvas (the shell's 640x360) and integer scales |
| Battle board | Cell size and camera (flat or tilted), unit draw size, what reads at that size |
| Region map | Tile size and camera for exploration maps |
| Sprites | Frame size of the rig, the figure bounds inside it, which clips exist, front/side/back views |
| Palette | The master hues (named, with hex values), how many shade steps each, skin ramps, and the signal colours reserved for mechanics |
| UI | Panel, border and text colours from the palette; text sizes (every screen fits at all of them) |
| Portraits | Size, framing, style reference, the pipeline that makes them; the anchor sidecar (eyes, glow, veins) and expression frames, since the effects follow the engine's portrait layer |
| Props and tiles | Footprints, which stand upright and sort with units, which are flat decals |
| Effects | Hit, death, projectile and status visuals; hit-stop and shake limits |
| Pipelines | For each asset kind: the sprite rig, the portrait pipeline, code-drawn art, or an image-API sheet (asked each time); what the rig cannot draw yet |
| Accessibility | Colour-blind safety of the signal colours, contrast against the ground |

## Rules

- One palette per game; asset art is quantised to it at export.
- Fix board and sprite sizes before any art batch; changing them later redraws everything.
- Every named asset has a brief ([naming_and_asset_briefs.md](naming_and_asset_briefs.md)) before it is made.
