// different implementations for the matrix operations we need

pub trait MatrixTools<T> {
    type Error;

    fn from_vec_with_shape(
        vec: Vec<T>,
        shape: (usize, usize),
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn diag_cube(&self) -> Vec<T>;
}

pub mod ndarray;
