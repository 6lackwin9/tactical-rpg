# Session U4: passives, sources, affiliation and temperament

MASTERPLAN1 sub-block U4. Interactive design session with the user, 2026-11-07. Status: design closed (user 2026-11-07).

## Decided (user 2026-11-07, all defaults approved)

- **Collapse the 13 ad-hoc passive effect types into 4 unified primitives.** Today `passives.json` has 13 separate effect schemas (`stat`, `resist`, `skill_potency`, `status_potency`, `stagger_taken`, `ap_regen`, `proficiency`, `unlock`, `foresight`, `forced_move`, `meter`, `skill`, `trigger`). Under `U1`, 9 of those 13 are simply standing or conditional **`U1` Modifiers**. Every passive entry now uses one or more of 4 clean grant kinds:
  1. **`modifier`**: Emits a standard `U1` modifier (`target`: any stat, Pool max/regen, Rating incl. `foresight`, or 4-stage pipeline stage for damage/heal/poise/forced-move; `op`: `flat | add_pct | mult`; optional `condition`).
  2. **`proficiency`**: Adds `+N` proficiency level in a gear family (used by drills and class/character training).
  3. **`grant_skill`**: Unlocks a skill or skill family (or lowers unlock level threshold, covering `unlock_early` moved from `U1` and `OFFHAND1` item skills from `U3`).
  4. **`trigger`**: Reactive combat hook (`when` event + `condition` -> `effect`, sharing the unified trigger/status application vocabulary with `A4` and `S1`).
- **Four passive sources, deduplicated (`C04`).** A unit gathers passive IDs from four sources in order—**Class/Discipline**, **Character (drills + temperament)**, **Equipped Items (`U3`)**, and **Affiliation (Faction `grants`)**—applying each unique passive ID at most once.
- **Affiliation and background (`MASTERPLAN1` Q2).**
  - **Background**: Text and lore/dialogue tag only; no combat rules.
  - **Affiliation (Faction)**: Keeps its two existing hooks—faction `grants` (a passive source in `C04`) and reputation bands (read by dialogue and shop in `T1`/`E2`).
- **`ROSTER1-T` (Temperaments) folded into `U4` as a character passive + AI preference.** Instead of a separate `temperaments.json` subsystem, a character's temperament is a standard `U4` passive ID on the character record (combining any of the 4 grant kinds above, such as a conditional `U1` modifier when Wounded) plus an optional `default_intent` / `ai_role` override tag read by `F3`/`I1`.

## Fixed vs parameter (`U4`)

| Item | Fixed (kit behaviour) | Parameter (data), with default |
| --- | --- | --- |
| Passive grant kinds | Four kinds: `modifier` (`U1`), `proficiency` (`U3`), `grant_skill` (`U2`/`A1`), `trigger` (`A4`/`S1`) | Passive rows in `passives.json` |
| Passive sources (`C04`) | Collected from Class/Discipline, Character, Equipped Items, and Faction `grants`; deduplicated by passive ID | Which passives each class, character, item, or faction grants |
| Compatibility loader | Existing 13-type `passives.json` rows map cleanly into the 4 grant kinds so `timeline_kit.json` and shipped rows keep working during migration | - |
| Affiliation & background | Faction `grants` feed `C04`; rep bands feed `T1`/`E2`; background is text/tag only | Faction rows and character background strings |
| Temperaments (`ROSTER1-T`) | Character field naming a passive ID + optional `default_intent` / `ai_role` override | Temperament passive rows and character assignments |

## Acceptance test for `U4.2`

1. All 55 shipped passives load and evaluate through the 4 unified grant kinds (`modifier`, `proficiency`, `grant_skill`, `trigger`) with existing combat and rules goldens byte-identical before new content tweaks.
2. A character with a drill passive (`proficiency`), an off-hand skill item (`grant_skill`), a faction grant (`modifier`), and a temperament (`modifier` + `default_intent`) builds through the single `C05` unit builder (`FB-12`) and enters battle with all four sources active and deduplicated.
