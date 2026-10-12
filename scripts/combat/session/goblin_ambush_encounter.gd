class_name GoblinAmbushEncounter
extends CombatEncounter

# ==============================================================================
# Tactical RPG - Scripted Demo Encounter: Goblin Ambush (3 rounds)
# Warrior & Ranger vs a Brawler and a fleeing Scout; the Scout's escape triggers
# a 4-goblin ambush at Round 1 Block 4. Rounds 2-3 are the Warrior's last stand.
# All commands are pre-planned (no AI yet) so the outcome is deterministic and
# serves as the reference battle for the combat UI and its tests.
# ==============================================================================

var warrior: Combatant
var ranger: Combatant
var brawler: Combatant
var scout: Combatant
var skirmisher: Combatant
var archer: Combatant
var berserker: Combatant
var shaman: Combatant

func _init() -> void:
	encounter_name = "Goblin Ambush"
	max_rounds = 3
	zone_id = "forest_clearing"

func build() -> void:
	grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	grid.apply_zone(zone_id)
	reinforcements_spawned = false

	warrior = Combatant.new("Warrior", 100, 15, 1)
	ranger = Combatant.new("Ranger", 80, 14, 2)
	brawler = Combatant.new("Enemy Brawler", 45, 12, 1)
	scout = Combatant.new("Fleeing Scout", 30, 16, 1)
	skirmisher = Combatant.new("Goblin Skirmisher", 40, 15, 1)
	archer = Combatant.new("Goblin Archer", 35, 13, 3)
	berserker = Combatant.new("Goblin Berserker", 55, 11, 1)
	shaman = Combatant.new("Goblin Shaman", 35, 12, 2)

	allies = [warrior, ranger]
	enemies = [brawler, scout, skirmisher, archer, berserker, shaman]

	# Center-ring deployment for the initial combatants only; the ambush arrives later.
	var initial_enemies: Array[Combatant] = [brawler, scout]
	grid.deploy_combatants(allies, initial_enemies)

	party_stash = Inventory.new()
	ItemCatalog.populate_party_starter_stash(party_stash)
	unit_classes = {
		warrior: "warrior", ranger: "ranger", brawler: "brawler", scout: "scout",
		skirmisher: "skirmisher", archer: "archer", berserker: "berserker", shaman: "shaman",
	}

# ------------------------------------------------------------------------------
# Round setup & narration
# ------------------------------------------------------------------------------

func prepare_round(round_num: int) -> Dictionary:
	match round_num:
		1:
			return {
				"phase": "PLANNING PHASE",
				"status": "Round 1: Set party actions, then Auto-Resolve.",
				"log": "[color=cyan]>>> New 10-Block Tactical Battle Initialized (Round 1 Pre-planned Actions Loaded).[/color]",
			}
		2:
			# Exact end state of Round 1 (no movement between rounds).
			_set_state(warrior, 73, Vector2i(4, 4))
			_set_state(skirmisher, 40, Vector2i(5, 4))
			_set_state(berserker, 30, Vector2i(4, 5))
			_set_state(shaman, 35, Vector2i(6, 4))
			_set_state(archer, 35, Vector2i(6, 5))
			brawler.hp = 0
			scout.is_withdrawn = true
			ranger.hp = 0
			return {
				"entry": CombatEncounter.make_event_entry(2, 1, "Warrior", true, "Round 2 Commenced", "Last Stand", "ROUND 2", "Warrior stands alone at (4, 4) against 4 goblins", "Round 2 Active"),
				"phase": "ROUND 2: PLANNING PHASE",
				"status": "Round 2: Warrior (73 HP) stands alone against 4 goblins! Set actions or Auto-Resolve.",
				"log": "\n[color=#38bdf8][b]⚔⚔⚔ COMMENCING ROUND 2: WARRIOR'S LAST STAND ⚔⚔⚔[/b][/color]",
			}
		3:
			_set_state(warrior, 35, Vector2i(5, 4))
			_set_state(shaman, 15, Vector2i(6, 4))
			_set_state(archer, 35, Vector2i(6, 5))
			berserker.hp = 0
			skirmisher.hp = 0
			brawler.hp = 0
			scout.is_withdrawn = true
			ranger.hp = 0
			return {
				"entry": CombatEncounter.make_event_entry(3, 1, "Warrior", true, "Round 3 Commenced", "Final Showdown", "ROUND 3", "Warrior (35 HP) at (5, 4) against Shaman (15 HP) & Archer (35 HP)", "Round 3 Active"),
				"phase": "ROUND 3: PLANNING PHASE (FINAL SHOWDOWN)",
				"status": "Round 3: Final Showdown! Warrior confronts surviving Shaman & Archer. Auto-Resolve to conclude combat.",
				"log": "\n[color=#38bdf8][b]⚔⚔⚔ COMMENCING ROUND 3: FINAL SHOWDOWN ⚔⚔⚔[/b][/color]",
			}
	return {}

func _set_state(c: Combatant, hp_value: int, pos: Vector2i) -> void:
	c.hp = hp_value
	c.set_stance(CombatTypes.StanceType.NEUTRAL)
	grid.combatant_positions[c] = pos

func round_summary(round_num: int, combat_over: bool) -> Dictionary:
	if combat_over:
		return {
			"phase": "🏆 COMBAT CONCLUDED - TOTAL VICTORY",
			"status": "🏆 All hostiles eliminated! Warrior stands victorious. Replay available.",
			"log": "\n[color=#38bdf8][b]🏆 TOTAL VICTORY: All hostiles eliminated![/b][/color]",
		}
	match round_num:
		1:
			return {"phase": "ROUND 1 CONCLUDED - REPLAY MODE", "status": "Round 1 complete! Click Advance to Round 2.", "log": "\n[color=#4ade80][b]🎉 ROUND 1 RESOLVED: Ranger fell in the ambush; Warrior holds against 4 goblins.[/b][/color]"}
		2:
			return {"phase": "ROUND 2 CONCLUDED - REPLAY MODE", "status": "Round 2 complete! Skirmisher & Berserker eliminated. Advance to Round 3.", "log": "\n[color=#4ade80][b]🎉 ROUND 2 RESOLVED.[/b][/color]"}
	return {"phase": "ROUND %d CONCLUDED - REPLAY MODE" % round_num, "status": "Round %d concluded. Review battle replay." % round_num, "log": ""}

func advance_label(round_num: int) -> String:
	return "⚔ ADVANCE TO ROUND 3 (FINAL SHOWDOWN)" if round_num == 2 else "⚔ ADVANCE TO ROUND %d" % (round_num + 1)

# ------------------------------------------------------------------------------
# Mid-round trigger: the Scout's escape alerts the pack at Round 1, Block 4
# ------------------------------------------------------------------------------

func on_block_start(round_num: int, block_num: int) -> ActionLogEntry:
	if round_num != 1 or block_num != 4 or reinforcements_spawned:
		return null
	reinforcements_spawned = true
	grid.place_combatant(berserker, Vector2i(4, 5))
	grid.place_combatant(skirmisher, Vector2i(5, 4))
	grid.place_combatant(shaman, Vector2i(6, 4))
	grid.place_combatant(archer, Vector2i(6, 5))
	return CombatEncounter.make_event_entry(1, 4, "Goblin Pack", false, "Ambush Reinforcements", "Perimeter Breach", "AMBUSH!", "4 Goblin reinforcements arrive from eastern ridge!", "Hostiles: 6 total")

# ------------------------------------------------------------------------------
# Pre-planned commands per round
# ------------------------------------------------------------------------------

func schedule_round(resolver: CombatResolver, round_num: int) -> void:
	match round_num:
		1:
			_schedule_round_1(resolver)
		2:
			_schedule_round_2(resolver)
		3:
			_schedule_round_3(resolver)

func _schedule_round_1(r: CombatResolver) -> void:
	# Warrior: step to center, strike the Brawler, turn on the ambush, defend, hold.
	r.schedule_action(warrior, MoveCommand.new(warrior, grid, Vector2i(0, -1), 1), 1)
	r.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, grid), 2)
	r.schedule_action(warrior, TargetCommand.new(warrior, berserker, 1), 4)
	r.schedule_action(warrior, AttackCommand.new(warrior, berserker, 25, 2, 0, grid), 5)
	r.schedule_action(warrior, DefendCommand.new(warrior, 0.5, 2, 1), 7)
	r.schedule_action(warrior, HoldStanceCommand.new(warrior, 0), 9)
	# Ranger: Focus, advance, boosted shot on the Brawler, then the Archer (cancelled if fallen).
	r.schedule_action(ranger, FocusCommand.new(ranger, 1.5, 2, 2, 0), 1)
	r.schedule_action(ranger, MoveCommand.new(ranger, grid, Vector2i(0, -1), 1), 4)
	r.schedule_action(ranger, AttackCommand.new(ranger, brawler, 30, 2, 0, grid), 5)
	r.schedule_action(ranger, AttackCommand.new(ranger, archer, 25, 2, 0, grid), 7)
	# Enemy Brawler: target, defend, counter.
	r.schedule_action(brawler, TargetCommand.new(brawler, warrior, 2), 1)
	r.schedule_action(brawler, DefendCommand.new(brawler, 0.5, 2, 1), 2)
	r.schedule_action(brawler, AttackCommand.new(brawler, warrior, 15, 2, 0, grid), 4)
	# Fleeing Scout: run for the exit at (8, 4).
	r.schedule_action(scout, MoveCommand.new(scout, grid, Vector2i(1, 0), 1), 1)
	r.schedule_action(scout, MoveCommand.new(scout, grid, Vector2i(1, 0), 2, 2), 2)
	# Ambush pack (acts from Block 4 once spawned).
	r.schedule_action(berserker, TargetCommand.new(berserker, ranger, 1), 4)
	r.schedule_action(berserker, AttackCommand.new(berserker, ranger, 30, 2, 0, grid), 5)
	r.schedule_action(berserker, TargetCommand.new(berserker, warrior, 1), 7)
	r.schedule_action(berserker, AttackCommand.new(berserker, warrior, 15, 3, 0, grid), 8)
	r.schedule_action(archer, TargetCommand.new(archer, ranger, 1), 4)
	r.schedule_action(archer, AttackCommand.new(archer, ranger, 20, 2, 0, grid), 5)
	r.schedule_action(archer, TargetCommand.new(archer, warrior, 1), 7)
	r.schedule_action(archer, AttackCommand.new(archer, warrior, 12, 3, 0, grid), 8)
	r.schedule_action(shaman, TargetCommand.new(shaman, ranger, 1), 4)
	r.schedule_action(shaman, AttackCommand.new(shaman, ranger, 20, 2, 0, grid), 5)
	r.schedule_action(shaman, TargetCommand.new(shaman, warrior, 1), 7)
	r.schedule_action(shaman, AttackCommand.new(shaman, warrior, 10, 3, 0, grid), 8)
	r.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior, 1), 4)
	r.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 12, 3, 0, grid), 5)
	r.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.4, 2, 1), 8)
	r.schedule_action(skirmisher, HoldStanceCommand.new(skirmisher, 0), 10)

func _schedule_round_2(r: CombatResolver) -> void:
	r.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 40, 2, 0, grid), 1)
	r.schedule_action(warrior, DefendCommand.new(warrior, 0.5, 2, 1), 3)
	r.schedule_action(warrior, AttackCommand.new(warrior, berserker, 35, 2, 0, grid), 5)
	r.schedule_action(warrior, MoveCommand.new(warrior, grid, Vector2i(1, 0), 1), 7)
	r.schedule_action(warrior, AttackCommand.new(warrior, shaman, 20, 2, 0, grid), 8)
	r.schedule_action(warrior, DefendCommand.new(warrior, 0.5, 1, 1), 10)
	r.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior, 1), 1)
	r.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 10, 1, 0, grid), 2)
	r.schedule_action(berserker, TargetCommand.new(berserker, warrior, 1), 1)
	r.schedule_action(berserker, AttackCommand.new(berserker, warrior, 18, 3, 0, grid), 2)
	r.schedule_action(berserker, TargetCommand.new(berserker, warrior, 1), 5)
	r.schedule_action(archer, TargetCommand.new(archer, warrior, 1), 1)
	r.schedule_action(archer, AttackCommand.new(archer, warrior, 14, 3, 0, grid), 2)
	r.schedule_action(archer, TargetCommand.new(archer, warrior, 1), 5)
	r.schedule_action(archer, AttackCommand.new(archer, warrior, 14, 3, 0, grid), 6)
	r.schedule_action(archer, DefendCommand.new(archer, 0.4, 2, 1), 9)
	r.schedule_action(shaman, TargetCommand.new(shaman, warrior, 1), 1)
	r.schedule_action(shaman, AttackCommand.new(shaman, warrior, 12, 3, 0, grid), 2)
	r.schedule_action(shaman, TargetCommand.new(shaman, warrior, 1), 5)
	r.schedule_action(shaman, AttackCommand.new(shaman, warrior, 12, 3, 0, grid), 6)
	r.schedule_action(shaman, DefendCommand.new(shaman, 0.5, 2, 1), 9)

func _schedule_round_3(r: CombatResolver) -> void:
	r.schedule_action(warrior, AttackCommand.new(warrior, shaman, 25, 2, 0, grid), 1)
	r.schedule_action(warrior, MoveCommand.new(warrior, grid, Vector2i(1, 0), 1), 3)
	r.schedule_action(warrior, DefendCommand.new(warrior, 0.7, 2, 1), 4)
	r.schedule_action(warrior, TargetCommand.new(warrior, archer, 1), 6)
	r.schedule_action(warrior, AttackCommand.new(warrior, archer, 40, 2, 1, grid), 7)
	r.schedule_action(warrior, HoldStanceCommand.new(warrior, 0), 9)
	r.schedule_action(shaman, TargetCommand.new(shaman, warrior, 1), 1)
	# Starts after the 1-block target lock (the original script overlapped both on block 1).
	r.schedule_action(shaman, AttackCommand.new(shaman, warrior, 10, 2, 0, grid), 2)
	r.schedule_action(archer, TargetCommand.new(archer, warrior, 1), 1)
	r.schedule_action(archer, AttackCommand.new(archer, warrior, 14, 3, 0, grid), 2)
	r.schedule_action(archer, TargetCommand.new(archer, warrior, 1), 5)
	r.schedule_action(archer, AttackCommand.new(archer, warrior, 16, 3, 0, grid), 6)
	r.schedule_action(archer, DefendCommand.new(archer, 0.5, 2, 1), 9)
