///
// pub trait Variable {}

// struct RobotId(usize);
// type RobotId = usize;
// struct NodeId(usize);

// #[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
// pub struct Variable {
//     node_id: NodeId,
//     robot_id: RobotId,
// }

// impl Variable {
//     fn new(node_id: NodeId, robot_id: RobotId) -> Self {
//         Self { node_id, robot_id }
//     }
// }
use crate::gaussian::MultivariateNormal;

pub trait Variable: std::fmt::Debug {
    fn update_belief(&mut self);
    fn prior_energy(&self) -> f64;

    fn belief(&self) -> &MultivariateNormal;
    fn belief_mut(&mut self) -> &mut MultivariateNormal;
    fn prior(&self) -> &MultivariateNormal;
    fn prior_mut(&mut self) -> &mut MultivariateNormal;
}
