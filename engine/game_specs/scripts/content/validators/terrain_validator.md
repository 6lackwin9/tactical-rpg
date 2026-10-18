---
class: TerrainValidator
extends: RefCounted
description: "Validation of the catalog kind `terrain` (S12T-L). entry_errors checks one normalised row (exact messages from docs/specs/S12T_L.md); missing_ids lists the Terrain types without a row (validate_all: terrain:missing once terrain.json exists, warnings_all before that, when the code defaults apply)."
---

# Properties
* `MOVE_COST_MIN`: int = 1 (const)
* `MOVE_COST_MAX`: int = 3 (const)
* `COVER_MAX`: float = 0.5 (const)
* `POTENCY_MIN`: int = 1 (const)
* `POTENCY_MAX`: int = 100 (const)
* `ROUNDS_MIN`: int = 1 (const)
* `ROUNDS_MAX`: int = 5 (const)
* `errors`: Array[String] = []
* `id`: String = str(e.get("id", ""))
* `walkable`: bool = Terrain.is_walkable(Terrain.from_id(id))
* `cost`: int = DictUtil.to_int(e.get("move_cost", 1), 1)
* `cover`: float = DictUtil.to_float(e.get("cover_guard", 0.0))
* `status`: String = str(e.get("hazard_status", ""))
* `potency`: int = DictUtil.to_int(e.get("hazard_potency", 100), 100)
* `magnitude`: int = DictUtil.to_int(e.get("hazard_magnitude", 0))
* `mag_min`: int = StatusRegistry.magnitude_min(status)
* `mag_max`: int = StatusRegistry.magnitude_max(status)
* `rounds`: int = DictUtil.to_int(e.get("hazard_rounds", 1), 1)
* `have`: Dictionary = {}
* `out`: Array[String] = []

