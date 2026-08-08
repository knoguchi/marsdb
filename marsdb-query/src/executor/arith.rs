//! Arithmetic over `Value` operands (`+`/`-`/`*`/`/`/`%`/`^`, unary minus)
//! and the numeric coercions they share.

use super::*;

/// A number coerced out of a `Value`, for `apply_arith` below -- separate
/// from `PropertyValue`/`Literal` since either could hold the operand
/// (`n.price + 1` mixes a stored property with a literal).
pub(crate) enum ArithNum {
    Int(i64),
    Float(f64),
}

pub(crate) fn as_arith_num(v: &Value) -> Option<ArithNum> {
    match v {
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => {
            Some(ArithNum::Int(*i))
        }
        Value::Property(PropertyValue::Float(f)) | Value::Literal(Literal::Float(f)) => {
            Some(ArithNum::Float(*f))
        }
        _ => None,
    }
}

/// `datetime.fromepoch(seconds, nanos)`/`datetime.fromepochmillis(millis)`'s
/// own argument check -- both take a required, definite integer, not the
/// wider "any arithmetic-ish value" `as_arith_num` allows (no float
/// coercion for a raw epoch count) and not optional (missing/null isn't a
/// documented no-op the way it is for e.g. `date()`'s own no-arg form).
pub(crate) fn require_int_arg(v: Option<&Value>, fn_name: &str) -> Result<i64, QueryError> {
    match v {
        Some(Value::Property(PropertyValue::Int(i))) | Some(Value::Literal(Literal::Int(i))) => {
            Ok(*i)
        }
        other => Err(QueryError::Type(format!(
            "{fn_name}() expects an integer argument, got {other:?}"
        ))),
    }
}

pub(crate) fn as_arith_str(v: &Value) -> Option<&str> {
    match v {
        Value::Property(PropertyValue::String(s)) | Value::Literal(Literal::String(s)) => {
            Some(s.as_str())
        }
        _ => None,
    }
}

/// `-x` for `ReturnExpr::Neg` -- a negative numeric *literal* (`-3`)
/// never reaches this (see `cypher.pest`'s `unary_minus_expr` docs), so
/// this only ever handles negating a genuinely computed/bound value
/// (`-n.prop`, `-(1+2)`, ...). Same null-propagation/numeric-only
/// convention as `apply_arith`.
pub(crate) fn apply_neg(v: &Value) -> Result<Value, QueryError> {
    if matches!(v, Value::Null) {
        return Ok(Value::Null);
    }
    Ok(match as_arith_num(v) {
        Some(ArithNum::Int(i)) => {
            Value::Property(PropertyValue::Int(i.checked_neg().ok_or_else(|| {
                QueryError::Type("integer arithmetic overflow".into())
            })?))
        }
        Some(ArithNum::Float(f)) => Value::Property(PropertyValue::Float(-f)),
        None => {
            return Err(QueryError::Type(format!(
                "unary minus needs a number -- got {v:?}"
            )))
        }
    })
}

/// `lhs op rhs` for `ReturnExpr::Arith`. Null propagates (matches every
/// other operator's null-handling convention in this file). `+` also
/// concatenates two strings, real Cypher's other overload for that
/// operator; every other combination of non-numeric operands is a real
/// type error, not a silent `Null`/`false` fallback -- an arithmetic
/// expression that can't be evaluated should say so, not produce a
/// plausible-looking wrong answer.
pub(crate) fn apply_arith(op: ArithOp, a: &Value, b: &Value) -> Result<Value, QueryError> {
    if matches!(a, Value::Null) || matches!(b, Value::Null) {
        return Ok(Value::Null);
    }
    if op == ArithOp::Add {
        // Real Cypher's list concatenation/append/prepend via `+` --
        // `[1,2] + [3]` concatenates, `[1,2] + 3`/`3 + [1,2]` appends/
        // prepends the scalar. Only `+` has this meaning for a list;
        // every other `ArithOp` still rejects one via the numeric-only
        // fallback below (and at compile time, `semantic.rs`'s own
        // `ReturnExpr::Arith` check).
        match (a, b) {
            (Value::List(xs), Value::List(ys)) => {
                let mut combined = xs.clone();
                combined.extend(ys.iter().cloned());
                return Ok(Value::List(combined));
            }
            (Value::List(xs), scalar) => {
                let mut combined = xs.clone();
                combined.push(scalar.clone());
                return Ok(Value::List(combined));
            }
            (scalar, Value::List(ys)) => {
                let mut combined = vec![scalar.clone()];
                combined.extend(ys.iter().cloned());
                return Ok(Value::List(combined));
            }
            _ => {}
        }
        if let (Some(sa), Some(sb)) = (as_arith_str(a), as_arith_str(b)) {
            return Ok(Value::Property(PropertyValue::String(format!("{sa}{sb}"))));
        }
    }
    if let Some(result) = apply_temporal_arith(op, a, b)? {
        return Ok(result);
    }
    let (Some(na), Some(nb)) = (as_arith_num(a), as_arith_num(b)) else {
        return Err(QueryError::Type(format!(
            "arithmetic needs two numbers (or, for +, two strings) -- got {a:?} and {b:?}"
        )));
    };
    // `^` always produces a Float, even for two Ints (real Cypher's own
    // rule) -- handled up front, separately from the Int/Int-stays-Int
    // branch below, rather than folding it into that match's own `op`
    // dispatch.
    if op == ArithOp::Pow {
        let to_f64 = |n: ArithNum| match n {
            ArithNum::Int(i) => i as f64,
            ArithNum::Float(f) => f,
        };
        return Ok(Value::Property(PropertyValue::Float(
            to_f64(na).powf(to_f64(nb)),
        )));
    }
    // Int/Int stays Int (truncating division/modulo, matching Rust's `/`/
    // `%` on integers) -- any Float operand promotes the whole expression
    // to Float, same numeric-promotion rule `compare()` already follows.
    Ok(match (na, nb) {
        (ArithNum::Int(x), ArithNum::Int(y)) => {
            if matches!(op, ArithOp::Div | ArithOp::Mod) && y == 0 {
                return Err(QueryError::Type("division by zero".into()));
            }
            let value = match op {
                ArithOp::Add => x.checked_add(y),
                ArithOp::Sub => x.checked_sub(y),
                ArithOp::Mul => x.checked_mul(y),
                ArithOp::Div => x.checked_div(y),
                ArithOp::Mod => x.checked_rem(y),
                ArithOp::Pow => unreachable!("handled above"),
            }
            .ok_or_else(|| QueryError::Type("integer arithmetic overflow".into()))?;
            Value::Property(PropertyValue::Int(value))
        }
        (x, y) => {
            let x = match x {
                ArithNum::Int(i) => i as f64,
                ArithNum::Float(f) => f,
            };
            let y = match y {
                ArithNum::Int(i) => i as f64,
                ArithNum::Float(f) => f,
            };
            Value::Property(PropertyValue::Float(match op {
                ArithOp::Add => x + y,
                ArithOp::Sub => x - y,
                ArithOp::Mul => x * y,
                ArithOp::Div => x / y,
                ArithOp::Mod => x % y,
                ArithOp::Pow => unreachable!("handled above"),
            }))
        }
    })
}

pub(crate) fn value_as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => Some(*i),
        _ => None,
    }
}

pub(crate) fn value_as_f64(v: &Value) -> Option<f64> {
    match as_arith_num(v)? {
        ArithNum::Int(i) => Some(i as f64),
        ArithNum::Float(f) => Some(f),
    }
}
