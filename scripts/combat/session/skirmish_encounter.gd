class_name SkirmishEncounter
extends CatalogEncounter

# ==============================================================================
# Tactical RPG - Skirmish Encounter (player-planned, AI-opposed)
# The catalog encounter "goblin_skirmish" (Warrior & Ranger vs five goblins,
# 6-block rounds, 12 AP +4 per round) with its own narration. Rules and roster
# live in res://data/catalog (edit them in the Content Forge); the named fields
# below give tests direct access to each unit.
# ==============================================================================

var warrior: Combatant
var ranger: Combatant
var brawler: Combatant
var skirmisher: Combatant
var archer: Combatant
var shaman: Combatant
var berserker: Combatant

func _init() -> void:
	super("goblin_skirmish")

func build() -> void:
	super.build()
	warrior = unit("warrior")
	ranger = unit("ranger")
	brawler = unit("goblin_brawler")
	skirmisher = unit("goblin_skirmisher")
	archer = unit("goblin_archer")
	shaman = unit("goblin_shaman")
	berserker = unit("goblin_berserker")

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

func round_summary(round_num: int, combat_over: bool) -> Dictionary:
	if combat_over and party_alive():
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
