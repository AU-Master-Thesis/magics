use bevy::prelude::*;

use super::{super::robot::Waypoints, RobotTracker};
use crate::{
    asset_loader::SceneAssets,
    config::{Config, DrawSetting},
    ui::DrawSettingsEvent,
};

pub struct WaypointVisualiserPlugin;

impl Plugin for WaypointVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (init_waypoints, update_waypoints, show_or_hide_waypoints),
        );
    }
}

/// A **Bevy** [`Component`] to mark an entity as a visualised _waypoint_
#[derive(Component)]
pub struct WaypointVisualiser;

/// A **Bevy** [`Component`] to mark a robot that it has a corresponding
/// `WaypointVis` entity Useful for easy _exclusion_ in queries
#[derive(Component)]
pub struct HasWaypointVisualiser;

/// A **Bevy** [`Update`] system
/// Initialises each new [`Waypoints`] component to have a matching
/// [`PbrBundle`] and [`WaypointVisualiser`] component I.e. if the [`Waypoints`]
/// component already has a [`HasWaypointVisualiser`], it will be ignored
fn init_waypoints(
    mut commands: Commands,
    query: Query<(Entity, &Waypoints), Without<HasWaypointVisualiser>>,
    scene_assets: Res<SceneAssets>,
    config: Res<Config>,
) {
    for (entity, waypoints) in query.iter() {
        // Mark the robot with `RobotHasTracker` to exclude next time
        commands.entity(entity).insert(HasWaypointVisualiser);

        if let Some(next_waypoint) = waypoints.0.front() {
            // info!("Next waypoint: {:?}", next_waypoint);

            let transform = Transform::from_translation(Vec3::new(
                next_waypoint.x,
                config.visualisation.height.objects,
                next_waypoint.y,
            ));
            info!("{:?}: Initialising waypoints at {:?}", entity, transform);

            // Spawn a `RobotTracker` component with a corresponding `PbrBundle`
            commands.spawn((
                WaypointVisualiser,
                super::RobotTracker::new(entity),
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

/// A **Bevy** [`Update`] system
/// Updates the [`Transform`]s of all [`WaypointVisualiser`] entities
/// Done by cross-referencing [`Entity`] with the `RobotTracker.robot_id`
fn update_waypoints(
    mut tracker_query: Query<(&RobotTracker, &mut Transform), With<WaypointVisualiser>>,
    robots_query: Query<(Entity, &Waypoints)>,
    config: Res<Config>,
) {
    // Update the `RobotTracker` components
    for (tracker, mut transform) in tracker_query.iter_mut() {
        for (entity, waypoints) in robots_query.iter() {
            if let Some(next_waypoint) = waypoints.0.front() {
                if tracker.robot_id == entity {
                    // Update the `Transform` component
                    // to match the `Waypoints` component

                    // info!("{:?}: Updating waypoints to {:?}", entity, next_waypoint);
                    transform.translation = Vec3::new(
                        next_waypoint.x,
                        config.visualisation.height.objects,
                        next_waypoint.y,
                    );
                }
            } else {
                // info!("Robot {:?} has no more waypoints", tracker.robot_id);
            }
        }
    }
}

/// A **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::Waypoints` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`WaypointVisualiser`] entities
fn show_or_hide_waypoints(
    mut query: Query<(&WaypointVisualiser, &mut Visibility)>,
    mut draw_setting_event: EventReader<DrawSettingsEvent>,
) {
    for event in draw_setting_event.read() {
        if matches!(event.setting, DrawSetting::Waypoints) {
            for (_, mut visibility) in query.iter_mut() {
                if event.draw {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
