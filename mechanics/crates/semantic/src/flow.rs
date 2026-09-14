/// Indicates that an instruction has a branching effect in the target architecture.
pub enum Jump {
    /// The instruction calls another function.
    Call,
    /// The instruction returns from the current function.
    Return,
    /// The instruction jumps to another basic block in the function unconditionally.
    Unconditional,
    /// The instruction potentially jumps to another basic block or moves to next instruction.
    Conditional,
}

struct FlowGraph {
    nodes: Vec<()>,

    edges: Vec<(Jump, usize, usize)>,
}
