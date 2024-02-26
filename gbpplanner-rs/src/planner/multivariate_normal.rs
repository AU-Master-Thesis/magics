// use nalgebra::{DMatrix, DVector};
use super::{Matrix, Scalar, Vector};
use ndarray_linalg::Inverse;
// use num::Float;

// trait MultivariateNormal {
//     fn mean(&self) -> &DVector<f64>;
//     fn covariance(&self) -> &DMatrix<f64>;
//     fn information_vector(&self) -> &DVector<f64> {
//         self.precision_matrix() * self.mean()
//     }
//     fn precision_matrix(&self) -> &DMatrix<f64> {
//         // self.covariance().cholesky().unwrap().inverse()
//         self.covariance().try_inverse().unwrap()
//     }
// }

// TODO: finish
// #[derive(Debug, thiserror::Error)]
// pub enum FromMeanAndCovarianceError {
//     NonSquareCovarianceMatrix { rows: usize, cols: usize },
//     MeanCovarianceDimensionMismatch,
// }
// pub struct MeanCovarianceDimensionMismatch

/// A multivariate normal distribution stored in the information form.
/// $ cal(N)(X; mu, Sigma) = cal(N)^(-1)(X; eta, Lambda) $,
/// where $Lambda = Sigma^(-1)$ and $eta = Lambda * mu$
/// $Lambda$ is called the precision matrix, and $eta$ is called the information vector.
/// The precision matrix is the inverse of the covariance matrix $Sigma$.
/// The information vector is the product of the precision matrix and the mean.
#[derive(Debug, Clone)]
pub struct MultivariateNormal<T: Scalar> {
    /// $eta = Lambda * mu$, where $Lambda$ is the precision matrix and $mu$ is the mean
    pub information_vector: Vector<T>,
    /// $Lambda = Sigma^(-1)$, where $Sigma$ is the covariance matrix
    pub precision_matrix: Matrix<T>,
    // mean: Vector<T>,
    // Consider including the following fields:
    //     gbpplanner includes the means as a computational optimization
    // pub mean: nalgebra::DVector<f32>,
    // pub covariance: nalgebra::DMatrix<f32>,
}

impl<T: Float> MultivariateNormal<T> {
    /// Create a default MultivariateNormal, initialized with zeros
    pub fn zeros(dim: usize) -> Self {
        MultivariateNormal {
            information_vector: Vector::<T>::zeros(dim),
            precision_matrix: Matrix::<T>::zeros((dim, dim)),
            // mean: Vector::<T>::zeros(dim),
        }
    }

    // TODO: return result
    /// Create a MultivariateNormal from a given information vector and precision matrix
    pub fn new(information_vector: Vector<T>, precision_matrix: Matrix<T>) -> Self {
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }

    pub fn from_mean_and_covariance(mean: Vector<T>, covariance: Matrix<T>) -> Self {
        assert_eq!(mean.len(), covariance.shape()[0]);
        assert_eq!(mean.len(), covariance.shape()[1]);
        let precision_matrix = covariance
            .inv()
            .expect("the covariance matrix should be nonsingular");
        let information_vector = &precision_matrix * mean;
        MultivariateNormal {
            information_vector,
            precision_matrix,
            // mean,
        }
    }

    // pub fn mean(&self) -> &Vector<T> {
    //     &self.mean
    // }
    pub fn mean(&self) -> Vector<T> {
        self.precision_matrix
            .inv()
            .expect("the precision matrix is invertible")
            .dot(&self.information_vector)
    }

    // pub fn mean(&self) -> Vector<f32> {
    //     self.precision_matrix.clone().try_inverse().unwrap() * &self.information_vector
    // }

    pub fn zeroize(&mut self) {
        self.information_vector.fill(0.0);
        self.precision_matrix.fill(0.0);
    }
}

impl std::ops::Add for MultivariateNormal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let information_vector = self.information_vector + other.information_vector;
        let precision_matrix = self.precision_matrix + other.precision_matrix;
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }
}

impl std::ops::AddAssign for MultivariateNormal {
    fn add_assign(&mut self, other: Self) {
        self.information_vector += other.information_vector;
        self.precision_matrix += other.precision_matrix;
    }
}

impl std::ops::Sub for MultivariateNormal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let information_vector = self.information_vector - other.information_vector;
        let precision_matrix = self.precision_matrix - other.precision_matrix;
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }
}

impl std::ops::Sub<&MultivariateNormal> for MultivariateNormal {
    type Output = Self;

    fn sub(self, other: &MultivariateNormal) -> Self {
        let information_vector = self.information_vector - &other.information_vector;
        let precision_matrix = self.precision_matrix - &other.precision_matrix;
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }
}

impl std::ops::SubAssign for MultivariateNormal {
    fn sub_assign(&mut self, other: Self) {
        self.information_vector -= other.information_vector;
        self.precision_matrix -= other.precision_matrix;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ndarray::prelude::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_zeros() {
        let n = 5;
        let mvn = MultivariateNormal::zeros(n);
        assert_eq!(mvn.information_vector.len(), n);
        assert_eq!(mvn.precision_matrix.shape(), (n, n));

        assert_eq!(mvn.information_vector, Vector::<f32>::zeros(n));
        assert_eq!(mvn.precision_matrix, Matrix::<f32>::zeros((n, n)));
    }

    #[test]
    fn test_zeroize() {
        let n = 4;
        let information_vector = array![1., 2., 3., 4.];
        let precision_matrix = array![
            [5.0, 0.0, 1.0, 0.5],
            [0.0, 5.0, 0.0, 0.0],
            [1.0, 0.0, 5.0, 0.2],
            [0.5, 0.0, 0.5, 5.0]
        ];
        let mut mvn = MultivariateNormal::new(information_vector, precision_matrix);

        assert!(!mvn.information_vector.iter().all(|x| *x == 0.0));
        assert!(!mvn.precision_matrix.iter().all(|x| *x == 0.0));

        mvn.zeroize();

        assert!(mvn.information_vector.iter().all(|x| *x == 0.0));
        assert!(mvn.precision_matrix.iter().all(|x| *x == 0.0));
    }

    #[test]
    fn test_addition() {
        let n = 3;
        let mvn0 = MultivariateNormal::new(array![1., 2., 3.], Matrix::<f32>::eye(n));
        let mvn1 =
            MultivariateNormal::new(na::dvector![6., 5., 4.], 3. * Matrix::<f32>::eye(n));

        let mvn2 = &mvn0 + &mvn1;

        assert_eq!(mvn2.information_vector, array![7., 7., 7.]);
        assert_eq!(
            mvn2.precision_matrix,
            array![[4., 0., 0.], [0., 4., 0.], [0., 0., 4.]]
        );
    }

    #[test]
    fn test_substraction() {
        let n = 3;
        let mvn0 = MultivariateNormal::new(array![1., 2., 3.], Matrix::<f32>::eye(n));
        let mvn1 =
            MultivariateNormal::new(array![6., 5., 4.], 3. * Matrix::<f32>::eye(n));

        let mvn2 = &mvn0 + &mvn1;

        assert_eq!(mvn2.information_vector, array![-5., -3., -1.]);
        assert_eq!(
            mvn2.precision_matrix,
            array![[-2., 0., 0.], [0., -2., 0.], [0., 0., -2.]]
        );
    }

    // TODO: write unit tests for the rest of the methods
}
