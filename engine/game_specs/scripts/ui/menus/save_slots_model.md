---
class: SaveSlotsModel
extends: RefCounted
description: "Tactical RPG - Save / load slots model (UI1-L): one row per slot 0..5 from SaveGame.peek (0 = autosave, 1-5 manual; never written by a manual save). Load enters the game (warnings shown once, rule 6); save asks before overwriting and is refused during a battle, conversation or shop (rule 5); delete always asks. Answers to a question land in `last_errors`. UIR-2: overwrite / delete questions name what is lost (`describe`) and focus No; loading over a game in progress with unsaved changes asks first."
---

# Properties
* `BUSY`: String = "You can't save during a battle, a conversation or while a shop is open." (const)
* `NO_GAME`: String = "There is no game to save." (const)
* `MANUAL_ONLY`: String = "Manual saves use slots 1-5 (slot 0 is the autosave)." (const)
* `router`: MenuRouter = null
* `saving`: bool = false            # true = the hub's Save screen, false = Load
* `last_errors`: Array[String] = [] # problems of the last action (also after a confirmed question)
* `out`: Array[int] = [SaveGame.AUTOSAVE_SLOT]
* `out`: Array[Dictionary] = []
* `head`: Dictionary = SaveGame.peek(slot)
* `exists`: bool = DictUtil.to_bool(head["exists"])
* `ok`: bool = DictUtil.to_bool(head["ok"])
* `status`: String = "ok" if ok else (str(head["error"]) if exists else "empty")
* `party`: PackedStringArray = PackedStringArray()
* `m`: Dictionary = DictUtil.dict_of(raw)
* `unit`: Dictionary = ContentCatalog.get_entry("units", str(m["base_unit"]))
* `world_id`: String = str(head["world_id"])
* `at`: int = DictUtil.to_int(head["saved_at"])
* `res`: Dictionary = SaveGame.read(slot)
* `errors`: Array[String] = []
* `gs`: GameState = GameState.of(res["state"])
* `warnings`: Array[String] = []
* `what`: String = describe(slot)
* `what`: String = describe(slot)
* `what`: String = describe(slot)

# Methods

### `_init(p_router: MenuRouter, p_saving: bool = false) -> void`
```logic
router = p_router
saving = p_saving
```

### `static describe(slot: int) -> String`
```logic
var head: Dictionary = SaveGame.peek(slot)
if not DictUtil.to_bool(head["ok"]):
	return ""
var world_id: String = str(head["world_id"])
var parts: PackedStringArray = PackedStringArray([str(WorldCatalog.manifest(world_id).get("name", world_id))])
var where: String = location_name(world_id, str(head["location_id"]))
if not where.is_empty():
	parts.append(where)
var at: int = DictUtil.to_int(head["saved_at"])
if at > 0:
	parts.append(Time.get_datetime_string_from_unix_time(at, true).left(16))
return ", ".join(parts)
```

### `static location_name(world_id: String, location_id: String) -> String`
```logic
if location_id.is_empty():
	return ""
var hit: Dictionary = WorldCatalog.resolve(world_id, "locations", location_id) if WorldCatalog.has_world(world_id) else {}
return str(hit.get("name", location_id))
```

### `_load_now(slot: int) -> void`
```logic
last_errors = load_slot(router, slot)
```

### `_save_now(slot: int) -> void`
```logic
last_errors = SaveGame.write(CurrentGame.state(), slot)
if last_errors.is_empty():
	CurrentGame.mark_saved(slot)
	router.changed.emit()
```

### `static export_name(slot: int, world_id: String, saved_at: int) -> String`
```logic
var stamp: String = Time.get_datetime_string_from_unix_time(saved_at, false).replace("-", "").replace(":", "").replace("T", "-").left(13) if saved_at > 0 else "save"
return "%s_slot%d_%s.json" % [world_id if not world_id.is_empty() else "save", slot, stamp]
```

### `_import_now(path: String, slot: int) -> void`
```logic
var res: Dictionary = SaveGame.import_from(path, slot)
last_errors.assign(DictUtil.array_of(res["errors"]))
var warnings: Array[String] = []
warnings.assign(DictUtil.array_of(res["warnings"]))
if last_errors.is_empty() and not warnings.is_empty():
	router.notice.emit(warnings)
router.changed.emit()
```

### `_delete_now(slot: int) -> void`
```logic
last_errors = []
if not SaveGame.delete(slot):
	last_errors.append("Slot %d could not be deleted." % slot)
router.changed.emit()
```

