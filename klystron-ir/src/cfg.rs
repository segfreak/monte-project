use std::collections::{HashMap, HashSet};

use crate::ir::*;

#[derive(Debug, Clone)]
pub struct CfgEdge {
    pub from: BlockId,
    pub to: BlockId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeKind {
    Unconditional,
    True,
    False,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub successors: HashMap<BlockId, Vec<(BlockId, EdgeKind)>>,
    pub predecessors: HashMap<BlockId, Vec<BlockId>>,
    pub entry: BlockId,
}

impl ControlFlowGraph {
    pub fn build(func: &FunctionDef) -> Self {
        let mut successors: HashMap<ValueId, Vec<(BlockId, EdgeKind)>> = HashMap::new();
        let mut predecessors: HashMap<ValueId, Vec<BlockId>> = HashMap::new();

        for block in &func.blocks {
            let mut succ = Vec::new();

            match &block.term {
                TerminatorKind::Ret(_) => {}

                TerminatorKind::Br { target, .. } => {
                    succ.push((*target, EdgeKind::Unconditional));
                }

                TerminatorKind::BrIf {
                    then_block,
                    else_block,
                    ..
                } => {
                    succ.push((*then_block, EdgeKind::True));
                    if let Some(else_b) = else_block {
                        succ.push((*else_b, EdgeKind::False));
                    }
                }
            }

            successors.insert(block.id, succ.clone());

            for (target, _) in succ {
                predecessors.entry(target).or_default().push(block.id);
            }
        }

        ControlFlowGraph {
            successors,
            predecessors,
            entry: func.entry,
        }
    }

    pub fn successors(&self, block: BlockId) -> &[(BlockId, EdgeKind)] {
        self.successors
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn predecessors(&self, block: BlockId) -> &[BlockId] {
        self.predecessors
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_reachable(&self, from: BlockId, to: BlockId) -> bool {
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                return true;
            }
            for &(succ, _) in self.successors(current) {
                if visited.insert(succ) {
                    queue.push_back(succ);
                }
            }
        }
        false
    }
}
