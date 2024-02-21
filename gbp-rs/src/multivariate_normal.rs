use nalgebra::{DMatrix, DVector};

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

/// A multivariate normal distribution stored in the information form.
/// $ cal(N)(X; mu, Sigma) = cal(N)^(-1)(X; eta, Lambda) $,
/// where $Lambda = Sigma^(-1)$ and $eta = Lambda * mu$
/// $Lambda$ is called the precision matrix, and $eta$ is called the information vector.
/// The precision matrix is the inverse of the covariance matrix $Sigma$.
/// The information vector is the product of the precision matrix and the mean.
#[derive(Debug, Clone)]
pub struct MultivariateNormal {
    /// $eta = Lambda * mu$, where $Lambda$ is the precision matrix and $mu$ is the mean
    pub information_vector: DVector<f32>,
    /// $Lambda = Sigma^(-1)$, where $Sigma$ is the covariance matrix
    pub precision_matrix: DMatrix<f32>,
    // Consider including the following fields:
    //     gbpplanner includes the means as a computational optimization
    // pub mean: nalgebra::DVector<f32>,
    // pub covariance: nalgebra::DMatrix<f32>,
}

impl MultivariateNormal {
    /// Create a default MultivariateNormal, initialized with zeros
    pub fn zeros(dim: usize) -> Self {
        MultivariateNormal {
            information_vector: DVector::zeros(dim),
            precision_matrix: DMatrix::zeros(dim, dim),
        }
    }

    /// Create a MultivariateNormal from a given information vector and precision matrix
    pub fn new(information_vector: DVector<f32>, precision_matrix: DMatrix<f32>) -> Self {
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }

    pub fn from_mean_and_covariance(
        mean: DVector<f32>,
        covariance: DMatrix<f32>,
    ) -> Self {
        assert_eq!(mean.nrows(), covariance.nrows());
        assert_eq!(mean.nrows(), covariance.ncols());
        let precision_matrix = covariance
            .try_inverse()
            .expect("Covariance matrix should be nonsingular");
        let information_vector = &precision_matrix * mean;
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }

    pub fn mean(&self) -> DVector<f32> {
        self.precision_matrix.try_inverse().unwrap() * &self.information_vector
    }

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

impl std::ops::SubAssign for MultivariateNormal {
    fn sub_assign(&mut self, other: Self) {
        self.information_vector -= other.information_vector;
        self.precision_matrix -= other.precision_matrix;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use nalgebra as na;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_zeros() {
        let n = 5;
        let mvn = MultivariateNormal::zeros(n);
        assert_eq!(mvn.information_vector.len(), n);
        assert_eq!(mvn.precision_matrix.nrows(), n);
        assert_eq!(mvn.precision_matrix.ncols(), n);

        assert_eq!(mvn.information_vector, na::DVector::<f32>::zeros(n));
        assert_eq!(mvn.precision_matrix, na::DMatrix::<f32>::zeros(n, n));
    }

    #[test]
    fn test_zeroize() {
        let n = 4;
        let information_vector = na::dvector![1., 2., 3., 4.];
        let precision_matrix = na::dmatrix![
            5.0, 0.0, 1.0, 0.5;
            0.0, 5.0, 0.0, 0.0;
            1.0, 0.0, 5.0, 0.2;
            0.5, 0.0, 0.5, 5.0
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
        let mvn0 = MultivariateNormal::new(
            na::dvector![1., 2., 3.],
            na::DMatrix::<f32>::identity(n, n),
        );
        let mvn1 = MultivariateNormal::new(
            na::dvector![6., 5., 4.],
            3. * na::DMatrix::<f32>::identity(n, n),
        );

        let mvn2 = mvn0 + mvn1;

        assert_eq!(mvn2.information_vector, na::dvector![7., 7., 7.]);
        assert_eq!(
            mvn2.precision_matrix,
            na::dmatrix![
                4., 0., 0.;
                0., 4., 0.;
                0., 0., 4.
            ]
        );
    }

    #[test]
    fn test_substraction() {
        let n = 3;
        let mvn0 = MultivariateNormal::new(
            na::dvector![1., 2., 3.],
            na::DMatrix::<f32>::identity(n, n),
        );
        let mvn1 = MultivariateNormal::new(
            na::dvector![6., 5., 4.],
            3. * na::DMatrix::<f32>::identity(n, n),
        );

        let mvn2 = mvn0 + mvn1;

        assert_eq!(mvn2.information_vector, na::dvector![-5., -3., -1.]);
        assert_eq!(
            mvn2.precision_matrix,
            na::dmatrix![
                -2., 0., 0.;
                0., -2., 0.;
                0., 0., -2.
            ]
        );
    }

    // TODO: write unit tests for the rest of the methods
}
