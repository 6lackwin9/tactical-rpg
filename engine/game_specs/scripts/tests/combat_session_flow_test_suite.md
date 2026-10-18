---
class: CombatSessionFlowTestSuite
extends: RefCounted
description: "TEST 5: COMBAT SESSION FLOW (catalog Goblin Ambush, headless)"
---

# Methods

### `static run(_tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING COMBAT SESSION FLOW TEST (GOBLIN AMBUSH) ===")
print("=======================================================")

var encounter := CatalogEncounter.new("goblin_ambush")
var session := CombatSession.new(encounter)
session.start()
var warrior: Combatant = encounter.unit("warrior")

# 1. Before the wave only the initial enemies are in view
var pre_view_ok: bool = session.enemies_in_view().size() == 2 and session.phase == CombatSession.Phase.PLANNING

# 2. The wave enters exactly at Round 1, Block 4 and brings 4 goblins into view
session.plan_apply_preset_all("balanced")
var ambush: ActionLogEntry = null
for i: int in range(300):
	session.step_next()
	if session.log_entries.is_empty():
		continue
	var last: ActionLogEntry = session.log_entries[session.log_entries.size() - 1]
	if last.outcome_tag == "ARRIVED":
		ambush = last
		break
var ambush_ok: bool = ambush != null and ambush.ref_tag == "R1-B04" and session.enemies_in_view().size() == 6

# 3. Round 1 resolves into replay mode with the round advance offered
TestHelpers.resolve_until_replay(session)
var round1_ok: bool = (
	session.phase == CombatSession.Phase.REPLAY and not session.combat_over
	and session.can_advance_round() and not session.next_round_label.is_empty()
	and warrior.is_alive()
)

# 4. Replay works by log points: a block opens at its start (state after the blocks
# before it), a round opens at its start, and stepping walks the log forward and back
var snap0: BlockSnapshot = session.snapshot_at(1, 0)
var snap1: BlockSnapshot = session.snapshot_at(1, 1)
var start_hp: int = snap0.hps.get(warrior, -1)
session.select_block(2)
var at_block2: bool = snap1 != null and session.position_of(warrior) == snap1.positions.get(warrior, Vector2i(-1, -1))
session.select_block(1)
var at_start: bool = session.replay_cursor == -1 and session.display_snapshot() == snap0 and session.position_of(warrior) == snap0.positions.get(warrior, Vector2i(-1, -1))
session.step_next()
var first_point: int = session.replay_cursor
var stepped: bool = first_point >= 0 and session.log_entries[first_point].round_num == 1 and session.display_snapshot() != snap0
session.step_prev()
var back_to_start: bool = session.replay_cursor == -1
var replay_ok: bool = at_block2 and at_start and stepped and back_to_start and start_hp == warrior.max_hp

# 5. Advancing carries the state into Round 2; Round 1 stays reviewable
var hp_end_r1: int = warrior.hp
session.advance_round()
var r2_live: bool = session.current_round == 2 and session.phase == CombatSession.Phase.PLANNING and warrior.hp == hp_end_r1
session.select_round(1)
var viewing_r1: bool = session.viewed_round == 1 and not session.is_live_view()
session.select_round(2)
var traversal_ok: bool = r2_live and viewing_r1 and session.is_live_view() and session.max_unlocked_round == 2

# 6. Later rounds (AI presets on both sides) play on to a conclusion
for round_num: int in range(encounter.max_rounds):
	session.plan_apply_preset_all("balanced")
	session.confirm_plan()
	TestHelpers.resolve_until_replay(session)
	if session.combat_over or not session.can_advance_round():
		break
	session.advance_round()
var victory_ok: bool = (session.combat_over or session.current_round == encounter.max_rounds) and not session.can_advance_round()

# 7. Auto-play ticks resolve a round without a timer (headless)
session.reset_battle()
session.toggle_auto_play()
for i: int in range(600):
	if not session.is_auto_playing:
		break
	session.auto_play_tick()
var autoplay_ok: bool = session.phase == CombatSession.Phase.REPLAY and not session.is_auto_playing

# 8. Resetting the round restores Round 1 planning with an empty log
session.reset_round()
warrior = encounter.unit("warrior")   # a round-1 reset rebuilds the battle
var reset_ok: bool = session.current_round == 1 and session.log_entries.is_empty() and warrior.hp == warrior.max_hp and session.phase == CombatSession.Phase.PLANNING

print("\n=======================================================")
print("=== COMBAT SESSION FLOW ASSERTIONS                  ===")
print("=======================================================")
print("  [Session] Only deployed enemies in view before the wave: %s" % ("PASS" if pre_view_ok else "FAIL"))
print("  [Session] Wave event at R1-B04 brings reinforcements into view: %s" % ("PASS" if ambush_ok else "FAIL"))
print("  [Session] Round 1 resolves into replay with round advance offered: %s" % ("PASS" if round1_ok else "FAIL"))
print("  [Session] Replay steps by log points; blocks/rounds open at their start: %s" % ("PASS" if replay_ok else "FAIL"))
print("  [Session] Round advance carries state over; past rounds reviewable: %s" % ("PASS" if traversal_ok else "FAIL"))
print("  [Session] Later rounds (AI presets) play on to a conclusion: %s" % ("PASS" if victory_ok else "FAIL"))
print("  [Session] Auto-play ticks resolve a full round headlessly: %s" % ("PASS" if autoplay_ok else "FAIL"))
print("  [Session] Round reset restores planning with an empty log: %s" % ("PASS" if reset_ok else "FAIL"))

session.dispose()
return pre_view_ok and ambush_ok and round1_ok and replay_ok and traversal_ok and victory_ok and autoplay_ok and reset_ok
```

