extends Node2D

# ==============================================================================
# Tactical RPG - Combat Arena (foundation scene for the combat UI)
# Creates a CombatSession for an encounter and mounts the CombatDockHost UI.
#   Rules & battle flow -> scripts/combat/session/ (CombatSession, CombatEncounter)
#   Panels & layout     -> scripts/ui/combat/      (CombatPanel subclasses, CombatDockHost)
# Callers (game flow, test harness) may assign `encounter` before the scene
# enters the tree (the Scene Runner does), or pass `-- --encounter=<catalog id>`;
# it defaults to the Goblin Skirmish.
# Playtests (PLAYTEST1): set `playtest` (a recording path) and `battle` before the scene enters the tree
# to play a recorded battle back (fast-forwarded by PlaytestReplayer before the UI mounts, so no FX
# flood), plus `take_over` = N to replay rounds 1..N-1 and plan round N live as a branch recording.
# `battle_seed` (>= 0) overrides the encounter seed (scenarios); live battles are followed by the
# PlaytestHub when there is one. A recording that cannot be played back shows its load_error in the banner
# over a live default battle that is not recorded.
# ==============================================================================

var encounter: CombatEncounter = null
var session: CombatSession = null
var host: CombatDockHost = null

var playtest: String = ""          # recording to play back ("" = a live battle)
var battle: int = 0                # battle index in the recording
var take_over: int = -1            # > 0: replay rounds 1..N-1, then plan round N live (branch)
var battle_seed: int = -1          # >= 0: encounter.rng_seed for this battle (scenario seed)
var seed_source: String = ""       # recorded seed_source of a live battle ("" = encounter)
var scenario: String = ""          # scenario id of a live battle
var replayer: PlaytestReplayer = null
var banner: PlaybackBanner = null
var playback_data: Dictionary = {}
var load_error: String = ""

var _canvas: CanvasLayer = null

func _ready() -> void:
	_canvas = CanvasLayer.new()
	add_child(_canvas)
	if not playtest.is_empty() and _ready_playback():
		return
	var unplayable: bool = not playtest.is_empty()   # the recording could not be played back: a live battle, not recorded
	if encounter == null:
		encounter = encounter_from_args(OS.get_cmdline_user_args())
	if encounter == null:
		encounter = CatalogEncounter.new("goblin_skirmish")
	if battle_seed >= 0:
		encounter.rng_seed = battle_seed
	session = CombatSession.new(encounter)
	host = CombatDockHost.new()
	_canvas.add_child(host)
	host.setup(session)
	if unplayable:
		banner = PlaybackBanner.new()
		_canvas.add_child(banner)
		banner.show_error(playtest, load_error)
	var hub: PlaytestHub = PlaytestHub.instance
	if hub != null and not unplayable:
		hub.attach_session(session, "", seed_source, scenario)
	session.start()

# Plays the recording back (or takes it over). False = it could not load: the live default runs instead.
func _ready_playback() -> bool:
	var loaded: Dictionary = PlaytestStore.load_json(playtest)
	var raw: Dictionary = DictUtil.dict_of(loaded.get("data", {}))
	var problems: Array[String] = [str(loaded.get("error", ""))]
	if DictUtil.to_bool(loaded.get("ok", false)):
		problems = PlaytestRecord.errors(raw)
	if problems.is_empty():
		playback_data = PlaytestRecord.normalize(raw)
		replayer = PlaytestReplayer.for_battle(PlaytestRecord.battle_at(playback_data, battle))
		replayer.take_over = take_over
		if not replayer.run():
			problems = replayer.errors
	if not problems.is_empty():
		load_error = "; ".join(problems)
		push_warning("combat_arena: playtest %s not played back: %s" % [playtest, load_error])
		return false
	session = replayer.session
	encounter = session.encounter
	host = CombatDockHost.new()
	_canvas.add_child(host)
	host.setup(session)
	var stale: bool = DictUtil.to_bool(ContentFingerprint.diff(DictUtil.dict_of(playback_data.get("content", {})), ContentFingerprint.compute())["stale"])
	banner = PlaybackBanner.new()
	_canvas.add_child(banner)
	banner.take_over_requested.connect(_on_take_over_requested)
	var hub: PlaytestHub = PlaytestHub.instance
	if take_over > 0:
		banner.setup(session, playback_data, battle, stale, take_over)
		if hub != null:
			hub.start_branch(playback_data, battle, take_over, session)
	else:
		banner.setup(session, playback_data, battle, stale)
		if hub != null:
			hub.show_playback(session, playback_data, battle, playtest)
		session.round_started.connect(_on_playback_round_started)
	return true

# Advancing past the last recorded round in playback is a take-over at that round (rule 5).
func _on_playback_round_started(round_num: int) -> void:
	if round_num <= banner.recorded_rounds:
		return
	session.round_started.disconnect(_on_playback_round_started)
	take_over = round_num
	banner.setup(session, playback_data, battle, banner.stale_badge.visible, round_num)
	var hub: PlaytestHub = PlaytestHub.instance
	if hub != null:
		hub.start_branch(playback_data, battle, round_num, session)

func _on_take_over_requested(round_num: int) -> void:
	var hub: PlaytestHub = PlaytestHub.instance
	if hub != null:
		hub.request_take_over(playtest, battle, round_num)

# `-- --encounter=<id>` loads any content-catalog encounter into the arena, e.g.
#   godot --path . res://scenes/combat_arena.tscn -- --encounter=veterans_drill
static func encounter_from_args(args: PackedStringArray) -> CombatEncounter:
	for arg: String in args:
		if arg.begins_with("--encounter="):
			var id: String = arg.trim_prefix("--encounter=")
			if EncounterSource.exists(id):
				return EncounterSource.create(id)
			push_warning("combat_arena: unknown encounter '%s', using the default." % id)
	return null

func _exit_tree() -> void:
	if session != null:
		session.dispose()
