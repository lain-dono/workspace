use crate::graph_impl as graph;
use crate::repr;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::mem;

pub struct FuncGraph {
    func: repr::Func,
    start_block: BlockIndex,
    blocks: Vec<BlockKind>,
    successors: Vec<Vec<BlockIndex>>,
    predecessors: Vec<Vec<BlockIndex>>,
    block_indices: BTreeMap<repr::BlockName, BlockIndex>,
    skolemized_end_indices: BTreeMap<repr::RegionName, BlockIndex>,
    skolemized_end_actions: BTreeMap<repr::RegionName, [repr::Action; 1]>,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct BlockIndex {
    index: usize,
}

#[derive(Clone, Debug)]
pub enum BlockKind {
    Code(repr::BlockName),
    SkolemizedEnd(repr::RegionName),
}

#[derive(Copy, Clone, Debug)]
pub enum BasicBlockData<'a> {
    Code(&'a repr::BlockDecl),
    SkolemizedEnd(&'a [repr::Action]),
}

impl FuncGraph {
    pub fn new(func: repr::Func) -> Self {
        let blocks: Vec<_> = func
            .data
            .keys()
            .map(|&block| BlockKind::Code(block))
            .chain(
                func.regions
                    .iter()
                    .map(|rd| BlockKind::SkolemizedEnd(rd.name)),
            )
            .collect();

        let block_indices: BTreeMap<_, _> = func
            .data
            .keys()
            .copied()
            .enumerate()
            .map(|(index, block)| (block, BlockIndex { index }))
            .collect();

        let skolemized_end_indices: BTreeMap<_, _> = func
            .regions
            .iter()
            .enumerate()
            .map(|(index, rd)| {
                let index = index + block_indices.len();
                (rd.name, BlockIndex { index })
            })
            .collect();

        let skolemized_end_actions: BTreeMap<_, _> = func
            .regions
            .iter()
            .map(|rd| {
                let action = repr::Action {
                    kind: repr::ActionKind::SkolemizedEnd(rd.name),
                    should_have_error: None,
                };
                (rd.name, [action])
            })
            .collect();

        let mut predecessors: Vec<_> = (0..blocks.len()).map(|_| Vec::new()).collect();
        let mut successors: Vec<_> = (0..blocks.len()).map(|_| Vec::new()).collect();

        for (block, &index) in &block_indices {
            let data = &func.data[block];
            for successor in &data.successors {
                let successor_index = block_indices
                    .get(successor)
                    .copied()
                    .unwrap_or_else(|| panic!("no index for {successor:?}"));
                successors[index.index].push(successor_index);
                predecessors[successor_index.index].push(index);
            }
        }

        let start_block = block_indices[&repr::BlockName::START];

        Self {
            func,
            start_block,
            blocks,
            successors,
            predecessors,
            block_indices,
            skolemized_end_indices,
            skolemized_end_actions,
        }
    }

    pub fn block(&self, name: repr::BlockName) -> BlockIndex {
        self.block_indices[&name]
    }

    pub fn skolemized_end(&self, name: repr::RegionName) -> BlockIndex {
        self.skolemized_end_indices[&name]
    }

    pub fn block_data(&self, index: BlockIndex) -> BasicBlockData {
        match &self.blocks[index.index] {
            BlockKind::Code(block) => BasicBlockData::Code(&self.func.data[block]),
            BlockKind::SkolemizedEnd(r) => {
                BasicBlockData::SkolemizedEnd(&self.skolemized_end_actions[r])
            }
        }
    }

    pub fn free_regions(&self) -> &[repr::RegionDecl] {
        &self.func.regions
    }

    pub fn decls(&self) -> &[repr::VariableDecl] {
        &self.func.decls
    }

    pub fn assertions(&self) -> &[repr::Assertion] {
        &self.func.assertions
    }

    pub fn struct_decls(&self) -> &[repr::StructDecl] {
        &self.func.structs
    }
}

impl graph::Graph for FuncGraph {
    type Node = BlockIndex;

    fn len(&self) -> usize {
        self.blocks.len()
    }

    fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    fn start(&self) -> Self::Node {
        self.start_block
    }

    fn predecessors(&self, node: BlockIndex) -> impl Iterator<Item = Self::Node> {
        self.predecessors[node.index].iter().copied()
    }

    fn successors(&self, node: BlockIndex) -> impl Iterator<Item = Self::Node> {
        self.successors[node.index].iter().copied()
    }
}

impl From<usize> for BlockIndex {
    fn from(v: usize) -> Self {
        Self { index: v }
    }
}

impl From<BlockIndex> for usize {
    fn from(val: BlockIndex) -> Self {
        val.index
    }
}

thread_local! {
    static NAMES: RefCell<Vec<BlockKind>> = const { RefCell::new(vec![]) }
}

pub fn with_graph<R>(g: &FuncGraph, op: impl FnOnce() -> R) -> R {
    NAMES.with(|names| {
        let old_names = mem::replace(&mut *names.borrow_mut(), g.blocks.clone());
        let result = op();
        *names.borrow_mut() = old_names;
        result
    })
}

impl fmt::Debug for BlockIndex {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        NAMES.with(|names| {
            let names = names.borrow();
            if names.is_empty() {
                write!(fmt, "BB{}", self.index)
            } else {
                match &names[self.index] {
                    BlockKind::Code(bb) => write!(fmt, "{bb}"),
                    BlockKind::SkolemizedEnd(rn) => write!(fmt, "{rn}"),
                }
            }
        })
    }
}

impl<'a> BasicBlockData<'a> {
    pub fn actions(self) -> &'a [repr::Action] {
        match self {
            Self::Code(d) => &d.actions,
            Self::SkolemizedEnd(actions) => actions,
        }
    }
}
