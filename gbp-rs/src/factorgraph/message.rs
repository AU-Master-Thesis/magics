// enum MsgPassingMode {EXTERNAL, INTERNAL};

/// Ways message passing can be performed
#[derive(Debug)]
enum MessagePassingMode {
    /// Between two different robots/factorgraphs
    External,
    /// Within a robot/factorgraph
    Internal,
}
// Eigen::VectorXd eta;
// Eigen::MatrixXd lambda;
// Eigen::VectorXd mu;

#[derive(Debug)]
pub struct Message {
    pub eta: nalgebra::DVector<f64>,
    pub lambda: nalgebra::DMatrix<f64>,
    pub mu: nalgebra::DVector<f64>,
}

// Implement addtion and subtraction for messages
impl std::ops::Add for Message {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            eta: self.eta + other.eta,
            lambda: self.lambda + other.lambda,
            mu: self.mu + other.mu,
        }
    }
}

impl std::ops::Sub for Message {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            eta: self.eta - other.eta,
            lambda: self.lambda - other.lambda,
            mu: self.mu - other.mu,
        }
    }
}
