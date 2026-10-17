class_name NewGameView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - New game view (UI1-L): one card per playable world with its fixed starting party; Start = choose().
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: NewGameModel = model as NewGameModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("New game"))
	var worlds: Array[Dictionary] = m.worlds()
	if worlds.is_empty():
		add_child(MainMenuView.text("No world is playable yet (a world needs a start party and a start location).", UiTheme.COL_WARN))
	for w: Dictionary in worlds:
		var card: VBoxContainer = UiTheme.card(self)
		card.add_child(MainMenuView.text(str(w["name"]), UiTheme.COL_HEADER))
		if not str(w["tagline"]).is_empty():
			card.add_child(MainMenuView.text(str(w["tagline"]), UiTheme.COL_MUTED))
		var names: PackedStringArray = PackedStringArray()
		for raw: Variant in DictUtil.array_of(w["party"]):
			var p: Dictionary = DictUtil.dict_of(raw)
			var cls: String = str(p["class"])
			names.append("%s (%sLv %d)" % [str(p["name"]), "" if cls.is_empty() or cls == str(p["name"]) else cls + ", ", DictUtil.to_int(p["level"])])
		card.add_child(MainMenuView.text("Party: " + ", ".join(names)))
		card.add_child(MainMenuView.button("Start in %s" % str(w["name"]), host.act.bind(m.choose.bind(str(w["id"]), -1))))
	add_child(MainMenuView.button("Back", host.router.pop))
