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
/// $ cal(N)(X; mu, Sigma) = cal(N)^(-1)(X; nu, Lambda) $,
/// where $Lambda = Sigma^(-1)$ and $nu = Lambda * mu$
/// $Lambda$ is called the precision matrix, and $nu$ is called the information vector.
/// The precision matrix is the inverse of the covariance matrix $Sigma$.
/// The information vector is the product of the precision matrix and the mean.
#[derive(Debug, Clone)]
pub struct MultivariateNormal {
    /// $nu = Lambda * mu$, where $Lambda$ is the precision matrix and $mu$ is the mean
    pub information_vector: DVector<f64>,
    /// $Lambda = Sigma^(-1)$, where $Sigma$ is the covariance matrix
    pub precision_matrix: DMatrix<f64>,

    // Consider including the following fields:
    //     gbpplanner includes the means as a computational optimization
    // pub mean: nalgebra::DVector<f64>,
    // pub covariance: nalgebra::DMatrix<f64>,
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
    pub fn new(information_vector: DVector<f64>, precision_matrix: DMatrix<f64>) -> Self {
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
    }

    pub fn from_mean_and_covariance(mean: DVector<f64>, covariance: DMatrix<f64>) -> Self {
        let precision_matrix = covariance.try_inverse().unwrap();
        let information_vector = &precision_matrix * mean;
        MultivariateNormal {
            information_vector,
            precision_matrix,
        }
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
