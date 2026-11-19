# UI frames (Emberwake)

Nine-slice frame art for the game's panels, buttons and cards. Each `<name>.png` has a sidecar `<name>.json` with its
`slice` margins (left, top, right, bottom, in source pixels) and the `scale` it draws at; the large ornate frames add
corner and sampling hints. Owner: Art/UI. Style lock 2026-11-17 (F2 frame), frame tiers 2026-11-18: main screens keep
the ornate frame, popups and the HUD use frame A (Tushar: "Main current pop-up A").

| Frame | Size | Slice, scale | Use |
| --- | --- | --- | --- |
| `ornate.png` | 432 x 432 | 200, x8 | Main panels and screens: Character, Journal, board, shop, results, save slots (`Ui::frame_in("ornate")`) |
| `ornate_full.png` | 512 x 512 | 224, x8, edges tiled | Settings screen and its popups (the mock's full ornate frame) |
| `panel.png` | 24 x 24 | 8, x2 | Frame A for popups and cards (talk box, bunk card, tooltips that need a frame): 1 px bronze `#74543c` outer line, a dark gap, 1 px iron `#4b5563` inner line, an oxblood `#991b1b` top line, no corners. Replaced the old iron line with corner rivets (2026-11-19, `PR7-36`, `docs/HISTORY.md`) |
| `hud.png` | 24 x 24 | 8, x2 | Heads-up panels (the `hud` frame role, `PR7-15`): frame A with the oxblood top line (AI-22) |
| `card.png`, `card_hot.png`, `card_on.png` | 16 x 16 | 4, x2 | List cards: normal, hovered, selected |
| `button.png`, `button_hot.png`, `button_down.png`, `button_off.png` | 16 x 16 | 4, x2 | Buttons: normal, hovered, pressed, disabled |
| `tab.png`, `tab_on.png` | 16 x 16 | 4, x2 | Tabs: normal, active |
| `inset.png` | 16 x 16 | 4, x2 | Sunken wells (slots, fields) |
| `tip.png` | 16 x 16 | 4, x2 | Plain tooltips |
| `parchment.png` | 16 x 16 | 4, x2, `ink` | Parchment notes; text on it uses the theme's `ink` colour |

`panel.png` is a placeholder made by `tools/gen_art.py` (`fr_double_line`) until Art/UI replaces it in place. The battle
screen's frame rules are in `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` section 1.
