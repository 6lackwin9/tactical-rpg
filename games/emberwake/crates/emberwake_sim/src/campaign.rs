//! Balance sim `campaign` mode (tools/sim/sim_campaign.gd, BAL4-ATTR): a party of three (warrior + two of the other
//! five classes: all 10 trios, or comp=) plays the Ashlands story fights in quest order for both steel choices. HP
//! carries over (statuses reset); a hero at 0 sits out the later fights; the run ends at the first battle not cleared.
//! Between battles `heal` applies: none, full (a downed hero revives), rally (HealRules "won") or a percent of max HP
//! for heroes still up; rest=<b,b> restores everyone before those story battles; extra=<b>:<enc> plays a filler fight
//! before story battle b.

use crate::args::CLASSES;
use crate::runner::Runner;
use crate::scenarios::{self, Enc};
use crate::world;
use aegis_core::fmt::fixed;
use emberwake_rules::stats::MAX_LEVEL;
use indexmap::IndexMap;

/// arms = ashmaw_stand (full finale), gate = ashmaw_stand_barred.
const STEELS: [&str; 2] = ["arms", "gate"];
/// Battle i of a run uses seed + i * SEED_STEP.
const SEED_STEP: i64 = 101;

/// The 10 trios: the warrior plus every pair of the five other classes.
pub fn trios() -> Vec<Vec<String>> {
    let others: Vec<&str> = CLASSES.iter().copied().filter(|c| *c != "warrior").collect();
    let mut out = Vec::new();
    for i in 0..others.len() {
        for j in i + 1..others.len() {
            out.push(vec!["warrior".to_string(), others[i].to_string(), others[j].to_string()]);
        }
    }
    out
}

/// Unit id -> (hp, max).
pub type State = IndexMap<String, (i32, i32)>;

/// Hero HP after the between-battle heal.
pub fn apply_heal(state: &mut State, mode: &str) {
    for (hp, max_hp) in state.values_mut() {
        if mode == "full" {
            *hp = *max_hp;
        } else if mode == "rally" {
            *hp = emberwake_world::progression::hp_after(*hp as i64, *max_hp as i64, "won") as i32;
        } else if mode != "none" && *hp > 0 {
            let pct: i32 = mode.parse().unwrap_or(0);
            *hp = (*max_hp).min(*hp + (*max_hp as f64 * pct as f64 / 100.0).ceil() as i32);
        }
    }
}

fn hp_pct(state: &State) -> f64 {
    let hp: f64 = state.values().map(|s| s.0 as f64).sum();
    let max: f64 = state.values().map(|s| s.1 as f64).sum();
    hp / max.max(1.0) * 100.0
}

struct Run {
    wins: i32,
    /// 1-based story battle (0 = none).
    fell_at: i32,
    /// 0-based index into the fights of the lost fight (-1 = none).
    fell_fight: i32,
    hp: Vec<f64>,
    /// 1-based battle where a jitter PLAN SKIP dropped the run.
    skipped: Option<usize>,
}

/// True when fight `i` is the first entry of its story battle (rest is taken once, before the extras).
fn first_of_story(fights: &[Enc], i: usize) -> bool {
    i == 0 || !fights[i - 1].extra
}

fn run_party(r: &mut Runner, ids: &[String], fights: &[Enc], seed: i64) -> Run {
    let a = r.args;
    let mut state: State = IndexMap::new();
    let (mut wins, mut fell_at, mut fell_fight) = (0, 0, -1);
    let mut hp_after = Vec::new();
    let mut story_no = 0;
    for (i, enc) in fights.iter().enumerate() {
        if !enc.extra {
            story_no += 1;
        }
        let this_no = if enc.extra { story_no + 1 } else { story_no };
        if a.rest.contains(&this_no) && first_of_story(fights, i) {
            apply_heal(&mut state, "full");
        }
        let mut up = Vec::new();
        r.start_hp.clear();
        for id in ids {
            match state.get(id) {
                None => up.push(id.clone()),
                Some(&(hp, _)) if hp > 0 => {
                    up.push(id.clone());
                    r.start_hp.insert(id.clone(), hp);
                }
                _ => {}
            }
        }
        r.rounds_override = enc.max_rounds;
        let res = r.fight(&up, &enc.enemies, seed + i as i64 * SEED_STEP, &enc.base);
        r.start_hp.clear();
        r.rounds_override = 0;
        if res.skipped {
            return Run { wins, fell_at: 0, fell_fight: -1, hp: hp_after, skipped: Some(i + 1) };
        }
        if !res.ok {
            fell_at = this_no;
            fell_fight = i as i32;
            break;
        }
        for id in &up {
            state.insert(id.clone(), (res.party_hp.get(id).copied().unwrap_or(0), res.party_max.get(id).copied().unwrap_or(1)));
        }
        for id in ids {
            state.entry(id.clone()).or_insert((0, 1));
        }
        let n_alive = state.values().filter(|s| s.0 > 0).count();
        if !enc.extra {
            hp_after.push(hp_pct(&state));
        }
        if !res.won || n_alive == 0 {
            fell_at = this_no;
            fell_fight = i as i32;
            break;
        }
        if !enc.extra {
            wins += 1;
        }
        apply_heal(&mut state, &a.heal);
    }
    Run { wins, fell_at, fell_fight, hp: hp_after, skipped: None }
}

fn fights_for(r: &mut Runner, steel: &str) -> Vec<Enc> {
    let a = r.args;
    let foe_level = (a.level + a.elevel).clamp(1, MAX_LEVEL);
    let mut out = Vec::new();
    let mut story_no = 0;
    for f in world::story_fights(r.d, steel == "gate") {
        let Some(enc) = world::encounter(r, &f.encounter, foe_level, a.elevel_set, world::WORLD, true) else { continue };
        story_no += 1;
        for extra_id in a.extra.get(&story_no).cloned().unwrap_or_default() {
            if let Some(mut filler) = world::encounter(r, &extra_id, foe_level, a.elevel_set, world::WORLD, true) {
                filler.extra = true;
                out.push(filler);
            }
        }
        out.push(enc);
    }
    out
}

pub fn campaign_mode(r: &mut Runner) {
    let a = r.args;
    let parties = if a.comp_set { vec![a.comp.clone()] } else { trios() };
    let story: Vec<String> = STEELS
        .iter()
        .map(|steel| format!("{steel}: {}", world::story_fights(r.d, *steel == "gate").iter().map(|f| f.encounter.as_str()).collect::<Vec<_>>().join(" > ")))
        .collect();
    r.print(format!(
        "CAMPAIGN party level {} (foes at authored levels{}), heal {}, {} seeds, {} parties; story order {}",
        a.level,
        if a.elevel_set { format!(" {:+}", a.elevel) } else { String::new() },
        a.heal,
        a.seed_count,
        parties.len(),
        story.join(" | ")
    ));
    for comp in parties {
        let ids = scenarios::define_party(r, &comp, a.level);
        let (mut overall_wins, mut overall_total, mut overall_runs) = (0.0, 0.0, 0usize);
        let name = comp.join(",");
        for steel in STEELS {
            let fights = fights_for(r, steel);
            let total = fights.len();
            let (mut wins_sum, mut clears, mut runs) = (0.0, 0, 0usize);
            let mut fell: IndexMap<i32, i32> = IndexMap::new();
            let (mut hp_sum, mut hp_n): (Vec<f64>, Vec<i32>) = (Vec::new(), Vec::new());
            for s in a.seeds() {
                let run = run_party(r, &ids, &fights, s);
                if let Some(at) = run.skipped {
                    r.print(format!("  {name} steel {steel} seed {s}: skipped at battle {at} (PLAN SKIP: jitter, unreachable target)"));
                    continue;
                }
                runs += 1;
                let mut hp_text = Vec::new();
                for (k, v) in run.hp.iter().enumerate() {
                    hp_text.push((v.round() as i64).to_string());
                    if hp_sum.len() <= k {
                        hp_sum.push(0.0);
                        hp_n.push(0);
                    }
                    hp_sum[k] += v;
                    hp_n[k] += 1;
                }
                wins_sum += run.wins as f64;
                if run.fell_at == 0 {
                    clears += 1;
                } else {
                    *fell.entry(run.fell_at).or_insert(0) += 1;
                }
                let outcome = if run.fell_at == 0 { "cleared the campaign".to_string() } else { format!("fell at battle {} ({})", run.fell_at, fights[run.fell_fight as usize].label) };
                r.print(format!("  {name} steel {steel} seed {s}: survived {}/{total}, {outcome}, party hp% after each battle [{}]", run.wins, hp_text.join(", ")));
            }
            let avg_hp: Vec<String> = hp_sum.iter().zip(&hp_n).map(|(s, n)| ((s / (*n).max(1) as f64).round() as i64).to_string()).collect();
            let fell_text: Vec<String> = (1..=total as i32).filter_map(|b| fell.get(&b).map(|n| format!("b{b} x{n}"))).collect();
            let seeds_n = runs.max(1) as f64;
            r.print(format!(
                "{name:<24} steel {steel:<4} survived {}/{total}, campaign cleared {clears}/{runs}, fell: {}, mean party hp% after: [{}]",
                fixed(wins_sum / seeds_n, 1),
                if fell_text.is_empty() { "never".to_string() } else { fell_text.join(", ") },
                avg_hp.join(", ")
            ));
            overall_wins += wins_sum;
            overall_total += (total * runs) as f64;
            overall_runs += runs;
        }
        let n = overall_runs.max(1) as f64;
        r.print(format!("{name:<24} TRIO summary (both steel choices): survived {} of {} battles per run", fixed(overall_wins / n, 1), fixed(overall_total / n, 1)));
    }
}
