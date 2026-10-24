# Combat & Timeline Architecture: The 6-Tick Tactical Engine

> **Category:** Gameplay Systems & Narrative-Combat Parity  
> **Reference File:** [master_specification.md §5, §9](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)

---

## 1. The 6-Tick Simultaneous Timeline

Combat in *Emberwake* discards passive I-Go-You-Go turn structures. Instead, every round resolves simultaneously across a unified **6-Tick Timeline Track**.

At the start of each round, all combatants (player operatives and enemies) schedule their maneuvers. Once both sides commit, the round unrolls in real-time, executing actions tick-by-tick.

```text
TIMELINE TRACK (1 ROUND = 6 TICKS):
[ TICK 1 ] ===> [ TICK 2 ] ===> [ TICK 3 ] ===> [ TICK 4 ] ===> [ TICK 5 ] ===> [ TICK 6 ]
  Windup /        Fast Snap       Advance /       Standard        Heavy           Recovery /
  Reposition      Impacts         Telegraphs      Impacts         Impacts         Stance Reset
```

---

## 2. Maneuver Blocks (Action Chunking)

Skills and attacks are not scheduled individually per micro-tick. Instead, players issue **Maneuver Blocks** containing three contiguous phases:

1. **Windup (Preparation):** The unit raises their weapon, chants a heretical incantation, or draws a bowstring. During windup:
   - Stagger resistance is halved.
   - The unit is acutely vulnerable to interrupts.
   - Ground or entity danger decals are telegraphed to the opponent.
2. **Impact (Resolution):** Damage, armor shredding, knockback, and status conditions calculate instantaneously at a designated tick.
3. **Recovery (Reset / Guard):** After delivery, the unit recovers balance. Overextended attacks leave the unit locked in recovery; efficient attacks convert unused ticks into **Active Guard** or **Strain Venting**.

---

## 3. Standardized Speed Tiers

To keep tactical forecasting legible and predictable, all maneuvers adhere to three fixed timeline tiers:

| Speed Tier | Windup Window | Impact Tick | Recovery Window | Primary Tactical Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Snap (Fast)** | None / T1 | **Tick 2** | Ticks 3–4 | Precision interrupts, throwing ash/smoke flasks, fast parries, quick daggers, evasive steps. |
| **Standard (Mid)** | Ticks 1–2 | **Tick 4** | Ticks 5–6 | Broadsword swings, polearm thrusts, arbalest shots, field bandages, defensive bracing. |
| **Heavy (Slow)** | Ticks 1–4 | **Tick 5 or 6** | Tick 6 / Recoil | Greatsword cleaves, channeled aberrant blights, gate-breaking hammer blows. Tremendous damage, but easily interrupted. |

---

## 4. Movement Integration: "Lead or Follow"

Movement is fluidly bound into Maneuver Blocks rather than being restricted to an arbitrary pre-action phase:

* **Advance (Move-Then-Act):** Operative spends Ticks 1–2 stepping 1–2 tiles into range, then immediately begins a Standard windup hitting on Tick 4.
* **Disengage (Act-Then-Move):** Operative strikes with a Snap attack on Tick 2, then retreats 1–2 tiles during Ticks 3–4 to evade slower enemy retaliation.
* **Auto-Chase Resolution Rules:**
  - *Reach 1 (Brawlers / Daggers):* **Tenacious Chase.** If the designated target retreats 1 tile during the attacker's windup, the attacker automatically steps forward 1 tile to maintain contact.
  - *Reach 2 (Spears / Halberds):* **Range Hold.** The attacker holds their ground; if the retreating target remains within Reach 2, the strike lands without stepping forward.
  - *Ranged (Arbalests / Slings):* Rotates facing arc toward the target automatically; fails only if the target breaks line-of-sight behind complete solid cover before the impact tick.

---

## 5. The Stagger Bar & The Interrupt Rule

Interrupting enemy windups is the core mechanical lever of tactical mastery in *Emberwake*.

```text
WINDUP PHASE (Ticks 1–2)  ---[ BLUNT HIT FILLS STAGGER BAR ]--->  [ TIMELINE SHATTERS ]
                                                                       |
                                                                       v
                                                           [ ACTION REPLACED BY STAGGER BLOCK ]
                                                           [ UNIT DROPS GUARD & LOSES TURN ]
```

### 5.1 Mechanics of Interruption
1. Every combatant has a **Stagger Bar** derived from physical Poise, armor weight, and current combat Stance.
2. Blunt attacks (maces, poleaxe pommels, heavy shields, kick skills) deal high Stagger damage.
3. If an attacker receives enough Stagger damage during their **Windup Ticks** to cap their Stagger Bar:
   - Their queued action **collapses immediately**.
   - Their remaining active ticks on the timeline shatter and convert into an amber-striped **Stagger Block**.
   - The unit loses all guard, active evasion drops to 0%, and they forfeit any further action for the remainder of the round.

---

## 6. UI/UX & Visual Communication

Legibility of the simultaneous timeline is paramount to prevent chaos and ensure deep tactical deliberation.

### 6.1 Trajectory & Threat Decals
* **Tick-Beaded Paths:** Movement vectors across the grid appear as segmented, glowing chevrons. Each segment denotes **1 tick of movement**, marked with micro-labels (`T1`, `T2`, etc.).
* **Ground-Locked Decals (Heavy / Area):** Inscribed, glowing rune borders etched onto the floor grid showing the danger zone, adorned with an Impact Badge (e.g., `[ ⚠ T4 ]`). Units avoid damage simply by stepping off the tile before Tick 4.
* **Entity-Locked Decals (Single-Target / Fast):** A pulsing reticle anchored beneath the targeted unit, tethered by a curved targeting line back to the attacker. Mitigated via fast parries, active guards, or pre-emptive interrupts.

### 6.2 Timeline Scrubbing & Ghost Projections
* **The Master Timeline Bar:** A clean, horizontal Gantt chart spanning the top of the combat screen showing Ticks 1 through 6.
* **Scrubbing Controls:**
  - *Mouse:* Dragging the timeline playhead or scrolling the mouse wheel scrubs the simulation frame-by-frame.
  - *Keyboard:* Number keys `1`–`6` jump directly to specific ticks; `,` and `.` step tick-by-tick; `Spacebar` runs a continuous real-time preview dry run.
* **Ghost Projections:** As the player scrubs to a tick, translucent "ghost" silhouettes of all units populate the grid on their exact predicted tiles, striking their corresponding windup, impact, or recovery poses.

### 6.3 Tactile Combat Feedback
* **Timeline Shatter:** When an interrupt lands, the victim's timeline bar physically cracks with a jagged red strike-through.
* **Hit-Stop Micro-Pause:** Any successful interrupt or critical hit triggers a **30–60ms hit-stop freeze** accompanied by a heavy resonant anvil chime.
* **Decal Dissolution:** The moment an attacker is staggered, their ground hazard decals and targeting tethers instantly dissolve into dispersing black ash.
