# Seeds

Idempotent `ContentCatalog.upsert/remove` scripts that author game data. Rule: a seed leaves this folder in the cleanup pass after its task is merged (git history keeps it).

## Archived seeds (CLN-2)

Restore with `git show <hash>:data/seeds/<file>`.

| Seed | Task | Merge commit |
| --- | --- | --- |
| `c1a1.gd` | C1-A1 (skill gating, priest proficiency, tome retype) | `11ffe67` |
| `m2c2b_a.gd` | M2C2B (class skill families) | `b7c4016` |
| `m2c2b_b.gd` | M2C2B | `b7c4016` |
| `m2c2b_c.gd` | M2C2B | `b7c4016` |
| `m2c2b_d.gd` | M2C2B | `b7c4016` |
| `m2c2b_lib.gd` | M2C2B (shared helpers for m2c2b_a..d) | `b7c4016` |
| `sk2.gd` | SK2-A (data step) | `9686a36` |
| `sk2_schema.gd` | SK2-L (schema step) | `7bed659` |
| `skill1_ranks.gd` | SKILL1 (ranked skills v2 migration) | `745962a` |
| `item1_migrate.gd` | ITEM1-A (item families, items v2 migration) | pending merge |
| `item1_tokens.gd` | ITEM1-A (number-quoting item descriptions to tokens) | pending merge |
| `class1_migrate.gd` | CLASS1-M (class skill_path list to dict, units heal_below default) | pending merge |

Example: `git show 11ffe67:data/seeds/c1a1.gd`

## Active seeds

| Seed | Task |
| --- | --- |
| `m4a.gd` | M4-A (quest templates, Ashlands loot, linked quests) |
| `exp1a.gd` | EXP1-A (map templates, Ashlands exploration data) |
| `dlg1a.gd` | DLG1-A (service lines, Ashlands dialogues, keep shop, gold) |
| `s14_a1.gd` | S14-A1 (first manifest rows: goblin_brawler alias, four sample file overrides; goblin_brawler art key) |
