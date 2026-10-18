use crate::ivec2::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Dynamic variant value representing any value in the Aegis runtime.
/// Equivalent to Godot's `Variant` type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Variant {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Vector2i(IVec2),
    Array(Vec<Variant>),
    Dictionary(HashMap<String, Variant>),
}

impl Default for Variant {
    fn default() -> Self {
        Variant::Nil
    }
}

impl Variant {
    pub fn is_nil(&self) -> bool {
        matches!(self, Variant::Nil)
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Variant::Nil => false,
            Variant::Bool(b) => *b,
            Variant::Int(i) => *i != 0,
            Variant::Float(f) => *f != 0.0 && !f.is_nan(),
            Variant::String(s) => !s.is_empty(),
            Variant::Vector2i(v) => *v != IVec2::ZERO,
            Variant::Array(a) => !a.is_empty(),
            Variant::Dictionary(d) => !d.is_empty(),
        }
    }

    pub fn to_int(&self) -> i64 {
        match self {
            Variant::Nil => 0,
            Variant::Bool(b) => if *b { 1 } else { 0 },
            Variant::Int(i) => *i,
            Variant::Float(f) => *f as i64,
            Variant::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => 0,
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            Variant::Nil => 0.0,
            Variant::Bool(b) => if *b { 1.0 } else { 0.0 },
            Variant::Int(i) => *i as f64,
            Variant::Float(f) => *f,
            Variant::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Variant::String(s) => s.as_str(),
            _ => "",
        }
    }

    pub fn as_vec2i(&self) -> Option<IVec2> {
        match self {
            Variant::Vector2i(v) => Some(*v),
            Variant::Array(a) if a.len() >= 2 => {
                Some(IVec2::new(a[0].to_int() as i32, a[1].to_int() as i32))
            }
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Variant>> {
        match self {
            Variant::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Variant>> {
        match self {
            Variant::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&HashMap<String, Variant>> {
        match self {
            Variant::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_dict_mut(&mut self) -> Option<&mut HashMap<String, Variant>> {
        match self {
            Variant::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    /// Access dictionary key or return Variant::Nil
    pub fn get_key(&self, key: &str) -> Variant {
        match self {
            Variant::Dictionary(d) => d.get(key).cloned().unwrap_or(Variant::Nil),
            _ => Variant::Nil,
        }
    }

    /// Access array index or return Variant::Nil
    pub fn get_index(&self, index: usize) -> Variant {
        match self {
            Variant::Array(a) => a.get(index).cloned().unwrap_or(Variant::Nil),
            _ => Variant::Nil,
        }
    }

    /// Converts serde_json::Value to Variant
    pub fn from_json(val: &serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => Variant::Nil,
            serde_json::Value::Bool(b) => Variant::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Variant::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Variant::Float(f)
                } else {
                    Variant::Nil
                }
            }
            serde_json::Value::String(s) => Variant::String(s.clone()),
            serde_json::Value::Array(arr) => {
                // Special case: [x, y] of ints can also be converted on demand to IVec2
                Variant::Array(arr.iter().map(Variant::from_json).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), Variant::from_json(v));
                }
                Variant::Dictionary(map)
            }
        }
    }

    /// Converts Variant to serde_json::Value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Variant::Nil => serde_json::Value::Null,
            Variant::Bool(b) => serde_json::Value::Bool(*b),
            Variant::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Variant::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Variant::String(s) => serde_json::Value::String(s.clone()),
            Variant::Vector2i(v) => serde_json::json!([v.x, v.y]),
            Variant::Array(arr) => serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect()),
            Variant::Dictionary(map) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in map {
                    obj.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(obj)
            }
        }
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variant::Nil => write!(f, "<null>"),
            Variant::Bool(b) => write!(f, "{}", b),
            Variant::Int(i) => write!(f, "{}", i),
            Variant::Float(fl) => write!(f, "{}", fl),
            Variant::String(s) => write!(f, "{}", s),
            Variant::Vector2i(v) => write!(f, "{}", v),
            Variant::Array(a) => write!(f, "{:?}", a),
            Variant::Dictionary(d) => write!(f, "{:?}", d),
        }
    }
}

impl From<bool> for Variant {
    fn from(v: bool) -> Self {
        Variant::Bool(v)
    }
}

impl From<i64> for Variant {
    fn from(v: i64) -> Self {
        Variant::Int(v)
    }
}

impl From<i32> for Variant {
    fn from(v: i32) -> Self {
        Variant::Int(v as i64)
    }
}

impl From<f64> for Variant {
    fn from(v: f64) -> Self {
        Variant::Float(v)
    }
}

impl From<&str> for Variant {
    fn from(v: &str) -> Self {
        Variant::String(v.to_string())
    }
}

impl From<String> for Variant {
    fn from(v: String) -> Self {
        Variant::String(v)
    }
}

impl From<IVec2> for Variant {
    fn from(v: IVec2) -> Self {
        Variant::Vector2i(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_conversions() {
        assert_eq!(Variant::Int(42).to_int(), 42);
        assert_eq!(Variant::Int(42).to_float(), 42.0);
        assert_eq!(Variant::Float(3.14).to_int(), 3);
        assert_eq!(Variant::String("100".into()).to_int(), 100);
        assert!(Variant::Int(1).to_bool());
        assert!(!Variant::Int(0).to_bool());
        assert!(!Variant::Nil.to_bool());
    }

    #[test]
    fn test_json_roundtrip() {
        let json_str = r#"{"name": "Warrior", "hp": 100, "pos": [5, 4]}"#;
        let json_val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let var = Variant::from_json(&json_val);

        assert_eq!(var.get_key("name").as_str(), "Warrior");
        assert_eq!(var.get_key("hp").to_int(), 100);
        assert_eq!(var.get_key("pos").as_vec2i(), Some(IVec2::new(5, 4)));

        let serialized = var.to_json();
        assert_eq!(serialized["name"], "Warrior");
        assert_eq!(serialized["hp"], 100);
    }
}
