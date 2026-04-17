use std::fs;

use klystron_ir::{
    analysis::{cfg::ControlFlowGraph, dfg::DataFlowGraph},
    builder::FunctionBuilder,
    repr::FunctionDef,
};
use klystron_types::{FunctionSig, TypeKind};

pub fn create_test_function() -> FunctionDef {
    let sig = FunctionSig::new(&[TypeKind::Int32, TypeKind::Int32], None);

    let mut b = FunctionBuilder::new("foo".to_string(), sig);

    {
        let mut entry = b.entry();

        let x = entry.param(TypeKind::Int32);
        let y = entry.param(TypeKind::Int32);

        let const0 = entry.const_int32(228);
        let const1 = entry.const_int32(1337);
        let add2consts = entry.add(const0, const1);

        let res0 = entry.add(x, y);
        let res = entry.add(add2consts, res0);

        entry.ret(Some(res));
    }

    b.finish()
}

fn main() {
    let def = create_test_function();
    def.verify().expect("verify error");

    let cfg = ControlFlowGraph::build(&def);
    let dfg = DataFlowGraph::build(&def);

    fs::write("cfg.dot", cfg.to_dot()).expect("cannot write");
    fs::write("dfg.dot", dfg.to_dot()).expect("cannot write");

    println!("{}", def);
}
