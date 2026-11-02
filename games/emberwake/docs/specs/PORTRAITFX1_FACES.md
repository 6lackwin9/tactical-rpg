# PORTRAITFX1 expression sheets: prompts

Status: PORTRAITFX1-XA batch 1 PLACED 2026-11-01 (the seven aligned sets are in the game's portraits folders with sidecars) (re-queued 2026-11-01 with the accessory rule, a stronger Angry cell and Bram's loupe line) (the seven characters with their own ART3 portraits; the six founders wait for
FOUNDERPORT1-P). Laid out like [ART3_PROMPTS.md](ART3_PROMPTS.md): the shared style block (the 2x2 template of
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

## Tess (`tess`)

reference: art_src/tess.png

Hub NPC (non-combat), the founders' portraits row. The game: Tess Morn, the cell's dead-drop courier; runs the mission
board at the Water-Lock Basin; Vesper's former runner contact; deaf in one ear from an Inquisition flash-bomb.

```text
Character: Tess Morn, Dead-Drop Courier. Stylization warm and readable: large, bright, alert eyes, a small pointed chin, expressive dark brows, a quick lopsided half-smile, a sharp, lively face with soot smudges and a few freckles. A young woman of nineteen, an adult, wiry and compact. Long, thick, wild dark-brown hair: one messy loose braid over her left shoulder, the rest falling free in tangled strands, swept back behind her right ear. Her right ear, the deaf one, on the near side toward the viewer (the left side of the picture), is fully visible and not covered by hair: scarred and notched at the rim, the skin round it and down onto her jaw puckered, shiny and pinkish-pale from an old burn from a flash-bomb, clearly readable at small size. Patched leather shingle-armour of overlapping scraps over greasy wool wraps, a courier's satchel strap across the chest with a rolled message tube.
Mood: quick, wary, determined.
Main colours: boiled leather brown and rotting oak, grey wool wraps, a small tallow amber accent.
```

## Bram (`bram`)

reference: art_src/bram.png

Hub NPC (non-combat), the founders' portraits row. The game: Bram "Iron-Lung" Karr, the smith at the Tinker's Hearth;
a former master foundryman, lungs ruined, mind intact.

```text
Character: Bram "Iron-Lung" Karr, Tinker of the Hearth. Stylization warm and readable: a big square head, a huge barrel chest and wide shoulders, a broad flattened nose, a heavy kind-but-tired brow over deep-set eyes. A man in his thirties who looks older. A wild, shaggy mane of dark-brown hair streaked with grey, worn loose to the shoulders, and a big bushy soot-streaked beard; a thick, scarred throat. An iron jeweller's loupe strapped over his right eye. The iron loupe stays strapped over his right eye in every cell, also when the eyes are squeezed shut. A scorched cowhide apron studded with riveted pouches over a rolled-sleeve work shirt, soot and burn marks on his forearms. Warm forge glow on one side of his face.
Mood: gruff, steady, stubborn.
Main colours: boiled leather and scorched cowhide brown, scavenged iron, tallow amber forge glow.
```

## Sister Aline (`aline`)

reference: art_src/aline.png

Hub NPC (non-combat), the founders' portraits row. The game: Sister Aline Vance, surgeon at the Apothecary's Slab and
the Flesh-Broker voice; a former Anointed Prosector of the Hall of Holy Dissection.

```text
Character: Sister Aline, Surgeon of the Apothecary's Slab. Stylization readable but cool: a long elegant neck, a long narrow face with high sharp cheekbones and hollow cheeks, large cool calculating eyes under straight dark brows, a small firm mouth. A tall, gaunt woman in her mid-twenties. Cropped soot-black hair, cut short and severe with a razor, ragged at the nape, one longer lock falling over her brow; bareheaded, no veil and no wimple. A clerical habit with its holy emblem cut away and defaced, under a stiff leather embalmer's apron stained with vitriol. A bandolier of silver scalpels and bone cannulas across the chest; sulfur-yellow stained fingertips on one hand raised near the strap. Faint early black-violet veins creeping up the side of her neck.
Mood: cold, pragmatic, unflinching.
Main colours: bleached linen and damp basalt habit, boiled leather apron, polished steel scalpels, a hint of blight violet.
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
