//! The hub's bunks on the region map (BUNKS1 BK1-BK4, BK8-BK12; engine-only, no Godot original): the bunk states'
//! art, the members idling at the tables and their bunks, the candles and speech marks of waiting talks, and the click
//! card of a member (portrait, name, condition, a waiting talk; Take into the squad or Stand down, Take the lead,
//! Talk) in the dialogue box, and each occupied bunk's brass plate, class blanket, gear on its peg board and a
//! benched member asleep in the bunk (HUB-BUNKS2); the dressed north wall with an arched niche and a founder's
//! lunette cloth over each niche bed, and the free-standing pair's plates on posts (BUNKS-LAYOUT3, the take 8 mock).
//! The rules are the kit's (`emberwake_world::bunks`, `emberwake_world::talks`); this file draws and asks.

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

/// The niche bed's width (`niche_bunk`, art px; the art is the bed; placeholder: a copy of `cot_side`).
pub const ART_SIDE: i32 = 48;
/// HUB-BUNKS2 (2): the plate's widest (art px of the bunk's art scale), and the least gap (screen px) between two plates
/// of a row of bunks (a plate is at most its nearest neighbour's centre distance less the gap, `plate_room`).
pub const PLATE_MAX_W: i32 = 96;
pub const PLATE_GAP: i32 = 2;
/// HUB-BUNKS2 (2), BUNKS-LAYOUT3: the plate's text size (logical px; the take 8 mock's 10 px at 720p): the bunks
/// stand 2.5 cells apart, so a `SMALL` name does not fit between two neighbours (the two-line plate is `PR7-46`).
pub const PLATE_TEXT: f32 = 5.0;
/// BUNKS-LAYOUT3 (2): the one brass plate of every bunk, founders and recruits alike (theme game colours, the take 8
/// mock's brass where the theme has none).
pub fn plate_colours() -> (aegis_ui::canvas::Rgba, aegis_ui::canvas::Rgba, aegis_ui::canvas::Rgba) {
    (th().game("bunk_plate", rgb(92, 66, 34)), th().game("bunk_plate_edge", rgb(176, 132, 64)), th().game("bunk_plate_text", rgb(236, 220, 180)))
}
/// BUNKS-LAYOUT3 (7): an empty free-standing bunk's blank plate width (art px, the take 8 mock's).
pub const BLANK_PLATE_W: i32 = 27;
/// HUB-BUNKS2 (2): the class glyph on a plate (`ui/markers/class_<class id>.png`, placeholders until Art/UI
/// `icons_class_glyphs_1`; the class letter in a ring without it).
pub const CLASS_GLYPH_DIR: &str = "ui/markers/class_";
/// HUB-BUNKS2 (3), BUNKS-LAYOUT3: the prop arts a bunk's dressing draws from (each `props/<set>/<art>/v<n>.png` along
/// the world lineage; placeholders cut from the take 8 mock until Art/UI's art lands): the class blanket over the bed
/// (`<bunk art>_blanket_<class id>`), a founder's gear on the peg board (`bunk_gear_<character id>`); the north wall's
/// dressed stone (`bunk_wall`, a tile repeated along the wall), the plank strip at its foot (`bunk_floor`), each niche
/// bunk's arched niche with its keystone (`bunk_niche`), a founder's lunette cloth in the arch head (`bunk_lunette`; the
/// emblem on it is `PR7-52`) and the peg board right of the bed (`bunk_pegboard`).
pub const BLANKET_ART: &str = "_blanket_";
pub const GEAR_ART: &str = "bunk_gear_";
pub const WALL_ART: &str = "bunk_wall";
pub const FLOOR_ART: &str = "bunk_floor";
pub const NICHE_ART: &str = "bunk_niche";
pub const LUNETTE_ART: &str = "bunk_lunette";
pub const PEGBOARD_ART: &str = "bunk_pegboard";

/// BUNKS-LAYOUT3: how a bunk is laid out, in art px of its bed art from the bed's drawn top-left (measured from the take
/// 8 mock, `art_src/style/hub_bunks_take8_mock.py`, at 720p where an art px is a screen px): the art's size; how far the
/// bed is drawn from where the prop's art would sit (bottom-centred on the footprint): a niche bed 8 px west, so its
/// peg board fits east, and 11 px forward of the wall foot; whether it is a niche bunk on the north wall (the wall, the
/// niche, the lunette, the peg board and the wall plate) or free-standing (head on a side wall, the plate on a post by
/// the pillow); the sleeper's front `idle_down` frame (48 px, its top-left) and the part of it shown (x, y, w, h: the
/// head and shoulders low on the pillow), the bed rows from which the bed and blanket are drawn again over the
/// sleeper (None: no cover); and the plate's place (its centre x and its bottom: a larger text size grows it up).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedArt {
    pub size: (i32, i32),
    pub offset: (i32, i32),
    pub niche: bool,
    pub frame: (i32, i32),
    pub head: (i32, i32, i32, i32),
    pub cover_y: Option<i32>,
    pub plate_mid: (i32, i32),
}

/// The niche bunk's dressing (art px from the bed's top-left, take 8): the wall band's top and the plank strip's top
/// and bottom (the wall foot: the footprint's bottom edge), the niche, the lunette and the peg board's top-left, where
/// the gear hangs (its centre x and top) on the board.
pub const WALL_TOP: i32 = -36;
pub const STRIP_TOP: i32 = 26;
pub const WALL_FOOT: i32 = 69;
pub const NICHE_AT: (i32, i32) = (-1, -18);
pub const LUNETTE_AT: (i32, i32) = (4, -12);
pub const PEGBOARD_AT: (i32, i32) = (50, -18);
pub const GEAR_AT: (i32, i32) = (63, -12);
/// A free-standing bunk's plate post (art px: its width and its reach above and below the bed's top).
pub const POST: (i32, i32, i32) = (4, 6, 7);

/// `BedArt` of bunk art `art`: the free-standing bed lying sideways (`cot`, the Timber Frames pair; `timber_bunk`
/// before BUNKS-LAYOUT3), else the niche bed (`niche_bunk`; `kiln_bunk` before BUNKS-LAYOUT3).
pub fn bed_art(art: &str) -> BedArt {
    if art.starts_with("niche_bunk") || art.starts_with("kiln_bunk") {
        BedArt { size: (ART_SIDE, 80), offset: (-8, 11), niche: true, frame: (0, 6), head: (14, 6, 20, 25), cover_y: Some(27), plate_mid: (24, -20) }
    } else {
        BedArt { size: (96, 64), offset: (0, 0), niche: false, frame: (-5, 6), head: (9, 8, 20, 24), cover_y: None, plate_mid: (34, -6) }
    }
}

/// BUNKS-LAYOUT3: the logical rect bunk art `bed` is drawn in, from `r` (where the prop's art would sit).
pub fn drawn_rect(bed: &BedArt, r: Rect) -> Rect {
    let k = |v: i32| (v * r.w).div_euclid(bed.size.0.max(1));
    Rect::new(r.x + k(bed.offset.0), r.y + k(bed.offset.1), r.w, r.h)
}

/// Art px of a bed art drawn at physical px: the physical point of the art's top-left and the physical px per art px,
/// so the dressing keeps the mock's single pixels at any window size.
#[derive(Debug, Clone, Copy)]
pub struct Pen {
    pub x: f32,
    pub y: f32,
    pub k: f32,
    /// Mirrored left to right (a flipped prop): art x counts from the right.
    pub flip: bool,
    pub w: i32,
}

impl Pen {
    /// The pen of bed `bed` whose prop's art would sit in logical `r` (`drawn_rect` moves it).
    pub fn of(p: &Painter, bed: &BedArt, r: Rect, flip: bool) -> Pen {
        let pr = p.v.rect(r);
        let k = pr.w as f32 / bed.size.0.max(1) as f32;
        Pen { x: pr.x as f32 + bed.offset.0 as f32 * k, y: pr.y as f32 + bed.offset.1 as f32 * k, k, flip, w: bed.size.0 }
    }

    /// Physical rect of art rect (x, y, w, h).
    pub fn rect(&self, x: i32, y: i32, w: i32, h: i32) -> Rect {
        let x = if self.flip { self.w - x - w } else { x };
        let (x0, y0) = ((self.x + x as f32 * self.k).round() as i32, (self.y + y as f32 * self.k).round() as i32);
        let (x1, y1) = ((self.x + (x + w) as f32 * self.k).round() as i32, (self.y + (y + h) as f32 * self.k).round() as i32);
        Rect::new(x0, y0, (x1 - x0).max(1), (y1 - y0).max(1))
    }

    /// The logical rect over art rect (x, y, w, h) (the agent link's).
    pub fn logical(&self, p: &Painter, x: i32, y: i32, w: i32, h: i32) -> Rect {
        let r = self.rect(x, y, w, h);
        let (x0, y0) = p.v.to_logical(r.x as f32, r.y as f32);
        let (x1, y1) = p.v.to_logical((r.x + r.w) as f32, (r.y + r.h) as f32);
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }
}

/// `img` over physical `pr` (nearest sampling, mirrored when `flip`), under the canvas's clip.
fn blit(p: &mut Painter, img: &aegis_ui::canvas::Image, pr: Rect, flip: bool) {
    if pr.w <= 0 || pr.h <= 0 || img.w <= 0 || img.h <= 0 {
        return;
    }
    for y in 0..pr.h {
        let sy = y * img.h / pr.h;
        for x in 0..pr.w {
            let sx = x * img.w / pr.w;
            let sx = if flip { img.w - 1 - sx } else { sx };
            let i = ((sy * img.w + sx) * 4) as usize;
            p.c.blend(pr.x + x, pr.y + y, [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]);
        }
    }
}

/// Draws file `file` (its own size in art px) with `pen` at art (x, y); its physical rect, None without the file.
fn pen_file(ui: &mut Ui<impl Clone>, cx: &mut Cx, pen: &Pen, file: &str, x: i32, y: i32) -> Option<Rect> {
    let (w, h) = cx.art.size_of(file)?;
    let pr = pen.rect(x, y, w, h);
    let img = cx.art.image(file, pr.w, pr.h)?;
    blit(&mut ui.p, img, pr, pen.flip);
    Some(pr)
}

/// Draws prop art `art` (along `lineage`) with `pen` at art (x, y); its physical rect, None without the art.
fn pen_art(ui: &mut Ui<impl Clone>, cx: &mut Cx, lineage: &[String], pen: &Pen, art: &str, x: i32, y: i32) -> Option<Rect> {
    let file = prop_file(cx, lineage, art)?;
    pen_file(ui, cx, pen, &file, x, y)
}

/// BUNKS-LAYOUT3 (1): the runs of north wall behind the niche bunks of map `t`: from each niche bunk's footprint
/// along its bottom row, the wall cells (`#`) with an open cell south of them (x from, x to exclusive, the row), each
/// run once, west to east.
pub fn wall_runs(t: &Dict, bunks: &[Bunk]) -> Vec<(i64, i64, i64)> {
    let rows: Vec<Vec<char>> = json::aget(t, "rows").iter().map(|r| r.as_str().unwrap_or("").chars().collect()).collect();
    let wall = |x: i64, y: i64| x >= 0 && y >= 0 && rows.get(y as usize).and_then(|r| r.get(x as usize)) == Some(&'#');
    let backed = |x: i64, y: i64| wall(x, y) && !wall(x, y + 1) && rows.get(y as usize + 1).is_some_and(|r| (x as usize) < r.len());
    let mut out: Vec<(i64, i64, i64)> = Vec::new();
    for b in bunks.iter().filter(|b| bed_art(&b.art).niche) {
        let y = b.foot.1 + b.foot.3 - 1;
        let (mut x0, mut x1) = (b.foot.0, b.foot.0 + b.foot.2);
        if !(x0..x1).all(|x| backed(x, y)) {
            continue;
        }
        while backed(x0 - 1, y) {
            x0 -= 1;
        }
        while backed(x1, y) {
            x1 += 1;
        }
        if !out.contains(&(x0, x1, y)) {
            out.push((x0, x1, y));
        }
    }
    out.sort();
    out
}

/// BUNKS-LAYOUT3 (1): the dressed north wall and the niches, drawn once before the bunks: along each wall run (`runs`,
/// logical x from and to and the wall foot y) the stone tile over the wall band and the plank strip at its foot; then
/// each niche bunk's (`niches`: its pen and whether a named founder sleeps there) arched niche with its keystone, and a
/// founder's lunette cloth in the arch head. `k` is the physical px per art px. The runs it drew (logical).
pub fn draw_wall(ui: &mut Ui<impl Clone>, cx: &mut Cx, lineage: &[String], runs: &[(i32, i32, i32)], niches: &[(Pen, bool)], k: f32) -> Vec<Rect> {
    let mut out = Vec::new();
    let kept = ui.p.c.clip;
    let tile = |ui: &mut Ui<_>, cx: &mut Cx, art: &str, x0: i32, x1: i32, top: i32, bottom: i32| {
        let Some(file) = prop_file(cx, lineage, art) else { return };
        let Some((w, h)) = cx.art.size_of(&file) else { return };
        let band = Rect::new(x0, top, x1 - x0, bottom - top);
        ui.p.c.clip = Some(kept.map_or(band, |c| c.intersect(band)));
        let (tw, tall) = (((w as f32 * k).round() as i32).max(1), ((h as f32 * k).round() as i32).max(1));
        let mut y = top;
        while y < bottom {
            let mut x = x0;
            while x < x1 {
                let pen = Pen { x: x as f32, y: y as f32, k, flip: false, w };
                pen_file(ui, cx, &pen, &file, 0, 0);
                x += tw;
            }
            y += tall;
        }
        ui.p.c.clip = kept;
    };
    for &(lx0, lx1, foot) in runs {
        let (x0, x1, f) = (ui.p.v.x(lx0 as f32), ui.p.v.x(lx1 as f32), ui.p.v.y(foot as f32));
        let at = |v: i32| f + (v as f32 * k).round() as i32;
        tile(ui, cx, WALL_ART, x0, x1, at(WALL_TOP - WALL_FOOT), at(STRIP_TOP - WALL_FOOT));
        tile(ui, cx, FLOOR_ART, x0, x1, at(STRIP_TOP - WALL_FOOT), f);
        out.push(Rect::new(lx0, foot - ((WALL_FOOT - WALL_TOP) as f32 * k / ui.p.v.s).round() as i32, lx1 - lx0, ((WALL_FOOT - WALL_TOP) as f32 * k / ui.p.v.s).round() as i32));
    }
    for (pen, founder) in niches {
        pen_art(ui, cx, lineage, pen, NICHE_ART, NICHE_AT.0, NICHE_AT.1);
        if *founder {
            pen_art(ui, cx, lineage, pen, LUNETTE_ART, LUNETTE_AT.0, LUNETTE_AT.1);
        }
    }
    out
}

/// BUNKS-LAYOUT3: draws bunk `b`'s bed (art file `file`, its state's where it has one) with `pen`.
pub fn draw_bed(ui: &mut Ui<impl Clone>, cx: &mut Cx, pen: &Pen, file: Option<&str>) -> bool {
    file.is_some_and(|f| pen_file(ui, cx, pen, f, 0, 0).is_some())
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

/// HUB-BUNKS2 (2), BUNKS-LAYOUT3 (2), (7): the brass plate of a bunk of art `bed` drawn in `dr` (`drawn_rect`;
/// `flip`: mirrored): centred on and standing on the art's `plate_mid` (a niche bunk's on the wall band above its niche, a
/// free-standing bunk's over the post by its pillow); as wide as the glyph and the name need (`plate_need`; `name` ""
/// is the blank plate of an empty bunk, `BLANK_PLATE_W`), at most `PLATE_MAX_W` (art px) and `room` less `PLATE_GAP`
/// (that bound wins).
pub fn plate_rect(p: &mut Painter, dr: Rect, bed: &BedArt, name: &str, room: i32, flip: bool) -> Rect {
    let h = p.line_h(PLATE_TEXT).ceil() as i32 + 2;
    let scale = |v: i32| (v * dr.w).div_euclid(bed.size.0.max(1));
    let want = if name.is_empty() { scale(BLANK_PLATE_W) } else { plate_need(p, name) };
    let most = scale(PLATE_MAX_W).min(room.saturating_sub(PLATE_GAP)).max(1);
    let w = want.min(most).max(1);
    let mx = if flip { bed.size.0 - bed.plate_mid.0 } else { bed.plate_mid.0 };
    let (mid, bottom) = (dr.x + scale(mx), dr.y + scale(bed.plate_mid.1));
    Rect::new(mid - w / 2, bottom - h, w, h)
}

/// BUNKS-LAYOUT3 (7): a free-standing bunk's plate post by the pillow (`POST`, under the plate's centre), with `pen`.
pub fn draw_post(ui: &mut Ui<impl Clone>, pen: &Pen, bed: &BedArt) {
    let (w, up, down) = POST;
    let mx = if pen.flip { bed.size.0 - bed.plate_mid.0 } else { bed.plate_mid.0 };
    let r = Pen { flip: false, ..*pen }.rect(mx - 1, -up, w, up + down);
    let (fill, edge) = (th().game("bunk_screen", rgb(120, 84, 50)), th().game("bunk_step", rgb(70, 46, 28)));
    ui.p.c.fill(r, edge);
    if r.w > 2 && r.h > 2 {
        ui.p.c.fill(Rect::new(r.x + 1, r.y + 1, r.w - 2, r.h - 2), fill);
    }
}

/// BUNKS-LAYOUT3 (7): the blank brass plate of an empty free-standing bunk.
pub fn draw_blank_plate(ui: &mut Ui<impl Clone>, pr: Rect) {
    let (fill, edge, _) = plate_colours();
    ui.p.panel(pr, fill, edge, 0.0);
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
    let (fill, edge, ink) = plate_colours();
    ui.p.panel(pr, fill, edge, 0.0);
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
    ui.p.text_fit(tx as f32, pr.y as f32 + 1.0, &shown, PLATE_TEXT, (pr.x + pr.w - 2 - tx) as f32, ink);
}

/// A prop art's file along the world lineage (`props/<set>/<art>/v<n>.png`), None without one.
pub fn prop_file(cx: &mut Cx, lineage: &[String], art: &str) -> Option<String> {
    let dirs = emberwake_content::art::prop_dirs(lineage, art);
    let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    cx.art.pick(&dirs, Some(0), 0, 0)
}

/// What a bunk's dressing drew (the agent link and the tests): the blanket's class, the gear on the peg board (a
/// founder's own piece, a recruit's weapon), whether its member sleeps there, the pose they sleep in and the shown part
/// of it (logical; None when no one sleeps there).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dressed {
    pub blanket: String,
    pub gear: bool,
    pub board: bool,
    pub asleep: bool,
    pub pose: String,
    pub head: Option<Rect>,
}

/// BUNKS-LAYOUT3 (4): the clip a sleeper is drawn in (the unit's front idle pose, head and shoulders on the pillow).
pub const SLEEP_CLIP: &str = "idle_down";

/// HUB-BUNKS2 (3), (4), BUNKS-LAYOUT3 (3), (4): dresses bunk `b` (art `bed`, drawn by `pen` from file `file`) after its
/// bed: an occupied bed takes its member's class blanket; a benched member sleeps there in the front `idle_down` pose,
/// the head and shoulders low on the pillow and the bed's blanket drawn again over the shoulders (the resting pose
/// stand-in until AI-17); a niche bunk has its peg board right of the bed with the member's gear on it (a founder's
/// own piece, `bunk_gear_<character>`; a recruit's first equipped item's icon).
pub fn dress(ui: &mut Ui<impl Clone>, cx: &mut Cx, lineage: &[String], b: &Bunk, bed: &BedArt, pen: &Pen, file: Option<&str>) -> Dressed {
    let mut out = Dressed::default();
    if bed.niche {
        out.board = pen_art(ui, cx, lineage, pen, PEGBOARD_ART, PEGBOARD_AT.0, PEGBOARD_AT.1).is_some();
    }
    let d = cx.data.clone();
    let gs = cx.gs().clone();
    let Some(m) = gs.party.member(&b.member) else { return out };
    let class = m.class_id(&d);
    let blanket = format!("{}{BLANKET_ART}{class}", b.art);
    if pen_art(ui, cx, lineage, pen, &blanket, 0, 0).is_some() {
        out.blanket = class.clone();
    }
    if b.state == kit::BunkState::Benched {
        out.asleep = true;
        out.pose = SLEEP_CLIP.to_string();
        let (body, palette) = member_art(&d, &gs, m);
        if let Some(body) = cx.art.first_body(&emberwake_content::art::body_chain(&body)) {
            let frame = pen.rect(bed.frame.0, bed.frame.1, 48, 48);
            let (hx, hy, hw, hh) = bed.head;
            let cut = pen.rect(hx, hy, hw, hh);
            let face = aegis_core::assets::Facing { view: aegis_core::assets::View::Down, left: false };
            let player = aegis_core::assets::ClipPlayer::default();
            if let Some((_, img)) = cx.art.played_facing(&body, &palette, &player, 0, frame.w, face) {
                let img = img.clone();
                let kept = ui.p.c.clip;
                ui.p.c.clip = Some(kept.map_or(cut, |c| c.intersect(cut)));
                blit(&mut ui.p, &img, frame, false);
                if let Some(y) = bed.cover_y {
                    let cover = pen.rect(0, y, bed.size.0, bed.size.1 - y);
                    ui.p.c.clip = Some(kept.map_or(cover, |c| c.intersect(cover)));
                    if let Some(f) = file {
                        pen_file(ui, cx, pen, f, 0, 0);
                    }
                    pen_art(ui, cx, lineage, pen, &blanket, 0, 0);
                }
                ui.p.c.clip = kept;
                out.head = Some(pen.logical(&ui.p, hx, hy, hw, hh));
            }
        }
    }
    if !bed.niche {
        return out;
    }
    let character = json::s(d.resolve(&gs.world_id, "characters", &m.lore_ref), "id");
    let (gx, gy) = GEAR_AT;
    if !character.is_empty() {
        if let Some(file) = prop_file(cx, lineage, &format!("{GEAR_ART}{character}")) {
            let w = cx.art.size_of(&file).map_or(22, |s| s.0);
            out.gear = pen_file(ui, cx, pen, &file, gx - w / 2, gy).is_some();
        }
    } else if let Some(item) = m.equipment.values().find(|i| !i.is_empty()) {
        // a recruit's weapon or first piece, its icon at the gear's size (the take 5 mock's 22 px)
        const ICON: i32 = 22;
        let files = crate::tokens::item_icons(&d, item);
        if let Some(file) = files.iter().find(|f| cx.art.size_of(f).is_some()) {
            let pr = pen.rect(gx - ICON / 2, gy, ICON, ICON);
            if let Some(img) = cx.art.image(file, pr.w, pr.h) {
                let img = img.clone();
                blit(&mut ui.p, &img, pr, false);
                out.gear = true;
            }
        }
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
        // all in view; the ones past today's cap draw empty, unless a prop holds their place (BUNKS-UPGRADE-PROPS1:
        // then they do not draw until the cap reaches them)
        let cap = roster::bunk_cap(cx.gs()) as usize;
        let max = cx.gs().party.roster.manifest.bunks_max as usize;
        assert!(cap <= max);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())).clone();
        assert_eq!(kit::bunk_props(&t).len(), max, "a bunk prop per reachable bunk");
        let shown = kit::bunk_props(&t).iter().filter(|p| kit::prop_shown(cx.gs(), &t, p)).count();
        assert_eq!(bunks.len(), shown, "every bunk that shows, all in view: {bunks:?}");
        let max = shown;
        let states: Vec<(String, String)> = bunks.iter().map(|b| (b["member"].as_str().unwrap().to_string(), b["state"].as_str().unwrap().to_string())).collect();
        let want: Vec<(String, String)> = [(ids[0].as_str(), "active"), (ids[1].as_str(), "active"), (ids[2].as_str(), "benched"), (ids[3].as_str(), "crypt")]
            .iter()
            .map(|(m, s)| (m.to_string(), s.to_string()))
            .chain(std::iter::repeat_n(("".to_string(), "empty".to_string()), max - 4))
            .collect();
        assert_eq!(states, want);
        for b in &bunks {
            let file = b["file"].as_str().unwrap_or("");
            // BUNKS-LAYOUT3: the beds (`niche_bunk` on the north wall, `cot` free-standing) have no state art yet; the
            // state shows in the dressing (blanket, sleeper, plate)
            let state_art = format!("_{}/", b["state"].as_str().unwrap());
            assert!(file.contains(&state_art) || file.contains("niche_bunk/") || file.contains("/cot/"), "the bed's art: {b}");
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
                // BUNKS-LAYOUT3 (2), (7): a niche bunk's plate centred over the bed on the wall band above its niche
                // (the keystone's top `NICHE_AT`); a free-standing bunk's over the post by its pillow, above the bed
                let bed = bed_art(b["file"].as_str().unwrap_or("").rsplit('/').nth(1).unwrap_or(""));
                let _ = bh;
                if plate["wall"] == json!(true) {
                    assert!(((px + pw / 2) - (bx + bw / 2)).abs() <= 1, "{size}: centred on the bunk: {b}");
                    assert!(py + ph <= by + NICHE_AT.1 as i64 * bw / bed.size.0 as i64, "{size}: on the wall above the niche: {b}");
                } else {
                    let mx = if b["flip"] == json!(true) { bed.size.0 - bed.plate_mid.0 } else { bed.plate_mid.0 } as i64;
                    assert!(((px + pw / 2) - (bx + mx * bw / bed.size.0 as i64)).abs() <= 1, "{size}: over the post: {b}");
                    assert!(py + ph <= by, "{size}: above the bed: {b}");
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
    /// room) and their bunk still opens their card; every occupied bed takes the class blanket (BUNKS-LAYOUT3, the take
    /// 8 mock: the crypt member's too); named founders get their gear on the peg board; the dressed north wall is two
    /// runs (one each side of the door).
    #[test]
    fn hub_bunks2_bunk_states_with_a_benched_member_asleep() {
        let (mut cx, ids) = hub_cx(false);
        let mut g = gfx();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())).clone();
        let all = kit::bunks(cx.gs(), &t);
        let niche: Vec<&Bunk> = all.iter().filter(|b| bed_art(&b.art).niche).collect();
        let free: Vec<&Bunk> = all.iter().filter(|b| !bed_art(&b.art).niche).collect();
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
            assert_eq!(dressed["blanket"], json!(m.class_id(&cx.data)), "the class blanket on an occupied bed: {e}");
            assert_eq!((dressed["gear"].clone(), dressed["board"].clone()), (json!(true), json!(true)), "a founder's gear on the peg board: {e}");
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
        assert_eq!(map.state(&cx)["bunk_walls"].as_array().map(|a| a.len()), Some(2), "a dressed wall run each side of the door");
        shot(&g, "hub_bunks2");
    }

    /// A 1280 x 720 window (the take 8 mock's size: an art px is a screen px).
    fn gfx_720() -> Gfx {
        let mut g = gfx();
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        g
    }

    /// The six founders in the take 8 mock's order, the first four active and the last two benched; the party in the
    /// bunk room.
    fn hub_cx_founders() -> (Cx, Vec<String>) {
        let (mut cx, _) = hub_cx(false);
        let d = cx.data.clone();
        let mut ids: Vec<String> = d.catalogs.own_entries(WORLD, "characters").iter().filter(|c| json::s(c, "role") == "party").map(|c| json::s(c, "id")).collect();
        let order = ["kaelen", "vesper", "maren", "hollis", "dunstan", "elyse"];
        ids.sort_by_key(|id| order.iter().position(|o| o == id).unwrap_or(order.len()));
        let gs = cx.gs_mut();
        gs.party.members = ids.iter().map(|c| new_game::hero_from_character(&d, WORLD, c).expect("a party character")).collect();
        gs.party.active_ids = ids.iter().take(4).cloned().collect();
        (cx, ids)
    }

    /// BUNKS-LAYOUT3 (1), (3), (4), (7) (the take 8 mock): on the shipped hub at 1280 x 720 the six niche beds stand
    /// 80 px apart, the first 40 px in from its wall run's west end and the last 56 px in from its east end (its peg
    /// board fits), each bed's top 69 px over the wall foot (its foot 11 px forward of it) with a peg board on every niche bunk;
    /// the benched founders sleep in the front `idle_down` pose, the head shown in the bed's top rows above the cover;
    /// the free-standing pair lies sideways on the south wall, head on a side wall, the east one mirrored.
    #[test]
    fn bunks_layout3_take8_positions_and_the_sleep_pose() {
        let (mut cx, ids) = hub_cx_founders();
        let mut g = gfx_720();
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let st = map.state(&cx);
        let px = |v: &Value| (v.as_i64().unwrap() * 2) as i32;
        let walls: Vec<(i32, i32, i32)> = st["bunk_walls"].as_array().expect("the wall runs").iter().map(|w| (px(&w[0]), px(&w[0]) + px(&w[2]), px(&w[1]) + px(&w[3]))).collect();
        assert_eq!(walls.len(), 2, "{walls:?}");
        let bunks = st["bunks"].as_array().cloned().unwrap();
        let niche: Vec<&Value> = bunks.iter().filter(|b| b["niche"] == json!(true)).collect();
        assert_eq!(niche.len(), 6);
        let mid = |b: &Value| px(&b["drawn"][0]) + px(&b["drawn"][2]) / 2;
        for (run, beds) in walls.iter().zip(niche.chunks(3)) {
            let xs: Vec<i32> = beds.iter().map(|b| mid(b)).collect();
            assert_eq!(xs, vec![run.0 + 40, run.0 + 120, run.0 + 200], "80 px apart from 40 px in: {run:?}");
            assert_eq!(run.1 - xs[2], 56, "the last bed's peg board fits: {run:?} {xs:?}");
            for b in beds {
                assert!((run.2 - px(&b["drawn"][1]) - WALL_FOOT).abs() <= 1, "the bed's top over the wall foot: {b}");
            }
        }
        for b in &niche {
            assert_eq!(b["dressed"]["board"], json!(true), "a peg board on every niche bunk: {b}");
        }
        for (i, id) in ids.iter().enumerate() {
            let e = bunks.iter().find(|b| b["member"] == json!(id)).unwrap_or_else(|| panic!("{id}'s bunk"));
            let asleep = i >= 4;
            assert_eq!(e["dressed"]["asleep"], json!(asleep), "{e}");
            if asleep {
                assert_eq!(e["dressed"]["pose"], json!(SLEEP_CLIP), "the front idle pose: {e}");
                let head = &e["dressed"]["head"];
                let (top, hy, hh) = (px(&e["drawn"][1]), px(&head[1]), px(&head[3]));
                let bed = bed_art("niche_bunk");
                assert!(hy >= top && hy + hh <= top + bed.head.1 + bed.head.3 + 1, "the head low on the pillow, in the bed's top rows: {e}");
                assert!(hy + hh > top + bed.cover_y.unwrap(), "the cover over the shoulders: {e}");
            }
        }
        // the south pair: `cot` lying sideways, the west one's head on the west wall, the east one mirrored
        let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())).clone();
        let free: Vec<Bunk> = kit::bunks(cx.gs(), &t).into_iter().filter(|b| !bed_art(&b.art).niche).collect();
        assert_eq!(free.len(), 2);
        let flips: Vec<bool> = free.iter().map(|b| json::aget(&t, "props").iter().find(|p| p["id"] == json!(b.prop)).and_then(|p| p["flip"].as_bool()).unwrap_or(false)).collect();
        assert_eq!(flips, vec![false, true], "the east one mirrored");
        let open = |x: i64, y: i64| explore::is_open(&t, x, y);
        let (w, e) = (&free[0], &free[1]);
        assert!(w.foot.2 > w.foot.3 && e.foot.2 > e.foot.3, "lying sideways: {free:?}");
        assert!((0..w.foot.3).all(|dy| !open(w.foot.0 - 1, w.foot.1 + dy)), "head on the west wall: {w:?}");
        assert!((0..e.foot.3).all(|dy| !open(e.foot.0 + e.foot.2, e.foot.1 + dy)), "head on the east wall: {e:?}");
        shot(&g, "hub_layout3_1280");
    }

    /// BUNKS-LAYOUT3 (5), (6): the stove stands in the middle of the hall, between the two tables, and no barrels stand
    /// in the hall (the west barrels went).
    #[test]
    fn bunks_layout3_the_stove_between_the_tables_and_no_hall_barrels() {
        let (cx, _) = hub_cx(false);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())).clone();
        let props: Vec<Dict> = json::aget(&t, "props").iter().filter_map(|p| p.as_object().cloned()).collect();
        let mid = |p: &Dict| 2 * json::i(p, "x", 0) + json::i(p, "w", 1);
        let tables: Vec<&Dict> = props.iter().filter(|p| json::s(p, "id").starts_with(kit::TABLE_PROP_PREFIX)).collect();
        assert_eq!(tables.len(), 2);
        let stove = props.iter().find(|p| json::s(p, "art") == "hearth_stove").expect("the stove");
        let (a, b) = (mid(tables[0]).min(mid(tables[1])), mid(tables[0]).max(mid(tables[1])));
        assert!((mid(stove) - (a + b) / 2).abs() <= 1, "midway between the tables (half cells): {a} {b} {}", mid(stove));
        assert_eq!(json::i(stove, "y", 0), json::i(tables[0], "y", 0), "on the tables' row");
        // the hall: the cells the bunks' footprints span west to east, south of the north wall row
        let bunks = kit::bunk_props(&t);
        let (x0, x1) = (bunks.iter().map(|p| json::i(p, "x", 0)).min().unwrap(), bunks.iter().map(|p| json::i(p, "x", 0) + json::i(p, "w", 1)).max().unwrap());
        let y0 = bunks.iter().map(|p| json::i(p, "y", 0) + json::i(p, "h", 1)).min().unwrap();
        let barrels: Vec<String> = props.iter().filter(|p| json::s(p, "art").contains("barrel") && (x0..x1).contains(&json::i(p, "x", 0)) && json::i(p, "y", 0) >= y0).map(|p| json::s(p, "id")).collect();
        assert!(barrels.is_empty(), "no barrels in the hall: {barrels:?}");
    }

    /// BUNKS-UPGRADE-PROPS1 (`PR7-50`): on the shipped hub, before the bunk cap reaches the Act 2 bunks the props
    /// holding their places (`until_bunk`) draw and those bunks do not (no bed, no click area); once the cap reaches
    /// them (the upgrade) the bunks draw and the props go. The cap is raised in the test's game only.
    #[test]
    fn bunks_upgrade_props1_props_hold_the_act2_bunks_until_the_upgrade() {
        let (mut cx, _) = hub_cx(false);
        let mut g = gfx_720();
        let mut map = RegionMap::new(&mut cx);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())).clone();
        let holds: Vec<(String, String)> = json::aget(&t, "props").iter().map(|p| (json::s(json::dict(p), "id"), json::s(json::dict(p), kit::UNTIL_BUNK))).filter(|(_, b)| !b.is_empty()).collect();
        let mut held_bunks: Vec<String> = holds.iter().map(|(_, b)| b.clone()).collect();
        held_bunks.dedup();
        assert!(!held_bunks.is_empty(), "the hub holds bunk places: {holds:?}");
        let seen = |map: &RegionMap, cx: &Cx| {
            let st = map.state(cx);
            let bunks: Vec<String> = st["bunks"].as_array().cloned().unwrap_or_default().iter().map(|b| b["prop"].as_str().unwrap().to_string()).collect();
            let held: Vec<String> = st["bunk_held"].as_array().cloned().unwrap_or_default().iter().map(|b| b["prop"].as_str().unwrap().to_string()).collect();
            let clickable: Vec<String> = map.widgets().iter().filter_map(|w| w["id"].as_str().and_then(|i| i.strip_prefix(BUNK_PREFIX)).map(str::to_string)).collect();
            (bunks, held, clickable)
        };
        frame(&mut map, &mut cx, &mut g, 1.0);
        let open = kit::open_bunk_ids(cx.gs(), &t);
        assert!(held_bunks.iter().all(|b| !open.contains(b)), "before the upgrade the held bunks are past the cap: {open:?}");
        let (bunks, held, clickable) = seen(&map, &cx);
        assert!(held_bunks.iter().all(|b| !bunks.contains(b) && !clickable.contains(b)), "before: no held bunk drawn: {bunks:?}");
        assert_eq!(held, holds.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(), "before: every holding prop drawn");
        shot(&g, "hub_upgrade_before_1280");
        // the upgrade: the cap reaches every bunk prop
        let all = kit::bunk_props(&t).len() as i32;
        let m = &mut cx.gs_mut().party.roster.manifest;
        m.bunks = all;
        m.bunks_max = all;
        frame(&mut map, &mut cx, &mut g, 1.0);
        let (bunks, held, clickable) = seen(&map, &cx);
        assert!(held_bunks.iter().all(|b| bunks.contains(b) && clickable.contains(b)), "after: the held bunks drawn: {bunks:?}");
        assert!(held.is_empty(), "after: the props gone: {held:?}");
        shot(&g, "hub_upgrade_after_1280");
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
