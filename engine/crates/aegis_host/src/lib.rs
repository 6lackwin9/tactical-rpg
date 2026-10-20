//! The command API protocol (ENGINE_SPEC section 4.2), game-agnostic: one JSON request in, one JSON reply out (always
//! an object with `ok`, plus the `step` counter). Every state change goes through `Host::handle`, which records the
//! successful mutating requests, so a log of the requests (`history`) replays the game's state exactly. The game
//! supplies the command set (`Commands`); the host reaches it through Deref. After a mutating request the game may
//! hand back a checkpoint (its state at a boundary it chooses, such as a round's end): what a replay compares
//! (`playtest`: recordings, replay, variants, seed sweeps).
//!
//! Requests the host answers itself (`cmd`):
//!   history                                    the mutating requests so far (feed them back to replay) and the
//!                                              checkpoints taken after them

use serde_json::{json, Value};
use std::ops::{Deref, DerefMut};

pub mod playtest;

/// A game's command set.
pub trait Commands {
    /// Requests that change state (recorded in the history when they succeed).
    fn mutating(&self, cmd: &str) -> bool;
    /// Requests that start over (the history restarts with them).
    fn restarts(&self, cmd: &str) -> bool;
    /// The reply to one request (an object with `ok`).
    fn handle(&mut self, cmd: &str, req: &Value) -> Value;
    /// After a successful mutating request: the game's state when it has just reached a boundary worth comparing
    /// (a round's end), else None. A checkpoint may carry a `result` string once the game is over (seed sweeps count
    /// those).
    fn checkpoint(&mut self) -> Option<Value> {
        None
    }
}

pub struct Host<C> {
    pub commands: C,
    pub history: Vec<Value>,
    /// {"at": the number of history requests it follows, "state": the game's checkpoint}, in order.
    pub checkpoints: Vec<Value>,
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
        Self { commands, history: Vec::new(), checkpoints: Vec::new(), steps: 0 }
    }

    /// Forgets the history and checkpoints (a game that starts a session outside `handle` calls it).
    pub fn restart(&mut self) {
        self.history.clear();
        self.checkpoints.clear();
    }

    /// Handles one request and returns the reply (always an object with `ok`).
    pub fn handle(&mut self, req: &Value) -> Value {
        let cmd = req.get("cmd").and_then(Value::as_str).unwrap_or("");
        let mutating = self.commands.mutating(cmd);
        let mut reply = match cmd {
            "history" => json!({"ok": true, "history": self.history, "checkpoints": self.checkpoints}),
            _ => self.commands.handle(cmd, req),
        };
        if mutating && reply["ok"] == json!(true) {
            if self.commands.restarts(cmd) {
                self.restart();
            }
            self.history.push(req.clone());
            self.steps += 1;
            if let Some(state) = self.commands.checkpoint() {
                self.checkpoints.push(json!({"at": self.history.len(), "state": state}));
            }
        }
        if let Some(o) = reply.as_object_mut() {
            o.insert("step".into(), json!(self.steps));
        }
        reply
    }
}

/// A command host as a replay sees it (object safe, so a shell can hold any game's).
pub trait Driver {
    fn send(&mut self, req: &Value) -> Value;
    fn checkpoints(&self) -> &[Value];
    fn history(&self) -> &[Value];
}

impl<C: Commands> Driver for Host<C> {
    fn send(&mut self, req: &Value) -> Value {
        self.handle(req)
    }

    fn checkpoints(&self) -> &[Value] {
        &self.checkpoints
    }

    fn history(&self) -> &[Value] {
        &self.history
    }
}
