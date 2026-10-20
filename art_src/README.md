# Art source images

Full-size masters the game art is made from (ART2, 2026-10-20), stored in Git LFS (`.gitattributes`): portrait
sources from an image model, one per character as `<id>.png` (the first take) and `<id>_gen<time>_<n>.png` (later
takes), plus variant sheets such as `mara_voss_variant.png`.

- `tools/portraits/generate.py run <model> <id>...` makes new sources from the prompts in
  `games/emberwake/docs/specs/ART2_PROMPTS.md`, with `mara_voss.png` as the style reference.
- `tools/portraits/pixelize.py` turns a source into the 240 px game portrait (96 px grid, 4 shades per colour family).
- Keep every take: a portrait can be reused in another context or swapped later for a different feel.
