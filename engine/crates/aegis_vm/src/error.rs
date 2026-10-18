use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Variable not found: '{0}'")]
    VariableNotFound(String),

    #[error("Method not found: '{0}' on class '{1}'")]
    MethodNotFound(String, String),

    #[error("Property not found: '{0}' on instance")]
    PropertyNotFound(String),

    #[error("Signal not found: '{0}'")]
    SignalNotFound(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Runtime evaluation error: {0}")]
    EvalError(String),
}
