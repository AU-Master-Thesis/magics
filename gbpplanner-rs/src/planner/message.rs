use gbp_linalg::{prelude::*, Float};
use gbp_multivariate_normal::MultivariateNormal;

// #[derive(Debug, Clone)]
// // pub struct Message(pub MultivariateNormal<f32>);
// pub enum Message {
//     Content { gaussian: MultivariateNormal },
//     Empty(usize), // dofs
// }

// type Payload = MultivariateNormal;

#[derive(Debug, Clone)]
pub struct Payload {
    pub eta: Vector<Float>,
    pub lam: Matrix<Float>,
    pub mu:  Vector<Float>,
}

pub struct Eta(pub Vector<Float>);
pub struct Lam(pub Matrix<Float>);
pub struct Mu(pub Vector<Float>);

#[derive(Debug, Clone)]
pub struct Message {
    // TODO: wrap in Cow<'_> to avoid cloning when sending messages from variables to factors,
    // as the messages are identical.
    // payload: Option<MultivariateNormal>,
    payload: Option<Payload>,
    dofs:    usize,
}

impl Message {
    pub fn mean(&self) -> Option<&Vector<Float>> {
        // self.payload.as_ref().map(|gaussian| gaussian.mean())
        self.payload.as_ref().map(|payload| &payload.mu)
    }

    pub fn precision_matrix(&self) -> Option<&Matrix<Float>> {
        self.payload
            .as_ref()
            // .map(|gaussian| gaussian.precision_matrix())
            .map(|payload| &payload.lam)
    }

    pub fn information_vector(&self) -> Option<&Vector<Float>> {
        self.payload
            .as_ref()
            // .map(|gaussian| gaussian.information_vector())
            .map(|payload| &payload.eta)
    }

    // pub fn mean(&self) -> Vector<Float> {
    //     match self {
    //         Self::Content { gaussian } => gaussian.mean().clone(),
    //         Self::Empty(dofs) => Vector::<Float>::zeros(*dofs),
    //     }
    // }

    // pub fn precision_matrix(&self) -> Matrix<Float> {
    //     match self {
    //         Self::Content { gaussian } => gaussian.precision_matrix().clone(),
    //         Self::Empty(dofs) => Matrix::<Float>::zeros((*dofs, *dofs)),
    //     }
    // }

    // pub fn information_vector(&self) -> Vector<Float> {
    //     match self {
    //         Self::Content { gaussian } => gaussian.information_vector().clone(),
    //         Self::Empty(dofs) => Vector::<Float>::zeros(*dofs),
    //     }
    // }

    // /// Returns `true` if the message is [`Content`].
    // ///
    // /// [`Content`]: Message::Content
    // #[must_use]
    // pub fn is_content(&self) -> bool {
    //     matches!(self, Self::Content { .. })
    // }

    // /// Returns `true` if the message is [`Empty`].
    // ///
    // /// [`Empty`]: Message::Empty
    // #[must_use]
    // pub fn is_empty(&self) -> bool {
    //     matches!(self, Self::Empty(..))
    // }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.payload.is_none()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.dofs
    }

    /// Take the inner `MultivariateNormal` from the message.
    /// Leaving the message in an empty state.
    #[inline]
    pub fn take(&mut self) -> Option<Payload> {
        self.payload.take()
    }

    #[inline]
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    // pub fn with_dofs(dofs: usize) -> Self {
    //     let information_vector = Vector::<Float>::from_elem(dofs, 1.0 / dofs as
    // Float);     let precision_matrix = Matrix::<Float>::eye(dofs);
    //     MultivariateNormal::from_information_and_precision(information_vector,
    // precision_matrix)         .map(|gaussian| Self::Content { gaussian })
    //         .expect("An identity matrix and uniform vector is a valid
    // multivariate normal")     // MultivariateNormal::empty(dofs)
    //     //     .map(|gaussian| Self { gaussian })
    //     //     .expect("Empty `MultiVarianteNormal` is always valid")
    // }

    // pub fn mean(&self) -> Vector<f32> {
    //     self.0.mean()
    // }

    pub fn empty(dofs: usize) -> Self {
        Self {
            payload: None,
            dofs,
        }
    }

    pub fn new(eta: Eta, lam: Lam, mu: Mu) -> Self {
        let dofs = eta.0.len();
        Self {
            payload: Some(Payload {
                eta: eta.0,
                lam: lam.0,
                mu:  mu.0,
            }),
            dofs,
        }
    }

    // pub fn new(eta: Vector<Float>, lam: Matrix<Float>, mu: Vector<Float>) -> Self
    // {     let dofs = eta.len();
    //     Self {
    //         payload: Some(Payload { eta, lam, mu }),
    //         dofs,
    //     }
    // }

    // pub fn new(normal: MultivariateNormal) -> Self {
    //     let dofs = normal.len();
    //     Self {
    //         payload: Some(normal),
    //         dofs,
    //     }
    // }

    // pub fn new(
    //     information_vector: Vector<Float>,
    //     precision_matrix: Matrix<Float>,
    // ) -> gbp_multivariate_normal::Result<Self> {
    //     MultivariateNormal::from_information_and_precision(information_vector,
    // precision_matrix)         .map(|gaussian| Self::Content { gaussian })
    // }

    // pub fn zeros(dims: usize) -> Self {
    //     Self(MultivariateNormal::zeros(dims))
    // }

    // pub fn zeroize(&mut self) {
    //     self.0.zeroize();
    // }

    // pub fn empty(dofs: usize) -> Self {
    //     let information_vector = Vector::<f32>::from_elem(dofs, 0.0);
    //     let precision_matrix = Matrix::<f32>::zeros((dofs, dofs));
    //     MultivariateNormal::from_information_and_precision(information_vector,
    // precision_matrix)         .map(|gaussian| Self { gaussian })
    //         .expect("An identity matrix and uniform vector is a valid
    // multivariate normal") }
}
// impl From<MultivariateNormal> for Message {
//     fn from(value: MultivariateNormal) -> Self {
//         let dofs = value.len();
//         Self {
//             payload: Some(value),
//             dofs,
//         }
//     }
// }
