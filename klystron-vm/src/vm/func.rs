use klystron_types::*;

use std::sync::Arc;

use super::instr::*;
use crate::{
    error::{Error, Result},
    vm::value::Value,
};

pub type LocalId = u32;
pub type FuncId = u32;

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub sig: FunctionSig,
    pub locals: Vec<TypeKind>,
    pub code: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub func: Arc<FuncDef>,
    /// instruction pointer (inside a function)
    pub ip: usize,
    /// locals (None = uninitialized value)
    pub locals: Vec<Option<Value>>,
    /// stack of frame
    pub stack: Vec<Value>,
}

impl Frame {
    pub fn new(func: Arc<FuncDef>, args: Vec<Value>) -> Self {
        let mut locals = vec![None; func.locals.len()];
        for (i, arg) in args.into_iter().enumerate() {
            locals[i] = Some(arg);
        }
        Self {
            func,
            ip: 0,
            locals,
            stack: Vec::new(),
        }
    }

    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::StackUnderflow)
    }

    pub fn pop_n(&mut self, n: usize) -> Result<Vec<Value>> {
        if self.stack.len() < n {
            return Err(Error::StackUnderflow);
        }
        let at = self.stack.len() - n;
        Ok(self.stack.drain(at..).collect())
    }

    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }
}
