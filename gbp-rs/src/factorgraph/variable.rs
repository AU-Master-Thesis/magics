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

pub trait Variable {
    fn id(&self) -> super::NodeId;
    fn update_belief(&mut self);
    fn prior_energy(&self) -> f64;
}
