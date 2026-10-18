use crate::variant::Variant;
use std::sync::{Arc, Mutex};

pub type SignalHandler = Arc<dyn Fn(&[Variant]) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SignalConnection {
    pub id: u64,
    handler: SignalHandler,
}

/// A pub/sub signal emitter, equivalent to Godot's `signal` system.
#[derive(Default, Clone)]
pub struct Signal {
    next_id: Arc<Mutex<u64>>,
    connections: Arc<Mutex<Vec<SignalConnection>>>,
}

impl Signal {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(Mutex::new(1)),
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Connects a callback closure to this signal. Returns a connection ID.
    pub fn connect<F>(&self, handler: F) -> u64
    where
        F: Fn(&[Variant]) + Send + Sync + 'static,
    {
        let mut id_guard = self.next_id.lock().unwrap();
        let id = *id_guard;
        *id_guard += 1;

        let mut conns = self.connections.lock().unwrap();
        conns.push(SignalConnection {
            id,
            handler: Arc::new(handler),
        });
        id
    }

    /// Disconnects a callback using its connection ID.
    pub fn disconnect(&self, id: u64) -> bool {
        let mut conns = self.connections.lock().unwrap();
        if let Some(pos) = conns.iter().position(|c| c.id == id) {
            conns.remove(pos);
            true
        } else {
            false
        }
    }

    /// Emits the signal with the given arguments to all connected handlers.
    pub fn emit(&self, args: &[Variant]) {
        let handlers: Vec<SignalHandler> = {
            let conns = self.connections.lock().unwrap();
            conns.iter().map(|c| c.handler.clone()).collect()
        };

        for handler in handlers {
            handler(args);
        }
    }

    /// Emits without arguments.
    pub fn emit_empty(&self) {
        self.emit(&[]);
    }

    /// Returns the number of active connections.
    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    #[test]
    fn test_signal_emit_and_disconnect() {
        let signal = Signal::new();
        let counter = Arc::new(AtomicI64::new(0));

        let c_clone = counter.clone();
        let id1 = signal.connect(move |args| {
            let val = args.first().map(|v| v.to_int()).unwrap_or(1);
            c_clone.fetch_add(val, Ordering::SeqCst);
        });

        signal.emit(&[Variant::Int(10)]);
        assert_eq!(counter.load(Ordering::SeqCst), 10);

        signal.emit(&[Variant::Int(5)]);
        assert_eq!(counter.load(Ordering::SeqCst), 15);

        assert!(signal.disconnect(id1));
        assert_eq!(signal.connection_count(), 0);

        signal.emit(&[Variant::Int(5)]);
        assert_eq!(counter.load(Ordering::SeqCst), 15); // Unchanged
    }
}
