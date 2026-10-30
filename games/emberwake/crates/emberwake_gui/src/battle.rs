//! The battle screen (COMBAT-UI3 layout A). Two layers: the battlefield is pixel art scaled with nearest sampling;
//! panels, text, hover cards and menus are drawn smooth at the window's real resolution (painter.rs). Every change goes
//! through the command API (emberwake_host on aegis_host), the same one `aegis serve` and the agent link use.
//! Two modes: the Test Arena (a catalog encounter, Restart / Play again) and a world battle (GameEncounter: the saved
//! party at its stored HP against a world encounter; on Continue the HP is written back, the runner's after_fight
//! settles the outcome and the results popup opens over the map).
//! Mouse: click a hero to select; click a foe, an ally, the hero itself or a tile for its options; hover anything for
//! details; right click closes a menu or undoes the hero's last step; the wheel scrolls the log. Keys: Enter confirm,
//! Space step / next round, A autoplay, Z undo, L log, R restart (arena), Esc close; hold Alt to outline every unit,
//! covered or not (STAGE1-O).

use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::music;
use crate::intent_text;
use crate::fx::{self, Impact, Shown};
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
use emberwake_battle::world::{AiProfile, World};
use emberwake_data::UnitDef;
use emberwake_host::{Battle, Host};
use emberwake_world::{dialogue, explore, progression, story, travel};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// The Flat view's cell (and the grid panel's: 12 x 9 cells of `TILE` at (`GRID_X`, `GRID_Y`)).
const TILE: i32 = 32;
/// The battle screen's panels by name (`BattleScreen::panels`; the UI Gallery's battle panel entries).
pub const PANELS: [&str; 7] = ["strip", "party", "hero", "grid", "hostiles", "intel", "bar"];
const GRID_X: i32 = 128;
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
    /// RULESET1-U (intent UI1): the selected hero's round intent (a button or keys 1-9, one per intent in the ruleset's order).
    Intent(RoundIntent),
    /// The selected hero's Maneuver kind and opener menus (UI1, MV11, PL1).
    KindMenu,
    OpenerMenu,
    /// MM1: the selected hero's Manual toggle; MM3: "Fill the rest" after its last step.
    Manual,
    Fill,
}

#[derive(Debug, Clone)]
enum RowAct {
    Step(Value),
    PartyTactic(usize),
    HeroTactic(usize),
    /// RULESET1-U: fields of a `round_intent` request for the selected hero (target, maneuver, opener).
    Round(Value),
}

/// RULESET1-U (intent PL7, UI1): the selected hero's intents of the ruleset in order, each with the planner's reason it
/// is not offered (None = offered; never computed outside planning), kept until the battle changes (`rev`) or another
/// hero is picked.
struct Offers {
    key: (usize, u64),
    intents: Vec<(RoundIntent, Option<String>)>,
}

/// One line of help a round intent's button shows (RULESET1_INTENT section 2).
fn intent_help(i: RoundIntent) -> &'static str {
    match i {
        RoundIntent::Assault => "Attack one enemy: an approach or a kite step, an opener, then follow-ups.",
        RoundIntent::Guard => "Hold yourself or a tile: Active Guard and the Guard skills.",
        RoundIntent::Support => "Heal or buff yourself or an ally.",
        RoundIntent::Maneuver => "Engage a flank, Disengage, or use a foe-aimed item.",
        RoundIntent::Channel => "Channel a meter.",
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

/// UI5: the intent glyph the strip, the cards and the board share: a plate in the intent's colour with its initial.
fn intent_glyph(p: &mut Painter, i: RoundIntent, r: Rect) {
    p.panel(r, intent_color(i), rgba(255, 255, 255, 170), 2.0);
    let letter = &i.name()[..1];
    p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, letter, SMALL, TEXT);
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

/// The intents of the battle's ruleset, in its order (empty without the rule round_intent).
fn ruleset_intents(w: &World) -> Vec<RoundIntent> {
    if !ri::on(w) {
        return Vec::new();
    }
    w.ruleset.numbers.intents.iter().filter_map(|i| RoundIntent::parse(i)).collect()
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

/// The Maneuver kind menu's label: the player's kind, or Auto with the planner's pick.
fn kind_label(w: &World, h: usize) -> String {
    let rs = w.round_states.get(&h);
    match (rs.and_then(|r| r.maneuver), rs.is_some_and(|r| r.maneuver_auto)) {
        (Some(k), false) => kind_name(k).into(),
        (Some(k), true) => format!("Auto ({})", kind_name(k)),
        (None, _) => "Auto".into(),
    }
}

/// The opener menu's label: the player's opener (a skill or a pouch item), else Auto (the planner's, PL3).
fn opener_label(w: &World, h: usize) -> String {
    match w.round_states.get(&h).and_then(|r| r.opener.clone()) {
        Some(ri::Opener::Skill(i)) => w.cat.skills[i].name.clone(),
        Some(ri::Opener::Item(id)) => emberwake_battle::command::pouch_item(w, h, &id).map_or(id, |it| it.name.clone()),
        None => "Auto".into(),
    }
}

/// The hero's round in a few words ("Maneuver (Engage) on Wolf").
fn round_line(w: &World, h: usize) -> String {
    let Some(i) = ri::intent_of(w, h) else { return "no intent".into() };
    let kind = if i == RoundIntent::Maneuver { w.round_states.get(&h).and_then(|r| r.maneuver).map(|k| format!(" ({})", kind_name(k))).unwrap_or_default() } else { String::new() };
    let target = w.round_states.get(&h).and_then(|r| r.target).map(|t| format!(" on {}", target_name(w, t))).unwrap_or_default();
    format!("{}{kind}{target}", i.name())
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
}

#[derive(Debug, Clone)]
struct Popup {
    title: String,
    x: i32,
    y: i32,
    w: i32,
    rows: Vec<Row>,
}

struct Float {
    unit: usize,
    text: String,
    color: Rgba,
    born: Instant,
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

    /// DIR4-G: the facing drawn at `now` (the animation clock): its step's while the slide runs (`walk_ms`), else `rest`.
    fn faces(&self, now: u64, walk_ms: u64) -> Facing {
        match self.slide {
            Some((_, t0)) if now.saturating_sub(t0) < walk_ms => self.step,
            _ => self.rest(),
        }
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
        let revealed: Vec<usize> = if planning && tells { s.enemy_intents().iter().filter(|r| r.revealed).map(|r| r.unit).collect() } else { Vec::new() };
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
                        a.slide = Some((from, now));
                        a.player.play_for("walk", now, walk_ms);
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
                    let t0 = if a.slide.is_some_and(|(_, t)| t == now) { now + walk_ms } else { now };
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
/// `story` (STORYUI1 Q5: the story sequence moves on, retries or aborts), else `explore`. The battle screen and the
/// debug fight popup both settle through it.
pub fn after_fight(d: &emberwake_world::Data, gs: &mut emberwake_world::GameState, runner: &str, fight: &Dict, outcome: &str, tally: Option<&emberwake_world::battle::Tally>) -> Dict {
    match runner {
        "travel" => travel::after_fight(d, gs, fight, outcome, tally),
        "dialogue" => dialogue::after_fight(d, gs, fight, outcome, tally),
        story::FIGHT_SOURCE => story::after_fight(d, gs, fight, outcome, tally),
        _ => explore::after_fight(d, gs, fight, outcome, tally),
    }
}

/// The log lines of a settled world fight: its events, and a story fight's notes ("Try again.", the effects').
pub fn log_settled(cx: &mut Cx, runner: &str, res: &Dict) {
    crate::region::log_events(cx, json::aget(res, "events"));
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
    /// RULESET1-U: the selected hero's intent offers (None in classic) and the battle's revision they were made at
    /// (every request and battle start moves it).
    offers: Option<Offers>,
    rev: u64,
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
            offers: None,
            rev: 0,
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
        let mut b = Self::blank(cx, Mode::Arena);
        b.encounter = encounter.to_string();
        b.arena_world = world;
        b.arena_ruleset = ruleset;
        b.arena_rules = rules.to_string();
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
        let hit_world = s(hit, "_world");
        let world = if d.lineage(&gs.world_id).contains(&hit_world) { gs.world_id.clone() } else { hit_world };
        let entry = d.resolve(&world, "encounters", &s(hit, "id"));
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
        // RULESET1 9 D1-D5: under the deck rule a member fights with its deck (classic: None, built as before)
        let deck = emberwake_rules::DeckRule::of(&def.ruleset);
        for m in active {
            let Some(built) = progression::build_in(&d, m, &world, deck.as_ref()) else { continue };
            let mut unit = built.unit;
            unit.hp = progression::start_hp(unit.max_hp as i64, m.hp_lost) as i32;
            let base = d.cat.units.get(&m.base_unit).cloned().unwrap_or_else(|| UnitDef::from_obj(&Default::default()));
            // ASSETORG2: the member's character's body and palette (its base unit's, its class's when it has none)
            let character = d.resolve(&world, "characters", &m.lore_ref);
            let (art, palette, _) = if character.is_empty() {
                let unit = d.entry("units", &m.base_unit);
                let class = d.entry("classes", &base.class_id);
                (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit), Vec::new())
            } else {
                crate::screens::character_art(&d, character)
            };
            party.push(Prebuilt { unit, art, palette, profile: AiProfile::for_unit(&d.cat, &base), key: m.id.clone() });
            members.push((m.id.clone(), built.pouch_unloaded));
        }
        let session = Session::start_with(d.cat.clone(), def, &roster, party);
        let mut b = Self::blank(cx, Mode::World { fight: fight.clone(), runner: runner.to_string(), members });
        b.class_world = world;
        b.host.begin(session);
        b.host.restart();
        b.after_start(&s(entry, "intro"));
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
        self.last_hp = hps;
        let log = &self.session().resolver.log;
        let lines: Vec<String> = log.iter().skip(self.log_seen).cloned().collect();
        for line in &lines {
            if let Some(key) = sound_of(line) {
                sounds.push(format!("sfx/{key}.wav"));
            }
        }
        self.log_seen = log.len();
        let (n, last) = { let s = self.session(); (s.events.len(), s.events.last().cloned()) };
        if n > self.events_seen {
            self.message = last.unwrap_or_default();
            self.events_seen = n;
        }
        self.phase_shakes(cx);
        let pending = if skip || self.req_cmd != "step" { None } else { self.popped_impact(t.walk_ms) };
        let after = self.sync_anims_deferred(cx, pending.is_some());
        match pending {
            Some(mut i) => {
                i.mods = fx::Mods::of(&lines, !struck.is_empty());
                i.broke = fx::broke(&lines);
                (i.after, i.floats, i.struck, i.sounds) = (after, floats, struck, sounds);
                i.marks = self.session().w.units.iter().map(|u| fx::marks_of(&u.st)).collect();
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
                self.show(cx, floats, struck, sounds)
            }
        }
        self.push_bursts(cx);
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
        self.floats.extend(floats.into_iter().map(|(unit, text, color)| Float { unit, text, color, born: now }));
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
        let t0 = if moved && walk_ms > 0 { now + walk_ms } else { now };
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
        self.gap_until = Some(Instant::now() + Duration::from_millis(t.impact_gap_ms));
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
        self.last_hp = self.session().w.units.iter().map(|u| u.hp).collect();
        self.log_seen = self.session().resolver.log.len();
        self.selected = self.session().w.allies.first().copied();
        self.popup = None;
        self.playing = false;
        self.auto = false;
        self.pending_next_round = false;
        self.floats.clear();
        self.log_scroll = 0;
        let (objective, n) = {
            let s = self.session();
            (emberwake_battle::encounter::Objectives::text_of(&s.def.raw, |r| s.units_of_ref(r).first().map(|&c| s.w.units[c].name.clone()).unwrap_or_else(|| r.to_string())), s.events.len())
        };
        self.events_seen = n;
        self.message = if intro.is_empty() { format!("{objective} Plan your round, then confirm.") } else { format!("{intro} {objective}") };
    }

    fn planning(&self) -> bool {
        let s = self.session();
        s.phase == Phase::Planning && !s.plan_confirmed
    }

    /// BattleBridge.outcome: the session's verdict, else won / lost / fled by who stands; "" while it can go on.
    fn outcome(&self) -> String {
        let s = self.session();
        if !s.outcome.is_empty() {
            return s.outcome.clone();
        }
        let standing = s.w.allies.iter().any(|a| s.w.units[*a].is_active());
        let alive = s.w.allies.iter().any(|a| s.w.units[*a].is_alive());
        if s.combat_over {
            return if standing { "won" } else if alive { "fled" } else { "lost" }.into();
        }
        if s.phase == Phase::Replay && !s.can_advance_round() {
            return "fled".into();
        }
        String::new()
    }

    /// World battle over: HP written back (won / fled), the runner settles the outcome, the results popup opens.
    fn settle(&mut self, cx: &mut Cx) {
        let outcome = self.outcome();
        let Mode::World { fight, runner, members } = &self.mode else { return };
        if outcome.is_empty() {
            return;
        }
        let d = cx.data.clone();
        // REWARD1: how the battle left each enemy, for the rewards per defeated enemy
        let tally = emberwake_world::battle::Tally::from_session(self.session());
        let units: Vec<(i64, i64, bool, Vec<String>)> = self.session().w.allies.iter().map(|a| {
            let u = &self.session().w.units[*a];
            (u.hp as i64, u.max_hp as i64, u.is_alive(), u.inventory.pouch.iter().map(|p| p.id.clone()).collect())
        }).collect();
        let gs = cx.gs_mut();
        // ITEMUSE1 rule 7 (GameEncounter.write_back_pouches): the heroes' unused pouch items go back to their saved
        // pouches on every outcome, then the ids that found no battle slot; guests have no member.
        for ((id, unloaded), (.., left)) in members.iter().zip(&units) {
            if let Some(m) = gs.party.member_mut(id) {
                m.pouch = left.iter().chain(unloaded).cloned().collect();
            }
        }
        if outcome == "won" || outcome == "fled" {
            for ((id, _), (hp, max, alive, _)) in members.iter().zip(units) {
                if let Some(m) = gs.party.member_mut(id) {
                    let hp = progression::hp_after(if alive { hp.min(max) } else { 0 }, max, &outcome);
                    m.hp_lost = (max - hp).max(0);
                }
            }
        }
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
        if matches!(a, Action::Confirm | Action::Step | Action::Autoplay | Action::NextRound) {
            cx.audio.play(Bus::Ui, "sfx/ui_click.wav");
        }
        self.run(cx, a);
    }

    /// An action, from the player or chained by another action or autoplay.
    fn run(&mut self, cx: &mut Cx, a: Action) {
        match a {
            Action::Select(u) => {
                self.selected = Some(u);
                self.popup = None;
            }
            Action::UnitClick(u) => self.unit_click(cx, u),
            Action::TileClick(c) => self.tile_click(cx, c),
            Action::Choose(i) => {
                let Some(p) = self.popup.take() else { return };
                let Some(row) = p.rows.get(i).filter(|r| r.valid).cloned() else { return };
                match row.act {
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
                        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[t]}));
                        self.message = format!("Party drafted: {}", TACTIC_LABELS[t]);
                    }
                    RowAct::HeroTactic(t) => {
                        self.hero_tactic = t;
                        if let Some(h) = self.selected {
                            self.send(cx, json!({"cmd": "preset", "unit": h, "preset": TACTICS[t]}));
                            self.message = format!("{} drafted: {}", self.session().w.units[h].name, TACTIC_LABELS[t]);
                        }
                    }
                    RowAct::Round(fields) => self.round_req(cx, fields),
                }
            }
            Action::Intent(i) => self.pick_intent(cx, i),
            Action::KindMenu => self.kind_menu(),
            Action::OpenerMenu => self.opener_menu(),
            Action::Manual => {
                if let Some(h) = self.selected.filter(|_| self.planning()) {
                    let on = !rip::manual(&self.session().w, h);
                    let r = self.send(cx, json!({"cmd": "manual", "unit": h, "on": on}));
                    if r["ok"] == json!(true) {
                        let name = self.session().w.units[h].name.clone();
                        self.message = match r["reason"].as_str() {
                            Some(why) => format!("{name}: {why}"),
                            None if on => format!("{name} plans by hand: click the board for steps, Fill completes the round."),
                            None => format!("{name}: the planner's round again."),
                        };
                    }
                }
            }
            Action::Fill => {
                if let Some(h) = self.selected.filter(|_| self.planning()) {
                    let r = self.send(cx, json!({"cmd": "fill", "unit": h}));
                    if r["ok"] == json!(true) {
                        self.message = format!("{}: follow-ups and Rest fill the round.", self.session().w.units[h].name);
                    }
                }
            }
            Action::ClosePopup => self.popup = None,
            Action::Confirm => {
                let r = self.send(cx, json!({"cmd": "confirm"}));
                if r["ok"] == json!(true) {
                    self.playing = true;
                    self.next_at = Instant::now();
                    self.message = "Resolving the round...".into();
                }
            }
            Action::Clear => {
                if let Some(h) = self.selected {
                    self.send(cx, json!({"cmd": "clear", "unit": h}));
                }
            }
            Action::Undo => {
                if let Some(h) = self.selected {
                    self.send(cx, json!({"cmd": "undo", "unit": h}));
                }
            }
            Action::Autoplay => {
                self.auto = !self.auto;
                if self.auto && self.planning() {
                    self.send(cx, json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                    self.run(cx, Action::Confirm);
                }
            }
            Action::PartyMenu => self.tactic_menu(true),
            Action::HeroMenu => self.tactic_menu(false),
            Action::NextRound => {
                let r = self.send(cx, json!({"cmd": "advance"}));
                if r["ok"] == json!(true) {
                    self.message = format!("Round {}: plan your party.", self.session().current_round);
                    if self.auto {
                        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                        self.run(cx, Action::Confirm);
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
        }
    }

    /// Dropdown of the six tactics under the Party / Hero picker.
    fn tactic_menu(&mut self, party: bool) {
        let current = if party { self.party_tactic } else { self.hero_tactic };
        let help = if emberwake_battle::round_intent::on(&self.session().w) { TACTIC_INTENT_HELP } else { TACTIC_HELP };
        let rows = (0..TACTICS.len())
            .map(|i| Row {
                label: TACTIC_LABELS[i].to_string(),
                detail: if i == current { "current".into() } else { String::new() },
                valid: true,
                act: if party { RowAct::PartyTactic(i) } else { RowAct::HeroTactic(i) },
                tip: vec![(TACTIC_LABELS[i].to_string(), GOLD), (help[i].to_string(), TEXT)],
            })
            .collect();
        let anchor = self.w.iter().find(|w| w.id == if party { "strip.party" } else { "strip.hero" }).map(|w| w.rect).unwrap_or_default();
        self.popup = Some(Popup { title: if party { "Draft the whole party".into() } else { "Draft the selected hero".into() }, x: anchor.x, y: anchor.y + anchor.h + 1, w: 120, rows });
    }

    fn unit_click(&mut self, cx: &Cx, u: usize) {
        let s = self.session();
        let is_ally = s.w.allies.contains(&u);
        let names: Vec<String> = s.w.units.iter().map(|x| x.name.clone()).collect();
        if !self.planning() || self.selected.is_none() {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        }
        let h = self.selected.unwrap();
        if ri::on(&s.w) {
            self.intent_click(cx, h, Some(u), None);
            return;
        }
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

    fn tile_click(&mut self, cx: &Cx, c: V2) {
        if !self.planning() {
            return;
        }
        let Some(h) = self.selected else { return };
        if ri::on(&self.session().w) {
            self.intent_click(cx, h, None, Some(c));
            return;
        }
        let rows = self.options_for(h, "tile", None, Some(c));
        self.open_popup(format!("Tile {}, {}", c.x, c.y), rows);
    }

    // ---- RULESET1-U: the round intent's controls (RULESET1_INTENT UI1, UI2, MM1-MM3) --------------------------------

    /// RULESET1-U: the selected hero's intent offers for this frame (made again when the battle or the hero changed).
    fn refresh_offers(&mut self) {
        let Some(s) = self.host.session.as_ref() else {
            self.offers = None;
            return;
        };
        let Some(h) = self.selected.filter(|h| ri::on(&s.w) && s.w.allies.contains(h)) else {
            self.offers = None;
            return;
        };
        if self.offers.as_ref().is_some_and(|o| o.key == (h, self.rev)) {
            return;
        }
        let planning = s.phase == Phase::Planning && !s.plan_confirmed;
        let intents = ruleset_intents(&s.w).into_iter().map(|i| (i, if planning { rip::preview(&s.w, &s.plan, h, i, None).err() } else { None })).collect();
        self.offers = Some(Offers { key: (h, self.rev), intents });
    }

    /// The planner's reason `h` is not offered `intent` now (None = offered, or no offers made).
    fn refusal(&self, h: usize, intent: RoundIntent) -> Option<String> {
        let o = self.offers.as_ref().filter(|o| o.key.0 == h)?;
        o.intents.iter().find(|x| x.0 == intent).and_then(|x| x.1.clone())
    }

    /// UI1: the selected hero picks `intent` (a button or its key). In auto mode an intent the planner cannot plan is
    /// refused with its reason (PL7); Manual may pick it and place the steps by hand.
    fn pick_intent(&mut self, cx: &Cx, intent: RoundIntent) {
        let Some(h) = self.selected.filter(|_| self.planning()) else { return };
        self.refresh_offers();
        if !rip::manual(&self.session().w, h) {
            if let Some(why) = self.refusal(h, intent) {
                self.message = format!("{}: {} is not offered: {why}", self.session().w.units[h].name, intent.name());
                return;
            }
        }
        self.round_req(cx, json!({"intent": intent.id()}));
    }

    /// A `round_intent` request for the selected hero with `fields`; the message names the planner's refusal (PL7), a
    /// later member it could not plan (PL8), the buffs the change dropped (IN6, from the battle's events) or the round.
    fn round_req(&mut self, cx: &Cx, mut req: Value) {
        let Some(h) = self.selected else { return };
        req["cmd"] = json!("round_intent");
        req["unit"] = json!(h);
        let events = self.session().events.len();
        let r = self.send(cx, req);
        if r["ok"] != json!(true) {
            return;
        }
        let w = &self.session().w;
        let name = w.units[h].name.clone();
        let message = if let Some(why) = r["reason"].as_str() {
            format!("{name}: {why}")
        } else if let Some(f) = r["refused"].as_array().and_then(|a| a.first()) {
            format!("{}: {}", f["name"].as_str().unwrap_or(""), f["reason"].as_str().unwrap_or(""))
        } else if self.session().events.len() > events {
            return; // `after_change` showed the change's line ("Bryn leaves Guard: Riposte ends")
        } else {
            format!("{name}: {}", round_line(w, h))
        };
        self.message = message;
    }

    /// UI1, MM1-MM2: a board or card click under the round intent. Auto: a unit or tile that fits the hero's intent
    /// becomes the round target (the planner rebuilds the round), another ally is selected, anything else says why not.
    /// Manual: the classic step menu (the plan refuses another intent's skill or target), led by a "Round target" row
    /// when the click names a new target that fits.
    fn intent_click(&mut self, cx: &Cx, h: usize, unit: Option<usize>, cell: Option<V2>) {
        let w = &self.session().w;
        let target = match (unit, cell) {
            (Some(u), _) => ri::normalize(h, RoundTarget::Unit(u)),
            (None, Some(c)) => RoundTarget::Cell(c),
            (None, None) => return,
        };
        let other_ally = unit.is_some_and(|u| u != h && w.allies.contains(&u));
        let Some(intent) = ri::intent_of(w, h) else {
            if other_ally {
                self.selected = unit;
            } else {
                self.message = format!("{}: pick a round intent first (keys 1-{}).", w.units[h].name, ruleset_intents(w).len());
            }
            return;
        };
        let problem = ri::target_problem(w, h, intent, target);
        let is_current = w.round_states.get(&h).and_then(|r| r.target) == Some(target);
        let name = target_name(w, target);
        let hero = w.units[h].name.clone();
        if !rip::manual(w, h) {
            if !problem.is_empty() {
                if other_ally {
                    self.selected = unit;
                } else {
                    self.message = problem;
                }
            } else if is_current {
                self.message = format!("{name} is already {hero}'s round target.");
            } else {
                self.round_req(cx, json!({"target": target_json(target)}));
            }
            return;
        }
        let (what, title) = match (unit, cell) {
            (Some(u), _) if u == h => ("self", format!("{hero}: own actions")),
            (Some(u), _) if other_ally => ("ally", format!("{hero} -> {}", w.units[u].name)),
            (Some(u), _) => ("foe", format!("Attack {}", w.units[u].name)),
            (None, Some(c)) => ("tile", format!("Tile {}, {}", c.x, c.y)),
            (None, None) => return,
        };
        let mut rows = self.options_for(h, what, unit.filter(|u| *u != h), cell);
        if problem.is_empty() && !is_current {
            let tip = vec![("Round target".into(), GOLD), (format!("Every command of {hero}'s round acts on {name} (one target a round)."), TEXT)];
            rows.insert(0, Row { label: "Round target".into(), detail: name, valid: true, act: RowAct::Round(json!({"target": target_json(target)})), tip });
        }
        if other_ally && !rows.iter().any(|r| r.valid) {
            self.selected = unit;
            return;
        }
        self.open_popup(title, rows);
    }

    /// MV11: the Maneuver kind menu under its dropdown: Auto and the three kinds, each offered or not with the
    /// planner's reason (PL7).
    fn kind_menu(&mut self) {
        let Some(h) = self.selected.filter(|_| self.planning()) else { return };
        let s = self.session();
        let current = s.w.round_states.get(&h).filter(|r| !r.maneuver_auto).and_then(|r| r.maneuver);
        let kinds = [("auto", "Auto", None), ("engage", "Engage", Some(ManeuverKind::Engage)), ("disengage", "Disengage", Some(ManeuverKind::Disengage)), ("utility", "Utility", Some(ManeuverKind::Utility))];
        let rows = kinds
            .iter()
            .map(|&(id, label, k)| {
                let refused = rip::preview(&s.w, &s.plan, h, RoundIntent::Maneuver, k).err();
                let mut tip: Tip = vec![(label.to_string(), GOLD), (if k.is_none() { "The planner's kind: Engage, else Disengage, else Utility." } else { "Plan the Maneuver round as this kind." }.to_string(), TEXT)];
                if let Some(why) = &refused {
                    tip.push((why.clone(), FOE));
                }
                let detail = match &refused {
                    Some(why) => why.clone(),
                    None if current == k => "current".into(),
                    None => String::new(),
                };
                Row { label: label.into(), detail, valid: refused.is_none(), act: RowAct::Round(json!({"maneuver": id})), tip }
            })
            .collect();
        self.anchored_popup("hero.kind", "Maneuver kind".into(), rows);
    }

    /// PL1, FU1: the opener menu: the planner's pick, then the hero's order for its intent (FU4) and the pouch items of
    /// that intent (IN5), with their AP and blocks.
    fn opener_menu(&mut self) {
        let Some(h) = self.selected.filter(|_| self.planning()) else { return };
        let skills = self.host.handle(&json!({"cmd": "skills", "unit": h}))["skills"].as_array().cloned().unwrap_or_default();
        let w = &self.session().w;
        let Some(intent) = ri::intent_of(w, h) else { return };
        let current = w.round_states.get(&h).and_then(|r| r.opener.clone());
        let row = |label: String, detail: String, opener: Value, is: bool, help: &str| Row {
            detail: if is { format!("current  {detail}") } else { detail },
            tip: vec![(label.clone(), GOLD), (help.to_string(), TEXT)],
            label,
            valid: true,
            act: RowAct::Round(json!({"opener": opener})),
        };
        let mut rows = vec![row("Auto".into(), String::new(), Value::Null, current.is_none(), "The planner's opener: the first of the order the hero can afford and reach.")];
        for sid in rip::order(w, h, intent) {
            let sk = &w.cat.skills[sid];
            let info = skills.iter().find(|x| x["skill"] == json!(sk.id)).map(|x| format!("{} AP, {} bl", x["ap"], x["blocks"])).unwrap_or_default();
            rows.push(row(sk.name.clone(), info, json!(sk.id), current == Some(ri::Opener::Skill(sid)), "Open the round with this skill; follow-ups come after it."));
        }
        for id in emberwake_battle::command::pouch_ids(w, h) {
            let Some(item) = emberwake_battle::command::pouch_item(w, h, &id).filter(|it| ri::item_intent(it) == intent.id()) else { continue };
            let left = emberwake_battle::command::pouch_count(w, h, &id);
            rows.push(row(format!("Use {}", item.name), format!("{left} left"), json!({"item": id}), current == Some(ri::Opener::Item(id.clone())), "Open the round with this pouch item (the planner spends no other)."));
        }
        self.anchored_popup("hero.opener", "Opener".into(), rows);
    }

    /// A menu under the widget `anchor` (the tactic menus' look).
    fn anchored_popup(&mut self, anchor: &str, title: String, rows: Vec<Row>) {
        let a = self.w.iter().find(|w| w.id == anchor).map(|w| w.rect).unwrap_or_default();
        let w = 170;
        let h = 15 + rows.len() as i32 * 11;
        self.popup = Some(Popup { title, x: a.x.clamp(2, W - w - 2), y: (a.y + a.h + 1).clamp(2, H - h - 2), w, rows });
    }

    /// The ruleset chip of an Arena fight beyond classic (RULESET1-U, user 2026-10-29: the Scene Runner card's text);
    /// None in a world battle and in a plain classic one.
    fn arena_chip(&self) -> Option<String> {
        let s = self.host.session.as_ref()?;
        if self.world_mode() || s.w.ruleset.rules.is_empty() {
            return None;
        }
        Some(crate::runner::chip_text(&s.w.ruleset.name, &self.arena_rules, s.w.plan_reveal && s.w.ruleset.has(emberwake_rules::ruleset::PLAN_BADGES)))
    }

    fn open_popup(&mut self, title: String, rows: Vec<Row>) {
        if rows.is_empty() {
            self.message = "Nothing to do there.".into();
            return;
        }
        let (mx, my) = self.mouse;
        let w = 170;
        let h = 15 + rows.len() as i32 * 11;
        self.popup = Some(Popup { title, x: mx.clamp(2, W - w - 2), y: my.clamp(2, H - h - 2), w, rows });
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
            rows.push(Row { label, detail, valid, act: RowAct::Step(step), tip });
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
        self.refresh_offers();
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
        let speed = cx.settings.v["anim_speed"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0);
        self.clock.advance((now.duration_since(self.clock_at).as_secs_f64() * 1000.0).min(100.0), speed);
        self.clock_at = now;
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
        self.floats.retain(|f| now.duration_since(f.born) < Duration::from_millis(t.float_ms));
        self.flashes.retain(|f| now.duration_since(f.1) < Duration::from_millis(t.hit_flash_ms));
        // The playtest session is saved after every concluded round (and once more at the end).
        let ended = self.host.session.as_ref().is_some_and(|s| s.combat_over) && !self.end_logged;
        if self.host.checkpoints.len() != self.saved.1 || !self.parked.is_empty() || !self.trail_out.is_empty() || ended {
            self.save_session(cx);
        }
        if self.host.session.is_none() {
            return;
        }
        // FX1 rule P: the next step waits for `step_ms` and for the last impact plus `impact_gap_ms`.
        let ready = now >= self.next_at && self.impacts_done(now);
        if self.pending_next_round {
            if ready && self.session().phase == Phase::Replay {
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
            self.send(cx, json!({"cmd": "step"}));
            self.next_at = now + Duration::from_millis(t.step_ms);
            if self.session().phase == Phase::Replay {
                self.playing = false;
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
                if self.auto && can_advance {
                    self.next_at = now + Duration::from_millis(t.round_pause_ms);
                    self.pending_next_round = true;
                }
            }
        }
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if self.host.session.is_none() {
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
            // RULESET1-U (intent UI1): keys 1-9 pick the selected hero's intent, one per intent in the ruleset's order
            // (classic: nothing)
            Key::Char(c @ '1'..='9') if self.planning() => {
                let k = c as usize - '1' as usize;
                if let Some(i) = ruleset_intents(&self.session().w).get(k).copied() {
                    self.act(cx, Action::Intent(i));
                }
            }
            _ => {}
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        if self.popup.is_some() {
            self.popup = None;
        } else if self.show_log {
            self.show_log = false;
        } else if self.host.session.is_some() && self.planning() {
            self.act(cx, Action::Undo);
        }
    }

    fn escape(&mut self, cx: &mut Cx) {
        if self.popup.is_some() {
            self.popup = None;
        } else if self.show_log {
            self.show_log = false;
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
            out["ruleset"] = json!({"id": s.w.ruleset.id, "rules": s.w.ruleset.rules, "plan_reveal": s.w.plan_reveal});
            // RULESET1-U: the Arena fight's ruleset chip as drawn
            if let Some(chip) = self.arena_chip() {
                out["ruleset"]["chip"] = json!(chip);
            }
        }
        // RULESET1-U (PL7): the selected hero's intents as last drawn, offered or not with the planner's reason
        if let Some(o) = &self.offers {
            out["intents"] = json!({"unit": o.key.0, "offers": o.intents.iter().map(|(i, why)| json!({"intent": i.id(), "offered": why.is_none(), "reason": why})).collect::<Vec<_>>()});
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
        if let Some(o) = self.host.session.as_ref().and_then(|s| overlay_of(s, self.selected.filter(|u| s.w.allies.contains(u)), s.phase == Phase::Planning && !s.plan_confirmed)) {
            out["overlay"] = overlay_json(&o);
        }
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
        let reply = self.host.handle(body);
        if restart && reply["ok"] == json!(true) {
            self.new_session();
            self.tokens = Tokens::default();
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
    let w = &s.w;
    let u = &w.units[i];
    let ally = w.allies.contains(&i);
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
    if !ally && u.is_active() {
        match tell {
            Some(r) => {
                t.push((format!("Intent: {}", intent_text::first_line(w, &r.line)), GOLD));
                // V1, V2: who reads the plan (a reader who fell stays listed), the enrage or the battle's plan reveal
                if let Some(head) = intent_text::reveal_head(w, r, |p| s.w.cat.passives.get(p).map(|x| x.name.clone()).unwrap_or_else(|| p.to_string())) {
                    t.push((head, ALLY));
                    for l in &r.plan {
                        t.push((intent_text::plan_line(w, l), TEXT));
                    }
                }
                if let Some(f) = sel.and_then(|h| intent_text::forecast(r, h)) {
                    t.push((f, FOE));
                }
            }
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
    let mut ui: Ui<Action> = Ui::new(gfx, mouse, live);
    let Some(s) = g.host.session.as_ref() else {
        ui.p.text(8.0, 8.0, &g.message, BODY, FOE);
        g.w = ui.widgets;
        return;
    };
    let w = &s.w;
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let sel = g.selected.filter(|u| w.allies.contains(u));
    // INTENT1: the enemy tells while planning; their chips and arrows on the grid follow the Show enemy tells setting (I).
    let tells: Vec<Intent> = s.enemy_intents();
    let show_tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
    // RULESET1-U slice 4 (UI5, V5): every unit's round intent record, both sides, planning and playback (empty without
    // the rule plan_badges: classic)
    let rounds: Vec<Intent> = s.round_intents();
    // RULESET1-U slice 5: each overlay group behind its flag (settings `battle_overlays`; all on by default)
    let layers = Layers::of(&cx.settings.v["battle_overlays"]);
    // HOOKS1-UI slice 1: the ground hazards and the arena telegraph (empty in classic)
    let marks = HookMarks::of(s, &layers);
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
            intent_glyph(&mut ui.p, i, Rect::new(78, 8, 10, 10));
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
    let entries: Vec<(i32, i32, String, String, Option<String>, i32, i32)> = match sel {
        Some(u) if s.plan_confirmed => committed(s, u).into_iter().map(|c| (c.0, c.1, c.2, c.3, c.4, 0, 0)).collect(),
        Some(u) if w.units[u].is_active() => s
            .plan
            .compile(w, u)
            .entries
            .iter()
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                (e.start_block, e.end_block(w), sk.name.clone(), sk.kind.clone(), e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), e.total_ap(), e.est_damage)
            })
            .collect(),
        _ => Vec::new(),
    };
    // RULESET1-U (intent UI2, PL6): under the round intent the planned blocks carry the planner's reason tags and the
    // Rest blocks the round's Rest gain
    let timeline = ri::on(w);
    let (reasons, rest) = match sel {
        Some(u) if timeline && !s.plan_confirmed && w.units[u].is_active() => (rip::reasons(w, &s.plan, u), Some(rip::rest_preview(w, &s.plan, u))),
        _ => (Vec::new(), None),
    };
    // RULESET1-U slice 5: each span's wind-up, impact and recovery (and a break), the planned hits' forecasts
    let spans: Vec<SpanParts> = match sel {
        Some(u) if timeline && layers.parts => span_parts(s, u, &entries),
        _ => Vec::new(),
    };
    let hits: Vec<Option<HitForecast>> = match sel {
        Some(u) if timeline && layers.forecasts && !s.plan_confirmed && w.units[u].is_active() => hit_forecasts(s, u),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 3 review: each span's HP cost from its own skill (the plan entry's) or, once confirmed, its command
    let costs: Vec<i32> = match sel {
        Some(u) if s.plan_confirmed => s.resolver.scheduled.iter().filter(|a| a.unit == u).map(|a| a.command.hp_cost).collect(),
        Some(u) if w.units[u].is_active() => s.plan.compile(w, u).entries.iter().map(|e| emberwake_battle::command::hp_cost_of(w, &w.cat.skills[e.skill])).collect(),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 4: each planned entry's push and pull forecasts (rule forced_move; planning, layer `pushes`)
    let pushes: Vec<Vec<emberwake_battle::forced::Forecast>> = match sel {
        Some(u) if planning && layers.pushes && w.allies.contains(&u) && w.units[u].is_active() => push_forecasts(s, u),
        _ => Vec::new(),
    };
    // TEXTFIT1: the controls right of the blocks take their text's width; the blocks give up width first (34 down to
    // 26 each), then the two dropdowns, the wider first (their text cuts).
    let party_label = format!("Party: {}", TACTIC_LABELS[g.party_tactic]);
    let hero_label = format!("Hero: {}", TACTIC_LABELS[g.hero_tactic]);
    let (label, action, enabled) = if planning {
        ("Confirm round", Action::Confirm, true)
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
    let room = W - 6 - bx - 6 - 8 - auto_w - clear_w - confirm_w;
    let bw = ((room - party_w - hero_w) / blocks).clamp(26, 34);
    let mut over = blocks * bw + party_w + hero_w - room;
    while over > 0 && (party_w > 82 || hero_w > 80) {
        let (big, floor) = if party_w - 82 >= hero_w - 80 { (&mut party_w, 82) } else { (&mut hero_w, 80) };
        let cut = over.min(*big - floor);
        *big -= cut;
        over -= cut;
    }
    let live_block = if !planning && s.phase == Phase::Resolving { s.current_block } else { 0 };
    for b in 1..=s.def.total_blocks {
        let r = Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16);
        let at = entries.iter().position(|e| b >= e.0 && b <= e.1);
        let entry = at.map(|i| &entries[i]);
        let mut tip: Tip = Vec::new();
        match entry {
            Some(e) => {
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
                if let Some(why) = at.and_then(|i| reasons.get(i)).filter(|r| !r.is_empty()) {
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
        ui.p.panel(r, if dim { rgb(34, 40, 54) } else { kind_color(&e.3) }, if live { GOLD } else { rgba(255, 255, 255, if dim { 24 } else { 50 }) }, RADIUS);
        // RULESET1-U slice 5: the timeline widget's look: wind-up shaded, impact bright with its edge, recovery hatched
        let sp = spans.get(k);
        for part in sp.map(|sp| sp.parts.as_slice()).unwrap_or(&[]) {
            let pr = Rect::new(bx + (part.start - 1) * bw, 6, (part.end - part.start + 1) * bw - 1, 16);
            aegis_ui::timeline::draw_part(&mut ui, part.style, r, pr);
        }
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 4.0, &e.2, SMALL, r.w as f32 - 6.0, if dim { DIM } else { TEXT });
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
    let mut x = bx + blocks * bw + 6;
    ui.dropdown("strip.party", Rect::new(x, 6, party_w, 16), &party_label, Action::PartyMenu, planning, vec![("Party tactic".into(), GOLD), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), TEXT)]);
    x += party_w + 2;
    ui.dropdown("strip.hero", Rect::new(x, 6, hero_w, 16), &hero_label, Action::HeroMenu, planning && sel.is_some(), vec![("Hero tactic".into(), GOLD), ("AI-drafts the selected hero only.".into(), TEXT)]);
    x += hero_w + 2;
    ui.button_full("strip.auto", Rect::new(x, 6, auto_w, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto, BODY, vec![("Autoplay".into(), GOLD), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), TEXT)]);
    x += auto_w + 2;
    ui.button_full("strip.clear", Rect::new(x, 6, clear_w, 16), "Clear", Action::Clear, planning && sel.is_some(), false, BODY, vec![("Clears the selected hero's steps.".into(), TEXT)]);
    x += clear_w + 2;
    ui.button_full("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, planning, BODY, vec![(label.into(), GOLD)]);

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
            intent_glyph(&mut ui.p, i, Rect::new(r.x + r.w - 13, r.y + 2, 10, 10));
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
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 2.0, &u.name, BODY, 64.0, GOLD);
        // RULESET1-U: under the round intent the corner names the hero's intent (the classic stance is inert: IN3)
        let corner = if timeline { ri::intent_of(w, h).map_or("No intent", |i| i.name()) } else { stance_name(u) };
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 2.0, corner, SMALL, DIM);
        let top = if timeline { draw_intent_controls(&mut ui, w, h, r, g.offers.as_ref(), planning) } else { r.y + 12 };
        let st = s.plan.compile(w, h);
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
            // RULESET1-U (UI1, UI2): the round first (the controls take the panel's top): the target, the planner's refusal
            // (PL7, auto mode), the planned commands without the Rest blocks, then the Rest gain; the stats after.
            let intent = ri::intent_of(w, h);
            match (intent, w.round_states.get(&h).and_then(|r| r.target)) {
                (None, _) => lines.push((format!("Pick a round intent (keys 1-{}).", ruleset_intents(w).len()), GOLD)),
                (Some(_), Some(t)) => lines.push((format!("Target: {}", target_name(w, t)), TEXT)),
                (Some(_), None) => lines.push(("Target: click the board".into(), DIM)),
            }
            if let Some(why) = intent.filter(|_| !s.plan_confirmed && !rip::manual(w, h)).and_then(|i| g.offers.as_ref().filter(|o| o.key.0 == h)?.intents.iter().find(|x| x.0 == i)?.1.clone()) {
                lines.push((why, FOE));
            }
            if s.plan_confirmed {
                for (start, _, name, _, tgt) in committed(s, h) {
                    let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                    lines.push((format!("B{start} {name}{tgt}"), TEXT));
                }
            } else {
                for e in st.entries.iter().filter(|e| e.step_index >= 0 && emberwake_battle::skill::kind(&w.cat.skills[e.skill]) != Kind::Rest) {
                    let name = &w.cat.skills[e.skill].name;
                    let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                    lines.push((format!("B{} {name}{tgt}", e.start_block), TEXT));
                }
                if let Some(gain) = rest.filter(|r| r.blocks > 0) {
                    lines.push((rest_text(gain), GOOD));
                }
                let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(w))).map(|b| b.to_string()).collect();
                if !empty.is_empty() && intent.is_some() && u.is_active() {
                    lines.push((format!("Empty: B{}", empty.join(", B")), GOLD));
                }
                if st.error_index >= 0 {
                    lines.push((st.error_text.clone(), FOE));
                }
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
            let top = ly;
            for part in ui.p.wrap(&text, SMALL, 108.0) {
                if ly > (r.y + r.h) as f32 - pitch - 1.0 {
                    break;
                }
                ui.p.text(r.x as f32 + 4.0, ly, &part, SMALL, col);
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
    let overlay = overlay_of(s, sel, planning);
    let push_list: Vec<emberwake_battle::forced::Forecast> = pushes.iter().flatten().cloned().collect();
    let board = Board { s, tokens: &mut g.tokens, floats: &g.floats, flashes: &g.flashes, popup: g.popup.is_some(), anim_ms: g.clock.ms, sel, planning, tells: &tells, show_tells, card_hover, fx: &g.fx, shift, sidesteps: &g.sidesteps, overlay: overlay.as_ref(), rounds: &rounds, layers, alt: cx.held.alt, marks: &marks, classes: &classes, pushes: &push_list };
    let board_hover = match view {
        View::Stage => draw_stage(board, &mut ui, cx, &tuning, now),
        View::Flat => {
            draw_flat(board, &mut ui, cx, &tuning, now);
            None
        }
    };
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
            intent_glyph(&mut ui.p, i, Rect::new(r.x + 3, r.y + 2, 10, 10));
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
    let detail_foe = hovered_foe.or(ally_hover).or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
        g.panels.push(("intel", r));
        ui.p.panel(r, CARD, if w.allies.contains(&e) { ALLY } else { FOE }, RADIUS);
        let mut ly = r.y as f32 + 3.0;
        let mut lines = unit_tip(s, e, sel, &tells, &rounds, &classes);
        // RULESET1-U slice 5: the selected hero's planned hits on this unit, forecast (rule poise)
        if let Some(h) = sel.filter(|h| layers.forecasts && planning && ri::on(w) && w.units[*h].is_active()) {
            for f in hit_forecasts(s, h).into_iter().flatten().filter(|f| f.target == e) {
                lines.push(f.short(w));
            }
        }
        // the agent link names the panel's unit (UI5: either side; classic adds no widget)
        if !rounds.is_empty() {
            ui.push("intel.unit", "panel", lines.first().map(|l| l.0.clone()).unwrap_or_default(), r, true, false, None, Vec::new());
        }
        // HOOKS1-UI slice 5: the lines that fit (the Level/Speed line gives way first, then "More on its hover card")
        for (part, col, head) in intel_parts(&mut ui.p, &lines, (r.h - 12) as f32) {
            let size = if head { BODY } else { SMALL };
            ui.p.text(r.x as f32 + 4.0, ly, &part, size, col);
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
    let log_r = Rect::new(6, 341, W - 64, 14);
    ui.p.panel(log_r, CARD, BORDER, RADIUS);
    ui.p.text_fit(10.0, 342.5, &last, SMALL, (W - 72) as f32, DIM);
    ui.push("bar.last", "label", last.clone(), log_r, true, false, Some(Action::ToggleLog), vec![(last, TEXT), ("Click to open the full log.".into(), DIM)]);
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
        ui.push("log.panel", "panel", format!("{} lines", s.resolver.log.len()), r, true, true, Some(Action::Step), Vec::new());
        ui.p.panel(r, rgba(10, 14, 24, 246), BORDER, 3.0);
        ui.p.text(r.x as f32 + 6.0, r.y as f32 + 3.0, &format!("Combat log ({} entries)", s.resolver.log.len()), TITLE, GOLD);
        let mut wrapped: Vec<(String, Rgba)> = Vec::new();
        for l in &s.resolver.log {
            let col = if l.starts_with("[Attack]") { TEXT } else if l.starts_with("[Support]") { GOOD } else if l.contains("Failed") || l.contains("Out of Range") { FOE } else { DIM };
            for (k, part) in ui.p.wrap(l, SMALL, (r.w - 22) as f32).into_iter().enumerate() {
                wrapped.push((if k == 0 { part } else { format!("   {part}") }, col));
            }
        }
        let visible = ((r.h - 18) as f32 / 8.0) as i32;
        let max_scroll = (wrapped.len() as i32 - visible).max(0);
        g.log_scroll = g.log_scroll.min(max_scroll);
        let start = (wrapped.len() as i32 - visible - g.log_scroll).max(0) as usize;
        for (k, (t, col)) in wrapped.iter().skip(start).take(visible as usize).enumerate() {
            ui.p.text(r.x as f32 + 6.0, r.y as f32 + 15.0 + k as f32 * 8.0, t, SMALL, *col);
        }
        if max_scroll > 0 {
            let track = Rect::new(r.x + r.w - 8, r.y + 15, 4, r.h - 19);
            ui.p.panel(track, rgba(30, 40, 60, 255), rgba(30, 40, 60, 255), 1.5);
            let thumb_h = (track.h * visible / wrapped.len() as i32).max(8);
            let pos = (track.h - thumb_h) * (max_scroll - g.log_scroll) / max_scroll;
            ui.p.panel(Rect::new(track.x, track.y + pos, 4, thumb_h), GOLD, GOLD, 1.5);
        }
    }

    if let Some(pop) = g.popup.clone() {
        let h = 15 + pop.rows.len() as i32 * 11;
        let r = Rect::new(pop.x, pop.y, pop.w, h);
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup), Vec::new());
        ui.p.panel(r, rgba(14, 20, 34, 252), GOLD, 3.0);
        ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, &pop.title, BODY, r.w as f32 - 10.0, GOLD);
        for (k, row) in pop.rows.iter().enumerate() {
            let rr = Rect::new(r.x + 2, r.y + 13 + k as i32 * 11, r.w - 4, 11);
            let hover = ui.hover(rr);
            if hover && row.valid {
                ui.p.panel(rr, PANEL_HI, rgb(110, 140, 196), 1.5);
            }
            let col = if row.valid { TEXT } else { DIM };
            ui.p.text_fit(rr.x as f32 + 4.0, rr.y as f32 + 1.5, &row.label, BODY, 78.0, col);
            ui.p.text_fit(rr.x as f32 + 84.0, rr.y as f32 + 2.0, &row.detail, SMALL, rr.w as f32 - 86.0, if row.valid { GOLD } else { DIM });
            ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
        }
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
    /// RULESET1 V2 (slices 4, 5): the moves of every revealed enemy (foresight, enrage, the battle's plan reveal) in
    /// its plan's order: its walks as the kit would walk them now (`command::walk_preview`: they stop where the walk
    /// would), a carried Engage dash, a Disengage retreat or jump; (enemy, tiles from where it starts, kind). The board
    /// draws one only for the hovered enemy or while Alt is held (user 2026-10-30). Planning only.
    revealed_moves: Vec<(usize, Vec<V2>, MoveKind)>,
}

/// RULESET1-U slice 6: the overlay groups as the Settings Battle tab lists them: (id, label, what it shows).
pub(crate) const OVERLAY_GROUPS: [(&str, &str, &str); 20] = [
    ("band", "Reach band", "The selected hero's attack reach around the tile it strikes from (Assault, Maneuver)."),
    ("front", "Maneuver front", "The target's front, where an Engage never stands (Maneuver)."),
    ("valid", "Valid targets", "Rings on the units a click would make the round target."),
    ("veils", "Stealth veils", "A veil under each stealthed unit, filled once it is hidden."),
    ("moves", "Planned moves", "The selected hero's moves: a walk, a carried move, a step after its commands."),
    ("target", "Target ring", "The ring on the selected hero's round target."),
    ("facings", "Enemy facings", "A chevron where each enemy faces (a flank comes from beside or behind)."),
    ("walks", "Revealed enemy moves", "A revealed enemy's moves for the enemy under the mouse (hold Alt: all)."),
    ("glyphs", "Intent glyphs", "The round intent's plate on the strip, the cards and the tokens."),
    ("badges", "Impact badges", "The row of blocks on each hostile card with its impacts."),
    ("poise", "Poise bars", "The poise bars under the HP bars."),
    ("breaks", "Breaks", "BREAK over a broken unit and on its cancelled strip span."),
    ("parts", "Wind-up and impact", "The strip's wind-up, impact and recovery shading."),
    ("forecasts", "Hit forecasts", "Stagger and break forecasts in the strip tips and the intel panel."),
    ("summary", "Round summary", "The round summary in the message bar at round end."),
    // HOOKS1-UI slice 1: the hooks' marks (HOOKS1 GH6, AH3)
    ("hazards", "Ground hazards", "The hazards skills and items leave on tiles, in their tint, with their glyph (a foe's hidden one stays hidden)."),
    ("arena", "Arena telegraph", "The cells an arena hazard strikes this round, and a badge on the strip at the blocks it strikes."),
    // HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3)
    ("meters", "Unit meters", "Each unit's meters as pips or a bar on its card, and the selected hero's under the strip title."),
    ("guard", "Held guard", "A blue edge on the card of a unit holding a guard a trigger gave it (it counts as guarding)."),
    // HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5)
    ("pushes", "Push forecasts", "Where the selected hero's planned pushes and pulls would move their targets, and what they would collide with."),
];

/// RULESET1-U slice 5 (preference "Battle overlays and the clarity pass", user 2026-10-30): every timeline overlay
/// group behind its own flag, all on by default (they help debugging the fight); the settings key `battle_overlays`
/// ({<group>: false}) switches a group off; slice 6 lists them on the Settings Battle tab; the clarity pass (VISCLAR1) decides
/// what the game view keeps. Classic draws none of them whatever the flags say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layers {
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
    badges: bool,
    /// Slice 5: poise bars, BREAK markers, wind-up / impact / recovery on the strip, hit forecasts, the round summary.
    poise: bool,
    breaks: bool,
    parts: bool,
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
}

impl Layers {
    /// The group ids, as the settings key names them (`OVERLAY_GROUPS`' order).
    const IDS: [&'static str; 20] = {
        let mut ids = [""; 20];
        let mut k = 0;
        while k < 20 {
            ids[k] = OVERLAY_GROUPS[k].0;
            k += 1;
        }
        ids
    };

    /// The flags of the settings key `battle_overlays` (a missing group, or no key, is on).
    fn of(v: &Value) -> Layers {
        let on = |id: &str| v.get(id).and_then(Value::as_bool).unwrap_or(true);
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
            badges: on("badges"),
            poise: on("poise"),
            breaks: on("breaks"),
            parts: on("parts"),
            forecasts: on("forecasts"),
            summary: on("summary"),
            hazards: on("hazards"),
            arena: on("arena"),
            meters: on("meters"),
            guard: on("guard"),
            pushes: on("pushes"),
        }
    }

    /// The flags by group id (the agent link's state).
    fn json(&self) -> Value {
        let v = [self.band, self.front, self.valid, self.veils, self.moves, self.target, self.facings, self.walks, self.glyphs, self.badges, self.poise, self.breaks, self.parts, self.forecasts, self.summary, self.hazards, self.arena, self.meters, self.guard, self.pushes];
        Value::Object(Self::IDS.iter().zip(v).map(|(k, b)| (k.to_string(), json!(b))).collect())
    }
}

impl Overlay {
    /// What the board draws of it: the groups whose layer is on; of the revealed enemy moves only `focus`'s (the
    /// hovered enemy), or all while `all_walks` (Alt held).
    fn shown(&self, l: &Layers, focus: Option<usize>, all_walks: bool) -> Overlay {
        let keep = |on: bool, v: &Vec<V2>| if on { v.clone() } else { Vec::new() };
        Overlay {
            target: self.target.filter(|_| l.target),
            valid: keep(l.valid, &self.valid),
            band: keep(l.band, &self.band),
            front: keep(l.front, &self.front),
            moves: if l.moves { self.moves.clone() } else { Vec::new() },
            facings: if l.facings { self.facings.clone() } else { Vec::new() },
            veils: if l.veils { self.veils.clone() } else { Vec::new() },
            // slice 6 (decision "RULESET1-U slice 5 answers"): the hovered enemy's moves draw above the tokens
            // (`draw_focus_moves`), so the ground keeps only the others Alt shows
            revealed_moves: self.revealed_moves.iter().filter(|m| l.walks && all_walks && focus != Some(m.0)).cloned().collect(),
        }
    }
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
fn intel_parts(p: &mut Painter, lines: &[(String, Rgba)], room: f32) -> Vec<(String, Rgba, bool)> {
    let wrap = |p: &mut Painter, lines: &[(String, Rgba)]| {
        let mut out = Vec::new();
        for (i, (t, col)) in lines.iter().enumerate() {
            for part in p.wrap(t, if i == 0 { BODY } else { SMALL }, INTEL_W) {
                out.push((part, *col, i == 0));
            }
        }
        out
    };
    let height = |parts: &[(String, Rgba, bool)]| parts.iter().map(|x| if x.2 { INTEL_HEAD } else { INTEL_LINE }).sum::<f32>();
    let mut parts = wrap(p, lines);
    if height(&parts) > room {
        let kept: Vec<(String, Rgba)> = lines.iter().enumerate().filter(|(i, l)| *i == 0 || !l.0.starts_with("Level ")).map(|(_, l)| l.clone()).collect();
        parts = wrap(p, &kept);
    }
    if height(&parts) > room {
        while !parts.is_empty() && height(&parts) + INTEL_LINE > room {
            parts.pop();
        }
        parts.push((INTEL_MORE.to_string(), DIM, false));
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
    use aegis_ui::timeline::{Part, PartStyle};
    let w = &s.w;
    let parts_of = |start: i32, impact: i32, end: i32, winds: bool| {
        let mut out = Vec::new();
        if winds && impact > start {
            out.push(Part { start, end: impact - 1, style: PartStyle::Windup });
        }
        if winds && (impact > start || end > impact) {
            out.push(Part { start: impact, end: impact, style: PartStyle::Impact });
        }
        if end > impact {
            out.push(Part { start: impact + 1, end, style: PartStyle::Recovery });
        }
        out
    };
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
    if planning {
        let revealed: Vec<usize> = s.enemy_intents().iter().filter(|r| r.revealed).map(|r| r.unit).collect();
        for &e in &revealed {
            let mut acts: Vec<&emberwake_battle::resolver::ScheduledAction> = s.resolver.scheduled.iter().filter(|a| a.unit == e && !a.cancelled).collect();
            acts.sort_by_key(|a| a.start);
            let mut at = w.pos(e);
            for a in acts {
                let (tiles, how) = match (&a.command.kind, a.command.carry) {
                    (CmdKind::Walk { path, expected_start, .. }, _) => (emberwake_battle::command::walk_preview(w, e, path, *expected_start), MoveKind::Walk),
                    (CmdKind::Disengage { dest, .. }, _) => (vec![at, *dest], MoveKind::After),
                    (_, emberwake_battle::command::Carry::Engage(c)) => (vec![at, c], MoveKind::Carried),
                    (_, emberwake_battle::command::Carry::Retreat(c)) => (vec![at, c], MoveKind::After),
                    _ => continue,
                };
                if let Some(end) = tiles.last() {
                    at = *end;
                }
                if tiles.len() > 1 && tiles[0] != tiles[tiles.len() - 1] {
                    o.revealed_moves.push((e, tiles, how));
                }
            }
        }
    }
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
        "revealed_moves": o.revealed_moves.iter().map(|(u, t, k)| json!({"unit": u, "kind": k.id(), "tiles": t.iter().map(cell).collect::<Vec<_>>()})).collect::<Vec<_>>(),
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

/// RULESET1-U slice 3: draws the overlays on the ground, under the tokens (Flat: after the tiles; Stage: the markers
/// pass), in the order back to front: the reach band, the Maneuver front, the valid targets, the stealth veils, the
/// revealed enemy moves (slice 5: only `focus`'s, or every one while `all_walks`: Alt held), the move paths, the target
/// ring, the enemy facings; each group only while its layer is on.
fn draw_overlay(p: &mut Painter, o: &Overlay, geo: &Geom, layers: &Layers, focus: Option<usize>, all_walks: bool) {
    let o = &o.shown(layers, focus, all_walks);
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
    // V2: a revealed enemy's moves, dashed red with an arrow head (a carried or after move dashed wider)
    for m in &o.revealed_moves {
        draw_revealed_move(p, m, geo, with_alpha(FOE, 200));
    }
    for (tiles, how) in &o.moves {
        let (col, dash) = match how {
            MoveKind::Walk => (GOLD, 0.0),
            MoveKind::Carried => (AMBER, 3.0),
            MoveKind::After => (GOLD, 2.0),
        };
        let pts: Vec<(f32, f32)> = tiles.iter().map(|c| ground_pt(geo, *c)).map(|(x, y)| (x, y - 3.0)).collect();
        for (k, seg) in pts.windows(2).enumerate() {
            let last = k + 2 == pts.len();
            if last && *how == MoveKind::After {
                stage::wide_arrow(p, seg[0].0, seg[0].1, seg[1].0, seg[1].1, col, 1.5, dash, 4.0);
            } else {
                stage::wide_line(p, seg[0].0, seg[0].1, seg[1].0, seg[1].1, col, if *how == MoveKind::Walk { 2.0 } else { 1.5 }, dash);
            }
        }
        if let (Some(end), true) = (tiles.last(), *how != MoveKind::After) {
            let r = match geo.view {
                View::Stage => inset2(geo.cell_rect(*end), 3, 2),
                View::Flat => geo.cell_rect(*end).inset(3),
            };
            p.panel(r, with_alpha(col, 40), col, 2.0);
        }
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
        // pale over a dark underlay: apart from the tells' red arrows and the foot rings (slice 4: red read as part
        // of the tell arrows next to the intent glyphs)
        for (col, width) in [(rgba(10, 8, 12, 200), 3.2), (FACING, 1.6)] {
            stage::wide_line(p, bx + px, by + py, tx, ty, col, width, 0.0);
            stage::wide_line(p, bx - px, by - py, tx, ty, col, width, 0.0);
        }
    }
}

/// One revealed enemy move (V2) on its tiles' ground points: dashed (a walk finer, a carried or after move wider) with
/// an arrow head on its last stretch.
fn draw_revealed_move(p: &mut Painter, m: &(usize, Vec<V2>, MoveKind), geo: &Geom, col: Rgba) {
    let dash = if m.2 == MoveKind::Walk { 2.0 } else { 3.5 };
    let pts: Vec<(f32, f32)> = m.1.iter().map(|c| ground_pt(geo, *c)).map(|(x, y)| (x, y - 3.0)).collect();
    for (k, seg) in pts.windows(2).enumerate() {
        if k + 2 == pts.len() {
            stage::wide_arrow(p, seg[0].0, seg[0].1, seg[1].0, seg[1].1, col, 1.5, dash, 4.0);
        } else {
            stage::wide_line(p, seg[0].0, seg[0].1, seg[1].0, seg[1].1, col, 1.5, dash);
        }
    }
}

/// RULESET1-U slice 6 (decision "RULESET1-U slice 5 answers"): the hovered enemy's revealed moves above the tokens
/// (the air layer), over a dark underlay so a short step reads over the sprites; its walks layer on.
fn draw_focus_moves(p: &mut Painter, o: &Overlay, geo: &Geom, layers: &Layers, focus: Option<usize>) {
    if !layers.walks {
        return;
    }
    for m in o.revealed_moves.iter().filter(|m| Some(m.0) == focus) {
        let pts: Vec<(f32, f32)> = m.1.iter().map(|c| ground_pt(geo, *c)).map(|(x, y)| (x, y - 3.0)).collect();
        for seg in pts.windows(2) {
            stage::wide_line(p, seg[0].0, seg[0].1, seg[1].0, seg[1].1, rgba(10, 8, 12, 170), 3.4, 0.0);
        }
        draw_revealed_move(p, m, geo, FOE);
    }
}

/// RULESET1-U (intent UI1): the selected hero's round controls at the top of the hero panel `r` (below its name line):
/// the ruleset's intent buttons two a row (`hero.intent.<id>`, keys 1-9 in the ruleset's order; lit = the current one; disabled with the
/// planner's reason, PL7, unless Manual; a red corner and the tip warn of the intent-bound buffs a change drops, IN6),
/// then with an intent the Maneuver kind menu (`hero.kind`, Maneuver only), the opener menu (`hero.opener`) and the
/// Manual toggle with Fill (`hero.manual`, `hero.fill`). Returns the y below them.
fn draw_intent_controls(ui: &mut Ui<Action>, w: &World, h: usize, r: Rect, offers: Option<&Offers>, planning: bool) -> i32 {
    let mut y = r.y + 12;
    let current = ri::intent_of(w, h);
    let manual = rip::manual(w, h);
    let bw = (r.w - 10) / 2;
    let list: &[(RoundIntent, Option<String>)] = offers.filter(|o| o.key.0 == h).map_or(&[], |o| o.intents.as_slice());
    // RULESET1-U slice 6 (TEXTFIT1): two buttons a row while every name fits, else one a row at the panel's width
    let per_row = if list.iter().all(|(i, _)| ui.p.text_w(i.name(), SMALL) <= (bw - 6) as f32) { 2 } else { 1 };
    let ib = if per_row == 2 { bw } else { r.w - 8 };
    for (k, (intent, refused)) in list.iter().enumerate() {
        let b = Rect::new(r.x + 4 + (k as i32 % per_row) * (ib + 2), y + (k as i32 / per_row) * 12, ib, 11);
        let mut tip: Tip = vec![(format!("{} (key {})", intent.name(), k + 1), GOLD), (intent_help(*intent).into(), TEXT)];
        if let Some(why) = refused {
            tip.push((format!("Not offered: {why}"), FOE));
            if manual {
                tip.push(("Manual: you may still pick it and place the steps.".into(), DIM));
            }
        }
        let drops = ri::would_drop(w, h, *intent);
        if !drops.is_empty() {
            tip.push((format!("Changing loses {}.", drops.join(", ")), FOE));
        }
        ui.button_full(&format!("hero.intent.{}", intent.id()), b, intent.name(), Action::Intent(*intent), planning && (refused.is_none() || manual), current == Some(*intent), SMALL, tip);
        if !drops.is_empty() && planning {
            ui.p.fill(Rect::new(b.x + b.w - 4, b.y + 1, 3, 3), FOE);
        }
    }
    y += (list.len() as i32 + per_row - 1) / per_row * 12;
    if current.is_none() {
        return y + 1;
    }
    let row = |y: i32| Rect::new(r.x + 4, y, r.w - 8, 11);
    if current == Some(RoundIntent::Maneuver) {
        let tip = vec![("Maneuver kind".into(), GOLD), ("Engage a flank tile, Disengage after a hit, or use a foe-aimed item; Auto tries them in that order.".into(), TEXT)];
        ui.dropdown("hero.kind", row(y), &format!("Kind: {}", kind_label(w, h)), Action::KindMenu, planning, tip);
        y += 12;
    }
    let tip = vec![("Opener".into(), GOLD), ("The round's first skill or item on the target; the planner's pick is the first of the order it can afford and reach.".into(), TEXT)];
    ui.dropdown("hero.opener", row(y), &format!("Opener: {}", opener_label(w, h)), Action::OpenerMenu, planning, tip);
    y += 12;
    let tip = vec![("Manual".into(), GOLD), ("Plan by hand: board clicks open the steps (the intent's skills on the round target, Rest, moves). Off: the planner's round again.".into(), TEXT)];
    ui.button_full("hero.manual", Rect::new(r.x + 4, y, bw, 11), "Manual", Action::Manual, planning, manual, SMALL, tip);
    let tip = vec![("Fill".into(), GOLD), ("Manual: follow-ups from the order and Rest after the last step.".into(), TEXT)];
    ui.button_full("hero.fill", Rect::new(r.x + 6 + bw, y, bw, 11), "Fill", Action::Fill, planning && manual, false, SMALL, tip);
    y + 13
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
    planning: bool,
    /// INTENT1: the enemy tells while planning; drawn while `show_tells` (the Show enemy tells setting, I).
    tells: &'a [Intent],
    show_tells: bool,
    /// STAGE1 3.9: the unit whose party or hostile card the mouse is over.
    card_hover: Option<usize>,
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
    /// RULESET1-U slice 5: the overlay groups on (`Layers`) and Alt held (every revealed enemy move shows).
    layers: Layers,
    alt: bool,
    /// HOOKS1-UI slice 1: the ground hazards and the arena telegraph (empty in classic).
    marks: &'a HookMarks,
    /// HOOKS1-UI slice 2: each unit's class name for the hover cards (`class_names`).
    classes: &'a [String],
    /// HOOKS1-UI slice 4: the selected hero's push and pull forecasts (planning, layer `pushes`).
    pushes: &'a [emberwake_battle::forced::Forecast],
}

/// An action clip a token wants starts with the first candidate its body has (SPRITE1-U), once the clock reaches its
/// start (FX1-L1: after the actor's slide, so the walk shows until then); WINDUP1 W4: after a held wind-up from the
/// clip's `strike` frame.
fn start_wanted(tokens: &mut Tokens, art: &mut Art, i: usize, body: Option<&str>, now: u64) {
    if let (Some(a), Some(b)) = (tokens.anims.get_mut(i), body) {
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
            if let Some((from, t0)) = a.slide.filter(|s| anim_now.saturating_sub(s.1) < t.walk_ms && !instant) {
                (dx, dy) = geo.slide(from, pos, 1.0 - anim_now.saturating_sub(t0) as f32 / t.walk_ms.max(1) as f32);
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
fn draw_flat(b: Board, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, planning, tells, show_tells, card_hover, fx: live, shift, sidesteps, overlay, rounds, layers, alt, marks, classes, pushes } = b;
    let w = &s.w;
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
                ui.p.panel(r.inset(7), rgba(70, 200, 90, 220), rgba(200, 255, 210, 255), 2.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "EXIT", SMALL, BG);
            }
            if planning && sel.is_some() && ui.hover(r) && !popup {
                ui.p.panel(r.inset(1), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160), 1.5);
            }
            let mut tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), GOOD), ("A unit that steps here leaves the battle.".into(), TEXT)] } else { Vec::new() };
            tip.extend(marks.tile_tip(c));
            ui.push(format!("tile.{gx}.{gy}"), "tile", tid, r, planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    // HOOKS1-UI slice 1: the ground hazards and the arena's struck cells, under the round overlays
    draw_hook_marks(&mut ui.p, marks, &Geom::new(View::Flat, tuning));
    let center = |c: V2| ((GRID_X + c.x * TILE + TILE / 2) as f32, (GRID_Y + c.y * TILE + TILE / 2) as f32);
    if let Some(o) = overlay {
        // RULESET1-U slice 3: the round intent's overlays on the ground (they draw the hero's moves); slice 5: a revealed
        // enemy's moves for the enemy under the mouse (its card or its tile)
        let focus = card_hover.or_else(|| w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p)))));
        draw_overlay(&mut ui.p, o, &Geom::new(View::Flat, tuning), &layers, focus, alt);
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
    if tuning.intent_arrows && planning {
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
            let t = anim_now.saturating_sub(t0);
            if t < tuning.walk_ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / tuning.walk_ms.max(1) as f32);
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
        let outline = if Some(i) == sel { Some(GOLD) } else if hover { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { windup_outline(a, tuning) };
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
        hp_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 3 + dy, TILE - 6, 2), hp, u.max_hp, if ally { GOOD } else { FOE });
        // RULESET1-U slice 5: the poise bar over the HP bar (rule poise)
        if let Some(st) = poise_shown(w, i, &layers) {
            poise_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 6 + dy, TILE - 6, 2), &st);
        }
        let tip = unit_tip(s, i, sel, tells, rounds, classes);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    if show_tells {
        for r in tells {
            let Some(from) = w.grid.pos(r.unit) else { continue };
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
        intent_glyph(&mut ui.p, i, Rect::new(GRID_X + at.x * TILE + TILE - 11, GRID_Y + at.y * TILE + 1, 10, 10));
    }
    // RULESET1-U slice 6: the hovered enemy's revealed moves above the tokens
    if let Some(o) = overlay {
        let focus = card_hover.or_else(|| w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p)))));
        draw_focus_moves(&mut ui.p, o, &Geom::new(View::Flat, tuning), &layers, focus);
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
            let t = now.duration_since(f.born).as_millis() as f32 / tuning.float_ms.max(1) as f32;
            let (cx, cy) = center(pos);
            float_text(ui, &f.text, f.color, cx, cy - 14.0 - tuning.float_rise_px as f32 * t);
        }
    }
    ui.p.shift(-shift.0, -shift.1);
    if shift != (0, 0) {
        ui.p.clip(None);
    }
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
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, planning, tells, show_tells, card_hover, fx: live, shift, sidesteps, overlay, rounds, layers, alt, marks, classes, pushes } = b;
    let w = &s.w;
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
            let t = anim_now.saturating_sub(t0);
            if t < tuning.walk_ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / tuning.walk_ms.max(1) as f32);
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
                if let Some(o) = overlay {
                    // RULESET1-U slice 3: the round intent's overlays on the ground (they draw the hero's moves)
                    draw_overlay(&mut ui.p, o, &geo, &layers, hovered.filter(|u| w.enemies.contains(u)), alt);
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
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    plate(&mut ui.p, inset2(geo.cell_rect(*c), 3, 2), rgba(70, 200, 90, 90), rgba(200, 255, 210, 200));
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
                            let outline = if Some(i) == sel { Some(GOLD) } else if hovered == Some(i) { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { windup_outline(a, tuning) };
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
                // RULESET1-U slice 6: the hovered enemy's revealed moves above the tokens
                if let Some(o) = overlay {
                    draw_focus_moves(&mut ui.p, o, &geo, &layers, hovered.filter(|u| w.enemies.contains(u)));
                }
                // HOOKS1-UI slice 4: the push forecasts above the tokens
                draw_pushes(&mut ui.p, pushes, &geo);
                for t in &units {
                    let u = &w.units[t.i];
                    let (fx, fy) = t.foot;
                    if !t.standing {
                        ui.p.text_center(fx as f32, fy as f32 - ui.p.line_h(BODY) / 2.0, "x", BODY, DIM);
                    } else if t.alive {
                        hp_bar(&mut ui.p, Rect::new(fx - 12, fy + 3, 24, 2), t.hp, u.max_hp, if t.ally { GOOD } else { FOE });
                        // RULESET1-U slice 5: the poise bar under the HP bar (rule poise)
                        if let Some(st) = poise_shown(w, t.i, &layers) {
                            poise_bar(&mut ui.p, Rect::new(fx - 12, fy + 6, 24, 2), &st);
                        }
                    }
                }
                if show_tells {
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
                    intent_glyph(&mut ui.p, i, Rect::new(t.foot.0 + 5, (t.foot.1 as f32 - 0.95 * tp as f32 - 2.0).round() as i32, 10, 10));
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
                    ui.p.panel(sign, rgba(70, 200, 90, 230), rgba(200, 255, 210, 255), 2.0);
                    ui.p.text_center(cx_ as f32, sign.y as f32 + (sign.h as f32 - ui.p.line_h(SMALL)) / 2.0, "EXIT", SMALL, BG);
                }
                for f in floats {
                    let Some(pos) = w.grid.pos(f.unit) else { continue };
                    let t = now.duration_since(f.born).as_millis() as f32 / tuning.float_ms.max(1) as f32;
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
mod intent_ui_tests {
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

    /// The first intent the planner offers the hero now that targets an enemy (Assault, else Maneuver).
    fn foe_intent(b: &BattleScreen) -> RoundIntent {
        let s = b.host.session.as_ref().unwrap();
        let h = hero(b);
        [RoundIntent::Assault, RoundIntent::Maneuver].into_iter().find(|i| ruleset_intents(&s.w).contains(i) && rip::preview(&s.w, &s.plan, h, *i, None).is_ok()).expect("an offered intent against the foes")
    }

    /// UI1, PL7: one button per intent of the ruleset, in its order; a button is enabled exactly when the planner offers
    /// the intent now (auto mode), a refused one names the planner's reason; the agent link's state lists the same.
    #[test]
    fn the_intent_buttons_are_the_rulesets_and_follow_the_planner() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
        drawn(&mut cx, &mut g, &mut b);
        let (s, h) = (b.host.session.as_ref().unwrap(), hero(&b));
        let intents = ruleset_intents(&s.w);
        assert!(!intents.is_empty(), "the timeline row has round intents");
        let ids: Vec<String> = b.w.iter().filter(|x| x.id.starts_with("hero.intent.")).map(|x| x.id.clone()).collect();
        assert_eq!(ids, intents.iter().map(|i| format!("hero.intent.{}", i.id())).collect::<Vec<_>>(), "the ruleset's intents in order");
        let offers = b.state(&cx)["intents"]["offers"].clone();
        for (k, i) in intents.iter().enumerate() {
            let refused = rip::preview(&s.w, &s.plan, h, *i, None).err();
            let x = widget(&b, &format!("hero.intent.{}", i.id()));
            assert_eq!(x.enabled, refused.is_none(), "{}: enabled when offered", i.id());
            if let Some(why) = &refused {
                assert!(tip_has(x, why), "{}: the tip names the reason {why}", i.id());
            }
            assert_eq!((offers[k]["intent"].clone(), offers[k]["offered"].clone()), (json!(i.id()), json!(refused.is_none())));
        }
        assert!(b.w.iter().all(|x| x.id != "hero.opener" && x.id != "hero.manual"), "no intent yet: only the intent buttons");
    }

    /// UI1, TG1, PL6, UI2: a click on an offered intent sets it through the command API and the planner builds the round
    /// (the strip's blocks carry its reason tags); a board click on a tile or a foe that fits the intent becomes the
    /// round target with no menu; a click on the hero itself under a foe intent says why not and changes nothing; key 1
    /// picks the ruleset's first intent the same way.
    #[test]
    fn an_intent_plans_the_round_and_a_board_click_sets_its_target() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
        drawn(&mut cx, &mut g, &mut b);
        let intent = foe_intent(&b);
        let h = hero(&b);
        b.click_id(&mut cx, &format!("hero.intent.{}", intent.id())).unwrap();
        assert_eq!(ri::intent_of(w(&b), h), Some(intent), "{}", b.message);
        drawn(&mut cx, &mut g, &mut b);
        let s = b.host.session.as_ref().unwrap();
        let reasons = rip::reasons(&s.w, &s.plan, h);
        let entries = s.plan.compile(&s.w, h).entries;
        assert!(!entries.is_empty() && reasons.iter().any(|r| !r.is_empty()), "the planner's round with its reason tags");
        for (e, why) in entries.iter().zip(&reasons).filter(|(_, r)| !r.is_empty()) {
            assert!(tip_has(widget(&b, &format!("strip.block{}", e.start_block)), &format!("Why: {why}")), "block {} names {why}", e.start_block);
        }
        assert!(widget(&b, &format!("hero.intent.{}", intent.id())).active, "the current intent is lit");
        // a free tile, then a foe
        let free = free_tile(&s.w);
        assert!(ri::target_problem(&s.w, h, intent, RoundTarget::Cell(free)).is_empty(), "a foe intent takes a tile");
        b.run(&mut cx, Action::TileClick(free));
        assert!(b.popup.is_none(), "auto mode opens no menu");
        assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Cell(free)), "{}", b.message);
        let foe = w(&b).enemies.iter().copied().find(|&e| w(&b).units[e].is_active() && w(&b).grid.pos(e).is_some()).expect("a foe on the field");
        b.run(&mut cx, Action::UnitClick(foe));
        assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Unit(foe)), "{}", b.message);
        let problem = ri::target_problem(w(&b), h, intent, RoundTarget::Myself);
        b.run(&mut cx, Action::UnitClick(h));
        assert_eq!((b.message.clone(), w(&b).round_states[&h].target), (problem, Some(RoundTarget::Unit(foe))), "self does not fit a foe intent");
        // key 1: the ruleset's first intent, picked as its button would (refused with the reason when not offered)
        let first = ruleset_intents(w(&b))[0];
        let s = b.host.session.as_ref().unwrap();
        let offered = rip::preview(&s.w, &s.plan, h, first, None).is_ok();
        b.key(&mut cx, Key::Char('1'));
        assert_eq!(ri::intent_of(w(&b), h) == Some(first), offered || first == intent, "{}", b.message);
    }

    /// MM1-MM3: Manual keeps the steps for the player: a board click opens the step menu led by a "Round target" row
    /// for a new target that fits; Fill completes the round (follow-ups and Rest); Manual off is the planner's round.
    /// UI2: a Rest block's tip names the kit's Rest gain.
    #[test]
    fn manual_mode_opens_the_steps_and_fills_the_rest() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
        drawn(&mut cx, &mut g, &mut b);
        let intent = foe_intent(&b);
        let h = hero(&b);
        b.click_id(&mut cx, &format!("hero.intent.{}", intent.id())).unwrap();
        // the round target a free tile first, so every foe is a new target for the Manual click
        let free = free_tile(w(&b));
        b.run(&mut cx, Action::TileClick(free));
        assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Cell(free)), "{}", b.message);
        drawn(&mut cx, &mut g, &mut b);
        b.click_id(&mut cx, "hero.manual").unwrap();
        assert!(rip::manual(w(&b), h), "{}", b.message);
        let foe = w(&b).enemies.iter().copied().find(|&e| w(&b).units[e].is_active() && w(&b).grid.pos(e).is_some()).expect("a foe on the field");
        b.run(&mut cx, Action::UnitClick(foe));
        let rows = &b.popup.as_ref().expect("the step menu").rows;
        assert_eq!(rows[0].label, "Round target", "a new target leads the menu");
        b.run(&mut cx, Action::Choose(0));
        assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Unit(foe)), "{}", b.message);
        // Rest by hand (neutral: any intent, IN5), then its tip names the Rest gain
        b.run(&mut cx, Action::Clear);
        let r = b.send(&cx, json!({"cmd": "add_step", "unit": h, "step": {"mode": "self", "skill": "rest"}}));
        assert_eq!(r["ok"], json!(true), "{r}");
        drawn(&mut cx, &mut g, &mut b);
        let s = b.host.session.as_ref().unwrap();
        let gain = rip::rest_preview(&s.w, &s.plan, h);
        assert!(gain.blocks > 0, "a planned Rest block");
        let rest_block = s.plan.compile(&s.w, h).entries.iter().find(|e| emberwake_battle::skill::kind(&s.w.cat.skills[e.skill]) == Kind::Rest).map(|e| e.start_block).unwrap();
        assert!(tip_has(widget(&b, &format!("strip.block{rest_block}")), &rest_text(gain)), "the Rest gain on its block");
        b.click_id(&mut cx, "hero.fill").unwrap();
        let s = b.host.session.as_ref().unwrap();
        let st = s.plan.compile(&s.w, h);
        assert!((1..=s.def.total_blocks).all(|k| st.entries.iter().any(|e| k >= e.start_block && k <= e.end_block(&s.w))), "Fill leaves no block empty");
        b.click_id(&mut cx, "hero.manual").unwrap();
        assert!(!rip::manual(w(&b), h), "{}", b.message);
    }

    /// IN6, UI1: with an intent-bound buff held, the other intents' buttons warn of what a change drops (the kit's
    /// `would_drop`), the current one warns of nothing.
    #[test]
    fn a_change_that_drops_a_bound_buff_is_warned() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
        let h = hero(&b);
        assert!(ruleset_intents(w(&b)).contains(&RoundIntent::Guard), "the timeline row offers Guard");
        let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "guard", "target": "self"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let s = b.host.session.as_mut().unwrap();
        emberwake_rules::status::grant_now(&mut s.w.units[h], "riposte", 2, 0, "test");
        ri::bind(&mut s.w, h, h, &["riposte".to_string()], false);
        b.after_change(&cx);
        drawn(&mut cx, &mut g, &mut b);
        for i in ruleset_intents(w(&b)) {
            let drops = ri::would_drop(w(&b), h, i);
            assert_eq!(drops.is_empty(), i == RoundIntent::Guard, "{}: {drops:?}", i.id());
            if !drops.is_empty() {
                assert!(tip_has(widget(&b, &format!("hero.intent.{}", i.id())), &format!("Changing loses {}", drops.join(", "))), "{}", i.id());
            }
        }
    }

    /// PL1, FU1, MV11: the opener menu lists Auto, then the hero's order for its intent (FU4); a pick becomes the opener.
    /// The Maneuver kind menu offers Auto and the three kinds as the planner does.
    #[test]
    fn the_opener_and_kind_menus_come_from_the_planner() {
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
        drawn(&mut cx, &mut g, &mut b);
        let intent = foe_intent(&b);
        let h = hero(&b);
        b.click_id(&mut cx, &format!("hero.intent.{}", intent.id())).unwrap();
        drawn(&mut cx, &mut g, &mut b);
        b.click_id(&mut cx, "hero.opener").unwrap();
        let order = rip::order(w(&b), h, intent);
        assert!(!order.is_empty(), "an offered intent has an order to open from (PL3)");
        let names: Vec<String> = b.popup.as_ref().expect("the opener menu").rows.iter().map(|r| r.label.clone()).collect();
        assert_eq!(names[0], "Auto");
        assert_eq!(names[1..=order.len()].to_vec(), order.iter().map(|&i| w(&b).cat.skills[i].name.clone()).collect::<Vec<_>>(), "the order");
        b.run(&mut cx, Action::Choose(1));
        assert_eq!(w(&b).round_states[&h].opener, Some(ri::Opener::Skill(order[0])), "{}", b.message);
        assert!(ruleset_intents(w(&b)).contains(&RoundIntent::Maneuver), "the timeline row offers Maneuver");
        b.run(&mut cx, Action::ClosePopup);
        let r = b.send(&cx, json!({"cmd": "manual", "unit": h, "on": true}));
        assert_eq!(r["ok"], json!(true), "{r}");
        let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "maneuver"}));
        assert_eq!(r["ok"], json!(true), "{r}");
        drawn(&mut cx, &mut g, &mut b);
        b.click_id(&mut cx, "hero.kind").unwrap();
        let s = b.host.session.as_ref().unwrap();
        let rows = &b.popup.as_ref().expect("the kind menu").rows;
        let kinds = [None, Some(ManeuverKind::Engage), Some(ManeuverKind::Disengage), Some(ManeuverKind::Utility)];
        assert_eq!(rows.len(), kinds.len());
        for (row, k) in rows.iter().zip(kinds) {
            assert_eq!(row.valid, rip::preview(&s.w, &s.plan, h, RoundIntent::Maneuver, k).is_ok(), "{}", row.label);
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
        let want = crate::runner::chip_text(&timeline.name, &spec, s.w.ruleset.has(PLAN_BADGES));
        assert_eq!(widget(&b, "bar.chip").text, want);
        assert_eq!(b.state(&cx)["ruleset"]["chip"], json!(want));
        assert!(want.contains(&format!("-{off}")));
        let (mut cx, mut g, mut b) = arena(None, "", false);
        drawn(&mut cx, &mut g, &mut b);
        assert!(w(&b).ruleset.is_classic());
        assert!(b.w.iter().all(|x| !x.id.starts_with("hero.intent") && !["hero.kind", "hero.opener", "hero.manual", "hero.fill", "bar.chip"].contains(&x.id.as_str())), "classic draws no round control");
        let st = b.state(&cx);
        assert!(st.get("intents").is_none() && st.get("ruleset").is_none(), "a classic battle's state keeps its shape");
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
        assert_eq!(b.state(&cx)["overlay"], overlay_json(&o), "the agent link's state lists the overlays");
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

    /// V2: with the plan reveal every enemy's tip says so and lists its plan; each revealed walk is drawn tile by tile
    /// from where it starts.
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
        // slice 5: each revealed walk is the kit's walk preview of its scheduled walk, tile by tile
        let o = overlay_of(s(&b), b.selected, true).unwrap();
        let walks: Vec<Vec<V2>> = s(&b)
            .resolver
            .scheduled
            .iter()
            .filter(|a| w.enemies.contains(&a.unit) && !a.cancelled)
            .filter_map(|a| match &a.command.kind {
                CmdKind::Walk { path, expected_start, .. } => Some(emberwake_battle::command::walk_preview(w, a.unit, path, *expected_start)),
                _ => None,
            })
            .filter(|t| t.len() > 1 && t[0] != t[t.len() - 1])
            .collect();
        assert!(!walks.is_empty(), "the enemies walk at the round start");
        let drawn: Vec<&Vec<V2>> = o.revealed_moves.iter().filter(|m| m.2 == MoveKind::Walk).map(|m| &m.1).collect();
        assert_eq!(drawn.len(), walks.len(), "{:?}", o.revealed_moves);
        for m in &drawn {
            assert!(walks.contains(m) && m.windows(2).all(|p| p[0].dist(p[1]) == 1) && m.iter().all(|c| w.grid.is_valid(*c)), "{m:?}");
        }
        // slice 5 review: an unrevealed enemy's moves are left out (no reveal, no foresight: every walk stays hidden)
        let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let tells = s(&b).enemy_intents();
        let hidden: Vec<usize> = tells.iter().filter(|t| !t.revealed).map(|t| t.unit).collect();
        let hidden_walks = s(&b).resolver.scheduled.iter().filter(|a| hidden.contains(&a.unit) && matches!(&a.command.kind, CmdKind::Walk { path, .. } if !path.is_empty())).count();
        assert!(hidden_walks > 0, "an unrevealed enemy walks this round");
        let o = overlay_of(s(&b), b.selected, true).unwrap();
        assert!(o.revealed_moves.iter().all(|m| !hidden.contains(&m.0)), "{:?}", o.revealed_moves);
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

    /// The preference (user 2026-10-30): every group is on by default; the settings key switches one off and the board
    /// stops drawing it; the state lists the flags.
    #[test]
    fn each_overlay_group_has_its_flag() {
        let all = Layers::of(&Value::Null);
        assert_eq!(all.json().as_object().unwrap().len(), Layers::IDS.len());
        assert!(all.json().as_object().unwrap().values().all(|v| *v == json!(true)));
        let off = Layers::of(&json!({"band": false, "walks": false}));
        assert!(!off.band && !off.walks && off.front && off.poise);
        let (mut cx, mut g, mut b) = arena(false);
        assault(&cx, &mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let o = overlay_of(s(&b), b.selected, true).unwrap();
        assert!(!o.band.is_empty() && o.shown(&off, None, true).band.is_empty() && o.shown(&all, None, true).band == o.band);
        cx.settings.v["battle_overlays"] = json!({"band": false});
        assert_eq!(b.state(&cx)["overlay_layers"]["band"], json!(false));
    }

    /// The slice 4 review (user 2026-10-30): a revealed enemy's moves show only for the hovered enemy, or all while Alt
    /// is held.
    #[test]
    fn revealed_moves_show_for_the_hovered_enemy_or_with_alt() {
        let (mut cx, mut g, mut b) = arena(true);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let o = overlay_of(s(&b), b.selected, true).unwrap();
        assert!(!o.revealed_moves.is_empty(), "the revealed enemies move this round");
        let l = Layers::of(&Value::Null);
        assert!(o.shown(&l, None, false).revealed_moves.is_empty(), "nothing hovered, no Alt: none");
        assert_eq!(o.shown(&l, None, true).revealed_moves, o.revealed_moves, "Alt: all");
        let e = o.revealed_moves[0].0;
        // slice 6: the hovered enemy's moves draw above the tokens (`draw_focus_moves`), not on the ground
        assert!(o.shown(&l, Some(e), false).revealed_moves.is_empty(), "the hovered one is not on the ground");
        assert!(o.shown(&l, Some(e), true).revealed_moves.iter().all(|m| m.0 != e), "Alt: the others on the ground");
        assert_eq!(o.shown(&l, Some(e), true).revealed_moves.len() + o.revealed_moves.iter().filter(|m| m.0 == e).count(), o.revealed_moves.len());
    }

    /// The scratch fight of MV8: a dancer next to a footman, Maneuver Disengage with AP for one Slip Strike, which then
    /// carries the retreat after its hit.
    fn disengage_session() -> (Session, usize, usize) {
        scratch_session(true)
    }

    /// The scratch fight: Maneuver Disengage (`disengage`) or Assault (with AP for two Cleaves) on the footman.
    fn scratch_session(disengage: bool) -> (Session, usize, usize) {
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
        let cat = emberwake_data::Catalog::from_value(&json!({"skills": skills, "units": [unit("g_dancer", "party", json!({"maneuver": ["g_slip"], "assault": ["g_cut"]})), unit("g_footman", "enemy", json!({"assault": ["g_heavy"]}))]}));
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

    /// The preference (user 2026-10-30): the Settings Battle tab lists every overlay group, on by default; a click
    /// switches one off in the draft (Apply saves it under `battle_overlays`, which the battle screen reads).
    #[test]
    fn the_settings_list_every_overlay_group() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut g = crate::hud::hud_tests::gfx();
        let defaults = crate::screens::default_settings();
        assert!(OVERLAY_GROUPS.iter().all(|(id, _, _)| defaults["battle_overlays"][*id] == json!(true)), "all on by default");
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
                        b.floats.extend(living.iter().zip(&texts).map(|(&unit, text)| Float { unit, text: text.clone(), color: if text.starts_with('-') { damage } else { heal }, born: now }));
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
        let ours = |e: &Value| e["widget"].as_str().is_some_and(|w| ["strip.", "party.", "foe.", "tile.", "intel."].iter().any(|p| w.starts_with(p)));
        let mut out: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| ours(e)).cloned().collect();
        // cut by design and not HOOKS1-UI's (TEXTFIT1, RULESET1-U): a strip span's label (its block row, y under 26) and
        // the bar's ruleset chip (y 322 and below)
        let by_design = |e: &Value| e["at"][1].as_f64().is_some_and(|y| !(26.0..322.0).contains(&y));
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

    /// The Settings Battle tab lists the 20 overlay groups without a cut, spilled or off-canvas label at any size.
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
            let spill: Vec<&Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("settings.overlay."))).collect();
            assert!(spill.is_empty() && v["cut"].as_array().unwrap().is_empty() && v["edge"].as_array().unwrap().is_empty(), "scale {scale}: {v}");
            let ids = st.widgets().iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("settings.overlay."))).count();
            assert_eq!(ids, OVERLAY_GROUPS.len());
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
