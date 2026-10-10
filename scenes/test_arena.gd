extends Node2D

func _ready() -> void:
	print("\n=======================================================")
	print("=== STARTING HEADLESS COMBAT HARNESS (10-BLOCK RPG) ===")
	print("=======================================================")

	var all_tests_passed: bool = _run_warrior_vs_3_goblins_battle()

	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Ready for manual interaction.\n")


func _run_warrior_vs_3_goblins_battle() -> bool:
	var resolver := CombatResolver.new()

	# Create Warrior with 3 skills: Attack (2b), Defend (2b), Focus (2b)
	var warrior := Combatant.new("Warrior", 120, 15)

	# Create 3 Goblins with 2 skills: Attack (2b), Defend (2b)
	var scout := Combatant.new("Goblin Scout", 35, 18)
	var brawler := Combatant.new("Goblin Brawler", 45, 12)
	var skirmisher := Combatant.new("Goblin Skirmisher", 40, 16)

	resolver.register_combatant(warrior)
	resolver.register_combatant(scout)
	resolver.register_combatant(brawler)
	resolver.register_combatant(skirmisher)

	# ----------------------------------------------------
	# ROUND 1 SCHEDULING (10-BLOCK TIMELINE)
	# ----------------------------------------------------
	print("\n>>> Scheduling Round 1 Actions...")

	# Warrior actions (10 blocks):
	# Block 1: Target Scout (1b, Neutral)
	# Block 2-3: Focus (2b, Tactical, +50% attack boost for 2 rounds)
	# Block 4: Enter Neutral (1b, Neutral)
	# Block 5-6: Attack Scout (2b, Offensive, boosted)
	# Block 7: Enter Neutral (1b, Neutral)
	# Block 8-9: Defend (2b, Defensive, 50% damage mitigation)
	# Block 10: Hold Stance (1b, extends Defensive into Round 2)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, scout), 1)
	resolver.schedule_action(warrior, FocusCommand.new(warrior, 1.5, 2), 2)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 4)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, scout, 30), 5)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 7)
	resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5), 8)
	resolver.schedule_action(warrior, HoldStanceCommand.new(warrior), 10)

	# Goblin Scout actions (fast striker):
	# Block 1-2: Defend (2b)
	# Block 3-4: Attack Warrior (2b, power 12) -> lands on Block 4
	# Block 5-6: Attack Warrior (2b, power 12) -> strikes before falling on Block 6
	resolver.schedule_action(scout, DefendCommand.new(scout, 0.5), 1)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 12), 3)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 12), 5)

	# Goblin Brawler actions (heavy hitter):
	# Block 1-2: Defend (2b)
	# Block 3-4: Defend (2b)
	# Block 5-6: Defend (2b)
	# Block 7: Enter Neutral (1b)
	# Block 8-9: Attack Warrior (2b, power 14) -> lands on Block 9 while Warrior Defends (mitigated to 7)!
	# Block 10: Hold Stance (1b)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 1)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 3)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 5)
	resolver.schedule_action(brawler, EnterNeutralCommand.new(brawler), 7)
	resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 14), 8)
	resolver.schedule_action(brawler, HoldStanceCommand.new(brawler), 10)

	# Goblin Skirmisher actions:
	# Block 1: Target Warrior (1b)
	# Block 2-3: Attack Warrior (2b, power 10) -> hits Warrior on Block 3
	# Block 4: Enter Neutral (1b)
	# Block 5-6: Defend (2b)
	# Block 7: Enter Neutral (1b)
	# Block 8-9: Attack Warrior (2b, power 12) -> lands on Block 9 while Warrior Defends (mitigated to 6)!
	# Block 10: Hold Stance (1b)
	resolver.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior), 1)
	resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 10), 2)
	resolver.schedule_action(skirmisher, EnterNeutralCommand.new(skirmisher), 4)
	resolver.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.5), 5)
	resolver.schedule_action(skirmisher, EnterNeutralCommand.new(skirmisher), 7)
	resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 12), 8)
	resolver.schedule_action(skirmisher, HoldStanceCommand.new(skirmisher), 10)

	# Execute Round 1
	resolver.resolve_round()

	# Record & Validate Round 1 Outcomes:
	var warrior_hp_r1: int = warrior.hp
	var scout_defeated_r1: bool = scout.hp == 0
	# Warrior took: 10 dmg (Block 3) + 12 dmg (Block 4) + 12 dmg (Block 6) + 7 mitigated (Block 9) + 6 mitigated (Block 9) = 47 dmg -> HP 73
	var warrior_hp_r1_ok: bool = warrior_hp_r1 == 73
	# Focus buff persisted into Round 2 with 1 round remaining
	var focus_buff_persisted: bool = warrior.attack_boost_multiplier == 1.5 and warrior.attack_boost_rounds_remaining == 1
	# Warrior ended Round 1 in Defensive Stance
	var stance_persisted: bool = warrior.current_stance == CombatTypes.StanceType.DEFENSIVE

	# ----------------------------------------------------
	# ROUND 2 SCHEDULING (TESTING ROLLOVER & FOCUS BUFF)
	# ----------------------------------------------------
	print("\n>>> Scheduling Round 2 Actions (Testing Stance Rollover & Buff Persistence)...")

	# Warrior begins Round 2 in Defensive Stance:
	# Block 1: Enter Neutral (1b) -> transitions out of Defensive
	# Block 2: Target Brawler (1b, Neutral)
	# Block 3-4: Attack Brawler (2b, boosted 30 * 1.5 = 45 dmg) -> defeats Brawler (45 HP)!
	# Block 5: Enter Neutral (1b)
	# Block 6: Target Skirmisher (1b, Neutral)
	# Block 7-8: Attack Skirmisher (2b, boosted 30 * 1.5 = 45 dmg) -> defeats Skirmisher (40 HP)!
	# Block 9-10: Defend (2b)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 1)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, brawler), 2)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 30), 3)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 5)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, skirmisher), 6)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 30), 7)
	resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5), 9)

	# Brawler attempts attack on Block 1-2 before defeat on Block 4:
	resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 10), 1)

	# Skirmisher defends early:
	resolver.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.5), 1)

	# Execute Round 2
	resolver.resolve_round()

	# Validate Final Battle Outcomes:
	var warrior_hp_r2: int = warrior.hp
	var brawler_defeated: bool = brawler.hp == 0
	var skirmisher_defeated: bool = skirmisher.hp == 0
	var warrior_survived: bool = warrior.is_alive() and warrior_hp_r2 == 63
	var all_goblins_down: bool = scout_defeated_r1 and brawler_defeated and skirmisher_defeated
	var buff_expired_after_r2: bool = warrior.attack_boost_multiplier == 1.0 and warrior.attack_boost_rounds_remaining == 0

	print("\n==========================================")
	print("=== FINAL INTEGRATION TEST ASSERTIONS ===")
	print("==========================================")
	print("  [R1] Goblin Scout defeated (0 HP): %s" % ("PASS" if scout_defeated_r1 else "FAIL"))
	print("  [R1] Warrior mitigated damage (HP == 73): %s (HP: %d)" % [("PASS" if warrior_hp_r1_ok else "FAIL"), warrior_hp_r1])
	print("  [R1] Defensive Stance carried to R2: %s" % ("PASS" if stance_persisted else "FAIL"))
	print("  [R1] Focus 2-round buff active into R2: %s" % ("PASS" if focus_buff_persisted else "FAIL"))
	print("  [R2] Goblin Brawler defeated (0 HP): %s" % ("PASS" if brawler_defeated else "FAIL"))
	print("  [R2] Goblin Skirmisher defeated (0 HP): %s" % ("PASS" if skirmisher_defeated else "FAIL"))
	print("  [R2] Warrior survived battle (HP == 63): %s (HP: %d/120)" % [("PASS" if warrior_survived else "FAIL"), warrior_hp_r2])
	print("  [R2] All 3 Goblins eliminated: %s" % ("PASS" if all_goblins_down else "FAIL"))
	print("  [R2] Focus 2-round buff expired cleanly: %s" % ("PASS" if buff_expired_after_r2 else "FAIL"))

	# Clean up references to prevent ObjectDB leaks on exit
	warrior.clear_target()
	scout.clear_target()
	brawler.clear_target()
	skirmisher.clear_target()
	resolver.registered_combatants.clear()

	return (scout_defeated_r1 and warrior_hp_r1_ok and stance_persisted and
		focus_buff_persisted and brawler_defeated and skirmisher_defeated and
		warrior_survived and all_goblins_down and buff_expired_after_r2)