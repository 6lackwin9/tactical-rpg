# tools/tiles (LOOK1-T)

Tile pipeline for the LOOK1 sets. Python 3.12 with Pillow. Gemini art is already pixel art, so every step is scale only
(no palette pass). Run from this folder.

| Tool | Use |
| --- | --- |
| `cut.py SRC --size WxH --out DST` | Detects the pixel block size (largest b dividing 95 % of the colour runs, RGB step at most 12), samples block centres, trims a uniform border of at most one block; the result must be exactly W x H, else exit 1. |
| `seam.py score PNG` / `seam.py fix PNG [--out DST]` | Wrap step mean over inner step mean (at most 1.5 accepted); `fix` replaces the edge columns and rows by the inner neighbour nearer the tile's mean. |
| `masks.py [N]` | Ragged corner masks 1-14 (bit 1 NW, 2 NE, 4 SW, 8 SE is upper), prints mask N. Library: `mask_grid`. |
| `compose.py --upper U --lower L --size WxH --out DIR` | U and L are PNGs or folders (their `v0.png`); writes opaque `m01.png`-`m14.png` (use `tilesets/<set>/<upper>/over_<lower>`). |
| `board.py SET_DIR --out DIR` | Tile sheets (3 x 3, 1x and 2x) with checks, piece sheets, the sample map square and tilted (1x-3x), `index.html` with `file:///` links. |

Sizes (kit checks A13-A15): region tiles 32x32 or 32x24, faces 32x16 or 32x12, stage tiles 64x48, stage faces 64x24.
Checks on the board: seam at most 1.5, at most 16 colours, mean luminance at most 25 %.

Test: `python -m unittest test_tiles` (temp dirs and fixtures only).
