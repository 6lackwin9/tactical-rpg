//! FLOATPANELS1 (`PR7-33`; `preferences/ui.md` "Floating panels", `P-UI-03`): the game's floating panels on the engine's
//! `aegis_ui::float_panel` (the region's "Here (N)" legend and event ticker, the battle log; the world map's legend is
//! its own). One frame's bookkeeping: the place and lock kept in the settings under the panel's key, else its default
//! corner, locked; a change the player made is saved.

use crate::screens::Cx;
use aegis_ui::canvas::Rect;
use aegis_ui::float_panel::FloatPanel;
use serde_json::Value;

/// The settings keys the floating panels' place and lock are kept under (`{"pos": [x, y], "locked": bool}`).
pub const REGION_LEGEND_KEY: &str = "region_legend";
pub const REGION_LOG_KEY: &str = "region_log";
pub const BATTLE_LOG_KEY: &str = "battle_log";

/// One frame of panel `fp` (kept under settings `key`, baseline `kept`) at `size` (chip `chip`) inside `area`: on the
/// first frame the kept place and lock, else `default` (the panel's top left), locked; then the drag (`held`, the
/// pointer at `mouse`) and the clamp; a change the player made is written to the settings and saved.
#[allow(clippy::too_many_arguments)]
pub fn frame(cx: &mut Cx, fp: &mut FloatPanel, kept: &mut Option<Value>, key: &str, area: Rect, default: (i32, i32), size: (i32, i32), chip: (i32, i32), mouse: (i32, i32), held: bool) {
    if kept.is_none() && fp.pos.is_none() {
        let stored = cx.settings.v[key].clone();
        if !fp.restore_saved(&stored, area, size) {
            fp.pos = Some(default);
            fp.locked = true;
        }
    }
    if let Some(v) = fp.keep(kept, &Value::Null, area, size, chip, &[], mouse, held) {
        cx.settings.v[key] = v;
        if let Err(e) = cx.settings.save() {
            cx.log(format!("Settings could not be saved: {e}"));
        }
    }
}
