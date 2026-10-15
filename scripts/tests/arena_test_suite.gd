class_name ArenaTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Headless Integration & UI Test Runner
# Thin entry point used by scenes/test_arena.gd (exit code for check_project.ps1).
# The ordered suite list lives in SuiteRegistry; shared helpers in TestHelpers.
# Suites: CombatUiFramework, CoreRules, ClassRules, CombatExtensions, Riposte, GearBudget,
# FocusGear, SkillFamily, EnginePieces, StatusRegistry, Weaken, Foundation, CharacterSpecs,
# GearEquip (+ test dummy), CombatSessionFlow, ArtTerrain, Planning (+ PlanningAiExtra),
# Content, Runner, World.
# ==============================================================================

static func run_all_tests(tree_parent: Node = null) -> bool:
	return SuiteRegistry.run_all(tree_parent)
