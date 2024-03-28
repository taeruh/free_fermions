// This matrix implementation is as simple as possible, without using any shortcuts or
// tricks. We only care about correctness and not at all about performance. Essentially,
// we just use the ndarray crate and call basic methods.

use ndarray::{linalg::Dot, Array2, LinalgScalar};

use super::MatrixTools;

impl<T> MatrixTools<T> for Array2<T>
where
    T: LinalgScalar,
{
    type Error = ndarray::ShapeError;

    fn from_vec_with_shape(
        vec: Vec<T>,
        shape: (usize, usize),
    ) -> Result<Self, Self::Error> {
        Array2::from_shape_vec(shape, vec)
    }

    fn diagonal_of_cubed(&self) -> Vec<T>
    where
        Self: Dot<Self, Output = Self>,
    {
        let dims = self.shape();
        assert_eq!(dims[0], dims[1]);
        let cubed = self.dot(self).dot(self);
        cubed.diag().to_vec()
    }
}
