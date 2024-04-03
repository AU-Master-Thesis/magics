#![deny(missing_docs)]
//!

use gbp_linalg::prelude::*;

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

    /// Create an empty message
    pub fn empty(dofs: usize) -> Self {
        // Self {
        //     payload: None,
        //     dofs,
        // }

        Self {
            payload: Some(Payload {
                eta: Vector::<Float>::zeros(dofs),
                lam: Matrix::<Float>::zeros((dofs, dofs)),
                mu:  Vector::<Float>::zeros(dofs),
            }),
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
}
