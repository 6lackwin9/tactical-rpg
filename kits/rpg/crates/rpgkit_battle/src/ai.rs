//! TacticalAI (scripts/combat/ai/tactical_ai.gd): drafts one unit's round as plan steps, growing a shared plan one
//! validated step at a time. Used for enemies every round and for the party's AI presets. Under the rule `round_intent`
//! the round intent AI at the end of this file picks intents and targets instead (RULESET1_INTENT 9, engine-only).

use crate::command::{self, cleansable};
use crate::encounter;
use crate::grid::{V2, DIRS, GRID_H, GRID_W, NO_CELL, ZERO};
use crate::path::{self, Cells, Hooks};
use crate::plan::{cover_pref, expected, PlanEntry, PlanStep, SimState, Step, MAX_MOVE_BLOCKS};
use crate::plan::Plan;
use crate::round_intent::{self, ManeuverKind, Opener, RoundIntent, RoundTarget};
use crate::round_intent_plan;
use crate::skill::{self, Kind, SkillId};
use crate::world::{AiProfile, Role, Tactic, TargetPref, UnitId, UnitTactic, World};
use aegis_core::sort::godot_sort;
use rpgkit_rules::damage;
use rpgkit_rules::status;
use std::collections::HashMap;
use std::rc::Rc;

pub const MAX_STEPS: usize = 16;
pub const MAX_HEALS: usize = 2;
pub const FOCUS_PRESSURE_RATIO: f64 = 0.5;
pub const FAST_BLOCK: i32 = 2;
pub const SELF_CARE_RATIO: f64 = 0.5;
pub const AREA_FF_MIN_FOES: i32 = 3;
pub const AREA_FF_HP_RATIO: f64 = 0.25;
/// ITEMUSE1 rule 9: drink a potion when the round would leave the unit at or below this HP share.
pub const ITEM_HEAL_RATIO: f64 = 0.35;

#[derive(PartialEq)]
enum Care {
    None,
    Healed,
    Retreated,
}

fn approx_eq(a: f64, b: f64) -> bool {
    path::approx_eq(a, b)
}

fn hp_ratio(w: &World, u: UnitId) -> f64 {
    w.units[u].hp as f64 / w.units[u].max_hp.max(1) as f64
}

fn basic_band(w: &World, u: UnitId) -> (i32, i32) {
    skill::reach_by(&w.cat.skills[w.book().lowest(Kind::Attack)], &w.units[u], &w.ruleset)
}

/// TacticalAI.build_steps. With a shared plan the draft is written into it (replacing the actor's steps).
pub fn build_steps(w: &World, plan: &mut Plan, actor: UnitId, profile: &AiProfile) -> Vec<Step> {
    plan.clear(actor);
    let foes: Vec<UnitId> = w.foes_of(actor).to_vec();
    if !w.units[actor].is_active() || !w.has(actor) {
        return plan.steps_of(actor);
    }
    plan.path_penalties.insert(actor);
    plan.ai_pressure.shift_remove(&actor);
    // ENCOUNTER2 H6: a hold escape's keeper walks to its tile and defends, the others leave once the gate opens (none
    // without a hold escape: nothing below changes)
    if hold_steps(w, plan, actor, profile) {
        return plan.steps_of(actor);
    }
    if incoming_damage(w, plan, actor, w.total_blocks, w.pos(actor), None) >= w.units[actor].hp {
        plan.ai_pressure.insert(actor);
    }
    if w.units[actor].attack_range > 1 {
        let shooters = shooters_of(w, &foes);
        if !shooters.is_empty() {
            plan.goal_preferences.insert(actor, shooters);
        }
    }
    if profile.use_items {
        // ITEMUSE1 rule 9 (sim only): at most one item step, first in the sequence
        try_item(w, plan, actor);
    }
    let book = w.book();
    let cap = w.skill_cap_of(actor);
    let mut guarded = false;
    let mut focused = false;
    let mut fled = false;
    let mut healed: Vec<UnitId> = Vec::new();
    let mut rests = 0;
    let mut urgent_cleanse_tried = false;
    let mut late_cleanse_tried = false;
    let mut disengage_tried = false;
    let mut precast_done = false;
    let mut riposte_tried = false;
    let mut self_care_tried = false;
    let band = basic_band(w, actor);
    let reach = band.1;

    for _ in 0..MAX_STEPS {
        let st = plan.compile(w, actor);
        if st.blocks_left() < 1 || st.withdrawn {
            break;
        }
        let ratio = hp_ratio(w, actor);
        if profile.flee_ratio > 0.0 && !profile.reckless && ratio <= profile.flee_ratio && !fled {
            fled = true;
            if plan.add_step(w, actor, PlanStep::withdraw(), true).is_empty() {
                continue;
            }
        }
        if profile.role == Role::Support && !self_care_tried {
            self_care_tried = true;
            let care = self_care(w, plan, actor, profile, cap, &st, &foes, reach);
            if care == Care::Healed {
                healed.push(actor);
            }
            if care != Care::None {
                continue;
            }
        }
        if !urgent_cleanse_tried {
            urgent_cleanse_tried = true;
            if try_cleanse(w, plan, actor, cap, true) {
                late_cleanse_tried = true;
                continue;
            }
        }
        if profile.role == Role::Support && healed.len() < MAX_HEALS {
            if let Some(patient) = try_heal(w, plan, actor, profile, cap, &healed) {
                healed.push(patient);
                continue;
            }
        }
        if !late_cleanse_tried {
            late_cleanse_tried = true;
            if try_cleanse(w, plan, actor, cap, false) {
                continue;
            }
        }
        let mut ranked = rank_targets(w, plan, actor, &st, &foes, profile);
        if ranked.is_empty() {
            break;
        }
        let pulled = taunt_pull(w, plan, actor, &st, &ranked);
        if let Some(p) = pulled {
            ranked = vec![p];
        }
        let target = ranked[0];
        let dist = st.pos.dist(w.pos(target));
        let low_hp = ratio <= profile.defend_below && !profile.reckless;

        if !disengage_tried && !profile.reckless {
            disengage_tried = true;
            let adjacent = nearest_foe(w, st.pos, &foes) <= 1;
            let ranged_role = profile.role == Role::Sniper || profile.role == Role::Support;
            if adjacent && ranged_role && try_disengage(w, plan, actor, cap, profile, &ranked, true) {
                continue;
            }
            if pulled.is_none() && profile.role == Role::Skirmisher && ratio <= profile.defend_below && dist <= 1 && try_disengage(w, plan, actor, cap, profile, &ranked, false) {
                continue;
            }
        }
        if !precast_done {
            precast_done = true;
            if precast(w, plan, actor, &ranked, cap, profile) {
                continue;
            }
        }
        if !focused && (profile.role == Role::Support || w.units[actor].attack_range > 1) {
            focused = true;
            if can_strike_in_place(w, actor, cap, dist, profile) && buff_then_engage(w, plan, actor, &ranked, cap, profile) {
                continue;
            }
        }
        if low_hp && !guarded && !guard_held(w, actor) && dist <= reach + 1 && (pulled.is_none() || dist <= reach) {
            guarded = true;
            if guard_smart(w, plan, actor, cap, profile, &foes, st.pos) {
                continue;
            }
        }
        if !profile.reckless {
            match profile.role {
                Role::Sniper | Role::Support => {
                    if dist < reach && kite_then_engage(w, plan, actor, &st, &foes, &ranked, band, cap, profile) {
                        continue;
                    }
                }
                Role::Skirmisher => {
                    if ratio <= profile.defend_below && dist <= 1 {
                        let back = retreat_dir(w, plan, actor, st.pos, &foes, reach, w.pos(target));
                        if back != ZERO && plan.add_step(w, actor, PlanStep::mv(book.lowest(Kind::Move), back, 1), false).is_empty() {
                            continue;
                        }
                    }
                }
                Role::Aggressor => {}
            }
        }
        if engage_any(w, plan, actor, &ranked, cap, profile) {
            if !riposte_tried {
                riposte_tried = true;
                if riposte_finisher(w, plan, actor, cap, profile, &foes, ratio) {
                    break;
                }
            }
            continue;
        }
        if advance_toward(w, plan, actor, &st, &ranked) {
            continue;
        }
        let strike_cost = skill::ap_cost_by(&w.cat.skills[book.lowest(Kind::Attack)], Some(&w.units[actor]), &w.ruleset);
        if rests < 2 && st.blocks_left() >= 2 && st.ap_left < strike_cost + 1 {
            rests += 1;
            if plan.add_step(w, actor, PlanStep::self_skill(book.find("rest").unwrap()), true).is_empty() {
                continue;
            }
        }
        break;
    }
    if profile.role == Role::Support && healed.is_empty() && !plan.compile(w, actor).withdrawn {
        reserve_heal(w, plan, actor, profile, cap);
    }
    plan.ai_pressure.shift_remove(&actor);
    plan.steps_of(actor)
}

fn shooters_of(w: &World, foes: &[UnitId]) -> Vec<UnitId> {
    foes.iter().copied().filter(|f| w.units[*f].is_active() && w.has(*f) && w.units[*f].attack_range > 1).collect()
}

/// Tries skills of `kind` in preference order; true when one was added.
#[allow(clippy::too_many_arguments)]
fn try_skills(w: &World, plan: &mut Plan, actor: UnitId, kind: Kind, cap: i32, profile: &AiProfile, make_step: Option<&dyn Fn(SkillId) -> Step>, filter: Option<&dyn Fn(SkillId) -> bool>, target: Option<UnitId>) -> bool {
    for sid in skill_order(w, plan, actor, kind, cap, profile, target) {
        if filter.is_some_and(|f| !f(sid)) {
            continue;
        }
        let step = match make_step {
            Some(f) => f(sid),
            None => PlanStep::self_skill(sid),
        };
        if plan.add_step(w, actor, step, false).is_empty() {
            return true;
        }
    }
    false
}

fn sort_by_preferred_level(w: &World, skills: &mut [SkillId], profile: &AiProfile) {
    if profile.prefer_level > 0 {
        let p = profile.prefer_level;
        godot_sort(skills, |a, b| (w.cat.skills[*a].level - p).abs() < (w.cat.skills[*b].level - p).abs());
    }
}

fn skill_order(w: &World, plan: &Plan, actor: UnitId, kind: Kind, cap: i32, profile: &AiProfile, target: Option<UnitId>) -> Vec<SkillId> {
    let mut skills = if kind == Kind::Attack { strikes(w, actor, cap) } else { w.book().available(kind, cap, Some(&w.units[actor])) };
    skills.reverse();
    sort_by_preferred_level(w, &mut skills, profile);
    let mut ordered: Vec<SkillId> = skills.iter().copied().filter(|s| !skill::is_off_hand(&w.cat.skills[*s])).collect();
    ordered.extend(skills.iter().copied().filter(|s| skill::is_off_hand(&w.cat.skills[*s])));
    if kind == Kind::Attack {
        if let Some(t) = target {
            ordered = combo_ordered(w, plan, &ordered, t);
        }
    }
    ordered
}

fn engage_any(w: &World, plan: &mut Plan, actor: UnitId, ranked: &[UnitId], cap: i32, profile: &AiProfile) -> bool {
    if plan.ai_pressure.contains(&actor) && fast_strike(w, plan, actor, ranked, cap, profile) {
        return true;
    }
    if try_area(w, plan, actor, cap, profile) {
        return true;
    }
    for &cand in ranked {
        let make = move |s: SkillId| PlanStep::engage(s, cand);
        if try_skills(w, plan, actor, Kind::Attack, cap, profile, Some(&make), None, Some(cand)) {
            return true;
        }
    }
    false
}

fn combo_ordered(w: &World, plan: &Plan, skills: &[SkillId], target: UnitId) -> Vec<SkillId> {
    let t = &w.units[target];
    let guarded = t.stance == rpgkit_rules::Stance::Defensive || held_with(t, counters) || plans_riposte(w, plan, target);
    let (mut first, mut middle, mut last) = (Vec::new(), Vec::new(), Vec::new());
    for &sid in skills {
        let s = &w.cat.skills[sid];
        let breaks = guarded && w.ruleset.stances() && skill::is_stance_break(s);
        // HOOKS1 EX1: an execute that applies now leads too (rule execute)
        let applies = !skill::exploit_match_by(&s.exploit, t, &[], false, &w.ruleset).0.is_empty();
        if breaks || applies {
            first.push(sid);
        } else if !s.exploit.is_empty() {
            last.push(sid);
        } else {
            middle.push(sid);
        }
    }
    first.extend(middle);
    first.extend(last);
    first
}

fn plans_riposte(w: &World, plan: &Plan, foe: UnitId) -> bool {
    plan.steps_of(foe).iter().any(|st| st.skill.is_some_and(|s| skill::kind(&w.cat.skills[s]) == Kind::Defend && counters(&w.cat.skills[s].grant_status)))
}

fn can_strike_in_place(w: &World, actor: UnitId, cap: i32, dist: i32, profile: &AiProfile) -> bool {
    let basic = basic_band(w, actor);
    if dist >= basic.0 && dist <= basic.1 {
        return true;
    }
    if profile.role != Role::Support {
        return false;
    }
    strikes(w, actor, cap).into_iter().any(|sid| {
        let s = &w.cat.skills[sid];
        let b = skill::reach_by(s, &w.units[actor], &w.ruleset);
        s.dash_tiles <= 0 && round_intent::position_of(w, s).is_none() && dist >= b.0 && dist <= b.1
    })
}

fn buff_then_engage(w: &World, plan: &mut Plan, actor: UnitId, ranked: &[UnitId], cap: i32, profile: &AiProfile) -> bool {
    let mut buffs = w.book().available(Kind::Focus, cap, Some(&w.units[actor]));
    buffs.reverse();
    for b in buffs {
        let mark = plan.steps_of(actor).len();
        if !plan.add_step(w, actor, PlanStep::self_skill(b), false).is_empty() {
            continue;
        }
        if engage_any(w, plan, actor, ranked, cap, profile) {
            if focus_safe(w, plan, actor) {
                return true;
            }
            plan.truncate(actor, mark);
            continue;
        }
        plan.remove_step(actor, mark);
    }
    false
}

fn focus_safe(w: &World, plan: &Plan, actor: UnitId) -> bool {
    let st = plan.compile(w, actor);
    let Some(cast) = last_attack(w, &st) else { return true };
    let incoming = incoming_damage(w, plan, actor, cast.end_block(w), cast.pos_before, None);
    (incoming as f64) < FOCUS_PRESSURE_RATIO * w.units[actor].hp as f64
}

fn last_attack<'a>(w: &World, st: &'a SimState) -> Option<&'a PlanEntry> {
    st.entries.iter().rev().find(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Attack)
}

fn fast_strike(w: &World, plan: &mut Plan, actor: UnitId, ranked: &[UnitId], cap: i32, profile: &AiProfile) -> bool {
    let mark = plan.steps_of(actor).len();
    let mut best: Option<Step> = None;
    let mut best_damage = -1;
    if try_area(w, plan, actor, cap, profile) {
        let st = plan.compile(w, actor);
        if let Some(e) = last_attack(w, &st) {
            if e.end_block(w) <= FAST_BLOCK {
                best = Some(plan.steps_of(actor)[mark].clone());
                best_damage = e.est_damage;
            }
        }
        plan.truncate(actor, mark);
    }
    for &cand in ranked {
        let mut strikable = false;
        for sid in skill_order(w, plan, actor, Kind::Attack, cap, profile, Some(cand)) {
            let step = PlanStep::engage(sid, cand);
            if !plan.add_step(w, actor, step.clone(), false).is_empty() {
                continue;
            }
            strikable = true;
            let st = plan.compile(w, actor);
            if let Some(e) = last_attack(w, &st) {
                if e.end_block(w) <= FAST_BLOCK && e.est_damage > best_damage {
                    best = Some(step.clone());
                    best_damage = e.est_damage;
                }
            }
            plan.truncate(actor, mark);
        }
        if strikable {
            break;
        }
    }
    match best {
        Some(step) => plan.add_step(w, actor, step, false).is_empty(),
        None => false,
    }
}

/// ITEMUSE1 rule 9: a heal potion on itself when the expected incoming damage leaves it at or below ITEM_HEAL_RATIO
/// of max HP by round end and no friend plans a heal on it; else a bomb on the weakest foe in the bomb's reach whose
/// HP is at or below the bomb's damage. Elixirs are never AI-used. True when a step was added.
fn try_item(w: &World, plan: &mut Plan, actor: UnitId) -> bool {
    let pos = w.pos(actor);
    let mut potion: Option<&rpgkit_data::ItemDef> = None;
    let mut bomb: Option<&rpgkit_data::ItemDef> = None;
    for id in command::pouch_ids(w, actor) {
        let con = command::pouch_item(w, actor, &id).unwrap();
        if !con.offered_in_battle() || plan.items_left(w, actor, &id, -1) <= 0 {
            continue;
        }
        if potion.is_none() && con.heal > 0 && !con.aims_at_foes() {
            potion = Some(con);
        } else if bomb.is_none() && con.aims_at_foes() {
            bomb = Some(con);
        }
    }
    let u = &w.units[actor];
    if let Some(p) = potion {
        let after = u.hp - incoming_damage(w, plan, actor, w.total_blocks, pos, None);
        if after as f64 <= ITEM_HEAL_RATIO * u.max_hp as f64 && !heal_planned_on(w, plan, actor) && plan.add_step(w, actor, PlanStep::use_item(&p.id, actor), false).is_empty() {
            return true;
        }
    }
    if let Some(b) = bomb {
        let mut best: Option<UnitId> = None;
        for &foe in w.foes_of(actor) {
            let fu = &w.units[foe];
            if !fu.is_active() || w.grid.positions[foe].is_none() || fu.hp > b.damage || pos.dist(w.pos(foe)) > b.battle_reach() {
                continue;
            }
            if best.is_none_or(|x| fu.hp < w.units[x].hp) {
                best = Some(foe);
            }
        }
        if let Some(t) = best {
            if plan.add_step(w, actor, PlanStep::use_item(&b.id, t), false).is_empty() {
                return true;
            }
        }
    }
    false
}

/// True when a friend of `actor` already plans a heal (support skill or heal item) on it this round.
fn heal_planned_on(w: &World, plan: &Plan, actor: UnitId) -> bool {
    for &friend in w.friends_of(actor) {
        if friend == actor {
            continue;
        }
        for e in plan.entries_for(w, friend) {
            let heal = match &e.item {
                Some(i) => i.heal > 0,
                None => skill::heals(&w.cat.skills[e.skill]),
            };
            if e.target == Some(actor) && heal {
                return true;
            }
        }
    }
    false
}

fn try_heal(w: &World, plan: &mut Plan, actor: UnitId, profile: &AiProfile, cap: i32, already: &[UnitId]) -> Option<UnitId> {
    let mut skills: Vec<SkillId> = w.book().available(Kind::Support, cap, Some(&w.units[actor])).into_iter().filter(|s| skill::heals(&w.cat.skills[*s])).collect();
    if skills.is_empty() {
        return None;
    }
    skills.reverse();
    let mut wounded: Vec<UnitId> = w.friends_of(actor).iter().copied().filter(|f| !already.contains(f) && w.units[*f].is_active() && w.has(*f) && (needs_heal(w, *f, profile) || locked_down(w, *f))).collect();
    godot_sort(&mut wounded, |a, b| {
        let (ra, rb) = (hp_ratio(w, *a), hp_ratio(w, *b));
        if !approx_eq(ra, rb) {
            ra < rb
        } else {
            w.units[*a].hp < w.units[*b].hp
        }
    });
    for ally in wounded {
        let hurt = needs_heal(w, ally, profile);
        for &sid in &skills {
            let s = &w.cat.skills[sid];
            if !hurt && !frees_lock(w, ally, sid) {
                continue;
            }
            let step = if ally == actor {
                (s.target_mode != "ALLY").then(|| PlanStep::self_skill(sid))
            } else {
                skill::targets_ally(s).then(|| PlanStep::ally(sid, ally))
            };
            if let Some(step) = step {
                if plan.add_step(w, actor, step, false).is_empty() {
                    return Some(ally);
                }
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn self_care(w: &World, plan: &mut Plan, actor: UnitId, profile: &AiProfile, cap: i32, st: &SimState, foes: &[UnitId], reach: i32) -> Care {
    if (incoming_damage(w, plan, actor, w.total_blocks, st.pos, None) as f64) < SELF_CARE_RATIO * w.units[actor].hp as f64 {
        return Care::None;
    }
    if w.units[actor].hp < w.units[actor].max_hp {
        let mut heals: Vec<SkillId> = w.book().available(Kind::Support, cap, Some(&w.units[actor])).into_iter().filter(|s| skill::heals(&w.cat.skills[*s]) && w.cat.skills[*s].target_mode != "ALLY").collect();
        sort_cheapest_heal(w, &mut heals);
        for sid in heals {
            if plan.add_step(w, actor, PlanStep::self_skill(sid), false).is_empty() {
                return Care::Healed;
            }
        }
    }
    let ranked = rank_targets(w, plan, actor, st, foes, profile);
    if ranked.is_empty() {
        return Care::None;
    }
    let back = retreat_dir(w, plan, actor, st.pos, foes, reach, w.pos(ranked[0]));
    if back != ZERO && plan.add_step(w, actor, PlanStep::mv(w.book().lowest(Kind::Move), back, 1), false).is_empty() {
        return Care::Retreated;
    }
    Care::None
}

fn sort_cheapest_heal(w: &World, heals: &mut [SkillId]) {
    godot_sort(heals, |a, b| {
        let (sa, sb) = (&w.cat.skills[*a], &w.cat.skills[*b]);
        if sa.level != sb.level {
            return sa.level < sb.level;
        }
        if sa.tier != sb.tier {
            return sa.tier < sb.tier;
        }
        sa.id < sb.id
    });
}

#[allow(clippy::too_many_arguments)]
fn kite_then_engage(w: &World, plan: &mut Plan, actor: UnitId, st: &SimState, foes: &[UnitId], ranked: &[UnitId], band: (i32, i32), cap: i32, profile: &AiProfile) -> bool {
    let target_pos = w.pos(ranked[0]);
    let blocked = plan.blocked_cells(w, actor);
    let limit = plan.max_move_tiles(w, actor).min(st.ap_left.max(0));
    if limit < 1 || st.rooted || st.acted {
        return false;
    }
    let cur_near = nearest_foe(w, st.pos, foes);
    let shooters = shooters_of(w, foes);
    // HOOKS1 GH8: a visible ground hazard that would harm the unit costs as a terrain hazard does
    let penalty = |cell: V2| crate::ground::ai_penalty(w, actor, cell);
    let prefer = |cell: V2| cover_pref(w, &shooters, cell);
    let hooks = Hooks { penalty: Some(&penalty), prefer: if shooters.is_empty() { None } else { Some(&prefer) } };
    let mut want = band.1;
    while want >= band.0 {
        if want <= st.pos.dist(target_pos) {
            break;
        }
        let mut goals = Cells::new();
        for dx in -want..=want {
            let dy = want - dx.abs();
            let signs: &[i32] = if dy != 0 { &[1, -1] } else { &[1] };
            for &sy in signs {
                let cell = target_pos + V2::new(dx, dy * sy);
                if cell == st.pos || !w.grid.is_valid(cell) || !w.grid.is_walkable(cell) || w.grid.is_exit(cell) || blocked.contains(&cell) {
                    continue;
                }
                if nearest_foe(w, cell, foes) < band.0.max(cur_near.min(band.1)) {
                    continue;
                }
                goals.insert(cell);
            }
        }
        if !goals.is_empty() {
            let route = path::find_path(&w.grid, st.pos, &goals, &blocked, false, &hooks);
            if !route.is_empty() && route.len() as i32 <= limit {
                let dest = route.iter().fold(st.pos, |a, d| a + *d);
                let mark = plan.steps_of(actor).len();
                let straight = route.iter().all(|d| *d == route[0]);
                let mv = w.book().lowest(Kind::Move);
                let step = if straight { PlanStep::mv(mv, route[0], route.len() as i32) } else { PlanStep::go_to(mv, dest) };
                if plan.add_step(w, actor, step, false).is_empty() {
                    if engage_any(w, plan, actor, ranked, cap, profile) {
                        return true;
                    }
                    plan.remove_step(actor, mark);
                }
            }
        }
        want -= 1;
    }
    false
}

fn advance_toward(w: &World, plan: &mut Plan, actor: UnitId, st: &SimState, ranked: &[UnitId]) -> bool {
    let tiles = plan.max_move_tiles(w, actor).min(st.ap_left.max(0));
    if tiles < 1 || st.rooted || st.acted {
        return false;
    }
    for &cand in ranked {
        let route = plan.approach_path(w, actor, st.pos, cand, -1, -1);
        if route.is_empty() {
            continue;
        }
        let reachable = path::fit_blocks(&w.grid, st.pos, &route, w.units[actor].move_per_block(), MAX_MOVE_BLOCKS);
        let dest = reachable.iter().take(tiles as usize).fold(st.pos, |a, d| a + *d);
        if plan.add_step(w, actor, PlanStep::go_to(w.book().lowest(Kind::Move), dest), false).is_empty() {
            return true;
        }
    }
    false
}

/// Foes best first: a reachable taunter, then nearest (aggressors, reckless) or weakest, route-penalised. HOOKS1 AT1
/// (rule `ai_tactics`): SOFTEST and TOUGHEST rank by `rank_by_build` (without the rule they read as ROLE).
fn rank_targets(w: &World, plan: &Plan, actor: UnitId, st: &SimState, foes: &[UnitId], profile: &AiProfile) -> Vec<UnitId> {
    let pref = w.target_pref(profile);
    if matches!(pref, TargetPref::Softest | TargetPref::Toughest) {
        return rank_by_build(w, plan, actor, st, foes, pref == TargetPref::Softest);
    }
    let nearest_first = profile.reckless || pref == TargetPref::Nearest;
    let attack_reach = basic_band(w, actor).1;
    let mut scored: Vec<(UnitId, f64)> = Vec::new();
    for &f in foes {
        if !w.units[f].is_active() || !w.has(f) || st.est(f) <= 0 {
            continue;
        }
        let d = st.pos.dist(w.pos(f));
        let hp_left = st.est(f) as f64;
        let mut score = if nearest_first { d as f64 * 100.0 + hp_left } else { hp_left + d as f64 * 4.0 };
        let mut reachable = d <= attack_reach;
        if d > attack_reach {
            let route = plan.approach_path(w, actor, st.pos, f, -1, -1);
            reachable = !route.is_empty();
            score += if route.is_empty() { 400.0 } else { route.len() as f64 * 8.0 };
        }
        if active_with(&w.units[f], pulls_targeting) && reachable {
            score -= 100000.0;
        }
        scored.push((f, score));
    }
    godot_sort(&mut scored, |a, b| a.1 < b.1);
    scored.into_iter().map(|(f, _)| f).collect()
}

/// HOOKS1 AT1: a reachable taunter first; then SOFTEST (`softest`): the foes `actor` reaches this round (in reach
/// now, or an approach path within its round's move) before the rest, each group by lowest armour rating, then lowest
/// HP (as planned so far), then nearest (the approach path's length); or TOUGHEST: melee foes before the rest, each by
/// highest max HP, ties nearest. A foe with no path at all comes last either way; full ties keep roster order.
fn rank_by_build(w: &World, plan: &Plan, actor: UnitId, st: &SimState, foes: &[UnitId], softest: bool) -> Vec<UnitId> {
    let attack_reach = basic_band(w, actor).1;
    let move_tiles = plan.max_move_tiles(w, actor) as usize;
    let mut keyed: Vec<((i32, i32, i32, i32, i32), UnitId)> = Vec::new();
    for &f in foes {
        if !w.units[f].is_active() || !w.has(f) || st.est(f) <= 0 {
            continue;
        }
        let d = st.pos.dist(w.pos(f));
        let (pathless, near, this_round) = if d <= attack_reach {
            (false, d, true)
        } else {
            let route = plan.approach_path(w, actor, st.pos, f, -1, -1);
            (route.is_empty(), if route.is_empty() { d } else { d.max(route.len() as i32) }, !route.is_empty() && route.len() <= move_tiles)
        };
        let taunter = active_with(&w.units[f], pulls_targeting) && !pathless;
        let u = &w.units[f];
        let key = if softest {
            (i32::from(!taunter), i32::from(pathless || !this_round), damage::armor_rating(u), st.est(f), near)
        } else {
            (i32::from(!taunter), i32::from(pathless), i32::from(u.attack_range > 1), -u.max_hp, near)
        };
        keyed.push((key, f));
    }
    keyed.sort_by_key(|&(k, f)| (k, f));
    keyed.into_iter().map(|(_, f)| f).collect()
}

fn retreat_dir(w: &World, plan: &Plan, actor: UnitId, pos: V2, foes: &[UnitId], reach: i32, target_pos: V2) -> V2 {
    let blocked = plan.blocked_cells(w, actor);
    let mut best = ZERO;
    let mut best_gain = 0;
    let cur = nearest_foe(w, pos, foes);
    for d in DIRS {
        let n = pos + d;
        if !w.grid.is_valid(n) || !w.grid.is_walkable(n) || w.grid.is_exit(n) || blocked.contains(&n) {
            continue;
        }
        if n.dist(target_pos) > reach {
            continue;
        }
        let gain = nearest_foe(w, n, foes) - cur;
        if gain > best_gain {
            best_gain = gain;
            best = d;
        }
    }
    best
}

fn nearest_foe(w: &World, pos: V2, foes: &[UnitId]) -> i32 {
    foes.iter().filter(|f| w.units[**f].is_active() && w.has(**f)).map(|f| pos.dist(w.pos(*f))).fold(99, i32::min)
}

fn nearest_foe_unit(w: &World, pos: V2, foes: &[UnitId]) -> Option<UnitId> {
    let mut best = None;
    let mut best_d = 99;
    for &f in foes {
        if w.units[f].is_active() && w.has(f) {
            let d = pos.dist(w.pos(f));
            if d < best_d {
                best_d = d;
                best = Some(f);
            }
        }
    }
    best
}

/// Single-target damaging attacks (zero-power debuff attacks are cast by precast).
fn strikes(w: &World, actor: UnitId, cap: i32) -> Vec<SkillId> {
    w.book().single_attacks(&w.units[actor], cap).into_iter().filter(|s| skill::deals_damage(&w.cat.skills[*s])).collect()
}

fn ordered(w: &World, skills: &[SkillId], profile: &AiProfile) -> Vec<SkillId> {
    let mut out = skills.to_vec();
    out.reverse();
    sort_by_preferred_level(w, &mut out, profile);
    out
}

fn optional(w: &World, skills: Vec<SkillId>, profile: &AiProfile) -> Vec<SkillId> {
    if profile.prefer_level <= 0 {
        return skills;
    }
    skills.into_iter().filter(|s| w.cat.skills[*s].level <= profile.prefer_level).collect()
}

fn needs_heal(w: &World, f: UnitId, profile: &AiProfile) -> bool {
    let u = &w.units[f];
    u.hp < u.max_hp && hp_ratio(w, f) <= profile.heal_below
}

/// S1.2-S4 (FB-48): `c` is guarded for the whole round already (an active `guard` status: a multi-round Guard skill's
/// later round), so a Defend picked to be safe would be redundant.
pub fn guard_held(w: &World, c: UnitId) -> bool {
    w.units[c].has_status(status::GUARD)
}

fn locked_down(w: &World, c: UnitId) -> bool {
    held_with(&w.units[c], stops_moving)
}

fn frees_lock(w: &World, c: UnitId, sid: SkillId) -> bool {
    cleansable(&w.units[c], &w.cat.skills[sid]).into_iter().any(stops_moving)
}

/// I1.2-S1 (docs/sessions/I_AI.md section 1): the AI's status reads, each a behaviour predicate over the kit status
/// table (`rpgkit_rules::status`) instead of a status id. A status that pulls enemy targeting toward its holder.
pub fn pulls_targeting(status: &str) -> bool {
    status::has_targeting_pull(status)
}

/// I1.2-S1: a reactive counter status (held on a Defend skill).
pub fn counters(status: &str) -> bool {
    status::has_reactive_counter(status)
}

/// I1.2-S1: a heal-over-time status.
pub fn heals_over_time(status: &str) -> bool {
    status::has_tick(status, status::Tick::Heal)
}

/// I1.2-S1: a status that stops its holder moving this round (a block lock over every move block, or on Move itself).
pub fn stops_moving(status: &str) -> bool {
    status::locks_movement(status, MAX_MOVE_BLOCKS)
}

/// I1.2-S1: a buff raising Armor Rating.
pub fn raises_armor(status: &str) -> bool {
    status::modifies_rating(status, status::Rating::Armor) && status::is_buff(status)
}

/// I1.2-S1: a debuff lowering Armor Rating.
pub fn lowers_armor(status: &str) -> bool {
    status::modifies_rating(status, status::Rating::Armor) && !status::is_buff(status)
}

/// I1.2-S1: a debuff lowering Move Rate.
pub fn lowers_move_rate(status: &str) -> bool {
    status::modifies_rating(status, status::Rating::MoveRate) && !status::is_buff(status)
}

/// Whether `u` has an active status passing `read` (the predicate form of `Unit::has_status`).
fn active_with(u: &rpgkit_rules::Unit, read: fn(&str) -> bool) -> bool {
    u.st.active.iter().any(|s| read(s))
}

/// Whether `u` has an active or queued status passing `read` (the predicate form of `StatusState::has_any`).
fn held_with(u: &rpgkit_rules::Unit, read: fn(&str) -> bool) -> bool {
    u.st.active.iter().chain(u.st.pending.iter()).any(|s| read(s))
}

fn try_disengage(w: &World, plan: &mut Plan, actor: UnitId, cap: i32, profile: &AiProfile, ranked: &[UnitId], require_strike: bool) -> bool {
    let skills = ordered(w, &w.book().available(Kind::Disengage, cap, Some(&w.units[actor])), profile);
    for sid in skills {
        let mark = plan.steps_of(actor).len();
        if plan.add_step(w, actor, PlanStep::self_skill(sid), false).is_empty() {
            if !require_strike || engage_any(w, plan, actor, ranked, cap, profile) {
                return true;
            }
            plan.remove_step(actor, mark);
        }
        if !require_strike || ranked.is_empty() {
            continue;
        }
        for cell in landing_cells(w, plan, actor, sid, ranked[0]) {
            let jump = PlanStep::self_skill(sid);
            jump.cell.set(cell);
            if !plan.add_step(w, actor, jump, false).is_empty() {
                continue;
            }
            if engage_any(w, plan, actor, ranked, cap, profile) {
                return true;
            }
            plan.remove_step(actor, mark);
        }
    }
    false
}

fn landing_cells(w: &World, plan: &Plan, actor: UnitId, sid: SkillId, target: UnitId) -> Vec<V2> {
    let st = plan.compile(w, actor);
    let band = basic_band(w, actor);
    let target_pos = w.pos(target);
    let here = st.pos.dist(target_pos);
    let blocked = plan.blocked_cells(w, actor);
    let reachable = plan.jump_tiles_from(w, st.pos, w.cat.skills[sid].jump_tiles);
    let mut scored: Vec<(V2, i32, i32)> = Vec::new();
    for (&cell, &steps) in &reachable {
        let d = cell.dist(target_pos);
        if blocked.contains(&cell) || d <= here || d < band.0 || d > band.1 {
            continue;
        }
        scored.push((cell, d, steps));
    }
    godot_sort(&mut scored, |a, b| {
        if a.1 != b.1 {
            return a.1 > b.1;
        }
        if a.2 != b.2 {
            return a.2 < b.2;
        }
        if a.0.y != b.0.y {
            a.0.y < b.0.y
        } else {
            a.0.x < b.0.x
        }
    });
    scored.into_iter().take(4).map(|s| s.0).collect()
}

fn try_cleanse(w: &World, plan: &mut Plan, actor: UnitId, cap: i32, urgent: bool) -> bool {
    let skills = w.book().cleanse_only(cap, Some(&w.units[actor]));
    if skills.is_empty() {
        return false;
    }
    let friends = w.friends_of(actor).to_vec();
    let mut scored: Vec<(UnitId, usize, f64, usize)> = Vec::new();
    for (i, &f) in friends.iter().enumerate() {
        let index = i + 1;
        if !w.units[f].is_active() || !w.has(f) {
            continue;
        }
        let mut needed: Vec<&str> = Vec::new();
        for &sid in &skills {
            if !command::target_problem(w, actor, sid, Some(f)).is_empty() {
                continue;
            }
            for st in cleansable(&w.units[f], &w.cat.skills[sid]) {
                if !needed.contains(&st) {
                    needed.push(st);
                }
            }
        }
        if needed.is_empty() || (urgent && !needed.iter().any(|s| stops_moving(s))) {
            continue;
        }
        scored.push((f, needed.len(), hp_ratio(w, f), index));
    }
    godot_sort(&mut scored, |a, b| {
        if a.1 != b.1 {
            return a.1 > b.1;
        }
        if !approx_eq(a.2, b.2) {
            return a.2 < b.2;
        }
        a.3 < b.3
    });
    for (patient, total, _, _) in scored {
        let mut eligible: Vec<SkillId> = skills.iter().copied().filter(|s| command::target_problem(w, actor, *s, Some(patient)).is_empty() && !cleansable(&w.units[patient], &w.cat.skills[*s]).is_empty()).collect();
        godot_sort(&mut eligible, |a, b| {
            let (sa, sb) = (&w.cat.skills[*a], &w.cat.skills[*b]);
            if sa.tier != sb.tier {
                return sa.tier < sb.tier;
            }
            if sa.level != sb.level {
                return sa.level < sb.level;
            }
            sa.id < sb.id
        });
        let mut order: Vec<SkillId> = Vec::new();
        let mut others: Vec<SkillId> = Vec::new();
        for sid in eligible {
            if cleansable(&w.units[patient], &w.cat.skills[sid]).len() >= total {
                order.push(sid);
            } else {
                others.push(sid);
            }
        }
        others.reverse();
        order.extend(others);
        for sid in order {
            let step = if patient == actor { PlanStep::self_skill(sid) } else { PlanStep::ally(sid, patient) };
            if plan.add_step(w, actor, step, false).is_empty() {
                return true;
            }
        }
    }
    false
}

/// Taunt, an ally buff, a debuff attack: each once per round, kept only when a strike still fits afterwards.
fn precast(w: &World, plan: &mut Plan, actor: UnitId, ranked: &[UnitId], cap: i32, profile: &AiProfile) -> bool {
    let mut added = false;
    for kind in 0..3 {
        let options = match kind {
            0 => taunt_options(w, plan, actor, cap, profile),
            1 => buff_options(w, plan, actor, cap, profile),
            _ => debuff_options(w, plan, actor, cap, profile, ranked),
        };
        let mut kind_added = false;
        for step in &options {
            let mark = plan.steps_of(actor).len();
            if !plan.add_step(w, actor, step.clone(), false).is_empty() {
                continue;
            }
            if engage_any(w, plan, actor, ranked, cap, profile) {
                plan.remove_step(actor, mark + 1);
                added = true;
                kind_added = true;
                break;
            }
            plan.remove_step(actor, mark);
        }
        if kind == 0 && !kind_added && !options.is_empty() && early_taunt_walk(w, plan, actor, &options) {
            added = true;
        }
    }
    added
}

fn early_taunt_walk(w: &World, plan: &mut Plan, actor: UnitId, taunts: &[Step]) -> bool {
    let st = plan.compile(w, actor);
    if st.moved || st.acted || st.rooted {
        return false;
    }
    let mut diver = None;
    let mut best = 1 << 20;
    for foe in divers(w, plan, actor) {
        let d = st.pos.dist(w.pos(foe));
        if d < best {
            best = d;
            diver = Some(foe);
        }
    }
    let Some(diver) = diver else { return false };
    let route = plan.approach_path(w, actor, st.pos, diver, -1, -1);
    if route.is_empty() {
        return false;
    }
    let reachable = path::fit_blocks(&w.grid, st.pos, &route, w.units[actor].move_per_block(), MAX_MOVE_BLOCKS);
    let tiles = (reachable.len() as i32).min(plan.max_move_tiles(w, actor).min(st.ap_left.max(0)));
    let mv = w.book().lowest(Kind::Move);
    let mark = plan.steps_of(actor).len();
    let mut n = tiles;
    while n > 0 {
        let dest = reachable.iter().take(n as usize).fold(st.pos, |a, d| a + *d);
        if plan.add_step(w, actor, PlanStep::go_to(mv, dest), false).is_empty() {
            for t in taunts {
                if plan.add_step(w, actor, t.clone(), false).is_empty() {
                    return true;
                }
            }
            plan.truncate(actor, mark);
        }
        n -= 1;
    }
    false
}

fn divers(w: &World, plan: &Plan, actor: UnitId) -> Vec<UnitId> {
    let friends = w.friends_of(actor).to_vec();
    let where_ = plan.planned_positions(w, &friends);
    let mut out = Vec::new();
    for &foe in w.foes_of(actor) {
        if !w.units[foe].is_active() || !w.has(foe) {
            continue;
        }
        for &ally in &friends {
            if ally != actor && w.units[ally].is_active() && w.has(ally) && likely_target(w, plan, foe, ally, plan.planned_position(w, ally), &where_) {
                out.push(foe);
                break;
            }
        }
    }
    out
}

fn taunt_options(w: &World, plan: &Plan, actor: UnitId, cap: i32, profile: &AiProfile) -> Vec<Step> {
    if held_with(&w.units[actor], pulls_targeting) {
        return Vec::new();
    }
    let foes = w.foes_of(actor).to_vec();
    let friends = w.friends_of(actor).to_vec();
    let where_ = plan.planned_positions(w, &friends);
    let mut exposed = false;
    for &ally in &friends {
        if exposed || ally == actor || !w.units[ally].is_active() || !w.has(ally) {
            continue;
        }
        let at = plan.planned_position(w, ally);
        exposed = nearest_foe(w, at, &foes) <= 3 || picked_by_a_foe(w, plan, ally, at, &foes, &where_);
    }
    if !exposed {
        return Vec::new();
    }
    let skills = ordered(w, &optional(w, w.book().granting(Kind::Support, pulls_targeting, cap, Some(&w.units[actor])), profile), profile);
    skills.into_iter().filter(|s| w.cat.skills[*s].target_mode != "ALLY").map(PlanStep::self_skill).collect()
}

fn picked_by_a_foe(w: &World, plan: &Plan, ally: UnitId, at: V2, foes: &[UnitId], where_: &HashMap<UnitId, V2>) -> bool {
    foes.iter().any(|&f| w.units[f].is_active() && w.has(f) && likely_target(w, plan, f, ally, at, where_))
}

fn buff_options(w: &World, plan: &Plan, actor: UnitId, cap: i32, profile: &AiProfile) -> Vec<Step> {
    let mut out = Vec::new();
    for sid in ordered(w, &optional(w, w.book().ally_buffs(pulls_targeting, cap, Some(&w.units[actor])), profile), profile) {
        for t in buff_targets(w, plan, actor, sid, profile) {
            out.push(if t == actor { PlanStep::self_skill(sid) } else { PlanStep::ally(sid, t) });
        }
    }
    out
}

/// The friends buff skill `sid` is tried on, in order: the Armor buff and the heal over time their own orders; W2-5
/// I1.2-S2 (ruleset `ai_rating_scorer`) the Attack Power and Move Rate buffs theirs (`rating_buff_targets`); any other
/// buff the friends with no foe within one tile past their basic reach, farthest first.
fn buff_targets(w: &World, plan: &Plan, actor: UnitId, sid: SkillId, profile: &AiProfile) -> Vec<UnitId> {
    let s = &w.cat.skills[sid];
    if raises_armor(&s.grant_status) {
        return fortify_targets(w, plan, actor, sid);
    }
    if heals_over_time(&s.grant_status) {
        return regen_targets(w, plan, actor, sid, profile);
    }
    if let Some(picked) = rating_buff_targets(w, plan, actor, sid) {
        return picked;
    }
    let foes = w.foes_of(actor).to_vec();
    let mut cands: Vec<(UnitId, i32, usize)> = Vec::new();
    for (i, &f) in w.friends_of(actor).iter().enumerate() {
        if !w.units[f].is_active() || !w.has(f) {
            continue;
        }
        if (f == actor && s.target_mode == "ALLY") || (f != actor && s.target_mode == "SELF") {
            continue;
        }
        if w.units[f].st.has_any(&s.grant_status) {
            continue;
        }
        let d = nearest_foe(w, plan.planned_position(w, f), &foes);
        if d <= basic_band(w, f).1 + 1 {
            continue;
        }
        cands.push((f, d, i + 1));
    }
    godot_sort(&mut cands, |a, b| if a.1 != b.1 { a.1 > b.1 } else { a.2 < b.2 });
    cands.into_iter().map(|c| c.0).collect()
}

/// W2-5 I1.2-S2 test hook: the units the classic AI tries status skill `sid` on, in order (a buff: `buff_targets`; an
/// attack with a status effect: `debuff_targets`, with the foes ranked from `actor`'s tile).
#[doc(hidden)]
pub fn status_targets(w: &World, plan: &Plan, actor: UnitId, sid: SkillId) -> Vec<UnitId> {
    let profile = w.profile_of(actor);
    if skill::kind(&w.cat.skills[sid]) == Kind::Attack {
        let st = plan.compile(w, actor);
        let ranked = rank_targets(w, plan, actor, &st, w.foes_of(actor), &profile);
        return debuff_targets(w, plan, actor, sid, &ranked);
    }
    buff_targets(w, plan, actor, sid, &profile)
}

fn buff_candidates(w: &World, actor: UnitId, sid: SkillId) -> Vec<UnitId> {
    let s = &w.cat.skills[sid];
    w.friends_of(actor)
        .iter()
        .copied()
        .filter(|&f| w.units[f].is_active() && w.has(f) && !((f == actor && s.target_mode == "ALLY") || (f != actor && s.target_mode == "SELF")) && !w.units[f].st.has_any(&s.grant_status))
        .collect()
}

fn fortify_targets(w: &World, plan: &Plan, actor: UnitId, sid: SkillId) -> Vec<UnitId> {
    let foes = w.foes_of(actor).to_vec();
    let mut cands: Vec<(UnitId, i32, i32, i32, usize)> = Vec::new();
    for (i, f) in buff_candidates(w, actor, sid).into_iter().enumerate() {
        let pos = plan.planned_position(w, f);
        let adjacent = foes.iter().filter(|&&e| w.units[e].is_active() && w.has(e) && pos.dist(w.pos(e)) <= 1).count() as i32;
        let taunting = held_with(&w.units[f], pulls_targeting) as i32;
        cands.push((f, adjacent, taunting, damage::armor_rating(&w.units[f]), i + 1));
    }
    godot_sort(&mut cands, |a, b| {
        if a.1 != b.1 {
            return a.1 > b.1;
        }
        if a.2 != b.2 {
            return a.2 > b.2;
        }
        if a.3 != b.3 {
            return a.3 < b.3;
        }
        a.4 < b.4
    });
    cands.into_iter().map(|c| c.0).collect()
}

/// W2-5 I1.2-S2 (I_AI section 1 item 5, FB-52b): whether the AI scores rating modifiers generically (ruleset
/// `ai_rating_scorer`, a plain tunable, default off). Off: only the Armor and Move Rate debuffs and the Armor buff have
/// a target order (I1.2-S1's 1-to-1 reads); on: every `modifier` row by its rating and direction as well.
pub fn rating_scorer(w: &World) -> bool {
    w.ruleset.numbers.ai_rating_scorer
}

/// Whether foe `foe` can reach a tile within its basic reach of `at` this round (its round's move tiles plus that
/// reach): the same threat test as the Move Rate debuff's.
fn threatens(w: &World, plan: &Plan, foe: UnitId, at: V2) -> bool {
    w.pos(foe).dist(at) <= plan.max_move_tiles(w, foe) + basic_band(w, foe).1
}

/// W2-5 I1.2-S2: the target order of a buff skill whose status raises Attack Power or Move Rate, under
/// `ai_rating_scorer` (None otherwise: the buff takes the generic order, as before). Among the friends the skill may
/// land on that lack the status (`buff_candidates`):
/// - Attack Power (`empower`): the friends that can reach a foe this round (from their planned tile, their round's
///   move tiles plus basic reach), hardest hitter first
///   (`damage::power_of`), ties roster order;
/// - Move Rate (`haste`): the friends with no foe within their basic reach, the one nearest to a foe first (the move
///   gets it into the fight soonest), ties roster order.
fn rating_buff_targets(w: &World, plan: &Plan, actor: UnitId, sid: SkillId) -> Option<Vec<UnitId>> {
    let st = &w.cat.skills[sid].grant_status;
    if !rating_scorer(w) {
        return None;
    }
    let foes: Vec<UnitId> = w.foes_of(actor).iter().copied().filter(|&e| w.units[e].is_active() && w.has(e)).collect();
    let cands = buff_candidates(w, actor, sid);
    if status::raises_rating(st, status::Rating::AttackPower) {
        let mut keyed: Vec<(i32, usize, UnitId)> = cands
            .into_iter()
            .enumerate()
            .filter(|&(_, f)| {
                let (at, reach) = (plan.planned_position(w, f), plan.max_move_tiles(w, f) + basic_band(w, f).1);
                foes.iter().any(|&e| w.pos(e).dist(at) <= reach)
            })
            .map(|(i, f)| {
                let u = &w.units[f];
                (-damage::power_of(u, u.attack_range > 1, false), i, f)
            })
            .collect();
        keyed.sort();
        return Some(keyed.into_iter().map(|k| k.2).collect());
    }
    if status::raises_rating(st, status::Rating::MoveRate) {
        let mut keyed: Vec<(i32, usize, UnitId)> = cands
            .into_iter()
            .enumerate()
            .filter_map(|(i, f)| {
                let d = nearest_foe(w, plan.planned_position(w, f), &foes);
                (d > basic_band(w, f).1).then_some((d, i, f))
            })
            .collect();
        keyed.sort();
        return Some(keyed.into_iter().map(|k| k.2).collect());
    }
    None
}

fn regen_targets(w: &World, _plan: &Plan, actor: UnitId, sid: SkillId, profile: &AiProfile) -> Vec<UnitId> {
    let mut out: Vec<UnitId> = buff_candidates(w, actor, sid).into_iter().filter(|f| needs_heal(w, *f, profile)).collect();
    godot_sort(&mut out, |a, b| {
        let (ra, rb) = (hp_ratio(w, *a), hp_ratio(w, *b));
        if !approx_eq(ra, rb) {
            ra < rb
        } else {
            w.units[*a].hp < w.units[*b].hp
        }
    });
    out
}

fn debuff_open(w: &World, foe: UnitId, st: &str) -> bool {
    let u = &w.units[foe];
    if status::stacking_of(st) == status::Stacking::StackPotency {
        let cap = status::cap_of(st);
        return cap <= 0 || u.st.magnitudes.get(st).copied().unwrap_or(0) + u.st.pending_magnitudes.get(st).copied().unwrap_or(0) < cap;
    }
    !u.st.has_any(st)
}

fn fragile_friend(w: &World, actor: UnitId) -> Option<UnitId> {
    let mut best: Option<UnitId> = None;
    for &f in w.friends_of(actor) {
        if w.units[f].is_active() && w.has(f) && best.is_none_or(|b| w.units[f].max_hp < w.units[b].max_hp) {
            best = Some(f);
        }
    }
    best
}

fn debuff_order(w: &World, plan: &Plan, actor: UnitId, st: &str, by_power: &[UnitId], ranked: &[UnitId]) -> Vec<UnitId> {
    let mut first: Vec<UnitId> = Vec::new();
    if lowers_armor(st) {
        if let Some(&r) = ranked.first() {
            if by_power.contains(&r) {
                first.push(r);
            }
        }
        for &foe in by_power {
            if first.contains(&foe) {
                continue;
            }
            for &f in w.friends_of(actor) {
                if f != actor && w.units[f].is_active() && w.has(f) && plan.planned_position(w, f).dist(w.pos(foe)) <= 1 {
                    first.push(foe);
                    break;
                }
            }
        }
    } else if lowers_move_rate(st) {
        if let Some(fragile) = fragile_friend(w, actor) {
            let target_pos = plan.planned_position(w, fragile);
            let actor_pos = w.pos(actor);
            for &foe in by_power {
                let threat = plan.max_move_tiles(w, foe) + basic_band(w, foe).1;
                if w.pos(foe).dist(target_pos) <= threat {
                    first.push(foe);
                }
            }
            godot_sort(&mut first, |a, b| {
                let (da, db) = (actor_pos.dist(w.pos(*a)), actor_pos.dist(w.pos(*b)));
                if da != db {
                    da < db
                } else {
                    by_power.iter().position(|x| x == a) < by_power.iter().position(|x| x == b)
                }
            });
        }
    } else if rating_scorer(w) && status::lowers_rating(st, status::Rating::AttackPower) {
        // W2-5 I1.2-S2 (`weaken`): the foes that threaten a friend this round first, hardest hitter first (`by_power`)
        let friends: Vec<V2> = w.friends_of(actor).iter().copied().filter(|&f| w.units[f].is_active() && w.has(f)).map(|f| plan.planned_position(w, f)).collect();
        first = by_power.iter().copied().filter(|&foe| friends.iter().any(|&at| threatens(w, plan, foe, at))).collect();
    }
    let mut out = first.clone();
    for &foe in by_power {
        if !out.contains(&foe) {
            out.push(foe);
        }
    }
    out
}

fn debuff_options(w: &World, plan: &Plan, actor: UnitId, cap: i32, profile: &AiProfile, ranked: &[UnitId]) -> Vec<Step> {
    let skills = ordered(w, &optional(w, w.book().debuff_attacks(cap, Some(&w.units[actor])), profile), profile);
    let mut out = Vec::new();
    for sid in skills {
        for foe in debuff_targets(w, plan, actor, sid, ranked) {
            out.push(PlanStep::engage(sid, foe));
        }
    }
    out
}

/// The foes debuff attack `sid` is tried on, in order: `debuff_order` over the foes by power, those its status still
/// lands on (`debuff_open`).
fn debuff_targets(w: &World, plan: &Plan, actor: UnitId, sid: SkillId, ranked: &[UnitId]) -> Vec<UnitId> {
    let st = plan.compile(w, actor);
    let mut scored: Vec<(UnitId, i32, usize)> = Vec::new();
    for (i, &foe) in w.foes_of(actor).iter().enumerate() {
        if !w.units[foe].is_active() || !w.has(foe) || st.est(foe) <= 0 {
            continue;
        }
        let u = &w.units[foe];
        scored.push((foe, damage::power_of(u, u.attack_range > 1, false), i + 1));
    }
    godot_sort(&mut scored, |a, b| if a.1 != b.1 { a.1 > b.1 } else { a.2 < b.2 });
    let by_power: Vec<UnitId> = scored.into_iter().map(|s| s.0).collect();
    let effect = &w.cat.skills[sid].status_effect;
    debuff_order(w, plan, actor, effect, &by_power, ranked).into_iter().filter(|&foe| debuff_open(w, foe, effect)).collect()
}

fn guard_smart(w: &World, plan: &mut Plan, actor: UnitId, cap: i32, profile: &AiProfile, foes: &[UnitId], pos: V2) -> bool {
    let near = nearest_foe_unit(w, pos, foes);
    let melee = near.is_some_and(|n| w.units[n].attack_range <= 1);
    let holding = held_with(&w.units[actor], counters);
    let riposte_only = |s: SkillId| counters(&w.cat.skills[s].grant_status);
    let plain_only = |s: SkillId| !counters(&w.cat.skills[s].grant_status);
    let order: Vec<&dyn Fn(SkillId) -> bool> = if holding {
        vec![&plain_only]
    } else if melee {
        vec![&riposte_only, &plain_only]
    } else {
        vec![&plain_only, &riposte_only]
    };
    for filter in order {
        if try_skills(w, plan, actor, Kind::Defend, cap, profile, None, Some(filter), None) {
            return true;
        }
    }
    false
}

fn riposte_finisher(w: &World, plan: &mut Plan, actor: UnitId, cap: i32, profile: &AiProfile, foes: &[UnitId], ratio: f64) -> bool {
    if profile.reckless || profile.defend_below <= 0.0 || ratio <= profile.defend_below || held_with(&w.units[actor], counters) {
        return false;
    }
    let st = plan.compile(w, actor);
    let near = nearest_foe_unit(w, st.pos, foes);
    let Some(near) = near else { return false };
    if st.withdrawn || w.units[near].attack_range > 1 || st.pos.dist(w.pos(near)) != 1 {
        return false;
    }
    for sid in ordered(w, &optional(w, w.book().granting(Kind::Defend, counters, cap, Some(&w.units[actor])), profile), profile) {
        if plan.add_step(w, actor, PlanStep::self_skill(sid), false).is_empty() {
            return true;
        }
    }
    false
}

fn try_area(w: &World, plan: &mut Plan, actor: UnitId, cap: i32, profile: &AiProfile) -> bool {
    let skills: Vec<SkillId> = optional(w, w.book().area_attacks(cap, Some(&w.units[actor])), profile).into_iter().filter(|s| skill::deals_damage(&w.cat.skills[*s])).collect();
    if skills.is_empty() {
        return false;
    }
    let st = plan.compile(w, actor);
    if st.withdrawn || st.pos == NO_CELL {
        return false;
    }
    let foes = w.foes_of(actor).to_vec();
    let friends = w.friends_of(actor).to_vec();
    // HOOKS1 AT2 `flush_cover` (rule ai_tactics): a foe hit on a tile with cover against the actor scores once more
    let flush = w.has_tactic(actor, UnitTactic::FlushCover);
    let mut cands: Vec<(SkillId, V2, f64, usize)> = Vec::new();
    for &sid in &skills {
        let s = &w.cat.skills[sid];
        let band = skill::reach_by(s, &w.units[actor], &w.ruleset);
        // A3.2-K2 (IR9): an area hits only the units its `affects` names in this round's row
        let affects = skill::affects_by(s, round_intent::intent_of(w, actor));
        let (on_foes, on_friends) = (skill::affects_unit(affects, false), skill::affects_unit(affects, true));
        for aim in aim_cells(w, st.pos, sid, band) {
            let cells: Vec<V2> = skill::area_cells(s, st.pos, aim).into_iter().filter(|c| w.grid.is_valid(*c) && *c != st.pos).collect();
            let hit: Vec<UnitId> = foes.iter().copied().filter(|&f| on_foes && w.units[f].is_active() && w.has(f) && st.est(f) > 0 && cells.contains(&w.pos(f))).collect();
            let foes_hit = hit.len() as i32;
            let hit_friends: Vec<UnitId> = friends.iter().copied().filter(|&f| on_friends && f != actor && w.units[f].is_active() && cells.contains(&plan.planned_position(w, f))).collect();
            if foes_hit < profile.area_threshold() || !friendly_fire_ok(w, actor, sid, &hit_friends, foes_hit) {
                continue;
            }
            let covered = if flush { hit.iter().filter(|&&f| command::cover_for(&w.grid, st.pos, w.pos(f), true) > 0.0).count() as i32 } else { 0 };
            cands.push((sid, aim, w.units[actor].power_for(s) * (foes_hit + covered) as f64, hit_friends.len()));
        }
    }
    godot_sort(&mut cands, |a, b| {
        if !approx_eq(a.2, b.2) {
            return a.2 > b.2;
        }
        if a.3 != b.3 {
            return a.3 < b.3;
        }
        let (la, lb) = (w.cat.skills[a.0].level, w.cat.skills[b.0].level);
        if la != lb {
            return la > lb;
        }
        if a.1.y != b.1.y {
            a.1.y < b.1.y
        } else {
            a.1.x < b.1.x
        }
    });
    for (sid, aim, _, _) in cands {
        if plan.add_step(w, actor, PlanStep::area_at(sid, aim), false).is_empty() {
            return true;
        }
    }
    false
}

fn friendly_fire_ok(w: &World, actor: UnitId, sid: SkillId, hit_friends: &[UnitId], foes_hit: i32) -> bool {
    if hit_friends.is_empty() {
        return true;
    }
    if hit_friends.len() > 1 || foes_hit < AREA_FF_MIN_FOES {
        return false;
    }
    let ally = hit_friends[0];
    (expected(w, actor, ally, sid, None, 0.0, 1.0) as f64) < AREA_FF_HP_RATIO * w.units[ally].hp as f64
}

fn aim_cells(w: &World, pos: V2, sid: SkillId, band: (i32, i32)) -> Vec<V2> {
    let s = &w.cat.skills[sid];
    match skill::area(s) {
        skill::Area::Line => DIRS.into_iter().map(|d| pos + d).filter(|c| w.grid.is_valid(*c)).collect(),
        // A3.2-S1: ARC (A3.2-K2: and CONE) toward each neighbouring tile; RING has one aim (the user's tile: the ring is
        // drawn around it).
        skill::Area::Arc | skill::Area::Cone => skill::ring_cells(pos).into_iter().filter(|c| w.grid.is_valid(*c)).collect(),
        skill::Area::Ring => vec![pos],
        _ => {
            let mut out = Vec::new();
            for dx in -band.1..=band.1 {
                for dy in -band.1..=band.1 {
                    let cell = pos + V2::new(dx, dy);
                    let d = pos.dist(cell);
                    if w.grid.is_valid(cell) && d >= band.0 && d <= band.1 {
                        out.push(cell);
                    }
                }
            }
            out
        }
    }
}

/// Damage `target` (at `at`) is expected to take from the foes that would pick it, by the end of `by_block`.
pub fn incoming_damage(w: &World, plan: &Plan, target: UnitId, by_block: i32, at: V2, positions: Option<&HashMap<UnitId, V2>>) -> i32 {
    if at == NO_CELL {
        return 0;
    }
    let owned;
    let where_ = match positions.filter(|p| !p.is_empty()) {
        Some(p) => p,
        None => {
            owned = plan.planned_positions(w, w.friends_of(target));
            &owned
        }
    };
    let mut total = 0;
    for &foe in w.foes_of(target) {
        if w.units[foe].is_active() && w.has(foe) && likely_target(w, plan, foe, target, at, where_) {
            total += strike_chain(w, foe, target, at, by_block);
        }
    }
    total
}

fn likely_target(w: &World, plan: &Plan, foe: UnitId, target: UnitId, at: V2, where_: &HashMap<UnitId, V2>) -> bool {
    let foe_pos = w.pos(foe);
    let threat = plan.max_move_tiles(w, foe) + basic_band(w, foe).1;
    let p = w.profile_of(foe);
    let nearest_first = p.reckless || p.target_pref == TargetPref::Nearest;
    let mut best = f64::INFINITY;
    let mut mine = f64::INFINITY;
    for &u in w.foes_of(foe) {
        if !w.units[u].is_active() || !w.has(u) {
            continue;
        }
        let pos = if u == target { at } else { where_.get(&u).copied().unwrap_or(NO_CELL) };
        if pos == NO_CELL {
            continue;
        }
        let d = foe_pos.dist(pos);
        if d > threat {
            continue;
        }
        let hp = w.units[u].hp as f64;
        let mut score = if nearest_first { d as f64 * 100.0 + hp } else { hp + d as f64 * 4.0 };
        if active_with(&w.units[u], pulls_targeting) {
            score -= 100000.0;
        }
        best = best.min(score);
        if u == target {
            mine = score;
        }
    }
    mine < f64::INFINITY && mine <= best
}

fn strike_chain(w: &World, foe: UnitId, target: UnitId, at: V2, by_block: i32) -> i32 {
    let d = w.pos(foe).dist(at);
    let mut best = 0;
    let fu = &w.units[foe];
    for sid in strikes(w, foe, w.skill_cap_of(foe)) {
        let s = &w.cat.skills[sid];
        let band = skill::reach_by(s, fu, &w.ruleset);
        let tiles = if d > band.1 {
            d - band.1
        } else if d < band.0 {
            band.0 - d
        } else {
            0
        };
        let move_blocks = if tiles == 0 { 0 } else { (tiles as f64 / fu.move_per_block().max(1) as f64).ceil() as i32 };
        if move_blocks > MAX_MOVE_BLOCKS as i32 {
            continue;
        }
        let mut hits = ((by_block - move_blocks) as f64 / s.blocks.max(1) as f64).floor() as i32;
        // RULESET1 T10, T11: the AP the foe pays (none under enemy_moves)
        let cost = if w.pays_ap(foe) { skill::ap_cost_by(s, Some(fu), &w.ruleset) } else { 0 };
        if cost > 0 {
            hits = hits.min(((fu.ap - tiles) as f64 / cost as f64).floor() as i32);
        }
        if hits > 0 {
            best = best.max(hits * expected(w, foe, target, sid, None, 0.0, 1.0));
        }
    }
    best
}

fn taunt_pull(w: &World, plan: &Plan, actor: UnitId, st: &SimState, ranked: &[UnitId]) -> Option<UnitId> {
    let &taunter = ranked.first()?;
    if !active_with(&w.units[taunter], pulls_targeting) {
        return None;
    }
    let band = basic_band(w, actor);
    let d = st.pos.dist(w.pos(taunter));
    if d >= band.0 && d <= band.1 {
        return Some(taunter);
    }
    if st.rooted || st.acted || st.moved {
        return None;
    }
    let route = plan.approach_path(w, actor, st.pos, taunter, -1, -1);
    (!route.is_empty() && plan.fits_move(w, actor, st.pos, &route)).then_some(taunter)
}

fn reserve_heal(w: &World, plan: &mut Plan, actor: UnitId, profile: &AiProfile, cap: i32) -> Option<UnitId> {
    let au = &w.units[actor];
    let mut heals: Vec<SkillId> = w.book().available(Kind::Support, cap, Some(au)).into_iter().filter(|s| skill::heals(&w.cat.skills[*s]) && (!w.pays_ap(actor) || skill::ap_cost_by(&w.cat.skills[*s], Some(au), &w.ruleset) <= au.ap)).collect();
    if heals.is_empty() {
        return None;
    }
    sort_cheapest_heal(w, &mut heals);
    let heal = heals[0];
    let hs = &w.cat.skills[heal];
    let friends = w.friends_of(actor).to_vec();
    let where_ = plan.planned_positions(w, &friends);
    let mut cands: Vec<(UnitId, f64, usize)> = Vec::new();
    for (i, &f) in friends.iter().enumerate() {
        if !w.units[f].is_active() || !w.has(f) {
            continue;
        }
        if (f == actor && hs.target_mode == "ALLY") || (f != actor && !skill::targets_ally(hs)) {
            continue;
        }
        let at = where_.get(&f).copied().unwrap_or(NO_CELL);
        let projected = w.units[f].hp - incoming_damage(w, plan, f, w.total_blocks, at, Some(&where_));
        let ratio = projected as f64 / w.units[f].max_hp.max(1) as f64;
        if ratio <= profile.heal_below {
            cands.push((f, ratio, i + 1));
        }
    }
    godot_sort(&mut cands, |a, b| if !approx_eq(a.1, b.1) { a.1 < b.1 } else { a.2 < b.2 });
    let saved = plan.steps_of(actor);
    let keep = move_floor(w, &plan.compile(w, actor));
    for (patient, _, _) in cands {
        let mut steps = saved.clone();
        loop {
            plan.set_steps(actor, steps.clone());
            let step = if patient == actor { PlanStep::self_skill(heal) } else { PlanStep::ally(heal, patient) };
            if plan.add_step(w, actor, step, false).is_empty() {
                if !step_moves(w, &plan.compile(w, actor), steps.len()) {
                    return Some(patient);
                }
                plan.set_steps(actor, steps.clone());
            }
            if steps.len() <= keep {
                break;
            }
            steps.pop();
        }
    }
    plan.set_steps(actor, saved);
    None
}

fn move_floor(w: &World, st: &SimState) -> usize {
    let mut floor = 0;
    for e in &st.entries {
        let k = skill::kind(&w.cat.skills[e.skill]);
        if e.step_index >= 0 && (k == Kind::Move || k == Kind::Disengage || e.pos_after != e.pos_before) {
            floor = floor.max(e.step_index as usize + 1);
        }
    }
    floor
}

fn step_moves(w: &World, st: &SimState, index: usize) -> bool {
    st.entries.iter().any(|e| e.step_index == index as i32 && skill::kind(&w.cat.skills[e.skill]) == Kind::Move)
}

// ---- Round intent AI (RULESET1_INTENT 9, RULESET1-AI; engine-only, no Godot original) -----------------------------
//
// Under the rule `round_intent` a unit's round is its intent and target; the auto planner (`round_intent_plan`) builds
// the steps. The AI only picks: enemies by the table of AI1 (first row that applies), party members by their AUTO1
// tactic (AI5). Each candidate pick is tried on the unit's own state and steps and rolled back, so a refused pick drops
// no intent-bound buff (IN6); the chosen one is then set for real (an intent change drops them and logs it).
// Deterministic: candidates come in roster, ranking or order-list order; nothing draws a random number.

/// AI1 row tags (`IntentRound::row`).
pub const ROW_FLEE: &str = "flee";
pub const ROW_SUPPORT: &str = "support";
pub const ROW_GUARD: &str = "guard";
/// HOOKS1 AT2 (rule `ai_tactics`): `brace_after_hit`, the row before row 4.
pub const ROW_BRACE: &str = "brace";
pub const ROW_ENGAGE: &str = "engage";
pub const ROW_ASSAULT: &str = "assault";
/// INTENT2 matrix row Z (A4.2-K1b minimal, W2-5 I1.2-S3 full): a Field round before row 5 (rule `field_intent` only).
pub const ROW_FIELD: &str = "field";
/// INTENT2 matrix ring row (W2-5 I1.2-S3): a Stand round opened by a RING on two or more foes, before row 4 (rule
/// `field_intent` only).
pub const ROW_RING: &str = "ring";
/// INTENT2 matrix Maneuver gap-close row (FB-130 Option A, CLASS-BASICS): any role holding a gap-close attack (a
/// Maneuver attack with `dash_tiles`) dashes at a foe out of its normal reach, before row 5 (rule `field_intent` only).
pub const ROW_GAP_CLOSE: &str = "gap close";
/// Row 5 with no foe in reach this round: a move toward the first ranked target, then Guard there (front line),
/// Support on itself or an ally (Support role) or Rest (reckless; also when the other two are not offered).
pub const ROW_ADVANCE: &str = "advance";
pub const ROW_IDLE_GUARD: &str = "idle guard";
pub const ROW_REST: &str = "rest";
/// AI5: a tactic other than Balanced decided the round.
pub const ROW_TACTIC: &str = "tactic";
/// ENCOUNTER2 H6: a hold escape's keeper Guards its hold tile (or walks toward it); before the tactic and the table.
pub const ROW_HOLD: &str = "hold";
/// ENCOUNTER2 H6: the gate is open (or about to be): a hero of a hold escape withdraws.
pub const ROW_ESCAPE: &str = "escape";

/// RULESET1_INTENT AI1, AI5: one intent pick: the intent, its round target, the Maneuver kind (None: the planner's MV11
/// pick), the opener (None: PL3's), Economy's opener-only round (no follow-ups, Rest after) and the advance of row 5
/// (`advance_to`: a plain move to that tile first, then the intent's skills that act from there (MM3's fill), Rest
/// after).
#[derive(Debug, Clone, PartialEq)]
pub struct IntentPick {
    pub intent: RoundIntent,
    pub target: RoundTarget,
    pub kind: Option<ManeuverKind>,
    pub opener: Option<Opener>,
    pub opener_only: bool,
    pub advance_to: Option<V2>,
    /// The advance's round must place a skill of its intent (a Support buff, a repositioned attack).
    pub must_act: bool,
    /// W2-5 I1.2-S3 (INTENT2 matrix AI rows: "reckless drops Defend skills, not the Stand intent"): the round keeps no
    /// Defend skill (a reckless unit's ring round); its other steps stay in order, Rest after.
    pub drop_defend: bool,
}

impl IntentPick {
    fn new(intent: RoundIntent, target: RoundTarget) -> Self {
        Self { intent, target, kind: None, opener: None, opener_only: false, advance_to: None, must_act: false, drop_defend: false }
    }
}

/// A Guard (Stand) pick on `target`. A4.2-K1b: under `field_intent` the Stand order also holds rings (IR21), so the
/// pick opens with the first Defend skill of the order when there is one (the AI's Guard rows guard; a ring follows in
/// the fill when an enemy stands in it). Without the rule: PL3's opener, as before.
fn stand_pick(w: &World, c: UnitId, target: RoundTarget) -> IntentPick {
    let mut p = IntentPick::new(RoundIntent::Guard, target);
    if round_intent::field_rows(w) {
        p.opener = round_intent_plan::order(w, c, RoundIntent::Guard).into_iter().find(|&sid| skill::kind(&w.cat.skills[sid]) == Kind::Defend).map(Opener::Skill);
    }
    p
}

/// What the round intent AI did for one unit: the row (or tactic) that applied, the pick (None: a Withdraw or a Rest
/// round, the intent unchanged) and the intent change's log lines (IN6: "`<unit>` leaves Guard: Riposte ends").
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntentRound {
    pub row: &'static str,
    pub pick: Option<IntentPick>,
    pub lines: Vec<String>,
}

enum Choice {
    Pick(&'static str, IntentPick),
    /// A Withdraw is planned (the row: flee, or a hold escape's escape).
    Withdrawn(&'static str),
    Rest,
}

/// RULESET1_INTENT AI1, AI5: picks `c`'s round intent and target (the AI table for `Tactic::Balanced`, else the
/// tactic's pick, falling back to the table when the tactic's intent is not offered) and plans the round with the
/// intent planner (its `deck.order`, auto when empty). Inert (an empty plan) without the rule `round_intent`.
pub fn plan_round_intent(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile, tactic: Tactic) -> IntentRound {
    plan.clear(c);
    if !round_intent::on(w) || !w.units[c].is_active() || !w.has(c) {
        return IntentRound::default();
    }
    // PK1-S2b: a re-pick starts from Cold Steel (the Channel row may open Channel again)
    if crate::channel::is_channel(w, c) {
        round_intent_plan::flip_channel(w, c, false);
    }
    plan.path_penalties.insert(c);
    if w.units[c].attack_range > 1 {
        let shooters = shooters_of(w, w.foes_of(c));
        if !shooters.is_empty() {
            plan.goal_preferences.insert(c, shooters);
        }
    }
    // ENCOUNTER2 H6: a hold escape's part comes first (None without one)
    let choice = match hold_choice(w, plan, c) {
        Some(ch) => ch,
        None => match tactic_pick(w, plan, c, profile, tactic) {
            Some(p) => Choice::Pick(ROW_TACTIC, p),
            None => table(w, plan, c, profile, 0),
        },
    };
    resolve(w, plan, c, profile, choice)
}

/// Commits `pick` for `c` as a tactic's pick would be (a failed commit goes on with the table from its first row); for
/// tests of the fallback, which no shipped pick reaches.
#[doc(hidden)]
pub fn plan_given(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile, pick: IntentPick) -> IntentRound {
    plan.clear(c);
    if !round_intent::on(w) || !w.units[c].is_active() || !w.has(c) {
        return IntentRound::default();
    }
    if crate::channel::is_channel(w, c) {
        round_intent_plan::flip_channel(w, c, false);
    }
    plan.path_penalties.insert(c);
    resolve(w, plan, c, profile, Choice::Pick(ROW_TACTIC, pick))
}

/// Commits the choice; a pick that fails for real goes on with the table from the next row.
fn resolve(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile, mut choice: Choice) -> IntentRound {
    let mut lines = Vec::new();
    loop {
        match choice {
            Choice::Pick(row, p) => match commit(w, plan, c, &p) {
                Ok(l) => {
                    lines.extend(l);
                    // PK1-S2b (PK1 4.4): a unit that can Channel may turn the committed round into a Channel round
                    lines.extend(channel_row(w, plan, c, &p));
                    return IntentRound { row, pick: Some(p), lines };
                }
                Err(l) => {
                    // the trial offered it, the real set did not (IN6 dropped a buff the round relied on, or an
                    // Economy round without an opener step): the table goes on from the next row (after a tactic:
                    // from the first)
                    lines.extend(l);
                    plan.clear(c);
                    let from = ROWS.iter().position(|r| *r == row).map_or(0, |i| i + 1);
                    choice = table(w, plan, c, profile, from);
                }
            },
            Choice::Withdrawn(row) => return IntentRound { row, pick: None, lines },
            Choice::Rest => {
                plan.clear(c);
                rest_round(w, plan, c, "rest: nothing usable");
                return IntentRound { row: ROW_REST, pick: None, lines };
            }
        }
    }
}

/// PK1 4.4 (`docs/sessions/PK1_CORRUPTION.md`, a Design default): the factor by which a Channel round's value must beat
/// the Cold Steel round's for the AI to open Channel (`channel_row`). PK1 section 7 lists `channel_ai_edge` as tunable;
/// it stays a kit constant until a ruleset field is approved (PR5-5: no contract change).
pub const CHANNEL_AI_EDGE: f64 = 1.2;

/// PK1-S2b: the value the Channel row compares: the expected damage and healing of `c`'s planned round (the planner's
/// estimates, `PlanEntry::est_damage` and `est_heal`; a marrow version's power and piercing included). An invalid round
/// is worth nothing.
pub fn round_value(w: &World, plan: &Plan, c: UnitId) -> i64 {
    let st = plan.compile(w, c);
    if st.error_index >= 0 {
        return 0;
    }
    st.entries.iter().map(|e| i64::from(e.est_damage.max(0)) + i64::from(e.est_heal.max(0))).sum()
}

/// PK1-S2b: the channel meter `c` would hold at the end of its planned round (its value now plus what the round's
/// actions pay, clamped as the meter is) and the meter's max; None when it holds none.
pub fn channel_meter_after(w: &World, plan: &Plan, c: UnitId) -> Option<(i32, i32)> {
    let m = crate::channel::meter_id(w);
    let max = crate::meter::pool(w, c, m)?.max;
    plan.compile(w, c).meter(m).map(|v| (v, max))
}

/// PK1-S2b: whether `c`'s planned round holds a marrow version (`channel::version` of a skill with
/// `channel::has_version`; not a `channel.only` skill, Feed or Vent).
fn marrow_planned(w: &World, plan: &Plan, c: UnitId) -> bool {
    plan.compile(w, c).entries.iter().any(|e| {
        let s = &w.cat.skills[e.skill];
        crate::channel::has_version(w, s) && crate::channel::version(w, c, s).is_some()
    })
}

/// PK1 4.4 (rule `channel`): the AI's Channel row. After the AI committed `p` for `c` (a Cold Steel round), a unit that
/// can Channel (`channel::can_channel`; others are never touched) tries the same pick as a Channel round, as the toggle
/// does for a member without picks: the round flag (`round_intent_plan::flip_channel`: the intent-bound buffs end), the
/// pick's opener kept only when it opens in Channel (else the Channel order opens), the round rebuilt by the planner.
/// It keeps the Channel round when the round compiles, plans at least one marrow version (`marrow_planned`: a Feed-only
/// or marrow-only round does not count), its value (`round_value`) is at least `CHANNEL_AI_EDGE` times the Cold Steel
/// round's, and the channel meter after it (`channel_meter_after`) stays below the meter's max (the crisis); else `c`'s
/// unit, round state and steps are put back as the Cold Steel round. A Cold Steel round worth nothing (out of reach, a
/// Guard round) has no option to beat and stays Cold Steel untried, as do Economy's opener-only rounds. Returns the
/// flip's lines when kept. Deterministic: nothing draws.
fn channel_row(w: &mut World, plan: &mut Plan, c: UnitId, p: &IntentPick) -> Vec<String> {
    if p.opener_only || !crate::channel::can_channel(w, c) || crate::channel::is_channel(w, c) {
        return Vec::new();
    }
    let cold = round_value(w, plan, c);
    if cold <= 0 {
        return Vec::new();
    }
    let unit = w.units[c].clone();
    let saved = w.round_states.get(&c).cloned();
    let steps = plan.steps_of(c);
    let lines = round_intent_plan::flip_channel(w, c, true);
    let opener = p.opener.clone().filter(|o| match o {
        Opener::Skill(sid) => crate::channel::opens(w, c, &w.cat.skills[*sid]),
        Opener::Item(_) => false,
    });
    w.round_states.entry(c).or_default().opener = opener;
    let built = build_pick(w, plan, c, p) && marrow_planned(w, plan, c);
    let value = if built { round_value(w, plan, c) } else { 0 };
    let safe = channel_meter_after(w, plan, c).is_some_and(|(v, max)| v < max);
    if built && value as f64 >= CHANNEL_AI_EDGE * cold as f64 && safe {
        return lines;
    }
    w.units[c] = unit;
    match saved {
        Some(rs) => {
            w.round_states.insert(c, rs);
        }
        None => {
            w.round_states.shift_remove(&c);
        }
    }
    plan.set_steps(c, steps);
    Vec::new()
}

/// AI1's rows in table order (`table` starts at an index into it).
const ROWS: [&str; 11] = [ROW_FLEE, ROW_SUPPORT, ROW_GUARD, ROW_BRACE, ROW_RING, ROW_ENGAGE, ROW_FIELD, ROW_GAP_CLOSE, ROW_ASSAULT, ROW_ADVANCE, ROW_IDLE_GUARD];

/// AI1 for the enemy side (CombatEncounter.schedule_enemy_ai under `round_intent`): every enemy on the field picks and
/// plans into the shared `plan` in roster order (earlier plans are claims), broken plans are re-picked, what is still
/// broken is trimmed. Returns the intent change lines.
pub fn plan_enemy_intents(w: &mut World, plan: &mut Plan) -> Vec<String> {
    plan_enemy_intents_except(w, plan, &[])
}

/// `plan_enemy_intents` for every enemy on the field but `fixed` (ENCOUNTER2-C SR2: a scripted round's enemy entries,
/// already in `plan`; the AI sees them as it sees earlier plans and never re-picks or trims them).
pub fn plan_enemy_intents_except(w: &mut World, plan: &mut Plan, fixed: &[UnitId]) -> Vec<String> {
    let drafted: Vec<UnitId> = w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.has(e) && !fixed.contains(&e)).collect();
    let picks: Vec<(UnitId, AiProfile, Tactic)> = drafted.iter().map(|&e| (e, w.profile_of(e), Tactic::Balanced)).collect();
    let lines = plan_side(w, plan, &picks, true);
    let mut trimmed = true;
    while trimmed {
        trimmed = false;
        for &e in &drafted {
            if draft_broken(w, plan, e) {
                let st = plan.compile(w, e);
                let keep = if st.error_index >= 0 { st.error_index as usize } else { 0 };
                let steps = plan.steps_of(e);
                plan.set_steps(e, steps[..keep].to_vec());
                trimmed = true;
            }
        }
    }
    lines
}

/// AI5 for party members (plan_apply_preset_all under `round_intent`): each `(unit, preset)` in order picks by its
/// tactic with the preset's profile; invalid plans are re-picked. A member holding picks (INPUT1 IF9) is skipped: its
/// intent, target and steps stay as the player left them (user 2026-10-28). Returns the intent change lines.
pub fn plan_party_intents(w: &mut World, plan: &mut Plan, presets: &[(UnitId, String)]) -> Vec<String> {
    let picks: Vec<(UnitId, AiProfile, Tactic)> = presets
        .iter()
        .filter(|(a, _)| w.units[*a].is_active() && round_intent_plan::picks_of(w, *a).is_empty())
        .map(|(a, p)| (*a, w.profile_of(*a).with_preset(p), Tactic::parse(p)))
        .collect();
    plan_side(w, plan, &picks, false)
}

/// Picks and plans each unit in order, then re-picks broken (enemies) or invalid (party) plans until none changes.
fn plan_side(w: &mut World, plan: &mut Plan, picks: &[(UnitId, AiProfile, Tactic)], enemies: bool) -> Vec<String> {
    let mut lines = Vec::new();
    for (c, profile, tactic) in picks {
        lines.extend(plan_round_intent(w, plan, *c, profile, *tactic).lines);
    }
    for _ in 0..picks.len() {
        let mut redrafted = false;
        for (c, profile, tactic) in picks {
            let bad = if enemies { draft_broken(w, plan, *c) } else { w.units[*c].is_active() && !plan.is_valid(w, *c) };
            if bad {
                lines.extend(plan_round_intent(w, plan, *c, profile, *tactic).lines);
                redrafted = true;
            }
        }
        if !redrafted {
            break;
        }
    }
    lines
}

/// A plan that does not compile or compiles to nothing (the session's draft check).
fn draft_broken(w: &World, plan: &Plan, c: UnitId) -> bool {
    if plan.steps_of(c).is_empty() {
        return false;
    }
    let st = plan.compile(w, c);
    st.error_index >= 0 || st.entries.is_empty()
}

/// RULESET1_INTENT AI1: the first row of the table, from `ROWS[from]` on, that applies.
fn table(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile, from: usize) -> Choice {
    let on = |row: &str| ROWS.iter().position(|r| *r == row).is_some_and(|i| i >= from);
    let ratio = hp_ratio(w, c);
    // 1: `flee_ratio` reached: today's Withdraw (a neutral move); the intent stays
    if on(ROW_FLEE) && profile.flee_ratio > 0.0 && !profile.reckless && ratio <= profile.flee_ratio && plan.add_step(w, c, PlanStep::withdraw(), true).is_empty() {
        return Choice::Withdrawn(ROW_FLEE);
    }
    // 2: Support role, or someone below `heal_below`, and a Support skill (or item) reaches
    if on(ROW_SUPPORT) {
        if let Some(p) = support_pick(w, plan, c, profile) {
            return Choice::Pick(ROW_SUPPORT, p);
        }
    }
    // 3: hurt, not reckless, a Guard skill usable: Guard self, or the best cover tile in one move against a shooter;
    // RULESET1-AI2: not after a round in Guard that no attack hit took HP in (the table goes on down); S1.2-S4: not while
    // a multi-round guard already holds this round (no redundant Defend)
    if on(ROW_GUARD) && ratio <= profile.defend_below && !profile.reckless && !w.hits.guarded_unhit(c) && !guard_held(w, c) {
        if let Some(p) = guard_pick(w, plan, c) {
            return Choice::Pick(ROW_GUARD, p);
        }
    }
    // HOOKS1 AT2 `brace_after_hit` (rule ai_tactics): a friend within 2 tiles, or itself, lost HP to an attack hit last
    // round, not reckless, a Guard skill usable: Guard self, or on its `hold_line` tile
    if on(ROW_BRACE) && !profile.reckless && w.has_tactic(c, UnitTactic::BraceAfterHit) && hit_nearby(w, c) && !guard_held(w, c) {
        if let Some(p) = brace_pick(w, plan, c) {
            return Choice::Pick(ROW_BRACE, p);
        }
    }
    // W2-5 I1.2-S3 (ring row, before row 4): under `field_intent`, two or more foes inside a usable ring: a Stand
    // round opened by it (reckless too, without Defend skills)
    if on(ROW_RING) {
        if let Some(p) = ring_pick(w, plan, c, profile) {
            return Choice::Pick(ROW_RING, p);
        }
    }
    let ranked = ranked_foes(w, plan, c, profile);
    // 4: a Skirmisher whose Maneuver attack reaches a flank tile on a ranked target engaged with another unit
    if on(ROW_ENGAGE) && profile.role == Role::Skirmisher {
        if let Some(p) = engage_pick(w, plan, c, &ranked) {
            return Choice::Pick(ROW_ENGAGE, p);
        }
    }
    // row Z (A4.2-K1b, W2-5 I1.2-S3): under `field_intent`, a Field round on a source that covers enough foes
    if on(ROW_FIELD) {
        if let Some(p) = field_pick(w, plan, c, profile, &ranked) {
            return Choice::Pick(ROW_FIELD, p);
        }
    }
    // gap-close row (FB-130 Option A, CLASS-BASICS): under `field_intent`, no foe in normal reach and a gap-close attack
    // dashes onto a ranked foe with a safe landing tile: a Maneuver round (the dash replaces row 5's plain close-in)
    if on(ROW_GAP_CLOSE) {
        if let Some(p) = gap_close_pick(w, plan, c, &ranked) {
            return Choice::Pick(ROW_GAP_CLOSE, p);
        }
    }
    // 5: Assault, the first ranked target it can reach (ranged units kite: AS2, AS3); none in reach this round: Assault
    // the first ranked target it has a path to and close in with a plain move (IN5), Rest after
    // (the tile ranking, user 2026-10-28: tier 1 the selected target, the first ranked; tier 2 the tile from which it
    // hits the most hostiles; then the other ranked targets the planner reaches)
    if on(ROW_ASSAULT) {
        // HOOKS1 AT2 `flush_cover` (rule ai_tactics): an area opener aimed to catch more foes, those in cover counting
        // twice, as long as it still hits the first ranked target
        if let Some(p) = flush_pick(w, plan, c, ranked.first().copied()) {
            return Choice::Pick(ROW_ASSAULT, p);
        }
        if let Some(p) = assault_pick(w, plan, c, &ranked[..ranked.len().min(1)]) {
            return Choice::Pick(ROW_ASSAULT, p);
        }
        if let Some(p) = reposition_pick(w, plan, c, &ranked) {
            return Choice::Pick(ROW_ASSAULT, p);
        }
        if let Some(p) = assault_pick(w, plan, c, ranked.get(1..).unwrap_or(&[])) {
            return Choice::Pick(ROW_ASSAULT, p);
        }
    }
    if on(ROW_ADVANCE) {
        if let Some(p) = advance_pick(w, plan, c, &ranked, profile) {
            return Choice::Pick(ROW_ADVANCE, p);
        }
    }
    // 6: nothing usable: Guard self, else Rest
    if on(ROW_IDLE_GUARD) {
        return idle(w, plan, c);
    }
    Choice::Rest
}

/// AI5: the tactic's pick (None for Balanced, or when the tactic's intent is not offered: the table decides).
fn tactic_pick(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile, tactic: Tactic) -> Option<IntentPick> {
    match tactic {
        Tactic::Balanced => None,
        Tactic::AllOut => {
            let ranked = ranked_foes(w, plan, c, profile);
            assault_pick(w, plan, c, &ranked)
        }
        Tactic::FocusWeakest => {
            let st = plan.compile(w, c);
            let mut foes: Vec<UnitId> = w.foes_of(c).iter().copied().filter(|&f| w.units[f].is_active() && w.has(f) && st.est(f) > 0).collect();
            foes.sort_by_key(|&f| (st.est(f), f));
            assault_pick(w, plan, c, &foes)
        }
        Tactic::GuardCounter => {
            let p = stand_pick(w, c, RoundTarget::Myself);
            trial(w, plan, c, &p).then_some(p)
        }
        Tactic::KiteAtRange => {
            let ranked = ranked_foes(w, plan, c, profile);
            if w.units[c].attack_range > 1 {
                return assault_pick(w, plan, c, &ranked);
            }
            for &t in &ranked {
                let mut p = IntentPick::new(RoundIntent::Maneuver, RoundTarget::Unit(t));
                p.kind = Some(ManeuverKind::Disengage);
                if trial(w, plan, c, &p) {
                    return Some(p);
                }
            }
            assault_pick(w, plan, c, &ranked)
        }
        Tactic::Economy => {
            let ranked = ranked_foes(w, plan, c, profile);
            let pays = w.pays_ap(c);
            let mut by_cost: Vec<(i32, usize, SkillId)> = round_intent_plan::order(w, c, RoundIntent::Assault)
                .into_iter()
                .enumerate()
                .map(|(i, sid)| (if pays { skill::ap_cost_by(&w.cat.skills[sid], Some(&w.units[c]), &w.ruleset) } else { 0 }, i, sid))
                .collect();
            by_cost.sort();
            for &t in &ranked {
                for &(_, _, sid) in &by_cost {
                    let mut p = IntentPick::new(RoundIntent::Assault, RoundTarget::Unit(t));
                    p.opener = Some(Opener::Skill(sid));
                    p.opener_only = true;
                    if trial(w, plan, c, &p) {
                        return Some(p);
                    }
                }
            }
            None
        }
    }
}

/// The foes on the field best first (`rank_targets` from the unit's tile, nothing planned yet).
fn ranked_foes(w: &World, plan: &Plan, c: UnitId, profile: &AiProfile) -> Vec<UnitId> {
    let st = plan.compile(w, c);
    let foes = w.foes_of(c).to_vec();
    rank_targets(w, plan, c, &st, &foes, profile)
}

/// AI1 row 2: Support on the friend (or self) with the lowest HP share that a Support skill (or, with `use_items`, a
/// self or ally pouch item) reaches. Candidates: friends at or below `heal_below`; a Support-role unit takes any hurt
/// friend.
fn support_pick(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile) -> Option<IntentPick> {
    let support_role = profile.role == Role::Support;
    let mut cands: Vec<UnitId> = w
        .friends_of(c)
        .iter()
        .copied()
        .filter(|&f| {
            let u = &w.units[f];
            u.is_active() && w.has(f) && u.hp < u.max_hp && (support_role || hp_ratio(w, f) <= profile.heal_below)
        })
        .collect();
    cands.sort_by(|&a, &b| hp_ratio(w, a).partial_cmp(&hp_ratio(w, b)).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(&b)));
    let items: Vec<String> = if profile.use_items {
        w.units[c].inventory.pouch.iter().map(|p| p.id.clone()).filter(|id| command::pouch_item(w, c, id).is_some_and(|it| round_intent::item_rows(w, it).contains(&"support"))).collect()
    } else {
        Vec::new()
    };
    // A1.2-K3: the pouch heals usable on itself in a Stand round (under `field_intent`)
    let self_heals: Vec<String> = if profile.use_items {
        w.units[c].inventory.pouch.iter().map(|p| p.id.clone()).filter(|id| command::pouch_item(w, c, id).is_some_and(|it| it.heal > 0 && round_intent::item_rows(w, it).contains(&RoundIntent::Guard.id()))).collect()
    } else {
        Vec::new()
    };
    for f in cands {
        // A4.2-K1b (IR3 b, IR15) and K3: under `field_intent` itself is a Stand round, opened by a self-or-ally Support
        // skill, else by a pouch heal whose rows include Stand; a self Support round is never built under the rule
        if f == c && round_intent::field_rows(w) {
            for sid in round_intent_plan::order(w, c, RoundIntent::Support) {
                if !round_intent::skill_rows(w, &w.cat.skills[sid]).contains(&RoundIntent::Guard.id()) {
                    continue;
                }
                let mut p = IntentPick::new(RoundIntent::Guard, RoundTarget::Myself);
                p.opener = Some(Opener::Skill(sid));
                if trial(w, plan, c, &p) {
                    return Some(p);
                }
            }
            // A1.2-K3 (IR15, FB-121): a heal item on itself is a Stand round too (Support on yourself is refused)
            for id in &self_heals {
                let mut p = IntentPick::new(RoundIntent::Guard, RoundTarget::Myself);
                p.opener = Some(Opener::Item(id.clone()));
                if trial(w, plan, c, &p) {
                    return Some(p);
                }
            }
            continue;
        }
        let mut p = IntentPick::new(RoundIntent::Support, round_intent::normalize(c, RoundTarget::Unit(f)));
        if trial(w, plan, c, &p) {
            return Some(p);
        }
        for id in &items {
            p.opener = Some(Opener::Item(id.clone()));
            if trial(w, plan, c, &p) {
                return Some(p);
            }
        }
    }
    None
}

/// A1.2-K3 (IR15, FB-121; K1b did it for skills): under `field_intent` the Stand picks that stand for a self Support:
/// each Support skill that may land on its user (its rows hold Stand), then each pouch heal whose rows hold Stand (when
/// the profile uses items). Empty without the rule (a self Support stays a Support round).
fn self_stand_picks(w: &World, c: UnitId, profile: &AiProfile) -> Vec<IntentPick> {
    if !round_intent::field_rows(w) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for sid in round_intent_plan::order(w, c, RoundIntent::Support) {
        if round_intent::skill_rows(w, &w.cat.skills[sid]).contains(&RoundIntent::Guard.id()) {
            let mut p = IntentPick::new(RoundIntent::Guard, RoundTarget::Myself);
            p.opener = Some(Opener::Skill(sid));
            out.push(p);
        }
    }
    if profile.use_items {
        for id in command::pouch_ids(w, c) {
            if command::pouch_item(w, c, &id).is_some_and(|it| it.heal > 0 && round_intent::item_rows(w, it).contains(&RoundIntent::Guard.id())) {
                let mut p = IntentPick::new(RoundIntent::Guard, RoundTarget::Myself);
                p.opener = Some(Opener::Item(id));
                out.push(p);
            }
        }
    }
    out
}

/// AI1 row 3: Guard on the best cover tile in one move when a ranged foe threatens (and a tile beats the unit's own),
/// else Guard self; None when no Guard skill is usable.
fn guard_pick(w: &mut World, plan: &mut Plan, c: UnitId) -> Option<IntentPick> {
    if let Some(cell) = cover_tile(w, plan, c) {
        let p = stand_pick(w, c, RoundTarget::Cell(cell));
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    let p = stand_pick(w, c, RoundTarget::Myself);
    trial(w, plan, c, &p).then_some(p)
}

/// HOOKS1 AT2 `brace_after_hit`: `c` itself, or a friend on the field within 2 tiles of it, lost HP to an attack hit
/// last round (`World::hits`).
fn hit_nearby(w: &World, c: UnitId) -> bool {
    let here = w.pos(c);
    w.hits.last_round.iter().any(|&f| (f == c || w.friends_of(c).contains(&f)) && w.has(f) && w.pos(f).dist(here) <= 2)
}

/// HOOKS1 AT2 `brace_after_hit`: Guard on the `hold_line` tile (a unit with that tactic, when the tile is not its own),
/// else Guard self; None when no Guard skill is usable.
fn brace_pick(w: &mut World, plan: &mut Plan, c: UnitId) -> Option<IntentPick> {
    if w.has_tactic(c, UnitTactic::HoldLine) {
        if let Some(cell) = hold_tile(w, plan, c) {
            let p = stand_pick(w, c, RoundTarget::Cell(cell));
            if trial(w, plan, c, &p) {
                return Some(p);
            }
        }
    }
    let p = stand_pick(w, c, RoundTarget::Myself);
    trial(w, plan, c, &p).then_some(p)
}

/// HOOKS1 AT2: `c`'s `hold_line` tile: among the free tiles one standard move block reaches and its own, the first by
/// PL4 (`round_intent::rank_tiles`, which ranks a `hold_line` unit's tiles next to a friend first, then fewer free
/// neighbours, after threat), cover and distance read toward the nearest foe. None when it is `c`'s own tile, or
/// without the tactic.
pub fn hold_tile(w: &World, plan: &Plan, c: UnitId) -> Option<V2> {
    if !w.has_tactic(c, UnitTactic::HoldLine) || !w.has(c) {
        return None;
    }
    let here = w.pos(c);
    let goal = w.foes_of(c).iter().copied().filter(|&f| w.has(f) && w.units[f].is_active()).map(|f| w.pos(f)).min_by_key(|p| (p.dist(here), p.y * GRID_W + p.x)).unwrap_or(here);
    let mut cands = walk_cands(w, plan, c, 1);
    round_intent::rank_tiles(w, c, goal, true, round_intent::Aim::Toward, &mut cands);
    cands.first().map(|(cell, _)| *cell).filter(|&cell| cell != here)
}

/// A Field source candidate of `field_pick`: its sort key (score, negated; friends hit; order index; cell index), the
/// skill and the source.
type FieldCand = ((i32, usize, usize, i32), SkillId, V2);

/// INTENT2 matrix row Z (A4.2-K1b minimal; W2-5 I1.2-S3 full; IR17): under `field_intent`, a Field round when one of
/// `c`'s Field skills (its Field order) aimed from where it stands covers at least `area_threshold` foes and passes
/// `friendly_fire_ok`, and `c` did not hold Field last round (it alternates). I1.2-S3 adds the matrix's conditions:
/// for an elite or boss (`LEADER_TIERS`) the source's tiles must also hold the first ranked foe; a unit whose first
/// ranked foe is a taunter (a targeting pull, `rank_targets` puts a reachable one first) picks only a source whose
/// tiles cover it; and the best source extends `flush_pick`: with the tactic `flush_cover` a foe hit in cover against
/// `c` counts twice in the source's score. The source with the best score first (then fewer friends, the order, the
/// cell index), the skill its opener. None otherwise: the Assault row decides (IR17: Assault is the default on one or
/// two).
fn field_pick(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile, ranked: &[UnitId]) -> Option<IntentPick> {
    if !round_intent::field_rows(w) || round_intent::intent_of(w, c) == Some(RoundIntent::Field) || !w.has(c) {
        return None;
    }
    let order = round_intent_plan::order(w, c, RoundIntent::Field);
    if order.is_empty() {
        return None;
    }
    let st = plan.compile(w, c);
    let here = st.pos;
    // GD5: a hidden foe is not counted (its side cannot see it)
    let foes: Vec<UnitId> = w.foes_of(c).iter().copied().filter(|&f| w.units[f].is_active() && w.has(f) && st.est(f) > 0 && !round_intent::hidden(w, f)).collect();
    let friends: Vec<UnitId> = w.friends_of(c).iter().copied().filter(|&f| f != c && w.units[f].is_active() && w.has(f)).collect();
    // the tile the source must cover: a taunter ranked first, else an elite's or boss's first ranked foe
    let first = ranked.first().copied().filter(|&f| foes.contains(&f));
    let leader = encounter::LEADER_TIERS.contains(&w.units[c].tier.as_str());
    let must = first.filter(|&f| leader || active_with(&w.units[f], pulls_targeting)).map(|f| w.pos(f));
    let flush = w.has_tactic(c, UnitTactic::FlushCover);
    let mut cands: Vec<FieldCand> = Vec::new();
    for (i, sid) in order.into_iter().enumerate() {
        let s = &w.cat.skills[sid];
        if !skill::is_area(s) || !skill::deals_damage(s) {
            continue;
        }
        let band = skill::field_reach(s, &w.units[c], &w.ruleset);
        let affects = skill::affects_by(s, Some(RoundIntent::Field));
        for aim in aim_cells(w, here, sid, band) {
            if !skill::from_user(skill::area(s)) && !(band.0..=band.1).contains(&here.dist(aim)) {
                continue;
            }
            let cells: Vec<V2> = skill::field_cells(s, here, aim).into_iter().filter(|cell| w.grid.is_valid(*cell) && *cell != here).collect();
            if must.is_some_and(|m| !cells.contains(&m) || !skill::affects_unit(affects, false)) {
                continue;
            }
            let hit: Vec<UnitId> = foes.iter().copied().filter(|&f| skill::affects_unit(affects, false) && cells.contains(&w.pos(f))).collect();
            let n = hit.len() as i32;
            let hit_friends: Vec<UnitId> = friends.iter().copied().filter(|&f| skill::affects_unit(affects, true) && cells.contains(&plan.planned_position(w, f))).collect();
            if n < profile.area_threshold() || !friendly_fire_ok(w, c, sid, &hit_friends, n) {
                continue;
            }
            let covered = if flush { hit.iter().filter(|&&f| command::cover_for(&w.grid, here, w.pos(f), true) > 0.0).count() as i32 } else { 0 };
            cands.push(((-(n + covered), hit_friends.len(), i, aim.y * GRID_W + aim.x), sid, aim));
        }
    }
    cands.sort_by_key(|(k, _, _)| *k);
    for (_, sid, aim) in cands {
        let mut p = IntentPick::new(RoundIntent::Field, RoundTarget::Cell(aim));
        p.opener = Some(Opener::Skill(sid));
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// FB-130 Option A: a gap-close attack: a single-target attack with `dash_tiles` and no `position` field (lunge, pounce,
/// charge_in; a positioned attack such as flank_cut is row 4's), which `intent::derive` makes a Maneuver skill.
fn gap_closer(s: &rpgkit_data::SkillDef) -> bool {
    skill::kind(s) == Kind::Attack && !skill::is_area(s) && s.dash_tiles > 0 && s.position.is_none()
}

/// INTENT2 matrix Maneuver gap-close row (FB-130 Option A, CLASS-BASICS; role-agnostic): under `field_intent`, when no
/// visible foe stands within `c`'s normal reach (its basic attack band, `basic_band`) from where it stands, a Maneuver
/// Engage round opened by a gap-close attack of its Maneuver order (`gap_closer`) on the first ranked visible foe that
/// attack reaches (`round_intent::engage_reach`: an open position tile within its dash), when the tile the dash lands
/// on is safe: no harmful ground or due arena hazard on it (`ground::harms`, `arena::due_on`) and the round's likely
/// incoming damage there (`incoming_damage` over `total_blocks`, the test that flags `ai_pressure`) below `c`'s HP. A
/// unit whose first ranked foe is a taunter dashes only at it. None otherwise: the Assault row decides.
fn gap_close_pick(w: &mut World, plan: &mut Plan, c: UnitId, ranked: &[UnitId]) -> Option<IntentPick> {
    if !round_intent::field_rows(w) || !w.has(c) {
        return None;
    }
    let attacks: Vec<SkillId> = round_intent_plan::order(w, c, RoundIntent::Maneuver).into_iter().filter(|&sid| gap_closer(&w.cat.skills[sid])).collect();
    if attacks.is_empty() {
        return None;
    }
    let st = plan.compile(w, c);
    let here = st.pos;
    // GD5: no dash at a hidden target (and a hidden foe is not counted as in reach)
    let foes: Vec<UnitId> = w.foes_of(c).iter().copied().filter(|&f| w.units[f].is_active() && w.has(f) && st.est(f) > 0 && !round_intent::hidden(w, f)).collect();
    let band = basic_band(w, c);
    let diag = round_intent::melee_band(band) && round_intent::diagonal(w, c, None);
    let dist: HashMap<UnitId, i32> = foes.iter().map(|&f| (f, round_intent::band_dist(diag, here, w.pos(f)))).collect();
    // a foe already in normal reach: the Assault row fights it from here
    if dist.values().any(|d| (band.0..=band.1).contains(d)) {
        return None;
    }
    let taunted = ranked.first().copied().filter(|&f| foes.contains(&f) && active_with(&w.units[f], pulls_targeting));
    let targets: Vec<UnitId> = match taunted {
        Some(f) => vec![f],
        None => ranked.iter().copied().filter(|f| foes.contains(f)).collect(),
    };
    let blocked = plan.blocked_cells(w, c);
    for t in targets {
        if dist[&t] <= band.1 {
            continue;
        }
        for &sid in &attacks {
            if !round_intent::engage_reach(w, c, &w.cat.skills[sid], t, here, &blocked) {
                continue;
            }
            let mut p = IntentPick::new(RoundIntent::Maneuver, RoundTarget::Unit(t));
            p.kind = Some(ManeuverKind::Engage);
            p.opener = Some(Opener::Skill(sid));
            if trial_read(w, plan, c, &p, &|w: &World, plan: &Plan| landing_safe(w, plan, c, sid)) {
                return Some(p);
            }
        }
    }
    None
}

/// FB-130: whether the tile `c`'s planned `sid` (the gap-close attack) lands on is safe (see `gap_close_pick`); false
/// when the round does not place it.
fn landing_safe(w: &World, plan: &Plan, c: UnitId, sid: SkillId) -> bool {
    let st = plan.compile(w, c);
    let Some(e) = st.entries.iter().find(|e| e.skill == sid) else { return false };
    let at = e.pos_after;
    !crate::ground::harms(w, c, at) && !crate::arena::due_on(w, at) && incoming_damage(w, plan, c, w.total_blocks, at, None) < w.units[c].hp
}

/// INTENT2 matrix ring row (W2-5 I1.2-S3, IR6, IR22): at least this many foes inside a ring's tiles.
pub const RING_MIN_FOES: usize = 2;

/// INTENT2 matrix ring row (W2-5 I1.2-S3): under `field_intent`, a RING of `c`'s Stand order (an attack dealing
/// damage that hits foes) with at least `RING_MIN_FOES` visible foes in its tiles around `c`: a Stand round on itself
/// opened by that ring (the Stand order fills the rest; a ring is cast only while a foe stands in it). Reckless units
/// take the row too ("reckless drops Defend skills, not the Stand intent, or a reckless boss never flails"): their
/// round keeps no Defend skill (`IntentPick::drop_defend`). The order's first ring the planner offers; None otherwise.
fn ring_pick(w: &mut World, plan: &mut Plan, c: UnitId, profile: &AiProfile) -> Option<IntentPick> {
    if !round_intent::field_rows(w) || !w.has(c) {
        return None;
    }
    let st = plan.compile(w, c);
    let here = st.pos;
    // GD5: a hidden foe is not counted
    let foes: Vec<V2> = w.foes_of(c).iter().copied().filter(|&f| w.units[f].is_active() && w.has(f) && st.est(f) > 0 && !round_intent::hidden(w, f)).map(|f| w.pos(f)).collect();
    for sid in round_intent_plan::order(w, c, RoundIntent::Guard) {
        let s = &w.cat.skills[sid];
        if skill::kind(s) != Kind::Attack || skill::area(s) != skill::Area::Ring || !skill::deals_damage(s) || !skill::affects_unit(skill::affects_by(s, Some(RoundIntent::Guard)), false) {
            continue;
        }
        let cells = skill::area_cells(s, here, here);
        if foes.iter().filter(|p| **p != here && cells.contains(p)).count() < RING_MIN_FOES {
            continue;
        }
        let mut p = IntentPick::new(RoundIntent::Guard, RoundTarget::Myself);
        p.opener = Some(Opener::Skill(sid));
        p.drop_defend = profile.reckless;
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// A4.2-K1b (IR2, IR17): under `field_intent` an aim at a tile is a Field round (the Field order, band and tiles), never
/// an Assault one.
/// HOOKS1 AT2 `flush_cover`: an area opener's aim from `c`'s tile, scored by foes hit plus 1 per foe hit on a tile with
/// cover against `c` (`cover_for`), among the aims whose area still holds the first ranked target `t` and hit no more
/// friends than an aim at `t`'s tile; the Assault order's area skills that reach `t` without a move, in order. Some
/// only when an aim beats the one at `t`'s tile (the round target is then that tile, the skill its opener).
fn flush_pick(w: &mut World, plan: &mut Plan, c: UnitId, t: Option<UnitId>) -> Option<IntentPick> {
    let t = t?;
    if !w.has_tactic(c, UnitTactic::FlushCover) || !w.has(t) {
        return None;
    }
    let (here, goal) = (w.pos(c), w.pos(t));
    let foes: Vec<V2> = w.foes_of(c).iter().copied().filter(|&f| w.units[f].is_active() && w.has(f)).map(|f| w.pos(f)).collect();
    let friends: Vec<V2> = w.friends_of(c).iter().copied().filter(|&f| f != c && w.units[f].is_active() && w.has(f)).map(|f| w.pos(f)).collect();
    let field = round_intent::field_rows(w);
    let intent = if field { RoundIntent::Field } else { RoundIntent::Assault };
    // (score, friends hit) of an aim: foes hit plus the covered ones, and the friends it catches
    let score = |s: &rpgkit_data::SkillDef, aim: V2| -> Option<(i32, usize)> {
        let shape = if field { skill::field_cells(s, here, aim) } else { skill::area_cells(s, here, aim) };
        let cells: Vec<V2> = shape.into_iter().filter(|cell| w.grid.is_valid(*cell) && *cell != here).collect();
        if !cells.contains(&goal) {
            return None;
        }
        // A3.2-K2 (IR9): the round's row's `affects`
        let affects = skill::affects_by(s, Some(intent));
        let hit: Vec<&V2> = foes.iter().filter(|p| skill::affects_unit(affects, false) && cells.contains(p)).collect();
        let covered = hit.iter().filter(|p| command::cover_for(&w.grid, here, ***p, true) > 0.0).count();
        Some(((hit.len() + covered) as i32, friends.iter().filter(|p| skill::affects_unit(affects, true) && cells.contains(p)).count()))
    };
    let mut cands: Vec<((i32, usize, i32), SkillId, V2)> = Vec::new();
    for (i, sid) in round_intent_plan::order(w, c, intent).into_iter().enumerate() {
        let s = &w.cat.skills[sid];
        if !skill::is_area(s) || !skill::deals_damage(s) {
            continue;
        }
        let band = if field { skill::field_reach(s, &w.units[c], &w.ruleset) } else { skill::reach_by(s, &w.units[c], &w.ruleset) };
        let d = here.dist(goal);
        let Some((base, base_friends)) = score(s, goal).filter(|_| d >= band.0 && d <= band.1) else { continue };
        for aim in aim_cells(w, here, sid, band) {
            // the planner casts an area aimed outside its band only after a move: such aims are not scored
            if aim == goal || !(band.0..=band.1).contains(&here.dist(aim)) {
                continue;
            }
            if let Some((sc, fr)) = score(s, aim) {
                if sc > base && fr <= base_friends {
                    cands.push(((-sc, i, aim.y * GRID_W + aim.x), sid, aim));
                }
            }
        }
    }
    cands.sort_by_key(|(k, _, _)| *k);
    for (_, sid, aim) in cands {
        let mut p = IntentPick::new(intent, RoundTarget::Cell(aim));
        p.opener = Some(Opener::Skill(sid));
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// The free tiles `c` can walk to within `blocks` standard move blocks (a unit that pays: at most its AP in tiles), with
/// their routes; its own tile first, with an empty route.
fn walk_cands(w: &World, plan: &Plan, c: UnitId, blocks: usize) -> Vec<(V2, Vec<V2>)> {
    let here = w.pos(c);
    let st = plan.compile(w, c);
    let blocked = plan.blocked_cells(w, c);
    let mpb = w.units[c].move_per_block();
    let ap = if w.pays_ap(c) { st.ap_left.max(0) as usize } else { usize::MAX };
    let mut out: Vec<(V2, Vec<V2>)> = vec![(here, Vec::new())];
    if st.rooted {
        return out;
    }
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let cell = V2::new(x, y);
            if cell == here || cell.dist(here) > mpb * blocks as i32 || !w.grid.is_walkable(cell) || w.grid.is_exit(cell) || blocked.contains(&cell) {
                continue;
            }
            let goals: Cells = [cell].into_iter().collect();
            let route = plan.route(w, c, here, &goals, &blocked, false);
            if !route.is_empty() && route.len() <= ap && path::blocks_needed(&w.grid, here, &route, mpb) <= blocks {
                out.push((cell, route));
            }
        }
    }
    out
}

/// The nearest ranged foe of `c` on the field (the cover tie-break is read against it); None without one.
fn nearest_shooter(w: &World, c: UnitId) -> Option<V2> {
    let here = w.pos(c);
    w.foes_of(c).iter().copied().filter(|&f| w.has(f) && w.units[f].attack_range > 1).min_by_key(|&f| (w.pos(f).dist(here), f)).map(|f| w.pos(f))
}

/// Cover on `cell` against `shooter` (x1000, for sort keys; 0 without a shooter).
fn shelter(w: &World, shooter: Option<V2>, cell: V2) -> i64 {
    shooter.map_or(0, |s| (command::cover_for(&w.grid, s, cell, true) * 1000.0).round() as i64)
}

/// AI1 row 3's tile (user 2026-10-28, tier 3 of the tile ranking: a unit at low HP looks to its own safety), when a
/// ranged foe's next-round reach covers `c` (PL4 (1)): among the free tiles one standard move block reaches and `c`'s
/// own, fewer threats (foes whose next-round reach covers the tile) first, then more cover against the nearest
/// threatening shooter, then farther from the nearest foe; ties: fewer path tiles, the cell index. The best, unless it
/// is `c`'s own tile.
pub fn cover_tile(w: &World, plan: &Plan, c: UnitId) -> Option<V2> {
    let here = w.pos(c);
    let shooter = w
        .foes_of(c)
        .iter()
        .copied()
        .filter(|&f| {
            let u = &w.units[f];
            u.is_active() && w.has(f) && u.attack_range > 1 && w.pos(f).dist(here) <= u.move_per_block() * 2 + skill::threat_reach(u, &w.ruleset)
        })
        .min_by_key(|&f| (w.pos(f).dist(here), f))?;
    let foes: Vec<V2> = w.foes_of(c).iter().copied().filter(|&f| w.has(f)).map(|f| w.pos(f)).collect();
    let from = Some(w.pos(shooter));
    let mut cands = walk_cands(w, plan, c, 1);
    cands.sort_by_key(|(cell, route)| {
        let near = foes.iter().map(|f| f.dist(*cell)).min().unwrap_or(0);
        (round_intent::threat(w, c, *cell), -shelter(w, from, *cell), -near, route.len(), cell.y * GRID_W + cell.x)
    });
    cands.first().map(|(cell, _)| *cell).filter(|&cell| cell != here)
}

/// Tier 2 of the tile ranking (user 2026-10-28; the selected target out of reach this round): the tiles within the
/// standard move from which `c` can hit at least one foe (a foe within its reach band), best first: the most foes in
/// reach first (a ranged unit: as many hostiles as it can hit; a melee unit: close quarters with as many as possible),
/// then a melee unit draws more attackers (more threats on the tile: the front line soaks pressure), then more cover
/// against the nearest shooter (a ranged unit: then fewer threats); ties: fewer path tiles, the cell index.
pub fn press_tiles(w: &World, plan: &Plan, c: UnitId) -> Vec<V2> {
    let band = basic_band(w, c);
    let ranged = w.units[c].attack_range > 1;
    let foes: Vec<V2> = w.foes_of(c).iter().copied().filter(|&f| w.has(f)).map(|f| w.pos(f)).collect();
    let shooter = nearest_shooter(w, c);
    let mut scored: Vec<(PressKey, V2)> = walk_cands(w, plan, c, MAX_MOVE_BLOCKS)
        .into_iter()
        .filter_map(|(cell, route)| {
            let hits = foes.iter().filter(|f| (band.0..=band.1).contains(&f.dist(cell))).count() as i64;
            if hits == 0 {
                return None;
            }
            let threat = round_intent::threat(w, c, cell) as i64;
            let cover = shelter(w, shooter, cell);
            let index = cell.y * GRID_W + cell.x;
            Some((if ranged { (-hits, 0, -cover, threat, route.len(), index) } else { (-hits, -threat, -cover, 0, route.len(), index) }, cell))
        })
        .collect();
    scored.sort_by_key(|(k, _)| *k);
    scored.into_iter().map(|(_, cell)| cell).collect()
}

/// `press_tiles`' sort key: (foes in reach, then a role's draw or cover, cover or threats, path tiles, cell index).
type PressKey = (i64, i64, i64, i64, usize, i32);

/// Row 5, tier 2: the selected target (the first ranked) is out of reach this round: Assault from the best `press_tiles`
/// tile on the first ranked foe within reach of it (a plain move there, then the attacks: the advance's round, which
/// must place an attack).
fn reposition_pick(w: &mut World, plan: &mut Plan, c: UnitId, ranked: &[UnitId]) -> Option<IntentPick> {
    let band = basic_band(w, c);
    let here = w.pos(c);
    for cell in press_tiles(w, plan, c) {
        let Some(&t) = ranked.iter().find(|&&f| (band.0..=band.1).contains(&w.pos(f).dist(cell))) else { continue };
        let mut p = IntentPick::new(RoundIntent::Assault, RoundTarget::Unit(t));
        p.advance_to = (cell != here).then_some(cell);
        p.must_act = true;
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// AI1 row 4 (INPUT1 MR14): Maneuver Engage on the first ranked target engaged with another unit (MV2) on which one of
/// `c`'s Maneuver attacks is in range (MR4) of an open position tile (`round_intent::engage_reach`), when the planner
/// offers the round.
fn engage_pick(w: &mut World, plan: &mut Plan, c: UnitId, ranked: &[UnitId]) -> Option<IntentPick> {
    let attacks: Vec<SkillId> = round_intent_plan::order(w, c, RoundIntent::Maneuver)
        .into_iter()
        .filter(|&sid| skill::kind(&w.cat.skills[sid]) == Kind::Attack && !skill::is_area(&w.cat.skills[sid]))
        .collect();
    if attacks.is_empty() {
        return None;
    }
    let blocked = plan.blocked_cells(w, c);
    let here = w.pos(c);
    for &t in ranked {
        let engaged = round_intent::engaged_with(w, t).is_some_and(|e| e != c);
        if !engaged || !attacks.iter().any(|&sid| round_intent::engage_reach(w, c, &w.cat.skills[sid], t, here, &blocked)) {
            continue;
        }
        let mut p = IntentPick::new(RoundIntent::Maneuver, RoundTarget::Unit(t));
        p.kind = Some(ManeuverKind::Engage);
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// AI1 row 5 (and the Assault tactics): Assault on the first of `targets` the planner offers it on.
fn assault_pick(w: &mut World, plan: &mut Plan, c: UnitId, targets: &[UnitId]) -> Option<IntentPick> {
    for &t in targets {
        let p = IntentPick::new(RoundIntent::Assault, RoundTarget::Unit(t));
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// AI1 row 5 when no Assault opener reaches any ranked target this round (user 2026-10-28): the unit closes in on the
/// first ranked target it has a path to (the round's standard move, `advance_dest`) and ends the round by its kind, read
/// off its AI profile (no other data): a Support-role unit Supports itself, else an ally, from the tile it moved to
/// (the Support intent, a plain move first: IN5); a reckless unit has no Guard: Assault on the target, the move, an
/// attack if one reaches from there, Rest after; a ranged unit uses a Support or Focus skill on itself from there when
/// it has one, else Guards; every other unit (the melee front line) Guards at the end of the move (the Guard intent on
/// that tile: GD1's standard move there, then hold). When the Support or Guard round is not offered (no such skill,
/// AP), the reckless round.
fn advance_pick(w: &mut World, plan: &mut Plan, c: UnitId, ranked: &[UnitId], profile: &AiProfile) -> Option<IntentPick> {
    for &t in ranked {
        let Some(dest) = advance_dest(w, plan, c, t) else { continue };
        // a reckless unit skips both: the move and Rest below
        if !profile.reckless && profile.role == Role::Support {
            let mut friends: Vec<UnitId> = w.friends_of(c).iter().copied().filter(|&f| f != c && w.has(f)).collect();
            friends.sort_by_key(|&f| (w.pos(f).dist(dest), f));
            // A1.2-K3 (IR15, FB-121): under `field_intent` a self Support is a Stand round with a self-capable opener
            for mut p in self_stand_picks(w, c, profile) {
                p.advance_to = Some(dest);
                p.must_act = true;
                if trial(w, plan, c, &p) {
                    return Some(p);
                }
            }
            let me = (!round_intent::field_rows(w)).then_some(RoundTarget::Myself);
            for target in me.into_iter().chain(friends.into_iter().map(RoundTarget::Unit)) {
                let mut p = IntentPick::new(RoundIntent::Support, target);
                p.advance_to = Some(dest);
                p.must_act = true;
                if trial(w, plan, c, &p) {
                    return Some(p);
                }
            }
        } else if !profile.reckless {
            // a ranged unit uses a self skill (a Support or Focus skill on itself) when it has one this round, and
            // Guards only without one (user 2026-10-28)
            if w.units[c].attack_range > 1 {
                for mut p in self_stand_picks(w, c, profile) {
                    p.advance_to = Some(dest);
                    p.must_act = true;
                    if trial(w, plan, c, &p) {
                        return Some(p);
                    }
                }
                if !round_intent::field_rows(w) {
                    let mut p = IntentPick::new(RoundIntent::Support, RoundTarget::Myself);
                    p.advance_to = Some(dest);
                    p.must_act = true;
                    if trial(w, plan, c, &p) {
                        return Some(p);
                    }
                }
            }
            let p = stand_pick(w, c, RoundTarget::Cell(dest));
            if trial(w, plan, c, &p) {
                return Some(p);
            }
        }
        let mut p = IntentPick::new(RoundIntent::Assault, RoundTarget::Unit(t));
        p.advance_to = Some(dest);
        if trial(w, plan, c, &p) {
            return Some(p);
        }
    }
    None
}

/// Where the round's standard move along the approach path toward `t` ends (at most the standard move's blocks, AP
/// for a unit that pays); None when the unit cannot move closer.
fn advance_dest(w: &World, plan: &Plan, c: UnitId, t: UnitId) -> Option<V2> {
    let st = plan.compile(w, c);
    let mut tiles = plan.max_move_tiles(w, c);
    if w.pays_ap(c) {
        tiles = tiles.min(st.ap_left.max(0));
    }
    if tiles < 1 || st.rooted {
        return None;
    }
    let route = plan.approach_path(w, c, st.pos, t, -1, -1);
    let reachable = path::fit_blocks(&w.grid, st.pos, &route, w.units[c].move_per_block(), MAX_MOVE_BLOCKS);
    let dest = reachable.iter().take(tiles as usize).fold(st.pos, |a, d| a + *d);
    (dest != st.pos).then_some(dest)
}

/// The advance's round (its state holds intent and target): a plain move (IN5: neutral) to `dest`, then MM3's fill (the
/// intent's skills on the target from that tile) and Rest on every block left; with `must_act` it must place a skill of
/// its intent. False when it is not offered.
fn advance(w: &World, plan: &mut Plan, c: UnitId, dest: V2, must_act: bool) -> bool {
    plan.set_steps(c, Vec::new());
    let mut step = PlanStep::go_to(w.book().lowest(Kind::Move), dest);
    if let Some(s) = Rc::get_mut(&mut step) {
        s.reason = round_intent_plan::APPROACH.to_string();
    }
    let ok = plan.add_step(w, c, step, false).is_empty()
        && round_intent_plan::fill_rest(w, plan, c).is_ok()
        && (!must_act || plan.entries_for(w, c).iter().any(|e| matches!(skill::kind(&w.cat.skills[e.skill]), Kind::Attack | Kind::Support | Kind::Defend | Kind::Focus)));
    if !ok {
        plan.set_steps(c, Vec::new());
    }
    ok
}

/// Builds `c`'s round for `p` (its state already holds intent, target, kind and opener): the intent planner, or the
/// advance. False when it is not offered.
fn build_pick(w: &mut World, plan: &mut Plan, c: UnitId, p: &IntentPick) -> bool {
    let ok = match p.advance_to {
        Some(dest) => advance(w, plan, c, dest, p.must_act),
        None => round_intent_plan::plan_unit(w, plan, c).is_ok(),
    };
    ok && (!p.drop_defend || drop_defend(w, plan, c))
}

/// W2-5 I1.2-S3 (`IntentPick::drop_defend`): takes the Defend skills out of `c`'s planned round: the other steps are
/// placed again in order (Rest steps dropped; a step that no longer fits ends the round), then Rest on every block
/// left. False when no step but Rest is left.
fn drop_defend(w: &World, plan: &mut Plan, c: UnitId) -> bool {
    let steps = plan.steps_of(c);
    if !steps.iter().any(|st| st.skill.is_some_and(|sid| skill::kind(&w.cat.skills[sid]) == Kind::Defend)) {
        return true;
    }
    plan.set_steps(c, Vec::new());
    for st in steps {
        if st.skill.is_some_and(|sid| matches!(skill::kind(&w.cat.skills[sid]), Kind::Defend | Kind::Rest)) {
            continue;
        }
        if !plan.add_step(w, c, st, false).is_empty() {
            break;
        }
    }
    let acted = !plan.steps_of(c).is_empty();
    rest_round(w, plan, c, round_intent_plan::REST);
    acted
}

/// AI1 row 6: Guard self, else a Rest round.
fn idle(w: &mut World, plan: &mut Plan, c: UnitId) -> Choice {
    let p = stand_pick(w, c, RoundTarget::Myself);
    if trial(w, plan, c, &p) {
        return Choice::Pick(ROW_IDLE_GUARD, p);
    }
    Choice::Rest
}

/// Whether the planner offers `p` to `c` now (PL7): the pick is set on `c`'s state without IN6's side effects, the round
/// is built, and `c`'s state, lock and steps are put back.
fn trial(w: &mut World, plan: &mut Plan, c: UnitId, p: &IntentPick) -> bool {
    trial_read(w, plan, c, p, &|_: &World, _: &Plan| true)
}

/// `trial`, and `read` must also hold on the built round (before it is put back).
fn trial_read(w: &mut World, plan: &mut Plan, c: UnitId, p: &IntentPick, read: &dyn Fn(&World, &Plan) -> bool) -> bool {
    if !round_intent::listed(w, p.intent) {
        return false;
    }
    let saved = w.round_states.get(&c).cloned();
    let lock = w.units[c].active_target;
    let steps = plan.sequences.contains_key(&c).then(|| plan.steps_of(c));
    {
        let rs = w.round_states.entry(c).or_default();
        rs.intent = Some(p.intent);
        rs.target = None;
        rs.maneuver = p.kind;
        rs.maneuver_auto = false;
        rs.opener = p.opener.clone();
    }
    let ok = round_intent::set_target(w, c, p.target).is_ok() && build_pick(w, plan, c, p) && read(w, plan);
    match saved {
        Some(rs) => {
            w.round_states.insert(c, rs);
        }
        None => {
            w.round_states.shift_remove(&c);
        }
    }
    w.units[c].active_target = lock;
    match steps {
        Some(s) => plan.set_steps(c, s),
        None => plan.clear(c),
    }
    ok
}

/// Sets `p` for real (IN2, IN6: an intent change drops the intent-bound buffs) and plans the round; Economy's round
/// keeps only the before-move and the opener, then Rests. Ok: the intent change lines. Err (the lines too): the pick
/// is not offered after all (the round state keeps the new intent with no target, opener or kind; no steps).
fn commit(w: &mut World, plan: &mut Plan, c: UnitId, p: &IntentPick) -> Result<Vec<String>, Vec<String>> {
    let lines = match round_intent::set_intent(w, c, p.intent) {
        Ok(l) => l,
        Err(_) => return Err(Vec::new()),
    };
    // set_target makes a unit target the lock (TG3): a failed commit puts the previous lock back
    let lock = w.units[c].active_target;
    let fail = |w: &mut World, plan: &mut Plan, lines: Vec<String>| {
        plan.clear(c);
        w.units[c].active_target = lock;
        let rs = w.round_states.entry(c).or_default();
        rs.target = None;
        rs.maneuver = None;
        rs.maneuver_auto = false;
        rs.opener = None;
        Err(lines)
    };
    if round_intent::set_target(w, c, p.target).is_err() {
        return fail(w, plan, lines);
    }
    {
        let rs = w.round_states.entry(c).or_default();
        rs.maneuver = p.kind;
        rs.maneuver_auto = false;
        rs.opener = p.opener.clone();
    }
    if !build_pick(w, plan, c, p) {
        return fail(w, plan, lines);
    }
    if p.opener_only {
        // Economy keeps the round up to its opener; a round with no opener step is not an Economy round, so the table
        // picks instead (the caller goes on from the first row)
        let steps = plan.steps_of(c);
        let Some(i) = steps.iter().position(|s| s.reason == round_intent_plan::OPENER) else { return fail(w, plan, lines) };
        plan.truncate(c, i + 1);
        rest_round(w, plan, c, round_intent_plan::REST);
    }
    Ok(lines)
}

/// RS1: Rest on every free block left of `c`'s round, tagged `tag` (PL6).
fn rest_round(w: &World, plan: &mut Plan, c: UnitId, tag: &str) {
    let rest = w.book().lowest(Kind::Rest);
    for _ in 0..w.total_blocks {
        if plan.compile(w, c).blocks_left() <= 0 {
            break;
        }
        let mut step = PlanStep::self_skill(rest);
        if let Some(s) = Rc::get_mut(&mut step) {
            s.reason = tag.to_string();
        }
        if !plan.add_step(w, c, step, false).is_empty() {
            break;
        }
    }
}


// ---- Hold escapes (ENCOUNTER2 H6; kit, no Godot original) ---------------------------------------------------------
//
// A hold escape (`stay_behind` mode hold) needs one hero on a hold tile while the others leave through the exits. The
// party AI (sims, autoplay, the Balanced draft) plays it for every hero: the keeper walks to the nearest free hold tile
// and defends there; the others withdraw once the gate is open or the keeper reaches a tile with this round's move,
// and draft by their preset before that. Without a hold escape nothing here applies (`World::hold` is None).

/// ENCOUNTER2 H6: a hero's part in a hold escape this round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldPart {
    /// Reach (or stay on) this hold tile and Defend there.
    Hold(V2),
    /// Withdraw: the gate is open, or the keeper reaches a hold tile with this round's move.
    Out,
}

/// ENCOUNTER2 H2, H6: the hero who keeps the hold tiles now: the holder while it is on the field; with it down and
/// `fallback` any, a hero already on a hold tile, else the hero on the field nearest to one (party order on a tie);
/// None before the pick, without a hold escape, or with `fallback` none and the holder down.
pub fn hold_keeper(w: &World) -> Option<UnitId> {
    let g = w.hold.as_ref()?;
    let h = g.holder?;
    if w.has(h) {
        return Some(h);
    }
    if !g.fallback_any {
        return None;
    }
    let field: Vec<UnitId> = g.heroes.iter().copied().filter(|&c| w.has(c)).collect();
    if let Some(&c) = field.iter().find(|&&c| g.holding(w, c)) {
        return Some(c);
    }
    field.into_iter().min_by_key(|&c| g.tiles.iter().map(|t| t.dist(w.pos(c))).min().unwrap_or(i32::MAX))
}

/// The route from `from` to the nearest free hold tile for `c` (free: no other unit stands there or plans to end its
/// move there); empty when `from` is one; None when no free tile can be reached.
fn hold_route(w: &World, plan: &Plan, c: UnitId, from: V2) -> Option<Vec<V2>> {
    let g = w.hold.as_ref()?;
    let blocked = plan.blocked_cells(w, c);
    let goals: Cells = g.tiles.iter().copied().filter(|t| !blocked.contains(t)).collect();
    if goals.contains(&from) {
        return Some(Vec::new());
    }
    let route = plan.route(w, c, from, &goals, &blocked, false);
    (!route.is_empty()).then_some(route)
}

/// ENCOUNTER2 H6: `c`'s part in the hold escape this round. None: no hold escape, `c` is not a hero on the field, the
/// keeper has no free hold tile to reach, or the gate stays shut this round (the preset drafts as in any fight).
pub fn hold_part(w: &World, plan: &Plan, c: UnitId) -> Option<HoldPart> {
    let g = w.hold.as_ref()?;
    if !g.heroes.contains(&c) || !w.has(c) {
        return None;
    }
    let keeper = hold_keeper(w)?;
    if keeper == c {
        let here = w.pos(c);
        let route = hold_route(w, plan, c, here)?;
        return Some(HoldPart::Hold(route.iter().fold(here, |a, d| a + *d)));
    }
    if g.is_open(w) {
        return Some(HoldPart::Out);
    }
    let at = w.pos(keeper);
    let route = hold_route(w, plan, keeper, at)?;
    plan.fits_move(w, keeper, at, &route).then_some(HoldPart::Out)
}

/// The tiles the round's standard move reaches along the route to `c`'s hold tile, farthest first (at most the move's
/// blocks; AP for a unit that pays); empty when `c` is on its tile or cannot move.
fn hold_walk(w: &World, plan: &Plan, c: UnitId) -> Vec<V2> {
    let st = plan.compile(w, c);
    let mut tiles = plan.max_move_tiles(w, c);
    if w.pays_ap(c) {
        tiles = tiles.min(st.ap_left.max(0));
    }
    if tiles < 1 || st.rooted || st.acted {
        return Vec::new();
    }
    let Some(route) = hold_route(w, plan, c, st.pos) else { return Vec::new() };
    let reachable = path::fit_blocks(&w.grid, st.pos, &route, w.units[c].move_per_block(), MAX_MOVE_BLOCKS);
    let mut cells: Vec<V2> = reachable.iter().take(tiles as usize).scan(st.pos, |a, d| {
        *a = *a + *d;
        Some(*a)
    }).collect();
    cells.reverse();
    cells
}

/// ENCOUNTER2 H6 for the step AI (`build_steps`): the keeper walks as far toward its hold tile as the round's move
/// reaches (the farthest free tile on the route), then Defends and Rests on the blocks left; a hero whose part is Out
/// plans Withdraw. False when no part applies or the Withdraw is refused (the preset drafts).
fn hold_steps(w: &World, plan: &mut Plan, c: UnitId, profile: &AiProfile) -> bool {
    match hold_part(w, plan, c) {
        None => false,
        Some(HoldPart::Out) => plan.add_step(w, c, PlanStep::withdraw(), true).is_empty(),
        Some(HoldPart::Hold(tile)) => {
            if tile != w.pos(c) {
                let mv = w.book().lowest(Kind::Move);
                for dest in hold_walk(w, plan, c) {
                    if plan.add_step(w, c, PlanStep::go_to(mv, dest), false).is_empty() {
                        break;
                    }
                }
            }
            let pos = plan.compile(w, c).pos;
            let foes: Vec<UnitId> = w.foes_of(c).to_vec();
            guard_smart(w, plan, c, w.skill_cap_of(c), profile, &foes, pos);
            rest_round(w, plan, c, "");
            true
        }
    }
}

/// ENCOUNTER2 H6 under `round_intent`: the keeper's Guard round on its hold tile (GD1: the tile as the Guard target),
/// or, when the tile is beyond this round's move, the advance toward it with Guard there; on the tile with no Guard
/// offered, a Rest round (it never leaves the tile). A hero whose part is Out plans Withdraw (the intent stays). None:
/// no part applies, or the Withdraw is refused (the tactic and the table pick).
fn hold_choice(w: &mut World, plan: &mut Plan, c: UnitId) -> Option<Choice> {
    match hold_part(w, plan, c)? {
        HoldPart::Out => plan.add_step(w, c, PlanStep::withdraw(), true).is_empty().then_some(Choice::Withdrawn(ROW_ESCAPE)),
        HoldPart::Hold(tile) => {
            let p = stand_pick(w, c, RoundTarget::Cell(tile));
            if trial(w, plan, c, &p) {
                return Some(Choice::Pick(ROW_HOLD, p));
            }
            if tile != w.pos(c) {
                for dest in hold_walk(w, plan, c) {
                    let mut p = stand_pick(w, c, RoundTarget::Myself);
                    p.advance_to = Some(dest);
                    if trial(w, plan, c, &p) {
                        return Some(Choice::Pick(ROW_HOLD, p));
                    }
                }
            }
            let p = stand_pick(w, c, RoundTarget::Myself);
            if trial(w, plan, c, &p) {
                return Some(Choice::Pick(ROW_HOLD, p));
            }
            Some(Choice::Rest)
        }
    }
}
