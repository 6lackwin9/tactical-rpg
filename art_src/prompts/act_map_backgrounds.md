# Act maps: one background per act (runnable prompt file)

Art/UI batch 2026-11-13 (Tushar: "a background for each of the acts map"). One bird's-eye district painting per act,
the ground the act's landmarks ([act_map_landmarks.md](act_map_landmarks.md)) and routes sit on. Each is laid out
the way its act's graph runs: entry at the bottom centre, two branches up the left and right, joining again near the
top, the way on at the top centre. It is a backdrop, so it stays dark and quiet, and no landmark is drawn into it.

Run through the image queue, which sets the 16:9 aspect (`generate.py` has no aspect option):
`python3 tools/portraits/artqueue.py request wm_sunk_ward wm_high_bastion wm_mantle_fissure --prompts
art_src/prompts/act_map_backgrounds.md --no-ref --n 2 --aspect 16:9 --route other`. Scale the pick to **1920 x 1080**
(the size of the existing `backgrounds/world_map/ashlands.png`).
Targets:

- `wm_sunk_ward` -> `games/emberwake/assets/backgrounds/world_map/sunk_ward.png`. The game's world-map lookup already reads
  `backgrounds/world_map/<world>.png` (`rpgkit_content::art::world_map_background`), so it shows in game straight away;
  today Act 1 falls back to the Ashlands map.
- `wm_high_bastion` -> `backgrounds/world_map/high_bastion.png` and `wm_mantle_fissure` ->
  `backgrounds/world_map/mantle_fissure.png`. All three acts are regions of the one world `sunk_ward`, so the game needs
  a per-region lookup before these show: Outbox `AO-01`.

Check every take: no text or map symbols, nothing that reads as a landmark, the diamond layout clear, dark enough that a
48 pixel landmark and a dashed route read on top of it.

```text
Pixel art district map background for a gothic dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG world maps. 16:9 wide image. A bird's-eye three-quarter view from high above of one whole district at night, painted as one continuous terrain with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour and light dithering. It is the backdrop behind map markers, so it stays quiet: dark, muted and low in contrast, with no single building, tower or landmark standing out, and wide plain stretches of ground, street, rock or water where markers will sit. Layout: the lowest point at the bottom centre, the way out at the top centre, two roads or paths branching up the left and right halves from the bottom and joining again just below the top, with the middle of the picture fairly open between them. A sunless volcanic city under a permanent ash-shroud, lit only by small lanterns, furnaces and glows: soot-black stone, wet slate, scavenged iron, fog and falling ash. Colours only from this palette and its shades: void black #0a0b10, damp basalt #14171f, wet slate #1e232d, rotting oak #2c221e, scavenged iron #4b5563, polished steel #9ca3af, bleached linen #d9cbb4, cinder ash #6b7280, boiled leather #92400e, tallow amber #d97706, candle gold #fbbf24, arterial red #e11d48, rust crimson #991b1b, blight violet #9333ea, resonant orchid #c084fc, salt cyan #06b6d4. No text, no letters, no labels, no map icons, no compass rose, no grid, no frame, no border, no UI, no people.
```

## The Sunk Ward, Act 1 (`wm_sunk_ward`)

```text
District: the Sunk Ward, the lowest, drowned quarter of the volcanic city of Cinderreach. Crowded crooked tenement rooftops and narrow alleys, black canals and sluice channels winding through them in fog, a scatter of tiny tallow-amber window lights; the left half leans industrial (low brick works, chimney stubs, faint furnace-orange glow in the gutters), the right half leans to vat yards and pipes with a faint sickly yellow-green haze; bottom centre a flooded drain basin of black water below a stone embankment; steep cliffs of slag and basalt close the top of the picture with one narrow gorge gap at the top centre, slag runoff glowing faint orange down its floor. Mood: drowning grime, squalor, survival.
```

## The High Bastion, Act 2 (`wm_high_bastion`)

```text
District: the High Bastion of Upper Cinderreach, the Clerisy's rich cathedral ridge. Soot-blackened gothic rooftops with candle-gold trim and flying buttresses, polished black basalt boulevards, cold pale-white cones of magnesium arc-lamp light sweeping through the fog, small gilded domes; bottom centre a broken gap in the ridge's lower wall where the way up from the slums comes in; the dark mass of a great cathedral fills the top of the picture, its front split by a narrow gap at the top centre between two spires, with a faint blue-violet glow rising behind it. Mood: oppressive wealth, cold white glare and candle gold over black.
```

## The Mantle Fissure, Act 3 (`wm_mantle_fissure`)

```text
District: the Mantle Fissure beneath the city, a vast volcanic chasm seen from above. Walls and ledges of petrified black basalt shaped like the giant ribs and vertebrae of a sleeping titan, thin blue-violet marrow veins glowing along the cracks, brass conduit pipes and rope bridges strung across the gaps, a few steaming vents with a faint amber glow; bottom centre a ledge of rock where a cave mouth opens from above; the chasm narrows and deepens towards the top centre into a black pit with a slow pulsing violet glow at its heart. Mood: cosmic dread, petrified anatomy, a deafening silence.
```
