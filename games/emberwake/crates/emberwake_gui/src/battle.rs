//! The battle screen (COMBAT-UI3 layout A). Two layers: the battlefield is pixel art scaled with nearest sampling;
//! panels, text, hover cards and menus are drawn smooth at the window's real resolution (painter.rs). Every change goes
//! through the command API (emberwake_host on aegis_host), the same one `aegis serve` and the agent link use.
//! Two modes: the Test Arena (a catalog encounter, Restart / Play again) and a world battle (GameEncounter: the saved
//! party at its stored HP against a world encounter; on Continue the runner's after_fight settles the battle's tally
//! (the kit settles the party: HP, the stay-behind and roster rules; then the rewards) and the results popup opens
//! over the map).
//! Mouse: click a hero to select; click a foe, an ally, the hero itself or a tile for its options; hover anything for
//! details; right click closes a menu or undoes the hero's last step; the wheel scrolls the log. Keys: Enter confirm,
//! Space step / next round, A autoplay, Z undo, L log, R restart (arena), Esc close; hold Alt to outline every unit,
//! covered or not (STAGE1-O). Timeline battles (INPUT1-U): a click on a target sets the round target and opens the skill
//! menu (picks through the host's `pick`, the planner's ghost fills the rest), keys 1-9 are the hero's skill hotkeys, S
//! targets the hero itself, Z undoes the last pick. INPUT1-T: T opens the full timeline view over the grid panel
//! (`timeline_view.rs`, every unit's track; T or Esc closes it). INPUT1-PB: during a timeline battle's playback each
//! impact draws a short tracer from its actor to what it hit, the strip frames the resolving block, P cycles the
//! playback speed (1x, 0.5x, block by block; N plays the next block), the log groups its lines by round and block, and
//! an open timeline view pauses the playback at the resolving block's end.
//! ENCOUNTER2-C2 (docs/specs/ENCOUNTER2_C.md 3.3): each new talk cue of `Session::talks` opens the shared talk box
//! (`talk.rs`) once its step's impacts have played, over a 35 % veil, the speaker's token in the gold ring; while it is
//! open the screen sends no request, autoplay waits and every board click and battle key is swallowed; replies run in
//! the game in play (a world fight) or a scratch game state of the Arena's world.

use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::music;
use crate::intent_text;
use crate::fx::{self, Impact, Shown};
use crate::portraits::{Faces, Who};
use crate::talk::{self, TalkAct};
use crate::timeline_view;
use aegis_ui::fx::{Camera, Clock, Fx, Layer};
use crate::stage::{self, Geom, Item, Pass, Through, View};
use crate::tuning::Tuning;
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use emberwake_battle::intent::Intent;
use aegis_ui::audio::Bus;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Tip, Ui, Widget, BODY, H, RADIUS, SMALL, TITLE, W};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::plan::expected;
use emberwake_battle::round_intent::{self as ri, ManeuverKind, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Prebuilt, Session, StartGrant};
use aegis_core::assets::{ClipFrame, ClipPlayer, Facing};
use aegis_ui::art::Art;
use aegis_ui::canvas::Image;
use aegis_core::json::{self, s, Dict};
use emberwake_rules::Stance;
use emberwake_world::battle::WorldRoster;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use emberwake_host::{Battle, Host};
use emberwake_world::campaign::{purse, trigger};
use emberwake_world::{dialogue, explore, story, travel};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// The Flat view's cell (and the grid panel's: 12 x 9 cells of `TILE` at (`GRID_X`, `GRID_Y`)).
const TILE: i32 = 32;
/// PORTRAITFX1-P (6.2 row 3): the hero and intel panels' portrait side, its inset from the panel's top-left, where the
/// text lines beside it start (from the panel's left edge) and the cue rows' surface.
const PANEL_PORTRAIT: i32 = 32;
const PANEL_INSET: i32 = 4;
const PANEL_TEXT_X: i32 = 40;
const PANEL_SURFACE: &str = "panel";
/// PORTRAITFX1-TL (6.2 row 4): the full timeline view's track icons draw on the cue rows' `timeline` surface.
const TIMELINE_SURFACE: &str = "timeline";
/// PORTRAITFX1-TL (user 2026-11-01): the events an enemy's timeline row reacts to: the blows (when the board shows them).
const TIMELINE_ENEMY_EVENTS: [&str; 3] = ["hit", "crit", "guard_hit"];
/// INPUT1 MR12: a dash (a skill with a position moving to it at its start) crosses its route in this many ms at 1x
/// (the FX clock: the animation speed applies), playing the rig's `dash` clip, else its walk (until DASH1's clip).
const DASH_MS: u64 = 250;
/// INPUT1-AP (AP6): the AP gauge's most pips (the user's first try); a `max_ap` above it puts `ceil(max_ap / AP_PIPS)`
/// AP in a pip.
const AP_PIPS: i32 = 12;
/// AP10: paid pips turn hollow, and regained ones fill, over this many ms (at once with skip animations or reduce
/// motion).
const AP_ANIM_MS: f32 = 150.0;
/// INPUT1 PB1: an impact tracer's life on the FX clock (ms; the animation speed and PB3's 0.5x apply), its alpha at the
/// impact and how far it stops short of each token centre (px).
const TRACER_MS: u64 = 400;
const TRACER_ALPHA: u8 = 200;
const TRACER_INSET: f32 = 3.0;
/// INPUT1 PB3: the settings key of the playback speed and its values (setting value, chip label, chip widget id).
pub(crate) const PLAYBACK_SPEED: &str = "playback_speed";
pub(crate) const PLAYBACK_SPEEDS: [(&str, &str, &str); 3] = [("1x", "1x", "playback.speed.1"), ("0.5x", "0.5x", "playback.speed.half"), ("block", "Block", "playback.speed.block")];
/// The battle screen's panels by name (`BattleScreen::panels`; the UI Gallery's battle panel entries).
pub const PANELS: [&str; 7] = ["strip", "party", "hero", "grid", "hostiles", "intel", "bar"];
const GRID_X: i32 = 128;
/// INPUT1 TV6: the timeline view's padding inside the grid panel's rect.
const TV_PAD: i32 = 4;
const GRID_Y: i32 = 30;
const TACTICS: [&str; 6] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy"];
const TACTIC_LABELS: [&str; 6] = ["Balanced", "All-out Attack", "Focus Weakest", "Guard & Counter", "Kite at Range", "Economy"];
const TACTIC_HELP: [&str; 6] = [
    "The unit's own role: fights, guards when hurt, flees when nearly down.",
    "Nearest enemy, strongest affordable skills, no guarding or fleeing.",
    "Goes for the enemy that is closest to falling.",
    "Guards first, then trades blows.",
    "Holds maximum range and backs away from melee.",
    "Uses level 1 skills to stretch AP over more actions.",
];
/// RULESET1_INTENT AI5: what each tactic picks under the round intent (the timeline ruleset).
const TACTIC_INTENT_HELP: [&str; 6] = [
    "The AI table: heals a hurt friend, guards when hurt, flanks an engaged foe, else Assault.",
    "Assault on the first ranked enemy.",
    "Assault on the lowest-HP enemy it can reach this round.",
    "Guard on yourself.",
    "Ranged: Assault and kite. Melee: Disengage when offered, else Assault.",
    "Assault with the cheapest opener, then Rest.",
];

const BG: Rgba = rgb(10, 14, 24);
const PANEL: Rgba = rgba(22, 32, 54, 245);
const PANEL_HI: Rgba = rgba(34, 50, 82, 250);
const CARD: Rgba = rgba(16, 24, 42, 250);
const BORDER: Rgba = rgb(58, 78, 116);
const TEXT: Rgba = rgb(222, 228, 240);
const DIM: Rgba = rgb(132, 146, 172);
const GOLD: Rgba = rgb(244, 198, 84);
const ALLY: Rgba = rgb(110, 170, 255);
const FOE: Rgba = rgb(240, 100, 92);
const GOOD: Rgba = rgb(96, 210, 110);
/// RULESET1-U slice 5: the poise bar's colour (a pale steel blue, apart from HP green and red).
const POISE: Rgba = rgb(150, 190, 235);
/// RULESET1-U slice 4: the enemy facing chevron (pale, so it reads apart from the red tell arrows).
const FACING: Rgba = rgb(250, 226, 160);
/// WINDUP1 W9: Tallow Amber (art bible 5), the wind-up outline.
const AMBER: Rgba = rgb(0xd9, 0x77, 0x06);
/// ENCOUNTER2-C DT7: the veil over the board and panels while a battle talk is open (35 % black).
const TALK_VEIL: Rgba = rgba(0, 0, 0, 89);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(usize),
    UnitClick(usize),
    TileClick(V2),
    Choose(usize),
    ClosePopup,
    Confirm,
    Clear,
    Undo,
    Autoplay,
    PartyMenu,
    HeroMenu,
    NextRound,
    Step,
    Restart,
    ToggleLog,
    /// World battle over: settle it and show the results.
    Continue,
    /// Leave the Test Arena.
    Leave,
    /// INPUT1 SL4 (round intent): the hero panel's Self button and key S: the selected hero targets itself (the self
    /// menu).
    SelfTarget,
    /// ENCOUNTER2 H1: the hero picked to hold a hold escape's tiles.
    StayPick(usize),
    /// ENCOUNTER2-C DT9: a reply or Continue in the open battle talk's box.
    Talk(TalkAct),
    /// INPUT1 TV1: key T opens and closes the full timeline view.
    TimelineView,
    /// TV5: a click in the open timeline view: a party track (or one of its spans) selects that hero; anything else in
    /// the view (a block, an enemy track, its backdrop) does nothing, so the board under it takes no click.
    TimelinePick(Option<usize>),
    /// INPUT1 PB3: a playback speed chip (an index of `PLAYBACK_SPEEDS`; key P cycles them).
    PlaybackSpeed(usize),
    /// PB3, PB5: Next block (key N): a playback held at a block's end plays the next block.
    NextBlock,
    /// ENCOUNTER2-C4 (decision "ENCOUNTER2-C answers" Q-C5): a locked round's Skip (`battle.skip`): resolves the rest of
    /// the round at once (host `resolve`: the same result, no animation).
    Skip,
}

/// INPUT1 PB3: the playback speed setting (`playback_speed`: "1x", "0.5x", "block"; anything else reads 1x).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaySpeed {
    Full,
    Half,
    Block,
}

impl PlaySpeed {
    fn of(settings: &Value) -> PlaySpeed {
        match settings[PLAYBACK_SPEED].as_str() {
            Some("0.5x") => PlaySpeed::Half,
            Some("block") => PlaySpeed::Block,
            _ => PlaySpeed::Full,
        }
    }

    /// Its index in `PLAYBACK_SPEEDS`.
    fn index(self) -> usize {
        match self {
            PlaySpeed::Full => 0,
            PlaySpeed::Half => 1,
            PlaySpeed::Block => 2,
        }
    }
}

/// ENCOUNTER2-C DT6: an open battle talk: the tree it plays and its runner (a scene: no services).
struct BattleTalk {
    dialogue: String,
    runner: dialogue::DialogueRunner,
}

#[derive(Debug, Clone)]
enum RowAct {
    Step(Value),
    PartyTactic(usize),
    HeroTactic(usize),
    /// INPUT1 IF2-IF4 (round intent): a row of the skill menu: the picks it appends on the menu's target.
    Pick(Vec<rip::Pick>),
    /// SL6: the ally menu's last row, "Select <ally>".
    SelectAlly(usize),
}

/// INPUT1 GH2, IN6 timing (decision "INPUT1 answers, part 4" Q18): a member's ghost that waits for Confirm because its
/// intent change would drop intent-bound buffs: its tactic's draft (`preset`) or a player-set target's round change
/// (`round_intent`). The screen draws it from copies (`view`) and Confirm sends the same change (`req`).
#[derive(Debug, Clone)]
enum Deferred {
    Preset(&'static str),
    Round(rip::RoundSet),
}

impl Deferred {
    /// The host request Confirm sends for member `m`.
    fn req(&self, m: usize) -> Value {
        match self {
            Deferred::Preset(p) => json!({"cmd": "preset", "unit": m, "preset": p}),
            Deferred::Round(set) => {
                debug_assert!(set.kind.is_none() && set.opener.is_none(), "the screen defers only an intent and a target");
                let mut r = json!({"cmd": "round_intent", "unit": m});
                if let Some(i) = set.intent {
                    r["intent"] = json!(i.id());
                }
                if let Some(t) = set.target {
                    r["target"] = target_json(t);
                }
                r
            }
        }
    }

    /// Member `m`'s round as the change would leave it, on copies (the tactic's draft as the host's `preset` makes it;
    /// a round change through the kit's `preview_round`, the source of the host's `round_intent`).
    fn view(&self, s: &Session, m: usize) -> Option<(World, emberwake_battle::plan::Plan)> {
        match self {
            Deferred::Preset(p) => {
                let mut w2 = s.w.clone();
                let mut p2 = rip::copy_plan(&s.plan);
                emberwake_battle::ai::plan_party_intents(&mut w2, &mut p2, &[(m, p.to_string())]);
                Some((w2, p2))
            }
            Deferred::Round(set) => rip::preview_round(&s.w, &s.plan, m, set).ok(),
        }
    }
}

/// RULESET1_INTENT UI5: a round intent's colour, its commands' colour (Assault an attack's, Guard a Defend's, Support's,
/// Maneuver a Disengage's, Channel a Focus's).
fn intent_color(i: RoundIntent) -> Rgba {
    kind_color(match i {
        RoundIntent::Assault => "ATTACK",
        RoundIntent::Guard => "DEFEND",
        RoundIntent::Support => "SUPPORT",
        RoundIntent::Maneuver => "DISENGAGE",
        RoundIntent::Channel => "FOCUS",
    })
}

/// UI5: an intent's colour for text on the dark cards (its colour lifted toward white, so it reads at SMALL).
fn intent_ink(i: RoundIntent) -> Rgba {
    let c = intent_color(i);
    let lift = |v: u8| (v as u16 + (255 - v as u16) * 45 / 100) as u8;
    rgb(lift(c[0]), lift(c[1]), lift(c[2]))
}

/// UI5: the intent glyph the strip, the cards and the board share: a plate in the intent's colour with its initial
/// (TERMS1 TU5: of the intent's name through `cat`'s terms).
fn intent_glyph(p: &mut Painter, cat: &emberwake_data::Catalog, i: RoundIntent, r: Rect) {
    p.panel(r, intent_color(i), rgba(255, 255, 255, 170), 2.0);
    let letter: String = i.label(cat).chars().take(1).collect();
    p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, &letter, SMALL, TEXT);
}

/// A tell line's kind as the command colour key (`kind_color`).
fn tell_color(kind: &str) -> Rgba {
    kind_color(match kind {
        "attack" | "area" => "ATTACK",
        "support" => "SUPPORT",
        "guard" => "DEFEND",
        "focus" => "FOCUS",
        "rest" => "REST",
        "move" | "flee" | "retreat" => "MOVE",
        "switch" => "TARGET",
        _ => "",
    })
}

/// RULESET1 V1, V2 on a hostile card: a row of the round's blocks with an impact badge at each impact block (a hit red,
/// anything else gold); a revealed plan (V2) also fills each action from its start to its impact in its kind's colour,
/// its recovery blocks dim. Returns the row's tip (a title, then a line per badge).
fn draw_badge_row(p: &mut Painter, t: &Intent, row: Rect, blocks: i32) -> Tip {
    use emberwake_battle::intent::HIT;
    let cw = row.w as f32 / blocks as f32;
    let cell = |b: i32| {
        let x0 = row.x + ((b - 1) as f32 * cw).round() as i32;
        Rect::new(x0, row.y, (row.x + (b as f32 * cw).round() as i32 - x0 - 1).max(1), row.h)
    };
    for b in 1..=blocks {
        p.fill(cell(b), rgba(14, 20, 34, 255));
    }
    if t.revealed {
        for l in t.plan.iter().filter(|l| l.blocks.0 >= 1) {
            for b in l.blocks.0..=l.blocks.1.min(blocks) {
                p.fill(cell(b), with_alpha(tell_color(l.kind), 210));
            }
            for b in (l.blocks.1 + 1)..=(l.blocks.1 + l.recovery).min(blocks) {
                p.fill(cell(b), rgba(120, 120, 140, 110));
            }
        }
    }
    let mut tip: Tip = vec![("Impacts this round".into(), GOLD)];
    for &(b, k) in t.impacts.iter().filter(|(b, _)| (1..=blocks).contains(b)) {
        let c = cell(b);
        let col = if k == HIT { FOE } else { GOLD };
        p.fill(Rect::new(c.x + c.w / 2 - 2, c.y, 4, c.h), col);
        p.fill(Rect::new(c.x + c.w / 2 - 1, c.y - 1, 2, c.h + 2), col);
        tip.push((format!("B{b}: {}", if k == HIT { "a hit lands" } else { "an action lands" }), col));
    }
    if tip.len() == 1 {
        tip.push(("Nothing lands this round".into(), DIM));
    }
    tip
}

fn kind_name(k: ManeuverKind) -> &'static str {
    match k {
        ManeuverKind::Engage => "Engage",
        ManeuverKind::Disengage => "Disengage",
        ManeuverKind::Utility => "Utility",
    }
}

/// A pick as the host's `pick` request takes it (INPUT1_TASKS 1.3).
fn pick_json(w: &World, p: &rip::Pick) -> Value {
    match p {
        rip::Pick::Skill { skill, kind } => {
            let mut v = json!({"skill": w.cat.skills[*skill].id});
            if let Some(k) = kind {
                v["kind"] = json!(emberwake_battle::intent::maneuver_id(*k));
            }
            v
        }
        rip::Pick::Item(id) => json!({"item": id}),
        rip::Pick::Rest => json!("rest"),
        rip::Pick::MoveHere(c) => json!({"move": [c.x, c.y]}),
        rip::Pick::StepAfter(c) => json!({"step_after": [c.x, c.y]}),
    }
}

/// INPUT1 GH2 b: the intents a round target of this kind tries, in order (an enemy: Assault, else Maneuver; an ally:
/// Support; the unit itself: Guard, else Support; a tile: Guard, else Assault, else Maneuver).
fn intents_for(w: &World, h: usize, at: RoundTarget) -> &'static [RoundIntent] {
    match at {
        RoundTarget::Unit(t) if w.foes_of(h).contains(&t) => &[RoundIntent::Assault, RoundIntent::Maneuver],
        RoundTarget::Unit(_) => &[RoundIntent::Support],
        RoundTarget::Myself => &[RoundIntent::Guard, RoundIntent::Support],
        RoundTarget::Cell(_) => &[RoundIntent::Guard, RoundIntent::Assault, RoundIntent::Maneuver],
    }
}

/// Whether `intent` on `at` plans a round for `h` (GH2 b: "when it can act on that target"), on copies.
fn plans_on(s: &Session, h: usize, intent: RoundIntent, at: RoundTarget) -> bool {
    if !s.w.ruleset.numbers.intents.iter().any(|i| i == intent.id()) || !ri::target_problem(&s.w, h, intent, at).is_empty() {
        return false;
    }
    let mut w2 = s.w.clone();
    let mut p2 = rip::copy_plan(&s.plan);
    ri::set_intent(&mut w2, h, intent).is_ok() && ri::set_target(&mut w2, h, at).is_ok() && rip::plan_unit(&mut w2, &mut p2, h).is_ok()
}

/// A menu row's label (IF3): the skill with its hotkey digit (HK1's first nine), "Use <item> (<n> left)", Rest, the
/// tile rows (ET1).
fn pick_label(s: &Session, h: usize, picks: &[rip::Pick]) -> String {
    let w = &s.w;
    let skill = |sid: usize| {
        let name = w.cat.skills[sid].name.clone();
        match rip::hotkey_skills(w, h).iter().position(|k| *k == sid).filter(|k| *k < 9) {
            Some(k) => format!("{} {name}", k + 1),
            None => name,
        }
    };
    match picks {
        [rip::Pick::Skill { skill: sid, .. }] => skill(*sid),
        [rip::Pick::Item(id)] => {
            let name = emberwake_battle::command::pouch_item(w, h, id).map_or(id.clone(), |i| i.name.clone());
            format!("Use {name} ({} left)", s.plan.items_left(w, h, id, -1))
        }
        [rip::Pick::Rest] => "Rest".into(),
        [rip::Pick::MoveHere(_)] => "Move here".into(),
        [rip::Pick::MoveHere(_), rip::Pick::Skill { skill: sid, .. }] => format!("Move here, then {}", w.cat.skills[*sid].name),
        [rip::Pick::StepAfter(_)] => "Step here after".into(),
        _ => picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", then "),
    }
}

/// A menu row's detail (IF3): "<blocks> blocks, <ap> AP; <Intent>" (a Maneuver row adds its kind), or its refusal.
fn pick_detail(cat: &emberwake_data::Catalog, o: &rip::PickOption) -> String {
    if let Some(why) = &o.problem {
        return why.clone();
    }
    let blocks = format!("{} block{}, {} AP", o.blocks, if o.blocks == 1 { "" } else { "s" }, o.ap);
    match (o.intent, o.kind) {
        (Some(i @ RoundIntent::Maneuver), Some(k)) => format!("{blocks}; {} ({})", i.label(cat), kind_name(k)),
        (Some(i), _) => format!("{blocks}; {}", i.label(cat)),
        (None, _) => blocks,
    }
}

/// A round target as the `round_intent` request takes it ("self", a unit index or a cell).
fn target_json(t: RoundTarget) -> Value {
    match t {
        RoundTarget::Myself => json!("self"),
        RoundTarget::Unit(u) => json!(u),
        RoundTarget::Cell(p) => json!([p.x, p.y]),
    }
}

/// A round target's name ("self", the unit's, "tile 3, 4").
fn target_name(w: &World, t: RoundTarget) -> String {
    match t {
        RoundTarget::Myself => "self".into(),
        RoundTarget::Unit(u) => w.units[u].name.clone(),
        RoundTarget::Cell(p) => format!("tile {}, {}", p.x, p.y),
    }
}

/// UI2's Rest gain ("Rest: 1 AP, 10 poise"; the poise only under the rule poise).
fn rest_text(g: rip::RestGain) -> String {
    let poise = if g.poise > 0 { format!(", {} poise", g.poise) } else { String::new() };
    format!("Rest: {} AP{poise}", g.ap)
}

#[derive(Debug, Clone)]
struct Row {
    label: String,
    detail: String,
    valid: bool,
    act: RowAct,
    tip: Tip,
    /// IF3, IN6: picking the row changes the intent and drops intent-bound buffs (a red corner).
    warn: bool,
    /// INPUT1 AP3, AP8: a skill-menu row refused for its AP: how many AP short it is (the kit's `PickOption::ap_short`).
    ap_short: Option<i32>,
}

#[derive(Debug, Clone)]
struct Popup {
    title: String,
    x: i32,
    y: i32,
    w: i32,
    rows: Vec<Row>,
    /// INPUT1 IF1-IF5: the skill menu's hero and target (None: a tactic or step menu).
    menu: Option<(usize, RoundTarget)>,
}

impl Popup {
    /// The menu's rect with rows `row_h` high (TEXTFIT1: the pitch grows with the text size; 11 at Normal).
    fn rect(&self, row_h: i32) -> Rect {
        Rect::new(self.x, self.y, self.w, row_h + 4 + self.rows.len() as i32 * row_h)
    }

    /// Row `k`'s rect (under the title).
    fn row(&self, row_h: i32, k: usize) -> Rect {
        Rect::new(self.x + 2, self.y + row_h + 2 + k as i32 * row_h, self.w - 4, row_h)
    }
}

struct Float {
    unit: usize,
    text: String,
    color: Rgba,
    born: Instant,
    /// How long it shows (wall ms: the tuning's `float_ms`, doubled at PB3's 0.5x).
    ms: u64,
}

/// What a unit was when its token last caught up with the battle (SPRITE1-U).
#[derive(Debug, Clone, Default, PartialEq)]
struct Seen {
    pos: Option<V2>,
    hp: i32,
    alive: bool,
    guarded: bool,
    /// WINDUP1: its poise broken this round; the Staggered status (active or pending).
    broken: bool,
    staggered: bool,
}

/// One unit's token animation (SPRITE1-U): its clips, a slide to a new tile, its fall. Times are on the battle's
/// FX clock (`BattleScreen::clock`, scaled by the animation speed setting; FX1-L1).
#[derive(Debug, Clone, Default)]
struct UnitAnim {
    player: ClipPlayer,
    /// An action clip to start (candidates best first, `action_clips`) and when; the draw picks the first the body has.
    want: Option<(Vec<&'static str>, u64)>,
    /// The tile it slides from and when the slide started.
    slide: Option<(V2, u64)>,
    /// INPUT1 MR12: the slide is a dash (`DASH_MS` long, not the walk's), and its `dash` clip is still to be picked
    /// once the body is known (`start_wanted`).
    dash: bool,
    dash_clip: bool,
    /// When it fell (the death clip plays and holds, then the dead marker shows); None while it stands.
    fell: Option<u64>,
    seen: Seen,
    /// DIR4-G (FACE1): the facing it keeps (`Facing`: the view and the last left/right; the rig's sheets face right,
    /// a side row is mirrored when `left`).
    facing: Facing,
    /// DIR4-G G2: the facing of its last step, shown while it slides (`slide`).
    step: Facing,
    /// FACE1: while the party plans, the facing toward its planned or intended target (None: no target).
    planned: Option<Facing>,
    /// WINDUP1 W4: the wanted clip starts at its `strike` frame (it struck from the wind-up shown).
    strike: bool,
    /// WINDUP1 W2: the action whose wind-up it shows (section 6).
    windup: Option<Windup>,
    /// WINDUP1 5: while the party plans, the pose of its round (`plan_pose`; `Pose::None` otherwise).
    stance: Pose,
    /// WINDUP1 6.2: the pose its base shows and the clip last drawn (the agent link's `tokens`).
    pose: Pose,
    drawn: String,
    /// STATUSFX1 S1, S5: the statuses with a look the board shows (rule I: the state before a step until its impact),
    /// each with whether it is only pending and the clock ms its look starts (its burst's).
    marks: Vec<fx::MarkOn>,
    /// STATUSFX1 S2: each slot's cycling (head, body, feet, tint) and the ids drawn last frame (the agent link's
    /// `status_fx`, S9).
    slots: [fx::SlotShow; 4],
    status_fx: Vec<&'static str>,
}

impl UnitAnim {
    /// DIR4-G G1/G3: the facing while it stands: its planned target's while planning, else the facing it keeps.
    fn rest(&self) -> Facing {
        self.planned.unwrap_or(self.facing)
    }

    /// DIR4-G: the facing drawn at `now` (the animation clock): its step's while the slide runs (`slide_ms`), else
    /// `rest`.
    fn faces(&self, now: u64, walk_ms: u64) -> Facing {
        match self.slide {
            Some((_, t0)) if walk_ms > 0 && now.saturating_sub(t0) < self.slide_ms(walk_ms) => self.step,
            _ => self.rest(),
        }
    }

    /// How long its slide runs: a dash's `DASH_MS` (INPUT1 MR12), else the walk's `walk_ms`.
    fn slide_ms(&self, walk_ms: u64) -> u64 {
        if self.dash { DASH_MS } else { walk_ms }
    }
}

/// WINDUP1 section 3 (rule W1): the pose a command shows, and the planning poses of section 5 (`Guard`, `Dazed`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Pose {
    #[default]
    None,
    Attack,
    Heavy,
    Buff,
    Debuff,
    Guard,
    Dazed,
}

impl Pose {
    /// The agent link's name (6.2; "" for none).
    fn id(self) -> &'static str {
        match self {
            Pose::None => "",
            Pose::Attack => "attack",
            Pose::Heavy => "heavy",
            Pose::Buff => "buff",
            Pose::Debuff => "debuff",
            Pose::Guard => "guard",
            Pose::Dazed => "dazed",
        }
    }

    /// Section 4: the clips showing it, best first (the first one the body has; none: the base as before WINDUP1).
    fn clips(self) -> &'static [&'static str] {
        match self {
            Pose::Heavy => &["windup_heavy", "windup"],
            Pose::Debuff => &["windup_debuff", "windup"],
            Pose::Buff => &["windup_buff"],
            Pose::Attack => &["windup"],
            Pose::Guard => &["idle_guarded"],
            Pose::Dazed => &["dazed"],
            Pose::None => &[],
        }
    }

    /// A wind-up or the dazed loop (6.1: skip animations starts it at `loop_from`, reduce motion holds it still).
    fn held(self) -> bool {
        !matches!(self, Pose::None | Pose::Guard)
    }
}

/// WINDUP1 4: the clips the screen looks up on a body (the four wind-ups, the reaction and the dazed loop).
const POSE_CLIPS: [&str; 6] = ["windup", "windup_heavy", "windup_buff", "windup_debuff", "stagger", "dazed"];

/// WINDUP1: the pose clips one unit's body has (`POSE_CLIPS`), each with its `loop_from`. Empty (no body, or a body
/// without them, as every body before WINDUP1-R1): the unit shows no pose, reaction or wind-up.
#[derive(Debug, Clone, Default, PartialEq)]
struct PoseSet(Vec<(&'static str, u32)>);

impl PoseSet {
    fn of(clips: &aegis_core::assets::Clips) -> PoseSet {
        PoseSet(POSE_CLIPS.iter().filter_map(|&c| clips.clip(c).map(|k| (c, k.loop_from))).collect())
    }

    fn has(&self, clip: &str) -> bool {
        self.0.iter().any(|(c, _)| *c == clip)
    }

    fn loop_from(&self, clip: &str) -> u32 {
        self.0.iter().find(|(c, _)| *c == clip).map_or(0, |(_, l)| *l)
    }

    /// The clip showing `p` (section 4's fallbacks): the guard is `idle_guarded` (A1, A3), on a body with pose rows
    /// (one without them stands as before WINDUP1); None for no pose or when the body has none of its clips.
    fn clip(&self, p: Pose) -> Option<&'static str> {
        match p {
            Pose::Guard if !self.0.is_empty() => Some("idle_guarded"),
            _ => p.clips().iter().copied().find(|c| self.has(c)),
        }
    }
}

/// WINDUP1 W2: the wind-up a unit shows: the scheduled action (this round's index), its pose and clip and its start and
/// impact blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Windup {
    round: i32,
    idx: usize,
    pose: Pose,
    clip: &'static str,
    blocks: (i32, i32),
}

/// WINDUP1 6.1: how a base is put: `instant` (skip animations) starts a pose at its `loop_from`, `still` (reduce
/// motion) holds a pose's hold and the dazed loop on one frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Motion {
    instant: bool,
    still: bool,
}

/// W1 rows 4-6: an attack or area skill's pose: debuff-only (`power_mult` 0), Heavy (3 blocks or more), else an
/// attack (also with no skill row).
fn attack_pose(def: Option<&emberwake_data::SkillDef>) -> Pose {
    match def {
        Some(d) if d.power_mult <= 0.0 => Pose::Debuff,
        Some(d) if d.blocks >= 3 => Pose::Heavy,
        _ => Pose::Attack,
    }
}

/// W1 row 8 (WQ3): an item aimed at foes (`aims_at_foes`) winds up as a debuff, any other as a buff.
fn item_pose(item: &emberwake_data::ItemDef) -> Pose {
    if item.aims_at_foes() { Pose::Debuff } else { Pose::Buff }
}

/// W1: the pose of a skill row by its kind (a plan step): attacks and areas by `attack_pose`, support and focus a buff,
/// anything else none (Defend, moves, Disengage, Rest, Target, statuses).
fn skill_pose(def: &emberwake_data::SkillDef) -> Pose {
    match emberwake_battle::skill::kind(def) {
        Kind::Attack => attack_pose(Some(def)),
        Kind::Support | Kind::Focus => Pose::Buff,
        _ => Pose::None,
    }
}

/// W1: the pose of a scheduled command (its skill row by name, so at its rank; its item); none for Defend, walks,
/// Disengage, Rest, Target and lost blocks.
fn command_pose(s: &Session, c: &emberwake_battle::command::Command) -> Pose {
    match &c.kind {
        CmdKind::Attack { .. } | CmdKind::Area { .. } => attack_pose(s.w.cat.skills.iter().find(|k| k.name == c.name)),
        CmdKind::Support { .. } | CmdKind::Focus { .. } => Pose::Buff,
        CmdKind::Item { item, .. } => item_pose(item),
        _ => Pose::None,
    }
}

/// WINDUP1 6: the block the round's wind-ups are measured at: 1 once the plan is confirmed (block-1 actions wind up at
/// the confirm), the current block while the round resolves; None while planning, between rounds or after the battle.
fn windup_block(s: &Session) -> Option<i32> {
    match s.phase {
        _ if s.combat_over => None,
        Phase::Planning if s.plan_confirmed => Some(1),
        Phase::Resolving => Some(s.current_block.max(1)),
        _ => None,
    }
}

/// WINDUP1 W2, W1a: unit `i`'s action in wind-up at `block` that shows one: this round's scheduled action in wind-up
/// (`poise::action_in_windup`) spanning more than one block, with a pose its body has a clip for; the earliest start.
fn windup_of(s: &Session, i: usize, block: i32, ps: &PoseSet) -> Option<Windup> {
    if !s.w.units[i].is_alive() || s.w.grid.pos(i).is_none() {
        return None;
    }
    s.resolver
        .scheduled
        .iter()
        .enumerate()
        .filter(|(_, a)| a.unit == i && a.impact > a.start && emberwake_battle::poise::action_in_windup(a, block))
        .filter_map(|(idx, a)| {
            let pose = command_pose(s, &a.command);
            ps.clip(pose).filter(|_| pose.held()).map(|clip| Windup { round: s.current_round, idx, pose, clip, blocks: (a.start, a.impact) })
        })
        .min_by_key(|w| w.blocks.0)
}

/// WINDUP1 5: the pose unit `i` stands in while the party plans, by priority: `Dazed` while Staggered; `Guard` for a
/// Guard round intent (classic: a first command that is a Defend); the opener's pose (W1, W1b: the first skill or item
/// command of its round, its span ignored; a hero's from its plan, an enemy's its tell when that is a skill or item
/// action, else, when its plan is `revealed`, the first such action of it); else by the round intent (Assault an
/// attack, Support a buff, Maneuver: Engage an attack, Utility a debuff); else none. An enemy shows no pose while the
/// enemy tells are hidden (WQ1; Staggered is a status, not a tell, and still shows).
fn plan_pose(s: &Session, i: usize, tells: bool, revealed: &[usize]) -> Pose {
    use emberwake_battle::round_intent::{self, ManeuverKind, RoundIntent};
    let u = &s.w.units[i];
    if !u.is_alive() || s.w.grid.pos(i).is_none() {
        return Pose::None;
    }
    if u.st.has_any("stagger") {
        return Pose::Dazed;
    }
    let enemy = !s.w.allies.contains(&i);
    if enemy && !tells {
        return Pose::None;
    }
    let (defends, opener) = if enemy {
        let mut acts: Vec<&emberwake_battle::resolver::ScheduledAction> = s.resolver.scheduled.iter().filter(|a| a.unit == i && !a.cancelled).collect();
        acts.sort_by_key(|a| a.start);
        let first = acts.first().map(|a| command_pose(s, &a.command)).unwrap_or_default();
        let opener = if first != Pose::None || !revealed.contains(&i) {
            first
        } else {
            acts.iter().map(|a| command_pose(s, &a.command)).find(|p| *p != Pose::None).unwrap_or_default()
        };
        (acts.first().is_some_and(|a| matches!(a.command.kind, CmdKind::Defend { .. })), opener)
    } else {
        use emberwake_battle::plan::{Mode, Step};
        let steps = s.plan.steps_of(i);
        let def = |st: &Step| st.skill.and_then(|k| s.w.cat.skills.get(k));
        let pose = |st: &Step| match st.mode {
            Mode::Item => s.w.cat.items.get(&st.item_id).map_or(Pose::None, item_pose),
            _ => def(st).map_or(Pose::None, skill_pose),
        };
        let defends = steps.first().filter(|st| st.mode != Mode::Item).and_then(|st| def(st)).is_some_and(|d| emberwake_battle::skill::kind(d) == Kind::Defend);
        (defends, steps.iter().map(pose).find(|p| *p != Pose::None).unwrap_or_default())
    };
    let intent = round_intent::intent_of(&s.w, i);
    if intent == Some(RoundIntent::Guard) || (intent.is_none() && defends) {
        return Pose::Guard;
    }
    if opener != Pose::None {
        return opener;
    }
    match intent {
        Some(RoundIntent::Assault) => Pose::Attack,
        Some(RoundIntent::Support) => Pose::Buff,
        Some(RoundIntent::Maneuver) => match round_intent::maneuver_of(&s.w, i) {
            ManeuverKind::Engage => Pose::Attack,
            ManeuverKind::Utility => Pose::Debuff,
            ManeuverKind::Disengage => Pose::None,
        },
        _ => Pose::None,
    }
}

/// WINDUP1 W3 and 5: the base a unit shows and its pose: its wind-up's clip; while planning its stance pose's clip
/// (when the body has one); `idle_guarded` while it guards; `dazed` while it is Staggered (when the body has it); else
/// `idle`.
fn base_of(a: &UnitAnim, guarded: bool, staggered: bool, ps: &PoseSet) -> (&'static str, Pose) {
    if let Some(w) = a.windup {
        return (w.clip, w.pose);
    }
    if let Some(c) = ps.clip(a.stance).filter(|_| a.stance != Pose::None) {
        return (c, a.stance);
    }
    if guarded {
        return ("idle_guarded", Pose::Guard);
    }
    match ps.clip(Pose::Dazed).filter(|_| staggered) {
        Some(c) => (c, Pose::Dazed),
        None => ("idle", Pose::None),
    }
}

/// WINDUP1 6.1: `a` loops the base `base_of` gives from `at` (an unchanged base keeps its clock): a pose starts at its
/// `loop_from` under skip animations and holds still under reduce motion; `idle` and the guard as before.
fn put_base(a: &mut UnitAnim, guarded: bool, staggered: bool, at: u64, m: Motion, ps: &PoseSet) {
    let (clip, pose) = base_of(a, guarded, staggered, ps);
    let from = if m.instant && pose.held() { ps.loop_from(clip) } else { 0 };
    a.player.set_base_from(clip, at, from, m.still && pose.held());
    a.pose = pose;
}

/// WINDUP1 W2: where a scheduled action aims: its target unit's tile (the tile it stood on before this change when it
/// left the grid), else an area's aim; the command kind `action_clips` reads ("attack", "area", "support"; "" else).
fn action_aim(s: &Session, act: &emberwake_battle::resolver::ScheduledAction, before: &[Option<V2>]) -> (&'static str, Option<V2>) {
    let at = |t: usize| s.w.grid.pos(t).or(before.get(t).copied().flatten());
    let (kind, aim) = match &act.command.kind {
        CmdKind::Attack { target, .. } => ("attack", target.and_then(at)),
        CmdKind::Area { aim, .. } => ("area", Some(*aim)),
        CmdKind::Support { target, .. } => ("support", at(*target)),
        CmdKind::Item { target, .. } => ("", at(*target)),
        _ => ("", None),
    };
    (kind, aim.filter(|c| *c != emberwake_battle::grid::NO_CELL))
}

/// DIR4-G: `f` turned to look from `from` toward `to` (`Facing::toward`: up or down when the vertical distance is
/// larger, keeping `left`; otherwise the side view, left when `to` is left; the same tile keeps `f`).
fn turn(f: Facing, from: V2, to: V2) -> Facing {
    f.toward(to.x - from.x, to.y - from.y)
}

/// The battle's token animations (SPRITE1-U): one per unit, the last popped action already animated and the action
/// log length last seen.
#[derive(Debug, Clone, Default)]
struct Tokens {
    anims: Vec<UnitAnim>,
    popped_seen: Option<(i32, i32, usize)>,
    log_len_seen: usize,
    /// FACE1: the Show enemy tells setting at the last catch-up (hidden tells: enemies do not show their intent).
    tells: bool,
    /// WINDUP1: the pose clips of each unit's body (`BattleScreen::learn_poses`; none until known: no poses) and the
    /// reduce motion setting at the last catch-up (6.1).
    poses: Vec<PoseSet>,
    still: bool,
    /// STATUSFX1 S4: the apply bursts waiting for the screen to place them (unit, status, clock ms;
    /// `BattleScreen::push_bursts`) and the wall clock's start for the slot cycling (S7: wall ms).
    bursts: Vec<(usize, &'static str, u64)>,
    epoch: Option<Instant>,
    /// VISCLAR1 VC4: the debug Party foresight off at the last catch-up (a foresight reveal poses as unrevealed).
    foresight_off: bool,
}

impl Tokens {
    /// The tokens catch up with `s` at `now` (the animation clock). While the battle moves forward (the action log
    /// does not shrink) a unit that moved slides there playing `walk` for `walk_ms`, a hit plays `hurt` (`hurt_guarded` while it
    /// guards), a fall plays `death`, raising a guard plays `defend`, and the action just resolved plays its actor's
    /// attack clip (`action_clips`); the base loop is `idle_guarded` while it guards, else `idle`. Anything else (a new
    /// battle, another session) only takes the new state; `instant` (the skip setting) takes the end state at once.
    /// FACE1, four-way since DIR4-G (also under `instant`): a unit turns toward its step (`turn`: up or down when the
    /// step is more vertical, which keeps its left/right; it shows that facing while it slides, `UnitAnim::faces`),
    /// then toward the target (or the area's aim) of the action it just resolved; while the party plans each unit
    /// shows its planned or intended target (`plan_facing`; an enemy only while `tells`, the Show enemy tells setting,
    /// is on), and the confirm keeps that facing. A new battle faces the party right and the foes left (side view).
    /// WINDUP1 (bodies with the pose clips, `poses`): while the party plans each unit stands in its round's pose
    /// (`plan_pose`); an action over one block shows its wind-up from its start (W2, facing its target), strikes from
    /// its clip's `strike` frame at its pop (W4), and the base is the wind-up, the guard, `dazed` while Staggered or
    /// `idle` (W3); a break or the Staggered status landing plays `stagger` (W6, W7).
    #[cfg(test)]
    fn catch_up(&mut self, s: &Session, now: u64, instant: bool, walk_ms: u64, tells: bool) {
        self.follow(s, now, instant, walk_ms, tells, false);
    }

    /// `catch_up`, and FX1-L1 rule I: with `defer` the units' reactions to the step (hurt, defend, death, the fall, the
    /// guard's base loop, the HP the board shows) wait for its impact (`react` with the returned state); moves, facing
    /// and the actor's clip start now. The popped action's clip starts at `t0`: the pop, or the end of the actor's
    /// slide when it moved in this step (FX1 4.2 step 1). Returns each unit's state after the step. WINDUP1: a wind-up
    /// starts now; one that ended waits for the impact too (a break's `stagger`, W6), but the actor's (its strike).
    fn follow(&mut self, s: &Session, now: u64, instant: bool, walk_ms: u64, tells: bool, defer: bool) -> Vec<Shown> {
        self.tells = tells;
        let n = s.w.units.len();
        let animate = self.anims.len() == n && s.log.len() >= self.log_len_seen;
        if self.anims.len() != n {
            self.anims = vec![UnitAnim::default(); n];
        }
        let m = Motion { instant, still: self.still };
        let none = PoseSet::default();
        let mut after = Vec::with_capacity(n);
        // The plan is confirmed (or the battle moved on): each unit keeps the facing its planned target gave it.
        if !s.is_planning() {
            for a in &mut self.anims {
                if let Some(f) = a.planned.take() {
                    a.facing = f;
                }
            }
        }
        // Where each unit stood before this change (an action's target may have fallen off the grid since).
        let before: Vec<Option<V2>> = self.anims.iter().map(|a| a.seen.pos).collect();
        // Only this round's pop: `scheduled` is rebuilt each round, so an older index names another action.
        let popped = s.last_popped.filter(|p| animate && Some(*p) != self.popped_seen && p.0 == s.current_round);
        let popped_act = popped.and_then(|(_, _, idx)| s.resolver.scheduled.get(idx).map(|a| (idx, a)));
        // WINDUP1 W4: the popped action is the wind-up its actor showed (it strikes from the held pose)
        let struck = popped_act.and_then(|(idx, a)| self.anims[a.unit].windup.filter(|w| w.round == s.current_round && w.idx == idx).map(|_| a.unit));
        // WINDUP1 5: the planning poses (none once the plan is confirmed)
        let planning = s.is_planning();
        let revealed: Vec<usize> = if planning && tells { seen_intents(&s.w, s.enemy_intents(), self.foresight_off).iter().filter(|r| r.revealed).map(|r| r.unit).collect() } else { Vec::new() };
        let block = windup_block(s);
        for (i, u) in s.w.units.iter().enumerate() {
            let pos = s.w.grid.pos(i);
            let guarded = u.guard > 0.0 || u.stance == Stance::Defensive;
            let staggered = u.st.has_any("stagger");
            let broken = s.w.poise.get(&i).is_some_and(|p| p.broken);
            let ps = self.poses.get(i).filter(|_| self.poses.len() == n).unwrap_or(&none);
            let stance = if planning { plan_pose(s, i, tells, &revealed) } else { Pose::None };
            let shows = block.and_then(|b| windup_of(s, i, b, ps));
            let a = &mut self.anims[i];
            a.stance = stance;
            // W2: a wind-up starts now; W8: one that ended ends now, unless its end waits for the step's impact (rule I)
            let key = |w: &Option<Windup>| w.map(|w| (w.round, w.idx));
            if shows.is_some() && key(&shows) != key(&a.windup) {
                a.windup = shows;
            } else if shows.is_none() && a.windup.is_some() && (!defer || !animate || struck == Some(i)) {
                a.windup = None;
            }
            let shown = Shown { hp: u.hp, alive: u.is_alive(), guarded, broken, staggered, winding: shows.is_some() };
            after.push(shown);
            if animate {
                if let (Some(from), Some(to)) = (a.seen.pos, pos) {
                    a.facing = turn(a.facing, from, to);
                    a.step = a.facing;
                    if from != to && !instant && walk_ms > 0 {
                        // INPUT1 MR12: a dash (its board event) runs faster than a walk, with its own clip
                        a.dash = dashed(s, i);
                        a.dash_clip = a.dash;
                        a.slide = Some((from, now));
                        a.player.play_for("walk", now, a.slide_ms(walk_ms));
                    }
                }
                a.seen.pos = pos;
                if !defer {
                    react_unit(a, shown, now, m, ps);
                    // STATUSFX1 S4: a step that popped an action without an impact to wait for bursts at once
                    set_marks(a, i, &fx::marks_of(&u.st), now, popped.is_some() && !instant && shown.alive, &mut self.bursts);
                }
            } else {
                a.player.stop();
                a.want = None;
                a.slide = None;
                // Fallen before this view: the dead marker at once.
                a.fell = if shown.alive { None } else { Some(0) };
                a.facing = Facing::side(!s.w.allies.contains(&i));
                a.step = a.facing;
                a.planned = None;
                a.seen = Seen { pos, hp: u.hp, alive: u.is_alive(), guarded, broken, staggered };
                a.marks = fx::marks_of(&u.st).into_iter().map(|(id, pending)| (id, pending, now)).collect();
            }
            // the base the board shows now (under `defer` the state before the step; an unchanged base keeps its clock)
            let (g, st) = (a.seen.guarded, a.seen.staggered);
            put_base(a, g, st, now, m, ps);
            // W2: a unit winding up faces its action's target (unit tile, else the aim cell) at every catch-up
            if let Some(act) = a.windup.and_then(|w| s.resolver.scheduled.get(w.idx).filter(|_| w.round == s.current_round)) {
                if let Some((from, to)) = pos.zip(action_aim(s, act, &before).1) {
                    a.facing = turn(a.facing, from, to);
                }
            }
        }
        if let Some((_, act)) = popped_act {
            let (kind, aim) = action_aim(s, act, &before);
            let u = &s.w.units[act.unit];
            if !act.cancelled && u.is_alive() {
                let at = s.w.grid.pos(act.unit);
                if let Some((from, to)) = at.zip(aim) {
                    let a = &mut self.anims[act.unit];
                    a.facing = turn(a.facing, from, to);
                }
                if !kind.is_empty() && !instant {
                    let skill = s.w.cat.skills.iter().find(|k| k.name == act.command.name).map_or("", |k| k.id.as_str());
                    let weapon = u.inventory.main_weapon().map_or("", |w| w.weapon_type());
                    let a = &mut self.anims[act.unit];
                    // FX1 4.2 step 1: after its slide when it moved in this step (a dash, a charge, an Engage carry)
                    let t0 = if a.slide.is_some_and(|(_, t)| t == now) { now + a.slide_ms(walk_ms) } else { now };
                    a.want = Some((action_clips(skill, kind, weapon), t0));
                    a.strike = struck == Some(act.unit);
                }
            }
        }
        self.plan_facing(s);
        self.popped_seen = s.last_popped;
        self.log_len_seen = s.log.len();
        after
    }

    /// FX1-L1 rule I: the units react at `at` to the step state `after` (`follow` with `defer`): hurt, defend, death,
    /// the fall, the base loop and the HP and life the board shows; WINDUP1: a break's or the Staggered status's
    /// `stagger`, and a wind-up that ended in the step ends.
    fn react(&mut self, after: &[Shown], marks: &[Vec<fx::Mark>], at: u64, instant: bool) {
        if after.len() != self.anims.len() {
            return;
        }
        let m = Motion { instant, still: self.still };
        let none = PoseSet::default();
        let n = self.anims.len();
        for (i, (a, shown)) in self.anims.iter_mut().zip(after).enumerate() {
            let ps = self.poses.get(i).filter(|_| self.poses.len() == n).unwrap_or(&none);
            react_unit(a, *shown, at, m, ps);
            // STATUSFX1 S4: the statuses the step put on it show from the impact, each new one with its burst
            if let Some(mk) = marks.get(i).filter(|_| marks.len() == n) {
                set_marks(a, i, mk, at, !instant && shown.alive, &mut self.bursts);
            }
        }
    }

    /// STATUSFX1 S7: the wall ms since the first frame drawn (the slot cycling's clock).
    fn wall_ms(&mut self, now: Instant) -> u64 {
        now.saturating_duration_since(*self.epoch.get_or_insert(now)).as_millis() as u64
    }

    /// What the board shows of unit `i` (FX1-L1 rule I: the state before a step until its impact): its HP and whether
    /// it stands; the unit's own before the tokens first caught up.
    fn shown(&self, i: usize, u: &emberwake_rules::unit::Unit, n: usize) -> (i32, bool) {
        match self.anims.get(i).filter(|_| self.anims.len() == n) {
            Some(a) => (a.seen.hp, a.seen.alive),
            None => (u.hp, u.is_alive()),
        }
    }

    /// WINDUP1 6.2, the agent link's `state.tokens`: per unit the clip last drawn, the base clip, the pose the base
    /// shows and the blocks of the wind-up it shows; STATUSFX1 S9: the status looks drawn (`status_fx`, slot order head,
    /// body, feet, tint, alpha).
    fn state(&self) -> Value {
        Value::Array(
            self.anims
                .iter()
                .enumerate()
                .map(|(i, a)| json!({"unit": i, "clip": a.drawn, "base": a.player.base(), "pose": a.pose.id(), "windup": a.windup.map(|w| [w.blocks.0, w.blocks.1]), "status_fx": a.status_fx}))
                .collect(),
        )
    }
}

/// STATUSFX1 S3-S5: unit `i` shows the statuses `now` (table order) from clock `at`: one it showed keeps its start
/// (pending or active as now: a pending status turns full at the round start); a new one starts at `at`, or with
/// `burst` (a step that popped an action; not under skip; it stands) at its apply burst, 40 ms apart, queued for the
/// screen. A status without a burst colour (stealth) starts at `at`.
fn set_marks(a: &mut UnitAnim, i: usize, now: &[fx::Mark], at: u64, burst: bool, bursts: &mut Vec<(usize, &'static str, u64)>) {
    let mut k = 0;
    let marks = now
        .iter()
        .map(|&(id, pending)| match a.marks.iter().find(|m| m.0 == id) {
            Some(m) => (id, pending, m.2),
            None if burst && fx::status_look(id).and_then(fx::first_col).is_some() => {
                let start = at + k * fx::BURST_GAP_MS;
                k += 1;
                bursts.push((i, id, start));
                (id, pending, start)
            }
            None => (id, pending, at),
        })
        .collect();
    a.marks = marks;
}

/// One unit's reaction to its state `now` at clock `at` (SPRITE1-U): a fall plays `death` (held), a hit `hurt`
/// (`hurt_guarded` while it guards), a raised guard `defend`; `instant` (skip) plays none. A unit standing again
/// drops its fall; the base loop is `put_base`'s (W3: the wind-up, `idle_guarded` while it guards, `dazed` while
/// Staggered, else `idle`). WINDUP1 W6, W7 (WQ2): a break (its poise newly broken) or the Staggered status newly on it
/// plays `stagger` instead of a hit's clip (once for both; a body without `stagger` reacts as before); a fall plays
/// only `death` (W8); a wind-up that no longer shows (`now.winding` false) ends.
fn react_unit(a: &mut UnitAnim, now: Shown, at: u64, m: Motion, ps: &PoseSet) {
    let was = a.seen.clone();
    let reels = ps.has("stagger") && ((now.broken && !was.broken) || (now.staggered && !was.staggered));
    if was.alive && !now.alive {
        a.fell = Some(at);
        a.slide = None;
        if !m.instant {
            a.player.play_and_hold("death", at);
        }
    } else if now.alive && reels && !m.instant {
        a.player.play("stagger", at);
    } else if now.alive && now.hp < was.hp && !m.instant {
        a.player.play(if now.guarded { "hurt_guarded" } else { "hurt" }, at);
    } else if now.alive && now.guarded && !was.guarded && !m.instant {
        a.player.play("defend", at);
    }
    if now.alive && a.fell.is_some() {
        a.fell = None;
        a.player.stop();
    }
    if !now.winding || !now.alive {
        a.windup = None;
    }
    put_base(a, now.guarded, now.staggered, at, m, ps);
    (a.seen.hp, a.seen.alive, a.seen.guarded, a.seen.broken, a.seen.staggered) = (now.hp, now.alive, now.guarded, now.broken, now.staggered);
}

impl Tokens {

    /// FACE1: while the party plans, each unit faces its target (`turn`, DIR4-G four-way): an enemy its tell's while
    /// tells are shown (INTENT1: the target unit, else the tile it aims at; hidden tells: it keeps its facing until it
    /// acts), a hero the first step of its plan that names another unit or an area tile; none shows the facing it keeps. Once the plan is confirmed the units keep the
    /// facing they showed (`catch_up`).
    fn plan_facing(&mut self, s: &Session) {
        if !s.is_planning() {
            return;
        }
        let side = |u: usize, to: Option<V2>, f: Facing| s.w.grid.pos(u).zip(to.filter(|c| *c != emberwake_battle::grid::NO_CELL)).map(|(from, to)| turn(f, from, to));
        for a in &mut self.anims {
            a.planned = None;
        }
        for r in if self.tells { s.enemy_intents() } else { Vec::new() } {
            let to = r.line.target.and_then(|t| s.w.grid.pos(t)).or(Some(r.line.cell));
            if let Some(a) = self.anims.get_mut(r.unit) {
                a.planned = side(r.unit, to, a.facing);
            }
        }
        for &h in &s.w.allies {
            let first = s.plan.steps_of(h).into_iter().find_map(|st| match (st.mode, st.target) {
                (emberwake_battle::plan::Mode::Area, _) => Some(st.cell.get()),
                (_, Some(t)) if t != h => s.w.grid.pos(t),
                _ => None,
            });
            if let Some(a) = self.anims.get_mut(h) {
                a.planned = side(h, first, a.facing);
            }
        }
    }
}

/// Skills with a clip of their own on the sprite rig (SPRITE1 section 2; ids from tools/sprites/classes.py). CAST1-S:
/// the Ash-Crawler's `caustic_vomit` plays the crawler rig's `spit` (CREATURE1; a body without it falls back).
const SKILL_CLIPS: [(&str, &str); 6] =
    [("shield_bash", "shield_bash"), ("offhand_jab", "jab"), ("haymaker", "haymaker"), ("sweep_kick", "sweep_kick"), ("cyclone_kick", "sweep_kick"), ("caustic_vomit", "spit")];
/// Weapon types that fight with fists (the brawler's jab and kicks).
const FIST_WEAPONS: [&str; 3] = ["", "fist", "unarmed"];

/// SPRITE1-U: the clips an action's actor may play, best first (the first one its body has is played; none: no clip).
/// `kind` is the command's: "attack" (one target), "area" or "support"; `weapon` the actor's main weapon type ("" =
/// none). A skill with its own rig clip plays it; fists jab one target and sweep an area; any other attack plays
/// `attack`; a support skill plays `cast` (no rig clip yet).
fn action_clips(skill: &str, kind: &str, weapon: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = SKILL_CLIPS.iter().filter(|(s, _)| *s == skill).map(|(_, c)| *c).collect();
    let fists = FIST_WEAPONS.contains(&weapon);
    match kind {
        "attack" if fists => out.extend(["jab", "attack"]),
        "area" if fists => out.extend(["sweep_kick", "kick", "jab", "attack"]),
        "attack" | "area" => out.push("attack"),
        "support" => out.push("cast"),
        _ => {}
    }
    out
}

/// The runner's settle of a world fight (BattleBridge's after_fight by the fight's source): `travel`, `dialogue`,
/// `story` (STORYUI1 Q5: the story sequence moves on, retries or aborts), `trigger` (CAMPAIGN1 T2: a queued battle,
/// settled like a dialogue fight), else `explore`. The battle screen and the debug fight popup both settle through it.
pub fn after_fight(d: &emberwake_world::Data, gs: &mut emberwake_world::GameState, runner: &str, fight: &Dict, outcome: &str, tally: Option<&emberwake_world::battle::Tally>) -> Dict {
    match runner {
        "travel" => travel::after_fight(d, gs, fight, outcome, tally),
        "dialogue" => dialogue::after_fight(d, gs, fight, outcome, tally),
        story::FIGHT_SOURCE => story::after_fight(d, gs, fight, outcome, tally),
        trigger::FIGHT_SOURCE => trigger::after_fight(d, gs, fight, outcome, tally),
        _ => explore::after_fight(d, gs, fight, outcome, tally),
    }
}

/// CAMPAIGN1 P3: the battle bridge's start-AP change: each of the first `heroes` units (the party members; guests and
/// enemies keep theirs) has `mods.start_ap` AP more in round 1 (never below 0, `purse::start_ap`), and the round's
/// start snapshot shows it. Nothing when the mods change nothing.
pub fn lower_start_ap(s: &mut Session, heroes: usize, mods: &purse::BattleMods) {
    if mods.start_ap == 0 {
        return;
    }
    for u in s.w.units.iter_mut().take(heroes) {
        u.ap = purse::start_ap(u.ap as i64, mods) as i32;
    }
    let start = s.snapshot();
    if let Some(first) = s.history.last_mut().and_then(|r| r.snapshots.first_mut()) {
        *first = start;
    }
}

/// The log lines of a settled world fight: its events, and a story fight's notes ("Try again.", the effects'); its
/// campaign events (CAMPAIGN1 W1: log lines, notices the map shows next, the act-start save).
pub fn log_settled(cx: &mut Cx, runner: &str, res: &Dict) {
    crate::region::log_events(cx, json::aget(res, "events"));
    crate::runover::take(cx, res);
    if runner == story::FIGHT_SOURCE {
        for n in json::sget(res, "notes") {
            cx.log(n);
        }
    }
}

enum Mode {
    Arena,
    /// A world fight: the fight dict, the runner that settles it (explore / travel / dialogue / story) and the member ids of
    /// the party units in battle order.
    /// `members`: party member id and the saved pouch ids that found no battle slot (kept on write-back), per ally.
    World { fight: Dict, runner: String, members: Vec<(String, Vec<String>)> },
}

pub struct BattleScreen {
    pub host: Host,
    w: Vec<Widget<Action>>,
    mouse: (i32, i32),
    selected: Option<usize>,
    popup: Option<Popup>,
    playing: bool,
    auto: bool,
    next_at: Instant,
    pending_next_round: bool,
    floats: Vec<Float>,
    flashes: Vec<(usize, Instant)>,
    last_hp: Vec<i32>,
    party_tactic: usize,
    hero_tactic: usize,
    pub message: String,
    show_log: bool,
    log_scroll: i32,
    tuning_checked: Instant,
    mode: Mode,
    encounter: String,
    seed: Option<i64>,
    /// Arena only: the world the encounter belongs to (played with that world's characters); None = catalog.
    arena_world: Option<String>,
    /// Arena only (RULESET1-R): the ruleset picked in the Scene Runner (classic or timeline); None = the encounter's own
    /// (its world's, classic for a catalog encounter).
    arena_ruleset: Option<String>,
    /// Arena only (RULESET1-G): the extensions switched for the run ("-id,+id", the host's `rules`; "" = none).
    arena_rules: String,
    /// Arena only (RULESET1 V2): "Reveal enemy plans" (the host's `plan_reveal`).
    arena_reveal: bool,
    /// Arena only (INPUT1 MR11): the "Diagonal melee" pick (none, maneuver, all; "" = the ruleset row's value: the
    /// host's `melee_diagonal`).
    arena_diagonal: String,
    /// HOOKS1 CN2 (HOOKS1-UI slice 2): the world whose `classes` overlay names the units' classes on the hover cards: a
    /// world fight's world; "" (the class rows' names) in the Arena, a catalog screen.
    class_world: String,
    /// Session events already shown in the message bar.
    events_seen: usize,
    /// Resolver log lines already sounded.
    log_seen: usize,
    /// Where the playtest hub keeps this battle's session (None until first saved).
    slot: Option<usize>,
    /// The panels of the last frame by name (UIGALLERY1: the UI Gallery shows each one on its own), named from
    /// `PANELS`.
    pub panels: Vec<(&'static str, Rect)>,
    /// (requests, checkpoints) of the session at its last save.
    saved: (usize, usize),
    /// Playtest trail events of the battle (plan edits, confirms, the end), handed to the hub on the next tick.
    trail_out: Vec<Value>,
    /// The battle's end is in the trail.
    end_logged: bool,
    /// Sessions an agent-link `new` ended before the hub could be reached (saved on the next tick).
    parked: Vec<(Option<usize>, aegis_host::playtest::Session)>,
    /// A playback of a recorded session (Scene Runner Playtests): watched, not recorded again.
    viewing: bool,
    /// A take-over: where its session branched off (recording id, battle, request), kept in its `end`.
    parent: Value,
    /// SPRITE1-U: the token animations; FX1-L1: the FX clock that replaced the animation clock (ms at the animation
    /// speed setting, standing still through a hit-stop; clips run on it) and when it last advanced.
    tokens: Tokens,
    clock: Clock,
    clock_at: Instant,
    /// FX1-L1 rule I: the step feedback waiting for its action to land (at most one: rule F fires it before the next).
    impacts: Vec<Impact>,
    /// FX1-L1 rule P: autoplay waits `impact_gap_ms` after the last impact.
    gap_until: Option<Instant>,
    /// The command of the request being shown (only a single `step` defers its feedback; rule 7).
    req_cmd: String,
    /// The battle view at the last tick (a view change flushes the impacts, rule F).
    view_seen: Option<View>,
    /// FX1-L1 slice 2: the live effects on the FX clock (board px of the current view), the camera whose shake moves
    /// the board (4.1, 4.5), and the phases already fired when last seen (a newly fired one may shake).
    fx: Fx,
    camera: Camera,
    phases_seen: usize,
    /// FX1-L2 (4.3): the targets stepping aside from a miss (unit, clock ms of the impact, direction away).
    sidesteps: Vec<fx::Sidestep>,
    /// The battle's revision (every request and battle start moves it): the previews below are kept for one.
    rev: u64,
    /// ENCOUNTER2-C (3.3): the open talk (None = none), how many of the session's talk cues were taken, the Arena's
    /// scratch game state of its world (made at the first talk, dropped with the battle), autoplay's plan held back
    /// for a talk (sent when it closes), and the talk box's portraits and node shown.
    talk: Option<BattleTalk>,
    talks_seen: usize,
    talk_gs: Option<emberwake_world::GameState>,
    auto_after_talk: bool,
    faces: Faces,
    talk_node: String,
    /// PORTRAITFX1-P (6.2 row 3, surface `panel`): the hero and intel panels' portraits, on a copy of the FX clock (a
    /// hit-stop pauses them, the animation speed scales them, 4.5); they react to each step's portrait events at its
    /// impact (rule I).
    panel_faces: Faces,
    /// PORTRAITFX1-TL (6.2 row 4, surface `timeline`): the full timeline view's track icons, on the same FX clock copy
    /// as the panels and reacting at the same impacts, capped at the minimal tier (4.6: flash, tints, fade; no jolt,
    /// no breath) however large the window scales them.
    timeline_faces: Faces,
    /// PORTRAITFX2 5.3: the battle's own portrait events: `battle_start` (and `ambush`) still to fire on the first
    /// frame, the round and outcome the last change left (`round_start`, `victory`) and the last popped action whose
    /// `skill` fired.
    portrait_start: bool,
    portrait_round: i32,
    portrait_outcome: String,
    portrait_popped: Option<(i32, i32, usize)>,
    /// ENCOUNTER2-C4: the last locked round whose playback the screen started by itself (0: none yet this battle).
    cutscene: i32,
    /// INPUT1 GH2 b: the members whose round target the player set this round (their ghost is the auto plan on it).
    player_target: Vec<usize>,
    /// GH2 a: each member's hero tactic (an index of `TACTICS`; none = the party tactic).
    member_tactic: std::collections::HashMap<usize, usize>,
    /// GH2 a, IN6 (decision "INPUT1 answers, part 4" Q18): members whose ghost would change their intent and drop
    /// intent-bound buffs: the change that realizes it waits for Confirm, the ghost shows on copies meanwhile.
    deferred: Vec<(usize, Deferred)>,
    /// The round whose planning start drafted the ghosts (GH2 a, GH6).
    ghosted: i32,
    /// IF5: the hovered menu row's preview (revision, row, the plan as if picked); MR13 reads it too.
    preview: Option<(u64, usize, Option<(World, emberwake_battle::plan::Plan)>)>,
    /// GH2 a: the selected deferred member's ghost (revision, member, its plan on copies).
    deferred_view: Option<(u64, usize, Option<(World, emberwake_battle::plan::Plan)>)>,
    /// MR13: the position tiles the last frame marked (the agent link's `overlay.positions`).
    positions_shown: Vec<(V2, bool)>,
    /// VISCLAR1 AR3, VC2: the target arrows and the focus set the last frame drew (the agent link's `overlay.arrows`
    /// and `focus`).
    arrows_shown: Vec<Arrow>,
    focus_shown: Vec<usize>,
    /// TEXTFIT1: the menus' row pitch at the last frame's text size (11 at Normal).
    menu_row: i32,
    /// INPUT1 SL2, SL5, SL7, IS1: the inspected unit and the round it was inspected in (the intel panel is pinned on it
    /// until another inspect, Esc or that round's end).
    inspect: Option<(usize, i32)>,
    /// INPUT1-AP (AP10): the AP gauge's animation (its member, round and look, the levels drawn from and to).
    ap_anim: Option<ApAnim>,
    /// AP10: the round and each party member's AP while it was planned (the gauge's hollow pips during playback).
    round_ap: (i32, Vec<(usize, i32)>),
    /// The AP gauge as last drawn (the agent link's `ap_gauge`; None: hidden).
    ap_gauge: Option<Value>,
    /// INPUT1 TV1: the full timeline view is open (key T), and the view as last drawn (the agent link's
    /// `timeline_view`).
    tv_open: bool,
    tv_shown: Option<aegis_ui::timeline::Timeline>,
    /// INPUT1 PB1: the live impact tracers with the FX clock ms each started at, and the popped action last traced.
    tracers: Vec<(fx::Trace, u64)>,
    traced: Option<(i32, i32, usize)>,
    /// PB2: the round and block the last playback step resolved (the strip's cursor, the view's playhead).
    play_block: Option<(i32, i32)>,
    /// PB3, PB5: the block the last step finished (its end may hold the playback: Block mode, the timeline view open),
    /// and the block the playback waits after with whether Block mode holds it (the agent link's `playback.waiting`).
    block_ended: Option<i32>,
    waiting: Option<(i32, bool)>,
}

/// INPUT1-AP (AP10): the AP gauge's levels in pips (full, solid outline) moving from `from` to `to` over `AP_ANIM_MS`
/// from `at`, for member `unit` in `round`, in the planning look or not.
#[derive(Debug, Clone, Copy)]
struct ApAnim {
    unit: usize,
    round: i32,
    planning: bool,
    from: (f32, f32),
    to: (f32, f32),
    at: Instant,
}

impl ApAnim {
    /// The levels at `now`.
    fn level(&self, now: Instant) -> (f32, f32) {
        let k = (now.saturating_duration_since(self.at).as_secs_f32() * 1000.0 / AP_ANIM_MS).clamp(0.0, 1.0);
        (self.from.0 + (self.to.0 - self.from.0) * k, self.from.1 + (self.to.1 - self.from.1) * k)
    }
}

impl BattleScreen {
    fn blank(cx: &Cx, mode: Mode) -> Self {
        Self {
            host: Host::new(Battle::with_data(cx.data.clone())),
            w: Vec::new(),
            mouse: (0, 0),
            selected: None,
            popup: None,
            playing: false,
            auto: false,
            next_at: Instant::now(),
            pending_next_round: false,
            floats: Vec::new(),
            flashes: Vec::new(),
            last_hp: Vec::new(),
            party_tactic: 0,
            hero_tactic: 0,
            message: String::new(),
            show_log: false,
            log_scroll: 0,
            tuning_checked: Instant::now(),
            mode,
            encounter: cx.encounter.clone(),
            seed: cx.seed,
            arena_world: None,
            arena_ruleset: None,
            arena_rules: String::new(),
            arena_reveal: false,
            arena_diagonal: String::new(),
            class_world: String::new(),
            events_seen: 0,
            log_seen: 0,
            slot: None,
            panels: Vec::new(),
            saved: (0, 0),
            parked: Vec::new(),
            trail_out: Vec::new(),
            end_logged: false,
            viewing: false,
            parent: Value::Null,
            tokens: Tokens::default(),
            clock: Clock::default(),
            clock_at: Instant::now(),
            impacts: Vec::new(),
            gap_until: None,
            req_cmd: String::new(),
            view_seen: None,
            fx: Fx::default(),
            camera: Camera::default(),
            phases_seen: 0,
            sidesteps: Vec::new(),
            rev: 0,
            talk: None,
            talks_seen: 0,
            talk_gs: None,
            auto_after_talk: false,
            faces: Faces::default(),
            panel_faces: Faces::default(),
            timeline_faces: Faces::capped(aegis_ui::portrait_fx::Tier::Minimal),
            portrait_start: false,
            portrait_round: 0,
            portrait_outcome: String::new(),
            portrait_popped: None,
            talk_node: String::new(),
            cutscene: 0,
            player_target: Vec::new(),
            member_tactic: Default::default(),
            deferred: Vec::new(),
            ghosted: 0,
            preview: None,
            deferred_view: None,
            positions_shown: Vec::new(),
            arrows_shown: Vec::new(),
            focus_shown: Vec::new(),
            menu_row: 11,
            inspect: None,
            ap_anim: None,
            round_ap: (0, Vec::new()),
            ap_gauge: None,
            tv_open: false,
            tv_shown: None,
            tracers: Vec::new(),
            traced: None,
            play_block: None,
            block_ended: None,
            waiting: None,
        }
    }

    /// A session worth recording: it starts with `new`, so it replays (a party battle of the world map starts outside
    /// it: PLAYTEST2).
    fn recordable(&self) -> bool {
        !self.viewing && self.host.history.first().is_some_and(|r| r["cmd"] == json!("new"))
    }

    /// Hands the battle's command-host session to the playtest hub when it changed since the last save (the hub saves
    /// it in place while the battle runs).
    fn save_session(&mut self, cx: &mut Cx) {
        for (mut slot, s) in self.parked.drain(..) {
            cx.playtest.put_session(&mut slot, s);
        }
        if let Some(s) = self.host.session.as_ref().filter(|s| s.combat_over && !self.end_logged) {
            self.end_logged = true;
            self.trail_out.push(json!({"type": "battle_end", "round": s.current_round, "result": s.outcome}));
        }
        for e in self.trail_out.drain(..) {
            cx.playtest.trail(e);
        }
        let now = (self.host.history.len(), self.host.checkpoints.len());
        if !self.recordable() || now == self.saved {
            return;
        }
        self.saved = now;
        let s = self.recorded();
        cx.playtest.put_session(&mut self.slot, s);
    }

    /// The command-host session as the recording keeps it (a take-over names its parent in `end`).
    fn recorded(&self) -> aegis_host::playtest::Session {
        let mut s = aegis_host::playtest::Session::of(&self.host);
        if !self.parent.is_null() {
            s.end["parent"] = self.parent.clone();
        }
        s
    }

    /// The playtest trail events of one request the player made: plan edits (add, undo, clear, plan, preset,
    /// preset_all with the unit and round) and confirms (ok, or refused with the reason).
    fn trail_of(&mut self, req: &Value, reply: &Value) {
        let Some(s) = self.host.session.as_ref() else { return };
        let ok = reply["ok"] == json!(true);
        let kind = match req["cmd"].as_str().unwrap_or("") {
            "add_step" => "add",
            "undo" => "undo",
            "clear" => "clear",
            "plan" => "plan",
            "preset" if req.get("unit").is_some() => "preset",
            "preset" => "preset_all",
            "confirm" => {
                let reason = if ok { String::new() } else { reply["error"].as_str().unwrap_or("refused").to_string() };
                self.trail_out.push(json!({"type": "confirm", "ok": ok, "reason": reason, "round": s.current_round}));
                return;
            }
            _ => return,
        };
        if ok {
            self.trail_out.push(json!({"type": "plan_edit", "kind": kind, "unit": req.get("unit").cloned().unwrap_or(Value::Null), "round": s.current_round}));
        }
    }

    /// A new battle starts a new session.
    fn new_session(&mut self) {
        self.slot = None;
        self.saved = (0, 0);
        self.end_logged = false;
        self.viewing = false;
        self.parent = Value::Null;
    }

    /// The Test Arena: the catalog encounter given on the command line (goblin_skirmish by default).
    pub fn sandbox(cx: &Cx) -> Self {
        let mut b = Self::blank(cx, Mode::Arena);
        b.start_battle();
        b
    }

    /// The arena on any encounter (Scene Runner): world None = a catalog encounter, else that world's encounter
    /// played with its authored party of world characters (WorldEncounter).
    pub fn arena(cx: &Cx, world: Option<String>, encounter: &str) -> Self {
        Self::arena_with(cx, world, encounter, None, "", false)
    }

    /// `arena` played by `ruleset` (classic or timeline; None = the encounter's own: RULESET1 3.1.3) with the extensions
    /// `rules` switches ("-id,+id"; "" = none: 3.2) and, with `plan_reveal`, every enemy plan shown (V2).
    pub fn arena_with(cx: &Cx, world: Option<String>, encounter: &str, ruleset: Option<String>, rules: &str, plan_reveal: bool) -> Self {
        Self::arena_full(cx, world, encounter, ruleset, rules, "", plan_reveal)
    }

    /// `arena_with` with the Scene Runner's "Diagonal melee" pick (INPUT1 MR11: none, maneuver or all; "" = the
    /// ruleset row's value), kept on Restart.
    pub fn arena_full(cx: &Cx, world: Option<String>, encounter: &str, ruleset: Option<String>, rules: &str, diagonal: &str, plan_reveal: bool) -> Self {
        let mut b = Self::blank(cx, Mode::Arena);
        b.encounter = encounter.to_string();
        b.arena_world = world;
        b.arena_ruleset = ruleset;
        b.arena_rules = rules.to_string();
        b.arena_diagonal = diagonal.to_string();
        b.arena_reveal = plan_reveal;
        b.start_battle();
        b
    }

    /// A recorded battle replayed into the arena (Scene Runner Playtests). `take_over` None plays it back to its end
    /// to watch (not recorded again); Some((parent, at)) stops after `at` requests and hands the battle to the player
    /// as a new session that names `parent` (recording id, battle, request) in its end. Restart then starts the
    /// encounter afresh with the recorded seed. The message says whether every checkpoint replayed as recorded.
    pub fn playback(cx: &Cx, session: &aegis_host::playtest::Session, take_over: Option<(Value, usize)>) -> Self {
        let mut b = Self::blank(cx, Mode::Arena);
        let first = session.requests.first().cloned().unwrap_or(Value::Null);
        let encounter = first["encounter"].as_str().unwrap_or("").to_string();
        (b.arena_world, b.encounter) = match encounter.split_once(':') {
            Some((w, e)) => (Some(w.to_string()), e.to_string()),
            None => (None, encounter),
        };
        b.seed = first["seed"].as_i64();
        b.arena_ruleset = first["ruleset"].as_str().filter(|r| !r.is_empty()).map(String::from);
        b.arena_rules = first["rules"].as_str().unwrap_or("").to_string();
        b.arena_reveal = first["plan_reveal"].as_bool().unwrap_or(false);
        b.arena_diagonal = first["melee_diagonal"].as_str().unwrap_or("").to_string();
        let until = take_over.as_ref().map(|(_, at)| *at);
        let report = aegis_host::playtest::replay(&mut b.host, session, until);
        if b.host.session.is_none() {
            b.message = match &report.broken {
                Some((i, e)) => format!("The recording does not replay: request {} was refused ({e}).", i + 1),
                None => "The recording has no battle.".into(),
            };
            return b;
        }
        let verdict = match (&report.broken, report.matched()) {
            (Some((i, e)), _) => format!("Replay stopped at request {}: {e}.", i + 1),
            (None, true) => format!("Replayed {} requests as recorded.", report.sent),
            (None, false) => format!("Replayed {} requests; the battle diverged from the recording (the data changed?).", report.sent),
        };
        b.new_session();
        match take_over {
            None => b.viewing = true,
            Some((parent, _)) => b.parent = parent,
        }
        b.after_start("");
        // INPUT1 GH2: the replayed round keeps the ghosts it recorded (no new draft)
        b.ghosted = b.session().current_round;
        b.message = format!("{verdict} {}", if b.viewing { "Watching (not recorded); Restart plays it afresh." } else { "Your battle from here." });
        b
    }

    /// A world fight with the saved party (BattleBridge.encounter_for + GameEncounter).
    pub fn world(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<Self, String> {
        let d = cx.data.clone();
        let gs = cx.gs();
        let enc_ref = s(fight, "encounter");
        let hit = d.resolve(&gs.world_id, "encounters", &enc_ref);
        if hit.is_empty() {
            return Err(format!("Unknown encounter '{enc_ref}'."));
        }
        let active = gs.party.active_members();
        if active.is_empty() {
            return Err("No active party member to fight.".into());
        }
        let world = emberwake_world::battle::fight_world(&d, gs, &enc_ref);
        let entry = d.resolve(&world, "encounters", &s(hit, "id"));
        // ROSTER1 BK6: the world refuses a fight its encounter's party_min does not allow (the fight world's entry)
        let short = emberwake_world::roster::party_min_refusal(gs, entry);
        if !short.is_empty() {
            return Err(short);
        }
        // RULESET1 3.1.2: the battle plays by the ruleset of the world that resolves the encounter; V3: the world's plan
        // reveal flag in the game state reveals the enemy plans
        let mut def = emberwake_world::battle::encounter_def_in(&d, gs, &world, entry);
        let zone = s(fight, "zone");
        if !zone.is_empty() {
            def.zone = zone;
        }
        def.id = format!("{world}:{}", s(hit, "id"));
        // PACE1 rule 7 (GameEncounter): a start_status entry is granted when its requires_flags all hold.
        let grants: Vec<bool> = def.start_status.iter().map(|sb| json::aget(json::dict(sb), "requires_flags").iter().all(|c| emberwake_world::quest::condition_met(gs, json::dict(c)))).collect();
        def.start_grant = StartGrant::Only(grants);
        let roster = WorldRoster { d: &d, world: world.clone() };
        let mut party = Vec::new();
        let mut members = Vec::new();
        // ROSTER1-L: the world fields each member (its class path and deck rule in this world, at its stored HP)
        for m in active {
            let Some(field) = emberwake_world::roster::field_unit(&d, gs, m, &world) else { continue };
            // ASSETORG2: the member's character's body and palette (its base unit's, its class's when it has none)
            let character = d.resolve(&world, "characters", &m.lore_ref);
            let (art, palette, _) = if character.is_empty() {
                let unit = d.entry("units", &m.base_unit);
                let class = d.entry("classes", &s(unit, "class_id"));
                (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit), Vec::new())
            } else {
                crate::screens::character_art(&d, character)
            };
            party.push(Prebuilt { unit: field.unit, art, palette, profile: field.profile, key: m.id.clone() });
            members.push((m.id.clone(), field.pouch_unloaded));
        }
        // CAMPAIGN1 P3, W6: at zero upkeep the party starts with less AP (read before the session, logged once)
        let mods = purse::battle_mods(&d, gs);
        let heroes = party.len();
        let mut session = Session::start_with(d.cat.clone(), def, &roster, party);
        lower_start_ap(&mut session, heroes, &mods);
        if !mods.note.is_empty() {
            cx.log(mods.note.clone());
        }
        let mut b = Self::blank(cx, Mode::World { fight: fight.clone(), runner: runner.to_string(), members });
        b.class_world = world;
        b.host.begin(session);
        b.host.restart();
        b.after_start(&s(entry, "intro"));
        if !mods.note.is_empty() {
            b.message = format!("{} {}.", b.message, mods.note);
        }
        cx.set_busy(true);
        Ok(b)
    }

    fn session(&self) -> &Session {
        self.host.session.as_ref().expect("a battle is running")
    }

    fn world_mode(&self) -> bool {
        matches!(self.mode, Mode::World { .. })
    }

    /// Sends one request through the command API and turns HP changes into floating numbers.
    fn send(&mut self, cx: &Cx, req: Value) -> Value {
        self.flush(cx);
        self.req_cmd = req["cmd"].as_str().unwrap_or("").to_string();
        self.note_step();
        let reply = self.host.handle(&req);
        if cx.playtest.on {
            self.trail_of(&req, &reply);
        }
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("refused").to_string();
        }
        self.after_change(cx);
        reply
    }

    /// The board catches up with the battle after a request: the HP changes become floats and hit flashes, new log
    /// lines their sounds, the tokens follow (`sync_anims`). FX1-L1: a pending impact fires first (rule F); a single
    /// `step` that popped an action with a delivery (4.3) holds the step's board feedback until its impact (rule I,
    /// `Impact`), everything else (and everything under skip animations) shows at once. The HUD updates now.
    pub fn after_change(&mut self, cx: &Cx) {
        self.flush(cx);
        self.rev += 1;
        let skip = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        let t = &cx.tuning.value;
        let mut floats = Vec::new();
        let mut struck = Vec::new();
        let mut sounds = Vec::new();
        let mut fell = Vec::new();
        let hps: Vec<i32> = self.session().w.units.iter().map(|u| u.hp).collect();
        if hps.len() == self.last_hp.len() {
            for (i, (&old, &new)) in self.last_hp.iter().zip(hps.iter()).enumerate() {
                if new < old {
                    floats.push((i, format!("-{}", old - new), t.damage_color));
                    struck.push(i);
                    if new <= 0 {
                        sounds.push("sfx/defeat.wav".to_string());
                        fell.push(i);
                    }
                } else if new > old {
                    floats.push((i, format!("+{}", new - old), t.heal_color));
                }
            }
        }
        let hp_before = std::mem::replace(&mut self.last_hp, hps);
        let log = &self.session().resolver.log;
        let lines: Vec<String> = log.iter().skip(self.log_seen).cloned().collect();
        // PORTRAITFX1-P (5.3): the step's portrait events, fired with its feedback (at the impact, rule I)
        let mut portrait = if hp_before.len() == self.last_hp.len() { emberwake_battle::portrait::session_step_events(self.session(), &hp_before, &lines) } else { Vec::new() };
        for line in &lines {
            if let Some(key) = sound_of(line) {
                sounds.push(format!("sfx/{key}.wav"));
            }
        }
        self.log_seen = log.len();
        // PORTRAITFX2 5.3: the step that wins the battle shows `victory` with its feedback
        let outcome_now = self.session().outcome.clone();
        let outcome_before = std::mem::replace(&mut self.portrait_outcome, outcome_now);
        portrait.extend(emberwake_battle::portrait::victory_events(self.session(), &outcome_before));
        let (n, last) = { let s = self.session(); (s.events.len(), s.events.last().cloned()) };
        if n > self.events_seen {
            self.message = last.unwrap_or_default();
            self.events_seen = n;
        }
        self.phase_shakes(cx);
        // INPUT1 PB1: the popped action's impact tracers (timeline battles): with its impact, else at once
        let traces = self.new_traces(cx);
        // PB3, PB5: a step that finished its block may hold the playback at the block's end
        self.note_block_end();
        let pending = if skip || self.req_cmd != "step" { None } else { self.popped_impact(t.walk_ms) };
        let after = self.sync_anims_deferred(cx, pending.is_some());
        match pending {
            Some(mut i) => {
                i.traces = traces;
                i.mods = fx::Mods::of(&lines, !struck.is_empty());
                i.broke = fx::broke(&lines);
                (i.after, i.floats, i.struck, i.sounds) = (after, floats, struck, sounds);
                i.marks = self.session().w.units.iter().map(|u| fx::marks_of(&u.st)).collect();
                i.portrait = portrait;
                self.impacts.push(i);
            }
            None => {
                // 4.5 row 2: a break whose step has no action to land shakes at once (not under skip animations, 4.6)
                if !skip && fx::broke(&lines) {
                    let now = self.clock.now();
                    let seed = { let s = self.session(); fx::shake_seed(s.current_round, s.current_block, fx::BREAK_KEY) };
                    self.shake(cx, fx::HEAVY, now, seed);
                }
                // decision "FX1-L2 answers": a unit that falls with no action to land (a status tick) raises its dust now
                if !skip {
                    self.fall_dust(cx, &fell);
                }
                // PB1: skip animations (or no delivery): the tracers draw as the step's result shows
                let now = self.clock.now();
                self.tracers.extend(traces.into_iter().map(|tr| (tr, now)));
                let shown: Vec<usize> = struck.iter().copied().chain(floats.iter().map(|f| f.0)).collect();
                self.react_portraits_shown(cx, &portrait, now, &shown);
                self.show(cx, floats, struck, sounds)
            }
        }
        self.react_battle_portraits(cx);
        self.push_bursts(cx);
    }

    /// PORTRAITFX2 5.3: the portrait events a change fires at once: `round_start` when the round counter rose
    /// (classic; none under the round intent rule) and `skill` at a popped skill's wind-up (its level against the rows'
    /// `min_level`). Presentation only.
    fn react_battle_portraits(&mut self, cx: &Cx) {
        let Some(s) = self.host.session.as_ref() else { return };
        let rounds = emberwake_battle::portrait::round_events(s, self.portrait_round);
        self.portrait_round = s.current_round;
        let skill = if s.last_popped != self.portrait_popped { emberwake_battle::portrait::skill_event(s) } else { None };
        self.portrait_popped = s.last_popped;
        let now = self.clock.now();
        if !rounds.is_empty() {
            let all: Vec<usize> = rounds.iter().map(|e| e.0).collect();
            self.react_portraits_shown(cx, &rounds, now, &all);
        }
        if let Some((u, level)) = skill {
            self.react_portrait_skill(cx, u, level, now);
        }
    }

    /// PORTRAITFX2 5.3: unit `u`'s `skill` event at FX clock ms `at` for a skill of level `level`: its panel portrait,
    /// and its timeline row while the view is open (a party row; an enemy's row reacts to blows only).
    fn react_portrait_skill(&mut self, cx: &Cx, u: usize, level: i32, at: u64) {
        let Some(s) = self.host.session.as_ref().filter(|s| u < s.w.units.len()) else { return };
        let (ruleset, party) = (s.w.ruleset.id.clone(), s.w.allies.contains(&u));
        let key = self.unit_portrait(cx, u).0.key;
        let rows = self.timeline_view_open() && party;
        self.panel_faces.clock = Clock { ms: at as f64, hold: 0.0 };
        self.timeline_faces.clock = self.panel_faces.clock;
        let event = [emberwake_battle::portrait::SKILL];
        self.panel_faces.react_at(cx, &key, &event, level, PANEL_SURFACE, &ruleset);
        if rows {
            self.timeline_faces.react_at(cx, &key, &event, level, TIMELINE_SURFACE, &ruleset);
        }
        self.panel_faces.clock = self.clock;
        self.timeline_faces.clock = self.clock;
    }

    /// PORTRAITFX2 5.3: `battle_start` (and `ambush`) on the battle screen's first frame, before the first step (a
    /// session that has popped an action or ended fires none).
    fn react_start_portraits(&mut self, cx: &Cx) {
        if !std::mem::take(&mut self.portrait_start) {
            return;
        }
        let Some(s) = self.host.session.as_ref().filter(|s| s.last_popped.is_none() && s.outcome.is_empty()) else { return };
        let events = emberwake_battle::portrait::start_events(s);
        let all: Vec<usize> = events.iter().map(|e| e.0).collect();
        let now = self.clock.now();
        self.react_portraits_shown(cx, &events, now, &all);
    }

    /// FX1-L2 4.4.4 for a fall shown at once (decision "FX1-L2 answers": a unit felled by a status tick): each of
    /// `fell`'s dust at its foot point now, seeded by the round, block and unit.
    fn fall_dust(&mut self, cx: &Cx, fell: &[usize]) {
        let Some(s) = self.host.session.as_ref() else { return };
        let t = &cx.tuning.value;
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), t);
        let squash = if geo.view == View::Stage { 0.75 } else { 1.0 };
        let now = self.clock.now();
        for &u in fell {
            if let Some(c) = s.w.grid.pos(u).or(self.tokens.anims.get(u).and_then(|a| a.seen.pos)) {
                for e in fx::fall_effects(foot_point(&geo, c), squash, now, fx::seed_of(s.current_round, s.current_block, 0xFA11, u), t) {
                    self.fx.push(e);
                }
            }
        }
    }

    /// STATUSFX1 S4: the apply bursts the tokens queued go on the live list at their unit's chest in the current view
    /// (none under skip animations, S7).
    fn push_bursts(&mut self, cx: &Cx) {
        let bursts = std::mem::take(&mut self.tokens.bursts);
        if cx.settings.v["skip_anim"].as_bool().unwrap_or(false) {
            return;
        }
        let Some(s) = self.host.session.as_ref() else { return };
        let t = &cx.tuning.value;
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), t);
        for (u, id, at) in bursts {
            let Some(c) = s.w.grid.pos(u).or(self.tokens.anims.get(u).and_then(|a| a.seen.pos)) else { continue };
            if let Some(e) = fx::status_look(id).and_then(|l| fx::apply_burst(l, &geo, t, foot_point(&geo, c), at)) {
                self.fx.push(e);
            }
        }
    }

    /// FX1 4.5: starts the preset `name` of the game's table at clock `at` (seeded by `seed`; the art bible's limits;
    /// none under reduce motion; a weaker shake never replaces a stronger running one).
    fn shake(&mut self, cx: &Cx, name: &str, at: u64, seed: u64) {
        let t = &cx.tuning.value;
        if let Some(s) = t.shakes.get(name) {
            self.camera.start(s, at, seed, t.shake_limits(), cx.settings.reduce_motion());
        }
    }

    /// FX1 4.5 row 3: the phases fired since the last look whose effects hold a `shake` start the strongest of their
    /// presets at once (also under skip animations; not under reduce motion, 4.6).
    fn phase_shakes(&mut self, cx: &Cx) {
        let Some(s) = self.host.session.as_ref() else { return };
        let fired = &s.phases.fired;
        let presets: Vec<String> = fired
            .keys()
            .skip(self.phases_seen)
            .filter_map(|id| s.phases.phase(id))
            .flat_map(|p| p.effects_of(emberwake_data::SHAKE_EFFECT))
            .filter_map(|e| e.get("preset").and_then(Value::as_str).map(str::to_string))
            .collect();
        // 4.2 step 8: seeded by the round, block and the first new phase's index (never the FX clock)
        let seed = fx::shake_seed(s.current_round, s.current_block, self.phases_seen as u64);
        self.phases_seen = fired.len();
        let t = &cx.tuning.value;
        if let Some(shake) = fx::strongest(presets.iter().map(String::as_str), &t.shakes) {
            let now = self.clock.now();
            self.camera.start(shake, now, seed, t.shake_limits(), cx.settings.reduce_motion());
        }
    }

    /// The step feedback shown now: floats, hit flashes (wall clock) and sounds.
    fn show(&mut self, cx: &Cx, floats: Vec<(usize, String, Rgba)>, struck: Vec<usize>, sounds: Vec<String>) {
        let now = Instant::now();
        // INPUT1 PB3: a float shows twice as long at 0.5x
        let ms = cx.tuning.value.float_ms * self.pace(cx);
        self.floats.extend(floats.into_iter().map(|(unit, text, color)| Float { unit, text, color, born: now, ms }));
        self.flashes.extend(struck.into_iter().map(|i| (i, now)));
        for s in sounds {
            cx.audio.play(Bus::Sfx, &s);
        }
    }

    /// FX1-L1 rule I: the impact of the action this step popped, when it has a delivery (4.3) and its actor stands:
    /// the delivery, `t0` (now, or the end of the actor's slide when it moved in this step, `walk_ms`) and the cells
    /// it goes between; when it lands is resolved with the actor's clip (`resolve_impacts`). Call before the tokens
    /// follow the step.
    fn popped_impact(&self, walk_ms: u64) -> Option<Impact> {
        let s = self.host.session.as_ref()?;
        let t = &self.tokens;
        if t.anims.len() != s.w.units.len() || s.log.len() < t.log_len_seen {
            return None;
        }
        let (_, _, idx) = s.last_popped.filter(|p| Some(*p) != t.popped_seen && p.0 == s.current_round)?;
        let act = s.resolver.scheduled.get(idx)?;
        let u = &s.w.units[act.unit];
        if act.cancelled || !u.is_alive() {
            return None;
        }
        let from = s.w.grid.pos(act.unit)?;
        let at = |i: usize| s.w.grid.pos(i).or(t.anims[i].seen.pos);
        let (mut on_self, mut harms, mut target, mut cells) = (false, false, None, Vec::new());
        let (kind, aim) = match &act.command.kind {
            CmdKind::Attack { target: tg, .. } => {
                target = *tg;
                ("attack", tg.and_then(at))
            }
            CmdKind::Area { aim, cells: c, .. } => {
                cells = c.iter().copied().filter(|c| s.w.grid.is_valid(*c)).collect();
                ("area", Some(*aim))
            }
            CmdKind::Support { target, .. } => ("support", at(*target)),
            CmdKind::Item { target, item, .. } => {
                (on_self, harms) = (*target == act.unit, item.damage > 0);
                ("item", at(*target))
            }
            _ => ("", None),
        };
        let to = aim.filter(|c| *c != emberwake_battle::grid::NO_CELL).unwrap_or(from);
        let def = s.w.cat.skills.iter().find(|k| k.name == act.command.name);
        let skill = def.map_or("", |k| k.id.as_str());
        let weapon = u.inventory.main_weapon();
        let family = weapon.map_or("", |w| w.weapon_type());
        let ranged = weapon.is_some_and(|w| w.is_ranged_weapon());
        let magic = weapon.is_some_and(|w| json::s(&w.item.raw, "damage_type") == "magic");
        let style = fx::style(fx::Act { kind, skill, family, ranged, magic, dist: from.dist(to), on_self, harms });
        let radius = cells.iter().map(|c| c.dist(to)).max().unwrap_or(0);
        let delivery = style.delivery;
        if delivery == fx::Delivery::None {
            return None;
        }
        let (round, block, _) = s.last_popped?;
        let now = self.clock.now();
        let moved = t.anims[act.unit].seen.pos.is_some_and(|p| p != from);
        // INPUT1 MR12: after a dash's shorter slide
        let slide = if dashed(s, act.unit) { DASH_MS } else { walk_ms };
        let t0 = if moved && walk_ms > 0 { now + slide } else { now };
        let clips = if kind == "item" { Vec::new() } else { action_clips(skill, kind, family) };
        Some(Impact {
            at: None,
            actor: act.unit,
            delivery,
            t0,
            clips,
            // WINDUP1 W4: it strikes from the wind-up its actor showed (the tokens have not followed the step yet)
            strike: t.anims[act.unit].windup.is_some_and(|w| w.round == s.current_round && w.idx == idx),
            from,
            to,
            after: Vec::new(),
            floats: Vec::new(),
            struck: Vec::new(),
            sounds: Vec::new(),
            style,
            area: kind == "area",
            seed: fx::seed_of(round, block, idx, act.unit),
            blocks: def.map_or(0, |k| k.blocks),
            mods: fx::Mods::default(),
            broke: false,
            launched: false,
            cells,
            radius,
            target,
            marks: Vec::new(),
            traces: Vec::new(),
            portrait: Vec::new(),
        })
    }

    /// FX1-L1: when each pending impact lands, from its actor's clip (the first candidate its body has; its `impact`
    /// frame, decision "Attack timing on the board" A) and the board's anchors in the current view (a shot's path).
    fn resolve_impacts(&mut self, cx: &mut Cx) {
        let Some(s) = self.host.session.as_ref() else { return };
        let t = cx.tuning.value.clone();
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), &t);
        for i in self.impacts.iter_mut().filter(|i| i.at.is_none()) {
            // WINDUP1 W4: a strike from a held wind-up starts at the clip's `strike` frame, so it lands that much sooner
            let clip_ms = s.w.units.get(i.actor).and_then(|u| cx.art.first_body(&body_keys(&s.art[i.actor], u))).and_then(|b| {
                let name = i.clips.iter().copied().find(|c| cx.art.has_clip(&b, c))?;
                let c = cx.art.clips(&b)?.clip(name)?;
                let skipped = if i.strike { c.ms_before(c.strike) } else { 0 };
                c.impact_ms().map(|ms| ms.saturating_sub(skipped))
            });
            // FX1-L2: the shot's own flight (a falling orb's from above), a volley's or a shockwave's spread
            let pl = place(&geo, i.from, i.to, i.area);
            let cells = foot_points(&geo, &i.cells);
            let at = fx::impact_at(i.delivery, i.t0, clip_ms, fx::flight_px(&i.style, pl), &t).map(|a| a + fx::spread_ms(&i.style, &cells, pl.to));
            i.at = Some(at.unwrap_or(i.t0));
        }
        // Slice 2: the delivery (the trail or the projectile) goes on the live list once its impact time is known.
        let mut out = Vec::new();
        for i in self.impacts.iter_mut().filter(|i| !i.launched) {
            if let Some(at) = i.at {
                i.launched = true;
                out.extend(fx::delivery_effects(&i.style, place(&geo, i.from, i.to, i.area), &foot_points(&geo, &i.cells), i.mods, i.t0, at, i.seed, &t));
            }
        }
        for e in out {
            self.fx.push(e);
        }
    }

    /// FX1-L1: fires the impacts due on the clock (rule I): the units react, the floats, flashes and sounds show, the
    /// hit-stop holds the clock when any HP fell, and autoplay waits `impact_gap_ms` from now (rule P).
    fn fire_due(&mut self, cx: &mut Cx) {
        self.resolve_impacts(cx);
        let now = self.clock.now();
        while self.impacts.first().is_some_and(|i| i.at.is_some_and(|at| at <= now)) {
            let i = self.impacts.remove(0);
            let at = i.at.unwrap_or(now);
            self.fire(cx, i, at);
        }
    }

    /// FX1-L1 rule F: every pending impact fires now, in order (before any later catch-up).
    fn flush(&mut self, cx: &Cx) {
        let now = self.clock.now();
        for i in std::mem::take(&mut self.impacts) {
            let at = i.at.map_or(now, |a| a.min(now));
            self.fire(cx, i, at);
        }
    }

    fn fire(&mut self, cx: &Cx, i: Impact, at: u64) {
        let t = &cx.tuning.value;
        let stop = fx::hit_stop(&i, t);
        if stop > 0.0 {
            self.clock.hold(stop);
        }
        self.tokens.react(&i.after, &i.marks, at, false);
        self.push_bursts(cx);
        // PORTRAITFX1-P: the panel portraits flinch as the action lands
        let shown: Vec<usize> = i.struck.iter().copied().chain(i.floats.iter().map(|f| f.0)).collect();
        self.react_portraits_shown(cx, &i.portrait, at, &shown);
        // INPUT1 PB1: the action's tracers from its impact; PB3: the gap after it doubles at 0.5x
        self.tracers.extend(i.traces.iter().map(|tr| (*tr, at)));
        self.gap_until = Some(Instant::now() + Duration::from_millis(t.impact_gap_ms * self.pace(cx)));
        // Slice 2: the impact effects (4.4) on the target, or on each unit an area struck, and the heavy shake of a
        // Heavy hit or a break (4.5 rows 1-2); neither once skip animations is on (4.6). FX1-L2: an area's aim ring
        // (and a BURST's flash), the effects on the actor for `on_actor` rows, the dust of each unit that falls, all
        // within the impact budget (sparks first); a miss's target steps aside (not under reduce motion).
        let skip = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        if let Some(s) = self.host.session.as_ref().filter(|_| !skip) {
            let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), t);
            let cell = |u: usize| s.w.grid.pos(u).or(self.tokens.anims.get(u).and_then(|a| a.seen.pos));
            let spot = |c: V2| fx::Spot { chest: fx::anchor(&geo, c), foot: foot_point(&geo, c) };
            let struck: Vec<V2> = i.struck.iter().filter_map(|&u| cell(u)).collect();
            let cells: Vec<V2> = if i.style.on_actor {
                vec![i.from]
            } else if i.area {
                struck.clone()
            } else {
                vec![i.to]
            };
            let pl = place(&geo, i.from, i.to, i.area);
            let mut out = Vec::new();
            if i.area || i.style.aim.is_some() {
                out.extend(fx::aim_effects(&i.style, i.area, foot_point(&geo, i.to), i.radius, struck.contains(&i.to), pl.squash, at, i.seed));
            }
            let hits: Vec<fx::Spot> = cells.iter().map(|&c| spot(c)).collect();
            out.extend(fx::impact_effects(&i.style, i.mods, pl, &hits, at, i.seed, t));
            for (k, &u) in i.struck.iter().enumerate() {
                if i.after.get(u).is_some_and(|a| !a.alive) {
                    if let Some(c) = cell(u) {
                        out.extend(fx::fall_effects(foot_point(&geo, c), pl.squash, at, i.seed ^ k as u64, t));
                    }
                }
            }
            for e in fx::budget(out, fx::IMPACT_BUDGET) {
                self.fx.push(e);
            }
            let standing = |u: usize| i.after.get(u).is_some_and(|a| a.alive) && s.w.grid.pos(u).is_some();
            if let Some(u) = i.target.filter(|&u| i.mods.miss && !i.area && u != i.actor && standing(u) && !cx.settings.reduce_motion()) {
                let away = fx::unit_dir(fx::anchor(&geo, i.from), fx::anchor(&geo, s.w.grid.pos(u).unwrap_or(i.to)));
                self.sidesteps.push((u, at, away));
            }
        }
        if !skip && fx::heavy_shake(i.blocks, i.hit(), i.broke) {
            self.shake(cx, fx::HEAVY, at, i.seed);
        }
        self.show(cx, i.floats, i.struck, i.sounds);
    }

    /// PORTRAITFX1-P (5.3, 6.2 row 3): the panel portraits react to a step's events at FX clock ms `at` (its impact, or
    /// now when the step has none to wait for): each unit's portrait fires the kit's `panel` event rows (none under skip
    /// animations, `Faces::react`). Presentation only: nothing here reads or moves the battle. Every blow counts as shown
    /// on the board (the tests' entry; the screen passes the step's own, `react_portraits_shown`).
    #[cfg(test)]
    fn react_portraits(&mut self, cx: &Cx, events: &[(usize, &'static str)], at: u64) {
        let shown: Vec<usize> = events.iter().map(|e| e.0).collect();
        self.react_portraits_shown(cx, events, at, &shown);
    }

    /// `react_portraits` with `shown`, the units the board shows the step's blow on (its floats and hit flashes).
    /// PORTRAITFX1-TL (user 2026-11-01): the timeline view's rows react too, only while the view is open; an enemy's
    /// row only to a blow (`hit`, `crit`, `guard_hit`) the board shows on it (party rows to every event, as the panels).
    fn react_portraits_shown(&mut self, cx: &Cx, events: &[(usize, &'static str)], at: u64, shown: &[usize]) {
        let Some(s) = self.host.session.as_ref().filter(|_| !events.is_empty()) else { return };
        let ruleset = s.w.ruleset.id.clone();
        let row_reacts = |u: usize, e: &str| s.w.allies.contains(&u) || (TIMELINE_ENEMY_EVENTS.contains(&e) && shown.contains(&u));
        let fired: Vec<(String, &str, bool)> = events.iter().filter(|(u, _)| *u < s.w.units.len()).map(|&(u, e)| (self.unit_portrait(cx, u).0.key, e, row_reacts(u, e))).collect();
        let rows = self.timeline_view_open();
        self.panel_faces.clock = Clock { ms: at as f64, hold: 0.0 };
        self.timeline_faces.clock = self.panel_faces.clock;
        for (key, event, row) in fired {
            self.panel_faces.react(cx, &key, &[event], PANEL_SURFACE, &ruleset);
            if rows && row {
                self.timeline_faces.react(cx, &key, &[event], TIMELINE_SURFACE, &ruleset);
            }
        }
        self.panel_faces.clock = self.clock;
        self.timeline_faces.clock = self.clock;
    }

    /// PORTRAITFX1-P: battle unit `c`'s panel portrait and its look. A world fight's party unit is its member
    /// (`hub::member_portrait`'s key, its ROSTER1 conditions); a unit whose hero key names a character of the battle's
    /// world draws as it; any other unit as itself (`Who::of_unit`). The look's HP and life are the board's (rule I:
    /// the state before a step until its impact).
    fn unit_portrait(&self, cx: &Cx, c: usize) -> (Who, emberwake_battle::portrait::Look) {
        let s = self.session();
        let d = &cx.data;
        let key = s.hero_key(c);
        let member = if self.world_mode() && !key.is_empty() { cx.gs().party.members.iter().find(|m| m.id == key) } else { None };
        let (who, conditions) = match member {
            Some(m) => (Who { key: format!("member:{}", m.id), ..Who::of_member(d, m) }, m.conditions.clone()),
            None => {
                let world = if self.world_mode() { self.class_world.clone() } else { self.arena_world.clone().unwrap_or_default() };
                let empty = Dict::new();
                let character = if key.is_empty() { &empty } else { d.resolve(&world, "characters", &key) };
                (Who::of_unit(d, s, c, character), Vec::new())
            }
        };
        let mut look = crate::portraits::look_of_unit(s, c, &conditions);
        let u = &s.w.units[c];
        let (hp, alive) = self.tokens.shown(c, u, s.w.units.len());
        look.hp = hp;
        look.down = !alive && !u.is_withdrawn;
        (who, look)
    }

    /// Rule P: autoplay may send the next step (no impact pending and its gap over).
    fn impacts_done(&self, now: Instant) -> bool {
        self.impacts.is_empty() && self.gap_until.is_none_or(|g| now >= g)
    }

    /// FX1 4.8, the agent link's `state.fx`: the effects not done on the FX clock (`live`, their `kinds`), the impacts
    /// waiting to land (`pending`) and the camera's offset (`shake`), so a test or an agent can wait for them to end.
    fn fx_state(&self) -> Value {
        let now = self.clock.now();
        let kinds = self.fx.kinds(now);
        let (dx, dy) = self.camera.offset_at(now);
        json!({"live": kinds.len(), "pending": self.impacts.len(), "shake": [dx, dy], "kinds": kinds})
    }

    /// SPRITE1-U: the tokens catch up with the battle (`Tokens::catch_up`; the skip animations setting applies). While
    /// an impact is pending they keep the state before its step (rule I).
    fn sync_anims(&mut self, cx: &Cx) {
        let defer = !self.impacts.is_empty();
        self.sync_anims_deferred(cx, defer);
        self.push_bursts(cx);
    }

    fn sync_anims_deferred(&mut self, cx: &Cx, defer: bool) -> Vec<Shown> {
        let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        self.tokens.still = cx.settings.reduce_motion();
        self.tokens.foresight_off = party_foresight_off(&cx.settings.v, cfg!(debug_assertions));
        match self.host.session.as_ref() {
            Some(s) => {
                let tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
                self.tokens.follow(s, self.clock.now(), instant, cx.tuning.value.walk_ms, tells, defer)
            }
            None => Vec::new(),
        }
    }

    /// WINDUP1: looks up the pose clips of each unit's body (`PoseSet`) once a battle's units are known (the catch-ups
    /// run where the art cannot be read); true when they were new, so the tokens catch up again to show the poses.
    fn learn_poses(&mut self, cx: &mut Cx) -> bool {
        let Some(s) = self.host.session.as_ref() else { return false };
        if self.tokens.poses.len() == s.w.units.len() {
            return false;
        }
        self.tokens.poses = s.w.units.iter().enumerate().map(|(i, u)| cx.art.first_body(&body_keys(&s.art[i], u)).and_then(|b| cx.art.clips(&b).map(PoseSet::of)).unwrap_or_default()).collect();
        true
    }

    /// ENCOUNTER2-C4: the Arena's encounter has a script and the ruleset the `new` request picks has the round intent
    /// (the host's choice: the request's ruleset and rules over the catalog's classic or the world's ruleset).
    fn arena_scripted(&self) -> bool {
        let (entry, default) = match (&self.arena_world, self.host.data.as_ref()) {
            (Some(world), Some(d)) => (d.resolve(world, "encounters", &self.encounter).clone(), d.ruleset(world)),
            (Some(_), None) => return false,
            (None, _) => match self.host.cat.encounters.get(&self.encounter) {
                Some(e) => (e.clone(), emberwake_rules::ruleset::Ruleset::classic()),
                None => return false,
            },
        };
        if json::arr(entry.get("script").unwrap_or(&Value::Null)).is_empty() {
            return false;
        }
        let mut r = match self.arena_ruleset.as_deref().filter(|id| !id.is_empty()) {
            Some(id) => match emberwake_rules::ruleset::Ruleset::of(&self.host.cat, id) {
                Ok(r) => r,
                Err(_) => return false,
            },
            None => default,
        };
        if !self.arena_rules.is_empty() && r.adjust(&self.arena_rules).is_err() {
            return false;
        }
        r.has(emberwake_rules::ruleset::ROUND_INTENT)
    }

    /// Starts the arena's encounter through the command host (a world one as "<world>:<id>"), so the session replays.
    fn start_battle(&mut self) {
        let encounter = match &self.arena_world {
            Some(world) => format!("{world}:{}", self.encounter),
            None => self.encounter.clone(),
        };
        let mut req = json!({"cmd": "new", "encounter": encounter});
        if let Some(s) = self.seed {
            req["seed"] = json!(s);
        }
        if let Some(r) = &self.arena_ruleset {
            req["ruleset"] = json!(r);
        }
        // RULESET1-G: the Arena's extension switches and plan reveal (absent when unset: a classic request is unchanged)
        if !self.arena_rules.is_empty() {
            req["rules"] = json!(self.arena_rules);
        }
        if self.arena_reveal {
            req["plan_reveal"] = json!(true);
        }
        // INPUT1 MR11: the Arena's diagonal melee pick (absent when the row's value plays)
        if !self.arena_diagonal.is_empty() {
            req["melee_diagonal"] = json!(self.arena_diagonal);
        }
        // ENCOUNTER2-C4: the party tactic drafts a scripted round's auto heroes (absent when balanced, the kit's default,
        // and in a battle that plays no script, so its recording is unchanged; the hero tactics start over each battle)
        if self.party_tactic != 0 && self.arena_scripted() {
            req["tactics"] = json!({"party": TACTICS[self.party_tactic]});
        }
        let reply = self.host.handle(&req);
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("could not start").to_string();
            return;
        }
        self.new_session();
        self.after_start(reply["intro"].as_str().unwrap_or(""));
    }

    fn after_start(&mut self, intro: &str) {
        self.tokens = Tokens::default();
        self.rev += 1;
        // A new battle: the last one's pending feedback has nothing left to show on (rule F's flush happens in `send`).
        self.impacts.clear();
        self.flashes.clear();
        self.gap_until = None;
        self.fx.clear();
        self.camera.stop();
        self.sidesteps.clear();
        self.phases_seen = self.session().phases.fired.len();
        self.reset_talks();
        self.last_hp = self.session().w.units.iter().map(|u| u.hp).collect();
        // PORTRAITFX2 5.3: the start events wait for the first frame; the round, outcome and popped action as they are
        (self.portrait_start, self.portrait_round, self.portrait_outcome, self.portrait_popped) = (true, self.session().current_round, self.session().outcome.clone(), self.session().last_popped);
        self.log_seen = self.session().resolver.log.len();
        self.selected = self.session().w.allies.first().copied();
        self.popup = None;
        self.playing = false;
        self.auto = false;
        self.pending_next_round = false;
        self.cutscene = 0;
        self.floats.clear();
        self.log_scroll = 0;
        self.player_target.clear();
        self.member_tactic.clear();
        self.deferred.clear();
        self.ghosted = 0;
        self.inspect = None;
        self.tracers.clear();
        self.traced = self.session().last_popped;
        self.play_block = None;
        self.block_ended = None;
        self.waiting = None;
        let (objective, n) = {
            let s = self.session();
            (emberwake_battle::encounter::Objectives::text_of(&s.def.raw, |r| s.units_of_ref(r).first().map(|&c| s.w.units[c].name.clone()).unwrap_or_else(|| r.to_string())), s.events.len())
        };
        self.events_seen = n;
        // ENCOUNTER2 H1, H7: a hold escape asks for its holder first; once picked the line names them
        let s = self.session();
        let objective = match &s.w.hold {
            Some(gate) if gate.holder.is_some() => format!("{objective} {}", hold_line(s)),
            Some(gate) => format!("{objective} Pick who holds {} first.", gate.label),
            None => objective,
        };
        self.message = if intro.is_empty() { format!("{objective} Plan your round, then confirm.") } else { format!("{intro} {objective}") };
    }

    fn planning(&self) -> bool {
        let s = self.session();
        s.phase == Phase::Planning && !s.plan_confirmed
    }

    /// ENCOUNTER2-C SR5, C4: the current round is scripted and locked (its plan is the script's, confirmed at its start).
    fn locked(&self) -> bool {
        self.host.session.as_ref().is_some_and(|s| s.round_locked())
    }

    /// ENCOUNTER2-C4: a locked round still to play shows Skip in Confirm's place.
    fn can_skip(&self) -> bool {
        self.locked() && self.session().phase != Phase::Replay && !self.session().combat_over
    }

    /// ENCOUNTER2-C4 (decision "ENCOUNTER2-C3 answers" (1)): the scripted rounds still to plan draft their auto heroes
    /// with the players' tactics (host `tactics`: the party tactic and each hero tactic by hero key); sent only in a
    /// battle with a script under the round intent, so other battles record no extra request.
    fn send_tactics(&mut self, cx: &Cx) {
        let Some(s) = self.host.session.as_ref().filter(|s| !s.script.is_empty() && ri::on(&s.w)) else { return };
        // sorted by hero key, so the recorded request is the same bytes whatever the map's order
        let mut keyed: Vec<(String, usize)> = self.member_tactic.iter().map(|(&m, &t)| (s.hero_key(m), t)).filter(|(k, _)| !k.is_empty()).collect();
        keyed.sort();
        let members: serde_json::Map<String, Value> = keyed.into_iter().map(|(k, t)| (k, json!(TACTICS[t]))).collect();
        let req = json!({"cmd": "tactics", "party": TACTICS[self.party_tactic], "members": members});
        self.send(cx, req);
    }

    /// BattleBridge.outcome: the session's verdict, else its `timeout_outcome` once the last round is over (ENCOUNTER2
    /// S2, `Session::final_outcome`; fled by default); "" while it can go on.
    fn outcome(&self) -> String {
        self.session().final_outcome()
    }

    /// World battle over: the runner settles the fight from the battle's tally (REWARD1's foes and ENCOUNTER2 W1's
    /// party side: the kit's `battle::apply` settles the party first by `roster::settle_party`: the stay-behind `then`,
    /// the world's rule row, HP; then the rewards), the results popup opens.
    fn settle(&mut self, cx: &mut Cx) {
        let outcome = self.outcome();
        let Mode::World { fight, runner, members } = &self.mode else { return };
        if outcome.is_empty() {
            return;
        }
        let d = cx.data.clone();
        let tally = emberwake_world::battle::Tally::of_world_fight(self.session(), members, &self.class_world);
        let gs = cx.gs_mut();
        let res = after_fight(&d, gs, runner, fight, &outcome, Some(&tally));
        cx.set_busy(false);
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
        }
        log_settled(cx, runner, &res);
        cx.autosave();
        let fight = fight.clone();
        cx.nav.push(Nav::Pop);
        cx.nav.push(Nav::Push(Box::new(crate::hub::Results::new(&res, &outcome, &fight))));
    }

    // ---- Actions (all mutations are command-API requests) -----------------------------------------------------

    /// A player's action (a click or a key): the playback controls click (SoundEvents ui_click), then `run`.
    fn act(&mut self, cx: &mut Cx, a: Action) {
        // ENCOUNTER2-C DT5: while a talk is open only its box acts
        if self.talk.is_some() != matches!(a, Action::Talk(_)) {
            return;
        }
        if matches!(a, Action::Confirm | Action::Step | Action::Autoplay | Action::NextRound | Action::PlaybackSpeed(_) | Action::NextBlock | Action::Skip) {
            cx.audio.play(Bus::Ui, "sfx/ui_click.wav");
        }
        self.run(cx, a);
    }

    /// An action, from the player or chained by another action or autoplay.
    fn run(&mut self, cx: &mut Cx, a: Action) {
        match a {
            Action::Select(u) => {
                // INPUT1 SL4, SL6 (round intent): a party card always selects; the selected hero's card selects nothing
                let again = self.selected == Some(u) && self.host.session.as_ref().is_some_and(|s| ri::on(&s.w));
                self.selected = if again { None } else { Some(u) };
                self.popup = None;
            }
            Action::UnitClick(u) => self.unit_click(cx, u),
            Action::TileClick(c) => self.tile_click(cx, c),
            Action::Choose(i) => {
                let Some(p) = self.popup.take() else { return };
                let Some(row) = p.rows.get(i).filter(|r| r.valid).cloned() else {
                    // a disabled row keeps the menu open (its reason is on the row)
                    self.popup = Some(p);
                    return;
                };
                match row.act {
                    RowAct::Pick(picks) => {
                        if let Some((h, at)) = p.menu {
                            self.popup = Some(p);
                            self.choose_pick(cx, h, at, &picks, &row.label);
                        }
                    }
                    RowAct::SelectAlly(u) => {
                        self.selected = Some(u);
                    }
                    RowAct::Step(step) => {
                        if let Some(h) = self.selected {
                            let r = self.send(cx, json!({"cmd": "add_step", "unit": h, "step": step}));
                            if r["ok"] == json!(true) {
                                self.message = format!("{}: {}", self.session().w.units[h].name, row.label);
                            }
                        }
                    }
                    RowAct::PartyTactic(t) => {
                        self.party_tactic = t;
                        // IF9: the party tactic drafts every member with no pick (the kit skips the others); its
                        // drafts are the player's own call, so no ghost waits for Confirm
                        self.member_tactic.clear();
                        let picked: Vec<usize> = self.session().w.allies.iter().copied().filter(|&a| !rip::picks_of(&self.session().w, a).is_empty()).collect();
                        self.player_target.retain(|a| picked.contains(a));
                        self.deferred.clear();
                        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[t]}));
                        self.send_tactics(cx);
                        self.message = format!("Party drafted: {}", TACTIC_LABELS[t]);
                    }
                    RowAct::HeroTactic(t) => {
                        self.hero_tactic = t;
                        if let Some(h) = self.selected {
                            // IF9: the hero tactic clears the hero's picks and its player-set target, then drafts it
                            self.member_tactic.insert(h, t);
                            if ri::on(&self.session().w) {
                                self.player_target.retain(|a| *a != h);
                                self.deferred.retain(|d| d.0 != h);
                                if !rip::picks_of(&self.session().w, h).is_empty() {
                                    self.send(cx, json!({"cmd": "clear", "unit": h}));
                                }
                            }
                            self.send(cx, json!({"cmd": "preset", "unit": h, "preset": TACTICS[t]}));
                            self.send_tactics(cx);
                            self.message = format!("{} drafted: {}", self.session().w.units[h].name, TACTIC_LABELS[t]);
                            if ri::on(&self.session().w) {
                                self.reghost_after(cx, h);
                            }
                        }
                    }
                }
            }
            Action::SelfTarget => {
                if let Some(h) = self.selected.filter(|_| self.planning() && ri::on(&self.session().w)) {
                    self.target_click(cx, h, RoundTarget::Myself);
                }
            }
            Action::ClosePopup => self.popup = None,
            Action::TimelineView => {
                if self.tv_open {
                    self.close_view(cx);
                } else if self.timeline_view_on() {
                    self.tv_open = true;
                }
            }
            Action::PlaybackSpeed(k) => {
                // INPUT1 PB3: the remembered setting (the Settings Battle tab shows it too); leaving Block mode lets a
                // held playback go on (unless the timeline view holds it)
                if let Some((v, label, _)) = PLAYBACK_SPEEDS.get(k).filter(|_| self.playback_aids()) {
                    cx.settings.v[PLAYBACK_SPEED] = json!(v);
                    self.message = match cx.settings.save() {
                        Err(e) => format!("Playback speed {label}, but the setting was not saved: {e}"),
                        Ok(()) => format!("Playback speed: {label} (P cycles)."),
                    };
                }
            }
            Action::NextBlock => {
                // PB3, PB5: a held playback plays the next block (it holds again at that block's end while it should)
                if self.waiting.is_some() {
                    self.release_block();
                }
            }
            Action::TimelinePick(Some(u)) => {
                // TV5: a party track selects its hero (the view stays open; the selected hero's track selects nothing)
                if self.session().w.allies.contains(&u) && self.selected != Some(u) {
                    self.selected = Some(u);
                    self.popup = None;
                }
            }
            Action::TimelinePick(None) => {}
            Action::StayPick(c) => {
                let r = self.send(cx, json!({"cmd": "stay_pick", "unit": c}));
                if r["ok"] == json!(true) {
                    let line = hold_line(self.session());
                    self.message = format!("{line} Plan your round, then confirm.");
                }
            }
            // ENCOUNTER2-C4: a locked round has no Confirm (the script confirmed it; it plays by itself)
            Action::Confirm if self.locked() => {}
            Action::Confirm => {
                // GH2 a, IN6: the ghosts that wait for Confirm are drafted now (their intent change drops its buffs)
                for (m, d) in std::mem::take(&mut self.deferred) {
                    if self.planning() && rip::picks_of(&self.session().w, m).is_empty() {
                        self.send(cx, d.req(m));
                    }
                }
                let r = self.send(cx, json!({"cmd": "confirm"}));
                if r["ok"] == json!(true) {
                    self.playing = true;
                    self.next_at = Instant::now();
                    self.message = "Resolving the round...".into();
                }
            }
            Action::Clear => {
                if let Some(h) = self.selected {
                    if ri::on(&self.session().w) {
                        self.unpick(cx, h, true);
                    } else {
                        self.send(cx, json!({"cmd": "clear", "unit": h}));
                    }
                }
            }
            Action::Undo => {
                if let Some(h) = self.selected {
                    if ri::on(&self.session().w) {
                        self.unpick(cx, h, false);
                    } else {
                        self.send(cx, json!({"cmd": "undo", "unit": h}));
                    }
                }
            }
            Action::Autoplay => {
                self.auto = !self.auto;
                self.auto_after_talk = false;
                if self.auto && self.planning() {
                    self.auto_plan(cx);
                }
            }
            Action::PartyMenu => self.tactic_menu(true),
            Action::HeroMenu => self.tactic_menu(false),
            Action::NextRound => {
                let r = self.send(cx, json!({"cmd": "advance"}));
                if r["ok"] == json!(true) {
                    self.message = format!("Round {}: plan your party.", self.session().current_round);
                    if self.auto {
                        self.auto_plan(cx);
                    }
                }
            }
            Action::Step => {
                if self.session().phase == Phase::Replay {
                    self.run(cx, Action::NextRound);
                } else {
                    self.send(cx, json!({"cmd": "step"}));
                }
            }
            Action::Restart => {
                if !self.world_mode() {
                    self.save_session(cx);
                    self.start_battle();
                }
            }
            Action::ToggleLog => {
                self.show_log = !self.show_log;
                self.log_scroll = 0;
            }
            Action::Continue => self.settle(cx),
            Action::Leave => cx.nav.push(Nav::Pop),
            Action::Talk(a) => self.talk_reply(cx, a),
            Action::Skip => {
                if !self.can_skip() {
                    return;
                }
                // ENCOUNTER2-C4: the rest of the locked round at once (a held block's wait ends with it)
                self.waiting = None;
                let r = self.send(cx, json!({"cmd": "resolve"}));
                if r["ok"] == json!(true) {
                    self.playing = false;
                    self.round_over(cx, Instant::now());
                }
            }
        }
    }

    /// Autoplay's plan for the round (the party tactic, then Confirm); ENCOUNTER2-C DT5: held back while a talk is open
    /// or waiting to open (a round-start talk), and sent when the last one closes.
    fn auto_plan(&mut self, cx: &mut Cx) {
        if self.talk_due() {
            self.auto_after_talk = true;
            return;
        }
        // ENCOUNTER2-C4: a locked round plays by itself (`tick`); nothing to draft
        if self.locked() {
            return;
        }
        // INPUT1 IF9: autoplay drafts whole rounds (members with picks keep them); no ghost waits for Confirm
        self.deferred.clear();
        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
        self.run(cx, Action::Confirm);
    }

    /// The round's playback is over: its message; autoplay (and, ENCOUNTER2-C4, a locked round) starts the next round on
    /// its own after the round-end pause.
    fn round_over(&mut self, cx: &Cx, now: Instant) {
        let t = cx.tuning.value.clone();
        let pace = self.pace(cx);
        let s = self.session();
        let can_advance = s.can_advance_round();
        let message = if s.combat_over {
            match s.outcome.as_str() {
                "won" => "Victory!".to_string(),
                "lost" => "Defeat.".to_string(),
                o => format!("Battle over: {o}"),
            }
        } else if !can_advance {
            "Round limit reached.".into()
        } else {
            // RULESET1-U slice 5 (RULESET1 12): the round summary under the round intent
            match round_summary_text(s, &Layers::of(&cx.settings.v["battle_overlays"])) {
                Some(sum) => format!("Round over: {sum}. Space (or Next) for the next round."),
                None => "Round over. Space (or Next) for the next round.".into(),
            }
        };
        self.message = message;
        if (self.auto || (self.locked() && !self.viewing)) && can_advance {
            self.next_at = now + Duration::from_millis(t.round_pause_ms * pace);
            self.pending_next_round = true;
        }
    }

    // ---- ENCOUNTER2-C2: the battle talk (docs/specs/ENCOUNTER2_C.md 3.3) ----------------------------------------

    /// A new battle: no talk open, every cue of the session still to open, the Arena's scratch state dropped.
    fn reset_talks(&mut self) {
        self.talk = None;
        self.talks_seen = 0;
        self.talk_gs = None;
        self.auto_after_talk = false;
    }

    /// A talk is open, or a cue of the session waits to open.
    fn talk_due(&self) -> bool {
        self.talk.is_some() || self.host.session.as_ref().is_some_and(|s| s.talks().len() > self.talks_seen)
    }

    /// DT5: once the impacts of the step that raised it have played (rule P's wait), the next cue opens; true while a
    /// talk is open or a cue waits (the tick sends nothing). With none left, autoplay's held plan goes out.
    fn talk_tick(&mut self, cx: &mut Cx, now: Instant) -> bool {
        if self.session().talks().len() < self.talks_seen {
            // another battle began under the screen
            self.reset_talks();
        }
        while self.talk.is_none() && self.talks_seen < self.session().talks().len() {
            if !self.impacts_done(now) {
                return true;
            }
            let cue = self.session().talks()[self.talks_seen].clone();
            self.talks_seen += 1;
            self.open_talk(cx, &cue);
        }
        if self.talk.is_some() {
            return true;
        }
        if std::mem::take(&mut self.auto_after_talk) && self.auto && self.planning() {
            self.auto_plan(cx);
        }
        false
    }

    /// DT6: opens `cue`'s scene (`DialogueRunner::open_scene`) in the game in play (a world fight) or the Arena's
    /// scratch game state of its world (`world_id` set, nothing else); one that cannot open is skipped, its reason in
    /// the message bar.
    fn open_talk(&mut self, cx: &mut Cx, cue: &emberwake_battle::encounter::TalkCue) {
        let d = cx.data.clone();
        let runner = if self.world_mode() {
            if !cx.has_game() {
                return;
            }
            dialogue::DialogueRunner::open_scene(&d, cx.gs_mut(), &cue.dialogue, &cue.speaker)
        } else {
            let world = self.arena_world.clone().unwrap_or_default();
            let gs = self.talk_gs.get_or_insert_with(|| emberwake_world::GameState { world_id: world, ..Default::default() });
            dialogue::DialogueRunner::open_scene(&d, gs, &cue.dialogue, &cue.speaker)
        };
        if runner.closed {
            self.message = if runner.errors.is_empty() { format!("The talk '{}' did not open.", cue.dialogue) } else { runner.errors.join(" ") };
            return;
        }
        self.popup = None;
        self.talk = Some(BattleTalk { dialogue: cue.dialogue.clone(), runner });
    }

    /// The UI Gallery's Battle talk entry: opens tree `dialogue` of `world` (speaker `speaker`) over this battle as a
    /// cue would, in a scratch game state of that world; false (the reason in the message bar) when it cannot open.
    pub(crate) fn preview_talk(&mut self, cx: &mut Cx, world: &str, dialogue: &str, speaker: &str) -> bool {
        self.talk_gs = Some(emberwake_world::GameState { world_id: world.to_string(), ..Default::default() });
        let cue = emberwake_battle::encounter::TalkCue { phase: "preview".into(), dialogue: dialogue.into(), speaker: speaker.into(), round: 1, block: 0 };
        self.open_talk(cx, &cue);
        self.talk.is_some()
    }

    /// The game state the open talk runs in (a clone; None without a talk).
    fn talk_state_of(&self, cx: &Cx) -> Option<emberwake_world::GameState> {
        self.talk.as_ref()?;
        Some(if self.world_mode() { cx.gs().clone() } else { self.talk_gs.clone().unwrap_or_default() })
    }

    /// The open talk's view in `gs` (None without one).
    fn talk_view(&self, cx: &Cx, gs: &emberwake_world::GameState) -> Option<Dict> {
        self.talk.as_ref().map(|t| t.runner.view(&cx.data, gs))
    }

    /// DT9: a reply or Continue in the talk (its effects, flags only, on the talk's game state); DT10: once the scene
    /// closes the battle goes on where the cue found it, the playback after `impact_gap_ms`.
    fn talk_reply(&mut self, cx: &mut Cx, a: TalkAct) {
        let d = cx.data.clone();
        let world = self.world_mode();
        let Some(t) = self.talk.as_mut() else { return };
        let gs = if world { cx.gs_mut() } else { self.talk_gs.get_or_insert_with(Default::default) };
        let res = match a {
            TalkAct::Choose(i) => t.runner.choose(&d, gs, i),
            TalkAct::Advance => t.runner.advance(&d, gs),
        };
        let closed = t.runner.closed;
        if !json::b(&res, "ok", false) {
            self.message = json::sget(&res, "errors").join(" ");
        }
        if closed {
            self.talk = None;
            self.gap_until = Some(Instant::now() + Duration::from_millis(cx.tuning.value.impact_gap_ms * self.pace(cx)));
        }
    }

    /// DT8: the token of the open node's speaker (the character's qualified id `speaker`): the unit on the field whose
    /// hero key is it, names it, or is the party member who plays it.
    fn speaker_unit(&self, cx: &Cx, gs: &emberwake_world::GameState, speaker: &str) -> Option<usize> {
        let s = self.host.session.as_ref()?;
        if speaker.is_empty() {
            return None;
        }
        let qid_of = |r: &str| qid_in(&cx.data, &gs.world_id, r);
        (0..s.w.units.len()).filter(|&c| s.w.grid.pos(c).is_some()).find(|&c| {
            let key = s.hero_key(c);
            !key.is_empty() && (key == speaker || qid_of(&key) == speaker || gs.party.members.iter().any(|m| m.id == key && qid_of(&m.lore_ref) == speaker))
        })
    }

    /// Dropdown of the six tactics under the Party / Hero picker.
    fn tactic_menu(&mut self, party: bool) {
        let current = if party { self.party_tactic } else { self.selected.map_or(self.hero_tactic, |h| self.tactic_of(h)) };
        let help = if emberwake_battle::round_intent::on(&self.session().w) { TACTIC_INTENT_HELP } else { TACTIC_HELP };
        let rows = (0..TACTICS.len())
            .map(|i| Row {
                label: TACTIC_LABELS[i].to_string(),
                detail: if i == current { "current".into() } else { String::new() },
                valid: true,
                act: if party { RowAct::PartyTactic(i) } else { RowAct::HeroTactic(i) },
                tip: vec![(TACTIC_LABELS[i].to_string(), GOLD), (help[i].to_string(), TEXT)],
                warn: false,
                ap_short: None,
            })
            .collect();
        let anchor = self.w.iter().find(|w| w.id == if party { "strip.party" } else { "strip.hero" }).map(|w| w.rect).unwrap_or_default();
        self.popup = Some(Popup { title: if party { "Draft the whole party".into() } else { "Draft the selected hero".into() }, x: anchor.x, y: anchor.y + anchor.h + 1, w: 120, rows, menu: None });
    }

    fn unit_click(&mut self, cx: &Cx, u: usize) {
        let s = self.session();
        let is_ally = s.w.allies.contains(&u);
        let names: Vec<String> = s.w.units.iter().map(|x| x.name.clone()).collect();
        // INPUT1 SL1, SL4-SL7 (round intent): with a hero selected and planning a click targets (SL1, SL6) and a click on
        // the hero itself selects nothing (SL4); with none selected, or outside planning, a click on an ally selects it
        // and one on an enemy inspects it (SL5, SL7)
        if ri::on(&s.w) {
            match self.selected {
                Some(h) if h == u => {
                    self.selected = None;
                    self.popup = None;
                }
                Some(h) if self.planning() => self.target_click(cx, h, RoundTarget::Unit(u)),
                _ if is_ally => {
                    self.selected = Some(u);
                    self.popup = None;
                }
                _ => self.inspect_unit(u),
            }
            return;
        }
        if !self.planning() || self.selected.is_none() {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        }
        let h = self.selected.unwrap();
        if is_ally && u != h {
            let rows = self.options_for(h, "ally", Some(u), None);
            if rows.iter().any(|r| r.valid) {
                self.open_popup(format!("{} -> {}", names[h], names[u]), rows);
            } else {
                self.selected = Some(u);
            }
            return;
        }
        let (title, rows) = if u == h { (format!("{}: own actions", names[h]), self.options_for(h, "self", None, None)) } else { (format!("Attack {}", names[u]), self.options_for(h, "foe", Some(u), None)) };
        self.open_popup(title, rows);
    }

    /// INPUT1 IS1: pins the threat intel panel on unit `u` until another inspect, Esc or the round's end.
    fn inspect_unit(&mut self, u: usize) {
        let (round, name) = (self.session().current_round, self.session().w.units[u].name.clone());
        self.inspect = Some((u, round));
        self.message = format!("Inspecting {name} (Esc clears).");
    }

    /// IS1: the inspected unit while its round lasts (round intent only; None = the panel follows the hover).
    fn inspected(&self) -> Option<usize> {
        let s = self.host.session.as_ref().filter(|s| ri::on(&s.w))?;
        self.inspect.filter(|&(u, r)| r == s.current_round && u < s.w.units.len()).map(|(u, _)| u)
    }

    /// INPUT1 TV1: whether key T may open the full timeline view: a timeline battle (the rule `round_intent`, or any
    /// ruleset beyond classic); classic keeps its screen as it was.
    fn timeline_view_on(&self) -> bool {
        self.host.session.as_ref().is_some_and(|s| ri::on(&s.w) || !s.w.ruleset.rules.is_empty())
    }

    /// TV1: the timeline view is open and shown (a classic battle never shows it).
    fn timeline_view_open(&self) -> bool {
        self.tv_open && self.timeline_view_on()
    }

    /// TV1, PB5: the timeline view closes; a playback it held at a block's end goes on (Block mode keeps holding it).
    fn close_view(&mut self, cx: &Cx) {
        self.tv_open = false;
        if self.waiting.is_some_and(|w| !w.1) && PlaySpeed::of(&cx.settings.v) != PlaySpeed::Block {
            self.release_block();
        }
    }

    /// INPUT1 PB1-PB5: the playback aids are on: a timeline battle (the round intent or any ruleset rule, as TV1).
    fn playback_aids(&self) -> bool {
        self.timeline_view_on()
    }

    /// PB3: a timeline battle's round is playing: from Confirm until the round is over and its last impact has played.
    fn in_playback(&self) -> bool {
        let Some(s) = self.host.session.as_ref() else { return false };
        self.playback_aids() && !(s.phase == Phase::Planning && !s.plan_confirmed) && (s.phase != Phase::Replay || !self.impacts_done(Instant::now()))
    }

    /// PB3: the playback's pacing factor: 2 at 0.5x while a round plays (every walk, clip, float, tracer and pause
    /// between steps takes twice as long, on top of the animation speed), else 1; skip animations plays 0.5x as 1x.
    fn pace(&self, cx: &Cx) -> u64 {
        let half = PlaySpeed::of(&cx.settings.v) == PlaySpeed::Half && !cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        u64::from(half && self.in_playback()) + 1
    }

    /// PB2: a step request is about to go: the block it resolves (block 1 from planning) is the cursor's from now on.
    fn note_step(&mut self) {
        let Some(s) = self.host.session.as_ref().filter(|_| self.req_cmd == "step") else { return };
        let block = match s.phase {
            Phase::Planning => 1,
            Phase::Resolving => s.current_block,
            Phase::Replay => return,
        };
        self.play_block = Some((s.current_round, block));
    }

    /// PB3, PB5: after a request: the block a step finished (the round's last one too, which holds autoplay before
    /// the next round); none from the party's planning on.
    fn note_block_end(&mut self) {
        let Some(s) = self.host.session.as_ref() else { return };
        if s.phase == Phase::Planning && !s.plan_confirmed {
            (self.block_ended, self.waiting) = (None, None);
            return;
        }
        if self.req_cmd != "step" {
            return;
        }
        let Some((_, b)) = self.play_block.filter(|p| p.0 == s.current_round) else { return };
        let ended = match s.phase {
            Phase::Resolving => s.current_block > b,
            Phase::Replay => !s.combat_over,
            Phase::Planning => false,
        };
        self.block_ended = ended.then_some(b);
    }

    /// PB3, PB5: holds the playback after the block the last step finished while Block mode is on or the timeline view
    /// is open (the message bar says how to go on); true while it holds.
    fn hold_at_block_end(&mut self, cx: &Cx) -> bool {
        let block_mode = PlaySpeed::of(&cx.settings.v) == PlaySpeed::Block;
        let Some(b) = self.block_ended.filter(|_| self.playback_aids() && (block_mode || self.timeline_view_open())) else {
            self.waiting = None;
            return false;
        };
        if self.waiting != Some((b, block_mode)) {
            self.waiting = Some((b, block_mode));
            let n = self.session().def.total_blocks;
            self.message = if block_mode {
                format!("Block {b} of {n}: N for the next")
            } else {
                format!("Block {b} of {n}: paused while the timeline view is open (T closes it, N plays the next block)")
            };
        }
        true
    }

    /// PB3, PB5: the held playback goes on with the next block (and holds again at its end while it should).
    fn release_block(&mut self) {
        (self.block_ended, self.waiting) = (None, None);
        self.next_at = Instant::now();
        if self.host.session.as_ref().is_some_and(|s| s.phase == Phase::Resolving) {
            self.message = "Resolving the round...".into();
        }
    }

    /// PB2: the block the strip frames and the timeline view's playhead marks: the one the playback resolves, from its
    /// first step until the next block's; none while the party plans, from Confirm to the first step (the round's
    /// start) and once the round is over and its last impact has played (its end).
    fn cursor_block(&self, now: Instant) -> Option<i32> {
        let s = self.host.session.as_ref()?;
        if !self.playback_aids() || s.phase == Phase::Planning {
            return None;
        }
        let (_, b) = self.play_block.filter(|p| p.0 == s.current_round)?;
        (s.phase != Phase::Replay || !self.impacts_done(now) || self.waiting.is_some()).then_some(b)
    }

    /// PB1: the tracers of the action the last step popped (once each; none outside timeline battles or for any other
    /// request).
    fn new_traces(&mut self, cx: &Cx) -> Vec<fx::Trace> {
        let Some(s) = self.host.session.as_ref() else { return Vec::new() };
        let popped = s.last_popped;
        let fresh = popped.filter(|p| self.req_cmd == "step" && Some(*p) != self.traced && p.0 == s.current_round);
        self.traced = popped;
        let Some((_, _, idx)) = fresh.filter(|_| self.playback_aids()) else { return Vec::new() };
        let tokens = &self.tokens;
        let at = |u: usize| s.w.grid.pos(u).or(tokens.anims.get(u).and_then(|a| a.seen.pos));
        traces_of(s, idx, &at, cx.tuning.value.heal_color)
    }

    /// PB1: the live tracers as the agent link reads them (`overlay.tracers`: from, to, alpha now, dashed); none with
    /// the group `tracers` off.
    fn tracers_json(&self, cx: &Cx) -> Value {
        let on = Layers::of(&cx.settings.v["battle_overlays"]).tracers;
        let (now, still) = (self.clock.now(), cx.settings.reduce_motion());
        let cell = |c: V2| json!([c.x, c.y]);
        Value::Array(self.tracers.iter().filter(|_| on).filter_map(|(t, at)| {
            let a = tracer_alpha(now.saturating_sub(*at), still);
            (a > 0).then(|| json!({"from": cell(t.from), "to": cell(t.to), "alpha": a, "dashed": t.dashed}))
        }).collect())
    }

    /// SL2: the unit (token or card, either side) under the mouse in the last frame (round intent only).
    fn unit_under_mouse(&self) -> Option<usize> {
        if !self.host.session.as_ref().is_some_and(|s| ri::on(&s.w)) {
            return None;
        }
        match aegis_ui::ui::hit(&self.w, self.mouse) {
            Some(Action::UnitClick(u) | Action::Select(u)) => Some(u),
            _ => None,
        }
    }

    fn tile_click(&mut self, cx: &Cx, c: V2) {
        if !self.planning() {
            return;
        }
        let Some(h) = self.selected else { return };
        if ri::on(&self.session().w) {
            self.target_click(cx, h, RoundTarget::Cell(c));
            return;
        }
        let rows = self.options_for(h, "tile", None, Some(c));
        self.open_popup(format!("Tile {}, {}", c.x, c.y), rows);
    }

    // ---- INPUT1-U: one input flow under the round intent (INPUT1 2-5: IF, HK, GH, ET) ------------------------------

    /// GH2 a: member `m`'s hero tactic (the party tactic unless the hero tactic menu set one).
    fn tactic_of(&self, m: usize) -> usize {
        self.member_tactic.get(&m).copied().unwrap_or(self.party_tactic)
    }

    /// GH2 a: `m`'s ghost is its tactic's draft: an active party member with no pick, no player-set target this round
    /// and not planning by hand.
    fn tactic_ghost(&self, m: usize) -> bool {
        let w = &self.session().w;
        ri::on(w) && w.allies.contains(&m) && w.units[m].is_active() && w.has(m) && !rip::manual(w, m) && rip::picks_of(w, m).is_empty() && !self.player_target.contains(&m)
    }

    /// GH2 a: drafts `m`'s ghost with its tactic (`preset`). A draft whose intent change would drop intent-bound buffs
    /// waits for Confirm (IN6, Q18 a: the panel warns "Changing loses ..." meanwhile); a draft equal to the plan is not
    /// sent again. Auto-fill off: nothing (the kit rests every free block).
    fn draft_ghost(&mut self, cx: &Cx, m: usize) {
        if !self.tactic_ghost(m) || !self.session().w.autofill {
            return;
        }
        let draft = Deferred::Preset(TACTICS[self.tactic_of(m)]);
        let s = self.session();
        let Some((w2, p2)) = draft.view(s, m) else { return };
        let drops = ri::intent_of(&w2, m).map(|i| ri::would_drop(&s.w, m, i)).unwrap_or_default();
        let same = ri::intent_of(&w2, m) == ri::intent_of(&s.w, m) && plan_rows(&w2, &p2, m) == plan_rows(&s.w, &s.plan, m) && w2.round_states.get(&m).and_then(|r| r.target) == s.w.round_states.get(&m).and_then(|r| r.target);
        self.deferred.retain(|d| d.0 != m);
        if !drops.is_empty() {
            self.deferred.push((m, draft));
        } else if !same {
            self.send(cx, draft.req(m));
        }
    }

    /// GH2, GH6, GH7: the planning start of a round: the player-set targets and the waiting ghosts reset, the battle
    /// takes the Auto-fill setting (host `autofill`, recorded), every member without a pick gets its tactic's ghost.
    fn plan_start(&mut self, cx: &Cx) {
        self.ghosted = self.session().current_round;
        self.player_target.clear();
        self.deferred.clear();
        let on = cx.settings.v["auto_fill"].as_bool().unwrap_or(true);
        if !on || self.session().w.autofill != on {
            self.send(cx, json!({"cmd": "autofill", "on": on}));
        }
        for m in self.session().w.allies.clone() {
            self.draft_ghost(cx, m);
        }
    }

    /// GH7: the setting changed while planning: every member's ghost is rebuilt at once.
    fn sync_autofill(&mut self, cx: &Cx, on: bool) {
        self.send(cx, json!({"cmd": "autofill", "on": on}));
        if !on {
            self.deferred.clear();
        }
        for m in self.session().w.allies.clone() {
            self.draft_ghost(cx, m);
        }
    }

    /// PL8, GH2: after `h`'s round changed, the members after it that show their tactic's ghost draft it again (the kit
    /// rebuilt them by their intent).
    fn reghost_after(&mut self, cx: &Cx, h: usize) {
        let later: Vec<usize> = self.session().w.allies.iter().copied().skip_while(|&m| m != h).skip(1).collect();
        for m in later {
            self.draft_ghost(cx, m);
        }
    }

    /// IF1, GH2 b: a click names `at` as `h`'s round target (no pick holds one): the hero's intent when it plans a round
    /// on it, else the first intent of that kind of target that does; an intent change that would drop intent-bound
    /// buffs waits for Confirm (IN6). The target becomes player-set.
    fn apply_target(&mut self, cx: &Cx, h: usize, at: RoundTarget) {
        let s = self.session();
        let cur = ri::intent_of(&s.w, h);
        if s.w.round_states.get(&h).and_then(|r| r.target) == Some(at) && self.player_target.contains(&h) {
            return;
        }
        let chosen = cur.filter(|i| plans_on(s, h, *i, at)).or_else(|| intents_for(&s.w, h, at).iter().copied().find(|i| plans_on(s, h, *i, at)));
        let fits_now = cur.is_some_and(|i| ri::target_problem(&s.w, h, i, at).is_empty());
        let drops = chosen.filter(|i| Some(*i) != cur).map(|i| ri::would_drop(&s.w, h, i)).unwrap_or_default();
        self.deferred.retain(|d| d.0 != h);
        let set = match chosen {
            None if fits_now => rip::RoundSet { target: Some(at), ..Default::default() },
            None => return,
            Some(i) if Some(i) == cur => rip::RoundSet { target: Some(at), ..Default::default() },
            Some(i) => rip::RoundSet { intent: Some(i), target: Some(at), ..Default::default() },
        };
        if !self.player_target.contains(&h) {
            self.player_target.push(h);
        }
        let change = Deferred::Round(set);
        if drops.is_empty() {
            self.send(cx, change.req(h));
        } else {
            self.deferred.push((h, change));
        }
    }

    /// IF1: a click on a unit or a tile while `h` plans: the round target first (GH2 b) unless a pick holds it, then the
    /// skill menu at the click.
    fn target_click(&mut self, cx: &Cx, h: usize, at: RoundTarget) {
        let held = rip::picks_of(&self.session().w, h).iter().any(|p| p.holds_target());
        if !held {
            self.apply_target(cx, h, at);
            self.reghost_after(cx, h);
        }
        self.open_menu(h, at, None, false);
    }

    /// IF2: the skill menu's title and rows for `h` on `at` (the kit's `pick_options`, in hotkey order), with "Select
    /// <ally>" last on another ally (SL6).
    fn menu_rows(&self, h: usize, at: RoundTarget) -> (String, Vec<Row>) {
        let s = self.session();
        let w = &s.w;
        let opts = rip::pick_options(w, &s.plan, h, at);
        let mut rows: Vec<Row> = opts
            .iter()
            .map(|o| Row { label: pick_label(s, h, &o.picks), detail: pick_detail(&w.cat, o), valid: o.problem.is_none(), act: RowAct::Pick(o.picks.clone()), tip: pick_tip(s, h, o), warn: !o.drops.is_empty(), ap_short: o.ap_short })
            .collect();
        let held = rip::picks_of(w, h).iter().any(|p| p.holds_target());
        let round = w.round_states.get(&h).and_then(|r| r.target);
        if let RoundTarget::Unit(u) = at {
            if u != h && w.allies.contains(&u) {
                rows.push(Row { label: format!("Select {}", w.units[u].name), detail: String::new(), valid: true, act: RowAct::SelectAlly(u), tip: vec![(format!("Select {}", w.units[u].name), GOLD), ("Plan this hero's round instead.".into(), TEXT)], warn: false, ap_short: None });
            }
        }
        let skill_rows = rows.iter().any(|r| matches!(r.act, RowAct::Pick(_)));
        let title = if held && !skill_rows && !matches!(at, RoundTarget::Cell(_)) && round.is_some_and(|r| r != at) {
            format!("One target per round: {}", target_name(w, round.unwrap()))
        } else if !rows.iter().any(|r| r.valid && matches!(r.act, RowAct::Pick(_))) {
            format!("No skill reaches {} this round", target_name(w, at))
        } else {
            format!("{} -> {}", w.units[h].name, target_name(w, at))
        };
        (title, rows)
    }

    /// IF1, IF4: opens (or, after a pick, re-lists in place) `h`'s skill menu on `at`. Another ally with no usable row
    /// is selected at once (SL6); after a pick the menu closes when no skill row is left.
    fn open_menu(&mut self, h: usize, at: RoundTarget, place: Option<(i32, i32)>, after_pick: bool) {
        let (title, rows) = self.menu_rows(h, at);
        let ally = match at {
            RoundTarget::Unit(u) if u != h && self.session().w.allies.contains(&u) => Some(u),
            _ => None,
        };
        let usable = rows.iter().any(|r| r.valid && matches!(r.act, RowAct::Pick(_)));
        if let Some(u) = ally.filter(|_| !usable && !after_pick) {
            self.selected = Some(u);
            self.popup = None;
            return;
        }
        if rows.is_empty() || (after_pick && !rows.iter().any(|r| matches!(r.act, RowAct::Pick(_)))) {
            self.popup = None;
            if !after_pick {
                self.message = title;
            }
            return;
        }
        let (x, y) = place.unwrap_or(self.mouse);
        let (pw, ph) = (MENU_W, self.menu_row + 4 + rows.len() as i32 * self.menu_row);
        self.popup = Some(Popup { title, x: x.clamp(2, W - pw - 2), y: y.clamp(2, H - ph - 2), w: pw, rows, menu: Some((h, at)) });
    }

    /// IF4: appends `picks` (a menu row or a hotkey) to `h`'s round on `at` through the host's `pick`; the message names
    /// the pick, a later member's dropped pick (GH5) or, from the battle's events, the buffs an intent change dropped
    /// (IN6). An open menu stays at its place and re-lists.
    fn choose_pick(&mut self, cx: &Cx, h: usize, at: RoundTarget, picks: &[rip::Pick], label: &str) {
        let w = &self.session().w;
        let mut req = json!({"cmd": "pick", "unit": h, "target": target_json(at), "pick": pick_json(w, &picks[0])});
        if let Some(then) = picks.get(1) {
            req["then"] = pick_json(w, then);
        }
        let events = self.session().events.len();
        let r = self.send(cx, req);
        if r["ok"] != json!(true) {
            return;
        }
        self.deferred.retain(|d| d.0 != h);
        let name = self.session().w.units[h].name.clone();
        let shown = label.split_once(' ').filter(|(d, _)| d.len() == 1 && d.chars().all(|c| c.is_ascii_digit())).map_or(label, |(_, rest)| rest);
        if let Some(d) = r["dropped"].as_array().and_then(|a| a.first()) {
            self.message = d["reason"].as_str().unwrap_or("").to_string();
        } else if self.session().events.len() == events {
            self.message = format!("{name}: {shown}");
        }
        self.reghost_after(cx, h);
        if let Some(place) = self.popup.as_ref().filter(|p| p.menu.is_some()).map(|p| (p.x, p.y)) {
            self.open_menu(h, at, Some(place), true);
        }
    }

    /// IF7: undo (Z, a right click) takes `h`'s last pick back, Clear (`all`) every pick; with none left the hero shows
    /// its ghost again (GH2 a or b).
    fn unpick(&mut self, cx: &Cx, h: usize, all: bool) {
        let name = self.session().w.units[h].name.clone();
        if !self.planning() {
            return;
        }
        if rip::picks_of(&self.session().w, h).is_empty() {
            self.message = format!("{name}: no pick to {}.", if all { "clear" } else { "undo" });
            return;
        }
        let r = self.send(cx, if all { json!({"cmd": "clear", "unit": h}) } else { json!({"cmd": "unpick", "unit": h}) });
        if r["ok"] != json!(true) {
            return;
        }
        self.message = if all { format!("{name}: picks cleared.") } else { format!("{name}: last pick undone.") };
        self.draft_ghost(cx, h);
        self.reghost_after(cx, h);
    }

    /// HK2, HK3: digit `d` (1-9). With the skill menu open it picks the listed row with that digit (or says why not);
    /// with it closed it picks that hotkey skill on the round target (the proposed one counts, TG6; a self-only skill
    /// picked first targets self).
    fn hotkey(&mut self, cx: &Cx, d: usize) {
        let Some(h) = self.selected.filter(|_| self.planning() && ri::on(&self.session().w)) else { return };
        let s = self.session();
        let w = &s.w;
        let keys = rip::hotkey_skills(w, h);
        let skill_name = keys.get(d - 1).map(|k| w.cat.skills[*k].name.clone());
        if let Some(p) = self.popup.as_ref().filter(|p| p.menu.is_some()) {
            let prefix = format!("{d} ");
            match p.rows.iter().position(|r| r.label.starts_with(&prefix)) {
                Some(k) if p.rows[k].valid => self.run_choose(cx, k),
                Some(k) => self.message = format!("{}: {}", &p.rows[k].label[prefix.len()..], p.rows[k].detail),
                None => self.message = skill_name.map_or_else(|| format!("No skill on key {d}."), |n| format!("{n} is not offered on this target now.")),
            }
            return;
        }
        let Some(&sid) = keys.get(d - 1) else {
            self.message = format!("No skill on key {d}.");
            return;
        };
        // HK2, HK3: the kit names the target (self for a self-only skill picked first)
        let Some(at) = rip::hotkey_target(w, h, sid) else {
            self.message = "Pick a target first (click a unit or a tile)".into();
            return;
        };
        let name = w.cat.skills[sid].name.clone();
        let opts = rip::pick_options(w, &s.plan, h, at);
        match opts.iter().find(|o| matches!(o.picks[..], [rip::Pick::Skill { skill, .. }] if skill == sid)) {
            Some(o) if o.problem.is_none() => {
                let picks = o.picks.clone();
                self.choose_pick(cx, h, at, &picks, &name);
            }
            Some(o) => self.message = format!("{name}: {}", o.problem.clone().unwrap_or_default()),
            None => self.message = format!("{name} can't act on {} this round.", target_name(w, at)),
        }
    }

    /// `Action::Choose(k)` from a key (the same path as a click).
    fn run_choose(&mut self, cx: &Cx, k: usize) {
        let Some(p) = self.popup.clone() else { return };
        let Some(row) = p.rows.get(k).filter(|r| r.valid).cloned() else { return };
        match (row.act, p.menu) {
            (RowAct::Pick(picks), Some((h, at))) => self.choose_pick(cx, h, at, &picks, &row.label),
            (RowAct::SelectAlly(u), _) => {
                self.selected = Some(u);
                self.popup = None;
            }
            _ => {}
        }
    }


    /// The ruleset chip of an Arena fight beyond classic (RULESET1-U, user 2026-10-29: the Scene Runner card's text);
    /// None in a world battle and in a plain classic one.
    fn arena_chip(&self) -> Option<String> {
        let s = self.host.session.as_ref()?;
        if self.world_mode() || s.w.ruleset.rules.is_empty() {
            return None;
        }
        Some(crate::runner::chip_text(&s.w.ruleset.name, &self.arena_rules, &self.arena_diagonal, s.w.plan_reveal && s.w.ruleset.has(emberwake_rules::ruleset::PLAN_BADGES)))
    }

    fn open_popup(&mut self, title: String, rows: Vec<Row>) {
        if rows.is_empty() {
            self.message = "Nothing to do there.".into();
            return;
        }
        let (mx, my) = self.mouse;
        let w = 170;
        let h = self.menu_row + 4 + rows.len() as i32 * self.menu_row;
        self.popup = Some(Popup { title, x: mx.clamp(2, W - w - 2), y: my.clamp(2, H - h - 2), w, rows, menu: None });
    }

    /// Candidate steps for hero `h` (`what`: foe / ally / self / tile), each probed through the API.
    fn options_for(&mut self, h: usize, what: &str, target: Option<usize>, cell: Option<V2>) -> Vec<Row> {
        let skills = self.host.handle(&json!({"cmd": "skills", "unit": h}))["skills"].as_array().cloned().unwrap_or_default();
        let mut steps: Vec<(String, Value, Tip)> = Vec::new();
        let target_cell = target.map(|t| {
            let p = self.session().w.pos(t);
            json!([p.x, p.y])
        });
        for sk in &skills {
            let id = sk["skill"].as_str().unwrap_or("").to_string();
            let name = sk["name"].as_str().unwrap_or("").to_string();
            let kind = sk["kind"].as_str().unwrap_or("");
            let area = sk["area"].as_str().unwrap_or("NONE") != "NONE";
            let mut tip: Tip = vec![(name.clone(), GOLD)];
            if let Some(d) = sk["description"].as_str().filter(|d| !d.is_empty()) {
                tip.push((d.to_string(), TEXT));
            }
            tip.push((format!("{} AP, {} block{}, reach {}-{}", sk["ap"], sk["blocks"], if sk["blocks"] == json!(1) { "" } else { "s" }, sk["reach"][0], sk["reach"][1]), DIM));
            // HOOKS1-UI slice 3 (HC1): the host's `hp_cost` (only on a skill that costs HP)
            if let Some(c) = sk["hp_cost"].as_i64().filter(|c| *c > 0) {
                tip.push((intent_text::hp_cost_text(c as i32), FOE));
            }
            match what {
                "foe" => match kind {
                    "ATTACK" if area => steps.push((name, json!({"mode": "area", "skill": id, "cell": target_cell}), tip)),
                    "ATTACK" | "DISENGAGE" => steps.push((name, json!({"mode": "engage", "skill": id, "target": target}), tip)),
                    _ => {}
                },
                "ally" if kind == "SUPPORT" => steps.push((name, json!({"mode": "ally", "skill": id, "target": target}), tip)),
                "self" if matches!(kind, "DEFEND" | "FOCUS" | "REST" | "DISENGAGE" | "SUPPORT") => steps.push((name, json!({"mode": "self", "skill": id}), tip)),
                "tile" if kind == "ATTACK" && area => steps.push((name, json!({"mode": "area", "skill": id, "cell": cell.map(|c| json!([c.x, c.y]))}), tip)),
                _ => {}
            }
        }
        // ITEMUSE1 rule 2: "Use <item> (<n> left)" for each pouch item that may be aimed at the clicked unit.
        if let Some(t) = if what == "self" { Some(h) } else { target.filter(|_| what != "tile") } {
            let items = self.host.handle(&json!({"cmd": "items", "unit": h, "target": t}))["items"].as_array().cloned().unwrap_or_default();
            for it in items {
                let name = it["name"].as_str().unwrap_or("").to_string();
                let mut tip: Tip = vec![(name.clone(), GOLD)];
                if let Some(d) = it["description"].as_str().filter(|d| !d.is_empty()) {
                    tip.push((d.to_string(), TEXT));
                }
                let reach = match it["reach"].as_i64().unwrap_or(0) {
                    0 => "self only".to_string(),
                    r => format!("reach {r}"),
                };
                tip.push((format!("{} AP, {} block{}, {reach}; used when the action resolves", it["ap"], it["blocks"], if it["blocks"] == json!(1) { "" } else { "s" }), DIM));
                steps.push((format!("Use {name} ({} left)", it["left"]), json!({"mode": "item", "item": it["item"], "target": t}), tip));
            }
        }
        if what == "tile" {
            if let Some(c) = cell {
                steps.insert(0, ("Move here".into(), json!({"mode": "goto", "cell": [c.x, c.y]}), vec![("Move here".into(), GOLD), ("1 AP per tile; moving must come first in a round.".into(), TEXT)]));
            }
        }
        let mut rows = Vec::new();
        for (label, step, mut tip) in steps {
            let p = self.host.handle(&json!({"cmd": "probe", "unit": h, "step": step}));
            let valid = p["valid"] == json!(true);
            let detail = if valid {
                let es = p["entries"].as_array().cloned().unwrap_or_default();
                let dmg: i64 = es.iter().map(|e| e["est_damage"].as_i64().unwrap_or(0)).sum();
                let heal: i64 = es.iter().map(|e| e["est_heal"].as_i64().unwrap_or(0)).sum();
                let first = es.first().map(|e| e["start"].as_i64().unwrap_or(0)).unwrap_or(0);
                let last = es.last().map(|e| e["end"].as_i64().unwrap_or(0)).unwrap_or(0);
                tip.push((format!("Blocks {first}-{last}, AP left after: {}", p["ap_left"]), ALLY));
                let mut d = format!("{} AP", p["ap"].as_i64().unwrap_or(0));
                if let Some(c) = skills_hp(&skills, &step) {
                    d.push_str(&format!("  {c} HP"));
                }
                if dmg > 0 {
                    d.push_str(&format!("  ~{dmg} dmg"));
                }
                if heal > 0 {
                    d.push_str(&format!("  +{heal} HP"));
                }
                d
            } else {
                // HOOKS1-UI slice 3: an HP-cost refusal in words
                let r = intent_text::refusal_text(p["reason"].as_str().unwrap_or(""));
                tip.push((r.clone(), FOE));
                r
            };
            rows.push(Row { label, detail, valid, act: RowAct::Step(step), tip, warn: false, ap_short: None });
        }
        rows.sort_by_key(|r| !r.valid);
        rows
    }
}

impl Screen<GameCx> for BattleScreen {
    fn id(&self) -> &'static str {
        if self.world_mode() {
            "battle"
        } else {
            "test_arena"
        }
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.mouse = mouse;
        self.resolve_impacts(cx);
        self.react_start_portraits(cx);
        draw(self, cx, g, mouse, live);
    }

    fn leave(&mut self, cx: &mut Cx) {
        self.save_session(cx);
    }

    /// A playtest note over the battle records its session (the recording's battle index), round, block and phase.
    fn note_context(&self, _cx: &Cx) -> Value {
        let Some(s) = self.host.session.as_ref() else { return Value::Null };
        let phase = match s.phase {
            Phase::Planning => "planning",
            Phase::Resolving => "resolving",
            Phase::Replay => "replay",
        };
        json!({"battle": self.slot.map_or(-1, |i| i as i64), "round": s.current_round, "block": s.current_block, "phase": phase})
    }

    fn tick(&mut self, cx: &mut Cx) {
        let now = Instant::now();
        // SPRITE1-U, FX1-L1 3.1: the tokens and effects run on the FX clock at the animation speed setting (FxSettings
        // 0.5-2x; at most 100 wall ms a frame), which a hit-stop holds still.
        // INPUT1 PB3: at 0.5x the playback's clock runs at half the animation speed
        let speed = cx.settings.v["anim_speed"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0) / self.pace(cx) as f64;
        self.clock.advance((now.duration_since(self.clock_at).as_secs_f64() * 1000.0).min(100.0), speed);
        self.clock_at = now;
        // PORTRAITFX1 4.5: the talk box's portrait clock moves by the frame's wall time
        self.faces.tick();
        // Rule F: the skip setting turning on or another battle view shows the pending impacts at once.
        let view = View::of_setting(&cx.settings.v["battle_view"]);
        if cx.settings.v["skip_anim"].as_bool().unwrap_or(false) || self.view_seen.is_some_and(|v| v != view) {
            self.flush(cx);
            // the live effects are in the old view's px; skip animations draws none (4.6)
            self.fx.clear();
        }
        self.view_seen = Some(view);
        self.fire_due(cx);
        // Slice 2: done effects leave the live list; a shake whose time is up ends.
        let fx_now = self.clock.now();
        self.fx.prune(fx_now);
        self.camera.offset(fx_now);
        self.sidesteps.retain(|s| fx_now < s.1 + fx::SIDESTEP_OUT_MS + fx::SIDESTEP_BACK_MS);
        // INPUT1 PB1: a tracer is gone after its time
        self.tracers.retain(|t| fx_now < t.1 + TRACER_MS);
        // FACE1: the Show enemy tells setting changed (I or the settings): the planning facing follows it. WINDUP1: so
        // do the planning poses, the bodies' pose clips once known and the reduce motion setting.
        let tells = self.tokens.tells != cx.settings.v["show_tells"].as_bool().unwrap_or(true);
        let still = self.tokens.still != cx.settings.reduce_motion();
        if self.learn_poses(cx) || tells || still {
            self.sync_anims(cx);
        }
        if now.duration_since(self.tuning_checked) > Duration::from_millis(500) {
            self.tuning_checked = now;
            if cx.tuning.reload_if_changed() {
                self.message = "Tuning reloaded.".into();
            }
        }
        let t = cx.tuning.value.clone();
        self.floats.retain(|f| now.duration_since(f.born) < Duration::from_millis(f.ms));
        self.flashes.retain(|f| now.duration_since(f.1) < Duration::from_millis(t.hit_flash_ms));
        // The playtest session is saved after every concluded round (and once more at the end).
        let ended = self.host.session.as_ref().is_some_and(|s| s.combat_over) && !self.end_logged;
        if self.host.checkpoints.len() != self.saved.1 || !self.parked.is_empty() || !self.trail_out.is_empty() || ended {
            self.save_session(cx);
        }
        if self.host.session.is_none() {
            return;
        }
        // ENCOUNTER2-C DT5: a talk open (or about to open) holds the playback and autoplay
        if self.talk_tick(cx, now) {
            return;
        }
        // INPUT1 GH2, GH7: a round's planning start drafts the ghosts; the Auto-fill setting reaches the battle (not
        // while watching a recording)
        if !self.viewing && ri::on(&self.session().w) && self.planning() {
            if self.ghosted != self.session().current_round {
                self.plan_start(cx);
            } else {
                let on = cx.settings.v["auto_fill"].as_bool().unwrap_or(true);
                if self.session().w.autofill != on {
                    self.sync_autofill(cx, on);
                }
            }
        }
        // ENCOUNTER2-C4: a locked round's playback starts by itself, once a round-start talk has closed (not while
        // watching a recording: its own requests play it)
        if !self.viewing && self.locked() && self.session().phase != Phase::Replay && self.cutscene != self.session().current_round {
            let s = self.session();
            let (round, banner) = (s.current_round, s.round_script().map(|r| r.banner.clone()).unwrap_or_default());
            self.cutscene = round;
            self.message = if banner.is_empty() { format!("Round {round}: the round plays out (Skip resolves it).") } else { format!("{banner} (Skip resolves the round.)") };
            if !self.playing {
                self.playing = true;
                self.next_at = now;
            }
        }
        // FX1 rule P: the next step waits for `step_ms` and for the last impact plus `impact_gap_ms`.
        let ready = now >= self.next_at && self.impacts_done(now);
        // INPUT1 PB3: at 0.5x the pacing between steps and rounds doubles
        let pace = self.pace(cx);
        if self.pending_next_round {
            if ready && self.session().phase == Phase::Replay {
                // PB3: Block mode holds autoplay after the round's last block too
                if self.hold_at_block_end(cx) {
                    return;
                }
                self.pending_next_round = false;
                self.run(cx, Action::NextRound);
            }
            return;
        }
        if self.playing && ready {
            if self.session().phase == Phase::Replay {
                self.playing = false;
                return;
            }
            // PB3, PB5: after a block the playback waits in Block mode and while the timeline view is open
            if self.hold_at_block_end(cx) {
                return;
            }
            self.send(cx, json!({"cmd": "step"}));
            self.next_at = now + Duration::from_millis(t.step_ms * pace);
            if self.session().phase == Phase::Replay {
                self.playing = false;
                self.round_over(cx, now);
            }
        }
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if self.host.session.is_none() {
            return;
        }
        // ENCOUNTER2-C DT5: the talk box's keys (1-9, Enter, Space) only
        if self.talk.is_some() {
            let pick = self.talk_state_of(cx).and_then(|gs| self.talk_view(cx, &gs)).and_then(|v| talk::talk_key(k, &v));
            if let Some(a) = pick {
                self.act(cx, Action::Talk(a));
            }
            return;
        }
        match k {
            Key::Enter if self.planning() => self.act(cx, Action::Confirm),
            Key::Enter if !self.outcome().is_empty() && self.world_mode() => self.act(cx, Action::Continue),
            Key::Space => self.act(cx, Action::Step),
            Key::PageUp if self.show_log => self.log_scroll += 10,
            Key::PageDown if self.show_log => self.log_scroll = (self.log_scroll - 10).max(0),
            Key::Char('a') => self.act(cx, Action::Autoplay),
            Key::Char('z') => self.act(cx, Action::Undo),
            Key::Char('l') => self.act(cx, Action::ToggleLog),
            Key::Char('i') => {
                let on = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
                cx.settings.v["show_tells"] = serde_json::Value::Bool(!on);
                self.message = match cx.settings.save() {
                    Err(e) => format!("Tells toggled but the setting was not saved: {e}"),
                    Ok(()) if on => "Enemy tells hidden (I shows them).".into(),
                    Ok(()) => "Enemy tells shown (I hides them).".into(),
                };
            }
            Key::Char('r') => self.act(cx, Action::Restart),
            // INPUT1 HK1-HK4: keys 1-9 are the selected hero's skill hotkeys (classic: nothing)
            Key::Char(c @ '1'..='9') if self.planning() => self.hotkey(cx, c as usize - '0' as usize),
            // SL4: S targets the selected hero itself (the self menu)
            Key::Char('s') if self.planning() => self.act(cx, Action::SelfTarget),
            // INPUT1 TV1: T opens and closes the full timeline view (a timeline battle, planning and playback)
            Key::Char('t') => self.act(cx, Action::TimelineView),
            // INPUT1 PB3: P cycles the playback speed (1x, 0.5x, Block), N plays the next block (timeline battles)
            Key::Char('p') => self.act(cx, Action::PlaybackSpeed((PlaySpeed::of(&cx.settings.v).index() + 1) % PLAYBACK_SPEEDS.len())),
            Key::Char('n') => self.act(cx, Action::NextBlock),
            _ => {}
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        if self.talk.is_some() {
            return;
        }
        // INPUT1 IF10, SL2: an open menu only closes; a right click on a unit (token or card) inspects it; elsewhere it
        // undoes the last pick
        if self.popup.is_some() {
            self.popup = None;
        } else if self.show_log {
            self.show_log = false;
        } else if let Some(u) = self.unit_under_mouse() {
            self.inspect_unit(u);
        } else if self.host.session.is_some() && self.planning() {
            self.act(cx, Action::Undo);
        }
    }

    fn escape(&mut self, cx: &mut Cx) {
        if self.talk.is_some() {
            return;
        }
        // INPUT1 SL3 (round intent): the menu, the log, the inspected unit, then the selection; then as before. TV1: an
        // open timeline view closes first
        let timeline = self.host.session.as_ref().is_some_and(|s| ri::on(&s.w));
        if self.tv_open {
            self.close_view(cx);
        } else if self.popup.is_some() {
            self.popup = None;
        } else if self.show_log {
            self.show_log = false;
        } else if self.inspected().is_some() {
            self.inspect = None;
            self.message = "Inspect cleared.".into();
        } else if timeline && self.selected.is_some() {
            self.selected = None;
        } else if !self.world_mode() {
            cx.nav.push(Nav::Pop);
        }
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        if self.show_log {
            self.log_scroll = (self.log_scroll + lines * 3).max(0);
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        let mut out = json!({"message": self.message, "playing": self.playing, "auto": self.auto, "log_open": self.show_log, "world": self.world_mode(),
            "fx": self.fx_state(), "tokens": self.tokens.state(),
            "outcome": if self.host.session.is_some() { self.outcome() } else { String::new() },
            "popup": self.popup.as_ref().map(|p| json!({"title": p.title, "rows": p.rows.iter().map(|r| json!({"label": r.label, "detail": r.detail, "valid": r.valid})).collect::<Vec<_>>()}))});
        // RULESET1-G: the rules the battle plays by, beyond classic (a classic battle's state keeps its shape)
        if let Some(s) = self.host.session.as_ref().filter(|s| !s.w.ruleset.rules.is_empty()) {
            out["ruleset"] = json!({"id": s.w.ruleset.id, "rules": s.w.ruleset.rules, "plan_reveal": s.w.plan_reveal, "melee_diagonal": s.w.ruleset.melee_diagonal().id()});
            // RULESET1-U: the Arena fight's ruleset chip as drawn
            if let Some(chip) = self.arena_chip() {
                out["ruleset"]["chip"] = json!(chip);
            }
        }
        // INPUT1 1.6: each party member's picks (round intent only), the ghosts waiting for Confirm and the player-set
        // targets (GH2)
        if let Some(sess) = self.host.session.as_ref().filter(|x| ri::on(&x.w)) {
            let w = &sess.w;
            out["picks"] = Value::Array(w.allies.iter().map(|&a| json!({"unit": a, "picks": rip::picks_of(w, a).iter().map(|p| pick_json(w, p)).collect::<Vec<_>>()})).collect());
            out["deferred"] = json!(self.deferred.iter().map(|d| d.0).collect::<Vec<_>>());
            out["player_target"] = json!(self.player_target);
            // IS1: the unit the intel panel is pinned on (null: it follows the hover)
            out["inspect"] = json!(self.inspected());
            // INPUT1-AP: the AP gauge as last drawn (pips "F H Hg Sg S", red, its "-<n>" label and side; null: hidden)
            out["ap_gauge"] = self.ap_gauge.clone().unwrap_or(Value::Null);
        }
        // INPUT1 PB1-PB4: the playback aids (timeline battles): the speed, whether the playback waits at a block's end
        // and the resolving block; the live tracers; the log by round and block
        if self.playback_aids() {
            let sess = self.session();
            out["playback"] = json!({"speed": PLAYBACK_SPEEDS[PlaySpeed::of(&cx.settings.v).index()].0, "waiting": self.waiting.is_some(), "block": self.cursor_block(Instant::now())});
            out["log_groups"] = log_groups_json(&sess.log_groups());
        }
        // INPUT1 TV1: the full timeline view as last drawn (null: closed; no key in classic)
        if self.timeline_view_on() {
            out["timeline_view"] = self.tv_shown.as_ref().filter(|_| self.timeline_view_open()).map(timeline_view::json).unwrap_or(Value::Null);
        }
        // RULESET1-U slice 5: the overlay groups' flags and, once the round is over, its summary (round intent only)
        if let Some(sess) = self.host.session.as_ref().filter(|x| ri::on(&x.w)) {
            out["overlay_layers"] = Layers::of(&cx.settings.v["battle_overlays"]).json();
            if sess.phase == Phase::Replay {
                let sum = ri::round_summary(&sess.w, &sess.resolver.scheduled);
                out["round_summary"] = json!({"breaks": sum.breaks, "windup_breaks": sum.windup_breaks, "windup_hits": sum.windup_hits, "flanks": sum.flanks, "follow_ups": sum.follow_ups, "rest_ap": sum.rest_ap});
            }
        }
        // HOOKS1-UI slice 1: the hook marks as drawn, while the battle plays by ground or arena hazards (classic: no key)
        if let Some(sess) = self.host.session.as_ref().filter(|x| emberwake_battle::ground::on(&x.w) || emberwake_battle::arena::on(&x.w)) {
            out["hook_marks"] = HookMarks::of(sess, &Layers::of(&cx.settings.v["battle_overlays"])).json();
        }
        // ENCOUNTER2-U: a hold escape's state (key only in a hold escape)
        if let Some(sess) = self.host.session.as_ref() {
            if let Some(gate) = &sess.w.hold {
                let cell = |c: &V2| json!([c.x, c.y]);
                out["hold"] = json!({"label": gate.label, "tiles": gate.tiles.iter().map(cell).collect::<Vec<_>>(), "holder": gate.holder, "open": gate.is_open(&sess.w),
                    "needs_pick": sess.needs_stay_pick(), "note": sess.gate_note(), "pick": hold_pick_ids(sess)});
            }
        }
        // HOOKS1-UI slice 4: the selected hero's push forecasts as drawn (key only while forced_move is on)
        if let Some(sess) = self.host.session.as_ref().filter(|x| emberwake_battle::forced::on(&x.w)) {
            let layers = Layers::of(&cx.settings.v["battle_overlays"]);
            let planning = sess.phase == Phase::Planning && !sess.plan_confirmed;
            let list: Vec<Value> = match self.selected.filter(|u| sess.w.allies.contains(u) && sess.w.units[*u].is_active()) {
                Some(h) if planning && layers.pushes => push_forecasts(sess, h).into_iter().flatten().map(|f| push_json(&f)).collect(),
                _ => Vec::new(),
            };
            out["push_forecasts"] = Value::Array(list);
        }
        // HOOKS1-UI slice 3: the meters and held guards as drawn (keys only while their rules are on)
        if let Some(sess) = self.host.session.as_ref() {
            let layers = Layers::of(&cx.settings.v["battle_overlays"]);
            if emberwake_battle::meter::on(&sess.w) {
                out["meters"] = Value::Array((0..sess.w.units.len()).filter_map(|i| {
                    let ms = meters_of(&sess.w, i, &layers);
                    (!ms.is_empty()).then(|| json!({"unit": i, "meters": ms.iter().map(|m| json!({"id": m.id, "value": m.value, "min": m.min, "max": m.max, "show": m.show, "band": m.band})).collect::<Vec<_>>()}))
                }).collect());
            }
            if emberwake_battle::trigger::on(&sess.w) {
                out["held_guards"] = Value::Array((0..sess.w.units.len()).filter(|_| layers.guard).filter_map(|i| held_guard(sess, i).map(|g| json!({"unit": i, "mitigation": g.mitigation, "last_block": g.last_block}))).collect());
            }
        }
        // RULESET1-U slice 3: the board overlays under the round intent (classic: no key)
        if let Some(mut o) = self.host.session.as_ref().and_then(|s| overlay_of(s, self.selected.filter(|u| s.w.allies.contains(u)), s.phase == Phase::Planning && !s.plan_confirmed)) {
            o.positions = self.positions_shown.clone();
            o.arrows = self.arrows_shown.clone();
            out["overlay"] = overlay_json(&o);
            // VISCLAR1 VC2: the units whose token bars the last frame drew by default
            out["focus"] = json!(self.focus_shown);
        }
        // INPUT1 PB1: the impact tracers drawing now (any timeline battle)
        if self.playback_aids() {
            if !out["overlay"].is_object() {
                out["overlay"] = json!({});
            }
            out["overlay"]["tracers"] = self.tracers_json(cx);
        }
        // ENCOUNTER2-C4: the scripted round (key only while one plays): its round, lock and banner, and whether Skip shows
        if let Some(rs) = self.host.session.as_ref().and_then(|s| s.round_script()) {
            out["script"] = json!({"round": rs.round, "lock": self.locked(), "banner": rs.banner, "skip": self.can_skip()});
        }
        // ENCOUNTER2-C DT11: the open talk (key only while one is open), its portrait as drawn and the speaker's token
        if let (Some(t), Some(gs)) = (self.talk.as_ref(), self.talk_state_of(cx)) {
            let view = t.runner.view(&cx.data, &gs);
            out["talk"] = talk::talk_state(&t.dialogue, &view);
            out["talk"]["portraits"] = self.faces.state();
            out["talk"]["speaker_unit"] = json!(self.speaker_unit(cx, &gs, &s(&view, "speaker")));
        }
        // PORTRAITFX1-P (4.11): the panel portraits kept (the hero and intel panels'), so a test or an agent can wait
        // for a reaction to end
        out["portraits"] = self.panel_faces.state();
        // PORTRAITFX1-TL: the timeline view's track icons kept (empty while the view is closed: rows react only while it
        // is open and `begin` drops the rest at the next draw)
        out["timeline_portraits"] = self.timeline_faces.state();
        out
    }

    fn command(&mut self, cx: &Cx, body: &Value) -> Option<Value> {
        // An agent-link `new` ends this session: park it for the hub (saved on the next tick), then start a new one.
        let restart = body["cmd"] == json!("new");
        if restart && self.recordable() {
            self.parked.push((self.slot, self.recorded()));
        }
        self.flush(cx);
        self.req_cmd = body["cmd"].as_str().unwrap_or("").to_string();
        self.note_step();
        let reply = self.host.handle(body);
        if restart && reply["ok"] == json!(true) {
            self.new_session();
            self.tokens = Tokens::default();
            self.reset_talks();
        }
        if self.host.session.is_some() {
            self.after_change(cx);
        }
        Some(reply)
    }

    /// MUSIC1: boss when an enemy unit has the boss tier, else combat; scoped to the battle's world and location.
    fn music(&self, cx: &Cx) -> Option<String> {
        let ctx = if self.host.session.as_ref().is_some_and(boss_fight) { music::BOSS } else { music::COMBAT };
        match &self.arena_world {
            Some(w) => music::track(cx, ctx, w, ""),
            None if self.world_mode() => music::game_track(cx, ctx),
            None => music::track(cx, ctx, "", ""),
        }
    }

    aegis_ui::widget_plumbing!();
}

/// ENCOUNTER2-C DT8: the qualified id of world character `r` in `world` ("" when there is none).
fn qid_in(d: &emberwake_world::Data, world: &str, r: &str) -> String {
    s(d.resolve(world, "characters", r), "_qid")
}

/// Whether any enemy unit has the boss tier (MusicDirector.combat_context). Wave and phase units are built when the
/// battle starts (held in reserve), so they count from the first round like Godot's encounter refs.
fn boss_fight(s: &Session) -> bool {
    (0..s.w.units.len()).any(|c| !s.w.allies.contains(&c) && s.tier_of(c) == "boss")
}

/// The sound a resolver log line plays (scripts/audio/sound_events.gd FOR_PRESET); None: silent. The engine's log
/// has no mitigation tag, so a hit for 0 plays mitigated; a status that is a buff plays nothing.
fn sound_of(line: &str) -> Option<&'static str> {
    let tag = line.split_once(']').map(|(t, _)| t.trim_start_matches('['))?;
    match tag {
        "Attack Miss" | "Area Miss" => Some("miss"),
        "Attack" if line.contains(" CRIT") => Some("crit"),
        "Attack" if line.contains(" for 0 ") => Some("mitigated"),
        "Attack" => Some("hit"),
        "Status" => {
            let id = line.split_once("(inflicts ")?.1.split(')').next()?;
            (!emberwake_rules::status::row_of(id).is_some_and(|r| r.buff)).then_some("debuff")
        }
        "Defend" if !line.contains("guard was broken") => Some("defend"),
        "Focus" => Some("buff"),
        "Support" => Some("heal"),
        "Move" | "Move Cut Short" | "Withdrawal" | "Dash" | "Chase" | "Disengage" => Some("move_step"),
        _ => None,
    }
}

// ---- Drawing ------------------------------------------------------------------------------------------------------

pub(crate) fn kind_color(kind: &str) -> Rgba {
    match kind {
        "ATTACK" => rgb(160, 56, 56),
        "MOVE" => rgb(52, 92, 160),
        "DEFEND" => rgb(40, 122, 122),
        "SUPPORT" => rgb(52, 128, 68),
        "FOCUS" => rgb(128, 98, 34),
        "REST" => rgb(74, 80, 96),
        "STATUS" => rgb(104, 56, 128),
        "TARGET" => rgb(92, 76, 120),
        "DISENGAGE" => rgb(56, 116, 160),
        _ => rgb(60, 66, 84),
    }
}

/// WAR2H1 (battle slice): the bodies a unit's token tries, by its art key and its equipped gear: a class base shows
/// the look its grip gives (`1h_buckler`, `2h_<family>`), then its own sheet; a named or variant body keeps its art.
fn body_keys(art: &str, u: &emberwake_rules::unit::Unit) -> Vec<String> {
    emberwake_content::art::gear_body_chain(art, &emberwake_content::art::Grip::of_inventory(&u.inventory))
}

/// The frame a battle token shows (DIR4-G G4): the clip `player` plays at `now` in the unit's `facing`
/// (`UnitAnim::faces`), fitted to a `size` square (`Art::played_facing`). The fallback is the engine's
/// (`Clips::resolve_view`): the clip's `up`/`down` row when the body has it, else its side row, else `idle` in that
/// view, else side `idle`. A side row is mirrored when the facing is left (the rig's sheets face right); an up or down
/// row never is. So a body without view clips draws exactly as FACE1 did (side, mirrored when left), and a hit while
/// facing up plays the side `hurt` with the last left/right, the view returning with the base loop.
fn token_frame<'a>(art: &'a mut Art, body: &str, palette: &str, player: &ClipPlayer, now: u64, size: i32, facing: Facing) -> Option<(ClipFrame, &'a Image)> {
    art.played_facing(body, palette, player, now, size, facing)
}

/// The highlight outline of the token frame `f` (`token_frame` with the same `facing`), mirrored with it.
fn token_outline<'a>(art: &'a mut Art, body: &str, palette: &str, f: &ClipFrame, size: i32, col: Rgba, facing: Facing) -> Option<&'a Image> {
    art.body_outline(body, palette, &f.clip, f.frame, size, col, facing.mirrors(f))
}

/// FX1-L1 (FQ7): the hit flash, the white silhouette of the token frame `f` (`fx::FLASH`, alpha 150) over the sprite
/// square `tr`, mirrored with it; it replaced the white square over the whole cell.
fn hit_flash(p: &mut Painter, art: &mut Art, body: &str, palette: &str, f: &ClipFrame, tr: Rect, facing: Facing) {
    if let Some(img) = art.body_silhouette(body, palette, &f.clip, f.frame, tr.w, fx::FLASH, facing.mirrors(f)) {
        p.image(img, tr);
    }
}

/// A cell's foot point as an effect point (4.1: an area's aim, support motes, falls).
fn foot_point(geo: &Geom, c: V2) -> (f32, f32) {
    let (x, y) = geo.foot(c);
    (x as f32, y as f32)
}

fn foot_points(geo: &Geom, cells: &[V2]) -> Vec<(f32, f32)> {
    cells.iter().map(|&c| foot_point(geo, c)).collect()
}

/// FX1 4.1: an action's effect places in the view `geo` (the anchors of its cells; the Stage's squash 0.75, Flat's 1).
/// FX1-L2: an area's aim is its cell's foot point (4.1); a target further above or below than aside faces the actor up
/// or down (DIR4-G `Facing::toward`), so its swing turns toward it.
fn place(geo: &Geom, from: V2, to: V2, area: bool) -> fx::Place {
    let squash = if geo.view == View::Stage { 0.75 } else { 1.0 };
    let to_pt = if area { foot_point(geo, to) } else { fx::anchor(geo, to) };
    let vertical = (to.y - from.y).abs() > (to.x - from.x).abs();
    fx::Place { from: fx::anchor(geo, from), to: to_pt, squash, vertical }
}

fn hp_bar(p: &mut Painter, r: Rect, hp: i32, max: i32, color: Rgba) {
    p.fill(r, rgb(46, 24, 30));
    let w = if max > 0 { (r.w * hp.max(0) + max - 1) / max } else { 0 };
    p.fill(Rect::new(r.x, r.y, w.min(r.w), r.h), color);
}

fn cell_rect(c: V2) -> Rect {
    Rect::new(GRID_X + c.x * TILE, GRID_Y + c.y * TILE, TILE, TILE)
}

fn short_name(name: &str) -> String {
    name.strip_prefix("Goblin ").unwrap_or(name).to_string()
}

fn stance_name(u: &emberwake_rules::Unit) -> &'static str {
    ["Neutral", "Offensive", "Defensive", "Tactical"][u.stance as usize]
}

/// The actions the resolver holds for `u` this round (after confirm): (start, end, name, kind, target name).
pub(crate) fn committed(s: &Session, u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    action_rows(&s.w, &s.resolver.scheduled, u)
}

/// Unit `u`'s actions among `actions` (this round's, or a past round's from the archive): (start, end, name, kind,
/// target name).
pub(crate) fn action_rows(w: &World, actions: &[emberwake_battle::resolver::ScheduledAction], u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    actions
        .iter()
        .filter(|a| a.unit == u)
        .map(|a| {
            let (kind, target) = match &a.command.kind {
                CmdKind::Attack { target, .. } => ("ATTACK", *target),
                CmdKind::Area { .. } => ("ATTACK", None),
                CmdKind::Defend { .. } => ("DEFEND", None),
                CmdKind::Focus { .. } => ("FOCUS", None),
                CmdKind::Rest { .. } => ("REST", None),
                CmdKind::StatusBlock { .. } => ("STATUS", None),
                CmdKind::Support { target, .. } => ("SUPPORT", Some(*target)),
                CmdKind::Item { target, .. } => ("ITEM", Some(*target)),
                CmdKind::Target { target } => ("TARGET", *target),
                CmdKind::Walk { .. } => ("MOVE", None),
                CmdKind::Disengage { .. } => ("DISENGAGE", None),
            };
            let name = if a.cancelled { format!("{} (cancelled)", a.command.name) } else { a.command.name.clone() };
            (a.start, a.end, name, kind.to_string(), target.filter(|t| *t != u).map(|t| w.units[t].name.clone()))
        })
        .collect()
}

/// A unit's next (or current) action this round, for cards and hover cards.
fn next_action(s: &Session, e: usize) -> String {
    if s.phase == Phase::Replay {
        return "-".into();
    }
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    match s.resolver.scheduled.iter().find(|a| a.unit == e && !a.cancelled && (planning || a.end >= s.current_block)) {
        Some(a) => match a.command.kind {
            CmdKind::Attack { target: Some(t), .. } => format!("{} on {} (B{}-{})", a.command.name, short_name(&s.w.units[t].name), a.start, a.end),
            _ => format!("{} (B{}-{})", a.command.name, a.start, a.end),
        },
        None => "-".into(),
    }
}

/// DERIVE1 4.4: unit `i`'s catalog derived stats shown in battle under the battle's ruleset (none in classic).
fn derived_rows(s: &Session, i: usize) -> Vec<crate::party::DRow> {
    crate::party::data_rows(&s.w.cat, &s.w.ruleset.id, &s.w.units[i], "battle")
}

/// Derived rows in one line (`Composure 26   Grit 3`), None when there are none.
fn derived_line(rows: &[crate::party::DRow]) -> Option<String> {
    (!rows.is_empty()).then(|| rows.iter().map(|r| format!("{} {}", r.label, r.shown)).collect::<Vec<_>>().join("   "))
}

/// HOOKS1 CN2 (HOOKS1-UI slice 2): each unit's class name in world `world` (its catalog row's class: the world's
/// `classes` overlay name, else the class row's; "" for a unit without a class).
fn class_names(d: &emberwake_world::Data, world: &str, w: &World) -> Vec<String> {
    w.units.iter().map(|u| d.cat.units.get(&u.catalog_id).filter(|def| !def.class_id.is_empty() && d.cat.classes.contains_key(&def.class_id)).map(|def| d.class_name(world, &def.class_id)).unwrap_or_default()).collect()
}

/// Hover card of unit `i` (`tells`: the enemy intents while planning, INTENT1).
/// RULESET1-U slice 4 (UI5): `rounds` is every unit's round intent record (`Session::round_intents`; empty in classic):
/// the card leads with the unit's intent and target (either side) and, during the playback, an enemy's record stands in
/// for its tell (so a reveal and its readers stay listed, a reader who fell included).
/// HOOKS1-UI slice 2: `classes[i]` is unit `i`'s class name in the battle's world ("" = none shown; `class_names`).
fn unit_tip(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent], rounds: &[Intent], classes: &[String]) -> Tip {
    unit_tip_as(s, i, sel, tells, rounds, classes, None)
}

/// INPUT1 IS1-IS3: the intel panel's lines for the inspected unit `i`, headed "Inspecting <name> (Esc clears)"
/// (`show_tells`: the Show enemy tells setting, I). An enemy shows only what the party knows (IS2's "Never" list: no
/// unrevealed plan's skills, blocks or walks, deck, order, AI row or profile): the hover card's lines with its facing,
/// without the tell's first action and its forecast while the tells are hidden and without the classic "Next:" line;
/// a party member adds its picks, its plan and its AP numbers (IS3, AP9's lines) while planning under the round intent.
fn inspect_tip(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent], rounds: &[Intent], classes: &[String], show_tells: bool) -> Tip {
    let mut t = unit_tip_as(s, i, sel, tells, rounds, classes, Some(show_tells));
    if let Some(head) = t.first_mut() {
        head.0 = format!("Inspecting {} (Esc clears)", s.w.units[i].name);
    }
    t
}

/// AP9 (INPUT1 7): a party member's AP numbers, one line each: "AP <now> of <max>", "Plan: <spent> (picked <picked>,
/// auto-filled <filled>)", "Left: <left>", "Next round: +<gain> -> <next> (Rest +<rest_gain>)", and with more than one
/// AP a pip (AP6) "1 pip = <u> AP".
pub(crate) fn ap_lines(f: &rip::ApForecast) -> Vec<String> {
    let mut lines = vec![
        format!("AP {} of {}", f.now, f.max),
        format!("Plan: {} (picked {}, auto-filled {})", f.spent, f.picked, f.filled),
        format!("Left: {}", f.left),
        format!("Next round: +{} -> {} (Rest +{})", f.gain, f.next, f.rest_gain),
    ];
    lines.extend(pip_line(f.max));
    lines
}

/// AP6, AP9: "1 pip = <u> AP" when a pip of the AP gauge stands for more than one AP (`max_ap` above `AP_PIPS`).
fn pip_line(max: i32) -> Option<String> {
    let u = aegis_ui::pips::unit(max, AP_PIPS);
    (u > 1).then(|| format!("1 pip = {u} AP"))
}

/// INPUT1-AP (AP6-AP10): the AP gauge of party member `u`. Planning (`f` its forecast, of the hovered row's preview when
/// one shows): kept pips full, the plan's hollow, next round's gain as ghost fills, red with "-<n>" after the pips for
/// a hovered row it cannot pay (`short`); playback: its live AP full, what it paid since `start` hollow. A debt shows
/// "-<n>" before ten slots. Its hover card: AP9's lines (playback: the live AP), "1 pip = <u> AP" above 12 AP.
fn ap_gauge_of(w: &World, u: usize, f: Option<&rip::ApForecast>, short: Option<i32>, start: i32) -> aegis_ui::pips::PipGauge {
    use aegis_ui::pips::PipGauge;
    let unit = &w.units[u];
    match f {
        Some(f) => {
            let g = PipGauge::planned(f.now, f.left, f.next, f.max, AP_PIPS).with_tip(ap_lines(f).into_iter().map(|l| (l, ALLY)).collect());
            match short {
                Some(n) => g.short(n),
                None => g,
            }
        }
        None => {
            let mut tip: Tip = vec![(format!("AP {} of {}", unit.ap, unit.max_ap), ALLY)];
            if start > unit.ap {
                tip.push((format!("Paid this round: {}", start - unit.ap), TEXT));
            }
            tip.extend(pip_line(unit.max_ap).map(|l| (l, TEXT)));
            PipGauge::live(start, unit.ap, unit.max_ap, AP_PIPS).with_tip(tip)
        }
    }
}

/// IS3: an inspected party member's picks and plan: "Picks: <pick>, ..." (or "Picks: none, the planner's round"), then
/// one line a planned entry: "B1-2 Heavy Cleave on Wolf" (", auto-filled" for a ghost entry, GH1); once the round is
/// confirmed its committed actions.
fn plan_lines(s: &Session, u: usize) -> Tip {
    let w = &s.w;
    let target = |t: Option<usize>| t.filter(|t| *t != u).map(|t| format!(" on {}", w.units[t].name)).unwrap_or_default();
    if s.plan_confirmed || s.phase != Phase::Planning {
        return committed(s, u).into_iter().map(|c| (format!("{} {}{}", intent_text::block_text((c.0, c.1)), c.2, c.4.map(|n| format!(" on {n}")).unwrap_or_default()), TEXT)).collect();
    }
    let picks = rip::picks_of(w, u);
    let mut t: Tip = vec![(if picks.is_empty() { "Picks: none, the planner's round".to_string() } else { format!("Picks: {}", picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", ")) }, GOLD)];
    let ghosts = rip::ghost_entries(w, &s.plan, u);
    for (k, e) in s.plan.entries_for(w, u).iter().enumerate() {
        let ghost = ghosts.get(k).copied().unwrap_or(false);
        let line = format!("{} {}{}{}", intent_text::block_text((e.start_block, e.end_block(w))), w.cat.skills[e.skill].name, target(e.target), if ghost { ", auto-filled" } else { "" });
        t.push((line, if ghost { DIM } else { TEXT }));
    }
    t
}

/// IS2: a facing in words (the board's top is north); None for no facing.
fn facing_name(f: V2) -> Option<&'static str> {
    match (f.x.signum(), f.y.signum()) {
        (0, -1) => Some("north"),
        (0, 1) => Some("south"),
        (1, 0) => Some("east"),
        (-1, 0) => Some("west"),
        (1, -1) => Some("north-east"),
        (-1, -1) => Some("north-west"),
        (1, 1) => Some("south-east"),
        (-1, 1) => Some("south-west"),
        _ => None,
    }
}

/// The hover card (`inspect` None) or the inspect lines (`inspect` Some(the Show enemy tells setting), IS2, IS3).
fn unit_tip_as(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent], rounds: &[Intent], classes: &[String], inspect: Option<bool>) -> Tip {
    let w = &s.w;
    let u = &w.units[i];
    let ally = w.allies.contains(&i);
    let tells_shown = inspect.unwrap_or(true);
    let mut t: Tip = vec![(u.name.clone(), if ally { ALLY } else { FOE })];
    let round = rounds.iter().find(|r| r.unit == i);
    let tell = tells.iter().find(|r| r.unit == i).or(round.filter(|_| !ally));
    if let Some((head, intent)) = round.filter(|_| u.is_active()).and_then(|r| Some((intent_text::round_head(w, r)?, r.round_intent?))) {
        t.push((head, intent_ink(intent)));
    }
    t.push((format!("HP {}/{}   AP {}/{}   {}", u.hp, u.max_hp, u.ap, u.max_ap, stance_name(u)), TEXT));
    let level = match classes.get(i).filter(|c| !c.is_empty()) {
        Some(c) if u.tier.is_empty() => format!("Level {} {c}", u.stats.level),
        Some(c) => format!("Level {} {c} ({})", u.stats.level, u.tier),
        None => format!("Level {} {}", u.stats.level, u.tier),
    };
    t.push((format!("{level}   Speed {}   Reach {}", u.speed, reach_text(w, i)), DIM));
    if let Some(line) = derived_line(&derived_rows(s, i)) {
        t.push((line, DIM));
    }
    // IS2: the facing (the round intent's, rule round_intent)
    if let Some(f) = inspect.filter(|_| u.is_active() && ri::on(w)).and_then(|_| facing_name(ri::facing(w, i))) {
        t.push((format!("Facing {f}"), DIM));
    }
    // RULESET1-U slice 5: the poise bar in words (rule poise)
    if let Some(st) = emberwake_battle::poise::state(w, i).filter(|st| st.max > 0 && u.is_active()) {
        t.push((format!("Poise {}/{}{}", st.value, st.max, if st.broken { "  BROKEN this round" } else if st.broken_last { "  (broke last round: stagger x repeat)" } else { "" }), if st.broken { FOE } else { POISE }));
    }
    // HOOKS1-UI slice 3: its meters (UM7; none without the rule, a hidden meter left out) and a held guard (8.3)
    if u.is_active() {
        for m in emberwake_battle::meter::shown(w, i) {
            t.push((meter_text(&m), METER));
        }
    }
    if let Some(g) = held_guard(s, i) {
        t.push((format!("Held guard {:.0}% through B{}", g.mitigation * 100.0, g.last_block), HELD));
    }
    // RULESET1-U slice 3 (GD5): a stealthed unit that moved this round is lost to attacks locked on it
    if ri::hidden(w, i) {
        // good news on a hero, a warning on an enemy
        t.push(("Hidden: an attack locked on it finds nothing this round".into(), if ally { GOOD } else { GOLD }));
    }
    if !u.st.active.is_empty() {
        t.push((format!("Now: {}", u.st.active.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), FOE));
    }
    if !u.st.pending.is_empty() {
        t.push((format!("Next round: {}", u.st.pending.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), GOLD));
    }
    // IS3: an inspected party member's AP numbers, picks and plan (round intent)
    if ally && u.is_active() && inspect.is_some() && ri::on(w) {
        if s.phase == Phase::Planning && !s.plan_confirmed {
            t.extend(ap_lines(&rip::ap_forecast(w, &s.plan, i)).into_iter().map(|l| (l, ALLY)));
        }
        t.extend(plan_lines(s, i));
    }
    if !ally && u.is_active() {
        match tell {
            Some(r) => {
                // IS2: the tell's first action only while the tells show (I)
                if tells_shown {
                    t.push((format!("Intent: {}", intent_text::first_line(w, &r.line)), GOLD));
                }
                // V1, V2: who reads the plan (a reader who fell stays listed), the enrage or the battle's plan reveal
                if let Some(head) = intent_text::reveal_head(w, r, |p| s.w.cat.passives.get(p).map(|x| x.name.clone()).unwrap_or_else(|| p.to_string())) {
                    t.push((head, ALLY));
                    for l in &r.plan {
                        t.push((intent_text::plan_line(w, l), TEXT));
                    }
                }
                if let Some(f) = sel.and_then(|h| intent_text::forecast(r, h)).filter(|_| tells_shown) {
                    t.push((f, FOE));
                }
            }
            // IS2: an inspect never names an unrevealed next action
            None if inspect.is_some() => {}
            None => t.push((format!("Next: {}", next_action(s, i)), GOLD)),
        }
        if let Some(h) = sel.filter(|h| w.units[*h].is_active()) {
            let strike = w.book().lowest(Kind::Attack);
            t.push((format!("{} deals about {} with a Strike, takes about {}", w.units[h].name, expected(w, h, i, strike, None, 0.0, 1.0), expected(w, i, h, strike, None, 0.0, 1.0)), ALLY));
        }
    }
    if !u.is_alive() {
        t.push(("Down".into(), DIM));
    } else if u.is_withdrawn {
        t.push(("Left the battle".into(), DIM));
    }
    t
}

fn draw(g: &mut BattleScreen, cx: &mut Cx, gfx: &mut Gfx, mouse: (i32, i32), live: bool) {
    let now = Instant::now();
    let tuning = cx.tuning.value.clone();
    gfx.canvas.clear(BG);
    // ENCOUNTER2-C DT7: the open talk's game state and view; a new node starts its line and replies at the top
    let talk_gs = g.talk_state_of(cx);
    let talk_view = talk_gs.as_ref().and_then(|gs| g.talk_view(cx, gs));
    let node = match (g.talk.as_ref(), talk_view.as_ref()) {
        (Some(t), Some(v)) => format!("{}/{}", t.dialogue, s(v, "node")),
        _ => String::new(),
    };
    talk::follow_node(gfx, &mut g.talk_node, node);
    g.faces.begin();
    // PORTRAITFX1-P: the panel portraits run on the FX clock (4.5)
    g.panel_faces.begin();
    g.panel_faces.clock = g.clock;
    g.timeline_faces.begin();
    g.timeline_faces.clock = g.clock;
    // DT8: the speaker's token wears the selection ring
    let talker = match (talk_gs.as_ref(), talk_view.as_ref()) {
        (Some(gs), Some(v)) => g.speaker_unit(cx, gs, &s(v, "speaker")),
        _ => None,
    };
    let mut ui: Ui<Action> = Ui::new(gfx, mouse, live);
    g.menu_row = ((ui.p.line_h(BODY) + 2.5).ceil() as i32).max(11);
    let Some(s) = g.host.session.as_ref() else {
        ui.p.text(8.0, 8.0, &g.message, BODY, FOE);
        g.w = ui.widgets;
        return;
    };
    let w = &s.w;
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let sel = g.selected.filter(|u| w.allies.contains(u));
    // INPUT1 PB1-PB5: the playback aids of a timeline battle; PB2: the block the playback resolves
    let aids = g.playback_aids();
    let cursor = g.cursor_block(now);
    // INPUT1 IS1: the inspected unit (the intel panel is pinned on it; its revealed moves show, AR3)
    let pinned = g.inspected();
    // INTENT1: the enemy tells while planning; their chips and arrows on the grid follow the Show enemy tells setting (I).
    // VISCLAR1 VC4: with the debug Party foresight off a foresight reveal reads unrevealed everywhere on the screen
    let foresight_off = party_foresight_off(&cx.settings.v, cfg!(debug_assertions));
    let tells: Vec<Intent> = seen_intents(w, s.enemy_intents(), foresight_off);
    let show_tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
    // RULESET1-U slice 4 (UI5, V5): every unit's round intent record, both sides, planning and playback (empty without
    // the rule plan_badges: classic)
    let rounds: Vec<Intent> = seen_intents(w, s.round_intents(), foresight_off);
    // RULESET1-U slice 5: each overlay group behind its flag (settings `battle_overlays`; all on by default)
    let layers = Layers::of(&cx.settings.v["battle_overlays"]);
    // HOOKS1-UI slice 1: the ground hazards and the arena telegraph (empty in classic)
    let marks = HookMarks::of(s, &layers);
    // ENCOUNTER2-U: a hold escape's tiles and gate (H7; None without a hold escape)
    let hold = HoldMarks::of(s, &layers);
    // HOOKS1-UI slice 2 (CN2): each unit's class name in the battle's world, for the hover cards
    let classes = class_names(&cx.data, &g.class_world, w);
    let round_of = |u: usize| rounds.iter().find(|r| r.unit == u).and_then(|r| r.round_intent).filter(|_| layers.glyphs);

    // -- Top strip: the selected hero's six blocks and the round controls.
    g.panels = vec![("strip", Rect::new(2, 2, W - 4, 24)), ("grid", Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4)), ("bar", Rect::new(2, 322, W - 4, 36))];
    ui.p.panel(Rect::new(2, 2, W - 4, 24), PANEL, BORDER, RADIUS);
    let hero_name = sel.map(|u| w.units[u].name.clone()).unwrap_or_else(|| "No hero".into());
    // UI5: the strip row leads with the hero's intent glyph (the board and the cards share it)
    let title_w = match sel.and_then(round_of) {
        Some(i) => {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(78, 8, 10, 10));
            68.0
        }
        None => 80.0,
    };
    ui.p.text_fit(8.0, 9.0, &format!("{hero_name} plan"), TITLE, title_w, GOLD);
    ui.push("strip.title", "label", format!("{hero_name} plan"), Rect::new(8, 8, title_w as i32, ui.p.line_h(TITLE).ceil() as i32), true, false, None, Vec::new());
    // HOOKS1-UI slice 3 (UM7): the selected hero's meters on a thin row under the title
    if let Some(u) = sel {
        let ms = meters_of(w, u, &layers);
        if !ms.is_empty() {
            let mr = Rect::new(8, 21, 76, 3);
            meter_row(&mut ui.p, mr, &ms);
            let tip: Tip = ms.iter().map(|m| (meter_text(m), METER)).collect();
            ui.push("strip.meters", "meters", tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>().join("; "), Rect::new(mr.x, mr.y - 1, mr.w, mr.h + 2), true, false, None, tip);
        }
    }
    // INPUT1 IF5: a hovered skill-menu row previews the round as if picked (copies of the world and the plan, kept for
    // the battle's revision); GH2 a: a member whose ghost waits for Confirm shows it from copies too
    let timeline = ri::on(w);
    let hovered_row = g.popup.as_ref().filter(|p| p.menu.is_some() && live).and_then(|p| (0..p.rows.len()).find(|&k| p.row(g.menu_row, k).contains(mouse.0, mouse.1)));
    let hovered_pick = g.popup.as_ref().and_then(|p| Some((p.menu?, p.rows.get(hovered_row?)?.clone())));
    if let Some(((h, at), row)) = hovered_pick.clone().filter(|_| planning) {
        let k = hovered_row.unwrap_or(0);
        if g.preview.as_ref().is_none_or(|p| (p.0, p.1) != (g.rev, k)) {
            let view = match &row.act {
                RowAct::Pick(picks) if row.valid => rip::preview_plan(w, &s.plan, h, at, picks).ok(),
                _ => None,
            };
            g.preview = Some((g.rev, k, view));
        }
    } else {
        g.preview = None;
    }
    let deferred_req = sel.and_then(|u| g.deferred.iter().find(|d| d.0 == u)).map(|d| (d.0, d.1.clone()));
    match (&deferred_req, planning) {
        (Some((m, change)), true) => {
            if g.deferred_view.as_ref().is_none_or(|v| (v.0, v.1) != (g.rev, *m)) {
                g.deferred_view = Some((g.rev, *m, change.view(s, *m)));
            }
        }
        _ => g.deferred_view = None,
    }
    let shown: Option<(&World, &emberwake_battle::plan::Plan)> = match (&g.preview, &g.deferred_view) {
        (Some((_, _, Some((pw, pp)))), _) => Some((pw, pp)),
        (_, Some((_, _, Some((pw, pp))))) => Some((pw, pp)),
        _ => None,
    };
    let (pw, pplan) = shown.unwrap_or((w, &s.plan));
    let previewing = shown.is_some();
    // INPUT1-AP (AP5, AP10): the selected party member's AP gauge, under the round intent only; while planning each
    // member's AP is kept for the playback's hollow pips
    if planning {
        g.round_ap = (s.current_round, w.allies.iter().map(|&a| (a, w.units[a].ap)).collect());
    }
    let gauge = match sel {
        Some(u) if timeline && w.units[u].is_active() => {
            let f = planning.then(|| rip::ap_forecast(pw, pplan, u));
            // AP3: a hovered row the hero cannot pay: the kit's shortfall for it (IF6 d)
            let short = hovered_pick.as_ref().filter(|((h, _), row)| planning && *h == u && !row.valid).and_then(|(_, row)| row.ap_short);
            let start = g.round_ap.1.iter().find(|r| r.0 == u && g.round_ap.0 == s.current_round).map_or(w.units[u].ap, |r| r.1);
            let mut gauge = ap_gauge_of(w, u, f.as_ref(), short, start);
            // AP10: paid pips go hollow and regained ones fill over AP_ANIM_MS during playback and into the next
            // round's planning; changes while planning (picks, hovers) and the switch to playback show at once
            let target = (gauge.fill, gauge.edge);
            let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false) || cx.settings.reduce_motion();
            let anim = match g.ap_anim {
                Some(a) if a.unit == u && a.to == target && a.planning == planning => a,
                Some(a) if a.unit == u && !instant && ((!planning && !a.planning) || (planning && a.round != s.current_round)) => ApAnim { unit: u, round: s.current_round, planning, from: a.level(now), to: target, at: now },
                _ => ApAnim { unit: u, round: s.current_round, planning, from: target, to: target, at: now },
            };
            g.ap_anim = Some(anim);
            (gauge.fill, gauge.edge) = anim.level(now);
            gauge.ghost = if planning { gauge.ghost.max(gauge.fill) } else { gauge.fill };
            Some(gauge)
        }
        _ => None,
    };
    let entries: Vec<(i32, i32, String, String, Option<String>, i32, i32)> = match sel {
        Some(u) if s.plan_confirmed => committed(s, u).into_iter().map(|c| (c.0, c.1, c.2, c.3, c.4, 0, 0)).collect(),
        Some(u) if w.units[u].is_active() => pplan
            .compile(pw, u)
            .entries
            .iter()
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                (e.start_block, e.end_block(pw), sk.name.clone(), sk.kind.clone(), e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), e.total_ap(), e.est_damage)
            })
            .collect(),
        _ => Vec::new(),
    };
    // RULESET1-U (intent UI2, PL6): under the round intent the planned blocks carry the planner's reason tags and the
    // Rest blocks the round's Rest gain; INPUT1 GH1: which are ghost (the planner's, after the picks)
    let (reasons, rest, ghosts) = match sel {
        Some(u) if timeline && !s.plan_confirmed && w.units[u].is_active() => (rip::reasons(pw, pplan, u), Some(rip::rest_preview(pw, pplan, u)), rip::ghost_entries(pw, pplan, u)),
        _ => (Vec::new(), None, Vec::new()),
    };
    // RULESET1-U slice 5: each span's wind-up, impact and recovery (and a break), the planned hits' forecasts (the
    // battle's own plan: a preview draws its spans plain)
    let spans: Vec<SpanParts> = match sel {
        Some(u) if timeline && layers.parts && !previewing => span_parts(s, u, &entries),
        _ => Vec::new(),
    };
    let hits: Vec<Option<HitForecast>> = match sel {
        Some(u) if timeline && layers.forecasts && !s.plan_confirmed && w.units[u].is_active() && !previewing => hit_forecasts(s, u),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 3 review: each span's HP cost from its own skill (the plan entry's) or, once confirmed, its command
    let costs: Vec<i32> = match sel {
        Some(u) if s.plan_confirmed => s.resolver.scheduled.iter().filter(|a| a.unit == u).map(|a| a.command.hp_cost).collect(),
        Some(u) if w.units[u].is_active() => pplan.compile(pw, u).entries.iter().map(|e| emberwake_battle::command::hp_cost_of(pw, &w.cat.skills[e.skill])).collect(),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 4: each planned entry's push and pull forecasts (rule forced_move; planning, layer `pushes`)
    let pushes: Vec<Vec<emberwake_battle::forced::Forecast>> = match sel {
        Some(u) if planning && layers.pushes && w.allies.contains(&u) && w.units[u].is_active() && !previewing => push_forecasts(s, u),
        _ => Vec::new(),
    };
    // TEXTFIT1: the controls right of the blocks take their text's width; the blocks give up width first (34 down to
    // 26 each), then the two dropdowns, the wider first (their text cuts).
    let party_label = format!("Party: {}", TACTIC_LABELS[g.party_tactic]);
    let hero_label = format!("Hero: {}", TACTIC_LABELS[sel.map_or(g.hero_tactic, |u| if timeline { g.tactic_of(u) } else { g.hero_tactic })]);
    // ENCOUNTER2-C4: a locked round still to play: Skip in Confirm's place (the rest of the round at once)
    let skip = g.can_skip();
    let (label, action, enabled) = if planning {
        // ENCOUNTER2 H1: planning cannot be confirmed before the holder is picked
        ("Confirm round", Action::Confirm, !s.needs_stay_pick())
    } else if skip {
        ("Skip", Action::Skip, true)
    } else if s.phase == Phase::Replay && s.can_advance_round() {
        ("Next round", Action::NextRound, !g.pending_next_round)
    } else {
        (if s.combat_over { "Battle over" } else if s.phase == Phase::Replay { "Round limit" } else { "Resolving" }, Action::NextRound, false)
    };
    let mut fit_w = |text: &str, pad: f32, min: i32| ((ui.p.text_w(text, BODY) + pad).ceil() as i32).max(min);
    let (mut party_w, mut hero_w) = (fit_w(&party_label, 18.0, 82), fit_w(&hero_label, 18.0, 80));
    let (auto_w, clear_w, confirm_w) = (fit_w("Auto", 10.0, 34), fit_w("Clear", 10.0, 34), fit_w(label, 12.0, 60));
    let bx = 92;
    let blocks = s.def.total_blocks.max(1);
    // INPUT1-AP (AP6): under the round intent the blocks leave the AP gauge its 12 pips and a label ("-<n>") at their
    // end, kept whether or not a hero is selected, so the strip does not shift
    let gauge_w = if timeline { aegis_ui::pips::pips_w(AP_PIPS) + aegis_ui::pips::LABEL_GAP + aegis_ui::pips::label_w(&mut ui, "-00") } else { 0 };
    let room = W - 6 - bx - 6 - 8 - auto_w - clear_w - confirm_w - if timeline { gauge_w + 2 } else { 0 };
    // AP6: with the gauge the blocks give up width down to 20 each (their span labels cut, TEXTFIT1) before the menus
    let bw = ((room - party_w - hero_w) / blocks).clamp(if timeline { 20 } else { 26 }, 34);
    let mut over = blocks * bw + party_w + hero_w - room;
    while over > 0 && (party_w > 82 || hero_w > 80) {
        let (big, floor) = if party_w - 82 >= hero_w - 80 { (&mut party_w, 82) } else { (&mut hero_w, 80) };
        let cut = over.min(*big - floor);
        *big -= cut;
        over -= cut;
    }
    // INPUT1 PB2: a timeline battle's strip follows the playback's resolving block (classic: the current block)
    let live_block = if aids { cursor.unwrap_or(0) } else if !planning && s.phase == Phase::Resolving { s.current_block } else { 0 };
    for b in 1..=s.def.total_blocks {
        let r = Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16);
        let at = entries.iter().position(|e| b >= e.0 && b <= e.1);
        let entry = at.map(|i| &entries[i]);
        let mut tip: Tip = Vec::new();
        let ghost = at.and_then(|i| ghosts.get(i)).copied().unwrap_or(false);
        let why = at.and_then(|i| reasons.get(i)).filter(|r| !r.is_empty());
        match entry {
            Some(e) => {
                // INPUT1 GH8: a ghost block's tip starts "Auto-filled" with its reason tag
                if ghost {
                    tip.push((why.map_or_else(|| "Auto-filled".to_string(), |r| format!("Auto-filled: {r}")), DIM));
                }
                tip.push((e.2.clone(), GOLD));
                tip.push((format!("Blocks {}-{}", e.0, e.1), TEXT));
                if let Some(t) = &e.4 {
                    tip.push((format!("Target: {t}"), FOE));
                }
                if e.5 > 0 {
                    tip.push((format!("{} AP", e.5), ALLY));
                }
                if e.6 > 0 {
                    tip.push((format!("Expected damage about {}", e.6), GOLD));
                }
                // HOOKS1-UI slice 3 (HC1): the step's HP cost
                let cost = at.and_then(|i| costs.get(i)).copied().unwrap_or(0);
                if cost > 0 {
                    tip.push((intent_text::hp_cost_text(cost), FOE));
                }
                if let Some(why) = why.filter(|_| !ghost) {
                    tip.push((format!("Why: {why}"), DIM));
                }
                if let Some(gain) = rest.filter(|r| e.3 == "REST" && r.blocks > 0) {
                    tip.push((format!("{} ({} Rest block{}; the AP comes at round end)", rest_text(gain), gain.blocks, if gain.blocks == 1 { "" } else { "s" }), GOOD));
                }
                for f in at.and_then(|i| pushes.get(i)).into_iter().flatten() {
                    tip.push(push_text(w, f));
                }
                if let Some(f) = at.and_then(|i| hits.get(i)).and_then(|f| f.as_ref()) {
                    tip.extend(f.lines(w));
                }
                if at.and_then(|i| spans.get(i)).is_some_and(|sp| sp.broken) {
                    tip.push(("BREAK: cancelled by a poise break".into(), FOE));
                }
            }
            None => {
                ui.p.panel(r, rgba(14, 20, 34, 255), if live_block == b { GOLD } else { BORDER }, RADIUS);
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 4.0, &format!("{b} Idle"), SMALL, DIM);
                tip.push((format!("Block {b}: nothing planned"), DIM));
            }
        }
        // RULESET1-U slice 6 (TEXTFIT1): a span's first block holds its whole span, so its label (drawn across the span)
        // sits in its own widget; the later blocks, pushed after it, keep their own hover cards
        let wr = match entry {
            Some(e) if e.0 == b => Rect::new(r.x, r.y, (e.1 - e.0 + 1) * bw - 1, r.h),
            _ => r,
        };
        // HOOKS1-UI slice 1 (AH3): the arena hazards that strike at the block's end
        tip.extend(marks.block_tip(b));
        ui.push(format!("strip.block{b}"), "block", entry.map(|e| e.2.clone()).unwrap_or_else(|| "empty".into()), wr, true, live_block == b, None, tip);
    }
    // Each action is one continuous span over its blocks, coloured by kind (COMBAT-UI3).
    for (k, e) in entries.iter().enumerate() {
        let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
        let live = live_block >= e.0 && live_block <= e.1;
        // RULESET1-U (UI2): under the round intent Rest blocks are dim (the planner fills every spare block with them)
        let dim = timeline && e.3 == "REST";
        // INPUT1 GH8: a ghost span (the planner's, after the picks) in the engine's ghost look; a ghost Rest stays dim
        let ghost = ghosts.get(k).copied().unwrap_or(false) && !dim;
        let fill =if dim { rgb(34, 40, 54) } else if ghost { aegis_ui::timeline::ghost_fill(kind_color(&e.3)) } else { kind_color(&e.3) };
        ui.p.panel(r, fill, if live { GOLD } else if ghost { rgba(0, 0, 0, 0) } else { rgba(255, 255, 255, if dim { 24 } else { 50 }) }, RADIUS);
        if ghost {
            aegis_ui::timeline::draw_ghost(&mut ui, r);
        }
        // RULESET1-U slice 5: the timeline widget's look: wind-up shaded, impact bright with its edge, recovery hatched
        let sp = spans.get(k);
        for part in sp.map(|sp| sp.parts.as_slice()).unwrap_or(&[]) {
            let pr = Rect::new(bx + (part.start - 1) * bw, 6, (part.end - part.start + 1) * bw - 1, 16);
            aegis_ui::timeline::draw_part(&mut ui, part.style, r, pr);
        }
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 4.0, &e.2, SMALL, r.w as f32 - 6.0, if dim || ghost { DIM } else { TEXT });
        if sp.is_some_and(|sp| sp.broken) && layers.breaks {
            let mark = aegis_ui::timeline::Mark { key: "break".into(), block: e.0, glyph: "BREAK".into(), colour: FOE, tip: vec![("BREAK".into(), FOE), (format!("{}: cancelled by a poise break (AP paid stays spent)", e.2), TEXT)] };
            let mr = Rect::new(r.x + 1, 7, (bw - 3).min(r.w - 2), 14);
            aegis_ui::timeline::draw_mark(&mut ui, &format!("strip.mark.{k}"), &mark, mr);
        }
    }
    // HOOKS1-UI slice 1 (AH3): a badge on each block at whose end an arena hazard strikes, over the spans
    for b in (1..=s.def.total_blocks).filter(|b| marks.arena.iter().any(|t| t.blocks.contains(b))) {
        arena_badge(&mut ui.p, Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16));
    }
    // INPUT1 PB2: a 1 px GOLD frame around the resolving block's column, over the spans
    if let Some(b) = cursor.filter(|b| (1..=blocks).contains(b)) {
        let c = Rect::new(bx + (b - 1) * bw - 1, 5, bw + 1, 18);
        for edge in [Rect::new(c.x, c.y, c.w, 1), Rect::new(c.x, c.y + c.h - 1, c.w, 1), Rect::new(c.x, c.y, 1, c.h), Rect::new(c.x + c.w - 1, c.y, 1, c.h)] {
            ui.p.fill(edge, GOLD);
        }
    }
    let mut x = bx + blocks * bw + 6;
    // AP6: the gauge (`strip.ap`) after the last block, before the tactic menus, centred on the strip lane
    if timeline {
        let gx = bx + blocks * bw + 4;
        g.ap_gauge = gauge.map(|ga| {
            ga.draw(&mut ui, "strip.ap", gx, 14);
            let label = ga.before.clone().map(|l| (l, "before")).or(ga.after.clone().map(|l| (l, "after")));
            json!({"unit": sel, "pips": ga.reading(), "red": ga.alert, "label": label.as_ref().map(|l| l.0.clone()), "label_side": label.map(|l| l.1), "per_pip": ga.unit, "slots": ga.slots})
        });
        x = gx + gauge_w + 4;
    } else {
        g.ap_gauge = None;
    }
    if aids && !planning {
        // INPUT1 PB3: during playback the speed chips take the tactic menus' place (1x, 0.5x, Block; Next block in
        // Block mode)
        let end = x + party_w + 2 + hero_w;
        let speed = PlaySpeed::of(&cx.settings.v);
        let tips = [
            "Playback at full speed (P cycles the speeds).",
            "Half speed: every walk, clip, float, tracer and pause takes twice as long (P cycles).",
            "Block by block: the playback waits after each block; N or Next block plays the next one (P cycles).",
        ];
        for (k, (_, label, id)) in PLAYBACK_SPEEDS.iter().enumerate() {
            let cw = ((ui.p.text_w(label, BODY) + 10.0).ceil() as i32).max(18);
            ui.button_full(id, Rect::new(x, 6, cw, 16), label, Action::PlaybackSpeed(k), true, speed.index() == k, BODY, vec![(format!("Playback speed: {label}"), GOLD), (tips[k].into(), TEXT)]);
            x += cw + 2;
        }
        if speed == PlaySpeed::Block {
            ui.button_full("playback.next", Rect::new(x, 6, (end - x).max(10), 16), "Next block", Action::NextBlock, g.waiting.is_some(), false, BODY, vec![("Next block (N)".into(), GOLD), ("Plays the next block, then waits again.".into(), TEXT)]);
        }
        x = end + 2;
    } else {
        ui.dropdown("strip.party", Rect::new(x, 6, party_w, 16), &party_label, Action::PartyMenu, planning, vec![("Party tactic".into(), GOLD), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), TEXT)]);
        x += party_w + 2;
        ui.dropdown("strip.hero", Rect::new(x, 6, hero_w, 16), &hero_label, Action::HeroMenu, planning && sel.is_some(), vec![("Hero tactic".into(), GOLD), ("AI-drafts the selected hero only.".into(), TEXT)]);
        x += hero_w + 2;
    }
    ui.button_full("strip.auto", Rect::new(x, 6, auto_w, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto, BODY, vec![("Autoplay".into(), GOLD), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), TEXT)]);
    x += auto_w + 2;
    ui.button_full("strip.clear", Rect::new(x, 6, clear_w, 16), "Clear", Action::Clear, planning && sel.is_some(), false, BODY, vec![(if timeline { "Clears the selected hero's picks (the planner fills the round again)." } else { "Clears the selected hero's steps." }.into(), TEXT)]);
    x += clear_w + 2;
    if skip {
        ui.button_full("battle.skip", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, true, BODY, vec![("Skip".into(), GOLD), ("Resolves the rest of this scripted round at once.".into(), TEXT)]);
    } else {
        ui.button_full("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, planning, BODY, vec![(label.into(), GOLD)]);
    }

    // -- Left: party cards and the selected hero.
    ui.p.panel(Rect::new(2, 28, 124, 290), PANEL, BORDER, RADIUS);
    let alive = w.allies.iter().filter(|a| w.units[**a].is_active()).count();
    ui.p.text(8.0, 32.0, &format!("Party {alive}/{}", w.allies.len()), TITLE, ALLY);
    let mut y = 44;
    // UI5: the party member the mouse is over (its card or its token) fills the threat intel panel
    let mut hovered_ally = None;
    for &a in &w.allies {
        let u = &w.units[a];
        // HOOKS1-UI slice 3: a meter row grows the card by 4 (UM7)
        let ms = meters_of(w, a, &layers);
        let r = Rect::new(6, y, 116, if ms.is_empty() { 30 } else { 34 });
        let is_sel = Some(a) == sel;
        let hover = ui.hover(r);
        if hover {
            hovered_ally = Some(a);
        }
        ui.p.panel(r, if is_sel { PANEL_HI } else if hover { rgba(28, 40, 66, 255) } else { CARD }, if is_sel { GOLD } else { BORDER }, RADIUS);
        // UI5: the member's intent glyph at the card's top right (the name gives way)
        let glyph = round_of(a).filter(|_| u.is_active());
        if let Some(i) = glyph {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(r.x + r.w - 13, r.y + 2, 10, 10));
        }
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 1.5, &u.name, BODY, if glyph.is_some() { 94.0 } else { 108.0 }, if u.is_active() { ALLY } else { DIM });
        hp_bar(&mut ui.p, Rect::new(r.x + 4, r.y + 12, 108, 4), u.hp, u.max_hp, GOOD);
        // RULESET1-U slice 5: the member's poise bar under its HP bar (rule poise)
        if let Some(st) = poise_shown(w, a, &layers) {
            poise_bar(&mut ui.p, Rect::new(r.x + 4, r.y + 16, 108, 2), &st);
        }
        ui.p.text(r.x as f32 + 4.0, r.y as f32 + 18.0, &format!("HP {}/{}", u.hp, u.max_hp), SMALL, TEXT);
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 18.0, &format!("AP {}/{}", u.ap, u.max_ap), SMALL, ALLY);
        meter_row(&mut ui.p, Rect::new(r.x + 4, r.y + 29, 108, 3), &ms);
        if layers.guard && held_guard(s, a).is_some() {
            ui.p.panel(r.inset(1), rgba(0, 0, 0, 0), HELD, RADIUS);
        }
        let tip = unit_tip(s, a, sel, &tells, &rounds, &classes);
        ui.push(format!("party.{a}"), "card", format!("{} HP {}/{} AP {}/{}", u.name, u.hp, u.max_hp, u.ap, u.max_ap), r, true, is_sel, Some(Action::Select(a)), tip);
        y += r.h + 3;
    }
    g.panels.push(("party", Rect::new(2, 28, 124, y - 26)));
    if let Some(h) = sel {
        let u = &w.units[h];
        let r = Rect::new(6, y + 2, 116, 318 - y - 6);
        g.panels.push(("hero", r));
        ui.p.panel(r, CARD, BORDER, RADIUS);
        // PORTRAITFX1-P (6.2 row 3): the hero's portrait at the panel's top-left; the name, the corner and the Self
        // button beside it, 40 px in
        let (who, look) = g.unit_portrait(cx, h);
        let face = panel_portrait(&mut ui, cx, &mut g.panel_faces, &who, &look, r, "hero.portrait");
        let (tx, tw) = (r.x + PANEL_TEXT_X, (r.w - PANEL_TEXT_X - 4) as f32);
        ui.p.text_fit(tx as f32, r.y as f32 + 2.0, &u.name, BODY, tw, GOLD);
        // RULESET1-U: under the round intent the corner names the hero's intent (the classic stance is inert: IN3);
        // PORTRAITFX1-P: on the line under the name
        let corner = if timeline { ri::intent_of(pw, h).map_or_else(|| "No intent".to_string(), |i| i.label(&pw.cat)) } else { stance_name(u).to_string() };
        let row2 = r.y + 2 + (ui.p.line_h(BODY).ceil() as i32).max(10);
        ui.p.text_fit(tx as f32, row2 as f32, &corner, SMALL, tw, DIM);
        let row3 = row2 + (ui.p.line_h(SMALL).ceil() as i32).max(8) + 1;
        // INPUT1 IF8, SL4: under the round intent the panel's one control is the Self button (key S)
        let top = if timeline {
            let tip = vec![("Self (S)".into(), GOLD), (format!("{} targets itself: its Guard and self skills.", u.name), TEXT)];
            let bw = ((ui.p.text_w("Self", SMALL) + 10.0).ceil() as i32).max(30);
            ui.button_full("hero.self", Rect::new(tx, row3, bw, 11), "Self", Action::SelfTarget, planning, false, SMALL, tip);
            row3 + 13
        } else {
            row3
        };
        let st = pplan.compile(pw, h);
        let weapon = u.inventory.main_weapon().map(|e| e.item.name.clone()).unwrap_or_else(|| "Bare hands".into());
        let mut lines: Vec<(String, Rgba)> = Vec::new();
        // DERIVE1 4.4: the derived-stat line hovers to each row's formula and breakdown
        let derived = derived_rows(s, h);
        let mut derived_at = None;
        let mut stats = |lines: &mut Vec<(String, Rgba)>| {
            lines.push((format!("Speed {}  Reach {}", u.speed, reach_text(w, h)), TEXT));
            if let Some(line) = derived_line(&derived) {
                derived_at = Some(lines.len());
                lines.push((line, TEXT));
            }
        };
        let statuses = |lines: &mut Vec<(String, Rgba)>| {
            if !u.st.active.is_empty() {
                lines.push((format!("Now: {}", u.st.active.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), FOE));
            }
            if !u.st.pending.is_empty() {
                lines.push((format!("Next: {}", u.st.pending.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), DIM));
            }
        };
        let ap_line = (format!("AP left {}/{}", if s.plan_confirmed { u.ap } else { st.ap_left }, u.max_ap), ALLY);
        if timeline {
            // INPUT1 IF8: the round first: "Intent: <Intent> (<kind>) from <first pick | tactic>" (or carried over), a
            // ghost's intent change that waits for Confirm (IN6), the target, the picks, the planned commands without
            // the Rest blocks (the ghost dim), then the Rest gain; the stats after.
            let intent = ri::intent_of(pw, h);
            let picks = rip::picks_of(w, h);
            let from = match picks.iter().find(|p| p.holds_target()) {
                Some(p) => format!(" from {}", rip::pick_name(w, p)),
                None if picks.is_empty() && !g.player_target.contains(&h) && !rip::manual(w, h) => format!(" from {}", TACTIC_LABELS[g.tactic_of(h)]),
                None => " (carried over)".into(),
            };
            match intent {
                Some(i) => {
                    let kind = if i == RoundIntent::Maneuver { pw.round_states.get(&h).and_then(|r| r.maneuver).map(|k| format!(" ({})", kind_name(k))).unwrap_or_default() } else { String::new() };
                    lines.push((format!("Intent: {}{kind}{from}", i.label(&pw.cat)), TEXT));
                }
                None => lines.push(("Intent: none yet (click a target)".into(), GOLD)),
            }
            if deferred_req.is_some() && planning {
                let drops = intent.map(|i| ri::would_drop(w, h, i)).unwrap_or_default();
                if !drops.is_empty() {
                    lines.push((format!("Changing loses {}.", drops.join(", ")), FOE));
                }
            }
            match pw.round_states.get(&h).and_then(|r| r.target) {
                Some(t) => lines.push((format!("Target: {}", target_name(pw, t)), TEXT)),
                None => lines.push(("Target: click the board".into(), DIM)),
            }
            if !picks.is_empty() && !s.plan_confirmed {
                lines.push((format!("Picks: {}", picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", ")), GOLD));
            }
            if s.plan_confirmed {
                for (start, _, name, _, tgt) in committed(s, h) {
                    let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                    lines.push((format!("B{start} {name}{tgt}"), TEXT));
                }
            } else {
                for (k, e) in st.entries.iter().enumerate().filter(|(_, e)| e.step_index >= 0 && emberwake_battle::skill::kind(&w.cat.skills[e.skill]) != Kind::Rest) {
                    let name = &w.cat.skills[e.skill].name;
                    let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                    lines.push((format!("B{} {name}{tgt}", e.start_block), if ghosts.get(k).copied().unwrap_or(false) { DIM } else { TEXT }));
                }
                if let Some(gain) = rest.filter(|r| r.blocks > 0) {
                    lines.push((rest_text(gain), GOOD));
                }
                let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(pw))).map(|b| b.to_string()).collect();
                if !empty.is_empty() && intent.is_some() && u.is_active() {
                    lines.push((format!("Empty: B{}", empty.join(", B")), GOLD));
                }
                if st.error_index >= 0 {
                    lines.push((st.error_text.clone(), FOE));
                }
            }
            // ENCOUNTER2 H3: why a Withdraw waits (the gate is shut)
            let note = s.gate_note();
            if !note.is_empty() && !s.plan_confirmed {
                lines.push((note, FOE));
            }
            lines.push(ap_line);
            statuses(&mut lines);
            stats(&mut lines);
            lines.push((format!("Gear: {weapon}"), GOLD));
        } else {
            stats(&mut lines);
            lines.push(ap_line);
            lines.push((format!("Gear: {weapon}"), GOLD));
            statuses(&mut lines);
            if s.plan_confirmed {
                lines.push(("This round".into(), TEXT));
                for (start, _, name, _, tgt) in committed(s, h) {
                    let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                    lines.push((format!("B{start} {name}{tgt}"), TEXT));
                }
            } else {
                lines.push(("Plan".into(), TEXT));
                // ENCOUNTER2 H3: why a Withdraw waits (the gate is shut)
                let note = s.gate_note();
                if !note.is_empty() {
                    lines.push((note, FOE));
                }
                if st.entries.iter().all(|e| e.step_index < 0) {
                    lines.push(("No plan yet".into(), GOLD));
                }
                for e in st.entries.iter().filter(|e| e.step_index >= 0) {
                    let name = &w.cat.skills[e.skill].name;
                    let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                    lines.push((format!("B{} {name}{tgt}", e.start_block), TEXT));
                }
                let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(w))).map(|b| b.to_string()).collect();
                if !empty.is_empty() && u.is_active() {
                    lines.push((format!("Empty: B{}", empty.join(", B")), GOLD));
                }
                if st.error_index >= 0 {
                    lines.push((st.error_text.clone(), FOE));
                }
            }
        }
        let mut ly = top as f32;
        // RULESET1-U slice 6 (TEXTFIT1): the line pitch grows with the text size (8 at Small and Normal, as before)
        let pitch = ui.p.line_h(SMALL).max(8.0);
        for (n, (text, col)) in lines.into_iter().enumerate() {
            // PORTRAITFX1-P: a line whose first row fits beside the portrait starts 40 px in (and wraps to the room
            // left); the first that does not starts under the portrait
            let below = (face.y + face.h + 1) as f32;
            let beside = ly + pitch <= below;
            if !beside && ly < below {
                ly = below;
            }
            let top = ly;
            let (lx, lw) = if beside { (tx as f32, tw) } else { (r.x as f32 + 4.0, 108.0) };
            for part in ui.p.wrap(&text, SMALL, lw) {
                if ly > (r.y + r.h) as f32 - pitch - 1.0 {
                    break;
                }
                ui.p.text(lx, ly, &part, SMALL, col);
                ly += pitch;
            }
            if Some(n) == derived_at && !derived.is_empty() && ly > top {
                let tip: Tip = derived.iter().flat_map(|d| [(format!("{} {}", d.label, d.shown), GOLD), (d.desc.clone(), TEXT)]).collect();
                ui.push("hero.derived", "row", text, Rect::new(r.x + 2, top as i32 - 1, r.w - 4, (ly - top).ceil() as i32), true, false, None, tip);
            }
        }
        ui.push("hero.panel", "panel", u.name.clone(), r, true, false, None, Vec::new());
    }

    // -- Grid (pixel layer): STAGE1 Stage (the tilted board) or Flat (the board seen from above, as before) by the
    // Battle view setting; both use the same tokens, actions, widget names and HUD.
    let view = View::of_setting(&cx.settings.v["battle_view"]);
    // 3.6 / 3.9: a unit hovered through its party or hostile card (the cards of the last frame) is hovered on the board.
    let card_hover = if live && g.popup.is_none() {
        g.w.iter().filter(|x| x.rect.contains(mouse.0, mouse.1)).find_map(|x| x.id.strip_prefix("party.").or_else(|| x.id.strip_prefix("foe.")).and_then(|n| n.parse::<usize>().ok()))
    } else {
        None
    };
    // FX1 4.1: the camera's shake shifts the board's passes (inside its clip); the HUD and every hit rect stay put.
    let shift = g.camera.offset_at(g.clock.now());
    // RULESET1-U slice 3: the round intent's board overlays (None in classic)
    let mut overlay = overlay_of(s, sel, planning);
    // INPUT1 MR13: the hovered Engage row marks its target's open position tiles
    let marked = match &hovered_pick {
        Some(((h, at), row)) if planning => match &row.act {
            RowAct::Pick(picks) => engage_positions(s, *h, *at, picks),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    if let Some(o) = overlay.as_mut() {
        o.positions = marked.clone();
        // VISCLAR1 AR1-AR3: every unit's target arrow; the board draws the selected and inspected unit's (Alt: all)
        o.arrows = arrows_of(s, &tells, &rounds, &layers, show_tells);
        g.arrows_shown = o.shown(&layers, sel, pinned, cx.held.alt).arrows;
    } else {
        g.arrows_shown = Vec::new();
    }
    g.positions_shown = marked;
    let push_list: Vec<emberwake_battle::forced::Forecast> = pushes.iter().flatten().cloned().collect();
    // VC2: the focus set but the unit under the mouse (each view adds its own hover)
    let focus = focus_set(w, sel, None, pinned);
    let board = Board { s, tokens: &mut g.tokens, floats: &g.floats, flashes: &g.flashes, popup: g.popup.is_some() || talk_view.is_some(), anim_ms: g.clock.ms, sel, talker, planning, tells: &tells, show_tells, card_hover, inspect: pinned, focus: &focus, fx: &g.fx, shift, sidesteps: &g.sidesteps, overlay: overlay.as_ref(), rounds: &rounds, layers, alt: cx.held.alt, marks: &marks, classes: &classes, pushes: &push_list, hold: hold.as_ref() };
    let (board_hover, flat_hover) = match view {
        View::Stage => (draw_stage(board, &mut ui, cx, &tuning, now), None),
        View::Flat => (None, draw_flat(board, &mut ui, cx, &tuning, now)),
    };
    g.focus_shown = focus_set(w, sel, board_hover.or(flat_hover).or(card_hover), pinned);
    // INPUT1 PB1: the impact tracers above the tokens (group `tracers`), shaken with the board, inside its panel
    if aids && layers.tracers && !g.tracers.is_empty() {
        let geo = Geom::new(view, &tuning);
        ui.p.clip(Some(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4)));
        ui.p.shift(shift.0, shift.1);
        draw_tracers(&mut ui.p, &g.tracers, &geo, g.clock.now(), cx.settings.reduce_motion());
        ui.p.shift(-shift.0, -shift.1);
        ui.p.clip(None);
    }
    // ENCOUNTER2 H1: the hold pick over the board while round 1 waits for it
    if planning && s.needs_stay_pick() {
        draw_hold_pick(&mut ui, s);
    }
    // WINDUP1 6.2: the clip each token drew (the agent link's `tokens`)
    note_drawn(&mut g.tokens, s, &mut cx.art, g.clock.ms as u64);

    // -- Right: hostile cards and the hovered (or first) foe's detail card.
    let right = Rect::new(W - 126, 28, 124, 290);
    ui.p.panel(right, PANEL, BORDER, RADIUS);
    let foes_active = w.enemies.iter().filter(|e| w.units[**e].is_active()).count();
    ui.p.text(right.x as f32 + 6.0, 32.0, &format!("Hostiles {foes_active}"), TITLE, FOE);
    let mut y = 44;
    let mut hovered_foe = None;
    for &e in &w.enemies {
        let u = &w.units[e];
        if w.grid.pos(e).is_none() && u.is_alive() && !u.is_withdrawn {
            continue; // reinforcements not on the field yet
        }
        if y > 206 {
            break;
        }
        // TEXTFIT1: the Next line wraps to a second line when it does not fit (the card grows).
        let tell = tells.iter().find(|t| t.unit == e);
        // RULESET1-U slice 4 (UI5): under the round intent the line leads with the intent ("Assault: Wolf; Cut B1")
        let next = tell.map(|t| intent_text::round_card_line(w, t)).unwrap_or_else(|| format!("Next: {}", next_action(s, e)));
        let next_w = if tell.is_some_and(|t| t.revealed) { 98.0 } else { 110.0 };
        let next_lines = if u.is_active() { ui.p.wrap(&next, SMALL, next_w) } else { Vec::new() };
        let two = next_lines.len() > 1;
        // V1: the impact badges on a row of the round's blocks under the line (rule plan_badges, while planning)
        let badges = tell.filter(|_| u.is_active() && emberwake_battle::intent::badges_on(w) && layers.badges);
        let row_h = if badges.is_some() { 6 } else { 0 };
        // HOOKS1-UI slice 3: a meter row grows the card by 5 (UM7)
        let ms = meters_of(w, e, &layers);
        let meter_h = if ms.is_empty() { 0 } else { 5 };
        let r = Rect::new(right.x + 4, y, 116, if two { 22 + ui.p.line_h(SMALL).ceil() as i32 } else { 22 } + row_h + meter_h);
        let on_board = match view {
            View::Stage => board_hover == Some(e),
            View::Flat => w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p))),
        };
        let hover = ui.hover(r) || on_board;
        if hover {
            hovered_foe = Some(e);
        }
        ui.p.panel(r, if hover { rgba(48, 28, 38, 255) } else { CARD }, if hover { FOE } else { BORDER }, RADIUS);
        let col = if u.is_active() { FOE } else { DIM };
        // UI5: the enemy's intent glyph before its name (a tell, so only while the tells show)
        // slice 5: from the round records, as the tokens draw it, so the two agree during the playback too
        let glyph = round_of(e).filter(|_| show_tells && u.is_active());
        if let Some(i) = glyph {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(r.x + 3, r.y + 2, 10, 10));
        }
        let gx = if glyph.is_some() { 12.0 } else { 0.0 };
        ui.p.text_fit(r.x as f32 + 3.0 + gx, r.y as f32 + 1.5, &short_name(&u.name), BODY, 70.0 - gx, col);
        if let Some(t) = badges {
            let row = Rect::new(r.x + 3, r.y + r.h - 7, r.w - 6, 4);
            let tip = draw_badge_row(&mut ui.p, t, row, s.def.total_blocks.max(1));
            ui.push(format!("foe.{e}.badges"), "badges", tip.iter().skip(1).map(|x| x.0.clone()).collect::<Vec<_>>().join("; "), row, true, false, None, tip);
        }
        ui.p.text_right(r.x as f32 + r.w as f32 - 3.0, r.y as f32 + 1.5, &format!("{}/{}", u.hp, u.max_hp), BODY, if u.is_active() { GOOD } else { DIM });
        if u.is_active() {
            if two {
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, &next_lines[0], SMALL, GOLD);
                // the rest of the line as written (a word `wrap` broke mid-word stays whole)
                let rest = next.get(next_lines[0].len()..).unwrap_or("").trim_start();
                ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5 + ui.p.line_h(SMALL), rest, SMALL, next_w, GOLD);
            } else {
                ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5, &next, SMALL, next_w, GOLD);
            }
            if tell.is_some_and(|t| t.revealed) {
                if let Some(img) = cx.art.image(&emberwake_content::art::badge("foresight"), 9, 9) {
                    ui.p.image(img, Rect::new(r.x + r.w - 11, r.y + 11, 9, 9));
                }
            }
        } else {
            ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, if u.is_withdrawn { "Fled" } else { "Down" }, SMALL, DIM);
        }
        meter_row(&mut ui.p, Rect::new(r.x + 3, r.y + r.h - row_h - 5, r.w - 6, 3), &ms);
        if layers.guard && held_guard(s, e).is_some() {
            ui.p.panel(r.inset(1), rgba(0, 0, 0, 0), HELD, RADIUS);
        }
        let tip = unit_tip(s, e, sel, &tells, &rounds, &classes);
        ui.push(format!("foe.{e}"), "card", format!("{} {}/{} {}", u.name, u.hp, u.max_hp, next), r, true, hover, Some(Action::UnitClick(e)), tip);
        y += r.h + 2;
    }
    g.panels.push(("hostiles", Rect::new(right.x, 28, right.w, y - 26)));
    debug_assert!(g.panels.iter().all(|p| PANELS.contains(&p.0)), "a panel recorded under a name PANELS lacks");
    // RULESET1-U slice 4 (UI5): under plan_badges the intel panel shows the hovered unit of either side (its card or
    // its token); classic keeps the hovered (or first) foe
    let ally_hover = if rounds.is_empty() {
        None
    } else {
        hovered_ally.or(match view {
            View::Stage => board_hover.filter(|u| w.allies.contains(u)),
            View::Flat => w.allies.iter().copied().find(|&a| w.units[a].is_active() && w.grid.pos(a).is_some_and(|p| ui.hover(cell_rect(p)))),
        })
    };
    // INPUT1 IS1: an inspected unit pins the panel (until another inspect, Esc or the round's end)
    let detail_foe = pinned.or(hovered_foe).or(ally_hover).or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
        g.panels.push(("intel", r));
        ui.p.panel(r, CARD, if w.allies.contains(&e) { ALLY } else { FOE }, RADIUS);
        let mut ly = r.y as f32 + 3.0;
        let mut lines = if pinned.is_some() { inspect_tip(s, e, sel, &tells, &rounds, &classes, show_tells) } else { unit_tip(s, e, sel, &tells, &rounds, &classes) };
        // RULESET1-U slice 5: the selected hero's planned hits on this unit, forecast (rule poise)
        if let Some(h) = sel.filter(|h| layers.forecasts && planning && ri::on(w) && w.units[*h].is_active()) {
            for f in hit_forecasts(s, h).into_iter().flatten().filter(|f| f.target == e) {
                lines.push(f.short(w));
            }
        }
        // the agent link names the panel's unit (UI5: either side; classic adds no widget)
        if !rounds.is_empty() || pinned.is_some() {
            ui.push("intel.unit", "panel", lines.first().map(|l| l.0.clone()).unwrap_or_default(), r, true, false, None, Vec::new());
        }
        // PORTRAITFX1-P (6.2 row 3): the unit's portrait at the panel's top-left, the lines beside it 40 px in
        let (who, look) = g.unit_portrait(cx, e);
        let face = panel_portrait(&mut ui, cx, &mut g.panel_faces, &who, &look, r, "intel.portrait");
        let beside = (face.y + face.h + 1) as f32 - ly;
        // HOOKS1-UI slice 5: the lines that fit (the Level/Speed line gives way first, then "More on its hover card")
        for (part, col, head, narrow) in intel_parts_at(&mut ui.p, &lines, (r.h - 12) as f32, beside) {
            let size = if head { BODY } else { SMALL };
            let x = if narrow { r.x + PANEL_TEXT_X } else { r.x + 4 };
            ui.p.text(x as f32, ly, &part, size, col);
            ly += if head { INTEL_HEAD } else { INTEL_LINE };
        }
    }

    // -- Bottom bar: playback, round and status, the last log line.
    ui.p.panel(Rect::new(2, 322, W - 4, 36), PANEL, BORDER, RADIUS);
    ui.button_full("bar.restart", Rect::new(6, 325, 40, 14), "Restart", Action::Restart, !g.world_mode(), false, BODY, vec![("Restart the battle (R)".into(), TEXT)]);
    ui.button_full("bar.step", Rect::new(48, 325, 34, 14), "Step", Action::Step, !planning || s.plan_confirmed, false, BODY, vec![("Resolve one action (Space)".into(), TEXT)]);
    // Test Arena only: leave at any time (Esc does the same); a world battle is left only through its end.
    let ox = if g.world_mode() {
        0
    } else {
        ui.button_full("bar.exit", Rect::new(84, 325, 30, 14), "Exit", Action::Leave, true, false, BODY, vec![("Back to the screen you came from (Esc)".into(), TEXT)]);
        32
    };
    let phase = match s.phase {
        Phase::Planning => "Planning",
        Phase::Resolving => "Resolving",
        Phase::Replay => "Round over",
    };
    ui.p.text((88 + ox) as f32, 326.0, &format!("Round {}/{}  {phase}", s.current_round, s.def.max_rounds), BODY, GOLD);
    // RULESET1-U (user 2026-10-29): an Arena fight beyond classic shows the Scene Runner card's ruleset chip at the
    // bar's right end; the message gives up the width
    let chip = g.arena_chip();
    let chip_w = chip.as_ref().map_or(0, |c| ((ui.p.text_w(c, SMALL) + 10.0).ceil() as i32).min(200) + 4);
    ui.p.text_fit((200 + ox) as f32, 326.0, &g.message, BODY, (430 - ox - chip_w) as f32, TEXT);
    ui.push("bar.status", "label", format!("Round {} {phase}: {}", s.current_round, g.message), Rect::new(88 + ox, 325, 540 - ox - chip_w, 12), true, false, None, Vec::new());
    if let Some(chip) = chip {
        let cr = Rect::new(W - 6 - (chip_w - 4), 325, chip_w - 4, 14);
        ui.p.panel(cr, rgba(30, 44, 70, 255), BORDER, 2.0);
        ui.p.text_fit(cr.x as f32 + 5.0, cr.y as f32 + 3.0, &chip, SMALL, (cr.w - 10) as f32, GOLD);
        let on = format!("Rules on: {}.", s.w.ruleset.rules.join(", "));
        ui.push("bar.chip", "label", chip.clone(), cr, true, false, None, vec![(format!("Ruleset: {chip}"), GOLD), (on, TEXT)]);
    }
    let last = s.resolver.log.last().cloned().unwrap_or_else(|| "No actions yet".into());
    // INPUT1 PB4: a timeline battle's log in its parts (round and block); during playback the compact log shows the
    // resolving block's part (its last line; the card lists the part)
    let groups = if aids { s.log_groups() } else { Vec::new() };
    let (last, mut card): (String, Tip) = match cursor {
        Some(b) => {
            let lines = groups.iter().find(|p| (p.round, p.block) == (s.current_round, b)).map(|p| p.lines.clone()).unwrap_or_default();
            let head = (format!("Round {} block {b}", s.current_round), GOLD);
            let shown = lines.last().map_or_else(|| format!("Block {b}: nothing logged yet"), |l| format!("Block {b}: {l}"));
            let start = lines.len().saturating_sub(LOG_CARD_LINES);
            (shown, std::iter::once(head).chain(lines[start..].iter().map(|l| (l.clone(), TEXT))).collect())
        }
        None => (last.clone(), vec![(last, TEXT)]),
    };
    card.push(("Click to open the full log.".into(), DIM));
    let log_r = Rect::new(6, 341, W - 64, 14);
    ui.p.panel(log_r, CARD, BORDER, RADIUS);
    ui.p.text_fit(10.0, 342.5, &last, SMALL, (W - 72) as f32, DIM);
    ui.push("bar.last", "label", last, log_r, true, false, Some(Action::ToggleLog), card);
    ui.button_full("bar.log", Rect::new(W - 54, 341, 48, 14), if g.show_log { "Close" } else { "Expand" }, Action::ToggleLog, true, g.show_log, BODY, vec![("Full combat log (L); the wheel scrolls".into(), TEXT)]);

    let outcome = g.outcome();
    if !outcome.is_empty() {
        let r = Rect::new(GRID_X + 92, GRID_Y + 110, 200, 56);
        ui.p.panel(r, rgba(10, 14, 22, 240), GOLD, 4.0);
        let (title, col) = match outcome.as_str() {
            "won" => ("Victory", GOOD),
            "lost" => ("Defeat", FOE),
            "fled" => ("Fled", GOLD),
            _ => ("Battle over", GOLD),
        };
        ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 8.0, title, 11.0, col);
        if g.world_mode() {
            ui.button_full("banner.continue", Rect::new(r.x + 64, r.y + 32, 72, 16), "Continue", Action::Continue, true, true, BODY, Vec::new());
        } else {
            ui.button_full("banner.restart", Rect::new(r.x + 24, r.y + 32, 72, 16), "Play again", Action::Restart, true, false, BODY, Vec::new());
            ui.button_full("banner.leave", Rect::new(r.x + 104, r.y + 32, 72, 16), "Leave", Action::Leave, true, false, BODY, Vec::new());
        }
    }

    if g.show_log {
        let r = Rect::new(GRID_X, GRID_Y + 4, GRID_W * TILE, GRID_H * TILE - 8);
        // ENCOUNTER2-U (decision "RULESET1-A3 loose ends"): the resolver log with the encounter's phase and kill lines
        let log = s.log_lines(0, 0);
        ui.push("log.panel", "panel", format!("{} lines", log.len()), r, true, true, Some(Action::Step), Vec::new());
        ui.p.panel(r, rgba(10, 14, 24, 246), BORDER, 3.0);
        ui.p.text(r.x as f32 + 6.0, r.y as f32 + 3.0, &format!("Combat log ({} entries)", log.len()), TITLE, GOLD);
        // the rows: text, colour, x offset, a rule after it; INPUT1 PB4: a timeline battle's lines under their round's
        // "Round <r>" and their part's DIM "Block <b>" ("Round start", "Round end") with a 1 px rule, 6 px in
        let mut wrapped: Vec<(String, Rgba, f32, bool)> = Vec::new();
        let wrap_line = |p: &mut Painter, out: &mut Vec<(String, Rgba, f32, bool)>, l: &str, indent: f32| {
            let col = if l.starts_with("[Phase]") { GOLD } else if l.starts_with("[Kill]") { FOE } else if l.starts_with("[Attack]") { TEXT } else if l.starts_with("[Support]") { GOOD } else if l.contains("Failed") || l.contains("Out of Range") { FOE } else { DIM };
            for (k, part) in p.wrap(l, SMALL, (r.w - 22) as f32 - indent).into_iter().enumerate() {
                out.push((if k == 0 { part } else { format!("   {part}") }, col, indent, false));
            }
        };
        if aids {
            let mut round = 0;
            for part in &groups {
                if part.round != round {
                    round = part.round;
                    wrapped.push((format!("Round {round}"), GOLD, 0.0, false));
                }
                wrapped.push((log_part_name(part.block), DIM, 0.0, true));
                for l in &part.lines {
                    wrap_line(&mut ui.p, &mut wrapped, l, LOG_INDENT);
                }
            }
        } else {
            for l in &log {
                wrap_line(&mut ui.p, &mut wrapped, l, 0.0);
            }
        }
        let visible = ((r.h - 18) as f32 / 8.0) as i32;
        let max_scroll = (wrapped.len() as i32 - visible).max(0);
        g.log_scroll = g.log_scroll.min(max_scroll);
        let start = (wrapped.len() as i32 - visible - g.log_scroll).max(0) as usize;
        for (k, (t, col, indent, rule)) in wrapped.iter().skip(start).take(visible as usize).enumerate() {
            let (x, y) = (r.x as f32 + 6.0 + indent, r.y as f32 + 15.0 + k as f32 * 8.0);
            ui.p.text(x, y, t, SMALL, *col);
            if *rule {
                let x0 = (x + ui.p.text_w(t, SMALL) + 3.0).ceil() as i32;
                ui.p.fill(Rect::new(x0, y as i32 + 4, (r.x + r.w - 12 - x0).max(0), 1), with_alpha(DIM, 110));
            }
        }
        if max_scroll > 0 {
            let track = Rect::new(r.x + r.w - 8, r.y + 15, 4, r.h - 19);
            ui.p.panel(track, rgba(30, 40, 60, 255), rgba(30, 40, 60, 255), 1.5);
            let thumb_h = (track.h * visible / wrapped.len() as i32).max(8);
            let pos = (track.h - thumb_h) * (max_scroll - g.log_scroll) / max_scroll;
            ui.p.panel(Rect::new(track.x, track.y + pos, 4, thumb_h), GOLD, GOLD, 1.5);
        }
    }

    // ENCOUNTER2-C4: a scripted round's banner (locked or guided) above the board, centred at its top edge
    if let Some(text) = s.round_script().map(|r| r.banner.clone()).filter(|b| !b.is_empty()) {
        let bw = ((ui.p.text_w(&text, BODY) + 16.0).ceil() as i32).min(GRID_W * TILE - 8);
        let r = Rect::new(GRID_X + (GRID_W * TILE - bw) / 2, GRID_Y + 2, bw, 16);
        ui.p.panel(r, CARD, GOLD, RADIUS);
        ui.p.text_fit(r.x as f32 + 8.0, r.y as f32 + 3.0, &text, BODY, r.w as f32 - 16.0, GOLD);
        ui.push("battle.banner", "label", text, r, true, false, None, Vec::new());
    }

    // INPUT1 TV1-TV6: the full timeline view over the grid panel's rect (key T; T or Esc closes it): the engine widget
    // on the shared builder (`timeline_view`), 4 px inside the panel; its backdrop keeps the board's clicks and hover
    // cards from showing through
    if g.timeline_view_open() {
        let r = Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4);
        let opts = timeline_view::Opts { viewed: s.current_round, reveal_all: false, foresight_off, show_tells, layers, selected: sel, block: None, shown: sel.zip(shown).map(|(u, (pw, pp))| (u, pw, pp)), playhead: cursor };
        let mut t = timeline_view::build(s, &opts);
        // PORTRAITFX1-TL (6.2 row 4): every unit's track leads with its portrait in the icon square (minimal tier,
        // `timeline_faces`), the panels' `unit_portrait` with the board's HP and life
        let mut icons: std::collections::BTreeMap<String, (Who, emberwake_battle::portrait::Look)> = std::collections::BTreeMap::new();
        for track in t.sections.iter_mut().flat_map(|sec| sec.tracks.iter_mut()) {
            if let Some(u) = timeline_view::unit_of(&track.key).filter(|&u| u < s.w.units.len()) {
                track.icon = true;
                let (who, mut look) = g.unit_portrait(cx, u);
                if timeline_row_plain(s, u, track.masked.is_some()) {
                    look = plain_look(&look);
                }
                icons.insert(track.key.clone(), (who, look));
            }
        }
        ui.push("timeline.backdrop", "backdrop", "", r, true, false, Some(Action::TimelinePick(None)), Vec::new());
        ui.p.panel(r, rgba(10, 14, 24, 250), BORDER, RADIUS);
        let on_track = |k: &str| Action::TimelinePick(timeline_view::unit_of(k));
        let faces = &mut g.timeline_faces;
        let icon = |ui: &mut Ui<Action>, track: &aegis_ui::timeline::Track, sq: Rect| {
            if let Some((who, look)) = icons.get(&track.key) {
                timeline_portrait(ui, cx, faces, who, look, sq);
            }
        };
        t.draw_icons(&mut ui, "timeline", Rect::new(r.x + TV_PAD, r.y + TV_PAD, r.w - 2 * TV_PAD, r.h - 2 * TV_PAD), |_| Action::TimelinePick(None), on_track, |k, _| on_track(k), icon);
        g.tv_shown = Some(t);
    } else {
        g.tv_shown = None;
    }

    if let Some(pop) = g.popup.clone() {
        let r = pop.rect(g.menu_row);
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup), Vec::new());
        ui.p.panel(r, rgba(14, 20, 34, 252), GOLD, 3.0);
        ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, &pop.title, BODY, r.w as f32 - 10.0, GOLD);
        for (k, row) in pop.rows.iter().enumerate() {
            let rr = pop.row(g.menu_row, k);
            let hover = ui.hover(rr);
            if hover && row.valid {
                ui.p.panel(rr, PANEL_HI, rgb(110, 140, 196), 1.5);
            }
            let col = if row.valid { TEXT } else { DIM };
            // the label column: 78 px in a 170 px menu, wider in the skill menu (INPUT1 IF3)
            let label_w = (pop.w - 92) as f32;
            ui.p.text_fit(rr.x as f32 + 4.0, rr.y as f32 + 1.5, &row.label, BODY, label_w, col);
            ui.p.text_fit(rr.x as f32 + label_w + 6.0, rr.y as f32 + 2.0, &row.detail, SMALL, rr.w as f32 - label_w - 8.0, if row.valid { GOLD } else { DIM });
            // IF3, IN6: a pick that changes the intent and drops intent-bound buffs: a red corner
            if row.warn {
                ui.p.fill(Rect::new(rr.x + rr.w - 4, rr.y + 1, 3, 3), FOE);
            }
            ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
        }
    }

    // ENCOUNTER2-C DT7: the talk over a 35 % veil, centred at the bottom; its backdrop keeps the board's hover cards
    // from showing through the box
    if let (Some(gs), Some(view)) = (talk_gs.as_ref(), talk_view.as_ref()) {
        ui.p.fill(Rect::new(0, 0, W, H), TALK_VEIL);
        let b = talk::talk_layout(&mut ui.p, view);
        ui.push("talk.backdrop", "backdrop", "", b.r, true, false, None, Vec::new());
        talk::draw_talk(&mut ui, cx, gs, &mut g.faces, view, &b, false, Action::Talk);
    }

    g.w = ui.widgets;
}

// ---- RULESET1-U slice 3: the board overlays under the round intent (RULESET1_INTENT UI3, RULESET1 K6) -------------

/// How far around the round target the Maneuver front is shaded (tiles, Manhattan).
const FRONT_RADIUS: i32 = 3;

/// A unit's "Reach" on its hover card and the hero panel: under the rule `reach` its plain attack band ("1-2", K6:
/// kit `skill::attack_band`), else its `attack_range` (classic, as before).
fn reach_text(w: &World, i: usize) -> String {
    if !w.ruleset.has(emberwake_rules::ruleset::REACH) {
        return w.units[i].attack_range.to_string();
    }
    match emberwake_battle::skill::attack_band(&w.units[i], &w.ruleset) {
        (lo, hi) if lo == hi => lo.to_string(),
        (lo, hi) => format!("{lo}-{hi}"),
    }
}

/// How a planned move of the selected hero is drawn: a plain move before its commands (walk), a move its skill
/// carries (MV4: an Engage's dash, a chase), or a step after its commands (AS3's kite step, MV8's retreat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveKind {
    Walk,
    Carried,
    After,
}

impl MoveKind {
    fn id(self) -> &'static str {
        match self {
            MoveKind::Walk => "walk",
            MoveKind::Carried => "carried",
            MoveKind::After => "after",
        }
    }
}

/// RULESET1-U slice 3: what the board draws under the round intent, every piece a kit query (none in classic).
#[derive(Debug, Clone, Default, PartialEq)]
struct Overlay {
    /// The selected hero's round target tile (a unit's or the target tile; None for self or none). Planning only.
    target: Option<V2>,
    /// The tiles of the units a board click would make the hero's round target (the intent table, PL2; UI1 "valid
    /// cells lit"), the current target left out. Planning only.
    valid: Vec<V2>,
    /// K6: the band of the hero's plain attack (`skill::attack_band`) around the tile its first attack strikes from
    /// (else where it stands), under Assault and Maneuver. Planning only.
    band: Vec<V2>,
    /// UI3: under Maneuver, the target's front within `FRONT_RADIUS` (`round_intent::in_front`: no flank there).
    front: Vec<V2>,
    /// The hero's planned moves in order: the tiles each passes (from first) and how it is drawn.
    moves: Vec<(Vec<V2>, MoveKind)>,
    /// MV1: every active enemy's tile and facing (a sign vector; units without one left out). Planning and playback.
    facings: Vec<(V2, V2)>,
    /// GD5: the tiles of stealthed units, and whether each is hidden now (moved this round). Planning and playback.
    veils: Vec<(V2, bool)>,
    /// VISCLAR1 AR1-AR6 (they replace RULESET1-U's tell arrows, planned-move lines and revealed enemy walks): every
    /// unit's one target arrow (`arrows_of`; the screen fills it, `shown` keeps the selected unit's and the inspected
    /// unit's, or all while Alt is held). Planning only.
    arrows: Vec<Arrow>,
    /// INPUT1 MR13: while a Maneuver Engage row of the skill menu is hovered, its target's open position tiles (true:
    /// the preferred tier); none when the flank is protected. Planning only, layer `moves`.
    positions: Vec<(V2, bool)>,
}

/// RULESET1-U slice 6: the overlay groups as the Settings Battle tab lists them: (id, label, what it shows, default);
/// VISCLAR1 VC6: every group on by default but `first_move` and `all_bars`.
pub(crate) const OVERLAY_GROUPS: [(&str, &str, &str, bool); 25] = [
    ("band", "Reach band", "The selected hero's attack reach around the tile it strikes from (Assault, Maneuver).", true),
    ("front", "Maneuver front", "The target's front, where an Engage never stands (Maneuver).", true),
    ("valid", "Valid targets", "Rings on the units a click would make the round target.", true),
    ("veils", "Stealth veils", "A veil under each stealthed unit, filled once it is hidden.", true),
    // VISCLAR1 AR5: the moves now draw as destination plates and the move leg of the party's target arrows
    ("moves", "Planned moves", "A plate where each of the selected hero's moves ends, and the move leg of the party's target arrows (off: straight).", true),
    ("target", "Target ring", "The ring on the selected hero's round target.", true),
    ("facings", "Enemy facings", "A chevron where each enemy faces (a flank comes from beside or behind).", true),
    // VISCLAR1 AR5: a revealed enemy's moves are its target arrow's move leg
    ("walks", "Revealed enemy moves", "The move leg of a revealed enemy's target arrow (off: the arrow goes straight to its target).", true),
    ("glyphs", "Intent glyphs", "The round intent's plate on the strip, the cards and the tokens.", true),
    // VISCLAR1 VC1: the tell's kind chip on an enemy's token, a debug view
    ("first_move", "Enemy first move", "The kind chip of each enemy's first action on its token (debug).", false),
    ("badges", "Impact badges", "The row of blocks on each hostile card with its impacts.", true),
    ("poise", "Poise bars", "The poise bars under the HP bars.", true),
    // VISCLAR1 VC3: today's look, every unit's bars on the board
    ("all_bars", "All bars", "Every unit's HP and poise bars on the board; off: only the selected hero, its target and the unit under the mouse (hold Alt: all).", false),
    ("breaks", "Breaks", "BREAK over a broken unit and on its cancelled strip span.", true),
    ("parts", "Wind-up and impact", "The strip's wind-up, impact and recovery shading.", true),
    ("forecasts", "Hit forecasts", "Stagger and break forecasts in the strip tips and the intel panel.", true),
    ("summary", "Round summary", "The round summary in the message bar at round end.", true),
    // HOOKS1-UI slice 1: the hooks' marks (HOOKS1 GH6, AH3)
    ("hazards", "Ground hazards", "The hazards skills and items leave on tiles, in their tint, with their glyph (a foe's hidden one stays hidden).", true),
    ("arena", "Arena telegraph", "The cells an arena hazard strikes this round, and a badge on the strip at the blocks it strikes.", true),
    // HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3)
    ("meters", "Unit meters", "Each unit's meters as pips or a bar on its card, and the selected hero's under the strip title.", true),
    ("guard", "Held guard", "A blue edge on the card of a unit holding a guard a trigger gave it (it counts as guarding).", true),
    // HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5)
    ("pushes", "Push forecasts", "Where the selected hero's planned pushes and pulls would move their targets, and what they would collide with.", true),
    // ENCOUNTER2-U: a hold escape's marks (ENCOUNTER2 H7)
    ("hold", "Hold tiles", "The tiles a hold escape's holder keeps, with the holder's name on their card.", true),
    ("gate", "Gate state", "A hold escape's exits read SHUT (red) until the holder stands on a hold tile, then OPEN (green).", true),
    // INPUT1 PB1: the playback's impact tracers
    ("tracers", "Impact tracers", "A short line from each actor to what its action hit, during playback.", true),
];

/// VISCLAR1 VC6: a group's default (`OVERLAY_GROUPS`; an unknown id is on).
pub(crate) fn overlay_default(id: &str) -> bool {
    OVERLAY_GROUPS.iter().find(|g| g.0 == id).is_none_or(|g| g.3)
}

/// RULESET1-U slice 5 (preference "Battle overlays and the clarity pass", user 2026-10-30): every timeline overlay
/// group behind its own flag, on by default (they help debugging the fight); the settings key `battle_overlays`
/// ({<group>: false}) switches a group off; slice 6 lists them on the Settings Battle tab; VISCLAR1 slice A adds
/// `first_move` and `all_bars`, off by default (VC6). Classic draws none of them whatever the flags say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Layers {
    /// Slice 3: the reach band, the Maneuver front, the valid-target rings, the stealth veils, the hero's moves, the
    /// target ring, the enemy facings.
    band: bool,
    front: bool,
    valid: bool,
    veils: bool,
    moves: bool,
    target: bool,
    facings: bool,
    /// Slices 4-5: revealed enemy moves, intent glyphs, impact badge rows.
    walks: bool,
    glyphs: bool,
    /// VISCLAR1 VC1: the tell's kind chip on an enemy's token (off by default).
    first_move: bool,
    pub(crate) badges: bool,
    /// Slice 5: poise bars, BREAK markers, wind-up / impact / recovery on the strip, hit forecasts, the round summary.
    poise: bool,
    /// VISCLAR1 VC3: every unit's token bars, not only the focus set's (off by default).
    all_bars: bool,
    pub(crate) breaks: bool,
    pub(crate) parts: bool,
    forecasts: bool,
    summary: bool,
    /// HOOKS1-UI slice 1: ground hazards (GH6), the arena telegraph (AH3).
    hazards: bool,
    arena: bool,
    /// HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3).
    meters: bool,
    guard: bool,
    /// HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5).
    pushes: bool,
    /// ENCOUNTER2-U: a hold escape's tiles and gate (H7).
    hold: bool,
    gate: bool,
    /// INPUT1 PB1: the playback's impact tracers.
    tracers: bool,
}

impl Layers {
    /// The group ids, as the settings key names them (`OVERLAY_GROUPS`' order).
    const IDS: [&'static str; 25] = {
        let mut ids = [""; 25];
        let mut k = 0;
        while k < 25 {
            ids[k] = OVERLAY_GROUPS[k].0;
            k += 1;
        }
        ids
    };

    /// The flags of the settings key `battle_overlays` (a missing group, or no key, takes its default: VC6).
    pub(crate) fn of(v: &Value) -> Layers {
        let on = |id: &str| v.get(id).and_then(Value::as_bool).unwrap_or_else(|| overlay_default(id));
        Layers {
            band: on("band"),
            front: on("front"),
            valid: on("valid"),
            veils: on("veils"),
            moves: on("moves"),
            target: on("target"),
            facings: on("facings"),
            walks: on("walks"),
            glyphs: on("glyphs"),
            first_move: on("first_move"),
            badges: on("badges"),
            poise: on("poise"),
            all_bars: on("all_bars"),
            breaks: on("breaks"),
            parts: on("parts"),
            forecasts: on("forecasts"),
            summary: on("summary"),
            hazards: on("hazards"),
            arena: on("arena"),
            meters: on("meters"),
            guard: on("guard"),
            pushes: on("pushes"),
            hold: on("hold"),
            gate: on("gate"),
            tracers: on("tracers"),
        }
    }

    /// The flags by group id (the agent link's state).
    fn json(&self) -> Value {
        let v = [self.band, self.front, self.valid, self.veils, self.moves, self.target, self.facings, self.walks, self.glyphs, self.first_move, self.badges, self.poise, self.all_bars, self.breaks, self.parts, self.forecasts, self.summary, self.hazards, self.arena, self.meters, self.guard, self.pushes, self.hold, self.gate, self.tracers];
        Value::Object(Self::IDS.iter().zip(v).map(|(k, b)| (k.to_string(), json!(b))).collect())
    }
}

impl Overlay {
    /// What the board draws of it: the groups whose layer is on; AR3: the target arrows of the selected unit `sel` and
    /// the inspected unit `inspect` (decision "INPUT1 answers, part 4" Q15), every unit's while `alt` is held.
    fn shown(&self, l: &Layers, sel: Option<usize>, inspect: Option<usize>, alt: bool) -> Overlay {
        let keep = |on: bool, v: &Vec<V2>| if on { v.clone() } else { Vec::new() };
        Overlay {
            target: self.target.filter(|_| l.target),
            valid: keep(l.valid, &self.valid),
            band: keep(l.band, &self.band),
            front: keep(l.front, &self.front),
            moves: if l.moves { self.moves.clone() } else { Vec::new() },
            facings: if l.facings { self.facings.clone() } else { Vec::new() },
            veils: if l.veils { self.veils.clone() } else { Vec::new() },
            arrows: self.arrows.iter().filter(|a| alt || Some(a.unit) == sel || Some(a.unit) == inspect).cloned().collect(),
            positions: if l.moves { self.positions.clone() } else { Vec::new() },
        }
    }
}

// ---- VISCLAR1 slice A: the target arrows, the bars in focus, Party foresight off (INPUT1 10) ------------------------

/// VISCLAR1 AR1: one unit's target arrow, a polyline of tiles: its tile, the end of its moves before its first command
/// when that move shows (AR2), the target (a unit's tile or the target tile; none for a self target, which ends at the
/// move's end). Two points or more.
#[derive(Debug, Clone, PartialEq)]
struct Arrow {
    unit: usize,
    points: Vec<V2>,
    ally: bool,
}

/// AR4: the arrows' alpha (of 255), dash (px on, px off), head (px) and width (Flat, Stage).
const ARROW_ALPHA: u8 = 90;
const ARROW_DASH: (f32, f32) = (4.0, 3.0);
const ARROW_HEAD: f32 = 4.0;
const ARROW_W_FLAT: f32 = 1.0;
const ARROW_W_STAGE: f32 = 1.5;

/// VC4: the debug settings key of "Party foresight off" (in no `default_settings`, VC5).
pub(crate) const FORESIGHT_OFF: &str = "debug_party_foresight_off";

/// VC5: whether the party's foresight reveals draw as unrevealed: the debug key, read only when `debug` (called with
/// `cfg!(debug_assertions)`), so a hand-edited settings file cannot turn it on in a release build.
pub(crate) fn party_foresight_off(settings: &Value, debug: bool) -> bool {
    debug && settings.get(FORESIGHT_OFF).and_then(Value::as_bool).unwrap_or(false)
}

/// VC4: the records `v` as the screen shows them; with `off` (Party foresight off) a plan that only a party foresight
/// passive revealed reads unrevealed (no plan lines, no readers); a reveal by enrage or by the battle's `plan_reveal`
/// stays. Display only: the kit's records are unchanged.
pub(crate) fn seen_intents(w: &World, mut v: Vec<Intent>, off: bool) -> Vec<Intent> {
    if !off {
        return v;
    }
    let reveal_all = emberwake_battle::intent::badges_on(w) && w.plan_reveal;
    for r in v.iter_mut().filter(|r| !r.readers.is_empty()) {
        r.readers.clear();
        if !r.enraged && !reveal_all {
            r.revealed = false;
            r.plan.clear();
        }
    }
    v
}

/// AR1, AR2: every unit's target arrow while the party plans (empty otherwise and in classic): a party member's from
/// its round target and its compiled plan (the move leg with the group `moves` on); an enemy's while the tells show,
/// to its round target (without a round intent: its tell's target), with the move leg only when its plan is revealed
/// (`tells` as the screen shows them, VC4) and the group `walks` is on. An after-move is no part of it (Q14: its
/// destination plate shows it).
fn arrows_of(s: &Session, tells: &[Intent], rounds: &[Intent], layers: &Layers, show_tells: bool) -> Vec<Arrow> {
    let w = &s.w;
    if !ri::on(w) || !s.is_planning() {
        return Vec::new();
    }
    let on_field = |u: usize| w.units[u].is_active() && w.has(u);
    let tile_of = |t: RoundTarget, me: usize| match t {
        RoundTarget::Unit(u) if u != me && on_field(u) => Some(Some(w.pos(u))),
        RoundTarget::Unit(u) if u == me => Some(None),
        RoundTarget::Myself => Some(None),
        RoundTarget::Cell(c) if w.grid.is_valid(c) => Some(Some(c)),
        _ => None,
    };
    let mut out = Vec::new();
    let mut push = |unit: usize, ally: bool, leg: Option<V2>, to: Option<Option<V2>>| {
        let from = w.pos(unit);
        let mut points = vec![from];
        if let Some(end) = leg.filter(|c| *c != from) {
            points.push(end);
        }
        match to {
            Some(Some(t)) if points.last() != Some(&t) => points.push(t),
            // a self target ends at the move's end (nothing without a move); no target draws nothing
            Some(_) => {}
            None => return,
        }
        if points.len() >= 2 {
            out.push(Arrow { unit, points, ally });
        }
    };
    for &h in w.allies.iter().filter(|&&h| on_field(h)) {
        let target = w.round_states.get(&h).and_then(|r| r.target).or_else(|| rounds.iter().find(|r| r.unit == h).and_then(|r| r.round_target));
        let leg = layers.moves.then(|| party_leg(s, h));
        push(h, true, leg, target.and_then(|t| tile_of(t, h)));
    }
    if show_tells {
        for t in tells.iter().filter(|t| on_field(t.unit)) {
            let rec = rounds.iter().find(|r| r.unit == t.unit && r.round_intent.is_some());
            let to = match rec {
                Some(r) => r.round_target.and_then(|x| tile_of(x, t.unit)),
                None => t.line.target.filter(|u| on_field(*u)).map(|u| Some(w.pos(u))).or_else(|| (w.grid.is_valid(t.line.cell)).then_some(Some(t.line.cell))),
            };
            let leg = (layers.walks && t.revealed).then(|| enemy_leg(s, t.unit));
            push(t.unit, false, leg, to);
        }
    }
    out
}

/// AR1: where party member `h`'s moves before its first command end (its walks, then the carried move its first
/// command rides; Rest is no command); its own tile without one.
fn party_leg(s: &Session, h: usize) -> V2 {
    let w = &s.w;
    let mut at = w.pos(h);
    for e in &s.plan.compile(w, h).entries {
        match emberwake_battle::skill::kind(&w.cat.skills[e.skill]) {
            Kind::Move => {
                if w.grid.is_valid(e.pos_after) {
                    at = e.pos_after;
                }
            }
            Kind::Rest => {}
            _ => {
                if w.grid.is_valid(strike_tile(e)) {
                    at = strike_tile(e);
                }
                break;
            }
        }
    }
    at
}

/// AR2: where a revealed enemy's moves before its first command end (its walks as the kit would walk them now,
/// `command::walk_preview`, then a carried Engage dash of its first command); its own tile without one.
fn enemy_leg(s: &Session, e: usize) -> V2 {
    let w = &s.w;
    let mut acts: Vec<&emberwake_battle::resolver::ScheduledAction> = s.resolver.scheduled.iter().filter(|a| a.unit == e && !a.cancelled).collect();
    acts.sort_by_key(|a| a.start);
    let mut at = w.pos(e);
    for a in acts {
        match (&a.command.kind, a.command.carry) {
            (CmdKind::Walk { path, expected_start, .. }, _) => {
                if let Some(end) = emberwake_battle::command::walk_preview(w, e, path, *expected_start).last() {
                    at = *end;
                }
            }
            (_, emberwake_battle::command::Carry::Engage(c)) => {
                at = c;
                break;
            }
            _ => break,
        }
    }
    at
}

/// VC2: the focus set whose token bars draw by default: the selected member, its round target when that is a unit,
/// the hovered unit (token or card) and the inspected unit.
fn focus_set(w: &World, sel: Option<usize>, hovered: Option<usize>, inspect: Option<usize>) -> Vec<usize> {
    let target = sel.and_then(|h| w.round_states.get(&h)).and_then(|r| match r.target {
        Some(RoundTarget::Unit(t)) => Some(t),
        _ => None,
    });
    let mut out: Vec<usize> = Vec::new();
    for u in [sel, target, hovered, inspect].into_iter().flatten() {
        if !out.contains(&u) {
            out.push(u);
        }
    }
    out
}

/// VC1: whether the tells' kind chips draw on the enemy tokens (while the tells show): in classic always; under the
/// round intent only with the group `first_move`.
fn first_move_chips(timeline: bool, layers: &Layers) -> bool {
    !timeline || layers.first_move
}

/// VC2, VC3: whether unit `i`'s token bars draw: in classic always; under the round intent for the focus set, every
/// unit while Alt is held or the group `all_bars` is on.
fn bars_shown(timeline: bool, layers: &Layers, alt: bool, focus: &[usize], i: usize) -> bool {
    !timeline || layers.all_bars || alt || focus.contains(&i)
}

/// AR4: a polyline of ground points dashed `ARROW_DASH` (the pattern runs on over the corners), its head of
/// `ARROW_HEAD` at the end.
fn draw_arrow_line(p: &mut Painter, pts: &[(f32, f32)], col: Rgba, width: f32) {
    dash_polyline(p, pts, col, width);
    if let [.., a, b] = pts {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = (dx * dx + dy * dy).sqrt();
        if len >= 0.5 {
            let (ux, uy) = (dx / len, dy / len);
            for side in [-1.0f32, 1.0] {
                let (c, s) = (0.866f32, 0.5 * side);
                let (bx, by) = (-(ux * c - uy * s), -(uy * c + ux * s));
                stage::wide_line(p, b.0, b.1, b.0 + bx * ARROW_HEAD, b.1 + by * ARROW_HEAD, col, width, 0.0);
            }
        }
    }
}

/// AR4, PB1: a polyline in `ARROW_DASH`'s dash (4 px on, 3 off), the dash running on over each corner.
fn dash_polyline(p: &mut Painter, pts: &[(f32, f32)], col: Rgba, width: f32) {
    let (on, off) = ARROW_DASH;
    let mut phase = 0.0f32;
    for seg in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (seg[0], seg[1]);
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if len < 0.5 {
            continue;
        }
        let (ux, uy) = ((x1 - x0) / len, (y1 - y0) / len);
        let mut d = 0.0f32;
        while d < len {
            let at = (phase + d) % (on + off);
            let step = if at < on { on - at } else { on + off - at };
            let e = (d + step).min(len);
            if at < on {
                stage::wide_line(p, x0 + ux * d, y0 + uy * d, x0 + ux * e, y0 + uy * e, col, width, 0.0);
            }
            d = e;
        }
        phase = (phase + len) % (on + off);
    }
}

/// AR4: the shown arrows on the ground, under the tokens (Flat: after the ground overlays; Stage: the Arrows pass):
/// party GOLD, enemy FOE at `ARROW_ALPHA`; each leg leaves its tile's token and the last stops short of its target's
/// (`stage::ground_arrow`).
fn draw_arrows(p: &mut Painter, arrows: &[Arrow], geo: &Geom) {
    let width = if geo.view == View::Stage { ARROW_W_STAGE } else { ARROW_W_FLAT };
    for a in arrows {
        let col = with_alpha(if a.ally { GOLD } else { FOE }, ARROW_ALPHA);
        let at: Vec<(f32, f32)> = a.points.iter().map(|c| ground_pt(geo, *c)).map(|(x, y)| (x, y - 3.0)).collect();
        // the first leg leaves the unit's foot ring, the last stops short of the target's (a leg too short keeps its end)
        let mut pts = at.clone();
        let n = at.len();
        if n >= 2 {
            if let Some((x0, y0, _, _)) = stage::ground_arrow(at[0], at[1]) {
                pts[0] = (x0, y0);
            }
            if let Some((_, _, x1, y1)) = stage::ground_arrow(at[n - 2], at[n - 1]) {
                pts[n - 1] = (x1, y1);
            }
        }
        draw_arrow_line(p, &pts, col, width);
    }
}

// ---- INPUT1-PB: the impact tracers (INPUT1 PB1) ---------------------------------------------------------------------

/// INPUT1 PB1: the tracers of popped action `idx` (tiles by `at`): from its actor to each unit it landed on (an attack's
/// or an area's hit, damage or stagger; a support or an item on another unit, in the heal colour `heal` unless the item
/// harms), dashed to its target when it missed (MR8 c's protected flank, GD5's lost target), to the aim tile an area
/// left its hazard on, and from each riposting unit back to the actor; party GOLD, enemy FOE. None for a self-only
/// command, a cancelled one, or one that failed otherwise.
fn traces_of(s: &Session, idx: usize, at: &dyn Fn(usize) -> Option<V2>, heal: Rgba) -> Vec<fx::Trace> {
    let w = &s.w;
    let mut out: Vec<fx::Trace> = Vec::new();
    let Some(a) = s.resolver.scheduled.get(idx).filter(|a| !a.cancelled) else { return out };
    let Some(from) = at(a.unit) else { return out };
    let c = &a.command;
    let side = |u: usize| if w.allies.contains(&u) { GOLD } else { FOE };
    let put = |out: &mut Vec<fx::Trace>, from: V2, to: Option<V2>, col: Rgba, dashed: bool| {
        if let Some(to) = to.filter(|to| *to != from && !out.iter().any(|t| (t.from, t.to) == (from, *to))) {
            out.push(fx::Trace { from, to, col, dashed });
        }
    };
    let landed: Vec<usize> = c.hits.iter().map(|h| h.0).chain(c.staggers.iter().map(|st| st.target)).filter(|&u| u != a.unit).collect();
    match &c.kind {
        CmdKind::Attack { target, .. } => {
            let missed = c.evaded.or(if c.missed { *target } else { None });
            if let Some(t) = missed {
                put(&mut out, from, at(t), side(a.unit), true);
            }
            for &u in &landed {
                put(&mut out, from, at(u), side(a.unit), false);
            }
        }
        CmdKind::Area { aim, .. } => {
            for &u in &landed {
                put(&mut out, from, at(u), side(a.unit), false);
            }
            // HOOKS1 GH2: the hazard an area leaves lands on its aim tile (rule ground_hazards)
            if c.hazard.as_ref().is_some_and(|h| !h.at_self) && emberwake_battle::ground::on(w) && !c.failed && w.grid.is_valid(*aim) {
                put(&mut out, from, Some(*aim), side(a.unit), false);
            }
        }
        CmdKind::Support { target, .. } if *target != a.unit && !c.failed => put(&mut out, from, at(*target), heal, false),
        CmdKind::Item { target, item, used: true, .. } if *target != a.unit => put(&mut out, from, at(*target), if item.damage > 0 { side(a.unit) } else { heal }, false),
        _ => {}
    }
    for r in &c.reactions {
        if let Some(back) = at(r.defender) {
            put(&mut out, back, at(r.attacker), side(r.defender), false);
        }
    }
    out
}

/// PB1: a tracer's alpha `age` FX-clock ms after its impact: `TRACER_ALPHA` fading to 0 over `TRACER_MS`; under reduce
/// motion held at `TRACER_ALPHA`, then gone.
fn tracer_alpha(age: u64, still: bool) -> u8 {
    if age >= TRACER_MS {
        0
    } else if still {
        TRACER_ALPHA
    } else {
        (TRACER_ALPHA as f32 * (1.0 - age as f32 / TRACER_MS as f32)).round() as u8
    }
}

/// PB1: the live tracers above the tokens, each a straight line between the two tokens' centres shortened by
/// `TRACER_INSET` at each end, 1 px on the Flat board and 1.5 px on the Stage, dashed (4 on, 3 off) for a miss.
fn draw_tracers(p: &mut Painter, tracers: &[(fx::Trace, u64)], geo: &Geom, now: u64, still: bool) {
    let width = if geo.view == View::Stage { ARROW_W_STAGE } else { ARROW_W_FLAT };
    for (t, at) in tracers {
        let a = tracer_alpha(now.saturating_sub(*at), still);
        let ((x0, y0), (x1, y1)) = (fx::anchor(geo, t.from), fx::anchor(geo, t.to));
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if a == 0 || len <= 2.0 * TRACER_INSET {
            continue;
        }
        let (ux, uy) = ((x1 - x0) / len, (y1 - y0) / len);
        let pts = [(x0 + ux * TRACER_INSET, y0 + uy * TRACER_INSET), (x1 - ux * TRACER_INSET, y1 - uy * TRACER_INSET)];
        let col = with_alpha(t.col, a);
        if t.dashed {
            dash_polyline(p, &pts, col, width);
        } else {
            stage::wide_line(p, pts[0].0, pts[0].1, pts[1].0, pts[1].1, col, width, 0.0);
        }
    }
}

// ---- INPUT1-PB: the combat log by round and block (INPUT1 PB4) ---------------------------------------------------

/// PB4: how many of the resolving block's lines the compact log's card lists (its last ones), and how far the expanded
/// log's lines sit in under their part's header (px).
const LOG_CARD_LINES: usize = 12;
const LOG_INDENT: f32 = 6.0;

/// PB4: a log part's header: "Round start", "Block <b>", "Round end".
fn log_part_name(block: i32) -> String {
    match block {
        emberwake_battle::session::LOG_ROUND_START => "Round start".into(),
        emberwake_battle::session::LOG_ROUND_END => "Round end".into(),
        b => format!("Block {b}"),
    }
}

/// PB4: the agent link's `log_groups`: [{round, block (a number, "start" or "end"), lines}].
fn log_groups_json(groups: &[emberwake_battle::session::LogGroup]) -> Value {
    Value::Array(groups.iter().map(|g| {
        let block = match g.block {
            emberwake_battle::session::LOG_ROUND_START => json!("start"),
            emberwake_battle::session::LOG_ROUND_END => json!("end"),
            b => json!(b),
        };
        json!({"round": g.round, "block": block, "lines": g.lines})
    }).collect())
}

/// The arrows for the agent link's state: [{unit, points [[x, y]], alpha}].
fn arrows_json(arrows: &[Arrow]) -> Value {
    Value::Array(arrows.iter().map(|a| json!({"unit": a.unit, "points": a.points.iter().map(|c| json!([c.x, c.y])).collect::<Vec<_>>(), "alpha": ARROW_ALPHA})).collect())
}

// ---- HOOKS1-UI slice 1: ground hazards and the arena telegraph (HOOKS1 GH6, AH3) ------------------------------------

/// The arena telegraph's colour (struck cells and the strip badge).
const ARENA: Rgba = rgb(255, 146, 48);

/// One ground hazard as the party sees it (GH6: a foe's hidden hazard is left out).
struct GroundMark {
    cell: V2,
    id: String,
    name: String,
    tint: Rgba,
    glyph: String,
    /// One of the party's own hidden hazards (the foes do not see it): drawn fainter.
    hidden: bool,
    last_round: i32,
    tip: Tip,
}

/// HOOKS1-UI slice 1: what the hook layers mark on the board and the strip: the ground hazards the party's side sees
/// (rule ground_hazards, layer `hazards`) and the arena hazards due this round (rule arena_hazards, layer `arena`; the
/// kit's tells, AH3). Classic has neither rule, so both stay empty whatever the flags say.
#[derive(Default)]
struct HookMarks {
    ground: Vec<GroundMark>,
    arena: Vec<emberwake_battle::intent::ArenaTell>,
    /// Each arena hazard's effect in words, aligned with `arena`.
    arena_effects: Vec<String>,
}

impl HookMarks {
    fn of(s: &Session, layers: &Layers) -> HookMarks {
        let w = &s.w;
        let mut out = HookMarks::default();
        if layers.hazards && emberwake_battle::ground::on(w) {
            // the party's side sees what any of its members sees (GH6 is per side)
            if let Some(&viewer) = w.allies.first() {
                out.ground = w.grid.ground.iter().filter(|(_, p)| p.seen_by(w, viewer)).map(|(c, p)| ground_mark(w, *c, p)).collect();
            }
        }
        if layers.arena {
            out.arena = emberwake_battle::intent::arena_tells(w);
            out.arena_effects = out.arena.iter().map(|t| w.arena.hazards.iter().find(|h| h.id == t.id).map(arena_effect_text).unwrap_or_default()).collect();
        }
        out
    }

    fn ground_at(&self, c: V2) -> Option<&GroundMark> {
        self.ground.iter().find(|g| g.cell == c)
    }

    /// The tile's hover lines: its ground hazard, then each arena hazard that strikes it this round.
    fn tile_tip(&self, c: V2) -> Tip {
        let mut tip: Tip = self.ground_at(c).map(|g| g.tip.clone()).unwrap_or_default();
        for (t, e) in self.arena.iter().zip(&self.arena_effects).filter(|(t, _)| t.cells.contains(&c)) {
            tip.push((format!("{}: strikes here this round", t.name), ARENA));
            tip.push((format!("At the end of {}", blocks_text(&t.blocks)), TEXT));
            if !e.is_empty() {
                tip.push((e.clone(), TEXT));
            }
        }
        tip
    }

    /// The strip block's arena lines: each hazard that strikes at the end of block `b` this round.
    fn block_tip(&self, b: i32) -> Tip {
        let mut tip: Tip = Vec::new();
        for (t, e) in self.arena.iter().zip(&self.arena_effects).filter(|(t, _)| t.blocks.contains(&b)) {
            tip.push((format!("{} strikes at the end of block {b} ({} cell{})", t.name, t.cells.len(), if t.cells.len() == 1 { "" } else { "s" }), ARENA));
            if !e.is_empty() {
                tip.push((e.clone(), TEXT));
            }
        }
        tip
    }

    /// The agent link's state: the marks as drawn (tiles as [x, y]).
    fn json(&self) -> Value {
        json!({
            "ground": self.ground.iter().map(|g| json!({"at": [g.cell.x, g.cell.y], "id": g.id, "name": g.name, "glyph": g.glyph, "hidden": g.hidden, "last_round": g.last_round})).collect::<Vec<_>>(),
            "arena": emberwake_battle::intent::arena_json(&self.arena),
        })
    }
}

/// "block 3" or "blocks 3 and 6" or "blocks 2, 4 and 6".
fn blocks_text(blocks: &[i32]) -> String {
    let b: Vec<String> = blocks.iter().map(|b| b.to_string()).collect();
    match b.len() {
        0 => "no block".into(),
        1 => format!("block {}", b[0]),
        n => format!("blocks {} and {}", b[..n - 1].join(", "), b[n - 1]),
    }
}

/// An arena hazard's effect in words (AH2: damage through guard, not armour; the status roll; stagger; a push).
fn arena_effect_text(h: &emberwake_battle::arena::Hazard) -> String {
    let e = &h.effect;
    let mut bits: Vec<String> = Vec::new();
    if e.damage > 0 {
        bits.push(format!("{} damage", e.damage));
    }
    if e.damage_pct > 0 {
        bits.push(format!("{}% of max HP", e.damage_pct));
    }
    if !e.status.is_empty() {
        bits.push(format!("{} ({}% chance)", emberwake_rules::status::display_name(&e.status), e.potency));
    }
    if e.stagger > 0 {
        bits.push(format!("stagger {}", e.stagger));
    }
    if let Some(p) = e.push {
        let dir = match (p.dir.x, p.dir.y) {
            (1, _) => "east",
            (-1, _) => "west",
            (_, -1) => "north",
            _ => "south",
        };
        bits.push(format!("pushes {dir} {} tile{}", p.tiles, if p.tiles == 1 { "" } else { "s" }));
    }
    bits.join(", ")
}

/// A ground hazard's mark and hover lines (GH1: its row's numbers; GH7: the round it lasts through).
fn ground_mark(w: &World, cell: V2, p: &emberwake_battle::ground::Placed) -> GroundMark {
    let d = &p.def;
    let placer = w.units.get(p.by).map(|u| u.name.clone()).unwrap_or_default();
    let tint = aegis_ui::vignette::hex(&d.tint).unwrap_or(rgb(200, 120, 220));
    let mut tip: Tip = vec![(d.name.clone(), tint)];
    if !d.description.is_empty() {
        tip.push((d.description.clone(), TEXT));
    }
    let mut does: Vec<String> = Vec::new();
    if d.damage > 0 {
        does.push(format!("{} damage", d.damage));
    }
    if !d.status.is_empty() {
        does.push(format!("{} ({}% chance)", emberwake_rules::status::display_name(&d.status), d.potency));
    }
    if !does.is_empty() {
        let when = match (d.trap, d.on.as_str()) {
            (true, _) => "When sprung",
            (_, "round_end") => "At round end",
            (_, "both") => "On entering and at round end",
            _ => "On entering",
        };
        tip.push((format!("{when}: {}", does.join(", ")), TEXT));
    }
    if d.trap {
        tip.push((format!("Trap: stops the move of the first unit it affects{}, then is gone", if d.stop_moves { " and its later moves this round" } else { "" }), FOE));
    } else if d.stop_moves {
        tip.push(("Stops the later moves this round of a unit that enters".into(), TEXT));
    }
    if d.cover_guard > 0.0 {
        tip.push((format!("Cover {:.0}% against ranged hits", d.cover_guard * 100.0), TEXT));
    }
    if d.move_cost > 0 {
        tip.push((format!("Move cost +{}", d.move_cost), TEXT));
    }
    let affects = match d.affects.as_str() {
        "foes" => format!("Affects {placer}'s foes"),
        "friends" => format!("Affects {placer}'s side"),
        _ => "Affects every unit".into(),
    };
    tip.push((affects, DIM));
    tip.push((format!("Left by {placer} in round {}; lasts through round {}", p.round, p.last_round), DIM));
    if d.hidden {
        tip.push(("Hidden from the foes".into(), DIM));
    }
    GroundMark { cell, id: d.id.clone(), name: d.name.clone(), tint, glyph: d.glyph.clone(), hidden: d.hidden, last_round: p.last_round, tip }
}

/// HOOKS1-UI slice 1: the ground hazards (a tinted plate with the row's glyph; the party's own hidden ones fainter) and
/// the arena's struck cells (an amber plate with a hatch) on the ground, under the tokens and the round overlays.
fn draw_hook_marks(p: &mut Painter, m: &HookMarks, geo: &Geom) {
    let plate_of = |c: V2| match geo.view {
        View::Stage => inset2(geo.cell_rect(c), 1, 1),
        View::Flat => geo.cell_rect(c).inset(1),
    };
    for g in &m.ground {
        let r = plate_of(g.cell);
        let (fill, edge) = if g.hidden { (36, 120) } else { (70, 200) };
        plate(p, r, with_alpha(g.tint, fill), with_alpha(g.tint, edge));
        if !g.glyph.is_empty() {
            let cy = r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0;
            p.text_center(r.x as f32 + r.w as f32 / 2.0, cy, &g.glyph, SMALL, with_alpha(TEXT, if g.hidden { 150 } else { 235 }));
        }
    }
    let mut struck: Vec<V2> = Vec::new();
    for c in m.arena.iter().flat_map(|t| &t.cells) {
        if !struck.contains(c) {
            struck.push(*c);
        }
    }
    for c in struck {
        let r = plate_of(c).inset(1);
        plate(p, r, with_alpha(ARENA, 34), with_alpha(ARENA, 190));
        // a hatch: two short strokes in the lower right corner, so a tinted ground hazard under it still reads
        let (x1, y1) = ((r.x + r.w) as f32 - 2.0, (r.y + r.h) as f32 - 2.0);
        for k in [5.0, 9.0] {
            p.line(x1 - k, y1, x1, y1 - k * 0.6, with_alpha(ARENA, 200), 1.0);
        }
    }
}

/// HOOKS1-UI slice 1: the arena badge on the strip block `r` (a hazard strikes at its end).
fn arena_badge(p: &mut Painter, r: Rect) {
    let b = Rect::new(r.x + r.w - 7, r.y + 1, 6, 6);
    p.panel(b, ARENA, rgba(60, 26, 6, 255), 1.0);
}

// ---- ENCOUNTER2-U: the hold escape's pick, tiles and gate (ENCOUNTER2 H1, H3, H7) -----------------------------------

/// A hold escape's marks as drawn: the tiles (layer `hold`), the gate's state (layer `gate`), the holder and the label.
struct HoldMarks {
    tiles: Vec<V2>,
    label: String,
    holder: Option<usize>,
    open: bool,
    show_tiles: bool,
    show_gate: bool,
}

impl HoldMarks {
    /// The session's hold gate (`World::hold`, the kit's `HoldGate`); None without a hold escape (classic and every other
    /// fight draw as before).
    fn of(s: &Session, layers: &Layers) -> Option<HoldMarks> {
        let gate = s.w.hold.as_ref()?;
        Some(HoldMarks { tiles: gate.tiles.clone(), label: gate.label.clone(), holder: gate.holder, open: gate.is_open(&s.w), show_tiles: layers.hold, show_gate: layers.gate })
    }

    /// A tile's hover lines: a hold tile names its holder; an exit tells the gate's state (`HoldGate::note`'s words).
    fn tile_tip(&self, w: &World, c: V2) -> Tip {
        let mut tip: Tip = Vec::new();
        if self.show_tiles && self.tiles.contains(&c) {
            let mut head = self.label.chars();
            tip.push((head.next().map(|f| f.to_uppercase().collect::<String>() + head.as_str()).unwrap_or_default(), GOLD));
            tip.push((
                match self.holder {
                    Some(h) => format!("{} holds it: the gate is open while they stand here, and they stay behind when the others escape.", w.units[h].name),
                    None => "Pick who holds it before round 1.".into(),
                },
                TEXT,
            ));
        }
        if self.show_gate && w.grid.is_exit(c) {
            let note = w.hold.as_ref().map(|g| g.note(w)).unwrap_or_default();
            tip.push(if self.open { ("The gate is open.".into(), GOOD) } else if note.is_empty() { (format!("The gate is shut until someone holds {}.", self.label), FOE) } else { (note, FOE) });
        }
        tip
    }
}

/// "Kaelen holds the winch." (ENCOUNTER2 H7's objective line; "" before the pick or without a hold escape).
fn hold_line(s: &Session) -> String {
    match s.w.hold.as_ref().and_then(|g| g.holder.map(|h| (h, &g.label))) {
        Some((h, label)) => format!("{} holds {label}.", s.w.units[h].name),
        None => String::new(),
    }
}

/// The heroes a hold pick offers (H1: the hold escape's heroes on the field, party order).
fn hold_pick_ids(s: &Session) -> Vec<usize> {
    s.w.hold.as_ref().map(|g| g.heroes.iter().copied().filter(|&c| s.w.has(c)).collect()).unwrap_or_default()
}

/// An exit's sign: EXIT (green) without a hold escape or with the `gate` layer off; else SHUT (red) or OPEN (green).
fn exit_sign(hold: Option<&HoldMarks>) -> (&'static str, Rgba, Rgba) {
    let green = (rgba(70, 200, 90, 230), rgba(200, 255, 210, 255));
    match hold.filter(|h| h.show_gate) {
        Some(h) if !h.open => ("SHUT", rgba(200, 60, 56, 235), rgba(255, 210, 200, 255)),
        Some(_) => ("OPEN", green.0, green.1),
        None => ("EXIT", green.0, green.1),
    }
}

/// H7: each hold tile with a gold objective ring and a plate with the label's first letter ("W" for the winch).
fn draw_hold_tiles(p: &mut Painter, h: &HoldMarks, geo: &Geom) {
    if !h.show_tiles {
        return;
    }
    let letter: String = h.label.trim_start_matches("the ").chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    for c in &h.tiles {
        let r = match geo.view {
            View::Stage => inset2(geo.cell_rect(*c), 1, 1),
            View::Flat => geo.cell_rect(*c).inset(1),
        };
        plate(p, r, rgba(244, 198, 84, 36), rgba(244, 198, 84, 230));
        plate(p, r.inset(2), rgba(0, 0, 0, 0), rgba(244, 198, 84, 120));
        if !letter.is_empty() {
            let b = Rect::new(r.x + 2, r.y + 2, 9, 9);
            p.panel(b, rgba(60, 44, 12, 235), GOLD, 2.0);
            p.text_center(b.x as f32 + 4.5, b.y as f32 + (b.h as f32 - p.line_h(SMALL)) / 2.0, &letter, SMALL, GOLD);
        }
    }
}

/// H1: "Who holds the winch?" with a button per hero on the field (the host's `stay_pick`); over the board's lower edge
/// so the board and the hold tiles stay in view.
fn draw_hold_pick(ui: &mut Ui<Action>, s: &Session) {
    let Some(gate) = s.w.hold.as_ref() else { return };
    let ids = hold_pick_ids(s);
    let title = format!("Who holds {}?", gate.label);
    let labels: Vec<String> = ids.iter().map(|&c| format!("{}  HP {}/{}", s.w.units[c].name, s.w.units[c].hp, s.w.units[c].max_hp)).collect();
    let widths: Vec<i32> = labels.iter().map(|l| (ui.p.text_w(l, BODY) + 12.0).ceil() as i32).collect();
    let inner = widths.iter().sum::<i32>() + 4 * (ids.len() as i32 - 1).max(0);
    let w = (inner.max((ui.p.text_w(&title, TITLE) + 12.0).ceil() as i32) + 12).min(GRID_W * TILE - 8);
    let h = 18 + ui.p.line_h(TITLE).ceil() as i32;
    let r = Rect::new(GRID_X + (GRID_W * TILE - w) / 2, GRID_Y + GRID_H * TILE - h - 6, w, h);
    ui.p.panel(r, rgba(14, 18, 30, 240), GOLD, RADIUS);
    ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 3.0, &title, TITLE, GOLD);
    ui.push("hold.pick", "panel", title.clone(), r, true, false, None, vec![(title.clone(), GOLD), (format!("The holder keeps {} so the gate stays open; they stay behind when the others escape.", gate.label), TEXT)]);
    let mut x = r.x + (r.w - inner) / 2;
    let y = r.y + r.h - 16;
    for ((c, label), bw) in ids.iter().zip(&labels).zip(&widths) {
        let tip = vec![(format!("{} holds {}", s.w.units[*c].name, gate.label), GOLD), ("They walk to it and guard it; the gate is open while they stand on it.".into(), TEXT)];
        ui.button_full(&format!("hold.pick.{c}"), Rect::new(x, y, *bw, 14), label, Action::StayPick(*c), true, false, BODY, tip);
        x += bw + 4;
    }
}

// ---- PORTRAITFX1-P: the panel portraits (6.2 row 3) --------------------------------------------------------------

/// PORTRAITFX1-P: `who`'s portrait at panel `r`'s top-left (`PANEL_PORTRAIT` px, `PANEL_INSET` in) on the talk box's
/// dark panel, with `look`'s state rows on surface `panel` (the initial letter without a file); its widget `id` (kind
/// `portrait`, no action). The portrait's rect.
#[allow(clippy::too_many_arguments)]
fn panel_portrait(ui: &mut Ui<Action>, cx: &mut Cx, faces: &mut Faces, who: &Who, look: &emberwake_battle::portrait::Look, r: Rect, id: &str) -> Rect {
    let pr = Rect::new(r.x + PANEL_INSET, r.y + PANEL_INSET, PANEL_PORTRAIT, PANEL_PORTRAIT);
    ui.p.panel(pr, rgb(22, 30, 48), BORDER, 2.0);
    faces.draw(&mut ui.p, cx, who, look, PANEL_SURFACE, pr);
    ui.push(id, "portrait", who.name.clone(), pr, true, false, None, Vec::new());
    pr
}

/// PORTRAITFX1-TL (user 2026-11-01): an enemy's timeline row holds no state cue (no fade, no wound tint, no other
/// hold) while the view masks its track (an unrevealed plan, TV4) or it is stealthed (the board's stealth veil, GD5);
/// party rows always show theirs.
fn timeline_row_plain(s: &Session, u: usize, masked: bool) -> bool {
    !s.w.allies.contains(&u) && (masked || s.w.units[u].has_status("stealth"))
}

/// `look` with no state a cue row can hold on but down (the board shows a fall): full HP, no statuses, conditions or
/// derived stats.
fn plain_look(look: &emberwake_battle::portrait::Look) -> emberwake_battle::portrait::Look {
    emberwake_battle::portrait::Look { hp: look.max_hp, max_hp: look.max_hp, down: look.down, statuses: Vec::new(), conditions: Vec::new(), derived: Vec::new(), ruleset: look.ruleset.clone(), unspent: 0 }
}

/// PORTRAITFX1-TL (6.2 row 4): `who`'s portrait in a timeline track's icon square `sq` (the widget's
/// `timeline.icon.<track>`), on the panels' dark backing, with `look`'s state rows on surface `timeline` through
/// `faces` (capped at the minimal tier); the initial letter without a file.
fn timeline_portrait(ui: &mut Ui<Action>, cx: &mut Cx, faces: &mut Faces, who: &Who, look: &emberwake_battle::portrait::Look, sq: Rect) {
    ui.p.panel(sq, rgb(22, 30, 48), BORDER, 2.0);
    faces.draw(&mut ui.p, cx, who, look, TIMELINE_SURFACE, Rect::new(sq.x + 1, sq.y + 1, sq.w - 2, sq.h - 2));
}

// ---- HOOKS1-UI slice 5: the intel panel fits its lines ------------------------------------------------------------

/// The intel panel's line steps: its head (BODY) and the rest (SMALL), and its text width.
const INTEL_HEAD: f32 = 10.0;
const INTEL_LINE: f32 = 8.0;
const INTEL_W: f32 = 108.0;
/// The panel's last line when its lines do not all fit (the unit's hover card has them all).
const INTEL_MORE: &str = "More on its hover card";

/// HOOKS1-UI slice 5 (decision "RULESET1-B2 tuning and HOOKS1-UI slice 3 answers"): the intel panel's wrapped lines
/// that fit in `room` px ((text, colour, head)): when they run past it the "Level ..." line (level, class, speed, reach)
/// gives way first; when they still do not fit the panel keeps what fits and ends with `INTEL_MORE`.
#[cfg(test)]
fn intel_parts(p: &mut Painter, lines: &[(String, Rgba)], room: f32) -> Vec<(String, Rgba, bool)> {
    intel_parts_at(p, lines, room, 0.0).into_iter().map(|(t, c, h, _)| (t, c, h)).collect()
}

/// `intel_parts` beside the panel's portrait (PORTRAITFX1-P): a line that starts within the first `beside` px of the
/// text wraps to the room right of the portrait and is marked narrow (drawn `PANEL_TEXT_X` in).
fn intel_parts_at(p: &mut Painter, lines: &[(String, Rgba)], room: f32, beside: f32) -> Vec<(String, Rgba, bool, bool)> {
    let narrow_w = INTEL_W - (PANEL_TEXT_X - 4) as f32;
    let wrap = |p: &mut Painter, lines: &[(String, Rgba)]| {
        let mut out = Vec::new();
        let mut at = 0.0;
        for (i, (t, col)) in lines.iter().enumerate() {
            let narrow = at < beside;
            for part in p.wrap(t, if i == 0 { BODY } else { SMALL }, if narrow { narrow_w } else { INTEL_W }) {
                at += if i == 0 { INTEL_HEAD } else { INTEL_LINE };
                out.push((part, *col, i == 0, narrow));
            }
        }
        out
    };
    let height = |parts: &[(String, Rgba, bool, bool)]| parts.iter().map(|x| if x.2 { INTEL_HEAD } else { INTEL_LINE }).sum::<f32>();
    let mut parts = wrap(p, lines);
    if height(&parts) > room {
        let kept: Vec<(String, Rgba)> = lines.iter().enumerate().filter(|(i, l)| *i == 0 || !l.0.starts_with("Level ")).map(|(_, l)| l.clone()).collect();
        parts = wrap(p, &kept);
    }
    if height(&parts) > room {
        while !parts.is_empty() && height(&parts) + INTEL_LINE > room {
            parts.pop();
        }
        let narrow = height(&parts) < beside;
        parts.push((INTEL_MORE.to_string(), DIM, false, narrow));
    }
    parts
}

// ---- HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5) ---------------------------------------------------------

/// A push forecast's arrow.
const PUSH: Rgba = rgb(110, 228, 210);

/// HOOKS1-UI slice 4: the forced moves each of hero `h`'s planned entries would make if it struck now (aligned with its
/// compiled plan): the kit's `forced::forecast` from the tile the entry strikes from at its last block, on the unit
/// an attack targets or the units an area's cells hold. Empty without the rule forced_move.
fn push_forecasts(s: &Session, h: usize) -> Vec<Vec<emberwake_battle::forced::Forecast>> {
    let w = &s.w;
    if !emberwake_battle::forced::on(w) {
        return Vec::new();
    }
    s.plan
        .compile(w, h)
        .entries
        .iter()
        .map(|e| {
            let cmd = e.build_command(w, h);
            let targets: Vec<usize> = match &cmd.kind {
                CmdKind::Attack { target, .. } => target.iter().copied().collect(),
                CmdKind::Area { cells, .. } => cells.iter().filter_map(|c| w.occupant(*c)).collect(),
                _ => Vec::new(),
            };
            emberwake_battle::forced::forecast(w, &s.resolver.scheduled, h, strike_tile(e), &cmd, e.end_block(w), &targets)
        })
        .collect()
}

/// A forecast in words: "Pushes Brawler 2 tiles to (7, 4); then hits the wall", "Brawler holds firm".
fn push_text(w: &World, f: &emberwake_battle::forced::Forecast) -> (String, Rgba) {
    let name = &w.units[f.unit].name;
    if f.held {
        return (format!("{name} holds firm (no {})", if f.pull { "pull" } else { "push" }), DIM);
    }
    let n = f.path.len();
    let verb = if f.pull { "Pulls" } else { "Pushes" };
    let mut line = if n == 0 { format!("{verb} {name} into {} at once (collision)", f.blocked_by) } else { format!("{verb} {name} {n} tile{} to {}", if n == 1 { "" } else { "s" }, f.to) };
    if n > 0 && !f.blocked_by.is_empty() {
        line += &format!("; then hits {} (collision)", f.blocked_by);
    }
    (line, if f.blocked_by.is_empty() { PUSH } else { FOE })
}

/// The agent link's record of a forecast (tiles as [x, y]).
fn push_json(f: &emberwake_battle::forced::Forecast) -> Value {
    let cell = |c: &V2| json!([c.x, c.y]);
    json!({"unit": f.unit, "pull": f.pull, "from": cell(&f.from), "to": cell(&f.to), "path": f.path.iter().map(cell).collect::<Vec<_>>(), "held": f.held, "blocked_by": f.blocked_by, "blocker": f.blocker})
}

/// The forecasts over the board: an arrow from the unit along its path with a head at its end; a collision's red cross
/// at the edge of the refused tile; a hold's HOLD plate.
fn draw_pushes(p: &mut Painter, list: &[emberwake_battle::forced::Forecast], geo: &Geom) {
    for f in list {
        let (fx, fy) = ground_pt(geo, f.from);
        if f.held {
            let tw = p.text_w("HOLD", SMALL).ceil() as i32 + 6;
            let r = Rect::new(fx.round() as i32 - tw / 2, fy.round() as i32 - 5, tw, 9);
            p.panel(r, rgba(30, 70, 110, 235), HELD, 2.0);
            p.text_center(fx, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, "HOLD", SMALL, TEXT);
            continue;
        }
        let (tx, ty) = ground_pt(geo, f.to);
        if f.to != f.from {
            stage::wide_line(p, fx, fy, tx, ty, PUSH, 2.0, 0.0);
            let (dx, dy) = (tx - fx, ty - fy);
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let (ux, uy) = (dx / len, dy / len);
            for s in [1.0f32, -1.0] {
                stage::wide_line(p, tx, ty, tx - ux * 6.0 - uy * 4.0 * s, ty - uy * 6.0 + ux * 4.0 * s, PUSH, 2.0, 0.0);
            }
        }
        if !f.blocked_by.is_empty() {
            let (nx, ny) = ground_pt(geo, f.to + f.dir);
            let (cx, cy) = ((tx + nx) / 2.0, (ty + ny) / 2.0);
            // a dark-edged cross so it reads over the tokens
            for (col, wd) in [(rgba(20, 8, 10, 230), 3.0), (FOE, 1.6)] {
                for s in [1.0f32, -1.0] {
                    stage::wide_line(p, cx - 6.0, cy - 6.0 * s, cx + 6.0, cy + 6.0 * s, col, wd, 0.0);
                }
            }
        }
    }
}

// ---- HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3) ------------------------------------------------------

/// A meter's pips and bar.
const METER: Rgba = rgb(196, 128, 255);
/// A held guard's card edge and hover line.
const HELD: Rgba = rgb(110, 190, 255);

/// Unit `i`'s meters while the `meters` layer is on and it is active (`meter::shown`: none without the rule unit_meters,
/// a `hidden` meter left out).
fn meters_of(w: &World, i: usize, layers: &Layers) -> Vec<emberwake_battle::meter::Shown> {
    if !layers.meters || !w.units[i].is_active() {
        return Vec::new();
    }
    emberwake_battle::meter::shown(w, i)
}

/// "Fury 3/5 (Hot)": a meter's name, value, max and band.
fn meter_text(m: &emberwake_battle::meter::Shown) -> String {
    let band = if m.band.is_empty() { String::new() } else { format!(" ({})", m.band) };
    format!("{} {}/{}{band}", m.name, m.value, m.max)
}

/// Up to two meters side by side in `r`: pips (one per step from min to max, the value's filled) or a bar.
fn meter_row(p: &mut Painter, r: Rect, ms: &[emberwake_battle::meter::Shown]) {
    let n = ms.len().min(2) as i32;
    if n == 0 {
        return;
    }
    let seg = (r.w - (n - 1) * 4) / n;
    for (k, m) in ms.iter().take(2).enumerate() {
        let x = r.x + k as i32 * (seg + 4);
        let steps = (m.max - m.min).max(1);
        let filled = (m.value - m.min).clamp(0, steps);
        if m.show == "pips" {
            let pw = ((seg - (steps - 1)) / steps).max(1);
            for j in 0..steps {
                p.fill(Rect::new(x + j * (pw + 1), r.y, pw, r.h), if j < filled { METER } else { rgb(44, 36, 62) });
            }
        } else {
            p.fill(Rect::new(x, r.y, seg, r.h), rgb(44, 36, 62));
            p.fill(Rect::new(x, r.y, seg * filled / steps, r.h), METER);
        }
    }
}

/// HOOKS1 8.3: unit `i`'s held guard now (`trigger::held_at` at the session's block; None without the rule triggers,
/// for a unit not on the field, or once it ends).
fn held_guard(s: &Session, i: usize) -> Option<emberwake_rules::unit::HeldGuard> {
    let w = &s.w;
    if !emberwake_battle::trigger::on(w) || !w.units[i].is_active() {
        return None;
    }
    emberwake_battle::trigger::held_at(w, i, s.current_block)?;
    w.units[i].held_guard
}

/// The HP cost of the skill a popup step names (the host's `skills` reply).
fn skills_hp(skills: &[Value], step: &Value) -> Option<i64> {
    skills.iter().find(|k| k["skill"] == step["skill"]).and_then(|k| k["hp_cost"].as_i64()).filter(|c| *c > 0)
}

// ---- RULESET1-U slice 5: poise, breaks, the strip's parts, forecasts, the round summary ----------------------------

/// Unit `i`'s poise bar while its layer is on and it has one (rule poise; never in classic).
fn poise_shown(w: &World, i: usize, layers: &Layers) -> Option<emberwake_battle::poise::PoiseState> {
    emberwake_battle::poise::state(w, i).filter(|st| layers.poise && st.max > 0 && w.units[i].is_active()).copied()
}

/// A poise bar in `r`: the share of Max Poise left in `POISE` (a windup bonus over the max shows as a gold tip); empty
/// on a red ground once broken this round.
fn poise_bar(p: &mut Painter, r: Rect, st: &emberwake_battle::poise::PoiseState) {
    p.fill(r, if st.broken { rgba(120, 30, 34, 255) } else { rgb(28, 34, 48) });
    if st.broken || st.max <= 0 {
        return;
    }
    let w = (r.w * st.value.clamp(0, st.max) + st.max - 1) / st.max;
    p.fill(Rect::new(r.x, r.y, w.min(r.w), r.h), POISE);
    if st.value > st.max {
        p.fill(Rect::new(r.x + r.w - 2, r.y, 2, r.h), GOLD);
    }
}

/// The units broken this round (P7), while the breaks layer is on.
fn broken_units(w: &World, layers: &Layers) -> Vec<usize> {
    if !layers.breaks {
        return Vec::new();
    }
    (0..w.units.len()).filter(|&i| w.units[i].is_active() && w.has(i) && emberwake_battle::poise::state(w, i).is_some_and(|st| st.broken)).collect()
}

/// The BREAK plate centred on x with its top at y (air layer, over the tokens).
fn break_plate(p: &mut Painter, x: f32, y: f32) {
    let tw = p.text_w("BREAK", SMALL).ceil() as i32 + 6;
    let r = Rect::new(x.round() as i32 - tw / 2, y.round() as i32, tw, 9);
    p.panel(r, rgba(150, 36, 40, 235), rgba(255, 220, 210, 230), 2.0);
    p.text_center(x, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, "BREAK", SMALL, TEXT);
}

/// A strip span's parts in the timeline widget's look (wind-up, impact, recovery) and whether a break cancelled it.
struct SpanParts {
    parts: Vec<aegis_ui::timeline::Part>,
    broken: bool,
}

/// RULESET1-U slice 5: the parts of the selected hero's strip spans (aligned with `entries`, the strip's spans):
/// planning, from its plan (a command that winds up, P5, longer than a block: its wind-up blocks, then its impact
/// block); once confirmed, from its scheduled actions (start, impact, recovery to its end; broken by a break, P7).
fn span_parts(s: &Session, u: usize, entries: &[(i32, i32, String, String, Option<String>, i32, i32)]) -> Vec<SpanParts> {
    use crate::timeline_view::parts_of;
    let w = &s.w;
    if s.plan_confirmed {
        return entries
            .iter()
            .map(|e| match s.resolver.scheduled.iter().find(|a| a.unit == u && a.start == e.0 && a.command.name == e.2) {
                Some(a) => SpanParts { parts: parts_of(a.start, a.impact, e.1, emberwake_battle::poise::winds_up(&a.command)), broken: a.broken },
                None => SpanParts { parts: Vec::new(), broken: false },
            })
            .collect();
    }
    s.plan.compile(w, u).entries.iter().map(|e| SpanParts { parts: parts_of(e.start_block, e.end_block(w), e.end_block(w), emberwake_battle::poise::winds_up(&e.build_command(w, u))), broken: false }).collect()
}

/// One planned hit's forecast (RULESET1 12 "forecasts").
struct HitForecast {
    target: usize,
    skill: String,
    block: i32,
    stagger: i32,
    fc: emberwake_battle::poise::Forecast,
    flank: bool,
    pierce: bool,
}

impl HitForecast {
    /// The strip tip's lines: the stagger and its factors, the verdict, flank and pierce.
    fn lines(&self, w: &World) -> Vec<(String, Rgba)> {
        let f = &self.fc.factors;
        let mut why = String::new();
        for (on, name, m) in [(f.windup != 1.0, "in wind-up", f.windup), (f.guard != 1.0, "guarding", f.guard), (f.repeat != 1.0, "broke last round", f.repeat), (f.flank != 1.0, "flank", f.flank), (f.passive != 1.0, "trait", f.passive), (f.hook != 1.0, "hook", f.hook)] {
            if on {
                why += &format!(" x {name} {m}");
            }
        }
        let mut out = vec![(format!("Forecast on {}", w.units[self.target].name), GOLD)];
        if self.staggers() {
            out.push((format!("Stagger {}{why} = {}", self.stagger, self.fc.taken), TEXT));
            out.push(self.verdict());
        }
        if self.flank {
            out.push(("Flank".into(), GOOD));
        }
        if self.pierce {
            out.push(("Armour pierced".into(), GOOD));
        }
        out
    }

    /// The verdict line ("Breaks: poise 20 -> 0", "May break: 40%", "Poise 40 -> 28").
    fn verdict(&self) -> (String, Rgba) {
        use emberwake_battle::poise::{BreakCause, Verdict};
        let (b, a) = (self.fc.before, self.fc.after);
        match self.fc.verdict {
            Verdict::Breaks(BreakCause::Sure) => ("Breaks: a sure break into its wind-up".into(), FOE),
            Verdict::Breaks(_) => (format!("Breaks: poise {b} -> 0"), FOE),
            Verdict::Chance(c) => (format!("May break: {c}% (poise {b} -> {a})"), GOLD),
            Verdict::Holds => (format!("Poise {b} -> {a}"), TEXT),
            Verdict::AlreadyBroken => ("Already broken this round".into(), DIM),
        }
    }

    /// Whether the hit touches poise at all (stagger, or a sure break).
    fn staggers(&self) -> bool {
        self.stagger > 0 || matches!(self.fc.verdict, emberwake_battle::poise::Verdict::Breaks(emberwake_battle::poise::BreakCause::Sure))
    }

    /// One intel panel line: "Your Jab B3: in wind-up, breaks: poise 20 -> 0, flank" style.
    fn short(&self, _w: &World) -> (String, Rgba) {
        let mut bits: Vec<String> = Vec::new();
        let mut col = TEXT;
        if self.staggers() {
            let (v, c) = self.verdict();
            col = c;
            if self.fc.windup {
                bits.push("in wind-up".into());
            }
            bits.push(v.to_lowercase());
        }
        if self.flank {
            bits.push("flank".into());
        }
        if self.pierce {
            bits.push("armour pierced".into());
        }
        (format!("Your {} B{}: {}", self.skill, self.block, bits.join(", ")), col)
    }
}

/// RULESET1-U slice 5: the forecast of each of hero `h`'s planned entries (aligned with its compiled plan; None for
/// anything but an attack on a unit with a poise bar): the kit's `poise::forecast` against the enemies' scheduled
/// actions, on the bar the hero's earlier planned hits leave (one break a round, P8), flank from the tile the hit
/// strikes from (`round_intent::flank_bonus_from`). Empty without the rule poise.
fn hit_forecasts(s: &Session, h: usize) -> Vec<Option<HitForecast>> {
    use emberwake_battle::poise;
    let w = &s.w;
    if !poise::on(w) {
        return Vec::new();
    }
    // the planned attacks on a unit, in plan order, forecast as a sequence by the kit (each on the bar the earlier
    // ones leave: slice 6)
    let st = s.plan.compile(w, h);
    let planned: Vec<Option<(usize, emberwake_battle::command::Command, i32, bool)>> = st
        .entries
        .iter()
        .map(|e| {
            let t = e.target.filter(|&t| t != h)?;
            if emberwake_battle::skill::kind(&w.cat.skills[e.skill]) != Kind::Attack {
                return None;
            }
            Some((t, e.build_command(w, h), e.end_block(w), ri::flank_bonus_from(w, h, strike_tile(e), t)))
        })
        .collect();
    let hits: Vec<poise::PlannedHit> = planned.iter().flatten().map(|(t, cmd, block, flank)| poise::PlannedHit { target: *t, block: *block, impact: cmd.impact, blocks: cmd.block_cost, flank: *flank, ignore_guard: cmd.ignore_guard }).collect();
    let mut fcs = poise::forecast_hits(w, &s.resolver.scheduled, &hits).into_iter();
    st.entries
        .iter()
        .zip(&planned)
        .map(|(e, p)| {
            let (t, cmd, block, flank) = p.as_ref()?;
            let fc = fcs.next()??;
            let f = HitForecast { target: *t, skill: w.cat.skills[e.skill].name.clone(), block: *block, stagger: cmd.impact.stagger, fc, flank: *flank, pierce: cmd.pierces(w) };
            // a hit with nothing to say (no stagger, no flank, no pierce) has no forecast
            (f.staggers() || f.flank || f.pierce).then_some(f)
        })
        .collect()
}

/// RULESET1 12: the round summary in words ("1 break (1 of 2 hits into a wind-up), 1 flank, 3 follow-ups, Rest +1
/// AP"; "no breaks" alone when nothing happened); None without the round intent or with the summary layer off.
fn round_summary_text(s: &Session, layers: &Layers) -> Option<String> {
    if !ri::on(&s.w) || !layers.summary {
        return None;
    }
    Some(ri::summary_text(&ri::round_summary(&s.w, &s.resolver.scheduled)))
}

/// The overlays of the battle now: None without the rule `round_intent` (classic draws none).
fn overlay_of(s: &Session, sel: Option<usize>, planning: bool) -> Option<Overlay> {
    let w = &s.w;
    if !ri::on(w) {
        return None;
    }
    let on_field = |u: usize| w.units[u].is_active() && w.has(u);
    let mut o = Overlay {
        facings: w.enemies.iter().copied().filter(|&e| on_field(e)).map(|e| (w.pos(e), ri::facing(w, e))).filter(|(_, f)| *f != emberwake_battle::grid::ZERO).collect(),
        veils: (0..w.units.len()).filter(|&u| on_field(u) && w.units[u].has_status("stealth")).map(|u| (w.pos(u), ri::hidden(w, u))).collect(),
        ..Overlay::default()
    };
    let Some(h) = sel.filter(|&h| planning && on_field(h) && w.allies.contains(&h)) else { return Some(o) };
    let target = w.round_states.get(&h).and_then(|r| r.target);
    o.target = match target {
        Some(RoundTarget::Unit(t)) if on_field(t) => Some(w.pos(t)),
        Some(RoundTarget::Cell(p)) => Some(p),
        _ => None,
    };
    let st = s.plan.compile(w, h);
    // the planned moves: a carried move rides in its skill's entry (Engage, a chase), a retreat or a move after the
    // round's first command is an after-move
    let mut from = w.pos(h);
    let mut acted = false;
    for e in &st.entries {
        let kind = emberwake_battle::skill::kind(&w.cat.skills[e.skill]);
        if e.pos_after != from && e.pos_after.x >= 0 {
            // the entry's path is its steps (direction vectors) from where it starts (the slice 3 review: the line
            // starts there too, not at the last drawn tile)
            let start = if w.grid.is_valid(e.pos_before) { e.pos_before } else { from };
            let mut tiles = vec![start];
            let mut at = start;
            for d in &e.path {
                at = at + *d;
                if tiles.last() != Some(&at) && w.grid.is_valid(at) {
                    tiles.push(at);
                }
            }
            if tiles.last() != Some(&e.pos_after) {
                tiles.push(e.pos_after);
            }
            let how = match (&e.carry, kind) {
                (emberwake_battle::command::Carry::Retreat(_), _) => MoveKind::After,
                (_, Kind::Move) if acted => MoveKind::After,
                (_, Kind::Move) => MoveKind::Walk,
                _ => MoveKind::Carried,
            };
            o.moves.push((tiles, how));
            from = e.pos_after;
        }
        if !matches!(kind, Kind::Move | Kind::Rest) {
            acted = true;
        }
    }
    let Some(intent) = ri::intent_of(w, h) else { return Some(o) };
    o.valid = (0..w.units.len())
        .filter(|&u| on_field(u) && Some(ri::normalize(h, RoundTarget::Unit(u))) != target)
        .filter(|&u| ri::target_problem(w, h, intent, RoundTarget::Unit(u)).is_empty())
        .map(|u| w.pos(u))
        .collect();
    let all_cells = || (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c));
    if matches!(intent, RoundIntent::Assault | RoundIntent::Maneuver) {
        let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[h], &w.ruleset);
        let at = st.entries.iter().find(|e| emberwake_battle::skill::kind(&w.cat.skills[e.skill]) == Kind::Attack).map(strike_tile).unwrap_or_else(|| w.pos(h));
        o.band = all_cells().filter(|c| (lo..=hi).contains(&c.dist(at))).collect();
    }
    if let (RoundIntent::Maneuver, Some(RoundTarget::Unit(t))) = (intent, target) {
        if on_field(t) {
            let tp = w.pos(t);
            o.front = all_cells().filter(|c| c.dist(tp) <= FRONT_RADIUS && *c != tp && ri::in_front(w, t, *c)).collect();
        }
    }
    Some(o)
}

/// The tile an attack entry strikes from: after its carried Engage move (MV4: the move comes before the hit), before
/// its Disengage retreat (MV8: the retreat comes after the hit).
fn strike_tile(e: &emberwake_battle::plan::PlanEntry) -> V2 {
    if matches!(e.carry, emberwake_battle::command::Carry::Retreat(_)) {
        e.pos_before
    } else {
        e.pos_after
    }
}

/// The overlays for the agent link's state: tiles as [x, y].
fn overlay_json(o: &Overlay) -> Value {
    let cell = |c: &V2| json!([c.x, c.y]);
    json!({
        "target": o.target.as_ref().map(cell),
        "valid": o.valid.iter().map(cell).collect::<Vec<_>>(),
        "band": o.band.iter().map(cell).collect::<Vec<_>>(),
        "front": o.front.iter().map(cell).collect::<Vec<_>>(),
        "moves": o.moves.iter().map(|(t, k)| json!({"kind": k.id(), "tiles": t.iter().map(cell).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "facings": o.facings.iter().map(|(c, f)| json!({"at": cell(c), "facing": [f.x, f.y]})).collect::<Vec<_>>(),
        "veils": o.veils.iter().map(|(c, hidden)| json!({"at": cell(c), "hidden": hidden})).collect::<Vec<_>>(),
        "arrows": arrows_json(&o.arrows),
        "positions": o.positions.iter().map(|(c, preferred)| json!({"at": cell(c), "preferred": preferred})).collect::<Vec<_>>(),
    })
}

/// The ground point of a cell in the view: the foot point (Stage), the cell's lower part (Flat, where the token's
/// foot ellipse sits).
fn ground_pt(geo: &Geom, c: V2) -> (f32, f32) {
    let (x, y) = geo.foot(c);
    match geo.view {
        View::Stage => (x as f32, y as f32),
        View::Flat => (x as f32, (geo.cell_rect(c).y + geo.cell_rect(c).h) as f32 - 4.0),
    }
}

/// RULESET1-U slice 3: draws the overlays `o` (as `Overlay::shown` keeps them) on the ground, under the tokens (Flat:
/// after the tiles; Stage: the markers pass), in the order back to front: the reach band, the Maneuver front, the valid
/// targets, the stealth veils, the moves' destination plates (VISCLAR1 AR5: the lines give way to the target arrows,
/// `draw_arrows`; Q14: an after-move shows as its plate only), the target ring, the enemy facings.
fn draw_overlay(p: &mut Painter, o: &Overlay, geo: &Geom) {
    let plate_of = |c: V2| match geo.view {
        View::Stage => inset2(geo.cell_rect(c), 1, 1),
        View::Flat => geo.cell_rect(c).inset(1),
    };
    for c in &o.band {
        plate(p, plate_of(*c), rgba(110, 170, 255, 22), rgba(110, 170, 255, 70));
    }
    for c in &o.front {
        plate(p, plate_of(*c), rgba(240, 100, 92, 46), rgba(240, 100, 92, 110));
    }
    // a ground ellipse's radii in the view (the Stage squashes it like the foot rings)
    let (rx, ry) = match geo.view {
        View::Stage => (0.36 * geo.token_px as f32, 4.2),
        View::Flat => (13.0, 5.0),
    };
    for c in &o.valid {
        let (x, y) = ground_pt(geo, *c);
        p.ellipse_ring(x, y, rx + 1.0, ry + 0.8, rgba(255, 255, 255, 150));
    }
    for (c, hidden) in &o.veils {
        let (x, y) = ground_pt(geo, *c);
        p.ellipse(x, y, rx, ry, rgba(186, 170, 255, if *hidden { 90 } else { 40 }));
        p.ellipse_ring(x, y, rx, ry, rgba(186, 170, 255, if *hidden { 230 } else { 150 }));
    }
    for (tiles, how) in &o.moves {
        let col = if *how == MoveKind::Carried { AMBER } else { GOLD };
        if let Some(end) = tiles.last() {
            let r = match geo.view {
                View::Stage => inset2(geo.cell_rect(*end), 3, 2),
                View::Flat => geo.cell_rect(*end).inset(3),
            };
            p.panel(r, with_alpha(col, 40), col, 2.0);
        }
    }
    // INPUT1 MR13: the hovered Engage row's open position tiles: a GOLD ring plate (preferred), a DIM one (fallback)
    for (c, preferred) in &o.positions {
        let col = if *preferred { GOLD } else { DIM };
        plate(p, plate_of(*c), with_alpha(col, 36), col);
    }
    if let Some(c) = o.target {
        let (x, y) = ground_pt(geo, c);
        p.ellipse_ring(x, y, rx + 2.5, ry + 1.6, GOLD);
        p.ellipse_ring(x, y, rx + 3.5, ry + 2.2, with_alpha(GOLD, 150));
    }
    // a chevron on the ground ahead of each enemy, pointing where it faces (the Stage squashes the y part)
    let squash = if geo.view == View::Stage { 0.6 } else { 1.0 };
    for (c, f) in &o.facings {
        let (x, y) = ground_pt(geo, *c);
        let len = ((f.x * f.x + f.y * f.y) as f32).sqrt().max(1.0);
        let (dx, dy) = (f.x as f32 / len, f.y as f32 / len * squash);
        let (tip, base) = (rx + 8.0, rx + 3.0);
        let (px, py) = (-dy * 4.0, dx * 4.0 * squash.max(0.8));
        let (tx, ty) = (x + dx * tip, y + dy * tip);
        let (bx, by) = (x + dx * base, y + dy * base);
        // pale over a dark underlay: apart from the red target arrows and the foot rings (slice 4: red read as part
        // of the tell arrows next to the intent glyphs; VISCLAR1 VC6 keeps it on)
        for (col, width) in [(rgba(10, 8, 12, 200), 3.2), (FACING, 1.6)] {
            stage::wide_line(p, bx + px, by + py, tx, ty, col, width, 0.0);
            stage::wide_line(p, bx - px, by - py, tx, ty, col, width, 0.0);
        }
    }
}

/// INPUT1 IF1: the skill menu's width (wider than the step menus: "4 blocks, 4 AP; Maneuver (Engage)" on a row).
const MENU_W: i32 = 214;

/// IF3: a skill-menu row's hover card: the name, the skill's description and its range line (MR4), the refusal, the
/// buffs an intent change drops (IN6).
fn pick_tip(s: &Session, h: usize, o: &rip::PickOption) -> Tip {
    let w = &s.w;
    let mut tip: Tip = vec![(o.picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", then "), GOLD)];
    if let Some(rip::Pick::Skill { skill, .. }) = o.picks.iter().find(|p| matches!(p, rip::Pick::Skill { .. })) {
        let sk = &w.cat.skills[*skill];
        if let Some(d) = sk.raw.get("description").and_then(Value::as_str).filter(|d| !d.is_empty()) {
            tip.push((d.to_string(), TEXT));
        }
        tip.push((intent_text::range_line(w, h, sk), DIM));
    }
    if let Some(why) = &o.problem {
        tip.push((why.clone(), FOE));
    }
    if !o.drops.is_empty() {
        tip.push((format!("Changing loses {}.", o.drops.join(", ")), FOE));
    }
    tip
}

/// INPUT1 MR13: the open position tiles of a Maneuver Engage pick of `h` on the unit `at` (true: the preferred tier),
/// read from where `h`'s picks leave it; empty for any other row and for a protected flank.
fn engage_positions(s: &Session, h: usize, at: RoundTarget, picks: &[rip::Pick]) -> Vec<(V2, bool)> {
    let w = &s.w;
    let (RoundTarget::Unit(t), [rip::Pick::Skill { skill, kind: Some(ManeuverKind::Engage) }]) = (at, picks) else { return Vec::new() };
    let sk = &w.cat.skills[*skill];
    let Some(position) = ri::position_of(w, sk).filter(|_| w.has(t) && w.has(h)) else { return Vec::new() };
    let ghost = rip::ghost_entries(w, &s.plan, h);
    let from = s.plan.entries_for(w, h).iter().zip(&ghost).filter(|(e, g)| !**g && e.step_index >= 0).last().map(|(e, _)| e.pos_after).unwrap_or_else(|| w.pos(h));
    let blocked = s.plan.blocked_cells(w, h);
    let p = ri::positions(w, t, h, position, from, ri::diagonal(w, h, Some(sk)), &blocked);
    p.preferred.iter().map(|c| (*c, true)).chain(p.fallback.iter().map(|c| (*c, false))).collect()
}

/// `m`'s compiled plan in short (skill, start, target, tile after) to tell two plans apart.
fn plan_rows(w: &World, plan: &emberwake_battle::plan::Plan, m: usize) -> Vec<(usize, i32, Option<usize>, V2)> {
    plan.entries_for(w, m).iter().map(|e| (e.skill, e.start_block, e.target, e.pos_after)).collect()
}

/// What the board draws from the battle screen in one frame (borrowed apart from the session).
struct Board<'a> {
    s: &'a Session,
    tokens: &'a mut Tokens,
    floats: &'a [Float],
    flashes: &'a [(usize, Instant)],
    popup: bool,
    /// The FX clock (ms; FX1-L1).
    anim_ms: f64,
    sel: Option<usize>,
    /// ENCOUNTER2-C DT8: the open talk's speaker on the field (wears the selection ring; None without one).
    talker: Option<usize>,
    planning: bool,
    /// INTENT1: the enemy tells while planning; drawn while `show_tells` (the Show enemy tells setting, I).
    tells: &'a [Intent],
    show_tells: bool,
    /// STAGE1 3.9: the unit whose party or hostile card the mouse is over.
    card_hover: Option<usize>,
    /// INPUT1 IS1, AR3 (decision "INPUT1 answers, part 4" Q15): the inspected unit; its target arrow shows by default.
    inspect: Option<usize>,
    /// VISCLAR1 VC2: the focus set but the hovered unit (each view adds the unit under the mouse).
    focus: &'a [usize],
    /// FX1-L1 slice 2: the live effects (drawn at `anim_ms` on the ground and air layers, 4.1) and the camera offset
    /// the board's passes are shifted by.
    fx: &'a Fx,
    shift: (i32, i32),
    /// FX1-L2: the miss sidesteps (4.3).
    sidesteps: &'a [fx::Sidestep],
    /// RULESET1-U slice 3: the round intent's ground overlays (None in classic: the board draws as before).
    overlay: Option<&'a Overlay>,
    /// RULESET1-U slice 4 (UI5): every unit's round intent record (the board's intent glyphs; empty in classic).
    rounds: &'a [Intent],
    /// RULESET1-U slice 5: the overlay groups on (`Layers`) and Alt held (VISCLAR1: every bar and target arrow shows).
    layers: Layers,
    alt: bool,
    /// HOOKS1-UI slice 1: the ground hazards and the arena telegraph (empty in classic).
    marks: &'a HookMarks,
    /// HOOKS1-UI slice 2: each unit's class name for the hover cards (`class_names`).
    classes: &'a [String],
    /// HOOKS1-UI slice 4: the selected hero's push and pull forecasts (planning, layer `pushes`).
    pushes: &'a [emberwake_battle::forced::Forecast],
    /// ENCOUNTER2-U: a hold escape's tiles and gate (None without one).
    hold: Option<&'a HoldMarks>,
}

/// INPUT1 MR12: `unit` moved in this step by a dash: a command of its whose start is this block made the dash board event
/// (`MoveKind::Dash`, the move of a skill with a position).
fn dashed(s: &Session, unit: usize) -> bool {
    s.resolver.scheduled.iter().any(|a| a.unit == unit && a.start == s.current_block && a.command.moved == Some(emberwake_battle::command::MoveKind::Dash))
}

/// An action clip a token wants starts with the first candidate its body has (SPRITE1-U), once the clock reaches its
/// start (FX1-L1: after the actor's slide, so the walk shows until then); WINDUP1 W4: after a held wind-up from the
/// clip's `strike` frame.
fn start_wanted(tokens: &mut Tokens, art: &mut Art, i: usize, body: Option<&str>, now: u64) {
    if let (Some(a), Some(b)) = (tokens.anims.get_mut(i), body) {
        // INPUT1 MR12: a dash plays the rig's `dash` clip over its slide when the body has one (else its walk shows)
        if std::mem::take(&mut a.dash_clip) && art.has_clip(b, "dash") {
            if let Some((_, t0)) = a.slide {
                a.player.play_for("dash", t0, DASH_MS);
            }
        }
        if a.want.as_ref().is_some_and(|w| now >= w.1) {
            let (cands, t0) = a.want.take().unwrap_or_default();
            if let Some(c) = cands.into_iter().find(|c| art.has_clip(b, c)) {
                // WINDUP1 W4: from its `strike` frame after a held wind-up (0 when the row marks none)
                let from = if a.strike { art.clips(b).and_then(|k| k.clip(c)).map_or(0, |k| k.strike) } else { 0 };
                a.player.play_from(c, t0, from);
            }
            a.strike = false;
        }
    }
}

/// WINDUP1 W9 (WQ5): the Tallow Amber outline of a unit showing a wind-up (resolution only; `windup_outline_alpha`, 0
/// off); the selected and hovered outlines win (the callers ask this last).
fn windup_outline(a: &UnitAnim, t: &Tuning) -> Option<Rgba> {
    (a.windup.is_some() && t.windup_outline_alpha > 0).then(|| with_alpha(AMBER, t.windup_outline_alpha))
}

/// WINDUP1 6.2: the clip each token shows at `now` (the side clip of the row its player draws, a missing clip's
/// fallback included; "" without a body), for the agent link's `tokens`.
fn note_drawn(tokens: &mut Tokens, s: &Session, art: &mut Art, now: u64) {
    for (i, a) in tokens.anims.iter_mut().enumerate().take(s.w.units.len()) {
        let body = art.first_body(&body_keys(&s.art[i], &s.w.units[i]));
        a.drawn = body.and_then(|b| art.clips(&b).and_then(|c| a.player.frame(c, now)).map(|f| f.clip)).unwrap_or_default();
    }
}

/// FX1-L1: a struck unit's flash (wall ms since its impact) and its token shake (4.5; none under reduce motion and skip).
fn flash_of(flashes: &[(usize, Instant)], i: usize, now: Instant, t: &Tuning, still: bool) -> (bool, i32) {
    match flashes.iter().rev().find(|f| f.0 == i) {
        Some(f) => (true, fx::token_shake(t.hit_shake_px, now.saturating_duration_since(f.1).as_millis() as u64, t.hit_flash_ms, still)),
        None => (false, 0),
    }
}

/// A fallen unit still playing its death clip (and holding its last frame for `death_hold_ms`); after it (or with no
/// clip) the dead marker shows.
fn still_dying(art: &mut Art, a: &UnitAnim, body: Option<&str>, now: u64, instant: bool, hold_ms: u64) -> bool {
    match (body, a.fell) {
        (Some(b), Some(_)) if !instant && art.has_clip(b, "death") => {
            art.clips(b).is_some_and(|clips| a.player.playing(clips, now) == "death" && a.player.held_for(clips, now).is_none_or(|h| h < hold_ms))
        }
        _ => false,
    }
}

/// STATUSFX1: one token's status looks this frame (`fx::status_frame`) at its drawn foot point (the slide; Flat: the
/// token's centre), facing left or not.
struct StatusOn {
    unit: usize,
    foot: (f32, f32),
    left: bool,
    looks: Vec<fx::Drawn>,
}

/// STATUSFX1 S1, S2: the looks unit `i`'s token draws this frame (none once it has fallen, S1), noted for the agent
/// link's `status_fx` (S9).
fn status_draws(tokens: &mut Tokens, i: usize, alive: bool, wall: u64, now: u64, t: &Tuning) -> Vec<fx::Drawn> {
    let Some(a) = tokens.anims.get_mut(i) else { return Vec::new() };
    let looks = if alive { fx::status_frame(&a.marks, &mut a.slots, wall, now, t.status_cycle_ms) } else { Vec::new() };
    a.status_fx = looks.iter().map(|d| d.look.id).collect();
    looks
}

/// STATUSFX1, the Flat view: the standing units' status looks at their drawn centres (with the slide).
fn flat_status(tokens: &mut Tokens, w: &World, geo: &Geom, t: &Tuning, anim_now: u64, now: Instant, instant: bool) -> Vec<StatusOn> {
    let wall = tokens.wall_ms(now);
    for a in &mut tokens.anims {
        a.status_fx.clear();
    }
    let mut out = Vec::new();
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let (_, alive) = tokens.shown(i, &w.units[i], w.units.len());
        let (mut dx, mut dy, mut left) = (0, 0, !w.allies.contains(&i));
        if let Some(a) = tokens.anims.get(i) {
            left = a.faces(anim_now, if instant { 0 } else { t.walk_ms }).left;
            let ms = a.slide_ms(t.walk_ms);
            if let Some((from, t0)) = a.slide.filter(|s| anim_now.saturating_sub(s.1) < ms && !instant) {
                (dx, dy) = geo.slide(from, pos, 1.0 - anim_now.saturating_sub(t0) as f32 / ms.max(1) as f32);
            }
        }
        let looks = status_draws(tokens, i, alive, wall, anim_now, t);
        if alive {
            let (x, y) = geo.foot(pos);
            out.push(StatusOn { unit: i, foot: ((x + dx) as f32, (y + dy) as f32), left, looks });
        }
    }
    out
}

/// STATUSFX1 S6, S7: the status loops on `layer` (feet looks on the ground, head and body looks in the air) at clock
/// `now`, or still (`Looped::draw_still`) under reduce motion.
fn draw_status_loops(p: &mut Painter, on: &[StatusOn], layer: Layer, geo: &Geom, t: &Tuning, now: u64, still: bool) {
    for s in on {
        for d in &s.looks {
            let Some(l) = fx::status_loop(d.look, geo, t, s.foot, s.left, d.start, fx::status_seed(s.unit, d.look.id), d.pending) else { continue };
            if l.layer() != layer {
                continue;
            }
            if still {
                l.draw_still(p);
            } else {
                l.draw(p, now);
            }
        }
    }
}

/// STATUSFX1: a tint, the silhouette of the token frame `f` of (body, palette) in `col` at alpha `a`, mirrored with it.
fn status_tint(p: &mut Painter, art: &mut Art, (body, palette, f): (&str, &str, &ClipFrame), tr: Rect, facing: Facing, col: Rgba, a: u8) {
    if a == 0 {
        return;
    }
    if let Some(img) = art.body_silhouette(body, palette, &f.clip, f.frame, tr.w, col, facing.mirrors(f)) {
        p.image_alpha(img, tr, a);
    }
}

/// `a` times `k` / 255 (an alpha under the token's).
fn scaled(a: u8, k: u8) -> u8 {
    ((a as u32 * k as u32 + 127) / 255) as u8
}

/// STAGE1 4.1, the Flat view (the board as it was before STAGE1, until the user's review): 32 x 32 cells at (128, 30),
/// the `common` tiles, tree tiles flat, tokens `flat_token_px` on a filled foot ellipse at `foot_alpha`, arrows centre
/// to centre over the units, the cell widgets.
/// The Flat board; returns the unit whose tile the mouse is over (VC2's hovered unit).
fn draw_flat(b: Board, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) -> Option<usize> {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, talker, planning, tells, show_tells, card_hover, inspect, focus, fx: live, shift, sidesteps, overlay, rounds, layers, alt, marks, classes, pushes, hold } = b;
    let w = &s.w;
    // VISCLAR1 VC2: the unit under the mouse (its tile) joins the focus set; AR3: the arrows of the selected and the
    // inspected unit (Alt: all)
    let tile_hover = if popup { None } else { (0..w.units.len()).find(|&u| w.units[u].is_active() && w.grid.pos(u).is_some_and(|p| ui.hover(cell_rect(p)))) };
    let mut focus = focus.to_vec();
    focus.extend(tile_hover.or(card_hover));
    let shown = overlay.map(|o| o.shown(&layers, sel, inspect, alt));
    let panel = Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4);
    ui.p.panel(panel, rgb(8, 10, 16), BORDER, RADIUS);
    // FX1 4.1: the board shifted by the camera, kept inside its panel while it shakes (at rest the Flat board draws
    // unclipped, as before)
    if shift != (0, 0) {
        ui.p.clip(Some(panel));
    }
    ui.p.shift(shift.0, shift.1);
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let r = cell_rect(c);
            let tid = w.grid.terrain_id(c);
            match cx.art.tile_in(&emberwake_content::art::BATTLE_SETS, tid, gx, gy, TILE) {
                Some(img) => ui.p.image(img, r),
                None => ui.p.fill(r, stage::terrain_color(tid)),
            }
            if w.grid.is_exit(c) {
                // ENCOUNTER2 H7: a hold escape's exit reads SHUT or OPEN (layer `gate`)
                let (word, fill, edge) = exit_sign(hold);
                ui.p.panel(r.inset(7), fill, edge, 2.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, word, SMALL, BG);
            }
            if planning && sel.is_some() && ui.hover(r) && !popup {
                ui.p.panel(r.inset(1), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160), 1.5);
            }
            let mut tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), GOOD), ("A unit that steps here leaves the battle.".into(), TEXT)] } else { Vec::new() };
            tip.extend(marks.tile_tip(c));
            tip.extend(hold.map(|h| h.tile_tip(w, c)).unwrap_or_default());
            ui.push(format!("tile.{gx}.{gy}"), "tile", tid, r, planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    // HOOKS1-UI slice 1: the ground hazards and the arena's struck cells, under the round overlays
    draw_hook_marks(&mut ui.p, marks, &Geom::new(View::Flat, tuning));
    if let Some(h) = hold {
        draw_hold_tiles(&mut ui.p, h, &Geom::new(View::Flat, tuning));
    }
    let center = |c: V2| ((GRID_X + c.x * TILE + TILE / 2) as f32, (GRID_Y + c.y * TILE + TILE / 2) as f32);
    if let Some(o) = &shown {
        // RULESET1-U slice 3: the round intent's overlays on the ground; VISCLAR1 AR5: the target arrows over them
        // (under the tokens)
        draw_overlay(&mut ui.p, o, &Geom::new(View::Flat, tuning));
        draw_arrows(&mut ui.p, &o.arrows, &Geom::new(View::Flat, tuning));
    } else if let Some(h) = sel {
        if w.units[h].is_active() && planning {
            let st = s.plan.compile(w, h);
            let mut from = w.pos(h);
            for e in &st.entries {
                if e.pos_after != from && e.pos_after.x >= 0 {
                    let (a, b) = (center(from), center(e.pos_after));
                    ui.p.line(a.0, a.1, b.0, b.1, GOLD, 3.0);
                    ui.p.panel(cell_rect(e.pos_after).inset(3), rgba(244, 198, 84, 40), GOLD, 2.0);
                    from = e.pos_after;
                }
            }
        }
    }
    // classic: the tuning's enemy attack lines (VISCLAR1 AR5: the round intent's target arrows replace them)
    if tuning.intent_arrows && planning && overlay.is_none() {
        let mut seen = Vec::new();
        for a in &s.resolver.scheduled {
            if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                continue;
            }
            if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                seen.push(a.unit);
                let (p, q) = (center(w.pos(a.unit)), center(w.pos(t)));
                ui.p.line(p.0, p.1, q.0, q.1, rgba(240, 100, 92, 210), 4.0);
            }
        }
    }
    // FX1 4.1: the ground layer under the units
    let anim_now = anim_ms as u64;
    let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
    let reduce = cx.settings.reduce_motion();
    let still = UnitAnim::default();
    let geo = Geom::new(View::Flat, tuning);
    // STATUSFX1: the standing units' status looks (feet on the ground layer, head and body in the air), the status
    // loops before FX1's live effects (S6)
    let status = flat_status(tokens, w, &geo, tuning, anim_now, now, instant);
    draw_status_loops(&mut ui.p, &status, Layer::Ground, &geo, tuning, anim_now, reduce);
    live.draw(&mut ui.p, anim_now, Layer::Ground);
    // SPRITE1-U: each unit stands on a flat ellipse in its side's colour and plays its clips (anims) in its facing
    // (DIR4-G: toward its target or its step, `Tokens::catch_up`, `UnitAnim::faces`; `token_frame`, its outline with it);
    // the selected hero's sprite has a gold outline, a hovered unit's a faint white one. A unit without a body keeps
    // the letter. STATUSFX1: a tint right after the sprite (under the hit flash); a stealthed token at 55 % alpha.
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let r = cell_rect(pos);
        let ally = w.allies.contains(&i);
        let body = cx.art.first_body(&body_keys(&s.art[i], u));
        let palette = s.palettes.get(i).map_or("", |p| p.as_str());
        start_wanted(tokens, &mut cx.art, i, body.as_deref(), anim_now);
        // FX1-L1 rule I: the HP and life the board shows (the state before a step until its impact)
        let (hp, alive) = tokens.shown(i, u, w.units.len());
        let a = tokens.anims.get(i).unwrap_or(&still);
        let facing = if tokens.anims.get(i).is_some() { a.faces(anim_now, if instant { 0 } else { tuning.walk_ms }) } else { Facing::side(!ally) };
        if !alive && !still_dying(&mut cx.art, a, body.as_deref(), anim_now, instant, tuning.death_hold_ms) {
            ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "x", BODY, DIM);
            continue;
        }
        let (flash, shake) = flash_of(flashes, i, now, tuning, reduce || instant);
        let tp = geo.token_px;
        // A moving unit slides from its old tile.
        let (mut dx, mut dy) = (0, 0);
        if let Some((from, t0)) = a.slide {
            let (t, ms) = (anim_now.saturating_sub(t0), a.slide_ms(tuning.walk_ms));
            if t < ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / ms.max(1) as f32);
            }
        }
        // FX1-L2: a miss's target steps aside and back
        let (sx, sy) = fx::sidestep_of(sidesteps, i, anim_now);
        let tr = geo.token_rect(pos, shake + dx + sx, dy + sy);
        let ring = if ally { ALLY } else { FOE };
        if alive {
            ui.p.ellipse(tr.x as f32 + tp as f32 / 2.0, (tr.y + tp) as f32 - 2.5, tp as f32 * 0.36, tp as f32 * 0.11, rgba(ring[0], ring[1], ring[2], tuning.foot_alpha));
        }
        let hover = ui.hover(r) && !popup;
        let outline = if Some(i) == sel || Some(i) == talker { Some(GOLD) } else if hover { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { windup_outline(a, tuning) };
        let looks = status.iter().find(|x| x.unit == i).map_or(&[][..], |x| &x.looks[..]);
        let alpha = fx::token_alpha(looks, tuning);
        let frame = body.as_deref().and_then(|b| token_frame(&mut cx.art, b, palette, &a.player, anim_now, tp, facing).map(|(f, img)| {
            ui.p.image_alpha(img, tr, alpha);
            f
        }));
        match (frame, body.as_deref()) {
            (Some(f), Some(b)) => {
                if let Some((col, ta)) = fx::tint_of(looks, anim_now, reduce, tuning) {
                    status_tint(&mut ui.p, &mut cx.art, (b, palette, &f), tr, facing, col, scaled(ta, alpha));
                }
                if flash {
                    hit_flash(&mut ui.p, &mut cx.art, b, palette, &f, tr, facing);
                }
                if let Some(o) = outline.and_then(|col| token_outline(&mut cx.art, b, palette, &f, tp, col, facing)) {
                    ui.p.image_alpha(o, Rect::new(tr.x - 1, tr.y - 1, tp + 2, tp + 2), alpha);
                }
            }
            _ => {
                let ring = with_alpha(ring, scaled(ring[3], alpha));
                ui.p.panel(tr.inset(4), ring, outline.unwrap_or(ring), 6.0);
                ui.p.text_center(tr.x as f32 + tp as f32 / 2.0, r.y as f32 + 11.0 + dy as f32, &u.name.chars().next().unwrap_or('?').to_string(), BODY, BG);
                if flash {
                    ui.p.panel(tr.inset(4), fx::FLASH, fx::FLASH, 6.0);
                }
            }
        }
        if !alive {
            continue;
        }
        // VISCLAR1 VC2, VC3: under the round intent the bars of the focus set (Alt or `all_bars`: every unit's)
        if bars_shown(overlay.is_some(), &layers, alt, &focus, i) {
            hp_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 3 + dy, TILE - 6, 2), hp, u.max_hp, if ally { GOOD } else { FOE });
            // RULESET1-U slice 5: the poise bar over the HP bar (rule poise)
            if let Some(st) = poise_shown(w, i, &layers) {
                poise_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 6 + dy, TILE - 6, 2), &st);
            }
        }
        let tip = unit_tip(s, i, sel, tells, rounds, classes);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    if show_tells {
        for r in tells {
            let Some(from) = w.grid.pos(r.unit) else { continue };
            // VISCLAR1 AR5, VC1: under the round intent no tell arrow (the target arrows replace it) and the kind chip
            // only with the group `first_move`
            if overlay.is_some() {
                if first_move_chips(true, &layers) {
                    intent_chip(ui, cx, r.line.kind, Rect::new(GRID_X + from.x * TILE + 1, GRID_Y + from.y * TILE + 1, 10, 10));
                }
                continue;
            }
            let (fx, fy) = center(from);
            let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
            if let Some(to) = to.filter(|c| *c != from) {
                let (tx, ty) = center(to);
                let (dx, dy) = (tx - fx, ty - fy);
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                // From the token's edge to just short of the target's.
                let (sx, sy, ex, ey) = (fx + dx / len * 10.0, fy + dy / len * 10.0, tx - dx / len * 9.0, ty - dy / len * 9.0);
                ui.p.arrow(sx, sy, ex, ey, rgba(240, 100, 92, 200), stage::arrow_dash(r.line.kind), 4.0);
            }
            let chip = Rect::new(GRID_X + from.x * TILE + 1, GRID_Y + from.y * TILE + 1, 10, 10);
            intent_chip(ui, cx, r.line.kind, chip);
        }
    }
    // RULESET1-U slice 4 (UI5): each unit's intent glyph at its cell's top right (an enemy's only while the tells show)
    for r in rounds.iter().filter(|_| layers.glyphs) {
        let (Some(i), Some(at)) = (r.round_intent, w.grid.pos(r.unit)) else { continue };
        if w.enemies.contains(&r.unit) && !show_tells {
            continue;
        }
        intent_glyph(&mut ui.p, &w.cat, i, Rect::new(GRID_X + at.x * TILE + TILE - 11, GRID_Y + at.y * TILE + 1, 10, 10));
    }
    // HOOKS1-UI slice 4: the push forecasts above the tokens (a collision mark sits between two of them)
    draw_pushes(&mut ui.p, pushes, &Geom::new(View::Flat, tuning));
    // RULESET1-U slice 5 (P7): a BREAK plate over each unit broken this round
    for i in broken_units(w, &layers) {
        let at = w.pos(i);
        break_plate(&mut ui.p, (GRID_X + at.x * TILE + TILE / 2) as f32, (GRID_Y + at.y * TILE + 2) as f32);
    }
    // FX1 4.1: the air layer over the units, before the floats (STATUSFX1 S6: the status loops first)
    draw_status_loops(&mut ui.p, &status, Layer::Air, &geo, tuning, anim_now, reduce);
    live.draw(&mut ui.p, anim_now, Layer::Air);
    for f in floats {
        if let Some(pos) = w.grid.pos(f.unit) {
            let t = now.duration_since(f.born).as_millis() as f32 / f.ms.max(1) as f32;
            let (cx, cy) = center(pos);
            float_text(ui, &f.text, f.color, cx, cy - 14.0 - tuning.float_rise_px as f32 * t);
        }
    }
    ui.p.shift(-shift.0, -shift.1);
    if shift != (0, 0) {
        ui.p.clip(None);
    }
    tile_hover
}

/// An intent chip (INTENT1): the kind's icon, else a lettered plate.
fn intent_chip(ui: &mut Ui<Action>, cx: &mut Cx, kind: &str, chip: Rect) {
    match cx.art.image(&emberwake_content::art::intent(kind), 10, 10) {
        Some(img) => ui.p.image(img, chip),
        None => {
            ui.p.panel(chip, rgba(48, 28, 38, 230), FOE, 2.0);
            ui.p.text_center(chip.x as f32 + 5.0, chip.y as f32 + 1.0, &kind[..1].to_uppercase(), SMALL, TEXT);
        }
    }
}

/// A floating number with its shadow, centred on x at line top y; transient (TEXTFIT2: the text audit skips it).
fn float_text(ui: &mut Ui<Action>, text: &str, col: Rgba, x: f32, y: f32) {
    ui.p.transient(|p| {
        p.text_center(x + 0.6, y + 0.6, text, TITLE, rgba(0, 0, 0, 220));
        p.text_center(x, y, text, TITLE, col);
    });
}

/// One unit on the Stage this frame (STAGE1 3.6, 3.8).
struct StageUnit {
    i: usize,
    pos: V2,
    ally: bool,
    /// FX1-L1 rule I: the life and HP the board shows (`Tokens::shown`).
    alive: bool,
    hp: i32,
    /// Its sprite draws (standing, or still playing its death clip); else the dead marker shows.
    standing: bool,
    body: Option<String>,
    palette: String,
    /// DIR4-G: the facing drawn this frame (`UnitAnim::faces`).
    facing: Facing,
    flash: bool,
    /// The foot point with the slide.
    foot: (i32, i32),
    /// The drawn sprite square (slide and hit shake).
    tr: Rect,
    /// The body's solid rect at its place (slide, no shake): the feet pass's rect and half of its widget rect.
    body_rect: Rect,
    /// STATUSFX1: the status looks it draws (none once it has fallen).
    looks: Vec<fx::Drawn>,
}

/// A 1 px edged plate (the Stage's ground markers, 3.7 pass 3).
fn plate(p: &mut Painter, r: Rect, fill: Rgba, edge: Rgba) {
    p.fill(r, fill);
    p.fill(Rect::new(r.x, r.y, r.w, 1), edge);
    p.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), edge);
    p.fill(Rect::new(r.x, r.y + 1, 1, r.h - 2), edge);
    p.fill(Rect::new(r.x + r.w - 1, r.y + 1, 1, r.h - 2), edge);
}

/// A cell plate inset `ix` across and `iy` down.
fn inset2(r: Rect, ix: i32, iy: i32) -> Rect {
    Rect::new(r.x + ix, r.y + iy, r.w - 2 * ix, r.h - 2 * iy)
}

fn with_alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

/// STAGE1 option B, the Stage view: the board tilted to 32 x 24 cells, run as `stage::passes` (backdrop, ground,
/// markers, ground arrows, shadows, front face, the feet-sorted blocks, props and units, the outlines through what
/// covers a unit, the top: HP bars, chips, the exit sign, floats). Cell widgets first, then the units' in feet order
/// (front-most last, so it takes the click). Returns the unit the mouse is over on the board.
fn draw_stage(b: Board, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) -> Option<usize> {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, talker, planning, tells, show_tells, card_hover, inspect, focus, fx: live, shift, sidesteps, overlay, rounds, layers, alt, marks, classes, pushes, hold } = b;
    let w = &s.w;
    // VISCLAR1 AR3: the arrows of the selected and the inspected unit (Alt: all)
    let shown = overlay.map(|o| o.shown(&layers, sel, inspect, alt));
    let geo = Geom::new(View::Stage, tuning);
    let tp = geo.token_px;
    let anim_now = anim_ms as u64;
    let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
    let reduce = cx.settings.reduce_motion();
    let still = UnitAnim::default();
    ui.p.panel(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), rgb(8, 10, 16), BORDER, RADIUS);

    // The units' places this frame (STATUSFX1: and their status looks).
    let wall = tokens.wall_ms(now);
    for a in &mut tokens.anims {
        a.status_fx.clear();
    }
    let mut units: Vec<StageUnit> = Vec::new();
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let ally = w.allies.contains(&i);
        let body = cx.art.first_body(&body_keys(&s.art[i], u));
        let palette = s.palettes.get(i).cloned().unwrap_or_default();
        start_wanted(tokens, &mut cx.art, i, body.as_deref(), anim_now);
        let (hp, alive) = tokens.shown(i, u, w.units.len());
        let a = tokens.anims.get(i).unwrap_or(&still);
        let facing = if tokens.anims.get(i).is_some() { a.faces(anim_now, if instant { 0 } else { tuning.walk_ms }) } else { Facing::side(!ally) };
        let standing = alive || still_dying(&mut cx.art, a, body.as_deref(), anim_now, instant, tuning.death_hold_ms);
        let (mut dx, mut dy) = (0, 0);
        if let Some((from, t0)) = a.slide {
            let (t, ms) = (anim_now.saturating_sub(t0), a.slide_ms(tuning.walk_ms));
            if t < ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / ms.max(1) as f32);
            }
        }
        let (flash, shake) = flash_of(flashes, i, now, tuning, reduce || instant);
        let (fx, fy) = geo.foot(pos);
        let rest = geo.token_rect(pos, dx, dy);
        // 3.8: the body's solid bounds of its idle frame 0 (animation frames do not move it); DIR4-HB: in the facing's
        // view (the front or back idle while it faces down or up, else the side idle with its left/right); no body:
        // the letter plate.
        let body_rect = match body.as_deref().and_then(|k| cx.art.body_bounds_facing(k, &palette, tp, facing)) {
            Some(bb) => Rect::new(rest.x + bb.x, rest.y + bb.y, bb.w, bb.h),
            None => rest.inset(4),
        };
        // FX1-L2: a miss's target steps aside and back (its sprite only; its foot keeps its place in the draw order)
        let (sx, sy) = fx::sidestep_of(sidesteps, i, anim_now);
        let looks = status_draws(tokens, i, alive, wall, anim_now, tuning);
        units.push(StageUnit { i, pos, ally, alive, hp, standing, body, palette, facing, flash, foot: (fx + dx, fy + dy), tr: geo.token_rect(pos, dx + shake + sx, dy + sy), body_rect, looks });
    }
    let status: Vec<StatusOn> = units.iter().filter(|t| t.alive).map(|t| StatusOn { unit: t.i, foot: (t.foot.0 as f32, t.foot.1 as f32), left: t.facing.left, looks: t.looks.clone() }).collect();
    // 3.7 pass 7: blocks, standing props and units sorted by their feet.
    let mut items = stage::board_items(&mut cx.art, &w.grid);
    items.extend(units.iter().filter(|t| t.standing).map(|t| stage::unit_item(t.i, t.foot, t.body_rect)));
    let order = stage::feet_order(&items);
    let unit_of = |i: usize| units.iter().find(|t| t.i == i);
    // 3.8: a standing unit's widget rect; the front-most under the mouse is the one a click reaches.
    let widget_rect = |t: &StageUnit| stage::unit_rect(geo.cell_rect(t.pos), t.body_rect);
    let board_hover = if ui.live && !popup {
        order.iter().rev().find_map(|p| match p.item {
            Item::Unit(i) => unit_of(i).filter(|t| t.alive && widget_rect(t).contains(ui.mouse.0, ui.mouse.1)).map(|t| t.i),
            _ => None,
        })
    } else {
        None
    };
    let hovered = board_hover.or(card_hover);
    // VC2: the unit under the mouse joins the focus set
    let mut focus = focus.to_vec();
    focus.extend(hovered);
    // The cell a click would reach (the hovered unit's, else the one under the mouse) shows the hover plate.
    let hover_cell = if planning && sel.is_some() && ui.live && !popup {
        board_hover.and_then(|u| w.grid.pos(u)).or_else(|| geo.cell_at(ui.mouse.0, ui.mouse.1))
    } else {
        None
    };
    let foot_f = |c: V2| {
        let (x, y) = geo.foot(c);
        (x as f32, y as f32)
    };
    let mut frames: Vec<(usize, ClipFrame)> = Vec::new();

    // FX1 4.1: passes 1-9 inside the board's clip, shifted by the camera (the clip stays put)
    ui.p.clip(Some(stage::INTERIOR));
    ui.p.shift(shift.0, shift.1);
    for pass in stage::passes() {
        match pass {
            // HOOKS1 ZN3: a data zone row keys its own backdrop and gradient ground (a code zone ignores any row)
            Pass::Backdrop => stage::draw_backdrop(&mut ui.p, &mut cx.art, &w.grid, &s.def.zone, &w.cat.zones),
            Pass::Ground => stage::draw_ground(&mut ui.p, &mut cx.art, &w.grid, tuning.grid_alpha),
            Pass::Markers => {
                // HOOKS1-UI slice 1: the ground hazards and the arena's struck cells, under the hover and the tells
                draw_hook_marks(&mut ui.p, marks, &geo);
                if let Some(h) = hold {
                    draw_hold_tiles(&mut ui.p, h, &geo);
                }
                if let Some(c) = hover_cell {
                    plate(&mut ui.p, geo.cell_rect(c), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160));
                }
                // INTENT1 area tells: the tiles an enemy's area attack will hit.
                if show_tells {
                    for r in tells.iter().filter(|r| r.line.kind == "area") {
                        let cells = s.resolver.scheduled.iter().find_map(|a| match &a.command.kind {
                            CmdKind::Area { cells, aim, .. } if a.unit == r.unit && *aim == r.line.cell => Some(cells.clone()),
                            _ => None,
                        });
                        for c in cells.unwrap_or_default().into_iter().filter(|c| w.grid.is_valid(*c)) {
                            plate(&mut ui.p, geo.cell_rect(c).inset(1), rgba(240, 100, 92, 70), rgba(240, 100, 92, 230));
                        }
                    }
                }
                if let Some(o) = &shown {
                    // RULESET1-U slice 3: the round intent's overlays on the ground (the moves' destination plates)
                    draw_overlay(&mut ui.p, o, &geo);
                } else if let Some(h) = sel.filter(|h| w.units[*h].is_active() && planning) {
                    let st = s.plan.compile(w, h);
                    let mut from = w.pos(h);
                    for e in &st.entries {
                        if e.pos_after != from && e.pos_after.x >= 0 {
                            let (a, b) = (foot_f(from), foot_f(e.pos_after));
                            stage::wide_line(&mut ui.p, a.0, a.1 - 3.0, b.0, b.1 - 3.0, GOLD, 2.0, 0.0);
                            ui.p.panel(inset2(geo.cell_rect(e.pos_after), 3, 2), rgba(244, 198, 84, 40), GOLD, 2.0);
                            from = e.pos_after;
                        }
                    }
                }
                // ENCOUNTER2 H7: a hold escape's shut gate plates its exits red (layer `gate`)
                let (_, fill, edge) = exit_sign(hold);
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    plate(&mut ui.p, inset2(geo.cell_rect(*c), 3, 2), with_alpha(fill, 90), with_alpha(edge, 200));
                }
            }
            // VISCLAR1 AR4, AR5: under the round intent the target arrows replace the tell arrows and the enemy attack
            // lines (classic keeps them)
            Pass::Arrows if shown.is_some() => {
                if let Some(o) = &shown {
                    draw_arrows(&mut ui.p, &o.arrows, &geo);
                }
            }
            Pass::Arrows => {
                let col = rgba(240, 100, 92, 210);
                let mut drawn: Vec<(V2, V2)> = Vec::new();
                if show_tells {
                    for r in tells {
                        let Some(from) = w.grid.pos(r.unit) else { continue };
                        let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
                        if let Some(to) = to.filter(|c| *c != from) {
                            if let Some((x0, y0, x1, y1)) = stage::ground_arrow(foot_f(from), foot_f(to)) {
                                stage::wide_arrow(&mut ui.p, x0, y0, x1, y1, col, 1.5, stage::arrow_dash(r.line.kind), 4.0);
                            }
                            drawn.push((from, to));
                        }
                    }
                }
                if tuning.intent_arrows && planning {
                    let mut seen = Vec::new();
                    for a in &s.resolver.scheduled {
                        if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                            continue;
                        }
                        if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                            seen.push(a.unit);
                            let (from, to) = (w.pos(a.unit), w.pos(t));
                            if from == to || drawn.contains(&(from, to)) {
                                continue;
                            }
                            if let Some((x0, y0, x1, y1)) = stage::ground_arrow(foot_f(from), foot_f(to)) {
                                stage::wide_arrow(&mut ui.p, x0, y0, x1, y1, col, 1.5, 0.0, 4.0);
                            }
                        }
                    }
                }
            }
            Pass::Shadows => {
                stage::draw_prop_shadows(&mut ui.p, &mut cx.art, &w.grid);
                for t in units.iter().filter(|t| t.alive) {
                    let (fx, fy) = (t.foot.0 as f32, t.foot.1 as f32);
                    ui.p.ellipse(fx + 1.0, fy, 0.2 * tp as f32, 2.4, rgba(8, 10, 6, tuning.shadow_alpha));
                    ui.p.ellipse_ring(fx, fy, 0.25 * tp as f32, 3.2, with_alpha(if t.ally { ALLY } else { FOE }, tuning.foot_ring_alpha));
                }
            }
            Pass::Face => stage::draw_face(&mut ui.p, &mut cx.art, &w.grid),
            Pass::Feet => {
                for p in &order {
                    match p.item {
                        Item::Block(c) => stage::draw_block(&mut ui.p, &mut cx.art, &w.grid, c),
                        Item::Prop(c) => stage::draw_prop(&mut ui.p, &mut cx.art, &w.grid, c),
                        Item::Unit(i) => {
                            let Some(t) = unit_of(i) else { continue };
                            let a = tokens.anims.get(i).unwrap_or(&still);
                            let ring = if t.ally { ALLY } else { FOE };
                            let outline = if Some(i) == sel || Some(i) == talker { Some(GOLD) } else if hovered == Some(i) { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { windup_outline(a, tuning) };
                            // STATUSFX1: a stealthed token (sprite and outline) at 55 % alpha
                            let alpha = fx::token_alpha(&t.looks, tuning);
                            let frame = t.body.as_deref().and_then(|b| token_frame(&mut cx.art, b, &t.palette, &a.player, anim_now, tp, t.facing).map(|(f, img)| {
                                ui.p.image_alpha(img, t.tr, alpha);
                                f
                            }));
                            match (frame, t.body.as_deref()) {
                                (Some(f), Some(b)) => {
                                    // STATUSFX1: the tint right after the sprite, under the hit flash (FX1 4.1)
                                    if let Some((col, ta)) = fx::tint_of(&t.looks, anim_now, reduce, tuning) {
                                        status_tint(&mut ui.p, &mut cx.art, (b, &t.palette, &f), t.tr, t.facing, col, scaled(ta, alpha));
                                    }
                                    // FX1 4.1: the hit flash right after its unit's sprite
                                    if t.flash {
                                        hit_flash(&mut ui.p, &mut cx.art, b, &t.palette, &f, t.tr, t.facing);
                                    }
                                    if let Some(o) = outline.and_then(|col| token_outline(&mut cx.art, b, &t.palette, &f, tp, col, t.facing)) {
                                        ui.p.image_alpha(o, Rect::new(t.tr.x - 1, t.tr.y - 1, tp + 2, tp + 2), alpha);
                                    }
                                    frames.push((i, f));
                                }
                                _ => {
                                    let ring = with_alpha(ring, scaled(ring[3], alpha));
                                    ui.p.panel(t.tr.inset(4), ring, outline.unwrap_or(ring), 6.0);
                                    let letter = w.units[i].name.chars().next().unwrap_or('?').to_string();
                                    ui.p.text_center(t.tr.x as f32 + tp as f32 / 2.0, (t.tr.y + tp / 2) as f32 - 5.0, &letter, BODY, BG);
                                    if t.flash {
                                        ui.p.panel(t.tr.inset(4), fx::FLASH, fx::FLASH, 6.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Pass::Through => {
                // STAGE1-O: by default only the selected unit shows through; Alt held outlines every unit
                let reveal = stage::Reveal::of(&cx.settings.v["see_through"], cx.held.alt);
                for (k, p) in order.iter().enumerate() {
                    let Item::Unit(i) = p.item else { continue };
                    let Some(t) = unit_of(i).filter(|t| t.alive) else { continue };
                    let (Some(b), Some((_, f))) = (t.body.as_deref(), frames.iter().find(|(j, _)| *j == i)) else { continue };
                    let col = match stage::through(&order, k, Some(i) == sel, hovered == Some(i), reveal) {
                        Some(Through::Side) => with_alpha(if t.ally { ALLY } else { FOE }, 230),
                        Some(Through::Selected) => GOLD,
                        Some(Through::Hovered) => rgba(255, 255, 255, tuning.hover_outline_alpha),
                        None => continue,
                    };
                    if let Some(o) = token_outline(&mut cx.art, b, &t.palette, f, tp, col, t.facing) {
                        ui.p.image_alpha(o, Rect::new(t.tr.x - 1, t.tr.y - 1, tp + 2, tp + 2), fx::token_alpha(&t.looks, tuning));
                    }
                }
            }
            Pass::Top => {
                // HOOKS1-UI slice 4: the push forecasts above the tokens
                draw_pushes(&mut ui.p, pushes, &geo);
                for t in &units {
                    let u = &w.units[t.i];
                    let (fx, fy) = t.foot;
                    if !t.standing {
                        ui.p.text_center(fx as f32, fy as f32 - ui.p.line_h(BODY) / 2.0, "x", BODY, DIM);
                    } else if t.alive && bars_shown(overlay.is_some(), &layers, alt, &focus, t.i) {
                        // VISCLAR1 VC2, VC3: under the round intent the focus set's bars (Alt or `all_bars`: all)
                        hp_bar(&mut ui.p, Rect::new(fx - 12, fy + 3, 24, 2), t.hp, u.max_hp, if t.ally { GOOD } else { FOE });
                        // RULESET1-U slice 5: the poise bar under the HP bar (rule poise)
                        if let Some(st) = poise_shown(w, t.i, &layers) {
                            poise_bar(&mut ui.p, Rect::new(fx - 12, fy + 6, 24, 2), &st);
                        }
                    }
                }
                // VISCLAR1 VC1: under the round intent the tell's kind chip only with the group `first_move`
                if show_tells && first_move_chips(overlay.is_some(), &layers) {
                    for r in tells {
                        let Some(t) = units.iter().find(|t| t.i == r.unit) else { continue };
                        let chip = Rect::new(t.foot.0 - 15, (t.foot.1 as f32 - 0.95 * tp as f32 - 2.0).round() as i32, 10, 10);
                        intent_chip(ui, cx, r.line.kind, chip);
                    }
                }
                // RULESET1-U slice 4 (UI5): each unit's intent glyph right of where the tell chip sits (an enemy's only
                // while the tells show)
                for r in rounds.iter().filter(|_| layers.glyphs) {
                    let (Some(i), Some(t)) = (r.round_intent, units.iter().find(|t| t.i == r.unit && t.alive)) else { continue };
                    if w.enemies.contains(&r.unit) && !show_tells {
                        continue;
                    }
                    intent_glyph(&mut ui.p, &w.cat, i, Rect::new(t.foot.0 + 5, (t.foot.1 as f32 - 0.95 * tp as f32 - 2.0).round() as i32, 10, 10));
                }
                // RULESET1-U slice 5 (P7): a BREAK plate over each unit broken this round, above its chips
                for i in broken_units(w, &layers) {
                    let Some(t) = units.iter().find(|t| t.i == i && t.alive) else { continue };
                    break_plate(&mut ui.p, t.foot.0 as f32, t.foot.1 as f32 - 0.95 * tp as f32 - 13.0);
                }
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    let r = geo.cell_rect(*c);
                    let (cx_, cy_) = (r.x + r.w / 2, r.y + r.h / 2);
                    let sign = Rect::new(cx_ - 11, cy_ - 2 - 5, 22, 9);
                    // ENCOUNTER2 H7: a hold escape's exit reads SHUT or OPEN (layer `gate`)
                    let (word, fill, edge) = exit_sign(hold);
                    ui.p.panel(sign, fill, edge, 2.0);
                    ui.p.text_center(cx_ as f32, sign.y as f32 + (sign.h as f32 - ui.p.line_h(SMALL)) / 2.0, word, SMALL, BG);
                }
                for f in floats {
                    let Some(pos) = w.grid.pos(f.unit) else { continue };
                    let t = now.duration_since(f.born).as_millis() as f32 / f.ms.max(1) as f32;
                    let (x, y) = geo.chest(pos);
                    float_text(ui, &f.text, f.color, x as f32, y as f32 - 14.0 - tuning.float_rise_px as f32 * t);
                }
            }
        }
        // FX1 4.1: the ground layer after pass 4 (the ground arrows), the air layer after pass 8 (before the floats);
        // STATUSFX1 S6: on each the status loops first, then FX1's live effects
        match pass {
            Pass::Arrows => {
                draw_status_loops(&mut ui.p, &status, Layer::Ground, &geo, tuning, anim_now, reduce);
                live.draw(&mut ui.p, anim_now, Layer::Ground);
            }
            Pass::Through => {
                draw_status_loops(&mut ui.p, &status, Layer::Air, &geo, tuning, anim_now, reduce);
                live.draw(&mut ui.p, anim_now, Layer::Air);
            }
            _ => {}
        }
    }
    ui.p.shift(-shift.0, -shift.1);
    ui.p.clip(None);

    // 3.8: the cell widgets (32 x 24), then the units' in feet order (front-most last).
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let mut tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), GOOD), ("A unit that steps here leaves the battle.".into(), TEXT)] } else { Vec::new() };
            tip.extend(marks.tile_tip(c));
            tip.extend(hold.map(|h| h.tile_tip(w, c)).unwrap_or_default());
            ui.push(format!("tile.{gx}.{gy}"), "tile", w.grid.terrain_id(c), geo.cell_rect(c), planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    for p in &order {
        let Item::Unit(i) = p.item else { continue };
        let Some(t) = unit_of(i).filter(|t| t.alive) else { continue };
        let u = &w.units[i];
        let tip = unit_tip(s, i, sel, tells, rounds, classes);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), widget_rect(t), true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    board_hover
}

#[cfg(test)]
mod tests {
    use super::{action_clips, boss_fight, sound_of, token_frame, token_outline, turn, Action, BattleScreen, Tokens};
    use crate::fx;
    use crate::screens::Screen;
    use aegis_ui::fx::Layer;
    use crate::stage::{Geom, View};
    use emberwake_battle::grid::GRID_H;
    use emberwake_battle::command::CmdKind;
    use emberwake_battle::grid::{V2, NO_CELL};
    use aegis_core::assets::{ClipPlayer, Clips, Facing, Layers, View as Look};
    use aegis_ui::art::Art;
    use emberwake_battle::session::{EncounterDef, Phase, Session};
    use emberwake_host::{Battle, Host};
    use emberwake_rules::Stance;
    use emberwake_world::battle::WorldRoster;
    use emberwake_world::Data;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn an_action_picks_the_rig_clip_of_its_skill_kind_and_weapon() {
        assert_eq!(action_clips("shield_bash", "attack", "sword"), ["shield_bash", "attack"], "a skill with its own clip first");
        assert_eq!(action_clips("cyclone_kick", "area", ""), ["sweep_kick", "sweep_kick", "kick", "jab", "attack"]);
        assert_eq!(action_clips("strike", "attack", "fist"), ["jab", "attack"], "fists jab");
        assert_eq!(action_clips("caustic_vomit", "attack", ""), ["spit", "jab", "attack"], "the crawler spits, other bodies fall back");
        assert_eq!(action_clips("strike", "attack", "bow"), ["attack"]);
        assert_eq!(action_clips("volley", "area", "bow"), ["attack"]);
        assert_eq!(action_clips("mend", "support", "mace"), ["cast"]);
        assert!(action_clips("guard", "", "sword").is_empty(), "guards play through the stance, moves through the slide");
    }

    /// The rig's sheets face right: a token facing left (FACE1) shows the sheet frame mirrored, its outline with it; one
    /// facing right the sheet frame as drawn. A scratch body whose frame is lopsided shows the difference.
    #[test]
    fn a_token_facing_left_is_the_sheet_frame_mirrored_and_one_facing_right_is_not() {
        const F: u32 = 8;
        let root = std::env::temp_dir().join(format!("emberwake_foe_facing_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("units/brawler/goblin");
        std::fs::create_dir_all(&dir).unwrap();
        // A bar on the left of the 8x8 frame (x 1..3, the back) and one pixel on the right (x 6, the face).
        let mut px = vec![0u8; (F * F * 4) as usize];
        for x in [1u32, 2, 3, 6] {
            let i = ((4 * F + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[90, 140, 60, 255]);
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": [{"id": "idle", "frames": 1, "ms": [100]}]}).to_string()).unwrap();
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        let (body, p, size) = ("brawler/goblin", ClipPlayer::default(), 3 * F as i32);
        let sheet = art.body_clip(body, "", "idle", 0, size).unwrap().clone();
        let left = token_frame(&mut art, body, "", &p, 0, size, Facing::side(true)).unwrap().1.clone();
        let right = token_frame(&mut art, body, "", &p, 0, size, Facing::side(false)).unwrap().1.clone();
        assert_eq!(left, sheet.mirrored(), "facing left: the sheet frame mirrored");
        assert_eq!(right, sheet, "facing right: as the sheet faces");
        assert_ne!(left, right);
        let f = token_frame(&mut art, body, "", &p, 0, size, Facing::side(true)).unwrap().0;
        let gold = [244, 198, 84, 255];
        let left_ring = token_outline(&mut art, body, "", &f, size, gold, Facing::side(true)).unwrap().clone();
        assert_eq!(left_ring, sheet.mirrored().outline(gold), "the selection outline follows the mirrored frame");
        assert_eq!(token_outline(&mut art, body, "", &f, size, gold, Facing::side(false)).unwrap().clone(), sheet.outline(gold));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIR4-G: a scratch body `units/<key>` with clips `ids` (one 8 x 8 frame each, 100 ms; `hurt` and `death` play
    /// once), each `<of>_<view>` id whose side clip is listed a view row of it; every row a lopsided bar (so a mirror
    /// shows) in its own colour (so the rows differ).
    fn scratch_body(root: &std::path::Path, key: &str, ids: &[&str]) {
        const F: u32 = 8;
        let dir = root.join("units").join(key);
        std::fs::create_dir_all(&dir).unwrap();
        let mut px = vec![0u8; (F * F * ids.len() as u32 * 4) as usize];
        for row in 0..ids.len() as u32 {
            for x in [1u32, 2, 3, 6] {
                let i = (((row * F + 4) * F + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[40 + 20 * row as u8, 140, 60, 255]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F * ids.len() as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        let clips: Vec<_> = ids.iter().map(|id| {
            let mut c = json!({"id": id, "frames": 1, "ms": [100], "loop": !["hurt", "death"].contains(id)});
            for v in ["up", "down"] {
                if let Some(of) = id.strip_suffix(&format!("_{v}")).filter(|of| ids.contains(of)) {
                    c["of"] = json!(of);
                    c["view"] = json!(v);
                }
            }
            c
        }).collect();
        std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": clips}).to_string()).unwrap();
    }

    /// DIR4-G: a scratch tree with a body with front and back views (`test/views`) and one with side clips only
    /// (`test/side`, as every body before DIR4-R1/R2/R3), and its `Art`.
    fn view_bodies(tag: &str) -> (std::path::PathBuf, Art) {
        let root = std::env::temp_dir().join(format!("emberwake_dir4g_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        scratch_body(&root, "test/views", &["idle", "walk", "hurt", "idle_down", "idle_up", "walk_up"]);
        scratch_body(&root, "test/side", &["idle", "walk", "hurt"]);
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        for body in ["test/views", "test/side"] {
            assert_eq!(art.body_problems(body), Vec::<String>::new(), "the scratch body {body} loads");
        }
        (root, art)
    }

    /// DIR4-G check 1: a unit whose target is two tiles straight below faces down and draws `idle_down` (never
    /// mirrored) when its body has it, its side `idle` (mirrored by its left/right) otherwise.
    #[test]
    fn a_target_straight_below_draws_idle_down_when_the_body_has_it_and_side_idle_otherwise() {
        let (root, mut art) = view_bodies("below");
        let size = 24;
        let mut p = ClipPlayer::default();
        p.set_base("idle", 0);
        for left in [false, true] {
            let f = turn(Facing::side(left), V2 { x: 5, y: 2 }, V2 { x: 5, y: 4 });
            assert_eq!(f, Facing { view: Look::Down, left }, "two tiles below: down, keeping left/right");
            let (fr, img) = token_frame(&mut art, "test/views", "", &p, 0, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
            assert_eq!((fr.clip.as_str(), fr.side), ("idle_down", false));
            assert_eq!(img, art.body_clip("test/views", "", "idle_down", 0, size).unwrap().clone(), "a down row is never mirrored");
            let (fr, img) = token_frame(&mut art, "test/side", "", &p, 0, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
            assert_eq!((fr.clip.as_str(), fr.side), ("idle", true), "no down view: the side idle");
            assert_eq!(img, art.body_clip_flipped("test/side", "", "idle", 0, size, left).unwrap().clone(), "mirrored by the kept left/right");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Unit `h` of `host` steps to `to` at `now` (a step the tokens see).
    fn slide_to(host: &mut Host, t: &mut Tokens, h: usize, to: V2, now: u64) {
        let s = host.session.as_mut().unwrap();
        s.w.grid.positions[h] = Some(to);
        t.catch_up(s, now, false, 300, true);
    }

    /// DIR4-G check 2 (G2): a slide plays its walk in the step's facing: (5, 4) to (5, 2) `walk_up`, then `idle_up`
    /// once the slide ends; (5, 4) to (7, 5) the side walk as drawn; (5, 4) to (3, 3) the side walk mirrored. A body
    /// without the up view walks side-on.
    #[test]
    fn a_slide_plays_walk_up_or_the_side_walk_mirrored_by_its_direction() {
        let (root, mut art) = view_bodies("slide");
        let size = 24;
        let mut host = skirmish(3);
        let mut t = Tokens::default();
        t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
        let h = host.session.as_ref().unwrap().w.allies[0];
        let mut now = 0;
        let mut go = |host: &mut Host, t: &mut Tokens, x: i32, y: i32| {
            now += 1000;
            slide_to(host, t, h, V2 { x, y }, now);
            now
        };
        let start = V2 { x: 5, y: 4 };
        go(&mut host, &mut t, start.x, start.y);
        let at = go(&mut host, &mut t, 5, 2);
        let a = t.anims[h].clone();
        assert_eq!(a.slide, Some((start, at)));
        assert_eq!(a.faces(at + 10, 300).view, Look::Up);
        let (fr, img) = token_frame(&mut art, "test/views", "", &a.player, at + 10, size, a.faces(at + 10, 300)).map(|(fr, i)| (fr, i.clone())).unwrap();
        assert_eq!((fr.clip.as_str(), fr.side), ("walk_up", false), "(5, 4) to (5, 2) walks up");
        assert_eq!(img, art.body_clip("test/views", "", "walk_up", 0, size).unwrap().clone());
        let fr = token_frame(&mut art, "test/side", "", &a.player, at + 10, size, a.faces(at + 10, 300)).unwrap().0;
        assert_eq!((fr.clip.as_str(), fr.side), ("walk", true), "no up view: the side walk");
        let fr = token_frame(&mut art, "test/views", "", &a.player, at + 400, size, a.faces(at + 400, 300)).unwrap().0;
        assert_eq!(fr.clip, "idle_up", "the slide over: it stands facing up");
        for ((x, y), mirrored) in [((7, 5), false), ((3, 3), true)] {
            go(&mut host, &mut t, start.x, start.y);
            let at = go(&mut host, &mut t, x, y);
            let a = t.anims[h].clone();
            let f = a.faces(at + 10, 300);
            assert_eq!(f, Facing::side(mirrored), "(5, 4) to ({x}, {y})");
            let (fr, img) = token_frame(&mut art, "test/views", "", &a.player, at + 10, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
            assert_eq!((fr.clip.as_str(), fr.side), ("walk", true), "(5, 4) to ({x}, {y}) walks side-on");
            assert_eq!(img, art.body_clip_flipped("test/views", "", "walk", 0, size, mirrored).unwrap().clone(), "(5, 4) to ({x}, {y}) mirrored: {mirrored}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIR4-G check 4 (G4): a unit facing up that is hit plays the side `hurt` with its last left/right (mirrored when
    /// left), its outline with it, then shows `idle_up` again.
    #[test]
    fn a_hit_while_facing_up_plays_the_side_hurt_then_idle_up_again() {
        let (root, mut art) = view_bodies("hurt");
        let size = 24;
        let mut host = skirmish(3);
        let mut t = Tokens::default();
        t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
        let h = host.session.as_ref().unwrap().w.allies[0];
        for (k, (x, y)) in [(5, 4), (4, 4), (4, 2)].into_iter().enumerate() {
            slide_to(&mut host, &mut t, h, V2 { x, y }, 1000 * (k as u64 + 1));
        }
        assert_eq!(t.anims[h].rest(), Facing { view: Look::Up, left: true }, "a step left, then up");
        let hit = 10_000;
        {
            let s = host.session.as_mut().unwrap();
            let u = &mut s.w.units[h];
            (u.guard, u.stance) = (0.0, Stance::Neutral);
            t.catch_up(s, hit - 1, false, 300, true);
            s.w.units[h].hp -= 1;
            t.catch_up(s, hit, false, 300, true);
        }
        let a = t.anims[h].clone();
        let f = a.faces(hit + 10, 300);
        let (fr, img) = token_frame(&mut art, "test/views", "", &a.player, hit + 10, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
        assert_eq!((fr.clip.as_str(), fr.side), ("hurt", true), "no hurt_up: the side hurt");
        assert_eq!(img, art.body_clip_flipped("test/views", "", "hurt", 0, size, true).unwrap().clone(), "with the last left");
        let gold = [244, 198, 84, 255];
        assert_eq!(token_outline(&mut art, "test/views", "", &fr, size, gold, f).unwrap().clone(), img.outline(gold), "the outline follows the side frame");
        let fr = token_frame(&mut art, "test/views", "", &a.player, hit + 500, size, a.faces(hit + 500, 300)).unwrap().0;
        assert_eq!((fr.clip.as_str(), fr.side), ("idle_up", false), "after the hit: idle_up again");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// DIR4-G check 5: a body without view clips (the scratch side body, and every shipped body of the skirmish that
    /// has none) draws in any facing exactly as FACE1 did: `Art::played` mirrored when left, its outline the same.
    #[test]
    fn a_body_without_view_clips_draws_exactly_as_face1() {
        let (root, mut scratch) = view_bodies("side");
        let host = skirmish(3);
        let s = host.session.as_ref().unwrap();
        let mut shipped = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let bodies: Vec<String> = s.art.iter().filter_map(|chain| shipped.first_body(&emberwake_content::art::body_chain(chain))).collect();
        let gold = [244, 198, 84, 255];
        let facings: Vec<Facing> = [Look::Side, Look::Down, Look::Up].into_iter().flat_map(|view| [false, true].map(|left| Facing { view, left })).collect();
        let check = |art: &mut Art, body: &str| {
            for clip in ["idle", "walk", "hurt"] {
                let mut p = ClipPlayer::default();
                p.set_base("idle", 0);
                p.play(clip, 0);
                for &f in &facings {
                    let (fr, img) = token_frame(art, body, "", &p, 10, 40, f).map(|(fr, i)| (fr, i.clone())).unwrap();
                    let (was, old) = art.played(body, "", &p, 10, 40, f.left).map(|(fr, i)| (fr, i.clone())).unwrap();
                    assert_eq!((fr.clip.as_str(), fr.frame, &img), (was.clip.as_str(), was.frame, &old), "{body} {clip} in {f:?}");
                    let ring = token_outline(art, body, "", &fr, 40, gold, f).unwrap().clone();
                    assert_eq!(&ring, art.body_outline(body, "", &was.clip, was.frame, 40, gold, f.left).unwrap(), "{body} {clip} outline in {f:?}");
                }
            }
        };
        check(&mut scratch, "test/side");
        let mut checked = 0;
        for body in &bodies {
            if shipped.clips(body).is_some_and(|c| c.clips.iter().all(|c| c.view == Look::Side)) {
                check(&mut shipped, body);
                checked += 1;
            }
        }
        println!("shipped bodies without views checked: {checked} of {}", bodies.len());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Every rig clip (SPRITE1 section 2), one frame each, looping as the export writes them.
    fn rig_clips() -> Clips {
        let ids = ["idle", "walk", "attack", "jab", "sweep_kick", "kick", "shield_bash", "haymaker", "hurt", "death", "defend", "idle_guarded", "hurt_guarded"];
        let clips: Vec<_> = ids.iter().map(|id| {
            let looped = ["idle", "walk", "idle_guarded"].contains(id);
            json!({"id": id, "frames": 1, "ms": [100], "loop": looped})
        }).collect();
        Clips::parse(&json!({"frame": [48, 48], "clips": clips})).unwrap()
    }

    /// SPRITE1-U: played step by step, every token follows its unit: a move slides and walks, a hit plays hurt (guarded
    /// while it guards), a fall plays death, the resolved attack queues its actor's clip; skip plays nothing.
    #[test]
    fn tokens_follow_the_battle_step_by_step() {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let mut host = Host::new(Battle::with_data(Arc::new(Data::open(&dir).expect("data").0)));
        assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": 3}))["ok"], json!(true));
        let clips = rig_clips();
        let (mut t, mut skip) = (Tokens::default(), Tokens::default());
        let mut now = 0u64;
        t.catch_up(host.session.as_ref().unwrap(), now, false, 300, true);
        skip.catch_up(host.session.as_ref().unwrap(), now, true, 300, true);
        assert!(t.anims.iter().all(|a| a.player.playing(&clips, now).is_empty() && a.slide.is_none()), "a new battle plays nothing");
        let (mut moves, mut hits, mut falls, mut attacks) = (0, 0, 0, 0);
        for _ in 0..600 {
            let s = host.session.as_ref().unwrap();
            if s.combat_over {
                break;
            }
            if s.phase == Phase::Replay {
                if !s.can_advance_round() {
                    break;
                }
                host.handle(&json!({"cmd": "advance"}));
            }
            if host.session.as_ref().unwrap().is_planning() {
                host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
            }
            let before: Vec<_> = t.anims.iter().map(|a| a.seen.clone()).collect();
            let popped = host.session.as_ref().unwrap().last_popped;
            assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
            now += 1000;
            let s = host.session.as_ref().unwrap();
            t.catch_up(s, now, false, 300, true);
            skip.catch_up(s, now, true, 300, true);
            for (i, u) in s.w.units.iter().enumerate() {
                let (a, was) = (&t.anims[i], &before[i]);
                let guarded = u.guard > 0.0 || u.stance == Stance::Defensive;
                let playing = a.player.playing(&clips, now);
                if was.alive && !u.is_alive() {
                    falls += 1;
                    assert_eq!((a.fell, playing), (Some(now), "death"), "{} fell", u.name);
                } else if u.is_alive() && u.hp < was.hp {
                    hits += 1;
                    assert_eq!(playing, if guarded { "hurt_guarded" } else { "hurt" }, "{} was hit", u.name);
                } else if u.is_alive() && guarded && !was.guarded {
                    assert_eq!(playing, "defend", "{} raised a guard", u.name);
                } else if was.pos.is_some() && s.w.grid.pos(i).is_some() && was.pos != s.w.grid.pos(i) {
                    moves += 1;
                    assert_eq!((a.slide, playing), (Some((was.pos.unwrap(), now)), "walk"), "{} moved", u.name);
                }
                assert_eq!(a.player.base(), if guarded { "idle_guarded" } else { "idle" });
                let k = &skip.anims[i];
                assert!(k.player.playing(&clips, now).is_empty() && k.slide.is_none() && k.want.is_none(), "skip: the end state at once");
                assert_eq!(k.fell.is_some(), !u.is_alive());
            }
            if let Some((_, _, idx)) = s.last_popped.filter(|p| Some(*p) != popped) {
                let act = &s.resolver.scheduled[idx];
                let attack = matches!(act.command.kind, emberwake_battle::command::CmdKind::Attack { .. } | emberwake_battle::command::CmdKind::Area { .. });
                if attack && !act.cancelled && s.w.units[act.unit].is_alive() {
                    attacks += 1;
                    let want = t.anims[act.unit].want.as_ref().expect("the attacker queues its clip");
                    assert_eq!(want.1, now);
                    assert!(want.0.last() == Some(&"attack"), "{:?} ends with the generic attack", want.0);
                }
            }
        }
        assert!(moves > 0 && hits > 0 && falls > 0 && attacks > 0, "moves {moves}, hits {hits}, falls {falls}, attacks {attacks}");
    }

    /// A goblin_skirmish battle through the command host (the data's own encounter; no number of it is assumed).
    fn skirmish(seed: i64) -> Host {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let mut host = Host::new(Battle::with_data(Arc::new(Data::open(&dir).expect("data").0)));
        assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": seed}))["ok"], json!(true));
        host
    }

    /// DIR4-G (spec 4, check 3): a unit looks up or down at a target more vertical than horizontal (keeping its
    /// left/right), sideways at anything else (a diagonal tie keeps the side view); its own tile keeps the facing.
    #[test]
    fn facing_turns_four_ways_toward_the_target_and_a_diagonal_keeps_the_side_view() {
        let at = V2 { x: 5, y: 4 };
        let (right, left) = (Facing::side(false), Facing::side(true));
        assert_eq!(turn(left, at, V2 { x: 5, y: 6 }), Facing { view: Look::Down, left: true }, "two tiles straight below");
        assert_eq!(turn(right, at, V2 { x: 5, y: 2 }), Facing { view: Look::Up, left: false }, "two tiles straight above");
        assert_eq!(turn(left, at, V2 { x: 6, y: 5 }), right, "the diagonal (+1, +1): side, right");
        assert_eq!(turn(right, at, V2 { x: 4, y: 3 }), left, "the diagonal (-1, -1): side, left");
        assert_eq!(turn(right, at, V2 { x: 3, y: 4 }), left, "a target on the left");
        assert_eq!(turn(left, at, V2 { x: 7, y: 5 }), right, "a target more right than down");
        let up = Facing { view: Look::Up, left: true };
        assert_eq!(turn(up, at, at), up, "its own tile: unchanged");
    }

    /// FACE1 and DIR4-G G2: a new battle faces the party right and the foes left (side view); a step turns a unit
    /// toward it: up or down on a vertical step (keeping its left/right), sideways by its x otherwise.
    #[test]
    fn a_new_battle_faces_the_party_right_and_a_vertical_step_faces_up_or_down() {
        let mut host = skirmish(3);
        let mut t = Tokens::default();
        t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
        let s = host.session.as_ref().unwrap();
        for i in 0..s.w.units.len() {
            assert_eq!(t.anims[i].facing, Facing::side(!s.w.allies.contains(&i)), "{} at the start", s.w.units[i].name);
        }
        let h = s.w.allies[0];
        let up = if s.w.grid.pos(h).expect("the hero stands on the grid").y > 0 { -1 } else { 1 };
        let (go, back) = if up < 0 { (Look::Up, Look::Down) } else { (Look::Down, Look::Up) };
        let mut walk = |t: &mut Tokens, dx: i32, dy: i32| {
            let s = host.session.as_mut().unwrap();
            let at = s.w.grid.pos(h).unwrap();
            s.w.grid.positions[h] = Some(V2 { x: at.x + dx, y: at.y + dy });
            t.catch_up(s, 0, false, 300, true);
            assert_eq!(t.anims[h].step, t.anims[h].facing, "the slide shows the step's facing");
            t.anims[h].facing
        };
        assert_eq!(walk(&mut t, 0, up), Facing { view: go, left: false }, "a vertical step faces that way and keeps right");
        assert_eq!(walk(&mut t, -1, 0), Facing::side(true), "a step left turns left");
        assert_eq!(walk(&mut t, 0, -up), Facing { view: back, left: true }, "a vertical step keeps left");
        assert_eq!(walk(&mut t, -1, up), Facing::side(true), "a diagonal step goes by its x");
        assert_eq!(walk(&mut t, 1, 0), Facing::side(false), "a step right turns right");
    }

    /// FACE1 and DIR4-G G3: a unit that resolves an attack turns toward its target: left when the target stands left
    /// of it, right when it stands right (a foe too), up at a target straight above keeping its left/right; skip turns
    /// it the same.
    #[test]
    fn an_attacker_faces_its_target_on_either_side() {
        let mut host = skirmish(3);
        assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let (mut t, mut skip) = (Tokens::default(), Tokens::default());
        let attacks: Vec<(usize, usize, usize)> = {
            let s = host.session.as_ref().unwrap();
            s.resolver.scheduled.iter().enumerate().filter_map(|(idx, a)| match a.command.kind {
                CmdKind::Attack { target: Some(tg), .. } if !a.cancelled && tg != a.unit && s.w.grid.pos(tg).is_some() => Some((idx, a.unit, tg)),
                _ => None,
            }).collect()
        };
        let mut sides = (false, false);
        let mut block = 0;
        for (idx, actor, target) in attacks {
            let tp = host.session.as_ref().unwrap().w.grid.pos(target).unwrap();
            for (dx, dy, keep) in [(1, 0, None), (-1, 0, None), (0, 1, Some(true)), (0, 1, Some(false))] {
                let s = host.session.as_mut().unwrap();
                s.w.grid.positions[actor] = Some(V2 { x: tp.x + dx, y: tp.y + dy });
                t.catch_up(s, 0, false, 300, true);
                skip.catch_up(s, 0, true, 300, true);
                if let Some(k) = keep {
                    (t.anims[actor].facing, skip.anims[actor].facing) = (Facing::side(k), Facing::side(k));
                }
                block += 1;
                s.last_popped = Some((s.current_round, block, idx));
                t.catch_up(s, 0, false, 300, true);
                skip.catch_up(s, 0, true, 300, true);
                let want = match keep {
                    Some(k) => Facing { view: Look::Up, left: k },
                    None => Facing::side(dx > 0),
                };
                assert_eq!(t.anims[actor].rest(), want, "{} attacks from {dx},{dy} of its target", s.w.units[actor].name);
                assert_eq!(skip.anims[actor].rest(), want, "skip: the same facing");
            }
            if host.session.as_ref().unwrap().w.allies.contains(&actor) {
                sides.0 = true;
            } else {
                sides.1 = true;
            }
        }
        assert!(sides.0 && sides.1, "a hero's and a foe's attack were turned both ways: {sides:?}");
    }

    /// FACE1 (user 2026-10-23): with the enemy tells shown an enemy faces its intent's target while the party plans; with
    /// them hidden it keeps its facing through the planning and the confirm (until it acts). Heroes show their plan
    /// either way. An enemy is set left of its intent's target so its intent would turn it away from its start facing.
    #[test]
    fn hidden_tells_keep_the_enemies_facing_while_planning() {
        let mut host = skirmish(3);
        assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
        let (e, target) = {
            let s = host.session.as_ref().unwrap();
            s.enemy_intents().into_iter().find_map(|r| r.line.target.filter(|t| s.w.grid.pos(*t).is_some()).map(|t| (r.unit, t))).expect("an enemy intends a unit")
        };
        let s = host.session.as_mut().unwrap();
        let (tp, home) = (s.w.grid.pos(target).unwrap(), s.w.grid.positions[e]);
        s.w.grid.positions[e] = Some(V2 { x: tp.x - 1, y: tp.y });
        let (mut shown, mut hidden) = (Tokens::default(), Tokens::default());
        shown.catch_up(s, 0, false, 300, true);
        hidden.catch_up(s, 0, false, 300, false);
        assert_eq!(shown.anims[e].rest(), Facing::side(false), "tells shown: {} faces its intent's target on its right", s.w.units[e].name);
        assert_eq!(hidden.anims[e].rest(), Facing::side(true), "tells hidden: {} keeps its start facing (left)", s.w.units[e].name);
        for &h in &s.w.allies {
            assert_eq!(shown.anims[h].rest(), hidden.anims[h].rest(), "{} shows its plan either way", s.w.units[h].name);
        }
        let hero_planned = s.w.allies.iter().any(|&h| shown.anims[h].planned.is_some());
        assert!(hero_planned, "the aggressive preset gives a hero a target");
        // Back to its own tile (the test tile may be taken) so the plan confirms; the facing shown is kept. The tokens
        // take that tile as seen, so the teleport is no step.
        s.w.grid.positions[e] = home;
        (shown.anims[e].seen.pos, hidden.anims[e].seen.pos) = (home, home);
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let s = host.session.as_ref().unwrap();
        shown.catch_up(s, 0, false, 300, true);
        hidden.catch_up(s, 0, false, 300, false);
        assert_eq!(shown.anims[e].rest(), Facing::side(false), "tells shown: the confirm keeps the facing it showed");
        assert_eq!(hidden.anims[e].rest(), Facing::side(true), "tells hidden: still its own facing after the confirm");
    }

    /// FACE1 and DIR4-G, played step by step (rule-derived over several seeds): after every step each unit faces its
    /// step turned toward the target of the action it just resolved (`turn`: four ways; no step or no target keeps
    /// the facing); while the party plans an enemy faces its tell's target. The skip setting ends every step in the
    /// same facing (G5).
    #[test]
    fn units_face_their_targets_and_steps_and_skip_agrees() {
        let (mut turned_left, mut turned_right, mut turned_vertical) = (0, 0, 0);
        for seed in 1..=4 {
            let mut host = skirmish(seed);
            let (mut t, mut skip) = (Tokens::default(), Tokens::default());
            t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
            skip.catch_up(host.session.as_ref().unwrap(), 0, true, 300, true);
            let mut now = 0u64;
            for _ in 0..600 {
                let s = host.session.as_ref().unwrap();
                if s.combat_over {
                    break;
                }
                if s.phase == Phase::Replay {
                    if !s.can_advance_round() {
                        break;
                    }
                    host.handle(&json!({"cmd": "advance"}));
                }
                if host.session.as_ref().unwrap().is_planning() {
                    host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
                    let s = host.session.as_ref().unwrap();
                    t.catch_up(s, now, false, 300, true);
                    skip.catch_up(s, now, true, 300, true);
                    for r in s.enemy_intents() {
                        let to = r.line.target.and_then(|u| s.w.grid.pos(u)).unwrap_or(r.line.cell);
                        if let (Some(from), true) = (s.w.grid.pos(r.unit), to != NO_CELL) {
                            let want = turn(t.anims[r.unit].facing, from, to);
                            assert_eq!(t.anims[r.unit].rest(), want, "{} shows its tell's target", s.w.units[r.unit].name);
                        }
                    }
                }
                let before: Vec<(Facing, Option<V2>)> = t.anims.iter().map(|a| (a.rest(), a.seen.pos)).collect();
                let popped = host.session.as_ref().unwrap().last_popped;
                assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
                now += 1000;
                let s = host.session.as_ref().unwrap();
                t.catch_up(s, now, false, 300, true);
                skip.catch_up(s, now, true, 300, true);
                let acted = s.last_popped.filter(|p| Some(*p) != popped && p.0 == s.current_round).map(|(_, _, idx)| &s.resolver.scheduled[idx]).filter(|a| !a.cancelled && s.w.units[a.unit].is_alive());
                for i in 0..s.w.units.len() {
                    let aim = acted.filter(|a| a.unit == i).and_then(|a| match &a.command.kind {
                        CmdKind::Attack { target, .. } => target.and_then(|u| s.w.grid.pos(u).or(before[u].1)),
                        CmdKind::Area { aim, .. } => Some(*aim),
                        CmdKind::Support { target, .. } | CmdKind::Item { target, .. } => s.w.grid.pos(*target).or(before[*target].1),
                        _ => None,
                    });
                    let stepped = before[i].1.zip(s.w.grid.pos(i)).map_or(before[i].0, |(from, to)| turn(before[i].0, from, to));
                    let aimed = s.w.grid.pos(i).zip(aim.filter(|c| *c != NO_CELL));
                    let want = aimed.map_or(stepped, |(from, to)| turn(stepped, from, to));
                    assert_eq!(t.anims[i].rest(), want, "seed {seed}: {} after the step", s.w.units[i].name);
                    assert_eq!(skip.anims[i].rest(), want, "seed {seed}: skip ends {} in the same facing", s.w.units[i].name);
                    if let Some((from, to)) = aimed.filter(|(from, to)| from != to) {
                        match turn(stepped, from, to) {
                            Facing { view: Look::Side, left: true } => turned_left += 1,
                            Facing { view: Look::Side, left: false } => turned_right += 1,
                            _ => turned_vertical += 1,
                        }
                    }
                }
            }
        }
        assert!(turned_left > 0 && turned_right > 0 && turned_vertical > 0, "actions turned left {turned_left}, right {turned_right}, up or down {turned_vertical}");
    }

    /// The Test Arena on `goblin_ambush` (the forest clearing) with the game's art, in battle view `view`.
    pub(super) fn ambush(view: serde_json::Value) -> (crate::screens::Cx, aegis_ui::ui::Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.app.encounter = "goblin_ambush".into();
        cx.app.seed = Some(1);
        if !view.is_null() {
            cx.settings.v["battle_view"] = view;
        }
        let b = BattleScreen::sandbox(&cx);
        assert!(b.host.session.is_some(), "the ambush starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    /// DERIVE1 4.4: the selected hero's panel and every unit's hover card list the catalog derived stats shown in
    /// battle (by the battle's ruleset), the panel's line hovering to the formula and breakdown; with no such row the
    /// panel and the cards are as before.
    #[test]
    fn the_hero_panel_lists_battle_derived_stats_with_their_breakdown() {
        for shown in [false, true] {
            let mut cx = crate::hud::hud_tests::game_cx();
            let mut data = (*cx.app.data).clone();
            data.edit_cat(|cat| {
                cat.derived_stats.clear();
                if shown {
                    let r = emberwake_data::DerivedStatDef::from_obj(json!({"id": "nerve", "name": "Nerve", "formula": "KNOW * 2", "show_on": ["battle"]}).as_object().unwrap()).unwrap();
                    cat.derived_stats.insert(r.id.clone(), r);
                }
            });
            cx.app.data = Arc::new(data);
            cx.app.encounter = "goblin_ambush".into();
            cx.app.seed = Some(1);
            let mut b = BattleScreen::sandbox(&cx);
            let mut g = crate::hud::hud_tests::gfx();
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let s = b.host.session.as_ref().unwrap();
            let h = b.selected.expect("a hero is selected");
            let card = b.w.iter().find(|w| w.id == format!("party.{h}")).unwrap();
            let line = b.w.iter().find(|w| w.id == "hero.derived");
            if !shown {
                assert!(line.is_none() && !card.tip.iter().any(|t| t.0.starts_with("Nerve")));
                continue;
            }
            let v = emberwake_rules::derived::value(&s.w.cat, &s.w.ruleset.id, &s.w.units[h], "nerve").unwrap();
            let line = line.expect("the derived line");
            assert_eq!(line.text, format!("Nerve {v}"));
            assert_eq!(line.tip[0].0, format!("Nerve {v}"));
            assert!(line.tip[1].0.starts_with("Formula: KNOW * 2. KNOW "), "{:?}", line.tip);
            assert!(card.tip.iter().any(|t| t.0 == format!("Nerve {v}")), "the hover card: {:?}", card.tip);
        }
    }

    fn rect_of(b: &BattleScreen, id: &str) -> aegis_ui::canvas::Rect {
        b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}")).rect
    }

    /// STAGE1 check 10: the Battle view setting picks the board: Flat keeps today's 32 x 32 cells at (128, 30) and a
    /// unit's widget is its cell; Stage (also an unknown value and the default) the 32 x 24 cells from (128, 82).
    #[test]
    fn the_battle_view_setting_picks_the_stage_or_the_flat_board() {
        use aegis_ui::canvas::Rect;
        for (view, flat) in [(json!("flat"), true), (json!("stage"), false), (json!("sideways"), false), (serde_json::Value::Null, false)] {
            let (mut cx, mut g, mut b) = ambush(view.clone());
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let (cell, last) = (rect_of(&b, "tile.0.0"), rect_of(&b, "tile.11.8"));
            if flat {
                assert_eq!((cell, last), (Rect::new(128, 30, 32, 32), Rect::new(480, 286, 32, 32)), "{view}");
                let s = b.host.session.as_ref().unwrap();
                let h = s.w.allies[0];
                let p = s.w.grid.pos(h).unwrap();
                assert_eq!(rect_of(&b, &format!("unit.{h}")), Rect::new(128 + p.x * 32, 30 + p.y * 32, 32, 32), "a unit's widget is its cell");
            } else {
                assert_eq!((cell, last), (Rect::new(128, 82, 32, 24), Rect::new(480, 274, 32, 24)), "{view}");
            }
            assert_eq!(b.w.iter().filter(|w| w.id.starts_with("tile.")).count(), 12 * 9, "every cell keeps its widget");
        }
    }

    /// STAGE1 check 6: on the Stage the hero's topmost solid pixel (his helmet, above his cell) clicks him; the top-left
    /// 2 x 2 px of a free cell north of a unit click that cell; where two units' rects overlap the southern one wins;
    /// the backdrop and the front face hold no cell.
    #[test]
    fn stage_clicks_reach_heads_free_cells_and_the_unit_in_front() {
        let (mut cx, mut g, mut b) = ambush(json!("stage"));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let geo = Geom::new(View::Stage, &cx.tuning.value);
        let (h, pos, body, palette) = {
            let s = b.host.session.as_ref().unwrap();
            let h = s.w.allies[0];
            (h, s.w.grid.pos(h).unwrap(), cx.art.first_body(&crate::battle::body_keys(&s.art[h], &s.w.units[h])).expect("the hero has a body"), s.palettes.get(h).cloned().unwrap_or_default())
        };
        assert_eq!(h, 0, "the first hero is unit 0");
        // before the first change the tokens have no animations yet and a hero faces right (as the draw does)
        let left = b.tokens.anims.get(h).is_some_and(|a| a.rest().left);
        let bb = cx.art.body_bounds(&body, &palette, geo.token_px, left).expect("solid pixels");
        let frame = cx.art.body_clip_flipped(&body, &palette, "idle", 0, geo.token_px, left).unwrap().clone();
        let top_x = (bb.x..bb.x + bb.w).find(|x| frame.px[((bb.y * frame.w + x) * 4 + 3) as usize] >= aegis_ui::canvas::OUTLINE_ALPHA).unwrap();
        let tr = geo.token_rect(pos, 0, 0);
        let helmet = (tr.x + top_x, tr.y + bb.y);
        assert!(helmet.1 < geo.cell_rect(pos).y, "the helmet is above the hero's cell");
        assert_eq!(aegis_ui::ui::hit(&b.w, helmet), Some(Action::UnitClick(h)), "the helmet pixel clicks the hero");
        // A free cell north of a unit.
        let free = {
            let s = b.host.session.as_ref().unwrap();
            s.w.grid.positions.iter().flatten().map(|p| V2 { x: p.x, y: p.y - 1 }).find(|c| s.w.grid.is_valid(*c) && s.w.occupant(*c).is_none()).expect("a free cell north of a unit")
        };
        let r = geo.cell_rect(free);
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            assert_eq!(aegis_ui::ui::hit(&b.w, (r.x + dx, r.y + dy)), Some(Action::TileClick(free)), "the corner of free cell {free}");
        }
        for pt in [(300, 50), (300, 305)] {
            assert!(!matches!(aegis_ui::ui::hit(&b.w, pt), Some(Action::TileClick(_))), "{pt:?} holds no cell");
        }
        // Two units in one column: the southern one wins where their rects overlap.
        let (north, south) = {
            let s = b.host.session.as_mut().unwrap();
            let other = (0..s.w.units.len()).find(|i| *i != h && s.w.units[*i].is_alive() && s.w.grid.pos(*i).is_some()).expect("another unit");
            let below = if pos.y + 1 < GRID_H { V2 { x: pos.x, y: pos.y + 1 } } else { V2 { x: pos.x, y: pos.y - 1 } };
            s.w.grid.positions[other] = Some(below);
            if below.y > pos.y { (h, other) } else { (other, h) }
        };
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let (rn, rs) = (rect_of(&b, &format!("unit.{north}")), rect_of(&b, &format!("unit.{south}")));
        let m = rn.intersect(rs);
        assert!(m.w > 0 && m.h > 0, "the southern unit's head reaches into the northern one's rect");
        assert_eq!(aegis_ui::ui::hit(&b.w, (m.x + m.w / 2, m.y + m.h / 2)), Some(Action::UnitClick(south)), "the unit in front wins");
        assert_eq!(aegis_ui::ui::hit(&b.w, (rn.x + rn.w / 2, rn.y + 1)), Some(Action::UnitClick(north)), "the unit behind still clicks on its head");
    }

    /// DIR4-HB: on the Stage a unit facing down or up is hit by its front or back `idle` frame (`body_bounds_facing`;
    /// the side idle for a body without that view row): its widget rect is its cell joined with that frame's solid
    /// bounds, and the frame's topmost solid pixel clicks it.
    #[test]
    fn stage_clicks_follow_a_unit_facing_up_or_down() {
        use aegis_ui::canvas::{Rect, OUTLINE_ALPHA};
        let (mut cx, mut g, mut b) = ambush(json!("stage"));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let geo = Geom::new(View::Stage, &cx.tuning.value);
        let (h, pos, body, palette) = {
            let s = b.host.session.as_ref().unwrap();
            let h = s.w.allies[0];
            (h, s.w.grid.pos(h).unwrap(), cx.art.first_body(&crate::battle::body_keys(&s.art[h], &s.w.units[h])).expect("the hero has a body"), s.palettes.get(h).cloned().unwrap_or_default())
        };
        let viewed = [Look::Down, Look::Up].map(|view| cx.art.clips(&body).and_then(|c| c.view_row("idle", view).map(|r| c.clips[r].id.clone())));
        assert!(viewed.iter().any(|v| v.is_some()), "the hero's body has a front or back idle (DIR4-R2)");
        // before the first change the tokens have no animations yet: catch them up to the session
        let now = b.clock.now();
        b.tokens.catch_up(b.host.session.as_ref().unwrap(), now, true, 300, false);
        for (view, row) in [Look::Down, Look::Up].into_iter().zip(viewed) {
            for left in [false, true] {
                let f = Facing { view, left };
                {
                    let a = &mut b.tokens.anims[h];
                    a.facing = f;
                    a.planned = None;
                    a.slide = None;
                }
                b.draw(&mut cx, &mut g, (-500, -500), true);
                assert_eq!(b.tokens.anims[h].rest(), f, "the hero keeps the facing set");
                let (clip, flip) = match &row {
                    Some(id) => (id.clone(), false),
                    None => ("idle".to_string(), left),
                };
                let bb = cx.art.body_bounds_facing(&body, &palette, geo.token_px, f).expect("solid pixels");
                let frame = cx.art.body_clip_flipped(&body, &palette, &clip, 0, geo.token_px, flip).unwrap().clone();
                assert_eq!(frame.opaque_bounds(OUTLINE_ALPHA), Some(bb), "{f:?}: the bounds of {clip}");
                let tr = geo.token_rect(pos, 0, 0);
                let want = crate::stage::unit_rect(geo.cell_rect(pos), Rect::new(tr.x + bb.x, tr.y + bb.y, bb.w, bb.h));
                assert_eq!(rect_of(&b, &format!("unit.{h}")), want, "{f:?}: the widget follows the view's silhouette");
                let top_x = (bb.x..bb.x + bb.w).find(|x| frame.px[((bb.y * frame.w + x) * 4 + 3) as usize] >= OUTLINE_ALPHA).unwrap();
                assert_eq!(aegis_ui::ui::hit(&b.w, (tr.x + top_x, tr.y + bb.y)), Some(Action::UnitClick(h)), "{f:?}: the top of {clip} clicks the hero");
            }
        }
    }

    #[test]
    fn a_boss_tier_enemy_makes_a_boss_fight() {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let d = Data::open(&dir).expect("data").0;
        let fight = |id: &str| {
            let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", id));
            boss_fight(&Session::start(d.cat.clone(), def, &WorldRoster { d: &d, world: "ashlands".into() }))
        };
        assert!(fight("ashmaw_stand"));
        assert!(!fight("courtyard_skirmish"));
    }

    #[test]
    fn log_lines_pick_their_sounds() {
        assert_eq!(sound_of("[Attack] Rook struck Goblin for 7 (HP 3/10)"), Some("hit"));
        assert_eq!(sound_of("[Attack] Rook struck Goblin for 14 CRIT (HP 0/10)"), Some("crit"));
        assert_eq!(sound_of("[Attack] Rook struck Goblin for 0 (HP 10/10)"), Some("mitigated"));
        assert_eq!(sound_of("[Attack Miss] Rook: target no longer in combat"), Some("miss"));
        assert_eq!(sound_of("[Area Miss] Mage hit nobody"), Some("miss"));
        assert_eq!(sound_of("[Status] Hex -> Goblin (inflicts poison)"), Some("debuff"));
        assert_eq!(sound_of("[Status] Hex -> Goblin (resisted poison)"), None);
        assert_eq!(sound_of("[Status] Bard -> Rook (inflicts haste)"), None);
        assert_eq!(sound_of("[Defend] Rook raises guard (40% less damage)"), Some("defend"));
        assert_eq!(sound_of("[Defend] Rook's guard was broken"), None);
        assert_eq!(sound_of("[Support] Cleric used Mend on Rook: healed 6"), Some("heal"));
        assert_eq!(sound_of("[Move] Rook moved 2 of 2 tiles to (3, 4)"), Some("move_step"));
        assert_eq!(sound_of("[Rest] Rook AP 3/4"), None);
    }

    /// FX1-L1: plays the ambush step by step (the party on the aggressive preset) until a single step leaves a pending
    /// impact `want` accepts; earlier impacts fire before the next step (rule F). False when the battle ends first.
    pub(super) fn step_until(cx: &mut crate::screens::Cx, b: &mut BattleScreen, want: impl Fn(&BattleScreen, &crate::fx::Impact) -> bool) -> bool {
        for _ in 0..600 {
            let (over, phase, planning, can) = {
                let s = b.host.session.as_ref().unwrap();
                (s.combat_over, s.phase, s.is_planning(), s.can_advance_round())
            };
            if over || (phase == Phase::Replay && !can) {
                return false;
            }
            if phase == Phase::Replay {
                b.send(cx, json!({"cmd": "advance"}));
                continue;
            }
            if planning {
                b.send(cx, json!({"cmd": "preset", "preset": "aggressive"}));
            }
            b.flush(cx);
            b.floats.clear();
            b.flashes.clear();
            b.send(cx, json!({"cmd": "step"}));
            if b.impacts.last().is_some_and(|i| want(b, i)) {
                return true;
            }
        }
        false
    }

    /// The clips of unit `i`'s body (the game's art).
    fn unit_clips(cx: &mut crate::screens::Cx, b: &BattleScreen, i: usize) -> Clips {
        let s = b.host.session.as_ref().unwrap();
        let body = cx.art.first_body(&super::body_keys(&s.art[i], &s.w.units[i])).expect("a body");
        cx.art.clips(&body).unwrap().clone()
    }

    /// FX1-L1 L3 (rule I): between the pop and the impact the struck unit's board HP is the old one, no float, flash or
    /// hurt clip shows and the step's sounds wait; at the impact all of them start, the board HP is the battle's and
    /// the hit-stop holds `hit_stop_ms`. The impact of a blow is at its attack clip's `impact` frame (decision "Attack
    /// timing on the board" A).
    #[test]
    fn a_hit_shows_on_the_board_at_its_impact_not_at_the_pop() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        let found = step_until(&mut cx, &mut b, |b, i| {
            let s = b.host.session.as_ref().unwrap();
            i.struck.iter().any(|&u| s.w.units[u].is_alive())
        });
        assert!(found, "the ambush has a hit");
        b.resolve_impacts(&mut cx);
        let i = b.impacts[0].clone();
        let at = i.at.expect("resolved");
        let target = *i.struck.iter().find(|&&u| b.host.session.as_ref().unwrap().w.units[u].is_alive()).unwrap();
        let (hp_now, n) = { let s = b.host.session.as_ref().unwrap(); (s.w.units[target].hp, s.w.units.len()) };
        // before the impact
        let (shown_hp, alive) = b.tokens.shown(target, &b.host.session.as_ref().unwrap().w.units[target], n);
        assert!(alive && shown_hp > hp_now, "the board keeps the HP before the step ({shown_hp} vs {hp_now})");
        assert!(b.floats.is_empty() && b.flashes.is_empty(), "no float or flash before the impact");
        assert!(!i.sounds.is_empty() && !i.floats.is_empty(), "the step's sounds and floats wait in the impact");
        let clips = unit_clips(&mut cx, &b, target);
        assert!(!b.tokens.anims[target].player.playing(&clips, b.clock.now()).starts_with("hurt"), "no hurt clip yet");
        assert!(at >= i.t0, "the impact comes after t0");
        if i.delivery == crate::fx::Delivery::Melee {
            let actor_clips = unit_clips(&mut cx, &b, i.actor);
            let clip = i.clips.iter().copied().find(|c| actor_clips.clip(c).is_some());
            let want = clip.and_then(|c| actor_clips.clip(c).unwrap().impact_ms()).unwrap_or(cx.tuning.value.melee_hit_ms);
            assert_eq!(at, i.t0 + want, "a blow lands at its clip's impact frame");
        }
        // the impact
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        assert!(b.impacts.is_empty());
        let n = b.host.session.as_ref().unwrap().w.units.len();
        assert_eq!(b.tokens.shown(target, &b.host.session.as_ref().unwrap().w.units[target], n).0, hp_now, "the board HP is the battle's");
        assert!(b.floats.iter().any(|f| f.unit == target) && b.flashes.iter().any(|f| f.0 == target));
        assert!(b.tokens.anims[target].player.playing(&clips, at).starts_with("hurt"), "the hurt clip from the impact");
        assert_eq!(b.clock.hold, cx.tuning.value.hit_stop_ms as f64, "the hit-stop");
    }

    /// FX1-L1 L7 (4.2 step 1): an actor that moved in the step it acts (a dash, a charge, an Engage carry) starts its
    /// attack clip at the end of its slide, the walk showing until then; one that stood starts it at the pop.
    #[test]
    fn a_dash_starts_its_attack_after_the_slide() {
        let mut host = skirmish(3);
        assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let mut art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let s = host.session.as_mut().unwrap();
        let (idx, actor, target) = s.resolver.scheduled.iter().enumerate().find_map(|(idx, a)| match a.command.kind {
            CmdKind::Attack { target: Some(tg), .. } if !a.cancelled && tg != a.unit && s.w.grid.pos(tg).is_some() => Some((idx, a.unit, tg)),
            _ => None,
        }).expect("an attack");
        let body = art.first_body(&super::body_keys(&s.art[actor], &s.w.units[actor])).expect("a body");
        let clips = art.clips(&body).unwrap().clone();
        let tp = s.w.grid.pos(target).unwrap();
        let dir = if tp.x < emberwake_battle::grid::GRID_W / 2 { 1 } else { -1 };
        let walk = 300;
        let mut t = Tokens::default();
        s.w.grid.positions[actor] = Some(V2 { x: tp.x + 2 * dir, y: tp.y });
        t.catch_up(s, 0, false, walk, true);
        // the step: it closes in and its attack pops
        s.w.grid.positions[actor] = Some(V2 { x: tp.x + dir, y: tp.y });
        s.last_popped = Some((s.current_round, 1, idx));
        t.catch_up(s, 1000, false, walk, true);
        assert_eq!(t.anims[actor].want.as_ref().map(|w| w.1), Some(1000 + walk), "t0 is the slide's end");
        super::start_wanted(&mut t, &mut art, actor, Some(&body), 1000 + walk - 1);
        assert_eq!(t.anims[actor].player.playing(&clips, 1000 + walk - 1), "walk", "the walk until then");
        super::start_wanted(&mut t, &mut art, actor, Some(&body), 1000 + walk);
        assert!(t.anims[actor].want.is_none());
        let playing = t.anims[actor].player.playing(&clips, 1000 + walk).to_string();
        assert!(!playing.is_empty() && playing != "walk", "the attack clip from t0: {playing}");
        // standing still, the next pop starts it at once
        s.last_popped = Some((s.current_round, 2, idx));
        t.catch_up(s, 5000, false, walk, true);
        assert_eq!(t.anims[actor].want.as_ref().map(|w| w.1), Some(5000));
    }

    /// INPUT1 MR12 (`dash_tests`): a unit moved by a dash (its command's start this block made the `MoveKind::Dash`
    /// board event) slides in `DASH_MS` (a walk takes the tuning's walk), playing the rig's `dash` clip when the body
    /// has one, else its walk; under skip animations it stands on the position at once.
    #[test]
    fn dash_tests_a_dash_plays_its_clip_faster_than_a_walk() {
        let root = std::env::temp_dir().join(format!("emberwake_dash_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        scratch_body(&root, "test/dash", &["idle", "walk", "dash", "hurt"]);
        scratch_body(&root, "test/side", &["idle", "walk", "hurt"]);
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        let mut host = skirmish(3);
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let s = host.session.as_mut().unwrap();
        let (idx, actor) = s.resolver.scheduled.iter().enumerate().find(|(_, a)| !a.cancelled && s.w.grid.pos(a.unit).is_some()).map(|(i, a)| (i, a.unit)).expect("an action");
        let walk = 300;
        let here = s.w.grid.pos(actor).unwrap();
        let next = [V2::new(1, 0), V2::new(-1, 0), V2::new(0, 1), V2::new(0, -1)].into_iter().map(|d| here + d).find(|c| s.w.grid.is_valid(*c) && s.w.occupant(*c).is_none()).expect("a free neighbour");
        for (body, clip) in [("test/dash", "dash"), ("test/side", "walk")] {
            let clips = art.clips(body).unwrap().clone();
            s.w.grid.positions[actor] = Some(here);
            s.resolver.scheduled[idx].command.moved = None;
            let mut t = Tokens::default();
            t.catch_up(s, 0, false, walk, true);
            // a plain walk slides the walk's time
            s.w.grid.positions[actor] = Some(next);
            t.catch_up(s, 100, false, walk, true);
            assert_eq!((t.anims[actor].slide, t.anims[actor].slide_ms(walk)), (Some((here, 100)), walk), "{body}: a walk");
            // the dash back
            s.resolver.scheduled[idx].start = s.current_block;
            s.resolver.scheduled[idx].command.moved = Some(emberwake_battle::command::MoveKind::Dash);
            s.w.grid.positions[actor] = Some(here);
            t.catch_up(s, 1000, false, walk, true);
            assert_eq!((t.anims[actor].slide, t.anims[actor].slide_ms(walk)), (Some((next, 1000)), super::DASH_MS), "{body}: a dash runs DASH_MS");
            super::start_wanted(&mut t, &mut art, actor, Some(body), 1000);
            assert_eq!(t.anims[actor].player.playing(&clips, 1000 + super::DASH_MS - 1), clip, "{body}: the clip over the dash");
            assert_eq!(t.anims[actor].player.playing(&clips, 1000 + super::DASH_MS), "", "{body}: over after DASH_MS");
            // skip animations: on the position at once
            s.w.grid.positions[actor] = Some(next);
            t.catch_up(s, 5000, true, walk, true);
            assert!(t.anims[actor].slide.is_none_or(|(_, t0)| t0 != 5000) && t.anims[actor].seen.pos == Some(next), "{body}: instant");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// FX1-L1 L4: a unit killed by the step is drawn standing until the impact, then plays `death` from it.
    #[test]
    fn a_fall_waits_for_its_impact() {
        let (mut cx, _g, mut b) = ambush(json!("flat"));
        let found = step_until(&mut cx, &mut b, |b, i| {
            let s = b.host.session.as_ref().unwrap();
            i.struck.iter().any(|&u| !s.w.units[u].is_alive())
        });
        assert!(found, "the ambush has a fall");
        b.resolve_impacts(&mut cx);
        let i = b.impacts[0].clone();
        let n = b.host.session.as_ref().unwrap().w.units.len();
        let dead = *i.struck.iter().find(|&&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive()).unwrap();
        assert!(b.tokens.shown(dead, &b.host.session.as_ref().unwrap().w.units[dead], n).1, "standing until the impact");
        assert_eq!(b.tokens.anims[dead].fell, None);
        let at = i.at.unwrap();
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        let clips = unit_clips(&mut cx, &b, dead);
        assert!(!b.tokens.shown(dead, &b.host.session.as_ref().unwrap().w.units[dead], n).1);
        assert_eq!((b.tokens.anims[dead].fell, b.tokens.anims[dead].player.playing(&clips, at)), (Some(at), "death"));
    }

    /// FX1-L1 L5 (rule F) and L11: the next request fires a pending impact first; `resolve` shows its end state with
    /// no pending impact; under skip animations a step shows at once with no pending impact and no hit-stop.
    #[test]
    fn a_later_request_flushes_and_skip_shows_at_once() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a hit");
        let first = b.impacts[0].clone();
        b.send(&cx, json!({"cmd": "step"}));
        assert!(first.struck.iter().all(|u| b.flashes.iter().any(|f| f.0 == *u)), "the first impact fired before the next step");
        assert!(b.impacts.len() <= 1, "at most the second step's impact pends");
        b.send(&cx, json!({"cmd": "resolve"}));
        assert!(b.impacts.is_empty(), "resolve: the end state, no pending impact");
        let s = b.host.session.as_ref().unwrap();
        let n = s.w.units.len();
        assert!((0..n).all(|i| b.tokens.shown(i, &s.w.units[i], n) == (s.w.units[i].hp, s.w.units[i].is_alive())), "the board shows the end state");
        // skip animations
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        cx.settings.v["skip_anim"] = json!(true);
        let hold = b.clock.hold;
        let mut hits = 0;
        for _ in 0..200 {
            let s = b.host.session.as_ref().unwrap();
            if s.combat_over || s.phase == Phase::Replay {
                break;
            }
            if s.is_planning() {
                b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
            }
            b.flashes.clear();
            b.send(&cx, json!({"cmd": "step"}));
            assert!(b.impacts.is_empty(), "skip: no pending impact");
            hits += b.flashes.len();
        }
        assert!(hits > 0, "skip: the hits flash at once");
        assert_eq!(b.clock.hold, hold, "skip: no hit-stop");
    }

    /// FX1-L1 L6 (rule P): autoplay sends no step while an impact pends, nor within `impact_gap_ms` of its firing.
    #[test]
    fn autoplay_waits_for_the_impact_and_its_gap() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a hit");
        b.resolve_impacts(&mut cx);
        let sent = b.host.history.len();
        (b.playing, b.next_at) = (true, std::time::Instant::now());
        b.clock_at = std::time::Instant::now();
        b.tick(&mut cx);
        assert_eq!(b.host.history.len(), sent, "no step while the impact pends");
        b.clock.ms = b.impacts[0].at.unwrap() as f64;
        b.fire_due(&mut cx);
        let now = std::time::Instant::now();
        assert!(!b.impacts_done(now), "the gap after the impact");
        assert!(b.impacts_done(now + std::time::Duration::from_millis(cx.tuning.value.impact_gap_ms)), "then the next step may go");
    }

    /// FX1-L1 L14: effects are presentation only: the same requests give the same battle log and HP with skip
    /// animations on and off.
    #[test]
    fn effects_never_change_the_battle() {
        let run = |skip: bool| {
            let (mut cx, _g, mut b) = ambush(json!("stage"));
            cx.settings.v["skip_anim"] = json!(skip);
            for _ in 0..60 {
                let s = b.host.session.as_ref().unwrap();
                if s.combat_over || (s.phase == Phase::Replay && !s.can_advance_round()) {
                    break;
                }
                if s.phase == Phase::Replay {
                    b.send(&cx, json!({"cmd": "advance"}));
                } else {
                    if s.is_planning() {
                        b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
                    }
                    b.send(&cx, json!({"cmd": "step"}));
                }
            }
            let s = b.host.session.as_ref().unwrap();
            (s.resolver.log.clone(), s.w.units.iter().map(|u| u.hp).collect::<Vec<_>>())
        };
        assert_eq!(run(false), run(true));
    }

    /// The canvas's RGBA bytes at logical (`x`, `y`) (the test canvas is at scale 1).
    fn pixel(g: &aegis_ui::ui::Gfx, x: i32, y: i32) -> [u8; 4] {
        let k = ((y * g.canvas.w + x) * 4) as usize;
        [g.canvas.px[k], g.canvas.px[k + 1], g.canvas.px[k + 2], g.canvas.px[k + 3]]
    }

    /// FX1-L1 slice 2 (4.2 steps 2-3, 4.4): the pop puts the delivery on the live list (the trail of a blow, the
    /// projectile of a shot) from `t0`; the impact adds the row's ring, flash and sparks at the impact; the effects draw
    /// on the board in both views (a frame with them differs from the same frame without).
    #[test]
    fn an_action_pushes_its_delivery_at_the_pop_and_its_impact_effects_at_the_impact() {
        for view in ["stage", "flat"] {
            let (mut cx, mut g, mut b) = ambush(json!(view));
            assert!(step_until(&mut cx, &mut b, |_, i| i.hit() && (i.style.swing.is_some() || i.style.shot.is_some())), "{view}: a hit with a delivery");
            b.resolve_impacts(&mut cx);
            let i = b.impacts[0].clone();
            let at = i.at.unwrap();
            let want = if i.style.swing.is_some() { "trail" } else { "projectile" };
            assert!(b.fx.kinds(b.clock.now()).contains(&want), "{view}: the {want} from the pop");
            assert!(b.fx.live.iter().all(|e| e.start >= i.t0 && e.start <= at), "{view}: the delivery between t0 and the impact");
            b.clock.ms = at as f64;
            b.fire_due(&mut cx);
            let landed: Vec<&str> = b.fx.live.iter().filter(|e| e.start == at).map(|e| e.kind()).collect();
            let rows = [(i.style.hit.ring.is_some(), "ring"), (i.style.hit.flash.is_some(), "flash"), (i.style.hit.sparks.is_some(), "sparks")];
            for (has, kind) in rows {
                assert_eq!(landed.contains(&kind), has && (kind != "sparks" || !i.mods.miss), "{view}: {kind} at the impact ({landed:?})");
            }
            // the effects draw: the frame with them differs from the frame without
            b.clock.ms = (at + 20) as f64;
            b.floats.clear();
            b.flashes.clear();
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let with = g.canvas.px.clone();
            let live = std::mem::take(&mut b.fx);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_ne!(with, g.canvas.px, "{view}: the effects show");
            b.fx = live;
        }
    }

    /// FX1-L1 L9 (4.5) and L11: at its impact a 3-block skill that hit starts `heavy`, a 2-block one does not, a crit
    /// alone starts none, a break starts `heavy`; under reduce motion none starts (the effects and the hit-stop stay).
    #[test]
    fn heavy_hits_and_breaks_shake_the_board_at_their_impact() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a hit");
        b.resolve_impacts(&mut cx);
        let i = b.impacts.remove(0);
        let at = i.at.unwrap();
        let heavy = cx.tuning.value.shakes.get("heavy").unwrap().clamped(cx.tuning.value.shake_limits());
        let fire = |b: &mut BattleScreen, cx: &crate::screens::Cx, blocks: i32, broke: bool, crit: bool| {
            b.camera.stop();
            let mut j = i.clone();
            (j.blocks, j.broke, j.mods.crit) = (blocks, broke, crit);
            b.fire(cx, j, at);
            b.camera.shake.map(|s| (s.0, s.1))
        };
        assert_eq!(fire(&mut b, &cx, 3, false, false), Some((heavy, at)), "a Heavy hit");
        assert_eq!(fire(&mut b, &cx, 2, false, false), None, "a 2-block hit");
        assert_eq!(fire(&mut b, &cx, 1, false, true), None, "a crit");
        assert_eq!(fire(&mut b, &cx, 1, true, false), Some((heavy, at)), "a break");
        cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(true);
        b.fx.clear();
        b.clock.hold = 0.0;
        assert_eq!(fire(&mut b, &cx, 3, true, false), None, "reduce motion: no shake");
        assert!(!b.fx.is_empty() && b.clock.hold > 0.0, "reduce motion keeps the effects and the hit-stop");
    }

    /// FX1-L1 L9 (4.5 row 3) and L11: a newly fired phase whose effects hold `shake` `tremor` starts it at once, also
    /// under skip animations; a phase already seen does not start it again; reduce motion starts none.
    #[test]
    fn a_fired_phase_with_a_shake_starts_it_at_once() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        cx.settings.v["skip_anim"] = json!(true);
        let fire_phase = |b: &mut BattleScreen, id: &str| {
            let s = b.host.session.as_mut().unwrap();
            let effect: aegis_core::value::Obj = serde_json::from_value(json!({"type": emberwake_data::SHAKE_EFFECT, "preset": "tremor"})).unwrap();
            s.phases.phases.push(emberwake_battle::encounter::PhaseDef { id: id.into(), name: String::new(), trigger: Default::default(), effects: vec![effect] });
            s.phases.fired.insert(id.into(), (s.current_round, s.current_block));
        };
        fire_phase(&mut b, "fx_tremor");
        b.after_change(&cx);
        let tremor = cx.tuning.value.shakes.get("tremor").unwrap().clamped(cx.tuning.value.shake_limits());
        assert_eq!(b.camera.shake.map(|s| (s.0, s.1)), Some((tremor, b.clock.now())), "tremor at once, under skip too");
        b.camera.stop();
        b.after_change(&cx);
        assert_eq!(b.camera.shake, None, "a phase starts its shake once");
        cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(true);
        fire_phase(&mut b, "fx_tremor_2");
        b.after_change(&cx);
        assert_eq!(b.camera.shake, None, "reduce motion");
    }

    /// FX1-L2 M6 (4.4.4, 4.1): a unit the impact fells raises its dust ring and dust at its foot point on the ground
    /// layer, which the Stage draws (pass Arrows) before the feet pass; under skip animations none.
    #[test]
    fn a_fall_raises_dust_on_the_ground_at_its_impact() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |b, i| i.struck.iter().any(|&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive())), "a fall");
        b.resolve_impacts(&mut cx);
        let i = b.impacts[0].clone();
        let at = i.at.unwrap();
        let dead = *i.struck.iter().find(|&&u| !i.after[u].alive).unwrap();
        let geo = Geom::new(View::Stage, &cx.tuning.value);
        let foot = super::foot_point(&geo, b.host.session.as_ref().unwrap().w.grid.pos(dead).unwrap());
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        let dust: Vec<_> = b.fx.live.iter().filter(|e| e.layer == Layer::Ground && e.start == at && matches!(e.prim, aegis_ui::fx::Prim::Ring(r) if r.at == foot && r.r1 == 10.0)).collect();
        assert_eq!(dust.len(), 1, "the dust ring at the fallen unit's foot");
        let order = crate::stage::passes();
        let pos = |p: crate::stage::Pass| order.iter().position(|q| *q == p).unwrap();
        assert!(pos(crate::stage::Pass::Arrows) < pos(crate::stage::Pass::Feet), "the ground layer draws before the feet pass");
        // skip animations: no effects at the impact
        b.fx.clear();
        cx.settings.v["skip_anim"] = json!(true);
        b.fire(&cx, i, at);
        assert!(b.fx.is_empty());
    }

    /// FX1-L2 M4 (4.4.4, 4.2): a support's impact rises the gold motes from its target's foot point; its heal float
    /// waits for the impact.
    #[test]
    fn mend_motes_rise_at_the_targets_foot_and_its_float_waits_for_the_impact() {
        let (mut cx, _g, mut b) = ambush(json!("flat"));
        assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a step with feedback");
        b.resolve_impacts(&mut cx);
        let mut i = b.impacts.remove(0);
        let target = i.struck[0];
        let cell = b.host.session.as_ref().unwrap().w.grid.pos(target).unwrap();
        (i.style, i.area, i.to, i.struck, i.floats) = (fx::MEND, false, cell, Vec::new(), vec![(target, "+5".into(), cx.tuning.value.heal_color)]);
        let at = i.at.unwrap();
        b.floats.clear();
        b.impacts.push(i);
        b.fire_due(&mut cx);
        assert!(b.floats.is_empty(), "no heal float before the impact");
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        assert!(b.floats.iter().any(|f| f.unit == target && f.text == "+5"), "the heal float at the impact");
        let foot = super::foot_point(&Geom::new(View::Flat, &cx.tuning.value), cell);
        let motes = b.fx.live.iter().find_map(|e| match e.prim {
            aegis_ui::fx::Prim::Sparks(s) if e.start == at => Some(s),
            _ => None,
        });
        assert_eq!(motes.map(|s| (s.at, s.cols[0], s.ms)), Some((foot, fx::GOLD, 500)), "the gold motes at its foot");
    }

    /// FX1-L2 M7 (4.3): a missed attack's target steps 3 px away from its attacker and back at the impact; not under
    /// reduce motion nor skip animations; a missed shot flies past it.
    #[test]
    fn a_missed_target_sidesteps_unless_reduce_motion() {
        let (mut cx, mut g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |b, i| !i.area && i.target.is_some_and(|u| b.host.session.as_ref().unwrap().w.units[u].is_alive() && u != i.actor)), "an attack on a standing unit");
        b.resolve_impacts(&mut cx);
        let mut i = b.impacts.remove(0);
        i.mods = fx::Mods { miss: true, ..fx::Mods::default() };
        let (u, at) = (i.target.unwrap(), i.at.unwrap());
        for (reduce, want) in [(true, false), (false, true)] {
            cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(reduce);
            b.sidesteps.clear();
            b.fire(&cx, i.clone(), at);
            assert_eq!(b.sidesteps.iter().any(|s| s.0 == u && s.1 == at), want, "reduce motion {reduce}");
        }
        assert_ne!(fx::sidestep_of(&b.sidesteps, u, at + fx::SIDESTEP_OUT_MS), (0, 0), "3 px out at 80 ms");
        // the sidestep moves the target's sprite on the board
        b.clock.ms = (at + fx::SIDESTEP_OUT_MS) as f64;
        b.fx.clear();
        b.floats.clear();
        b.flashes.clear();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let with = g.canvas.px.clone();
        b.sidesteps.clear();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(with, g.canvas.px, "the target is drawn aside");
        cx.settings.v["skip_anim"] = json!(true);
        b.fire(&cx, i, at);
        assert!(b.sidesteps.is_empty(), "skip animations: no sidestep");
    }

    /// FX1-L2 M8 (4.8): the agent link's `state.fx` counts the live effects and pending impacts with their kinds and the
    /// camera offset, and goes to 0 after the last impact plus its longest effect.
    #[test]
    fn the_agent_link_reports_the_effects_until_they_end() {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |_, i| i.hit() && (i.style.swing.is_some() || i.style.shot.is_some())), "a hit with a delivery");
        b.resolve_impacts(&mut cx);
        let st = b.state(&cx)["fx"].clone();
        assert_eq!(st["pending"], json!(1));
        assert!(st["live"].as_u64().unwrap() >= 1 && st["kinds"].as_array().is_some_and(|k| !k.is_empty()), "{st}");
        assert_eq!(st["shake"], json!([0, 0]));
        let at = b.impacts[0].at.unwrap();
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        let st = b.state(&cx)["fx"].clone();
        assert_eq!(st["pending"], json!(0));
        assert_eq!(st["live"].as_u64().unwrap() as usize, b.fx.kinds(at).len());
        b.clock.ms = b.fx.busy_until().unwrap() as f64;
        b.clock.hold = 0.0;
        let st = b.state(&cx)["fx"].clone();
        assert_eq!((st["live"].clone(), st["pending"].clone(), st["kinds"].clone()), (json!(0), json!(0), json!([])), "all done");
    }

    /// FX1-L1 L12 (4.1): in both views the camera's offset moves a board pixel and leaves the planning strip and the
    /// hostile cards unchanged; the `unit.<i>` and `tile.<x>.<y>` hit rects do not move.
    #[test]
    fn the_camera_shakes_the_board_only() {
        use aegis_ui::canvas::Rect;
        for view in ["stage", "flat"] {
            let (mut cx, mut g, mut b) = ambush(json!(view));
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let before = g.canvas.px.clone();
            let rects = |b: &BattleScreen| b.w.iter().filter(|w| w.id.starts_with("unit.") || w.id.starts_with("tile.")).map(|w| (w.id.clone(), w.rect)).collect::<Vec<_>>();
            let hit_rects = rects(&b);
            let now = b.clock.now();
            let quake = cx.tuning.value.shakes.get("quake").unwrap();
            assert!(b.camera.start(quake, now, 1, cx.tuning.value.shake_limits(), false));
            assert_ne!(b.camera.offset_at(now), (0, 0), "{view}: the shake moves the camera now");
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let at = |buf: &[u8], x: i32, y: i32| {
                let k = ((y * g.canvas.w + x) * 4) as usize;
                [buf[k], buf[k + 1], buf[k + 2], buf[k + 3]]
            };
            let board = crate::stage::INTERIOR;
            let moved = (board.y..board.y + board.h).any(|y| (board.x..board.x + board.w).any(|x| at(&before, x, y) != pixel(&g, x, y)));
            assert!(moved, "{view}: a board pixel moved");
            for r in [Rect::new(2, 2, aegis_ui::ui::W - 4, 24), Rect::new(aegis_ui::ui::W - 126, 28, 124, 290)] {
                let same = (r.y..r.y + r.h).all(|y| (r.x..r.x + r.w).all(|x| at(&before, x, y) == pixel(&g, x, y)));
                assert!(same, "{view}: the HUD at {r:?} does not shake");
            }
            assert_eq!(rects(&b), hit_rects, "{view}: the hit rects stay");
        }
    }
}

/// WINDUP1-L, the acceptance of WINDUP1.md 6.3: scratch bodies and synthetic `Clips` with the section 4 rows (no shipped
/// body has them yet); the battles are the data's own encounters and every expectation comes from a rule (W1, the clip
/// rows), never from a number of the data.
#[cfg(test)]
mod windup_tests {
    use super::{command_pose, item_pose, plan_pose, skill_pose, start_wanted, windup_outline, BattleScreen, Pose, PoseSet, Tokens, AMBER};
    use crate::screens::Screen;
    use crate::tuning::Tuning;
    use aegis_core::assets::{body_dir, Clips, Layers};
    use aegis_ui::art::Art;
    use emberwake_battle::command::CmdKind;
    use emberwake_battle::plan::PlanStep;
    use emberwake_battle::session::Session;
    use emberwake_host::{Battle, Host};
    use emberwake_world::Data;
    use serde_json::{json, Value};
    use std::sync::Arc;

    const WALK: u64 = 300;

    /// Section 4's rows (frames, ms, `loop_from` as the spec), an `attack` with its own wind-up frames (`strike` 2,
    /// `impact` 3) and the rows every body has; `drop` leaves rows out (a body without them).
    fn rows(drop: &[&str]) -> Value {
        let windup = [90, 110, 120, 180, 180, 180, 180];
        let all = vec![
            json!({"id": "idle", "frames": 2, "ms": [200, 200], "loop": true}),
            json!({"id": "walk", "frames": 1, "ms": [100], "loop": true}),
            json!({"id": "idle_guarded", "frames": 1, "ms": [100], "loop": true}),
            json!({"id": "attack", "frames": 5, "ms": [80, 90, 100, 120, 140], "loop": false, "strike": 2, "impact": 3}),
            json!({"id": "hurt", "frames": 1, "ms": [100], "loop": false}),
            json!({"id": "hurt_guarded", "frames": 1, "ms": [100], "loop": false}),
            json!({"id": "defend", "frames": 1, "ms": [100], "loop": false}),
            json!({"id": "death", "frames": 1, "ms": [100], "loop": false}),
            json!({"id": "windup", "frames": 7, "ms": windup, "loop": true, "loop_from": 3}),
            json!({"id": "windup_heavy", "frames": 7, "ms": [100, 120, 140, 160, 160, 160, 160], "loop": true, "loop_from": 3}),
            json!({"id": "windup_buff", "frames": 7, "ms": windup, "loop": true, "loop_from": 3}),
            json!({"id": "windup_debuff", "frames": 7, "ms": windup, "loop": true, "loop_from": 3}),
            json!({"id": "stagger", "frames": 5, "ms": [60, 90, 110, 120, 160], "loop": false}),
            json!({"id": "dazed", "frames": 8, "ms": [150, 150, 150, 150, 150, 150, 150, 150], "loop": true}),
        ];
        Value::Array(all.into_iter().filter(|r| !drop.contains(&r["id"].as_str().unwrap_or(""))).collect())
    }

    fn clips(drop: &[&str]) -> Clips {
        Clips::parse(&json!({"frame": [8, 8], "clips": rows(drop)})).expect("the scratch rows parse")
    }

    /// The humanoid set (every row) and the creature set (A7: `windup`, `stagger`, `dazed` only).
    fn humanoid() -> PoseSet {
        PoseSet::of(&clips(&[]))
    }

    fn creature() -> PoseSet {
        PoseSet::of(&clips(&["windup_heavy", "windup_buff", "windup_debuff"]))
    }

    /// A scratch body `units/<key>` with `rows` (8 x 8 frames, each a bar in its row's colour).
    fn write_body(root: &std::path::Path, key: &str, rows: &Value) {
        const F: u32 = 8;
        let dir = root.join(body_dir(key));
        std::fs::create_dir_all(&dir).unwrap();
        let list = rows.as_array().unwrap();
        let cols = list.iter().map(|r| r["frames"].as_u64().unwrap() as u32).max().unwrap();
        let (w, h) = (cols * F, list.len() as u32 * F);
        let mut px = vec![0u8; (w * h * 4) as usize];
        for (row, r) in list.iter().enumerate() {
            for f in 0..r["frames"].as_u64().unwrap() as u32 {
                for x in [1u32, 2, 3, 6] {
                    let i = (((row as u32 * F + 4) * w + f * F + x) * 4) as usize;
                    px[i..i + 4].copy_from_slice(&[30 + 12 * row as u8, 40 + 20 * f as u8, 60, 255]);
                }
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": rows}).to_string()).unwrap();
    }

    fn data() -> Arc<Data> {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        Arc::new(Data::open(&dir).expect("data").0)
    }

    /// The data's `goblin_skirmish` (classic) or, with `ruleset`, under that ruleset.
    fn battle(ruleset: Option<&str>) -> Host {
        let mut host = Host::new(Battle::with_data(data()));
        let mut req = json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": 3});
        if let Some(r) = ruleset {
            req["ruleset"] = json!(r);
        }
        assert_eq!(host.handle(&req)["ok"], json!(true));
        host
    }

    /// Plans on the aggressive preset and confirms; the first live action over one block that starts at block 1 with a
    /// wind-up pose (W1, W1a): (its index, its unit, its pose).
    fn confirmed(host: &mut Host) -> (usize, usize, Pose) {
        assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let s = host.session.as_ref().unwrap();
        s.resolver
            .scheduled
            .iter()
            .enumerate()
            .find_map(|(idx, a)| {
                let pose = command_pose(s, &a.command);
                let attack = matches!(a.command.kind, CmdKind::Attack { .. } | CmdKind::Area { .. });
                (attack && !a.cancelled && a.start == 1 && a.impact > a.start && pose.held()).then_some((idx, a.unit, pose))
            })
            .expect("a block-1 attack over one block")
    }

    /// Tokens whose every unit has the pose set `ps`.
    fn tokens(s: &Session, ps: &PoseSet) -> Tokens {
        Tokens { poses: vec![ps.clone(); s.w.units.len()], ..Tokens::default() }
    }

    /// 6.3.1 and 8: rule W1 gives each command its pose and the bodies' fallbacks pick the clip.
    #[test]
    fn rule_w1_gives_each_command_its_pose_and_bodies_fall_back() {
        let d = data();
        let cat = &d.cat;
        let pose = |id: &str| skill_pose(cat.skill(id).unwrap_or_else(|| panic!("the skill {id}")));
        assert_eq!(pose("strike"), Pose::Attack);
        assert_eq!((pose("tl_heavy_cleave"), pose("tl_overhead_smash")), (Pose::Heavy, Pose::Heavy));
        assert_eq!((pose("mend"), pose("focus")), (Pose::Buff, Pose::Buff));
        assert_eq!((pose("active_guard"), pose("move")), (Pose::None, Pose::None));
        let mut hex = cat.skill("strike").unwrap().clone();
        hex.power_mult = 0.0;
        assert_eq!(skill_pose(&hex), Pose::Debuff, "an attack with power_mult 0 is a debuff");
        for it in cat.items.values().filter(|i| i.damage > 0 || i.heal > 0) {
            assert_eq!(item_pose(it), if it.aims_at_foes() { Pose::Debuff } else { Pose::Buff }, "{}", it.id);
        }
        let (h, c, bare) = (humanoid(), creature(), PoseSet::default());
        assert_eq!([h.clip(Pose::Heavy), h.clip(Pose::Debuff), h.clip(Pose::Buff), h.clip(Pose::Dazed)], [Some("windup_heavy"), Some("windup_debuff"), Some("windup_buff"), Some("dazed")]);
        assert_eq!([c.clip(Pose::Heavy), c.clip(Pose::Debuff), c.clip(Pose::Attack)], [Some("windup"); 3], "no heavy or debuff wind-up: the windup");
        assert_eq!(c.clip(Pose::Buff), None, "a creature on Support idles");
        assert!([Pose::Attack, Pose::Heavy, Pose::Buff, Pose::Debuff, Pose::Dazed, Pose::None].iter().all(|p| bare.clip(*p).is_none()), "a body without the rows shows no pose");
        assert_eq!((h.clip(Pose::Guard), bare.clip(Pose::Guard)), (Some("idle_guarded"), None), "a body without the rows stands as before");
    }

    /// 6.3.2, 6.3.8, 6.3.11 (classic): from the confirm the actor of a block-1 action over one block loops its wind-up
    /// from frame 0 (frame `loop_from` once the intro has run); at its pop its attack starts at the `strike` frame at t0.
    /// A body without the rows idles and strikes from frame 0, as before.
    #[test]
    fn a_two_block_action_winds_up_from_the_confirm_and_strikes_from_its_strike_frame() {
        let mut host = battle(None);
        let (idx, u, pose) = confirmed(&mut host);
        let k = clips(&[]);
        let ps = humanoid();
        let clip = ps.clip(pose).unwrap();
        let (mut t, mut bare) = (tokens(host.session.as_ref().unwrap(), &ps), Tokens::default());
        t.catch_up(host.session.as_ref().unwrap(), 1000, false, WALK, true);
        bare.catch_up(host.session.as_ref().unwrap(), 1000, false, WALK, true);
        let a = &t.anims[u];
        assert_eq!((a.player.base(), a.windup.map(|w| w.idx)), (clip, Some(idx)), "the wind-up from the confirm");
        let intro = k.ms_before(clip, 3);
        assert_eq!(a.player.frame(&k, 1000).unwrap().frame, 0);
        assert_eq!(a.player.frame(&k, 1000 + intro - 1).unwrap().frame, 2);
        assert_eq!(a.player.frame(&k, 1000 + intro).unwrap().frame, 3, "the hold from loop_from");
        assert_eq!(bare.anims[u].player.base(), "idle", "no rows: idle as before");
        // step until the action pops
        let root = std::env::temp_dir().join(format!("emberwake_windup_strike_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        write_body(&root, "test/windup", &rows(&[]));
        write_body(&root, "test/bare", &rows(&["windup", "windup_heavy", "windup_buff", "windup_debuff", "stagger", "dazed"]));
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        // play on until an action over one block pops live (its wind-up shown) and check its strike
        let mut now = 1000;
        let mut struck = None;
        for _ in 0..600 {
            let (over, phase, planning) = { let s = host.session.as_ref().unwrap(); (s.combat_over, s.phase, s.is_planning()) };
            if over {
                break;
            }
            if phase == emberwake_battle::session::Phase::Replay {
                host.handle(&json!({"cmd": "advance"}));
                continue;
            }
            if planning {
                host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
            }
            assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
            now += 1000;
            let s = host.session.as_ref().unwrap();
            let shown: Vec<Option<(i32, usize)>> = t.anims.iter().map(|a| a.windup.map(|w| (w.round, w.idx))).collect();
            t.catch_up(s, now, false, WALK, true);
            bare.catch_up(s, now, false, WALK, true);
            assert!(bare.anims.iter().all(|a| a.windup.is_none() && !a.player.base().starts_with("windup")), "no rows: no wind-up");
            let Some((round, _, idx)) = s.last_popped else { continue };
            let act = &s.resolver.scheduled[idx];
            if round == s.current_round && !act.cancelled && s.w.units[act.unit].is_alive() && t.anims[act.unit].want.is_some() && shown[act.unit] == Some((round, idx)) {
                struck = Some(act.unit);
                break;
            }
        }
        let u = struck.expect("an attack over one block pops live after its wind-up");
        let a = &t.anims[u];
        assert!(a.strike && a.windup.is_none(), "the strike follows the shown wind-up, which ends");
        let t0 = a.want.as_ref().unwrap().1;
        start_wanted(&mut t, &mut art, u, Some("test/windup"), t0);
        let f = t.anims[u].player.frame(&k, t0).unwrap();
        assert_eq!((f.clip.as_str(), f.frame), ("attack", k.clip("attack").unwrap().strike), "the attack from its strike frame at t0");
        assert!(!bare.anims[u].strike);
        let t0 = bare.anims[u].want.as_ref().unwrap().1;
        start_wanted(&mut bare, &mut art, u, Some("test/bare"), t0);
        assert_eq!(bare.anims[u].player.frame(&k, t0).unwrap().frame, 0, "no wind-up shown: the attack from frame 0");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 6.3.3 (W1a): an action whose span is one block shows no wind-up and strikes from frame 0.
    #[test]
    fn a_one_block_action_shows_no_windup_and_strikes_from_frame_0() {
        let mut host = battle(None);
        let (idx, u, _) = confirmed(&mut host);
        let s = host.session.as_mut().unwrap();
        s.resolver.scheduled[idx].impact = s.resolver.scheduled[idx].start;
        let mut t = tokens(s, &humanoid());
        t.catch_up(s, 1000, false, WALK, true);
        assert!(t.anims[u].windup.is_none() && !t.anims[u].player.base().starts_with("windup"), "one block: no wind-up");
        s.resolver.scheduled[idx].landed = true;
        s.last_popped = Some((s.current_round, 1, idx));
        t.catch_up(s, 2000, false, WALK, true);
        assert!(t.anims[u].want.is_some() && !t.anims[u].strike, "the attack from frame 0");
    }

    /// The first unit showing a wind-up after the confirm (`tokens` with every body's rows) at clock 1000.
    fn winding(host: &mut Host) -> (Tokens, usize, usize) {
        let (idx, u, _) = confirmed(host);
        let s = host.session.as_ref().unwrap();
        let mut t = tokens(s, &humanoid());
        t.catch_up(s, 1000, false, WALK, true);
        assert!(t.anims[u].windup.is_some());
        (t, idx, u)
    }

    /// 6.3.4 (W6, WQ2): a break mid wind-up plays `stagger` at the breaking impact instead of `hurt`, the wind-up ends
    /// then (not before) and nothing strikes; a guarding unit's break plays `stagger` too.
    #[test]
    fn a_break_staggers_at_its_impact_instead_of_a_hurt_and_never_strikes() {
        let mut host = battle(None);
        let (mut t, idx, u) = winding(&mut host);
        let k = clips(&[]);
        let s = host.session.as_mut().unwrap();
        s.resolver.scheduled[idx].cancelled = true;
        s.resolver.scheduled[idx].broken = true;
        s.w.poise.entry(u).or_default().broken = true;
        s.w.units[u].hp -= 1;
        let after = t.follow(s, 2000, false, WALK, true, true);
        assert!(t.anims[u].windup.is_some() && t.anims[u].player.base().starts_with("windup"), "the wind-up holds until the impact");
        t.react(&after, &[], 2100, false);
        let a = &t.anims[u];
        assert_eq!(a.player.playing(&k, 2100), "stagger", "the stagger, not hurt");
        assert!(a.windup.is_none() && !a.player.base().starts_with("windup") && a.want.is_none(), "the wind-up ended, no strike");
        // a guarding unit that is not winding up
        let g = (0..s.w.units.len()).find(|&i| i != u && t.anims[i].windup.is_none() && s.w.units[i].is_alive()).expect("another unit");
        s.w.units[g].guard = 1.0;
        t.catch_up(s, 3000, false, WALK, true);
        s.w.poise.entry(g).or_default().broken = true;
        s.w.units[g].hp -= 1;
        t.catch_up(s, 4000, false, WALK, true);
        assert_eq!(t.anims[g].player.playing(&k, 4000), "stagger", "a guard's break staggers too");
    }

    /// 6.3.5 (W7): the Staggered status landing plays `stagger` once, the base is `dazed` while it lasts, then `idle`.
    #[test]
    fn the_staggered_status_reels_then_stands_dazed_until_it_ends() {
        let mut host = battle(None);
        let (mut t, _, u) = winding(&mut host);
        let k = clips(&[]);
        let s = host.session.as_mut().unwrap();
        let v = (0..s.w.units.len()).find(|&i| i != u && t.anims[i].windup.is_none() && s.w.units[i].is_alive() && s.w.units[i].guard <= 0.0 && s.w.units[i].stance != emberwake_rules::Stance::Defensive).expect("a unit standing idle");
        s.w.units[v].st.pending.insert("stagger".into());
        t.catch_up(s, 2000, false, WALK, true);
        assert_eq!((t.anims[v].player.playing(&k, 2000), t.anims[v].player.base()), ("stagger", "dazed"));
        t.catch_up(s, 3000, false, WALK, true);
        assert_eq!((t.anims[v].player.playing(&k, 3000), t.anims[v].player.base()), ("", "dazed"), "once; dazed while it lasts");
        s.w.units[v].st.pending.shift_remove("stagger");
        t.catch_up(s, 4000, false, WALK, true);
        assert_eq!(t.anims[v].player.base(), "idle", "idle once it is gone");
    }

    /// 6.3.6 (W8): an actor that falls mid wind-up plays `death`, no `stagger`, even when broken by the same hit.
    #[test]
    fn an_actor_that_falls_mid_windup_plays_death_not_stagger() {
        let mut host = battle(None);
        let (mut t, idx, u) = winding(&mut host);
        let k = clips(&[]);
        let s = host.session.as_mut().unwrap();
        s.resolver.scheduled[idx].cancelled = true;
        s.w.poise.entry(u).or_default().broken = true;
        s.w.units[u].hp = 0;
        t.catch_up(s, 2000, false, WALK, true);
        assert_eq!(t.anims[u].player.playing(&k, 2000), "death");
        assert!(t.anims[u].windup.is_none() && t.anims[u].fell == Some(2000));
    }

    /// 6.3.7 (section 5, timeline): while planning a Guard intent stands guarded, an Assault with a Heavy Cleave opener in
    /// the heavy wind-up, Support in the buff wind-up, Staggered dazed; an enemy shows its tell's pose while the tells
    /// show (a creature body: its `windup`) and idles while they are hidden (WQ1).
    #[test]
    fn planning_shows_each_units_round_pose() {
        let mut host = battle(Some("timeline"));
        let s = host.session.as_mut().unwrap();
        let ps = humanoid();
        let mut t = tokens(s, &ps);
        for &e in &s.w.enemies.clone() {
            t.poses[e] = creature();
        }
        let h = s.w.allies.iter().copied().find(|&a| s.w.units[a].is_alive() && s.w.grid.pos(a).is_some()).expect("a hero");
        let foe = s.w.enemies.iter().copied().find(|&e| s.w.grid.pos(e).is_some()).expect("a foe");
        let cleave = s.w.cat.skills.iter().position(|k| k.id == "tl_heavy_cleave").expect("tl_heavy_cleave");
        s.set_round_intent(h, "guard").expect("guard");
        t.catch_up(s, 0, false, WALK, true);
        assert_eq!(t.anims[h].player.base(), "idle_guarded", "Guard");
        s.set_round_intent(h, "assault").expect("assault");
        s.plan.set_steps(h, vec![PlanStep::engage(cleave, foe)]);
        t.catch_up(s, 100, false, WALK, true);
        assert_eq!((t.anims[h].player.base(), t.anims[h].pose), ("windup_heavy", Pose::Heavy), "Assault, Heavy Cleave opener");
        t.catch_up(s, 200, false, WALK, true);
        assert_eq!(t.anims[h].player.frame(&clips(&[]), 200).unwrap().frame, t.anims[h].player.frame(&clips(&[]), 100).unwrap().frame + 1, "an unchanged pose keeps its clock");
        s.set_round_intent(h, "support").expect("support");
        s.plan.set_steps(h, Vec::new());
        t.catch_up(s, 300, false, WALK, true);
        assert_eq!(t.anims[h].player.base(), "windup_buff", "Support");
        s.w.units[h].st.active.insert("stagger".into());
        t.catch_up(s, 400, false, WALK, true);
        assert_eq!(t.anims[h].player.base(), "dazed", "Staggered");
        s.w.units[h].st.active.shift_remove("stagger");
        // the enemies: their tell's pose with the tells shown, idle with them hidden
        let shown: Vec<(usize, Pose)> = s.w.enemies.iter().map(|&e| (e, plan_pose(s, e, true, &[]))).collect();
        let (e, p) = shown.iter().copied().find(|(_, p)| matches!(p, Pose::Attack | Pose::Heavy | Pose::Debuff)).expect("an enemy about to attack");
        t.catch_up(s, 500, false, WALK, true);
        assert_eq!((t.anims[e].player.base(), p.held()), ("windup", true), "an attacking enemy with the tells shown");
        t.catch_up(s, 600, false, WALK, false);
        assert!(s.w.enemies.iter().all(|&e| t.anims[e].player.base() == "idle" || t.anims[e].player.base() == "idle_guarded"), "the tells hidden: no pose");
    }

    /// 6.3.11 (classic, no intents): the planning pose comes from the first command: a Defend first stands guarded.
    #[test]
    fn classic_planning_poses_come_from_the_first_command() {
        let mut host = battle(None);
        let s = host.session.as_mut().unwrap();
        let mut t = tokens(s, &humanoid());
        let h = s.w.allies[0];
        let defend = s.w.cat.skills.iter().position(|k| emberwake_battle::skill::kind(k) == emberwake_battle::skill::Kind::Defend).expect("a defend skill");
        let strike = s.w.cat.skills.iter().position(|k| k.id == "strike").expect("strike");
        let foe = s.w.enemies[0];
        s.plan.set_steps(h, vec![PlanStep::self_skill(defend), PlanStep::engage(strike, foe)]);
        t.catch_up(s, 0, false, WALK, true);
        assert_eq!(t.anims[h].player.base(), "idle_guarded", "a Defend first");
        s.plan.set_steps(h, vec![PlanStep::engage(strike, foe)]);
        t.catch_up(s, 100, false, WALK, true);
        assert_eq!(t.anims[h].player.base(), "windup", "an attack first");
        assert!(windup_outline(&t.anims[h], &Tuning::default()).is_none(), "no amber outline while planning (WQ5)");
    }

    /// 6.3.9 (6.1): skip animations starts a wind-up at its `loop_from` and plays no `stagger` (the dazed base still
    /// shows); reduce motion plays the intro and then holds frame `loop_from`.
    #[test]
    fn skip_starts_the_hold_and_reduce_motion_holds_it_still() {
        let k = clips(&[]);
        let mut host = battle(None);
        let (idx, u, pose) = confirmed(&mut host);
        let s = host.session.as_mut().unwrap();
        let ps = humanoid();
        let clip = ps.clip(pose).unwrap();
        let mut skip = tokens(s, &ps);
        skip.catch_up(s, 1000, true, WALK, true);
        assert_eq!(skip.anims[u].player.frame(&k, 1000).unwrap().frame, 3, "skip: the hold at once");
        let mut still = tokens(s, &ps);
        still.still = true;
        still.catch_up(s, 1000, false, WALK, true);
        let intro = k.ms_before(clip, 3);
        assert_eq!(still.anims[u].player.frame(&k, 1000 + intro - 1).unwrap().frame, 2, "reduce motion: the intro plays");
        for dt in [0, 300, 900, 2000] {
            assert_eq!(still.anims[u].player.frame(&k, 1000 + intro + dt).unwrap().frame, 3, "reduce motion: the hold stands still");
        }
        s.resolver.scheduled[idx].cancelled = true;
        s.w.poise.entry(u).or_default().broken = true;
        s.w.units[u].st.active.insert("stagger".into());
        skip.catch_up(s, 2000, true, WALK, true);
        assert_eq!((skip.anims[u].player.playing(&k, 2000), skip.anims[u].player.base()), ("", "dazed"), "skip: no stagger, the dazed base");
    }

    /// 6.3.10 (W9, WQ5): the amber outline only while a wind-up shows (`windup_outline_alpha`, 0 off).
    #[test]
    fn the_amber_outline_shows_only_during_a_windup() {
        let mut host = battle(None);
        let (t, _, u) = winding(&mut host);
        let tu = Tuning::default();
        assert_eq!(windup_outline(&t.anims[u], &tu), Some([AMBER[0], AMBER[1], AMBER[2], tu.windup_outline_alpha]));
        let idle = t.anims.iter().find(|a| a.windup.is_none()).expect("a unit not winding up");
        assert_eq!(windup_outline(idle, &tu), None);
        assert_eq!(windup_outline(&t.anims[u], &Tuning { windup_outline_alpha: 0, ..Tuning::default() }), None, "0: off");
    }

    /// The Test Arena on `goblin_ambush` whose every unit's body is a scratch body with the section 4 rows (layered
    /// over the game's art).
    fn windup_arena(tag: &str) -> (crate::screens::Cx, aegis_ui::ui::Gfx, BattleScreen, std::path::PathBuf) {
        let mut cx = crate::hud::hud_tests::game_cx();
        let game = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        cx.art = Art::new(Some(game.clone()));
        cx.app.encounter = "goblin_ambush".into();
        cx.app.seed = Some(1);
        cx.settings.v["battle_view"] = json!("flat");
        let b = BattleScreen::sandbox(&cx);
        let root = std::env::temp_dir().join(format!("emberwake_windup_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let s = b.host.session.as_ref().unwrap();
        for (i, u) in s.w.units.iter().enumerate() {
            if let Some(key) = cx.art.first_body(&super::body_keys(&s.art[i], u)) {
                write_body(&root, &key, &rows(&[]));
            }
        }
        let mut layers = Layers::for_game(&game);
        layers.roots.insert(0, root.clone());
        cx.art = Art::layered(layers);
        (cx, crate::hud::hud_tests::gfx(), b, root)
    }

    /// 6.3.2 (W4, the screen): a strike from a held wind-up lands `ms_before(impact) - ms_before(strike)` after t0.
    #[test]
    fn a_strike_from_the_pose_lands_its_strike_to_impact_ms_after_t0() {
        let (mut cx, _g, mut b, root) = windup_arena("impact");
        assert!(b.learn_poses(&mut cx));
        b.sync_anims(&cx);
        assert!(b.tokens.poses.iter().any(|p| p.has("windup")), "the bodies' rows are known");
        let found = super::tests::step_until(&mut cx, &mut b, |_, i| i.strike && i.delivery == crate::fx::Delivery::Melee);
        assert!(found, "a melee strike from a wind-up");
        b.resolve_impacts(&mut cx);
        let i = b.impacts.last().unwrap().clone();
        let k = clips(&[]);
        let c = k.clip("attack").unwrap();
        assert_eq!(i.at, Some(i.t0 + c.impact_ms().unwrap() - c.ms_before(c.strike)), "from the strike frame to the impact frame");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 6.3.12 (6.2): the agent link's `state.tokens` names, per unit, the clip drawn, the base, the pose and the wind-up.
    #[test]
    fn the_agent_link_tokens_match_the_drawn_clips() {
        let (mut cx, mut g, mut b, root) = windup_arena("state");
        b.learn_poses(&mut cx);
        b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
        b.send(&cx, json!({"cmd": "confirm"}));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let st = b.state(&cx)["tokens"].clone();
        let s = b.host.session.as_ref().unwrap();
        let list = st.as_array().expect("tokens");
        assert_eq!(list.len(), s.w.units.len());
        let now = b.clock.now();
        let mut winding = 0;
        for (i, e) in list.iter().enumerate() {
            let a = &b.tokens.anims[i];
            let body = cx.art.first_body(&super::body_keys(&s.art[i], &s.w.units[i])).expect("a body");
            let drawn = a.player.frame(cx.art.clips(&body).unwrap(), now).unwrap().clip;
            assert_eq!((e["unit"].as_u64(), e["clip"].as_str(), e["base"].as_str(), e["pose"].as_str()), (Some(i as u64), Some(drawn.as_str()), Some(a.player.base()), Some(a.pose.id())));
            match a.windup {
                Some(w) => {
                    winding += 1;
                    assert_eq!(e["windup"], json!([w.blocks.0, w.blocks.1]));
                    assert!(e["pose"].as_str().is_some_and(|p| !p.is_empty()) && e["base"].as_str().is_some_and(|c| c.starts_with("windup")));
                }
                None => assert!(e["windup"].is_null()),
            }
        }
        assert!(winding > 0, "some unit winds up after the confirm");
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod arena_rules_tests {
    use super::*;
    use emberwake_rules::ruleset::{Ruleset, PLAN_BADGES, TIMELINE};

    /// RULESET1-G (index 12): the Arena passes its extension switches and "Reveal enemy plans" to the battle (and
    /// they survive Restart); without them the request is the classic one: no ruleset, no reveal, no state key.
    #[test]
    fn the_arena_plays_by_its_switches_and_reveal() {
        let cx = crate::hud::hud_tests::game_cx();
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let classic = BattleScreen::arena(&cx, None, &encounter);
        let s = classic.host.session.as_ref().expect("the battle starts");
        assert!(s.w.ruleset.is_classic() && s.w.ruleset.rules.is_empty() && !s.w.plan_reveal);
        assert!(classic.state(&cx).get("ruleset").is_none(), "a classic battle's state keeps its shape");
        let timeline = Ruleset::of(&cx.data.cat, TIMELINE).expect("the catalog's timeline row");
        let off = timeline.rules.iter().copied().find(|id| emberwake_rules::ruleset::rule(id).is_some_and(|r| r.kind == emberwake_rules::ruleset::RuleKind::Extension) && *id != PLAN_BADGES).expect("an extension besides plan_badges");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.into()), &format!("-{off},+{PLAN_BADGES}"), true);
        for start in ["first", "restarted"] {
            let s = b.host.session.as_ref().expect("the battle starts");
            assert!(!s.w.ruleset.has(off) && s.w.ruleset.has(PLAN_BADGES), "{start}: the switches apply");
            assert!(s.w.plan_reveal, "{start}: the plans are revealed");
            let st = b.state(&cx);
            assert_eq!((st["ruleset"]["id"].clone(), st["ruleset"]["plan_reveal"].clone()), (json!(TIMELINE), json!(true)), "{start}");
            // what the Arena's Restart does (without the playtest hub's save)
            b.start_battle();
        }
    }

    /// INPUT1 MR11: the Arena's "Diagonal melee" pick reaches the `new` request (recorded), the battle plays by it and
    /// keeps it on Restart; the battle's state and chip name it; "" (Ruleset) sends nothing and the row's value plays.
    #[test]
    fn the_arena_diagonal_melee_pick_reaches_the_battle_and_survives_restart() {
        let cx = crate::hud::hud_tests::game_cx();
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_full(&cx, None, &encounter, Some(TIMELINE.into()), "", "all", false);
        for start in ["first", "restarted"] {
            let s = b.host.session.as_ref().expect("the battle starts");
            assert_eq!(s.w.ruleset.melee_diagonal(), emberwake_rules::ruleset::MeleeDiagonal::All, "{start}");
            let st = b.state(&cx);
            assert_eq!(st["ruleset"]["melee_diagonal"], json!("all"), "{start}");
            assert!(st["ruleset"]["chip"].as_str().unwrap_or("").contains("diagonal: all"), "{start}: {}", st["ruleset"]["chip"]);
            let history = b.host.handle(&json!({"cmd": "history"}))["history"].as_array().cloned().unwrap_or_default();
            assert!(history.iter().any(|q| q["cmd"] == json!("new") && q["melee_diagonal"] == json!("all")), "{start}: the request carries it");
            b.start_battle();
        }
        let mut row = BattleScreen::arena_full(&cx, None, &encounter, Some(TIMELINE.into()), "", "", false);
        let history = row.host.handle(&json!({"cmd": "history"}))["history"].as_array().cloned().unwrap_or_default();
        assert!(history.iter().all(|q| q.get("melee_diagonal").is_none()), "Ruleset: nothing sent");
        let timeline = Ruleset::of(&cx.data.cat, TIMELINE).expect("the catalog's timeline row");
        assert_eq!(row.host.session.as_ref().unwrap().w.ruleset.melee_diagonal(), timeline.melee_diagonal(), "the row's value");
    }
}

/// STATUSFX1-L, the acceptance of STATUSFX1.md section 4 on the screen (the pure rules are tested in `fx.rs`): the
/// battles are the data's own `goblin_ambush`; a status an action puts on is injected into the session before that
/// action's step (the slice's status skills are not in the shipped data), and every expectation comes from a rule.
#[cfg(test)]
mod status_tests {
    use super::tests::{ambush, step_until};
    use super::{draw_status_loops, Action, BattleScreen, StatusOn};
    use crate::fx;
    use crate::screens::{Cx, Screen};
    use crate::stage::{Geom, View};
    use crate::tuning::Tuning;
    use aegis_ui::fx::{Layer, Prim};
    use aegis_ui::ui::Ui;
    use emberwake_battle::session::Phase;
    use serde_json::json;

    /// Plays the ambush as `step_until` does, calling `before` with the step's number just before each step; stops after
    /// the step `stop` accepts and returns its number (None: the battle ended first).
    fn play(cx: &mut Cx, b: &mut BattleScreen, mut before: impl FnMut(&mut BattleScreen, usize), stop: impl Fn(&BattleScreen, usize) -> bool) -> Option<usize> {
        let mut k = 0;
        for _ in 0..600 {
            let (over, phase, planning, can) = {
                let s = b.host.session.as_ref().unwrap();
                (s.combat_over, s.phase, s.is_planning(), s.can_advance_round())
            };
            if over || (phase == Phase::Replay && !can) {
                return None;
            }
            if phase == Phase::Replay {
                b.send(cx, json!({"cmd": "advance"}));
                continue;
            }
            if planning {
                b.send(cx, json!({"cmd": "preset", "preset": "aggressive"}));
            }
            b.flush(cx);
            b.floats.clear();
            b.flashes.clear();
            before(b, k);
            b.send(cx, json!({"cmd": "step"}));
            if stop(b, k) {
                return Some(k);
            }
            k += 1;
        }
        None
    }

    /// The first step whose action lands on another unit that still stands, and that unit.
    fn hit_step() -> (usize, usize) {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        let lands = |b: &BattleScreen| b.impacts.last().and_then(|i| i.target.filter(|&u| u != i.actor && i.after.get(u).is_some_and(|a| a.alive)));
        let k = play(&mut cx, &mut b, |_, _| {}, |b, _| lands(b).is_some()).expect("the ambush has an action landing on a unit");
        (k, lands(&b).unwrap())
    }

    fn poison_on(u: usize, at: usize) -> impl FnMut(&mut BattleScreen, usize) {
        move |b, k| {
            if k == at {
                b.host.session.as_mut().unwrap().w.units[u].st.pending.insert("poison".into());
            }
        }
    }

    fn burst(e: &aegis_ui::fx::Effect) -> bool {
        matches!(e.prim, Prim::Ring(r) if r.col == [fx::VIOLET[0], fx::VIOLET[1], fx::VIOLET[2], 200] && r.ms == fx::BURST_MS)
    }

    /// 4 (S4, S5): a status an action puts on bursts at that action's impact, not at its pop, and shows (pending) from
    /// it; none under skip animations, where it shows at once.
    #[test]
    fn a_status_an_action_puts_on_bursts_at_its_impact_not_at_the_pop() {
        let (k, u) = hit_step();
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        assert_eq!(play(&mut cx, &mut b, poison_on(u, k), |_, j| j == k), Some(k));
        let pop = b.clock.now();
        let i = b.impacts.last().cloned().expect("the same step lands an impact");
        assert_eq!(i.target, Some(u));
        assert!(i.marks[u].contains(&("poison", true)), "the step's statuses wait in the impact");
        assert!(!b.tokens.anims[u].marks.iter().any(|m| m.0 == "poison"), "not shown before the impact");
        assert!(!b.fx.live.iter().any(burst), "no burst at the pop");
        b.resolve_impacts(&mut cx);
        let at = b.impacts[0].at.expect("resolved");
        assert!(at > pop, "the impact comes after the pop");
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        let bursts: Vec<_> = b.fx.live.iter().filter(|e| burst(e)).collect();
        assert_eq!(bursts.len(), 1, "one burst");
        assert_eq!((bursts[0].start, bursts[0].layer), (at, Layer::Air), "at the impact, in the air");
        let geo = Geom::new(View::Stage, &cx.tuning.value);
        let (fx_, fy_) = geo.foot(b.host.session.as_ref().unwrap().w.grid.pos(u).unwrap());
        assert!(matches!(bursts[0].prim, Prim::Ring(r) if r.at == fx::slot_anchor(&geo, (fx_ as f32, fy_ as f32), fx::Slot::Body)), "at its chest");
        assert!(b.tokens.anims[u].marks.contains(&("poison", true, at)), "shown pending from the impact, its loop from the burst");

        let (mut cx, _g, mut b) = ambush(json!("stage"));
        cx.settings.v["skip_anim"] = json!(true);
        assert_eq!(play(&mut cx, &mut b, poison_on(u, k), |_, j| j == k), Some(k));
        assert!(b.impacts.is_empty(), "skip: no impact");
        assert!(b.tokens.anims[u].marks.iter().any(|m| m.0 == "poison"), "skip: shown at once");
        assert!(!b.fx.live.iter().any(burst), "skip: no burst");
    }

    /// 3 (S3): a status only pending shows pending; at the round start, when it turns active, it shows full, its loop
    /// running on (no new burst: no action).
    #[test]
    fn a_pending_status_turns_full_when_it_turns_active() {
        let (cx, _g, mut b) = ambush(json!("stage"));
        b.sync_anims(&cx);
        let v = (0..b.host.session.as_ref().unwrap().w.units.len()).find(|&i| b.host.session.as_ref().unwrap().w.units[i].is_alive()).unwrap();
        b.host.session.as_mut().unwrap().w.units[v].st.pending.insert("poison".into());
        b.clock.ms = 500.0;
        b.sync_anims(&cx);
        assert_eq!(b.tokens.anims[v].marks, [("poison", true, 500)]);
        let st = &mut b.host.session.as_mut().unwrap().w.units[v].st;
        st.pending.shift_remove("poison");
        st.active.insert("poison".into());
        b.clock.ms = 900.0;
        b.sync_anims(&cx);
        assert_eq!(b.tokens.anims[v].marks, [("poison", false, 500)], "full alpha, the same loop");
        assert!(b.fx.live.is_empty(), "no burst without an action");
    }

    /// 6 (S1, rule I): a unit that falls draws its looks until its fall at the impact, none after; a stealthed unit draws
    /// its sprite and outline at 55 % alpha (`status_fx` names it).
    #[test]
    fn a_fallen_unit_draws_no_looks_and_a_stealthed_one_is_faded() {
        let (mut cx, mut g, mut b) = ambush(json!("stage"));
        assert!(step_until(&mut cx, &mut b, |b, i| i.struck.iter().any(|&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive())), "the ambush has a fall");
        let dead = *b.impacts[0].struck.iter().find(|&&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive()).unwrap();
        b.tokens.anims[dead].marks = vec![("burn", false, 0)];
        b.impacts[0].marks[dead] = vec![("burn", false)];
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(b.tokens.anims[dead].status_fx, ["burn"], "burning until its fall");
        b.resolve_impacts(&mut cx);
        let at = b.impacts[0].at.unwrap();
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(b.tokens.anims[dead].marks.iter().any(|m| m.0 == "burn") && b.tokens.anims[dead].status_fx.is_empty(), "none from its fall");

        for view in ["stage", "flat"] {
            let (mut cx, mut g, mut b) = ambush(json!(view));
            b.sync_anims(&cx);
            let s = b.host.session.as_ref().unwrap();
            let v = (0..s.w.units.len()).find(|&i| s.w.units[i].is_alive() && s.w.grid.pos(i).is_some()).unwrap();
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let plain = g.canvas.px.clone();
            b.tokens.anims[v].marks = vec![("stealth", false, 0)];
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_eq!(b.tokens.anims[v].status_fx, ["stealth"], "{view}");
            assert_ne!(plain, g.canvas.px, "{view}: the stealthed token is faded");
            assert_eq!(fx::token_alpha(&[fx::Drawn { look: fx::status_look("stealth").unwrap(), pending: false, start: 0 }], &cx.tuning.value), fx::STEALTH_ALPHA);
        }
    }

    /// 8 (S9): each `state.tokens` entry's `status_fx` names the looks drawn, slot order head, body, feet, tint, alpha.
    #[test]
    fn the_agent_link_tokens_name_the_status_looks_drawn() {
        for view in ["stage", "flat"] {
            let (mut cx, mut g, mut b) = ambush(json!(view));
            b.sync_anims(&cx);
            let standing: Vec<usize> = {
                let s = b.host.session.as_ref().unwrap();
                (0..s.w.units.len()).filter(|&i| s.w.units[i].is_alive() && s.w.grid.pos(i).is_some()).collect()
            };
            assert!(standing.len() >= 3);
            let (a, c, d) = (standing[0], standing[1], standing[2]);
            {
                let w = &mut b.host.session.as_mut().unwrap().w;
                for id in ["burn", "stun"] {
                    w.units[a].st.active.insert(id.into());
                }
                w.units[c].st.pending.insert("bleed".into());
                for id in ["stealth", "fortify"] {
                    w.units[d].st.active.insert(id.into());
                }
            }
            b.sync_anims(&cx);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let st = b.state(&cx)["tokens"].clone();
            let list = st.as_array().expect("tokens");
            for (i, e) in list.iter().enumerate() {
                let want: &[&str] = match i {
                    _ if i == a => &["stun", "burn"],
                    _ if i == c => &["bleed"],
                    _ if i == d => &["fortify", "stealth"],
                    _ => &[],
                };
                assert_eq!(e["status_fx"], json!(want), "{view}: unit {i}");
                assert_eq!(e["status_fx"], json!(b.tokens.anims[i].status_fx), "{view}: unit {i} as drawn");
            }
        }
    }

    /// 5 (S7): under reduce motion the loops draw still (the same at any clock); otherwise they move.
    #[test]
    fn reduce_motion_draws_the_loops_still() {
        let t = Tuning::default();
        let geo = Geom::new(View::Stage, &t);
        let looks = ["stun", "burn"].iter().map(|id| fx::Drawn { look: fx::status_look(id).unwrap(), pending: false, start: 0 }).collect();
        let on = [StatusOn { unit: 0, foot: (300.0, 200.0), left: false, looks }];
        let render = |now: u64, still: bool, on: &[StatusOn]| {
            let mut g = crate::hud::hud_tests::gfx();
            {
                let mut ui: Ui<Action> = Ui::new(&mut g, (-1, -1), true);
                draw_status_loops(&mut ui.p, on, Layer::Air, &geo, &t, now, still);
            }
            g.canvas.px
        };
        let blank = render(100, true, &[]);
        assert_ne!(render(100, true, &on), blank, "still looks show");
        assert_eq!(render(100, true, &on), render(700, true, &on), "the same at any clock");
        assert_ne!(render(100, false, &on), render(350, false, &on), "they move otherwise");
    }

    /// Decision "FX1-L2 answers": a unit felled with no action to land (a status tick) raises FX1-L2's fall dust at once
    /// at its foot point on the ground layer; none under skip animations.
    #[test]
    fn a_unit_felled_without_an_impact_raises_its_dust_at_once() {
        for skip in [false, true] {
            let (mut cx, _g, mut b) = ambush(json!("stage"));
            cx.settings.v["skip_anim"] = json!(skip);
            b.sync_anims(&cx);
            let u = {
                let s = b.host.session.as_ref().unwrap();
                (0..s.w.units.len()).find(|&i| s.w.enemies.contains(&i) && s.w.units[i].is_alive() && s.w.grid.pos(i).is_some()).unwrap()
            };
            b.host.session.as_mut().unwrap().w.units[u].hp = 0;
            b.clock.ms = 2000.0;
            b.after_change(&cx);
            let geo = Geom::new(View::Stage, &cx.tuning.value);
            let (fx_, fy_) = geo.foot(b.host.session.as_ref().unwrap().w.grid.pos(u).unwrap());
            let dust = b.fx.live.iter().filter(|e| e.layer == Layer::Ground && e.start == 2000 && matches!(e.prim, Prim::Ring(r) if r.at == (fx_ as f32, fy_ as f32) && r.r1 == 10.0)).count();
            assert_eq!((b.impacts.len(), dust), (0, usize::from(!skip)), "skip {skip}: dust at once, none under skip");
        }
    }
}

/// RULESET1-U slice 1 (RULESET1_INTENT UI1, UI2, MM1-MM3; the Arena chip, user 2026-10-29): the selected hero's round
/// controls on the battle screen. A catalog encounter played by the catalog's timeline row; every expectation comes
/// from the kit's planner (`round_intent_plan::preview`, `reasons`, `order`, `rest_preview`) and the intent table
/// (`round_intent::target_problem`, `would_drop`), never from shipped numbers.
#[cfg(test)]
mod input1_tests {
    use super::*;
    use emberwake_rules::ruleset::{rule, RuleKind, Ruleset, PLAN_BADGES, TIMELINE};

    fn arena(ruleset: Option<&str>, rules: &str, reveal: bool) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), rules, reveal);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
        b.draw(cx, g, (-500, -500), true);
    }

    fn widget<'a>(b: &'a BattleScreen, id: &str) -> &'a Widget<Action> {
        b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}"))
    }

    fn tip_has(w: &Widget<Action>, text: &str) -> bool {
        w.tip.iter().any(|t| t.0.contains(text))
    }

    fn hero(b: &BattleScreen) -> usize {
        b.selected.expect("a hero is selected")
    }

    fn w(b: &BattleScreen) -> &World {
        &b.host.session.as_ref().unwrap().w
    }

    /// The first valid cell no unit stands on.
    fn free_tile(w: &World) -> V2 {
        (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).find(|c| w.grid.is_valid(*c) && (0..w.units.len()).all(|i| w.grid.pos(i) != Some(*c))).expect("a free tile")
    }

    /// A timeline Arena battle at its first planning (the tick drafts the ghosts, GH2 a), drawn.
    fn planning() -> (Cx, Gfx, BattleScreen) {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
        b.tick(&mut cx);
        drawn(&mut cx, &mut g, &mut b);
        (cx, g, b)
    }

    fn foes_on_field(b: &BattleScreen) -> Vec<usize> {
        w(b).enemies.iter().copied().filter(|&e| w(b).units[e].is_active() && w(b).grid.pos(e).is_some()).collect()
    }

    /// Puts the selected hero on a free tile next to `f` (so its melee skills reach), the battle catching up.
    fn next_to(cx: &Cx, b: &mut BattleScreen, f: usize) {
        let h = hero(b);
        let fp = w(b).pos(f);
        let cell = emberwake_battle::grid::DIRS.iter().map(|d| fp + *d).find(|c| w(b).grid.is_valid(*c) && w(b).grid.is_walkable(*c) && !w(b).grid.is_exit(*c) && w(b).occupant(*c).is_none_or(|o| o == h)).expect("a free tile next to the foe");
        b.host.session.as_mut().unwrap().w.grid.positions[h] = Some(cell);
        b.after_change(cx);
    }

    fn round_plan(b: &mut BattleScreen) -> Value {
        b.host.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone()
    }

    /// The intent of a skill-menu row (its first skill or item pick's; None: neutral).
    fn row_intent(b: &BattleScreen, row: &Row) -> Option<String> {
        match &row.act {
            RowAct::Pick(picks) => picks.iter().find_map(|p| match p {
                rip::Pick::Skill { skill, .. } => Some(ri::skill_intent(&w(b).cat.skills[*skill]).to_string()),
                _ => None,
            }),
            _ => None,
        }
    }

    fn first_valid_pick(b: &BattleScreen) -> usize {
        b.popup.as_ref().expect("the skill menu").rows.iter().position(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if p.iter().any(|x| x.holds_target()))).expect("a usable skill row")
    }

    /// IF1, IF2, IF4: a click on an enemy sets the round target and opens the skill menu at the click, titled "<hero> ->
    /// <target>", with only skills an enemy takes (no Support or Guard row); a pick appends it and the menu stays open,
    /// re-listed: only rows that can still be picked, of the round intent.
    #[test]
    fn a_target_click_opens_the_skill_menu_and_a_pick_relists_it() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Unit(f)), "the click sets the target at once: {}", b.message);
        let p = b.popup.as_ref().expect("the skill menu");
        assert_eq!((p.menu, p.title.clone()), (Some((h, RoundTarget::Unit(f))), format!("{} -> {}", w(&b).units[h].name, w(&b).units[f].name)));
        let s = b.host.session.as_ref().unwrap();
        assert_eq!(p.rows.len(), rip::pick_options(&s.w, &s.plan, h, RoundTarget::Unit(f)).len(), "one row per kit option");
        for row in &p.rows {
            assert!(row_intent(&b, row).is_none_or(|i| i == "assault" || i == "maneuver"), "an enemy takes Assault and Maneuver skills: {}", row.label);
        }
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        assert_eq!(rip::picks_of(w(&b), h).len(), 1, "{}", b.message);
        let intent = ri::intent_of(w(&b), h).map(|i| i.id().to_string());
        if let Some(p) = b.popup.as_ref() {
            assert!(p.rows.iter().all(|r| r.valid), "after the first pick no disabled row");
            assert!(p.rows.iter().all(|r| row_intent(&b, r).is_none_or(|i| Some(i) == intent)), "only the round intent's rows");
        }
        let rp = round_plan(&mut b);
        assert_eq!(rp.as_array().unwrap().iter().find(|r| r["unit"] == json!(h)).unwrap()["picks"].as_array().unwrap().len(), 1);
        drawn(&mut cx, &mut g, &mut b);
    }

    /// IF2, IF6 b: after a pick on one enemy another enemy offers no skill: "One target per round: <target>".
    #[test]
    fn another_enemy_after_a_pick_offers_no_skill() {
        let (mut cx, _g, mut b) = planning();
        let foes = foes_on_field(&b);
        assert!(foes.len() >= 2, "the arena fight has two enemies on the field");
        let h = hero(&b);
        next_to(&cx, &mut b, foes[0]);
        b.run(&mut cx, Action::UnitClick(foes[0]));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        assert!(!rip::picks_of(w(&b), h).is_empty(), "{}", b.message);
        b.run(&mut cx, Action::ClosePopup);
        b.run(&mut cx, Action::UnitClick(foes[1]));
        assert!(b.popup.as_ref().is_none_or(|p| !p.rows.iter().any(|r| matches!(r.act, RowAct::Pick(_)))), "no skill row");
        assert_eq!(b.message, format!("One target per round: {}", w(&b).units[foes[0]].name));
    }

    /// IF5: hovering a row previews it on the strip; leaving it restores the strip; the round plan never changes.
    #[test]
    fn hovering_a_row_previews_it_and_changes_nothing() {
        let (mut cx, mut g, mut b) = planning();
        let f = foes_on_field(&b)[0];
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        drawn(&mut cx, &mut g, &mut b);
        let before = round_plan(&mut b);
        let k = first_valid_pick(&b);
        let r = widget(&b, &format!("popup.{k}")).rect;
        b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
        assert!(b.preview.as_ref().is_some_and(|p| p.1 == k && p.2.is_some()), "the hovered row is previewed");
        assert_eq!(round_plan(&mut b), before, "a preview changes nothing");
        drawn(&mut cx, &mut g, &mut b);
        assert!(b.preview.is_none(), "leaving the row restores the strip");
        assert_eq!(round_plan(&mut b), before);
    }

    /// IF10: with the menu open a right click (back) or a click outside it closes it and does nothing else.
    #[test]
    fn closing_the_menu_does_nothing_else() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        b.run(&mut cx, Action::UnitClick(f));
        drawn(&mut cx, &mut g, &mut b);
        let p = b.popup.clone().expect("the skill menu");
        let outside = [(1, 1), (W - 2, H - 2), (1, H - 2)].into_iter().find(|&(x, y)| !p.rect(b.menu_row).contains(x, y)).unwrap();
        assert_eq!(aegis_ui::ui::hit(&b.w, outside), Some(Action::ClosePopup), "a click outside lands on the backdrop");
        let before = (round_plan(&mut b), b.selected);
        b.back(&mut cx);
        assert!(b.popup.is_none());
        assert_eq!((round_plan(&mut b), b.selected), before, "the right click only closed the menu");
        assert_eq!(b.selected, Some(h));
    }

    /// IF8, SL4: the intent buttons, kind and opener menus, Manual and Fill are gone; the panel names the intent and its
    /// source; the Self button (and key S) opens the self menu.
    #[test]
    fn the_round_controls_are_gone_and_self_opens_the_self_menu() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        assert!(b.w.iter().all(|x| !x.id.starts_with("hero.intent.") && !["hero.kind", "hero.opener", "hero.manual", "hero.fill"].contains(&x.id.as_str())), "the old controls are gone");
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        if ri::intent_of(w(&b), h).is_some() {
            let tactic = TACTIC_LABELS[b.tactic_of(h)];
            assert!(recs.iter().any(|r| r.text.starts_with("Intent: ") && r.text.ends_with(&format!("from {tactic}"))), "the ghost's intent from the tactic");
        }
        b.click_id(&mut cx, "hero.self").unwrap();
        assert_eq!(b.popup.as_ref().and_then(|p| p.menu), Some((h, RoundTarget::Myself)), "{}", b.message);
        b.run(&mut cx, Action::ClosePopup);
        b.key(&mut cx, Key::Char('s'));
        assert_eq!(b.popup.as_ref().and_then(|p| p.menu), Some((h, RoundTarget::Myself)));
    }

    /// HK1-HK2: with the menu open a digit picks its row; with it closed the hotkey skill goes on the round target;
    /// without a target the digit asks for one.
    #[test]
    fn digits_pick_the_hotkey_skills() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        let d = (1..=9).find(|d| b.popup.as_ref().unwrap().rows.iter().any(|r| r.valid && r.label.starts_with(&format!("{d} ")))).expect("a usable hotkey row");
        b.key(&mut cx, Key::Char(char::from_digit(d as u32, 10).unwrap()));
        assert_eq!(rip::picks_of(w(&b), h).len(), 1, "{}", b.message);
        b.run(&mut cx, Action::ClosePopup);
        b.run(&mut cx, Action::Clear);
        assert!(rip::picks_of(w(&b), h).is_empty(), "{}", b.message);
        b.key(&mut cx, Key::Char(char::from_digit(d as u32, 10).unwrap()));
        assert_eq!(rip::picks_of(w(&b), h).len(), 1, "menu closed: the skill on the round target: {}", b.message);
        // no target and no intent: the digit of a skill that is not self-only asks for a target
        let keys = rip::hotkey_skills(w(&b), h);
        let other = keys.iter().position(|k| { let sk = &w(&b).cat.skills[*k]; sk.target_mode != "SELF" && emberwake_battle::skill::kind(sk) != Kind::Defend }).filter(|k| *k < 9).expect("a hotkey skill aimed at others");
        b.run(&mut cx, Action::Clear);
        let rs = b.host.session.as_mut().unwrap().w.round_states.entry(h).or_default();
        rs.target = None;
        rs.intent = None;
        b.after_change(&cx);
        drawn(&mut cx, &mut g, &mut b);
        b.key(&mut cx, Key::Char(char::from_digit(other as u32 + 1, 10).unwrap()));
        assert_eq!(b.message, "Pick a target first (click a unit or a tile)");
    }

    /// GH2 a: at the planning start each hero without a pick shows its tactic's draft as ghost; GH2 b: a clicked target
    /// is player-set, and Clear after a pick returns to the auto plan on it; the hero tactic returns it to (a).
    #[test]
    fn heroes_show_their_tactic_ghost_and_clear_returns_to_it() {
        let (mut cx, _g, mut b) = planning();
        for a in w(&b).allies.clone().into_iter().filter(|&a| w(&b).units[a].is_active() && w(&b).has(a)) {
            let s = b.host.session.as_ref().unwrap();
            assert!(rip::picks_of(&s.w, a).is_empty() && rip::ghost_entries(&s.w, &s.plan, a).iter().all(|g| *g), "{a}: all ghost");
            let mut w2 = s.w.clone();
            let mut p2 = rip::copy_plan(&s.plan);
            emberwake_battle::ai::plan_party_intents(&mut w2, &mut p2, &[(a, TACTICS[b.tactic_of(a)].to_string())]);
            assert_eq!(plan_rows(&s.w, &s.plan, a), plan_rows(&w2, &p2, a), "{a}: the tactic's draft");
        }
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        assert!(b.player_target.contains(&h));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        b.run(&mut cx, Action::ClosePopup);
        b.run(&mut cx, Action::Clear);
        let s = b.host.session.as_ref().unwrap();
        let mut w2 = s.w.clone();
        let mut p2 = rip::copy_plan(&s.plan);
        rip::plan_unit(&mut w2, &mut p2, h).unwrap();
        assert_eq!(plan_rows(&s.w, &s.plan, h), plan_rows(&w2, &p2, h), "(b): the auto plan on the clicked target");
        assert_eq!(s.w.round_states[&h].target, Some(RoundTarget::Unit(f)), "the target stays");
        b.run(&mut cx, Action::HeroMenu);
        b.run(&mut cx, Action::Choose(b.tactic_of(h)));
        assert!(!b.player_target.contains(&h), "the hero tactic: back to (a)");
    }

    /// GH7: Auto-fill off reaches the battle and the free blocks Rest (the ghost of a hero with no pick, the blocks
    /// after a pick).
    #[test]
    fn auto_fill_off_rests_the_free_blocks() {
        let (mut cx, _g, mut b) = planning();
        cx.settings.v["auto_fill"] = json!(false);
        b.tick(&mut cx);
        assert!(!w(&b).autofill, "the setting reached the battle");
        let rest_only = |b: &BattleScreen, a: usize, from: usize| {
            let s = b.host.session.as_ref().unwrap();
            s.plan.entries_for(&s.w, a).iter().skip(from).filter(|e| e.step_index >= 0).all(|e| emberwake_battle::skill::kind(&s.w.cat.skills[e.skill]) == Kind::Rest)
        };
        for a in w(&b).allies.clone().into_iter().filter(|&a| w(&b).units[a].is_active()) {
            assert!(rest_only(&b, a, 0), "{a}: Rest on every block");
        }
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        let s = b.host.session.as_ref().unwrap();
        let from = rip::ghost_from(&s.w, &s.plan, h);
        assert!(rest_only(&b, h, from), "after the pick the free blocks Rest");
    }

    /// GH2, IN6 (Q18 a): a clicked target whose intent change would drop an intent-bound buff waits for Confirm: the
    /// buff stays, the panel warns "Changing loses ..."; Confirm makes the change.
    #[test]
    fn a_ghost_that_drops_a_bound_buff_waits_for_confirm() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "guard", "target": "self"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let s = b.host.session.as_mut().unwrap();
        emberwake_rules::status::grant_now(&mut s.w.units[h], "riposte", 2, 0, "test");
        ri::bind(&mut s.w, h, h, &["riposte".to_string()], false);
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        assert!(b.deferred.iter().any(|d| d.0 == h), "the change waits: {}", b.message);
        assert_eq!(ri::intent_of(w(&b), h), Some(RoundIntent::Guard));
        assert!(w(&b).units[h].has_status("riposte"), "the buff stays until Confirm");
        b.run(&mut cx, Action::ClosePopup);
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        assert!(recs.iter().any(|r| r.text.starts_with("Changing loses")), "the panel warns");
        b.run(&mut cx, Action::Confirm);
        assert_ne!(ri::intent_of(w(&b), h), Some(RoundIntent::Guard), "{}", b.message);
        assert!(!w(&b).units[h].has_status("riposte"));
    }

    /// GH8: a ghost block's tip starts "Auto-filled" with its reason tag; a picked block's does not.
    #[test]
    fn ghost_blocks_say_auto_filled() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        b.run(&mut cx, Action::ClosePopup);
        drawn(&mut cx, &mut g, &mut b);
        let s = b.host.session.as_ref().unwrap();
        let (entries, ghost, reasons) = (s.plan.entries_for(&s.w, h), rip::ghost_entries(&s.w, &s.plan, h), rip::reasons(&s.w, &s.plan, h));
        assert!(ghost.iter().any(|g| !*g), "a picked entry");
        for ((e, gh), why) in entries.iter().zip(&ghost).zip(&reasons).filter(|((e, _), _)| e.step_index >= 0) {
            let block = widget(&b, &format!("strip.block{}", e.start_block));
            let first = block.tip.first().map(|t| t.0.clone()).unwrap_or_default();
            assert_eq!(first.starts_with("Auto-filled"), *gh, "block {}: {first}", e.start_block);
            if *gh && !why.is_empty() {
                assert!(tip_has(block, &format!("Auto-filled: {why}")));
            }
        }
    }

    /// ET1: an empty tile's menu opens with Move here.
    #[test]
    fn an_empty_tile_offers_move_here_first() {
        let (mut cx, _g, mut b) = planning();
        let free = free_tile(w(&b));
        b.run(&mut cx, Action::TileClick(free));
        let p = b.popup.as_ref().expect("the tile menu");
        assert_eq!((p.menu, p.rows[0].label.as_str()), (Some((hero(&b), RoundTarget::Cell(free))), "Move here"));
    }

    /// MR13: hovering a Maneuver Engage row marks its target's open position tiles (the kit's `positions`).
    #[test]
    fn hovering_an_engage_row_marks_the_position_tiles() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        b.run(&mut cx, Action::UnitClick(f));
        drawn(&mut cx, &mut g, &mut b);
        let p = b.popup.clone().expect("the skill menu");
        let k = p.rows.iter().position(|r| matches!(&r.act, RowAct::Pick(x) if matches!(x[..], [rip::Pick::Skill { kind: Some(ManeuverKind::Engage), skill }] if ri::position_of(w(&b), &w(&b).cat.skills[skill]).is_some()))).expect("a Maneuver Engage row with a position");
        let RowAct::Pick(picks) = p.rows[k].act.clone() else { unreachable!() };
        let want = engage_positions(b.host.session.as_ref().unwrap(), h, RoundTarget::Unit(f), &picks);
        let r = widget(&b, &format!("popup.{k}")).rect;
        b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
        assert_eq!(b.positions_shown, want);
        assert_eq!(b.state(&cx)["overlay"]["positions"].as_array().unwrap().len(), want.len());
        drawn(&mut cx, &mut g, &mut b);
        assert!(b.positions_shown.is_empty(), "only while hovered");
    }

    /// TEXTFIT1: the skill menu and the hero panel fit at the four text sizes (no spill), 1280x720.
    #[test]
    fn the_menu_and_the_hero_panel_fit_at_every_text_size() {
        let (mut cx, mut g, mut b) = planning();
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        let f = foes_on_field(&b)[0];
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        b.run(&mut cx, Action::UnitClick(f));
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let ours: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("popup.") || w.starts_with("hero."))).cloned().collect();
            assert!(ours.is_empty(), "scale {scale}: {ours:?}");
        }
    }

    // ---- INPUT1-U slice 3: selection and inspect (INPUT1 SL1-SL7, IS1-IS3) ----------------------------------------

    /// A right click over widget `id` (the mouse there, as the shell's `back`), after a frame so the widgets are known.
    fn right_click(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen, id: &str, unit: usize) {
        drawn(cx, g, b);
        let r = widget(b, id).rect;
        let at = (r.x + r.w / 2, r.y + r.h / 2);
        assert!(matches!(aegis_ui::ui::hit(&b.w, at), Some(Action::UnitClick(u) | Action::Select(u)) if u == unit), "{id}'s middle is unit {unit}: {:?} {:?}", b.popup.as_ref().map(|p| p.title.clone()), b.w.iter().filter(|w| w.rect.contains(at.0, at.1)).map(|w| (w.id.clone(), w.enabled, w.action.clone())).collect::<Vec<_>>());
        b.mouse = at;
        b.back(cx);
    }

    /// The texts drawn inside the intel panel at the last frame.
    fn intel_text(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) -> String {
        let recs = crate::hud::hud_tests::frame(b, cx, g, 1.0);
        let r = b.panels.iter().find(|p| p.0 == "intel").expect("the intel panel").1;
        recs.iter().filter(|t| r.contains(t.x as i32 + 1, t.y as i32 + 1)).map(|t| t.text.clone()).collect::<Vec<_>>().join("\n")
    }

    /// SL1, SL4-SL7: with a hero selected and planning a click on an enemy targets it and a click on the hero itself (its
    /// token or its card) selects nothing; with none selected a click on an enemy inspects it (no target, no menu) and
    /// one on an ally selects it; outside planning the same.
    #[test]
    fn clicks_select_target_and_inspect() {
        let (mut cx, _g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        b.run(&mut cx, Action::UnitClick(f));
        assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Unit(f)), "SL1: the click targets");
        b.run(&mut cx, Action::ClosePopup);
        let before = round_plan(&mut b);
        b.run(&mut cx, Action::UnitClick(h));
        assert_eq!((b.selected, b.popup.is_none()), (None, true), "SL4: the hero's token selects nothing");
        assert_eq!(round_plan(&mut b), before, "and plans nothing");
        let target = w(&b).round_states[&h].target;
        b.run(&mut cx, Action::UnitClick(f));
        assert_eq!((b.inspected(), b.selected, b.popup.is_none()), (Some(f), None, true), "SL5: nothing selected: the enemy is inspected");
        assert_eq!(w(&b).round_states[&h].target, target, "no target set");
        b.run(&mut cx, Action::UnitClick(h));
        assert_eq!(b.selected, Some(h), "SL5: an ally is selected");
        b.run(&mut cx, Action::Select(h));
        assert_eq!(b.selected, None, "SL4: the selected hero's card selects nothing");
        b.run(&mut cx, Action::Select(h));
        assert_eq!(b.selected, Some(h), "SL6: a card selects");
        let state = b.state(&cx);
        assert_eq!(state["inspect"], json!(f), "the agent link's inspect");
        // SL7: outside planning
        b.run(&mut cx, Action::Confirm);
        assert!(!b.planning(), "{}", b.message);
        b.inspect = None;
        b.run(&mut cx, Action::UnitClick(f));
        assert_eq!((b.inspected(), b.selected), (Some(f), Some(h)), "SL7: playback: the enemy is inspected, the hero stays selected");
        let other = w(&b).allies.iter().copied().find(|&a| a != h);
        if let Some(a) = other {
            b.run(&mut cx, Action::UnitClick(a));
            assert_eq!(b.selected, Some(a), "SL7: playback: an ally is selected");
        }
    }

    /// SL2, IS1, IF10: a right click on any unit (token or card) inspects it and changes neither the selection nor the
    /// plan, also on the selected hero; with the menu open it only closes the menu; elsewhere it undoes the last pick.
    #[test]
    fn a_right_click_on_a_unit_inspects_it() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        let before = (round_plan(&mut b), b.selected);
        right_click(&mut cx, &mut g, &mut b, &format!("party.{h}"), h);
        assert_eq!(b.inspected(), Some(h), "{}", b.message);
        assert_eq!((round_plan(&mut b), b.selected), before, "the selected hero inspected: selection and plan unchanged");
        right_click(&mut cx, &mut g, &mut b, &format!("unit.{f}"), f);
        assert_eq!(b.inspected(), Some(f), "the enemy's token");
        assert_eq!((round_plan(&mut b), b.selected), before, "no target, no select");
        right_click(&mut cx, &mut g, &mut b, &format!("foe.{f}"), f);
        assert_eq!(b.inspected(), Some(f), "the enemy's card");
        // the menu open: the right click only closes it (IF10)
        next_to(&cx, &mut b, f);
        b.run(&mut cx, Action::UnitClick(f));
        assert!(b.popup.is_some());
        b.inspect = None;
        drawn(&mut cx, &mut g, &mut b);
        let r = widget(&b, &format!("party.{h}")).rect;
        b.mouse = (r.x + r.w / 2, r.y + r.h / 2);
        b.back(&mut cx);
        assert_eq!((b.popup.is_none(), b.inspected()), (true, None), "the menu closed, nothing inspected");
        // elsewhere: undo the last pick
        b.run(&mut cx, Action::UnitClick(f));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        b.run(&mut cx, Action::ClosePopup);
        assert_eq!(rip::picks_of(w(&b), h).len(), 1, "{}", b.message);
        drawn(&mut cx, &mut g, &mut b);
        b.mouse = (-500, -500);
        b.back(&mut cx);
        assert!(rip::picks_of(w(&b), h).is_empty(), "a right click off every unit undoes: {}", b.message);
    }

    /// SL3: Esc closes the menu (only), then clears the inspected unit, then selects nothing, then leaves the Arena.
    #[test]
    fn escape_steps_back_one_layer_at_a_time() {
        let (mut cx, mut g, mut b) = planning();
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        right_click(&mut cx, &mut g, &mut b, &format!("foe.{f}"), f);
        b.run(&mut cx, Action::UnitClick(f));
        assert!(b.popup.is_some());
        b.escape(&mut cx);
        assert_eq!((b.popup.is_none(), b.inspected(), b.selected), (true, Some(f), Some(h)), "the menu only");
        b.escape(&mut cx);
        assert_eq!((b.inspected(), b.selected), (None, Some(h)), "then the inspected unit");
        assert!(cx.nav.is_empty());
        b.escape(&mut cx);
        assert_eq!(b.selected, None, "then the selection");
        assert!(cx.nav.is_empty(), "nothing left yet");
        b.escape(&mut cx);
        assert!(!cx.nav.is_empty(), "then the Arena leaves, as before");
    }

    /// IS1, IS2: the inspected enemy pins the intel panel ("Inspecting <name> (Esc clears)") over the hovered unit until
    /// Esc or the round's end; an unrevealed enemy's panel names none of its plan's skills but the tell's first action,
    /// and that one only while the tells show.
    #[test]
    fn the_inspected_enemy_shows_only_what_the_party_knows() {
        let (mut cx, mut g, mut b) = planning();
        let f = foes_on_field(&b)[0];
        right_click(&mut cx, &mut g, &mut b, &format!("foe.{f}"), f);
        let s = b.host.session.as_ref().unwrap();
        let name = s.w.units[f].name.clone();
        let tell = s.enemy_intents().into_iter().find(|r| r.unit == f).expect("the enemy's tell");
        assert!(!tell.revealed, "the arena fight reveals no plan");
        let first = intent_text::first_line(&s.w, &tell.line);
        let skills: Vec<String> = s.resolver.scheduled.iter().filter(|a| a.unit == f).map(|a| a.command.name.clone()).filter(|n| !first.contains(n.as_str())).collect();
        // another unit under the mouse: the panel stays pinned
        let other = foes_on_field(&b).into_iter().find(|&e| e != f).unwrap_or(hero(&b));
        let id = if w(&b).allies.contains(&other) { format!("party.{other}") } else { format!("foe.{other}") };
        drawn(&mut cx, &mut g, &mut b);
        let r = widget(&b, &id).rect;
        b.draw(&mut cx, &mut g, (r.x + 2, r.y + 2), true);
        assert_eq!(widget(&b, "intel.unit").text, format!("Inspecting {name} (Esc clears)"), "pinned over the hover");
        let shown = intel_text(&mut cx, &mut g, &mut b);
        for n in &skills {
            assert!(!shown.contains(n.as_str()), "the panel names {n} of an unrevealed plan:\n{shown}");
        }
        assert!(shown.contains("Intent: "), "the tell's first action while the tells show:\n{shown}");
        cx.settings.v["show_tells"] = json!(false);
        let hidden = intel_text(&mut cx, &mut g, &mut b);
        assert!(!hidden.contains("Intent: "), "tells hidden: no first action:\n{hidden}");
        for n in s_names(&b, f) {
            assert!(!hidden.contains(n.as_str()), "tells hidden: no plan skill {n}:\n{hidden}");
        }
        cx.settings.v["show_tells"] = json!(true);
        // the round's end clears it
        b.send(&cx, json!({"cmd": "resolve"}));
        assert_eq!(b.inspected(), Some(f), "kept through the playback");
        b.run(&mut cx, Action::NextRound);
        assert_eq!(b.inspected(), None, "the next round starts unpinned: {}", b.message);
    }

    /// The names of `f`'s planned commands this round.
    fn s_names(b: &BattleScreen, f: usize) -> Vec<String> {
        b.host.session.as_ref().unwrap().resolver.scheduled.iter().filter(|a| a.unit == f).map(|a| a.command.name.clone()).collect()
    }

    /// IS3: an inspected party member shows its intent, its picks and plan (one line an entry, ghosts marked) and its AP
    /// numbers (AP9's lines from the kit's `ap_forecast`); the selected hero stays selected.
    #[test]
    fn an_inspected_member_shows_its_plan_and_ap() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        let m = w(&b).allies.iter().copied().find(|&a| a != h && w(&b).units[a].is_active()).unwrap_or(h);
        right_click(&mut cx, &mut g, &mut b, &format!("party.{m}"), m);
        assert_eq!((b.inspected(), b.selected), (Some(m), Some(h)), "IS3: the selection stays");
        drawn(&mut cx, &mut g, &mut b);
        let s = b.host.session.as_ref().unwrap();
        let lines: Vec<String> = inspect_tip(s, m, b.selected, &s.enemy_intents(), &s.round_intents(), &[], true).into_iter().map(|l| l.0).collect();
        for l in ap_lines(&rip::ap_forecast(&s.w, &s.plan, m)) {
            assert!(lines.contains(&l), "AP9 line {l:?} in {lines:?}");
        }
        let entries = s.plan.entries_for(&s.w, m);
        let ghosts = rip::ghost_entries(&s.w, &s.plan, m);
        assert!(lines.iter().any(|l| l.starts_with("Picks: ")), "{lines:?}");
        for (e, gh) in entries.iter().zip(&ghosts) {
            let name = &s.w.cat.skills[e.skill].name;
            assert!(lines.iter().any(|l| l.starts_with(&intent_text::block_text((e.start_block, e.end_block(&s.w)))) && l.contains(name.as_str()) && l.ends_with(", auto-filled") == *gh), "entry {name} at B{}: {lines:?}", e.start_block);
        }
        let rec = s.round_intents().into_iter().find(|r| r.unit == m);
        if let Some(head) = rec.as_ref().and_then(|r| intent_text::round_head(&s.w, r)) {
            assert!(lines.contains(&head), "its intent and target: {head}");
        }
        assert_eq!(widget(&b, "intel.unit").text, format!("Inspecting {} (Esc clears)", w(&b).units[m].name));
    }

    /// TEXTFIT1: the inspect panel's texts (an enemy's, a party member's) stay inside the panel at the four text sizes,
    /// 1280x720.
    #[test]
    fn the_inspect_panel_fits_at_every_text_size() {
        let (mut cx, mut g, mut b) = planning();
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        let (h, f) = (hero(&b), foes_on_field(&b)[0]);
        for u in [f, h] {
            b.inspect = Some((u, b.session().current_round));
            for scale in [0.85f32, 1.0, 1.15, 1.3] {
                let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
                let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                let r = b.panels.iter().find(|p| p.0 == "intel").expect("the intel panel").1;
                let inside = |e: &Value| r.contains(e["at"][0].as_f64().unwrap_or(-1.0) as i32 + 1, e["at"][1].as_f64().unwrap_or(-1.0) as i32 + 1);
                let ours: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("intel."))).chain(v["cut"].as_array().unwrap().iter().filter(|e| inside(e))).cloned().collect();
                assert!(ours.is_empty(), "unit {u} scale {scale}: {ours:?}");
            }
        }
    }

    // ---- INPUT1-AP: the AP pip gauge (INPUT1 AP3, AP5-AP10) ---------------------------------------------------------

    /// The agent link's `ap_gauge` (null: hidden).
    fn gauge(b: &BattleScreen, cx: &Cx) -> Value {
        b.state(cx)["ap_gauge"].clone()
    }

    /// AP6, AP7 written out from the kit's forecast: `u = ceil(max / 12)` AP a pip above 12, every count divided by it
    /// and rounded up; pip `i` full when `i <= left`, a ghost fill when `left < i <= next`, outlined when `i <= now`; a
    /// debt (`now < 0`): every pip an empty slot.
    fn pips_by_rule(f: &rip::ApForecast) -> String {
        let u = if f.max > AP_PIPS { (f.max + AP_PIPS - 1) / AP_PIPS } else { 1 };
        let c = |n: i32| if n <= 0 { 0 } else { (n + u - 1) / u };
        (1..=c(f.max))
            .map(|i| match (f.now >= 0, i <= c(f.left), c(f.left) < i && i <= c(f.next), i <= c(f.now)) {
                (false, ..) => "S",
                (true, true, _, _) => "F",
                (true, false, true, true) => "Hg",
                (true, false, false, true) => "H",
                (true, false, true, false) => "Sg",
                _ => "S",
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn forecast(b: &BattleScreen, u: usize) -> rip::ApForecast {
        let s = b.host.session.as_ref().unwrap();
        rip::ap_forecast(&s.w, &s.plan, u)
    }

    fn set_ap(cx: &Cx, b: &mut BattleScreen, u: usize, ap: i32, max: Option<i32>) {
        let unit = &mut b.host.session.as_mut().unwrap().w.units[u];
        unit.ap = ap;
        if let Some(m) = max {
            unit.max_ap = m;
        }
        b.after_change(cx);
    }

    /// AP5-AP7, AP9: while planning the selected party member's gauge (`strip.ap`) reads its kit forecast by AP7's rule,
    /// its hover card holds AP9's lines; with no hero selected it is hidden.
    #[test]
    fn the_gauge_reads_the_kit_forecast_while_planning() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        let f = forecast(&b, h);
        let st = gauge(&b, &cx);
        assert_eq!(st["pips"], json!(pips_by_rule(&f)), "{f:?}");
        assert_eq!((st["unit"].clone(), st["red"].clone(), st["label"].clone()), (json!(h), json!(false), Value::Null));
        let wg = widget(&b, "strip.ap");
        assert_eq!(wg.text, pips_by_rule(&f));
        for l in ap_lines(&f) {
            assert!(tip_has(wg, &l), "AP9 line {l} in {:?}", wg.tip);
        }
        // a pick changes the gauge at once (AP2)
        let foe = foes_on_field(&b)[0];
        next_to(&cx, &mut b, foe);
        b.run(&mut cx, Action::UnitClick(foe));
        let k = first_valid_pick(&b);
        b.run(&mut cx, Action::Choose(k));
        b.run(&mut cx, Action::ClosePopup);
        drawn(&mut cx, &mut g, &mut b);
        assert_eq!(gauge(&b, &cx)["pips"], json!(pips_by_rule(&forecast(&b, h))));
        b.selected = None;
        drawn(&mut cx, &mut g, &mut b);
        assert_eq!(gauge(&b, &cx), Value::Null, "no hero selected: hidden");
        assert!(b.w.iter().all(|x| x.id != "strip.ap"));
    }

    /// AP5: a classic battle draws no gauge and its state has no `ap_gauge` key.
    #[test]
    fn classic_draws_no_gauge() {
        let (mut cx, mut g, mut b) = arena(None, "", false);
        b.tick(&mut cx);
        drawn(&mut cx, &mut g, &mut b);
        assert!(b.w.iter().all(|x| x.id != "strip.ap"));
        assert!(b.state(&cx).get("ap_gauge").is_none());
    }

    /// The hero next to a foe with the skill menu open on it and a skill row it can pay at full AP; the row's index
    /// and AP (the kit option's).
    fn payable_row(cx: &mut Cx, b: &mut BattleScreen) -> (usize, usize, i32) {
        let (h, f) = (hero(b), foes_on_field(b)[0]);
        next_to(cx, b, f);
        b.run(cx, Action::UnitClick(f));
        let s = b.host.session.as_ref().unwrap();
        let opts = rip::pick_options(&s.w, &s.plan, h, RoundTarget::Unit(f));
        let k = opts.iter().position(|o| o.problem.is_none() && o.ap >= 1 && matches!(o.picks[..], [rip::Pick::Skill { .. }])).expect("a payable skill row with an AP cost");
        (f, k, opts[k].ap)
    }

    /// AP3, AP8: hovering a row the hero cannot pay (its AP one short of the row's) turns the gauge red with "-1" after
    /// the pips; leaving it restores the gauge.
    #[test]
    fn an_unpayable_row_turns_the_gauge_red_with_its_shortfall() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        let (f, k, cost) = payable_row(&mut cx, &mut b);
        b.run(&mut cx, Action::ClosePopup);
        set_ap(&cx, &mut b, h, cost - 1, None);
        b.run(&mut cx, Action::UnitClick(f));
        let row = b.popup.as_ref().expect("the menu").rows[k].clone();
        assert!(!row.valid && row.detail.starts_with("AP "), "the row is refused for its AP: {}", row.detail);
        assert_eq!(row.ap_short, Some(1), "the kit's shortfall: the cost less the AP free");
        drawn(&mut cx, &mut g, &mut b);
        let r = widget(&b, &format!("popup.{k}")).rect;
        b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
        let st = gauge(&b, &cx);
        assert_eq!((st["red"].clone(), st["label"].clone(), st["label_side"].clone()), (json!(true), json!("-1"), json!("after")), "{st}");
        assert_eq!(st["pips"], json!(pips_by_rule(&forecast(&b, h))), "the alert changes colours only");
        drawn(&mut cx, &mut g, &mut b);
        let st = gauge(&b, &cx);
        assert_eq!((st["red"].clone(), st["label"].clone()), (json!(false), Value::Null), "leaving the row clears both");
    }

    /// AP8: a hero at -2 AP shows "-2" before its slots, every pip an empty slot.
    #[test]
    fn a_debt_shows_before_the_slots() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        set_ap(&cx, &mut b, h, -2, None);
        drawn(&mut cx, &mut g, &mut b);
        let st = gauge(&b, &cx);
        let slots = st["slots"].as_i64().unwrap() as usize;
        assert_eq!((st["label"].clone(), st["label_side"].clone()), (json!("-2"), json!("before")));
        assert_eq!(st["pips"], json!(vec!["S"; slots].join(" ")));
        assert_eq!(st["pips"], json!(pips_by_rule(&forecast(&b, h))));
    }

    /// AP6, AP9, IS3: a 14 AP hero shows 7 pips at 2 AP each; the hover card and the inspect panel say "1 pip = 2 AP".
    #[test]
    fn above_twelve_ap_a_pip_holds_two() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        set_ap(&cx, &mut b, h, 14, Some(14));
        drawn(&mut cx, &mut g, &mut b);
        let st = gauge(&b, &cx);
        assert_eq!((st["slots"].clone(), st["per_pip"].clone()), (json!(7), json!(2)));
        assert_eq!(st["pips"], json!(pips_by_rule(&forecast(&b, h))));
        assert!(tip_has(widget(&b, "strip.ap"), "1 pip = 2 AP"));
        let s = b.host.session.as_ref().unwrap();
        let lines: Vec<String> = inspect_tip(s, h, b.selected, &s.enemy_intents(), &s.round_intents(), &[], true).into_iter().map(|l| l.0).collect();
        assert!(lines.contains(&"1 pip = 2 AP".to_string()), "IS3: {lines:?}");
        set_ap(&cx, &mut b, h, 10, Some(10));
        assert!(!ap_lines(&forecast(&b, h)).iter().any(|l| l.starts_with("1 pip")), "at most 12 AP: no pip line");
    }

    /// AP10: in playback the gauge shows the live AP full and what the hero paid this round hollow (one pip an AP); a
    /// payment turns its pips hollow over 150 ms (the frame it shows starts the change), at once with skip animations;
    /// the next planning reads the forecast again.
    #[test]
    fn playback_drains_the_paid_pips() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        let start = w(&b).units[h].ap;
        b.send(&cx, json!({"cmd": "resolve"}));
        assert_eq!(b.session().phase, Phase::Replay, "{}", b.message);
        drawn(&mut cx, &mut g, &mut b);
        let live = w(&b).units[h].ap;
        let max = w(&b).units[h].max_ap;
        let paid: i32 = b.session().resolver.scheduled.iter().filter(|a| a.unit == h && !a.cancelled).map(|a| a.command.ap_cost + a.command.stance_switch_cost).sum();
        let st = gauge(&b, &cx);
        assert_eq!(st["pips"], json!(aegis_ui::pips::PipGauge::live(start, live, max, AP_PIPS).reading()));
        let read = st["pips"].as_str().unwrap().to_string();
        let count = |code: &str| read.split(' ').filter(|p| *p == code).count() as i32;
        if max <= AP_PIPS && live >= 0 && live <= start {
            assert_eq!((count("F"), count("H")), (live, start - live), "full to the live AP, hollow what was paid: {read}");
            if paid > 0 && live == start - paid {
                assert_eq!(count("H"), paid, "a command paying {paid} AP turns {paid} pips hollow");
            }
        }
        // a payment shown mid-playback: the frame that sees it starts the change, 150 ms later it is done
        let ap = w(&b).units[h].ap;
        let (pay_from, pay_to) = if ap >= 2 { (ap, ap - 2) } else { (ap, ap) };
        set_ap(&cx, &mut b, h, pay_to, None);
        drawn(&mut cx, &mut g, &mut b);
        if pay_from != pay_to {
            assert_eq!(gauge(&b, &cx)["pips"], json!(read), "the change starts from the shown pips");
            let a = b.ap_anim.as_mut().unwrap();
            a.at = a.at.checked_sub(Duration::from_millis(200)).unwrap_or(a.at);
            drawn(&mut cx, &mut g, &mut b);
        }
        let done = aegis_ui::pips::PipGauge::live(start, pay_to, max, AP_PIPS).reading();
        assert_eq!(gauge(&b, &cx)["pips"], json!(done), "after 150 ms");
        // skip animations: at once
        cx.settings.v["skip_anim"] = json!(true);
        set_ap(&cx, &mut b, h, pay_to.max(1) - 1, None);
        drawn(&mut cx, &mut g, &mut b);
        assert_eq!(gauge(&b, &cx)["pips"], json!(aegis_ui::pips::PipGauge::live(start, pay_to.max(1) - 1, max, AP_PIPS).reading()), "skip animations: at once");
        // the next planning: the forecast again
        b.run(&mut cx, Action::NextRound);
        b.tick(&mut cx);
        drawn(&mut cx, &mut g, &mut b);
        if b.planning() && w(&b).units[h].is_active() {
            assert_eq!(gauge(&b, &cx)["pips"], json!(pips_by_rule(&forecast(&b, h))));
        }
    }

    /// A frame at text scale `scale` with the mouse at `mouse` (the text records kept).
    fn frame_at(b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, scale: f32, mouse: (i32, i32)) -> Vec<aegis_ui::painter::TextRec> {
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        b.draw(cx, g, mouse, true);
        g.fonts.audit.take().unwrap_or_default()
    }

    /// AP6, TEXTFIT1: at 640x360 and the four text sizes the gauge sits inside the strip, after the last block and
    /// before the tactic menus, with its label (the shortfall's and the debt's), and no strip text spills or is cut
    /// (the blocks' span labels cut by design).
    #[test]
    fn the_strip_with_the_gauge_fits_at_every_text_size() {
        let (mut cx, mut g, mut b) = planning();
        let h = hero(&b);
        let (f, k, cost) = payable_row(&mut cx, &mut b);
        b.run(&mut cx, Action::ClosePopup);
        let full = w(&b).units[h].ap;
        for (state, ap) in [("plain", full), ("red", cost - 1), ("debt", -2)] {
            set_ap(&cx, &mut b, h, ap, None);
            b.popup = None;
            if state == "red" {
                b.run(&mut cx, Action::UnitClick(f));
            }
            for (_, scale) in aegis_ui::screen::TEXT_SIZES {
                let scale = scale as f32;
                frame_at(&mut b, &mut cx, &mut g, scale, (-500, -500));
                let mouse = match (state, b.w.iter().find(|x| x.id == format!("popup.{k}"))) {
                    ("red", Some(r)) => (r.rect.x + r.rect.w / 2, r.rect.y + r.rect.h / 2),
                    _ => (-500, -500),
                };
                let recs = frame_at(&mut b, &mut cx, &mut g, scale, mouse);
                let st = gauge(&b, &cx);
                assert_eq!(st["label"].is_null(), state == "plain", "{state} scale {scale}: {st}");
                let ga = widget(&b, "strip.ap").rect;
                let strip = Rect::new(2, 2, W - 4, 24);
                assert!(ga.x >= strip.x && ga.y >= strip.y && ga.x + ga.w <= strip.x + strip.w && ga.y + ga.h <= strip.y + strip.h, "{state} scale {scale}: the gauge {ga:?} in the strip");
                let last = (1..=b.session().def.total_blocks).filter_map(|n| b.w.iter().find(|x| x.id == format!("strip.block{n}"))).map(|x| x.rect.x + x.rect.w).max().unwrap();
                let party = widget(&b, "strip.party").rect;
                assert!(ga.x >= last && ga.x + ga.w <= party.x, "{state} scale {scale}: blocks end {last}, gauge {ga:?}, party menu {party:?}");
                let confirm = widget(&b, "strip.confirm").rect;
                assert!(confirm.x + confirm.w <= W - 4, "{state} scale {scale}: the controls stay on the strip");
                let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                let widget_at = |x: f32, y: f32| boxes.iter().filter(|b| x >= b.1 && y >= b.2 && x < b.1 + b.3 && y < b.2 + b.4).min_by(|a, b| (a.3 * a.4).total_cmp(&(b.3 * b.4))).map(|b| b.0.clone());
                let on_strip = |e: &Value| {
                    let (x, y) = (e["at"][0].as_f64().unwrap_or(-1.0) as f32 + 0.5, e["at"][1].as_f64().unwrap_or(-1.0) as f32 + 0.5);
                    y < 26.0 && widget_at(x, y).is_none_or(|id| !id.starts_with("strip.block"))
                };
                // the open menu's own texts (it may open over the strip) are the menu's audit
                let menu = b.popup.as_ref().map(|p| p.rect(b.menu_row));
                let off_menu = |e: &Value| menu.is_none_or(|m| !m.contains(e["at"][0].as_f64().unwrap_or(-1.0) as i32 + 1, e["at"][1].as_f64().unwrap_or(-1.0) as i32 + 1));
                let ours: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("strip."))).chain(v["cut"].as_array().unwrap().iter().filter(|e| on_strip(e))).filter(|e| off_menu(e)).cloned().collect();
                assert!(ours.is_empty(), "{state} scale {scale}: {ours:?}");
            }
        }
    }

    /// User 2026-10-29: an Arena fight beyond classic shows the Scene Runner card's ruleset chip (same text: the
    /// ruleset's name, its switches, the reveal); a classic fight draws neither the chip nor any intent control and its
    /// state keeps its shape.
    #[test]
    fn the_arena_chip_shows_beyond_classic_and_classic_is_unchanged() {
        let timeline = Ruleset::of(&crate::hud::hud_tests::game_cx().data.cat, TIMELINE).expect("the catalog's timeline row");
        let off = timeline.rules.iter().copied().find(|id| rule(id).is_some_and(|r| r.kind == RuleKind::Extension) && *id != PLAN_BADGES).expect("an extension besides plan_badges");
        let spec = format!("-{off}");
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), &spec, true);
        drawn(&mut cx, &mut g, &mut b);
        let s = b.host.session.as_ref().unwrap();
        let want = crate::runner::chip_text(&timeline.name, &spec, "", s.w.ruleset.has(PLAN_BADGES));
        assert_eq!(widget(&b, "bar.chip").text, want);
        assert_eq!(b.state(&cx)["ruleset"]["chip"], json!(want));
        assert!(want.contains(&format!("-{off}")));
        let (mut cx, mut g, mut b) = arena(None, "", false);
        drawn(&mut cx, &mut g, &mut b);
        assert!(w(&b).ruleset.is_classic());
        assert!(b.w.iter().all(|x| !["hero.self", "bar.chip"].contains(&x.id.as_str())), "classic draws no round control");
        let st = b.state(&cx);
        assert!(st.get("picks").is_none() && st.get("ruleset").is_none(), "a classic battle's state keeps its shape");
    }
}

/// RULESET1-U slice 3 (RULESET1_INTENT UI3, RULESET1 K6, GD5, MV1): the board overlays under the round intent. A catalog
/// encounter played by the catalog's timeline row; every expectation is a kit query (`target_problem`, `attack_band`,
/// `in_front`, `facing`, `hidden`, the compiled plan), never a shipped number. Classic draws none (no state key).
#[cfg(test)]
mod overlay_tests {
    use super::*;
    use emberwake_rules::ruleset::TIMELINE;

    fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn overlay(b: &BattleScreen) -> Overlay {
        overlay_of(s(b), b.selected, true).expect("the round intent's overlays")
    }

    /// The hero and its first foe on the field, the hero set to Assault on that foe through the command API.
    fn assault(cx: &mut Cx, b: &mut BattleScreen) -> (usize, usize) {
        let h = b.selected.expect("a hero");
        let foe = s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).expect("a foe");
        let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
        assert_eq!(r["ok"], json!(true), "{r}");
        (h, foe)
    }

    /// Classic: no overlay, no state key, the hover card's Reach is the unit's attack range as before.
    #[test]
    fn classic_draws_no_overlay() {
        let (mut cx, mut g, mut b) = arena(None);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(overlay_of(s(&b), b.selected, true).is_none());
        assert!(b.state(&cx).get("overlay").is_none(), "a classic battle's state keeps its shape");
        let w = &s(&b).w;
        for i in 0..w.units.len() {
            assert_eq!(reach_text(w, i), w.units[i].attack_range.to_string());
        }
    }

    /// UI1 (valid cells lit), UI3 (the target ring, move paths), K6 (the reach band), MV1 (enemy facings): under Assault
    /// on a foe the ring sits on the foe; the lit units are exactly those the intent table takes (the target left out);
    /// the band is every tile at the plain attack band's distance from the tile the first attack strikes from; the moves
    /// follow the compiled plan tile by tile from the hero; every enemy with a facing shows it. The state lists the same.
    #[test]
    fn the_overlays_follow_the_kit() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        // the slice 3 review: the hero starts 3 or 4 tiles from its foe, so the plan walks 2 tiles or more
        {
            let h = b.selected.unwrap();
            let w = &s(&b).w;
            let foe = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).unwrap();
            let far = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).find(|c| w.grid.is_valid(*c) && w.occupant(*c).is_none() && (3..=4).contains(&c.dist(w.pos(foe))) && c.y == w.pos(foe).y).expect("a free tile 3-4 tiles from the foe on its row");
            b.host.session.as_mut().unwrap().w.grid.positions[h] = Some(far);
        }
        let (h, foe) = assault(&mut cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let o = overlay(&b);
        let w = &s(&b).w;
        assert!(o.moves.iter().any(|(t, k)| *k == MoveKind::Walk && t.len() >= 3), "a walk of 2+ tiles: {:?}", o.moves);
        assert_eq!(o.target, Some(w.pos(foe)), "the ring on the round target");
        let valid: Vec<V2> = (0..w.units.len())
            .filter(|&u| u != foe && w.units[u].is_active() && w.has(u) && ri::target_problem(w, h, RoundIntent::Assault, ri::normalize(h, RoundTarget::Unit(u))).is_empty())
            .map(|u| w.pos(u))
            .collect();
        assert_eq!(o.valid, valid);
        assert!(o.valid.iter().all(|c| w.occupant(*c).is_some_and(|u| w.foes_of(h).contains(&u))), "Assault lights foes only");
        let st = s(&b).plan.compile(w, h);
        let at = st.entries.iter().find(|e| emberwake_battle::skill::kind(&w.cat.skills[e.skill]) == Kind::Attack).map(|e| e.pos_after).unwrap_or_else(|| w.pos(h));
        let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[h], &w.ruleset);
        let band: Vec<V2> = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && c.dist(at) >= lo && c.dist(at) <= hi).collect();
        assert_eq!(o.band, band, "the band around {at:?}");
        let mut from = w.pos(h);
        let mut ends = Vec::new();
        for e in st.entries.iter().filter(|e| e.pos_after.x >= 0) {
            if e.pos_after != from {
                ends.push((from, e.pos_after));
                from = e.pos_after;
            }
        }
        assert_eq!(o.moves.iter().map(|(t, _)| (t[0], *t.last().unwrap())).collect::<Vec<_>>(), ends, "one drawn move per change of tile");
        assert!(o.moves.iter().all(|(t, _)| t.windows(2).all(|p| p[0] != p[1])));
        // the drawn tiles walk the plan's steps: every one on the board, each a step from the last (a carried move's
        // dash may cover more than one)
        assert!(o.moves.iter().flat_map(|(t, _)| t.iter()).all(|c| w.grid.is_valid(*c)), "{:?}", o.moves);
        assert!(o.moves.iter().filter(|(_, k)| *k == MoveKind::Walk).all(|(t, _)| t.windows(2).all(|p| p[0].dist(p[1]) == 1)), "{:?}", o.moves);
        assert!(o.front.is_empty(), "no front shaded under Assault");
        let facings: Vec<(V2, V2)> = w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.has(e) && ri::facing(w, e) != emberwake_battle::grid::ZERO).map(|e| (w.pos(e), ri::facing(w, e))).collect();
        assert_eq!(o.facings, facings);
        // (VISCLAR1 AR3: with the target arrows the last frame drew)
        let drawn = Overlay { arrows: b.arrows_shown.clone(), ..o.clone() };
        // (INPUT1 PB1: with the impact tracers drawing now: none while planning)
        let mut want = overlay_json(&drawn);
        want["tracers"] = json!([]);
        assert_eq!(b.state(&cx)["overlay"], want, "the agent link's state lists the overlays");
        // the slice 3 review: an attack strikes from after its carried Engage move, from before its Disengage retreat
        let mut e = st.entries[0].clone();
        (e.pos_before, e.pos_after) = (V2::new(1, 1), V2::new(2, 1));
        for (carry, want) in [(emberwake_battle::command::Carry::Retreat(V2::new(2, 1)), e.pos_before), (emberwake_battle::command::Carry::Engage(V2::new(2, 1)), e.pos_after), (emberwake_battle::command::Carry::None, e.pos_after)] {
            e.carry = carry;
            assert_eq!(strike_tile(&e), want, "{carry:?}");
        }
        // playback: no hero overlays, the facings stay
        let r = b.send(&cx, json!({"cmd": "confirm"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let o = overlay_of(s(&b), b.selected, false).unwrap();
        assert!(o.target.is_none() && o.valid.is_empty() && o.band.is_empty() && o.moves.is_empty());
    }

    /// UI3, MV3: under Maneuver the target's front within FRONT_RADIUS is shaded, exactly the tiles `in_front` names
    /// (dot(tile - target, facing) > 0); no front without a facing.
    #[test]
    fn maneuver_shades_the_targets_front() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        let (h, foe) = assault(&mut cx, &mut b);
        // Manual: the intent stands even when the planner offers no Maneuver round here
        for req in [json!({"cmd": "manual", "unit": h, "on": true}), json!({"cmd": "round_intent", "unit": h, "intent": "maneuver", "target": foe})] {
            let r = b.send(&cx, req);
            assert_eq!(r["ok"], json!(true), "{r}");
        }
        for f in [V2::new(-1, 0), V2::new(0, 1)] {
            b.host.session.as_mut().unwrap().w.round_states.entry(foe).or_default().facing = f;
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let o = overlay(&b);
            let w = &s(&b).w;
            let tp = w.pos(foe);
            assert!(!o.front.is_empty(), "facing {f:?}");
            for c in (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && *c != tp && c.dist(tp) <= FRONT_RADIUS) {
                let d = c - tp;
                assert_eq!(o.front.contains(&c), d.x * f.x + d.y * f.y > 0, "{c:?} facing {f:?}");
                assert_eq!(o.front.contains(&c), ri::in_front(w, foe, c));
            }
        }
        b.host.session.as_mut().unwrap().w.round_states.entry(foe).or_default().facing = emberwake_battle::grid::ZERO;
        assert!(overlay(&b).front.is_empty(), "no facing, no front");
    }

    /// GD5: a stealthed unit wears a veil, hidden once it moved this round (the hover card says so); K6: the hover card's
    /// Reach is the kit's plain attack band under the rule `reach`.
    #[test]
    fn a_stealthed_unit_wears_a_veil_and_reach_reads_the_band() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        let h = b.selected.unwrap();
        let sess = b.host.session.as_mut().unwrap();
        emberwake_rules::status::grant_now(&mut sess.w.units[h], "stealth", 2, 0, "test");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let at = s(&b).w.pos(h);
        assert_eq!(overlay(&b).veils, vec![(at, false)], "veiled, not hidden before it moves");
        b.host.session.as_mut().unwrap().w.round_states.entry(h).or_default().moved = true;
        assert_eq!(overlay(&b).veils, vec![(at, true)], "hidden once it moved (GD5)");
        assert!(unit_tip(s(&b), h, Some(h), &[], &[], &[]).iter().any(|t| t.0.starts_with("Hidden")));
        let w = &s(&b).w;
        for i in 0..w.units.len() {
            let want = if w.ruleset.has(emberwake_rules::ruleset::REACH) {
                let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[i], &w.ruleset);
                if lo == hi { lo.to_string() } else { format!("{lo}-{hi}") }
            } else {
                w.units[i].attack_range.to_string()
            };
            assert_eq!(reach_text(w, i), want, "{}", w.units[i].name);
        }
    }
}

/// RULESET1-U slice 4 (RULESET1_INTENT UI5, RULESET1 V1, V2): the round intent in tells and threat intel, the shared
/// glyph's place, the impact badges and the full reveal on the battle screen. A catalog encounter played by the
/// catalog's timeline row; expectations come from the kit's records (`Session::enemy_intents`, `round_intents`) and the
/// text helpers, never from shipped numbers. Classic keeps its cards, intel and widgets.
#[cfg(test)]
mod tells_tests {
    use super::*;
    use emberwake_rules::ruleset::{PLAN_BADGES, TIMELINE};

    fn arena(ruleset: Option<&str>, reveal: bool) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", reveal);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn widget<'a>(b: &'a BattleScreen, id: &str) -> &'a Widget<Action> {
        b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}"))
    }

    fn passive_name(p: &str) -> String {
        p.to_string()
    }

    /// UI5: each hostile card leads with the enemy's round intent and target ("Assault: Wolf; Cut B1"); a party member
    /// with an intent gets the glyph on the strip (the title gives way); V1: each card has its badge row, one tip line
    /// per impact badge at its block.
    #[test]
    fn the_cards_lead_with_the_intent_and_carry_the_badges() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        assert!(s(&b).w.ruleset.has(PLAN_BADGES), "the timeline row shows badges");
        let h = b.selected.unwrap();
        let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).unwrap();
        let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
        assert_eq!(r["ok"], json!(true), "{r}");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        let tells = s(&b).enemy_intents();
        assert!(!tells.is_empty());
        let mut led = 0;
        for t in &tells {
            let card = widget(&b, &format!("foe.{}", t.unit));
            let line = intent_text::round_card_line(w, t);
            assert!(card.text.ends_with(&line), "{} vs {line}", card.text);
            if let Some(i) = t.round_intent {
                assert!(line.starts_with(i.name()), "{line}");
                led += 1;
            }
            let row = widget(&b, &format!("foe.{}.badges", t.unit));
            let marks: Vec<String> = row.tip.iter().skip(1).map(|x| x.0.clone()).collect();
            if t.impacts.is_empty() {
                assert_eq!(marks, ["Nothing lands this round"]);
            } else {
                assert_eq!(marks.len(), t.impacts.len(), "{marks:?}");
                for ((blk, _), m) in t.impacts.iter().zip(&marks) {
                    assert!(m.starts_with(&format!("B{blk}:")), "{m}");
                }
            }
        }
        assert!(led > 0, "the enemies picked their intents at the round start (AI1)");
        assert_eq!(widget(&b, "strip.title").rect.w, 68, "the hero's glyph takes the title's end");
    }

    /// UI5: the threat intel panel shows the hovered unit of either side: over a party card it is that member, with
    /// its round intent line; over nothing, the first foe.
    #[test]
    fn the_intel_panel_shows_either_side() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        let h = b.selected.unwrap();
        let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).unwrap();
        b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(widget(&b, "intel.unit").text, s(&b).w.units[foe].name, "nothing hovered: the first foe");
        let card = widget(&b, &format!("party.{h}")).rect;
        b.draw(&mut cx, &mut g, (card.x + card.w / 2, card.y + card.h / 2), true);
        assert_eq!(widget(&b, "intel.unit").text, s(&b).w.units[h].name, "a party card hovered: that member");
        let rounds = s(&b).round_intents();
        let head = intent_text::round_head(&s(&b).w, rounds.iter().find(|r| r.unit == h).unwrap()).expect("the hero's intent");
        assert!(unit_tip(s(&b), h, Some(h), &[], &rounds, &[]).iter().any(|t| t.0 == head), "{head}");
        assert!(head.starts_with("Assault: "), "{head}");
    }

    /// V2: with the plan reveal every enemy's tip says so and lists its plan; VISCLAR1 AR1, AR2 (they replace slice 5's
    /// tile-by-tile revealed walks): a revealed enemy whose round opens with a walk has its target arrow's move leg
    /// end where the kit's walk preview ends; an unrevealed enemy's arrow goes straight (two points).
    #[test]
    fn the_plan_reveal_shows_every_enemy_plan() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), true);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        let tells = s(&b).enemy_intents();
        let rounds = s(&b).round_intents();
        assert!(tells.iter().all(|t| t.revealed), "V2: every enemy");
        for t in &tells {
            let head = intent_text::reveal_head(w, t, passive_name).unwrap();
            let tip = unit_tip(s(&b), t.unit, b.selected, &tells, &rounds, &[]);
            assert!(tip.iter().any(|x| x.0 == head), "{head}");
            if t.readers.is_empty() && !t.enraged {
                assert_eq!(head, "Plan revealed");
            }
            for l in &t.plan {
                assert!(tip.iter().any(|x| x.0 == intent_text::plan_line(w, l)));
            }
        }
        // AR2: each enemy whose round opens with walks, and where the kit's preview of the last of those walks ends
        // (the walks before its first command)
        let opening_walks = |sess: &Session| -> Vec<(usize, V2)> {
            let w = &sess.w;
            w.enemies
                .iter()
                .copied()
                .filter_map(|e| {
                    let mut acts: Vec<&emberwake_battle::resolver::ScheduledAction> = sess.resolver.scheduled.iter().filter(|a| a.unit == e && !a.cancelled).collect();
                    acts.sort_by_key(|a| a.start);
                    let ends: Vec<V2> = acts
                        .iter()
                        .map_while(|a| match &a.command.kind {
                            CmdKind::Walk { path, expected_start, .. } => emberwake_battle::command::walk_preview(w, e, path, *expected_start).last().copied(),
                            _ => None,
                        })
                        .collect();
                    // a walk whose command also carries a dash is left out (the dash ends the leg)
                    let carried = acts.get(ends.len()).is_some_and(|a| matches!(a.command.carry, emberwake_battle::command::Carry::Engage(_)));
                    ends.last().copied().filter(|end| *end != w.pos(e) && !carried).map(|end| (e, end))
                })
                .collect()
        };
        let all = |b: &BattleScreen| arrows_of(s(b), &s(b).enemy_intents(), &s(b).round_intents(), &Layers::of(&Value::Null), true);
        let walks = opening_walks(s(&b));
        assert!(!walks.is_empty(), "the enemies walk at the round start");
        let arrows = all(&b);
        let mut legs = 0;
        // (an enemy with no target draws no arrow, AR1; a leg that ends on its target tile is the arrow's end)
        for (e, end) in &walks {
            let Some(a) = arrows.iter().find(|a| a.unit == *e) else { continue };
            assert_eq!(a.points[0], w.pos(*e));
            assert_eq!(a.points[1], *end, "{a:?}");
            legs += 1;
        }
        assert!(legs > 0, "a walking enemy with a target: {arrows:?}");
        // slice 5 review, AR2: an unrevealed enemy's arrow has no move leg (no reveal, no foresight)
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let tells = s(&b).enemy_intents();
        let hidden: Vec<usize> = tells.iter().filter(|t| !t.revealed).map(|t| t.unit).collect();
        assert!(opening_walks(s(&b)).iter().any(|(e, _)| hidden.contains(e)), "an unrevealed enemy walks this round");
        let arrows = all(&b);
        assert!(arrows.iter().filter(|a| hidden.contains(&a.unit)).all(|a| a.points.len() == 2), "{arrows:?}");
    }

    /// The slice 2 review: a foresight reader who falls during the playback stays a reader for that round, and the
    /// enemy's card still names it.
    #[test]
    fn a_reader_who_falls_stays_listed() {
        let (cx, _g, mut b) = arena(Some(TIMELINE), false);
        let h = b.selected.unwrap();
        {
            let sess = b.host.session.as_mut().unwrap();
            sess.w.units[h].foresight_source = "t_sight".into();
            sess.w.units[h].foresight_range = emberwake_rules::unit::FORESIGHT_RANGE_MAX;
            sess.measure_reveals();
        }
        let r = b.send(&cx, json!({"cmd": "confirm"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        b.host.session.as_mut().unwrap().w.units[h].hp = 0;
        let sess = s(&b);
        assert!(!sess.w.units[h].is_active(), "the reader is down");
        let rounds = sess.round_intents();
        let read: Vec<&Intent> = rounds.iter().filter(|r| sess.w.enemies.contains(&r.unit) && r.readers.contains(&h)).collect();
        assert!(!read.is_empty(), "the range reached a foe at scheduling");
        for r in read {
            let tip = unit_tip(sess, r.unit, None, &[], &rounds, &[]);
            let line = intent_text::read_by(&sess.w, r, passive_name);
            assert!(tip.iter().any(|x| x.0 == line) && line.contains(&sess.w.units[h].name), "{line}");
        }
    }

    /// Classic: the cards keep their "Next:" line, no badge row, no intel widget, no glyph space on the strip.
    #[test]
    fn classic_cards_and_intel_are_unchanged() {
        let (mut cx, mut g, mut b) = arena(None, false);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        assert!(s(&b).round_intents().is_empty());
        for t in s(&b).enemy_intents() {
            assert!(widget(&b, &format!("foe.{}", t.unit)).text.ends_with(&intent_text::card_line(w, &t.line)));
        }
        assert!(b.w.iter().all(|x| !x.id.ends_with(".badges") && x.id != "intel.unit"));
        assert_eq!(widget(&b, "strip.title").rect.w, 80);
    }
}

/// RULESET1-U slice 5 (RULESET1 5, 12; RULESET1_INTENT UI2, MV8; the slice 4 review; preference "Battle overlays and
/// the clarity pass"): poise bars and BREAK, the strip's wind-up / impact / recovery, the hit forecasts, the round
/// summary, the overlay groups' flags, the revealed moves shown for the hovered enemy or with Alt, and the reach band of
/// a real Disengage plan. Expectations are kit queries (`poise::forecast`, `round_intent::round_summary`,
/// `skill::attack_band`, the plan), never shipped numbers; the Disengage plan runs on a scratch catalog built here.
#[cfg(test)]
mod slice5_tests {
    use super::*;
    use emberwake_rules::ruleset::{Ruleset, AP_COST, FLANK, POISE, ROUND_INTENT, TIMELINE};

    fn arena(reveal: bool) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.into()), "", reveal);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn assault(cx: &Cx, b: &mut BattleScreen) -> (usize, usize) {
        let h = b.selected.unwrap();
        let foe = s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).unwrap();
        let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
        assert_eq!(r["ok"], json!(true), "{r}");
        (h, foe)
    }

    /// The preference (user 2026-10-30): every group is at its default (on, but VISCLAR1's `first_move` and
    /// `all_bars`); the settings key switches one off and the board stops drawing it; the state lists the flags.
    #[test]
    fn each_overlay_group_has_its_flag() {
        let all = Layers::of(&Value::Null);
        assert_eq!(all.json().as_object().unwrap().len(), Layers::IDS.len());
        assert!(OVERLAY_GROUPS.iter().all(|g| all.json()[g.0] == json!(g.3)));
        let off = Layers::of(&json!({"band": false, "walks": false}));
        assert!(!off.band && !off.walks && off.front && off.poise);
        let (mut cx, mut g, mut b) = arena(false);
        assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let o = overlay_of(s(&b), b.selected, true).unwrap();
        assert!(!o.band.is_empty() && o.shown(&off, None, None, true).band.is_empty() && o.shown(&all, None, None, true).band == o.band);
        cx.settings.v["battle_overlays"] = json!({"band": false});
        assert_eq!(b.state(&cx)["overlay_layers"]["band"], json!(false));
    }

    /// The scratch fight of MV8: a dancer next to a footman, Maneuver Disengage with AP for one Slip Strike, which then
    /// carries the retreat after its hit.
    fn disengage_session() -> (Session, usize, usize) {
        scratch_session(true)
    }

    /// The scratch fight: Maneuver Disengage (`disengage`) or Assault (with AP for two Cleaves) on the footman.
    fn scratch_session(disengage: bool) -> (Session, usize, usize) {
        scratch_session_with(disengage, json!([]))
    }

    /// `scratch_session` with the catalog's `terms` rows (TERMS1 5.1).
    fn scratch_session_with(disengage: bool, terms: Value) -> (Session, usize, usize) {
        let atk = |id: &str, intent: &str, blocks: i32, ap: i32, dash: i32| json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 0.8, "dash_tiles": dash, "target_mode": "ENEMY"});
        let skills = vec![
            json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
            json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
            json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
            json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
            json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
            json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true}),
            atk("g_slip", "maneuver", 2, 2, 2),
            json!({"id": "g_cut", "name": "Cleave", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 0.5, "stagger": 12, "target_mode": "ENEMY"}),
            json!({"id": "g_heavy", "name": "Heavy", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 4, "ap_cost": 1, "power_mult": 0.5, "target_mode": "ENEMY"}),
        ];
        let unit = |id: &str, side: &str, order: Value| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": 40, "deck": {"order": order}});
        let cat = emberwake_data::Catalog::from_value(&json!({"skills": skills, "units": [unit("g_dancer", "party", json!({"maneuver": ["g_slip"], "assault": ["g_cut"]})), unit("g_footman", "enemy", json!({"assault": ["g_heavy"]}))], "terms": terms}));
        let rules = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK, POISE], "basics": ["move", "rest", "strike"]}).as_object().unwrap());
        let row = json!({"id": "g_fight", "name": "Fight", "party": ["g_dancer"], "enemies": ["g_footman"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8});
        let mut def = emberwake_battle::session::EncounterDef::from_obj(row.as_object().unwrap());
        def.ruleset = rules;
        let mut sess = Session::start(std::sync::Arc::new(cat), def, &indexmap::IndexMap::new());
        let (d, f) = (0, 1);
        sess.w.grid.positions[d] = Some(V2::new(4, 4));
        sess.w.grid.positions[f] = Some(V2::new(5, 4));
        sess.w.units[d].ap = if disengage { 2 } else { 4 };
        ri::set_intent(&mut sess.w, d, if disengage { RoundIntent::Maneuver } else { RoundIntent::Assault }).unwrap();
        ri::set_target(&mut sess.w, d, RoundTarget::Unit(f)).unwrap();
        if disengage {
            ri::set_maneuver(&mut sess.w, d, ManeuverKind::Disengage).unwrap();
        }
        let (w, plan) = (&mut sess.w, &mut sess.plan);
        rip::plan_unit(w, plan, d).unwrap_or_else(|e| panic!("the round: {e}"));
        (sess, d, f)
    }

    /// TERMS1 TU5: the battle screen's intent names read the game's `terms` table through the battle's catalog (the
    /// round head of a record, a pick's detail, the glyph's initial, the kit's intent texts); with no row the defaults.
    #[test]
    fn intent_names_read_the_games_terms() {
        let renamed = json!([{"id": "intent_assault", "name": "Onslaught", "origin": "test"}, {"id": "intent_maneuver", "name": "Shift", "origin": "test"}]);
        for (terms, assault, maneuver) in [(json!([]), "Assault", "Maneuver"), (renamed, "Onslaught", "Shift")] {
            let (sess, d, f) = scratch_session_with(false, terms);
            let w = &sess.w;
            assert_eq!(RoundIntent::Assault.label(&w.cat), assault);
            assert_eq!(crate::party::intent_names(&w.cat).get("assault").map(String::as_str), Some(assault), "the deck screens' names (party, Dojo)");
            let line = emberwake_battle::intent::Line { kind: "", target: None, cell: V2::new(0, 0), blocks: (0, 0), recovery: 0, skill: String::new() };
            let rec = emberwake_battle::intent::Intent { unit: d, line, revealed: false, enraged: false, readers: Vec::new(), plan: Vec::new(), morale_state: "", morale_group: -1,
                round_intent: Some(RoundIntent::Assault), round_target: Some(RoundTarget::Unit(f)), maneuver: None, impacts: Vec::new() };
            assert_eq!(intent_text::round_head(w, &rec), Some(format!("{assault}: {}", w.units[f].name)));
            let opt = |i: RoundIntent, kind: Option<ManeuverKind>| rip::PickOption { picks: Vec::new(), intent: Some(i), kind, ap: 2, blocks: 2, problem: None, ap_short: None, drops: Vec::new() };
            assert_eq!(pick_detail(&w.cat, &opt(RoundIntent::Assault, None)), format!("2 blocks, 2 AP; {assault}"));
            assert_eq!(pick_detail(&w.cat, &opt(RoundIntent::Maneuver, Some(ManeuverKind::Engage))), format!("2 blocks, 2 AP; {maneuver} (Engage)"));
            assert_eq!(ri::target_problem(w, d, RoundIntent::Assault, RoundTarget::Myself), format!("{assault} targets an enemy or a tile."), "the kit's intent text");
            let mut g = crate::hud::hud_tests::gfx();
            g.fonts.audit = Some(Vec::new());
            let view = g.view;
            intent_glyph(&mut Painter { c: &mut g.canvas, v: view, fonts: &mut g.fonts }, &w.cat, RoundIntent::Assault, Rect::new(0, 0, 10, 10));
            let letters: Vec<String> = g.fonts.audit.take().unwrap_or_default().into_iter().map(|r| r.text).collect();
            assert_eq!(letters, [assault[..1].to_string()], "the glyph's initial");
        }
    }

    /// The slice 3 review on a real plan: under a Disengage round the reach band centres on the tile the Slip Strike
    /// hits from (before its retreat), and the retreat is drawn as an after-move.
    #[test]
    fn a_disengage_retreat_keeps_the_band_on_the_strike_tile() {
        let (sess, d, _) = disengage_session();
        let w = &sess.w;
        let st = sess.plan.compile(w, d);
        let first = st.entries.iter().find(|e| emberwake_battle::skill::kind(&w.cat.skills[e.skill]) == Kind::Attack).expect("the Slip Strike");
        assert!(matches!(first.carry, emberwake_battle::command::Carry::Retreat(_)) && first.pos_before != first.pos_after, "it carries the retreat: {:?}", first.carry);
        let o = overlay_of(&sess, Some(d), true).unwrap();
        let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[d], &w.ruleset);
        let around = |at: V2| (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && (lo..=hi).contains(&c.dist(at))).collect::<Vec<_>>();
        assert_eq!(o.band, around(first.pos_before), "the strike tile");
        assert_ne!(o.band, around(first.pos_after), "not the retreat tile");
        assert!(o.moves.iter().any(|(t, k)| *k == MoveKind::After && t.last() == Some(&first.pos_after)), "{:?}", o.moves);
    }

    /// RULESET1 5, 12: under the rule poise the hover card tells the bar; a planned hit's strip tip carries the kit's
    /// forecast of it (`poise::forecast` on the target's bar now, flank from its strike tile); a unit broken this round
    /// is listed for its BREAK plate; the strip's spans carry wind-up then impact parts for a command that winds up.
    #[test]
    fn poise_breaks_parts_and_forecasts_follow_the_kit() {
        let (mut cx, mut g, mut b) = arena(false);
        assert!(s(&b).w.ruleset.has(POISE), "the timeline row plays poise");
        let (h, foe) = assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        let bar = *emberwake_battle::poise::state(w, foe).unwrap();
        let tip = unit_tip(s(&b), foe, Some(h), &[], &[], &[]);
        assert!(tip.iter().any(|t| t.0.starts_with(&format!("Poise {}/{}", bar.value, bar.max))), "{tip:?}");
        let st = s(&b).plan.compile(w, h);
        let spans = span_parts(s(&b), h, &st.entries.iter().map(|e| (e.start_block, e.end_block(w), String::new(), String::new(), None, 0, 0)).collect::<Vec<_>>());
        for (sp, e) in spans.iter().zip(&st.entries) {
            let winds = emberwake_battle::poise::winds_up(&e.build_command(w, h)) && e.end_block(w) > e.start_block;
            let styles: Vec<aegis_ui::timeline::PartStyle> = sp.parts.iter().map(|p| p.style).collect();
            assert_eq!(!styles.is_empty(), winds, "{styles:?}");
            if winds {
                assert_eq!((sp.parts[0].start, sp.parts[0].end, sp.parts[1].start), (e.start_block, e.end_block(w) - 1, e.end_block(w)));
            }
        }
        assert!(broken_units(w, &Layers::of(&Value::Null)).is_empty(), "nobody broke yet");
        b.host.session.as_mut().unwrap().w.poise.get_mut(&foe).unwrap().broken = true;
        assert_eq!(broken_units(&s(&b).w, &Layers::of(&Value::Null)), vec![foe]);
        assert!(broken_units(&s(&b).w, &Layers::of(&json!({"breaks": false}))).is_empty(), "behind its flag");
        assert!(unit_tip(s(&b), foe, Some(h), &[], &[], &[]).iter().any(|t| t.0.contains("BROKEN")));
    }

    /// RULESET1 12 (forecasts) on the scratch fight: two planned Cleaves (stagger 12) into the footman's wind-up: each
    /// is the kit's `poise::forecast` on the bar the earlier one left (the first on the bar as it stands); a hit with
    /// nothing to say (no stagger, flank or pierce) has none; the strip block's tip and the intel panel carry them.
    #[test]
    fn forecasts_follow_the_kit_hit_by_hit() {
        let (mut sess, d, f) = scratch_session(false);
        // the footman winds up a 4-block Heavy from block 1
        let mut fp = emberwake_battle::plan::Plan::new();
        assert_eq!(fp.add_step(&sess.w, f, emberwake_battle::plan::PlanStep::engage(sess.w.book().find("g_heavy").unwrap(), d), false), "");
        sess.resolver.scheduled.clear();
        fp.commit(&sess.w, &mut sess.resolver);
        let w = &sess.w;
        let st = sess.plan.compile(w, d);
        let fcs = hit_forecasts(&sess, d);
        assert_eq!(fcs.len(), st.entries.len());
        let hits: Vec<(usize, &HitForecast)> = fcs.iter().enumerate().filter_map(|(k, x)| x.as_ref().map(|x| (k, x))).collect();
        assert!(hits.len() >= 2, "two Cleaves: {}", hits.len());
        let mut bar = *emberwake_battle::poise::state(w, f).unwrap();
        for (k, h) in &hits {
            let e = &st.entries[*k];
            let cmd = e.build_command(w, d);
            let want = emberwake_battle::poise::forecast(w, &sess.resolver.scheduled, f, bar, e.end_block(w), &cmd.impact, cmd.block_cost, ri::flank_bonus_from(w, d, strike_tile(e), f), cmd.ignore_guard);
            assert_eq!(h.fc, want, "entry {k}");
            bar.value = want.after;
            if matches!(want.verdict, emberwake_battle::poise::Verdict::Breaks(_)) {
                (bar.value, bar.broken) = (0, true);
            }
        }
        assert!(hits[0].1.fc.windup, "the first lands in the footman's wind-up");
        // on the screen: the strip tip and the intel panel
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.settings.v["battle_view"] = json!("flat");
        let mut g = crate::hud::hud_tests::gfx();
        let mut b = BattleScreen::blank(&cx, Mode::Arena);
        b.host.begin(sess);
        b.after_start("");
        b.selected = Some(d);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let e0 = &b.host.session.as_ref().unwrap().plan.compile(&b.host.session.as_ref().unwrap().w, d).entries[hits[0].0].clone();
        let block = b.w.iter().find(|x| x.id == format!("strip.block{}", e0.start_block)).unwrap();
        assert!(block.tip.iter().any(|t| t.0 == "Forecast on g_footman"), "{:?}", block.tip);
        assert!(block.tip.iter().any(|t| t.0.starts_with("Stagger 12 x in wind-up")), "{:?}", block.tip);
    }

    /// RULESET1 12: once the round is over, the state's summary is the kit's `round_summary` and the message leads
    /// with it; classic has neither.
    #[test]
    fn the_round_summary_reads_the_kit() {
        let (mut cx, mut g, mut b) = arena(false);
        assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let r = b.send(&cx, json!({"cmd": "resolve"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let sum = ri::round_summary(&s(&b).w, &s(&b).resolver.scheduled);
        let st = b.state(&cx);
        assert_eq!(st["round_summary"]["breaks"], json!(sum.breaks));
        assert_eq!(st["round_summary"]["follow_ups"], json!(sum.follow_ups));
        let text = round_summary_text(s(&b), &Layers::of(&Value::Null)).unwrap();
        assert!(text.starts_with(if sum.breaks == 0 { "no breaks" } else { "1 break" }) || sum.breaks > 1, "{text}");
        assert!(round_summary_text(s(&b), &Layers::of(&json!({"summary": false}))).is_none());
    }

    /// The slice 4 review: during the playback a hostile card's glyph and its token's agree (both from the round
    /// records): the intent's initial is drawn in the card's glyph square for every enemy with an intent.
    #[test]
    fn card_and_token_glyphs_agree_in_playback() {
        let (mut cx, mut g, mut b) = arena(false);
        assault(&cx, &mut b);
        let r = b.send(&cx, json!({"cmd": "confirm"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        let rounds = s(&b).round_intents();
        let mut checked = 0;
        for r in rounds.iter().filter(|r| s(&b).w.enemies.contains(&r.unit)) {
            let Some(i) = r.round_intent else { continue };
            let card = b.w.iter().find(|x| x.id == format!("foe.{}", r.unit)).unwrap().rect;
            let letter = &i.name()[..1];
            let in_card = recs.iter().any(|t| t.text == letter && t.x >= card.x as f32 && t.x <= (card.x + 13) as f32 && t.y >= card.y as f32 && t.y <= (card.y + 12) as f32);
            let at = s(&b).w.pos(r.unit);
            let tile = cell_rect(at);
            let on_token = recs.iter().any(|t| t.text == letter && t.x >= (tile.x + TILE - 12) as f32 && t.x <= (tile.x + TILE) as f32 && t.y >= tile.y as f32 && t.y <= (tile.y + 12) as f32);
            assert!(in_card && on_token, "{}: card {in_card}, token {on_token}", s(&b).w.units[r.unit].name);
            checked += 1;
        }
        assert!(checked > 0, "the enemies hold intents in the playback");
    }
}

/// RULESET1-U slice 6: the overlay groups on the Settings Battle tab, the round summary in the combat log, and the
/// battle screen's text at the four text sizes (TEXTFIT1: no strip label or hero panel line spills, no intent button
/// label is cut). Expectations come from `OVERLAY_GROUPS`, the kit's `summary_line` and the text audit.
#[cfg(test)]
mod slice6_tests {
    use super::*;
    use emberwake_rules::ruleset::TIMELINE;

    fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    /// The preference (user 2026-10-30): the Settings Battle tab lists every overlay group, at its default; a click
    /// switches one off in the draft (Apply saves it under `battle_overlays`, which the battle screen reads).
    #[test]
    fn the_settings_list_every_overlay_group() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut g = crate::hud::hud_tests::gfx();
        let defaults = crate::screens::default_settings();
        assert!(OVERLAY_GROUPS.iter().all(|(id, _, _, on)| defaults["battle_overlays"][*id] == json!(on)), "each at its default");
        assert_eq!(Layers::of(&defaults["battle_overlays"]), Layers::of(&Value::Null));
        let mut st = crate::menus::SettingsScreen::new(&cx);
        st.draw(&mut cx, &mut g, (-500, -500), true);
        st.click_id(&mut cx, "settings.tab.battle").unwrap();
        st.draw(&mut cx, &mut g, (-500, -500), true);
        let ids: Vec<String> = st.widgets().iter().filter_map(|w| w["id"].as_str().filter(|i| i.starts_with("settings.overlay.")).map(String::from)).collect();
        assert_eq!(ids, OVERLAY_GROUPS.iter().map(|g| format!("settings.overlay.{}", g.0)).collect::<Vec<_>>());
        st.click_id(&mut cx, "settings.overlay.band").unwrap();
        let draft = st.state(&cx)["draft"]["battle_overlays"].clone();
        assert_eq!(draft["band"], json!(false));
        assert!(!Layers::of(&draft).band && Layers::of(&draft).front, "the battle screen's reading of it");
    }

    /// Decision "RULESET1-U slice 5 answers": a timeline round closes the combat log with its `[Round]` summary line;
    /// a classic round logs none.
    #[test]
    fn the_round_summary_closes_the_combat_log() {
        for (rules, logged) in [(Some(TIMELINE), true), (None, false)] {
            let (cx, _g, mut b) = arena(rules);
            let r = b.send(&cx, json!({"cmd": "resolve"}));
            assert_eq!(r["ok"], json!(true), "{r}");
            let s = b.host.session.as_ref().unwrap();
            let lines: Vec<&String> = s.resolver.log.iter().filter(|l| l.starts_with("[Round]")).collect();
            if logged {
                assert_eq!(lines, [&ri::summary_line(1, &ri::round_summary(&s.w, &s.resolver.scheduled))]);
            } else {
                assert!(lines.is_empty(), "classic: {lines:?}");
            }
        }
    }

    /// TEXTFIT1 at the four text sizes, 1280x720: in a planned and a played timeline round no strip label spills past
    /// its widget, no hero panel line spills, and no intent button label is cut.
    #[test]
    fn the_timeline_screen_fits_at_every_text_size() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        let h = b.selected.unwrap();
        let foe = b.host.session.as_ref().unwrap().w.enemies[0];
        b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
        for played in [false, true] {
            if played {
                b.send(&cx, json!({"cmd": "resolve"}));
            }
            for scale in [0.85f32, 1.0, 1.15, 1.3] {
                let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
                let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                let ours = |k: &str| v[k].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("strip.") || w.starts_with("hero."))).cloned().collect::<Vec<_>>();
                assert!(ours("spill").is_empty(), "played {played} scale {scale}: {:?}", ours("spill"));
                let cut: Vec<&Value> = v["cut"].as_array().unwrap().iter().filter(|e| ["Assault", "Guard", "Support", "Maneuver"].contains(&e["text"].as_str().unwrap_or(""))).collect();
                assert!(cut.is_empty(), "played {played} scale {scale}: {cut:?}");
            }
        }
    }

    /// TEXTFIT2 (decision "RULESET1-U slice 6 answers"): a played round's frame with a damage or heal float over every
    /// living unit, on the stage and on the flat board, audits clean of the floats at the four text sizes: every float
    /// is recorded as transient, no issue names one, and the frame reports what it reports without them.
    #[test]
    fn a_playback_frame_with_floats_audits_clean_at_every_text_size() {
        for view in ["stage", "flat"] {
            let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
            cx.settings.v["battle_view"] = json!(view);
            g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
            g.view = aegis_ui::painter::View::fit(1280, 720);
            let r = b.send(&cx, json!({"cmd": "resolve"}));
            assert_eq!(r["ok"], json!(true), "{r}");
            let living: Vec<usize> = b.host.session.as_ref().unwrap().w.units.iter().enumerate().filter(|(_, u)| u.hp > 0).map(|(i, _)| i).collect();
            assert!(!living.is_empty());
            let (damage, heal) = (cx.tuning.value.damage_color, cx.tuning.value.heal_color);
            let texts: Vec<String> = living.iter().map(|i| if i % 2 == 0 { format!("-{}", 10 + i) } else { format!("+{}", 10 + i) }).collect();
            for scale in [0.85f32, 1.0, 1.15, 1.3] {
                let audit = |b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, floats: bool| {
                    b.floats.clear();
                    if floats {
                        let now = Instant::now();
                        b.floats.extend(living.iter().zip(&texts).map(|(&unit, text)| Float { unit, text: text.clone(), color: if text.starts_with('-') { damage } else { heal }, born: now, ms: cx.tuning.value.float_ms }));
                    }
                    let recs = crate::hud::hud_tests::frame(b, cx, g, scale);
                    let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                    let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                    (recs, v)
                };
                let (_, bare) = audit(&mut b, &mut cx, &mut g, false);
                let (recs, v) = audit(&mut b, &mut cx, &mut g, true);
                for text in &texts {
                    let drawn: Vec<bool> = recs.iter().filter(|r| &r.text == text).map(|r| r.transient).collect();
                    assert!(!drawn.is_empty() && drawn.iter().all(|&x| x), "{view} scale {scale}: float {text} drawn transient: {drawn:?}");
                }
                let named = |e: &Value| ["text", "shown", "a", "b"].iter().any(|k| e[*k].as_str().is_some_and(|s| texts.iter().any(|t| t == s)));
                for kind in ["cut", "edge", "overlap", "spill"] {
                    let hits: Vec<&Value> = v[kind].as_array().unwrap().iter().filter(|e| named(e)).collect();
                    assert!(hits.is_empty(), "{view} scale {scale}: {kind} {hits:?}");
                }
                assert_eq!(v["counts"], bare["counts"], "{view} scale {scale}: the floats add no issue");
            }
        }
    }
}

/// HOOKS1-UI slice 1 (HOOKS1 GH6, AH3): the ground hazards and the arena telegraph on the board and the strip. A catalog
/// encounter played by the catalog's timeline row with both hook rules switched on; the hazards and the arena schedule
/// are test rows put on the battle's world, and every expectation is a kit query (`ground::seen_at`,
/// `intent::arena_tells`), never a shipped number. Classic marks nothing (no state key).
#[cfg(test)]
mod hook_marks_tests {
    use super::*;
    use emberwake_battle::arena::{self, Hazard};
    use emberwake_battle::ground::{self, Placed};
    use emberwake_data::HazardDef;
    use emberwake_rules::ruleset::TIMELINE;

    const RULES: &str = "+ground_hazards,+arena_hazards";

    fn arena_screen(ruleset: Option<&str>, rules: &str) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), rules, false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    /// Free walkable cells, left to right, top to bottom.
    fn free_cells(w: &World) -> Vec<V2> {
        (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && w.grid.is_walkable(*c) && w.occupant(*c).is_none() && !w.grid.is_exit(*c)).collect()
    }

    fn hazard(id: &str, hidden: bool) -> HazardDef {
        let row = json!({"id": id, "name": format!("Test {id}"), "on": "enter", "damage": 5, "status": "root", "potency": 60, "affects": "foes", "trap": hidden, "hidden": hidden, "duration": 2, "tint": "#40c0ff", "glyph": "T"});
        HazardDef::from_obj(row.as_object().unwrap())
    }

    /// Puts three hazards down (a foe's visible one, a foe's hidden trap, a hero's hidden trap) and an arena hazard on
    /// one column at blocks 2 and 4 of this round; returns the three cells and the column.
    pub(super) fn mark_up(b: &mut BattleScreen) -> ([V2; 3], i32) {
        let sess = b.host.session.as_mut().unwrap();
        let round = sess.current_round;
        let w = &mut sess.w;
        let (hero, foe) = (w.allies[0], w.enemies[0]);
        let free = free_cells(w);
        let cells = [free[0], free[1], free[2]];
        for (c, (id, by, hidden)) in cells.iter().zip([("smoke", foe, false), ("wire", foe, true), ("snare", hero, true)]) {
            w.grid.ground.insert(*c, Placed { def: hazard(id, hidden), by, round, last_round: round + 1 });
        }
        let col = free[free.len() / 2].x;
        let row = json!({"id": "vents", "name": "Test Vents", "columns": [col], "blocks": [2, 4], "effect": {"damage": 9, "stagger": 10}});
        w.arena.hazards = vec![Hazard::from_obj(row.as_object().unwrap())];
        arena::round_start(w, round);
        (cells, col)
    }

    fn tip_text(b: &BattleScreen, id: &str) -> String {
        b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
    }

    /// GH6, AH3: the party sees the foe's visible hazard and its own hidden trap (fainter), not the foe's hidden one;
    /// the struck cells are the kit's arena tells; the tile and strip hover cards name them; the state lists them.
    #[test]
    fn the_marks_follow_the_kit() {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
        let (cells, col) = mark_up(&mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        assert!(ground::on(w) && arena::on(w), "both hook rules on");
        let viewer = w.allies[0];
        let marks = HookMarks::of(s(&b), &Layers::of(&Value::Null));
        let seen: Vec<V2> = marks.ground.iter().map(|g| g.cell).collect();
        let want: Vec<V2> = cells.iter().copied().filter(|c| ground::seen_at(w, viewer, *c).is_some()).collect();
        assert_eq!(seen, want);
        assert_eq!(seen, vec![cells[0], cells[2]], "the foe's hidden trap stays hidden");
        assert_eq!(marks.ground.iter().map(|g| g.hidden).collect::<Vec<_>>(), [false, true]);
        let tells = emberwake_battle::intent::arena_tells(w);
        assert_eq!(marks.arena, tells);
        assert!(!tells[0].cells.is_empty() && tells[0].cells.iter().all(|c| c.x == col));
        // the hover cards
        let tile = |c: V2| format!("tile.{}.{}", c.x, c.y);
        assert!(tip_text(&b, &tile(cells[0])).contains("Test smoke"), "{}", tip_text(&b, &tile(cells[0])));
        assert!(!tip_text(&b, &tile(cells[1])).contains("Test wire"), "the hidden foe trap has no card");
        assert!(tip_text(&b, &tile(cells[2])).contains("Hidden from the foes"));
        let struck = tells[0].cells.iter().copied().find(|c| w.occupant(*c).is_none()).expect("a free struck cell");
        assert!(tip_text(&b, &tile(struck)).contains("Test Vents: strikes here this round"), "{}", tip_text(&b, &tile(struck)));
        for blk in 1..=s(&b).def.total_blocks {
            let t = tip_text(&b, &format!("strip.block{blk}"));
            assert_eq!(t.contains(&format!("Test Vents strikes at the end of block {blk}")), tells[0].blocks.contains(&blk), "block {blk}: {t}");
        }
        // the agent link
        let st = b.state(&cx);
        assert_eq!(st["hook_marks"]["ground"].as_array().unwrap().len(), 2);
        assert_eq!(st["hook_marks"]["arena"], emberwake_battle::intent::arena_json(&tells));
    }

    /// The marks draw on the ground in both views: a frame with them differs from the same frame with both layers off;
    /// with the layers off the marks, the hover lines and the strip lines are gone.
    #[test]
    fn the_layers_switch_the_marks() {
        for view in ["flat", "stage"] {
            let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
            cx.settings.v["battle_view"] = json!(view);
            let (cells, _) = mark_up(&mut b);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let on = g.canvas.px.clone();
            cx.settings.v["battle_overlays"] = json!({"hazards": false, "arena": false});
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_ne!(on, g.canvas.px, "{view}: the marks draw");
            let marks = HookMarks::of(s(&b), &Layers::of(&cx.settings.v["battle_overlays"]));
            assert!(marks.ground.is_empty() && marks.arena.is_empty());
            assert!(!tip_text(&b, &format!("tile.{}.{}", cells[0].x, cells[0].y)).contains("Test smoke"));
            assert!((1..=s(&b).def.total_blocks).all(|blk| !tip_text(&b, &format!("strip.block{blk}")).contains("Test Vents")));
            assert_eq!(b.state(&cx)["hook_marks"]["ground"], json!([]));
        }
    }

    /// Classic (and a timeline battle without the two rules): nothing is marked whatever the flags say, and the state
    /// has no `hook_marks` key.
    #[test]
    fn without_the_rules_nothing_is_marked() {
        for (ruleset, rules) in [(None, ""), (Some(TIMELINE), "-ground_hazards,-arena_hazards")] {
            let (mut cx, mut g, mut b) = arena_screen(ruleset, rules);
            mark_up(&mut b);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let marks = HookMarks::of(s(&b), &Layers::of(&Value::Null));
            assert!(marks.ground.is_empty() && marks.arena.is_empty(), "{ruleset:?} {rules}");
            assert!(b.state(&cx).get("hook_marks").is_none(), "{ruleset:?} {rules}: the state keeps its shape");
        }
    }


    /// HOOKS1 CN2 (HOOKS1-UI slice 2): the hover card names each unit's class as `Data::class_name` gives it in the
    /// battle's world (the Arena, a catalog screen: the class row's name); a unit without a class names none.
    #[test]
    fn the_hover_card_names_the_class_in_the_battles_world() {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), "");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(b.class_world, "", "the Arena names classes by their rows");
        let w = &s(&b).w;
        let names = class_names(&cx.data, &b.class_world, w);
        let mut seen = 0;
        for (i, u) in w.units.iter().enumerate() {
            let class = cx.data.cat.units.get(&u.catalog_id).map(|d| d.class_id.clone()).filter(|c| cx.data.cat.classes.contains_key(c));
            match class {
                Some(c) => {
                    assert_eq!(names[i], cx.data.class_name("", &c));
                    assert!(unit_tip(s(&b), i, None, &[], &[], &names).iter().any(|t| t.0.starts_with(&format!("Level {} {}", u.stats.level, names[i]))));
                    seen += 1;
                }
                None => assert_eq!(names[i], ""),
            }
        }
        assert!(seen > 0, "a unit with a class");
    }
}

/// HOOKS1-UI slice 3 (HOOKS1 UM7, 8.3, HC1): unit meters on the cards, the inspect card and the strip; the held-guard
/// edge; the HP cost in the strip cards and the step popup, and its refusal in words. A catalog encounter played by the
/// catalog's timeline row with the three hook rules switched on; the meters, the held guard and the HP cost are test
/// rows put on the battle's world. Every expectation is a kit query (`meter::shown`, `trigger::held_at`,
/// `command::hp_cost_of`, `command::hp_cost_problem`), never a shipped number. Classic shows none of it.
#[cfg(test)]
mod meter_guard_tests {
    use super::*;
    use emberwake_battle::meter;
    use emberwake_data::UnitMeterDef;
    use emberwake_rules::ruleset::TIMELINE;
    use emberwake_rules::unit::{HeldGuard, HeldMeter};

    const RULES: &str = "+unit_meters,+triggers,+hp_cost";
    const COST: i32 = 5;

    fn arena_screen(ruleset: Option<&str>, rules: &str) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), rules, false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn tip_text(b: &BattleScreen, id: &str) -> String {
        b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
    }

    fn rect(b: &BattleScreen, id: &str) -> Option<Rect> {
        b.w.iter().find(|x| x.id == id).map(|x| x.rect)
    }

    /// The hero holds a pips meter (0-4 from 3, in band Full), the first foe a bar meter (0-20 from 5), the foe a held
    /// guard 0.5 through block 3, and the hero's attack skills cost COST HP. Returns (hero, foe).
    pub(super) fn hook_up(b: &mut BattleScreen) -> (usize, usize) {
        let h = b.selected.expect("a hero");
        let sess = b.host.session.as_mut().unwrap();
        let w = &mut sess.w;
        let f = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe");
        let known = w.units[h].known_skills.clone();
        let cat = std::sync::Arc::make_mut(&mut w.cat);
        let phial = json!({"id": "t_phial", "name": "Test Phial", "short": "Phial", "max": 4, "show": "pips", "bands": [{"id": "low", "name": "Low", "from": 0}, {"id": "full", "name": "Full", "from": 3}]});
        let heat = json!({"id": "t_heat", "name": "Test Heat", "short": "Heat", "max": 20, "show": "bar"});
        for row in [phial, heat] {
            let def = UnitMeterDef::from_obj(row.as_object().unwrap());
            cat.unit_meters.insert(def.id.clone(), def);
        }
        for sk in cat.skills.iter_mut().filter(|k| known.contains(&k.id) && k.kind == "ATTACK") {
            sk.hp_cost = COST;
        }
        w.units[h].passive_meters.push(HeldMeter { meter: "t_phial".into(), start: 3, floor: None });
        w.units[f].passive_meters.push(HeldMeter { meter: "t_heat".into(), start: 5, floor: None });
        meter::battle_start(w);
        w.units[f].held_guard = Some(HeldGuard { mitigation: 0.5, last_block: 3 });
        (h, f)
    }

    /// UM7: the cards grow a meter row, the inspect card names each meter with its band, the strip row shows the
    /// selected hero's; the state lists them as `meter::shown` gives them. 8.3: the foe's held guard shows on its card
    /// and inspect card and in the state.
    #[test]
    fn meters_and_the_held_guard_follow_the_kit() {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
        let (h, f) = hook_up(&mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        let (mh, mf) = (meter::shown(w, h), meter::shown(w, f));
        assert_eq!(meters_of(w, h, &Layers::of(&Value::Null)), mh);
        assert_eq!((mh[0].show.as_str(), mf[0].show.as_str()), ("pips", "bar"));
        assert!(tip_text(&b, &format!("party.{h}")).contains(&meter_text(&mh[0])), "{}", tip_text(&b, &format!("party.{h}")));
        assert!(meter_text(&mh[0]).ends_with(&format!("({})", mh[0].band)) && !mh[0].band.is_empty());
        assert!(tip_text(&b, &format!("foe.{f}")).contains(&meter_text(&mf[0])));
        assert_eq!(rect(&b, &format!("party.{h}")).unwrap().h, 34, "the card grows a meter row");
        assert!(tip_text(&b, "strip.meters").contains(&meter_text(&mh[0])));
        let held = emberwake_battle::trigger::held_at(w, f, s(&b).current_block).expect("held");
        assert!(tip_text(&b, &format!("foe.{f}")).contains(&format!("Held guard {:.0}% through B3", held * 100.0)));
        let st = b.state(&cx);
        let units: Vec<i64> = st["meters"].as_array().unwrap().iter().map(|m| m["unit"].as_i64().unwrap()).collect();
        assert!(units.contains(&(h as i64)) && units.contains(&(f as i64)));
        assert_eq!(st["held_guards"], json!([{"unit": f, "mitigation": 0.5, "last_block": 3}]));
    }

    /// The `meters` and `guard` layers: off, the rows, the strip row and the edge go (a frame differs), the cards keep
    /// their classic height; the inspect lines stay (as the poise line does).
    #[test]
    fn the_layers_switch_meters_and_the_guard_edge() {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
        let (h, f) = hook_up(&mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let on = g.canvas.px.clone();
        cx.settings.v["battle_overlays"] = json!({"meters": false, "guard": false});
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(on, g.canvas.px);
        assert!(rect(&b, "strip.meters").is_none());
        assert_eq!(rect(&b, &format!("party.{h}")).unwrap().h, 30);
        assert!(tip_text(&b, &format!("foe.{f}")).contains("Held guard"), "the inspect line stays");
        let st = b.state(&cx);
        assert_eq!((st["meters"].clone(), st["held_guards"].clone()), (json!([]), json!([])));
    }

    /// HC1: a planned step that costs HP says so on its strip card and in the step popup (its detail and its card); at
    /// or below the cost the popup row is refused and reads the refusal in words.
    #[test]
    fn the_hp_cost_shows_on_the_strip_and_the_popup() {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
        let (h, f) = hook_up(&mut b);
        let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}));
        assert_eq!(r["ok"], json!(true), "{r}");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        let costly: Vec<String> = s(&b).plan.compile(w, h).entries.iter().map(|e| &w.cat.skills[e.skill]).filter(|k| emberwake_battle::command::hp_cost_of(w, k) > 0).map(|k| k.name.clone()).collect();
        assert!(!costly.is_empty(), "the plan holds a costly attack");
        let text = intent_text::hp_cost_text(COST);
        assert!((1..=s(&b).def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains(&text)));
        let rows = b.options_for(h, "foe", Some(f), None);
        let row = rows.iter().find(|r| r.tip.iter().any(|t| t.0 == text)).expect("a costly step in the popup");
        assert!(!row.valid || row.detail.contains(&format!("{COST} HP")), "{}", row.detail);
        b.host.session.as_mut().unwrap().w.units[h].hp = COST;
        let rows = b.options_for(h, "foe", Some(f), None);
        let refused = rows.iter().find(|r| r.tip.iter().any(|t| t.0 == text)).expect("the costly step");
        assert!(!refused.valid);
        assert_eq!(refused.detail, intent_text::refusal_text(&emberwake_battle::command::hp_cost_problem(COST, COST)));
        assert_eq!(intent_text::refusal_text(&emberwake_battle::command::hp_cost_problem(9, 10)), "Not enough HP: 9, it costs 10");
        assert_eq!(intent_text::refusal_text("Out of reach"), "Out of reach");
    }

    /// Classic: no meter, no held guard, no HP cost shows and the state has no `meters` or `held_guards` key.
    #[test]
    fn classic_shows_none_of_it() {
        let (mut cx, mut g, mut b) = arena_screen(None, "");
        let (h, _) = hook_up(&mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(meters_of(&s(&b).w, h, &Layers::of(&Value::Null)).is_empty());
        assert!(rect(&b, "strip.meters").is_none());
        let st = b.state(&cx);
        assert!(st.get("meters").is_none() && st.get("held_guards").is_none());
        assert!(!b.w.iter().any(|x| x.tip.iter().any(|t| t.0.starts_with("Held guard") || t.0 == intent_text::hp_cost_text(COST))));
    }


    /// The slice 3 review: a timeline battle without unit_meters, triggers and hp_cost shows none of the marks either
    /// (the kit queries answer nothing without their rules), and its state has no `meters` or `held_guards` key.
    #[test]
    fn a_timeline_battle_without_the_rules_shows_none_of_it() {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), "-unit_meters,-triggers,-hp_cost");
        let (h, f) = hook_up(&mut b);
        let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}));
        assert_eq!(r["ok"], json!(true), "{r}");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        assert!(!meter::on(w) && !emberwake_battle::trigger::on(w));
        assert!((0..w.units.len()).all(|i| meters_of(w, i, &Layers::of(&Value::Null)).is_empty() && held_guard(s(&b), i).is_none()));
        assert!(rect(&b, "strip.meters").is_none());
        assert_eq!(rect(&b, &format!("party.{h}")).unwrap().h, 30);
        let st = b.state(&cx);
        assert!(st.get("meters").is_none() && st.get("held_guards").is_none());
        assert!(!b.w.iter().any(|x| x.tip.iter().any(|t| t.0.starts_with("Held guard") || t.0 == intent_text::hp_cost_text(COST) || t.0.starts_with("Test Phial"))));
    }
}

/// HOOKS1-UI slice 4 (HOOKS1 5): the selected hero's push and pull forecasts. A catalog encounter played by the
/// catalog's timeline row with forced_move switched on; the hero's attack skills get a test push 2 in the battle's
/// catalog. Every expectation is the kit's `forced::forecast`; the kit test `forced_move.rs` checks that a forecast is
/// the move the hit makes.
#[cfg(test)]
mod push_forecast_tests {
    use super::*;
    use emberwake_rules::ruleset::TIMELINE;

    fn arena_screen(rules: &str) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), rules, false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn tip_text(b: &BattleScreen, id: &str) -> String {
        b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
    }

    /// The hero's attack skills push 2 (a test row on the battle's catalog), the hero plans Assault on its first foe.
    pub(super) fn plan_push(cx: &Cx, b: &mut BattleScreen) -> (usize, usize) {
        let h = b.selected.expect("a hero");
        let sess = b.host.session.as_mut().unwrap();
        let w = &mut sess.w;
        let f = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe");
        let known = w.units[h].known_skills.clone();
        let push = emberwake_data::ForcedMove::from_obj(json!({"kind": "push", "tiles": 2, "collide_mult": 0.5}).as_object().unwrap());
        for sk in std::sync::Arc::make_mut(&mut w.cat).skills.iter_mut().filter(|k| known.contains(&k.id) && k.kind == "ATTACK") {
            sk.forced_move = push.clone();
        }
        let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}));
        assert_eq!(r["ok"], json!(true), "{r}");
        (h, f)
    }

    /// The forecasts are the kit's for each planned entry; each one shows on its strip card and in the state; the board
    /// draws them (a frame differs with the layer off, and then no card line and no state entry stays).
    #[test]
    fn the_push_forecasts_follow_the_kit() {
        let (mut cx, mut g, mut b) = arena_screen("+forced_move");
        let (h, f) = plan_push(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let sess = s(&b);
        let w = &sess.w;
        let per_entry = push_forecasts(sess, h);
        let entries = sess.plan.compile(w, h).entries;
        assert_eq!(per_entry.len(), entries.len());
        for (e, got) in entries.iter().zip(&per_entry) {
            let cmd = e.build_command(w, h);
            let targets: Vec<usize> = match &cmd.kind {
                CmdKind::Attack { target, .. } => target.iter().copied().collect(),
                CmdKind::Area { cells, .. } => cells.iter().filter_map(|c| w.occupant(*c)).collect(),
                _ => Vec::new(),
            };
            assert_eq!(*got, emberwake_battle::forced::forecast(w, &sess.resolver.scheduled, h, strike_tile(e), &cmd, e.end_block(w), &targets));
        }
        let all: Vec<&emberwake_battle::forced::Forecast> = per_entry.iter().flatten().collect();
        assert!(all.iter().any(|x| x.unit == f), "the planned attack on the foe forecasts its push");
        for x in &all {
            let line = push_text(w, x).0;
            assert!((1..=sess.def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains(&line)), "{line}");
        }
        let st = b.state(&cx);
        assert_eq!(st["push_forecasts"], Value::Array(all.iter().map(|x| push_json(x)).collect()));
        let on = g.canvas.px.clone();
        cx.settings.v["battle_overlays"] = json!({"pushes": false});
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(on, g.canvas.px, "the arrows draw");
        assert!(!(1..=s(&b).def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains("Pushes ")));
        assert_eq!(b.state(&cx)["push_forecasts"], json!([]));
    }

    /// Without the rule nothing is forecast and the state has no `push_forecasts` key.
    #[test]
    fn without_the_rule_nothing_is_forecast() {
        let (mut cx, mut g, mut b) = arena_screen("-forced_move");
        let (h, _) = plan_push(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(push_forecasts(s(&b), h).is_empty());
        assert!(b.state(&cx).get("push_forecasts").is_none());
    }
}

/// HOOKS1-UI slice 5: the text audit of every HOOKS1-UI addition at 1280 x 720 and the four text sizes (TEXTFIT1), and
/// the intel panel that fits its lines (decision "RULESET1-B2 tuning and HOOKS1-UI slice 3 answers"). One timeline
/// battle with every hook rule on and every slice's test rows: ground hazards and the arena (slice 1), meters, a held
/// guard and HP costs (slice 3), a planned push (slice 4).
#[cfg(test)]
mod hooks_audit_tests {
    use super::*;
    use emberwake_rules::ruleset::TIMELINE;

    const ALL: &str = "+ground_hazards,+arena_hazards,+unit_meters,+triggers,+hp_cost,+forced_move";
    const SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];

    fn marked(view: &str) -> (Cx, Gfx, BattleScreen, usize, usize) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!(view);
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), ALL, false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        super::hook_marks_tests::mark_up(&mut b);
        super::meter_guard_tests::hook_up(&mut b);
        let (h, f) = super::push_forecast_tests::plan_push(&cx, &mut b);
        let mut g = crate::hud::hud_tests::gfx();
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        (cx, g, b, h, f)
    }

    /// The problems of the screen's own texts: spills out of the strip, the cards, the tiles and the intel panel; cut
    /// texts on the board and the side panels; texts off the canvas.
    fn faults(b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, scale: f32) -> Vec<Value> {
        let recs = crate::hud::hud_tests::frame(b, cx, g, scale);
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        // spills: only out of the widgets the HOOKS1-UI marks live in or add lines to (the strip, the party and hostile
        // cards, the board tiles, the intel panel); other screens' widgets have their own audits
        let ours = |e: &Value| e["widget"].as_str().is_some_and(|w| ["strip.", "party.", "foe.", "tile.", "intel."].iter().any(|p| w.starts_with(p)));
        let mut out: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| ours(e)).cloned().collect();
        // cut by design and not HOOKS1-UI's (TEXTFIT1, RULESET1-U): a strip span's label (`strip.block<n>`), the bar's
        // status line (`bar.status`) and its ruleset chip (`bar.chip`); the cut text's widget is the smallest box
        // holding its start
        let widget_at = |x: f32, y: f32| boxes.iter().filter(|b| x >= b.1 && y >= b.2 && x < b.1 + b.3 && y < b.2 + b.4).min_by(|a, b| (a.3 * a.4).total_cmp(&(b.3 * b.4))).map(|b| b.0.as_str());
        let by_design = |e: &Value| {
            let (x, y) = (e["at"][0].as_f64().unwrap_or(-1.0) as f32 + 0.5, e["at"][1].as_f64().unwrap_or(-1.0) as f32 + 0.5);
            widget_at(x, y).is_some_and(|id| id == "bar.status" || id == "bar.chip" || id.starts_with("strip.block"))
        };
        out.extend(v["cut"].as_array().unwrap().iter().filter(|e| !by_design(e)).cloned());
        out.extend(v["edge"].as_array().unwrap().iter().cloned());
        out
    }

    /// TEXTFIT1: in both views, planning and played, at the four sizes, no HOOKS1-UI text spills, is cut or leaves the
    /// canvas.
    #[test]
    fn every_hook_addition_fits_at_every_text_size() {
        for view in ["flat", "stage"] {
            let (mut cx, mut g, mut b, _, _) = marked(view);
            for played in [false, true] {
                if played {
                    b.send(&cx, json!({"cmd": "resolve"}));
                }
                for scale in SIZES {
                    let f = faults(&mut b, &mut cx, &mut g, scale);
                    assert!(f.is_empty(), "{view} played {played} scale {scale}: {f:?}");
                }
            }
        }
    }

    /// The Settings Battle tab lists the overlay groups (and, in a debug build, VISCLAR1's Debug row) without a cut,
    /// spilled, overlapping or off-canvas label at any size.
    #[test]
    fn the_overlay_settings_fit_at_every_text_size() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut g = crate::hud::hud_tests::gfx();
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        let mut st = crate::menus::SettingsScreen::new(&cx);
        st.draw(&mut cx, &mut g, (-500, -500), true);
        st.click_id(&mut cx, "settings.tab.battle").unwrap();
        for scale in SIZES {
            let recs = crate::hud::hud_tests::frame(&mut st, &mut cx, &mut g, scale);
            let boxes: Vec<(String, f32, f32, f32, f32)> = st.widgets().iter().map(|w| {
                let r = crate::hud::hud_tests::rect(w);
                (w["id"].as_str().unwrap_or("").to_string(), r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)
            }).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            // (INPUT1-PB: the playback speed select at the heading row's end too)
            let spill: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("settings.overlay.") || w.starts_with("settings.debug.") || w == "settings.playback_speed")).collect();
            assert!(spill.is_empty() && v["cut"].as_array().unwrap().is_empty() && v["edge"].as_array().unwrap().is_empty(), "scale {scale}: {v}");
            // VISCLAR1-A: the tighter rows (24 groups and the Debug row) do not overlap
            assert_eq!(v["counts"]["overlap"], json!(0), "scale {scale}: {}", v["overlap"]);
            let ids = st.widgets().iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("settings.overlay."))).count();
            assert_eq!(ids, OVERLAY_GROUPS.len());
            let debug = st.widgets().iter().filter(|w| w["id"] == json!("settings.debug.party_foresight_off")).count();
            assert_eq!(debug, usize::from(cfg!(debug_assertions)), "VC5: the debug row only in a debug build");
            assert!(st.widgets().iter().any(|w| w["id"] == json!("settings.playback_speed")), "PB3: the playback speed");
        }
    }

    /// The intel panel keeps every line that fits; past its room the Level line gives way first, then it ends with
    /// "More on its hover card", and what it draws never runs past the room.
    #[test]
    fn the_intel_panel_drops_the_level_line_first_then_points_to_the_hover_card() {
        let mut g = crate::hud::hud_tests::gfx();
        let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        let lines: Vec<(String, Rgba)> = vec![
            ("Goblin Brawler".into(), FOE),
            ("HP 108/108   AP 12/12   Neutral".into(), TEXT),
            ("Level 3 Brawler (minion)   Speed 21   Reach 1".into(), DIM),
            ("Poise 46/46".into(), POISE),
            ("Test Heat 5/20".into(), METER),
            ("Held guard 50% through B3".into(), HELD),
            ("Warrior deals about 47 with a Strike, takes about 22".into(), ALLY),
        ];
        let height = |parts: &[(String, Rgba, bool)]| parts.iter().map(|x| if x.2 { INTEL_HEAD } else { INTEL_LINE }).sum::<f32>();
        let all = intel_parts(&mut p, &lines, 1000.0);
        assert!(all.iter().any(|x| x.0.starts_with("Level ")) && all.last().unwrap().0 != INTEL_MORE, "room for all");
        let full = height(&all);
        let less = intel_parts(&mut p, &lines, full - 1.0);
        assert!(!less.iter().any(|x| x.0.starts_with("Level ")), "the Level line goes first");
        assert!(less.last().unwrap().0.contains("takes about") && less.last().unwrap().0 != INTEL_MORE, "the forecast keeps its place: {less:?}");
        let tight = intel_parts(&mut p, &lines, 4.0 * INTEL_LINE + INTEL_HEAD);
        assert_eq!(tight.last().unwrap().0, INTEL_MORE);
        assert!(height(&tight) <= 4.0 * INTEL_LINE + INTEL_HEAD, "{tight:?}");
    }
}

/// ENCOUNTER2-U (ENCOUNTER2 H1, H3, H7, S2): the hold escape's screen on the kit sample `kit_winch_escape` (a catalog
/// encounter). Every expectation is the kit's (`HoldGate`, `Session::needs_stay_pick`, `gate_note`, `final_outcome`);
/// classic and every fight without a hold escape draw as before.
#[cfg(test)]
mod hold_screen_tests {
    use super::*;

    const ENC: &str = "kit_winch_escape";

    fn screen(encounter: &str) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let b = BattleScreen::arena_with(&cx, None, encounter, None, "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a Widget<Action>> {
        b.w.iter().find(|x| x.id == id)
    }

    fn tip_text(b: &BattleScreen, id: &str) -> String {
        b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
    }

    fn exit_tile(w: &World) -> V2 {
        *w.grid.exits.iter().find(|c| w.grid.is_valid(**c)).expect("an exit")
    }

    /// H1: before the pick the panel offers each hero on the field, Confirm waits, the objective line asks for the pick
    /// and the gate is shut; a pick names the holder in the objective line and the hold tiles' cards, frees Confirm and
    /// closes the panel. H3: the hero panel shows the kit's gate note while the gate is shut.
    #[test]
    fn the_pick_comes_first_then_names_the_holder() {
        let (mut cx, mut g, mut b) = screen(ENC);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let gate = s(&b).w.hold.clone().expect("a hold escape");
        assert!(s(&b).needs_stay_pick());
        let offered: Vec<usize> = gate.heroes.iter().copied().filter(|&c| s(&b).w.has(c)).collect();
        for &c in &offered {
            assert!(widget(&b, &format!("hold.pick.{c}")).is_some_and(|x| x.enabled), "hero {c} offered");
        }
        assert!(!widget(&b, "strip.confirm").unwrap().enabled, "Confirm waits for the pick");
        assert!(b.message.contains(&format!("Pick who holds {} first.", gate.label)), "{}", b.message);
        let exit = exit_tile(&s(&b).w);
        assert!(tip_text(&b, &format!("tile.{}.{}", exit.x, exit.y)).contains("shut"));
        let st = b.state(&cx);
        assert_eq!((st["hold"]["needs_pick"].clone(), st["hold"]["open"].clone()), (json!(true), json!(false)));

        let pick = offered[0];
        b.run(&mut cx, Action::StayPick(pick));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(s(&b).w.hold.as_ref().unwrap().holder, Some(pick));
        assert!(b.message.contains(&hold_line(s(&b))) && !hold_line(s(&b)).is_empty(), "{}", b.message);
        assert!(widget(&b, &format!("hold.pick.{pick}")).is_none() && widget(&b, "hold.pick").is_none());
        assert!(widget(&b, "strip.confirm").unwrap().enabled);
        let tile = gate.tiles[0];
        assert!(tip_text(&b, &format!("tile.{}.{}", tile.x, tile.y)).contains(&format!("{} holds it", s(&b).w.units[pick].name)));
        let note = s(&b).gate_note();
        b.selected = Some(pick);
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        let words: String = recs.iter().filter(|r| r.x < 126.0).map(|r| r.text.clone()).collect::<Vec<_>>().join(" ");
        assert_eq!(words.contains("The gate is shut"), !note.is_empty(), "the hero panel shows the kit's note {note:?}: {words}");
        assert_eq!(b.state(&cx)["hold"]["holder"], json!(pick));
    }

    /// H2, H7: with the holder on a hold tile the gate opens (the exit's card says so and the sign reads OPEN); with the
    /// `hold` and `gate` layers off the cards and the frame lose the marks.
    #[test]
    fn the_gate_follows_the_holder_and_the_layers_switch_it() {
        let (mut cx, mut g, mut b) = screen(ENC);
        let pick = hold_pick_ids(s(&b))[0];
        b.run(&mut cx, Action::StayPick(pick));
        let tile = s(&b).w.hold.as_ref().unwrap().tiles[0];
        {
            let w = &mut b.host.session.as_mut().unwrap().w;
            if let Some(o) = w.occupant(tile) {
                w.grid.positions.swap(o, pick);
            } else {
                w.grid.positions[pick] = Some(tile);
            }
        }
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let w = &s(&b).w;
        assert!(w.hold.as_ref().unwrap().is_open(w), "the holder on its tile opens the gate");
        let exit = exit_tile(w);
        assert!(tip_text(&b, &format!("tile.{}.{}", exit.x, exit.y)).contains("The gate is open."));
        assert_eq!(exit_sign(HoldMarks::of(s(&b), &Layers::of(&Value::Null)).as_ref()).0, "OPEN");
        let on = g.canvas.px.clone();
        cx.settings.v["battle_overlays"] = json!({"hold": false, "gate": false});
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(on, g.canvas.px);
        assert!(!tip_text(&b, &format!("tile.{}.{}", exit.x, exit.y)).contains("gate"));
        assert!(!tip_text(&b, &format!("tile.{}.{}", tile.x, tile.y)).contains("holds it"));
        assert_eq!(exit_sign(HoldMarks::of(s(&b), &Layers::of(&cx.settings.v["battle_overlays"])).as_ref()).0, "EXIT");
    }

    /// S2: the screen's outcome is the session's `final_outcome`: after the last round a `timeout_outcome` of lost is
    /// honoured (the old screen rule read fled).
    #[test]
    fn the_outcome_is_the_sessions_final_outcome() {
        let (_cx, _g, mut b) = screen(ENC);
        assert_eq!(b.outcome(), s(&b).final_outcome());
        let sess = b.host.session.as_mut().unwrap();
        sess.timeout_outcome = "lost".into();
        sess.phase = Phase::Replay;
        sess.current_round = sess.def.max_rounds;
        assert!(!s(&b).can_advance_round());
        assert_eq!((b.outcome(), s(&b).final_outcome()), ("lost".to_string(), "lost".to_string()));
    }

    /// Classic and a fight without a hold escape: no pick panel, no `hold` key, exits read EXIT.
    #[test]
    fn without_a_hold_escape_nothing_changes() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let encounter = cx.data.cat.encounters.keys().find(|k| k.as_str() != ENC).cloned().unwrap();
        cx.settings.v["battle_view"] = json!("flat");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
        let mut g = crate::hud::hud_tests::gfx();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        if s(&b).w.hold.is_some() {
            return;
        }
        assert!(widget(&b, "hold.pick").is_none() && b.state(&cx).get("hold").is_none());
        assert!(HoldMarks::of(s(&b), &Layers::of(&Value::Null)).is_none());
        assert_eq!(exit_sign(None).0, "EXIT");
    }

    /// Decision "RULESET1-A3 loose ends": the battle screen's combat log panel reads the kit's `Session::log_lines`, so
    /// an encounter line (a phase's `line`, a phase kill; the kit test `phase_kill.rs` says them) shows in it where it
    /// was said, while the resolver log stays as it is.
    #[test]
    fn phase_lines_reach_the_combat_log_panel() {
        let (mut cx, mut g, mut b) = screen(ENC);
        let line = "[Phase] Test: The first captive falls.".to_string();
        b.host.session.as_mut().unwrap().encounter_lines.push((0, line.clone()));
        b.show_log = true;
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        assert!(recs.iter().any(|r| r.text.starts_with("[Phase] Test")), "the panel draws the phase line");
        assert!(widget(&b, "log.panel").is_some_and(|x| x.text == format!("{} lines", s(&b).resolver.log.len() + 1)));
        assert!(!s(&b).resolver.log.contains(&line));
    }
}

/// ENCOUNTER2-C2 (docs/specs/ENCOUNTER2_C.md 3.3; section 6 "battle.rs tests" and the text audit) on a scratch world
/// built here (no shipped data): encounter `t_talk` raises a talk at round 1's start (Ash speaks) and one at round 1
/// block 2 (the Grunt speaks); `t_plain` is the same fight without phases. Expectations come from the rules: while a
/// talk is open the screen sends no request and autoplay waits (DT5); its replies run in the Arena's scratch state or
/// the game in play (DT6, DT9); closing resumes the playback after `impact_gap_ms` (DT10); the agent link reads it
/// (DT11); the speaker's token is the one whose hero key plays the speaker (DT8).
#[cfg(test)]
mod talk_screen_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::screens::Game;
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_world::Data;
    use std::sync::Arc;

    const WORLD: &str = "talkw";
    const LONG_LINE: &str = "The riders took the north road at dusk, and the forge has been cold since; if we hold this alley until the bell, the wardens can close the gate behind us and nobody else has to bleed for it tonight.";

    fn with_ids(rows: Value) -> Value {
        Value::Array(
            json::arr(&rows)
                .iter()
                .map(|r| {
                    let mut r = json::dict(r).clone();
                    let id = s(&r, "id");
                    r.insert("_world".into(), WORLD.into());
                    r.insert("_qid".into(), format!("{WORLD}:{id}").into());
                    Value::Object(r)
                })
                .collect(),
        )
    }

    fn data() -> Data {
        let skills = json!([
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ]);
        let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5});
        // a party character's base unit is class-derived (the world's new game checks it)
        let hero = json!({"id": "t_hero", "name": "t_hero", "side": "party", "class_id": "drill", "level": 1});
        let class = json!({"id": "drill", "name": "Drill", "base_stats": {"STR": 8, "DEX": 10, "INT": 6, "CONS": 30, "PERC": 10, "KNOW": 10}});
        let catalog = json!({"skills": skills, "classes": [class], "units": [hero, unit("t_foe", "enemy")]});
        let encounter = |id: &str, phases: Value| {
            json!({"id": id, "name": id, "rows": vec![".".repeat(12); 9], "party": ["ash"], "enemies": ["grunt"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 3, "phases": phases})
        };
        let phases = json!([
            {"id": "opening", "name": "Opening", "trigger": {"kind": "round", "round": 1}, "effects": [{"type": "talk", "dialogue": "dlg_open", "speaker": "ash"}]},
            {"id": "mid", "name": "Mid", "trigger": {"kind": "round", "round": 1, "block": 2}, "effects": [{"type": "talk", "dialogue": "dlg_mid", "speaker": "grunt"}]}
        ]);
        let dialogues = json!([
            {"id": "dlg_open", "start": "a", "services": false, "speaker": "ash", "nodes": [
                {"id": "a", "text": "Hold the line.", "next": "b"},
                {"id": "b", "text": "Ready?", "choices": [{"text": "Ready.", "effects": [{"effect": "set_flag", "key": "ready", "value": true}]}]}]},
            {"id": "dlg_mid", "start": "a", "services": false, "speaker": "grunt", "nodes": [{"id": "a", "text": "Again.", "end": true}]},
            {"id": "dlg_long", "start": "a", "services": false, "speaker": "ash", "nodes": [{"id": "a", "text": LONG_LINE, "choices": [
                {"text": "We hold the alley until the bell, whatever comes down the north road.", "effects": [{"effect": "set_flag", "key": "hold", "value": true}]},
                {"text": "Fall back to the gate.", "effects": [{"effect": "add_flag", "key": "fallback", "delta": 1}]},
                {"text": "Say nothing."}]}]}
        ]);
        let worlds = json!({"talkw": {"manifest": {"id": WORLD, "name": "Talk", "start_location": "hub", "start_party": ["ash"]}, "own": {
            "locations": with_ids(json!([{"id": "hub", "name": "Hub"}])),
            "characters": with_ids(json!([{"id": "ash", "name": "Ash", "role": "party", "base_unit": "t_hero"}, {"id": "grunt", "name": "Grunt", "role": "enemy", "base_unit": "t_foe"}])),
            "dialogues": with_ids(dialogues),
            "encounters": with_ids(json!([encounter("t_talk", phases), encounter("t_plain", json!([]))]))
        }}});
        Data::from_values(&catalog, &worlds, &json!({}))
    }

    /// A debug game of the scratch world (skip animations on, so no impact holds a step).
    fn talk_cx() -> Cx {
        let mut cx = game_cx();
        let d = Arc::new(data());
        let gs = emberwake_world::new_game::create_for_world(&d, WORLD, 1).expect("the scratch world starts a game");
        cx.data = d;
        cx.game = Some(Game::new(gs, -1, true));
        cx.settings.v["battle_view"] = json!("flat");
        cx.settings.v["skip_anim"] = json!(true);
        cx
    }

    fn arena(cx: &Cx, encounter: &str) -> BattleScreen {
        let b = BattleScreen::arena_with(cx, Some(WORLD.into()), encounter, None, "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        b
    }

    /// One tick with the step pause and the impact gap over.
    fn tick_ready(b: &mut BattleScreen, cx: &mut Cx) {
        b.next_at = Instant::now();
        b.gap_until = None;
        b.tick(cx);
    }

    fn widget(b: &BattleScreen, id: &str) -> Option<Value> {
        b.widgets().into_iter().find(|w| w["id"] == id)
    }

    /// The unit whose hero key plays world character `id`.
    fn unit_of(b: &BattleScreen, cx: &Cx, id: &str) -> usize {
        let qid = qid_in(&cx.data, WORLD, id);
        let s = b.session();
        (0..s.w.units.len()).find(|&c| s.hero_key(c) == qid).expect("the character fights")
    }

    /// DT5, DT6, DT8, DT9, DT11: the round-1 talk opens at the first tick, before planning; while it is open no action,
    /// key, Esc or right click sends a request and autoplay's plan waits; its Continue and reply run in the Arena's
    /// scratch state; once it closes autoplay's plan goes out.
    #[test]
    fn a_round_one_talk_opens_before_planning_and_holds_every_request() {
        let (mut cx, mut g) = (talk_cx(), gfx());
        let mut b = arena(&cx, "t_talk");
        assert_eq!(b.session().talks().len(), 1, "round 1's talk is cued at the start");
        assert!(b.planning() && b.state(&cx).get("talk").is_none(), "nothing open before the first tick");
        b.tick(&mut cx);
        let st = b.state(&cx);
        let qid = qid_in(&cx.data, WORLD, "ash");
        assert!(!qid.is_empty());
        let read = (st["talk"]["dialogue"].clone(), st["talk"]["node"].clone(), st["talk"]["speaker"].clone(), st["talk"]["text"].clone());
        assert_eq!(read, (json!("dlg_open"), json!("a"), json!(qid), json!("Hold the line.")));
        assert_eq!(st["talk"]["replies"], json!([{"id": "dialogue.continue", "text": "Continue", "locked": false}]));
        assert_eq!(st["talk"]["speaker_unit"], json!(unit_of(&b, &cx, "ash")), "Ash's token wears the ring");
        let sent = b.host.history.len();
        for a in [Action::Confirm, Action::Step, Action::Autoplay, Action::Undo, Action::ToggleLog, Action::Leave] {
            b.act(&mut cx, a);
        }
        for k in [Key::Char('a'), Key::Char('z'), Key::Char('l'), Key::Char('r')] {
            b.key(&mut cx, k);
        }
        b.escape(&mut cx);
        b.back(&mut cx);
        assert_eq!(b.host.history.len(), sent, "no request while the talk is open");
        assert!(!b.auto && !b.show_log && b.planning() && cx.nav.is_empty(), "every battle action and key is swallowed");
        frame(&mut b, &mut cx, &mut g, 1.0);
        for id in ["dialogue", "talk.backdrop", "dialogue.continue"] {
            assert!(widget(&b, id).is_some(), "{id} drawn");
        }
        // autoplay switched on (as the A key does outside a talk) waits for the talk
        b.run(&mut cx, Action::Autoplay);
        assert!(b.auto && b.host.history.len() == sent && b.planning(), "autoplay's plan waits");
        b.click_id(&mut cx, "dialogue.continue").expect("Continue clicks");
        assert_eq!(b.state(&cx)["talk"]["node"], json!("b"));
        b.key(&mut cx, Key::Char('1'));
        assert!(b.talk.is_none() && b.state(&cx).get("talk").is_none(), "the reply ends the talk");
        assert_eq!(b.talk_gs.as_ref().and_then(|gs| gs.flags.get("ready")), Some(&json!(true)), "the flag lands in the Arena's scratch state");
        assert!(!cx.gs().flags.contains_key("ready"), "the game in play is not the Arena's");
        b.tick(&mut cx);
        assert!(b.host.history.len() > sent && b.session().plan_confirmed, "autoplay's plan goes out once the talk closed");
    }

    /// DT5, DT10: the block-2 talk opens during the playback and holds it (no step, the block stays); Continue closes it
    /// and the playback steps on once `impact_gap_ms` is over. DT8: the Grunt's token is the speaker's.
    #[test]
    fn the_playback_waits_for_a_block_talk_and_resumes_after_the_gap() {
        let mut cx = talk_cx();
        let mut b = arena(&cx, "t_talk");
        b.tick(&mut cx);
        b.run(&mut cx, Action::Talk(TalkAct::Advance));
        b.run(&mut cx, Action::Talk(TalkAct::Choose(0)));
        assert!(b.talk.is_none());
        b.act(&mut cx, Action::Confirm);
        assert!(b.playing, "{}", b.message);
        for _ in 0..40 {
            tick_ready(&mut b, &mut cx);
            if b.talk.is_some() {
                break;
            }
        }
        assert!(b.talk.is_some(), "the block talk opens during the playback");
        let cue = b.session().talks()[1].clone();
        assert_eq!((cue.dialogue.as_str(), cue.round, cue.block), ("dlg_mid", 1, 2));
        assert_eq!(b.state(&cx)["talk"]["speaker_unit"], json!(unit_of(&b, &cx, "grunt")), "an enemy character speaks from its token");
        let at = |b: &BattleScreen| (b.host.history.len(), b.session().current_round, b.session().current_block);
        let held = at(&b);
        for _ in 0..5 {
            tick_ready(&mut b, &mut cx);
        }
        assert_eq!(at(&b), held, "no step while it is open");
        assert!(b.playing, "the playback is held, not stopped");
        b.key(&mut cx, Key::Enter);
        assert!(b.talk.is_none(), "Continue on an end node closes it");
        if cx.tuning.value.impact_gap_ms > 0 {
            assert!(b.gap_until.is_some_and(|g| g > Instant::now()), "the gap starts at the close");
            b.next_at = Instant::now();
            b.tick(&mut cx);
            assert_eq!(b.host.history.len(), held.0, "not before the gap is over");
        }
        tick_ready(&mut b, &mut cx);
        assert_eq!(b.host.history.len(), held.0 + 1, "the playback steps on");
    }

    /// DT6, DT8, DT9: a world fight's talk runs in the game in play: the reply's flag is the game's; the member who plays
    /// the speaker wears the ring.
    #[test]
    fn a_world_fight_talk_runs_in_the_game_in_play() {
        let mut cx = talk_cx();
        let member = cx.gs().party.active_members().first().map(|m| m.id.clone()).expect("a party member");
        let fight = json::dict(&json!({"encounter": "t_talk"})).clone();
        let mut b = BattleScreen::world(&mut cx, &fight, "map").expect("the world fight starts");
        b.tick(&mut cx);
        let st = b.state(&cx);
        assert_eq!(st["talk"]["dialogue"], json!("dlg_open"));
        let s = b.session();
        let hero = (0..s.w.units.len()).find(|&c| s.hero_key(c) == member).expect("the member fights");
        assert_eq!(st["talk"]["speaker_unit"], json!(hero), "the member who plays Ash wears the ring");
        b.run(&mut cx, Action::Talk(TalkAct::Advance));
        b.run(&mut cx, Action::Talk(TalkAct::Choose(0)));
        assert!(b.talk.is_none() && b.talk_gs.is_none());
        assert_eq!(cx.gs().flags.get("ready"), Some(&json!(true)), "the flag is the game's");
    }

    /// Every battle without a talk: no `talk` key, no veil or box, and autoplay plans at once.
    #[test]
    fn without_a_talk_nothing_changes() {
        let (mut cx, mut g) = (talk_cx(), gfx());
        let mut b = arena(&cx, "t_plain");
        b.tick(&mut cx);
        frame(&mut b, &mut cx, &mut g, 1.0);
        assert!(b.talk.is_none() && b.state(&cx).get("talk").is_none() && widget(&b, "talk.backdrop").is_none() && widget(&b, "dialogue").is_none());
        let sent = b.host.history.len();
        b.act(&mut cx, Action::Autoplay);
        assert!(b.host.history.len() > sent && b.playing, "autoplay plans and confirms at once");
    }

    /// DT7 and the text audit (section 6): the box is the region map's (400 wide times the text size, centred, 6 above
    /// the bottom) over a veil that darkens the board, and its texts read clean (no cut, edge, spill or overlap) at the
    /// four text sizes, with a long line and three replies.
    #[test]
    fn the_battle_talk_box_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (talk_cx(), gfx());
        let mut b = arena(&cx, "t_plain");
        assert!(b.preview_talk(&mut cx, WORLD, "dlg_long", ""), "{}", b.message);
        let probe = |g: &Gfx| {
            let i = (((GRID_Y + 40) * g.canvas.w + GRID_X + 40) * 4) as usize;
            [g.canvas.px[i] as u32, g.canvas.px[i + 1] as u32, g.canvas.px[i + 2] as u32]
        };
        for (size, scale) in TEXT_SIZES {
            // the frame without the talk first: the talk's texts are the ones drawn after it
            let open = b.talk.take();
            let base = frame(&mut b, &mut cx, &mut g, scale as f32).len();
            let bare = probe(&g);
            b.talk = open;
            let recs = frame(&mut b, &mut cx, &mut g, scale as f32);
            let veiled = probe(&g);
            assert!(veiled.iter().zip(bare.iter()).all(|(v, o)| v <= o) && veiled != bare, "{size}: the veil darkens the board ({bare:?} to {veiled:?})");
            let (x, y, w, h) = rect(&widget(&b, "dialogue").expect("the box"));
            assert_eq!(w, ((W - 40) as i64).min((talk::TALK_W as f64 * scale).round() as i64), "{size}: width");
            assert!((x - (W as i64 - x - w)).abs() <= 1 && H as i64 - (y + h) == talk::TALK_GAP as i64, "{size}: centred at the bottom");
            let talk_recs: Vec<aegis_ui::painter::TextRec> = recs[base..].to_vec();
            assert!(talk_recs.iter().any(|r| !r.text.is_empty() && LONG_LINE.starts_with(r.text.as_str())), "{size}: the line is drawn");
            let boxes: Vec<(String, f32, f32, f32, f32)> = b
                .widgets()
                .iter()
                .filter(|wd| {
                    let id = wd["id"].as_str().unwrap_or("");
                    id == "dialogue" || id.starts_with("dialogue.") || id.starts_with("choice.")
                })
                .map(|wd| {
                    let (bx, by, bw, bh) = rect(wd);
                    (s(json::dict(wd), "id"), bx as f32, by as f32, bw as f32, bh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&talk_recs, W as f32, H as f32, &boxes);
            let counts: Vec<i64> = ["cut", "edge", "spill", "overlap"].iter().map(|k| v["counts"][*k].as_i64().unwrap_or(-1)).collect();
            assert_eq!(counts, [0, 0, 0, 0], "{size}: {v}");
        }
        // the replies run in the scratch state: add_flag counts up
        b.run(&mut cx, Action::Talk(TalkAct::Choose(1)));
        assert_eq!(b.talk_gs.as_ref().map(|gs| json::i(&gs.flags, "fallback", 0)), Some(1));
    }
}

/// VISCLAR1 slice A (INPUT1.md 10: VC1-VC6, AR1-AR6): the calmer board of a timeline battle. A catalog encounter on
/// the catalog's timeline row; every expectation is a rule of the spec or a kit query (the plan, the records, the
/// positions), never a shipped number. Classic keeps today's board.
#[cfg(test)]
mod visclar_tests {
    use super::*;
    use emberwake_rules::ruleset::TIMELINE;

    fn arena(ruleset: Option<&str>, view: &str) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!(view);
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    /// The selected hero takes the first foe on the field as its Assault target.
    fn assault(cx: &Cx, b: &mut BattleScreen) -> (usize, usize) {
        let h = b.selected.unwrap();
        let foe = s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).unwrap();
        let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
        assert_eq!(r["ok"], json!(true), "{r}");
        (h, foe)
    }

    fn arrows(b: &BattleScreen, cx: &Cx) -> Vec<Value> {
        b.state(cx)["overlay"]["arrows"].as_array().cloned().unwrap_or_default()
    }

    fn units_of(a: &[Value]) -> Vec<usize> {
        a.iter().map(|x| x["unit"].as_u64().unwrap() as usize).collect()
    }

    /// VC6, VC5: `default_settings` holds every group on but `first_move` and `all_bars`, and no debug key; a saved
    /// `battle_overlays` without a group takes the group's default; Restore defaults returns to them.
    #[test]
    fn the_defaults_are_the_calmer_view() {
        let d = crate::screens::default_settings();
        for (id, _, _, on) in OVERLAY_GROUPS {
            assert_eq!(d["battle_overlays"][id], json!(on), "{id}");
            assert_eq!(on, !matches!(id, "first_move" | "all_bars"), "{id}");
        }
        assert!(d.get(FORESIGHT_OFF).is_none());
        let older = Layers::of(&json!({"band": true, "moves": true}));
        assert!(!older.first_move && !older.all_bars && older.facings && older.glyphs, "an older file shows the calmer view");
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut g = crate::hud::hud_tests::gfx();
        let mut st = crate::menus::SettingsScreen::new(&cx);
        st.draw(&mut cx, &mut g, (-500, -500), true);
        st.click_id(&mut cx, "settings.tab.battle").unwrap();
        st.draw(&mut cx, &mut g, (-500, -500), true);
        st.click_id(&mut cx, "settings.overlay.first_move").unwrap();
        st.click_id(&mut cx, "settings.overlay.all_bars").unwrap();
        let draft = st.state(&cx)["draft"].clone();
        assert!(Layers::of(&draft["battle_overlays"]).first_move && Layers::of(&draft["battle_overlays"]).all_bars, "a click turns each on");
        st.click_id(&mut cx, "settings.defaults").unwrap();
        let draft = st.state(&cx)["draft"].clone();
        let l = Layers::of(&draft["battle_overlays"]);
        assert!(!l.first_move && !l.all_bars && l.moves && l.walks, "Restore defaults");
    }

    /// VC4, VC5: the reader is false outside a debug build whatever the file says, and the checkbox (a debug build)
    /// sets the key.
    #[test]
    fn party_foresight_off_is_a_debug_build_reader() {
        let on = json!({FORESIGHT_OFF: true});
        assert!(!party_foresight_off(&on, false), "a release build never reads it");
        assert!(party_foresight_off(&on, true));
        assert!(!party_foresight_off(&json!({}), true) && !party_foresight_off(&json!({FORESIGHT_OFF: false}), true));
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut g = crate::hud::hud_tests::gfx();
        let mut st = crate::menus::SettingsScreen::new(&cx);
        st.draw(&mut cx, &mut g, (-500, -500), true);
        st.click_id(&mut cx, "settings.tab.battle").unwrap();
        st.draw(&mut cx, &mut g, (-500, -500), true);
        let has = st.widgets().iter().any(|w| w["id"] == json!("settings.debug.party_foresight_off"));
        assert_eq!(has, cfg!(debug_assertions), "VC5: a debug build only");
        if has {
            st.click_id(&mut cx, "settings.debug.party_foresight_off").unwrap();
            assert_eq!(st.state(&cx)["draft"][FORESIGHT_OFF], json!(true));
        }
    }

    /// VC1: under the round intent the tells' kind chips need the group `first_move` (off by default); classic keeps
    /// them. VC2, VC3: the token bars draw for the focus set (the selected hero and its target here, nothing hovered),
    /// for every unit with Alt or `all_bars`; classic draws every unit's.
    #[test]
    fn chips_and_bars_follow_the_focus_set() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "flat");
        let (h, foe) = assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let l = Layers::of(&cx.settings.v["battle_overlays"]);
        assert!(!first_move_chips(true, &l) && first_move_chips(true, &Layers::of(&json!({"first_move": true}))) && first_move_chips(false, &l));
        let st = b.state(&cx);
        assert_eq!(st["focus"], json!([h, foe]), "the selected hero and its round target");
        let focus: Vec<usize> = vec![h, foe];
        let n = s(&b).w.units.len();
        assert!(n > 2);
        for i in 0..n {
            assert_eq!(bars_shown(true, &l, false, &focus, i), focus.contains(&i), "unit {i}");
            assert!(bars_shown(true, &l, true, &focus, i), "Alt: unit {i}");
            assert!(bars_shown(true, &Layers::of(&json!({"all_bars": true})), false, &focus, i), "all_bars: unit {i}");
            assert!(bars_shown(false, &l, false, &[], i), "classic: unit {i}");
        }
        // the inspected unit joins it
        let other = s(&b).w.enemies.iter().copied().find(|&e| e != foe && s(&b).w.units[e].is_active() && s(&b).w.has(e));
        if let Some(e) = other {
            b.inspect = Some((e, s(&b).current_round));
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_eq!(b.state(&cx)["focus"], json!([h, foe, e]));
        }
        // classic: no focus or overlay key
        let (mut cx, mut g, mut b) = arena(None, "flat");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let st = b.state(&cx);
        assert!(st.get("focus").is_none() && st.get("overlay").is_none());
    }

    /// AR1-AR4 and the acceptance: by default the agent link's `overlay.arrows` holds the selected hero's arrow only
    /// (to its target, alpha 90); with Alt one per unit with a target, never two for a unit; an unrevealed enemy's has
    /// two points (no move leg); the inspected unit's shows by default (Q15); with the tells hidden no enemy's.
    #[test]
    fn one_quiet_arrow_per_unit() {
        for view in ["stage", "flat"] {
            let (mut cx, mut g, mut b) = arena(Some(TIMELINE), view);
            let (h, foe) = assault(&cx, &mut b);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let a = arrows(&b, &cx);
            assert_eq!(units_of(&a), vec![h], "{view}: the selected hero's only");
            let w = &s(&b).w;
            let fp = w.pos(foe);
            assert_eq!(a[0]["points"][0], json!([w.pos(h).x, w.pos(h).y]));
            assert_eq!(*a[0]["points"].as_array().unwrap().last().unwrap(), json!([fp.x, fp.y]), "{view}: it ends at the target");
            assert_eq!(a[0]["alpha"], json!(90));
            cx.held.alt = true;
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let a = arrows(&b, &cx);
            let mut seen = units_of(&a);
            let total = seen.len();
            seen.sort();
            seen.dedup();
            assert_eq!(seen.len(), total, "{view}: at most one arrow a unit: {a:?}");
            assert!(seen.len() > 1 && seen.contains(&h), "{view}: Alt shows every unit's: {a:?}");
            assert!(a.iter().all(|x| x["alpha"] == json!(90) && x["points"].as_array().unwrap().len() >= 2));
            let tells = s(&b).enemy_intents();
            for x in a.iter().filter(|x| tells.iter().any(|t| x["unit"] == json!(t.unit) && !t.revealed)) {
                assert_eq!(x["points"].as_array().unwrap().len(), 2, "{view}: an unrevealed enemy goes straight: {x}");
            }
            // every party member on the field with a unit or tile target has one
            let w = &s(&b).w;
            for &m in w.allies.iter().filter(|&&m| w.units[m].is_active() && w.has(m)) {
                if let Some(RoundTarget::Unit(_) | RoundTarget::Cell(_)) = w.round_states.get(&m).and_then(|r| r.target) {
                    assert!(seen.contains(&m), "{view}: member {m}: {a:?}");
                }
            }
            // Q15: the inspected unit's arrow beside the selected hero's, without Alt
            cx.held.alt = false;
            let enemy_arrow = seen.iter().copied().find(|u| w.enemies.contains(u));
            if let Some(e) = enemy_arrow {
                b.inspect = Some((e, s(&b).current_round));
                b.draw(&mut cx, &mut g, (-500, -500), true);
                assert_eq!(units_of(&arrows(&b, &cx)), vec![h, e], "{view}");
            }
            // the tells hidden: no enemy arrow even with Alt
            cx.held.alt = true;
            cx.settings.v["show_tells"] = json!(false);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert!(units_of(&arrows(&b, &cx)).iter().all(|u| s(&b).w.allies.contains(u)), "{view}");
        }
    }

    /// AR1, AR5: the move leg: the hero's arrow bends where its moves before its first command end (with the group
    /// `moves` on; off: straight); an enemy's leg needs `walks`.
    #[test]
    fn the_move_leg_follows_the_plan_and_the_groups() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "stage");
        let (h, foe) = assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let sess = s(&b);
        let leg = party_leg(sess, h);
        let tells = sess.enemy_intents();
        let rounds = sess.round_intents();
        let on = arrows_of(sess, &tells, &rounds, &Layers::of(&Value::Null), true);
        let off = arrows_of(sess, &tells, &rounds, &Layers::of(&json!({"moves": false})), true);
        let mine = |v: &[Arrow]| v.iter().find(|a| a.unit == h).cloned().expect("the hero's arrow");
        let (hp, fp) = (sess.w.pos(h), sess.w.pos(foe));
        let want = if leg == hp { vec![hp, fp] } else { vec![hp, leg, fp] };
        assert_eq!(mine(&on).points, want, "the move leg ends where the moves before the first command end");
        assert_eq!(mine(&off).points, vec![hp, fp], "moves off: straight");
        // the leg's end is where a planned move ends or the first command strikes from (or the hero's own tile)
        let o = overlay_of(sess, Some(h), true).unwrap();
        let entries = sess.plan.compile(&sess.w, h).entries;
        assert!(leg == hp || o.moves.iter().any(|(t, _)| t.last() == Some(&leg)) || entries.iter().any(|e| strike_tile(e) == leg), "{leg:?} {:?}", o.moves);
        let foes_off = arrows_of(sess, &tells, &rounds, &Layers::of(&json!({"walks": false})), true);
        assert!(foes_off.iter().filter(|a| !a.ally).all(|a| a.points.len() == 2));
    }

    /// AR6 (Q6, Q16): during playback the board draws no target arrow and no destination plate (the overlay's moves
    /// are empty), with Alt too; the facing chevrons stay (VC6).
    #[test]
    fn playback_draws_no_arrow_and_no_plate() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "stage");
        assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(!arrows(&b, &cx).is_empty());
        let r = b.send(&cx, json!({"cmd": "confirm"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        assert!(!s(&b).is_planning());
        cx.held.alt = true;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let st = b.state(&cx);
        assert_eq!(st["overlay"]["arrows"], json!([]), "{}", st["overlay"]);
        assert_eq!(st["overlay"]["moves"], json!([]));
        let w = &s(&b).w;
        let facing = w.enemies.iter().filter(|&&e| w.units[e].is_active() && w.has(e) && ri::facing(w, e) != emberwake_battle::grid::ZERO).count();
        assert_eq!(st["overlay"]["facings"].as_array().unwrap().len(), facing, "the chevrons stay");
        assert!(Layers::of(&cx.settings.v["battle_overlays"]).facings);
    }

    /// VC4: with Party foresight off a Hawk Eye reveal reads unrevealed on the screen (no "Read by" line on the
    /// enemy's hover card; its record `revealed` false with no plan lines, so the badge row is not filled) while the
    /// kit's records keep it; off, the screen shows it. Its arrow goes straight (AR2).
    #[test]
    fn party_foresight_off_hides_foresight_reveals() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "flat");
        let h = b.selected.unwrap();
        {
            let sess = b.host.session.as_mut().unwrap();
            sess.w.units[h].foresight_source = "t_sight".into();
            sess.w.units[h].foresight_range = emberwake_rules::unit::FORESIGHT_RANGE_MAX;
            sess.measure_reveals();
        }
        let tells = s(&b).enemy_intents();
        let read: Vec<usize> = tells.iter().filter(|t| t.revealed && !t.readers.is_empty() && !t.enraged).map(|t| t.unit).collect();
        assert!(!read.is_empty(), "the range reveals a foe");
        let seen = seen_intents(&s(&b).w, tells.clone(), true);
        for t in seen.iter().filter(|t| read.contains(&t.unit)) {
            assert!(!t.revealed && t.plan.is_empty() && t.readers.is_empty(), "{t:?}");
        }
        assert_eq!(seen_intents(&s(&b).w, tells.clone(), false), tells, "off: unchanged");
        let tip_of = |b: &BattleScreen, e: usize| b.w.iter().find(|x| x.id == format!("unit.{e}")).map(|x| x.tip.iter().map(|l| l.0.clone()).collect::<Vec<_>>()).unwrap_or_default();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(read.iter().all(|&e| tip_of(&b, e).iter().any(|l| l.starts_with("Read by "))), "the reveal shows");
        cx.settings.v[FORESIGHT_OFF] = json!(true);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        for &e in &read {
            assert!(!tip_of(&b, e).iter().any(|l| l.starts_with("Read by ")), "foe {e}: {:?}", tip_of(&b, e));
        }
        assert!(s(&b).enemy_intents().iter().filter(|t| read.contains(&t.unit)).all(|t| t.revealed), "display only: the kit still reveals");
        cx.held.alt = true;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let a = arrows(&b, &cx);
        assert!(a.iter().filter(|x| read.iter().any(|e| x["unit"] == json!(e))).all(|x| x["points"].as_array().unwrap().len() == 2), "{a:?}");
    }
}

#[cfg(test)]
mod timeline_view_tests {
    //! INPUT1-T (INPUT1.md 8, TV1-TV7): the full timeline view over the grid panel (key T), built by
    //! `timeline_view::build`, the Gallery's Battle timeline's builder too.
    use super::*;
    use crate::timeline_view::{masked_note, track_key, unit_of};
    use aegis_ui::timeline::{Timeline, Track, HEADER_H, ROW_H, RULER_H};
    use emberwake_rules::ruleset::TIMELINE;

    fn arena(ruleset: Option<&str>, reveal: bool) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", reveal);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        // the planning start drafts the ghosts (GH2 a)
        b.tick(&mut cx);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
        b.draw(cx, g, (-500, -500), true);
    }

    /// The view as last drawn (it must be open).
    fn view(b: &BattleScreen) -> &Timeline {
        b.tv_shown.as_ref().expect("the view is open")
    }

    fn tracks(t: &Timeline, section: usize) -> &[Track] {
        &t.sections[section].tracks
    }

    fn track(t: &Timeline, u: usize) -> &Track {
        t.sections.iter().flat_map(|x| &x.tracks).find(|x| x.key == track_key(u)).unwrap_or_else(|| panic!("no track for unit {u}"))
    }

    /// TV1: T opens the view over the grid panel's rect and closes it; Esc closes it first (the inspected unit stays,
    /// SL3's steps after); while open the board takes no click (the view's widgets are on top); classic: T does nothing.
    #[test]
    fn t_opens_the_view_over_the_grid_and_t_or_esc_closes_it() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        drawn(&mut cx, &mut g, &mut b);
        assert_eq!(b.state(&cx)["timeline_view"], Value::Null, "closed at the start");
        let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.has(e)).unwrap();
        let cell = b.w.iter().find(|x| x.id == format!("unit.{foe}")).map(|x| x.rect).expect("the foe's token");
        b.key(&mut cx, Key::Char('t'));
        drawn(&mut cx, &mut g, &mut b);
        assert!(b.state(&cx)["timeline_view"].is_object());
        let grid = b.panels.iter().find(|p| p.0 == "grid").unwrap().1;
        assert_eq!((grid.x, grid.y, grid.w, grid.h), (126, 28, 388, 292), "the grid panel's rect");
        let back = b.w.iter().find(|x| x.id == "timeline.backdrop").expect("the backdrop").rect;
        assert_eq!(back, grid, "the view takes the grid panel's rect");
        assert!(matches!(aegis_ui::ui::hit(&b.w, (cell.x + cell.w / 2, cell.y + cell.h / 2)), Some(Action::TimelinePick(_))), "the board takes no click");
        b.key(&mut cx, Key::Char('t'));
        drawn(&mut cx, &mut g, &mut b);
        assert_eq!(b.state(&cx)["timeline_view"], Value::Null, "T closes it");
        assert!(b.w.iter().all(|x| !x.id.starts_with("timeline.")));
        b.key(&mut cx, Key::Char('t'));
        b.inspect_unit(foe);
        b.escape(&mut cx);
        assert!(!b.tv_open && b.inspected() == Some(foe), "Esc closes the view first");
        b.escape(&mut cx);
        assert_eq!(b.inspected(), None, "then SL3's steps");
        let (mut cx, mut g, mut b) = arena(None, false);
        b.key(&mut cx, Key::Char('t'));
        drawn(&mut cx, &mut g, &mut b);
        assert!(!b.tv_open && b.state(&cx).get("timeline_view").is_none(), "classic: no view, no key");
    }

    /// TV2: "Party" (allies in roster order, present or acting) above "Enemies" (the hostile list's order); the ruler
    /// is the round's blocks; no playhead while planning.
    #[test]
    fn the_sections_hold_one_track_per_unit_in_order() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.tv_open = true;
        drawn(&mut cx, &mut g, &mut b);
        let t = view(&b);
        let w = &s(&b).w;
        assert_eq!(t.sections.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(), ["Party", "Enemies"]);
        let keys = |side: &[usize]| side.iter().copied().filter(|&u| w.has(u) || s(&b).resolver.scheduled.iter().any(|a| a.unit == u)).map(track_key).collect::<Vec<_>>();
        assert_eq!(tracks(t, 0).iter().map(|x| x.key.clone()).collect::<Vec<_>>(), keys(&w.allies));
        assert_eq!(tracks(t, 1).iter().map(|x| x.key.clone()).collect::<Vec<_>>(), keys(&w.enemies));
        assert_eq!((t.blocks, t.playhead), (s(&b).def.total_blocks, 0));
        assert!(tracks(t, 0).iter().chain(tracks(t, 1)).all(|x| !x.hatched), "nobody is down yet");
    }

    /// TV3: a party track is the member's plan (each compiled entry a span, the planner's ghost in the ghost style),
    /// its label leads with the intent's letter, its card holds AP9's lines; the selected hero's track is selected.
    #[test]
    fn a_party_track_is_the_plan_with_its_ghost_and_ap_card() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.tv_open = true;
        drawn(&mut cx, &mut g, &mut b);
        let t = view(&b).clone();
        let sess = s(&b);
        let h = b.selected.expect("a hero is selected");
        assert_eq!(t.selected_track, Some(track_key(h)));
        for &u in sess.w.allies.iter().filter(|&&u| sess.w.has(u)) {
            let tr = track(&t, u);
            let entries = sess.plan.compile(&sess.w, u).entries;
            let ghosts = rip::ghost_entries(&sess.w, &sess.plan, u);
            let spans: Vec<(i32, i32, bool)> = entries.iter().enumerate().map(|(i, e)| (e.start_block, e.end_block(&sess.w), ghosts[i] && sess.w.cat.skills[e.skill].kind != "REST")).collect();
            assert_eq!(tr.segments.iter().map(|x| (x.start, x.end, x.ghost)).collect::<Vec<_>>(), spans, "unit {u}");
            assert!(tr.segments.iter().filter(|x| x.ghost).all(|x| x.tip[0].0.starts_with("Auto-filled")), "GH8: a ghost's card");
            let i = ri::intent_of(&sess.w, u).expect("the ghost's intent");
            assert_eq!(tr.label, format!("{} {}", &i.name()[..1], sess.w.units[u].name));
            let card: Vec<String> = tr.tip.iter().map(|l| l.0.clone()).collect();
            for l in ap_lines(&rip::ap_forecast(&sess.w, &sess.plan, u)) {
                assert!(card.contains(&l), "AP9 line {l:?} in {card:?}");
            }
        }
        assert!(t.sections[0].tracks.iter().any(|x| x.segments.iter().any(|g| g.ghost)), "the planner's ghost shows");
    }

    /// TV4: while planning an unrevealed enemy is masked: its note is the intent and target and the tell (no other
    /// skill of its plan), its impact badges are marks, it holds no span; with the tells hidden the note drops the tell.
    #[test]
    fn an_unrevealed_enemy_is_masked_with_its_intent_tell_and_badges() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.tv_open = true;
        drawn(&mut cx, &mut g, &mut b);
        let t = view(&b).clone();
        let sess = s(&b);
        let w = &sess.w;
        let tells = sess.enemy_intents();
        let mut masked = 0;
        for tr in tracks(&t, 1) {
            let e = unit_of(&tr.key).unwrap();
            let rec = tells.iter().find(|r| r.unit == e);
            if rec.is_some_and(|r| r.revealed) {
                continue;
            }
            masked += 1;
            let note = tr.masked.clone().expect("masked");
            assert_eq!(note, masked_note(w, rec, true));
            assert!(tr.segments.is_empty(), "no span of its plan in the value");
            let r = rec.expect("a tell");
            if let Some(head) = intent_text::round_head(w, r) {
                assert!(note.starts_with(&head), "{note}");
            }
            for a in sess.resolver.scheduled.iter().filter(|a| a.unit == e && a.command.name != r.line.skill) {
                assert!(!note.contains(&a.command.name), "{note} names {}", a.command.name);
            }
            let impacts: Vec<(i32, String)> = r.impacts.iter().map(|(b, k)| (*b, if *k == emberwake_battle::intent::HIT { "!" } else { "*" }.to_string())).collect();
            assert_eq!(tr.marks.iter().map(|m| (m.block, m.glyph.clone())).collect::<Vec<_>>(), impacts, "V1's badges as marks");
            assert!(tr.marks.iter().all(|m| m.colour == if m.glyph == "!" { FOE } else { GOLD }));
        }
        assert!(masked > 0, "an enemy plans in secret");
        cx.settings.v["show_tells"] = json!(false);
        drawn(&mut cx, &mut g, &mut b);
        for tr in tracks(view(&b), 1).iter().filter(|x| x.masked.is_some()) {
            let e = unit_of(&tr.key).unwrap();
            assert_eq!(tr.masked.clone().unwrap(), masked_note(&s(&b).w, s(&b).enemy_intents().iter().find(|r| r.unit == e), false));
            assert!(!tr.masked.clone().unwrap().contains("Next:"));
        }
    }

    /// TV4: during playback an unrevealed enemy's track holds only its actions that have started (block <= the
    /// playhead); the playhead is the resolving block (INPUT1 PB2: the strip's cursor, the block the last step
    /// resolved; the session's current block before the first step).
    #[test]
    fn during_playback_an_unrevealed_enemy_shows_only_what_has_started() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.act(&mut cx, Action::Confirm);
        b.tv_open = true;
        let mut checked = 0;
        assert!(s(&b).plan_confirmed, "{}", b.message);
        while s(&b).phase != Phase::Replay {
            drawn(&mut cx, &mut g, &mut b);
            let t = view(&b).clone();
            let at = b.cursor_block(Instant::now()).unwrap_or(s(&b).current_block);
            let sess = s(&b);
            assert_eq!(t.playhead, at);
            let rounds = sess.round_intents();
            for tr in tracks(&t, 1) {
                let e = unit_of(&tr.key).unwrap();
                assert!(tr.masked.is_none(), "no mask during playback");
                if rounds.iter().find(|r| r.unit == e).is_some_and(|r| r.revealed) {
                    continue;
                }
                assert!(tr.segments.iter().all(|x| x.start <= at), "block {at}: {:?}", tr.segments);
                let started = sess.resolver.scheduled.iter().filter(|a| a.unit == e && a.start <= at).count();
                assert_eq!(tr.segments.len(), started, "every started action");
                checked += 1;
            }
            b.send(&cx, json!({"cmd": "step"}));
        }
        assert!(checked > 0, "an unrevealed enemy was checked during playback");
    }

    /// TV4: a revealed enemy (the battle's plan reveal here) is unmasked with its whole plan and the card "Plan
    /// revealed (...)"; VC4: a foresight reveal is masked again with Party foresight off (debug builds).
    #[test]
    fn a_revealed_enemy_is_unmasked_and_foresight_off_masks_it_again() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), true);
        b.tv_open = true;
        drawn(&mut cx, &mut g, &mut b);
        let t = view(&b).clone();
        let sess = s(&b);
        for tr in tracks(&t, 1).iter().filter(|x| sess.w.units[unit_of(&x.key).unwrap()].is_active()) {
            let e = unit_of(&tr.key).unwrap();
            assert!(tr.masked.is_none(), "revealed: unmasked");
            assert_eq!(tr.segments.len(), sess.resolver.scheduled.iter().filter(|a| a.unit == e).count(), "its whole plan");
            assert!(tr.tip.iter().any(|l| l.0 == "Plan revealed (plan reveal)"), "{:?}", tr.tip);
        }
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        let h = b.selected.unwrap();
        {
            let sess = b.host.session.as_mut().unwrap();
            sess.w.units[h].foresight_source = "t_sight".into();
            sess.w.units[h].foresight_range = emberwake_rules::unit::FORESIGHT_RANGE_MAX;
            sess.measure_reveals();
        }
        let read: Vec<usize> = s(&b).enemy_intents().iter().filter(|t| t.revealed && !t.readers.is_empty() && !t.enraged).map(|t| t.unit).collect();
        assert!(!read.is_empty(), "the range reveals a foe");
        b.tv_open = true;
        drawn(&mut cx, &mut g, &mut b);
        for &e in &read {
            let tr = track(view(&b), e);
            assert!(tr.masked.is_none() && tr.tip.iter().any(|l| l.0.starts_with("Plan revealed (Read by ")), "{:?}", tr.tip);
        }
        cx.settings.v[FORESIGHT_OFF] = json!(true);
        drawn(&mut cx, &mut g, &mut b);
        for &e in &read {
            assert_eq!(track(view(&b), e).masked.is_some(), cfg!(debug_assertions), "VC4 masks it in a debug build");
        }
    }

    /// TV5: a click on a party track selects that hero and the view stays open; an enemy track or a block selects
    /// nothing.
    #[test]
    fn a_party_track_click_selects_its_hero() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.tv_open = true;
        drawn(&mut cx, &mut g, &mut b);
        let h = b.selected.unwrap();
        let other = s(&b).w.allies.iter().copied().find(|&a| a != h && s(&b).w.has(a)).expect("a second hero");
        b.click_id(&mut cx, &format!("timeline.track.{}", track_key(other))).unwrap();
        assert_eq!((b.selected, b.tv_open), (Some(other), true));
        let foe = unit_of(&tracks(view(&b), 1)[0].key).unwrap();
        b.click_id(&mut cx, &format!("timeline.track.{}", track_key(foe))).unwrap();
        b.click_id(&mut cx, "timeline.block.1").unwrap();
        assert_eq!((b.selected, b.tv_open, b.popup.is_none()), (Some(other), true, true), "nothing else happens");
        drawn(&mut cx, &mut g, &mut b);
        assert_eq!(view(&b).selected_track, Some(track_key(other)));
    }

    /// TV6: 16 tracks fit in the view before the widget scrolls; at the four text sizes no label or note spills, and a
    /// shortened one reads in full on its widget's hover card.
    #[test]
    fn the_view_fits_at_every_text_size() {
        let inner = GRID_H * TILE + 4 - 2 * TV_PAD;
        assert!(RULER_H + 2 + 2 * (HEADER_H + 3) + 16 * ROW_H <= inner, "16 tracks before the scroll");
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        b.tv_open = true;
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let spills: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("timeline."))).collect();
            assert!(spills.is_empty(), "scale {scale}: {spills:?}");
            for c in v["cut"].as_array().unwrap() {
                let (x, y) = (c["at"][0].as_f64().unwrap() as i32 + 1, c["at"][1].as_f64().unwrap() as i32 + 2);
                let full = c["text"].as_str().unwrap();
                let Some(wd) = b.w.iter().rev().find(|w| w.rect.contains(x, y) && w.id.starts_with("timeline.") && w.id != "timeline.backdrop") else { continue };
                assert!(wd.text == full || wd.tip.iter().any(|l| l.0.contains(full)), "scale {scale}: {full:?} cut on {} without its full text on hover", wd.id);
            }
        }
    }

    /// TV7: the battle's view and the Gallery's Battle timeline build the same tracks for the same session (one
    /// builder), planning and during playback; the battle's view alone gives every track its portrait square
    /// (PORTRAITFX1-TL).
    #[test]
    fn the_battle_view_and_the_gallery_build_the_same_timeline() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.tv_open = true;
        for _ in 0..2 {
            drawn(&mut cx, &mut g, &mut b);
            let sess = s(&b);
            let gallery = crate::gallery::battle_timeline(&cx, sess, sess.current_round, false, None, b.selected.map(track_key));
            let mut shown = view(&b).clone();
            for t in shown.sections.iter_mut().flat_map(|sec| sec.tracks.iter_mut()) {
                assert!(t.icon, "{}: the battle's view has the portrait square", t.key);
                t.icon = false;
            }
            assert_eq!(&shown, &gallery);
            b.act(&mut cx, Action::Confirm);
            b.send(&cx, json!({"cmd": "step"}));
        }
    }
}

/// INPUT1-PB (INPUT1.md 12, PB1-PB5): the playback aids of a timeline battle (the first catalog encounter played by the
/// timeline ruleset): the impact tracers, the strip's block cursor, the playback speed, the log by round and block and
/// the timeline view pausing the playback. Every expectation comes from the rules and the kit (the popped commands,
/// `Session::log_groups`, the tuning's own pacing), never a shipped number; classic is unchanged.
#[cfg(test)]
mod playback_tests {
    use super::*;
    use emberwake_rules::ruleset::TIMELINE;

    fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        b.tick(&mut cx);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn s(b: &BattleScreen) -> &Session {
        b.host.session.as_ref().unwrap()
    }

    fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a Widget<Action>> {
        b.w.iter().find(|x| x.id == id)
    }

    /// One playback tick as soon as it may go: a pending impact fired, the step pacing and the impact gap over.
    fn tick_now(b: &mut BattleScreen, cx: &mut Cx) {
        b.flush(cx);
        b.next_at = Instant::now();
        b.gap_until = None;
        b.tick(cx);
    }

    /// The tile of unit `u` as the tracers use it.
    fn tile(b: &BattleScreen, u: usize) -> Value {
        let c = s(b).w.grid.pos(u).or(b.tokens.anims.get(u).and_then(|a| a.seen.pos)).expect("a tile");
        json!([c.x, c.y])
    }

    /// Plays on to the next round (planned with the party tactic) once a round is over; false when the battle is.
    fn next_round(b: &mut BattleScreen, cx: &mut Cx) -> bool {
        if s(b).combat_over || !s(b).can_advance_round() {
            return false;
        }
        b.run(cx, Action::NextRound);
        b.act(cx, Action::Confirm);
        s(b).plan_confirmed
    }

    /// PB1: a step whose attack landed puts one tracer from the actor's tile to each unit it hit or staggered (solid,
    /// alpha 200 at its impact, the actor's side colour); a self-only command (Defend, Focus, Rest) puts none. Skip
    /// animations: the tracers draw as the step's result shows.
    #[test]
    fn a_landed_hit_puts_one_tracer_per_unit_and_a_self_command_none() {
        let (mut cx, _, mut b) = arena(Some(TIMELINE));
        cx.settings.v["skip_anim"] = json!(true);
        b.act(&mut cx, Action::Confirm);
        let (mut hits, mut selfs) = (0, 0);
        for _ in 0..600 {
            if s(&b).phase == Phase::Replay && ((hits > 0 && selfs > 0) || !next_round(&mut b, &mut cx)) {
                break;
            }
            b.tracers.clear();
            let before = s(&b).last_popped;
            b.send(&cx, json!({"cmd": "step"}));
            let sess = s(&b);
            let Some((_, _, idx)) = sess.last_popped.filter(|p| Some(*p) != before) else { continue };
            let a = sess.resolver.scheduled[idx].clone();
            let tr = b.state(&cx)["overlay"]["tracers"].as_array().cloned().expect("overlay.tracers");
            match a.command.kind {
                CmdKind::Defend { .. } | CmdKind::Focus { .. } | CmdKind::Rest { .. } => {
                    assert!(tr.is_empty(), "a self command: {tr:?}");
                    selfs += 1;
                }
                CmdKind::Attack { .. } if !a.cancelled && a.command.reactions.is_empty() && !a.command.hits.is_empty() => {
                    let mut landed: Vec<usize> = a.command.hits.iter().map(|h| h.0).chain(a.command.staggers.iter().map(|st| st.target)).filter(|&u| u != a.unit).collect();
                    landed.sort_unstable();
                    landed.dedup();
                    assert_eq!(tr.len(), landed.len(), "one tracer per unit it landed on: {tr:?}");
                    for u in landed {
                        assert!(tr.contains(&json!({"from": tile(&b, a.unit), "to": tile(&b, u), "alpha": TRACER_ALPHA, "dashed": false})), "{u}: {tr:?}");
                    }
                    let col = if sess.w.allies.contains(&a.unit) { GOLD } else { FOE };
                    assert!(b.tracers.iter().all(|t| t.0.col == col && !t.0.dashed));
                    hits += 1;
                }
                _ => {}
            }
        }
        assert!(hits > 0 && selfs > 0, "an attack landed ({hits}) and a self command went ({selfs})");
    }

    /// PB1 by the rules on a played command: a protected flank (MR8 c) or a lost target (GD5) is one dashed tracer to
    /// the target; a cancelled action none; a support on another unit draws in the heal colour, on itself none.
    #[test]
    fn a_miss_is_one_dashed_tracer_and_a_cancelled_action_none() {
        let (mut cx, _, mut b) = arena(Some(TIMELINE));
        cx.settings.v["skip_anim"] = json!(true);
        b.act(&mut cx, Action::Confirm);
        while s(&b).phase != Phase::Replay {
            b.send(&cx, json!({"cmd": "step"}));
        }
        let heal = cx.tuning.value.heal_color;
        let sess = b.host.session.as_mut().unwrap();
        let idx = sess.resolver.scheduled.iter().position(|a| matches!(a.command.kind, CmdKind::Attack { target: Some(_), .. })).expect("an attack");
        let CmdKind::Attack { target: Some(t), .. } = sess.resolver.scheduled[idx].command.kind else { unreachable!() };
        let actor = sess.resolver.scheduled[idx].unit;
        let at = |u: usize| Some(V2::new(u as i32, 0));
        let c = &mut sess.resolver.scheduled[idx].command;
        (c.missed, c.evaded) = (true, None);
        c.hits.clear();
        c.staggers.clear();
        c.reactions.clear();
        let col = if sess.w.allies.contains(&actor) { GOLD } else { FOE };
        let want = vec![fx::Trace { from: V2::new(actor as i32, 0), to: V2::new(t as i32, 0), col, dashed: true }];
        assert_eq!(traces_of(sess, idx, &at, heal), want, "MR8 c: a protected flank");
        let c = &mut sess.resolver.scheduled[idx].command;
        (c.missed, c.evaded) = (false, Some(t));
        assert_eq!(traces_of(sess, idx, &at, heal), want, "GD5: a lost target");
        sess.resolver.scheduled[idx].cancelled = true;
        assert!(traces_of(sess, idx, &at, heal).is_empty(), "a cancelled action");
        sess.resolver.scheduled[idx].cancelled = false;
        let other = (actor + 1) % sess.w.units.len();
        let skill = sess.w.cat.skills.iter().position(|k| emberwake_battle::skill::kind(k) == Kind::Support).unwrap_or(0);
        for (target, want) in [(other, vec![fx::Trace { from: V2::new(actor as i32, 0), to: V2::new(other as i32, 0), col: heal, dashed: false }]), (actor, Vec::new())] {
            let c = &mut sess.resolver.scheduled[idx].command;
            (c.evaded, c.failed) = (None, false);
            c.kind = CmdKind::Support { target, skill, last_heal: 5, last_status: String::new(), last_cleansed: Vec::new() };
            assert_eq!(traces_of(sess, idx, &at, heal), want, "a support on {target}");
        }
    }

    /// PB1: with the group `tracers` off the agent link lists none (and the board draws none); the tracer fades from
    /// alpha 200 to 0 over 400 ms on the FX clock, and under reduce motion holds 200 for 400 ms, then is gone.
    #[test]
    fn the_group_off_lists_no_tracer_and_a_tracer_fades_or_holds() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        let now = b.clock.now();
        b.tracers.push((fx::Trace { from: V2::new(1, 1), to: V2::new(3, 1), col: GOLD, dashed: false }, now));
        assert_eq!(b.state(&cx)["overlay"]["tracers"].as_array().map(Vec::len), Some(1));
        cx.settings.v["battle_overlays"]["tracers"] = json!(false);
        assert_eq!(b.state(&cx)["overlay"]["tracers"], json!([]), "the group off");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!((tracer_alpha(0, false), tracer_alpha(TRACER_MS / 2, false), tracer_alpha(TRACER_MS, false)), (TRACER_ALPHA, TRACER_ALPHA / 2, 0));
        assert_eq!((tracer_alpha(0, true), tracer_alpha(TRACER_MS - 1, true), tracer_alpha(TRACER_MS, true)), (TRACER_ALPHA, TRACER_ALPHA, 0), "reduce motion: no fade");
        // classic: no tracer, no playback state
        let (mut cx, _, mut b) = arena(None);
        cx.settings.v["skip_anim"] = json!(true);
        b.act(&mut cx, Action::Confirm);
        for _ in 0..40 {
            b.send(&cx, json!({"cmd": "step"}));
        }
        let st = b.state(&cx);
        assert!(b.tracers.is_empty() && st.get("playback").is_none() && st.get("log_groups").is_none(), "classic is unchanged");
    }

    /// PB1: with animations on a step's tracers wait in its impact and draw from the impact (rule I's moment).
    #[test]
    fn the_tracers_wait_for_the_impact() {
        let (mut cx, _, mut b) = arena(Some(TIMELINE));
        b.act(&mut cx, Action::Confirm);
        let mut seen = false;
        for _ in 0..300 {
            if s(&b).phase == Phase::Replay && !next_round(&mut b, &mut cx) {
                break;
            }
            b.flush(&cx);
            b.tracers.clear();
            b.send(&cx, json!({"cmd": "step"}));
            let Some(i) = b.impacts.first().filter(|i| !i.traces.is_empty()) else { continue };
            let traces = i.traces.clone();
            assert!(b.tracers.is_empty(), "nothing before the impact");
            b.flush(&cx);
            assert_eq!(b.tracers.iter().map(|t| t.0).collect::<Vec<_>>(), traces, "the impact's tracers");
            seen = true;
            break;
        }
        assert!(seen, "an action with tracers had an impact");
    }

    /// PB2: the strip frames (its block widget active) and `playback.block` names the block the last step resolved,
    /// from its first step until the next block's; none at the round's start (Confirm, no step yet) and once the round
    /// is over and its last impact has played.
    #[test]
    fn the_strip_cursor_follows_the_resolving_block() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        cx.settings.v["skip_anim"] = json!(true);
        b.act(&mut cx, Action::Confirm);
        assert_eq!(b.state(&cx)["playback"]["block"], Value::Null, "the round's start");
        let mut seen = Vec::new();
        while s(&b).phase != Phase::Replay {
            let resolving = if s(&b).phase == Phase::Planning { 1 } else { s(&b).current_block };
            b.send(&cx, json!({"cmd": "step"}));
            if s(&b).phase == Phase::Replay {
                break;
            }
            assert_eq!(b.state(&cx)["playback"]["block"], json!(resolving));
            b.draw(&mut cx, &mut g, (-500, -500), true);
            for blk in 1..=s(&b).def.total_blocks {
                assert_eq!(widget(&b, &format!("strip.block{blk}")).unwrap().active, blk == resolving, "block {blk} while {resolving} resolves");
            }
            seen.push(resolving);
        }
        assert!(seen.windows(2).all(|w| w[0] <= w[1]) && seen.first() == Some(&1), "{seen:?}");
        b.gap_until = None;
        assert_eq!(b.state(&cx)["playback"]["block"], Value::Null, "the round's end");
    }

    /// PB3: P cycles 1x, 0.5x, Block and the setting is kept; during playback the chips take the tactic menus' place
    /// (Next block in Block mode only), while planning the menus are back; classic: no chip and P changes nothing.
    #[test]
    fn p_cycles_the_speeds_and_the_chips_show_during_playback() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        assert_eq!(cx.settings.v[PLAYBACK_SPEED], json!("1x"), "the default");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(widget(&b, "strip.party").is_some() && widget(&b, "playback.speed.1").is_none(), "planning: the menus");
        let mut seen = Vec::new();
        for _ in 0..3 {
            b.key(&mut cx, Key::Char('p'));
            seen.push(cx.settings.v[PLAYBACK_SPEED].as_str().unwrap().to_string());
        }
        assert_eq!(seen, ["0.5x", "block", "1x"]);
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(cx.shot_dir.join("settings.json")).expect("saved")).unwrap();
        assert_eq!(saved[PLAYBACK_SPEED], json!("1x"), "remembered");
        b.act(&mut cx, Action::Confirm);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(widget(&b, "strip.party").is_none() && widget(&b, "strip.hero").is_none());
        for (k, (v, _, id)) in PLAYBACK_SPEEDS.iter().enumerate() {
            b.run(&mut cx, Action::PlaybackSpeed(k));
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert!(PLAYBACK_SPEEDS.iter().all(|c| widget(&b, c.2).is_some_and(|w| w.active == (c.2 == *id))), "{v} on");
            assert_eq!(widget(&b, "playback.next").is_some(), *v == "block", "{v}: Next block");
            assert_eq!(b.state(&cx)["playback"]["speed"], json!(v));
        }
        let (mut cx, mut g, mut b) = arena(None);
        b.key(&mut cx, Key::Char('p'));
        b.act(&mut cx, Action::Confirm);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(cx.settings.v[PLAYBACK_SPEED], json!("1x"));
        assert!(widget(&b, "playback.speed.1").is_none() && widget(&b, "strip.party").is_some(), "classic keeps its menus");
    }

    /// PB3: during playback the speed chips (and Next block) sit in the tactic menus' place, left of Auto, and their
    /// labels fit at the four text sizes (no cut, spilled, overlapping or off-canvas label).
    #[test]
    fn the_speed_chips_fit_at_every_text_size() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        cx.settings.v[PLAYBACK_SPEED] = json!("block");
        b.act(&mut cx, Action::Confirm);
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        let labels = ["1x", "0.5x", "Block", "Next block"];
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
            let auto = widget(&b, "strip.auto").expect("Auto").rect;
            let chips: Vec<Rect> = ["playback.speed.1", "playback.speed.half", "playback.speed.block", "playback.next"].iter().map(|id| widget(&b, id).unwrap_or_else(|| panic!("{id}")).rect).collect();
            assert!(chips.windows(2).all(|c| c[0].x + c[0].w <= c[1].x) && chips.last().is_some_and(|c| c.x + c.w < auto.x), "scale {scale}: {chips:?} left of Auto {auto:?}");
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let named = |e: &Value| e["widget"].as_str().is_some_and(|w| w.starts_with("playback.")) || ["text", "shown", "a", "b"].iter().any(|k| e[*k].as_str().is_some_and(|x| labels.contains(&x)));
            for kind in ["cut", "edge", "overlap", "spill"] {
                let hits: Vec<&Value> = v[kind].as_array().unwrap().iter().filter(|e| named(e)).collect();
                assert!(hits.is_empty(), "scale {scale}: {kind} {hits:?}");
            }
        }
    }

    /// PB3 (timed): at 0.5x a block's playback takes twice as long: the wait after a step, the floats, and the FX clock
    /// (walks, clips, tracers) runs at half the animation speed; while planning and under skip animations it plays as
    /// 1x.
    #[test]
    fn half_speed_doubles_the_playbacks_waits_and_clock() {
        let measure = |speed: &str| {
            let (mut cx, _, mut b) = arena(Some(TIMELINE));
            cx.settings.v[PLAYBACK_SPEED] = json!(speed);
            let planning_pace = b.pace(&cx);
            b.act(&mut cx, Action::Confirm);
            // the FX clock over 80 wall ms (no step goes: the next one is far off)
            b.next_at = Instant::now() + Duration::from_secs(60);
            let ms0 = b.clock.ms;
            let t0 = Instant::now();
            b.clock_at = t0 - Duration::from_millis(80);
            b.tick(&mut cx);
            let clock = (b.clock.ms - ms0, t0.elapsed().as_secs_f64() * 1000.0);
            // the wait after a step
            b.gap_until = None;
            b.impacts.clear();
            let t1 = Instant::now();
            b.next_at = t1;
            b.tick(&mut cx);
            let wait = (b.next_at.duration_since(t1).as_millis() as u64, t1.elapsed().as_millis() as u64);
            b.show(&cx, vec![(0, "-1".into(), FOE)], Vec::new(), Vec::new());
            let float = b.floats.last().unwrap().ms;
            cx.settings.v["skip_anim"] = json!(true);
            (planning_pace, clock, wait, float, b.pace(&cx), cx.tuning.value.clone())
        };
        let (p1, c1, w1, f1, skip1, t) = measure("1x");
        let (p2, c2, w2, f2, skip2, _) = measure("0.5x");
        assert_eq!((p1, p2, skip1, skip2), (1, 1, 1, 1), "planning and skip animations play as 1x");
        assert!(c1.0 >= 79.5 && c1.0 <= 80.0 + c1.1 + 0.5, "1x clock {c1:?}");
        assert!(c2.0 >= 39.5 && c2.0 <= (80.0 + c2.1) / 2.0 + 0.5, "0.5x clock {c2:?}");
        assert!(w1.0 + w1.1 >= t.step_ms && w1.0 <= t.step_ms, "1x step wait {w1:?}");
        assert!(w2.0 + w2.1 >= 2 * t.step_ms && w2.0 <= 2 * t.step_ms, "0.5x step wait {w2:?}");
        assert_eq!((f1, f2), (t.float_ms, 2 * t.float_ms), "the floats");
    }

    /// PB3: Block mode waits after each block (the message bar "Block <b> of <n>: N for the next", nothing sent while it
    /// waits), N plays the next one; skip animations still waits; autoplay waits after the round's last block too and
    /// N goes on to the next round.
    #[test]
    fn block_mode_waits_after_each_block_and_n_plays_the_next() {
        let (mut cx, _, mut b) = arena(Some(TIMELINE));
        cx.settings.v["skip_anim"] = json!(true);
        cx.settings.v[PLAYBACK_SPEED] = json!("block");
        b.act(&mut cx, Action::Autoplay);
        assert!(b.auto && s(&b).plan_confirmed, "{}", b.message);
        let n = s(&b).def.total_blocks;
        let mut waits = Vec::new();
        for _ in 0..800 {
            if s(&b).current_round > 1 || s(&b).combat_over {
                break;
            }
            tick_now(&mut b, &mut cx);
            let st = b.state(&cx)["playback"].clone();
            if st["waiting"] != json!(true) {
                continue;
            }
            assert_eq!(st["speed"], json!("block"));
            let blk = st["block"].as_i64().expect("the block") as i32;
            assert_eq!(b.message, format!("Block {blk} of {n}: N for the next"));
            let sent = b.host.history.len();
            tick_now(&mut b, &mut cx);
            assert_eq!(b.host.history.len(), sent, "nothing goes while it waits");
            waits.push(blk);
            b.key(&mut cx, Key::Char('n'));
            assert_eq!(b.state(&cx)["playback"]["waiting"], json!(false), "N lets it go on");
        }
        assert!(!waits.is_empty(), "it waited");
        assert_eq!(waits, (1..=waits.len() as i32).collect::<Vec<_>>(), "each block in turn");
        if !s(&b).combat_over {
            assert_eq!(waits.len() as i32, n, "the round's last block waits too under autoplay");
            assert_eq!(s(&b).current_round, 2, "N went on to the next round");
        }
    }

    /// PB5: T during playback pauses it at the end of the resolving block (`playback.waiting`, nothing sent); closing the
    /// view clears it and the playback goes on; in Block mode N steps the next block while the view stays open, its
    /// playhead following.
    #[test]
    fn t_pauses_the_playback_at_the_block_end_and_closing_resumes() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        cx.settings.v["skip_anim"] = json!(true);
        b.act(&mut cx, Action::Confirm);
        b.key(&mut cx, Key::Char('t'));
        assert!(b.tv_open);
        let mut paused = false;
        for _ in 0..200 {
            tick_now(&mut b, &mut cx);
            if b.state(&cx)["playback"]["waiting"] == json!(true) {
                paused = true;
                break;
            }
        }
        assert!(paused && s(&b).phase == Phase::Resolving, "paused at a block's end: {}", b.message);
        assert!(b.message.contains("timeline view"), "{}", b.message);
        let sent = b.host.history.len();
        tick_now(&mut b, &mut cx);
        assert_eq!(b.host.history.len(), sent, "held while open");
        b.key(&mut cx, Key::Char('t'));
        assert!(!b.tv_open && b.state(&cx)["playback"]["waiting"] == json!(false), "closing clears it");
        tick_now(&mut b, &mut cx);
        assert!(b.host.history.len() > sent, "and the playback goes on");
        // Block mode: N plays the next block with the view open
        cx.settings.v[PLAYBACK_SPEED] = json!("block");
        b.key(&mut cx, Key::Char('t'));
        let mut steps = 0;
        for _ in 0..200 {
            if s(&b).phase == Phase::Replay {
                break;
            }
            tick_now(&mut b, &mut cx);
            if b.state(&cx)["playback"]["waiting"] == json!(true) {
                let blk = b.state(&cx)["playback"]["block"].clone();
                b.draw(&mut cx, &mut g, (-500, -500), true);
                assert_eq!(json!(b.tv_shown.as_ref().expect("open").playhead), blk, "the playhead follows");
                b.key(&mut cx, Key::Char('n'));
                assert!(b.tv_open, "the view stays open");
                steps += 1;
            }
        }
        assert!(steps > 0, "N stepped with the view open");
    }

    /// PB4: the log by round and block: the agent link's `log_groups` is the kit's parts (their lines read in order are
    /// the log); the expanded log draws "Round <r>", then each part's header ("Round start", "Block <b>", "Round end")
    /// above its lines, 6 px in; during playback the compact log shows the resolving block's part; every header and
    /// line passes the text audit at the four text sizes.
    #[test]
    fn the_log_groups_lines_under_their_round_and_block() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
        cx.settings.v["skip_anim"] = json!(true);
        b.act(&mut cx, Action::Confirm);
        let mut compact = 0;
        while s(&b).phase != Phase::Replay {
            b.send(&cx, json!({"cmd": "step"}));
            if let Some(blk) = b.cursor_block(Instant::now()) {
                b.draw(&mut cx, &mut g, (-500, -500), true);
                let w = widget(&b, "bar.last").expect("the compact log");
                assert!(w.text.starts_with(&format!("Block {blk}: ")), "{}", w.text);
                assert_eq!(w.tip[0].0, format!("Round {} block {blk}", s(&b).current_round));
                compact += 1;
            }
        }
        assert!(compact > 0);
        let groups = s(&b).log_groups();
        assert_eq!(b.state(&cx)["log_groups"], log_groups_json(&groups));
        assert_eq!(groups.iter().flat_map(|p| p.lines.clone()).collect::<Vec<_>>(), s(&b).log_lines(0, 0), "every line once, in order");
        assert!(groups.iter().any(|p| p.block >= 1), "a block's part");
        g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
        g.view = aegis_ui::painter::View::fit(1280, 720);
        b.show_log = true;
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
            let panel = widget(&b, "log.panel").expect("the log").rect;
            let rows: Vec<&aegis_ui::painter::TextRec> = recs.iter().filter(|r| r.y > panel.y as f32 + 12.0 && r.y < (panel.y + panel.h) as f32 && r.x >= panel.x as f32 && r.x < (panel.x + panel.w) as f32).collect();
            assert!(!rows.is_empty(), "scale {scale}: rows drawn");
            // each line drawn sits below the header of the part it belongs to, 6 px further in
            let mut part: Option<&emberwake_battle::session::LogGroup> = None;
            let mut header_x = 0.0;
            for r in &rows {
                if let Some(p) = groups.iter().find(|p| log_part_name(p.block) == r.text) {
                    part = Some(p);
                    header_x = r.x;
                } else if r.text.starts_with("Round ") {
                    part = None;
                } else if let Some(p) = part {
                    let t = r.text.trim_start();
                    assert!(p.lines.iter().any(|l| l.contains(t)), "scale {scale}: {:?} under {}", r.text, log_part_name(p.block));
                    assert!(r.x >= header_x + LOG_INDENT - 0.6, "scale {scale}: {:?} 6 px in", r.text);
                }
            }
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
            let named = |e: &Value| ["text", "shown", "a", "b"].iter().any(|k| e[*k].as_str().is_some_and(|x| texts.contains(&x)));
            // (a spill is only the log's own: the panel lies over the board's tile widgets)
            let ours = |e: &Value| e["widget"].as_str().is_none_or(|w| w.starts_with("log.") || w.starts_with("bar."));
            for kind in ["cut", "edge", "overlap", "spill"] {
                let hits: Vec<&Value> = v[kind].as_array().unwrap().iter().filter(|e| named(e) && (kind != "spill" || ours(e))).collect();
                assert!(hits.is_empty(), "scale {scale}: {kind} {hits:?}");
            }
        }
    }
}

/// ENCOUNTER2-C4 (docs/specs/ENCOUNTER2_C.md section 8, the C4 row): the cutscene screen on a scratch catalog (no
/// shipped content): a locked round refuses the planning controls, shows its banner and Skip, plays by itself and
/// starts the next round after the round-end pause; Skip sends `resolve`; a guided round shows the banner and keeps
/// planning; the party tactic reaches the script's auto heroes (decision "ENCOUNTER2-C3 answers" (1)).
#[cfg(test)]
mod script_screen_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use emberwake_world::Data;
    use std::sync::Arc;

    const BANNER: &str = "Watch them";

    /// The scratch catalog: two heroes and a foe under a timeline ruleset; `t_locked` scripts round 1 locked,
    /// `t_guided` the same round guided; round 2 is not scripted.
    fn data() -> Data {
        let skills = json!([
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5},
            {"id": "t_cleave", "name": "Cleave", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 1.5, "target_mode": "ENEMY"}
        ]);
        let unit = |id: &str, side: &str| {
            json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5, "deck": {"order": {"assault": ["t_cleave", "strike"], "guard": ["active_guard"]}}})
        };
        let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost"], "basics": ["move", "rest", "strike", "active_guard"]});
        let encounter = |id: &str, lock: bool| {
            let script = json!([{"round": 1, "lock": lock, "others": "auto", "banner": BANNER, "entries": [
                {"unit": "party:t_hero", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "rest"},
                {"unit": "t_foe", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "auto"}
            ]}]);
            json!({"id": id, "name": id, "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "ap_regen": 4, "max_rounds": 3, "script": script})
        };
        let catalog = json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")], "rulesets": [ruleset],
            "encounters": [encounter("t_locked", true), encounter("t_guided", false), json!({"id": "t_plain", "name": "t_plain", "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "max_rounds": 3})]});
        Data::from_values(&catalog, &json!({}), &json!({}))
    }

    /// A debug game over the scratch catalog (skip animations on, so no impact holds a step).
    fn script_cx() -> Cx {
        let mut cx = game_cx();
        cx.data = Arc::new(data());
        cx.settings.v["battle_view"] = json!("flat");
        cx.settings.v["skip_anim"] = json!(true);
        cx
    }

    fn arena(cx: &Cx, encounter: &str) -> BattleScreen {
        let b = BattleScreen::arena_with(cx, None, encounter, Some("timeline".into()), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        b
    }

    fn tick_ready(b: &mut BattleScreen, cx: &mut Cx) {
        b.next_at = Instant::now();
        b.gap_until = None;
        b.tick(cx);
    }

    fn widget(b: &BattleScreen, id: &str) -> Option<Value> {
        b.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn cmds(b: &BattleScreen) -> Vec<String> {
        b.host.history.iter().map(|r| r["cmd"].as_str().unwrap_or("").to_string()).collect()
    }

    fn intent_of(b: &mut BattleScreen, name: &str) -> Value {
        let plan = b.host.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone();
        json::arr(&plan).iter().find(|r| r["name"] == json!(name)).map(|r| r["intent"].clone()).unwrap_or(Value::Null)
    }

    /// SR5 on the screen: the planning controls send nothing; the banner and Skip show, Confirm and the tactic menus do
    /// not act; the agent link reads the lock.
    #[test]
    fn a_locked_round_refuses_planning_and_shows_its_banner_and_skip() {
        let (mut cx, mut g) = (script_cx(), gfx());
        let mut b = arena(&cx, "t_locked");
        assert!(b.locked() && !b.planning(), "the script confirmed round 1");
        frame(&mut b, &mut cx, &mut g, 1.0);
        assert_eq!(widget(&b, "battle.banner").map(|w| w["text"].clone()), Some(json!(BANNER)));
        assert!(widget(&b, "battle.skip").is_some_and(|w| w["enabled"] == json!(true)), "Skip in the Confirm slot");
        assert!(widget(&b, "strip.confirm").is_none());
        for id in ["strip.party", "strip.hero", "strip.clear"] {
            assert!(widget(&b, id).is_none_or(|w| w["enabled"] == json!(false)), "{id} is locked");
        }
        let st = b.state(&cx);
        assert_eq!(st["script"], json!({"round": 1, "lock": true, "banner": BANNER, "skip": true}));
        let sent = b.host.history.len();
        let hero = b.session().w.allies[0];
        for a in [Action::Confirm, Action::Clear, Action::Undo, Action::SelfTarget, Action::TileClick(V2::new(3, 3))] {
            b.act(&mut cx, a);
        }
        b.selected = Some(hero);
        for k in [Key::Enter, Key::Char('1'), Key::Char('s'), Key::Char('z')] {
            b.key(&mut cx, k);
        }
        assert_eq!(b.host.history.len(), sent, "no planning request goes out: {:?}", cmds(&b));
    }

    /// The locked round plays by itself from the first tick and, at its end, the next round starts after the
    /// round-end pause (without autoplay); round 2 is not scripted, so planning is back and the script key is gone.
    #[test]
    fn a_locked_round_plays_by_itself_and_advances() {
        let mut cx = script_cx();
        let mut b = arena(&cx, "t_locked");
        assert!(!b.auto);
        b.tick(&mut cx);
        assert!(b.playing, "the playback starts by itself");
        for _ in 0..200 {
            if b.session().phase == Phase::Replay {
                break;
            }
            tick_ready(&mut b, &mut cx);
        }
        assert_eq!(b.session().phase, Phase::Replay);
        assert!(cmds(&b).iter().any(|c| c == "step"), "{:?}", cmds(&b));
        assert!(b.pending_next_round, "the next round is due after the round-end pause");
        tick_ready(&mut b, &mut cx);
        assert_eq!(b.session().current_round, 2);
        assert!(b.planning() && !b.locked());
        assert!(b.state(&cx).get("script").is_none());
        tick_ready(&mut b, &mut cx);
        assert!(!b.playing && b.planning(), "an unscripted round waits for the player");
    }

    /// Skip sends `resolve`: the round ends at once with the same log as resolving it through the host, and the next
    /// round follows after the pause.
    #[test]
    fn skip_resolves_the_rest_of_the_locked_round() {
        let mut cx = script_cx();
        let mut b = arena(&cx, "t_locked");
        b.tick(&mut cx);
        b.act(&mut cx, Action::Skip);
        assert_eq!(cmds(&b).last().map(String::as_str), Some("resolve"));
        assert_eq!(b.session().phase, Phase::Replay);
        assert!(!b.playing && b.pending_next_round);
        let mut direct = emberwake_host::Host::new(emberwake_host::Battle::with_data(cx.data.clone()));
        assert_eq!(direct.handle(&b.host.history[0].clone())["ok"], json!(true));
        direct.handle(&json!({"cmd": "resolve"}));
        assert_eq!(direct.handle(&json!({"cmd": "log"}))["log"], b.host.handle(&json!({"cmd": "log"}))["log"], "the same round as resolving it at once");
        b.act(&mut cx, Action::Skip);
        assert_eq!(cmds(&b).iter().filter(|c| *c == "resolve").count(), 1, "nothing to skip once the round is over");
        tick_ready(&mut b, &mut cx);
        assert_eq!(b.session().current_round, 2);
    }

    /// SR6 on the screen: a guided round shows the banner but keeps planning (Confirm, the tactic menus, no Skip, no
    /// playback of its own); the party tactic sends the tactics to the kit.
    #[test]
    fn a_guided_round_shows_the_banner_and_keeps_planning() {
        let (mut cx, mut g) = (script_cx(), gfx());
        let mut b = arena(&cx, "t_guided");
        assert!(!b.locked() && b.planning());
        b.tick(&mut cx);
        assert!(!b.playing, "a guided round waits for Confirm");
        frame(&mut b, &mut cx, &mut g, 1.0);
        assert_eq!(widget(&b, "battle.banner").map(|w| w["text"].clone()), Some(json!(BANNER)));
        assert!(widget(&b, "battle.skip").is_none());
        assert!(widget(&b, "strip.confirm").is_some_and(|w| w["enabled"] == json!(true)));
        assert!(widget(&b, "strip.party").is_some_and(|w| w["enabled"] == json!(true)));
        assert_eq!(b.state(&cx)["script"], json!({"round": 1, "lock": false, "banner": BANNER, "skip": false}));
        b.act(&mut cx, Action::PartyMenu);
        let row = b.popup.as_ref().and_then(|p| p.rows.iter().position(|r| matches!(r.act, RowAct::PartyTactic(3)))).expect("the Guard and Counter row");
        b.act(&mut cx, Action::Choose(row));
        let last = b.host.history.last().cloned().unwrap_or(Value::Null);
        assert_eq!((last["cmd"].clone(), last["party"].clone()), (json!("tactics"), json!(TACTICS[3])));
        b.act(&mut cx, Action::Confirm);
        assert!(b.playing, "Confirm plays the guided round");
    }

    /// Decision "ENCOUNTER2-C3 answers" (1): the Arena's party tactic drafts the locked round's hero with no entry
    /// (Guard and Counter: Guard on itself), where the kit's default drafts by the AI table (no Guard at full HP).
    #[test]
    fn the_party_tactic_drafts_the_scripted_rounds_auto_heroes() {
        let cx = script_cx();
        let mut plain = arena(&cx, "t_locked");
        assert_ne!(intent_of(&mut plain, "t_mate"), json!("guard"));
        let mut b = BattleScreen::blank(&cx, Mode::Arena);
        b.encounter = "t_locked".into();
        b.arena_ruleset = Some("timeline".into());
        b.party_tactic = 3;
        b.start_battle();
        assert_eq!(b.host.history[0]["tactics"], json!({"party": TACTICS[3]}));
        assert_eq!(intent_of(&mut b, "t_mate"), json!("guard"));
        // a battle with no script, or a script under classic, starts with the request it always had
        for (enc, ruleset) in [("t_plain", Some("timeline")), ("t_locked", None)] {
            b.encounter = enc.into();
            b.arena_ruleset = ruleset.map(String::from);
            b.start_battle();
            assert!(b.host.history[0].get("tactics").is_none(), "{enc} {ruleset:?}");
        }
    }
}

/// PORTRAITFX1-P (docs/specs/PORTRAITFX1.md 6.2 row 3, PORTRAITFX1_TASKS.md P1-P5): the hero and intel panels'
/// portraits, their state holds and the step events at FX1's impacts. Cue rows are this module's fixtures (the shipped
/// table's shape), portraits scratch layers where pixels are read.
#[cfg(test)]
mod portrait_panel_tests {
    use super::*;
    use crate::portraits::tests::{layer, row};
    use aegis_core::assets::Layers;
    use aegis_ui::portrait_fx::{PortraitOpts, EASE_MS};
    use emberwake_data::PortraitCue;
    use emberwake_rules::ruleset::TIMELINE;

    /// The fixture cue rows: the reactions of 5.3 and the holds P3 names (stealth, low HP, down).
    fn rows() -> Vec<PortraitCue> {
        vec![
            row(json!({"id": "hit_flash", "event": "hit", "intensity": 0.6, "ms": 140, "color": "#d9cbb4"})),
            row(json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"})),
            row(json!({"id": "crit_flash", "event": "crit", "intensity": 0.9, "ms": 180, "color": "#e11d48"})),
            row(json!({"id": "crit_jolt", "event": "crit", "effect": "jolt", "preset": "crit"})),
            row(json!({"id": "guard_flash", "event": "guard_hit", "intensity": 0.4, "ms": 120, "color": "#fbbf24"})),
            row(json!({"id": "guard_jolt", "event": "guard_hit", "effect": "jolt", "intensity": 0.5, "preset": "hit"})),
            row(json!({"id": "heal_glow", "event": "heal", "intensity": 0.3, "ms": 420, "color": "#fbbf24"})),
            row(json!({"id": "stealthed", "trigger": "status", "status": "stealth", "effect": "fade", "intensity": 0.6})),
            row(json!({"id": "hp_low", "trigger": "hp", "below": 25, "effect": "desaturate", "intensity": 0.5})),
            row(json!({"id": "downed", "trigger": "down", "effect": "desaturate"})),
            row(json!({"id": "downed_still", "trigger": "down", "effect": "still"})),
        ]
    }

    /// The game's cx with the fixture rows and the repo's art.
    fn cx_with(rows: Vec<PortraitCue>) -> Cx {
        let mut cx = crate::hud::hud_tests::game_cx();
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows);
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx
    }

    /// The ambush (FX1's impact tests' battle) with the fixture rows.
    fn ambush() -> (Cx, Gfx, BattleScreen) {
        let mut cx = cx_with(rows());
        cx.app.encounter = "goblin_ambush".into();
        cx.app.seed = Some(1);
        cx.settings.v["battle_view"] = json!("stage");
        let b = BattleScreen::sandbox(&cx);
        assert!(b.host.session.is_some(), "the ambush starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    /// An Arena battle of the first catalog encounter under `ruleset` (None: classic), ticked and drawn, a hero selected.
    fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
        let mut cx = cx_with(rows());
        cx.settings.v["battle_view"] = json!("flat");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        let mut g = crate::hud::hud_tests::gfx();
        b.tick(&mut cx);
        if b.selected.is_none() {
            b.selected = b.session().w.allies.first().copied();
        }
        b.draw(&mut cx, &mut g, (-500, -500), true);
        (cx, g, b)
    }

    fn rect_of(b: &BattleScreen, id: &str) -> Rect {
        b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}")).rect
    }

    /// The flash colour (rgb) the fixture rows give `event` on the panel.
    fn flash_of(event: &str) -> [u8; 3] {
        let rows = rows();
        let r = emberwake_battle::portrait::reactions(&rows, &[event], PANEL_SURFACE, "classic").into_iter().find(|r| r.effect == "flash").expect("a flash row").clone();
        let c = aegis_core::assets::hex_colour(&r.color).expect("a colour");
        [c[0], c[1], c[2]]
    }

    /// P1 (6.2 row 3, TEXTFIT1): at the four text sizes, classic and timeline, both panels draw a 32 px portrait at
    /// their top-left (inset 4); every text beside it starts 40 px in, none crosses it, and no text of either panel
    /// spills or leaves the canvas.
    #[test]
    fn p1_the_panels_draw_their_portraits_and_the_lines_beside_start_40_in() {
        for ruleset in [None, Some(TIMELINE)] {
            let (mut cx, mut g, mut b) = arena(ruleset);
            for (_, scale) in aegis_ui::screen::TEXT_SIZES {
                let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale as f32);
                for (panel, portrait) in [("hero", "hero.portrait"), ("intel", "intel.portrait")] {
                    let r = b.panels.iter().find(|p| p.0 == panel).unwrap_or_else(|| panic!("{ruleset:?}: the {panel} panel")).1;
                    let pr = rect_of(&b, portrait);
                    assert_eq!(pr, Rect::new(r.x + PANEL_INSET, r.y + PANEL_INSET, PANEL_PORTRAIT, PANEL_PORTRAIT), "{ruleset:?} {scale}: the {panel} portrait");
                    for t in recs.iter().filter(|t| !t.covered && t.x >= r.x as f32 && t.x < (r.x + r.w) as f32 && t.y >= r.y as f32 && t.y < (r.y + r.h) as f32) {
                        let inside = t.x >= pr.x as f32 && t.x + t.w <= (pr.x + pr.w) as f32 && t.y >= pr.y as f32 && t.y + t.h <= (pr.y + pr.h) as f32;
                        if inside {
                            assert_eq!(t.text.chars().count(), 1, "{ruleset:?} {scale}: only the initial letter inside the portrait: {:?}", t.text);
                            continue;
                        }
                        let rows_beside = t.y < (pr.y + pr.h) as f32 && t.y + t.h > pr.y as f32;
                        if rows_beside {
                            assert!(t.x >= (r.x + PANEL_TEXT_X) as f32 - 0.5, "{ruleset:?} {scale}: {:?} beside the {panel} portrait starts 40 px in (x {})", t.text, t.x);
                        }
                    }
                }
                let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                let ours = |e: &Value| e["widget"].as_str().is_some_and(|w| w.starts_with("hero.") || w.starts_with("intel."));
                let spills: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| ours(e)).collect();
                assert!(spills.is_empty(), "{ruleset:?} {scale}: {spills:?}");
                let panel_rects: Vec<Rect> = b.panels.iter().filter(|p| p.0 == "hero" || p.0 == "intel").map(|p| p.1).collect();
                let in_panels = |e: &Value| {
                    let (x, y) = (e["at"][0].as_f64().unwrap_or(-1.0) as i32, e["at"][1].as_f64().unwrap_or(-1.0) as i32);
                    panel_rects.iter().any(|r| r.contains(x, y))
                };
                let edges: Vec<&Value> = v["edge"].as_array().unwrap().iter().filter(|e| in_panels(e)).collect();
                assert!(edges.is_empty(), "{ruleset:?} {scale}: {edges:?}");
            }
        }
    }

    /// PORTRAITFX2 5.3: the battle screen's first frame fires `battle_start` on every living unit's panel portrait,
    /// once, and `ambush` on the player side's only in an ambush deployment (fixture rows: a flash and a tint).
    #[test]
    fn fx2_the_first_frame_fires_battle_start_once_and_ambush_on_the_player_side() {
        let mut fixture = rows();
        fixture.push(row(json!({"id": "start_flash", "event": "battle_start", "ms": 300, "color": "#fbbf24"})));
        fixture.push(row(json!({"id": "ambush_tint", "event": "ambush", "effect": "tint", "ms": 300, "color": "#e11d48"})));
        for deployment in ["standard", emberwake_battle::portrait::AMBUSH_DEPLOYMENT] {
            let mut cx = cx_with(fixture.clone());
            cx.settings.v["battle_view"] = json!("flat");
            let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
            let mut b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
            assert!(b.host.session.is_some(), "{encounter} starts: {}", b.message);
            // the deployment as the encounter row would set it (only `start_events` reads it after the start)
            b.host.session.as_mut().expect("a session").def.deployment = deployment.into();
            let encounter = format!("{encounter} ({deployment})");
            assert!(b.panel_faces.get(&b.unit_portrait(&cx, 0).0.key).is_none(), "{encounter}: nothing before the first frame");
            let mut g = crate::hud::hud_tests::gfx();
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let now = b.clock.now();
            let s = b.session();
            let ambush = s.def.deployment == "ambush";
            for u in (0..s.w.units.len()).filter(|&u| s.w.units[u].hp > 0 && !s.w.units[u].is_withdrawn) {
                let key = b.unit_portrait(&cx, u).0.key;
                let f = b.panel_faces.get(&key).unwrap_or_else(|| panic!("{encounter}: unit {u}'s portrait reacted"));
                assert_eq!(f.level("flash", now).map(|l| l.start), Some(now), "{encounter}: unit {u}'s battle_start flash");
                let party = s.w.allies.contains(&u);
                assert_eq!(f.level("tint", now).is_some(), ambush && party, "{encounter}: unit {u} (party {party}, ambush {ambush})");
            }
            assert!(!b.portrait_start, "{encounter}: the start events fire once");
        }
    }

    /// P2 (5.3, 6.2 row 3; FX1 rule I): a step's portrait events wait in its impact: before it lands no struck
    /// portrait reacts; at the impact each fires its event's rows from the impact's ms (the flash in its row's colour,
    /// a jolt for a blow). Each event through the binding: `crit` jolts harder than `hit`, `guard_hit` flashes the
    /// guard gold, `heal` glows gold without a jolt.
    #[test]
    fn p2_the_panel_portraits_react_at_the_impact_not_at_the_pop() {
        let (mut cx, _g, mut b) = ambush();
        let found = super::tests::step_until(&mut cx, &mut b, |_, i| !i.portrait.is_empty());
        assert!(found, "the ambush has a step with portrait events at an impact");
        b.resolve_impacts(&mut cx);
        // earlier impacts fired as the steps went (rule F): start the portraits afresh
        b.panel_faces = Faces::default();
        let i = b.impacts[0].clone();
        let at = i.at.expect("resolved");
        let s = b.host.session.as_ref().unwrap();
        for &u in &i.struck {
            let blows = i.portrait.iter().filter(|(v, e)| *v == u && ["hit", "crit", "guard_hit"].contains(e)).count();
            assert_eq!(blows, 1, "{} lost HP: one of hit, crit, guard_hit", s.w.units[u].name);
        }
        // the blows and heals (PORTRAITFX2: a step may carry `ally_down` too, which no flash row answers)
        let keys: Vec<(String, &str)> = i.portrait.iter().filter(|(_, e)| ["hit", "crit", "guard_hit", "heal"].contains(e)).map(|&(u, e)| (b.unit_portrait(&cx, u).0.key, e)).collect();
        assert!(keys.iter().all(|(k, _)| b.panel_faces.get(k).is_none()), "nothing reacts at the pop");
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        assert!(b.impacts.is_empty());
        for (k, ev) in &keys {
            let f = b.panel_faces.get(k).unwrap_or_else(|| panic!("{k} reacted to {ev}"));
            let flash = f.level("flash", at).unwrap_or_else(|| panic!("{k}: a flash for {ev}"));
            assert_eq!((flash.start, flash.color[..3].to_vec()), (at, flash_of(ev).to_vec()), "{k}: {ev}'s flash from the impact");
            assert_eq!(f.jolt_amplitude(at) > 0.0, *ev != "heal", "{k}: {ev}'s jolt");
        }

        // each event through the binding
        let u = b.session().w.allies[0];
        let key = b.unit_portrait(&cx, u).0.key;
        let mut amp = std::collections::BTreeMap::new();
        for ev in ["hit", "crit", "guard_hit", "heal"] {
            b.panel_faces = Faces::default();
            b.react_portraits(&cx, &[(u, ev)], 5000);
            let f = b.panel_faces.get(&key).expect("reacted");
            assert_eq!(f.level("flash", 5000).map(|l| l.color[..3].to_vec()), Some(flash_of(ev).to_vec()), "{ev}");
            amp.insert(ev, f.jolt_amplitude(5000));
        }
        assert!(amp["crit"] > amp["hit"] && amp["hit"] > 0.0 && amp["guard_hit"] > 0.0 && amp["heal"] == 0.0, "{amp:?}");
        assert_eq!(b.panel_faces.clock, b.clock, "the panel clock is the FX clock again after a reaction");
    }

    /// Draws unit `u`'s panel portrait at `t` and past `t + EASE_MS` (its holds eased in) from the scratch file in `art`;
    /// its composed centre pixel.
    fn settled(cx: &Cx, b: &mut BattleScreen, art: &mut Art, g: &mut Gfx, u: usize, t: u64) -> [u8; 4] {
        let rows = rows();
        let opts = PortraitOpts { reduce_motion: true, ..PortraitOpts::default() };
        // the second draw a frame step past the ease (time-varying effects read the 30 a second steps, 4.5)
        for ms in [t, t + EASE_MS + 100] {
            b.panel_faces.clock = Clock { ms: ms as f64, hold: 0.0 };
            let (who, look) = b.unit_portrait(cx, u);
            let mut ui: Ui<Action> = Ui::new(g, (-500, -500), true);
            b.panel_faces.begin();
            assert!(b.panel_faces.draw_parts(&mut ui.p, art, &rows, &who, &look, PANEL_SURFACE, Rect::new(4, 4, PANEL_PORTRAIT, PANEL_PORTRAIT), &opts));
        }
        let key = b.unit_portrait(cx, u).0.key;
        let img = b.panel_faces.get(&key).and_then(|f| f.composed()).expect("composed").clone();
        let i = ((img.h / 2 * img.w + img.w / 2) * 4) as usize;
        [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
    }

    /// P3 (4.2, 5.2): a unit with `stealth` draws at alpha 0.58 (`fade` 0.6); below 25 % HP `desaturate` 0.5; down:
    /// `still` and `desaturate` 1.0 (a grey pixel). The HP is the board's (rule I).
    #[test]
    fn p3_the_panel_holds_follow_the_units_state() {
        let (cx, mut g, mut b) = ambush();
        let u = b.session().w.allies[0];
        let rel = b.unit_portrait(&cx, u).0.chain("")[0].clone();
        let mut art = Art::layered(Layers::of(vec![layer("p3", &rel, 32, [180, 90, 40], None)]));
        let level = |b: &BattleScreen, e: &str| b.panel_faces.get(&b.unit_portrait(&cx, u).0.key).and_then(|f| f.level(e, b.panel_faces.now())).map(|l| l.i);
        let plain = settled(&cx, &mut b, &mut art, &mut g, u, 1000);
        assert_eq!((level(&b, "fade"), level(&b, "desaturate"), plain[3]), (None, None, 255), "no state: no hold");

        b.host.session.as_mut().unwrap().w.units[u].st.active.insert("stealth".into());
        let px = settled(&cx, &mut b, &mut art, &mut g, u, 2000);
        assert!(level(&b, "fade").is_some_and(|i| (i - 0.6).abs() < 1e-3), "{:?}", level(&b, "fade"));
        let want = (255.0 * (1.0 - 0.7 * 0.6f32)).round() as i32;
        assert!((px[3] as i32 - want).abs() <= 1, "stealth draws at alpha 0.58: {} vs {want}", px[3]);
        b.host.session.as_mut().unwrap().w.units[u].st.active.shift_remove("stealth");

        let max = b.session().w.units[u].max_hp;
        let low = (max * 25 - 1) / 100;
        assert!(low > 0, "a max HP the 25 % row can match above 0");
        b.host.session.as_mut().unwrap().w.units[u].hp = low;
        b.after_change(&cx);
        settled(&cx, &mut b, &mut art, &mut g, u, 3000);
        assert!(level(&b, "desaturate").is_some_and(|i| (i - 0.5).abs() < 1e-3), "below 25 %: {:?}", level(&b, "desaturate"));
        assert_eq!(level(&b, "still"), None);

        b.host.session.as_mut().unwrap().w.units[u].hp = 0;
        b.after_change(&cx);
        let px = settled(&cx, &mut b, &mut art, &mut g, u, 4000);
        assert!(level(&b, "desaturate").is_some_and(|i| (i - 1.0).abs() < 1e-3), "down: {:?}", level(&b, "desaturate"));
        assert!(level(&b, "still").is_some(), "down holds still");
        assert!(px[0] == px[1] && px[1] == px[2], "down draws grey: {px:?}");
    }

    /// P4 (4.5): the panel portraits run on the FX clock: a hit-stop holds them still (a flash keeps its level), and
    /// the animation speed 0.5 moves them at half the wall time.
    #[test]
    fn p4_hit_stop_pauses_the_panel_clock_and_anim_speed_scales_it() {
        let (mut cx, mut g, mut b) = ambush();
        let u = b.session().w.allies[0];
        let key = b.unit_portrait(&cx, u).0.key;
        b.selected = Some(u);
        b.clock = Clock { ms: 1000.0, hold: 0.0 };
        b.react_portraits(&cx, &[(u, "hit")], 1000);
        b.clock.advance(40.0, 1.0);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(b.panel_faces.now(), 1040, "the panels draw at the FX clock");
        let before = b.panel_faces.get(&key).and_then(|f| f.level("flash", b.panel_faces.now())).expect("the flash runs").i;
        b.clock.hold(120.0);
        b.clock.advance(100.0, 1.0);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(b.panel_faces.now(), 1040, "a hit-stop holds the portraits");
        assert_eq!(b.panel_faces.get(&key).and_then(|f| f.level("flash", b.panel_faces.now())).map(|l| l.i), Some(before), "the flash keeps its level");

        // the screen's tick: the FX clock moves by the wall time at the animation speed (at most 100 ms a frame)
        for (speed, lo, hi) in [(1.0, 60.0, 100.0), (0.5, 30.0, 50.0)] {
            cx.settings.v["anim_speed"] = json!(speed);
            b.clock = Clock { ms: 0.0, hold: 0.0 };
            b.clock_at = Instant::now() - Duration::from_millis(60);
            b.tick(&mut cx);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let moved = b.panel_faces.now() as f64;
            assert!(moved >= lo && moved <= hi, "anim_speed {speed}: {moved} ms for 60+ wall ms");
        }
    }

    /// P5 (8): presentation only: the session, the battle's RNG state and the request trace after a scripted battle are
    /// equal with the panel portraits drawn every step and never drawn; two drawn runs give the same portrait pixels
    /// for the same frames.
    #[test]
    fn p5_portraits_never_touch_the_battle_and_replay_the_same_pixels() {
        let run = |draw: bool| {
            let (mut cx, mut g, mut b) = ambush();
            let mut pixels = Vec::new();
            for k in 0..40u64 {
                let (over, phase, planning, can) = {
                    let s = b.host.session.as_ref().unwrap();
                    (s.combat_over, s.phase, s.is_planning(), s.can_advance_round())
                };
                if over || (phase == Phase::Replay && !can) {
                    break;
                }
                if phase == Phase::Replay {
                    b.send(&cx, json!({"cmd": "advance"}));
                    continue;
                }
                if planning {
                    b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
                }
                b.send(&cx, json!({"cmd": "step"}));
                if draw {
                    b.resolve_impacts(&mut cx);
                    b.clock = Clock { ms: (k * 700 + 600) as f64, hold: 0.0 };
                    b.fire_due(&mut cx);
                    b.clock.hold = 0.0;
                    b.draw(&mut cx, &mut g, (-500, -500), true);
                    for id in ["hero.portrait", "intel.portrait"] {
                        if let Some(r) = b.w.iter().find(|w| w.id == id).map(|w| w.rect) {
                            let c = &g.canvas;
                            let crop: Vec<u8> = (r.y..r.y + r.h).flat_map(|y| (r.x..r.x + r.w).flat_map(move |x| c.px[((y * c.w + x) * 4) as usize..((y * c.w + x) * 4 + 4) as usize].to_vec())).collect();
                            pixels.push((k, id, crop));
                        }
                    }
                }
            }
            b.flush(&cx);
            let state = b.host.handle(&json!({"cmd": "state"}));
            let s = b.host.session.as_ref().unwrap();
            (state, format!("{:?}", s.resolver.rng), s.resolver.log.clone(), b.host.history.clone(), pixels)
        };
        let drawn = run(true);
        let plain = run(false);
        assert_eq!(drawn.0, plain.0, "the session state");
        assert_eq!(drawn.1, plain.1, "the RNG state");
        assert_eq!((&drawn.2, &drawn.3), (&plain.2, &plain.3), "the log and the request trace");
        assert!(!drawn.4.is_empty(), "the panels drew portraits");
        let again = run(true);
        assert_eq!(drawn.4, again.4, "a replay draws the same portrait pixels for the same frames");
    }
}

/// PORTRAITFX1-TL (docs/specs/PORTRAITFX1.md 6.2 row 4, 6.5, 4.6; PORTRAITFX1_TASKS.md TL2): the full timeline view's
/// track icons: every unit's track leads with its portrait in the widget's icon square, in the minimal tier however
/// large the window (flash, tints, fade; no jolt, no breath), fading for a stealthed unit and flashing at a struck
/// unit's impact. Cue rows are this module's fixtures, portraits scratch layers.
#[cfg(test)]
mod portrait_timeline_tests {
    use super::*;
    use crate::portraits::tests::{layer, row};
    use aegis_core::assets::Layers;
    use aegis_ui::portrait_fx::{Tier, EASE_MS};
    use aegis_ui::timeline::ROW_H;
    use emberwake_data::PortraitCue;
    use emberwake_rules::ruleset::TIMELINE;

    /// The fixture rows: a blow's flash and jolt on every surface, stealth's fade.
    fn rows() -> Vec<PortraitCue> {
        vec![
            row(json!({"id": "hit_flash", "event": "hit", "intensity": 0.6, "ms": 140, "color": "#d9cbb4"})),
            row(json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"})),
            row(json!({"id": "crit_flash", "event": "crit", "intensity": 0.9, "ms": 180, "color": "#e11d48"})),
            row(json!({"id": "crit_jolt", "event": "crit", "effect": "jolt", "preset": "crit"})),
            row(json!({"id": "guard_flash", "event": "guard_hit", "intensity": 0.4, "ms": 120, "color": "#fbbf24"})),
            row(json!({"id": "heal_glow", "event": "heal", "intensity": 0.3, "ms": 420, "color": "#fbbf24"})),
            row(json!({"id": "stealthed", "trigger": "status", "status": "stealth", "effect": "fade", "intensity": 0.6})),
        ]
    }

    /// The flash colour (rgb) the fixture rows give `event` on the timeline surface.
    fn flash_of(event: &str) -> [u8; 3] {
        let rows = rows();
        let r = emberwake_battle::portrait::reactions(&rows, &[event], TIMELINE_SURFACE, TIMELINE).into_iter().find(|r| r.effect == "flash").expect("a flash row").clone();
        let c = aegis_core::assets::hex_colour(&r.color).expect("a colour");
        [c[0], c[1], c[2]]
    }

    /// The first catalog encounter under the timeline ruleset on the stage view (its steps wait for their impacts),
    /// the fixture rows, a scratch portrait for every unit (layers named after `tag`: one per test, as they run side
    /// by side), the timeline view open, drawn once.
    fn battle(tag: &str) -> (Cx, Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows());
        cx.settings.v["battle_view"] = json!("stage");
        let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
        assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
        let n = b.session().w.units.len();
        let layers: Vec<_> = (0..n).map(|u| layer(&format!("tl_{tag}_{u}"), &b.unit_portrait(&cx, u).0.chain("")[0], 32, [180, 90, 40], None)).collect();
        cx.art = Art::layered(Layers::of(layers));
        let mut g = crate::hud::hud_tests::gfx();
        b.tick(&mut cx);
        if b.selected.is_none() {
            b.selected = b.session().w.allies.first().copied();
        }
        b.tv_open = true;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(b.timeline_view_open(), "the timeline view opens");
        (cx, g, b)
    }

    fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a aegis_ui::ui::Widget<Action>> {
        b.w.iter().find(|w| w.id == id)
    }

    /// TL2 placement (6.5, TEXTFIT1): at the four text sizes every unit's track has its icon square (row height,
    /// left of its label, on its row), the portrait kept for it is in the minimal tier, and no timeline text spills or
    /// is cut without its full text on hover.
    #[test]
    fn tl2_every_track_leads_with_its_portrait_in_the_minimal_tier() {
        let (mut cx, mut g, mut b) = battle("place");
        let n = b.session().w.units.len();
        for (_, scale) in aegis_ui::screen::TEXT_SIZES {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale as f32);
            let view = b.tv_shown.clone().expect("the view is drawn");
            let mut seen = 0;
            for t in view.sections.iter().flat_map(|s| &s.tracks) {
                let u = timeline_view::unit_of(&t.key).expect("a unit's track");
                assert!(t.icon, "{scale}: {} has an icon square", t.key);
                let (Some(sq), Some(label)) = (widget(&b, &format!("timeline.icon.{}", t.key)), widget(&b, &format!("timeline.track.{}", t.key))) else { panic!("{scale}: {}'s square and label", t.key) };
                let (sq, label) = (sq.rect, label.rect);
                if sq.h < ROW_H {
                    continue; // scrolled partly out of the rows' view (its widget is cut to it)
                }
                assert_eq!((sq.w, sq.h, sq.y), (ROW_H, ROW_H, label.y - 1), "{scale}: {}'s square is the row's height, on its row", t.key);
                assert!(sq.x + sq.w <= label.x, "{scale}: {}'s square is left of its label", t.key);
                let key = b.unit_portrait(&cx, u).0.key;
                let f = b.timeline_faces.get(&key).unwrap_or_else(|| panic!("{scale}: {key} drawn on the timeline"));
                assert_eq!(f.tier(), Some(Tier::Minimal), "{scale}: {key}");
                seen += 1;
            }
            assert!(seen > 0 && seen <= n, "{scale}: {seen} squares in view");
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let spills: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("timeline."))).collect();
            assert!(spills.is_empty(), "{scale}: {spills:?}");
            for c in v["cut"].as_array().unwrap() {
                let (x, y) = (c["at"][0].as_f64().unwrap() as i32 + 1, c["at"][1].as_f64().unwrap() as i32 + 2);
                let full = c["text"].as_str().unwrap();
                let Some(wd) = b.w.iter().rev().find(|w| w.rect.contains(x, y) && w.id.starts_with("timeline.") && w.id != "timeline.backdrop") else { continue };
                assert!(wd.text == full || wd.tip.iter().any(|l| l.0.contains(full)), "{scale}: {full:?} cut on {} without its full text on hover", wd.id);
            }
        }
        assert!(b.state(&cx)["timeline_portraits"].as_array().is_some_and(|a| !a.is_empty()), "the agent link sees the timeline portraits");
    }

    /// TL2 tier (4.6): on a window four times the canvas the 15 px square is 60 device px (reduced by its side) and
    /// still draws minimal, while the 32 px panel portrait is full: a blow flashes the row in its row's colour with no
    /// jolt and no breath; the panel jolts.
    #[test]
    fn tl2_a_blow_flashes_the_row_without_a_jolt_at_any_window_size() {
        let (mut cx, _g, mut b) = battle("tier");
        let mut big = crate::hud::hud_tests::gfx();
        big.canvas = aegis_ui::canvas::Canvas::new(W * 4, H * 4);
        big.view = aegis_ui::painter::View::fit((W * 4) as u32, (H * 4) as u32);
        assert_eq!(Tier::of(big.view.rect(Rect::new(0, 0, ROW_H, ROW_H)).w), Tier::Reduced, "the square's side alone would be reduced");
        let u = b.session().w.allies[0];
        b.selected = Some(u);
        let key = b.unit_portrait(&cx, u).0.key;
        b.clock = Clock { ms: 1000.0, hold: 0.0 };
        b.react_portraits(&cx, &[(u, "hit")], 1000);
        b.clock.advance(40.0, 1.0);
        b.draw(&mut cx, &mut big, (-500, -500), true);
        let f = b.timeline_faces.get(&key).expect("the row's portrait");
        let now = b.timeline_faces.now();
        assert_eq!(f.tier(), Some(Tier::Minimal));
        let flash = f.level("flash", now).expect("the row flashes");
        assert_eq!((flash.start, flash.color[..3].to_vec()), (1000, flash_of("hit").to_vec()));
        assert_eq!(f.jolt_offset(now), (0, 0), "no jolt in the row");
        assert!(!f.live(now).contains(&"breath"), "no breath in the row: {:?}", f.live(now));
        let panel = b.panel_faces.get(&key).expect("the hero panel's portrait");
        assert_eq!(panel.tier(), Some(Tier::Full), "the panel portrait is uncapped");
        assert!(panel.jolt_amplitude(now) > 0.0, "the panel jolts for the same blow");
    }

    /// TL2 impact (FX1 rule I, 5.3): a step's portrait events wait in its impact; at the impact each struck unit's row
    /// flashes from the impact's ms in its event's colour.
    #[test]
    fn tl2_a_struck_units_row_flashes_at_the_impact_not_at_the_pop() {
        let (mut cx, _g, mut b) = battle("impact");
        let found = super::tests::step_until(&mut cx, &mut b, |_, i| i.portrait.iter().any(|(_, e)| ["hit", "crit", "guard_hit"].contains(e)));
        assert!(found, "the battle has a blow at an impact");
        b.resolve_impacts(&mut cx);
        b.timeline_faces = Faces::capped(Tier::Minimal);
        let i = b.impacts[0].clone();
        let at = i.at.expect("resolved");
        // the blows and heals (PORTRAITFX2: a step may carry `ally_down` too, which no flash row answers)
        let keys: Vec<(String, &str)> = i.portrait.iter().filter(|(_, e)| ["hit", "crit", "guard_hit", "heal"].contains(e)).map(|&(u, e)| (b.unit_portrait(&cx, u).0.key, e)).collect();
        assert!(keys.iter().all(|(k, _)| b.timeline_faces.get(k).is_none()), "nothing reacts at the pop");
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        for (k, ev) in &keys {
            let f = b.timeline_faces.get(k).unwrap_or_else(|| panic!("{k}'s row reacted to {ev}"));
            let flash = f.level("flash", at).unwrap_or_else(|| panic!("{k}: a flash for {ev}"));
            assert_eq!((flash.start, flash.color[..3].to_vec()), (at, flash_of(ev).to_vec()), "{k}: {ev}'s flash from the impact");
        }
        assert_eq!(b.timeline_faces.clock, b.clock, "the timeline clock is the FX clock again after a reaction");
    }

    /// TL2 fade (4.2, 5.2): a stealthed unit's row draws at alpha 0.58 (`fade` 0.6) once its hold has eased in.
    #[test]
    fn tl2_a_stealthed_units_row_fades() {
        let (mut cx, mut g, mut b) = battle("fade");
        let u = b.session().w.allies[0];
        let key = b.unit_portrait(&cx, u).0.key;
        let centre = |b: &BattleScreen| {
            let img = b.timeline_faces.get(&key).and_then(|f| f.composed()).expect("composed").clone();
            let i = ((img.h / 2 * img.w + img.w / 2) * 4) as usize;
            img.px[i + 3]
        };
        b.clock = Clock { ms: 1000.0, hold: 0.0 };
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!((b.timeline_faces.get(&key).and_then(|f| f.level("fade", 1000)).map(|l| l.i), centre(&b)), (None, 255), "no state: no fade");
        b.host.session.as_mut().unwrap().w.units[u].st.active.insert("stealth".into());
        for ms in [2000, 2000 + EASE_MS + 100] {
            b.clock = Clock { ms: ms as f64, hold: 0.0 };
            b.draw(&mut cx, &mut g, (-500, -500), true);
        }
        let now = b.timeline_faces.now();
        assert!(b.timeline_faces.get(&key).and_then(|f| f.level("fade", now)).is_some_and(|l| (l.i - 0.6).abs() < 1e-3));
        let want = (255.0 * (1.0 - 0.7 * 0.6f32)).round() as i32;
        assert!((centre(&b) as i32 - want).abs() <= 1, "stealth draws the row at alpha 0.58: {} vs {want}", centre(&b));
    }

    /// TL2 enemy rows (user 2026-11-01): a stealthed enemy's row stays plain (no fade) while a stealthed party member's
    /// fades; an enemy's row reacts only to a blow the board shows on it (no heal, no unshown blow), a party row to
    /// every event; no row reacts while the view is closed.
    #[test]
    fn tl2_a_stealthed_enemys_row_is_plain_while_a_stealthed_members_fades() {
        let (mut cx, mut g, mut b) = battle("enemy");
        let (a, e) = (b.session().w.allies[0], b.session().w.enemies[0]);
        let (ka, ke) = (b.unit_portrait(&cx, a).0.key, b.unit_portrait(&cx, e).0.key);
        for u in [a, e] {
            b.host.session.as_mut().unwrap().w.units[u].st.active.insert("stealth".into());
        }
        for ms in [2000, 2000 + EASE_MS + 100] {
            b.clock = Clock { ms: ms as f64, hold: 0.0 };
            b.draw(&mut cx, &mut g, (-500, -500), true);
        }
        let now = b.timeline_faces.now();
        let level = |b: &BattleScreen, k: &str, eff: &str, t: u64| b.timeline_faces.get(k).and_then(|f| f.level(eff, t)).map(|l| l.i);
        assert!(level(&b, &ka, "fade", now).is_some_and(|i| (i - 0.6).abs() < 1e-3), "the stealthed member's row fades");
        assert_eq!(level(&b, &ke, "fade", now), None, "the stealthed enemy's row stays plain");
        assert!(b.timeline_faces.get(&ke).is_some_and(|f| f.composed().is_some()), "the enemy's row is drawn");

        b.react_portraits_shown(&cx, &[(e, "hit"), (a, "heal")], 5000, &[]);
        assert_eq!(level(&b, &ke, "flash", 5000), None, "a blow the board does not show: no flash on the enemy's row");
        assert!(level(&b, &ka, "flash", 5000).is_some(), "a party row reacts to every event");
        b.react_portraits_shown(&cx, &[(e, "heal")], 6000, &[e]);
        assert_eq!(level(&b, &ke, "flash", 6000), None, "an enemy's row reacts to blows only");
        b.react_portraits_shown(&cx, &[(e, "hit")], 7000, &[e]);
        assert!(level(&b, &ke, "flash", 7000).is_some_and(|_| b.timeline_faces.get(&ke).unwrap().level("flash", 7000).unwrap().color[..3] == flash_of("hit")), "a shown blow flashes the enemy's row");

        b.tv_open = false;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        b.react_portraits_shown(&cx, &[(a, "hit")], 8000, &[a]);
        assert!(b.timeline_faces.get(&ka).is_none() && b.panel_faces.get(&ka).is_some(), "the view closed: the rows do not react, the panel does");
        assert!(b.state(&cx)["timeline_portraits"].as_array().is_some_and(|v| v.is_empty()), "and the agent state lists none");
    }
}
