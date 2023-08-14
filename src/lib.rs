#![deny(unsafe_op_in_unsafe_fn)]

macro_rules! non_semantic_default {
    () => {
        "Note that semantically, this impl makes not much sense. It is rather useful \
         for initialization."
    };
}

pub mod graph;
pub mod hamiltonian;
pub mod run;
pub mod mat_mul;

// pub mod mat_mul;
