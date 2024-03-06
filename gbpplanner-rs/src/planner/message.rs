use gbp_linalg::{prelude::*, Float};
use gbp_multivariate_normal::MultivariateNormal;

#[derive(Debug, Clone)]
// pub struct Message(pub MultivariateNormal<f32>);
pub enum Message {
    Content { gaussian: MultivariateNormal },
    Empty(usize), // dofs
}

// pub struct Message<T: GbpFloat> {
//     payload: Option<MultivariateNormal<T>>,
//     dofs: usize,
// }

impl Message {
    pub fn mean(&self) -> Vector<Float> {
        match self {
            Self::Content { gaussian } => gaussian.mean().clone(),
            Self::Empty(dofs) => Vector::<Float>::zeros(*dofs),
        }
    }

    pub fn precision_matrix(&self) -> Matrix<Float> {
        match self {
            Self::Content { gaussian } => gaussian.precision_matrix().clone(),
            Self::Empty(dofs) => Matrix::<Float>::zeros((*dofs, *dofs)),
        }
    }

    pub fn information_vector(&self) -> Vector<Float> {
        match self {
            Self::Content { gaussian } => gaussian.information_vector().clone(),
            Self::Empty(dofs) => Vector::<Float>::zeros(*dofs),
        }
    }

    /// Returns `true` if the message is [`Content`].
    ///
    /// [`Content`]: Message::Content
    #[must_use]
    pub fn is_content(&self) -> bool {
        matches!(self, Self::Content { .. })
    }

    /// Returns `true` if the message is [`Empty`].
    ///
    /// [`Empty`]: Message::Empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty(..))
    }

    pub fn with_dofs(dofs: usize) -> Self {
        let information_vector = Vector::<Float>::from_elem(dofs, 1.0 / dofs as Float);
        let precision_matrix = Matrix::<Float>::eye(dofs);
        MultivariateNormal::from_information_and_precision(information_vector, precision_matrix)
            .map(|gaussian| Self::Content { gaussian })
            .expect("An identity matrix and uniform vector is a valid multivariate normal")
        // MultivariateNormal::empty(dofs)
        //     .map(|gaussian| Self { gaussian })
        //     .expect("Empty `MultiVarianteNormal` is always valid")
    }

    // pub fn mean(&self) -> Vector<f32> {
    //     self.0.mean()
    // }

    pub fn new(
        information_vector: Vector<Float>,
        precision_matrix: Matrix<Float>,
    ) -> gbp_multivariate_normal::Result<Self> {
        MultivariateNormal::from_information_and_precision(information_vector, precision_matrix)
            .map(|gaussian| Self::Content { gaussian })
    }

    // pub fn zeros(dims: usize) -> Self {
    //     Self(MultivariateNormal::zeros(dims))
    // }

    // pub fn zeroize(&mut self) {
    //     self.0.zeroize();
    // }

    // pub fn empty(dofs: usize) -> Self {
    //     let information_vector = Vector::<f32>::from_elem(dofs, 0.0);
    //     let precision_matrix = Matrix::<f32>::zeros((dofs, dofs));
    //     MultivariateNormal::from_information_and_precision(information_vector, precision_matrix)
    //         .map(|gaussian| Self { gaussian })
    //         .expect("An identity matrix and uniform vector is a valid multivariate normal")
    // }
}

impl From<MultivariateNormal> for Message {
    fn from(value: MultivariateNormal) -> Self {
        Self::Content { gaussian: value }
    }
}
