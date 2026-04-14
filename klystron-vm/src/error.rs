use typesys::*;

use crate::vm::func::{FuncId, LocalId};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("type errror: {0}")]
    TypeError(String),
    #[error("type mismatch: expected {expected:?}, got {got:?}")]
    TypeMismatch { expected: TypeKind, got: TypeKind },
    #[error("invalid local #{0}")]
    InvalidLocal(LocalId),
    #[error("uninitialized local #{0}")]
    UninitializedLocal(LocalId),
    #[error("unknown function #{0}")]
    UnknownFunc(FuncId),
    #[error("empty call stack")]
    EmptyCallStack,
    #[error("wrong arg count: expected {expected}, got {got}")]
    WrongArgCount { expected: usize, got: usize },
    #[error("division by zero")]
    DivisionByZero,
}
