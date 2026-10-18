class_name BattleResultsModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle results model (UI2-L rule 8): XP and levels per member from the award dict
# (LevelUp.award_encounter / BattleBridge.apply()["xp"]: member id -> {xp, old_level, new_level, unspent}); only listed
# (active) members show. The level-up prompt is skippable; continue_on() autosaves and returns to the caller.
# `outcome` (won / lost / fled, BATTLE1-L) sets the headline; only a won battle awards XP.
# BATTLE-FLOW: the screen is a popup over the combat arena; each row carries the XP bar values (fill = progress in the
# current level, gain = the part of it this battle earned), `rewards` lists every non-XP reward the battle gave, and
# `minimised` / `replay_handler` drive the minimise pill and Replay (the host wires the handler to the arena).
# BREPORT1-L: `report` is the battle's BattleReport dict ({} = none; a dict with BattleReport.problems is treated as none);
# report_columns / report_rows / best_moment give the view its table, cells ("-" for 0) and hover tips.
# ==============================================================================

var router: MenuRouter = null
var award: Dictionary = {}
var advanced: Array[String] = []   # quest qids advanced by this battle
var outcome: String = "won"
var rewards: Dictionary = {}       # non-XP rewards: gold, supplies, items {ref: count}, loot (qid or Array), reputation {faction: delta}
var minimised: bool = false
var replay_handler: Callable = Callable()   # set while the arena is still mounted
var report: Dictionary = {}        # BREPORT1: the battle report ({} = no report section)

const HEADLINES: Dictionary = {"won": "Victory", "lost": "Defeat", "fled": "Fled"}
const REWARD_KEYS: Array[String] = ["gold", "supplies", "items", "loot", "reputation"]
const REPORT_LABELS: Dictionary = {"name": "Hero", "dealt": "Damage", "taken": "Taken", "healed": "Healed", "effects": "Effects", "combos": "Combos"}
const NOTES: Dictionary = {
	"lost": "The party woke at the last safe place, fully healed. No XP.",
	"fled": "The party got away. No XP.",
}

func _init(p_router: MenuRouter, p_award: Dictionary, p_advanced: Array[String] = [], p_outcome: String = "won", p_rewards: Dictionary = {}, p_report: Dictionary = {}) -> void:
	router = p_router
	award = p_award
	advanced = p_advanced
	outcome = p_outcome
	rewards = p_rewards
	report = p_report if BattleReport.problems(p_report).is_empty() else {}

# The non-XP reward keys of a settled fight's result dict (the router args carry them as `rewards`).
static func rewards_of(res: Dictionary) -> Dictionary:
	var out: Dictionary = DictUtil.dict_of(res.get("rewards", {})).duplicate(true)
	for key: String in REWARD_KEYS:
		if res.has(key) and not out.has(key):
			out[key] = res[key]
	return out

func headline() -> String:
	return str(HEADLINES.get(outcome, outcome))

# One line on what a lost or fled battle meant ("" when won).
func outcome_note() -> String:
	return str(NOTES.get(outcome, ""))

# [{id, name, xp, old_level, new_level, unspent, fill, gain, to_next, maxed}] in roster order; unspent is read live (it
# drops after a level-up). fill / gain are 0..1 shares of the current level's XP span: the bar's filled part and the
# part of it this battle earned (a level-up counts only the XP inside the new level).
func rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var gs: GameState = CurrentGame.state()
	if gs == null:
		return out
	for m: UnitState in gs.party.members:
		if not award.has(m.id):
			continue
		var a: Dictionary = DictUtil.dict_of(award[m.id])
		var xp: int = DictUtil.to_int(a.get("xp", 0))
		var level: int = m.level()
		var base: int = XpCurve.xp_for_level(level)
		var span: int = XpCurve.xp_for_level(level + 1) - base
		var maxed: bool = level >= StatRules.MAX_LEVEL or span <= 0
		var fill: float = 1.0 if maxed else clampf(float(m.total_xp - base) / float(span), 0.0, 1.0)
		var gain: float = 0.0 if maxed else clampf(float(m.total_xp - maxi(m.total_xp - xp, base)) / float(span), 0.0, fill)
		out.append({"id": m.id, "name": PartyPreview.name_of(m), "xp": xp, "level": level,
			"old_level": DictUtil.to_int(a.get("old_level", 0)), "new_level": DictUtil.to_int(a.get("new_level", 0)), "unspent": m.unspent(),
			"fill": fill, "gain": gain, "to_next": XpCurve.xp_to_next(m.total_xp), "maxed": maxed, "total_xp": m.total_xp})
	return out

func prompt_level_up() -> bool:
	for r: Dictionary in rows():
		if DictUtil.to_int(r["new_level"]) > DictUtil.to_int(r["old_level"]) and DictUtil.to_int(r["unspent"]) > 0:
			return true
	return false

# Quests this battle advanced, by resolved name.
func rewards_text() -> String:
	var names: PackedStringArray = PackedStringArray()
	var gs: GameState = CurrentGame.state()
	for qid: String in advanced:
		names.append(str(QuestRunner.resolved(gs, qid).get("name", qid)) if gs != null else qid)
	return "Quests advanced: " + ", ".join(names) if not names.is_empty() else ""

# Every non-XP reward as a line, in a fixed order: gold, supplies, items, loot, reputation, quest progress.
func reward_lines() -> Array[String]:
	var out: Array[String] = []
	if DictUtil.to_int(rewards.get("gold", 0)) > 0:
		out.append("Gold +%d" % DictUtil.to_int(rewards["gold"]))
	if DictUtil.to_int(rewards.get("supplies", 0)) > 0:
		out.append("Supplies +%d" % DictUtil.to_int(rewards["supplies"]))
	var items: Dictionary = DictUtil.dict_of(rewards.get("items", {}))
	for ref: Variant in items:
		if DictUtil.to_int(items[ref]) > 0:
			out.append("%s x%d" % [_pretty(str(ref)), DictUtil.to_int(items[ref])])
	var loot: Variant = rewards.get("loot", "")
	var loot_refs: Array[String] = JsonUtil.strings(loot)
	if not (loot is Array):
		loot_refs = [str(loot)]
	for ref: String in loot_refs:
		if not ref.is_empty():
			out.append("Loot: %s" % _pretty(ref))
	var rep: Dictionary = DictUtil.dict_of(rewards.get("reputation", {}))
	for faction: Variant in rep:
		var delta: int = DictUtil.to_int(rep[faction])
		if delta != 0:
			out.append("Reputation, %s: %s%d" % [_pretty(str(faction)), "+" if delta > 0 else "", delta])
	var quests: String = rewards_text()
	if not quests.is_empty():
		out.append(quests)
	return out

static func _pretty(ref: String) -> String:
	var tail: String = ref.get_slice(":", ref.get_slice_count(":") - 1)
	return tail.replace("_", " ").capitalize()

# ------------------------------------------------------------------------------
# Battle report (BREPORT1-L 1.5)
# ------------------------------------------------------------------------------

func has_report() -> bool:
	return not report.is_empty() and not DictUtil.array_of(report.get("rows", [])).is_empty()

# [{id, label}] of the shown columns, in table order.
func report_columns() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if not has_report():
		return out
	for raw: Variant in DictUtil.array_of(report.get("columns", [])):
		var id: String = str(raw)
		out.append({"id": id, "label": str(REPORT_LABELS.get(id, id))})
	return out

# [{name, state, guest, cells: {id: text}, tips: {id: text}}] per row: the name cell carries " (guest)" and the
# "down" / "left" tag, numbers show "-" for 0, tips are "" when there is nothing to add.
func report_rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if not has_report():
		return out
	for raw: Variant in DictUtil.array_of(report.get("rows", [])):
		var row: Dictionary = DictUtil.dict_of(raw)
		var state: String = str(row.get("state", "standing"))
		var guest: bool = DictUtil.to_bool(row.get("guest", false))
		var name_cell: String = str(row.get("name", "")) + (" (guest)" if guest else "") + ("" if state == "standing" else " (%s)" % state)
		var cells: Dictionary = {"name": name_cell}
		for id: String in ["dealt", "taken", "healed", "effects", "combos"]:
			var n: int = DictUtil.int_of(row, id)
			cells[id] = "-" if n == 0 else str(n)
		var tips: Dictionary = {"name": "", "dealt": BattleReport.dealt_tip(row), "taken": BattleReport.taken_tip(row), "healed": BattleReport.healed_tip(row),
			"effects": BattleReport.effects_tip(row), "combos": BattleReport.combos_tip(row)}
		out.append({"name": str(row.get("name", "")), "state": state, "guest": guest, "cells": cells, "tips": tips})
	return out

# "Best moment: ..." or "" (no report, or no moment).
func best_moment() -> String:
	var best: String = str(report.get("best", "")) if has_report() else ""
	return "" if best.is_empty() else "Best moment: " + best

# ------------------------------------------------------------------------------
# Popup state (BATTLE-FLOW)
# ------------------------------------------------------------------------------

func set_minimised(value: bool) -> void:
	if minimised == value:
		return
	minimised = value
	router.changed.emit()

func toggle_minimised() -> void:
	set_minimised(not minimised)

func can_replay() -> bool:
	return replay_handler.is_valid()

# Replay: the arena replays the battle from round 1 while the popup is minimised (the pill brings it back).
func replay() -> Array[String]:
	var errors: Array[String] = []
	if not can_replay():
		errors.append("The battle is no longer on screen to replay.")
		return errors
	minimised = true
	var _started: Variant = replay_handler.call()
	router.changed.emit()
	return errors

func level_up_now(id: String) -> Array[String]:
	var errors: Array[String] = []
	if not award.has(id):
		errors.append("'%s' took no part in this battle." % id)
	else:
		router.push(MenuRouter.LEVEL_UP, {"member": id})
	return errors

# UI2-L rule 4: a member with a free class or extra loadout slot can slot a newly unlocked family.
func has_free_slot(id: String) -> bool:
	if PartyPreview.member(id) == null:
		return false
	var lm := LoadoutModel.new(id)
	return lm.class_slots.size() < StatRules.MAX_CLASS_SLOTS or lm.extra_slots.size() < StatRules.MAX_EXTRA_SLOTS

func open_loadout(id: String) -> Array[String]:
	var errors: Array[String] = []
	if not award.has(id):
		errors.append("'%s' took no part in this battle." % id)
	else:
		router.push(MenuRouter.LOADOUT, {"member": id})
	return errors

func skip() -> Array[String]:
	return continue_on()

var _done: bool = false

func continue_on() -> Array[String]:
	if _done:
		return []
	_done = true
	var errors: Array[String] = SaveGame.autosave(CurrentGame.state())
	if errors.is_empty():
		CurrentGame.mark_saved(SaveGame.AUTOSAVE_SLOT)
	router._pop_now()
	router.results_done.emit()
	return errors
