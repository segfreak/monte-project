use std::collections::HashSet;

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

    Br {
        target: BlockId,
        params: Vec<ValueId>,
    },
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
            TerminatorKind::Br { target, params } => {
                write!(f, "br block {} ({:?})", target, params)
            }
            TerminatorKind::BrIf {
                cond,
                then_block,
                then_params,
                else_block,
                else_params,
            } => {
                if let (Some(else_b), Some(else_p)) = (else_block, else_params) {
                    write!(
                        f,
                        "br_if v{} -> block {} ({:?}), else block {} ({:?})",
                        cond, then_block, then_params, else_b, else_p
                    )
                } else {
                    write!(
                        f,
                        "br_if v{} -> block {} ({:?})",
                        cond, then_block, then_params
                    )
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct VerifyError {
    pub message: String,
}

pub fn verify_function(func: &FunctionDef) -> Result<(), Vec<VerifyError>> {
    let mut errors = Vec::new();
    let mut defined: HashSet<ValueId> = HashSet::new();

    for block in &func.blocks {
        for &p in &block.params {
            if defined.contains(&p) {
                errors.push(VerifyError {
                    message: format!("value {} defined multiple times", p),
                });
            }
            defined.insert(p);
        }

        for instr in &block.instrs {
            match &instr.kind {
                InstructionKind::Const(_) => {}

                InstructionKind::Add(a, b)
                | InstructionKind::Sub(a, b)
                | InstructionKind::Mul(a, b)
                | InstructionKind::Div(a, b) => {
                    if !defined.contains(a) {
                        errors.push(VerifyError {
                            message: format!("use of undefined value {}", a),
                        });
                    }
                    if !defined.contains(b) {
                        errors.push(VerifyError {
                            message: format!("use of undefined value {}", b),
                        });
                    }
                }
            }

            if defined.contains(&instr.id) {
                errors.push(VerifyError {
                    message: format!("value {} redefined", instr.id),
                });
            }

            defined.insert(instr.id);
        }

        match &block.term {
            TerminatorKind::Ret(v) => {
                if let Some(v) = v && !defined.contains(v) {
                    errors.push(VerifyError {
                        message: format!("return uses undefined value {}", v),
                    });
                }
            }

            TerminatorKind::Br { target, params } => {
                let target_block = &func.blocks[*target as usize];

                if target_block.params.len() != params.len() {
                    errors.push(VerifyError {
                        message: "branch argument count mismatch".into(),
                    });
                }

                for p in params {
                    if !defined.contains(p) {
                        errors.push(VerifyError {
                            message: format!("branch uses undefined value {}", p),
                        });
                    }
                }
            }

            TerminatorKind::BrIf {
                cond,
                then_block,
                then_params,
                else_block,
                else_params,
            } => {
                if !defined.contains(cond) {
                    errors.push(VerifyError {
                        message: format!("BrIf uses undefined cond {}", cond),
                    });
                }

                let then_b = &func.blocks[*then_block as usize];
                if then_b.params.len() != then_params.len() {
                    errors.push(VerifyError {
                        message: "then branch param mismatch".into(),
                    });
                }

                if let (Some(else_b), Some(else_p)) = (else_block, else_params) {
                    let block = &func.blocks[*else_b as usize];
                    if block.params.len() != else_p.len() {
                        errors.push(VerifyError {
                            message: "else branch param mismatch".into(),
                        });
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl FunctionDef {
    pub fn verify(&self) -> Result<(), Vec<VerifyError>> {
        verify_function(self)
    }
}
