use bevy::prelude::*;

use crate::{
    asset_loader::SceneAssets,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

use super::{robot::Waypoints, RobotId};

/// A **Bevy** `Plugin` for visualising aspects of the planner
/// Includes visualising parts of the factor graph
pub struct VisualiserPlugin;

impl Plugin for VisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_waypoints,
                draw_waypoints,
                // draw_f
            ),
        );
    }
}

/// A **Bevy** `Component` for a waypoint
/// Keeps track of the `RobotId` and `Vec2` position
#[derive(Component)]
pub struct RobotTracker {
    pub robot_id: RobotId,
}

/// A **Bevy** `Component` to mark a robot that it has a corresponding `RobotTracker` component
/// This is for easy exclusion in queries
#[derive(Component)]
pub struct RobotHasTracker;

/// A **Bevy** `Update` system
/// Initialises each new `Waypoints` component to have a matching `PbrBundle` and `RobotTracker` component
/// I.e. if the `Waypoints` component already has a `RobotTracker`, it will be ignored
fn init_waypoints(
    mut commands: Commands,
    query: Query<(Entity, &Waypoints), Without<RobotHasTracker>>,
    scene_assets: Res<SceneAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    for (entity, waypoints) in query.iter() {
        // Mark the robot with `RobotHasTracker`
        // to exclude next time
        commands.entity(entity).insert(RobotHasTracker);

        if let Some(next_waypoint) = waypoints.0.front() {
            // info!("Next waypoint: {:?}", next_waypoint);

            let transform =
                Transform::from_translation(Vec3::new(next_waypoint.x, 0.0, next_waypoint.y));
            info!("{:?}: Initialising waypoints at {:?}", entity, transform);

            // Spawn a `RobotTracker` component with a corresponding `PbrBundle`
            commands.spawn((
                RobotTracker { robot_id: entity },
                PbrBundle {
                    mesh: scene_assets.meshes.waypoint.clone(),
                    material: scene_assets.materials.waypoint.clone(),
                    // mesh: mesh.clone(),
                    // material: material.clone(),
                    transform,
                    ..Default::default()
                },
            ));
        } else {
            info!("No waypoints for robot {:?}", entity);
        }
    }
}

/// A **Bevy** `Update` system
/// Updates all `Transform` components that also have a `RobotTracker` component
/// Queries all entities with `Waypoints` and `RobotState` components
/// Uses the `Entity` as
fn draw_waypoints(
    mut tracker_query: Query<(&RobotTracker, &mut Transform)>,
    robots_query: Query<(Entity, &Waypoints)>,
) {
    // Update the `RobotTracker` components
    // by cross-referencing with the `Waypoints` components
    // that have matching `Entity` with the `RobotTracker.robot_id`
    for (tracker, mut transform) in tracker_query.iter_mut() {
        for (entity, waypoints) in robots_query.iter() {
            if let Some(next_waypoint) = waypoints.0.front() {
                if tracker.robot_id == entity {
                    // Update the `Transform` component
                    // to match the `Waypoints` component

                    // info!("{:?}: Updating waypoints to {:?}", entity, next_waypoint);
                    transform.translation = Vec3::new(next_waypoint.x, 0.0, next_waypoint.y);
                }
            } else {
                info!("Robot {:?} has no more waypoints", tracker.robot_id);
            }
            // if tracker.robot_id == waypoints.robot_id {
            //     // Update the `Transform` component
            //     // to match the `Waypoints` component
            //     transform.translation = Vec3::new(waypoints.position.x, 0.0, waypoints.position.y);
            // }
        }
    }
}
