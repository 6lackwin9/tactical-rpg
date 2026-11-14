# Act 1 slab-removal edits (Flow, 2026-11-14)

The Act 1 cleanup's first step (`region_maps_prep.md` section 5): every Act 1 landmark loses its rock block in a Flow
edit, as Acts 2 and 3 did (`README.md`, "Blending an act's landmarks"). A landmark standing on its own block reads as
a token set on the map rather than a place in it.

**Sources**: the Drive picks in `image_pipeline/flow/out/World Map/` (checked 2026-11-14 on the mounted Drive,
`G:\My Drive\...`). The eight Act 1 places below still stand on a thick block. The hub is already the slab-free
edit on Drive (`00_Sluice_Undercroft_98ec71d6.jpg`, cut with `--desmoke 0.69,0.30,0.80,0.52`), and so are the Act 2
and Act 3 files, so none of them needs a new edit.

**Returns**: save each edit as `image_pipeline/flow/out/World Map/slab_free/<file>.jpg`, keeping the source's name.
Art/UI then reviews each one against its source: the design, size and colours are unchanged, the block and its
dark sides are gone, and the magenta keys clean. Each edit is then cut with the fix from its section below.

Each prompt is one of the two wordings already used for Acts 2 and 3, plus one sentence naming what stays. The
**ground** wording keeps the place's top surface (yard, water, road) as a thin layer. The **building** wording keeps
only the structure on a thin edge at its foundations. Attach only the source file.

## 01 Drowned Runoff Grate

Source `01_Runoff_Grate_2e64a3c1.jpg`. Ground wording. No cut fix.

```text
Edit this image: remove only the thick dark side faces of the rock block under this place. Keep its top surface (the big grate, the black water around it, the square ring posts and the steps, and everything on it) exactly as it is, as a thin flat layer of ground, and keep the arched stone alcove and its lantern. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## 02 Alleyway Junction 7

Source `02_Alleyway_Junction_7_ff08682d.jpg`. Ground wording: the yard with the barricade and torch is the place. No
cut fix.

```text
Edit this image: remove only the thick dark side faces of the rock block under this place, including its jagged broken bottom. Keep its top surface (the cobbled yard with the spiked barricade, the torch and the barrels, and everything on it) exactly as it is, as a thin flat layer of ground, and keep every tenement house around it. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## 03 Ash-Crawl Crossroads

Source `03_Ashcrawl_Crossroads_beaa5fff.jpg`. Ground wording. No cut fix (the violet lamp keyed clean before; check
it again).

```text
Edit this image: remove only the thick dark side faces of the rock and soil block under this place. Keep its top surface (the ashy cobbled crossroads with the cage platform, the chained toll-pen gate, the violet lamp-post and its violet pool of light, and everything on it) exactly as it is, as a thin flat layer of ground, and keep the shacks around it. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## 04 Fourth Kiln

Source `04_Fourth_Kiln_a2620ecd.jpg`. Ground wording, because the bone-meal heap and the chute stand on the apron in
front of the furnace mouth and the building wording could drop them. No cut fix.

```text
Edit this image: remove only the thick dark side faces of the rock block under this place. Keep its top surface (the cobbled apron with the bone-meal heap, the conveyor chute, the brass tank and the pipes, and everything on it) exactly as it is, as a thin flat layer of ground, and keep the kiln with its three glowing chimney tops and its furnace mouth. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## 05 Tallow Vats

Source `05_Tallow_Vats_451b11a0.jpg`. Ground wording: the barrel yard is part of the place. Cut fix as before:
`--tint 0.05-0.16:0.19@0.33,0,1,0.62` (the orange vat tops to the Guild's sickly yellow). Check that the tint box
still covers the vat tops on the edit.

```text
Edit this image: remove only the thick dark side faces of the rock block under this place. Keep its top surface (the yard with the barrels, crates and spilt tallow, and everything on it) exactly as it is, as a thin flat layer of ground, and keep the three vats, their catwalks, the pipes and the shed. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## 06 Crucible Watchpost

Source `06_Crucible_Watchpost_44f6ff1b.jpg`. Building wording: the tall rock crag under the walls is the block here.
**One judgement call**: Act 3 kept the broken cliff under the Chasm Ingress and the Weeping Basalt Chasm, because there
it reads as a ledge over a drop. The Crucible stands at the gorge mouth on the painted ground, not over a drop, so its
crag would only hang down into the painting. If the edit looks wrong without the crag, the fallback is to keep the
crag and cut the landmark smaller. No cut fix.

```text
Edit this image: remove the tall crag of rock that the watchpost stands on, and its dark side faces. Keep the watchpost and everything built on it exactly as it is (the walls with their row of red-orange slit windows, the battlements, the tower with its violet window, the winch drum and chain, the crane arm and the banners), standing on only a thin, flat edge of its own ground at its foundations. Keep the solid pure magenta background everywhere else. Do not change the building, its size, its colours or its details.
```

## 07 Slag Gate

Source `07_Slag_Gate_6a2b7327.jpg`. Ground wording: the gate is set into the cliff wall, so the cliff behind it stays
and only the block under the road goes. No cut fix.

```text
Edit this image: remove only the thick dark side faces of the rock block under this place. Keep its top surface (the short paved road and the glowing L-shaped slag channel, and everything on it) exactly as it is, as a thin flat layer of ground, and keep the fortified cliff wall with the shut portcullis, its torches and its red banners. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## 1d Corvo's Gate

Source `01d_Corvo_Gate_b4f51bc4.jpg`. Ground wording. No cut fix.

```text
Edit this image: remove only the thick dark side faces of the rock block under this place. Keep its top surface (the yard with the striped barrier, the toll drum and the rubble, and everything on it) exactly as it is, as a thin flat layer of ground, and keep the crumbling brick wall with its breach, the guard booth and the lantern. Keep the solid pure magenta background everywhere else. Do not change the place, its size, its colours or its details.
```

## After the edits

1. Review each edit (above), then cut it for review with `--out`, never straight into the repository:
   `cut_ai_sheets.py landmark <edit> <location_id> [fix] --out <review dir>`.
2. Place and size Act 1 in the Act Map Layout page; Art/UI writes `art_src/prompts/act1_map_layout.json`.
3. `blend_act_map.py codeblend art_src/prompts/act1_map_layout.json`, then rebuild the Act 1 preview.
4. `AO-01` is queued now as READY `PR5-47` (`MAP-HOOKS1`). If the final Act 1 positions differ from the painted-region
   `map_pos` numbers in that row, send the new ones to Design in a new Outbox row.
