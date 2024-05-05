use bevy::prelude::*;

use crate::{
    config::Config,
    planner::{robot::Ball, RobotState},
};

#[derive(Default)]
pub struct ColliderVisualizerPlugin;

impl Plugin for ColliderVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_robot_colliders.run_if(enabled));
    }
}

/// **Bevy** run condition for drawing robot colliders
fn enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.robot_colliders
}

fn render_robot_colliders(mut gizmos: Gizmos, q: Query<(&Transform, &Ball), With<RobotState>>) {
    for (transform, ball) in &q {
        // gizmos.sphere(transform.translation, Quat::IDENTITY, ball.radius,
        // Color::YELLOW);

        let position = parry2d::na::Isometry2::translation(transform.translation.x, transform.translation.y);
        let aabb = ball.aabb(&position);
        let aabb = Transform {
            translation: transform.translation,
            scale: Vec3::new(aabb.half_extents().x * 2.0, 1.0, aabb.half_extents().y * 2.0),
            ..Default::default()
        };
        gizmos.cuboid(aabb, Color::RED);
    }
}
