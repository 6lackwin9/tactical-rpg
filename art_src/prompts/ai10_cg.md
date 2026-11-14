# AI-10: Act 1 M5 Slag Gate departure CG (prompts)

Art/UI batch 2026-11-12, inbox row AI-10. Target `games/emberwake/assets/backgrounds/story/vg_m5_gate.png`
at **1120 x 472** (the story vignette picture, 2x the 560 x 236 picture area; `hearth.png` has the same size), replacing
the `vn_hearth_stage.png` placeholder used by vignette `vg_m5_gate` ("The Winch"). Generate at 16:9 and crop to
1120 x 472 around the middle band, so keep everything that matters away from the top and bottom fifths.

Story rule: any of the four squad founders can be the one who stays at the winch (`{member:sunk_ward.crucible_stayed}`),
so **no founder is identifiable**: the stayer is a dark silhouette seen from behind, the ones escaping are small,
low and turned away. Inquisitor-Prelate Nemeth is the only face, and he is far and small.

## Flow route (primary, story CGs from Flow, `ART_UI_STUDIO.md` 3.7)

No `@` character is called, on purpose (see the story rule). Paste as the scene prompt:

```text
Pixel art story illustration for a gothic dark fantasy tactical RPG, detailed 16-bit SNES/GBA-era style, 16:9 wide image, all key action kept in the middle band of the frame. Night at the Crucible Watchpost, a soot-black iron fortress gate in a sunless city. A colossal black-iron portcullis, the Slag Gate, has ground up only a forearm's height off the ground, glowing slag runoff spilling orange under it. Three small figures in scavenged leather and iron throw themselves flat and roll under the gap into falling ash, seen from behind and low, faces hidden. On a raised iron platform to the left, one lone figure seen from behind, a dark silhouette with no face, strains with both arms against the spokes of a huge chain winch, the chain screaming taut up into the dark. High on a balcony to the right, small and far, a tall gaunt man with ash-grey skin and long white hair in a crimson-and-steel gorget watches, a small violet phial glowing at his throat. Watch torches give hard red-orange light, the slag gives orange glow from below, a cold violet light from the phial; deep blue-black shadows, ash drifting everywhere, sparks off the chain. Clear dark outlines, hand-placed pixel clusters, limited palette, light dithering. No text, no UI, no border, no logo.
```

## Gemini route (fallback, the [Story scene](../../games/emberwake/docs/specs/ART_PROMPT_TEMPLATES.md#story-scene) shape)

Same text as above with `games/emberwake/assets/portraits/bosses/inquisitor_prelate_nemeth.png` attached, adding
this line at the end: `Nemeth is the man in the attached image (same face, hair and gorget), small and far on the balcony.`
