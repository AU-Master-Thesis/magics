use nalgebra::{DMatrix, DVector};

// TODO: maybe there should be something to ensure the dimensions of the information vector and precision matrix match

/// Multivariate Gaussian/Normal
#[derive(Debug)]
pub struct MultivariateNormal {
    /// dim
    size: usize,
    /// eta
    pub information_vector: DVector<f64>,
    /// lam
    pub precision_matrix: DMatrix<f64>,
}

impl MultivariateNormal {
    /// information_vector commonly used symbol: lowercase eta (η)
    /// precision_matrix commmonly used symbol: uppercase lambda (Λ)
    pub fn new(
        size: usize,
        information_vector: Option<DVector<f64>>,
        precision_matrix: Option<DMatrix<f64>>,
    ) -> Self {
        let information_vector = information_vector.unwrap_or_else(|| DVector::zeros(size));
        // check if the precision_matrix has an inverse
        let precision_matrix = precision_matrix.unwrap_or_else(|| DMatrix::zeros(size, size));

        Self {
            size,
            information_vector,
            precision_matrix,
        }
    }

    pub fn mean(&self) -> DVector<f64> {
        self.precision_matrix.clone().try_inverse().unwrap() * &self.information_vector
    }

    pub fn covariance(&self) -> DMatrix<f64> {
        self.precision_matrix.clone().try_inverse().unwrap()
    }

    pub fn mean_and_covariance(&self) -> (DVector<f64>, DMatrix<f64>) {
        let covariance = self.covariance();
        let mean = &covariance * &self.information_vector;

        (mean, covariance)
    }

    pub fn set_with_covariance_form(&mut self, mean: DVector<f64>, covariance: DMatrix<f64>) {
        // check for invertibility of covariance matrix input
        self.precision_matrix = covariance.try_inverse().unwrap();
        self.information_vector = &self.precision_matrix * mean;
    }
}
