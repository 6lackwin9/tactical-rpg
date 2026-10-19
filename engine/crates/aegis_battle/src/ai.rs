//! TacticalAI (scripts/combat/ai/tactical_ai.gd): drafts one unit's round as plan steps, growing a shared plan one
//! validated step at a time. Used for enemies every round and for the party's AI presets.

use crate::command::{self, cleansable};
use crate::grid::{V2, DIRS, NO_CELL, ZERO};
use crate::path::{self, Cells, Hooks};
use crate::plan::{cover_pref, expected, PlanEntry, PlanStep, SimState, Step, AI_HAZARD_PENALTY, MAX_MOVE_BLOCKS};
use crate::plan::Plan;
use crate::skill::{self, Kind, SkillId};
use crate::world::{AiProfile, Role, TargetPref, UnitId, World};
use aegis_rules::damage;
use aegis_rules::sort::godot_sort;
use aegis_rules::status;
use std::collections::HashMap;

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
    skill::reach_for(&w.cat.skills[w.book().lowest(Kind::Attack)], &w.units[u])
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
        if low_hp && !guarded && dist <= reach + 1 && (pulled.is_none() || dist <= reach) {
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
        let strike_cost = skill::ap_cost(&w.cat.skills[book.lowest(Kind::Attack)], Some(&w.units[actor]));
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
    let guarded = t.stance == aegis_rules::Stance::Defensive || t.st.has_any("riposte") || plans_riposte(w, plan, target);
    let (mut first, mut middle, mut last) = (Vec::new(), Vec::new(), Vec::new());
    for &sid in skills {
        let s = &w.cat.skills[sid];
        let breaks = guarded && skill::is_stance_break(s);
        let applies = !skill::exploit_match(&s.exploit, t, &[]).0.is_empty();
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
    plan.steps_of(foe).iter().any(|st| st.skill.is_some_and(|s| skill::kind(&w.cat.skills[s]) == Kind::Defend && w.cat.skills[s].grant_status == "riposte"))
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
        let b = skill::reach_for(s, &w.units[actor]);
        s.dash_tiles <= 0 && dist >= b.0 && dist <= b.1
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
    let mut potion: Option<&aegis_data::ItemDef> = None;
    let mut bomb: Option<&aegis_data::ItemDef> = None;
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
    let penalty = |cell: V2| if w.grid.has_hazard(cell) { AI_HAZARD_PENALTY } else { 0 };
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

/// Foes best first: a reachable taunter, then nearest (aggressors, reckless) or weakest, route-penalised.
fn rank_targets(w: &World, plan: &Plan, actor: UnitId, st: &SimState, foes: &[UnitId], profile: &AiProfile) -> Vec<UnitId> {
    let nearest_first = profile.reckless || profile.target_pref == TargetPref::Nearest;
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
        if w.units[f].has_status("taunt") && reachable {
            score -= 100000.0;
        }
        scored.push((f, score));
    }
    godot_sort(&mut scored, |a, b| a.1 < b.1);
    scored.into_iter().map(|(f, _)| f).collect()
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

fn locked_down(w: &World, c: UnitId) -> bool {
    w.units[c].st.has_any("stun") || w.units[c].st.has_any("root")
}

fn frees_lock(w: &World, c: UnitId, sid: SkillId) -> bool {
    let l = cleansable(&w.units[c], &w.cat.skills[sid]);
    l.contains(&"stun") || l.contains(&"root")
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
        if needed.is_empty() || (urgent && !(needed.contains(&"stun") || needed.contains(&"root"))) {
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
    if w.units[actor].st.has_any("taunt") {
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
    let skills = ordered(w, &optional(w, w.book().granting(Kind::Support, "taunt", cap, Some(&w.units[actor])), profile), profile);
    skills.into_iter().filter(|s| w.cat.skills[*s].target_mode != "ALLY").map(PlanStep::self_skill).collect()
}

fn picked_by_a_foe(w: &World, plan: &Plan, ally: UnitId, at: V2, foes: &[UnitId], where_: &HashMap<UnitId, V2>) -> bool {
    foes.iter().any(|&f| w.units[f].is_active() && w.has(f) && likely_target(w, plan, f, ally, at, where_))
}

fn buff_options(w: &World, plan: &Plan, actor: UnitId, cap: i32, profile: &AiProfile) -> Vec<Step> {
    let mut out = Vec::new();
    let foes = w.foes_of(actor).to_vec();
    for sid in ordered(w, &optional(w, w.book().ally_buffs("taunt", cap, Some(&w.units[actor])), profile), profile) {
        let s = &w.cat.skills[sid];
        if s.grant_status == "fortify" || s.grant_status == "regen" {
            let picked = if s.grant_status == "fortify" { fortify_targets(w, plan, actor, sid) } else { regen_targets(w, plan, actor, sid, profile) };
            for t in picked {
                out.push(if t == actor { PlanStep::self_skill(sid) } else { PlanStep::ally(sid, t) });
            }
            continue;
        }
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
        for (t, _, _) in cands {
            out.push(if t == actor { PlanStep::self_skill(sid) } else { PlanStep::ally(sid, t) });
        }
    }
    out
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
        let taunting = w.units[f].st.has_any("taunt") as i32;
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
    if st == "expose" {
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
    } else if st == "slow" {
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
    if skills.is_empty() {
        return Vec::new();
    }
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
    let mut out = Vec::new();
    for sid in skills {
        let effect = w.cat.skills[sid].status_effect.clone();
        for foe in debuff_order(w, plan, actor, &effect, &by_power, ranked) {
            if debuff_open(w, foe, &effect) {
                out.push(PlanStep::engage(sid, foe));
            }
        }
    }
    out
}

fn guard_smart(w: &World, plan: &mut Plan, actor: UnitId, cap: i32, profile: &AiProfile, foes: &[UnitId], pos: V2) -> bool {
    let near = nearest_foe_unit(w, pos, foes);
    let melee = near.is_some_and(|n| w.units[n].attack_range <= 1);
    let holding = w.units[actor].st.has_any("riposte");
    let riposte_only = |s: SkillId| w.cat.skills[s].grant_status == "riposte";
    let plain_only = |s: SkillId| w.cat.skills[s].grant_status != "riposte";
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
    if profile.reckless || profile.defend_below <= 0.0 || ratio <= profile.defend_below || w.units[actor].st.has_any("riposte") {
        return false;
    }
    let st = plan.compile(w, actor);
    let near = nearest_foe_unit(w, st.pos, foes);
    let Some(near) = near else { return false };
    if st.withdrawn || w.units[near].attack_range > 1 || st.pos.dist(w.pos(near)) != 1 {
        return false;
    }
    for sid in ordered(w, &optional(w, w.book().granting(Kind::Defend, "riposte", cap, Some(&w.units[actor])), profile), profile) {
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
    let mut cands: Vec<(SkillId, V2, f64, usize)> = Vec::new();
    for &sid in &skills {
        let s = &w.cat.skills[sid];
        let band = skill::reach_for(s, &w.units[actor]);
        for aim in aim_cells(w, st.pos, sid, band) {
            let cells: Vec<V2> = skill::area_cells(s, st.pos, aim).into_iter().filter(|c| w.grid.is_valid(*c) && *c != st.pos).collect();
            let foes_hit = foes.iter().filter(|&&f| w.units[f].is_active() && w.has(f) && st.est(f) > 0 && cells.contains(&w.pos(f))).count() as i32;
            let hit_friends: Vec<UnitId> = friends.iter().copied().filter(|&f| f != actor && w.units[f].is_active() && cells.contains(&plan.planned_position(w, f))).collect();
            if foes_hit < profile.area_threshold() || !friendly_fire_ok(w, actor, sid, &hit_friends, foes_hit) {
                continue;
            }
            cands.push((sid, aim, w.units[actor].power_for(s) * foes_hit as f64, hit_friends.len()));
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
        skill::Area::Sweep => [V2::new(1, 1), V2::new(1, -1), V2::new(-1, 1), V2::new(-1, -1)].into_iter().map(|d| pos + d).filter(|c| w.grid.is_valid(*c)).collect(),
        skill::Area::Line => DIRS.into_iter().map(|d| pos + d).filter(|c| w.grid.is_valid(*c)).collect(),
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
        if w.units[u].has_status("taunt") {
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
        let band = skill::reach_for(s, fu);
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
        let cost = skill::ap_cost(s, Some(fu));
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
    if !w.units[taunter].has_status("taunt") {
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
    let mut heals: Vec<SkillId> = w.book().available(Kind::Support, cap, Some(au)).into_iter().filter(|s| skill::heals(&w.cat.skills[*s]) && skill::ap_cost(&w.cat.skills[*s], Some(au)) <= au.ap).collect();
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

