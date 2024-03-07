use bevy::prelude::*;

use crate::asset_loader::SceneAssets;

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
                update_waypoints,
                init_factorgraphs,
                update_factorgraphs,
            ),
        );
    }
}

/// A **Bevy** `Component` for keeping track of a robot
/// Keeps track of the `RobotId` and `Vec2` position
#[derive(Component)]
pub struct RobotTracker {
    pub robot_id: RobotId,
    pub variable_id: usize,
}

impl RobotTracker {
    pub fn new(robot_id: RobotId) -> Self {
        Self {
            robot_id,
            variable_id: 0,
        }
    }

    pub fn with_variable_id(mut self, id: usize) -> Self {
        self.variable_id = id;
        self
    }
}

/// A **Bevy** `Component` to mark an entity as a visualised waypoint
#[derive(Component)]
pub struct WaypointVisualiser;

/// A **Bevy** `Component` to mark an entity as a visualised factor graph
#[derive(Component)]
pub struct FactorGraphVisualiser;

/// A **Bevy** `Component` to mark a robot that it has a corresponding `WaypointVis` entity
/// Useful for easy exclusion in queries
#[derive(Component)]
pub struct HasWaypointVisualiser;

/// A **Bevy** `Component` to mark a robot that it has a corresponding `FactorGraphVis` entity
/// Useful for easy exclusion in queries
#[derive(Component)]
pub struct HasFactorGraphVisualiser;

/// A **Bevy** `Update` system
/// Initialises each new `FactorGraph` component to have a matching `PbrBundle` and `FactorGraphVisualiser` component
/// I.e. if the `FactorGraph` component already has a `FactorGraphVisualiser`, it will be ignored
fn init_factorgraphs(
    mut commands: Commands,
    query: Query<(Entity, &super::FactorGraph), Without<HasFactorGraphVisualiser>>,
    scene_assets: Res<SceneAssets>,
) {
    for (entity, factorgraph) in query.iter() {
        // Mark the robot with `HasFactorGraphVisualiser` to exclude next time
        commands.entity(entity).insert(HasFactorGraphVisualiser);

        factorgraph.variables().for_each(|v| {
            let mean = v.belief.mean();
            let transform = Vec3::new(mean[0] as f32, 0.0, mean[1] as f32);

            info!("{:?}: Initialising variable at {:?}", entity, transform);

            // Spawn a `FactorGraphVisualiser` component with a corresponding `PbrBundle`
            commands.spawn((
                RobotTracker::new(entity).with_variable_id(v.get_node_index().index()),
                FactorGraphVisualiser,
                PbrBundle {
                    mesh: scene_assets.meshes.variable.clone(),
                    material: scene_assets.materials.variable.clone(),
                    transform: Transform::from_translation(transform),
                    ..Default::default()
                },
            ));
        });
    }
}

/// A **Bevy** `Update` system
/// Updates the `Transform`s of all `FactorGraphVisualiser` entities
/// Done by cross-referencing with the `FactorGraph` components
/// that have matching `Entity` with the `RobotTracker.robot_id`
/// and variables in the `FactorGraph` that have matching `RobotTracker.variable_id`
fn update_factorgraphs(
    mut tracker_query: Query<(&RobotTracker, &mut Transform), With<FactorGraphVisualiser>>,
    factorgraph_query: Query<(Entity, &super::FactorGraph)>,
) {
    // Update the `RobotTracker` components
    for (tracker, mut transform) in tracker_query.iter_mut() {
        for (entity, factorgraph) in factorgraph_query.iter() {
            // continue if we're not looking at the right robot
            if tracker.robot_id != entity {
                continue;
            }

            // else look through the variables
            for v in factorgraph.variables() {
                // continue if we're not looking at the right variable
                if v.get_node_index().index() != tracker.variable_id {
                    continue;
                }

                info!("{:?}: Updating variable to {:?}", entity, v.belief.mean());

                // else update the transform
                let mean = v.belief.mean();
                transform.translation = Vec3::new(mean[0] as f32, 0.0, mean[1] as f32);
            }
        }
    }
}

/// A **Bevy** `Update` system
/// Initialises each new `Waypoints` component to have a matching `PbrBundle` and `RobotTracker` component
/// I.e. if the `Waypoints` component already has a `RobotTracker`, it will be ignored
fn init_waypoints(
    mut commands: Commands,
    query: Query<(Entity, &Waypoints), Without<HasWaypointVisualiser>>,
    scene_assets: Res<SceneAssets>,
) {
    for (entity, waypoints) in query.iter() {
        // Mark the robot with `RobotHasTracker` to exclude next time
        commands.entity(entity).insert(HasWaypointVisualiser);

        if let Some(next_waypoint) = waypoints.0.front() {
            // info!("Next waypoint: {:?}", next_waypoint);

            let transform =
                Transform::from_translation(Vec3::new(next_waypoint.x, 0.0, next_waypoint.y));
            info!("{:?}: Initialising waypoints at {:?}", entity, transform);

            // Spawn a `RobotTracker` component with a corresponding `PbrBundle`
            commands.spawn((
                WaypointVisualiser,
                RobotTracker::new(entity),
                PbrBundle {
                    mesh: scene_assets.meshes.waypoint.clone(),
                    material: scene_assets.materials.waypoint.clone(),
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
/// Updates the `Transform`s of all `WaypointVisualiser` entities
/// Done by cross-referencing `Entity` with the `RobotTracker.robot_id`
fn update_waypoints(
    mut tracker_query: Query<(&RobotTracker, &mut Transform), With<WaypointVisualiser>>,
    robots_query: Query<(Entity, &Waypoints)>,
) {
    // Update the `RobotTracker` components
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
        }
    }
}
