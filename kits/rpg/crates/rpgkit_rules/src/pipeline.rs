//! The damage pipeline (U1.2 slice 1, `FB-05`): fixed stages in a fixed order, base -> flat -> mult -> mitigation ->
//! round, and modifiers as data the stages read. A game supplies modifiers; it cannot reorder the stages.
//!
//! Each stage applies its modifiers in the order they are given, so a caller that lists today's factors in today's
//! order multiplies in today's f64 order (multiplication is not associative; nothing is pre-merged). Two rounding
//! points: an optional one at the end of the base stage (the attack power, minimum 1 for damage) and the finish at the
//! round stage (`roundi`, then the floor). Every applied modifier and both roundings leave a step in the trace.
//!
//! Where each op may sit: `Add` in base and flat (flat adds stay before every multiplier); `Mul` in base (the tier power
//! scale), mult and mitigation; `Reduce` (value x (1 - ratio)) in mitigation. The round stage takes no modifiers. A run
//! panics, naming the modifier, on either mistake (only the kit builds modifiers today; data modifiers will be checked
//! when they load).

use std::borrow::Cow;

use crate::stats::roundi;

/// The stages, in the order they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// The starting value: base terms added, then base multipliers (tier power); may round (the attack power).
    Base,
    /// Flat adds after the base (none in the default pipeline).
    Flat,
    /// Amplifiers: Focus, skill, stance, crit.
    Mult,
    /// Mitigation layers: guard and cover, armour.
    Mitigation,
    /// The finish: `roundi`, then the floor.
    Round,
}

impl Stage {
    pub const ORDER: [Stage; 5] = [Stage::Base, Stage::Flat, Stage::Mult, Stage::Mitigation, Stage::Round];

    pub fn name(self) -> &'static str {
        match self {
            Stage::Base => "base",
            Stage::Flat => "flat",
            Stage::Mult => "mult",
            Stage::Mitigation => "mitigation",
            Stage::Round => "round",
        }
    }
}

/// What a modifier does to the running value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    /// value + x
    Add(f64),
    /// value x x
    Mul(f64),
    /// value x (1 - x): a mitigation ratio
    Reduce(f64),
}

impl Op {
    /// Whether the op may sit in `stage` (module docs).
    pub fn fits(self, stage: Stage) -> bool {
        match self {
            Op::Add(_) => matches!(stage, Stage::Base | Stage::Flat),
            Op::Mul(_) => matches!(stage, Stage::Base | Stage::Mult | Stage::Mitigation),
            Op::Reduce(_) => stage == Stage::Mitigation,
        }
    }

    fn apply(self, v: f64) -> f64 {
        match self {
            Op::Add(x) => v + x,
            Op::Mul(x) => v * x,
            Op::Reduce(x) => v * (1.0 - x),
        }
    }
}

/// One modifier: its id (for the trace), its stage and its op.
#[derive(Debug, Clone, PartialEq)]
pub struct Modifier {
    pub id: Cow<'static, str>,
    pub stage: Stage,
    pub op: Op,
}

impl Modifier {
    pub fn new(id: &'static str, stage: Stage, op: Op) -> Self {
        Self { id: Cow::Borrowed(id), stage, op }
    }
}

/// A trace step: a modifier applied, or a rounding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepKind {
    Apply(Op),
    /// `roundi`, then at least `min`.
    Round { min: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub stage: Stage,
    /// The modifier's id; `round` for a rounding.
    pub id: Cow<'static, str>,
    pub kind: StepKind,
    pub before: f64,
    pub after: f64,
}

/// Every step of one run, in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trace {
    pub steps: Vec<Step>,
}

impl Trace {
    /// The first step of modifier `id`.
    pub fn step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }

    /// The steps of one stage.
    pub fn stage(&self, stage: Stage) -> impl Iterator<Item = &Step> {
        self.steps.iter().filter(move |s| s.stage == stage)
    }
}

/// A run's result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outcome {
    /// The base stage's rounded value (None when the pipeline does not round there).
    pub base: Option<i32>,
    /// The value before the finish.
    pub gross: f64,
    /// The finished value.
    pub value: i32,
}

/// A pipeline's two rounding points (the stages themselves are fixed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pipeline {
    /// Round at the end of the base stage, to at least this (None: no rounding there).
    pub base_round: Option<i32>,
    /// The finish's floor.
    pub finish_min: i32,
}

/// U1.2-S2 (FB-45): the attack power's floor at rounding point one: an authored or derived power below it counts as it
/// before the multipliers (an authored unit `power` of 0 still means "derive it").
pub const MIN_POWER: i32 = 1;
/// Damage: the attack power rounds (minimum `MIN_POWER`), the hit rounds (minimum 1).
pub const DAMAGE: Pipeline = Pipeline { base_round: Some(MIN_POWER), finish_min: 1 };
/// Poise stagger (RULESET1 P6): one rounding, floor 0.
pub const STAGGER: Pipeline = Pipeline { base_round: None, finish_min: 0 };

impl Pipeline {
    /// Runs `mods` (the value starts at 0).
    pub fn run(&self, mods: &[Modifier]) -> Outcome {
        self.exec(mods, None)
    }

    /// `run`, keeping every step.
    pub fn run_traced(&self, mods: &[Modifier]) -> (Outcome, Trace) {
        let mut trace = Trace { steps: Vec::with_capacity(mods.len() + 2) };
        let out = self.exec(mods, Some(&mut trace));
        (out, trace)
    }

    fn exec(&self, mods: &[Modifier], trace: Option<&mut Trace>) -> Outcome {
        // hard checks (only the kit builds modifiers today): a modifier in the round stage, or an op outside its stages
        let mut sorted = true;
        let mut prev = Stage::Base;
        for m in mods {
            assert!(m.stage != Stage::Round, "modifier {}: the round stage takes no modifiers", m.id);
            assert!(m.op.fits(m.stage), "modifier {}: {:?} cannot sit in stage {}", m.id, m.op, m.stage.name());
            sorted &= prev <= m.stage;
            prev = m.stage;
        }
        let mut run = Run { pipe: *self, v: 0.0, base: None, past_base: false, trace };
        // already in stage order (the kit's lists): one pass; else stage by stage, each in list order (the same result)
        if sorted {
            for m in mods {
                run.apply(m);
            }
        } else {
            for stage in Stage::ORDER {
                for m in mods {
                    if m.stage == stage {
                        run.apply(m);
                    }
                }
            }
        }
        run.finish()
    }
}

/// One run's running value.
struct Run<'t> {
    pipe: Pipeline,
    v: f64,
    base: Option<i32>,
    past_base: bool,
    trace: Option<&'t mut Trace>,
}

impl Run<'_> {
    fn apply(&mut self, m: &Modifier) {
        if m.stage != Stage::Base {
            self.end_base();
        }
        let after = m.op.apply(self.v);
        if let Some(t) = self.trace.as_deref_mut() {
            t.steps.push(Step { stage: m.stage, id: m.id.clone(), kind: StepKind::Apply(m.op), before: self.v, after });
        }
        self.v = after;
    }

    /// Rounding point one, once, when the base stage ends.
    fn end_base(&mut self) {
        if self.past_base {
            return;
        }
        self.past_base = true;
        if let Some(min) = self.pipe.base_round {
            let r = self.round(Stage::Base, min);
            self.base = Some(r);
            self.v = r as f64;
        }
    }

    fn round(&mut self, stage: Stage, min: i32) -> i32 {
        let r = roundi(self.v).max(min);
        if let Some(t) = self.trace.as_deref_mut() {
            t.steps.push(Step { stage, id: Cow::Borrowed("round"), kind: StepKind::Round { min }, before: self.v, after: r as f64 });
        }
        r
    }

    fn finish(mut self) -> Outcome {
        self.end_base();
        let gross = self.v;
        let value = self.round(Stage::Round, self.pipe.finish_min);
        Outcome { base: self.base, gross, value }
    }
}
