class_name CombatHudMetrics
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat HUD metrics (COMBAT-UI3, layout A "slim top strip"; was layout C of COMBAT-UI2-L1)
# The one place the combat screen's sizes come from: pure math from the view size, the text scale
# and the unit counts, so the dock host, the panels and the tests read the same numbers. Every number a layout
# review may want to change is a constant here (UI preferences, not final: docs/preferences/ui.md).
#   top row      the planning strip (`top_h`: one control line, at least STRIP_MIN_H)
#   centre row   party column (`left_w`: compact cards + the selected-hero panel) | grid (the biggest tile that fits)
#                | hostile column (`right_w`: hostile cards + the merged enemy card)
#   bottom row   the one-line status bar (playback buttons, phase, hint, log ticker), `bottom_h`
# Side columns grow with the text size (their text never clips); the grid takes what is left.
# ==============================================================================

const LEFT_W: int = 200         # party column at Normal text (x text scale above Normal)
const RIGHT_W: int = 240        # hostile column at Normal text (x text scale above Normal)
const MARGIN_H: int = 12
const MARGIN_V: int = 6
const ROW_SEP: int = 6          # between the three rows of the screen
const COL_SEP: int = 10         # between the columns and the grid
const MIN_TILE: int = 48
const FRAME: int = 4            # grid frame padding (2 + 2)
const GRID_COLS: int = 12
const GRID_ROWS: int = 9

# Planning strip (top row)
const STRIP_MIN_H: int = 44
const STRIP_PAD_V: int = 6      # strip panel margin above and below its one line of controls
const STRIP_PAD_H: int = 6
const STRIP_SEP: int = 4        # between the strip's controls
const CTRL_EXTRA_H: int = 8     # a strip button / dropdown / block: one text line plus this
const BLOCK_GAP: int = 4        # between two blocks of the plan
const BLOCK_MIN_W: int = 28
const BLOCK_MAX_W: int = 150
const OPTION_EM: float = 7.0    # a tactic dropdown's width in font sizes
const TITLE_EM: float = 9.0     # the strip title ("Warrior plan") in font sizes; longer names end in an ellipsis
const ARROW_W: int = 22         # the round arrows at both ends of the strip
const BLOCK_CARD_W: int = 280   # the block hover card at Normal text (x text scale above Normal)
const QUESTION_W: int = 360     # the confirm / redraft question popup at Normal text (x text scale above Normal)
const SHORT_ABOVE: float = 1.1  # above this text scale the strip drops its dropdown captions and shortens its buttons

# Bottom status bar
const BOTTOM_MIN_H: int = 32
const TICKER_SHARE: float = 0.34   # the log ticker's share of the bar's free width

# Side columns
const PARTY_SLOTS: int = 3      # fallback slot count when the battle names no party cap; never fewer than the heroes present
const HOSTILE_SLOTS: int = 5    # cards the column is sized for
const PARTY_ROWS: int = 3       # compact party card: name + level, HP bar, "HP x/y · AP a/b"
const SLIM_ROWS: int = 1        # a muted empty party slot
const HOSTILE_ROWS: int = 2     # hostile card: name + HP, the "Next:" line
const HERO_MIN_LINES: int = 4   # the selected-hero panel keeps at least this many lines (else it scrolls)
const ENEMY_MIN_LINES: int = 7  # the merged enemy card keeps at least this many lines before the hostile list scrolls
const SKILL_NAMES_MAX: int = 5  # "Can use:" names on the enemy card before "+n more"

const PANEL_PAD: int = 4
const CARD_PAD: int = 4
const CARD_GAP: int = 3
const ROW_GAP: int = 1
const COL_GAP: int = 6

var view: Vector2 = Vector2.ZERO
var scale: float = 1.0
var fs: int = 14                # HUD font size at this text size
var lh: int = 19                # one text line
var ctrl_h: int = 27            # a strip control (button, dropdown, block)
var top_h: int = STRIP_MIN_H
var bottom_h: int = BOTTOM_MIN_H
var left_w: int = LEFT_W
var right_w: int = RIGHT_W
var area_h: int = 0             # height of the centre row
var area_w: int = 0             # width left for the grid
var tile: int = MIN_TILE
var party_slots: int = PARTY_SLOTS
var party_card_h: int = 0
var party_slim_h: int = 0       # a muted empty slot
var party_panel_h: int = 0
var hero_panel_h: int = 0       # the selected-hero panel under the party cards
var hostile_slots: int = HOSTILE_SLOTS
var hostile_card_h: int = 0
var hostile_panel_h: int = 0
var hostile_scroll: bool = false   # more hostiles than the column holds next to the enemy card
var enemy_card_h: int = 0       # the merged enemy card under the hostile cards

# The biggest tile for a grid area (frame excluded): floor(min(h / 9, w / 12)), never below MIN_TILE.
static func tile_for(area: Vector2) -> int:
	return maxi(MIN_TILE, floori(minf(area.y / float(GRID_ROWS), area.x / float(GRID_COLS))))

# One text line at this size: the default theme font's height (what a Label of that size measures).
static func line_height(font_px: int) -> int:
	return ceili(ThemeDB.fallback_font.get_height(font_px))

static func card_height(rows: int, line: int) -> int:
	return rows * line + (rows - 1) * ROW_GAP + 2 * CARD_PAD

# A width that grows with the text size above Normal (never shrinks below the Normal width).
static func grown(px: int, p_scale: float) -> int:
	return roundi(float(px) * maxf(1.0, p_scale))

# The strip's controls at a font size: one text line plus the flat style's margins.
static func control_height(font_px: int) -> int:
	return line_height(font_px) + CTRL_EXTRA_H

static func strip_height(font_px: int) -> int:
	return maxi(STRIP_MIN_H, control_height(font_px) + 2 * STRIP_PAD_V)

static func compute(p_view: Vector2, p_scale: float, party_n: int, hostile_n: int, party_cap: int = 0) -> CombatHudMetrics:
	var m := CombatHudMetrics.new()
	m.view = p_view
	m.scale = p_scale
	m.fs = roundi(float(UiTheme.HUD_FONT) * p_scale)
	m.lh = line_height(m.fs)
	m.ctrl_h = control_height(m.fs)
	m.top_h = strip_height(m.fs)
	m.bottom_h = maxi(BOTTOM_MIN_H, m.lh + 12)
	m.left_w = grown(LEFT_W, p_scale)
	m.right_w = grown(RIGHT_W, p_scale)
	m.area_h = int(p_view.y) - 2 * MARGIN_V - m.top_h - m.bottom_h - 2 * ROW_SEP
	m.area_w = int(p_view.x) - 2 * MARGIN_H - m.left_w - m.right_w - 2 * COL_SEP
	m.tile = tile_for(Vector2(m.area_w - FRAME, m.area_h - FRAME))
	m._size_party(party_n, party_cap)
	m._size_hostile(hostile_n)
	return m

# Party column: the compact cards on top (fixed), the selected-hero panel takes the rest of the column.
func _size_party(party_n: int, party_cap: int) -> void:
	party_slots = maxi(party_cap if party_cap >= 1 else PARTY_SLOTS, party_n)
	party_card_h = card_height(PARTY_ROWS, lh)
	party_slim_h = card_height(SLIM_ROWS, lh)
	var empty: int = party_slots - party_n
	party_panel_h = 2 * PANEL_PAD + lh + party_slots * CARD_GAP + party_n * party_card_h + empty * party_slim_h
	hero_panel_h = area_h - party_panel_h - COL_GAP

# Hostile column: the cards for HOSTILE_SLOTS hostiles, the merged enemy card underneath keeps ENEMY_MIN_LINES.
func _size_hostile(hostile_n: int) -> void:
	hostile_slots = maxi(HOSTILE_SLOTS, hostile_n)
	hostile_card_h = card_height(HOSTILE_ROWS, lh)
	hostile_panel_h = 2 * PANEL_PAD + lh + hostile_slots * (hostile_card_h + CARD_GAP)
	var budget: int = area_h - COL_GAP - (2 * PANEL_PAD + ENEMY_MIN_LINES * lh)
	if hostile_panel_h > budget:
		hostile_scroll = true
		hostile_panel_h = budget
	enemy_card_h = area_h - hostile_panel_h - COL_GAP
