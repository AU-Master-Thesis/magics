use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone, Copy)]
pub struct MultivariateNormal {
    /// information_vector commonly used symbol: lowercase eta (η)
    pub information_vector: DVector<f64>,
    /// precision_matrix commmonly used symbol: uppercase lambda (Λ)
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
