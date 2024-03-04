// #![warn(missing_docs)]
use gbp_linalg::{Matrix, Vector};
use ndarray_inverse::Inverse;

#[derive(Debug, thiserror::Error)]
pub enum MultivariateNormalError {
    #[error("the precision matrix is not square, it has shape {0}x{1}")]
    NonSquarePrecisionMatrix(usize, usize),
    #[error(
        "the length of the information vector ({0}) is not equal to the number of rows ({1}) or columns ({2}) of the precision matrix"
    )]
    VectorLengthNotEqualMatrixShape(usize, usize, usize),
    #[error("the covariance matrix is not invertible, which is required to calculate the precision matrix")]
    NonInvertibleCovarianceMatrix,
    #[error("the precision matrix is not invertible, which is required to calculate the covariance matrix")]
    NonInvertiblePrecisionMatrix,
}

pub type Result<T> = std::result::Result<T, MultivariateNormalError>;

/// Properties of a scalar type that is used in the multivariate normal distribution
pub trait Scalar: ndarray::NdFloat + Copy + std::iter::Sum {}

impl Scalar for f32 {}
impl Scalar for f64 {}

#[allow(clippy::len_without_is_empty)]
#[derive(Debug, Clone)]
pub struct MultivariateNormal<T: Scalar> {
    information: Vector<T>,
    precision: Matrix<T>,
    mean: Vector<T>,
    /// Whether the mean needs to be updated
    dirty: bool,
}

impl<T: Scalar> MultivariateNormal<T> {
    /// Create a new multivariate normal distribution in information form.
    ///
    /// # Example:
    /// ```
    /// use gbp_multivariate_normal::{MultivariateNormal, Result};
    /// use gbp_linalg::{Matrix, Vector, array};
    /// fn main() -> Result<()> {
    ///     let information = array![1.0, 2.0, 3.0];
    ///     let precision = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    ///     let normal = MultivariateNormal::from_information_and_precision(information, precision)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_information_and_precision(
        information_vector: Vector<T>,
        precision_matrix: Matrix<T>,
    ) -> Result<Self> {
        if !precision_matrix.is_square() {
            Err(MultivariateNormalError::NonSquarePrecisionMatrix(
                precision_matrix.nrows(),
                precision_matrix.ncols(),
            ))
        } else if information_vector.len() != precision_matrix.nrows()
            || information_vector.len() != precision_matrix.ncols()
        {
            Err(MultivariateNormalError::VectorLengthNotEqualMatrixShape(
                information_vector.len(),
                precision_matrix.nrows(),
                precision_matrix.ncols(),
            ))
        } else {
            if precision_matrix.det().is_zero() {
                return Err(MultivariateNormalError::NonInvertiblePrecisionMatrix);
            }
            let mean = precision_matrix.dot(&information_vector);
            Ok(Self {
                information: information_vector,
                precision: precision_matrix,
                mean,
                dirty: false,
            })
        }
    }

    /// Create a new multivariate normal distribution from the mean and covariance matrix
    ///
    /// # Example:
    /// ```
    /// use gbp_multivariate_normal::{MultivariateNormal, MultivariateNormalError, Result};
    /// use gbp_linalg::{Matrix, Vector, array};
    /// fn main() -> Result<()> {
    ///     let mean = array![1.0, 2.0, 3.0];
    ///     let covariance = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    ///     let normal = MultivariateNormal::from_mean_and_covariance(mean, covariance)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_mean_and_covariance(mean: Vector<T>, covariance: Matrix<T>) -> Result<Self> {
        if !covariance.is_square() {
            Err(MultivariateNormalError::NonSquarePrecisionMatrix(
                covariance.nrows(),
                covariance.ncols(),
            ))
        } else if mean.len() != covariance.nrows() || mean.len() != covariance.ncols() {
            Err(MultivariateNormalError::VectorLengthNotEqualMatrixShape(
                mean.len(),
                covariance.nrows(),
                covariance.ncols(),
            ))
        } else {
            let Some(precision) = covariance.inv() else {
                return Err(MultivariateNormalError::NonInvertibleCovarianceMatrix);
            };
            let information = precision.dot(&mean);
            Ok(Self {
                information,
                precision,
                mean,
                dirty: false,
            })
        }
    }

    /// Returns the "dimension" of the multivariate normal distribution, which is the length of the information vector
    /// equal to the number of rows and columns of the precision matrix.
    pub fn len(&self) -> usize {
        self.information.len()
    }

    /// Get the information vector of the multivariate normal distribution
    #[inline(always)]
    pub fn information_vector(&self) -> &Vector<T> {
        &self.information
    }
    /// Get the precision matrix of the multivariate normal distribution
    #[inline(always)]
    pub fn precision_matrix(&self) -> &Matrix<T> {
        &self.precision
    }

    pub fn update_information_vector(&mut self, value: &Vector<T>) {
        self.information.clone_from(value);
        self.update();
    }

    // pub fn update_precision_matrix(mut self, value: &Matrix<T>) -> Result<Self> {
    //     if value.det() == T::zero() {
    //         Err(MultivariateNormalError::NonInvertiblePrecisionMatrix)
    //     } else {
    //         self.precision.clone_from(value);
    //         self.update();
    //         Ok(self)
    //     }
    // }

    pub fn update_precision_matrix(&mut self, value: &Matrix<T>) -> Result<()> {
        if value.det() == T::zero() {
            Err(MultivariateNormalError::NonInvertiblePrecisionMatrix)
        } else {
            self.precision.clone_from(value);
            self.update();
            Ok(())
        }
    }

    /// Get the mean of the multivariate normal distribution
    #[inline(always)]
    pub fn mean(&self) -> &Vector<T> {
        &self.mean
    }

    /// Get the covariance matrix of the multivariate normal distribution
    /// Returns an owned value `Matrix<T>`, as the covariance matrix is not stored internally
    pub fn covariance(&self) -> Matrix<T> {
        self.precision
            .inv()
            .expect("the precision matrix is invertible")
    }

    /// Set the information vector of the multivariate normal distribution
    ///
    /// The motivation for this method is to allow the user to set the information vector directly,
    /// without having to update the mean. For example if you have a loop where you add assign the information
    /// multiple times, it is wasteful to update the mean after each assignment.
    ///
    /// # Safety
    /// No checks are performed to ensure that the given vector is the same length as the one stored
    /// The mean is not updated after setting the information vector, so it is the responsibility of the caller to call [`Self::update()`] after setting the information vector
    #[inline(always)]
    pub unsafe fn set_information_vector(&mut self, value: &Vector<T>) {
        self.information.clone_from(value);
        self.dirty = true;
    }

    /// Set the precision matrix of the multivariate normal distribution
    ///
    /// The motivation for this method is to allow the user to set the precision matrix directly,
    /// without having to update the mean. For example if you have a loop where you add assign the precision matrix
    /// multiple times, it is wasteful to update the mean after each assignment.
    ///
    /// # Safety
    /// No checks are performed to ensure that the precision matrix is invertible
    /// It is the responsibility of the caller to ensure that the precision matrix is invertible
    /// The mean is not updated after setting the precision matrix, so it is the responsibility of the caller to call [`Self::update()`] after setting the precision matrix
    #[inline(always)]
    pub unsafe fn set_precision_matrix(&mut self, value: &Matrix<T>) {
        self.precision.clone_from(value);
        self.dirty = true;
    }

    /// Add a vector to the information vector of the multivariate normal distribution
    ///
    /// The motivation for this method is to allow the user to interract with the information vector directly,
    /// without having to update the mean. For example if you have a loop where you add assign the information
    /// multiple times, it is wasteful to update the mean after each assignment.
    ///
    /// # Safety
    /// No checks are performed to ensure that the given vector is the same length as the one stored
    /// The mean is not updated after setting the information vector, so it is the responsibility of the caller to call [`Self::update()`] after setting the information vector
    pub unsafe fn add_assign_information_vector(&mut self, value: &Vector<T>) {
        self.information += value;
        self.dirty = true;
    }

    /// Add a matrix to the precision matrix of the multivariate normal distribution
    ///
    /// The motivation for this method is to allow the user to interract with the information vector directly,
    /// without having to update the mean. For example if you have a loop where you add assign the information
    /// multiple times, it is wasteful to update the mean after each assignment.
    ///
    /// # Safety
    /// No checks are performed to ensure that the precision matrix is invertible
    /// It is the responsibility of the caller to ensure that the precision matrix is invertible
    /// The mean is not updated after setting the precision matrix, so it is the responsibility of the caller to call [`Self::update()`] after setting the precision matrix
    pub unsafe fn add_assign_precision_matrix(&mut self, value: &Matrix<T>) {
        self.precision += value;
        self.dirty = true;
    }

    /// Update the mean of the multivariate normal distribution
    /// Returns true if the mean was updated, false otherwise
    /// This method is meant to be called after using [`Self::set_information_vector()`] or [`Self::set_precision_matrix()`]
    pub fn update(&mut self) -> bool {
        if self.dirty {
            self.mean = self.precision.dot(&self.information);
            self.dirty = false;
            true
        } else {
            false
        }
    }
}

impl<T: Scalar> std::ops::Add<&MultivariateNormal<T>> for MultivariateNormal<T> {
    type Output = MultivariateNormal<T>;

    fn add(self, rhs: &MultivariateNormal<T>) -> Self::Output {
        let information = self.information + &rhs.information;
        let precision = self.precision + &rhs.precision;
        let mean = precision.dot(&information);
        Self::Output {
            information,
            precision,
            mean,
            dirty: false,
        }
    }
}

impl<T: Scalar> std::ops::Add<&MultivariateNormal<T>> for &MultivariateNormal<T> {
    type Output = MultivariateNormal<T>;

    fn add(self, rhs: &MultivariateNormal<T>) -> Self::Output {
        let information = &self.information + &rhs.information;
        let precision = &self.precision + &rhs.precision;
        let mean = precision.dot(&information);
        Self::Output {
            information,
            precision,
            mean,
            dirty: false,
        }
    }
}

impl<T: Scalar> std::ops::AddAssign<&MultivariateNormal<T>> for MultivariateNormal<T> {
    fn add_assign(&mut self, rhs: &MultivariateNormal<T>) {
        self.information += &rhs.information;
        self.precision += &rhs.precision;
        self.dirty = true;
        self.update();
    }
}

impl<T: Scalar> std::ops::Sub<&MultivariateNormal<T>> for MultivariateNormal<T> {
    type Output = MultivariateNormal<T>;

    fn sub(self, rhs: &MultivariateNormal<T>) -> Self::Output {
        let information = self.information - &rhs.information;
        let precision = self.precision - &rhs.precision;
        let mean = precision.dot(&information);
        Self::Output {
            information,
            precision,
            mean,
            dirty: false,
        }
    }
}

impl<T: Scalar> std::ops::Sub<&MultivariateNormal<T>> for &MultivariateNormal<T> {
    type Output = MultivariateNormal<T>;

    fn sub(self, rhs: &MultivariateNormal<T>) -> Self::Output {
        let information = &self.information - &rhs.information;
        let precision = &self.precision - &rhs.precision;
        let mean = precision.dot(&information);
        Self::Output {
            information,
            precision,
            mean,
            dirty: false,
        }
    }
}

impl<T: Scalar> std::ops::SubAssign<&MultivariateNormal<T>> for MultivariateNormal<T> {
    fn sub_assign(&mut self, rhs: &MultivariateNormal<T>) {
        self.information -= &rhs.information;
        self.precision -= &rhs.precision;
        self.dirty = true;
        self.update();
    }
}

impl<T: Scalar> std::ops::Mul<&MultivariateNormal<T>> for MultivariateNormal<T> {
    type Output = MultivariateNormal<T>;

    fn mul(self, rhs: &MultivariateNormal<T>) -> Self::Output {
        // In the information form, the product of two multivariate normal distributions is the sum of the information vectors and the sum of the precision matrices
        let information = self.information + &rhs.information;
        let precision = self.precision + &rhs.precision;
        let mean = precision.dot(&information);
        Self::Output {
            information,
            precision,
            mean,
            dirty: false,
        }
    }
}

impl<T: Scalar> std::ops::MulAssign<&MultivariateNormal<T>> for MultivariateNormal<T> {
    fn mul_assign(&mut self, rhs: &MultivariateNormal<T>) {
        // In the information form, the product of two multivariate normal distributions is the sum of the information vectors and the sum of the precision matrices
        self.information += &rhs.information;
        self.precision += &rhs.precision;
        self.dirty = true;
        self.update();
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use gbp_linalg::array;

    #[test]
    fn create_from_information_and_precision() {
        let information = array![1.0, 2.0, 3.0];
        let precision = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal = MultivariateNormal::from_information_and_precision(
            information.clone(),
            precision.clone(),
        )
        .unwrap();
        assert_eq!(normal.information_vector(), &information);
        assert_eq!(normal.precision_matrix(), &precision);
        assert_eq!(normal.covariance(), precision.inv().unwrap());
        assert_eq!(normal.mean(), precision.dot(&information));
    }

    #[test]
    fn create_from_mean_and_covariance() {
        let mean = array![1.0, 2.0, 3.0];
        let covariance = array![[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.5]];
        let normal =
            MultivariateNormal::from_mean_and_covariance(mean.clone(), covariance.clone()).unwrap();
        assert_eq!(normal.mean(), &mean);
        assert_eq!(normal.covariance(), covariance);
        assert_eq!(normal.precision_matrix(), covariance.inv().unwrap());
        assert_eq!(
            normal.information_vector(),
            covariance.inv().unwrap().dot(&mean)
        );
    }

    #[test]
    fn information_and_precision_of_unequal_dimensions_should_fail() {
        let information = array![1.0, 2.0, 3.0];
        let precision = array![[1.0, 0.0], [0.0, 1.0]];
        let result = MultivariateNormal::from_information_and_precision(information, precision);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::VectorLengthNotEqualMatrixShape(
                3, 2, 2
            ))
        ));

        let information = array![1.0, 2.0];
        let precision = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let result = MultivariateNormal::from_information_and_precision(information, precision);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::VectorLengthNotEqualMatrixShape(
                2, 3, 3
            ))
        ));
    }

    #[test]
    fn mean_and_covariance_of_unequal_dimensions_should_fail() {
        let mean = array![1.0, 2.0, 3.0];
        let covariance = array![[1.0, 0.0], [0.0, 1.0]];
        let result = MultivariateNormal::from_mean_and_covariance(mean, covariance);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::VectorLengthNotEqualMatrixShape(
                3, 2, 2
            ))
        ));

        let mean = array![1.0, 2.0];
        let covariance = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let result = MultivariateNormal::from_mean_and_covariance(mean, covariance);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::VectorLengthNotEqualMatrixShape(
                2, 3, 3
            ))
        ));
    }

    #[test]
    fn non_square_precision_matrix_should_fail() {
        let information = array![1.0, 2.0];
        let precision = array![[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];
        let result = MultivariateNormal::from_information_and_precision(information, precision);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::NonSquarePrecisionMatrix(3, 2))
        ));

        let information = array![1.0, 2.0, 3.0];
        let precision = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = MultivariateNormal::from_information_and_precision(information, precision);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::NonSquarePrecisionMatrix(2, 3))
        ));
    }

    #[test]
    fn non_square_covariance_matrix_should_fail() {
        let mean = array![1.0, 2.0];
        let covariance = array![[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];
        let result = MultivariateNormal::from_mean_and_covariance(mean, covariance);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::NonSquarePrecisionMatrix(3, 2))
        ));

        let mean = array![1.0, 2.0, 3.0];
        let covariance = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = MultivariateNormal::from_mean_and_covariance(mean, covariance);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::NonSquarePrecisionMatrix(2, 3))
        ));
    }

    #[test]
    fn singular_covariance_matrix_should_fail() {
        let mean = array![1.0, 2.0, 3.0];
        let covariance = array![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let result = MultivariateNormal::from_mean_and_covariance(mean, covariance);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::NonInvertibleCovarianceMatrix)
        ));
    }

    #[test]
    fn singular_precision_matrix_should_fail() {
        let information = array![1.0, 2.0, 3.0];
        let precision = array![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let result = MultivariateNormal::from_information_and_precision(information, precision);
        assert!(matches!(
            result,
            Err(MultivariateNormalError::NonInvertiblePrecisionMatrix)
        ));
    }

    #[test]
    fn update_mean() {
        let information = array![1.0, 2.0, 3.0];
        let precision = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut normal = MultivariateNormal::from_information_and_precision(
            information.clone(),
            precision.clone(),
        )
        .unwrap();
        assert_eq!(normal.mean(), precision.dot(&information));
        assert!(!normal.update());
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            normal.set_information_vector(&array![3.0, 2.0, 1.0]);
        }
        assert!(normal.update());
        assert_eq!(normal.mean(), precision.dot(&array![3.0, 2.0, 1.0]));
        assert!(!normal.update());

        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            normal.set_precision_matrix(&array![[2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]]);
        }
        assert!(normal.update());
        assert_eq!(normal.mean(), array![6.0, 4.0, 2.0]);
        assert!(!normal.update());
    }

    #[test]
    fn add_two_normals() {
        let information1 = array![1.0, 2.0, 3.0];
        let precision1 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal1 = MultivariateNormal::from_information_and_precision(
            information1.clone(),
            precision1.clone(),
        )
        .unwrap();

        let information2 = array![3.0, 2.0, 1.0];
        let precision2 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal2 = MultivariateNormal::from_information_and_precision(
            information2.clone(),
            precision2.clone(),
        )
        .unwrap();

        let sum = normal1 + &normal2;
        assert_eq!(sum.information_vector(), &information1 + &information2);
        assert_eq!(sum.precision_matrix(), &precision1 + &precision2);
        assert_eq!(
            sum.mean(),
            (precision1 + precision2).dot(&(information1 + information2))
        );
    }

    #[test]
    fn add_assign_two_normals() {
        let information1 = array![1.0, 2.0, 3.0];
        let precision1 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut normal1 = MultivariateNormal::from_information_and_precision(
            information1.clone(),
            precision1.clone(),
        )
        .unwrap();

        let information2 = array![3.0, 2.0, 1.0];
        let precision2 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal2 = MultivariateNormal::from_information_and_precision(
            information2.clone(),
            precision2.clone(),
        )
        .unwrap();

        normal1 += &normal2;
        assert_eq!(normal1.information_vector(), &information1 + &information2);
        assert_eq!(normal1.precision_matrix(), &precision1 + &precision2);
        assert_eq!(
            normal1.mean(),
            (precision1 + precision2).dot(&(information1 + information2))
        );
    }

    #[test]
    fn sub_two_normals() {
        let information1 = array![1.0, 2.0, 3.0];
        let precision1 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal1 = MultivariateNormal::from_information_and_precision(
            information1.clone(),
            precision1.clone(),
        )
        .unwrap();

        let information2 = array![3.0, 2.0, 1.0];
        let precision2 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal2 = MultivariateNormal::from_information_and_precision(
            information2.clone(),
            precision2.clone(),
        )
        .unwrap();

        let diff = normal1 - &normal2;
        assert_eq!(diff.information_vector(), &information1 - &information2);
        assert_eq!(diff.precision_matrix(), &precision1 - &precision2);
        assert_eq!(
            diff.mean(),
            (precision1 - precision2).dot(&(information1 - information2))
        );
    }

    #[test]
    fn sub_assign_two_normals() {
        let information1 = array![1.0, 2.0, 3.0];
        let precision1 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut normal1 = MultivariateNormal::from_information_and_precision(
            information1.clone(),
            precision1.clone(),
        )
        .unwrap();

        let information2 = array![3.0, 2.0, 1.0];
        let precision2 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal2 = MultivariateNormal::from_information_and_precision(
            information2.clone(),
            precision2.clone(),
        )
        .unwrap();

        normal1 -= &normal2;
        assert_eq!(normal1.information_vector(), &information1 - &information2);
        assert_eq!(normal1.precision_matrix(), &precision1 - &precision2);
        assert_eq!(
            normal1.mean(),
            (precision1 - precision2).dot(&(information1 - information2))
        );
    }

    #[test]
    fn mul_two_normals() {
        let information1 = array![1.0, 2.0, 3.0];
        let precision1 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal1 = MultivariateNormal::from_information_and_precision(
            information1.clone(),
            precision1.clone(),
        )
        .unwrap();

        let information2 = array![3.0, 2.0, 1.0];
        let precision2 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal2 = MultivariateNormal::from_information_and_precision(
            information2.clone(),
            precision2.clone(),
        )
        .unwrap();

        let product = normal1 * &normal2;
        assert_eq!(product.information_vector(), &information1 + &information2);
        assert_eq!(product.precision_matrix(), &precision1 + &precision2);
        assert_eq!(
            product.mean(),
            (precision1 + precision2).dot(&(information1 + information2))
        );
    }

    #[test]
    fn mul_assign_two_normals() {
        let information1 = array![1.0, 2.0, 3.0];
        let precision1 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut normal1 = MultivariateNormal::from_information_and_precision(
            information1.clone(),
            precision1.clone(),
        )
        .unwrap();

        let information2 = array![3.0, 2.0, 1.0];
        let precision2 = array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normal2 = MultivariateNormal::from_information_and_precision(
            information2.clone(),
            precision2.clone(),
        )
        .unwrap();

        normal1 *= &normal2;
        assert_eq!(normal1.information_vector(), &information1 + &information2);
        assert_eq!(normal1.precision_matrix(), &precision1 + &precision2);
        assert_eq!(
            normal1.mean(),
            (precision1 + precision2).dot(&(information1 + information2))
        );
    }
}
