use crate::{
    error::{Error, Result},
    types::{HostFloat, HostInt, TypeKind},
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),

    Float32(f32),
    Float64(f64),

    Bool(bool),
}

fn arith_int(a: HostInt, b: HostInt, op: &str) -> Result<HostInt> {
    match op {
        "add" => Ok(a.wrapping_add(b)),
        "sub" => Ok(a.wrapping_sub(b)),
        "mul" => Ok(a.wrapping_mul(b)),
        "div" => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            Ok(a / b)
        }
        _ => unreachable!(),
    }
}

fn arith_float(a: HostFloat, b: HostFloat, op: &str) -> Result<f64> {
    match op {
        "add" => Ok(a + b),
        "sub" => Ok(a - b),
        "mul" => Ok(a * b),
        "div" => Ok(a / b),
        _ => unreachable!(),
    }
}

impl Value {
    pub fn ty(&self) -> TypeKind {
        match self {
            Self::Int8(_) => TypeKind::Int8,
            Self::Int16(_) => TypeKind::Int16,
            Self::Int32(_) => TypeKind::Int32,
            Self::Int64(_) => TypeKind::Int64,

            Self::Float32(_) => TypeKind::Float32,
            Self::Float64(_) => TypeKind::Float64,

            Self::Bool(_) => TypeKind::Bool,
        }
    }

    pub(super) fn arith(self, rhs: Value, op: &str) -> Result<Value> {
        match (self, rhs) {
            (Value::Int8(a), Value::Int8(b)) => {
                Ok(Value::Int8(arith_int(a as HostInt, b as HostInt, op)? as i8))
            }
            (Value::Int16(a), Value::Int16(b)) => {
                Ok(Value::Int16(
                    arith_int(a as HostInt, b as HostInt, op)? as i16
                ))
            }
            (Value::Int32(a), Value::Int32(b)) => {
                Ok(Value::Int32(
                    arith_int(a as HostInt, b as HostInt, op)? as i32
                ))
            }
            (Value::Int64(a), Value::Int64(b)) => {
                Ok(Value::Int64(
                    arith_int(a as HostInt, b as HostInt, op)? as i64
                ))
            }
            (Value::Float32(a), Value::Float32(b)) => {
                Ok(Value::Float32(
                    arith_float(a as HostFloat, b as HostFloat, op)? as f32,
                ))
            }
            (Value::Float64(a), Value::Float64(b)) => {
                Ok(Value::Float64(
                    arith_float(a as HostFloat, b as HostFloat, op)? as f64,
                ))
            }
            (a, b) => Err(Error::TypeMismatch {
                expected: a.ty(),
                got: b.ty(),
            }),
        }
    }

    pub(super) fn cmp(self, rhs: Value, op: &str) -> Result<Value> {
        macro_rules! do_cmp {
            ($a:expr, $b:expr) => {
                Ok(Value::Bool(match op {
                    "eq" => $a == $b,
                    "ne" => $a != $b,
                    "lt" => $a < $b,
                    "gt" => $a > $b,
                    "le" => $a <= $b,
                    "ge" => $a >= $b,
                    _ => unreachable!(),
                }))
            };
        }
        match (self, rhs) {
            (Value::Int8(a), Value::Int8(b)) => do_cmp!(a, b),
            (Value::Int16(a), Value::Int16(b)) => do_cmp!(a, b),
            (Value::Int32(a), Value::Int32(b)) => do_cmp!(a, b),
            (Value::Int64(a), Value::Int64(b)) => do_cmp!(a, b),
            (Value::Float32(a), Value::Float32(b)) => do_cmp!(a, b),
            (Value::Float64(a), Value::Float64(b)) => do_cmp!(a, b),
            (Value::Bool(a), Value::Bool(b)) => do_cmp!(a, b),
            (a, b) => Err(Error::TypeMismatch {
                expected: a.ty(),
                got: b.ty(),
            }),
        }
    }
}
