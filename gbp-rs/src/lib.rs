pub mod bbox;
pub mod factor;
pub mod factorgraph;
pub mod factors;
pub mod message;
pub mod multivariate_normal;
pub mod robot;
mod utils;
pub mod variable;

mod macros;

pub mod prelude {
    pub use super::Factor;
    // pub use super::FactorGraph;
    // pub use super::MessagePassingMode;
    pub use super::Variable;
}

use std::{collections::HashMap, rc::Rc};

pub use factor::Factor;
use multivariate_normal::MultivariateNormal;
use nalgebra::{DMatrix, DVector};
use robot::{Robot, RobotId};
pub use variable::Variable;

pub type NodeId = usize;
pub type Timestep = u32;

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
        write!(
            f,
            "(robot_id: {}, node_id: {})",
            self.robot_id, self.node_id
        )
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
    pub fn new(information_vector: DVector<f32>, precision_matrix: DMatrix<f32>) -> Self {
        Self(MultivariateNormal {
            information_vector,
            precision_matrix,
        })
    }

    pub fn with_dofs(dofs: usize) -> Self {
        Self(MultivariateNormal::zeros(dofs))
    }

    pub fn zeroize(&mut self) {
        self.0.information_vector.fill(0.0);
        self.0.precision_matrix.fill(0.0);
    }
}

pub type Mailbox = HashMap<Key, Message>;

pub struct World<'a> {
    pub robots: Vec<Rc<Robot<'a>>>,
}

impl<'a> World<'a> {
    // TODO: ask `r/rust` which about the pros/cons of these two implementations and which is more idiomatic
    // pub fn get_robot_with_id(&self, id: RobotId) -> Option<&Rc<Robot<'a>>> {
    pub fn robot_with_id(&self, id: RobotId) -> Option<Rc<Robot<'a>>> {
        // self.robots.iter().find(|it| it.id == id)
        self.robots
            .iter()
            .find(|it| it.id == id)
            .and_then(|robot| Some(Rc::clone(robot)))
    }

    // pub fn robot_with_id_mut(&mut self, id: RobotId) -> Option<&mut Rc<Robot<'a>>> {
    pub fn robot_with_id_mut(&mut self, id: RobotId) -> Option<Rc<Robot<'a>>> {
        self.robots
            .iter_mut()
            .find(|it| it.id == id)
            .and_then(|robot| Some(Rc::clone(robot)))
    }
}

/*
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.robot_id == other.robot_id && self.node_id == other.node_id
    }
}

impl Eq for Key {}*/

#[derive(Debug)]
struct IdGenerator {
    next_robot_id: RobotId,
    next_variable_id: NodeId,
    next_factor_id: NodeId,
}

impl IdGenerator {
    fn new() -> Self {
        Self {
            next_robot_id: 0,
            next_variable_id: 0,
            next_factor_id: 0,
        }
    }

    fn next_robot_id(&mut self) -> RobotId {
        let id = self.next_robot_id;
        self.next_robot_id += 1;
        id
    }

    fn next_variable_id(&mut self) -> NodeId {
        let id = self.next_variable_id;
        self.next_variable_id += 1;
        id
    }

    fn next_factor_id(&mut self) -> NodeId {
        let id = self.next_factor_id;
        self.next_factor_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_id_generator() {
        let mut gen = IdGenerator::new();
        assert_eq!(gen.next_robot_id, 0);
        assert_eq!(gen.next_factor_id, 0);
        assert_eq!(gen.next_variable_id, 0);

        let robot_id0 = gen.next_robot_id();
        assert_eq!(robot_id0, 0);
        assert_ne!(robot_id0, gen.next_robot_id());

        let variable_id0 = gen.next_variable_id();
        assert_eq!(variable_id0, 0);
        assert_ne!(variable_id0, gen.next_variable_id());

        let factor_id0 = gen.next_factor_id();
        assert_eq!(factor_id0, 0);
        assert_ne!(factor_id0, gen.next_factor_id());
    }
}
