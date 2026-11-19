//! The hub's bunks on the region map (BUNKS1 BK1-BK4, BK8-BK12; engine-only, no Godot original): the bunk states'
//! art, the members idling at the tables and their bunks, the candles and speech marks of waiting talks, and the click
//! card of a member (portrait, name, condition, a waiting talk; Take into the squad or Stand down, Take the lead,
//! Talk) in the dialogue box, and each occupied bunk's brass wall plate, class blanket, founder's gear and keepsake, a
//! benched member asleep in the bunk and the nooks' plank floor and screens (HUB-BUNKS2, bunk layout B). The rules are the kit's (`emberwake_world::bunks`, `emberwake_world::talks`); this file draws and asks.

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
        // PR7-41 (PT-36): a member with unspent points says so on its bunk (a benched recruit too)
        Some(m) => {
            let pts = m.unspent(d);
            let word = kit::condition(d, gs, m).to_string();
            (progression::display_name(d, m), if pts > 0 { format!("{word} · {pts} point{} to spend", if pts == 1 { "" } else { "s" }) } else { word })
        }
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

// ---- The wall plate and the bunk's dressing (BK2, HUB-BUNKS2) -------------------------------------------------------

/// The niche bunk art's measure (AI-17 `kiln_bunk_<state>`, in art px of its square): the art's side and the bed's
/// width (centred).
pub const ART_SIDE: i32 = 80;
pub const BED_W: i32 = 42;
/// HUB-BUNKS2 (2): the plate's widest (art px of the bunk's art scale), and the least gap (screen px) between two plates
/// of a row of bunks (a plate is at most its nearest neighbour's centre distance less the gap, `plate_room`).
pub const PLATE_MAX_W: i32 = 96;
pub const PLATE_GAP: i32 = 2;
/// HUB-BUNKS2 (2): the plate's text size (logical px; the take 2 mock's 10 px at 720p): the bunks stand 2.5 cells
/// apart, so a `SMALL` name does not fit between two neighbours.
pub const PLATE_TEXT: f32 = 5.0;
/// HUB-BUNKS2 (2): the class glyph on a plate (`ui/markers/class_<class id>.png`, placeholders until Art/UI
/// `icons_class_glyphs_1`; the class letter in a ring without it).
pub const CLASS_GLYPH_DIR: &str = "ui/markers/class_";
/// HUB-BUNKS2 (3): the prop arts a bunk's dressing draws from (each `props/<set>/<art>/v<n>.png` along the world
/// lineage; placeholders until Art/UI `props_hub_founder_bunks_1`): the class blanket over the bed
/// (`<bunk art>_blanket_<class id>`), a founder's gear on the wall pegs (`bunk_gear_<character id>`) and keepsake on the
/// bed (`bunk_keepsake_<character id>`); item (5): the nooks' plank floor run and the timber screen between two bunks.
pub const BLANKET_ART: &str = "_blanket_";
pub const GEAR_ART: &str = "bunk_gear_";
pub const KEEPSAKE_ART: &str = "bunk_keepsake_";
pub const NOOK_FLOOR_ART: &str = "bunk_floor";
pub const NOOK_SCREEN_ART: &str = "bunk_screen";

/// HUB-BUNKS2: how a bunk art is laid out (art px of its PNG): its size, where a sleeper's head lies on the pillow
/// (top-left; the resting pose stand-in until AI-17), where the keepsake sits, where the founder's gear hangs (None: no
/// wall beside it), whether its head is to the south (a free-standing bunk: the plate on its headboard at the bottom;
/// else a niche bunk on the north wall: the plate on the wall above its arch) and where the nook floor starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedArt {
    pub size: (i32, i32),
    pub head: (i32, i32),
    pub keepsake: (i32, i32),
    pub gear: Option<(i32, i32)>,
    pub head_south: bool,
    pub floor_y: i32,
}

/// The sleeper's head (art px, the unit frame's head cut: frame x 12..36, y 4..22 of 48).
pub const HEAD: (i32, i32) = (24, 18);

/// `BedArt` of bunk art `art`: the south-facing timber bunk (`timber_bunk`, the Timber Frames pair), else the niche
/// bunk (`kiln_bunk`), measured from the take 2 mock (`hub_bunks_take2_mock.py`).
pub fn bed_art(art: &str) -> BedArt {
    if art.starts_with("timber_bunk") {
        BedArt { size: (48, 80), head: (12, 58), keepsake: (30, 10), gear: None, head_south: true, floor_y: 80 }
    } else {
        BedArt { size: (ART_SIDE, ART_SIDE), head: (28, 40), keepsake: (46, 58), gear: Some((62, 8)), head_south: false, floor_y: 39 }
    }
}

/// `r` scaled from art px of `bed` drawn in `dr` (the bunk's drawn rect) to screen px.
fn art_rect(bed: &BedArt, dr: Rect, x: i32, y: i32, w: i32, h: i32) -> Rect {
    let (sx, sy) = (|v: i32| v * dr.w / bed.size.0.max(1), |v: i32| v * dr.h / bed.size.1.max(1));
    Rect::new(dr.x + sx(x), dr.y + sy(y), sx(w).max(1), sy(h).max(1))
}

/// HUB-BUNKS2 (2): the room a plate of the bunk drawn in `r` has among the hub's bunks `all` (their drawn rects): the
/// least horizontal distance (screen px) between its centre and the centre of another bunk whose rows overlap it
/// (`i32::MAX` when none). Plates are centred on their bunks, so plates at most `room` less `PLATE_GAP` wide never meet.
pub fn plate_room(r: Rect, all: &[Rect]) -> i32 {
    let mid = |o: &Rect| 2 * o.x + o.w;
    all.iter().filter(|o| **o != r && o.y < r.y + r.h && r.y < o.y + o.h).map(|o| (mid(o) - mid(&r)).abs() / 2).min().unwrap_or(i32::MAX)
}

/// The width (screen px) a plate needs to show `name` whole: the class glyph and the name with their margins.
pub fn plate_need(p: &mut Painter, name: &str) -> i32 {
    let side = p.line_h(PLATE_TEXT).ceil() as i32;
    1 + side + 1 + p.text_w(name, PLATE_TEXT).ceil() as i32 + 2
}

/// HUB-BUNKS2 (2): the brass plate of an occupied bunk of art `bed` drawn in `r`: centred on the bunk; a niche bunk's
/// on the wall just above its arch (its bottom a px over the art's top), a free-standing bunk's on its headboard (its
/// bottom a px over the art's bottom); as wide as the glyph and the name need (`plate_need`), at least the bed and at
/// most `PLATE_MAX_W` (art px) and `room` less `PLATE_GAP` (that bound wins).
pub fn plate_rect(p: &mut Painter, r: Rect, bed: &BedArt, name: &str, room: i32) -> Rect {
    let h = p.line_h(PLATE_TEXT).ceil() as i32 + 2;
    let want = plate_need(p, name);
    let scale = |v: i32| v * r.w / bed.size.0.max(1);
    let most = scale(PLATE_MAX_W).min(room.saturating_sub(PLATE_GAP)).max(1);
    let w = want.max(scale(BED_W.min(bed.size.0))).min(most);
    let mid = r.x + r.w / 2;
    let y = if bed.head_south { r.y + r.h - h - 1 } else { r.y - h - 1 };
    Rect::new(mid - w / 2, y, w, h)
}

/// The name a plate of width `w` shows: the whole name, else its first word (the take 2 mock), shortened by the draw.
pub fn plate_name(p: &mut Painter, name: &str, w: i32) -> String {
    if plate_need(p, name) <= w {
        name.to_string()
    } else {
        name.split_whitespace().next().unwrap_or(name).to_string()
    }
}

/// Draws the plate `pr` (`plate_rect`): a brass-edged plate, the class glyph (`CLASS_GLYPH_DIR`; the class name's first
/// letter in a ring without the art) at the left and the member's name (`plate_name`, shortened to fit).
pub fn draw_plate(ui: &mut Ui<impl Clone>, cx: &mut Cx, pr: Rect, name: &str, class_id: &str, class: &str) {
    ui.p.panel(pr, th().card, th().gold, 1.0);
    let side = pr.h - 2;
    let gr = Rect::new(pr.x + 1, pr.y + 1, side, side);
    let px = (side as f32 * ui.p.v.s).round().max(1.0) as i32;
    match cx.art.image(&format!("{CLASS_GLYPH_DIR}{class_id}.png"), px, px) {
        Some(img) => ui.p.image(img, gr),
        None => {
            let letter = class.chars().next().map_or('?', |c| c.to_ascii_uppercase());
            aegis_ui::ui::draw_glyph(&mut ui.p, aegis_ui::ui::Glyph::Letter(letter), gr, th().gold);
        }
    }
    let tx = pr.x + 1 + side + 1;
    let shown = plate_name(&mut ui.p, name, pr.w);
    ui.p.text_fit(tx as f32, pr.y as f32 + 1.0, &shown, PLATE_TEXT, (pr.x + pr.w - 2 - tx) as f32, th().text);
}

/// A prop art's file along the world lineage (`props/<set>/<art>/v<n>.png`), None without one.
pub fn prop_file(cx: &mut Cx, lineage: &[String], art: &str) -> Option<String> {
    let dirs = emberwake_content::art::prop_dirs(lineage, art);
    let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    cx.art.pick(&dirs, Some(0), 0, 0)
}

/// Draws prop art `art` (its file along `lineage`) in `r`; whether it drew.
fn draw_art(ui: &mut Ui<impl Clone>, cx: &mut Cx, lineage: &[String], art: &str, r: Rect) -> bool {
    let Some(file) = prop_file(cx, lineage, art) else { return false };
    let (w, h) = ((r.w as f32 * ui.p.v.s).round().max(1.0) as i32, (r.h as f32 * ui.p.v.s).round().max(1.0) as i32);
    match cx.art.image(&file, w, h) {
        Some(img) => {
            ui.p.image(img, r);
            true
        }
        None => false,
    }
}

/// What a bunk's dressing drew (the agent link and the tests): the blanket's class, the founder's gear and keepsake
/// and whether its member sleeps there.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dressed {
    pub blanket: String,
    pub gear: bool,
    pub keepsake: bool,
    pub asleep: bool,
}

/// HUB-BUNKS2 (3), (4): dresses bunk `b` (art `bed`, drawn in `r`) after its art: an occupied made bed (active or
/// benched) takes its member's class blanket; a benched member sleeps there (the unit frame's head on the pillow, the
/// resting pose stand-in until AI-17); a named founder (a member with a character) gets their keepsake on the bed and,
/// on a niche bunk, their gear on the wall pegs beside the arch. Recruits get the class colour only.
pub fn dress(ui: &mut Ui<impl Clone>, cx: &mut Cx, lineage: &[String], b: &Bunk, bed: &BedArt, r: Rect) -> Dressed {
    let mut out = Dressed::default();
    let d = cx.data.clone();
    let gs = cx.gs().clone();
    let Some(m) = gs.party.member(&b.member) else { return out };
    let class = m.class_id(&d);
    if matches!(b.state, kit::BunkState::Active | kit::BunkState::Benched) && draw_art(ui, cx, lineage, &format!("{}{BLANKET_ART}{class}", b.art), r) {
        out.blanket = class.clone();
    }
    if b.state == kit::BunkState::Benched {
        out.asleep = true;
        let (body, palette) = member_art(&d, &gs, m);
        if let Some(body) = cx.art.first_body(&emberwake_content::art::body_chain(&body)) {
            // the frame drawn at the bed's art scale, its head cut onto the pillow
            let head = art_rect(bed, r, bed.head.0, bed.head.1, HEAD.0, HEAD.1);
            let side = 48 * r.w / bed.size.0.max(1);
            let frame = Rect::new(head.x - 12 * r.w / bed.size.0.max(1), head.y - 4 * r.h / bed.size.1.max(1), side, side);
            let px = (side as f32 * ui.p.v.s).round().max(1.0) as i32;
            let face = aegis_core::assets::Facing { view: aegis_core::assets::View::Down, left: false };
            let player = aegis_core::assets::ClipPlayer::default();
            if let Some((_, img)) = cx.art.played_facing(&body, &palette, &player, 0, px, face) {
                let kept = ui.p.c.clip;
                let cut = ui.p.v.rect(head);
                ui.p.c.clip = Some(kept.map_or(cut, |c| c.intersect(cut)));
                ui.p.image(img, frame);
                ui.p.c.clip = kept;
            }
        }
    }
    let character = json::s(d.resolve(&gs.world_id, "characters", &m.lore_ref), "id");
    if !character.is_empty() {
        let keep = format!("{KEEPSAKE_ART}{character}");
        if let Some(file) = prop_file(cx, lineage, &keep) {
            let (w, h) = cx.art.size_of(&file).unwrap_or((12, 12));
            out.keepsake = draw_art(ui, cx, lineage, &keep, art_rect(bed, r, bed.keepsake.0, bed.keepsake.1, w, h));
        }
        if let Some((gx, gy)) = bed.gear {
            let gear = format!("{GEAR_ART}{character}");
            if let Some(file) = prop_file(cx, lineage, &gear) {
                let (w, h) = cx.art.size_of(&file).unwrap_or((22, 22));
                out.gear = draw_art(ui, cx, lineage, &gear, art_rect(bed, r, gx, gy, w, h));
            }
        }
    }
    out
}

/// HUB-BUNKS2 (5) `[PROVISIONAL]`: the nooks of the niche bunks drawn in `rects` (one row each side of a door): the
/// hall's plank floor runs under each run of neighbouring bunks (from the art's `floor_y` down), a low timber screen
/// stands between two neighbours and a dark timber step edges the run. Drawn before the bunks. The runs it drew.
pub fn draw_nooks(ui: &mut Ui<impl Clone>, cx: &mut Cx, lineage: &[String], rects: &[Rect]) -> Vec<Rect> {
    let mut row: Vec<Rect> = rects.to_vec();
    row.sort_by_key(|r| (r.y, r.x));
    let bed = bed_art("kiln_bunk");
    let mut runs: Vec<Vec<Rect>> = Vec::new();
    for r in row {
        match runs.last_mut() {
            Some(run) if run.last().is_some_and(|l| l.y == r.y && r.x - (l.x + l.w) <= r.w / 2) => run.push(r),
            _ => runs.push(vec![r]),
        }
    }
    let mut out = Vec::new();
    for run in runs.iter().filter(|run| run.len() > 1) {
        let (first, last) = (run[0], run[run.len() - 1]);
        let top = first.y + bed.floor_y * first.h / bed.size.1;
        let gap = (run[1].x - (first.x + first.w)).max(0) / 2;
        let floor = Rect::new(first.x - gap, top, last.x + last.w + gap - (first.x - gap), first.y + first.h - top);
        if let Some(file) = prop_file(cx, lineage, NOOK_FLOOR_ART) {
            let (fw, fh) = cx.art.size_of(&file).unwrap_or((64, 40));
            let (tw, thh) = (fw * first.w / bed.size.0, fh * first.h / bed.size.1);
            let kept = ui.p.c.clip;
            let cut = ui.p.v.rect(floor);
            ui.p.c.clip = Some(kept.map_or(cut, |c| c.intersect(cut)));
            let mut x = floor.x;
            while x < floor.x + floor.w {
                draw_art(ui, cx, lineage, NOOK_FLOOR_ART, Rect::new(x, floor.y, tw.max(1), thh.max(floor.h)));
                x += tw.max(1);
            }
            ui.p.c.clip = kept;
        }
        let step = (2 * first.h / bed.size.1).max(2);
        ui.p.fill(Rect::new(floor.x, floor.y + floor.h - step, floor.w, step), th().game("bunk_step", rgb(70, 46, 28)));
        for (a, b) in run.iter().zip(run.iter().skip(1)) {
            let mid = (a.x + a.w + b.x) / 2;
            let (sw, sh) = ((6 * a.w / bed.size.0).max(2), (44 * a.h / bed.size.1).max(4));
            let sr = Rect::new(mid - sw / 2, floor.y + floor.h - step - sh + (2 * a.h / bed.size.1), sw, sh);
            if !draw_art(ui, cx, lineage, NOOK_SCREEN_ART, sr) {
                ui.p.fill(sr, th().game("bunk_screen", rgb(120, 84, 50)));
            }
        }
        out.push(floor);
    }
    out
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
pub(crate) mod tests {
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
    pub(crate) fn hub_cx(talk: bool) -> (Cx, Vec<String>) {
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
        // PR7-19 / PR7-23: a bunk prop for every bunk the cap can reach (`bunks_max`, the Timber Frames bunks too),
        // all in view; the ones past today's cap draw empty
        let cap = roster::bunk_cap(cx.gs()) as usize;
        let max = cx.gs().party.roster.manifest.bunks_max as usize;
        assert!(cap <= max);
        assert_eq!(bunks.len(), max, "a bunk prop per reachable bunk, all in view: {bunks:?}");
        let states: Vec<(String, String)> = bunks.iter().map(|b| (b["member"].as_str().unwrap().to_string(), b["state"].as_str().unwrap().to_string())).collect();
        let want: Vec<(String, String)> = [(ids[0].as_str(), "active"), (ids[1].as_str(), "active"), (ids[2].as_str(), "benched"), (ids[3].as_str(), "crypt")]
            .iter()
            .map(|(m, s)| (m.to_string(), s.to_string()))
            .chain(std::iter::repeat_n(("".to_string(), "empty".to_string()), max - 4))
            .collect();
        assert_eq!(states, want);
        for b in &bunks {
            let file = b["file"].as_str().unwrap_or("");
            // HUB-BUNKS2 (1): the free-standing Timber Frames bunks (`timber_bunk`) have no state art yet (placeholder)
            assert!(file.contains(&format!("kiln_bunk_{}/", b["state"].as_str().unwrap())) || file.contains("timber_bunk/"), "the state's art: {b}");
        }
        // BK9: the active member who does not lead idles at the tables; HUB-BUNKS2 (4): the benched one sleeps in its
        // bunk (`hub_bunks2_bunk_states_with_a_benched_member_asleep`); the crypt member is not shown
        let lead = crate::region::leader_id(&cx);
        let at = spots(&map, &cx);
        assert!(!at.iter().any(|(m, _)| *m == lead || *m == ids[2] || *m == ids[3]), "{at:?}");
        assert!(at.iter().any(|(m, _)| *m == ids[1]), "the active member idles: {at:?}");
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
        // PR7-41 (PT-36): unspent points follow the condition word
        let pts = cx.gs().party.member(&ids[2]).unwrap().unspent(&cx.data);
        let word = if pts > 0 { format!("{} · {pts} point{} to spend", kit::RESTING, if pts == 1 { "" } else { "s" }) } else { kit::RESTING.to_string() };
        assert_eq!(w["tip"], json!([name, word]), "benched and well: resting");
        cx.gs_mut().party.members[2].add_condition(roster::WOUNDED);
        map.draw(&mut cx, &mut g, (r.0 + r.2 / 2, r.1 + r.3 / 2), true);
        let w = map.widgets().into_iter().find(|w| w["id"] == format!("{BUNK_PREFIX}bunk_3")).unwrap();
        assert!(w["tip"][1].as_str().unwrap_or_default().starts_with(kit::WOUNDED_WORD), "{}", w["tip"]);
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

    /// HUB-BUNKS2 (2), replacing the HUB-POLISH1 bed plate and the PR7-23 two-height stagger (`FB-314`, `FB-316`): every
    /// occupied bunk state (active, benched, crypt) wears a brass plate with its member's name and class, centred on the
    /// bunk: a niche bunk's on the wall just above its arch, a free-standing bunk's on its headboard; at most its room
    /// among its neighbours less `PLATE_GAP`, so no two plates overlap, at every text size; an empty bunk wears none; at
    /// Normal every shipped party character's name reads whole on every occupied bunk (larger sizes may show the first
    /// word or shorten it).
    #[test]
    fn hub_bunks2_each_occupied_bunk_wears_a_brass_wall_plate() {
        let (mut cx, _) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        for (size, scale) in aegis_ui::screen::TEXT_SIZES {
            frame(&mut map, &mut cx, &mut g, scale as f32);
            let bunks = map.state(&cx)["bunks"].as_array().cloned().expect("the hub's bunks");
            let mut states: Vec<String> = Vec::new();
            let mut plates: Vec<(String, (i64, i64, i64, i64))> = Vec::new();
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
                assert_eq!(plate["class_id"], json!(m.class_id(&cx.data)), "{size}: {b}");
                let n = |v: &Value, i: usize| v[i].as_i64().unwrap();
                let (bx, by, bw, bh) = (n(&b["drawn"], 0), n(&b["drawn"], 1), n(&b["drawn"], 2), n(&b["drawn"], 3));
                let (px, py, pw, ph) = (n(&plate["rect"], 0), n(&plate["rect"], 1), n(&plate["rect"], 2), n(&plate["rect"], 3));
                assert!(((px + pw / 2) - (bx + bw / 2)).abs() <= 1, "{size}: centred on the bunk: {b}");
                if plate["wall"] == json!(true) {
                    assert_eq!(py + ph, by - 1, "{size}: on the wall just above the arch: {b}");
                } else {
                    assert_eq!(py + ph, by + bh - 1, "{size}: on the headboard: {b}");
                }
                let room = plate["room"].as_i64().unwrap();
                assert!(pw > 0 && ph > 0 && pw <= (room - PLATE_GAP as i64).max(1), "{size}: within its room: {b}");
                plates.push((b["prop"].as_str().unwrap().to_string(), (px, py, pw, ph)));
            }
            assert!(plates.len() >= 2, "{size}: neighbour plates to compare: {plates:?}");
            for (i, (a, (ax, ay, aw, ah))) in plates.iter().enumerate() {
                for (b, (bx, by, bw, bh)) in &plates[i + 1..] {
                    let apart = ax + aw <= *bx || bx + bw <= *ax || ay + ah <= *by || by + bh <= *ay;
                    assert!(apart, "{size}: the plates of {a} and {b} overlap: {plates:?}");
                }
            }
            states.sort();
            states.dedup();
            assert_eq!(states, vec!["active", "benched", "crypt"], "{size}: every occupied state");
            if size == "Normal" {
                let names: Vec<String> = cx.data.catalogs.own_entries(WORLD, "characters").iter().filter(|c| json::s(c, "role") == "party").map(|c| json::s(c, "name")).collect();
                assert!(!names.is_empty());
                let keep: Vec<String> = cx.gs().party.members.iter().map(|m| m.name.clone()).collect();
                for name in &names {
                    for m in cx.gs_mut().party.members.iter_mut() {
                        m.name = name.clone();
                    }
                    frame(&mut map, &mut cx, &mut g, scale as f32);
                    for b in map.state(&cx)["bunks"].as_array().unwrap().iter().filter(|b| b.get("plate").is_some()) {
                        assert_eq!(b["plate"]["full"], json!(true), "{size}: '{name}' reads whole: {b}");
                        assert_eq!(b["plate"]["shown"], json!(name), "{size}: {b}");
                    }
                }
                for (m, n) in cx.gs_mut().party.members.iter_mut().zip(keep) {
                    m.name = n;
                }
            }
            match size {
                "Normal" => shot(&g, "hub_plates"),
                "Extra large" => shot(&g, "hub_plates_xl"),
                _ => {}
            }
        }
    }

    /// HUB-BUNKS2 (1), (3)-(5) (bunk states test): on the shipped hub (layout B) the six niche bunks stand three each
    /// side of the door, their centres an equal step apart, and the Timber Frames pair free-standing south of them; the
    /// active, benched and crypt bunks show their states; a benched member sleeps in their bunk (not idling in the
    /// room) and their bunk still opens their card; made beds (active, benched) take the class blanket, a crypt bed
    /// none; named founders get their gear and keepsake; the niche bunks' nooks are two floor runs (one each side of the
    /// door).
    #[test]
    fn hub_bunks2_bunk_states_with_a_benched_member_asleep() {
        let (mut cx, ids) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())).clone();
        let all = kit::bunks(cx.gs(), &t);
        let niche: Vec<&Bunk> = all.iter().filter(|b| !bed_art(&b.art).head_south).collect();
        let free: Vec<&Bunk> = all.iter().filter(|b| bed_art(&b.art).head_south).collect();
        let cap = roster::bunk_cap(cx.gs()) as usize;
        assert_eq!(niche.len(), cap, "a niche bunk per bunk of today's cap: {all:?}");
        assert_eq!(free.len(), all.len() - cap, "the bunks past the cap stand free: {all:?}");
        // centres in half cells: one row, an equal step apart each side of the door, three each side
        let mid = |b: &Bunk| 2 * b.foot.0 + b.foot.2;
        assert!(niche.iter().all(|b| b.foot.1 == niche[0].foot.1), "one row on the north wall");
        let steps: Vec<i64> = niche.windows(2).map(|w| mid(w[1]) - mid(w[0])).collect();
        let step = steps[0];
        assert!(step > 0 && steps.iter().filter(|s| **s == step).count() == steps.len() - 1, "an equal step but at the door: {steps:?}");
        let door = steps.iter().position(|s| *s != step).expect("a wider step at the door");
        assert_eq!(door + 1, cap / 2, "half each side of the door: {steps:?}");
        assert!(free.iter().all(|b| b.foot.1 > niche[0].foot.1), "the free-standing pair south of the row");
        // the states and the dressing
        let bunks = map.state(&cx)["bunks"].as_array().cloned().unwrap();
        let entry = |m: &str| bunks.iter().find(|b| b["member"] == json!(m)).cloned().unwrap_or_else(|| panic!("{m}'s bunk"));
        for (i, state) in [(0, "active"), (1, "active"), (2, "benched"), (3, "crypt")] {
            let e = entry(&ids[i]);
            assert_eq!(e["state"], json!(state), "{e}");
            let m = cx.gs().party.member(&ids[i]).unwrap();
            let dressed = &e["dressed"];
            assert_eq!(dressed["asleep"], json!(state == "benched"), "only the benched member sleeps there: {e}");
            let class = if state == "crypt" { String::new() } else { m.class_id(&cx.data) };
            assert_eq!(dressed["blanket"], json!(class), "the class blanket on a made bed: {e}");
            assert_eq!((dressed["gear"].clone(), dressed["keepsake"].clone()), (json!(true), json!(true)), "a founder's gear and keepsake: {e}");
        }
        let at = spots(&map, &cx);
        assert!(!at.iter().any(|(m, _)| *m == ids[2]), "the sleeper idles nowhere: {at:?}");
        let mut asleep: Vec<String> = kit::sleepers(cx.gs(), &t).into_iter().map(|b| b.member).collect();
        asleep.sort();
        assert_eq!(asleep, vec![ids[2].clone()]);
        let prop = entry(&ids[2])["prop"].as_str().unwrap().to_string();
        map.click_id(&mut cx, &format!("{BUNK_PREFIX}{prop}")).expect("the sleeper's bunk is clickable");
        match cx.nav.pop() {
            Some(Nav::Push(mut card)) => {
                frame(card.as_mut(), &mut cx, &mut g, 1.0);
                assert_eq!(card.state(&cx)["member"], json!(ids[2]), "the sleeper's card");
            }
            _ => panic!("the card opens"),
        }
        assert_eq!(map.state(&cx)["nooks"].as_array().map(|a| a.len()), Some(2), "a nook floor run each side of the door");
        shot(&g, "hub_bunks2");
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
            shot(&g, if size == "Normal" { "bunk_card_talkbox" } else { "bunk_card_talkbox_xl" });
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
