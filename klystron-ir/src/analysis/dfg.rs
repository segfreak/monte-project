use std::collections::{HashMap, HashSet};

use crate::repr::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseSite {
    Instr(ValueId),
    Term(BlockId),
}

#[derive(Debug, Clone)]
pub struct Use {
    pub site: UseSite,
    pub operand_index: u8,
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Inst(InstructionKind),
    BlockParam { block: BlockId, index: usize },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub def: DefKind,
    pub uses: Vec<Use>,
    pub used_by: Vec<ValueId>,
}

#[derive(Debug, Default)]
pub struct DataFlowGraph {
    pub nodes: HashMap<ValueId, Node>,
    pub def_block: HashMap<ValueId, BlockId>,
}

impl DataFlowGraph {
    pub fn build(func: &FunctionDef) -> Self {
        let mut nodes: HashMap<ValueId, Node> = HashMap::new();
        let mut def_block: HashMap<ValueId, BlockId> = HashMap::new();

        for (bidx, block) in func.blocks.iter().enumerate() {
            let block_id = bidx as BlockId;

            for (i, &p) in block.params.iter().enumerate() {
                def_block.insert(p, block_id);

                nodes.insert(
                    p,
                    Node {
                        def: DefKind::BlockParam {
                            block: block_id,
                            index: i,
                        },
                        uses: Vec::new(),
                        used_by: Vec::new(),
                    },
                );
            }

            for instr in &block.instrs {
                let id = instr.id;

                def_block.insert(id, block_id);

                let mut operands: Vec<(ValueId, u8)> = Vec::new();

                match &instr.kind {
                    InstructionKind::Const(_) => {}

                    InstructionKind::Add(a, b)
                    | InstructionKind::Sub(a, b)
                    | InstructionKind::Mul(a, b)
                    | InstructionKind::Div(a, b) => {
                        operands.push((*a, 0));
                        operands.push((*b, 1));
                    }

                    InstructionKind::Call(_, args) => {
                        for (i, a) in args.iter().enumerate() {
                            operands.push((*a, i as u8));
                        }
                    }
                }

                nodes.insert(
                    id,
                    Node {
                        def: DefKind::Inst(instr.kind.clone()),
                        uses: Vec::new(),
                        used_by: Vec::new(),
                    },
                );

                for (v, idx) in operands {
                    let operand_node = nodes.entry(v).or_insert_with(|| Node {
                        def: DefKind::Inst(InstructionKind::Const(Constant::Bool(false))),
                        uses: Vec::new(),
                        used_by: Vec::new(),
                    });

                    operand_node.uses.push(Use {
                        site: UseSite::Instr(id),
                        operand_index: idx,
                    });

                    operand_node.used_by.push(id);
                }
            }

            match &block.term {
                TerminatorKind::Ret(v) => {
                    if let Some(v) = v {
                        Self::add_term_use(&mut nodes, *v, block_id, 0);
                    }
                }

                TerminatorKind::Br { params, .. } => {
                    for (i, p) in params.iter().enumerate() {
                        Self::add_term_use(&mut nodes, *p, block_id, i as u8);
                    }
                }

                TerminatorKind::BrIf {
                    cond,
                    then_params,
                    else_params,
                    ..
                } => {
                    Self::add_term_use(&mut nodes, *cond, block_id, 0);

                    for (i, p) in then_params.iter().enumerate() {
                        Self::add_term_use(&mut nodes, *p, block_id, (i + 1) as u8);
                    }

                    if let Some(ep) = else_params {
                        for (i, p) in ep.iter().enumerate() {
                            Self::add_term_use(&mut nodes, *p, block_id, (i + 100) as u8);
                        }
                    }
                }
            }
        }

        Self { nodes, def_block }
    }

    fn add_term_use(nodes: &mut HashMap<ValueId, Node>, v: ValueId, block: BlockId, idx: u8) {
        let node = nodes.entry(v).or_insert_with(|| Node {
            def: DefKind::Inst(InstructionKind::Const(Constant::Bool(false))),
            uses: Vec::new(),
            used_by: Vec::new(),
        });

        node.uses.push(Use {
            site: UseSite::Term(block),
            operand_index: idx,
        });
    }

    pub fn uses(&self, v: ValueId) -> &[Use] {
        self.nodes.get(&v).map(|n| n.uses.as_slice()).unwrap_or(&[])
    }

    pub fn used_by(&self, v: ValueId) -> &[ValueId] {
        self.nodes
            .get(&v)
            .map(|n| n.used_by.as_slice())
            .unwrap_or(&[])
    }

    pub fn def_block(&self, v: ValueId) -> Option<BlockId> {
        self.def_block.get(&v).copied()
    }

    pub fn defs_in_block(&self, b: BlockId) -> impl Iterator<Item = ValueId> + '_ {
        self.def_block
            .iter()
            .filter(move |(_, blk)| **blk == b)
            .map(|(v, _)| *v)
    }

    pub fn uses_in_block(&self, b: BlockId) -> HashSet<ValueId> {
        let mut s = HashSet::new();

        for (v, node) in &self.nodes {
            for u in &node.uses {
                let use_block = match u.site {
                    UseSite::Instr(consumer_id) => self.def_block.get(&consumer_id).copied(),
                    UseSite::Term(blk) => Some(blk),
                };

                if use_block == Some(b) {
                    s.insert(*v);
                    break;
                }
            }
        }

        s
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph DataFlow {\n");
        dot.push_str("  node [shape=ellipse];\n");

        for (v, node) in &self.nodes {
            match &node.def {
                DefKind::Inst(inst) => {
                    let label = format!("v{}: {:?}", v, inst);
                    dot.push_str(&format!(
                        "  \"v{}\" [label=\"{}\"];\n",
                        v,
                        label.replace("\"", "\\\"")
                    ));

                    for u in &node.uses {
                        let consumer = match u.site {
                            UseSite::Instr(x) => format!("v{}", x),
                            UseSite::Term(b) => format!("block{}_term", b),
                        };

                        dot.push_str(&format!("  \"v{}\" -> \"{}\";\n", v, consumer));
                    }
                }

                DefKind::BlockParam { block, index } => {
                    let label = format!("v{}: param b{}[{}]", v, block, index);

                    dot.push_str(&format!(
                        "  \"v{}\" [label=\"{}\", shape=box, style=filled, fillcolor=lightgray];\n",
                        v,
                        label.replace("\"", "\\\"")
                    ));
                }
            }
        }

        dot.push_str("}\n");
        dot
    }
}
