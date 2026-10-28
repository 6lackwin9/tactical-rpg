//! The sim's `feel` mode (RULESET1_TASKS 3, RULESET1-G; engine-only, no Godot original): AI-vs-AI fights of catalog
//! encounters with their authored party, under the timeline ruleset by default, read for how the rules feel rather
//! than for balance scores. Per encounter and in total it reports wins and median rounds; poise (breaks by cause,
//! windup hits and breaks, chance rolls and successes, sure breaks, the lock rate: a unit broken two rounds running);
//! hits (flank and pierce shares, hits into a guard, lost attacks); per side the AP paid per round, the intent share and
//! changes, the intent-bound buffs dropped, follow-ups, Rest blocks and their AP, carried moves, guard blocks and the
//! rounds whose intent had no active (OI1); the party's kite steps and their AP and the planner's reason tags. The last
//! line is the same numbers as JSON (`FEEL_JSON {...}`) for RULESET1-B. Deterministic: the same arguments print the
//! same lines.
//!
//! The runner calls the hooks (`start`, `planned`, `round_end`, `end`) while it fights; they only read the session.

use crate::runner::Runner;
use aegis_core::json;
use indexmap::IndexMap;
use rpgkit_battle::command::{Carry, CmdKind};
use rpgkit_battle::round_intent::{self, RoundIntent};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::Session;
use rpgkit_battle::world::UnitId;
use rpgkit_rules::ruleset;
use serde_json::{json, Map, Value};

/// The seeds the mode runs by default (RULESET1_TASKS 3).
pub const FEEL_SEEDS: usize = 10;
/// The prefix of the machine-readable line.
pub const JSON_PREFIX: &str = "FEEL_JSON ";

/// One side's numbers (party or enemy).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Side {
    /// Units planned, summed over rounds (the denominator of the per-round numbers).
    pub unit_rounds: i64,
    /// AP paid (a unit's AP when the round was planned less its AP at the round's end).
    pub ap_paid: i64,
    /// Each unit-round's intent ("none" without one).
    pub intents: IndexMap<String, i64>,
    /// A unit's intent differed from its previous round's.
    pub intent_changes: i64,
    /// IN2, IN6: intent-bound buffs dropped by an intent change (read from the session's events, where the party's
    /// planning writes them: the enemy side stays 0).
    pub buffs_dropped: i64,
    /// FU2: landed skill and item commands after a unit's first of the round.
    pub follow_ups: i64,
    /// RS2: Rest blocks that landed, and the AP they pay at the round's end.
    pub rest_blocks: i64,
    pub rest_ap: i64,
    /// MV4, MV8: landed commands with a carried move (Engage or Disengage).
    pub carried_moves: i64,
    /// Blocks of landed Defend commands.
    pub guard_blocks: i64,
    /// OI1: unit-rounds (deck rule) whose intent had no active of that intent.
    pub no_active: i64,
}

impl Side {
    fn add(&mut self, o: &Side) {
        self.unit_rounds += o.unit_rounds;
        self.ap_paid += o.ap_paid;
        for (k, v) in &o.intents {
            *self.intents.entry(k.clone()).or_default() += v;
        }
        self.intent_changes += o.intent_changes;
        self.buffs_dropped += o.buffs_dropped;
        self.follow_ups += o.follow_ups;
        self.rest_blocks += o.rest_blocks;
        self.rest_ap += o.rest_ap;
        self.carried_moves += o.carried_moves;
        self.guard_blocks += o.guard_blocks;
        self.no_active += o.no_active;
    }

    fn to_json(&self) -> Value {
        let share: Map<String, Value> = self.intents.iter().map(|(k, v)| (k.clone(), json!(round1(pct(*v, self.unit_rounds))))).collect();
        json!({
            "unit_rounds": self.unit_rounds, "ap_per_round": round2(ratio(self.ap_paid, self.unit_rounds)), "intent_share_pct": share,
            "intent_changes": self.intent_changes, "buffs_dropped": self.buffs_dropped, "follow_ups": self.follow_ups,
            "rest_blocks": self.rest_blocks, "rest_ap": self.rest_ap, "carried_moves": self.carried_moves, "guard_blocks": self.guard_blocks,
            "no_active_rounds": self.no_active,
        })
    }

    fn line(&self, name: &str) -> String {
        let share: Vec<String> = self.intents.iter().map(|(k, v)| format!("{k} {}%", round1(pct(*v, self.unit_rounds)))).collect();
        format!(
            "  {name}: AP/round {}, intents {}, changes {}, buffs dropped {}, follow-ups {}, Rest blocks {} (AP {}), carried moves {}, guard blocks {}, no-active rounds {}",
            round2(ratio(self.ap_paid, self.unit_rounds)),
            if share.is_empty() { "-".to_string() } else { share.join(" ") },
            self.intent_changes,
            self.buffs_dropped,
            self.follow_ups,
            self.rest_blocks,
            self.rest_ap,
            self.carried_moves,
            self.guard_blocks,
            self.no_active
        )
    }
}

/// One encounter's numbers (or the total).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Counts {
    pub fights: i64,
    pub won: i64,
    /// Each fight's rounds (the median's input).
    pub rounds: Vec<i32>,
    /// Breaks by cause (`poise`, `chance`, `sure`, `collision`: the log's BREAK cells).
    pub breaks: IndexMap<String, i64>,
    /// P5: damaging hits on a unit in windup, and those that broke it.
    pub windup_hits: i64,
    pub windup_breaks: i64,
    /// P13: break chance rolls and the breaks they gave.
    pub chance_rolls: i64,
    pub chance_breaks: i64,
    /// P14: `sure_break` breaks.
    pub sure_breaks: i64,
    /// P10: unit-rounds with a poise bar, and those broken two rounds running.
    pub bar_rounds: i64,
    pub locked: i64,
    /// Damaging hits of landed commands; flank hits (MV3); hits of armour-piercing commands under `pierce` (A2); hits
    /// into a guard (P6's guard factor below 1).
    pub hits: i64,
    pub flank_hits: i64,
    pub pierce_hits: i64,
    pub guarded_hits: i64,
    /// GD5: attacks that lost their locked target.
    pub lost_attacks: i64,
    /// The party's kite steps (planner tag `kite: ...`) and the AP of their entries.
    pub kite_steps: i64,
    pub kite_ap: i64,
    /// PL6: the party planner's reason tags.
    pub reasons: IndexMap<String, i64>,
    pub party: Side,
    pub enemy: Side,
}

impl Counts {
    fn add(&mut self, o: &Counts) {
        self.fights += o.fights;
        self.won += o.won;
        self.rounds.extend(&o.rounds);
        for (k, v) in &o.breaks {
            *self.breaks.entry(k.clone()).or_default() += v;
        }
        self.windup_hits += o.windup_hits;
        self.windup_breaks += o.windup_breaks;
        self.chance_rolls += o.chance_rolls;
        self.chance_breaks += o.chance_breaks;
        self.sure_breaks += o.sure_breaks;
        self.bar_rounds += o.bar_rounds;
        self.locked += o.locked;
        self.hits += o.hits;
        self.flank_hits += o.flank_hits;
        self.pierce_hits += o.pierce_hits;
        self.guarded_hits += o.guarded_hits;
        self.lost_attacks += o.lost_attacks;
        self.kite_steps += o.kite_steps;
        self.kite_ap += o.kite_ap;
        for (k, v) in &o.reasons {
            *self.reasons.entry(k.clone()).or_default() += v;
        }
        self.party.add(&o.party);
        self.enemy.add(&o.enemy);
    }

    /// The median of the fights' rounds (the mean of the middle two for an even count; 0 without fights).
    pub fn median_rounds(&self) -> f64 {
        let mut r = self.rounds.clone();
        r.sort_unstable();
        match r.len() {
            0 => 0.0,
            n if n % 2 == 1 => r[n / 2] as f64,
            n => (r[n / 2 - 1] + r[n / 2]) as f64 / 2.0,
        }
    }

    fn total_breaks(&self) -> i64 {
        self.breaks.values().sum()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "fights": self.fights, "won_pct": round1(pct(self.won, self.fights)), "median_rounds": self.median_rounds(),
            "breaks": self.total_breaks(), "breaks_by_cause": counts_json(&self.breaks), "windup_hits": self.windup_hits, "windup_breaks": self.windup_breaks,
            "chance_rolls": self.chance_rolls, "chance_breaks": self.chance_breaks, "sure_breaks": self.sure_breaks,
            "lock_rate_pct": round1(pct(self.locked, self.bar_rounds)),
            "hits": self.hits, "flank_hits": self.flank_hits, "flank_pct": round1(pct(self.flank_hits, self.hits)),
            "pierce_hits": self.pierce_hits, "pierce_pct": round1(pct(self.pierce_hits, self.hits)), "guarded_hits": self.guarded_hits,
            "lost_attacks": self.lost_attacks, "kite_steps": self.kite_steps, "kite_ap": self.kite_ap, "reasons": counts_json(&self.reasons),
            "party": self.party.to_json(), "enemy": self.enemy.to_json(),
        })
    }

    fn lines(&self, title: &str) -> Vec<String> {
        let causes: Vec<String> = self.breaks.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let reasons: Vec<String> = self.reasons.iter().map(|(k, v)| format!("{k} {v}")).collect();
        vec![
            format!("FEEL {title}: {} fights, won {}%, median rounds {}", self.fights, round1(pct(self.won, self.fights)), self.median_rounds()),
            format!(
                "  poise: breaks {}{}, windup hits {} (breaks {}), chance rolls {} (breaks {}), sure breaks {}, lock rate {}%",
                self.total_breaks(),
                if causes.is_empty() { String::new() } else { format!(" ({})", causes.join(", ")) },
                self.windup_hits,
                self.windup_breaks,
                self.chance_rolls,
                self.chance_breaks,
                self.sure_breaks,
                round1(pct(self.locked, self.bar_rounds))
            ),
            format!(
                "  hits: {}, flank {} ({}%), pierce {} ({}%), into a guard {}, lost attacks {}, kite steps {} (AP {})",
                self.hits,
                self.flank_hits,
                round1(pct(self.flank_hits, self.hits)),
                self.pierce_hits,
                round1(pct(self.pierce_hits, self.hits)),
                self.guarded_hits,
                self.lost_attacks,
                self.kite_steps,
                self.kite_ap
            ),
            self.party.line("party"),
            self.enemy.line("enemy"),
            format!("  planner reasons: {}", if reasons.is_empty() { "-".to_string() } else { reasons.join(", ") }),
        ]
    }
}

/// The observer of the fights of one encounter: its counts and the running fight's state.
#[derive(Debug, Default)]
pub struct Feel {
    pub counts: Counts,
    /// Each unit's intent last round (this fight).
    last_intent: IndexMap<UnitId, String>,
    /// Each unit's AP when the round was planned.
    ap_planned: IndexMap<UnitId, i32>,
    /// Session events already read.
    events_seen: usize,
}

/// A planner reason tag without its numbers ("rest: AP 0 < 2" -> "rest: AP", "follow-up 1" -> "follow-up"), so the
/// tally counts kinds.
fn reason_kind(tag: &str) -> String {
    tag.split(|ch: char| ch.is_ascii_digit()).next().unwrap_or("").trim().to_string()
}

/// A tally as a JSON object (its order kept).
fn counts_json(m: &IndexMap<String, i64>) -> Value {
    Value::Object(m.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
}

fn pct(n: i64, d: i64) -> f64 {
    if d <= 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

fn ratio(n: i64, d: i64) -> f64 {
    if d <= 0 {
        0.0
    } else {
        n as f64 / d as f64
    }
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// The units on the field and able to act.
fn active_units(s: &Session) -> Vec<UnitId> {
    s.w.allies.iter().chain(&s.w.enemies).copied().filter(|&c| s.w.units[c].is_active() && s.w.has(c)).collect()
}

impl Feel {
    fn side(&mut self, s: &Session, c: UnitId) -> &mut Side {
        if s.w.allies.contains(&c) {
            &mut self.counts.party
        } else {
            &mut self.counts.enemy
        }
    }

    /// A fight starts (after its setup): the per-fight state resets.
    pub fn start(&mut self, s: &Session) {
        self.last_intent.clear();
        self.ap_planned.clear();
        self.events_seen = s.events.len();
    }

    /// The round is planned and confirmed: intents, intent changes and the buffs they dropped, OI1, AP, the party's
    /// planner reasons and kite steps.
    pub fn planned(&mut self, s: &Session) {
        let w = &s.w;
        let dropped = s.events[self.events_seen.min(s.events.len())..].iter().filter(|l| l.contains(" leaves ") && l.ends_with(" ends")).count() as i64;
        self.events_seen = s.events.len();
        self.counts.party.buffs_dropped += dropped;
        for c in active_units(s) {
            let intent = round_intent::intent_of(w, c);
            let id = intent.map_or("none", RoundIntent::id).to_string();
            let changed = self.last_intent.get(&c).is_some_and(|prev| *prev != id);
            // OI1: the deck's actives (their skills now) hold none of the round's intent
            let u = &w.units[c];
            let no_active = u.deck_rule.is_some() && intent.is_some_and(|i| !u.active_families().iter().any(|f| rip::family_skill(w, c, f).is_some_and(|sid| round_intent::skill_intent(&w.cat.skills[sid]) == i.id())));
            self.ap_planned.insert(c, u.ap);
            let side = self.side(s, c);
            side.unit_rounds += 1;
            *side.intents.entry(id.clone()).or_default() += 1;
            side.intent_changes += changed as i64;
            side.no_active += no_active as i64;
            self.last_intent.insert(c, id);
        }
        for &c in &w.allies {
            if !w.units[c].is_active() {
                continue;
            }
            let entries = s.plan.entries_for(w, c);
            for (k, tags) in rip::reasons(w, &s.plan, c).into_iter().enumerate() {
                for tag in tags.split(';').map(reason_kind).filter(|t| !t.is_empty()) {
                    if tag.starts_with("kite") {
                        self.counts.kite_steps += 1;
                        self.counts.kite_ap += entries.get(k).map_or(0, |e| e.total_ap()) as i64;
                    }
                    *self.counts.reasons.entry(tag).or_default() += 1;
                }
            }
        }
    }

    /// The round's blocks are resolved: the landed commands, the stagger records, the Rest blocks, the locks, AP.
    pub fn round_end(&mut self, s: &Session) {
        let w = &s.w;
        let pierce_on = w.ruleset.has(ruleset::PIERCE);
        let mut firsts: IndexMap<UnitId, i32> = IndexMap::new();
        let mut actions: Vec<_> = s.resolver.scheduled.iter().filter(|a| a.landed).collect();
        actions.sort_by_key(|a| (a.unit, a.start));
        for a in actions {
            let cmd = &a.command;
            let skill_like = matches!(cmd.kind, CmdKind::Attack { .. } | CmdKind::Area { .. } | CmdKind::Support { .. } | CmdKind::Focus { .. } | CmdKind::Defend { .. } | CmdKind::Item { .. });
            let follow = skill_like && *firsts.entry(a.unit).and_modify(|n| *n += 1).or_insert(0) > 0;
            let carried = !matches!(cmd.carry, Carry::None);
            let guard = if matches!(cmd.kind, CmdKind::Defend { .. }) { (a.end - a.start + 1).max(0) as i64 } else { 0 };
            let side = self.side(s, a.unit);
            side.follow_ups += follow as i64;
            side.carried_moves += carried as i64;
            side.guard_blocks += guard;
            let c = &mut self.counts;
            c.lost_attacks += cmd.evaded.is_some() as i64;
            c.hits += cmd.hits.len() as i64;
            c.flank_hits += cmd.hits.iter().filter(|h| h.1).count() as i64;
            if pierce_on && cmd.impact.armor_piercing {
                c.pierce_hits += cmd.hits.len() as i64;
            }
            for st in &cmd.staggers {
                c.windup_hits += st.windup as i64;
                c.windup_breaks += (st.windup && st.broke.is_some()) as i64;
                c.guarded_hits += (st.factors.guard < 1.0) as i64;
                if let Some((chance, roll)) = st.chance {
                    c.chance_rolls += 1;
                    c.chance_breaks += (roll <= chance) as i64;
                }
                c.sure_breaks += st.broke.as_ref().is_some_and(|b| b.cause == rpgkit_battle::poise::BreakCause::Sure) as i64;
            }
        }
        for c in s.w.allies.iter().chain(&s.w.enemies).copied() {
            if let Some(p) = w.poise.get(&c).filter(|p| p.max > 0 && w.units[c].is_alive()) {
                self.counts.bar_rounds += 1;
                self.counts.locked += (p.broken && p.broken_last) as i64;
            }
            let Some(planned) = self.ap_planned.get(&c).copied() else { continue };
            let blocks = w.round_states.get(&c).map_or(0, |r| r.rest_blocks);
            let gain = if w.pays_ap(c) && w.units[c].is_active() { round_intent::rest_gain(&w.ruleset.numbers, blocks) } else { 0 };
            let paid = (planned - w.units[c].ap).max(0) as i64;
            let side = self.side(s, c);
            side.ap_paid += paid;
            side.rest_blocks += blocks as i64;
            side.rest_ap += gain as i64;
        }
        self.ap_planned.clear();
    }

    /// The fight is over: its outcome, rounds and the log's breaks by cause.
    pub fn end(&mut self, s: &Session, won: bool, rounds: i32) {
        let c = &mut self.counts;
        c.fights += 1;
        c.won += won as i64;
        c.rounds.push(rounds);
        for e in s.log.iter().filter(|e| e.outcome == "BREAK") {
            *c.breaks.entry(e.status.clone()).or_default() += 1;
        }
    }
}

/// The `feel` mode: each of the arguments' encounters (catalog encounters with a party) fought over every seed with its
/// authored party, the encounter's round limit and board; a block of lines per encounter, the total, then FEEL_JSON.
pub fn feel_mode(r: &mut Runner) {
    let rules = r.fight_ruleset();
    let seeds = r.args.seeds();
    r.print(format!("FEEL: {} rules [{}], {} seeds per encounter, party AI preset {}", rules.name, rules.rules.join(", "), seeds.len(), r.preset));
    let mut total = Counts::default();
    let mut per: Map<String, Value> = Map::new();
    for id in r.args.encounters.clone() {
        let entry = r.d.entry("encounters", &id).clone();
        let party = json::sget(&entry, "party");
        let enemies = json::sget(&entry, "enemies");
        // the encounter as authored but its party, enemies, round limit and seed (the runner's)
        let mut base = Map::new();
        for key in ["zone", "arena", "exits", "rows", "total_blocks", "unit_ap", "ap_regen", "deployment", "objective", "phases", "guests"] {
            if let Some(v) = entry.get(key) {
                base.insert(key.into(), v.clone());
            }
        }
        r.rounds_override = json::i(&entry, "max_rounds", r.args.cap as i64) as i32;
        r.feel = Some(Feel::default());
        for &seed in &seeds {
            r.fight(&party, &enemies, seed, &base);
        }
        r.rounds_override = 0;
        let counts = r.feel.take().map(|f| f.counts).unwrap_or_default();
        let lines = counts.lines(&format!("{id} ({})", json::s_or(&entry, "name", &id)));
        r.out.extend(lines);
        per.insert(id.clone(), counts.to_json());
        total.add(&counts);
    }
    let lines = total.lines("TOTAL");
    r.out.extend(lines);
    let (fails, skips) = (r.plan_fails, r.plan_skips);
    r.print(format!("plan checks: {fails} PLAN FAIL, {skips} PLAN SKIP over {} fights", r.fights));
    let out = json!({"ruleset": rules.id, "rules": rules.rules, "seeds": seeds.len(), "plan_fails": fails, "plan_skips": skips, "encounters": per, "total": total.to_json()});
    r.print(format!("{JSON_PREFIX}{out}"));
}
