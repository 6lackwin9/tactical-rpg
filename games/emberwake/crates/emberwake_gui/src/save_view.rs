//! UISPLIT1-H (ENGINE-21) Stage A: the save and load screens' view-model (`menus::Slots`). What the slot list shows and
//! offers, as a `screen_layout::View`, for a layout file to place later (Stage B, its own row once the mock is signed
//! off); today's hand drawing stays. Its fixture is `ui/views/slots.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` ("Save game" or "Load game"), `mode` ("save" or "load"), `slots` (a row per slot, the autosave
//! first: its first line as the label, the day of the lead as the detail, its second line as the sub, the hover
//! card's line as the note; on when a click saves or loads; lock or unlock and delete as row buttons, delete off while
//! locked). Actions: `open` (a row's click: save or load by the mode, the slot as arg), `lock`, `unlock`, `delete` and
//! `back`.

use crate::menus::{slot_row, slot_text, slot_when, SlotAct, Slots};
use crate::screens::Cx;
use aegis_core::json;
use aegis_ui::screen_layout::{Act, Field, RowButton, RowView, View};
use emberwake_world::save;

pub const SCREEN: &str = "slots";
pub const OPEN: &str = "open";
pub const LOCK: &str = "lock";
pub const UNLOCK: &str = "unlock";
pub const DELETE: &str = "delete";
pub const BACK: &str = "back";

/// The view of the slot list in save mode (`saving`) or load mode, from the save folder as it is now.
pub fn view(cx: &Cx, saving: bool) -> View {
    let no_save = Slots::no_save_now(cx);
    let rows: Vec<RowView> = (save::AUTOSAVE_SLOT..=save::LAST_SLOT)
        .map(|slot| {
            let head = save::peek(&cx.data, &cx.save_dir, slot);
            let (exists, ok) = (json::b(&head, "exists", false), json::b(&head, "ok", false));
            let locked = save::is_locked(&cx.save_dir, slot);
            let row = slot_row(saving, slot, exists, ok, locked, no_save);
            let (label, status) = slot_text(cx, slot, &head);
            let mut buttons = Vec::new();
            if row.lock {
                buttons.push(if locked { RowButton { action: UNLOCK.into(), label: "Unlock".into(), enabled: true } } else { RowButton { action: LOCK.into(), label: "Lock".into(), enabled: true } });
            }
            if row.delete {
                buttons.push(RowButton { action: DELETE.into(), label: "Delete".into(), enabled: !locked });
            }
            RowView {
                id: slot.to_string(),
                label,
                detail: if ok { slot_when(cx, slot).unwrap_or_default() } else { String::new() },
                sub: status,
                enabled: row.action.is_some(),
                tone: if ok { String::new() } else { "dim".into() },
                note: row.why,
                buttons,
                ..Default::default()
            }
        })
        .collect();
    View::new(SCREEN)
        .field("title", Field::Text(if saving { "Save game" } else { "Load game" }.into()))
        .field("mode", Field::Text(if saving { "save" } else { "load" }.into()))
        .field("slots", Field::Rows(rows))
        .action(OPEN, if saving { "Save" } else { "Load" }, true)
        .action(LOCK, "Lock", true)
        .action(UNLOCK, "Unlock", true)
        .action(DELETE, "Delete", true)
        .action(BACK, "Back", true)
}

/// A view click as the screen's own (`SlotAct`) in save mode (`saving`) or load mode; None for an unknown action or a
/// row with no slot number. Stage B's layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn slot_act(saving: bool, a: &Act) -> Option<SlotAct> {
    let slot = a.arg.as_deref().and_then(|x| x.parse::<i64>().ok());
    match a.action.as_str() {
        OPEN => slot.map(|n| if saving { SlotAct::Save(n) } else { SlotAct::Load(n) }),
        LOCK => slot.map(|n| SlotAct::Lock(n, true)),
        UNLOCK => slot.map(|n| SlotAct::Lock(n, false)),
        DELETE => slot.map(SlotAct::Delete),
        BACK => Some(SlotAct::Back),
        _ => None,
    }
}
