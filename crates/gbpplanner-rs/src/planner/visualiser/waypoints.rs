//! **Bevy** Plugin to visualize robot waypoints
use bevy::prelude::*;

use crate::{
    asset_loader::SceneAssets,
    bevy_utils::run_conditions::event_exists,
    config::{Config, DrawSetting},
    input::DrawSettingsEvent,
    planner::{
        robot::RobotReachedWaypoint,
        spawner::{WaypointCreated, WaypointReached},
        RobotId,
    },
    simulation_loader,
};

/// **Bevy** Plugin to visualize robot waypoints
pub struct WaypointVisualiserPlugin;

impl Plugin for WaypointVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                listen_for_robot_reached_waypoint_event,
                create_waypoint_visualizer,
                delete_mesh_of_reached_waypoints,
                show_or_hide_waypoint_visualizers.run_if(event_exists::<DrawSettingsEvent>),
            ),
        );
    }
}

fn listen_for_robot_reached_waypoint_event(
    mut evr_robot_reached_waypoint: EventReader<RobotReachedWaypoint>,
    mut evw_waypoint_reached: EventWriter<WaypointReached>,
    waypoint_visualizers: Query<(Entity, &AssociatedWithRobot), With<WaypointVisualiser>>,
) {
    for event in evr_robot_reached_waypoint.read() {
        // Find the entity id of the waypoint visualizer that has just been reached
        if let Some(waypoint_id) = waypoint_visualizers
            .iter()
            .find(|(_, AssociatedWithRobot(robot_id))| *robot_id == event.robot_id)
            .map(|(entity, _)| entity)
        {
            evw_waypoint_reached.send(WaypointReached(waypoint_id));
        };
    }
}

/// **Bevy** system to delete the mesh of the allocated waypoint visualizer
/// whenever the waypoint has been reached.
fn delete_mesh_of_reached_waypoints(
    mut commands: Commands,
    mut evr_delete_waypoint: EventReader<WaypointReached>,
) {
    for WaypointReached(vis) in evr_delete_waypoint.read() {
        commands.entity(*vis).despawn();
    }
}

/// **Bevy** Component to store an association to a robot.
/// Used to make it easier to retrieve the entity id, of the robot
/// a visualizer is associated with.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct AssociatedWithRobot(pub RobotId);

fn create_waypoint_visualizer(
    mut commands: Commands,
    config: Res<Config>,
    scene_assets: Res<SceneAssets>,
    mut evr_waypoint_created: EventReader<WaypointCreated>,
) {
    for event in evr_waypoint_created.read() {
        let transform = Transform::from_translation(Vec3::new(
            event.position.x,
            config.visualisation.height.objects,
            event.position.y,
        ));

        commands.spawn((
            simulation_loader::Reloadable,
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
    }
}

/// **Bevy** [`Component`] to mark an entity as a visualised _waypoint_
#[derive(Component)]
pub struct WaypointVisualiser;

/// **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::Waypoints` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`WaypointVisualiser`] entities
fn show_or_hide_waypoint_visualizers(
    mut visualizers: Query<&mut Visibility, With<WaypointVisualiser>>,
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, DrawSetting::Waypoints) {
            for mut visibility in &mut visualizers {
                if event.draw {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
