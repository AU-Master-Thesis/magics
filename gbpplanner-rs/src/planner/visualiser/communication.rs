use super::{super::RobotState, LineSegment, Path, Z_FIGHTING_OFFSET};
use crate::{
    asset_loader::SceneAssets, config::Config, theme::CatppuccinTheme,
    theme::ColorFromCatppuccinColourExt,
};

use bevy::prelude::*;

pub struct CommunicationGraphVisualiserPlugin;

impl Plugin for CommunicationGraphVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // init_communication_graph, // TODO: when [`Path`] is no updatable
                draw_communication_graph,
                // show_or_hide_communication_graph, // TODO: when [`Path`] is no updatable
            ),
        );
    }
}

/// A **Bevy** [`Component`] to mark an entity as a visualised _communication graph_
#[derive(Component)]
pub struct CommunicationGraphVisualiser;

/// A **Bevy** [`Update`] system
/// Draws the communication graph with a [`Path`], through a [`PbrBundle`] and [`CommunicationGraphVisualiser`] component
///
/// Draws a line segment between each robot and its neighbours
/// A robot is a neighbour if it is within the communication range `config.communication.range`
///
/// However, if the robot's comms are off `RobotState.interrobot_comms_active == false`, it will not draw a line segment
fn draw_communication_graph(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query_previous_line_segments: Query<Entity, With<LineSegment>>,
    robots_query: Query<(Entity, &RobotState, &Transform)>,
    config: Res<Config>,
    catppuccin_theme: Res<CatppuccinTheme>,
    scene_assets: Res<SceneAssets>,
) {
    // If there are no robots, return
    if robots_query.iter().count() == 0 {
        return;
    }

    // Remove previous lines
    query_previous_line_segments.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });

    // If we're not supposed to draw the communication graph, return
    if !config.visualisation.draw.communication_graph {
        return;
    }

    // necessary to remake the line material, as it needs to change with the theme
    let line_material = materials.add(Color::from_catppuccin_colour(catppuccin_theme.yellow()));

    // TODO: Don't double-draw lines from and to the same two robots
    for (robot_id, robot_state, transform) in robots_query.iter() {
        // if !robot_state.interrobot_comms_active {
        //     continue;
        // }

        // Find all neighbour transforms within the communication range
        // but filter out all robots that do not have comms on
        let neighbours = robots_query
            .iter()
            .filter(|(other_robot_id, other_robot_state, _)| {
                robot_id != *other_robot_id //&& !other_robot_state.interrobot_comms_active
            })
            .filter_map(|(_, _, other_transform)| {
                let distance = transform.translation.distance(other_transform.translation);
                if distance < config.robot.communication.radius {
                    Some(other_transform.translation)
                } else {
                    None
                }
            })
            .collect::<Vec<Vec3>>();

        if neighbours.is_empty() {
            continue;
        }

        // Make a line for each neighbour
        for neighbour_transform in neighbours {
            let line = Path::new(vec![
                transform.translation + Vec3::new(0.0, Z_FIGHTING_OFFSET, 0.0),
                neighbour_transform + Vec3::new(0.0, Z_FIGHTING_OFFSET, 0.0),
            ])
            .with_width(0.2);
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(line)),
                    material: line_material.clone(),
                    ..Default::default()
                },
                LineSegment,
            ));
            // gizmos.primitive_3d(
            //     Polyline3d::<100>::new(vec![transform.translation, neighbour_transform]),
            //     Vec3::ZERO,
            //     Quat::IDENTITY,
            //     Color::from_catppuccin_colour(catppuccin_theme.yellow()),
            // );
        }
    }
}
