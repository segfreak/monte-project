use klystron_types::{FunctionSig, HostFloat, HostInt, TypeKind};

pub type ValueId = u32;
pub type BlockId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(HostInt),
    Float(HostFloat),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueInfo {
    pub ty: TypeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BlockId,
    pub params: Vec<ValueId>,
    pub instrs: Vec<Instruction>,
    pub term: TerminatorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub values: Vec<ValueInfo>,
    pub next_value_id: ValueId,
}

impl FunctionDef {
    pub fn fresh_value(&mut self, ty: TypeKind) -> ValueId {
        let id = self.next_value_id;
        self.values.push(ValueInfo { ty });
        self.next_value_id += 1;
        id
    }

    pub fn get_type(&self, id: ValueId) -> TypeKind {
        self.get_info(id).ty.clone()
    }

    pub fn get_info(&self, id: ValueId) -> &ValueInfo {
        &self.values[id as usize]
    }

    pub fn is_const(&self, id: ValueId) -> bool {
        self.get_const(id).is_some()
    }

    pub fn get_const(&self, id: ValueId) -> Option<&Constant> {
        if !self.is_valid(id) {
            return None;
        }

        for block in &self.blocks {
            for instr in &block.instrs {
                if instr.id == id {
                    return match &instr.kind {
                        InstructionKind::Const(c) => Some(c),
                        _ => None,
                    };
                }
            }
        }
        None
    }

    pub fn is_valid(&self, id: ValueId) -> bool {
        (id as usize) < self.values.len()
    }

    pub fn is_valid_with_type(&self, id: ValueId, ty: TypeKind) -> bool {
        if !self.is_valid(id) {
            return false;
        }

        self.get_info(id).ty == ty
    }
}

impl std::fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "function ({} values, {} blocks):",
            self.values.len(),
            self.blocks.len()
        )?;

        for block in &self.blocks {
            let is_entry = block.id == self.entry;

            writeln!(
                f,
                "\n{}block {}:",
                if is_entry { "entry " } else { "" },
                block.id
            )?;

            if !block.params.is_empty() {
                write!(f, "  params: ")?;
                for (i, &p) in block.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "v{}", p)?;
                }
                writeln!(f)?;
            }

            for instr in &block.instrs {
                writeln!(f, "  v{} = {}", instr.id, instr.kind)?;
            }

            writeln!(f, "  {}", block.term)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub sig: FunctionSig,
    pub def: FunctionDef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionKind {
    Const(Constant),

    Add(ValueId, ValueId),
    Sub(ValueId, ValueId),
    Mul(ValueId, ValueId),
    Div(ValueId, ValueId),
}

impl std::fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionKind::Const(c) => match c {
                Constant::Int(v) => write!(f, "const {}", v),
                Constant::Float(v) => write!(f, "const {}", v),
                Constant::Bool(v) => write!(f, "const {}", v),
            },
            InstructionKind::Add(a, b) => write!(f, "add v{}, v{}", a, b),
            InstructionKind::Sub(a, b) => write!(f, "sub v{}, v{}", a, b),
            InstructionKind::Mul(a, b) => write!(f, "mul v{}, v{}", a, b),
            InstructionKind::Div(a, b) => write!(f, "div v{}, v{}", a, b),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub id: ValueId,
    pub kind: InstructionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerminatorKind {
    Ret(Option<ValueId>),

    Br(BlockId),
    BrIf {
        cond: ValueId,
        then_block: BlockId,
        then_params: Vec<ValueId>,
        else_block: Option<BlockId>,
        else_params: Option<Vec<ValueId>>,
    },
}

impl std::fmt::Display for TerminatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminatorKind::Ret(None) => write!(f, "ret"),
            TerminatorKind::Ret(Some(v)) => write!(f, "ret v{}", v),
            TerminatorKind::Br(target) => write!(f, "br block {}", target),
            TerminatorKind::BrIf {
                cond,
                then_block,
                else_block,
                ..
            } => {
                if let Some(else_b) = else_block {
                    write!(
                        f,
                        "br_if v{} -> block {}, else block {}",
                        cond, then_block, else_b
                    )
                } else {
                    write!(f, "br_if v{} -> block {}", cond, then_block)
                }
            }
        }
    }
}
