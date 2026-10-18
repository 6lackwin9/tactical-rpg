---
class: BlockSnapshot
extends: RefCounted
description: "Tactical RPG - Combat State Block Snapshot Captures immutable unit state at a given block of a round for timeline replay"
---

# Properties
* `block_index`: int = 0
* `round_num`: int = 1
* `reinforcements_spawned`: bool = false
* `phase_state`: Dictionary = {}       # CombatEncounter.phase_state() copy (BOSS1: fired / due / dropped)
* `objective_state`: Dictionary = {}   # CombatEncounter.objective_state() copy (OBJ1: hold rounds counted)
* `positions`: Dictionary = {}         # Combatant -> Vector2i
* `hps`: Dictionary = {}               # Combatant -> int
* `max_hps`: Dictionary = {}           # Combatant -> int
* `stances`: Dictionary = {}           # Combatant -> CombatTypes.StanceType
* `targets`: Dictionary = {}           # Combatant -> Variant
* `is_active`: Dictionary = {}         # Combatant -> bool
* `is_withdrawn`: Dictionary = {}      # Combatant -> bool
* `attack_boost_mult`: Dictionary = {} # Combatant -> float
* `attack_boost_rds`: Dictionary = {}  # Combatant -> int
* `defense_mit`: Dictionary = {}       # Combatant -> float
* `aps`: Dictionary = {}               # Combatant -> int (runtime AP)
* `statuses`: Dictionary = {}          # Combatant -> Dictionary (active statuses, copy)
* `status_rounds`: Dictionary = {}     # Combatant -> Dictionary (rounds left per status, copy)
* `status_magnitudes`: Dictionary = {} # Combatant -> Dictionary (magnitude per status, copy)
* `pending_status_magnitudes`: Dictionary = {} # Combatant -> Dictionary (queued magnitude per status, copy)
* `status_sources`: Dictionary = {}    # Combatant -> Dictionary (status -> applier unit name, copy)
* `pending_status_sources`: Dictionary = {} # Combatant -> Dictionary (queued status -> applier unit name, copy)
* `pouches`: Dictionary = {}           # Combatant -> Array[Consumable] (battle_pouch copy, ITEMUSE1 rule 6: undo / reset give items back)

