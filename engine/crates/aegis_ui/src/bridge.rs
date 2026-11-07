//! Agent link (ENGINE_SPEC section 4.2): a localhost TCP port speaking the same JSON lines as `aegis serve`, plus
//! GUI-only requests (ui_tree, screenshot, click). Requests are handed to the GUI loop, so the agent acts on the very
//! battle the user is looking at.

use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};

pub const DEFAULT_PORT: u16 = 47800;

pub struct Request {
    pub body: Value,
    pub reply: Sender<Value>,
}

/// Starts the listener thread; requests arrive on the returned receiver, and `wake` runs after each (a loop asleep in a
/// window's event queue needs it; a loop that waits on the receiver passes nothing to do).
pub fn start(port: u16, wake: impl Fn() + Clone + Send + 'static) -> Option<Receiver<Request>> {
    let listener = TcpListener::bind(("127.0.0.1", port)).ok()?;
    let (tx, rx) = channel::<Request>();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let tx = tx.clone();
            let wake = wake.clone();
            std::thread::spawn(move || {
                // A reply is written in pieces: without this each one waits on the other side's delayed ack, about 0.3 s.
                let _ = stream.set_nodelay(true);
                let mut writer = match stream.try_clone() {
                    Ok(w) => w,
                    Err(_) => return,
                };
                for line in BufReader::new(stream).lines() {
                    let Ok(line) = line else { break };
                    if line.trim().is_empty() {
                        continue;
                    }
                    let body = serde_json::from_str::<Value>(&line).unwrap_or_else(|e| serde_json::json!({"cmd": "_bad", "error": e.to_string()}));
                    let (rtx, rrx) = channel();
                    if tx.send(Request { body, reply: rtx }).is_err() {
                        return;
                    }
                    wake();
                    let reply = rrx.recv().unwrap_or(Value::Null);
                    if writeln!(writer, "{reply}").is_err() {
                        break;
                    }
                }
            });
        }
    });
    Some(rx)
}
