//! The command API protocol (ENGINE_SPEC section 4.2), game-agnostic: one JSON request in, one JSON reply out (always
//! an object with `ok`, plus the `step` counter). Every state change goes through `Host::handle`, which records the
//! successful mutating requests, so a log of the requests (`history`) replays the game's state exactly. The game
//! supplies the command set (`Commands`); the host reaches it through Deref.
//!
//! Requests the host answers itself (`cmd`):
//!   history                                    the mutating requests so far (feed them back to replay)

use serde_json::{json, Value};
use std::ops::{Deref, DerefMut};

/// A game's command set.
pub trait Commands {
    /// Requests that change state (recorded in the history when they succeed).
    fn mutating(&self, cmd: &str) -> bool;
    /// Requests that start over (the history restarts with them).
    fn restarts(&self, cmd: &str) -> bool;
    /// The reply to one request (an object with `ok`).
    fn handle(&mut self, cmd: &str, req: &Value) -> Value;
}

pub struct Host<C> {
    pub commands: C,
    pub history: Vec<Value>,
    pub steps: u64,
}

impl<C> Deref for Host<C> {
    type Target = C;

    fn deref(&self) -> &C {
        &self.commands
    }
}

impl<C> DerefMut for Host<C> {
    fn deref_mut(&mut self) -> &mut C {
        &mut self.commands
    }
}

impl<C: Commands> Host<C> {
    pub fn new(commands: C) -> Self {
        Self { commands, history: Vec::new(), steps: 0 }
    }

    /// Handles one request and returns the reply (always an object with `ok`).
    pub fn handle(&mut self, req: &Value) -> Value {
        let cmd = req.get("cmd").and_then(Value::as_str).unwrap_or("");
        let mutating = self.commands.mutating(cmd);
        let mut reply = match cmd {
            "history" => json!({"ok": true, "history": self.history}),
            _ => self.commands.handle(cmd, req),
        };
        if mutating && reply["ok"] == json!(true) {
            if self.commands.restarts(cmd) {
                self.history.clear();
            }
            self.history.push(req.clone());
            self.steps += 1;
        }
        if let Some(o) = reply.as_object_mut() {
            o.insert("step".into(), json!(self.steps));
        }
        reply
    }
}
