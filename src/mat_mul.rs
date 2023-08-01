#![allow(unused)]

// imagine a bit vector
#[derive(Debug)]
struct Array(Vec<bool>);

// imagine a simd vector
#[derive(Debug)]
struct Chunk([bool; 4]);

#[derive(Debug)]
struct Matrix {
    data: Array,
    dims: (usize, usize),
}

impl Array {
    fn with_len(mut len: usize) -> Self {
        unsafe { Self::with_len_raw(Self::correct_len(len)) }
    }

    fn correct_len(mut len: usize) -> usize {
        let past = len % 4;
        if past == 0 { len } else { len + 4 - past }
    }

    unsafe fn with_len_raw(len: usize) -> Self {
        Self(vec![false; len])
    }

    /// # Safety
    /// `idx` must be *strictly* less than `self.0.len() - 4`.
    unsafe fn load_chunk(&self, idx: usize) -> &Chunk {
        unsafe { &*(self.0.as_ptr().add(idx) as *const Chunk) }
    }

    /// # Safety ...
    unsafe fn load_chunk_mut(&mut self, idx: usize) -> &mut Chunk {
        unsafe { &mut *(self.0.as_mut_ptr().add(idx) as *mut Chunk) }
    }

    /// # Safety ...
    unsafe fn copy_chunk(&self, idx: usize) -> Chunk {
        unsafe { ptr::read(self.0.as_ptr().add(idx) as *const Chunk) }
    }
}

// imagine simd operations
impl Chunk {
    fn mul(&mut self, val: bool) {
        for e in self.0.iter_mut() {
            *e &= val;
        }
    }

    fn add(&mut self, other: &Chunk) {
        for (e, o) in self.0.iter_mut().zip(other.0.iter()) {
            *e ^= *o;
        }
    }
}

impl Matrix {
    fn with_dims(dims: (usize, usize)) -> Self {
        let dims_1 = Array::correct_len(dims.1);
        Self {
            data: unsafe { Array::with_len_raw(dims.0 * dims_1) },
            dims: (dims.0, dims_1),
        }
    }

    fn idx(&self, i: usize, j: usize) -> usize {
        i * self.dims.1 + j
    }

    fn chunk_idx(&self, i: usize, j: usize) -> usize {
        i * self.dims.1 + j * 4
    }

    fn get(&self, i: usize, j: usize) -> &bool {
        self.data.0.get(self.idx(i, j)).unwrap()
    }

    fn get_mut(&mut self, i: usize, j: usize) -> &mut bool {
        let idx = self.idx(i, j);
        self.data.0.get_mut(idx).unwrap()
    }

    /// # Safety
    /// `j` must be *strictly* less than `self.dims.1 / 4`.
    unsafe fn load_chunk(&self, i: usize, j: usize) -> &Chunk {
        unsafe { self.data.load_chunk(self.chunk_idx(i, j)) }
    }

    /// # Safety ...
    unsafe fn copy_chunk(&self, i: usize, j: usize) -> Chunk {
        unsafe { self.data.copy_chunk(self.chunk_idx(i, j)) }
    }

    /// # Safety ...
    unsafe fn load_chunk_mut(&mut self, i: usize, j: usize) -> &mut Chunk {
        unsafe { self.data.load_chunk_mut(self.chunk_idx(i, j)) }
    }

    fn row(&self, i: usize) -> &[bool] {
        let start = self.idx(i, 0);
        let end = start + self.dims.1;
        &self.data.0[start..end]
    }

    fn complement(&mut self) {
        for (i, e) in self.data.0.iter_mut().enumerate() {
            *e ^= true;
        }
        for i in 0..self.dims.0 {
            self.data.0[i * self.dims.1 + i] = false;
        }
    }
}

fn a_mut_b(a: &Matrix, b: &Matrix, c: &mut Matrix) {
    assert_eq!(c.dims.0, a.dims.0);
    assert_eq!(c.dims.1, b.dims.1);
    assert_eq!(a.dims.1, b.dims.0);

    // c_ij = sum_k a_ik * b_kj; swap j-loop with k-loop for less cache misses and simd
    for i in 0..c.dims.0 {
        // instead of reloading c's chunks in row i every time in the j-loop, whan could
        // save them here (the loading itself is cheap, after compiler optimizations,
        // but the calculation of idx is redundant -> simpler to just store the
        // calculation of idx here)
        for k in 0..a.dims.1 {
            let a_val_ik = *a.get(i, k);
            for jc in 0..(c.dims.1 / 4) {
                let mut b_chunk_kjc = unsafe { b.copy_chunk(k, jc) };
                b_chunk_kjc.mul(a_val_ik);
                let c_chunk_ijc = unsafe { c.load_chunk_mut(i, jc) };
                c_chunk_ijc.add(&b_chunk_kjc);
            }
        }
    }
}

fn cub_diag(a: &Matrix) -> Vec<bool> {
    assert_eq!(a.dims.0, a.dims.1);
    let dim = a.dims.0;

    let mut diagonal = vec![false; dim];

    // diagonal_i = sum_k sum_j a_ik * a_kj * a_ji
    for (i, d) in diagonal.iter_mut().enumerate() {
        for k in 0..dim {
            let a_val_ik = *a.get(i, k);
            for jc in 0..(dim / 4) {
                let mut a_chunk_kjc = unsafe { a.copy_chunk(k, jc) };
                a_chunk_kjc.mul(a_val_ik);
                // so far the same as in `a_mut_b`; now, instead of adding to rows of a
                // temporary matrix, we directly take the row and multiply it with the
                // i-th column; this loop really hurts regarding cache misses
                for (j, e) in a_chunk_kjc.0.into_iter().enumerate() {
                    *d ^= e & *a.get(jc * 4 + j, i);
                }
            }
        }
    }

    diagonal
}

use std::ptr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube() {
        let mut a = Matrix::with_dims((8, 8));
        // *a.get_mut(0, 0) = true;
        *a.get_mut(0, 1) = true;
        *a.get_mut(0, 2) = true;
        *a.get_mut(0, 3) = true;
        *a.get_mut(1, 0) = true;
        *a.get_mut(2, 0) = true;
        *a.get_mut(3, 0) = true;

        // for i in 0..a.dims.0 {
        //     println!("{:?}", a.row(i));
        // }
        // println!();

        // function to get sub-adjacency-matrix

        let mut a_0 = Matrix::with_dims((4, 4));
        *a_0.get_mut(0, 1) = true;
        *a_0.get_mut(0, 2) = true;
        *a_0.get_mut(0, 3) = true;
        *a_0.get_mut(1, 0) = true;
        *a_0.get_mut(2, 0) = true;
        *a_0.get_mut(3, 0) = true;

        let mut a_1 = Matrix::with_dims((4, 4));
        *a_0.get_mut(0, 1) = true;
        *a_0.get_mut(1, 0) = true;

        a_0.complement();
        // println!("{:?}", cub_diag(&a_0));

        a_1.complement();
        // println!("{:?}", &cub_diag(&a_1)[..2]);
    }

    #[test]
    fn ab() {
        let mut a = Matrix::with_dims((2, 5));
        let mut b = Matrix::with_dims((8, 3));
        let mut c = Matrix::with_dims((2, 3));

        assert_eq!(a.dims, (2, 8));
        assert_eq!(b.dims, (8, 4));
        assert_eq!(c.dims, (2, 4));

        *a.get_mut(0, 0) = true;
        *a.get_mut(0, 5) = true;
        *b.get_mut(5, 3) = true;

        // for i in 0..a.dims.0 {
        //     println!("{:?}", a.row(i));
        // }
        // println!();
        // for i in 0..b.dims.0 {
        //     println!("{:?}", b.row(i));
        // }

        a_mut_b(&a, &b, &mut c);

        assert_eq!(c.row(0), &[false, false, false, true]);
        assert_eq!(c.row(1), &[false, false, false, false]);
    }
}
