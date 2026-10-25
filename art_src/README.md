# Art source images

Full-size masters the game art is made from (ART2, 2026-10-20), stored in Git LFS (`.gitattributes`): portrait
sources from an image model, one per character as `<id>.png` (the first take) and `<id>_gen<time>_<n>.png` (later
takes), plus variant sheets such as `mara_voss_variant.png`.

- `tools/portraits/generate.py run <model> <id>...` makes new sources from the prompts in
  `games/emberwake/docs/specs/ART2_PROMPTS.md`, with `mara_voss.png` as the style reference.
- `--prompts <file>` reads another prompts file in that layout (a shared style block, then one block per id):
  `games/emberwake/docs/specs/ART3_PROMPTS.md` holds the Scarlet Lullaby portraits (ART3, 2026-10-25).
- Local drafts (LOCALGEN1, user 2026-10-25: drafts and iteration run on the PC's own model, the image API is the
  fallback): `tools/portraits/localgen.py` runs OpenVINO GenAI on the Intel NPU, the Arc GPU or the CPU. It runs from
  the venv outside the repo (`C:\dev\localgen\venv\Scripts\python.exe tools/portraits/localgen.py ...`; the model,
  the LoRA files and the compile cache stay in `C:\dev\localgen`, or `LOCALGEN_HOME`). `run <id>...` writes
  `<id>_loc<time>_<n>.png` takes from the id's prompts-file block, shortened for the 77-token encoder
  (`ART_PROMPT_TEMPLATES.md`, "Local short prompt").
  `vary <id> --from art_src/<id>.png --change "..." --strength 0.3` makes variants of an existing image. `--model`
  picks another model folder; its family (LCM, SD 1.5, SDXL or FLUX, from `model_index.json`) sets the default steps,
  guidance and size, the negative prompt and the prompt form (short for the CLIP-only families, whole for FLUX). `--sheet` (or `sheet OUT PNG...`) puts each take beside its `pixelize`
  portrait, and `--dry-run` prints the prompt and record without generating. Benchmark and throwaway takes go to
  `--out <scratch folder>`, not here.
- Prompt records: beside every generated image, `<same name>.json` (plain Git, not LFS) keeps the model, the id, the
  full prompt sent (style block included), the reference image, the prompts file, the UTC time and the candidate
  index `n`; `generate.py` writes them since ART3. Records written afterwards carry `"backfilled": true` and a
  `source` (where the prompt text came from; the ART2 portraits from the Portrait Prompts page the user ran them
  from, the icon sheets from their prompts file at the commit that added them, `added_in`); a renamed pick names its
  original file in `generated_as`; Mara Voss's prompt is the one the user supplied (her own run, no reference), and
  `mara_voss_variant` has no known prompt (`null`). Never the key. A local take's record has the same keys first, with
  `model` `local:<model folder>`, then `family`, `device`, `seed`, `steps`, `size`, `guidance`, `negative`, `lora`, `cut` (the
  block parts left out of the short prompt) and `seconds`; a variant's also has `source_image`, `strength` and
  `change`.
- `tools/portraits/pixelize.py` turns a source into the 240 px game portrait (96 px grid, 4 shades per colour family);
  `clear_mark` and `clear_corner` remove the image model's star mark first.
- ART3 (2026-10-25): the user's pick of each Scarlet Lullaby portrait is `<id>.png` (`render_beast`, `ash_crawler`,
  `inquisitorial_purifier`, `calcified_husk`, `captain_vane`, `proctor_quell`, `guild_master_orlov`,
  `inquisitor_prelate_nemeth`, `tess`, `bram`, `aline`); the other candidates stay as `<id>_gen<time>_<n>.png` takes
  (`render_beast` has three: the two first candidates and one re-roll).
- ART3B (2026-10-25): a stylized re-roll of the nine human portraits (the `ART3_PROMPTS.md` style block with its
  stylization and hair lines), 2 takes each. The user's picks are now `<id>.png` with `generated_as` naming the
  original take: take 1 of `captain_vane`, `proctor_quell`, `guild_master_orlov`, `inquisitor_prelate_nemeth`,
  `calcified_husk`, `tess` and `bram`, take 2 of `inquisitorial_purifier` and `aline`. Each ART3 pick went back to its
  original `_gen` name as a take; `render_beast` and `ash_crawler` keep their ART3 picks.
- ART3C (2026-10-25): Tess's right-ear re-roll, 2 takes (`tess_gen1792945776_1`, `tess_gen1792945812_2`). The user
  picked take 2; it is now `tess.png` with `generated_as`, and the ART3B pick went back to `tess_gen1792942586_1.png`
  as a take.
- `ui_1.png`: the PLAYUI1 UI icon sheet (5x5 on a magenta key; prompt in
  `games/emberwake/docs/specs/PLAYUI1_ICONS.md`); `tools/portraits/sheets.py ui art_src/ui_1.png ui_1` cuts it into
  the 22 icons at their layer paths (`--contact` makes a review sheet).
- Keep every take: a portrait can be reused in another context or swapped later for a different feel.
- Reusable prompt shapes (2x2 variations, full body, animation sheet, props) are in
  `games/emberwake/docs/specs/ART_PROMPT_TEMPLATES.md`; `mara_voss_variant.json` holds Gemini's reconstruction of its prompt.
