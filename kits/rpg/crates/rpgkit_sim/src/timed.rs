//! RULESET1-G2 (RULESET1_BALANCE1 4.3, decision "RULESET1_BALANCE1 answers" Q3): the `feel` mode's party drivers. `auto`
//! is the battle screen's autoplay (the auto driver: `Session::auto_draft` with the party's tactic); `timed` stands for
//! a player who reads the tells: after the auto draft it times an Assault into the target's visible windup. It reads
//! only what the player sees while planning (TD1): each enemy's tell (its first action: kind and blocks) and its
//! impact badges (block only). The in-game planner and every in-game AI stay as they are: they read no tells (MM4, V3).
//! Deterministic (TD4): lists in order, no random draw.

use rpgkit_battle::intent::{self, Intent};
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::Session;
use rpgkit_battle::skill::{self, Kind, SkillId};
use rpgkit_battle::world::{UnitId, World};

/// The party drivers of `party=` (`both` runs each).
pub const DRIVERS: [&str; 2] = ["auto", "timed"];

/// The tell kinds whose action winds up (P5: Attack, Area, Support, Focus; items show as "act" and are not read).
const WINDUP_KINDS: [&str; 4] = ["attack", "area", "support", "focus"];

/// Who plans the party's rounds in the feel mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Driver {
    /// The battle screen's autoplay (the auto driver, the default).
    #[default]
    Auto,
    /// The tell reader (4.3).
    Timed,
}

impl Driver {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "auto" => Some(Driver::Auto),
            "timed" => Some(Driver::Timed),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Driver::Auto => "auto",
            Driver::Timed => "timed",
        }
    }
}

/// TD1: the blocks a player sees an enemy wind up in, from its tell record: every block of the tell's action from its
/// start to its impact (when that action winds up), and the impact block of every hit badge; ascending, each once.
pub fn visible_windup_blocks(rec: &Intent) -> Vec<i32> {
    let mut out: Vec<i32> = Vec::new();
    let (start, impact) = rec.line.blocks;
    if WINDUP_KINDS.contains(&rec.line.kind) && start > 0 {
        out.extend(start..=impact);
    }
    out.extend(rec.impacts.iter().filter(|(_, k)| *k == intent::HIT).map(|(b, _)| *b));
    out.sort_unstable();
    out.dedup();
    out
}

/// TD2-TD3: after `Session::auto_draft`, before the confirm: each active party member whose round is an Assault on an
/// enemy with a visible windup block is re-planned by manual steps (MM2): Rest blocks, then the Assault skill that lands
/// in the earliest visible windup block it can reach (a `sure_break` skill first, then the most stagger per AP, then
/// the member's order), then "Fill the rest" (MM3). No visible block, no skill that lands in one, or a plan that no
/// longer holds (the member's or a later member's): the auto draft is kept. Inert without the rule `round_intent`.
/// Returns how many members it re-planned.
pub fn adjust(s: &mut Session) -> usize {
    let mut timed = 0;
    if !round_intent::on(&s.w) {
        return timed;
    }
    let tells = s.enemy_intents();
    for c in s.w.allies.clone() {
        if !s.w.units[c].is_active() || round_intent::intent_of(&s.w, c) != Some(RoundIntent::Assault) {
            continue;
        }
        let Some(RoundTarget::Unit(t)) = s.w.round_states.get(&c).and_then(|r| r.target) else { continue };
        let Some(rec) = tells.iter().find(|r| r.unit == t) else { continue };
        let blocks = visible_windup_blocks(rec);
        if blocks.is_empty() {
            continue;
        }
        let kept = s.plan.steps_of(c);
        if timed_round(s, c, t, &blocks) && s.plan_is_valid() {
            timed += 1;
        } else {
            s.plan.set_steps(c, kept);
        }
    }
    timed
}

/// The member's Assault skills that hit one unit, in TD2's preference: `sure_break` first, then the most stagger per
/// AP, then its order (FU4).
pub fn timed_skills(w: &World, c: UnitId) -> Vec<SkillId> {
    let mut out: Vec<(usize, SkillId)> = rip::order(w, c, RoundIntent::Assault).into_iter().enumerate().filter(|(_, sid)| {
        let sk = &w.cat.skills[*sid];
        skill::kind(sk) == Kind::Attack && !skill::is_area(sk)
    }).collect();
    let per_ap = |sid: SkillId| {
        let sk = &w.cat.skills[sid];
        sk.stagger as f64 / skill::ap_cost_by(sk, Some(&w.units[c]), &w.ruleset).max(1) as f64
    };
    out.sort_by(|(i, a), (j, b)| {
        let (sa, sb) = (&w.cat.skills[*a], &w.cat.skills[*b]);
        sb.sure_break.cmp(&sa.sure_break).then(per_ap(*b).total_cmp(&per_ap(*a))).then(i.cmp(j))
    });
    out.into_iter().map(|(_, sid)| sid).collect()
}

/// Plans `c`'s round as Rest blocks, then a skill on `t` landing in the earliest of `blocks` that one reaches, then the
/// fill. False (the steps then are whatever was tried last; the caller restores): none lands in a visible block.
fn timed_round(s: &mut Session, c: UnitId, t: UnitId, blocks: &[i32]) -> bool {
    let w = &s.w;
    let rest = w.book().lowest(Kind::Rest);
    let skills = timed_skills(w, c);
    for &b in blocks {
        for &sid in &skills {
            for rests in 0..w.total_blocks {
                s.plan.set_steps(c, Vec::new());
                if (0..rests).any(|_| !s.plan.add_step(w, c, PlanStep::self_skill(rest), false).is_empty()) {
                    break;
                }
                if !s.plan.add_step(w, c, PlanStep::engage(sid, t), false).is_empty() {
                    break;
                }
                let lands = s.plan.entries_for(w, c).iter().rev().find(|e| e.skill == sid && e.step_index == rests).map(|e| e.end_block(w));
                match lands {
                    Some(at) if at == b => {
                        // MM3: follow-ups and Rest after the timed hit (its refusal keeps the timed steps)
                        let _ = rip::fill_rest(w, &mut s.plan, c);
                        return true;
                    }
                    Some(at) if at > b => break,
                    _ => {}
                }
            }
        }
    }
    false
}
