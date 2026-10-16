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

Example: `git show 11ffe67:data/seeds/c1a1.gd`
