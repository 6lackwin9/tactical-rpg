# New game onboarding and the cold open

Status: canon (consistency pass 2026-10-25). This doc follows [CANON.md](../CANON.md); where they differ, CANON wins.
Names follow the [cast registry](../background/cast_registry.md). Related:
[cosmology and setting](../background/cosmology_and_setting.md), [factions and sociology](../background/factions_and_sociology.md),
[recruits and prologue](recruits_and_prologue_architecture.md), [art and visual style](../specs/art_and_visual_style.md).
Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Intent

"New Game" does not open an encyclopedia, an empty map or a stat screen. It plays a short illustrated vignette and then
hands the player one character in the mud and rain.

```text
New Game -> ~30 s vignette (bell, rain, tallow crackle; violet sky, cathedral, iron masks)
         -> cold open in the Alleyway of Cinders (Kaelen panting against the stone)
         -> "Rise": Captain Vane withdraws, two Castellan Footmen close in, the first fight (0a)
```

## 2. The opening vignette

- Player `[kit hook]` (vignette player: slides with fade and typewriter text) `[data]` (slides, text, audio cues).
- Look: high-contrast 2D pixel-art slides centred in the art bible's 640x360 native viewport, framed in Void Abyss
  (`#0a0b10`).
- Sound: silence broken by the deep tolling of a great iron bell; rain on wet slate; gutters dripping; wind in the
  stone arches.

| Slide | Image | Text |
| --- | --- | --- |
| 1 | A bruised-violet, sunless sky choked with petrified cloud and drifting white ash | "Two centuries ago the stars and the sun went out. In their place hangs the Shroud. No morning comes. Time is kept only by the cathedral bells." |
| 2 | A cross-section of the earth: stone cities on calcified titan bones and pulsing black veins | "Beneath the basalt sleeps the Low Sleep, an ancient dreaming leviathan whose nightmares shake the earth and bleed marrow into the wells." |
| 3 | The windowless iron masks of the Sovereign Clerisy over lines of youths in chains, led toward crypt arches | "The Clerisy says that if the titan wakes, the world breaks, and that the Red Tithing, sung in young blood each generation, is the lullaby that keeps it asleep. Those who live past five-and-twenty are spent, docile and blind." |
| 4 | A shattered city-watch insignia in the gutter mud, washed with fresh blood | "In the Seventh Ward, a garrison guard refuses to lock the cages." |

Slide 3 states only what the Clerisy preaches; the truth is the [central truth](../background/central_truth.md)
(Option A, canon). The folk know only the yearly rite (hundreds), so the vignette names no number; the Great Tithing of
thousands is revealed at 2.4 ([CANON](../CANON.md) 3.6). Tone rule: the slides show the rite and the chains, never
violence to a child.

## 3. The cold open: control of Kaelen (0a)

- Map: the Alleyway of Cinders, Seventh Ward, night, rain; a narrow 10x6-tile corridor on the STAGE1 option B battle
  board (tilted, zone backdrop) `[data]` (map, ENC-PRO-0A).
- Light: dark Wet Slate ground (`#1e232d`) lit by one sputtering warded tallow lantern in Candle Gold (`#fbbf24`).
- Start: Kaelen kneels in the mud, panting, gripping a notched billhook poleaxe; his left forearm is wrapped in bloody
  cloth with white spurs showing through the linen.
- Prompt: "Rise" (press or click).
- Confrontation: Kaelen stands. Captain Vane and two Castellan Footmen of the Watch in kettle helms and iron
  breastplates step in from the street with billhooks, blocking the exit.

Dialogue `[kit hook]` (dialogue) `[data]`:

- **Captain Vane:** "Order 44, deserter. The culling cages aren't full yet. You were sworn to drive the young stock into
  the pens, not break your corporal's jaw."
- **Kaelen:** "They're fifteen years old, Vane. You're herding children into the crypts."
- **Captain Vane:** "They're tithes for the bells, boy. Draw your steel, or take their place on the wagons."

Vane withdraws and leaves the two Footmen. The camera pulls back, the timeline strip (6 blocks per round) appears at the
top of the screen `[engine]` (`aegis_ui::timeline`), and the first battle begins.

## 4. How the factions are introduced

The player meets each faction through what it does on screen, not through lore menus.

### 4.1 The Sovereign Clerisy and the Low Sleep

- The bells of St. Lyra's toll in the background every few minutes with a heavy screen shake `[engine]` (camera shake)
  `[data]` (cue timing).
- Midway through the fight a deep rumble shakes the alley: a tremor from the sleeping titan. Water ripples in the
  puddles; Kaelen grunts as the spurs in his arm pulse with a faint light (visual only, no mechanic) `[kit hook]`
  (scripted battle event) `[data]`.

### 4.2 The Iron Inquisition and the Castellan Watch

- They are the enemy in the fight. The Footmen hold Active Guard and strike with slow windups; the fight teaches Kaelen
  to answer a windup with Pommel Strike and break the Footman's poise (Stagger) `[data]` (enemy AI and skills).

### 4.3 The Tallow Guild and the Spent

- Between 0a and 0b Kaelen runs through the Lower Meat Market `[kit hook]` (vignette or scripted walk) `[data]`.
- He passes an open-air rendering station where cauldrons boil carcasses and unclaimed corpses into warded tallow.
- Hollow-eyed, wheezing adults (the Spent, 25 and older) stand in silent queues with wooden bowls, trading curfew badges
  for communion draughts and cups of grey salt. When an inquisitor shoves a young girl, none of them react; they stare
  into their cups, dulled by decades of draughts and tallow fumes.

### 4.4 The Severed Root (0b)

- Inquisition whistles shriek; an ambush squad corners Kaelen on the Meat Market rooftops `[data]` (ENC-PRO-0B).
- A shadow drops from a roof overhang, a soot flask shatters into a 3x3 smoke zone, and Vesper appears at Kaelen's back:
  "Move your feet, tin-can. Grates are open. Follow the smoke."
- The player now controls both and learns smoke zones, roots and disengaging, then heads for the canal bridge.

## 5. To the first hub

```text
0a The Alley (Kaelen alone)
 -> Meat Market walk (the Spent)
 -> 0b The Rooftop (Vesper joins)
 -> 0c The Bridge (Maren and Hollis freed from the mortuary cart)
 -> the Drowned Runoff Grate (node 01)
 -> the Sluice Undercroft (hub opens)
```

1. 0c The Bridge: Kaelen and Vesper storm the bridge and free Maren and Hollis from the iron cart
   ([prologue and starter squad](prologue_and_starter_squad_lore.md) 3).
2. The descent: sirens sound and searchlights sweep the water; Vesper levers open the drainage grate.
3. Arrival: the four wash up on the flagstones of the Sluice Undercroft, where Bram waits by the coals of the Tinker's
   Hearth and Sister Aline sharpens scalpels at the Apothecary's Slab. After the hearth scene the hub opens
   `[kit hook]` (hub facilities; today's `hub.rs` `Hub` is only the in-game menu) with the four founders in four of the
   six bunks; the Doom Clock appears at 42 days; the board offers Act 1 M1, The Sweep.

## 6. Gaps (Open)

- The size of the Red Tithing: decided (a yearly rite of hundreds; the Great Tithing of thousands is a 2.4 reveal,
  Q-04); slide 3 still avoids a number.
- Whether the engine has a camera shake for the bell tolls (needed by 4.1).
