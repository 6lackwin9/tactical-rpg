class_name CombatTypes
extends RefCounted

enum ActionType {
	ATTACK,
	DEFEND,
	SKILL,
	ITEM,
	MOVE,
	ENTER_NEUTRAL,
	HOLD_STANCE,
	TARGET
}

enum StanceType {
	NEUTRAL,
	OFFENSIVE,
	DEFENSIVE,
	TACTICAL,
	UTILITY,
	MOBILE,
	STAGGERED
}

# Calculates stance matchup multiplier when attacker strikes defender
static func get_matchup_multiplier(attacker_stance: StanceType, defender_stance: StanceType) -> float:
	match attacker_stance:
		StanceType.OFFENSIVE:
			match defender_stance:
				StanceType.DEFENSIVE:
					return 0.75  # Defensive absorbs/parries Offensive
				StanceType.TACTICAL:
					return 1.25  # Offensive punishes Tactical prep
				StanceType.MOBILE:
					return 1.15  # High pressure catches light armor
				StanceType.UTILITY:
					return 1.25  # Overwhelms utility prep
				_:
					return 1.0
		StanceType.DEFENSIVE:
			match defender_stance:
				StanceType.TACTICAL:
					return 0.75  # Tactical status/magic bypasses physical defense
				StanceType.OFFENSIVE:
					return 1.25  # Defensive counter/retaliation against Offensive
				_:
					return 1.0
		StanceType.TACTICAL:
			match defender_stance:
				StanceType.DEFENSIVE:
					return 1.25  # Tactical bypasses static guard
				StanceType.OFFENSIVE:
					return 0.75  # Rushed down by Offensive
				StanceType.MOBILE:
					return 1.20  # AoE / crowd control pins down mobility
				_:
					return 1.0
		StanceType.MOBILE:
			match defender_stance:
				StanceType.DEFENSIVE:
					return 1.25  # Mobile flanks static guard
				StanceType.OFFENSIVE:
					return 0.85  # Vulnerable to direct high-power strikes
				StanceType.UTILITY:
					return 1.25  # Raids utility users
				_:
					return 1.0
		_:
			return 1.0

static func stance_to_string(stance: StanceType) -> String:
	match stance:
		StanceType.NEUTRAL: return "Neutral"
		StanceType.OFFENSIVE: return "Offensive"
		StanceType.DEFENSIVE: return "Defensive"
		StanceType.TACTICAL: return "Tactical"
		StanceType.UTILITY: return "Utility"
		StanceType.MOBILE: return "Mobile"
		StanceType.STAGGERED: return "Staggered"
		_: return "Unknown"

# Stance flow for planned sequences: from Neutral anything is open; otherwise a
# unit stays in its stance pool, except that Offensive and Mobile flow freely
# into each other (advance and strike, strike and reposition). Anything else
# needs a Reset Stance (back to Neutral) first.
static func can_flow(from_stance: StanceType, to_stance: StanceType) -> bool:
	if from_stance == StanceType.NEUTRAL or to_stance == StanceType.NEUTRAL or from_stance == to_stance:
		return true
	var striking: Array[StanceType] = [StanceType.OFFENSIVE, StanceType.MOBILE]
	return striking.has(from_stance) and striking.has(to_stance)
