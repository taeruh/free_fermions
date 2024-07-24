use crate::graph::{Label, Node, VLabels, VNodes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Obstinate {
    True(ObstinateKind, (VNodes, VNodes)),
    False,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObstinateMapped {
    True(ObstinateKind, (VLabels, VLabels)),
    False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstinateKind {
    Itself,
    Complement,
}

impl Obstinate {
    pub fn map(&self, map: impl Fn(Node) -> Label) -> ObstinateMapped {
        match self {
            Obstinate::True(kind, (a, b)) => ObstinateMapped::True(
                *kind,
                (
                    a.iter().map(|&n| map(n)).collect(),
                    b.iter().map(|&n| map(n)).collect(),
                ),
            ),
            Obstinate::False => ObstinateMapped::False,
        }
    }
}
