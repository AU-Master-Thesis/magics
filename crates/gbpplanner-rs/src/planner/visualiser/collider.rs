use bevy::prelude::*;

use crate::{
    config::Config,
    planner::{robot::Ball, RobotState},
};

#[derive(Default)]
pub struct ColliderVisualizerPlugin;

impl Plugin for ColliderVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                robot_colliders::render.run_if(robot_colliders::enabled),
                environment_colliders::render.run_if(environment_colliders::enabled),
            ),
        );
    }
}

mod robot_colliders {
    use super::*;
    /// **Bevy** run condition for drawing robot colliders
    pub(super) fn enabled(config: Res<Config>) -> bool {
        config.visualisation.draw.robot_colliders
    }

    pub(super) fn render(mut gizmos: Gizmos, q: Query<(&Transform, &Ball), With<RobotState>>) {
        for (transform, ball) in &q {
            // gizmos.sphere(transform.translation, Quat::IDENTITY, ball.radius,
            // Color::YELLOW);

            let position = parry2d::na::Isometry2::translation(
                transform.translation.x,
                transform.translation.y,
            );
            let aabb = ball.aabb(&position);
            let aabb = Transform {
                translation: transform.translation,
                scale: Vec3::new(
                    aabb.half_extents().x * 2.0,
                    1.0,
                    aabb.half_extents().y * 2.0,
                ),
                ..Default::default()
            };
            gizmos.cuboid(aabb, Color::RED);
        }
    }
}

mod environment_colliders {
    use super::*;
    pub(super) fn enabled(config: Res<Config>) -> bool {
        config.visualisation.draw.environment_colliders
    }

    pub(super) fn render(
        mut gizmos: Gizmos,
        env_colliders: Res<crate::environment::map_generator::Colliders>,
    ) {
        for collider in env_colliders.iter() {
            let aabb = collider.aabb();
            let center = aabb.center();
            let translation = Vec3::new(center.x, 0.0, center.y);
            let half_extents = aabb.half_extents();
            let aabb = Transform {
                translation,
                scale: Vec3::new(half_extents.x * 2.0, 1.0, half_extents.y * 2.0),
                ..Default::default()
            };
            gizmos.cuboid(aabb, Color::ORANGE_RED);
        }
    }
}
