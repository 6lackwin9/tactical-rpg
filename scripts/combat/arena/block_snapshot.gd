class_name BlockSnapshot
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat State Block Snapshot
# Captures immutable unit state at a given block of a round for timeline replay
# ==============================================================================

var block_index: int = 0
var round_num: int = 1
var reinforcements_spawned: bool = false
var phase_state: Dictionary = {}       # CombatEncounter.phase_state() copy (BOSS1: fired / due / dropped)
var objective_state: Dictionary = {}   # CombatEncounter.objective_state() copy (OBJ1: hold rounds counted)
var positions: Dictionary = {}         # Combatant -> Vector2i
var hps: Dictionary = {}               # Combatant -> int
var max_hps: Dictionary = {}           # Combatant -> int
var stances: Dictionary = {}           # Combatant -> CombatTypes.StanceType
var targets: Dictionary = {}           # Combatant -> Variant
var is_active: Dictionary = {}         # Combatant -> bool
var is_withdrawn: Dictionary = {}      # Combatant -> bool
var attack_boost_mult: Dictionary = {} # Combatant -> float
var attack_boost_rds: Dictionary = {}  # Combatant -> int
var defense_mit: Dictionary = {}       # Combatant -> float
var aps: Dictionary = {}               # Combatant -> int (runtime AP)
var statuses: Dictionary = {}          # Combatant -> Dictionary (active statuses, copy)
var status_rounds: Dictionary = {}     # Combatant -> Dictionary (rounds left per status, copy)
var status_magnitudes: Dictionary = {} # Combatant -> Dictionary (magnitude per status, copy)
var pending_status_magnitudes: Dictionary = {} # Combatant -> Dictionary (queued magnitude per status, copy)
var status_sources: Dictionary = {}    # Combatant -> Dictionary (status -> applier unit name, copy)
var pending_status_sources: Dictionary = {} # Combatant -> Dictionary (queued status -> applier unit name, copy)
var pouches: Dictionary = {}           # Combatant -> Array[Consumable] (battle_pouch copy, ITEMUSE1 rule 6: undo / reset give items back)
