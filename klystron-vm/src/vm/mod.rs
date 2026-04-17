use std::sync::Arc;

use klystron_types::*;

use crate::{
    error::{Error, Result},
    vm::{
        func::{Frame, FuncDef, FuncId},
        instr::Instruction,
        value::{CompareOp, Value},
    },
};

pub mod func;
pub mod instr;
pub mod value;

#[derive(Debug, Default)]
pub struct Vm {
    pub call_stack: Vec<Frame>,
    pub functions: Vec<Arc<FuncDef>>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn define_function(&mut self, def: Arc<FuncDef>) -> FuncId {
        self.functions.push(def);
        (self.functions.len() - 1) as FuncId
    }

    pub fn get_function(&self, func_id: FuncId) -> Result<Arc<FuncDef>> {
        Ok(self
            .functions
            .get(func_id as usize)
            .ok_or(Error::UnknownFunc(func_id))?
            .clone())
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.call_stack.push(frame);
    }

    pub fn push_call(&mut self, func_id: FuncId, args: Vec<Value>) -> Result<()> {
        let func = self.get_function(func_id)?;
        if args.len() != func.sig.params.len() {
            return Err(Error::WrongArgCount {
                expected: func.sig.params.len(),
                got: args.len(),
            });
        }
        self.call_stack.push(Frame::new(func, args));
        Ok(())
    }

    fn ret(&mut self) -> Result<Option<Value>> {
        let mut old_frame = self.call_stack.pop().ok_or(Error::EmptyCallStack)?;

        let ret_val = old_frame.stack.pop();

        if self.call_stack.is_empty() {
            return Ok(ret_val);
        }

        if let Some(val) = ret_val {
            self.call_stack.last_mut().unwrap().stack.push(val);
        }

        Ok(None)
    }

    pub fn run(&mut self) -> Result<Option<Value>> {
        loop {
            let frame = self.call_stack.last_mut().ok_or(Error::EmptyCallStack)?;

            let instr = frame.func.code[frame.ip].clone();
            frame.ip += 1;

            log::trace!("{:?}", instr);

            match interpret_instr(frame, instr)? {
                ExecEvent::Call(func_id) => {
                    let func = self.get_function(func_id)?;
                    let n = func.sig.params.len();
                    let args = self
                        .call_stack
                        .last_mut()
                        .ok_or(Error::EmptyCallStack)?
                        .pop_n(n)?;
                    self.call_stack.push(Frame::new(func, args));
                }
                ExecEvent::Ret => {
                    let ret_val = self.ret()?;
                    if self.call_stack.is_empty() {
                        return Ok(ret_val);
                    }
                }
                ExecEvent::Halt => {
                    return Ok(self.call_stack.last_mut().and_then(|f| f.stack.pop()));
                }
                ExecEvent::Continue => {}
            }
        }
    }
}

pub enum ExecEvent {
    Continue,
    Halt,
    Call(FuncId),
    Ret,
}

pub fn interpret_instr(frame: &mut Frame, instr: Instruction) -> Result<ExecEvent> {
    match instr {
        Instruction::Push(v) => frame.push(v),
        Instruction::Pop => {
            frame.pop()?;
        }
        Instruction::Dup => {
            let v = frame.pop()?;
            frame.push(v.clone());
            frame.push(v);
        }
        Instruction::Load(id) => match frame.locals.get(id as usize) {
            None => return Err(Error::InvalidLocal(id)),
            Some(None) => return Err(Error::UninitializedLocal(id)),
            Some(Some(v)) => frame.push(v.clone()),
        },
        Instruction::Store(id) => {
            let val = frame.pop()?;
            *frame
                .locals
                .get_mut(id as usize)
                .ok_or(Error::InvalidLocal(id))? = Some(val);
        }
        Instruction::Br(target) => {
            frame.ip = target;
        }
        Instruction::BrIf(target) => match frame.pop()? {
            Value::Bool(true) => frame.ip = target,
            Value::Bool(false) => {}
            other => {
                return Err(Error::TypeMismatch {
                    expected: TypeKind::Bool,
                    got: other.ty(),
                });
            }
        },
        Instruction::Call(func_id) => {
            return Ok(ExecEvent::Call(func_id));
        }
        Instruction::Ret => {
            return Ok(ExecEvent::Ret);
        }
        Instruction::Halt => {
            return Ok(ExecEvent::Halt);
        }
        Instruction::Add => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.add(b)?);
        }
        Instruction::Sub => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.sub(b)?);
        }
        Instruction::Mul => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.mul(b)?);
        }
        Instruction::Div => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.div(b)?);
        }
        Instruction::Eq => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.cmp(b, CompareOp::Eq)?);
        }
        Instruction::Ne => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.cmp(b, CompareOp::Ne)?);
        }
        Instruction::Lt => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.cmp(b, CompareOp::Lt)?);
        }
        Instruction::Gt => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.cmp(b, CompareOp::Gt)?);
        }
        Instruction::Le => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.cmp(b, CompareOp::Le)?);
        }
        Instruction::Ge => {
            let b = frame.pop()?;
            let a = frame.pop()?;
            frame.push(a.cmp(b, CompareOp::Ge)?);
        }
        Instruction::BitAnd => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.bitand(b)?);
        }
        Instruction::BitOr => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.bitor(b)?);
        }
        Instruction::BitXor => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.bitxor(b)?);
        }
        Instruction::BitShl => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.shl(b)?);
        }
        Instruction::BitShr => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.shr(b)?);
        }
        Instruction::And => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.and(b)?);
        }
        Instruction::Or => {
            let b = frame.pop()?;
            let a = frame.pop()?;

            frame.push(a.or(b)?);
        }
        Instruction::Not => {
            let b = frame.pop()?;
            frame.push(b.not()?);
        }
        Instruction::Neg => {
            let b = frame.pop()?;
            frame.push(b.neg()?);
        }
    };

    Ok(ExecEvent::Continue)
}
