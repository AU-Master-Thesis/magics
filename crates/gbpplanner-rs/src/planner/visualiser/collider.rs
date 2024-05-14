use bevy::prelude::*;
use gbp_config::Config;

use crate::planner::{robot::Ball, RobotState};

#[derive(Default)]
pub struct ColliderVisualizerPlugin;

impl Plugin for ColliderVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                robot_colliders::render.run_if(robot_colliders::enabled),
                environment_colliders::render.run_if(
                    environment_colliders::enabled
                        .and_then(resource_exists::<gbp_global_planner::Colliders>),
                ),
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
            // let position = parry2d::na::Isometry2::translation(
            //     transform.translation.x,
            //     transform.translation.y,
            // );
            // let aabb = ball.aabb(&position);
            // let half_extents = aabb.half_extents();
            // let aabb = Transform {
            //     translation: transform.translation,
            //     scale: Vec3::new(half_extents.x * 2.0, 1.0, half_extents.y * 2.0),
            //     ..Default::default()
            // };
            // gizmos.cuboid(aabb, Color::RED);

            gizmos.sphere(
                transform.translation,
                Quat::IDENTITY,
                ball.radius,
                Color::RED,
            );
        }
    }
}

mod environment_colliders {
    use gbp_environment::Environment;

    use super::*;
    pub(super) fn enabled(config: Res<Config>) -> bool {
        config.visualisation.draw.environment_colliders
    }

    pub(super) fn render(
        mut gizmos: Gizmos,
        env_colliders: Res<gbp_global_planner::Colliders>,
        // config: Res<Config>,
        env_config: Res<Environment>,
    ) {
        // let height = config.visualisation.height.objects;
        let height = -env_config.tiles.settings.obstacle_height;

        for collider in env_colliders.iter() {
            let aabb = collider.aabb();
            let center = aabb.center();
            // let height

            let translation = Vec3::new(center.x, height / 2.0, center.y);
            let half_extents = aabb.half_extents();
            let aabb = Transform {
                translation,
                scale: Vec3::new(half_extents.x * 2.0, height, half_extents.y * 2.0),
                ..Default::default()
            };
            gizmos.cuboid(aabb, Color::ORANGE_RED);
        }
    }
}
