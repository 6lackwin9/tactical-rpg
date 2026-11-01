# Art prompt templates

Reusable image-API prompt shapes in the house pixel-art style, and the shapes for the local models
([Local short prompt](#local-short-prompt), with Klein's whole prompt and edit wording). Gemini wrote them on 2026-10-25 when the user asked it
for the prompts behind Mara Voss's portrait and her 2x2 variation sheet (`art_src/mara_voss*.png`); they name Mara as the
worked example. To reuse one, swap the character lines (name, age, features, clothing, weapon, mood, colours) and attach
the character's existing portrait as the reference. The game draws battle bodies with the code rig
(`tools/sprites/rig.py`), so the sheet template gives reference art, not shipped sprites. Records of what each run made sit
beside the images in `art_src/` (`<image>.json`); the Art Prompt Library page holds the user's tweaks and test notes.

## 2x2 portrait variations

Four design choices for one character at once. Keep the framing paragraph; change the four bullets (armour, state, age, hood, expression).

```text
A 2x2 grid of four distinct pixel art character portrait variations for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. 1:1 square image. No borders, no frame, no text, no dividers, no logo. Plain flat very dark background (#120e12) behind all figures. Each variation shows a head-and-shoulders, three-quarter view of Mara Voss, a woman in her early thirties with a strong build, square jaw, and dark copper-red hair. Each variation utilizes a 16-bit pixel art style with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per color, and light dithering on metal and cloth. Strong key light from the upper left and warm orange rim light from the right.

The four variations are:
- Top-Left: A battle-worn version, showing a larger scar across her cheek, messy loose hair, and scuffed, dirt-spattered steel armor with deep red brigandine.
- Top-Right: A heavily-armored, disciplined variant with tight-bound hair, heavily reinforced dark steel pauldrons, and a rugged leather shoulder strap over her deep red brigandine.
- Bottom-Left: An older, battle-hardened version with a visible chainmail hood (coif) framing her face, worn iron armor, and deep red brigandine.
- Bottom-Right: A survival-ready "Ember Warden" variant with looser hair, a frayed dark fur/wool collar over her steel gorget, and the warm orange glow of embers in the background.
```

## Full body from three angles

Non-animated full-body sheets, concept art or large static sprites (a party screen).

```text
2D pixel art full-body character sprite sheet for a dark fantasy tactical RPG, in the style of 16-bit SNES/GBA-era games (e.g., Tactics Ogre, Fire Emblem). Single full-body character shown from multiple angles: a front view, three-quarter front view, and side profile view, standing side-by-side on a plain, flat very dark background (#120e12) with no grid, no floor, and no text. No UI, no border, no watermark. Hand-placed pixel clusters, dark outlines, and a limited palette with light dithering.

Character: Mara Voss, Captain of the Ember Wardens. A woman in her early thirties with a strong, athletic build and dark copper-red hair tied back. She is wearing a deep red riveted brigandine with brass rivets over chainmail, worn steel pauldrons, a steel gorget, and dark steel greaves and boots. She has a sheathed sword on her hip or back and wears a dark leather belt with pouches. Ground shadows are cast beneath her feet. The style is compact but highly readable, emphasizing the deep red, steel gray, and brass color scheme in a 16-bit aesthetic.
```

## Animation sprite sheet

Combat and animation frames (idle, run, attack, hurt). Change the character lines in row 1 and the actions per row; the rig's clips are idle 8, walk 8, attack 6, hurt 4, death 6.

```text
2D pixel art animation sprite sheet for a dark fantasy tactical RPG, 16-bit SNES/GBA style (approx. 48x48 or 64x64 pixels per frame size). Showcase of a single character, Mara Voss, in a side-by-side horizontal and vertical sequence on a flat, solid dark background (#120e12) with no grids, no borders, no text, and no UI. Hand-placed pixel clusters, high-contrast dark outlines, and a limited palette of deep red, steel gray, and brass.

The sheet contains multiple rows of frame-by-frame animation cycles, from left to right:
- Row 1: Idle stance. A woman with dark copper-red hair in deep red brigandine and steel armor, hand on sword hilt, subtly breathing and looking right (4 frames).
- Row 2: Combat ready & Running/moving. Slightly crouched, sword drawn, moving forward with swinging arms and legs (6 frames).
- Row 3: Melee sword attack. Winding up, slashing the sword forward with a brief trail effect, and returning to stance (6 frames).
- Row 4: Hurt & Knockdown. Character flinching back from an impact, then falling or kneeling, showing damage, and recovering (5 frames).

Character has a tiny but highly readable silhouette with exaggerated key movement frames to preserve readability at small sizes.
```

## Local short prompt

### Klein: the whole prompt (the main local model)

Since LOCALGEN2 (user 2026-10-26) the main local model is FLUX.2 [klein] 4B in ComfyUI, the default backend of
`localgen.py`. It needs no short form: its Qwen3 4B text encoder reads the whole prompt (Bram's recorded prompt is
about 870 tokens by the tool's estimate, which the record keeps as `tokens_estimate`), so nothing is cut. `run` sends
the character block first, then the shared style block without its "Use the attached image" line (a text-to-image
run attaches no image, and the rest of that line speaks of it). `localgen.py prompt <id>` prints it.

An edit (`vary`, `faces`) keeps the source as a reference latent, and the change is the whole prompt: say what stays,
then the one thing that changes. The expression cells (`faces`, the cell lines of `PORTRAITFX1_TASKS.md` 2.4.2) use:

```text
Keep the same character, face shape, hair, scars and markings, outfit, pixel-art style, framing, lighting and dark background exactly. Change only the facial expression to <expression>: <cell line>.
```

### Short prompt for the CLIP models (OpenVINO drafts)

For the OpenVINO draft model (LOCALGEN1, `--backend openvino`: LCM Dreamshaper v7, SD 1.5 family). Its CLIP text
encoder reads 77 tokens, 75 of them the prompt's own, and drops the rest without a word, so the long image-API blocks
above do not fit. `localgen.py prompt <id>` builds the short form from a prompts-file block and lists what it cut;
`run` and `vary` send that form and keep the cut list in the take's record (`cut`). When a detail that makes the
character is cut (Tess's burned ear is), write the prompt by hand and pass it as `--prompt`.

The short form is only for models whose text encoders are all CLIP (LCM, SD 1.5 and SDXL; the tool reads the family
from the model folder's `model_index.json`). FLUX gets the whole block, subject first, then the style block without
its line about the attached reference. Its T5 encoder reads 256 tokens, though, and one ART3 character block nearly
fills that (Bram's is 245 by FLUX's own tokenizer), so the style block mostly falls off; the record's `cut` says how
much. The rules for the short form, from the 2026-10-25 and 2026-10-26 runs:

- Subject first, style last. A style-first prompt made the small model draw a game scene instead of a portrait.
- Who comes first, with "adult". The model draws young faces, and the tone rule (no child fighters or victims)
  needs the word. Then two or three face features, the hair, one signature detail, the lead clothing item and the mood.
- No names, no hex codes and no negations. The model cannot know a name, and hex codes waste tokens. A CLIP prompt
  cannot say "no": "no hood" draws a hood. The ART3B hair rule becomes the negative prompt (`hood, cowl`) for models
  that take one; LCM takes none.
- Keep an adjective list with its noun ("large, bright, alert eyes"), and keep one look per clause.
- The style tail is about 15 tokens and keeps the ART3B "stylized" rule. A LoRA's trigger words go after it
  (`--trigger`), and `--style` replaces it.
- For a variant (`vary`), the change goes first and replaces the mood.

```text
head and shoulders portrait of <who: age, build, adult>, <two or three face features>, <hair>, <signature detail>, <lead clothing>, <mood> expression, stylized dark fantasy character art, facing right, amber rim light, plain dark background
```

Worked example, Tess by hand (71 tokens; the automatic form spends its words on her eyes and armour and loses the
ear):

```text
head and shoulders portrait of a wiry adult woman of nineteen, sharp lively freckled face, lopsided half-smile, wild dark-brown hair in one loose braid, burn-scarred notched right ear, patched leather armour, courier satchel strap, stylized dark fantasy character art, facing right, amber rim light, plain dark background
```

## Props and objects

Game objects in the same style (chests, campfires, barricades, furnaces); swap the bullets for the props needed.

```text
2D pixel art game assets and prop sprites for a dark fantasy tactical RPG, in the style of 16-bit SNES/GBA-era games. A collection of object sprites shown from an isometric/three-quarter top-down perspective, arranged in rows on a flat, very dark background (#120e12). Clear dark outlines, hand-placed pixel clusters, high-contrast lighting, and a limited, earthy color palette with warm orange highlights. No text, no grid, no borders, no UI.

Sprites include:
- An "Ember Warden" campfire with burning logs, embers, and glowing orange flame animation frames.
- A heavy iron-banded wooden loot chest, shown in both closed and opened states.
- A ruined stone barricade/obstacle with moss and rubble.
- A glowing forge/embers furnace with brass accents and sparks.
- Stacked crates, weapons rack with swords, and a medieval tent.

Compact scale, highly readable, matches the 16-bit dark fantasy aesthetic.
```

## Routes by asset type (ARTPROMPT1, 2026-11-01)

Since 2026-11-01 the "Gemini" route runs through the Antigravity queue by default (decision "Antigravity queue as the default
image route"; Imagen 3; no approval, warn on batches over about 10 images); the user's Gemini web runs are the fallback.
From the ARTPROMPT1 tests (decision "ARTPROMPT1 routes"; Gemini web results beside Klein 4B, ERNIE-Image Turbo and
Z-Image Turbo at game size; boards and prompt records in `C:/dev/localgen/review/artprompt1/`). Local runs need no
approval; a Gemini run follows the image rules (the image API is asked first).

| Asset type | Route | Prompt | Notes |
| --- | --- | --- | --- |
| Story scene, no recurring face (places, crowds) | ERNIE-Image Turbo locally (UNet fp8_e4m3fn) | [Story scene](#story-scene) | close to Gemini in style; follows layout less exactly (extra figures) |
| Story scene with a named character, face in shot | Gemini, the character's portrait attached | [Story scene](#story-scene) | ERNIE plus a Klein whole-image edit restores hair, scars and armour but not the face shape |
| Story scene, named characters small, far or turned away | ERNIE, then a Klein whole-image edit with the portrait | [Story scene](#story-scene) | the edit also fixes copied hair or armour on others (about 100 s) |
| Full body front, side and back (rig reference) | Gemini, the portrait attached | [Full body, three views](#full-body-three-views) | no local model kept identity and true views together |
| Single skill icon (read at 32 px) | Z-Image Turbo locally (2-4 seeds), else Gemini | [Single skill icon](#single-skill-icon) | one icon per run beats a sheet at 32 px; frame and tint added in post |
| 2x2 icon sheet (settling four ideas) | Gemini | [Icon sheet](#icon-sheet) | keep subjects whole, not close-ups; local models lost colour |

Gemini icon results are saved from the preview (right click, Save image as); the full-size download came back as a
blurred crop. The model's corner mark is cleared at pixelize (`pixelize.clear_mark`; icons can keep a speck).

### Full body, three views

```text
The same woman as the attached portrait, drawn full body three times side by side: front view, side view facing right, back view. Identical character in all three: same face, hair tied back, deep red riveted brigandine with brass rivets over mail, steel pauldrons and gorget, dark steel greaves, sword sheathed across her back with the hilt above the right shoulder. Same height, same scale, feet on one baseline, arms relaxed at her sides. 16-bit pixel art, clear dark outlines, limited palette with 4 shades per colour, light dithering on metal. Key light from the upper left, warm orange rim light from the right. Plain flat very dark background (#120e12). No floor, no shadow, no text, no frame.
```

### Story scene

```text
Pixel art story illustration for a dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image. Night in a ruined stone courtyard: Mara Voss (the woman in the attached image, same face, hair and armour) sits by a campfire with two Ember Warden soldiers in red brigandines, one sharpening a sword, one standing watch with a spear. Warm orange firelight on faces and armour, deep blue-black shadows, broken arches and rubble behind them, sparks rising. Clear dark outlines, hand-placed pixel clusters, limited palette, light dithering. No text, no UI, no border, no logo.
```

### Single skill icon

```text
A single square pixel art skill icon for a dark fantasy tactical RPG, drawn as a 32x32 pixel icon scaled up (visible large pixels, no anti-aliasing). One bold centred silhouette with a thick dark outline and 3-4 colours plus black: a sword striking from the side with a curved orange motion trail (the skill "Flank Cut"). Plain very dark background (#120e12), no text, no frame.
```

### Icon sheet

```text
A 2x2 grid of four square pixel art skill icons for a dark fantasy tactical RPG, 16-bit SNES/GBA style, each icon designed to read clearly at 32x32 pixels. Bold silhouettes, thick dark outlines, high contrast, limited palette, one strong idea per icon, plain very dark background (#120e12) in each cell, no text, no borders between cells beyond a thin dark gap. Top-left: Shoulder Check, an armoured shoulder slamming forward with impact lines. Top-right: Heavy Cleave, a broad axe blade mid-swing with a heavy arc. Bottom-left: Hamstring Cut, a dagger slashing low behind a leg. Bottom-right: Flank Cut, a sword striking from the side with a curved motion trail.
```

## Antigravity queue (ARTQUEUE1, 2026-11-01)

Decision "Antigravity image queue" (user 2026-11-01): a Gemini image run can go through the Antigravity image subagent
instead of the image API. This side writes request files and ingests the results with `tools/portraits/artqueue.py`
(never an API call, never a key); Antigravity makes the images. The contract extends the Antigravity-side spec
(`image_pipeline_spec.md`, updated by the user 2026-11-01 to match it).
Folder: `.image_pipeline/` in the main checkout `C:/dev/tactical-rpg` (the same absolute path from every worktree; Git
ignores it), with `queue/`, `processing/`, `output/`, `completed/` and `failed/`. The ingest adds `rejected/` (refused
images, each with a `.note.txt`) and `pixelized/` (game portraits of portrait requests without a target).
Request `queue/<name>.json`, written as `queue/<name>.json.tmp` and then renamed, so a scan never reads half a file.
`<name>` is unique: `<id>_q<time>` (`<id>_faces_q<time>` for a faces sheet, `<given name>_q<time>` for `text`), a
second later while the name is taken in any folder. A reference that does not exist refuses the request (nothing is
queued):
| Field | Meaning |
| --- | --- |
| `prompt` | required; the exact text to send, unchanged |
| `aspect_ratio` | `1:1` (default), `16:9`, `9:16`, `4:3` or `3:4` |
| `style_hints` | empty for our prompts (the style is in the prompt) |
| `references` | absolute image paths, attached in this order |
| `n` | how many images (variants), default 1 |
| `metadata` | `asset_id`, `task`, `target` (where the asset lane wants the result), `route` (one of `portrait`, `faces`, `full_body`, `story_scene`, `icon`, `icon_sheet`, `other`; see "Routes by asset type"), `record` (fields for the prompt record, such as `prompts_file`, `kind`, `faction`) |
Antigravity moves the request to `processing/`, makes `n` images and saves `output/<name>.png` (n = 1) or
`output/<name>_<k>.png` (k from 1), writes `output/<name>.result.json` (`model`, often the generic
`image-generator`; `time`; `prompt_sent`, the exact text it sent; `references`; `n`; `files` [{`file` (relative, such
as `output\<name>.png`), `width`, `height`}]; `notes`), checks each PNG opens, is full size and matches the aspect,
then moves the request to `completed/`. On a failure: `failed/<name>.json` and `failed/<name>.error.txt`.
Quota pause (ARTQUEUE2, 2026-11-01): on HTTP 429 or quota exhaustion the request goes back to `queue/` (not
`failed/`), Antigravity writes `paused.json` {`paused_until` (ISO UTC), `reason`, `request`} and stops the run; runs
skip until `paused_until`, then it deletes `paused.json`; only real errors go to `failed/`. `artqueue.py status` prints
`paused until <time> UTC (<n> min): <reason>` (or `pause expired`; a malformed file is reported), `request` and `text`
still queue during a pause and note it, and `ingest` is unaffected.
Archive (ARTQUEUE3): once `ingest` writes a request's marker it moves the request's leftover `output/` files (images,
`<name>.result.json`, `<name>.ingested.json`) into `archive/<UTC date>/output/` and its request from `completed/` into
`archive/<UTC date>/completed/` (never overwriting: a clash gets `_2`, `_3`); `rejected/` stays; a dry run moves nothing.
"Ingested already" also looks in `archive/*/output`. `artqueue.py archive [--dry-run]` sweeps earlier ingests that have a
marker in `output/`; `status` shows `archived N`. Antigravity never reads `archive/`.
Ingest (a request counts as finished when `output/<name>.result.json` exists and the request is in `completed/`):
each image is checked (present, a whole PNG, the aspect within 3 %, the long edge at least 1024 px); a refused one
moves to `rejected/` with a note and the valid variants of the same request are still taken. A request in `failed/`
is reported with its `error.txt` (and nothing is ingested).
An accepted one is copied into `art_src/` with the house naming of `generate.py` (`<id>.png` for a single first take,
else `<id>_gen<time>_<k>.png`; a faces sheet `<id>_faces_gen<time>_<k>.png`), its model mark is cleared
(`pixelize.clear_mark`) and its prompt record is written beside it: `model` (as the result names it), `"via":
"antigravity"`, `id`, `prompt` (the
exact `prompt_sent`; `prompt_requested` too when it differs), `reference` (the first) and `references`,
`prompts_file`, `time`, `n`, the request's `metadata` and `record` fields, `request`, `mark_cleared` and
`"source": "antigravity image subagent"`. Route `portrait` is pixelized into the 120 px game portrait (the request's
`target` when that file does not exist yet, else `pixelized/<art_src name>.png`); `faces` is cut by `sheets.py faces`
and the rest is left for the asset lane. `output/<name>.ingested.json` lists what went where; a second ingest skips it.
Expression sheets from this route are already pixel art, so `sheets.py faces` takes `--reduce scale` (decision
"Expression sheets: scale only and aligned cells", PORTRAITFX1-XA): each cell is only cut from its background
(`pixelize.cut_background`), scaled to the 120 grid (`smooth_reduce`) and framed on the faction backdrop, with no
palette pass (the default `--reduce pixelize` is unchanged). `--align` first scales (0.85 to 1.15, steps of 0.01) and
shifts each non-neutral cell so its figure best covers the neutral cell's, and prints the chosen scale and offset
(pixels of the cell) per cell. `--cell <name>=<image>` (repeatable) replaces one cell by a 1:1 edit of the neutral cell
(for example `--cell afraid=art_src/bram_gen1793543824_1.png`), resized to the cell before the alignment. Both
need `--reduce scale`; `--contact` works as before.
Commands (from `tools/portraits`, with `$LOCALAPPDATA/Programs/Python/Python312/python.exe`; test
`test_artqueue.py`):

- `artqueue.py request <id>... [--prompts <file> | --file icons] [--faces] [--no-ref]`, plus the shared options
  `[--ref <png>]... [--n N] [--aspect A] [--route R] [--task T] [--target <path>] [--record KEY=VALUE]... [--dry-run]`:
  one request per id, the prompt and the references as `generate.py run` builds them (the section's `reference:`
  image, else Mara Voss), then each `--ref`
- `artqueue.py text <name> (--text "..." | --text-file <file>) [--id ID] [same options]`: a template prompt from
  this file, filled in
- `artqueue.py ingest [<name>...] [--dry-run]`: every finished request, or the names given
- `artqueue.py status`: counts per folder and the failed requests' errors
- `--pipeline <dir>` (or `IMAGE_PIPELINE`) points any command at another queue folder (tests, a scratch run)
Each Antigravity run is an image run like any other: the image rules still apply (ask first where they say so).
