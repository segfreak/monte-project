use std::collections::{HashMap, HashSet};

use crate::repr::{BlockId, FunctionDef, TerminatorKind};

#[derive(Debug, Clone)]
pub struct ControlFlowNode {
    pub block_id: BlockId,
    pub predecessors: Vec<BlockId>,
    pub successors: Vec<BlockId>,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub nodes: HashMap<BlockId, ControlFlowNode>,
    pub entry: BlockId,
}

impl ControlFlowGraph {
    pub fn build(func: &FunctionDef) -> Self {
        let mut nodes = HashMap::new();
        let mut successors_map: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

        for (idx, block) in func.blocks.iter().enumerate() {
            let block_id = idx as BlockId;
            let successors = Self::get_successors(&block.term);
            successors_map.insert(block_id, successors);
        }

        let mut predecessors_map: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for (block_id, succs) in &successors_map {
            for succ in succs {
                predecessors_map.entry(*succ).or_default().push(*block_id);
            }
        }

        for (block_id, successors) in successors_map {
            let predecessors = predecessors_map.get(&block_id).cloned().unwrap_or_default();

            nodes.insert(
                block_id,
                ControlFlowNode {
                    block_id,
                    predecessors,
                    successors,
                },
            );
        }

        ControlFlowGraph {
            nodes,
            entry: func.entry,
        }
    }

    fn get_successors(term: &TerminatorKind) -> Vec<BlockId> {
        match term {
            TerminatorKind::Ret { .. } => Vec::new(),
            TerminatorKind::Br { block, .. } => vec![*block],
            TerminatorKind::BrIf {
                then_block,
                else_block,
                ..
            } => match else_block {
                Some(b) => vec![*then_block, *b],
                None => vec![*then_block],
            },
        }
    }

    pub fn compute_dominators(&self) -> HashMap<BlockId, HashSet<BlockId>> {
        let mut dominators: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let all_blocks: HashSet<BlockId> = self.nodes.keys().cloned().collect();

        dominators.insert(self.entry, [self.entry].iter().cloned().collect());

        for block_id in all_blocks.iter() {
            if *block_id != self.entry {
                dominators.insert(*block_id, all_blocks.clone());
            }
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block_id in all_blocks.iter() {
                if *block_id == self.entry {
                    continue;
                }

                let node = &self.nodes[block_id];

                // Dom(n) = {n} ∪ (∩ Dom(p) for all predecessors p)
                let mut new_dom = if node.predecessors.is_empty() {
                    HashSet::new()
                } else {
                    let mut intersection = dominators[&node.predecessors[0]].clone();
                    for pred in &node.predecessors[1..] {
                        intersection = intersection
                            .intersection(&dominators[pred])
                            .cloned()
                            .collect();
                    }
                    intersection
                };

                new_dom.insert(*block_id);

                if new_dom != dominators[block_id] {
                    dominators.insert(*block_id, new_dom);
                    changed = true;
                }
            }
        }

        dominators
    }

    /// Generate DOT format for visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph ControlFlow {\n");
        dot.push_str("  node [shape=box];\n");

        for (block_id, node) in &self.nodes {
            if *block_id == self.entry {
                dot.push_str(&format!(
                    "  {} [label=\"{} (entry)\", style=filled, fillcolor=lightblue];\n",
                    block_id, block_id
                ));
            } else {
                dot.push_str(&format!("  {} [label=\"{}\"];\n", block_id, block_id));
            }

            for succ in &node.successors {
                dot.push_str(&format!("  {} -> {};\n", block_id, succ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    #[inline]
    pub fn node(&self, b: BlockId) -> &ControlFlowNode {
        &self.nodes[&b]
    }

    #[inline]
    pub fn successors(&self, b: BlockId) -> &[BlockId] {
        &self.nodes[&b].successors
    }

    #[inline]
    pub fn predecessors(&self, b: BlockId) -> &[BlockId] {
        &self.nodes[&b].predecessors
    }

    pub fn postorder(&self, id: BlockId) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut out = Vec::new();
        self.post_dfs(id, &mut visited, &mut out);
        out
    }

    fn post_dfs(&self, b: BlockId, visited: &mut HashSet<BlockId>, out: &mut Vec<BlockId>) {
        if !visited.insert(b) {
            return;
        }
        for &s in &self.nodes[&b].successors {
            self.post_dfs(s, visited, out);
        }
        out.push(b);
    }

    pub fn reverse_postorder(&self, id: BlockId) -> Vec<BlockId> {
        let mut v = self.postorder(id);
        v.reverse();
        v
    }

    pub fn dominates(dom: &HashMap<BlockId, HashSet<BlockId>>, a: BlockId, b: BlockId) -> bool {
        dom.get(&b).map_or(false, |s| s.contains(&a))
    }
}
