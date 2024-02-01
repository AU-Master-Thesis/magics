/// 

// pub trait Variable {}

// struct RobotId(usize);
type RobotId = usize;
// struct NodeId(usize);
type NodeId = usize;


#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
pub struct Variable {
    node_id: NodeId,
    robot_id: RobotId,
}

impl Variable {
    fn new(node_id: NodeId, robot_id: RobotId) -> Self {
        Self { node_id, robot_id }
    }
}
