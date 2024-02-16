pub mod bbox;
pub mod factor;
pub mod message;
pub mod multivariate_normal;
pub mod robot;
pub mod variable;
pub mod factors;
pub mod factorgraph;

pub mod prelude {
    pub use super::Factor;
    // pub use super::FactorGraph;
    // pub use super::MessagePassingMode;
    pub use super::Variable;
}


use std::collections::HashMap;

pub use factor::Factor;
use multivariate_normal::MultivariateNormal;
use robot::RobotId;
pub use variable::Variable;

pub type NodeId = usize;
pub type FactorGraphId = usize;

/// Datastructure used to identify both variables and factors
/// It includes the id of the robot that the variable/factor belongs to, as well as its own id.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub struct Key {
    pub robot_id: RobotId,
    pub node_id: NodeId,
    // valid: bool, // Set in gbpplanner as `valid_` but not used
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(robot_id: {}, node_id: {})", self.robot_id, self.node_id)
    }
}


impl Key {
    pub fn new(robot_id: RobotId, node_id: NodeId) -> Self {
        Self {
            robot_id,
            node_id,
            // valid: true,
        }
    }
}


#[derive(Debug)]
pub struct Message(MultivariateNormal);

impl Message {
    pub fn with_dofs(dofs: usize) -> Self {
        Self(MultivariateNormal::zeros(dofs))
    }
}


pub type Mailbox = HashMap<Key, Message>;
/*
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.robot_id == other.robot_id && self.node_id == other.node_id
    }
}

impl Eq for Key {}*/

