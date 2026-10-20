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

## Generic base portraits

One per mob kind, with faces no named character has; more mob variants are recolours of these (tools/portraits/recolor.py).

## Ember Warden soldier (`warrior`)

Generic base portrait (warrior). The game: Shared by every unnamed warrior.

```text
Character: Ember Warden soldier, a nameless warrior of the Wardens. An ordinary Warden foot soldier, a man in his thirties with a plain, forgettable, honest face and a short dark beard, an open-faced steel helmet. Plain red tabard over mail, a steel gorget, a kite shield rim behind the shoulder. Not a hero: nothing distinctive.
Mood: dutiful, tired, steady.
Main colours: red and steel grey with brass.
```

## Warden archer (`ranger`)

Generic base portrait (ranger). The game: Shared by every unnamed ranger.

```text
Character: Warden archer, a nameless ranger. An ordinary Warden archer, a woman in her twenties with a plain face and tied-back brown hair, a brown leather hood down, a green cloak, a quiver strap and a longbow over the shoulder. Nothing distinctive.
Mood: alert, quiet, plain.
Main colours: green and brown leather with a red cord.
```

## Pit brawler (`brawler`)

Generic base portrait (brawler). The game: Shared by every unnamed brawler.

```text
Character: Pit brawler, a nameless brawler. An ordinary brawler, a bald man with a flattened nose and a thick neck, cloth-wrapped fists raised in a guard, a sleeveless leather vest, a plain grey headband. Nothing distinctive.
Mood: tough, wary, plain.
Main colours: tan skin, grey cloth and brown leather.
```

## Warden acolyte (`priest`)

Generic base portrait (priest). The game: Shared by every unnamed priest.

```text
Character: Warden acolyte, a nameless priest. An ordinary acolyte, a young woman with a round, plain face, a cream-white robe and cowl with a simple gold trim, a small mace head at the edge of the frame. Nothing distinctive.
Mood: earnest, calm, plain.
Main colours: cream white and gold.
```

## Cutpurse (`rogue`)

Generic base portrait (rogue). The game: Shared by every unnamed rogue.

```text
Character: Cutpurse, a nameless rogue. An ordinary rogue, a lean woman with a narrow plain face half-hidden by a dark hood and a cloth mask over the nose, dark purple-grey leathers, a dagger at the shoulder. Nothing distinctive.
Mood: shifty, watchful, plain.
Main colours: dusk purple and charcoal.
```

## Hedge-mage (`mage`)

Generic base portrait (mage). The game: Shared by every unnamed mage.

```text
Character: Hedge-mage, a nameless mage. An ordinary hedge-mage, an older bald man with a white stubble beard, a plain blue robe with a hood down, no hat, a short staff with a dim blue gem. Nothing distinctive.
Mood: thoughtful, guarded, plain.
Main colours: blue and grey with a pale gem blue.
```

## Horde brawler (`goblin_brawler`)

Generic base portrait (goblin). The game: Shared by every unnamed goblin brawler of the Ashen Horde.

```text
Character: Horde brawler, a nameless goblin brawler. An ordinary goblin brawler of the Ashen Horde: ash-grey skin, long ears, a heavy brow, small yellow eyes, cloth-wrapped fists, a crude leather harness, soot smears. A common grunt, nothing distinctive.
Mood: brutish, eager, plain.
Main colours: ash grey and brown leather.
```

## Horde skirmisher (`goblin_skirmisher`)

Generic base portrait (goblin). The game: Shared by every unnamed goblin skirmisher.

```text
Character: Horde skirmisher, a nameless goblin skirmisher. An ordinary goblin skirmisher of the Ashen Horde: ash-grey skin, long ears, a thin sharp face, a ragged grey hood, a crooked knife held close. A common raider, nothing distinctive.
Mood: sneaky, quick, plain.
Main colours: ash grey and dark rags.
```

## Horde archer (`goblin_archer`)

Generic base portrait (goblin). The game: Shared by every unnamed goblin archer.

```text
Character: Horde archer, a nameless goblin archer. An ordinary goblin archer of the Ashen Horde: ash-grey skin, long ears, squinting eyes, a crude short bow over the shoulder, a quiver of black-fletched arrows, a soot-stained hide jerkin. Head and shoulders. A common archer, nothing distinctive.
Mood: wary, sly, plain.
Main colours: ash grey and black hide.
```

## Horde shaman (`goblin_shaman`)

Generic base portrait (goblin). The game: Shared by every unnamed goblin shaman.

```text
Character: Horde shaman, a nameless goblin shaman. An ordinary goblin shaman of the Ashen Horde: ash-grey skin, long ears, an ash-white painted face, a few bone beads, a short crooked staff topped with a small skull. Less ornate than a named shaman.
Mood: mystic, sly, plain.
Main colours: ash grey and bone white.
```

## Horde berserker (`goblin_berserker`)

Generic base portrait (goblin). The game: Shared by every unnamed goblin berserker.

```text
Character: Horde berserker, a nameless goblin berserker. An ordinary goblin berserker of the Ashen Horde: ash-grey skin, long ears, a snarling open mouth, red war paint stripes, bare scarred shoulders under a scrap of fur. A common berserker, nothing distinctive.
Mood: furious, wild, plain.
Main colours: ash grey and red paint with fur.
```

## Training dummy (`dummy`)

Generic base portrait (prop). The game: The training dummy of the Gear Dojo and the arena.

```text
Character: Training dummy, the yard's straw target. Not a person: a battered training dummy of straw and sackcloth on a wooden post, a painted red target circle on the chest, a dented old helmet on its head, straw poking out of slashes, a rope belt.
Mood: battered, patient, comic.
Main colours: straw yellow, sackcloth brown and faded red.
```
