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
use crate::gaussian::Gaussian;

pub trait Variable {
    fn update_belief(&mut self);
    fn prior_energy(&self) -> f64;

    fn get_belief(&self) -> &Gaussian;
    fn get_prior(&self) -> &Gaussian;
}
