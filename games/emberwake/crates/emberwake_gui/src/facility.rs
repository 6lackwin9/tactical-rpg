//! The facility pane (PR6-8, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` section 4; engine-only, no Godot original;
//! story-free): the NPC service `facility` over the dimmed backdrop in the shop's frame, like the recipe pane: the
//! keeper's name, then each facility it keeps with its level of its top, the effects in force, the next level (its
//! name, effects and cost, why it can't be bought now) and Upgrade; Close at the bottom. The rules are the kit's
//! (`facility::view`, `facility::upgrade`, the runner's `close_facilities`); the pane only draws them. `talk::Services`
//! draws it while its conversation has the facility pane open.

use crate::screens::Cx;
use aegis_core::json::{self, s};
use aegis_ui::canvas::{Rect, Rgba};
use aegis_ui::theme::th;
use aegis_ui::ui::{tip, Ui, BODY, SMALL};
use emberwake_world::facility;
use serde_json::{json, Value};

/// The frame's inner margin (the shop's) and the name's size.
const PAD: i32 = 7;
const HEAD: f32 = 10.0;

/// A click on the facility pane.
#[derive(Debug, Clone)]
pub enum FacilityAct {
    /// Buys the next level of a facility (its qid).
    Upgrade(String),
    /// Back to the conversation.
    Close,
}

/// The pane's own state: the last upgrade's notes and refusal.
#[derive(Debug, Default)]
pub struct FacilityPane {
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

/// The effects of the levels reached of a facility row of `facility::view`, in level order.
fn in_force(row: &json::Dict) -> Vec<String> {
    json::aget(row, "levels").iter().map(json::dict).filter(|l| json::b(l, "reached", false)).flat_map(|l| json::sget(l, "effects")).collect()
}

/// A line of the pane: drawn fitted to `w` and pushed as a label `id` carrying its full text (a shortened line keeps
/// it for the hover and the agent link).
#[allow(clippy::too_many_arguments)]
fn line<A: Clone>(ui: &mut Ui<A>, id: String, text: &str, x: i32, y: i32, size: f32, w: i32, col: Rgba) {
    ui.p.text_fit(x as f32, y as f32, text, size, w as f32, col);
    let h = ui.p.line_h(size).ceil() as i32;
    ui.push(id, "label", text.to_string(), Rect::new(x, y, w, h), true, false, None, Vec::new());
}

impl FacilityPane {
    /// The agent link's reading.
    pub fn state(&self) -> Value {
        json!({"notes": self.notes, "errors": self.errors})
    }

    /// One click: an upgrade (its notes logged and kept; a refusal kept). True for Close.
    pub fn act(&mut self, cx: &mut Cx, a: FacilityAct) -> bool {
        match a {
            FacilityAct::Upgrade(fqid) => {
                let d = cx.data.clone();
                let res = facility::upgrade(&d, cx.gs_mut(), &fqid);
                self.errors = json::sget(&res, "errors");
                self.notes = json::sget(&res, "notes");
                for n in &self.notes {
                    cx.log(n.clone());
                }
                false
            }
            FacilityAct::Close => true,
        }
    }

    /// Draws character `npc`'s facilities in the shop's frame.
    pub fn draw<A: Clone>(&mut self, ui: &mut Ui<A>, cx: &Cx, npc: &str, wrap: impl Fn(FacilityAct) -> A) {
        let gs = cx.gs();
        let d = &cx.data;
        let fr = crate::shop::frame_rect(&ui.p);
        let name = json::s_or(d.resolve(&gs.world_id, "characters", npc), "name", npc);
        ui.p.panel(fr, th().panel, th().panel_edge, 4.0);
        ui.push("facility", "panel", name.clone(), fr, true, false, None, Vec::new());
        let (lb, ls) = (ui.p.line_h(BODY).ceil() as i32, ui.p.line_h(SMALL).ceil() as i32);
        let (x0, right, inner) = (fr.x + PAD, fr.x + fr.w - PAD, fr.w - 2 * PAD);
        ui.p.text_fit(x0 as f32, fr.y as f32 + 5.0, &name, HEAD, inner as f32, th().gold);
        let mut y = fr.y + 5 + ui.p.line_h(HEAD).ceil() as i32 + 4;
        let bh = 17.max(lb + 6);
        let bar_y = fr.y + fr.h - PAD - bh;
        let up_w = (ui.p.text_w("Upgrade", BODY).ceil() as i32 + 16).max(60);
        let text_w = inner - up_w - 16;
        let view = facility::view(d, gs, npc);
        // -- each facility: its name and level, the effects in force, then the next level with its cost and Upgrade
        for row in view.iter().map(json::dict) {
            let next = json::dget(row, "next");
            let reason = s(row, "reason");
            let lines = 2 + if next.is_empty() { 0 } else { 2 } + usize::from(!next.is_empty() && !reason.is_empty());
            let h = lb + ls * (lines as i32 - 1) + 6;
            if y + h > bar_y - lb - 4 {
                break;
            }
            let id = s(row, "id");
            let head = format!("{}  Level {} of {}", s(row, "name"), json::i(row, "level", 1), json::i(row, "top", 1));
            line(ui, format!("facility.{id}.head"), &head, x0, y, BODY, text_w, th().text);
            let (sx, sw) = (x0 + 8, text_w - 8);
            let mut ly = y + lb;
            let now = in_force(row);
            let now_line = if now.is_empty() { "No effects yet.".to_string() } else { format!("Now: {}", now.join(", ")) };
            line(ui, format!("facility.{id}.now"), &now_line, sx, ly, SMALL, sw, th().dim);
            ly += ls;
            if next.is_empty() {
                line(ui, format!("facility.{id}.top"), "Highest level reached.", sx, ly, SMALL, sw, th().good);
            } else {
                let effects = json::sget(next, "effects");
                let gains = if effects.is_empty() { String::new() } else { format!(": {}", effects.join(", ")) };
                let next_line = format!("Next: level {} {}{gains}", json::i(next, "level", 0), s(next, "name"));
                line(ui, format!("facility.{id}.next"), &next_line, sx, ly, SMALL, sw, th().text);
                ly += ls;
                let cost = json::sget(next, "cost").join(", ");
                let cost_line = format!("Cost: {}", if cost.is_empty() { "Free".to_string() } else { cost });
                line(ui, format!("facility.{id}.cost"), &cost_line, sx, ly, SMALL, sw, th().dim);
                if !reason.is_empty() {
                    ly += ls;
                    line(ui, format!("facility.{id}.reason"), &reason, sx, ly, SMALL, sw, th().foe);
                }
                let ready = json::b(row, "ready", false);
                let body = if ready { next_line.clone() } else { reason.clone() };
                ui.button_full(&format!("facility.up.{id}"), Rect::new(right - up_w, y, up_w, bh), "Upgrade", wrap(FacilityAct::Upgrade(id.clone())), ready, false, BODY, tip(&s(row, "name"), &body));
            }
            y += h;
        }
        if view.is_empty() {
            ui.p.text(x0 as f32, y as f32, "No facility is kept here.", BODY, th().dim);
        }
        // -- the last upgrade's refusal or notes, and Close
        let line = if self.errors.is_empty() { self.notes.join(" ") } else { self.errors.join(" ") };
        if !line.is_empty() {
            let r = Rect::new(x0, bar_y - lb - 3, inner, lb);
            ui.p.text_fit(r.x as f32, r.y as f32, &line, BODY, r.w as f32, if self.errors.is_empty() { th().gold } else { th().foe });
            ui.push("facility.line", "label", line.clone(), r, true, false, None, Vec::new());
        }
        let close_w = (ui.p.text_w("Close", BODY).ceil() as i32 + 16).max(44);
        ui.button_full("facility.close", Rect::new(x0, bar_y, close_w, bh), "Close", wrap(FacilityAct::Close), true, false, BODY, tip("Close", "Back to the conversation (Esc)."));
    }
}

/// PR6-8 on a scratch world `ward` (no shipped data): Smith keeps a forge facility; the conversation's facility reply
/// opens the pane in the dialogue box's place, Upgrade buys the next level (the cost paid, the level raised, the line
/// says so) and turns off with its reason once the next level is gated, Back returns to the conversation; the pane reads
/// clean at the four text sizes (TEXTFIT1's audit).
#[cfg(test)]
mod tests {
    use crate::facility::FacilityAct;
    use crate::hud::hud_tests::{frame, gfx, rect};
    use crate::screens::{enter_game, GameCx, Screen};
    use crate::story::tests::story_cx;
    use crate::talk::{ServiceScreen, SvcAct};
    use aegis_core::json::{self, s};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::TEXT_SIZES;
    use aegis_ui::ui::{Gfx, H, W};
    use emberwake_world::{explore, facility, new_game, Data};
    use serde_json::{json, Value};

    const FUNDS: i64 = 100;
    const COST_2: i64 = 40;
    const GATE: &str = "ward.gate";

    fn with_ids(w: &str, rows: Value) -> Value {
        let rows = json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect();
        Value::Array(rows)
    }

    /// Smith keeps the forge (level 2 for `COST_2` with a shop discount, level 3 gated by `GATE` with long names and
    /// several effects so its lines run long) and a second facility with no levels past the first.
    fn data() -> Data {
        let catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter"}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
            "items": [{"id": "timber", "name": "Seasoned Timber Beams", "item_type": "MATERIAL"}]
        });
        let forge = json!({"id": "forge", "name": "The Forge", "keeper_character": "smith", "levels": [
            {"level": 1, "name": "Cold Hearth", "scrap_cost": 0, "effects": []},
            {"level": 2, "name": "Warm Hearth", "scrap_cost": COST_2, "effects": [{"effect": "shop_pct", "pct": 85}]},
            {"level": 3, "name": "The Great Roaring Furnace of the Lower Ward", "scrap_cost": 90, "materials": {"timber": 4}, "requires_flags": [{"key": GATE, "value": true}],
             "effects": [{"effect": "recipe_pct", "pct": 75, "recipe_tags": ["repair", "reforge"]}, {"effect": "rest_upkeep", "delta": -1}, {"effect": "shop_pct", "pct": 90}]}
        ]});
        let shed = json!({"id": "shed", "name": "Shed", "keeper_character": "smith", "levels": [{"level": 1, "name": "Shed", "scrap_cost": 0, "effects": []}]});
        let worlds = json!({"ward": {"manifest": {"id": "ward", "name": "Ward", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids("ward", json!([{"id": "hub", "name": "Hub"}])),
            "characters": with_ids("ward", json!([{"id": "hero", "name": "Hero", "role": "party", "base_unit": "hero"}, {"id": "smith", "name": "Smith", "role": "npc"}])),
            "facilities": with_ids("ward", json!([forge, shed]))
        }}});
        Data::from_values(&catalog, &worlds, &json!({}))
    }

    fn widget(s: &dyn Screen<GameCx>, id: &str) -> Option<Value> {
        s.widgets().into_iter().find(|w| w["id"] == id)
    }

    /// A host talking to Smith with the facility pane open.
    fn open(cx: &mut crate::screens::Cx, g: &mut Gfx) -> ServiceScreen {
        let mut gs = new_game::create_for_world(&cx.data.clone(), "ward", 1).expect("a new game");
        gs.party.gold = FUNDS;
        enter_game(cx, gs, -1, Vec::new());
        cx.nav.clear();
        let mut host = ServiceScreen::default();
        host.talk(cx, "smith");
        frame(&mut host, cx, g, 1.0);
        let choices = json::aget(json::dict(&host.state(cx)["dialogue"]), "choices").to_vec();
        let k = choices.iter().position(|c| s(json::dict(c), "service") == "facility").expect("the facility reply");
        host.click_id(cx, &format!("choice.{k}")).expect("the facility reply");
        frame(&mut host, cx, g, 1.0);
        host
    }

    #[test]
    fn the_facility_reply_opens_the_pane_and_upgrade_raises_a_level() {
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut host = open(&mut cx, &mut g);
        assert!(explore::npc_services(&cx.data, cx.gs(), "ward:smith").contains(&"facility"), "the service badge");
        assert_eq!(host.state(&cx)["facility"], json!("ward:smith"));
        assert!(widget(&host, "facility").is_some() && widget(&host, "dialogue").is_none(), "the pane in the dialogue box's place");
        assert!(widget(&host, "facility.up.ward:forge").is_some_and(|w| w["enabled"] == json!(true)), "Upgrade on a level that can be bought");
        assert!(widget(&host, "facility.up.ward:shed").is_none(), "no Upgrade at the top level");
        assert!(widget(&host, "facility.ward:shed.top").is_some(), "the top level says so");
        host.click_id(&mut cx, "facility.up.ward:forge").expect("Upgrade");
        frame(&mut host, &mut cx, &mut g, 1.0);
        assert_eq!((facility::level(cx.gs(), "ward:forge"), cx.gs().party.gold), (2, FUNDS - COST_2), "raised and paid");
        assert!(widget(&host, "facility.line").is_some_and(|w| s(json::dict(&w), "text").contains("level 2")), "the line names the upgrade");
        let up = widget(&host, "facility.up.ward:forge").expect("Upgrade to level 3");
        assert!(up["enabled"] == json!(false) && up["tip"].to_string().contains(facility::NOT_YET_TEXT), "{up}");
        assert!(widget(&host, "facility.ward:forge.reason").is_some_and(|w| s(json::dict(&w), "text") == facility::NOT_YET_TEXT));
        let before = cx.gs().to_value();
        host.svc.act(&mut cx, SvcAct::Facility(FacilityAct::Upgrade("ward:forge".into())));
        frame(&mut host, &mut cx, &mut g, 1.0);
        assert_eq!(cx.gs().to_value(), before, "a refused upgrade changes nothing");
        assert_eq!(host.state(&cx)["facility_pane"]["errors"], json!([facility::NOT_YET_TEXT]));
        host.back(&mut cx);
        frame(&mut host, &mut cx, &mut g, 1.0);
        assert!(host.state(&cx)["facility"].is_null() && widget(&host, "dialogue").is_some(), "back in the conversation");
    }

    /// TEXTFIT1's audit of the pane: every pane control inside the frame; nothing past the canvas, overlapping or spilling
    /// out of its control; a shortened text keeps its full text in a control's text or hover card.
    fn audit(screen: &dyn Screen<GameCx>, recs: Vec<TextRec>, what: &str) {
        let widgets = screen.widgets();
        let (fx, fy, fw, fh) = rect(&widget(screen, "facility").expect("the pane"));
        for w in widgets.iter().filter(|w| w["id"] != "facility" && rect(w).2 > 0 && rect(w).3 > 0 && w["id"].as_str().is_some_and(|i| i.starts_with("facility."))) {
            let (x, y, ww, hh) = rect(w);
            assert!(x >= fx && y >= fy && x + ww <= fx + fw && y + hh <= fy + fh, "{what}: {w} inside the pane");
        }
        let inside: Vec<TextRec> = recs.into_iter().filter(|r| r.x >= fx as f32 && r.y >= fy as f32 && r.x < (fx + fw) as f32 && r.y < (fy + fh) as f32).collect();
        assert!(!inside.is_empty(), "{what}: the pane draws its texts");
        let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
            .iter()
            .map(|w| {
                let (x, y, ww, hh) = rect(w);
                (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
            })
            .collect();
        let v = aegis_ui::text_audit::issues(&inside, W as f32, H as f32, &boxes);
        let c = &v["counts"];
        assert_eq!((c["edge"].clone(), c["overlap"].clone(), c["spill"].clone()), (json!(0), json!(0), json!(0)), "{what}: {v}");
        for cut in v["cut"].as_array().into_iter().flatten() {
            let full = cut["text"].as_str().unwrap_or("");
            let named = widgets.iter().any(|w| w["text"].as_str().is_some_and(|t| t.starts_with(full)) || w["tip"].as_array().into_iter().flatten().any(|t| t.as_str() == Some(full)));
            assert!(named, "{what}: '{full}' is shortened without its full text in its control");
        }
    }

    #[test]
    fn the_facility_pane_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut host = open(&mut cx, &mut g);
        host.click_id(&mut cx, "facility.up.ward:forge").expect("Upgrade");
        for &(size, scale) in TEXT_SIZES.iter() {
            let recs = frame(&mut host, &mut cx, &mut g, scale as f32);
            audit(&host, recs, &format!("pane at {size}"));
            // a review render of the pane at each text size: only with `AEGIS_REVIEW_DIR` set
            if let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") {
                let path = std::path::Path::new(&out).join(format!("facility_pane_{}.png", size.to_lowercase().replace(' ', "_")));
                aegis_ui::shell::save_png(&g.canvas, &path, 0).expect("the review render");
            }
        }
    }
}
