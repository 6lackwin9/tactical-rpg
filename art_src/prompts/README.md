# Art/UI prompt batch (2026-11-12)

Every image the Art/UI inbox (`games/emberwake/docs/specs/ART_UI_REQUESTS.md`) still needs, as ready-to-run prompts.
Each file is laid out the way `tools/portraits/generate.py` reads prompt files: a style block, then one
`## Name (`id`)` section per asset. Run any of them with `--dry-run` first, which needs no key.
Every result is checked by eye before it is kept (`ART_PROMPT_TEMPLATES.md`, "Mandatory post-generation visual
review"). Placeholders are replaced **in place**: the same path, size and format.

| Inbox | Asset | File | Route and command | Target |
| --- | --- | --- | --- | --- |
| AI-04 | 6 founder portraits | [ai04_founders.md](ai04_founders.md) | `generate.py run <model> kaelen vesper maren hollis dunstan elyse --n 2 --prompts art_src/prompts/ai04_founders.md`, then `pixelize.py` | `portraits/heroes/<id>.png` |
| AI-04 | 6 founder expression sheets | [ai_faces.md](ai_faces.md) | after the pick: `generate.py run <model> <id> --faces --prompts art_src/prompts/ai_faces.md`; `sheets.py faces <sheet> <id> hero --faction hearth` | `portraits/heroes/faces/<id>/*.png` |
| AI-05 | Talia Rusk, Widow Varka, Ivo | [ai_portraits.md](ai_portraits.md) | `generate.py run <model> talia_rusk widow_varka ivo --n 2 --prompts art_src/prompts/ai_portraits.md` | `portraits/npcs/<id>.png` |
| AI-05 | Chirurgeon Sabina | `ART3_PROMPTS.md` | `generate.py run <model> sabina_kress --n 2 --prompts games/emberwake/docs/specs/ART3_PROMPTS.md` | `portraits/npcs/sabina_kress.png` |
| AI-05 | Talia and Sabina sheets | `PORTRAITFX1_FACES.md` | `generate.py run <model> talia_rusk sabina_kress --faces` | `portraits/npcs/faces/<id>/*.png` |
| AI-06 | 7 boss portraits | [ai_portraits.md](ai_portraits.md) | `generate.py run <model> overseer_grendel foundry_master_jessop inquisitor_general_ruthven chief_anatomist_lazar warden_inquisitor_ulric cantor_ysolde arch_prelate_othemar --n 2 --prompts art_src/prompts/ai_portraits.md` | `portraits/bosses/<id>.png` |
| AI-06 | 7 boss expression sheets | [ai_faces.md](ai_faces.md) | after the picks: `--faces --prompts art_src/prompts/ai_faces.md`; `sheets.py faces <sheet> <id> boss --faction <f>` | `portraits/bosses/faces/<id>/*.png` |
| AI-07 | `recipe` service badge | [ai_icons.md](ai_icons.md) `services_hud_1` | `generate.py run <model> services_hud_1 --file art_src/prompts/ai_icons.md --no-ref`; `cut_ai_sheets.py services_hud_1 <sheet>` | `kits/rpg/assets/ui/services/recipe.png` (32) |
| AI-07 | 6 keepsake icons | [ai_icons.md](ai_icons.md) `keepsakes_1` | `generate.py run <model> keepsakes_1 --file art_src/prompts/ai_icons.md --no-ref`; `cut_ai_sheets.py keepsakes_1 <sheet>` | `items/accessory/keepsake_<id>.png` (96) |
| AI-08 | Calcified Amalgam, Bailiff Enforcer | [ai_portraits.md](ai_portraits.md) | `generate.py run <model> calcified_amalgam bailiff_enforcer --n 2 --prompts art_src/prompts/ai_portraits.md` | `portraits/bosses/calcified_amalgam.png`, `portraits/elites/bailiff_enforcer.png` |
| AI-10 | M5 Slag Gate CG | [ai10_cg.md](ai10_cg.md) | Flow scene prompt (no `@` character: the stayer must stay anonymous); Gemini fallback with Nemeth attached | `backgrounds/story/vg_m5_gate.png` (1120 x 472) |
| AI-11 | 4 HUD chips | [ai_icons.md](ai_icons.md) `services_hud_1` | the same sheet as the recipe badge | `ui/hud/{scrap,tallow,alert,doom_clock}.png` (14) |

| Act maps (2026-11-13) | 24 location landmarks, the hub shared by all acts | [act_map_landmarks.md](act_map_landmarks.md) | the hub first with `--no-ref`, then the rest with the picked hub as reference; `cut_ai_sheets.py landmark <pick> <location_id>` | `map/landmarks/<location>.png` (96; drawn once Outbox `AO-01` lands) |
| Act maps (2026-11-13) | Tallow Vats colour fix | the pick (take A) | `cut_ai_sheets.py landmark <pick> tallow_vats --tint 0.05-0.16:0.19@0.33,0,1,0.62` (orange vat tops to the Guild's sickly yellow-green) | `map/landmarks/tallow_vats.png` |
| Act maps (2026-11-13) | 3 act map backgrounds | [act_map_backgrounds.md](act_map_backgrounds.md) | `artqueue.py request wm_sunk_ward wm_high_bastion wm_mantle_fissure --prompts art_src/prompts/act_map_backgrounds.md --no-ref --n 2 --aspect 16:9 --route other` | `backgrounds/world_map/{sunk_ward,high_bastion,mantle_fissure}.png` (1920 x 1080; Act 1 shows now, Acts 2-3 after `AO-01`) |

Paths without a layer prefix are under `games/emberwake/assets/`.

## Act 1 map picks (reviewed 2026-11-13, full-size files to follow)

Each pick was reviewed from the chat copy, cut and checked on its painted place in background take 1. When the
full-size files arrive (Drive), save each as `art_src/<id>.png` and run its cut; the background is scaled to 1920 x 1080.

| Location | Pick | Cut |
| --- | --- | --- |
| background | take 1 (open-centre diamond, gorge at the top) | `backgrounds/world_map/sunk_ward.png`, 1920 x 1080 |
| 00 Sluice Undercroft | take 2 (rounded patch, water reflection) | `cut_ai_sheets.py landmark art_src/lm_sluice_undercroft.png sluice_undercroft` |
| 01 Drowned Runoff Grate | revised take 2 (square ring posts, grate over water) | `... landmark art_src/lm_drowned_runoff_grate.png drowned_runoff_grate` |
| 02 Alleyway Junction 7 | take B (dark slab base, barricade and torch) | `... landmark art_src/lm_alleyway_junction_7.png alleyway_junction_7` |
| 03 Ash-Crawl Crossroads | take A (cages, violet lamp) | `... landmark art_src/lm_ash_crawl_crossroads.png ash_crawl_crossroads` |
| 04 Fourth Kiln | take B (no smoke, dark base) | `... landmark art_src/lm_fourth_kiln.png fourth_kiln` |
| 05 Tallow Vats | take A (three vats) with the colour fix | `... landmark art_src/lm_tallow_vats.png tallow_vats --tint 0.05-0.16:0.19@0.33,0,1,0.62` |
| 06 Crucible Watchpost | take B (red window band) | `... landmark art_src/lm_crucible_watchpost.png crucible_watchpost` |
| 07 Slag Gate | take B (portcullis in a cliff wall, slag channel) | `... landmark art_src/lm_slag_gate.png slag_gate` |
| 1d Corvo's Gate | take A (warm crumbling brick, breach) | `... landmark art_src/lm_corvos_gate.png corvos_gate` |

## Act 2 map picks (reviewed 2026-11-13, full-size files to follow)

Checked the same way on background take A, at the positions proposed in Outbox `AO-01`.

| Location | Pick | Cut |
| --- | --- | --- |
| background | take A (dark stone road, cold-blue lamps, cathedral cropped to the spire gap) | `backgrounds/world_map/high_bastion.png`, 1920 x 1080 |
| 08 Slag Gate Breach | take A (gatehouse tower, broken portcullis) | `... landmark art_src/lm_slag_gate_breach.png slag_gate_breach` |
| 09 Gilded Boulevard | take A (white magnesium lamp, red Watch lanterns) | `... landmark art_src/lm_gilded_boulevard.png gilded_boulevard` |
| 10 Tallow Reservoir | take B (grey riveted silos, red pipe) with the colour fix | `... landmark art_src/lm_tallow_reservoir.png tallow_reservoir --tint 0.0-0.16:0.19` |
| 12 Culling Stockades | take B (gatehouse, wagon, rust-topped pens), slab removed in Flow, the torch smoke cleared | `... landmark art_src/lm_culling_stockades.png culling_stockades --clear 0.69,0.15,0.77,0.25 --size 115` |
| 11 Embalming Cloisters | take A (pale stone, lit courtyard, glowing dome) with the chimney wisp cleared | `... landmark art_src/lm_embalming_cloisters.png embalming_cloisters --clear 0.635,0.17,0.69,0.24` |
| 13 Grand Belltower | take A (lit bronze bell, buttresses, red banners) | `... landmark art_src/lm_grand_belltower.png grand_belltower` |
| 14 Reliquary Crypts | take B (candle-lit stair down to the gilded sarcophagus door, skull niches) | `... landmark art_src/lm_reliquary_crypts.png reliquary_crypts` |
| 15 Grand Spire Gate | take A (iron gate between two spires, round lock-seal, violet-lit cliff lip) | `... landmark art_src/lm_grand_spire_gate.png grand_spire_gate` |

Act 2 layout (Tushar, 2026-11-13): each landmark sits on open ground in its own quarter and is drawn larger than 48 px, so each is cut at 96 x its map scale with `--size`: belltower x1.8 `--size 173`, spire gate x1.5 `--size 144`, stockades, reservoir and cloisters x1.2 `--size 115`, breach, boulevard and crypts x1.15 `--size 110`; the hub keeps 96. Positions and scales: Outbox `AO-01`.

## Act 3 map picks (in progress, 2026-11-13)

| Location | Pick | Cut |
| --- | --- | --- |
| background | take A, edited twice in Flow (the pit and its stepped plateau moved lower; a landing for the hideout beside the cave ledge) | `backgrounds/world_map/mantle_fissure.png`, 1920 x 1080 |
| 16 Chasm Ingress | take B (rib-vault cave mouth with a spine on top, brass lift cage on a winch gantry, two lit lanterns) | `... landmark art_src/lm_chasm_ingress.png chasm_ingress --size 110` |
| 17 Weeping Basalt Chasm | take A (gauge-house with a caved-in roof, cyan seep down the basalt columns, rope bridge) | `... landmark art_src/lm_weeping_basalt_chasm.png weeping_basalt_chasm --size 115` |
| 18 Marrow Conduits | take A (two brass conduits looping down a basalt wall, valve wheels, pump house, violet windows) with the steam filled in and toned down | `... landmark art_src/lm_marrow_conduits.png marrow_conduits --desmoke 0.62,0.35,0.79,0.52 --grade 0.72:0.85 --size 115` |

## Blending an act's landmarks into its background (integration pass, 2026-11-13)

Once an act's landmarks are all picked and placed, [blend_act_map.py](blend_act_map.py) builds the image to blend and
cuts the result back apart; the layout (each landmark's source, map position, map scale and cut fixes) is a JSON file,
[act2_map_layout.json](act2_map_layout.json) for Act 2.

1. `python3 art_src/prompts/blend_act_map.py compose art_src/prompts/act2_map_layout.json <composite.png>`: every
   landmark on the background at its map spot and size, at the background's own resolution, with no labels, routes or
   badges.
2. Blend it in Flow with the prompt below (the landmarks stay where they are; only how they sit in the scene changes).
3. `python3 art_src/prompts/blend_act_map.py cut art_src/prompts/act2_map_layout.json <composite.png> <blended.png>`:
   each landmark cut out of the blended image with its composite silhouette (lined up within 4 px), to
   `map/landmarks/<id>.png` at 96 x its scale, and the blended image to `backgrounds/world_map/<target>.png`. A landmark
   whose mismatch is flagged was moved or redrawn: blend that quarter again on its own (crop it, blend, paste it back).

Before blending, every picked landmark loses its rock block in a Flow edit (adopted 2026-11-13 after testing the
belltower and the reservoir): the block makes a landmark read as a token placed on the map. Two wordings:

```text
Edit this image: remove the thick block of rock and ground that the building stands on, and its dark side faces. Keep the building and everything built on it exactly as it is, standing on only a thin, flat edge of its own ground at its foundations. Keep the solid pure magenta background everywhere else. Do not change the building, its size, its colours or its details.
```

```text
Edit this image: remove only the thick dark side faces of the rock block under this place. Keep its top surface (the yard, terrace or water and everything on it) exactly as it is, as a thin flat layer of ground. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

The game keeps drawing each landmark as its own sprite exactly over its painted copy, so hover, the selection glow,
the grey locked look and the quest badge still work.

```text
Edit this image. It is a pixel-art district map with small landmark buildings standing on it. Blend the landmarks into the scene so each one looks painted together with the place it stands on: give each a soft contact shadow, let the ground, rock, cobbles, fences and walls around its base meet it naturally, let the nearby light (lamps, windows, furnace and marrow glows) fall on the ground around it, and match its colours and contrast to the painting around it. Keep every landmark exactly where it is, at exactly the same size and outline, with its details; do not move, redraw, add or remove anything. Keep the roads, lamps, open ground and everything else as they are. Same pixel-art style and resolution, no text.
```

## Running through the image queue

The image queue (`tools/portraits/artqueue.py request`, Drive `image_pipeline/queue/`) takes the same prompt text when
the Gemini web app runs instead of the API; attach the reference each section names (`art_src/mara_voss.png` for
portraits, the picked portrait for sheets).

## Not prompts in this batch

- **Story and VN stage sprites (user 2026-11-12)**: no full-body or keyframe-animation generation for now. A story
  actor without signed-off VN art plays its **code-rig animation** (the battle body from `tools/sprites`, scaled up) as
  the placeholder: the mock UI's story stage does this (`ART_UI_STUDIO.md` 3.10). So Sabina's full body
  (`portraits/full_body/sabina_kress.png`, AI-05) waits; the old Aline art stays at that path until then.
- **AI-01** (the `warrior/2h_polearm` look): battle bodies are drawn by the code rig (`tools/sprites/rig.py`,
  `export_units.py`), not by an image model, so it is rig work for a later session, not a prompt.
- **AI-03** (story music): Flow Music Compose prompts, a separate batch (`art_src/music/prompts/`).

## Cutting the 3 x 3 icon sheets

`tools/portraits/sheets.py slice` only knows its own 4 x 4 ART2 sheets, so [cut_ai_sheets.py](cut_ai_sheets.py) cuts
these two with the same helpers (`cells_of`, `key_out`, `snap`, `pixelize.mode_reduce`). It writes a `<sheet>_cells.png`
contact sheet beside the input; `--pick name=cell` takes a spare cell (7-9) instead of the first take, and `--out DIR`
writes a review copy instead of the repository files.
