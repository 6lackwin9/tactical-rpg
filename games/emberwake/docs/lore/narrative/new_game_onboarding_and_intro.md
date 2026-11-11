# New game onboarding and the cold open

Status: **`[LOCKED]` (`2026-11-11`, synced with [CANON.md](../CANON.md) & [STORY_GUIDE.md](../STORY_GUIDE.md); `CL-08`, `CL-11`)**. Related:
[Sunk Ward map](../background/sunk_ward_campaign_map.md), [CAST1 encounters](../../specs/CAST1_ENCOUNTERS.md),
[FOUNDER_PORTRAITS](../../specs/FOUNDER_PORTRAITS.md).
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

Slide 3 states only what the Clerisy preaches; the truth (`CANON` 2.1, `CL-10`) is uncovered across Act 2 (`2.4`, `2.6`).
The folk know only the yearly rite (hundreds), so the vignette names no number; the Great Tithing of thousands is revealed
at `2.4`. Tone rule: the slides show the rite and the chains, never violence to a child.

## 3. The cold open: control of Kaelen (`0a`, `CL-08`)

- Map: the Alleyway of Cinders, Seventh Ward, night, rain; a narrow `12x9` corridor (`enc_pro_0a`).
- Light: dark Wet Slate ground (`#1e232d`) lit by one sputtering warded tallow lantern in Candle Gold (`#fbbf24`).
- Start (`CANON` 3.1, 3.3, `CL-08` — 100% clean Day-1 mortal human): `Kaelen` kneels in the mud, panting, gripping his
  notched Watch billhook; bloodied linen wraps a fresh cudgel-graze over the **faded Lullaby House number tattooed on his
  inner left wrist** (no forearm bone spurs in Act 1).
- Prompt: `"Rise"` (press or click).
- Confrontation: `Kaelen` stands. `Captain Vane` (`38`, candle-gold bribe-shield on his arm) and two `Castellan Footmen`
  in kettle helms and iron breastplates step in from the street with billhooks, blocking the exit.

Dialogue `[kit hook]` (`dialogue`) `[data]`:

- **Captain Vane:** "Order 44, deserter. The culling cages aren't full yet. You were sworn to drive the young stock into
  the pens, not knock my iron aside."
- **Kaelen:** "They're fifteen years old, Vane. You're selling Seventh Ward blood for a Bastion commission."
- **Captain Vane:** "Better them in the wagons than me turning Spent in the mud at forty, boy. Draw your steel."

`Vane` withdraws and leaves the two `footman_patrol` guards. The camera pulls back, the timeline strip (`6` blocks per round)
appears at the top of the screen (`aegis_ui::timeline`), and the first battle begins.

## 4. How the factions are introduced

The player meets each faction through what it does on screen, not through lore menus.

### 4.1 The Sovereign Clerisy and the Low Sleep (`0a` Round 2)

- Midway through `0a` (`round: 2`), St. Lyra's curfew bell tolls with a camera shake (`aegis_ui` screen shake): puddles
  ripple in the gutter and Watch whistles shrill down the lane (`"The alley shudders as the distant curfew bell tolls. Watch whistles shrill down the lane."`, `CL-08`).

### 4.2 The Iron Inquisition and the Castellan Watch

- The `Footmen` hold `Active Guard` and strike with slow windups; the fight teaches `Kaelen` to answer a windup with
  Snap `Pommel Strike` and break poise (`Stagger`).

### 4.3 The Tallow Guild and the Spent

- Between `0a` and `0b`, `Kaelen` runs through the Lower Meat Market past boiling rendering cauldrons where hollow-eyed,
  grey-veined **Spent** (`25+`) stand in silent queues with wooden bowls, trading curfew tallies for communion draughts
  and grey salt—numb to everything around them.

### 4.4 The Severed Root (`0b` & `0c` `talk` Cues, `CL-11`)

- Inquisition whistles shriek; three `Mortuary Flagellants` corner `Kaelen` on the Meat Market rooftops (`enc_pro_0b`).
- A soot flask shatters into smoke and **`Vesper`** drops to `Kaelen`'s back (`"Move your feet, tin-can. Grates are open. Follow the smoke."`), calling out chimney cover mid-fight (`chimney_call` `talk` cue on Round 2).
- On the canal bridge (`enc_pro_0c`), `Kaelen` and `Vesper` smash an iron mortuary cart to free **`Maren`** and **`Hollis`**, who call out targets mid-fight (`bridge_call` `talk` cue on Round 2).

## 5. To the Sluice Hearth (`Hub Debate 1`)

```text
0a The Alley (Kaelen alone)
 -> Meat Market walk (the Spent)
 -> 0b The Rooftop (Vesper joins; smoke talk cue)
 -> 0c The Bridge (Maren and Hollis freed; cart talk cue)
 -> 01 Drowned Runoff Grate
 -> 00 Sluice Undercroft (Hub Debate 1 -> Tier 1 board opens M1 & M2)
```

1. Sirens sound and searchlights sweep the canal; `Vesper` levers open `01 Drowned Runoff Grate`.
2. Arrival (**Hub Debate 1 — *What Keeps Us Alive***, `CL-11`): The four Founders wash up in `00 Sluice Undercroft`, where **`Berek "Cinder-Lung" Dorn`** tends the `Tinker's Hearth`, **`Chirurgeon Sabina`** cleans silver scalpels at the `Apothecary's Slab`, and **`Talia Rusk`** tracks patrol tallies at the `Water-Lock Basin`. Over the coals they clash over how `18-24` fugitives can survive what rules Cinderreach; `4/6` bunks fill, the `42d` soft Doom Clock appears, and `Talia`'s cipher board opens **Tier 1** (`M1` *The Sweep* and `M2` *The Crossroads Cages* in either order, `CL-08`).
