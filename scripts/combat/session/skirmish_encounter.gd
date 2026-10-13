class_name SkirmishEncounter
extends CombatEncounter

# ==============================================================================
# Tactical RPG - Skirmish Encounter (player-planned, AI-opposed)
# Warrior & Ranger vs five goblins. The player plans the party's 10-block round;
# every goblin plans its own round with TacticalAI (roles below). State carries
# from round to round until one side is gone or the round limit is reached.
# ==============================================================================

var warrior: Combatant
var ranger: Combatant
var brawler: Combatant
var skirmisher: Combatant
var archer: Combatant
var shaman: Combatant
var berserker: Combatant

func _init() -> void:
	encounter_name = "Goblin Skirmish"
	max_rounds = 6
	zone_id = "ruined_courtyard"
	player_planning = true

func build() -> void:
	grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	grid.apply_zone(zone_id)
	reinforcements_spawned = false

	warrior = Combatant.new("Warrior", 100, 15, 1)
	ranger = Combatant.new("Ranger", 80, 14, 2)
	brawler = Combatant.new("Goblin Brawler", 50, 12, 1)
	skirmisher = Combatant.new("Goblin Skirmisher", 40, 15, 1)
	archer = Combatant.new("Goblin Archer", 35, 13, 3)
	shaman = Combatant.new("Goblin Shaman", 35, 12, 2)
	berserker = Combatant.new("Goblin Berserker", 50, 11, 1)

	allies = [warrior, ranger]
	enemies = [brawler, skirmisher, archer, shaman, berserker]
	grid.deploy_combatants(allies, enemies)

	party_stash = Inventory.new()
	ItemCatalog.populate_party_starter_stash(party_stash)
	unit_classes = {
		warrior: "warrior", ranger: "ranger", brawler: "brawler",
		skirmisher: "skirmisher", archer: "archer", shaman: "shaman", berserker: "berserker",
	}
	ai_profiles = {
		warrior: AiProfile.create(AiProfile.Role.AGGRESSOR, 25, 0.0),
		ranger: AiProfile.create(AiProfile.Role.SNIPER, 22, 0.0),
		brawler: AiProfile.create(AiProfile.Role.AGGRESSOR, 18),
		skirmisher: AiProfile.create(AiProfile.Role.SKIRMISHER, 16),
		archer: AiProfile.create(AiProfile.Role.SNIPER, 16),
		shaman: AiProfile.create(AiProfile.Role.SUPPORT, 14),
		berserker: AiProfile.create(AiProfile.Role.AGGRESSOR, 20, 0.0),
	}

func prepare_round(round_num: int) -> Dictionary:
	if round_num == 1:
		return {
			"phase": "ROUND 1: PLANNING PHASE",
			"status": "Plan your party's round on the timeline, then press Step / Auto-Resolve.",
			"log": "[color=cyan]>>> Goblin Skirmish begins. Goblins have planned their round (hidden).[/color]",
		}
	return {
		"phase": "ROUND %d: PLANNING PHASE" % round_num,
		"status": "Round %d: re-plan your party. The goblins have chosen their moves." % round_num,
		"log": "\n[color=#38bdf8][b]⚔ ROUND %d: PLANNING ⚔[/b][/color]" % round_num,
	}

func schedule_round(resolver: CombatResolver, _round_num: int) -> void:
	schedule_enemy_ai(resolver)

func round_summary(round_num: int, combat_over: bool) -> Dictionary:
	var party_alive: bool = false
	for a: Combatant in allies:
		if a.is_active():
			party_alive = true
	if combat_over and party_alive:
		return {
			"phase": "🏆 VICTORY - ALL GOBLINS ROUTED",
			"status": "🏆 The goblins are down or have fled. Review the replay.",
			"log": "\n[color=#38bdf8][b]🏆 VICTORY: the party holds the courtyard![/b][/color]",
		}
	if combat_over:
		return {
			"phase": "💀 DEFEAT - PARTY FALLEN",
			"status": "💀 Your party was wiped out. Reset the battle to try a new plan.",
			"log": "\n[color=#f87171][b]💀 DEFEAT: the party has fallen.[/b][/color]",
		}
	if round_num >= max_rounds:
		return {
			"phase": "ROUND %d CONCLUDED - STALEMATE" % round_num,
			"status": "Round limit reached with both sides standing.",
			"log": "\n[color=#fde047][b]The skirmish ends in a stalemate.[/b][/color]",
		}
	return {
		"phase": "ROUND %d CONCLUDED - REPLAY MODE" % round_num,
		"status": "Round %d complete. Review the replay, then advance." % round_num,
		"log": "",
	}
