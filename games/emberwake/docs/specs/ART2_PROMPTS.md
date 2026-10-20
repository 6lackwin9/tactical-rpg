# ART2: portrait prompts

Status: for the user's image generation (2026-10-20). Attach Mara Voss's portrait as the style reference; save each result as `art_src/<id>.png`; `tools/portraits/pixelize.py` turns it into the game portrait. Where the game data does not say whether a character is a man or a woman, the prompt uses the planner's best guess (user 2026-10-20: "use your best guess"): Tev, Kit and the patrol warchief men, Sella, Alda, Hest, Nok and Vex women; change any of them.

Every prompt starts with the same style block:

```text
Use the attached image (Mara Voss) as the style reference: the same pixel-art style, level of detail, framing, lighting and flat dark background. This is a different person, not the person in the reference.
Pixel art character portrait for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. Square 1:1 image. Head and shoulders, the head filling about half the frame, three-quarter view facing slightly to the viewer's right, eyes looking right. Strong key light from the upper left, a warm orange rim light from the right edge, deep shadows on the far side of the face. Clear dark outlines, hand-placed pixel clusters, limited palette with 4 shades per colour, light dithering on metal and cloth. Plain flat very dark background (#120e12), no gradient, nothing behind the figure. No border, no frame, no text, no logo, no watermark.
```

## Bryn Hale (`bryn_hale`)

Playable hero (warrior). The game: A Warden's son from the burned post at Emberfall; carries his father's shield and his temper.

```text
Character: Bryn Hale, Warden Recruit. A young man of about nineteen, lean but broad in the shoulders, a hot-tempered scowl, soot on one cheek, short unruly dark brown hair. Wearing a plain deep red Ember Warden tabard over mail, a steel gorget. The dented rim of an old kite shield with a faded ember emblem shows behind his shoulder.
Mood: angry, eager, proud.
Main colours: deep red and steel grey with worn brass accents.
```

## Tev Quill (`tev_quill`)

Playable hero (ranger). The game: A quiet orphan of the Marches who reads smoke like other people read books. Watchful, wry.

```text
Character: Tev Quill, Scout of the Pass. A young man in his twenties, slim, sharp watchful eyes and a wry half-smile, a smudge of soot by the eye. A deep green hood pushed back, brown leather jerkin, a quiver strap across the chest, a red Warden cord on the shoulder.
Mood: watchful, wry, quiet.
Main colours: forest green and brown leather with a red accent.
```

## Sella Marsh (`sella_marsh`)

Playable hero (ranger). The game: Hunts ash-hares in the wastes and counts every arrow twice.

```text
Character: Sella Marsh, Archer of the North Wall. A woman in her thirties, weathered by the ash wastes, narrowed careful eyes, wind-chapped skin. A fur-trimmed grey-green hood, a pale ash-hare pelt over one shoulder, the string of a longbow across the chest, a red Warden armband.
Mood: careful, frugal, patient.
Main colours: grey-green and pale fur with a red accent.
```

## Dagny Coal (`dagny_coal`)

Playable hero (brawler). The game: Fought for coin in the slag pits until the Horde burned them; fists are what she kept.

```text
Character: Dagny Coal, Pit Fighter of Slag Gate. A woman pit fighter with a powerful build, a once-broken nose, cropped dark hair under an orange cloth headband, burn scars and soot on bare muscular shoulders. A sleeveless dark leather vest. One cloth-wrapped fist raised near the chin in a guard.
Mood: tough, blunt, unbowed.
Main colours: warm tan skin, charcoal leather and an orange headband.
```

## Alda Wren (`alda_wren`)

Playable hero (priest). The game: Tends the keep's one fire and every Warden who comes back from the pass.

```text
Character: Alda Wren, Hearth-Keeper. A woman in her forties, kind but tired eyes, faint laugh lines. A cream-white robe and cowl with gold trim, a small iron ember-lantern hanging at the chest, warm firelight on the face. The head of a mace shows at the edge of the frame.
Mood: warm, steady, weary.
Main colours: cream white and gold with ember orange.
```

## Kit Vane (`kit_vane`)

Playable hero (rogue). The game: Stole from the Wardens once, got caught, and stayed.

```text
Character: Kit Vane, Oathbound Cutpurse. A young man, a sly grin and quick eyes, a dark hood and a cloth mask pulled down round the neck. Purple-grey leathers with many small pockets, a dagger hilt at the shoulder, a newly pinned red Warden ember badge.
Mood: sly, restless, loyal.
Main colours: dusk purple and charcoal with a red accent.
```

## Oren Dusk (`oren_dusk`)

Playable hero (mage). The game: Reads embers the way Tev reads smoke; nobody asks where he learned fire.

```text
Character: Oren Dusk, Hedge-Mage of the Forge. A man in his forties, a lean face, grey-streaked dark hair under a wide-brimmed blue hat, eyes reflecting orange embers. A deep blue robe with gold trim, the top of a staff with a glowing pale-blue gem beside him.
Mood: secretive, calm, intense.
Main colours: deep blue and gold with ember orange and gem blue.
```

## Brask Toller (`brask_toller`)

Important NPC (warrior). The game: Keeps the duty roll in his head and the names of the dead on his belt.

```text
Character: Brask Toller, Muster Sergeant of the Wardens. A veteran sergeant in his fifties, a grizzled grey beard, a heavy jaw, deep-set eyes. Dented steel plate over a faded red tabard, a leather strap across the chest hung with small wooden name tags.
Mood: gruff, dutiful, grieving.
Main colours: faded red and dented steel with leather brown.
```

## Hest the Quartermaster (`hest_the_quartermaster`)

Important NPC (warrior). The game: Counts every arrow twice and every soldier three times.

```text
Character: Hest the Quartermaster, Keeper of the Stores. A woman in her fifties, thin and precise, a sharp nose, a pencil-straight mouth, ink on the fingers. A red Warden coat with brass buttons over mail, a ring of iron keys and a quill behind the ear.
Mood: exacting, dry, tireless.
Main colours: red and brass with ink black.
```

## Wenn Tallow (`wenn_tallow`)

Important NPC (ranger). The game: Orsa's nephew; drives the steel wagon and swears he can shoot.

```text
Character: Wenn Tallow, Orsa's Carter. A young man, freckled and eager, a boastful grin, a dusty driver's cap. A road-dusted carter's coat with straps, a short bow slung awkwardly over one shoulder. Not a Warden: no red.
Mood: eager, boastful, good-hearted.
Main colours: dusty brown and mustard yellow.
```

## Grakka Ashmaw (`grakka_ashmaw`)

Boss (goblin warlord). The game: Wears a crown of rusted nails, one for every Warden post he burned.

```text
Character: Grakka Ashmaw, Chieftain of the Horde. A huge scarred goblin chieftain of the Ashen Horde: ash-grey skin, long ragged ears, a tusked underbite, burning orange eyes, soot-and-ember war paint. A crown of rusted iron nails driven through a band. Crude iron plates, chains and a heavy notched blade over the shoulder; faint fire glow behind.
Mood: menacing, cruel, proud.
Main colours: ash grey and rust with ember red.
```

## Horde Warchief (`ash_warchief`)

Leader (goblin). The game: Leads the patrols that circle the ruins.

```text
Character: Horde Warchief, Patrol Leader. A male goblin patrol leader of the Ashen Horde, ash-grey skin, long ears, a hard alert stare, a bone-and-iron gorget, a horn signal on a strap, cloth-wrapped fists, a red sash as a mark of rank.
Mood: hard, alert, commanding.
Main colours: ash grey and bone with a red sash.
```

## Bolg Redhand (`bolg_redhand`)

Named enemy (goblin). The game: Has never retreated, mostly because he forgets there is a direction for it.

```text
Character: Bolg Redhand, Berserker. A goblin berserker of the Ashen Horde: ash-grey skin, long ears, wild eyes, a frothing grin, hands painted blood red raised into the frame, tattered fur over bare scarred shoulders.
Mood: reckless, wild, gleeful.
Main colours: ash grey and blood red with dirty fur.
```

## Skarn Ironjaw (`skarn_ironjaw`)

Named enemy (goblin). The game: Cracked his jaw on a Warden shield and never stopped grinning about it.

```text
Character: Skarn Ironjaw, Brawler of the Horde. A goblin brawler of the Ashen Horde: ash-grey skin, long ears, a riveted iron plate bracing his broken jaw, a wide crooked grin, cloth-wrapped fists, a leather harness.
Mood: cocky, tough, grinning.
Main colours: ash grey and iron with leather brown.
```

## Gruk Smokecaller (`gruk_smokecaller`)

Named enemy (goblin). The game: Claims the ash talks to him. It mostly says 'attack'.

```text
Character: Gruk Smokecaller, Warband Shaman. A goblin shaman of the Ashen Horde: ash-grey skin, long ears, milky half-closed eyes, an ash-smeared face, bone charms and feathers, wisps of grey smoke curling round him, a crude totem staff.
Mood: eerie, fervent, sly.
Main colours: ash grey and bone with smoke grey and ember.
```

## Nok Flamebow (`nok_flamebow`)

Named enemy (goblin). The game: Fires pitch-tipped arrows from the ridgeline and runs when anyone gets close.

```text
Character: Nok Flamebow, Horde Archer. A female goblin archer of the Ashen Horde: ash-grey skin, long ears, skittish darting eyes, pitch-stained fingers, a crude bow with a burning pitch-tipped arrow nocked.
Mood: skittish, sneaky, spiteful.
Main colours: ash grey and pitch black with flame orange.
```

## Vex the Cutpurse (`vex_cutpurse`)

Named enemy (goblin). The game: Goes for whoever bleeds first.

```text
Character: Vex the Cutpurse, Skirmisher. A female goblin skirmisher of the Ashen Horde: ash-grey skin, long ears, hungry predatory eyes, a ragged dark hood, twin crooked knives held close.
Mood: predatory, quick, cruel.
Main colours: ash grey and dark rags with steel.
```
