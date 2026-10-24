# New Game Onboarding & The Cold Open

> **Category:** Player Onboarding, Narrative UX & Cinematic Opening  
> **Location:** `games/emberwake/lore/narrative/new_game_onboarding_and_intro.md`  
> **Reference Lore:** [cosmology_and_setting.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/cosmology_and_setting.md), [factions_and_sociology.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/factions_and_sociology.md), [recruits_and_prologue_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/recruits_and_prologue_architecture.md)

---

## 1. Design Vision: Zero Friction, Maximum Atmosphere

When a player clicks **"New Game"**, *Emberwake* does not present an encyclopedia screen, an empty overworld map, or complex stat-allocation menus.

The player is dropped straight into an immersive, atmospheric cold open that seamlessly transitions from a **30-second environmental prologue** directly into **tactical control of a single character in the mud and rain**.

```text
[ NEW GAME CLICKED ]
         |
         v
[ 30-SEC CINEMATIC VIGNETTE ] ===> Audio: Heavy bell chimes, rain, tallow crackle.
                                  Visual: Bruised-violet sky, towering cathedral, iron masks.
         |
         v
[ IN MEDIAS RES: COLD OPEN ]  ===> Alleyway of Cinders (Seventh Ward).
                                  Kaelen is panting against the stone, bloody knuckles.
         |
         v
[ GAIN TACTICAL CONTROL ]     ===> "Stand up." -> Two Inquisition Castellans corner him.
                                  First interactive tutorial fight begins!
```

---

## 2. Step 1: The 30-Second Opening Vignette

* **Visual Style:** High-contrast 2D pixel art vignettes centered in the $640 \times 360$ native canvas, framed by deep `#14171c` shadow borders.
* **Audio Landscape:** Complete silence, broken by the deep, mournful tolling of a massive iron bell. Rain hisses against wet slate; water drips into gutters; wind howls through stone arches.
* **Illustrated Vignettes (Fade In/Out with Typewriter Text):**

> **Slide 1:** *A bruised-violet, sunless sky choked with petrified clouds and drifting white ash.*  
> *"Two centuries ago, the stars and sun were snuffed out. In their place hangs the Shroud. No morning comes. Time is measured only by the tolling of the Cathedral bells."*

> **Slide 2:** *A cross-section of the earth: stone cities resting directly on top of massive, calcified titan bones and pulsing black veins.*  
> *"Beneath the basalt pavement sleeps the Low Sleep—an ancient, dreaming leviathan whose nightmares shake the earth and bleed mutagenic marrow into the wells."*

> **Slide 3:** *Windowless iron masks of the Sovereign Clerisy gazing down over thousands of youth herded in chains toward cavernous crypt arches.*  
> *"To keep the titan asleep, the Sovereign Clerisy demands the Red Tithing: a harvest of young blood to feed its dreams. Those who survive past five-and-twenty are spent, docile, and blind."*

> **Slide 4:** *Close-up of a shattered city watch insignia in the gutter mud, washed with fresh blood.*  
> *"In the Seventh Ward, a garrison guard refuses to lock the cages."*

---

## 3. Step 2: The Cold Open (Gaining Control of Kaelen)

The screen dissolves from the text directly into the active gameplay canvas:

```text
ALLEYWAY OF CINDERS (SEVENTH WARD) — NIGHT, RAIN
Map Grid: 10x6 narrow chokepoint corridor.
```

* **The Scene:** Rain lashes the cobblestones. The lighting is dark slate (`#1e232d`), illuminated only by a single sputtering warded tallow lantern on a wall bracket (`#fbbf24`).
* **Initial State:** Kaelen is kneeling in the mud, panting, clutching a notched billhook poleaxe. His left forearm is wrapped in bloody cloth; white calcified spurs poke through the linen.
* **The Player Prompt:** A subtle prompt appears: `[ Spacebar / Click: Rise ]`.
* **The Confrontation:** Kaelen stands. Two heavily armored **Iron Inquisition Castellan Wardens** in kettle helms and iron breastplates step into the alley from the street, blocking his exit with billhooks.

### Environmental Dialogue (In-World Voice)
* **Castellan Miller:** *"Order 44, deserter. The culling cages aren't full yet. You were sworn to drive the young stock into the pens, not break your corporal's jaw."*
* **Kaelen:** *"They're fifteen years old, Miller. You're herding children into the crypts."*
* **Castellan Miller:** *"They're tithes for the bells, boy. The entity hungers. Draw your steel, or you take their place on the slaughter wagons."*

The camera pulls back slightly, the **6-Tick Timeline HUD** slides down from the top of the viewport, and the first combat encounter begins.

---

## 4. How the Factions Are Viscerally Introduced

Rather than reading lore menus, the player learns the nature of each faction through immediate visual, audio, and tactical interaction:

### 4.1 The Sovereign Clerisy & The Low Sleep (Cosmic Dominion)
* **How Exposed:** The High Cathedral's warded bells toll in the background every few minutes with a heavy screen-shake effect. 
* **The Subterranean Tremor:** Midway through combat, a deep subsonic rumble shakes the alley—a ground tremor from the sleeping leviathan. Water ripples in the puddles; Kaelen grunts in discomfort as his calcified arm bone spurs pulse with faint bioluminescence.

### 4.2 The Iron Inquisition (Paramilitary Oppression)
* **How Exposed:** Faced as the immediate tactical enemy.
* **Combat Lesson:** They fight in tight, interlocking formations with kite shields and poleaxes. The game teaches that attacking their frontal guard is ineffective, forcing the player to use Kaelen's *Pommel Strike* during their windup tick to inflict **Stagger**, breaking their stance.

### 4.3 The Tallow Guild & The Docile Masses (Socio-Economic Despair)
* **How Exposed:** After defeating the two guards, Kaelen runs into the **Lower Meat Market**.
* **The Environment:** He witnesses an open-air rendering station where massive caldrons boil animal carcasses and unclaimed corpses into warded tallow. 
* **The Commoners:** Scores of hollow-eyed, wheezing elders (the Spent Generation, ages 25+) stand in silent, docile queues with wooden ration bowls, exchanging curfew badges for communion draughts and cups of gray salt. When an inquisitor violently shoves a young girl, none of the elders react—they stare blankly into their salt cups, deadened by church sedatives.

### 4.4 The Severed Root (The Insurrection)
* **How Exposed:** Inquisitorial whistles shriek. An ambush squad corners Kaelen in a dead end.
* **Vesper's Entrance:** From a roof overhang, a shadow drops. A soot flask shatters, filling the screen with a 3x3 smoke zone. **Vesper (The Gutter-Stalker)** appears at Kaelen's back:
  > *"Move your feet, tin-can. Grates are open. Follow the smoke."*
* **The Cell Doctrine:** The player now controls both Kaelen and Vesper, learning flanking backstabs and smoke disengages before continuing toward the canal bridge.

---

## 5. The Progression to the First Safehouse

```text
ALLEY FIGHT (Kaelen Solo) 
        |
        v
MARKET FLIGHT (Meet Vesper & The Docile Masses)
        |
        v
CANAL BRIDGE AMBUSH (Free Sister Maren & Bran from Mortuary Cage)
        |
        v
DROWNED RUNOFF DIVE (Escape through drainage lock)
        |
        v
THE SLUICE UNDERCROFT REACHED! (Safehouse Hub UI Unlocked)
```

1. **Step 3 of Prologue (The Bridge):** Kaelen and Vesper assault the bridge stockade, releasing **Sister Maren** and **Bran** from the iron cage.
2. **The Descent:** With heavy inquisitorial sirens sounding and searchlights sweeping the water, Vesper uses a crowbar to lever open a heavy sewer drainage grate.
3. **Arrival in The Sluice Undercroft:**
   - The four drop into the dark subterranean current and wash up on the stone flags of the safehouse hub.
   - They are met by **Bram Karr** by the glowing coals of his ventless hearth and **Sister Aline Vance** sharpening scalpels over her stone slab.
   - **Act I, Mission 1 begins:** The safehouse interface unlocks, the roster screen opens showing the 4 founding heroes occupying 4 of the 6 bunks, and the rebellion is born.
