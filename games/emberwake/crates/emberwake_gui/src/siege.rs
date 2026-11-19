//! The siege screen (PR6-9, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` section 5, B6; engine-only, no Godot original;
//! story-free): while a siege runs (the effect `start_siege`), a map with nothing open hands over to it
//! (`map_tick`, after the story's queue). It shows the siege's name, its rooms (cleared, next, ahead), the fighters left
//! downed with a Triage button each (the first stash item that revives), the re-stock count and Re-stock, and Fight
//! (the next room, runner source `siege::FIGHT_SOURCE`) or, with every fighter down, Yield the room. No Close: the
//! siege is fought out (no rest, `no_camp`). The rules are the kit's (`siege::view`, `restock`, `triage`, `forfeit`);
//! the screen only draws them. A placeholder layout until the Art/UI studio's siege board lands.

use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use aegis_core::json::{self, s};
use aegis_ui::canvas::Rect;
use aegis_ui::theme::th;
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, TITLE, W};
use aegis_ui::widget_plumbing;
use emberwake_world::siege;
use serde_json::{json, Value};

/// The card's width, inner margin and row gap.
const CARD_W: i32 = 320;
const PAD: i32 = 10;

/// A click on the siege screen.
#[derive(Debug, Clone)]
pub enum SiegeAct {
    /// Fight the next room.
    Fight,
    /// Every fighter down: the room is lost without a battle.
    Yield,
    Restock,
    /// Revive a downed member (member id, item id).
    Triage(String, String),
}

/// The siege screen: the last action's lines and refusal.
#[derive(Default)]
pub struct SiegeScreen {
    notes: Vec<String>,
    errors: Vec<String>,
    w: Vec<Widget<SiegeAct>>,
}

/// A map screen's tick with nothing open: while a siege runs (and the run is not over) the siege screen opens. True
/// when it opened.
pub fn map_tick(cx: &mut Cx) -> bool {
    if !cx.has_game() || cx.busy() || !cx.nav.is_empty() || !siege::running(cx.gs()) || crate::runover::is_over(cx) {
        return false;
    }
    cx.nav.push(Nav::Push(Box::new(SiegeScreen::default())));
    true
}

impl SiegeScreen {
    /// One click (or key): Fight leaves for the room's battle; the rest act here and keep their lines.
    pub fn act(&mut self, cx: &mut Cx, a: SiegeAct) {
        let d = cx.data.clone();
        self.notes.clear();
        self.errors.clear();
        match a {
            SiegeAct::Fight => {
                let why = siege::fight_refusal(cx.gs());
                let Some(fight) = siege::fight(&d, cx.gs()).filter(|_| why.is_empty()) else {
                    self.errors.push(why.to_string());
                    return;
                };
                cx.nav.push(Nav::Pop);
                crate::region::fight_now(cx, &fight, siege::FIGHT_SOURCE);
            }
            SiegeAct::Yield => {
                let res = siege::forfeit(&d, cx.gs_mut());
                for l in json::sget(json::dget(&res, "siege"), "lines") {
                    cx.log(l);
                }
                crate::runover::take(cx, &res);
                cx.autosave();
                cx.nav.push(Nav::Pop);
            }
            SiegeAct::Restock => {
                let res = siege::restock(&d, cx.gs_mut());
                self.errors = json::sget(&res, "errors");
                if self.errors.is_empty() {
                    let short = json::sget(&res, "short");
                    self.notes.push(if short.is_empty() { "Pouches re-stocked.".to_string() } else { format!("Re-stocked; short: {}", short.join(", ")) });
                }
            }
            SiegeAct::Triage(member, item) => {
                let res = siege::triage(&d, cx.gs_mut(), &member, &item);
                self.errors = json::sget(&res, "errors");
                if self.errors.is_empty() {
                    self.notes.push(format!("Back on their feet (HP {}).", json::i(&res, "hp", 0)));
                }
            }
        }
        for n in &self.notes {
            cx.log(n.clone());
        }
    }
}

impl SiegeScreen {
    /// ENGINE-25: the line above the buttons: the last action's refusal (`foe`), else its notes (`gold`), else the
    /// no-rest rule while it holds (`dim`).
    pub(crate) fn line(&self, view: &json::Dict) -> (String, &'static str) {
        if !self.errors.is_empty() {
            (self.errors.join(" "), "foe")
        } else if !self.notes.is_empty() {
            (self.notes.join(" "), "gold")
        } else {
            (if json::b(view, "no_camp", false) { "No rest until the siege is broken.".to_string() } else { String::new() }, "dim")
        }
    }
}

/// A room's mark and its theme role by its state (cleared, next, ahead).
pub(crate) fn room_mark(state: &str) -> (&'static str, &'static str) {
    match state {
        "cleared" => ("Held", "good"),
        "next" => ("Next", "gold"),
        _ => ("Ahead", "dim"),
    }
}

/// The heading over the downed fighters.
pub(crate) fn down_head(none: bool) -> &'static str {
    if none {
        "No one is down."
    } else {
        "Down (sits out until triaged):"
    }
}

/// A Triage button's label (with the reviving item's name) and its hover line.
pub(crate) fn triage_label(item: Option<&str>) -> String {
    item.map_or("Triage".to_string(), |n| format!("Triage ({n})"))
}

pub(crate) fn triage_note(can: bool, who: &str) -> String {
    if can {
        format!("Spend one to revive {who}.")
    } else {
        "No item in the stash can revive.".to_string()
    }
}

/// Re-stock's label with the re-stocks left.
pub(crate) fn restock_label(view: &json::Dict) -> String {
    format!("Re-stock ({})", json::i(view, "restocks", 0))
}

/// The main button: Fight, or with every fighter down Yield the room; its label, click and hover line.
pub(crate) fn fight_button(view: &json::Dict) -> (&'static str, SiegeAct, &'static str) {
    if s(view, "refusal") == siege::ALL_DOWN_TEXT {
        ("Yield the room", SiegeAct::Yield, "Every fighter is down: the room is lost.")
    } else {
        ("Fight", SiegeAct::Fight, "Fight the next room (Enter).")
    }
}

impl Screen<GameCx> for SiegeScreen {
    fn id(&self) -> &'static str {
        "siege"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<SiegeAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().scrim);
        let Some(view) = siege::view(&cx.data, cx.gs()) else {
            self.w = ui.widgets;
            return;
        };
        let view = json::dict(&view).clone();
        let scale = ui.p.fonts.scale.max(1.0);
        let inner = ((CARD_W as f32 * scale).ceil() as i32).min(W - 40 - 2 * PAD);
        let (tl, bl, sl) = (ui.p.line_h(TITLE).ceil() as i32, ui.p.line_h(BODY).ceil() as i32, ui.p.line_h(SMALL).ceil() as i32);
        let bh = bl + 6;
        let rooms = json::aget(&view, "rooms").to_vec();
        let downed = json::aget(&view, "downed").to_vec();
        let items = json::aget(&view, "items").to_vec();
        let h = PAD + tl + 6 + rooms.len() as i32 * bl + 6 + sl + downed.len() as i32 * (bh + 2) + 6 + sl + 6 + bh + PAD;
        let r = Rect::new((W - inner - 2 * PAD) / 2, ((H - h) / 2).max(4), inner + 2 * PAD, h);
        ui.p.panel(r, th().panel, th().foe, 4.0);
        let name = s(&view, "name");
        ui.push("siege", "panel", name.clone(), r, true, false, None, Vec::new());
        let x0 = r.x + PAD;
        let mut y = r.y + PAD;
        ui.p.text_fit(x0 as f32, y as f32, &name, TITLE, inner as f32, th().foe);
        y += tl + 6;
        // -- the rooms in order
        for (i, room) in rooms.iter().map(json::dict).enumerate() {
            let (mark, role) = room_mark(&s(room, "state"));
            let col = th().role(role).unwrap_or(th().text);
            let text = format!("{}. {}  ({mark})", i + 1, s(room, "name"));
            ui.p.text_fit(x0 as f32, y as f32, &text, BODY, inner as f32, col);
            ui.push(format!("siege.room.{i}"), "label", text, Rect::new(x0, y, inner, bl), true, false, None, Vec::new());
            y += bl;
        }
        y += 6;
        // -- the downed fighters, each with Triage when the stash holds an item that revives
        let head = down_head(downed.is_empty()).to_string();
        ui.p.text(x0 as f32, y as f32, &head, SMALL, th().dim);
        y += sl;
        let first_item = items.first().map(json::dict).map(|i| (s(i, "id"), s(i, "name")));
        for m in downed.iter().map(json::dict) {
            let (id, who) = (s(m, "member"), s(m, "name"));
            ui.p.text_fit((x0 + 8) as f32, (y + 3) as f32, &who, BODY, (inner / 2) as f32, th().foe);
            ui.push(format!("siege.down.{id}"), "label", who.clone(), Rect::new(x0 + 8, y, inner / 2, bh), true, false, None, Vec::new());
            let label = triage_label(first_item.as_ref().map(|(_, n)| n.as_str()));
            let bw = (ui.p.text_w(&label, BODY).ceil() as i32 + 16).max(60);
            let act = SiegeAct::Triage(id.clone(), first_item.as_ref().map(|(i, _)| i.clone()).unwrap_or_default());
            let body = triage_note(first_item.is_some(), &who);
            ui.button_full(&format!("siege.triage.{id}"), Rect::new(r.x + r.w - PAD - bw, y, bw, bh), &label, act, first_item.is_some(), false, BODY, tip("Triage", &body));
            y += bh + 2;
        }
        y += 6;
        // -- the last action's line (or the rule), then Re-stock and Fight / Yield
        let line = self.line(&view);
        let line = (line.0, th().role(line.1).unwrap_or(th().text));
        ui.p.text_fit(x0 as f32, y as f32, &line.0, SMALL, inner as f32, line.1);
        ui.push("siege.line", "label", line.0.clone(), Rect::new(x0, y, inner, sl), true, false, None, Vec::new());
        let bar_y = r.y + r.h - PAD - bh;
        let restock = restock_label(&view);
        let rw = (ui.p.text_w(&restock, BODY).ceil() as i32 + 16).max(60);
        let can_restock = json::b(&view, "can_restock", false);
        ui.button_full("siege.restock", Rect::new(x0, bar_y, rw, bh), &restock, SiegeAct::Restock, can_restock, false, BODY, tip("Re-stock", "Top the pouches up from the stash (between rooms)."));
        let (label, act, body) = fight_button(&view);
        let fw = (ui.p.text_w(label, BODY).ceil() as i32 + 16).max(60);
        ui.button_full("siege.fight", Rect::new(r.x + r.w - PAD - fw, bar_y, fw, bh), label, act, true, false, BODY, tip(label, body));
        self.w = ui.widgets;
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if k == Key::Enter || k == Key::Space {
            let all_down = siege::fight_refusal(cx.gs()) == siege::ALL_DOWN_TEXT;
            self.act(cx, if all_down { SiegeAct::Yield } else { SiegeAct::Fight });
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        // ENGINE-25 (UISPLIT1-J Stage A): `layout_view`, the screen's view-model (`view` stays the kit's siege view)
        json!({"view": siege::view(&cx.data, cx.gs()), "layout_view": crate::siege_view::view(cx, self).to_json(), "notes": self.notes, "errors": self.errors})
    }

    widget_plumbing!();
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::screens::GameCx;
    use emberwake_world::Data;

    /// A scratch world (no shipped data): the siege `hold` of three rooms (Grate, Lock, Hearth), two heroes and salts
    /// that revive in the stash.
    pub(crate) fn siege_cx() -> Cx {
        let with_ids = |rows: Value| -> Value {
            Value::Array(
                rows.as_array()
                    .unwrap()
                    .iter()
                    .map(|r| {
                        let mut r = r.as_object().unwrap().clone();
                        let id = r["id"].as_str().unwrap().to_string();
                        r.insert("_world".into(), json!("ward"));
                        r.insert("_qid".into(), json!(format!("ward:{id}")));
                        Value::Object(r)
                    })
                    .collect(),
            )
        };
        let catalog = json!({
            "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
            "skills": [{"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
                {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
                {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3}, {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1}],
            "items": [{"id": "salts", "name": "Salts", "item_type": "CONSUMABLE", "value": 5, "revive": true, "heal": 15}]
        });
        let worlds = json!({"ward": {"manifest": {"id": "ward", "name": "Ward", "start_location": "hub", "start_party": ["lead", "second"], "campaign": {"start_act": 1}}, "own": {
            "locations": with_ids(json!([{"id": "hub", "name": "Hub"}])),
            "characters": with_ids(json!([{"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"}, {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
                {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"}])),
            "encounters": with_ids(json!([{"id": "grate", "name": "Grate", "enemies": ["rat"]}, {"id": "lock", "name": "Lock", "enemies": ["rat"]}, {"id": "hearth", "name": "Hearth", "enemies": ["rat"]}])),
            "sieges": with_ids(json!([{"id": "hold", "name": "The Hold", "restocks": 1, "rooms": [{"encounter": "grate", "name": "Grate"}, {"encounter": "lock", "name": "Lock"}, {"encounter": "hearth", "name": "Hearth"}]}]))
        }}});
        let d = Data::from_values(&catalog, &worlds, &json!({}));
        let mut cx = crate::story::tests::story_cx(d.clone());
        let mut gs = emberwake_world::new_game::create_for_world(&d, "ward", 1).expect("a new game");
        gs.party.add_to_stash("salts", 1);
        crate::screens::enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        cx
    }

    pub(crate) fn open(cx: &mut Cx) {
        let d = cx.data.clone();
        assert_eq!(siege::start(&d, cx.gs_mut(), "hold"), "");
    }

    fn widget(s: &SiegeScreen, id: &str) -> Option<Value> {
        s.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn take_screen(cx: &mut Cx) -> Box<dyn Screen<GameCx>> {
        match cx.nav.pop() {
            Some(Nav::Push(s)) => s,
            _ => panic!("no screen was pushed"),
        }
    }

    #[test]
    fn a_running_siege_takes_over_a_map_with_nothing_open() {
        let mut cx = siege_cx();
        assert!(!map_tick(&mut cx), "no siege: nothing opens");
        open(&mut cx);
        assert!(map_tick(&mut cx));
        assert_eq!(take_screen(&mut cx).id(), "siege");
        // through the story's map tick too (the region and world maps call it)
        assert!(crate::story::map_tick(&mut cx));
        assert_eq!(take_screen(&mut cx).id(), "siege");
    }

    #[test]
    fn the_screen_shows_the_rooms_the_downed_and_the_buttons() {
        let mut cx = siege_cx();
        open(&mut cx);
        cx.gs_mut().siege.as_mut().unwrap().room = 1;
        cx.gs_mut().siege.as_mut().unwrap().downed = vec!["second".into()];
        let mut g = crate::hud::hud_tests::gfx();
        let mut s = SiegeScreen::default();
        let texts = crate::hud::hud_tests::frame(&mut s, &mut cx, &mut g, 1.0);
        assert!(texts.iter().any(|t| t.text == "The Hold"), "the siege's name");
        assert_eq!(widget(&s, "siege.room.0").unwrap()["text"], json!("1. Grate  (Held)"));
        assert_eq!(widget(&s, "siege.room.1").unwrap()["text"], json!("2. Lock  (Next)"));
        assert_eq!(widget(&s, "siege.room.2").unwrap()["text"], json!("3. Hearth  (Ahead)"));
        let triage = widget(&s, "siege.triage.second").expect("a downed hero has Triage");
        assert_eq!((triage["text"].clone(), triage["enabled"].clone()), (json!("Triage (Salts)"), json!(true)));
        assert_eq!(widget(&s, "siege.restock").unwrap()["enabled"], json!(true));
        assert_eq!(widget(&s, "siege.fight").unwrap()["text"], json!("Fight"));
        assert_eq!(widget(&s, "siege.line").unwrap()["text"], json!("No rest until the siege is broken."));
        // triage revives and the button goes; the re-stock is spent once
        s.click_id(&mut cx, "siege.triage.second").unwrap();
        assert!(cx.gs().siege.as_ref().unwrap().downed.is_empty());
        s.click_id(&mut cx, "siege.restock").unwrap();
        crate::hud::hud_tests::frame(&mut s, &mut cx, &mut g, 1.0);
        assert!(widget(&s, "siege.triage.second").is_none());
        assert_eq!(widget(&s, "siege.restock").unwrap()["enabled"], json!(false));
        assert_eq!(widget(&s, "siege.restock").unwrap()["text"], json!("Re-stock (0)"));
        // the text fits at a large text size
        let big = crate::hud::hud_tests::frame(&mut s, &mut cx, &mut g, 1.5);
        assert!(big.iter().any(|t| t.text == "The Hold"));
        if let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") {
            cx.gs_mut().siege.as_mut().unwrap().downed = vec!["second".into()];
            cx.gs_mut().party.add_to_stash("salts", 1);
            for (name, w, h) in [("siege.png", 1280, 720), ("siege_640.png", 640, 360)] {
                g.canvas = aegis_ui::canvas::Canvas::new(w, h);
                g.view = aegis_ui::painter::View::fit(w as u32, h as u32);
                g.fonts.scale = 1.0;
                let mut fresh = SiegeScreen::default();
                fresh.draw(&mut cx, &mut g, (-500, -500), true);
                let file = std::fs::File::create(std::path::Path::new(&out).join(name)).unwrap();
                let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
            }
        }
    }

    #[test]
    fn fight_leaves_for_the_room_and_every_fighter_down_yields_it() {
        let mut cx = siege_cx();
        open(&mut cx);
        let mut g = crate::hud::hud_tests::gfx();
        let mut s = SiegeScreen::default();
        crate::hud::hud_tests::frame(&mut s, &mut cx, &mut g, 1.0);
        s.click_id(&mut cx, "siege.fight").unwrap();
        assert!(cx.log.iter().any(|l| l == "Fight: Grate!"), "{:?}", cx.log);
        let fight = take_screen(&mut cx);
        assert_eq!(fight.state(&cx)["source"], json!(siege::FIGHT_SOURCE));
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)), "the siege screen closes first");
        // every fighter down: Yield the room ends the siege as lost
        cx.nav.clear();
        cx.gs_mut().siege.as_mut().unwrap().downed = vec!["lead".into(), "second".into()];
        let mut s = SiegeScreen::default();
        crate::hud::hud_tests::frame(&mut s, &mut cx, &mut g, 1.0);
        assert_eq!(widget(&s, "siege.fight").unwrap()["text"], json!("Yield the room"));
        s.click_id(&mut cx, "siege.fight").unwrap();
        assert!(cx.gs().siege.is_none(), "the siege is over");
        assert!(cx.log.iter().any(|l| l == "Grate is lost."), "{:?}", cx.log);
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)), "the siege screen closes");
        cx.nav.clear();
        assert!(!map_tick(&mut cx), "no siege screen after it ends");
    }
}
