# PORTRAITFX1 expression sheets: prompts

Status: PORTRAITFX1-XA batch 1 PLACED 2026-11-01 (synced `2026-11-11` to canonical `CANON.md` 3.2 Hub NPC IDs `talia_rusk`, `berek_dorn`, `sabina_kress` for `PR5-39` & `AI-05`; `berek_dorn` keeps his loupe rule `1:1`). Laid out like [ART3_PROMPTS.md](ART3_PROMPTS.md): the shared style block (the 2x2 template of
`docs/specs/PORTRAITFX1_TASKS.md` 2.4.2, own-portrait wording, the four cells fixed), then one section per id with
its character lines from ART3_PROMPTS.md; the sent prompt is the style block, a blank line, the id's block. Each
`reference:` line names the character's own portrait (sent as the reference image, never as prompt text). Cells:
Neutral top-left, Hurt top-right, Angry bottom-left, Afraid bottom-right.

## Style block

```text
Use the attached image as the reference: the same person: face, hair, outfit, colours, framing, lighting and pixel-art style; only the expression changes.
A 2x2 grid of four pixel art character portrait variations for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. 1:1 square image. No borders, no frame, no text, no dividers, no logo. Plain flat very dark background (#120e12) behind all figures. Each variation shows the same head-and-shoulders, three-quarter view of the character below, the head the same size and in the same place in every cell. Each variation utilizes a 16-bit pixel art style with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per color, and light dithering on metal and cloth. Strong cool key light from the upper left and a warm amber tallow-lantern rim light (#d97706) from the right. Same outfit, hair and colours in all four.

Keep every accessory, scar, marking and piece of equipment exactly as in the reference in all four cells, including eyepieces, masks and jewellery.

The four variations are:
- Top-Left: Neutral: the character's usual expression (the Mood line below), mouth closed, eyes looking right.
- Top-Right: Hurt: in sharp pain: both eyes squeezed tightly shut with deep dark creases, brows pinched up and together, mouth pulled wide open in a grimace showing clenched teeth, strong dark lines around the eyes and mouth.
- Bottom-Left: Angry: a deep scowl, brows drawn down hard into a V, nostrils flared, teeth bared or jaw clenched, eyes narrowed and glaring, cheeks flushed red; unmistakably furious, clearly different from Neutral.
- Bottom-Right: Afraid: eyes wide, brows raised, mouth slightly open, head drawn in.
```

## Talia Rusk (`talia_rusk`)

reference: art_src/talia_rusk.png

Hub NPC (non-combat). The game: Talia Rusk (29), crippled cipher spymaster who runs the dead-drop mission board at the
Water-Lock Basin; mentored Vesper and shelters Ivo; deaf in her right ear from Ruthven's Inquisition sweep.

```text
Character: Talia Rusk, Cipher Spymaster of the Basin. Stylization warm and readable: sharp observant eyes, weathered late-twenties lines around the mouth and brow, a small pointed chin, expressive dark brows, a dry guarded half-smile. A Spent woman of twenty-nine, lean and tough. Dark-brown hair pulled back in a low utilitarian knot with stray soot-dulled strands, swept completely clear of her right ear. Her right ear, the deaf one on the near side toward the viewer (the left side of the picture), is fully visible: scarred and notched at the rim, puckered pale burn tissue running onto her jaw from an Inquisition flash-pot. Heavy grey-brown wool dispatch cloak pinned over patched leather, bone-cipher tally rods at her collar, the carved top of an ash-wood underarm crutch resting against her near shoulder.
Mood: sharp, watchful, unsentimental.
Main colours: damp wool grey-brown and boiled leather, carved pale bone rods, tallow amber rim light.
```

## Berek Dorn (`berek_dorn`)

reference: art_src/berek_dorn.png

Hub NPC (non-combat). The game: Berek "Cinder-Lung" Dorn (33), the smith at the Tinker's Hearth and Preservationist
voice; former Fourth Kiln master valve-caster, lungs ruined, mind intact.

```text
Character: Berek "Cinder-Lung" Dorn, Smith of the Tinker's Hearth. Stylization warm and readable: a big square head, a huge barrel chest and wide shoulders, a broad flattened nose, a heavy kind-but-tired brow over deep-set eyes. A man in his early thirties who looks older. A wild, shaggy mane of dark-brown hair streaked with grey, worn loose to the shoulders, and a big bushy soot-streaked beard; a thick, scarred throat. An iron jeweller's loupe strapped over his right eye. The iron loupe stays strapped over his right eye in every cell, also when the eyes are squeezed shut. A scorched cowhide apron studded with riveted pouches over a rolled-sleeve work shirt, soot and burn marks on his forearms. Warm forge glow on one side of his face.
Mood: gruff, steady, stubborn.
Main colours: boiled leather and scorched cowhide brown, scavenged iron, tallow amber forge glow.
```

## Chirurgeon Sabina (`sabina_kress`)

reference: art_src/sabina_kress.png

Hub NPC (non-combat). The game: Chirurgeon Sabina Kress (31), surgeon at the Apothecary's Slab and the Flesh-Broker
voice; exiled outer-ward anatomist who once studied under Chief Anatomist Lazar.

```text
Character: Chirurgeon Sabina Kress, Surgeon of the Apothecary's Slab. Stylization readable but cool: a long narrow face with high sharp cheekbones and hollow cheeks, severe silver-grey eyes under straight dark brows, a small clinical mouth, no spectacles. A tall, gaunt woman of thirty-one. Cropped soot-black hair streaked with early silver at both temples, bareheaded, no veil and no wimple. A high-collared dark oiled-green prosector coat over a vitriol-stained leather surgical apron, an unhooked dark leather surgical half-mask hanging loose from one collar button, a bandolier of glass pipettes and polished steel scalpels across the chest, sulfur-stained fingertips.
Mood: clinical, unsentimental, unflinching.
Main colours: dark oiled prosector green and damp basalt, boiled leather apron, polished steel scalpels, faint amber-green glass pipettes.
```

## Captain Vane (`captain_vane`)

reference: art_src/captain_vane.png

Boss (warrior), the founders' portraits row (STORYUI1_TASKS 1.5). The game: Kaelen's former commanding officer, who
sold his own men to the Inquisition for a church commission and leads the sweep hunting Kaelen.

```text
Character: Captain Vane, Castellan Captain of the Seventh Ward. Stylization in between: readable and human but pushed, a broad, square, oversized jaw going to fat beneath, a heavy jutting brow, a long hawkish nose, cold pale narrowed eyes and a thin contemptuous mouth turned down at one corner. A man in his forties, clean-shaven, once handsome. Thick iron-grey hair to the collar, swept straight back from a sharp widow's peak and worn loose, its ends curling over the gorget. Castellan plate armour that he has had gilded, the candle-gold gilding now scratched and defaced with gouges down to the dark iron beneath, a rust-crimson Castellan tabard over it, a gorget. The long haft and hooked blade of an executioner's billhook rises behind his shoulder; the rim of an iron pavise painted with a weeping cathedral eye shows at the edge of the frame.
Mood: cold, ambitious, contemptuous.
Main colours: candle gold gilding over scavenged iron plate, rust crimson tabard.
```

## Proctor Quell (`proctor_quell`)

reference: art_src/proctor_quell.png

Boss (warrior), brief N6. The game: "the Scalpel of St. Lyra's", an Inquisition vivisector with an illegal marrow
valve in his neck; he denounced Maren.

```text
Character: Proctor Quell, the Scalpel of St. Lyra's. Stylization pushed far: gaunt, elongated and uncanny, a long narrow skull, an overlong thin neck, cheeks sunk to hollows under sharp cheekbones, a long pointed chin, heavy-lidded pale eyes and a thin, too-wide, pleased smile. An inquisitor in his forties. His crown is bald and shining; from the back and sides long, lank, thinning black hair hangs straight and loose past his shoulders. A high-collared white linen cassock gone stiff with black marrow stains and spattered with blood. A prominent brass valve set into the side of his long neck, the skin puckered round it, a thin wisp of violet steam venting from it. A bronze bird-beak mask hangs on a chain at his chest; the tips of two bone-saws show at the edge of the frame.
Mood: cruel, clinical, amused.
Main colours: bleached linen white with blight violet marrow stains, brass and scavenged iron.
```

## Orlov (`guild_master_orlov`)

reference: art_src/guild_master_orlov.png

Boss (brawler), brief N7. The game: Guild-Master Orlov, "the Ash-Crawl Butcher", owns the rendering pipelines of the
Sunk Ward; Vesper's enemy.

```text
Character: Guild-Master Orlov, the Ash-Crawl Butcher. Stylization pushed far: grotesque, a huge bulk filling the frame, the head small for the vast body, the neck lost in rolls of fat and muscle, massive sloping shoulders, heavy hanging jowls, tiny hard eyes sunk deep in fat, a broad flattened nose. A shaved head shining with grease, and a long, greasy black beard twisted into two thick braids bound with iron rings, hanging down over the harness. An iron-plated butcher's harness of boiled leather straps over a stained leather apron, a brass boiler on his back with a steam hose curling over his shoulder, large barbed iron meat hooks hanging from the harness at his chest. Warm tallow light glistening on his skin.
Mood: greedy, brutal, self-satisfied.
Main colours: scavenged iron plates, boiled leather brown, tallow amber brass and light.
```

## Nemeth (`inquisitor_prelate_nemeth`)

reference: art_src/inquisitor_prelate_nemeth.png

Boss (warrior), brief N8. The game: Inquisitor-Prelate Nemeth, the marrow-infused commander at the Crucible
Watchpost whom cold steel cannot finish; the Act 1 climax villain, who returns in Act 3.

```text
Character: Inquisitor-Prelate Nemeth. Stylization pushed far: elongated and uncanny, a very long, narrow, severe face, sharp angular cheekbones over sunken cheeks, a high domed forehead, no eyebrows, pale glassy eyes glowing faintly violet, a thin lipless expressionless mouth. A tall, upright man of indeterminate age, his skin an unnatural infused ash grey, smooth and dead like wet stone. Long, straight, fine white hair falls loose from a middle parting, past his shoulders and over the plate like a pale curtain. Mirror-polished steel plate armour that reflects the light in hard bright bands, worn over a deep rust-crimson cassock with a high collar; a glass vial of glowing violet tallow-marrow set into the throat of the collar, thin violet veins spreading from it under the grey skin of his neck and jaw.
Mood: cold, upright, untouchable.
Main colours: polished steel and scavenged iron, rust crimson cassock, blight violet vial.
```
