use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: &'static str,
        found: String,
    },

    #[error("Out of bounds: index {index} (length {len})")]
    IndexOutOfBounds { index: usize, len: usize },

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Evaluation error: {0}")]
    EvalError(String),
}
