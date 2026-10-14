# Decision history: tried, changed, deprecated (append-only)

Never delete or rewrite entries. When something is replaced, add a row with the date, what changed and WHY. Agents must read this before proposing ideas and must not re-propose a deprecated concept unless the original reason no longer holds (say so explicitly).

| Date | Concept | Status | Replaced by | Why |
|---|---|---|---|---|
| before 2026-10 | Hand-scripted combat layer (10-block rounds, scripted commands) | deprecated | Planner + resolver, 6 blocks, player-planned sequences | Scripted battles could not express planning, AP and AI; double maintenance |
| before 2026-10 | Hard-coded classes, items, skills, units in code | deprecated | JSON catalog + Content Forge | Content needed to be editable, validated and testable without code |
| before 2026-10 | Fixed-power fixed-HP goblins (authored 7-11 power, 30-55 HP) beside derived-stat heroes | deprecated | All combat units class-derived with a tier | Two incompatible battle scales; goblins could not level or share class rules |
| 2026-10 | Unequal class stat totals (60-70) | deprecated | Equal budget 62, growth 5/level (StatRules) | Class balance was not comparable; difference hid in totals |
| 2026-10 | Fixed 1.5 Unit Power scaling for all gear | deprecated | Per-class per-gear Proficiency scaling (1.0-2.0) | Needed an editable "how well does this class wield this" lever |
| 2026-10 | Gear proficiency as a plain allowed list | deprecated | Proficiency dict `{type: {level, scaling}}` | Lists carried no strength; levels now add power, discount, crit, burden, block cost, skill gates |
| 2026-10 | MasterySystem (weapon and lore mastery on the unit/class) | deprecated | Proficiency (class) + Passives (granted by sources) | Mastery was coupled to units; lore/quests/factions should grant traits, not the class |
| 2026-10 | Allocating level-up points freely (1 per level, no base pool) | deprecated | Spend pool 4 + (level-1), spent exactly | Gave units of one class an authorable variance with an enforced total |
| 2026-10 | Enemy tier defined per world | not adopted | One base curve (class + tier + AI); worlds set level and tuning | A goblin must behave the same across worlds; lore is a layer |
| 2026-10 | Ally "vacated tile" resolved in planning order | deprecated | Evaluated fastest-first | Speed gap between classes made planning order wrong |
| 2026-10 | Tier power scaling applied to authored `power` | deprecated | Authored power is absolute | Double scaling broke world tuning deltas |

## Lessons (avoid repeating)
- Hand-edited JSON and ad-hoc string patch scripts corrupted files; use `ContentCatalog.upsert` and the Edit tool (skill `patch-and-seed`).
- Test expectations hard-coded to old numbers broke on every data change; derive them from rules.
