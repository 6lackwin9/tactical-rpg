//! Unit tokens and their animation: floats, poses, wind-ups, slides and the clips each token plays (`Tokens`, `UnitAnim`, `Pose`).

use super::board::dashed;
use super::marks::seen_intents;
use super::DASH_MS;
use aegis_ui::canvas::{Rect, Rgba};
use crate::fx::{self, Shown};
use aegis_ui::painter::Painter;
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::session::{Phase, Session};
use aegis_core::assets::{ClipFrame, ClipPlayer, Facing};
use aegis_ui::art::Art;
use aegis_ui::canvas::Image;
use emberwake_rules::Stance;
use emberwake_battle::skill::Kind;
use serde_json::{json, Value};
use aegis_core::clock::Instant;

pub(super) struct Float {
    pub(super) unit: usize,
    pub(super) text: String,
    pub(super) color: Rgba,
    pub(super) born: Instant,
    /// How long it shows (wall ms: the tuning's `float_ms`, doubled at PB3's 0.5x).
    pub(super) ms: u64,
}

/// What a unit was when its token last caught up with the battle (SPRITE1-U).
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Seen {
    pub(super) pos: Option<V2>,
    pub(super) hp: i32,
    pub(super) alive: bool,
    pub(super) guarded: bool,
    /// WINDUP1: its poise broken this round; the Staggered status (active or pending).
    broken: bool,
    staggered: bool,
}

/// One unit's token animation (SPRITE1-U): its clips, a slide to a new tile, its fall. Times are on the battle's
/// FX clock (`BattleScreen::clock`, scaled by the animation speed setting; FX1-L1).
#[derive(Debug, Clone, Default)]
pub(super) struct UnitAnim {
    pub(super) player: ClipPlayer,
    /// An action clip to start (candidates best first, `action_clips`) and when; the draw picks the first the body has.
    pub(super) want: Option<(Vec<&'static str>, u64)>,
    /// The tile it slides from and when the slide started.
    pub(super) slide: Option<(V2, u64)>,
    /// INPUT1 MR12: the slide is a dash (`DASH_MS` long, not the walk's), and its `dash` clip is still to be picked
    /// once the body is known (`start_wanted`).
    dash: bool,
    pub(super) dash_clip: bool,
    /// When it fell (the death clip plays and holds, then the dead marker shows); None while it stands.
    pub(super) fell: Option<u64>,
    pub(super) seen: Seen,
    /// DIR4-G (FACE1): the facing it keeps (`Facing`: the view and the last left/right; the rig's sheets face right,
    /// a side row is mirrored when `left`).
    pub(super) facing: Facing,
    /// DIR4-G G2: the facing of its last step, shown while it slides (`slide`).
    pub(super) step: Facing,
    /// FACE1: while the party plans, the facing toward its planned or intended target (None: no target).
    pub(super) planned: Option<Facing>,
    /// WINDUP1 W4: the wanted clip starts at its `strike` frame (it struck from the wind-up shown).
    pub(super) strike: bool,
    /// WINDUP1 W2: the action whose wind-up it shows (section 6).
    pub(super) windup: Option<Windup>,
    /// WINDUP1 5: while the party plans, the pose of its round (`plan_pose`; `Pose::None` otherwise).
    stance: Pose,
    /// WINDUP1 6.2: the pose its base shows and the clip last drawn (the agent link's `tokens`).
    pub(super) pose: Pose,
    pub(super) drawn: String,
    /// STATUSFX1 S1, S5: the statuses with a look the board shows (rule I: the state before a step until its impact),
    /// each with whether it is only pending and the clock ms its look starts (its burst's).
    pub(super) marks: Vec<fx::MarkOn>,
    /// STATUSFX1 S2: each slot's cycling (head, body, feet, tint) and the ids drawn last frame (the agent link's
    /// `status_fx`, S9).
    pub(super) slots: [fx::SlotShow; 4],
    pub(super) status_fx: Vec<&'static str>,
}

impl UnitAnim {
    /// DIR4-G G1/G3: the facing while it stands: its planned target's while planning, else the facing it keeps.
    pub(super) fn rest(&self) -> Facing {
        self.planned.unwrap_or(self.facing)
    }

    /// DIR4-G: the facing drawn at `now` (the animation clock): its step's while the slide runs (`slide_ms`), else
    /// `rest`.
    pub(super) fn faces(&self, now: u64, walk_ms: u64) -> Facing {
        match self.slide {
            Some((_, t0)) if walk_ms > 0 && now.saturating_sub(t0) < self.slide_ms(walk_ms) => self.step,
            _ => self.rest(),
        }
    }

    /// How long its slide runs: a dash's `DASH_MS` (INPUT1 MR12), else the walk's `walk_ms`.
    pub(super) fn slide_ms(&self, walk_ms: u64) -> u64 {
        if self.dash { DASH_MS } else { walk_ms }
    }
}

/// WINDUP1 section 3 (rule W1): the pose a command shows, and the planning poses of section 5 (`Guard`, `Dazed`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Pose {
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
    pub(super) fn id(self) -> &'static str {
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
    pub(super) fn held(self) -> bool {
        !matches!(self, Pose::None | Pose::Guard)
    }
}

/// WINDUP1 4: the clips the screen looks up on a body (the four wind-ups, the reaction and the dazed loop).
const POSE_CLIPS: [&str; 6] = ["windup", "windup_heavy", "windup_buff", "windup_debuff", "stagger", "dazed"];

/// WINDUP1: the pose clips one unit's body has (`POSE_CLIPS`), each with its `loop_from`. Empty (no body, or a body
/// without them, as every body before WINDUP1-R1): the unit shows no pose, reaction or wind-up.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct PoseSet(Vec<(&'static str, u32)>);

impl PoseSet {
    pub(super) fn of(clips: &aegis_core::assets::Clips) -> PoseSet {
        PoseSet(POSE_CLIPS.iter().filter_map(|&c| clips.clip(c).map(|k| (c, k.loop_from))).collect())
    }

    pub(super) fn has(&self, clip: &str) -> bool {
        self.0.iter().any(|(c, _)| *c == clip)
    }

    fn loop_from(&self, clip: &str) -> u32 {
        self.0.iter().find(|(c, _)| *c == clip).map_or(0, |(_, l)| *l)
    }

    /// The clip showing `p` (section 4's fallbacks): the guard is `idle_guarded` (A1, A3), on a body with pose rows
    /// (one without them stands as before WINDUP1); None for no pose or when the body has none of its clips.
    pub(super) fn clip(&self, p: Pose) -> Option<&'static str> {
        match p {
            Pose::Guard if !self.0.is_empty() => Some("idle_guarded"),
            _ => p.clips().iter().copied().find(|c| self.has(c)),
        }
    }
}

/// WINDUP1 W2: the wind-up a unit shows: the scheduled action (this round's index), its pose and clip and its start and
/// impact blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Windup {
    pub(super) round: i32,
    pub(super) idx: usize,
    pose: Pose,
    clip: &'static str,
    pub(super) blocks: (i32, i32),
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
pub(super) fn item_pose(item: &emberwake_data::ItemDef) -> Pose {
    if item.aims_at_foes() { Pose::Debuff } else { Pose::Buff }
}

/// W1: the pose of a skill row by its kind (a plan step): attacks and areas by `attack_pose`, support and focus a buff,
/// anything else none (Defend, moves, Disengage, Rest, Target, statuses).
pub(super) fn skill_pose(def: &emberwake_data::SkillDef) -> Pose {
    match emberwake_battle::skill::kind(def) {
        Kind::Attack => attack_pose(Some(def)),
        Kind::Support | Kind::Focus => Pose::Buff,
        _ => Pose::None,
    }
}

/// W1: the pose of a scheduled command (its skill row by name, so at its rank; its item); none for Defend, walks,
/// Disengage, Rest, Target and lost blocks.
pub(super) fn command_pose(s: &Session, c: &emberwake_battle::command::Command) -> Pose {
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
pub(super) fn plan_pose(s: &Session, i: usize, tells: bool, revealed: &[usize]) -> Pose {
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

/// INPUT1 MR12: the command of `unit`'s whose start this block made its dash, if one did (`dashed`).
fn dash_action(s: &Session, unit: usize) -> Option<&emberwake_battle::resolver::ScheduledAction> {
    s.resolver.scheduled.iter().find(|a| a.unit == unit && a.start == s.current_block && a.command.moved == Some(emberwake_battle::command::MoveKind::Dash))
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
pub(super) fn turn(f: Facing, from: V2, to: V2) -> Facing {
    f.toward(to.x - from.x, to.y - from.y)
}

/// The battle's token animations (SPRITE1-U): one per unit, the last popped action already animated and the action
/// log length last seen.
#[derive(Debug, Clone, Default)]
pub(super) struct Tokens {
    pub(super) anims: Vec<UnitAnim>,
    pub(super) popped_seen: Option<(i32, i32, usize)>,
    pub(super) log_len_seen: usize,
    /// FACE1: the Show enemy tells setting at the last catch-up (hidden tells: enemies do not show their intent).
    pub(super) tells: bool,
    /// WINDUP1: the pose clips of each unit's body (`BattleScreen::learn_poses`; none until known: no poses) and the
    /// reduce motion setting at the last catch-up (6.1).
    pub(super) poses: Vec<PoseSet>,
    pub(super) still: bool,
    /// STATUSFX1 S4: the apply bursts waiting for the screen to place them (unit, status, clock ms;
    /// `BattleScreen::push_bursts`) and the wall clock's start for the slot cycling (S7: wall ms).
    pub(super) bursts: Vec<(usize, &'static str, u64)>,
    pub(super) epoch: Option<Instant>,
    /// VISCLAR1 VC4: the debug Party foresight off at the last catch-up (a foresight reveal poses as unrevealed).
    pub(super) foresight_off: bool,
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
    pub(super) fn catch_up(&mut self, s: &Session, now: u64, instant: bool, walk_ms: u64, tells: bool) {
        self.follow(s, now, instant, walk_ms, tells, false);
    }

    /// `catch_up`, and FX1-L1 rule I: with `defer` the units' reactions to the step (hurt, defend, death, the fall, the
    /// guard's base loop, the HP the board shows) wait for its impact (`react` with the returned state); moves, facing
    /// and the actor's clip start now. The popped action's clip starts at `t0`: the pop, or the end of the actor's
    /// slide when it moved in this step (FX1 4.2 step 1). Returns each unit's state after the step. WINDUP1: a wind-up
    /// starts now; one that ended waits for the impact too (a break's `stagger`, W6), but the actor's (its strike).
    pub(super) fn follow(&mut self, s: &Session, now: u64, instant: bool, walk_ms: u64, tells: bool, defer: bool) -> Vec<Shown> {
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
                        // VISCLAR1-C: a dash faces its target from its first frame, not once it has landed
                        if let Some(aim) = dash_action(s, i).and_then(|act| action_aim(s, act, &before).1) {
                            a.facing = turn(a.facing, to, aim);
                            a.step = a.facing;
                        }
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
    pub(super) fn react(&mut self, after: &[Shown], marks: &[Vec<fx::Mark>], at: u64, instant: bool) {
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
    pub(super) fn wall_ms(&mut self, now: Instant) -> u64 {
        now.saturating_duration_since(*self.epoch.get_or_insert(now)).as_millis() as u64
    }

    /// What the board shows of unit `i` (FX1-L1 rule I: the state before a step until its impact): its HP and whether
    /// it stands; the unit's own before the tokens first caught up.
    pub(super) fn shown(&self, i: usize, u: &emberwake_rules::unit::Unit, n: usize) -> (i32, bool) {
        match self.anims.get(i).filter(|_| self.anims.len() == n) {
            Some(a) => (a.seen.hp, a.seen.alive),
            None => (u.hp, u.is_alive()),
        }
    }

    /// WINDUP1 6.2, the agent link's `state.tokens`: per unit the clip last drawn, the base clip, the pose the base
    /// shows and the blocks of the wind-up it shows; STATUSFX1 S9: the status looks drawn (`status_fx`, slot order head,
    /// body, feet, tint, alpha).
    pub(super) fn state(&self) -> Value {
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
pub(super) fn action_clips(skill: &str, kind: &str, weapon: &str) -> Vec<&'static str> {
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

/// WAR2H1 (battle slice): the bodies a unit's token tries, by its art key and its equipped gear: a class base shows
/// the look its grip gives (`1h_buckler`, `2h_<family>`), then its own sheet; a named or variant body keeps its art.
pub(super) fn body_keys(art: &str, u: &emberwake_rules::unit::Unit) -> Vec<String> {
    emberwake_content::art::gear_body_chain(art, &emberwake_content::art::Grip::of_inventory(&u.inventory))
}

/// The frame a battle token shows (DIR4-G G4): the clip `player` plays at `now` in the unit's `facing`
/// (`UnitAnim::faces`), fitted to a `size` square (`Art::played_facing`). The fallback is the engine's
/// (`Clips::resolve_view`): the clip's `up`/`down` row when the body has it, else its side row, else `idle` in that
/// view, else side `idle`. A side row is mirrored when the facing is left (the rig's sheets face right); an up or down
/// row never is. So a body without view clips draws exactly as FACE1 did (side, mirrored when left), and a hit while
/// facing up plays the side `hurt` with the last left/right, the view returning with the base loop.
pub(super) fn token_frame<'a>(art: &'a mut Art, body: &str, palette: &str, player: &ClipPlayer, now: u64, size: i32, facing: Facing) -> Option<(ClipFrame, &'a Image)> {
    art.played_facing(body, palette, player, now, size, facing)
}

/// The highlight outline of the token frame `f` (`token_frame` with the same `facing`), mirrored with it.
pub(super) fn token_outline<'a>(art: &'a mut Art, body: &str, palette: &str, f: &ClipFrame, size: i32, col: Rgba, facing: Facing) -> Option<&'a Image> {
    art.body_outline(body, palette, &f.clip, f.frame, size, col, facing.mirrors(f))
}

/// FX1-L1 (FQ7): the hit flash, the white silhouette of the token frame `f` (`fx::FLASH`, alpha 150) over the sprite
/// square `tr`, mirrored with it; it replaced the white square over the whole cell.
pub(super) fn hit_flash(p: &mut Painter, art: &mut Art, body: &str, palette: &str, f: &ClipFrame, tr: Rect, facing: Facing) {
    if let Some(img) = art.body_silhouette(body, palette, &f.clip, f.frame, tr.w, fx::FLASH, facing.mirrors(f)) {
        p.image(img, tr);
    }
}
