class_name BattleResultsModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle results model (UI2-L rule 8): XP and levels per member from the award dict
# (LevelUp.award_encounter / BattleBridge.apply()["xp"]: member id -> {xp, old_level, new_level, unspent}); only listed
# (active) members show. The level-up prompt is skippable; continue_on() autosaves and returns to the caller.
# `outcome` (won / lost / fled, BATTLE1-L) sets the headline; only a won battle awards XP.
# ==============================================================================

var router: MenuRouter = null
var award: Dictionary = {}
var advanced: Array[String] = []   # quest qids advanced by this battle
var outcome: String = "won"

const HEADLINES: Dictionary = {"won": "Victory", "lost": "Defeat", "fled": "Withdrew"}
const NOTES: Dictionary = {
	"lost": "The party woke at the last safe place, fully healed. No XP.",
	"fled": "The party got away. No XP.",
}

func _init(p_router: MenuRouter, p_award: Dictionary, p_advanced: Array[String] = [], p_outcome: String = "won") -> void:
	router = p_router
	award = p_award
	advanced = p_advanced
	outcome = p_outcome

func headline() -> String:
	return "Battle results: %s" % str(HEADLINES.get(outcome, outcome))

# One line on what a lost or fled battle meant ("" when won).
func outcome_note() -> String:
	return str(NOTES.get(outcome, ""))

# [{id, name, xp, old_level, new_level, unspent}] in roster order; unspent is read live (it drops after a level-up).
func rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var gs: GameState = CurrentGame.state()
	if gs == null:
		return out
	for m: UnitState in gs.party.members:
		if not award.has(m.id):
			continue
		var a: Dictionary = DictUtil.dict_of(award[m.id])
		out.append({"id": m.id, "name": PartyPreview.name_of(m), "xp": DictUtil.to_int(a.get("xp", 0)),
			"old_level": DictUtil.to_int(a.get("old_level", 0)), "new_level": DictUtil.to_int(a.get("new_level", 0)), "unspent": m.unspent()})
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

func level_up_now(id: String) -> Array[String]:
	var errors: Array[String] = []
	if not award.has(id):
		errors.append("'%s' took no part in this battle." % id)
	else:
		router.push(MenuRouter.LEVEL_UP, {"member": id})
	return errors

func skip() -> Array[String]:
	return continue_on()

func continue_on() -> Array[String]:
	var errors: Array[String] = SaveGame.autosave(CurrentGame.state())
	if errors.is_empty():
		CurrentGame.mark_saved(SaveGame.AUTOSAVE_SLOT)
	router._pop_now()
	router.results_done.emit()
	return errors
