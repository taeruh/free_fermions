use std::fmt::{self, Debug};

use crate::graph::Node;

pub struct Matrix {
    data: Vec<Node>,
    dims: (usize, usize),
}

impl Matrix {
    pub fn from_vec_with_shape(vec: Vec<Node>, shape: (usize, usize)) -> Self {
        assert!(vec.len() == shape.0 * shape.1, "vec.len() != shape.0 * shape.1");
        Self { data: vec, dims: shape }
    }

    fn idx(&self, i: usize, j: usize) -> usize {
        i * self.dims.1 + j
    }

    pub fn get(&self, i: usize, j: usize) -> &Node {
        self.data.get(self.idx(i, j)).unwrap()
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut Node {
        let idx = self.idx(i, j);
        self.data.get_mut(idx).unwrap()
    }

    /// # Safety
    pub unsafe fn get_unchecked(&self, i: usize, j: usize) -> &Node {
        unsafe { self.data.get_unchecked(self.idx(i, j)) }
    }

    /// # Safety
    pub unsafe fn get_mut_unchecked(&mut self, i: usize, j: usize) -> &mut Node {
        let idx = self.idx(i, j);
        unsafe { self.data.get_unchecked_mut(idx) }
    }

    pub fn row(&self, i: usize) -> &[Node] {
        let start = self.idx(i, 0);
        let end = start + self.dims.1;
        &self.data[start..end]
    }

    pub fn diag_cube(&self) -> Vec<Node> {
        assert_eq!(self.dims.0, self.dims.1);
        let dim = self.dims.0;

        let mut diag = vec![0; dim];

        // we checked the dimensions above
        unsafe {
            // diag_i = sum_k sum_j a_ik * a_kj * a_ji
            for (i, d) in diag.iter_mut().enumerate() {
                for k in 0..dim {
                    let self_ik = *self.get_unchecked(i, k);
                    for j in 0..dim {
                        *d += self_ik
                            * *self.get_unchecked(k, j)
                            * *self.get_unchecked(j, i);
                    }
                }
            }
        }

        diag
    }
}

impl Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} x {} Matrix: [", self.dims.0, self.dims.1)?;
        for i in 0..self.dims.0 - 1 {
            writeln!(f, "{:?}", self.row(i))?;
        }
        write!(f, "{:?} ]", self.row(self.dims.0 - 1))
    }
}
