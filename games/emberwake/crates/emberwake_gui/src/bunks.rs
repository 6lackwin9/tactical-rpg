//! The hub's bunks on the region map (BUNKS1 BK1-BK4, BK8-BK12; engine-only, no Godot original): the bunk states'
//! art, the members idling at the tables and their bunks, the candles and speech marks of waiting talks, and the click
//! card of a member (portrait, name, condition, a waiting talk; Take into the squad or Stand down, Take the lead,
//! Talk). The rules are the kit's (`emberwake_world::bunks`, `emberwake_world::talks`); this file draws and asks.

use aegis_core::json::{self, Dict};
use aegis_ui::canvas::{rgb, rgba, Rect};
use aegis_ui::theme::th;
use aegis_ui::ui::{tip, Gfx, Ui, Widget, H, SMALL, W};
use aegis_ui::widget_plumbing;
use crate::screens::{Cx, GameCx, Nav, Screen};
use emberwake_world::bunks::{self as kit, Bunk, Spots};
use emberwake_world::explore;
use emberwake_world::state::{GameState, UnitState};
use emberwake_world::{progression, roster, talks, Data};
use serde_json::{json, Value};

/// The widget id prefix of an idling member (`member.<id>`) and of a bunk (`bunk.<prop>`).
pub const MEMBER_PREFIX: &str = "member.";
pub const BUNK_PREFIX: &str = "bunk.";
/// The answer the card leaves for the region map to play the member's waiting talk (`bunk.talk.<member>`).
pub const TALK_ANSWER: &str = "bunk.talk.";
/// The candle over a bunk and the speech mark over a member with a waiting talk (placeholders until Art/UI AI-17).
pub const CANDLE_ICON: &str = "ui/markers/bunk_candle.png";
pub const TALK_ICON: &str = "ui/markers/bunk_talk.png";

/// The region map's hub state: the location it was placed for, what the places depend on, the lead and where each
/// member idles (BK9). Never saved: entering the hub places everyone again (BK12).
#[derive(Debug, Clone, Default)]
pub struct HubState {
    loc: String,
    sig: String,
    leader: String,
    pub spots: Spots,
}

impl HubState {
    /// Keeps the places up to date with lead `leader`: off outside a hub; a change of the lead alone trades the two
    /// places (BK10, `bunks::pass_lead`: the party moves to where the new lead stands up); any other change (members,
    /// housing, the squad, the location) places everyone again.
    pub fn sync(&mut self, d: &Data, gs: &mut GameState, leader: &str) {
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        if t.is_empty() || !kit::is_hub(gs, t) {
            *self = HubState::default();
            return;
        }
        let sig = kit::signature(gs, leader);
        if self.loc == gs.location_id && self.sig == sig {
            return;
        }
        let old = self.leader.clone();
        if self.loc == gs.location_id && !old.is_empty() && old != leader && kit::signature(gs, &old) == self.sig && kit::pass_lead(gs, t, &mut self.spots, &old, leader) {
            self.sig = sig;
            self.leader = leader.to_string();
            return;
        }
        self.spots = kit::place(gs, loc, t, leader);
        self.loc = gs.location_id.clone();
        self.sig = sig;
        self.leader = leader.to_string();
    }

    /// Whether the map is a hub now.
    pub fn on(&self) -> bool {
        !self.loc.is_empty()
    }

    /// The member idling on tile `at` ("" = none).
    pub fn member_at(&self, at: explore::Pos) -> String {
        self.spots.iter().find(|(_, p)| *p == at).map(|(id, _)| id.clone()).unwrap_or_default()
    }

    /// The agent link's line: each idling member's tile.
    pub fn state(&self) -> Value {
        if !self.on() {
            return Value::Null;
        }
        Value::Array(self.spots.iter().map(|(id, p)| json!({"member": id, "at": [p.0, p.1]})).collect())
    }
}

/// A member's body key and palette (spec PLAYUI1 2.4): its character's art, else its base unit's (its class's).
pub fn member_art(d: &Data, gs: &GameState, m: &UnitState) -> (String, String) {
    let character = d.resolve(&gs.world_id, "characters", &m.lore_ref);
    if character.is_empty() {
        let unit = d.entry("units", &m.base_unit);
        let class = d.entry("classes", &json::s(unit, "class_id"));
        (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit))
    } else {
        let (body, palette, _) = crate::screens::character_art(d, character);
        (body, palette)
    }
}

/// BK2: map `t` with each bunk prop's art swapped for its state's (`<art>_<state>`) where that art exists (`has`
/// says), for the props' drawing only (their fit and light keep the plain art). None when nothing changes.
pub fn state_props(t: &Dict, bunks: &[Bunk], mut has: impl FnMut(&str, i64, i64) -> bool) -> Option<Dict> {
    let mut props = json::aget(t, "props").to_vec();
    let mut changed = false;
    for p in props.iter_mut().filter_map(Value::as_object_mut) {
        let id = json::s(p, "id");
        let Some(b) = bunks.iter().find(|b| b.prop == id) else { continue };
        let art = format!("{}_{}", b.art, b.state.name());
        if has(&art, json::i(p, "x", 0), json::i(p, "y", 0)) {
            p.insert("art".into(), art.into());
            changed = true;
        }
    }
    changed.then(|| {
        let mut out = t.clone();
        out.insert("props".into(), Value::Array(props));
        out
    })
}

/// BK3: the hover text of bunk `b`: the member's name and condition word, or an empty bunk.
pub fn bunk_tip(d: &Data, gs: &GameState, b: &Bunk) -> (String, String) {
    match gs.party.member(&b.member) {
        None => ("Empty bunk".into(), String::new()),
        Some(m) => (progression::display_name(d, m), kit::condition(d, gs, m).to_string()),
    }
}

/// BK8: a candle over a bunk, or a speech mark over a member, in `r` (the icon; a drawn stand-in without it).
pub fn draw_mark(ui: &mut Ui<impl Clone>, cx: &mut Cx, icon: &str, r: Rect) {
    let px = ui.icon_px(Rect::new(0, 0, r.w + 4, r.h + 4));
    match cx.art.image(icon, px, px) {
        Some(img) => ui.p.image(img, r),
        None if icon == CANDLE_ICON => {
            ui.p.fill(Rect::new(r.x + r.w / 2 - 1, r.y + r.h / 3, 3, r.h - r.h / 3), th().game("bunk_candle", rgb(236, 220, 180)));
            ui.p.circle(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 4.0, r.w as f32 / 5.0, th().gold);
        }
        None => {
            ui.p.panel(r, th().game("bunk_talk", rgba(240, 232, 210, 235)), rgba(20, 20, 24, 200), 1.0);
            ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0 - 5.0, "...", SMALL, rgb(30, 26, 20));
        }
    }
}

// ---- The click card (BK4) ------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum CardAct {
    /// `roster::toggle_active`: into the squad or out of it.
    Squad,
    /// BK10: this member takes the lead (an active member who can lead).
    Lead,
    /// BK8: play the waiting talk (the region map opens it).
    Talk,
    Close,
}

/// BK4: the small card a click on an occupied bunk or an idling member opens.
pub struct BunkCard {
    member: String,
    /// The last refusal (the squad's).
    message: String,
    faces: crate::portraits::Faces,
    w: Vec<Widget<CardAct>>,
}

const CARD_BOX: (i32, i32) = (300, 136);
const CARD_PORTRAIT: i32 = 64;

impl BunkCard {
    pub fn new(member: &str) -> Self {
        Self { member: member.to_string(), message: String::new(), faces: Default::default(), w: Vec::new() }
    }

    /// Whether the member is in the squad (an active member).
    fn active(gs: &GameState, id: &str) -> bool {
        gs.party.active_members().iter().any(|m| m.id == id)
    }

    /// BK10: why this member can't take the lead ("" = it can): only another active member who can stand leads.
    fn lead_refusal(&self, cx: &Cx) -> String {
        let gs = cx.gs();
        let d = &cx.data;
        if crate::region::leader_id(cx) == self.member {
            return "Already leading.".into();
        }
        let can = gs.leader_choices(crate::region::standing(d, &gs.world_id)).iter().any(|m| m.id == self.member);
        if can {
            String::new()
        } else {
            "Only a member of the squad who can stand takes the lead.".into()
        }
    }

    fn act(&mut self, cx: &mut Cx, a: CardAct) {
        let d = cx.data.clone();
        let name = cx.gs().party.member(&self.member).map(|m| progression::display_name(&d, m)).unwrap_or_default();
        match a {
            CardAct::Squad => {
                let was = Self::active(cx.gs(), &self.member);
                match roster::toggle_active(&mut cx.gs_mut().party, &self.member) {
                    Err(e) => self.message = e,
                    Ok(()) => {
                        self.message.clear();
                        cx.log(if was { format!("{name} stands down.") } else { format!("{name} joins the squad.") });
                    }
                }
            }
            CardAct::Lead => {
                let why = self.lead_refusal(cx);
                if !why.is_empty() {
                    self.message = why;
                    return;
                }
                let world = cx.gs().world_id.clone();
                if cx.gs_mut().set_explore_leader(&self.member, crate::region::standing(&d, &world)) {
                    cx.log(format!("{name} leads the party."));
                    cx.nav.push(Nav::Pop);
                }
            }
            CardAct::Talk => {
                cx.answers.push(format!("{TALK_ANSWER}{}", self.member));
                cx.nav.push(Nav::Pop);
            }
            CardAct::Close => cx.nav.push(Nav::Pop),
        }
    }
}

impl Screen<GameCx> for BunkCard {
    fn id(&self) -> &'static str {
        "bunk_card"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<CardAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().scrim);
        let r = Rect::new((W - CARD_BOX.0) / 2, (H - CARD_BOX.1) / 2, CARD_BOX.0, CARD_BOX.1);
        let Some(m) = gs.party.member(&self.member) else {
            ui.frame(r, "Bunk", false);
            ui.button("bunk.close", Rect::new(r.x + r.w - 70, r.y + r.h - 26, 60, 18), "Close", CardAct::Close, true);
            self.w = ui.widgets;
            return;
        };
        let name = progression::display_name(&d, m);
        ui.frame(r, &name, false);
        self.faces.begin();
        let pr = Rect::new(r.x + 10, r.y + 24, CARD_PORTRAIT, CARD_PORTRAIT);
        ui.p.panel(pr, th().game("portrait_well", rgb(22, 30, 48)), th().edge, 2.0);
        let (who, look) = crate::hub::member_portrait(&d, &gs, m);
        self.faces.draw(&mut ui.p, cx, &who, &look, "roster", pr);
        ui.push("bunk.portrait", "portrait", name.clone(), pr, true, false, None, Vec::new());
        let tx = (pr.x + pr.w + 10) as f32;
        let tw = (r.x + r.w - 10) as f32 - tx;
        let word = kit::condition(&d, &gs, m);
        ui.p.text_fit(tx, (r.y + 28) as f32, &format!("{} - {word}", crate::hub::class_name(cx, m)), SMALL, tw, th().dim);
        let mut y = (r.y + 44) as f32;
        let waiting = talks::waiting_for(&d, &gs, &self.member);
        if let Some(w) = &waiting {
            let other = gs.party.member(w.other(&self.member)).map(|o| progression::display_name(&d, o)).unwrap_or_default();
            ui.p.text_fit(tx, y, &format!("A talk waits with {other}."), SMALL, tw, th().gold);
            y += 14.0;
        }
        // FZ4 (as today): a Martyr whose fuse is spent says so; the choice comes at the fight
        if roster::fuse_choices(&d, &gs).contains(&self.member) {
            for line in ui.p.wrap(&roster::fuse_refusal(&d, &gs), SMALL, tw) {
                ui.p.text(tx, y, &line, SMALL, th().dim);
                y += 12.0;
            }
        }
        if !self.message.is_empty() {
            ui.p.text_fit(tx, (r.y + r.h - 44) as f32, &self.message, SMALL, tw, th().foe);
        }
        let by = r.y + r.h - 26;
        let active = Self::active(&gs, &self.member);
        let (label, help) = if active { ("Stand down", "Out of the squad: rests at the bunk.") } else { ("Take into the squad", "Into the squad for the next fight.") };
        ui.button_tip("bunk.squad", Rect::new(r.x + 10, by, 112, 18), label, CardAct::Squad, true, tip(label, help));
        let lead_why = self.lead_refusal(cx);
        let mut lead_tip = tip("Take the lead", "They lead on the map; the two trade places.");
        if !lead_why.is_empty() {
            lead_tip.push((lead_why.clone(), th().dim));
        }
        ui.button_tip("bunk.lead", Rect::new(r.x + 128, by, 80, 18), "Take the lead", CardAct::Lead, active && lead_why.is_empty(), lead_tip);
        ui.button_tip("bunk.talk", Rect::new(r.x + 214, by, 36, 18), "Talk", CardAct::Talk, waiting.is_some(), tip("Talk", "Play the waiting talk."));
        ui.button("bunk.close", Rect::new(r.x + r.w - 46, r.y + 4, 40, 14), "Close", CardAct::Close, true);
        self.faces.tick();
        self.w = ui.widgets;
    }
    fn back(&mut self, cx: &mut Cx) {
        cx.nav.push(Nav::Pop);
    }
    fn state(&self, cx: &Cx) -> Value {
        let d = &cx.data;
        let gs = cx.gs();
        let condition = gs.party.member(&self.member).map(|m| kit::condition(d, gs, m)).unwrap_or_default();
        let waiting = talks::waiting_for(d, gs, &self.member).map(|w| w.id);
        json!({"member": self.member, "condition": condition, "active": Self::active(gs, &self.member), "talk": waiting, "message": self.message})
    }
    widget_plumbing!();
}

/// BUNKS1 section 4 (GUI audit) on the shipped `sunk_ward` hub (the world's retreat hub, picked by rule), its bunk
/// props and the placeholder state art: the four bunk states, the hover word, the card's refusals and the lead pass
/// (positions after Q); a test-only talk row lights the candles and plays from the card. With `AEGIS_REVIEW_DIR` set
/// the hub and the card are written there as PNGs.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use crate::region::RegionMap;
    use crate::screens::{Game, Key};
    use emberwake_world::new_game;
    use std::path::Path;
    use std::sync::Arc;

    const WORLD: &str = "sunk_ward";
    /// A tile in the bunk room, so the camera shows every bunk prop.
    const ROOM: (i64, i64) = (18, 17);

    fn root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The first four party characters of the world (roster order); the first two active, the third benched, the
    /// fourth in the crypts; the party in the hub's bunk room. `talk`: a test-only talk row for the first two.
    fn hub_cx(talk: bool) -> (Cx, Vec<String>) {
        let mut cx = game_cx();
        cx.art = aegis_ui::art::Art::new(Some(root()));
        cx.settings.v["reduce_motion"] = json!(true);
        let mut d = (*cx.data).clone();
        let ids: Vec<String> = d.catalogs.own_entries(WORLD, "characters").iter().filter(|c| json::s(c, "role") == "party").map(|c| json::s(c, "id")).take(4).collect();
        if talk {
            let dialogue = json::s(d.catalogs.own_entries(WORLD, "dialogues").first().expect("a dialogue"), "id");
            let row = json!({"id": "zz_talk", "name": "Test", "pair": [ids[0], ids[1]], "dialogue": dialogue, "_world": WORLD, "_qid": format!("{WORLD}:zz_talk")});
            d.catalogs.worlds.get_mut(WORLD).expect("the world").own.entry(emberwake_data::talk::TALKS_KIND.into()).or_default().push(json::dict(&row).clone());
        }
        let d = Arc::new(d);
        let mut gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
        gs.party.members = ids.iter().map(|c| new_game::hero_from_character(&d, WORLD, c).expect("a party character")).collect();
        gs.party.active_ids = vec![ids[0].clone(), ids[1].clone()];
        gs.party.members[3].housing = "crypt".into();
        let hub = json::s(json::dget(d.manifest(WORLD), "defeat"), "to");
        explore::enter(&d, &mut gs, &hub, explore::SPAWN);
        gs.position = ROOM;
        cx.app.data = d;
        cx.game = Some(Game::new(gs, -1, true));
        (cx, ids)
    }

    fn spots(map: &RegionMap, cx: &Cx) -> Vec<(String, (i64, i64))> {
        map.state(cx)["hub"].as_array().cloned().unwrap_or_default().iter().map(|s| (s["member"].as_str().unwrap_or("").to_string(), (s["at"][0].as_i64().unwrap_or(-1), s["at"][1].as_i64().unwrap_or(-1)))).collect()
    }

    fn shot(g: &Gfx, name: &str) {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        std::fs::create_dir_all(&out).unwrap();
        let file = std::fs::File::create(Path::new(&out).join(format!("{name}.png"))).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), g.canvas.w as u32, g.canvas.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
    }

    #[test]
    fn bk1_bk2_the_hub_bunks_draw_the_four_states() {
        let (mut cx, ids) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let bunks = map.state(&cx)["bunks"].as_array().cloned().expect("the hub's bunks");
        let cap = roster::bunk_cap(cx.gs()) as usize;
        assert_eq!(bunks.len(), cap, "a bunk prop per bunk, all in view: {bunks:?}");
        let states: Vec<(String, String)> = bunks.iter().map(|b| (b["member"].as_str().unwrap().to_string(), b["state"].as_str().unwrap().to_string())).collect();
        let want: Vec<(String, String)> = [(ids[0].as_str(), "active"), (ids[1].as_str(), "active"), (ids[2].as_str(), "benched"), (ids[3].as_str(), "crypt")]
            .iter()
            .map(|(m, s)| (m.to_string(), s.to_string()))
            .chain(std::iter::repeat_n(("".to_string(), "empty".to_string()), cap - 4))
            .collect();
        assert_eq!(states, want);
        for b in &bunks {
            let file = b["file"].as_str().unwrap_or("");
            assert!(file.contains(&format!("kiln_bunk_{}/", b["state"].as_str().unwrap())), "the state's art: {b}");
        }
        // BK9: the active member who does not lead idles at the tables, the benched one in front of its bunk; the
        // crypt member is not shown
        let lead = crate::region::leader_id(&cx);
        let at = spots(&map, &cx);
        assert!(!at.iter().any(|(m, _)| *m == lead || *m == ids[3]), "{at:?}");
        let benched = at.iter().find(|(m, _)| *m == ids[2]).expect("the benched member idles").1;
        let b = kit::bunks(cx.gs(), explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())));
        assert_eq!(benched, b[2].spot());
        shot(&g, "hub_bunks");
    }

    #[test]
    fn bk3_hovering_a_bunk_names_the_member_and_its_condition() {
        let (mut cx, ids) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let w = map.widgets().into_iter().find(|w| w["id"] == format!("{BUNK_PREFIX}bunk_3")).expect("the third bunk's click area");
        let r = (w["rect"][0].as_i64().unwrap() as i32, w["rect"][1].as_i64().unwrap() as i32, w["rect"][2].as_i64().unwrap() as i32, w["rect"][3].as_i64().unwrap() as i32);
        map.draw(&mut cx, &mut g, (r.0 + r.2 / 2, r.1 + r.3 / 2), true);
        let w = map.widgets().into_iter().find(|w| w["id"] == format!("{BUNK_PREFIX}bunk_3")).unwrap();
        let name = progression::display_name(&cx.data, cx.gs().party.member(&ids[2]).unwrap());
        assert_eq!(w["tip"], json!([name, kit::RESTING]), "benched and well: resting");
        cx.gs_mut().party.members[2].add_condition(roster::WOUNDED);
        map.draw(&mut cx, &mut g, (r.0 + r.2 / 2, r.1 + r.3 / 2), true);
        let w = map.widgets().into_iter().find(|w| w["id"] == format!("{BUNK_PREFIX}bunk_3")).unwrap();
        assert_eq!(w["tip"][1], json!(kit::WOUNDED_WORD));
    }

    #[test]
    fn bk4_the_card_moves_members_in_and_out_and_refuses_as_the_party_screen() {
        let (mut cx, ids) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        // a click on the benched member's bunk opens its card
        map.click_id(&mut cx, &format!("{BUNK_PREFIX}bunk_3")).expect("the bunk is clickable");
        let mut card = match cx.nav.pop() {
            Some(Nav::Push(s)) => s,
            _ => panic!("the card opens"),
        };
        assert_eq!(card.id(), "bunk_card");
        frame(card.as_mut(), &mut cx, &mut g, 1.0);
        assert_eq!(card.state(&cx)["member"], json!(ids[2]));
        card.click_id(&mut cx, "bunk.squad").expect("Take into the squad");
        assert!(cx.gs().party.active_members().iter().any(|m| m.id == ids[2]), "joined the squad");
        // the cap refuses one more, the last active member stays (roster::toggle_active's refusals)
        cx.gs_mut().party.apply_cap(3);
        let mut c4 = BunkCard::new(&ids[3]);
        frame(&mut c4, &mut cx, &mut g, 1.0);
        c4.click_id(&mut cx, "bunk.squad").unwrap();
        assert_eq!(c4.state(&cx)["message"], json!("The active party holds at most 3."));
        cx.gs_mut().party.active_ids = vec![ids[0].clone()];
        let mut c0 = BunkCard::new(&ids[0]);
        frame(&mut c0, &mut cx, &mut g, 1.0);
        c0.click_id(&mut cx, "bunk.squad").unwrap();
        assert_eq!(c0.state(&cx)["message"], json!("Keep at least one active member."));
        // the lead is not passed to a benched member
        let mut c2 = BunkCard::new(&ids[1]);
        frame(&mut c2, &mut cx, &mut g, 1.0);
        assert!(c2.click_id(&mut cx, "bunk.lead").is_err() || c2.state(&cx)["message"] != json!(""), "a benched member can't take the lead");
        frame(&mut c0, &mut cx, &mut g, 1.0);
        shot(&g, "bunk_card");
    }

    #[test]
    fn bk10_q_passes_the_lead_and_the_two_trade_places() {
        let (mut cx, ids) = hub_cx(false);
        cx.gs_mut().party.members[3].housing = String::new();
        cx.gs_mut().party.active_ids = vec![ids[0].clone(), ids[1].clone(), ids[3].clone()];
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let lead = crate::region::leader_id(&cx);
        let before = spots(&map, &cx);
        map.key(&mut cx, Key::Char('q'));
        let new = crate::region::leader_id(&cx);
        assert_ne!(new, lead);
        let freed = before.iter().find(|(m, _)| *m == new).expect("the new lead idled").1;
        frame(&mut map, &mut cx, &mut g, 1.0);
        let after = spots(&map, &cx);
        assert_eq!(after.iter().find(|(m, _)| *m == lead).map(|s| s.1), Some(freed), "the old lead takes the freed spot");
        assert!(!after.iter().any(|(m, _)| *m == new), "the new lead stands");
        let p = cx.gs().position;
        assert_eq!((p.0 - freed.0).abs() + (p.1 - freed.1).abs(), 1, "the new lead stands up beside the spot: {p:?} by {freed:?}");
        for (m, at) in before.iter().filter(|(m, _)| *m != new) {
            assert_eq!(after.iter().find(|(x, _)| x == m).map(|s| s.1), Some(*at), "{m} did not move");
        }
        assert_eq!(map.state(&cx)["leader"], json!(new));
    }

    #[test]
    fn bk8_a_waiting_talk_lights_the_candles_and_the_card_plays_it_once() {
        let (mut cx, ids) = hub_cx(true);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let bunks = map.state(&cx)["bunks"].as_array().cloned().unwrap();
        let lit: Vec<&str> = bunks.iter().filter(|b| b["candle"] == json!(true)).map(|b| b["member"].as_str().unwrap()).collect();
        assert_eq!(lit, vec![ids[0].as_str(), ids[1].as_str()], "a candle on both members' bunks");
        shot(&g, "hub_talk_waiting");
        let mut card = BunkCard::new(&ids[1]);
        frame(&mut card, &mut cx, &mut g, 1.0);
        assert_eq!(card.state(&cx)["talk"], json!("zz_talk"));
        card.click_id(&mut cx, "bunk.talk").expect("Talk");
        cx.nav.clear();
        map.tick(&mut cx);
        assert!(map.svc.talking(), "the talk plays over the map");
        assert!(cx.gs().flag_true("talk.zz_talk"), "its seen flag");
        map.svc.end(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let bunks = map.state(&cx)["bunks"].as_array().cloned().unwrap();
        assert!(bunks.iter().all(|b| b["candle"] == json!(false)), "played once: no candle left");
    }
}
