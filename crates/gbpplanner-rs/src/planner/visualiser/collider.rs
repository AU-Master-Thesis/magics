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

    const COLOR: Color = Color::ORANGE_RED;

    fn render_triangle(
        mut gizmos: &mut Gizmos,
        height: f32,
        isometry: &parry2d::na::Isometry2<f32>,
        triangle: &parry2d::shape::Triangle,
    ) {
        let top_surface = triangle
            .vertices()
            .iter()
            .cycle()
            .take(4)
            .map(|v| Vec3::new(v.x, height / 2.0, v.y));
        let bottom_surface = triangle
            .vertices()
            .iter()
            .cycle()
            .take(4)
            .map(|v| Vec3::new(v.x, -height / 2.0, v.y));

        gizmos.linestrip(top_surface, COLOR);
        gizmos.linestrip(bottom_surface, COLOR);

        for v in triangle.vertices() {
            let start = Vec3::new(v.x, height / 2.0, v.y);
            let end = Vec3::new(v.x, -height / 2.0, v.y);
            gizmos.line(start, end, COLOR);
        }
    }

    fn render_rectangle(
        mut gizmos: &mut Gizmos,
        height: f32,
        collider: &gbp_global_planner::Collider,
    ) {
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
        gizmos.cuboid(aabb, COLOR);
    }

    fn render_circle(
        mut gizmos: &mut Gizmos,
        height: f32,
        isometry: &parry2d::na::Isometry2<f32>,
        ball: &parry2d::shape::Ball,
    ) {
        let radius = ball.radius;
        let center_x = isometry.translation.x;
        let center_y = isometry.translation.y;
        let normal = Direction3d::Y;
        // top circle
        let position = Vec3::new(center_x, height, center_y);
        let segments = 32;
        gizmos
            .circle(position, normal, radius, COLOR)
            .segments(segments);
        // bottom circle
        let position = Vec3::new(center_x, 0., center_y);
        gizmos
            .circle(position, normal, radius, COLOR)
            .segments(segments);

        // draw 4 vertical lines 90degree spread apart around the cylinder
        for i in 0..4 {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / 4.0;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            let position = Vec3::new(center_x + x, height, center_y + y);
            gizmos.line(position, position + Vec3::new(0., -height, 0.), COLOR);
        }
    }
    fn render_convex_polygon(
        mut gizmos: &mut Gizmos,
        height: f32,
        isometry: &parry2d::na::Isometry2<f32>,
        collider: &parry2d::shape::ConvexPolygon,
    ) {
    }

    pub(super) fn render(
        mut gizmos: Gizmos,
        env_colliders: Res<gbp_global_planner::Colliders>,
        // config: Res<Config>,
        env_config: Res<Environment>,
    ) {
        // let height = config.visualisation.height.objects;
        let height = -env_config.tiles.settings.obstacle_height;
        let color = Color::ORANGE_RED;

        for collider @ gbp_global_planner::Collider {
            associated_mesh,
            isometry,
            shape,
        } in env_colliders.iter()
        {
            if let Some(triangle) = shape.downcast_ref::<parry2d::shape::Triangle>() {
                render_triangle(&mut gizmos, height, isometry, triangle);
            } else if let Some(circle) = shape.downcast_ref::<parry2d::shape::Ball>() {
                render_circle(&mut gizmos, height, isometry, circle);
            } else if let Some(convex_polygon) =
                shape.downcast_ref::<parry2d::shape::ConvexPolygon>()
            {
                render_convex_polygon(&mut gizmos, height, isometry, convex_polygon);
            } else {
                render_rectangle(&mut gizmos, height, collider);
                // // gizmos.
                // let aabb = collider.aabb();
                // let center = aabb.center();
                // // let height
                //
                // let translation = Vec3::new(center.x, height / 2.0,
                // center.y); let half_extents =
                // aabb.half_extents(); let aabb = Transform {
                //     translation,
                //     scale: Vec3::new(half_extents.x * 2.0, height,
                // half_extents.y * 2.0),
                //     ..Default::default()
                // };
                // gizmos.cuboid(aabb, Color::ORANGE_RED);
            }
        }
    }
}
