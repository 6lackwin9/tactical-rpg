//! The hub's bunks on the region map (BUNKS1 BK1-BK4, BK8-BK12; engine-only, no Godot original): the bunk states'
//! art, the members idling at the tables and their bunks, the candles and speech marks of waiting talks, and the click
//! card of a member (portrait, name, condition, a waiting talk; Take into the squad or Stand down, Take the lead,
//! Talk) in the dialogue box, and each occupied bunk's name plate (HUB-POLISH1). The rules are the kit's (`emberwake_world::bunks`, `emberwake_world::talks`); this file draws and asks.

use aegis_core::json::{self, Dict};
use aegis_ui::canvas::{rgb, rgba, Rect};
use aegis_ui::painter::Painter;
use aegis_ui::theme::th;
use aegis_ui::ui::{tip, Gfx, Ui, Widget, H, SMALL, W};
use aegis_ui::widget_plumbing;
use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use crate::talk::{box_layout, draw_box, TalkAct};
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
/// The bunk card's frame widget (the dialogue box's frame, HUB-POLISH1).
pub const CARD_ID: &str = "bunk.card";

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

// ---- The name plate (BK2, HUB-POLISH1 (1)) --------------------------------------------------------------------------

/// The bunk art's measure (AI-17 `kiln_bunk_<state>`, in art px of its square): the art's side, the bed's width
/// (centred) and the bed head's top (the arched niche is above it).
pub const ART_SIDE: i32 = 80;
pub const BED_W: i32 = 42;
pub const BED_HEAD_Y: i32 = 28;
/// The plate's widest (art px, centred on the bed): the bunks' pitch (three cells, 96 art px) less a gap, so two
/// neighbours' plates never meet.
pub const PLATE_MAX_W: i32 = 92;

/// The name plate of an occupied bunk drawn in `r` (the bunk's drawn art rect): centred on the bed, its top on the
/// bed's head (under the niche, clear of the candle in the art's top corner), as wide as the class sigil and the name
/// need, at least the bed and at most `PLATE_MAX_W`; the name shortened to fit.
pub fn plate_rect(p: &mut Painter, r: Rect, name: &str) -> Rect {
    let lh = p.line_h(SMALL).ceil() as i32;
    let h = lh + 2;
    let side = h - 2;
    let want = 2 + side + 3 + p.text_w(name, SMALL).ceil() as i32 + 4;
    let w = want.clamp(BED_W * r.w / ART_SIDE, PLATE_MAX_W * r.w / ART_SIDE);
    let mid = r.x + r.w / 2;
    let head = r.y + BED_HEAD_Y * r.h / ART_SIDE;
    Rect::new(mid - w / 2, head, w, h)
}

/// Draws the plate `pr` (`plate_rect`): a brass-edged plate, the class sigil (the class name's first letter in a
/// ring, drawn: no art) at the left and the member's name (shortened to fit, the audit keeping it whole).
pub fn draw_plate(p: &mut Painter, pr: Rect, name: &str, class: &str) {
    p.panel(pr, th().card, th().gold, 1.0);
    let side = pr.h - 2;
    let letter = class.chars().next().map_or('?', |c| c.to_ascii_uppercase());
    aegis_ui::ui::draw_glyph(p, aegis_ui::ui::Glyph::Letter(letter), Rect::new(pr.x + 2, pr.y + 1, side, side), th().gold);
    let tx = pr.x + 2 + side + 3;
    p.text_fit(tx as f32, pr.y as f32 + 1.0, name, SMALL, (pr.x + pr.w - 3 - tx) as f32, th().text);
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

/// BK4: the card a click on an occupied bunk or an idling member opens. HUB-POLISH1 (2): it is the dialogue box
/// (`talk::box_layout`, `talk::draw_box`): the member's portrait at the left with the name and the class and condition
/// line under it, what the member has to say at the right (a refusal, a waiting talk, the fuse; else where they
/// stand) and the card's buttons as numbered choices (1-9 pick them).
pub struct BunkCard {
    member: String,
    /// The last refusal (the squad's).
    message: String,
    faces: crate::portraits::Faces,
    /// The choices drawn last frame (their action and whether they are on), for the keys.
    acts: Vec<(TalkAct, bool)>,
    w: Vec<Widget<CardAct>>,
}

/// The card's choices as the talk box's actions (`TalkAct::Choose(code)`, the code fixed per choice).
fn card_act(a: TalkAct) -> CardAct {
    match a {
        TalkAct::Choose(0) => CardAct::Squad,
        TalkAct::Choose(1) => CardAct::Lead,
        TalkAct::Choose(2) => CardAct::Talk,
        _ => CardAct::Close,
    }
}

/// The card's last choice: it closes the card.
pub const FAREWELL: &str = "Farewell.";

impl BunkCard {
    pub fn new(member: &str) -> Self {
        Self { member: member.to_string(), message: String::new(), faces: Default::default(), acts: Vec::new(), w: Vec::new() }
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
        let farewell = ("bunk.close".to_string(), FAREWELL.to_string(), false, TalkAct::Choose(3));
        let Some(m) = gs.party.member(&self.member) else {
            let b = box_layout(&mut ui.p, "Empty bunk", "", "", vec![farewell]);
            draw_box(&mut ui, &b, CARD_ID, "Empty bunk", false, card_act, |_, _| {});
            self.acts = b.acts();
            self.w = ui.widgets;
            return;
        };
        let name = progression::display_name(&d, m);
        let word = kit::condition(&d, &gs, m);
        let band = format!("{} - {word}", crate::hub::class_name(cx, m));
        // what the member says: the last refusal, a waiting talk, a spent fuse (FZ4); else where they stand
        let active = Self::active(&gs, &self.member);
        let waiting = talks::waiting_for(&d, &gs, &self.member);
        let mut said: Vec<String> = Vec::new();
        if !self.message.is_empty() {
            said.push(self.message.clone());
        }
        if let Some(w) = &waiting {
            let other = gs.party.member(w.other(&self.member)).map(|o| progression::display_name(&d, o)).unwrap_or_default();
            said.push(format!("A talk waits with {other}."));
        }
        if roster::fuse_choices(&d, &gs).contains(&self.member) {
            said.push(roster::fuse_refusal(&d, &gs));
        }
        if said.is_empty() {
            said.push(if active { "In the squad.".into() } else { "Out of the squad.".into() });
        }
        let (label, help) = if active { ("Stand down", "Out of the squad: rests at the bunk.") } else { ("Take into the squad", "Into the squad for the next fight.") };
        let lead_why = self.lead_refusal(cx);
        let lead_on = active && lead_why.is_empty();
        let lead_shown = if lead_on { "Take the lead".to_string() } else { format!("Take the lead  ({lead_why})") };
        let mut rows = vec![("bunk.squad".to_string(), label.to_string(), false, TalkAct::Choose(0)), ("bunk.lead".to_string(), lead_shown, !lead_on, TalkAct::Choose(1))];
        if waiting.is_some() {
            rows.push(("bunk.talk".to_string(), "Talk".to_string(), false, TalkAct::Choose(2)));
        }
        rows.push(farewell);
        let mut b = box_layout(&mut ui.p, &name, &band, &said.join(" "), rows);
        b.set_tip("bunk.squad", tip(label, help));
        b.set_tip("bunk.lead", tip("Take the lead", "They lead on the map; the two trade places."));
        b.set_tip("bunk.talk", tip("Talk", "Play the waiting talk."));
        self.faces.begin();
        let (who, look) = crate::hub::member_portrait(&d, &gs, m);
        let faces = &mut self.faces;
        draw_box(&mut ui, &b, CARD_ID, &name, false, card_act, |p, pr| {
            faces.draw(p, cx, &who, &look, "dialogue", pr);
        });
        ui.push("bunk.portrait", "portrait", name.clone(), b.portrait_rect(), true, false, None, Vec::new());
        self.faces.tick();
        self.acts = b.acts();
        self.w = ui.widgets;
    }
    /// 1-9 pick that choice (as the dialogue box).
    fn key(&mut self, cx: &mut Cx, k: Key) {
        let Key::Char(c @ '1'..='9') = k else { return };
        if let Some((a, true)) = self.acts.get(c as usize - '1' as usize).copied() {
            self.act(cx, card_act(a));
        }
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

    /// HUB-POLISH1 (1): every occupied bunk state (active, benched, crypt) wears a plate with its member's name and
    /// class, centred on the bed and its top on the bed's head (`plate_rect` over the AI-17 measure), at every text size;
    /// an empty bunk wears none.
    #[test]
    fn hub_polish1_each_occupied_bunk_wears_its_members_name_plate() {
        let (mut cx, _) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        for (size, scale) in aegis_ui::screen::TEXT_SIZES {
            frame(&mut map, &mut cx, &mut g, scale as f32);
            let widgets = map.widgets();
            let bunks = map.state(&cx)["bunks"].as_array().cloned().expect("the hub's bunks");
            let mut states: Vec<String> = Vec::new();
            for b in &bunks {
                let member = b["member"].as_str().unwrap();
                if member.is_empty() {
                    assert!(b.get("plate").is_none(), "{size}: an empty bunk has no plate: {b}");
                    continue;
                }
                states.push(b["state"].as_str().unwrap().to_string());
                let m = cx.gs().party.member(member).unwrap().clone();
                let plate = &b["plate"];
                assert_eq!(plate["name"], json!(progression::display_name(&cx.data, &m)), "{size}: {b}");
                assert_eq!(plate["class"], json!(crate::hub::class_name(&cx, &m)), "{size}: {b}");
                let w = widgets.iter().find(|w| w["id"] == format!("{BUNK_PREFIX}{}", b["prop"].as_str().unwrap())).expect("the bunk's click area");
                let (bx, by, bw, bh) = crate::hud::hud_tests::rect(w);
                let n = |i: usize| plate["rect"][i].as_i64().unwrap();
                let (px, py, pw, ph) = (n(0), n(1), n(2), n(3));
                assert!(((px + pw / 2) - (bx + bw / 2)).abs() <= 1, "{size}: centred on the bed: {plate} on {w}");
                assert!(pw >= BED_W as i64 * bw / ART_SIDE as i64 && pw <= PLATE_MAX_W as i64 * bw / ART_SIDE as i64 + 1, "{size}: between the bed and the niche: {plate}");
                assert_eq!(py, by + BED_HEAD_Y as i64 * bh / ART_SIDE as i64, "{size}: on the bed's head: {plate} on {w}");
                assert!(ph > 0 && py + ph <= by + bh, "{size}: on the bunk: {plate} on {w}");
            }
            states.sort();
            states.dedup();
            assert_eq!(states, vec!["active", "benched", "crypt"], "{size}: every occupied state");
            match size {
                "Normal" => shot(&g, "hub_plates"),
                "Extra large" => shot(&g, "hub_plates_xl"),
                _ => {}
            }
        }
    }

    /// HUB-POLISH1 (2): the card is the dialogue box: the talk box's frame and place (`TALK_W` wide times the text
    /// size, centred, `TALK_GAP` over the bottom), the portrait in its left column, the buttons as numbered choices
    /// (the keys pick them) with the Farewell last; a refused lead reads its reason in the row.
    #[test]
    fn hub_polish1_the_card_reads_as_a_conversation_in_the_talk_box() {
        use crate::talk::{TALK_GAP, TALK_LEFT, TALK_W};
        let (mut cx, ids) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        for (size, scale) in [("Normal", 1.0f32), ("Extra large", 1.3)] {
            // the card over the hub it opens on (as the region map's overlay)
            frame(&mut map, &mut cx, &mut g, scale);
            let mut card = BunkCard::new(&ids[2]);
            frame(&mut card, &mut cx, &mut g, scale);
            let widgets = card.widgets();
            let get = |id: &str| widgets.iter().find(|w| w["id"] == id).unwrap_or_else(|| panic!("{size}: {id}")).clone();
            let (x, y, w, h) = crate::hud::hud_tests::rect(&get(CARD_ID));
            assert_eq!(w, ((W - 40) as f32).min((TALK_W as f32 * scale).round()) as i64, "{size}: the talk box's width");
            assert_eq!((x, y + h), (((W as i64) - w) / 2, (H - TALK_GAP) as i64), "{size}: the talk box's place");
            let (px, _, pw, _) = crate::hud::hud_tests::rect(&get("bunk.portrait"));
            assert!(px >= x && px + pw <= x + (TALK_LEFT as f32 * scale).round() as i64, "{size}: the portrait in the left column");
            let choices: Vec<(String, String, bool)> = widgets.iter().filter(|w| w["kind"] == "choice").map(|w| (w["id"].as_str().unwrap().to_string(), w["text"].as_str().unwrap_or("").to_string(), w["enabled"].as_bool().unwrap_or(false))).collect();
            let ids_shown: Vec<&str> = choices.iter().map(|c| c.0.as_str()).collect();
            assert_eq!(ids_shown, vec!["bunk.squad", "bunk.lead", "bunk.close"], "{size}: no talk waits: {choices:?}");
            assert!(choices[0].1.starts_with("1  Take into the squad") && choices[0].2, "{size}: {choices:?}");
            assert!(!choices[1].2 && choices[1].1.contains("(Only a member of the squad"), "{size}: a benched member's lead is refused in its row: {choices:?}");
            assert!(choices[2].1.contains(FAREWELL) && choices[2].2, "{size}: {choices:?}");
            if size == "Normal" {
                shot(&g, "bunk_card_talkbox");
            } else {
                shot(&g, "bunk_card_talkbox_xl");
            }
        }
        // key 1 takes the benched member into the squad, as the click
        let mut card = BunkCard::new(&ids[2]);
        frame(&mut card, &mut cx, &mut g, 1.0);
        card.key(&mut cx, Key::Char('1'));
        assert!(cx.gs().party.active_members().iter().any(|m| m.id == ids[2]), "1 picks the first choice");
        // the waiting talk is a choice of its own
        let (mut cx, ids) = hub_cx(true);
        let mut card = BunkCard::new(&ids[1]);
        frame(&mut card, &mut cx, &mut g, 1.0);
        assert!(card.widgets().iter().any(|w| w["id"] == "bunk.talk" && w["kind"] == "choice" && w["enabled"] == json!(true)), "the Talk choice");
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
