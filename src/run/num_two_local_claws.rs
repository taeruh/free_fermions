use crate::hamiltonian::Pauli;

type LocalOperator = crate::hamiltonian::LocalOperator<2, Pauli>;

#[allow(dead_code)]
pub fn run() {
    let _center = LocalOperator {
        index: [1, 2],
        operator_at_index: [Pauli::X, Pauli::X],
    };
}
