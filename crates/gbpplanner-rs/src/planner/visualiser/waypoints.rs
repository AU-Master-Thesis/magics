#![deny(missing_docs)]

//! ...
use bevy::prelude::*;

use crate::{
    asset_loader::SceneAssets,
    config::{Config, DrawSetting},
    planner::{
        robot::RobotReachedWaypoint,
        spawner::{CreateWaypointEvent, DeleteWaypointEvent},
        RobotId,
    },
    ui::DrawSettingsEvent,
};

pub struct WaypointVisualiserPlugin;

impl Plugin for WaypointVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                listen_for_robot_reached_waypoint_event,
                create_waypoint_mesh,
                delete_waypoint_mesh,
                show_or_hide_waypoints_meshes,
            ),
        );
    }
}

fn listen_for_robot_reached_waypoint_event(
    mut robot_reached_waypoint_event: EventReader<RobotReachedWaypoint>,
    mut delete_waypoint_event: EventWriter<DeleteWaypointEvent>,
    query_waypoints: Query<(Entity, &AssociatedWithRobot), With<WaypointVisualiser>>,
) {
    for event in robot_reached_waypoint_event.read() {
        // Find the
        if let Some(waypoint_id) = query_waypoints
            .iter()
            .find(|(_, AssociatedWithRobot(robot_id))| *robot_id == event.robot_id)
            .map(|(entity, _)| entity)
        {
            // info!("sending delete waypoint event: {:?}", waypoint_id);
            delete_waypoint_event.send(DeleteWaypointEvent(waypoint_id));
        };
    }
}

fn delete_waypoint_mesh(
    mut commands: Commands,
    mut delete_waypoint_event: EventReader<DeleteWaypointEvent>,
) {
    for event in delete_waypoint_event.read() {
        commands.entity(event.0).despawn();
        // info!("deleted waypoint {:?}", event.0);
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct AssociatedWithRobot(pub RobotId);

fn create_waypoint_mesh(
    mut commands: Commands,
    config: Res<Config>,
    scene_assets: Res<SceneAssets>,
    mut create_waypoint_event: EventReader<CreateWaypointEvent>,
) {
    for event in create_waypoint_event.read() {
        let transform = Transform::from_translation(Vec3::new(
            event.position.x,
            config.visualisation.height.objects,
            event.position.y,
        ));

        commands.spawn((
            WaypointVisualiser,
            AssociatedWithRobot(event.for_robot),
            PbrBundle {
                mesh: scene_assets.meshes.waypoint.clone(),
                material: scene_assets.materials.waypoint.clone(),
                transform,
                visibility: if config.visualisation.draw.waypoints {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                ..default()
            },
        ));
        // info!(
        //     "created waypoint at {:?} for robot {:?}",
        //     event.position, event.for_robot
        // );
    }
}

/// A **Bevy** [`Component`] to mark an entity as a visualised _waypoint_
#[derive(Component)]
pub struct WaypointVisualiser;

/// A **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::Waypoints` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`WaypointVisualiser`] entities
fn show_or_hide_waypoints_meshes(
    mut query: Query<(&WaypointVisualiser, &mut Visibility)>,
    mut draw_settings_event: EventReader<DrawSettingsEvent>,
) {
    for event in draw_settings_event.read() {
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
