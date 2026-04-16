use crate::ir::*;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseKind {
    Regular,
    Return,
    BranchCond,
    BlockArg,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub user: ValueId,
    pub operand_index: u8,
    pub kind: UseKind,
}

#[derive(Debug, Default)]
pub struct DataFlowGraph {
    pub uses: HashMap<ValueId, Vec<Use>>,
    pub defined_values: HashSet<ValueId>,
    pub def_map: HashMap<ValueId, ValueId>,
}

impl DataFlowGraph {
    pub fn build(func: &FunctionDef) -> Self {
        let mut uses = HashMap::new();
        let mut defined_values = HashSet::new();
        let mut def_map = HashMap::new();

        for block in &func.blocks {
            for instr in &block.instrs {
                let result_id = instr.id;

                defined_values.insert(result_id);
                def_map.insert(result_id, result_id);

                match &instr.kind {
                    InstructionKind::Const(_) => {}

                    InstructionKind::Add(a, b)
                    | InstructionKind::Sub(a, b)
                    | InstructionKind::Mul(a, b)
                    | InstructionKind::Div(a, b) => {
                        Self::add_use(&mut uses, *a, result_id, 0, UseKind::Regular);
                        Self::add_use(&mut uses, *b, result_id, 1, UseKind::Regular);
                    }
                }
            }

            match &block.term {
                TerminatorKind::Ret(Some(val)) => {
                    Self::add_use(&mut uses, *val, block.id, 0, UseKind::Return);
                }

                TerminatorKind::BrIf {
                    cond,
                    then_params,
                    else_params,
                    ..
                } => {
                    Self::add_use(&mut uses, *cond, block.id, 0, UseKind::BranchCond);

                    for (i, &param) in then_params.iter().enumerate() {
                        Self::add_use(&mut uses, param, block.id, (i + 1) as u8, UseKind::BlockArg);
                    }

                    if let Some(else_params) = else_params {
                        for (i, &param) in else_params.iter().enumerate() {
                            Self::add_use(
                                &mut uses,
                                param,
                                block.id,
                                (i + 100) as u8,
                                UseKind::BlockArg,
                            );
                        }
                    }
                }

                TerminatorKind::Br { params, .. } => {
                    for (i, &param) in params.iter().enumerate() {
                        Self::add_use(&mut uses, param, block.id, (i + 1) as u8, UseKind::BlockArg);
                    }
                }
                TerminatorKind::Ret(None) => {}
            }
        }

        for &v in &defined_values {
            uses.entry(v).or_default();
        }

        DataFlowGraph {
            uses,
            defined_values,
            def_map,
        }
    }

    fn add_use(
        uses: &mut HashMap<ValueId, Vec<Use>>,
        value: ValueId,
        user: ValueId,
        operand_index: u8,
        kind: UseKind,
    ) {
        uses.entry(value).or_default().push(Use {
            user,
            operand_index,
            kind,
        });
    }

    pub fn get_uses(&self, value: ValueId) -> &[Use] {
        self.uses.get(&value).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn is_defined(&self, value: ValueId) -> bool {
        self.defined_values.contains(&value)
    }

    pub fn is_used(&self, value: ValueId) -> bool {
        !self.get_uses(value).is_empty()
    }

    pub fn unused_values(&self) -> Vec<ValueId> {
        self.defined_values
            .iter()
            .copied()
            .filter(|&v| self.get_uses(v).is_empty())
            .collect()
    }

    pub fn uses_of_kind(&self, value: ValueId, kind: UseKind) -> Vec<&Use> {
        self.get_uses(value)
            .iter()
            .filter(|u| u.kind == kind)
            .collect()
    }
}
