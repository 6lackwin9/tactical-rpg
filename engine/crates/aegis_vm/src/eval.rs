use crate::env::Scope;
use crate::error::VmError;
use aegis_core::Variant;
use aegis_md_parser::ast::*;

pub struct Evaluator;

impl Evaluator {
    /// Evaluates an AST expression in the given scope.
    pub fn eval_expr(expr: &Expression, scope: &mut Scope) -> Result<Variant, VmError> {
        match expr {
            Expression::Literal(lit) => Ok(lit.clone()),
            Expression::Identifier(name) => {
                // Built-in constants
                match name.as_str() {
                    "true" => Ok(Variant::Bool(true)),
                    "false" => Ok(Variant::Bool(false)),
                    "null" | "nil" => Ok(Variant::Nil),
                    _ => scope
                        .get(name)
                        .ok_or_else(|| VmError::VariableNotFound(name.clone())),
                }
            }
            Expression::Unary { op, expr } => {
                let val = Self::eval_expr(expr, scope)?;
                match op {
                    UnaryOp::Neg => match val {
                        Variant::Int(i) => Ok(Variant::Int(-i)),
                        Variant::Float(f) => Ok(Variant::Float(-f)),
                        _ => Err(VmError::TypeError("Cannot negate non-number".into())),
                    },
                    UnaryOp::Not => Ok(Variant::Bool(!val.to_bool())),
                }
            }
            Expression::Binary { left, op, right } => {
                let l_val = Self::eval_expr(left, scope)?;
                let r_val = Self::eval_expr(right, scope)?;
                Self::eval_binary_op(&l_val, *op, &r_val)
            }
            Expression::Call { callee, args } => {
                if let Expression::Identifier(func_name) = callee.as_ref() {
                    let mut evaluated_args = Vec::new();
                    for a in args {
                        evaluated_args.push(Self::eval_expr(a, scope)?);
                    }
                    Self::eval_builtin_func(func_name, &evaluated_args)
                } else {
                    Err(VmError::EvalError("Indirect calls not yet supported".into()))
                }
            }
            Expression::MemberAccess { object, member } => {
                let obj = Self::eval_expr(object, scope)?;
                if let Variant::Dictionary(ref map) = obj {
                    Ok(map.get(member).cloned().unwrap_or(Variant::Nil))
                } else {
                    Err(VmError::PropertyNotFound(member.clone()))
                }
            }
            Expression::IndexAccess { object, index } => {
                let obj = Self::eval_expr(object, scope)?;
                let idx_val = Self::eval_expr(index, scope)?;
                match obj {
                    Variant::Array(ref arr) => {
                        let i = idx_val.to_int() as usize;
                        Ok(arr.get(i).cloned().unwrap_or(Variant::Nil))
                    }
                    Variant::Dictionary(ref map) => {
                        let key = idx_val.as_str();
                        Ok(map.get(key).cloned().unwrap_or(Variant::Nil))
                    }
                    _ => Err(VmError::TypeError("Cannot index non-collection".into())),
                }
            }
            Expression::ArrayLiteral(elements) => {
                let mut arr = Vec::new();
                for e in elements {
                    arr.push(Self::eval_expr(e, scope)?);
                }
                Ok(Variant::Array(arr))
            }
            Expression::DictLiteral(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), Self::eval_expr(v, scope)?);
                }
                Ok(Variant::Dictionary(map))
            }
        }
    }

    fn eval_binary_op(left: &Variant, op: BinaryOp, right: &Variant) -> Result<Variant, VmError> {
        match op {
            BinaryOp::Add => match (left, right) {
                (Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a + b)),
                (Variant::Float(a), Variant::Float(b)) => Ok(Variant::Float(a + b)),
                (Variant::Int(a), Variant::Float(b)) => Ok(Variant::Float(*a as f64 + b)),
                (Variant::Float(a), Variant::Int(b)) => Ok(Variant::Float(a + *b as f64)),
                (Variant::String(a), Variant::String(b)) => Ok(Variant::String(format!("{}{}", a, b))),
                _ => Err(VmError::TypeError("Invalid operand types for Add".into())),
            },
            BinaryOp::Sub => match (left, right) {
                (Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a - b)),
                (Variant::Float(a), Variant::Float(b)) => Ok(Variant::Float(a - b)),
                (Variant::Int(a), Variant::Float(b)) => Ok(Variant::Float(*a as f64 - b)),
                (Variant::Float(a), Variant::Int(b)) => Ok(Variant::Float(a - *b as f64)),
                _ => Err(VmError::TypeError("Invalid operand types for Sub".into())),
            },
            BinaryOp::Mul => match (left, right) {
                (Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a * b)),
                (Variant::Float(a), Variant::Float(b)) => Ok(Variant::Float(a * b)),
                (Variant::Int(a), Variant::Float(b)) => Ok(Variant::Float(*a as f64 * b)),
                (Variant::Float(a), Variant::Int(b)) => Ok(Variant::Float(a * *b as f64)),
                _ => Err(VmError::TypeError("Invalid operand types for Mul".into())),
            },
            BinaryOp::Div => match (left, right) {
                (Variant::Int(a), Variant::Int(b)) => {
                    if *b == 0 {
                        Ok(Variant::Int(0))
                    } else {
                        Ok(Variant::Int(a / b))
                    }
                }
                (Variant::Float(a), Variant::Float(b)) => Ok(Variant::Float(a / b)),
                _ => Err(VmError::TypeError("Invalid operand types for Div".into())),
            },
            BinaryOp::Mod => match (left, right) {
                (Variant::Int(a), Variant::Int(b)) => {
                    if *b == 0 {
                        Ok(Variant::Int(0))
                    } else {
                        Ok(Variant::Int(a % b))
                    }
                }
                _ => Err(VmError::TypeError("Invalid operand types for Mod".into())),
            },
            BinaryOp::Eq => Ok(Variant::Bool(left == right)),
            BinaryOp::Neq => Ok(Variant::Bool(left != right)),
            BinaryOp::Lt => Ok(Variant::Bool(left.to_float() < right.to_float())),
            BinaryOp::Lte => Ok(Variant::Bool(left.to_float() <= right.to_float())),
            BinaryOp::Gt => Ok(Variant::Bool(left.to_float() > right.to_float())),
            BinaryOp::Gte => Ok(Variant::Bool(left.to_float() >= right.to_float())),
            BinaryOp::And => Ok(Variant::Bool(left.to_bool() && right.to_bool())),
            BinaryOp::Or => Ok(Variant::Bool(left.to_bool() || right.to_bool())),
        }
    }

    /// Evaluates built-in Godot math functions
    fn eval_builtin_func(name: &str, args: &[Variant]) -> Result<Variant, VmError> {
        match name {
            "max" | "maxi" => {
                let a = args.first().map(|v| v.to_int()).unwrap_or(0);
                let b = args.get(1).map(|v| v.to_int()).unwrap_or(0);
                Ok(Variant::Int(a.max(b)))
            }
            "min" | "mini" => {
                let a = args.first().map(|v| v.to_int()).unwrap_or(0);
                let b = args.get(1).map(|v| v.to_int()).unwrap_or(0);
                Ok(Variant::Int(a.min(b)))
            }
            "abs" | "absi" => {
                let a = args.first().map(|v| v.to_int()).unwrap_or(0);
                Ok(Variant::Int(a.abs()))
            }
            "sign" | "signi" => {
                let a = args.first().map(|v| v.to_int()).unwrap_or(0);
                Ok(Variant::Int(a.signum()))
            }
            "clamp" | "clampi" => {
                let val = args.first().map(|v| v.to_int()).unwrap_or(0);
                let min = args.get(1).map(|v| v.to_int()).unwrap_or(0);
                let max = args.get(2).map(|v| v.to_int()).unwrap_or(val);
                Ok(Variant::Int(val.clamp(min, max)))
            }
            _ => Err(VmError::EvalError(format!("Unknown built-in function '{}'", name))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_binary_math() {
        let mut scope = Scope::new();
        scope.set("atk", Variant::Int(15));
        scope.set("armor", Variant::Int(4));

        let expr = Expression::Binary {
            left: Box::new(Expression::Identifier("atk".into())),
            op: BinaryOp::Sub,
            right: Box::new(Expression::Identifier("armor".into())),
        };

        let result = Evaluator::eval_expr(&expr, &mut scope).unwrap();
        assert_eq!(result.to_int(), 11);
    }

    #[test]
    fn test_eval_builtin_max() {
        let mut scope = Scope::new();
        let call = Expression::Call {
            callee: Box::new(Expression::Identifier("maxi".into())),
            args: vec![
                Expression::Literal(Variant::Int(1)),
                Expression::Literal(Variant::Int(-5)),
            ],
        };

        let result = Evaluator::eval_expr(&call, &mut scope).unwrap();
        assert_eq!(result.to_int(), 1);
    }
}
