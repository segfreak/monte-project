#![allow(unused)]

use std::sync::Arc;

use klystron_types::*;
use klystron_vm::{
    error::Result,
    vm::{
        func::{FuncDef, FuncSignature},
        instr::Instruction,
        value::Value,
        Vm,
    },
    *,
};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut vm = Vm::new();

    let fib = vm.define_function(Arc::new(FuncDef {
        sig: FuncSignature {
            params: vec![TypeKind::Int32],
            returns: Some(TypeKind::Int32),
        },
        locals: vec![
            TypeKind::Int32, // n (0)
            TypeKind::Int32, // a (1)
            TypeKind::Int32, // b (2)
            TypeKind::Int32, // i (3)
            TypeKind::Int32, // t (4)
        ],
        code: vec![
            /* 0 */ Instruction::Load(0),
            /* 1 */ Instruction::Push(Value::Int32(1)),
            /* 2 */ Instruction::Le, // n <= 1
            /* 3 */ Instruction::BrIf(27), // → early return
            /* 4 */ Instruction::Push(Value::Int32(0)),
            /* 5 */ Instruction::Store(1), // a = 0
            /* 6 */ Instruction::Push(Value::Int32(1)),
            /* 7 */ Instruction::Store(2), // b = 1
            /* 8 */ Instruction::Push(Value::Int32(2)),
            /* 9 */ Instruction::Store(3), // i = 2
            // loop start
            /* 10 */ Instruction::Load(3),
            /* 11 */ Instruction::Load(0),
            /* 12 */ Instruction::Gt, // i > n
            /* 13 */ Instruction::BrIf(29), // → loop exit
            /* 14 */ Instruction::Load(1),
            /* 15 */ Instruction::Load(2),
            /* 16 */ Instruction::Add,
            /* 17 */ Instruction::Store(4), // t = a + b
            /* 18 */ Instruction::Load(2),
            /* 19 */ Instruction::Store(1), // a = b
            /* 20 */ Instruction::Load(4),
            /* 21 */ Instruction::Store(2), // b = t
            /* 22 */ Instruction::Load(3),
            /* 23 */ Instruction::Push(Value::Int32(1)),
            /* 24 */ Instruction::Add,
            /* 25 */ Instruction::Store(3), // i = i + 1
            /* 26 */ Instruction::Br(10), // → loop start
            // early return: n <= 1
            /* 27 */ Instruction::Load(0), // push n
            /* 28 */ Instruction::Ret,
            // loop exit
            /* 29 */ Instruction::Load(2), // push b
            /* 30 */ Instruction::Ret,
        ],
    }));

    let main = vm.define_function(Arc::new(FuncDef {
        sig: FuncSignature {
            params: vec![],
            returns: Some(TypeKind::Int32),
        },
        locals: vec![],
        code: vec![
            Instruction::Push(Value::Int32(20)),
            Instruction::Call(fib),
            Instruction::Halt,
        ],
    }));

    vm.push_call(main, vec![]);

    let result = vm.run()?;
    log::info!("main() = {:?}", result);

    Ok(())
}
