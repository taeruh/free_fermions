use crate::graph::VNodes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Obstinate {
    True(ObstinateKind, (VNodes, VNodes)),
    False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstinateKind {
    Itself,
    Complement,
}
