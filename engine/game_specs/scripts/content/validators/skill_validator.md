---
class: SkillValidator
extends: RefCounted
description: "Skill validation rules (moved from ContentCatalog, CLN-9): family rules, row rules and soft warnings. ContentCatalog.validate_entry / warnings_for dispatch here; the ranked-skill API stays in ContentCatalog. One problem of a skill row: {\"field\", \"rank\", \"message\"}."
---

# Properties
* `errors`: Array[String] = []
* `id`: String = str(e.get("id", ""))
* `family`: String = str(e.get("family", ""))
* `head`: String = id if family.is_empty() else family
* `tier`: int = DictUtil.to_int(e.get("tier", 1), 1)
* `basic`: bool = DictUtil.to_bool(e.get("basic", false))
* `system`: bool = DictUtil.to_bool(e.get("system", false))
* `kind`: String = str(e.get("kind", ""))
* `tiers`: Array[int] = [tier]
* `other_id`: String = str(other.get("id", ""))
* `other_tier`: int = DictUtil.to_int(other.get("tier", 1), 1)
* `key`: String = str(f["key"])
* `errors`: Array[String] = family_errors(e)
* `template`: String = str(e.get("template", ""))
* `rows`: Array[Dictionary] = [e]
* `folded`: Dictionary = SkillRanks.fold(str(e.get("id", "")), rows)
* `fold_errors`: Array = folded["errors"]
* `out`: Array[Dictionary] = []
* `kind`: String = str(e["kind"])
* `id`: String = str(e["id"])
* `tier`: int = DictUtil.to_int(e.get("tier", 1), 1)
* `basic`: bool = DictUtil.to_bool(e.get("basic", false))
* `system`: bool = DictUtil.to_bool(e.get("system", false))
* `grade`: String = str(e["grade"])
* `unlock_offset`: int = DictUtil.to_int(e.get("unlock_offset", -1), -1)
* `requires`: Array[String] = ContentFactory._strings(e.get("requires", []))
* `head`: String = SkillRanks.head_of(e)
* `target`: Dictionary = ContentCatalog.get_entry("skills", required)
* `guarded`: Array[String] = SkillKinds.sectioned_fields()
* `f`: Dictionary = ContentSchema.field("skills", key)
* `status_id`: String = str(e["status_effect"])
* `magnitude`: int = DictUtil.to_int(e["status_magnitude"])
* `magnitude_status`: String = SkillText.magnitude_status(e)
* `mag_min`: int = StatusRegistry.magnitude_min(magnitude_status)
* `mag_max`: int = StatusRegistry.magnitude_max(magnitude_status)
* `grant`: String = str(e["grant_status"])
* `reach_min`: int = DictUtil.to_int(e["reach_min"])
* `reach_max`: int = DictUtil.to_int(e["reach_max"])
* `heal_stat`: String = str(e["heal_stat"])
* `heal_mult`: float = DictUtil.to_float(e["heal_mult"])
* `heal_base`: int = DictUtil.to_int(e["heal_base"])
* `cleanse`: Array[String] = ContentFactory._strings(e["cleanse_statuses"])
* `defend_options`: Array[String] = StatusEffects.defend_buffs()
* `area`: String = str(e["area"])
* `dash`: int = DictUtil.to_int(e["dash_tiles"])
* `hand`: String = str(e["weapon_hand"])
* `types`: Array[String] = ContentFactory._strings(e["weapon_types"])
* `EXPLOIT_MAX_ENTRIES`: int = 2 (const)
* `EXPLOIT_MULT_MIN`: float = 1.05 (const)
* `EXPLOIT_MULT_MAX`: float = 2.0 (const)
* `out`: Array[Dictionary] = []
* `ends`: String = str(e.get("ends_in_stance", ""))
* `own`: CombatTypes.StanceType = ContentFactory.skill(e).stance()
* `zero_power`: bool = DictUtil.to_float(e.get("power_mult", 1.0), 1.0) <= 0.0
* `exploit`: Array = DictUtil.array_of(e.get("exploit", []))
* `seen`: Array[String] = []
* `debuffs`: Array[String] = StatusEffects.debuffs()
* `row`: Dictionary = DictUtil.dict_of(raw)
* `status`: String = str(row.get("status", ""))
* `mult`: float = DictUtil.to_float(row.get("power_mult", 0.0))
* `out`: Array[String] = []
* `ranked`: bool = not raw.has("tier") and not raw.has("family")
* `base`: Dictionary = SkillRanks.effective(raw, 1)
* `mods`: Array[Dictionary] = SkillRanks.modifiers_of(raw)

# Methods

### `static problem(field: String, rank: int, message: String) -> Dictionary`
```logic
return {"field": field, "rank": rank, "message": message}
```

