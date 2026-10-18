---
class: EncounterObjectives
extends: RefCounted
description: "Tactical RPG - Encounter Objectives (OBJ1, spec docs/specs/OBJ1.md) What wins an encounter (rout, hold, escape, slay), whom the party must keep alive (guests, protect) and how both sides start (deployment). Fields on catalog and world encounters: objective         enum    rout | hold | escape | slay                   (default rout) objective_tiles   json    [[x, y], ...]; hold only; [] = the ring RING  (default []) objective_rounds  int     1..max_rounds; hold only                      (default 3) objective_unit    String  slay: an id in `enemies`                      (default \"\") objective_count   int     escape: 1..4 heroes out by an exit            (default 1) objective_text    String  0..80 chars; \"\" = the generated text (text_of) (default \"\") deployment        enum    standard | ambush | centre                    (default standard) guests            refs    party-side extras, at most MAX_GUESTS, no repeats (default []) protect           refs    each also in guests                           (default []) Static part: the shared validator `problems` (both catalogs) and the generated objective text. Runtime part (one instance per battle, built by CatalogEncounter.build()): the parsed objective, `held` (hold rounds counted, kept in every BlockSnapshot) and `verdict` (spec section 2, rules 3 to 7)."
---

# Properties
* `ROUT`: String = "rout" (const)
* `HOLD`: String = "hold" (const)
* `ESCAPE`: String = "escape" (const)
* `SLAY`: String = "slay" (const)
* `OBJECTIVES`: Array[String] = [ROUT, HOLD, ESCAPE, SLAY] (const)
* `STANDARD`: String = "standard" (const)
* `AMBUSH`: String = "ambush" (const)
* `CENTRE`: String = "centre" (const)
* `DEPLOYMENTS`: Array[String] = [STANDARD, AMBUSH, CENTRE] (const)
* `RING`: Array[Vector2i] = [Vector2i(5, 4), Vector2i(6, 4)] (const)
* `MAX_GUESTS`: int = 2 (const)
* `MAX_TEXT`: int = 80 (const)
* `COUNT_MIN`: int = 1 (const)
* `COUNT_MAX`: int = 4 (const)
* `DEFAULT_ROUNDS`: int = 3 (const)
* `WON`: String = "won" (const)
* `LOST`: String = "lost" (const)
* `FLED`: String = "fled" (const)
* `objective`: String = ROUT
* `tiles`: Array[Vector2i] = []
* `rounds`: int = DEFAULT_ROUNDS
* `unit_ref`: String = ""           # slay target roster ref
* `count`: int = COUNT_MIN          # escape: heroes that must leave
* `held`: int = 0                   # hold: rounds counted so far (rule 5; snapshotted)
* `note`: String = ""               # set with a verdict that needs a reason ("%s got away."); "" otherwise
* `heroes_down`: bool = true
* `hero_active`: bool = false
* `withdrew`: int = 0
* `foe_active`: bool = false
* `protect_out`: bool = true
* `out`: Array[Vector2i] = []
* `pair`: Array = raw if raw is Array else []
* `errors`: Array[String] = []
* `objective`: String = str(entry.get("objective", ROUT))
* `max_rounds`: int = DictUtil.to_int(entry.get("max_rounds", 0))
* `arena_open`: bool = str(entry.get("arena", "CLOSED")) == "OPEN"
* `exits`: Array[Vector2i] = _cells(entry.get("exits", []))
* `raw_tiles`: Variant = entry.get("objective_tiles", [])
* `terrain`: Dictionary = CombatZones.terrain_map(str(entry.get("zone", "")))
* `pair`: Array = raw if raw is Array else []
* `cell`: Variant
* `rounds`: int = DictUtil.to_int(entry.get("objective_rounds", DEFAULT_ROUNDS))
* `target`: String = str(entry.get("objective_unit", ""))
* `guests`: Array[String] = _strings(entry.get("guests", []))
* `seen`: Array[String] = []
* `out`: Array[Vector2i] = []
* `pair`: Array = v if v is Array else []
* `out`: Array[String] = []

# Methods

### `static create(entry: Dictionary) -> EncounterObjectives`
```logic
var o := EncounterObjectives.new()
var raw: String = str(entry.get("objective", ROUT))
o.objective = raw if OBJECTIVES.has(raw) else ROUT
o.tiles = tiles_of(entry)
o.rounds = maxi(1, DictUtil.to_int(entry.get("objective_rounds", DEFAULT_ROUNDS), DEFAULT_ROUNDS))
o.unit_ref = str(entry.get("objective_unit", ""))
o.count = clampi(DictUtil.to_int(entry.get("objective_count", COUNT_MIN), COUNT_MIN), COUNT_MIN, COUNT_MAX)
return o
```

### `state() -> Dictionary`
```logic
return {"held": held}
```

### `restore(saved: Dictionary) -> void`
```logic
held = maxi(0, DictUtil.to_int(saved.get("held", 0)))
note = ""
```

### `_count_hold(party_side: Array[Combatant], foes: Array[Combatant], grid: CombatGrid) -> void`
```logic
var ours: bool = _on_tiles(party_side, grid)
var theirs: bool = _on_tiles(foes, grid)
if ours and not theirs:
	held += 1
elif theirs and not ours:
	held = maxi(0, held - 1)
```

### `_on_tiles(units: Array[Combatant], grid: CombatGrid) -> bool`
```logic
for c: Combatant in units:
	if c.is_active() and grid.has_combatant(c) and tiles.has(grid.get_position(c)):
		return true
return false
```

### `static text_of(entry: Dictionary, name_of: Callable) -> String`
```logic
var authored: String = str(entry.get("objective_text", "")).strip_edges()
if not authored.is_empty():
	return authored
var text: String = "Defeat every enemy."
match str(entry.get("objective", ROUT)):
	HOLD:
		text = "Hold the ring for %d rounds, or defeat every enemy." % DictUtil.to_int(entry.get("objective_rounds", DEFAULT_ROUNDS))
	ESCAPE:
		text = "Get %d out by the exit, or defeat every enemy." % DictUtil.to_int(entry.get("objective_count", COUNT_MIN))
	SLAY:
		text = "Defeat %s." % str(name_of.call(str(entry.get("objective_unit", ""))))
var names: Array[String] = []
for ref: String in _strings(entry.get("protect", [])):
	names.append(str(name_of.call(ref)))
if not names.is_empty():
	text += " Keep %s alive." % " and ".join(names)
return text
```

### `static _tile_ok(cell: Vector2i, terrain: Dictionary, arena_open: bool, exits: Array[Vector2i]) -> bool`
```logic
if cell.x < 0 or cell.y < 0 or cell.x >= CombatGrid.GRID_W or cell.y >= CombatGrid.GRID_H:
	return false
var raw: Variant = terrain.get(cell, Terrain.Type.FLOOR)
var t: Terrain.Type = raw if raw is Terrain.Type else Terrain.Type.FLOOR
if not Terrain.is_walkable(t):
	return false
if arena_open:
	return cell.x > 0 and cell.y > 0 and cell.x < CombatGrid.GRID_W - 1 and cell.y < CombatGrid.GRID_H - 1
return not exits.has(cell)
```

