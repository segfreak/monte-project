use klystron_types::{FunctionSig, HostFloat, HostInt, TypeKind};

use crate::repr::*;

#[derive(Debug)]
pub struct BlockBuilder<'a> {
    func: &'a mut FunctionBuilder,
    block_id: BlockId,
    params: Vec<ValueId>,
    instrs: Vec<Instruction>,
    term: Option<TerminatorKind>,
}

#[derive(Debug)]
pub struct FunctionBuilder {
    pub name: String,
    pub sig: FunctionSig,

    pub def: FunctionDef,
    current_block: Option<BlockId>,
    next_block_id: BlockId,
}

impl FunctionBuilder {
    pub fn new(name: String, sig: FunctionSig) -> Self {
        let def = FunctionDef {
            blocks: Vec::new(),
            entry: 0,
            values: Vec::new(),
            next_value_id: 0,
        };

        let mut fb = Self {
            name,
            sig,
            def,
            current_block: None,
            next_block_id: 0,
        };

        fb.entry();
        fb
    }

    pub fn create_block<'a>(&'a mut self) -> BlockBuilder<'a> {
        let block_id = self.next_block_id;
        self.next_block_id += 1;

        let bb = BasicBlock {
            id: block_id,
            params: vec![],
            instrs: vec![],
            term: TerminatorKind::Ret(None),
        };

        self.def.blocks.push(bb);
        self.current_block = Some(block_id);

        BlockBuilder {
            func: self,
            block_id,
            params: vec![],
            instrs: vec![],
            term: None,
        }
    }

    pub fn entry<'a>(&'a mut self) -> BlockBuilder<'a> {
        if self.def.blocks.is_empty() {
            self.def.entry = 0;
            self.create_block()
        } else {
            self.switch_to_block(0)
        }
    }

    pub fn switch_to_block<'a>(&'a mut self, block_id: BlockId) -> BlockBuilder<'a> {
        self.current_block = Some(block_id);
        BlockBuilder {
            func: self,
            block_id,
            params: vec![],
            instrs: vec![],
            term: None,
        }
    }

    pub fn finish(self) -> FunctionDef {
        self.def
    }
}

impl<'a> BlockBuilder<'a> {
    pub fn param(&mut self, ty: TypeKind) -> ValueId {
        let id = self.func.def.fresh_value(ty);
        self.params.push(id);
        id
    }

    pub fn inst(&mut self, kind: InstructionKind, ty: TypeKind) -> ValueId {
        let id = self.func.def.fresh_value(ty);
        let instr = Instruction { id, kind };

        self.instrs.push(instr);
        id
    }

    pub fn add(&mut self, a: ValueId, b: ValueId) -> ValueId {
        let ty = self.func.def.get_type(a);
        self.inst(InstructionKind::Add(a, b), ty)
    }

    pub fn sub(&mut self, a: ValueId, b: ValueId) -> ValueId {
        let ty = self.func.def.get_type(a);
        self.inst(InstructionKind::Sub(a, b), ty)
    }

    pub fn mul(&mut self, a: ValueId, b: ValueId) -> ValueId {
        let ty = self.func.def.get_type(a);
        self.inst(InstructionKind::Mul(a, b), ty)
    }

    pub fn div(&mut self, a: ValueId, b: ValueId) -> ValueId {
        let ty = self.func.def.get_type(a);
        self.inst(InstructionKind::Div(a, b), ty)
    }

    fn constant(&mut self, ty: TypeKind, constant: Constant) -> ValueId {
        let id = self.func.def.fresh_value(ty);
        let instr = Instruction {
            id,
            kind: InstructionKind::Const(constant),
        };
        self.instrs.push(instr);
        id
    }

    fn const_int(&mut self, ty: TypeKind, value: HostInt) -> ValueId {
        self.constant(ty, Constant::Int(value))
    }

    fn const_float(&mut self, ty: TypeKind, value: HostFloat) -> ValueId {
        self.constant(ty, Constant::Float(value))
    }

    pub fn const_int8(&mut self, value: HostInt) -> ValueId {
        self.const_int(TypeKind::Int8, value)
    }

    pub fn const_int16(&mut self, value: HostInt) -> ValueId {
        self.const_int(TypeKind::Int16, value)
    }
    pub fn const_int32(&mut self, value: HostInt) -> ValueId {
        self.const_int(TypeKind::Int32, value)
    }
    pub fn const_int64(&mut self, value: HostInt) -> ValueId {
        self.const_int(TypeKind::Int64, value)
    }

    pub fn const_float32(&mut self, value: HostFloat) -> ValueId {
        self.const_float(TypeKind::Float32, value)
    }
    pub fn const_float64(&mut self, value: HostFloat) -> ValueId {
        self.const_float(TypeKind::Float64, value)
    }

    pub fn const_bool(&mut self, value: bool) -> ValueId {
        self.constant(TypeKind::Bool, Constant::Bool(value))
    }

    pub fn ret(&mut self, value: Option<ValueId>) {
        self.term = Some(TerminatorKind::Ret(value));
        self.finish();
    }

    pub fn br(&mut self, block: BlockId, params: Vec<ValueId>) {
        self.term = Some(TerminatorKind::Br { block, params });
        self.finish();
    }

    pub fn br_if(
        &mut self,
        cond: ValueId,
        then_block: BlockId,
        then_params: Vec<ValueId>,
        else_block: Option<BlockId>,
        else_params: Option<Vec<ValueId>>,
    ) {
        self.term = Some(TerminatorKind::BrIf {
            cond,
            then_block,
            then_params,
            else_block,
            else_params,
        });
        self.finish();
    }

    pub fn finish(&mut self) {
        if let Some(term) = &self.term {
            if let Some(block) = self
                .func
                .def
                .blocks
                .iter_mut()
                .find(|b| b.id == self.block_id)
            {
                block.params = self.params.clone();
                block.instrs = self.instrs.clone();
                block.term = term.clone();
            } else {
                log::error!("could not find block {}", self.block_id);
            }
        } else {
            log::error!("block {} must be terminated", self.block_id)
        }
    }
}
