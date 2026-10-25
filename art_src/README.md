# Art source images

Full-size masters the game art is made from (ART2, 2026-10-20), stored in Git LFS (`.gitattributes`): portrait
sources from an image model, one per character as `<id>.png` (the first take) and `<id>_gen<time>_<n>.png` (later
takes), plus variant sheets such as `mara_voss_variant.png`.

- `tools/portraits/generate.py run <model> <id>...` makes new sources from the prompts in
  `games/emberwake/docs/specs/ART2_PROMPTS.md`, with `mara_voss.png` as the style reference.
- `--prompts <file>` reads another prompts file in that layout (a shared style block, then one block per id):
  `games/emberwake/docs/specs/ART3_PROMPTS.md` holds the Scarlet Lullaby portraits (ART3, 2026-10-25).
- Prompt records: beside every generated image, `<same name>.json` (plain Git, not LFS) keeps the model, the id, the
  full prompt sent (style block included), the reference image, the prompts file, the UTC time and the candidate
  index `n`; `generate.py` writes them since ART3. Records written afterwards carry `"backfilled": true` and a
  `source` (where the prompt text came from; the ART2 portraits from the Portrait Prompts page the user ran them
  from, the icon sheets from their prompts file at the commit that added them, `added_in`); a renamed pick names its
  original file in `generated_as`; Mara Voss's prompt is the one the user supplied (her own run, no reference), and
  `mara_voss_variant` has no known prompt (`null`). Never the key.
- `tools/portraits/pixelize.py` turns a source into the 240 px game portrait (96 px grid, 4 shades per colour family);
  `clear_mark` and `clear_corner` remove the image model's star mark first.
- ART3 (2026-10-25): the user's pick of each Scarlet Lullaby portrait is `<id>.png` (`render_beast`, `ash_crawler`,
  `inquisitorial_purifier`, `calcified_husk`, `captain_vane`, `proctor_quell`, `guild_master_orlov`,
  `inquisitor_prelate_nemeth`, `tess`, `bram`, `aline`); the other candidates stay as `<id>_gen<time>_<n>.png` takes
  (`render_beast` has three: the two first candidates and one re-roll).
- `ui_1.png`: the PLAYUI1 UI icon sheet (5x5 on a magenta key; prompt in
  `games/emberwake/docs/specs/PLAYUI1_ICONS.md`); `tools/portraits/sheets.py ui art_src/ui_1.png ui_1` cuts it into
  the 22 icons at their layer paths (`--contact` makes a review sheet).
- Keep every take: a portrait can be reused in another context or swapped later for a different feel.
- Reusable prompt shapes (2x2 variations, full body, animation sheet, props) are in
  `games/emberwake/docs/specs/ART_PROMPT_TEMPLATES.md`; `mara_voss_variant.json` holds Gemini's reconstruction of its prompt.
