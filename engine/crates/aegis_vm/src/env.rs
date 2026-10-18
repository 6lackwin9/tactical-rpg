use aegis_core::Variant;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Scope {
    variables: HashMap<String, Variant>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn set(&mut self, name: &str, value: Variant) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Variant> {
        if let Some(val) = self.variables.get(name) {
            Some(val.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, value: Variant) -> bool {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            true
        } else if let Some(ref mut parent) = self.parent {
            parent.assign(name, value)
        } else {
            false
        }
    }
}
