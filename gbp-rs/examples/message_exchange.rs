use nalgebra::Vector2;
use std::time::Duration;

use gbp_rs::{
    bbox::BoundingBox2d,
    robot::{
        CommunicationMedia, Meter, Pose2d, Position2d, Probability, Robot, RobotState, Velocity2d,
    },
    FactorGraph,
};

fn create_robots() -> Vec<Robot> {
    let mut robots = Vec::new();

    let initial_position = Position2d::new(0.0, 0.0);
    for i in 0..5 {
        let communication_media = CommunicationMedia {
            max_range: Meter(10.0),
            #[allow(clippy::expect_used)]
            failure_probability: Probability::new(0.1).expect("value is between 0 and 1"),
        };

        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        let id = i;
        let factorgraph = FactorGraph::new();
        let initial_state = RobotState {
            pose: Pose2d {
                position: Position2d::new(initial_position.x + i as f64 * 2.0, 0.0),
                orientation: 0.0,
            },
            velocity: Velocity2d::new(0.0, 0.0),
        };
        // let robot = Robot::new(id, factorgraph, initial_state, communication_media, bbox);
        // robots.push(robot);
    }

    robots
}

fn main() {
    let mut robots: Vec<Robot> = create_robots();

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
