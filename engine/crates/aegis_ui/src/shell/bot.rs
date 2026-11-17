//! UIBOT (REPLAY-CI1): a seeded UI bot. The same shell as a window or a headless run (`Shell::open`, `Shell::frame`, the
//! agent link's `click` and `key`), in the calling process with no window, no port and no sound device. Each step draws one
//! frame, then presses one valid widget of the top screen (enabled and clickable, not on the caller's skip list) or, now
//! and then or when there is none, a key. A panic anywhere in a step (a screen's tick, draw, click or key) ends the run
//! and is reported with its step, what was pressed and the panic's place and message; nothing is retried.
//!
//! The run is seeded (`Bot::seed`): the same seed picks the same widget index from the same list. Screens read wall time,
//! so a long run may not walk the same path twice; the report lists every press, which is the repro.
//!
//! The game supplies the start (`Cx`, its screens, `Options`), and what the bot must never press (`Bot::skip`: widgets
//! that write outside a scratch folder, open an OS window or end the program). Native only: it shares the headless run's
//! pieces.

use super::Shell;
use crate::screen::{Context, Cx, Screen};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

/// The keys pressed by default (agent link names, `Key::parse`).
pub const KEYS: [&str; 8] = ["esc", "enter", "space", "tab", "up", "down", "left", "right"];
/// One step in this many presses a key even when a widget is there to press.
pub const KEY_EVERY: u64 = 8;
/// The canvas size of a bot run unless the caller names one.
pub const SIZE: (u32, u32) = (1280, 720);

/// One bot run's settings.
#[derive(Clone, Debug)]
pub struct Bot {
    pub seed: u64,
    pub steps: usize,
    /// The canvas, in px.
    pub size: (u32, u32),
    /// Widget ids never pressed: an entry skips the id itself and every id that starts with it.
    pub skip: Vec<String>,
    /// The keys it may press (agent link names).
    pub keys: Vec<String>,
}

impl Bot {
    pub fn new(seed: u64, steps: usize) -> Self {
        Self { seed, steps, size: SIZE, skip: Vec::new(), keys: KEYS.iter().map(|k| k.to_string()).collect() }
    }

    /// True when `id` is on the skip list.
    pub fn skips(&self, id: &str) -> bool {
        self.skip.iter().any(|s| id.starts_with(s.as_str()))
    }
}

/// Where and why a step panicked.
#[derive(Clone, Debug, PartialEq)]
pub struct Panic {
    pub step: usize,
    /// The press of that step ("click <id>", "key <name>"), or "frame" when the frame before it panicked.
    pub action: String,
    /// The top screen when the step started.
    pub screen: String,
    /// "file:line:column".
    pub location: String,
    pub message: String,
}

/// What a run did.
#[derive(Clone, Debug, Default)]
pub struct Run {
    pub seed: u64,
    /// Steps done (a panic's step included).
    pub steps: usize,
    /// Every press in order, "click <id>" or "key <name>".
    pub presses: Vec<String>,
    /// The screens the top of the stack showed, first visit order.
    pub screens: Vec<String>,
    /// The program asked to quit before the last step.
    pub quit: bool,
    pub panic: Option<Panic>,
}

impl Run {
    /// One line: the seed, steps, screens seen and the end.
    pub fn summary(&self) -> String {
        let end = match &self.panic {
            Some(p) => format!("PANIC at step {} on {} after {}: {} ({})", p.step, p.screen, p.action, p.message, p.location),
            None if self.quit => "quit".to_string(),
            None => "ok".to_string(),
        };
        format!("seed {} steps {} screens [{}]: {end}", self.seed, self.steps, self.screens.join(", "))
    }
}

/// splitmix64: a small seeded stream, the same on every platform.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

thread_local! {
    /// While a bot step runs on this thread: the last panic's place and message (the hook writes it).
    static CAUGHT: RefCell<Option<Option<(String, String)>>> = const { RefCell::new(None) };
}
static HOOK: Once = Once::new();

/// Installs, once, a panic hook that records a bot step's panic quietly and leaves every other panic to the hook before it.
fn install_hook() {
    HOOK.call_once(|| {
        let before = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let mine = CAUGHT.with(|c| {
                let mut c = c.borrow_mut();
                match c.as_mut() {
                    Some(slot) => {
                        let at = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_default();
                        let p = info.payload();
                        let msg = p.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| p.downcast_ref::<String>().cloned()).unwrap_or_else(|| "(no message)".into());
                        *slot = Some((at, msg));
                        true
                    }
                    None => false,
                }
            });
            if !mine {
                before(info);
            }
        }));
    });
}

/// Runs `f`, catching a panic: Err((location, message)).
fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, (String, String)> {
    install_hook();
    CAUGHT.with(|c| *c.borrow_mut() = Some(None));
    let out = panic::catch_unwind(AssertUnwindSafe(f));
    let caught = CAUGHT.with(|c| c.borrow_mut().take().flatten());
    out.map_err(|_| caught.unwrap_or_else(|| (String::new(), "(no message)".into())))
}

/// The pressable widgets of the top screen: enabled, clickable, not skipped (agent infos, in draw order).
fn pressable(widgets: &[Value], bot: &Bot) -> Vec<String> {
    widgets.iter().filter(|w| w["enabled"] == json!(true) && w["clickable"] == json!(true)).filter_map(|w| w["id"].as_str()).filter(|id| !id.is_empty() && !bot.skips(id)).map(String::from).collect()
}

/// Runs the bot on the game's start: `bot.steps` steps, or until the program quits or a step panics; then every screen
/// leaves (as the program closing). A panic while leaving is reported too, as the step after the last.
pub fn run<G: Context>(cx: Cx<G>, stack: Vec<Box<dyn Screen<G>>>, options: &super::Options, bot: &Bot) -> Run {
    let mut out = Run { seed: bot.seed, ..Run::default() };
    let mut shell = match guarded(|| Shell::open(cx, stack, options, bot.size, 1.0, None)) {
        Ok(s) => s,
        Err((location, message)) => {
            out.panic = Some(Panic { step: 0, action: "open".into(), screen: String::new(), location, message });
            return out;
        }
    };
    let mut rng = Rng(bot.seed);
    for step in 0..bot.steps {
        out.steps = step + 1;
        let screen = shell.stack.last().map(|s| s.id().to_string()).unwrap_or_default();
        if !out.screens.contains(&screen) {
            out.screens.push(screen.clone());
        }
        if let Err((location, message)) = guarded(|| shell.frame()) {
            out.panic = Some(Panic { step, action: "frame".into(), screen, location, message });
            let _ = guarded(move || drop(shell));
            return out;
        }
        if shell.cx.quit {
            out.quit = true;
            break;
        }
        let ids = pressable(&shell.stack.last().map(|s| s.widgets()).unwrap_or_default(), bot);
        let roll = rng.next();
        let (action, body) = if ids.is_empty() || roll % KEY_EVERY == 0 {
            let k = bot.keys.get(rng.below(bot.keys.len())).cloned().unwrap_or_else(|| "esc".into());
            (format!("key {k}"), json!({"cmd": "key", "key": k}))
        } else {
            let id = ids[rng.below(ids.len())].clone();
            (format!("click {id}"), json!({"cmd": "click", "id": id}))
        };
        out.presses.push(action.clone());
        if let Err((location, message)) = guarded(|| shell.bridge(&body)) {
            out.panic = Some(Panic { step, action, screen, location, message });
            let _ = guarded(move || drop(shell));
            return out;
        }
        if shell.cx.quit {
            out.quit = true;
            break;
        }
    }
    let step = out.steps;
    if let Err((location, message)) = guarded(|| shell.leave_all()) {
        out.panic = Some(Panic { step, action: "leave".into(), screen: shell.stack.last().map(|s| s.id().to_string()).unwrap_or_default(), location, message });
    }
    let _ = guarded(move || drop(shell));
    out
}
