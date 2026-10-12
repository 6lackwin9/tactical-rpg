class_name DojoStatDefs
extends RefCounted

# Display metadata for the 6 core stats (shared by the inspector and the allocation overlay).

const STATS: Array[Dictionary] = [
	{"k": "STR", "name": "Strength", "kind": "Active", "role": "Melee scaling (×1.5), flat mitigation (×0.25) & party carry capacity.", "c": Color(0.95, 0.45, 0.45)},
	{"k": "CONS", "name": "Constitution", "kind": "Passive", "role": "Max Health (×8) & party carry capacity.", "c": Color(0.95, 0.65, 0.3)},
	{"k": "DEX", "name": "Dexterity", "kind": "Active", "role": "Ranged scaling (×1.5), initiative speed & crit rate.", "c": Color(0.4, 0.9, 0.5)},
	{"k": "PERC", "name": "Perception", "kind": "Passive", "role": "Intent unmasking & sight range.", "c": Color(0.4, 0.8, 0.95)},
	{"k": "INT", "name": "Intelligence", "kind": "Active", "role": "Skill power (×1.5), buff potency & debuff resistance.", "c": Color(0.75, 0.5, 0.95)},
	{"k": "KNOW", "name": "Knowledge", "kind": "Passive", "role": "Lore checks & danger mitigation.", "c": Color(0.5, 0.7, 1.0)},
]
