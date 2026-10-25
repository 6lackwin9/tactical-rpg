# ART3: Scarlet Lullaby portrait prompts

Status: for the agreed image run (user 2026-10-25: "ok to generate monster and character art using API"; 11 portraits,
2 candidates each, Gemini 3 Pro Image). Same route as [ART2](ART2.md): Mara Voss's portrait
(`art_src/mara_voss.png`) goes along as the style reference, so the new portraits sit beside the ART2 ones;
`tools/portraits/generate.py run <model> <id>... --n 2 --prompts games/emberwake/docs/specs/ART3_PROMPTS.md` writes
the candidates to `art_src/<id>_gen<time>_<n>.png`; `tools/portraits/pixelize.py` turns each into a game portrait
(96 px grid, 4 shades per colour family). The user picks a candidate before anything is placed under
`games/emberwake/assets/portraits/`.

Sources for each block: the CAST1 asset briefs N1, N2 and N4-N8 ([CAST1_ASSETS.md](CAST1_ASSETS.md) section 7), the
founders' portraits row of `docs/specs/STORYUI1_TASKS.md` 1.5 (Vane, Tess, Bram, Aline), the art bible
(`games/emberwake/docs/lore/specs/art_and_visual_style.md` sections 1, 3 and 7.2) and the hub cast looks
(`games/emberwake/docs/lore/background/npcs_and_questlines.md` section 2). Ids are the CAST1 unit ids
([CAST1_BESTIARY.md](CAST1_BESTIARY.md), [CAST1_BOSSES.md](CAST1_BOSSES.md)) and the hub NPC ids of STORYUI1
(`tess`, `bram`, `aline`). Hue numbers are the art bible's 16 master hues. Tone rule: children are never fighters or
victims on screen, so every Unsealed youth drawn here is an adult; Tess (19, an adult; decisions/lore.md "Tess's age") is drawn as a young
adult. Where the sources leave a detail open (hair, beard, age in looks), the block makes a best guess; change it
freely.

Every prompt starts with the same style block:

```text
Use the attached image (Mara Voss) as the style reference: the same pixel-art style, level of detail, framing, lighting, outline weight and flat dark background. This is a different subject, not the person in the reference.
Pixel art character portrait for a gothic dark fantasy tactical RPG set in a sunless church city lit only by tallow lanterns, in the style of detailed 16-bit SNES/GBA-era RPG portraits. Square 1:1 image. Head and shoulders (for a creature: head and forequarters), the head filling about half the frame, three-quarter view facing slightly to the viewer's right, eyes looking right. Strong cool key light from the upper left, a warm amber tallow-lantern rim light (#d97706) from the right edge, deep shadows on the far side of the face. Clear dark outlines, hand-placed pixel clusters, limited palette with 4 shades per colour, light dithering on metal and cloth. Muted, matte, grimy materials: dented iron, boiled leather, wool and soot-stained linen; anything corrupted is jet-black bone, chalk-white calcified growth and glowing violet marrow. Colours only from this palette and its shades: void black #0a0b10, damp basalt #14171f, wet slate #1e232d, rotting oak #2c221e, scavenged iron #4b5563, polished steel #9ca3af, bleached linen #d9cbb4, cinder ash #6b7280, boiled leather #92400e, tallow amber #d97706, candle gold #fbbf24, arterial red #e11d48, rust crimson #991b1b, blight violet #9333ea, resonant orchid #c084fc, salt cyan #06b6d4; human skin #6b4a3a / #a47a5f / #d4a988, pale and underfed. The subject reads clearly against the dark: mid-tone figure, bright accents only on small details. Plain flat very dark background (#120e12), no gradient, no scenery, nothing behind the figure. No border, no frame, no text, no logo, no watermark.
```

## Monsters

## Render-Beast (`render_beast`)

Minion (rogue), brief N1. The game: the Tallow Guild's and the Inquisition's hounds; Orlov's pack at the Tallow Vats.

```text
Character: Render-Beast, a starved horror hound of the rendering yards. Not a person and not any real dog breed: a mastiff-sized hound, emaciated and starving, completely hairless, sickly pale bleached-linen skin with grey-violet blotches stretched tight over the bones. Every rib, the jutting hip bones and the knobs of the spine show through the skin; a ridge of jagged bone spurs breaks out along the spine behind the head. A long gaunt skull, sunken cheeks, small clouded white eyes deep in their sockets. No muzzle, no collar, no harness: crude iron wire is stitched through its lips and through the hinge of the jaw like surgical sutures, the wire pulling the lips back so the gums are torn and raw red around long yellow teeth. Low, forward-leaning, crouched to pounce, head lowered toward the viewer's right.
Mood: starved, unnatural, vicious, about to lunge.
Main colours: bleached linen skin with damp basalt shadow, scavenged iron wire, arterial red gums.
```

## Ash-Crawler (`ash_crawler`)

Minion (rogue), brief N2. The game: the undercroft's aberrations around the Amalgam.

```text
Character: Ash-Crawler, an aberration of the undercroft. Not a person: a giant many-legged chitinous centipede about a human's length, coiled, its head and front segments raised toward the viewer's right. Dark basalt-black chitin plates with sharp edges, many thin jointed legs, hooked mandibles dripping. Behind the head its translucent swollen abdomen glows with bubbling blight-violet acid, lit from inside by a resonant orchid glow. The body coils round below the head, low and long.
Mood: alien, coiled, ready to spit.
Main colours: damp basalt chitin, blight violet acid, resonant orchid glow.
```

## Inquisitorial Purifier (`inquisitorial_purifier`)

Elite (mage), brief N4. The game: the Inquisition's burner with a pyre nozzle; flushes the cell out of cover.

```text
Character: Inquisitorial Purifier, a burner of the Iron Inquisition. Faceless: the whole face is hidden by a bronze bird-beak plague-doctor gas mask with a long curved beak, riveted seams and two round dark glass eye lenses with a faint amber glint; a soot-grey cloth hood over the head. A heavy warded iron apron etched with small ward marks, a thick leather strap across the chest carrying a sulfur canister on the back, a brass hose from it over the shoulder. Soot on everything.
Mood: inhuman, methodical, menacing.
Main colours: scavenged iron and bronze (tallow amber shades) with cinder ash soot.
```

## Calcified Husk (`calcified_husk`)

Standard (warrior), brief N5. The game: a corpse the titan's marrow has turned to stone, walking with the Amalgam.

```text
Character: Calcified Husk, a petrified corpse. The corpse of a grown man, a former miner in his twenties, risen and half turned to stone: grey rotten skin cracked like dry clay, jet-black bone spurs jutting through the cheek, the collarbone and the shoulder, chalk-white calcified crust over the jaw, empty dark eye sockets with no eyes, the mouth slack. The rags of a miner's leather jerkin and a torn hood. Not gory: dry, dead and stony.
Mood: empty, dead, relentless.
Main colours: void black bone and cinder ash skin with chalk white crust.
```

## Bosses

## Captain Vane (`captain_vane`)

Boss (warrior), the founders' portraits row (STORYUI1_TASKS 1.5). The game: Kaelen's former commanding officer, who
sold his own men to the Inquisition for a church commission and leads the sweep hunting Kaelen.

```text
Character: Captain Vane, Castellan Captain of the Seventh Ward. A man in his forties, a hard handsome clean-shaven face going to fat at the jaw, cropped iron-grey hair, cold pale eyes, a thin contemptuous mouth. Castellan plate armour that he has had gilded, the candle-gold gilding now scratched and defaced with gouges down to the dark iron beneath, a rust-crimson Castellan tabard over it, a gorget. The long haft and hooked blade of an executioner's billhook rises behind his shoulder; the rim of an iron pavise painted with a weeping cathedral eye shows at the edge of the frame.
Mood: cold, ambitious, contemptuous.
Main colours: candle gold gilding over scavenged iron plate, rust crimson tabard.
```

## Proctor Quell (`proctor_quell`)

Boss (warrior), brief N6. The game: "the Scalpel of St. Lyra's", an Inquisition vivisector with an illegal marrow
valve in his neck; he denounced Maren.

```text
Character: Proctor Quell, the Scalpel of St. Lyra's. A lean, narrow-faced inquisitor in his forties, hollow cheeks, slicked-back thin dark hair, heavy-lidded eyes and a thin, pleased smile. A high-collared white linen cassock gone stiff with black marrow stains and spattered with blood. A brass valve set into the side of his neck, a thin wisp of violet steam venting from it. A bronze bird-beak mask hangs on a chain at his chest; the tips of two bone-saws show at the edge of the frame.
Mood: cruel, clinical, amused.
Main colours: bleached linen white with blight violet marrow stains, brass and scavenged iron.
```

## Orlov (`guild_master_orlov`)

Boss (brawler), brief N7. The game: Guild-Master Orlov, "the Ash-Crawl Butcher", owns the rendering pipelines of the
Sunk Ward; Vesper's enemy.

```text
Character: Guild-Master Orlov, the Ash-Crawl Butcher. An enormous man, obese and heavily muscled, a shaved head shining with grease, heavy jowls, small hard eyes, a thick neck. An iron-plated butcher's harness of boiled leather straps over a stained leather apron, a brass boiler on his back with a steam hose curling over his shoulder, barbed iron meat hooks hanging from the harness at his chest. Warm tallow light glistening on his skin.
Mood: greedy, brutal, self-satisfied.
Main colours: scavenged iron plates, boiled leather brown, tallow amber brass and light.
```

## Nemeth (`inquisitor_prelate_nemeth`)

Boss (warrior), brief N8. The game: Inquisitor-Prelate Nemeth, the marrow-infused commander at the Crucible
Watchpost whom cold steel cannot finish; the Act 1 climax villain, who returns in Act 3.

```text
Character: Inquisitor-Prelate Nemeth. A tall, upright man of indeterminate age, gaunt and severe, his skin an unnatural ash grey from marrow infusion, close-cropped white hair, eyes faintly violet, an expressionless mouth. Mirror-polished steel plate armour that reflects the light in hard bright bands, worn over a deep rust-crimson cassock with a high collar; a glass vial of glowing violet tallow-marrow set into the throat of the collar, thin violet veins spreading from it under the grey skin of his neck.
Mood: cold, upright, untouchable.
Main colours: polished steel and scavenged iron, rust crimson cassock, blight violet vial.
```

## Hub cast

## Tess (`tess`)

Hub NPC (non-combat), the founders' portraits row. The game: Tess Morn, the cell's dead-drop courier; runs the mission
board at the Water-Lock Basin; Vesper's former runner contact; deaf in one ear from an Inquisition flash-bomb.

```text
Character: Tess Morn, Dead-Drop Courier. A young woman, a young adult, wiry and compact, a sharp quick face with soot smudges, alert narrowed eyes, short choppy dark hair tucked behind one ear. That visible ear is scarred and notched, the skin round it puckered by an old burn from a flash-bomb. Patched leather shingle-armour of overlapping scraps over greasy wool wraps, a courier's satchel strap across the chest with a rolled message tube.
Mood: quick, wary, determined.
Main colours: boiled leather brown and rotting oak, grey wool wraps, a small tallow amber accent.
```

## Bram (`bram`)

Hub NPC (non-combat), the founders' portraits row. The game: Bram "Iron-Lung" Karr, the smith at the Tinker's Hearth;
a former master foundryman, lungs ruined, mind intact.

```text
Character: Bram "Iron-Lung" Karr, Tinker of the Hearth. A broad, barrel-chested man in his thirties who looks older, close-cropped dark hair and a short soot-streaked beard, a thick scarred throat, a heavy tired brow. An iron jeweller's loupe strapped over his right eye. A scorched cowhide apron studded with riveted pouches over a rolled-sleeve work shirt, soot and burn marks on his forearms. Warm forge glow on one side of his face.
Mood: gruff, steady, stubborn.
Main colours: boiled leather and scorched cowhide brown, scavenged iron, tallow amber forge glow.
```

## Sister Aline (`aline`)

Hub NPC (non-combat), the founders' portraits row. The game: Sister Aline Vance, surgeon at the Apothecary's Slab and
the Flesh-Broker voice; a former Anointed Prosector of the Hall of Holy Dissection.

```text
Character: Sister Aline, Surgeon of the Apothecary's Slab. A tall, gaunt woman in her late twenties, a sharp pale face with hollow cheeks, cool calculating eyes, cropped soot-black hair. A clerical habit with its holy emblem cut away and defaced, under a stiff leather embalmer's apron stained with vitriol. A bandolier of silver scalpels and bone cannulas across the chest; sulfur-yellow stained fingertips on one hand raised near the strap. Faint early black-violet veins creeping up the side of her neck.
Mood: cold, pragmatic, unflinching.
Main colours: bleached linen and damp basalt habit, boiled leather apron, polished steel scalpels, a hint of blight violet.
```
