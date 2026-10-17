class_name SuiteRegistry
extends RefCounted

# Every headless suite in run order. To add a suite: write scripts/tests/<name>_test_suite.gd
# (class_name <Name>TestSuite, static func run(...) -> bool) and add ONE results.append line below.
# Nothing else needs editing; scenes/test_arena.gd calls run_all directly.

static func run_all(tree_parent: Node = null) -> bool:
	var results: Array[bool] = []
	results.append(CombatUiFrameworkTestSuite.run(tree_parent))
	results.append(CoreRulesTestSuite.run())
	results.append(ClassRulesTestSuite.run())
	results.append(CombatExtensionsTestSuite.run())
	results.append(RiposteTestSuite.run())
	results.append(SkillUnlockTestSuite.run())
	results.append(GearBudgetTestSuite.run())
	results.append(FocusGearTestSuite.run())
	results.append(SkillFamilyTestSuite.run())
	results.append(SkillRanksTestSuite.run())
	results.append(SkillTextTestSuite.run())
	results.append(SkillModelTestSuite.run())
	results.append(OriginTestSuite.run())
	results.append(ItemFamiliesTestSuite.run())
	results.append(ItemModifiersTestSuite.run())
	results.append(SkillFormModelTestSuite.run())
	results.append(EnginePiecesTestSuite.run())
	results.append(StatusRegistryTestSuite.run())
	results.append(WeakenTestSuite.run())
	results.append(Bal2RulesTestSuite.run())
	results.append(Bal2SimTestSuite.run())
	results.append(AiSkillsTestSuite.run())
	results.append(SkillEngineTestSuite.run())
	results.append(ProgressionTestSuite.run())
	results.append(SaveTestSuite.run())
	results.append(FoundationTestSuite.run())
	results.append(CharacterSpecsTestSuite.run(tree_parent))
	results.append(GearEquipTestSuite.run(tree_parent))
	results.append(CombatSessionFlowTestSuite.run(tree_parent))
	results.append(ArtTerrainTestSuite.run(tree_parent))
	results.append(PlanningTestSuite.run(tree_parent))
	results.append(PlanClaimsTestSuite.run())
	results.append(ContentTestSuite.run(tree_parent))
	results.append(RunnerTestSuite.run(tree_parent))
	results.append(WorldTestSuite.run(tree_parent))
	results.append(QuestTestSuite.run())
	results.append(QuestRuntimeTestSuite.run())
	results.append(ExplorationTestSuite.run(tree_parent))
	return not results.has(false)
