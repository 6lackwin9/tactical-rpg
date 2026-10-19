class_name GameRolls
extends RefCounted

# ==============================================================================
# Tactical RPG - Game rolls (S9 / S10, spec EXP1-L 2.4): the per-save deterministic random streams.
# Roll n of stream s (n = gs.rolls[s], default 0) is one randi() of a RandomNumberGenerator seeded with
# hash([gs.rng_seed, s, n]); every roll then counts gs.rolls[s] up by one. Same save + same inputs = same rolls,
# also across save / load. Streams: "explore" (ExploreRunner), "travel" (TRAV1).
# ==============================================================================

const EXPLORE: String = "explore"
const TRAVEL: String = "travel"

static func roll(gs: GameState, stream: String) -> int:
	var n: int = DictUtil.to_int(gs.rolls.get(stream, 0))
	var rng := RandomNumberGenerator.new()
	rng.seed = hash([gs.rng_seed, stream, n])
	var value: int = rng.randi()
	gs.rolls[stream] = n + 1
	return value

# An int in lo..hi (inclusive; the bounds may come in either order). One roll.
static func range_i(gs: GameState, stream: String, lo: int, hi: int) -> int:
	var a: int = mini(lo, hi)
	var b: int = maxi(lo, hi)
	return a + posmod(roll(gs, stream), b - a + 1)

# Always one roll: true when range_i(0, 99) < pct.
static func chance(gs: GameState, stream: String, pct: int) -> bool:
	return range_i(gs, stream, 0, 99) < pct

# One roll over [[id, weight], ...] in authored order (weights < 1 count as 0). "" and no roll when nothing is
# pickable.
static func pick(gs: GameState, stream: String, pairs: Array) -> String:
	var total: int = 0
	for p: Variant in pairs:
		total += maxi(0, _weight(p))
	if total <= 0:
		return ""
	var r: int = range_i(gs, stream, 0, total - 1)
	for p: Variant in pairs:
		var w: int = maxi(0, _weight(p))
		if r < w:
			return str(DictUtil.array_of(p)[0])
		r -= w
	return ""

static func _weight(p: Variant) -> int:
	var pair: Array = DictUtil.array_of(p)
	return DictUtil.to_int(pair[1]) if pair.size() == 2 else 0
