# Motion profiles for portrait faces

Status: canon (from PORTRAITFX2-M, 2026-11-01). Game-agnostic: this page names no game. The engine piece is
`aegis_ui::portrait_fx::Motion`; a kit gives each face's row its fields and maps them to `Motion`.

A face (an expression the portrait shows) moves in its own way while it holds. A profile is a mix of parts that one
function combines (`Motion::eval`), so a new face is a new data row, never engine code.

## The parts

Distances are px at a 240 px portrait (`MOTION_REF`), scaled to whole device px (`motion_px`: at least 1 px for any
amplitude that is not 0). Times are ms.

| Part | Fields | Use it for |
| --- | --- | --- |
| Cycle | `breath_amp`, `breath_ms`, `breath_in`, `breath_jitter` | The breath: depth, length, the share breathing in (low: a quick in, a long out), unevenness |
| Lean | `lean_px` | A sideways lean with each breath (positive toward the picture's right) |
| Bob | `bob_px`, `bob_ms` | A light bounce |
| Entry | `enter_dy`, `enter_stretch`, `enter_ms` | A move when the face starts: a hop or jolt (dy), a wince (negative stretch) |
| Rest | `rest_dy`, `rest_ms` | A held offset: a lift (negative) or a sink (positive) |
| Settle | `settle_ms` | The cycle, lean and bob wait for the entry, then grow back over this long |
| Shiver | `shiver_px`, `shiver_ms`, `shiver_gap_ms` | Short bursts of jitter with gaps |
| Blink | `blink` | Off for a face whose eyes are shut or wide |
| Flush | `tint_breath` | The face's tint pulses with the cycle |

The default profile is the plain breath (2.88 px, 3600 ms, an even cosine), so a face with no motion fields, and the
plain face, breathe as before.

## Rules that keep it readable

- Keep offsets small: the stretch is clamped to -3 to 4 px and each offset axis to 3 px at 240. A kit should warn when
  the offset parts add up past 3 px.
- Pick one signature part per face and keep the rest quiet: a wince is an entry squeeze plus shallow breaths, a heave
  is a deep fast cycle plus a lean, fear is a quick cycle plus a shiver, sadness a slow sink with a long out-breath.
- Only the inner picture moves; the frame never does. Anchored layers (blink lids, eye glow, glow, veins, the face
  tint and marks) move with it; falling particles (ash, smoke) do not.
- Small portraits move less: below the full tier (96 device px) there is no cycle, lean or bob and the entry, rest and
  shiver are halved; the smallest tier and reduce motion draw no motion at all, and the flush holds steady.
- A face change blends the old motion into the new over 200 ms (`MOTION_BLEND_MS`), so nothing jumps.
- Everything is seeded from the portrait key and the face name, so the same portrait moves the same way every run.

## Trying a profile

The Kit widgets Portraits page offers eight sample profiles (`MOTION_SAMPLES`: breath, wince, heave, shiver, lift,
bounce, sink, jolt) on the code-drawn sample face, and its foot line names the live parts. The agent link's portrait
state reports `motion` (`stretch`, `offset`, `parts`).
