//! The sim's `feel` mode (RULESET1_TASKS 3, RULESET1-G; engine-only, no Godot original): AI-vs-AI fights of catalog
//! encounters with their authored party, under the timeline ruleset by default, read for how the rules feel rather
//! than for balance scores. Per encounter and in total it reports the outcomes (won, lost, capped: the round limit with
//! both sides standing, fled; RULESET1-G2) and the median rounds of every fight and of the won ones; poise (breaks by cause,
//! windup hits and breaks, chance rolls and successes, sure breaks, the lock rate: a unit broken two rounds running);
//! hits (flank and pierce shares, hits into a guard, lost attacks); per side the AP paid per round, the intent share and
//! changes, the intent-bound buffs dropped, follow-ups, Rest blocks and their AP, carried moves, guard blocks and the
//! rounds whose intent had no active (OI1); the party's kite steps and their AP and the planner's reason tags. The last
//! line is the same numbers as JSON (`FEEL_JSON {...}`) for RULESET1-B. Deterministic: the same arguments print the
//! same lines. RULESET1-G2: the party plays by the auto driver (`Session::auto_draft`, the battle screen's autoplay)
//! with the `tactic=` tactic, or by the `timed` driver (timed.rs); `party=both` prints a block (and a FEEL_JSON line) per
//! driver. `armor=` and `tune=` change the run's in-memory units and ruleset only.
//!
//! SIMCOUNT1: `counts=on` adds the rule counts of `RULE_COUNTERS` per side to each block and to FEEL_JSON. It is a key, not
//! the default, because `sim_golden` holds the default lines. A new count is one row of that table. PK2-FOLLOWUP
//! (PR5-36): it also reports each side's weapon wear (`worn`, `dulled`; non-zero only with `durability=` under the rule
//! `durability`, which `feel` applies as every mode does).
//!
//! WORLD-SIM1 (PR5-42, FB-239 (a)): an encounter may be a world encounter (`world:<world>/<id>`): it fights with its
//! authored side as its world builds it (`Runner::fight_world`, `rpgkit_world::battle::authored_fight`), by the world's
//! ruleset unless `ruleset=` picks one. `counts=on` also adds the hook counts (`HOOK_COUNTERS`, read off the battle
//! metrics' HOOKS1 4.2 records), the skills landed by id, each unit's intents, and per encounter the arena strikes, the windup breaks by
//! skill, the phases fired and the fights with each count (CAST1 T10-T14).
//!
//! The runner calls the hooks (`start`, `planned`, `round_end`, `end`) while it fights; they only read the session.

use crate::runner::Runner;
use aegis_core::json;
use indexmap::IndexMap;
use rpgkit_battle::command::{self, Carry, CmdKind, MoveKind};
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::resolver::ScheduledAction;
use rpgkit_battle::round_intent::{self, RoundIntent};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::Session;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_rules::ruleset;
use serde_json::{json, Map, Value};

/// The seeds the mode runs by default (RULESET1_TASKS 3).
pub const FEEL_SEEDS: usize = 10;
/// The prefix of the machine-readable line.
pub const JSON_PREFIX: &str = "FEEL_JSON ";

/// SIMCOUNT1: reads one count from a scheduled action at the round's end, every action including the cancelled ones.
pub type RuleCount = fn(&World, &ScheduledAction) -> i64;

/// SIMCOUNT1: the rule counts, by the side of the acting unit: JSON key, human label, how an action counts. ADD A COUNTER
/// HERE: one row, and the JSON and the human line carry it. Read structured command fields, never log strings.
pub const RULE_COUNTERS: &[(&str, &str, RuleCount)] = &[
    ("protected_flanks", "protected flanks", |_, a| a.command.missed as i64),
    ("maneuver_commands", "Maneuver commands", |_, a| is_maneuver(a) as i64),
    ("dashes", "dashes", |_, a| a.command.dashed as i64),
    ("diagonal_hits", "diagonal hits", |_, a| a.command.diagonal_hit as i64),
    ("chases_melee", "melee chases", |w, a| (chase_kind(w, a) == Some(ChaseKind::Melee)) as i64),
    ("chases_ranged", "ranged chases", |w, a| (chase_kind(w, a) == Some(ChaseKind::Ranged)) as i64),
    ("chases_support", "Support chases", |w, a| (chase_kind(w, a) == Some(ChaseKind::Support)) as i64),
    ("items_used", "items used", |_, a| matches!(a.command.kind, CmdKind::Item { used: true, .. }) as i64),
    ("dodges", "dodges", |_, a| a.command.dodged.is_some() as i64),
    // PR5-26 (FB-191 (f)): the Channel rule's actions that landed: channeled versions, and Vents (`is_vent`)
    ("channeled_actions", "channeled actions", |_, a| (a.landed && a.command.channeled) as i64),
    ("vents", "Vents", |w, a| (a.landed && is_vent(w, a)) as i64),
];

/// WORLD-SIM1 (PR5-42): reads one unit's hook count for the fight from the battle metrics (HOOKS1 4.2), by its display
/// name.
pub type HookCount = fn(&BattleMetrics, &str) -> i64;

/// WORLD-SIM1 (PR5-42, CAST1 T12-T14): the hook counts (`counts=on`), by the side of the unit the metrics credit: JSON
/// key, human label, how a unit counts. Pure reads of the battle metrics' HOOKS1 4.2 records (triggers and riders by
/// their holder, hazards placed and traps sprung by their placer, pushes, pulls and collisions by their driver, arena
/// strikes' hits, pushes and collisions by the struck unit). ADD A COUNTER HERE: one row.
pub const HOOK_COUNTERS: &[(&str, &str, HookCount)] = &[
    ("triggers_fired", "triggers fired", |m, n| m.triggers.get(n).map_or(0, |r| r.fired.into())),
    ("riders", "riders", |m, n| m.triggers.get(n).map_or(0, |r| r.riders.into())),
    ("hazards_placed", "hazards placed", |m, n| m.hazards.get(n).map_or(0, |r| r.placed.into())),
    ("traps_sprung", "traps sprung", |m, n| m.hazards.get(n).map_or(0, |r| r.traps.into())),
    ("hazard_hits", "hazard hits taken", |m, n| m.hazards.get(n).map_or(0, |r| r.hits.into())),
    ("pushes", "pushes", |m, n| m.forced.get(n).map_or(0, |r| r.pushes.into())),
    ("pulls", "pulls", |m, n| m.forced.get(n).map_or(0, |r| r.pulls.into())),
    ("collisions", "collisions", |m, n| m.forced.get(n).map_or(0, |r| r.collisions.into())),
    ("arena_hits", "arena hits taken", |m, n| m.arena.get(n).map_or(0, |r| r.struck.into())),
    ("arena_pushed", "arena pushes taken", |m, n| m.arena.get(n).map_or(0, |r| r.pushed.into())),
    ("arena_collisions", "arena collisions", |m, n| m.arena.get(n).map_or(0, |r| r.collisions.into())),
];

/// WORLD-SIM1: the `fights_with` key of the fights with a windup break by skill `skill`.
pub const WINDUP_BREAK_BY: &str = "windup_break:";

/// PR5-26: a Vent: a channeled-only skill (`channel.only`) whose meter cost lowers the channel meter.
fn is_vent(w: &World, a: &ScheduledAction) -> bool {
    let m = rpgkit_battle::channel::meter_id(w);
    !m.is_empty() && w.cat.skill(&a.command.skill).is_some_and(rpgkit_battle::channel::only) && a.command.meters.iter().any(|d| d.meter == m && d.delta < 0)
}

/// A command that started as a dash to a position tile (INPUT1 MR1, MR8), whatever came of it: it dashed, stayed on its
/// tile, could not reach, or found the flank protected.
fn is_maneuver(a: &ScheduledAction) -> bool {
    let c = &a.command;
    c.started && c.position.is_some() && matches!(c.carry, Carry::Engage(_)) && matches!(c.kind, CmdKind::Attack { .. })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChaseKind {
    Melee,
    Ranged,
    Support,
}

/// What kind of skill chased at its start (INPUT1 MR16). An attack whose band reaches at most 2 is melee, reach weapons
/// too, as MR10's `round_intent::diagonal` reads it. A dash whose budget took MR16's extra tile counts as a dash.
fn chase_kind(w: &World, a: &ScheduledAction) -> Option<ChaseKind> {
    if a.command.moved != Some(MoveKind::Chase) {
        return None;
    }
    match a.command.kind {
        CmdKind::Support { .. } => Some(ChaseKind::Support),
        CmdKind::Attack { min_range, max_range, .. } if matches!(command::reach_band(w, a.unit, min_range, max_range), (0..=1, 0..=2)) => Some(ChaseKind::Melee),
        CmdKind::Attack { .. } => Some(ChaseKind::Ranged),
        _ => None,
    }
}

/// One side's numbers (party or enemy).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Side {
    /// Units planned, summed over rounds (the denominator of the per-round numbers).
    pub unit_rounds: i64,
    /// R1-BAL (FB-97 (1)(A)): unit-rounds of a unit that pays no AP (`World::pays_ap` false: enemies under the
    /// timeline's `enemy_moves`). When every unit-round is one, the human line prints "no AP (enemy_moves)" instead of
    /// a 0 AP/round that reads like a metric gap.
    pub unpaid_rounds: i64,
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
    /// Blocks a unit started guarded (S2.2-S3, FB-85: under Guard Option 1C a Defend's guard runs from its wind-up start
    /// to the round's end, so the resolver's `guarded` blocks, not the Defend's own span).
    pub guard_blocks: i64,
    /// OI1: unit-rounds (deck rule) whose intent had no active of that intent.
    pub no_active: i64,
    /// SIMCOUNT1: `RULE_COUNTERS` by key (a key with no count yet is absent; `rule_count` reads 0).
    pub rule_counts: IndexMap<&'static str, i64>,
    /// SIMCOUNT1: the pouch items used, by item id.
    pub items_by_id: IndexMap<String, i64>,
    /// PR5-26 (FB-191 (f), counts=on): the Channel rule's numbers: unit-rounds played as a Channel round, the channel
    /// meter's own triggers fired (its crisis), unit-rounds ending in each band of the channel meter (by band name; "-"
    /// outside every band), and the poise breaks the side's units took. PR5-26 follow-up (FB-201): `crises`, the firings
    /// of the channel meter's crisis rows (`crisis_at`: a `round_end` row that needs the meter at its max), counted as
    /// they fire (the bands sample after a crisis reset, so they never show the max band).
    pub channel_rounds: i64,
    pub channel_triggers: i64,
    pub crises: i64,
    pub channel_bands: IndexMap<String, i64>,
    pub breaks_taken: i64,
    /// PK2-FOLLOWUP (PR5-36, FB-218 (c-e), FB-220): the durability the side's tracked weapons spent and how many ended a
    /// fight dulled (`rpgkit_battle::wear::tally`; 0 without `durability=` and the rule `durability`).
    pub worn: i64,
    pub dulled: i64,
    /// WORLD-SIM1 (PR5-42, counts=on): `HOOK_COUNTERS` by key (absent = 0; `hook_count` reads it) and the landed skill
    /// commands by skill id.
    pub hooks: IndexMap<&'static str, i64>,
    pub skills_landed: IndexMap<String, i64>,
    /// WORLD-SIM1 (counts=on): each unit's round intents by its display name ("none" without one), so a Founder's
    /// `default_intent` and its intent variety read per unit.
    pub intents_by_unit: IndexMap<String, IndexMap<String, i64>>,
}

impl Side {
    fn add(&mut self, o: &Side) {
        self.unit_rounds += o.unit_rounds;
        self.unpaid_rounds += o.unpaid_rounds;
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
        for (k, v) in &o.rule_counts {
            *self.rule_counts.entry(k).or_default() += v;
        }
        for (k, v) in &o.items_by_id {
            *self.items_by_id.entry(k.clone()).or_default() += v;
        }
        self.channel_rounds += o.channel_rounds;
        self.channel_triggers += o.channel_triggers;
        self.crises += o.crises;
        for (k, v) in &o.channel_bands {
            *self.channel_bands.entry(k.clone()).or_default() += v;
        }
        self.breaks_taken += o.breaks_taken;
        self.worn += o.worn;
        self.dulled += o.dulled;
        for (k, v) in &o.hooks {
            *self.hooks.entry(k).or_default() += v;
        }
        for (k, v) in &o.skills_landed {
            *self.skills_landed.entry(k.clone()).or_default() += v;
        }
        for (unit, m) in &o.intents_by_unit {
            let mine = self.intents_by_unit.entry(unit.clone()).or_default();
            for (k, v) in m {
                *mine.entry(k.clone()).or_default() += v;
            }
        }
    }

    /// WORLD-SIM1: one `HOOK_COUNTERS` count (0 when nothing counted it).
    pub fn hook_count(&self, key: &str) -> i64 {
        self.hooks.get(key).copied().unwrap_or(0)
    }

    /// SIMCOUNT1: one `RULE_COUNTERS` count (0 when nothing counted it).
    pub fn rule_count(&self, key: &str) -> i64 {
        self.rule_counts.get(key).copied().unwrap_or(0)
    }

    /// SIMCOUNT1: the protected share of the Maneuver commands, in percent.
    fn protected_share(&self) -> f64 {
        round1(pct(self.rule_count("protected_flanks"), self.rule_count("maneuver_commands")))
    }

    /// SIMCOUNT1: the rule counts as JSON; `fights` and `rounds` (fight rounds, all the fights') are the denominators.
    fn rule_json(&self, fights: i64, rounds: i64) -> Value {
        let counts: Map<String, Value> = RULE_COUNTERS.iter().map(|(k, _, _)| (k.to_string(), json!(self.rule_count(k)))).collect();
        let per_fight: Map<String, Value> = RULE_COUNTERS.iter().map(|(k, _, _)| (k.to_string(), json!(round2(ratio(self.rule_count(k), fights))))).collect();
        json!({
            "counts": counts, "per_fight": per_fight, "protected_share_pct": self.protected_share(),
            "rest_blocks_per_round": round2(ratio(self.rest_blocks, rounds)), "items_by_id": counts_json(&self.items_by_id),
            "channel": {"rounds": self.channel_rounds, "meter_triggers": self.channel_triggers, "crises": self.crises, "bands": counts_json(&self.channel_bands)},
            "breaks_taken": self.breaks_taken, "wear": {"worn": self.worn, "dulled": self.dulled},
            "hooks": Value::Object(HOOK_COUNTERS.iter().map(|(k, _, _)| (k.to_string(), json!(self.hook_count(k)))).collect()),
            "skills_landed": counts_json(&self.skills_landed),
            "intents_by_unit": Value::Object(self.intents_by_unit.iter().map(|(u, m)| (u.clone(), counts_json(m))).collect()),
        })
    }

    /// SIMCOUNT1: the human line of the rule counts.
    fn rule_line(&self, name: &str, fights: i64, rounds: i64) -> String {
        let counts: Vec<String> = RULE_COUNTERS.iter().map(|(k, label, _)| format!("{label} {} ({}/fight)", self.rule_count(k), round2(ratio(self.rule_count(k), fights)))).collect();
        let items: Vec<String> = self.items_by_id.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let bands: Vec<String> = self.channel_bands.iter().map(|(k, v)| format!("{k} {v}")).collect();
        format!(
            "  {name} rule counts: {}; protected share {}%, Rest blocks/round {}, items {}; Channel rounds {}, channel meter triggers {}, crises {}, rounds by band {}; breaks taken {}; worn {}, dulled {}",
            counts.join(", "),
            self.protected_share(),
            round2(ratio(self.rest_blocks, rounds)),
            if items.is_empty() { "-".to_string() } else { items.join(" ") },
            self.channel_rounds,
            self.channel_triggers,
            self.crises,
            if bands.is_empty() { "-".to_string() } else { bands.join(" ") },
            self.breaks_taken,
            self.worn,
            self.dulled
        )
    }

    /// WORLD-SIM1: the human line of the hook counts and the skills landed (counts=on).
    fn hook_line(&self, name: &str) -> String {
        let hooks: Vec<String> = HOOK_COUNTERS.iter().map(|(k, label, _)| format!("{label} {}", self.hook_count(k))).collect();
        let landed: Vec<String> = self.skills_landed.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let intents: Vec<String> = self.intents_by_unit.iter().map(|(u, m)| format!("{u}: {}", m.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(" "))).collect();
        format!(
            "  {name} hooks: {}; skills landed {}; intents by unit {}",
            hooks.join(", "),
            if landed.is_empty() { "-".to_string() } else { landed.join(" ") },
            if intents.is_empty() { "-".to_string() } else { intents.join(", ") }
        )
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

    /// R1-BAL (FB-97 (1)(A)): the AP/round text: "no AP (enemy_moves)" when every unit-round paid no AP by rule, else
    /// the number.
    fn ap_text(&self) -> String {
        if self.unit_rounds > 0 && self.unpaid_rounds >= self.unit_rounds {
            "no AP (enemy_moves)".to_string()
        } else {
            round2(ratio(self.ap_paid, self.unit_rounds)).to_string()
        }
    }

    fn line(&self, name: &str) -> String {
        let share: Vec<String> = self.intents.iter().map(|(k, v)| format!("{k} {}%", round1(pct(*v, self.unit_rounds)))).collect();
        format!(
            "  {name}: AP/round {}, intents {}, changes {}, buffs dropped {}, follow-ups {}, Rest blocks {} (AP {}), carried moves {}, guard blocks {}, no-active rounds {}",
            self.ap_text(),
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
    /// RULESET1-G2: the outcomes, summing to `fights`: won, lost, capped (the round limit, or a refused plan, with both
    /// sides standing: the session gave no outcome) and fled (the party left).
    pub won: i64,
    pub lost: i64,
    pub capped: i64,
    pub fled: i64,
    /// Each fight's rounds (the median's input), and the won fights' (PACE1 reads won-fight medians).
    pub rounds: Vec<i32>,
    pub won_rounds: Vec<i32>,
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
    /// RULESET1-G2 (4.3): party unit-rounds the `timed` driver re-planned into a visible windup (0 under `auto`).
    pub timed_rounds: i64,
    pub party: Side,
    pub enemy: Side,
    /// WORLD-SIM1 (PR5-42, CAST1 T10-T14; printed with counts=on): the arena strikes (HOOKS1 4.2), the windup breaks by
    /// the breaking command's skill, the fights each encounter phase fired in (by phase id), and the fights with at least
    /// one of a count (`fights_with`: every `HOOK_COUNTERS` key over both sides, `arena_strikes`, `windup_breaks` and
    /// `WINDUP_BREAK_BY<skill>`), so "in most seeds" reads off one run.
    pub arena_strikes: i64,
    pub windup_breaks_by_skill: IndexMap<String, i64>,
    pub phases_fired: IndexMap<String, i64>,
    pub fights_with: IndexMap<String, i64>,
}

impl Counts {
    fn add(&mut self, o: &Counts) {
        self.fights += o.fights;
        self.won += o.won;
        self.lost += o.lost;
        self.capped += o.capped;
        self.fled += o.fled;
        self.rounds.extend(&o.rounds);
        self.won_rounds.extend(&o.won_rounds);
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
        self.timed_rounds += o.timed_rounds;
        self.party.add(&o.party);
        self.enemy.add(&o.enemy);
        self.arena_strikes += o.arena_strikes;
        for (k, v) in &o.windup_breaks_by_skill {
            *self.windup_breaks_by_skill.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &o.phases_fired {
            *self.phases_fired.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &o.fights_with {
            *self.fights_with.entry(k.clone()).or_default() += v;
        }
    }

    /// WORLD-SIM1: the fights with at least one of `key` (`fights_with`; 0 when none).
    pub fn fights_with(&self, key: &str) -> i64 {
        self.fights_with.get(key).copied().unwrap_or(0)
    }

    /// The median of the fights' rounds (the mean of the middle two for an even count; 0 without fights).
    pub fn median_rounds(&self) -> f64 {
        median(&self.rounds)
    }

    /// RULESET1-G2: the median rounds of the won fights (0 without one).
    pub fn median_won_rounds(&self) -> f64 {
        median(&self.won_rounds)
    }

    fn total_breaks(&self) -> i64 {
        self.breaks.values().sum()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "fights": self.fights, "won_pct": round1(pct(self.won, self.fights)), "median_rounds": self.median_rounds(),
            "won": self.won, "lost": self.lost, "capped": self.capped, "fled": self.fled, "median_rounds_won": self.median_won_rounds(),
            "breaks": self.total_breaks(), "breaks_by_cause": counts_json(&self.breaks), "windup_hits": self.windup_hits, "windup_breaks": self.windup_breaks,
            "chance_rolls": self.chance_rolls, "chance_breaks": self.chance_breaks, "sure_breaks": self.sure_breaks,
            "lock_rate_pct": round1(pct(self.locked, self.bar_rounds)),
            "hits": self.hits, "flank_hits": self.flank_hits, "flank_pct": round1(pct(self.flank_hits, self.hits)),
            "pierce_hits": self.pierce_hits, "pierce_pct": round1(pct(self.pierce_hits, self.hits)), "guarded_hits": self.guarded_hits,
            "lost_attacks": self.lost_attacks, "kite_steps": self.kite_steps, "kite_ap": self.kite_ap, "reasons": counts_json(&self.reasons), "timed_rounds": self.timed_rounds,
            "party": self.party.to_json(), "enemy": self.enemy.to_json(),
        })
    }

    /// SIMCOUNT1: `to_json`, with each side's rule counts under `rule_counts` when `counts=on`.
    pub fn report_json(&self, rules: bool) -> Value {
        let mut v = self.to_json();
        if rules {
            let rounds: i64 = self.rounds.iter().map(|r| *r as i64).sum();
            v["party"]["rule_counts"] = self.party.rule_json(self.fights, rounds);
            v["enemy"]["rule_counts"] = self.enemy.rule_json(self.fights, rounds);
            v["hooks"] = json!({
                "arena_strikes": self.arena_strikes, "windup_breaks_by_skill": counts_json(&self.windup_breaks_by_skill),
                "phases_fired": counts_json(&self.phases_fired), "fights_with": counts_json(&self.fights_with),
            });
        }
        v
    }

    /// SIMCOUNT1: `lines`, with each side's rule counts when `counts=on`.
    pub fn report_lines(&self, title: &str, rules: bool) -> Vec<String> {
        let mut out = self.lines(title);
        if rules {
            let rounds: i64 = self.rounds.iter().map(|r| *r as i64).sum();
            out.push(self.party.rule_line("party", self.fights, rounds));
            out.push(self.enemy.rule_line("enemy", self.fights, rounds));
            out.push(self.party.hook_line("party"));
            out.push(self.enemy.hook_line("enemy"));
            let tally = |m: &IndexMap<String, i64>| if m.is_empty() { "-".to_string() } else { m.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(" ") };
            out.push(format!(
                "  hooks: arena strikes {}; windup breaks by skill {}; phases fired (fights) {}; fights with {}",
                self.arena_strikes,
                tally(&self.windup_breaks_by_skill),
                tally(&self.phases_fired),
                tally(&self.fights_with)
            ));
        }
        out
    }

    fn lines(&self, title: &str) -> Vec<String> {
        let causes: Vec<String> = self.breaks.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let reasons: Vec<String> = self.reasons.iter().map(|(k, v)| format!("{k} {v}")).collect();
        vec![
            format!(
                "FEEL {title}: {} fights, won {}% (won {}, lost {}, capped {}, fled {}), median rounds {} (won {})",
                self.fights,
                round1(pct(self.won, self.fights)),
                self.won,
                self.lost,
                self.capped,
                self.fled,
                self.median_rounds(),
                self.median_won_rounds()
            ),
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
            format!("  planner reasons: {}; timed rounds {}", if reasons.is_empty() { "-".to_string() } else { reasons.join(", ") }, self.timed_rounds),
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
    /// WORLD-SIM1: the skills that broke a unit in its windup this fight.
    fight_windup: IndexMap<String, i64>,
}

/// A planner reason tag without its numbers ("rest: AP 0 < 2" -> "rest: AP", "follow-up 1" -> "follow-up"), so the
/// tally counts kinds.
fn reason_kind(tag: &str) -> String {
    tag.split(|ch: char| ch.is_ascii_digit()).next().unwrap_or("").trim().to_string()
}

/// The median of `rounds` (the mean of the middle two for an even count; 0 when empty).
fn median(rounds: &[i32]) -> f64 {
    let mut r = rounds.to_vec();
    r.sort_unstable();
    match r.len() {
        0 => 0.0,
        n if n % 2 == 1 => r[n / 2] as f64,
        n => (r[n / 2 - 1] + r[n / 2]) as f64 / 2.0,
    }
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
        self.fight_windup.clear();
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
            let no_active = u.deck_rule.is_some() && intent.is_some_and(|i| !u.active_families().iter().any(|f| rip::family_skill(w, c, f).is_some_and(|sid| round_intent::skill_rows(w, &w.cat.skills[sid]).contains(&i.id()))));
            self.ap_planned.insert(c, u.ap);
            let pays = w.pays_ap(c);
            let side = self.side(s, c);
            side.unit_rounds += 1;
            side.unpaid_rounds += (!pays) as i64;
            *side.intents.entry(id.clone()).or_default() += 1;
            side.intent_changes += changed as i64;
            side.no_active += no_active as i64;
            *side.intents_by_unit.entry(u.name.clone()).or_default().entry(id.clone()).or_default() += 1;
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
        // SIMCOUNT1: every action, the cancelled ones too: a command cancelled in its wind-up has already moved
        for a in &s.resolver.scheduled {
            let side = self.side(s, a.unit);
            for (key, _, count) in RULE_COUNTERS {
                let n = count(w, a);
                if n != 0 {
                    *side.rule_counts.entry(key).or_default() += n;
                }
            }
            if let CmdKind::Item { used: true, item, .. } = &a.command.kind {
                *side.items_by_id.entry(item.id.clone()).or_default() += 1;
            }
        }
        // S2.2-S3 (FB-85): the guarded blocks as the resolver saw them (Guard Option 1C holds past the Defend's span)
        for &(c, _) in &s.resolver.guarded {
            self.side(s, c).guard_blocks += 1;
        }
        let pierce_on = w.ruleset.has(ruleset::PIERCE);
        let mut firsts: IndexMap<UnitId, i32> = IndexMap::new();
        let mut actions: Vec<_> = s.resolver.scheduled.iter().filter(|a| a.landed).collect();
        actions.sort_by_key(|a| (a.unit, a.start));
        for a in actions {
            let cmd = &a.command;
            let skill_like = matches!(cmd.kind, CmdKind::Attack { .. } | CmdKind::Area { .. } | CmdKind::Support { .. } | CmdKind::Focus { .. } | CmdKind::Defend { .. } | CmdKind::Item { .. });
            let follow = skill_like && *firsts.entry(a.unit).and_modify(|n| *n += 1).or_insert(0) > 0;
            let carried = !matches!(cmd.carry, Carry::None);
            let side = self.side(s, a.unit);
            side.follow_ups += follow as i64;
            side.carried_moves += carried as i64;
            if skill_like && !cmd.skill.is_empty() {
                *side.skills_landed.entry(cmd.skill.clone()).or_default() += 1;
            }
            let c = &mut self.counts;
            // RULESET1_INTENT GD5 only: an ATTACKLOCK1 dodge raises `evaded` too and has its own row
            c.lost_attacks += (cmd.evaded.is_some() && cmd.dodged.is_none()) as i64;
            c.hits += cmd.hits.len() as i64;
            c.flank_hits += cmd.hits.iter().filter(|h| h.1).count() as i64;
            if pierce_on && cmd.impact.armor_piercing {
                c.pierce_hits += cmd.hits.len() as i64;
            }
            for st in &cmd.staggers {
                c.windup_hits += st.windup as i64;
                c.windup_breaks += (st.windup && st.broke.is_some()) as i64;
                if st.windup && st.broke.is_some() {
                    *c.windup_breaks_by_skill.entry(cmd.skill.clone()).or_default() += 1;
                    *self.fight_windup.entry(cmd.skill.clone()).or_default() += 1;
                }
                c.guarded_hits += (st.factors.guard < 1.0) as i64;
                if let Some((chance, roll)) = st.chance {
                    c.chance_rolls += 1;
                    c.chance_breaks += (roll <= chance) as i64;
                }
                c.sure_breaks += st.broke.as_ref().is_some_and(|b| b.cause == rpgkit_battle::poise::BreakCause::Sure) as i64;
            }
        }
        // PR5-26 (FB-191 (f)): each holder of the channel meter on the field: a Channel round, and the band its meter
        // ends the round in
        let m = rpgkit_battle::channel::meter_id(w).to_string();
        for c in active_units(s).into_iter().filter(|&c| !m.is_empty() && rpgkit_battle::meter::value(w, c, &m).is_some()) {
            let channel = rpgkit_battle::channel::is_channel(w, c);
            let band = rpgkit_battle::meter::shown(w, c).into_iter().find(|v| v.id == m).map(|v| v.band).filter(|b| !b.is_empty()).unwrap_or_else(|| "-".into());
            let side = self.side(s, c);
            side.channel_rounds += channel as i64;
            *side.channel_bands.entry(band).or_default() += 1;
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

    /// The fight is over: its outcome (the session's: won, lost, fled; none = capped), rounds and the log's breaks by
    /// cause.
    pub fn end(&mut self, s: &Session, rounds: i32) {
        let c = &mut self.counts;
        c.fights += 1;
        match s.outcome.as_str() {
            "won" => {
                c.won += 1;
                c.won_rounds.push(rounds);
            }
            "lost" => c.lost += 1,
            "fled" => c.fled += 1,
            _ => c.capped += 1,
        }
        c.rounds.push(rounds);
        // PK2-FOLLOWUP: each side's weapon wear this fight (counts=on)
        for (side, units) in [(&mut c.party, &s.w.allies), (&mut c.enemy, &s.w.enemies)] {
            let (worn, dulled) = rpgkit_battle::wear::tally(&s.w, units);
            side.worn += worn;
            side.dulled += i64::from(dulled);
        }
        // WORLD-SIM1 (PR5-42): the fight's hook counts (the metrics' HOOKS1 4.2 records, by the credited unit's side),
        // its phases fired, its windup breaks by skill, and which counts it had at least one of
        let mut had: IndexMap<String, i64> = IndexMap::new();
        if let Some(m) = s.metrics.as_deref() {
            c.arena_strikes += i64::from(m.arena_strikes);
            *had.entry("arena_strikes".into()).or_default() += i64::from(m.arena_strikes);
            for (id, u) in s.w.units.iter().enumerate() {
                let ally = s.w.allies.contains(&id);
                for (key, _, count) in HOOK_COUNTERS {
                    let n = count(m, &u.name);
                    if n != 0 {
                        let side = if ally { &mut c.party } else { &mut c.enemy };
                        *side.hooks.entry(key).or_default() += n;
                        *had.entry(key.to_string()).or_default() += n;
                    }
                }
            }
        }
        for id in s.phases.fired.keys() {
            *c.phases_fired.entry(id.clone()).or_default() += 1;
        }
        *had.entry("windup_breaks".into()).or_default() += self.fight_windup.values().sum::<i64>();
        for (skill, n) in &self.fight_windup {
            *had.entry(format!("{WINDUP_BREAK_BY}{skill}")).or_default() += n;
        }
        for (key, n) in had {
            *c.fights_with.entry(key).or_default() += (n > 0) as i64;
        }
        for e in s.log.iter().filter(|e| e.outcome == "BREAK") {
            *c.breaks.entry(e.status.clone()).or_default() += 1;
            // PR5-26 (FB-191 (f)): by the broken unit's side (the entry's actor)
            if e.actor_is_ally {
                c.party.breaks_taken += 1;
            } else {
                c.enemy.breaks_taken += 1;
            }
        }
        // PR5-26 (FB-191 (f)): the channel meter's own trigger rows fired this battle (its crisis), by the holder's side
        let m = rpgkit_battle::channel::meter_id(&s.w);
        if !m.is_empty() {
            let prefix = format!("{}{m}#", rpgkit_battle::meter::METER_KEY);
            for (unit, n) in s.resolver.triggers.battle_counts(&prefix) {
                if s.w.allies.contains(&unit) {
                    c.party.channel_triggers += i64::from(n);
                } else {
                    c.enemy.channel_triggers += i64::from(n);
                }
            }
            // PR5-26 follow-up (FB-201): the crises among them, by the holder's side
            if let Some(def) = s.w.cat.unit_meters.get(m) {
                for (n, row) in def.triggers.iter().enumerate() {
                    let Some(at) = crisis_at(row, m) else { continue };
                    let key = format!("{}{m}#{n}", rpgkit_battle::meter::METER_KEY);
                    for unit in 0..s.w.units.len() {
                        let max = rpgkit_battle::meter::pool(&s.w, unit, m).map_or(def.max, |p| p.max);
                        let fired = s.resolver.triggers.battle_count(unit, &key);
                        if fired == 0 || at < max {
                            continue;
                        }
                        if s.w.allies.contains(&unit) {
                            c.party.crises += i64::from(fired);
                        } else {
                            c.enemy.crises += i64::from(fired);
                        }
                    }
                }
            }
        }
    }
}

/// PR5-26 follow-up (FB-201): a crisis row of meter `m`: a `round_end` trigger row whose `when` holds `m` at least some
/// value (Some(that value); a crisis when it reaches the holder's max).
fn crisis_at(row: &rpgkit_data::TriggerRow, m: &str) -> Option<i32> {
    let cond = row.when.get("meter")?.as_object()?;
    (row.on == "round_end" && cond.get("id").and_then(Value::as_str) == Some(m)).then(|| cond.get("at_least").and_then(Value::as_i64).unwrap_or(0) as i32)
}

/// The `feel` mode: each of the arguments' encounters (catalog encounters with a party, or world encounters with an
/// authored party) fought over every seed with its
/// authored party, the encounter's round limit and board; per party driver a block of lines per encounter, the total,
/// then FEEL_JSON.
pub fn feel_mode(r: &mut Runner) {
    // RULESET1-G2: the run's armour (in memory), the party's tactic; `preset=` (classic only) drafts both sides
    for (id, armor) in r.args.armor.clone() {
        r.set_armor(&id, armor);
    }
    r.preset = r.args.preset.clone();
    r.party_tactic = (r.args.tactic != "balanced").then(|| r.args.tactic_preset().to_string());
    for driver in r.args.drivers.clone() {
        r.driver = driver;
        feel_block(r);
    }
    r.driver = crate::timed::Driver::Auto;
}

/// Encounter `id` (a catalog encounter, or WORLD-SIM1 a world reference: `Runner::fight_world`) fought once per seed
/// with the runner's driver and tactic: its counts. RULESET1-G2: the
/// encounter as authored (every field: objective, morale, timeout outcome, hazards...) but its round limit and seed,
/// as the host's `new` and `autoplay` and the CLI `battle` fight it (jitter, distance and the enemy block aside).
pub fn fight_encounter(r: &mut Runner, id: &str, seeds: &[i64]) -> Counts {
    // WORLD-SIM1 (PR5-42): a world encounter fights with its authored side as its world builds it (`Runner::fight_world`)
    if let Some((world, local)) = crate::args::parse_ref(id, &r.args.world) {
        let max_rounds = json::i(r.d.resolve(&world, "encounters", &local), "max_rounds", r.args.cap as i64) as i32;
        r.rounds_override = max_rounds;
        r.feel = Some(Feel::default());
        for &seed in seeds {
            r.fight_world(&world, &local, seed);
        }
        r.rounds_override = 0;
        return r.feel.take().map(|f| f.counts).unwrap_or_default();
    }
    let entry = r.d.entry("encounters", id).clone();
    let party = json::sget(&entry, "party");
    let enemies = json::sget(&entry, "enemies");
    let mut base = entry.clone();
    for key in ["party", "enemies", "max_rounds", "rng_seed"] {
        base.shift_remove(key);
    }
    r.rounds_override = json::i(&entry, "max_rounds", r.args.cap as i64) as i32;
    r.feel = Some(Feel::default());
    for &seed in seeds {
        r.fight(&party, &enemies, seed, &base);
    }
    r.rounds_override = 0;
    r.feel.take().map(|f| f.counts).unwrap_or_default()
}

/// One party driver's block of the feel mode.
fn feel_block(r: &mut Runner) {
    let rules = r.fight_ruleset();
    let seeds = r.args.seeds();
    let driver = r.driver.id();
    let rule_counts = r.args.counts;
    r.print(format!(
        "FEEL: {} rules [{}], {} seeds per encounter, party driver {driver}, tactic {}, preset {}",
        rules.name,
        rules.rules.join(", "),
        seeds.len(),
        r.args.tactic,
        r.preset
    ));
    let (fails_before, skips_before, fights_before) = (r.plan_fails, r.plan_skips, r.fights);
    let mut total = Counts::default();
    let mut per: Map<String, Value> = Map::new();
    for id in r.args.encounters.clone() {
        let counts = fight_encounter(r, &id, &seeds);
        let name = match crate::args::parse_ref(&id, &r.args.world) {
            Some((world, local)) => json::s_or(r.d.resolve(&world, "encounters", &local), "name", &local),
            None => json::s_or(r.d.entry("encounters", &id), "name", &id),
        };
        let lines = counts.report_lines(&format!("{id} ({name})"), rule_counts);
        r.out.extend(lines);
        per.insert(id.clone(), counts.report_json(rule_counts));
        total.add(&counts);
    }
    let lines = total.report_lines("TOTAL", rule_counts);
    r.out.extend(lines);
    let (fails, skips, fights) = (r.plan_fails - fails_before, r.plan_skips - skips_before, r.fights - fights_before);
    r.print(format!("plan checks: {fails} PLAN FAIL, {skips} PLAN SKIP over {fights} fights"));
    let armor: Map<String, Value> = r.args.armor.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
    let tune: Map<String, Value> = r.args.tune.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
    let out = json!({
        "ruleset": rules.id, "rules": rules.rules, "seeds": seeds.len(), "driver": driver, "tactic": r.args.tactic, "preset": r.preset,
        "armor": armor, "tune": tune, "plan_fails": fails, "plan_skips": skips, "encounters": per, "total": total.report_json(rule_counts),
    });
    r.print(format!("{JSON_PREFIX}{out}"));
}
