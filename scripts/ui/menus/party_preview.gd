class_name PartyPreview
extends RefCounted

# ==============================================================================
# Tactical RPG - Party preview (UI2-L): the derived block of a party member, built by PartyBuilder with unspent points
# LEFT unspent (only confirmed and drafted points show), and the small display helpers the party screens share.
# ==============================================================================

const DERIVED_LABELS: Dictionary = {
	"max_hp": "Max HP", "speed": "Speed", "move_per_block": "Tiles per move", "crit_pct": "Crit %",
	"melee_power": "Melee power", "ranged_power": "Ranged power", "armor_rating": "Armor", "debuff_resist": "Debuff resist %", "ap": "AP",
}

# {max_hp, speed, move_per_block, crit_pct, melee_power, ranged_power, armor_rating, debuff_resist, ap}; {} when the
# state does not build.
static func derived(state: UnitState) -> Dictionary:
	var c: Combatant = PartyBuilder.build(state.duplicate_state(), false)
	if c == null:
		return {}
	return {
		"max_hp": c.max_hp, "speed": c.speed, "move_per_block": c.move_per_block(),
		"crit_pct": roundi(DamageCalculator.crit_chance(c)),
		"melee_power": DictUtil.to_int(DamageCalculator.attack_power(c, false)["attack_power"]),
		"ranged_power": DictUtil.to_int(DamageCalculator.attack_power(c, true)["attack_power"]),
		"armor_rating": DamageCalculator.armor_rating(c), "debuff_resist": roundi(c.debuff_resistance()), "ap": c.max_ap,
	}

# The member's display name (base unit name, else its id).
static func name_of(m: UnitState) -> String:
	return PartyBuilder.display_name(m)   # RECRUIT1-L rule 2: a named hero shows its character name

static func class_name_of(m: UnitState) -> String:
	var cls: Dictionary = ContentCatalog.get_entry("classes", m.class_id())
	return str(cls.get("name", m.class_id()))

static func member(member_id: String) -> UnitState:
	var gs: GameState = CurrentGame.state()
	return gs.party.member(member_id) if gs != null else null
