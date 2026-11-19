# Opening CGs (prompts, 2026-11-18)

Tushar asked for a pass over the opening CGs (2026-11-18). Vignette `vg_open` (The Red Tithing) shows
`backgrounds/story/intro_1..4.png`; the next scene, `vg_meat_market`, shows `meat_market.png`. All five are the old
code-drawn cards (flat vector shapes, not pixel art, brighter than the locked look; STYLE_BIBLE 6 lists them as
placeholders). Targets stay `1120 x 472` at the same paths (generate 16:9, crop the middle band).

Route (Tushar 2026-11-18, STYLE_BIBLE 6): all five, `meat_market` included, go through image_pipeline (Gemini web
queue) for now, one request each, with the M5 gate CG (`vg_m5_gate.png`, the approved painted look) as the style
reference (`refs/cg_style_ref_m5_gate.png`). They are placeholders: final opening CGs are made in Flow. An earlier
note in this file made intro 1-4 finals and moved `meat_market` to Flow; superseded the same day ("lets use gemini
webapp for now, but note that these will be placeholder"). The Flow version of `meat_market` is kept below.
Slide briefs: `lore/narrative/new_game_onboarding_and_intro.md` 2. Tone rule: the rite and the chains, never violence
to a child.

## story_intro_1

```text
Use the attached image only as the reference for style, palette and rendering, not for its subject. A sunless city under a permanent bruised-violet sky choked with petrified, frozen-looking cloud, no sun, no moon, no stars; pale white ash drifting down everywhere. Below, a sprawling black basalt city of steep slate roofs, chimneys and gothic spires, its windows and streets lit only by small amber tallow lamps; a great cathedral bell tower rises in the right third, its huge iron bell catching amber light from braziers. The sky is dull and dark, the warm light comes from the city below. Wide, quiet, oppressive. Pixel art story illustration for a gothic dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image; the picture is cropped to a wide band, so keep everything that matters in the middle three-fifths of the height, away from the top and bottom. Match the attached image exactly in rendering: painted pixel clusters, low-key soot-gothic palette of basalt black, slate, iron grey, oxblood and rust, amber firelight and tallow glow as the only warm light; light comes from fire, never from the sky. Clear dark outlines, limited palette, light dithering. No text, no caption, no UI, no border, no logo, no rain unless asked.
```

## story_intro_2

```text
Use the attached image only as the reference for style, palette and rendering, not for its subject. A cross-section cut through the earth, like a cutaway diorama: at the top a thin band of black basalt city with tiny amber-lit windows; below it layers of dark rock, and deep under them the enormous calcified skeleton of a sleeping titan curled on its side, ribs, spine and a vast horned skull the size of districts, bone turned to grey stone. Black veins of oozing marrow run from the titan up through the rock to the wells of the city, pulsing with a faint sickly violet glow. Faint amber light from the city above, deep blue-black darkness below. Ominous, still, the titan dreaming. Pixel art story illustration for a gothic dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image; the picture is cropped to a wide band, so keep everything that matters in the middle three-fifths of the height, away from the top and bottom. Match the attached image exactly in rendering: painted pixel clusters, low-key soot-gothic palette of basalt black, slate, iron grey, oxblood and rust, amber firelight and tallow glow as the only warm light; light comes from fire, never from the sky. Clear dark outlines, limited palette, light dithering. No text, no caption, no UI, no border, no logo, no rain unless asked.
```

## story_intro_3

```text
Use the attached image only as the reference for style, palette and rendering, not for its subject. Inside a vast gothic crypt hall of black stone, a procession: a line of young people in their late teens in plain grey shifts, wrists chained to one long iron chain, heads bowed, walking away from the viewer toward three tall arched crypt doorways glowing with cold candle-white light. Flanking them, tall cloaked Clerisy priests in cloth-of-gold copes wearing smooth windowless iron masks with no eye holes, holding tall reliquary staffs; rows of tallow candles on iron stands along the walls give amber light. Solemn, cold, ritual; no violence, no blood, no one hurt. Pixel art story illustration for a gothic dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image; the picture is cropped to a wide band, so keep everything that matters in the middle three-fifths of the height, away from the top and bottom. Match the attached image exactly in rendering: painted pixel clusters, low-key soot-gothic palette of basalt black, slate, iron grey, oxblood and rust, amber firelight and tallow glow as the only warm light; light comes from fire, never from the sky. Clear dark outlines, limited palette, light dithering. No text, no caption, no UI, no border, no logo, no rain unless asked.
```

## story_intro_4

```text
Use the attached image only as the reference for style, palette and rendering, not for its subject. A close, low view of a cobbled gutter at night in light rain: a shattered city-watch insignia lies in the black mud, a cracked iron shield-badge of the Castellan Watch (a kettle helm over crossed billhooks on a rust-crimson field), broken in two pieces, washed with streaks of fresh dark blood that run into the rain water between the cobbles. Behind it, out of focus, the foot of a row of iron cage doors standing open and a single dropped ring of keys. One sputtering tallow lantern off frame gives amber light; everything else is wet slate and shadow. No people. Pixel art story illustration for a gothic dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image; the picture is cropped to a wide band, so keep everything that matters in the middle three-fifths of the height, away from the top and bottom. Match the attached image exactly in rendering: painted pixel clusters, low-key soot-gothic palette of basalt black, slate, iron grey, oxblood and rust, amber firelight and tallow glow as the only warm light; light comes from fire, never from the sky. Clear dark outlines, limited palette, light dithering. No text, no caption, no UI, no border, no logo, no rain unless asked.
```

## story_meat_market (Flow)

Flow project "Emberwake", main prompt box, `@Kaelen` chip for him, the M5 gate CG uploaded as the style guide, 16:9.
The text below, with "one small young man ... halberd in hand" replaced by "`@Kaelen`, small in the middle
distance, only his dull dark-grey scarred breastplate, no shoulder plates, halberd in hand". Crop to 1120 x 472.

```text
Use the attached image only as the reference for style, palette and rendering, not for its subject. The Lower Meat Market at night, a crowded covered market hall of black brick and iron hooks: huge boiling rendering cauldrons glowing orange in the middle, steam rising, carcasses on hooks along the beams. Long silent queues of hollow-eyed grey-skinned adults with grey veins on their faces, in drab wool, holding wooden bowls, waiting at a counter where communion draughts and grey salt are handed out; none of them looks up. In the middle distance, one small young man in a dented dark breastplate over a cream gambeson, a halberd in hand, seen from behind, running through the crowd toward the right. Steam, amber cauldron light, deep shadows. Pixel art story illustration for a gothic dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image; the picture is cropped to a wide band, so keep everything that matters in the middle three-fifths of the height, away from the top and bottom. Match the attached image exactly in rendering: painted pixel clusters, low-key soot-gothic palette of basalt black, slate, iron grey, oxblood and rust, amber firelight and tallow glow as the only warm light; light comes from fire, never from the sky. Clear dark outlines, limited palette, light dithering. No text, no caption, no UI, no border, no logo, no rain unless asked.
```

## Placed (2026-11-18, placeholders)

Takes `story_*_q1795029565` from image_pipeline/output, each cropped to a 1376 x 580 band (rows from the top: intro_1
110, intro_2 8, intro_3 110, intro_4 96 (its own letterbox bars dropped), meat_market 100), resized to 1120 x 472.
`meat_market`: Gemini painted a "communiee draughts" label on the counter; painted out (filled along the wood grain).
Known drift, left for Flow: a few star-like ash specks in intro_1's sky (CANON: no stars), daylight-white windows in
intro_3, Kaelen with shoulder plates in meat_market.
