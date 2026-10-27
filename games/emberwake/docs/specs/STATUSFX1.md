# STATUSFX1: status effects on the board

Status update (2026-10-28): approved; the open questions answered as recommended (user 2026-10-28, decision "WINDUP1 and STATUSFX1 spec answers" in combat.md).

Status: spec (2026-10-28). Approved by the user 2026-10-28 (decision "WINDUP1 answers" 5, `decisions/combat.md`):
a looping effect on a unit's token for each status it carries, on FX1's engine effects kit, starting with the slice's
statuses; Staggered is WINDUP1's `dazed` loop, not an effect; FX1 covers impacts only (FX1.md 7)
Depends on: FX1-E (the effects kit, done), FX1-L1 (`emberwake_gui/src/fx.rs`, the fx clock, the impact queue of rule
I); WINDUP1-L only through the shared `tokens` agent-link array and `battle.rs` (serialise, no logic dependency)
Lane: logic (STATUSFX1-E engine, STATUSFX1-L game); no asset work, no image API
Touches: `engine/crates/aegis_ui/src/{fx,painter,specimens}.rs`, `games/emberwake/crates/emberwake_gui/src/{fx,battle,
stage,tuning}.rs`, `games/emberwake/tuning.json`

## Goal

A glance at the board says who is burning, poisoned, stunned or rooted: stars circle a stunned head, embers rise from
a burning body, an iron ring holds a rooted unit's feet, a fortified unit glows. Drawn in code from the art bible's
hues, presentation only (no rule, plan, AI, save or trace changes), readable under the reduce-motion setting.

## 1. Survey

- Statuses are 17 code rows (`kits/rpg/crates/rpgkit_rules/src/status.rs` `ROWS`); a unit's `StatusState` has
  `active` (in effect this round) and `pending` (a debuff inflicted this round acts next round). The board shows none
  of them; the hover card lists "Now:" and "Next round:" (`battle.rs`).
- The slice uses all 17 (CAST1_GEAR section 4): `stagger`, `stun`, `root`, `slow`, `bleed`, `poison`, `burn`,
  `expose`, `weaken` (debuffs); `taunt`, `haste`, `fortify`, `regen`, `riposte`, `empower`, `steadfast`, `stealth`
  (buffs). Blind, Flammable, mark and frenzy have no row (CAST1_GAPS G17): no look until their row exists.
- FX1-E's primitives (`aegis_ui::fx`: Projectile, Beam, Trail, Sparks, Flash, Ring) are one-shots with `start` and
  `ms` on a live list of at most 32; nothing loops. `Art::body_silhouette` draws a frame's silhouette in one colour.
  `Painter::image` has no alpha.

## 2. Engine (STATUSFX1-E, `aegis_ui`, game-agnostic)

- `fx::Looped { effect: Effect, period: u64 }`: drawn by the caller every frame, never on the live list (so it never
  evicts an impact). `draw(p, now)`: `t = now - effect.start` (nothing before the start); cycle `k = t / period`; the
  inner effect is drawn at local time `t mod period` (nothing once it exceeds the effect's `ms`) with seed
  `hash(effect.seed, k)`, so Sparks vary from cycle to cycle and stay deterministic. `draw_still(p)`: the inner effect
  at `u` 0.5 with cycle 0 (reduce motion). `kind()` as the inner effect's.
- New primitive `Orbit { at, rx, ry, ms, count (1-8), size (1-2), cols (two), phase_deg, star }`: dot `i` at angle
  `360 x (u + i / count) + phase_deg` on the ellipse (rx, ry); a dot in the back half (sin < 0) at alpha 110, the
  front at 255; colour `cols[i % 2]`; `star` draws a 3 px plus instead of a square. `PRIM_KINDS` gains `orbit`.
- `Ring` accepts `r0` above `r1` (a closing ring), if FX1-E refuses it today.
- `Painter::image_alpha(img, r, a: u8)`: as `image`, each pixel's alpha scaled by `a / 255`.
- Effects page (UI Gallery, Kit widgets): a button `kit.fx.orbit` and a toggle `kit.fx.loop` that draws the chosen
  primitive looped (period = its `ms` + 200); `kit.fx.reduce` shows `draw_still`.

Acceptance (engine tests): E1 a looped Sparks draws nothing before the start and between `ms` and `period`, and
different sparks in cycles 0 and 1 (same in two runs); a looped Ring draws the same in every cycle; E2 `draw_still`
equals the inner effect at `u` 0.5; E3 Orbit count 3: dot 0 at angle 0 at `u` 0 and 90 at 0.25, back dots at alpha
110; E4 `image_alpha` with 128 halves every pixel's alpha; E5 the Effects page names.

## 3. The look table (STATUSFX1-L, code in `emberwake_gui/src/fx.rs`, as FX1 FQ1)

Colours (art bible): Gold `#fbbf24`, Linen `#d9cbb4`, Amber `#d97706`, Arterial `#e11d48`, Crimson `#991b1b`, Iron
`#4b5563`, Steel `#9ca3af`, Ash `#6b7280`, Cyan `#06b6d4`, Violet `#9333ea`, Orchid `#c084fc`. Sizes for the Stage
(`token_px` 40); Flat scales px by `flat_token_px / token_px` (at least 1); rings and orbits squash 0.75 on the Stage,
1.0 Flat (FX1 4.1). Anchors: head = the foot point raised `0.75 x token_px` (Flat: the token centre raised 0.45 x its
size); chest = STAGE1's chest point (Flat: the centre); feet = the foot point (Flat: the centre lowered 0.4 x its
size). Every look is a `Looped` effect, except tints and stealth.

| Status | Slot | Look (inner effect) | Period ms | Colours |
| --- | --- | --- | --- | --- |
| `stun` | head | Orbit rx 7, ry 2.5, count 3, size 2, star, ms 900 | 900 | Gold / Linen |
| `taunt` | head | Ring r0 2, r1 7, alpha 200, ms 700 | 1100 | Crimson |
| `burn` | body | Sparks at the chest lowered 4 px, count 4, dir -90, spread 70, speed 25, gravity -20, size 1, ms 500 | 600 | Amber / Gold |
| `poison` | body | Sparks at the chest, count 3, dir -90, spread 50, speed 12, gravity -15, size 1, ms 800 | 1000 | Violet / Orchid (SQ1) |
| `bleed` | body | Sparks at the chest, count 2, dir 90, spread 20, speed 15, gravity 90, size 1, ms 600 | 900 | Arterial / Crimson |
| `expose` | body | Ring r0 9, r1 4 (closing), alpha 180, ms 900 | 1300 | Crimson |
| `riposte` | body | Trail at the chest, radius 9, -40 to 40 deg, width 1, ms 300, mirrored when facing left | 1500 | Steel |
| `root` | feet | Orbit rx 8, ry 3, count 8, size 1, ms 4000 (a ring of links creeping round; SQ2) | 4000 | Iron / Steel |
| `slow` | feet | Ring r0 4, r1 9, alpha 140, ms 1400 | 1400 | Ash |
| `steadfast` | feet | Flash rx 9, ry 3, alpha 80, ms 1200 | 1200 | Gold |
| `haste` | feet | Sparks count 3, dir -90, spread 30, speed 70, gravity 0, size 1, ms 220 | 600 | Cyan / Linen |
| `regen` | feet | Sparks count 2, dir -90, spread 40, speed 18, gravity -10, size 1, ms 900 | 1100 | Gold / Linen |
| `weaken` | tint | the body silhouette, alpha 40 to 70 and back (a sine) | 1600 | Ash |
| `empower` | tint | the body silhouette, alpha 30 to 70 | 900 | Amber |
| `fortify` | tint | the body silhouette, alpha 25 to 50 | 1600 | Cyan |
| `stealth` | alpha | the token (sprite and outline) at 55 % alpha (`image_alpha`) | - | - |
| `stagger` | - | none: WINDUP1's `stagger` reaction and `dazed` loop | - | - |

Feet looks draw on FX1's ground layer, head and body looks on the air layer; tints draw right after the unit's sprite
in the feet pass (FX1 4.1, under FX1's hit flash), with the frame and mirror the token shows.

## 4. Rules (STATUSFX1-L)

- S1 Which: every id in a unit's `active` or `pending` statuses with a row in table 3. A unit that has fallen (from its
  fall at the impact, FX1 rule I), withdrawn or left the grid shows none.
- S2 Several at once: one look per slot (head, body, feet, tint) plus stealth's alpha, so at most three loops and one
  tint per token. A slot with two or more statuses shows them in table order, `status_cycle_ms` each, round and
  round; a switch restarts that look's loop (SQ4).
- S3 Pending (SQ3): a status only in `pending` (it acts next round) draws at `status_pending_alpha` times its alphas;
  at the round start, when it turns active, at full alpha.
- S4 Apply burst: a status id that newly appears on a unit in a step that popped an action gets, at that action's
  impact (FX1 rule I), a one-shot Ring r0 3, r1 10, alpha 200, 220 ms at the chest in its first colour, on the live
  list; several on one unit 40 ms apart. Steps without an action (round start, ticks) burst nothing. `stagger` has no
  burst (WINDUP1 W7).
- S5 Its loop starts at the burst's time (else at once); it follows the token's drawn foot point (the slide) and the
  camera shift (the board passes).
- S6 Draw order on the air layer: the status loops, then FX1's live effects, then the floats (pass 9); WINDUP1's amber
  outline and the selection outlines draw in pass 8 over tints. WINDUP1's poses are the sprite underneath: a Staggered
  and Stunned unit is `dazed` with stars.
- S7 Settings:

| Setting | Loops | Tints | Burst | Slot cycling |
| --- | --- | --- | --- | --- |
| default | run | pulse | yes | yes |
| `anim_speed` s | on the fx clock (held by hit-stop) | fx clock | fx clock | wall ms |
| `skip_anim` | run | pulse | none (FX1: no effects) | yes |
| `reduce_motion` | `draw_still` | the middle alpha, still | yes (effects stay, FX1 FQ4) | yes |

- S8 Enemy tells: statuses are not tells; enemies show them under either Show enemy tells setting (the hover card
  already lists them).
- S9 Agent link: each entry of the battle screen's `state.tokens` (WINDUP1 6.2; whichever task lands first adds the
  array) gains `"status_fx": [ids drawn now, in slot order head, body, feet, tint, alpha]`.
- Tuning (`games/emberwake/tuning.json`, live reload, clamped where read): `status_cycle_ms` 1500 (600-4000),
  `status_pending_alpha` 0.5 (0.2-1.0), `status_tint_max` 70 (0-120: the ceiling on tint alpha).

Acceptance (game tests in `fx.rs` and `battle.rs`, fixed fx clock):

1. Every `status::ROWS` id but `stagger` has a look (a new row without one fails the test).
2. Stun and burn: head and body looks both drawn; burn, poison and bleed: the body slot shows burn, then poison at 1500
   ms, then bleed at 3000, then burn.
3. A poison only in `pending` draws at half alpha; after the round start at full.
4. Caustic Vomit's poison: the burst at the action's impact time, not at the pop; none under `skip_anim`.
5. `reduce_motion`: loops drawn still, tints at their middle alpha, bursts still drawn.
6. A unit that falls draws no looks from its fall; a stealthed unit is drawn at 55 % alpha with its outline.
7. Stage and Flat anchors as section 3; feet looks on the ground layer.
8. `state.tokens[i].status_fx` matches the drawn looks.

## 5. Costs and task split

| Id | Title | Layer | Lane | Depends on | Touches | Size |
| --- | --- | --- | --- | --- | --- | --- |
| STATUSFX1-E | `fx::Looped`, the `Orbit` primitive, a closing `Ring`, `Painter::image_alpha`, Effects page buttons | engine | logic | FX1-E | `engine/crates/aegis_ui/src/{fx,painter,specimens}.rs`, its tests | S (0.5 session) |
| STATUSFX1-L | Status looks on battle tokens: the table, slots and cycling, pending alpha, bursts, tints, stealth, settings, agent link | game | logic | STATUSFX1-E, FX1-L1; after WINDUP1-L when both are queued (`battle.rs`) | `emberwake_gui/src/{fx,battle,stage,tuning}.rs`, `games/emberwake/tuning.json` | M (1 session) |

No rig, data or image-API work. Parity: none retires (the GUI is outside every parity suite).

## 6. Out of scope

Status icons on cards, the strip and the hover (ART2 icons, ICONUI1), the Staggered look (WINDUP1), ground hazards on
tiles (HOOKS1), Inoculated miasma (Act 2), statuses on the region map, status sounds, looks for the G17 statuses until
their rows exist, a setting to hide the looks.

## 7. Open questions (recommended answer first; the planner takes it while the user is away)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| SQ1 | Poison's colour: the art bible's 16 hues have no green | (a) Blight Violet with Orchid (the Ash-Crawler's acid is hue 14, CAST1_ASSETS N2); (b) Orchid alone; (c) a green hue added to the art bible | (a) for the slice: the only poison source is the crawler's violet acid; revisit when Act 2's Inoculated violet arrives |
| SQ2 | Root's look: the user's example was vines; the slice's roots are Shackle Drag and the Render-Beast's jaws, and there is no green | (a) a ring of iron links at the feet; (b) brown vines in Rotting Oak | (a): it matches the slice's chains and reads as held fast |
| SQ3 | Debuffs inflicted this round act next round (`pending`) | (a) the look at half alpha from the hit, full when active; (b) only when active; (c) full from the hit | (a): the hit shows something landed, and a half-strength look does not claim a stunned unit cannot act this round |
| SQ4 | More statuses than slots | (a) one look per slot, cycling every 1.5 s; (b) the top one per slot and a "+n" pip | (a): every status shows within a few seconds without extra marks; the hover card lists them all |
| SQ5 | Reduce motion | (a) still looks, cycling kept; (b) hide the looks | (a): the looks carry information, as FX1 FQ4 kept the effects |
