#![warn(missing_docs)]

use bevy::prelude::*;

use super::super::RobotState;
use crate::{
    config::Config,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

/// A **Bevy** Plugin for visualising the communication graph between robots
pub struct CommunicationGraphVisualiserPlugin;

impl Plugin for CommunicationGraphVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_communication_graph_v2.run_if(draw_communication_graph_enabled),),
        );
    }
}

/// Used to check if the communication graph should be drawn
fn draw_communication_graph_enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.communication_graph
}

/// Used to keep track of which undirected edges have already been drawn
/// to avoid double-drawing them
#[derive(Default)]
struct Edges(Vec<(Entity, Entity)>);

impl Edges {
    /// Check if the edge exists in the set
    /// Edges are undirected, therefore true is returned if either (a, b) or (b, a) exists
    fn contains(&self, edge: &(Entity, Entity)) -> bool {
        self.0.contains(edge) || self.0.contains(&(edge.1, edge.0))
    }
}

fn draw_communication_graph_v2(
    mut gizmos: Gizmos,
    catppuccin_theme: Res<CatppuccinTheme>,
    robots_query: Query<(Entity, &RobotState, &Transform)>,
    mut edges: Local<Edges>,
) {
    let color = Color::from_catppuccin_colour(catppuccin_theme.yellow());

    // # Safety
    // Only this system has access to `edges`, as it is `Local<T>` state
    // Instead of heap allocating a new Vec for each call to this function
    // we reuse the space allocated in previous calls
    unsafe {
        edges.0.set_len(0);
    }

    for (robot_id, robot_state, transform) in robots_query.iter() {
        if !robot_state.interrobot_comms_active {
            debug!(
                "interrobot_comms_active is false, for robot: {:?}",
                robot_id
            );
            continue;
        }

        for connected_with_id in robot_state.ids_of_robots_connected_with.iter() {
            if let Some((_, _, other_transform)) = robots_query
                .iter()
                .find(|(id, _, _)| id == connected_with_id)
            {
                let edge = (robot_id, *connected_with_id);
                if edges.contains(&edge) {
                    continue;
                }

                edges.0.push(edge);

                debug!(
                    "drawing line between {:?} and {:?}",
                    robot_id, connected_with_id
                );
                gizmos.line(transform.translation, other_transform.translation, color);
            }
        }
    }
}

// / A **Bevy** [`Update`] system
// / Draws the communication graph with a [`Path`], through a [`PbrBundle`] and
// / [`CommunicationGraphVisualiser`] component
// /
// / Draws a line segment between each robot and its neighbours
// / A robot is a neighbour if it is within the communication range
// / `config.communication.range`
// /
// / However, if the robot's comms are off `RobotState.interrobot_comms_active ==
// / false`, it will not draw a line segment
// fn draw_communication_graph(
//     mut gizmos: Gizmos,
//     mut commands: Commands,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     query_previous_line_segments: Query<Entity, With<LineSegment>>,
//     robots_query: Query<(Entity, &RobotState, &Transform)>,
//     config: Res<Config>,
//     catppuccin_theme: Res<CatppuccinTheme>,
// ) {
//     // If there are no robots, return
//     if robots_query.iter().count() == 0 {
//         return;
//     }

//     // Remove previous lines
//     query_previous_line_segments.iter().for_each(|entity| {
//         commands.entity(entity).despawn();
//     });

//     // If we're not supposed to draw the communication graph, return
//     // if !config.visualisation.draw.communication_graph {
//     //     return;
//     // }

//     // necessary to remake the line material, as it needs to change with the theme
//     let color = Color::from_catppuccin_colour(catppuccin_theme.yellow());
//     let line_material = materials.add(color);

//     let communication_radius = config.robot.communication.radius.get();

//     // TODO: Don't double-draw lines from and to the same two robots
//     for (robot_id, robot_state, transform) in robots_query.iter() {
//         if !robot_state.interrobot_comms_active {
//             continue;
//         }

//         // Find all neighbour transforms within the communication range
//         // but filter out all robots that do not have comms on
//         let neighbours = robots_query
//             .iter()
//             .filter(|(other_robot_id, other_robot_state, _)| {
//                 robot_id != *other_robot_id && !other_robot_state.interrobot_comms_active
//             })
//             .filter_map(|(_, _, other_transform)| {
//                 let distance = transform.translation.distance(other_transform.translation);
//                 if distance < communication_radius {
//                     Some(other_transform.translation)
//                 } else {
//                     None
//                 }
//             })
//             .collect::<Vec<Vec3>>();

//         if neighbours.is_empty() {
//             continue;
//         }

//         // Make a line for each neighbour
//         for neighbour_transform in neighbours {
//             let line = Path::new(vec![
//                 transform.translation + Vec3::new(0.0, Z_FIGHTING_OFFSET, 0.0),
//                 neighbour_transform + Vec3::new(0.0, Z_FIGHTING_OFFSET, 0.0),
//             ])
//             .with_width(0.2);
//             commands.spawn((
//                 PbrBundle {
//                     mesh: meshes.add(Mesh::from(line)),
//                     material: line_material.clone(),
//                     ..Default::default()
//                 },
//                 LineSegment,
//             ));
//             // gizmos.primitive_3d(
//             //     Polyline3d::<100>::new(vec![transform.translation,
//             // neighbour_transform]),     Vec3::ZERO,
//             //     Quat::IDENTITY,
//             //     Color::from_catppuccin_colour(catppuccin_theme.yellow()),
//             // );
//         }
//     }
// }
