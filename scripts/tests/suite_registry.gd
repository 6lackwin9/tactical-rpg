class_name SuiteRegistry
extends RefCounted

# Every headless suite in run order. To add a suite: write scripts/tests/<name>_test_suite.gd
# (class_name <Name>TestSuite, static func run(...) -> bool) and add ONE `if _on("<system>"): results.append` line below.
# Nothing else needs editing; scenes/test_arena.gd calls run_all directly.
# Systems (GATE-SPLIT, user 2026-10-18): `tools/fc.sh gate <system,...>` runs only those systems' suites;
# the release PR runs every system as its own parallel CI job. Pick the system the suite's code belongs to.
const SYSTEMS: PackedStringArray = ["rules", "ai", "combat_ui", "balance", "world", "party_save", "content", "menus"]

static var _only: PackedStringArray = []

static func _on(system: String) -> bool:
	return _only.is_empty() or _only.has(system)

# Empty `systems` runs every suite. Unknown names fail the run.
static func run_all(tree_parent: Node = null, systems: PackedStringArray = PackedStringArray()) -> bool:
	for name: String in systems:
		if not SYSTEMS.has(name):
			push_error("SuiteRegistry: unknown system '%s' (known: %s)" % [name, ", ".join(SYSTEMS)])
			return false
	_only = systems
	PlaytestStore.force_enabled = 0   # suites that launch scenes through the Scene Runner never record playtests
	var results: Array[bool] = []
	if _on("combat_ui"): results.append(CombatUiFrameworkTestSuite.run(tree_parent))
	if _on("rules"): results.append(CoreRulesTestSuite.run())
	if _on("rules"): results.append(ClassRulesTestSuite.run())
	if _on("rules"): results.append(CombatExtensionsTestSuite.run())
	if _on("rules"): results.append(CombatExtAttacksTestSuite.run())
	if _on("ai"): results.append(CombatExtAiTestSuite.run())
	if _on("rules"): results.append(RiposteTestSuite.run())
	if _on("rules"): results.append(SkillUnlockTestSuite.run())
	if _on("rules"): results.append(GearBudgetTestSuite.run())
	if _on("rules"): results.append(FocusGearTestSuite.run())
	if _on("rules"): results.append(FocusGearBudgetTestSuite.run())
	if _on("rules"): results.append(SkillFamilyTestSuite.run())
	if _on("rules"): results.append(SkillRanksTestSuite.run())
	if _on("rules"): results.append(SkillTextTestSuite.run())
	if _on("rules"): results.append(SkillModelTestSuite.run())
	if _on("content"): results.append(OriginTestSuite.run())
	if _on("rules"): results.append(ItemFamiliesTestSuite.run())
	if _on("rules"): results.append(ItemModifiersTestSuite.run())
	if _on("rules"): results.append(ItemTiersTestSuite.run())
	if _on("content"): results.append(SkillFormModelTestSuite.run())
	if _on("content"): results.append(ClassPathTestSuite.run())
	if _on("content"): results.append(ClassFormModelTestSuite.run(tree_parent))
	if _on("content"): results.append(ItemFormModelTestSuite.run(tree_parent))
	if _on("rules"): results.append(EnginePiecesTestSuite.run())
	if _on("rules"): results.append(UnarmedTestSuite.run())
	if _on("rules"): results.append(StatusRegistryTestSuite.run())
	if _on("rules"): results.append(WeakenTestSuite.run())
	if _on("rules"): results.append(StatusExpansionTestSuite.run())
	if _on("rules"): results.append(StatusExpansionEffectsTestSuite.run())
	if _on("ai"): results.append(StatusExpansionAiTestSuite.run())
	if _on("rules"): results.append(TerrainRulesTestSuite.run())
	if _on("rules"): results.append(ComboRulesTestSuite.run())
	if _on("balance"): results.append(Bal2RulesTestSuite.run())
	if _on("balance"): results.append(Bal2SimTestSuite.run())
	if _on("balance"): results.append(Bal4CampaignTestSuite.run())
	if _on("combat_ui"): results.append(BattleReportTestSuite.run(tree_parent))
	if _on("ai"): results.append(AiSkillsTestSuite.run())
	if _on("ai"): results.append(AiSkillsTacticsTestSuite.run())
	if _on("rules"): results.append(SkillEngineTestSuite.run())
	if _on("party_save"): results.append(ProgressionTestSuite.run())
	if _on("party_save"): results.append(SaveTestSuite.run())
	if _on("party_save"): results.append(SaveCompatTestSuite.run())
	if _on("party_save"): results.append(HealTestSuite.run())
	if _on("party_save"): results.append(ItemUseTestSuite.run())
	if _on("party_save"): results.append(FoundationTestSuite.run())
	if _on("party_save"): results.append(CharacterSpecsTestSuite.run(tree_parent))
	if _on("party_save"): results.append(GearEquipTestSuite.run(tree_parent))
	if _on("rules"): results.append(CombatSessionFlowTestSuite.run(tree_parent))
	if _on("ai"): results.append(AutoplayTestSuite.run())
	if _on("content"): results.append(ArtTerrainTestSuite.run(tree_parent))
	if _on("ai"): results.append(PlanningTestSuite.run(tree_parent))
	if _on("ai"): results.append(PlanClaimsTestSuite.run())
	if _on("ai"): results.append(IntentTestSuite.run())
	if _on("rules"): results.append(BossPhaseTestSuite.run())
	if _on("rules"): results.append(ObjectiveTestSuite.run())
	if _on("rules"): results.append(MoraleTestSuite.run())
	if _on("content"): results.append(ContentTestSuite.run(tree_parent))
	if _on("menus"): results.append(RunnerTestSuite.run(tree_parent))
	if _on("world"): results.append(WorldTestSuite.run(tree_parent))
	if _on("world"): results.append(QuestTestSuite.run())
	if _on("world"): results.append(QuestRuntimeTestSuite.run())
	if _on("world"): results.append(ExplorationTestSuite.run(tree_parent))
	if _on("world"): results.append(DialogueTestSuite.run(tree_parent))
	if _on("world"): results.append(TravelTestSuite.run(tree_parent))
	if _on("world"): results.append(ReputationTestSuite.run(tree_parent))
	if _on("menus"): results.append(MenusTestSuite.run(tree_parent))
	if _on("menus"): results.append(GameMenuTestSuite.run(tree_parent))
	if _on("world"): results.append(RecruitTestSuite.run())
	if _on("menus"): results.append(SettingsUiTestSuite.run(tree_parent))
	if _on("menus"): results.append(ControlsHelpTestSuite.run(tree_parent))
	if _on("menus"): results.append(AudioSettingsTestSuite.run(tree_parent))
	if _on("menus"): results.append(DisplayTestSuite.run(tree_parent))
	if _on("world"): results.append(BattleLinkTestSuite.run(tree_parent))
	if _on("combat_ui"): results.append(BattleFlowTestSuite.run(tree_parent))
	if _on("party_save"): results.append(PartyScreenTestSuite.run(tree_parent))
	if _on("party_save"): results.append(SaveIoTestSuite.run())
	if _on("party_save"): results.append(JournalTestSuite.run())
	if _on("content"): results.append(AssetPipelineTestSuite.run())
	if _on("content"): results.append(AnimationLifecycleTestSuite.run(tree_parent))
	if _on("menus"): results.append(AudioTestSuite.run(tree_parent))
	if _on("menus"): results.append(SoundPreviewTestSuite.run(tree_parent))
	if _on("menus"): results.append(PlaytestTestSuite.run(tree_parent))
	PlaytestStore.force_enabled = -1
	_only = PackedStringArray()
	return not results.has(false)
