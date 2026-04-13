use crate::vm::{
    func::{FuncId, LocalId},
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// push value at stack
    Push(Value),
    /// pop value from stack
    Pop,
    Dup,

    /// load local into stack
    Load(LocalId),
    /// store top of stack into local
    Store(LocalId),

    /// branch
    Br(usize),
    /// conditional branch
    BrIf(usize),

    /// pop value if non-void, restore frame
    Ret,
    Call(FuncId),

    /// halt execution
    Halt,

    Add,
    Sub,
    Mul,
    Div,

    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}
