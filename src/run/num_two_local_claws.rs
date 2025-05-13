use crate::hamiltonian::Pauli;


type LocalOperator = crate::hamiltonian::LocalOperator<2, Pauli>;

pub fn run() {
    let center = LocalOperator {
        index: [1, 2],
        pauli: [Pauli::X, Pauli::X],
    };
}
