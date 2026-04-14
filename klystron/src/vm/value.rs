use typesys::{HostFloat, HostInt, TypeKind};

use crate::error::{Error, Result};

use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub};

macro_rules! impl_unop {
    ($method:ident, $($variant:ident),*) => {
        impl Value {
            pub fn $method(self) -> Result<Self> {
                match self {
                    $(
                        Value::$variant(a) => Ok(Value::$variant(a.$method())),
                    )*
                    _ => Err(Error::TypeError(format!(
                        "unary operation '{}' is not supported for this type",
                        stringify!($method)
                    ))),
                }
            }
        }
    };
    ($method:ident, $op:tt, $($variant:ident),*) => {
        impl Value {
            pub fn $method(self) -> Result<Self> {
                match self {
                    $(
                        Value::$variant(a) => Ok(Value::$variant($op a)),
                    )*
                    _ => Err(Error::TypeError(format!(
                        "unary operation '{}' is not supported for this type",
                        stringify!($method)
                    ))),
                }
            }
        }
    };
}

macro_rules! impl_binop {
    ($method:ident, $($variant:ident),*) => {
        impl Value {
            pub fn $method(self, rhs: Self) -> Result<Self> {
                match (self, rhs) {
                    $(
                        (Value::$variant(a), Value::$variant(b)) => Ok(Value::$variant(a.$method(b))),
                    )*
                    _ => Err(Error::TypeError(format!(
                        "operation '{}' is not supported for these types",
                        stringify!($method)
                    ))),
                }
            }
        }
    };
    ($method:ident, $op:tt, $($variant:ident),*) => {
        impl Value {
            pub fn $method(self, rhs: Self) -> Result<Self> {
                match (self, rhs) {
                    $(
                        (Value::$variant(a), Value::$variant(b)) => Ok(Value::$variant(a $op b)),
                    )*
                    _ => Err(Error::TypeError(format!(
                        "operation '{}' is not supported for these types",
                        stringify!($method)
                    ))),
                }
            }
        }
    };
}

impl_binop!(add, Int8, Int16, Int32, Int64, Float32, Float64);
impl_binop!(sub, Int8, Int16, Int32, Int64, Float32, Float64);
impl_binop!(mul, Int8, Int16, Int32, Int64, Float32, Float64);
impl_binop!(div, Int8, Int16, Int32, Int64, Float32, Float64);

impl_binop!(bitand, Bool, Int8, Int16, Int32, Int64);
impl_binop!(bitor, Bool, Int8, Int16, Int32, Int64);
impl_binop!(bitxor, Bool, Int8, Int16, Int32, Int64);

impl_binop!(shl, Int8, Int16, Int32, Int64);
impl_binop!(shr, Int8, Int16, Int32, Int64);

impl_unop!(not, Bool, Int8, Int16, Int32, Int64);

impl_binop!(or, ||, Bool);
impl_binop!(and, &&, Bool);

impl_unop!(neg, Int8, Int16, Int32, Int64, Float32, Float64);

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

pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
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

    pub fn as_int(&self) -> Result<HostInt> {
        match self {
            Value::Int8(x) => Ok(*x as HostInt),
            Value::Int16(x) => Ok(*x as HostInt),
            Value::Int32(x) => Ok(*x as HostInt),
            Value::Int64(x) => Ok(*x as HostInt),
            _ => Err(Error::TypeError("must be integer".into())),
        }
    }

    pub fn as_float(&self) -> Result<HostFloat> {
        match self {
            Value::Float32(x) => Ok(*x as HostFloat),
            Value::Float64(x) => Ok(*x as HostFloat),
            _ => Err(Error::TypeError("must be float".into())),
        }
    }

    pub(super) fn cmp(self, rhs: Value, op: CompareOp) -> Result<Value> {
        macro_rules! do_cmp {
            ($a:expr, $b:expr) => {
                Ok(Value::Bool(match op {
                    CompareOp::Eq => $a == $b,
                    CompareOp::Ne => $a != $b,
                    CompareOp::Lt => $a < $b,
                    CompareOp::Gt => $a > $b,
                    CompareOp::Le => $a <= $b,
                    CompareOp::Ge => $a >= $b,
                    #[allow(unused)]
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
