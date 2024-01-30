// struct RobotId(usize);
type RobotId = usize;
// struct NodeId(usize);
type NodeId = usize;

trait Factor {}

#[derive(Debug)]
struct DefaultFactor;
#[derive(Debug)]
struct DynamicFactor;
#[derive(Debug)]
struct InterRobotFactor;
#[derive(Debug)]
struct ObstacleFactor;

impl Factor for DefaultFactor {}
impl Factor for DynamicFactor {}
impl Factor for InterRobotFactor {}
impl Factor for ObstacleFactor {}

#[derive(Debug)]
enum FactorType {
    Default(DefaultFactor),
    Dynamic(DynamicFactor),
    InterRobot(InterRobotFactor),
    Ocstacle(ObstacleFactor),
}

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
struct Message {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
struct Variable {
    node_id: NodeId,
    robot_id: RobotId,
}

impl Variable {
    fn new(node_id: NodeId, robot_id: RobotId) -> Self {
        Self { node_id, robot_id }
    }
}
