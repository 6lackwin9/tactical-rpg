class_name CombatHudMetrics
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat HUD metrics (COMBAT-UI2-L1, layout C "compact columns")
# The one place the combat screen's sizes come from: pure math from the view size, the text scale
# and the unit counts, so the dock host, the panels and the tests read the same numbers.
#   top row      timeline, fixed TOP_H
#   centre row   party column (SIDE_W) | grid (the biggest tile that fits) | hostile + forecast column (SIDE_W)
#   bottom row   playback buttons + the one-line log ticker, fixed `bottom_h`
# Panel heights are fixed per text size. As text grows, cards drop lines in the order the spec gives:
# speed/reach, effects, the hostile behaviour line (all of them stay in the hover card).
# The party column takes the whole centre-row height (the old inspector region is gone, COMBAT-UI2-L2).
# ==============================================================================

const SIDE_W: int = 220
const MARGIN_H: int = 14
const MARGIN_V: int = 4
const ROW_SEP: int = 4      # between the three rows of the screen
const COL_SEP: int = 10     # between the columns and the grid
const TOP_H: int = 96
const MIN_TILE: int = 48
const FRAME: int = 4        # grid frame padding (2 + 2)
const GRID_COLS: int = 12
const GRID_ROWS: int = 9

const PARTY_SLOTS: int = 3      # fallback slot count when the battle names no party cap; never fewer than the heroes present
const HOSTILE_SLOTS: int = 5    # cards the column is sized for
const PARTY_ROWS_MAX: int = 6   # name, stance + plan, HP, AP, effects, speed + reach
const PARTY_ROWS_MIN: int = 4
const HOSTILE_ROWS_MAX: int = 3 # name + stance, HP, behaviour
const HOSTILE_ROWS_MIN: int = 2
const FORECAST_LINES: int = 5   # the forecast scrolls when its text needs more

const PANEL_PAD: int = 4
const CARD_PAD: int = 4
const CARD_GAP: int = 3
const ROW_GAP: int = 1
const COL_GAP: int = 6

var view: Vector2 = Vector2.ZERO
var scale: float = 1.0
var fs: int = 14                # HUD font size at this text size
var lh: int = 19                # one text line
var bottom_h: int = 36
var area_h: int = 0             # height of the centre row
var area_w: int = 0             # width left for the grid
var tile: int = MIN_TILE
var party_slots: int = PARTY_SLOTS
var party_rows: int = PARTY_ROWS_MAX
var party_card_h: int = 0
var party_slim_h: int = 0       # a muted empty slot
var party_panel_h: int = 0
var hostile_slots: int = HOSTILE_SLOTS
var hostile_rows: int = HOSTILE_ROWS_MAX
var hostile_card_h: int = 0
var hostile_panel_h: int = 0
var hostile_scroll: bool = false   # more hostiles than the column holds even at the fewest lines
var forecast_h: int = 0

# The biggest tile for a grid area (frame excluded): floor(min(h / 9, w / 12)), never below MIN_TILE.
static func tile_for(area: Vector2) -> int:
	return maxi(MIN_TILE, floori(minf(area.y / float(GRID_ROWS), area.x / float(GRID_COLS))))

# One text line at this size: the default theme font's height (what a Label of that size measures).
static func line_height(font_px: int) -> int:
	return ceili(ThemeDB.fallback_font.get_height(font_px))

static func card_height(rows: int, line: int) -> int:
	return rows * line + (rows - 1) * ROW_GAP + 2 * CARD_PAD

static func compute(p_view: Vector2, p_scale: float, party_n: int, hostile_n: int, party_cap: int = 0) -> CombatHudMetrics:
	var m := CombatHudMetrics.new()
	m.view = p_view
	m.scale = p_scale
	m.fs = roundi(float(UiTheme.HUD_FONT) * p_scale)
	m.lh = line_height(m.fs)
	m.bottom_h = maxi(36, m.lh + 16)
	m.area_h = int(p_view.y) - 2 * MARGIN_V - TOP_H - m.bottom_h - 2 * ROW_SEP
	m.area_w = int(p_view.x) - 2 * MARGIN_H - 2 * SIDE_W - 2 * COL_SEP
	m.tile = tile_for(Vector2(m.area_w - FRAME, m.area_h - FRAME))
	m._size_party(party_n, party_cap)
	m._size_hostile(hostile_n)
	return m

# Party column: as many lines per card as fit in the centre-row height; the panel fills the column.
func _size_party(party_n: int, party_cap: int) -> void:
	party_slots = maxi(party_cap if party_cap >= 1 else PARTY_SLOTS, party_n)
	var filled: int = party_n
	var empty: int = party_slots - filled
	party_slim_h = card_height(2, lh)
	var budget: int = area_h
	party_rows = PARTY_ROWS_MIN
	for r: int in range(PARTY_ROWS_MAX, PARTY_ROWS_MIN - 1, -1):
		if _party_panel_h(r, filled, empty) <= budget:
			party_rows = r
			break
	party_card_h = card_height(party_rows, lh)
	party_panel_h = area_h   # the freed inspector space goes back to the party column

func _party_panel_h(rows: int, filled: int, empty: int) -> int:
	return 2 * PANEL_PAD + lh + (filled + empty) * CARD_GAP + filled * card_height(rows, lh) + empty * party_slim_h

# Hostile column: lines per card chosen so the forecast keeps room for its lines underneath.
func _size_hostile(hostile_n: int) -> void:
	hostile_slots = maxi(HOSTILE_SLOTS, hostile_n)
	var forecast_min: int = 2 * PANEL_PAD + lh + CARD_GAP + FORECAST_LINES * lh
	var budget: int = area_h - forecast_min - COL_GAP
	hostile_rows = HOSTILE_ROWS_MIN
	for r: int in range(HOSTILE_ROWS_MAX, HOSTILE_ROWS_MIN - 1, -1):
		if _hostile_panel_h(r) <= budget:
			hostile_rows = r
			break
	hostile_card_h = card_height(hostile_rows, lh)
	hostile_panel_h = _hostile_panel_h(hostile_rows)
	if hostile_panel_h > budget:
		hostile_scroll = true
		hostile_panel_h = budget
	forecast_h = area_h - hostile_panel_h - COL_GAP

func _hostile_panel_h(rows: int) -> int:
	return 2 * PANEL_PAD + lh + hostile_slots * (card_height(rows, lh) + CARD_GAP)
