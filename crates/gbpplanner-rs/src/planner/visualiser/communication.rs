//! A **Bevy** Plugin for visualising the communication graph between robots

use bevy::prelude::*;
use gbp_config::Config;

use super::super::RobotConnections;
use crate::{
    planner::robot::RadioAntenna,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

/// A **Bevy** Plugin for visualising the communication graph between robots
pub struct CommunicationGraphVisualiserPlugin;

impl Plugin for CommunicationGraphVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_communication_graph_v3.run_if(enabled));
    }
}

/// Used to check if the communication graph should be drawn
#[inline]
fn enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.communication_graph
}

// /// Used to keep track of which undirected edges have already been drawn
// /// to avoid double-drawing them
// #[derive(Default)]
// struct Edges(Vec<(Entity, Entity)>);
//
// impl Edges {
//     /// Check if the edge exists in the set
//     /// Edges are undirected, therefore true is returned if either (a, b) or
// (b,     /// a) exists
//     fn contains(&self, edge: &(Entity, Entity)) -> bool {
//         self.0.contains(edge) || self.0.contains(&(edge.1, edge.0))
//     }
// }
//
// fn draw_communication_graph(
//     mut gizmos: Gizmos,
//     catppuccin_theme: Res<CatppuccinTheme>,
//     robots_query: Query<(Entity, &RobotState, &Transform)>,
//     mut edges: Local<Edges>,
// ) {
//     let connected_color =
// Color::from_catppuccin_colour(catppuccin_theme.green());     // let
// disconnected_color =     // Color::from_catppuccin_colour(catppuccin_theme.
// red());
//
//     // # Safety
//     // Only this system has access to `edges`, as it is `Local<T>` state
//     // Instead of heap allocating a new Vec for each call to this function
//     // we reuse the space allocated in previous calls
//     #[allow(clippy::undocumented_unsafe_blocks)]
//     unsafe {
//         edges.0.set_len(0);
//     }
//
//     for (robot_id, robot_state, transform) in &robots_query {
//         if !robot_state.interrobot_comms_active {
//             debug!("interrobot_comms_active is false, for robot: {:?}",
// robot_id);             continue;
//         }
//
//         for connected_with_id in &robot_state.ids_of_robots_connected_with {
//             if let Some((_, _, other_transform)) =
// robots_query.iter().find(|(id, _, _)| id == connected_with_id) {
// let edge = (robot_id, *connected_with_id);                 if
// edges.contains(&edge) {                     continue;
//                 }
//
//                 edges.0.push(edge);
//
//                 debug!("drawing line between {:?} and {:?}", robot_id,
// connected_with_id);                 gizmos.line(transform.translation,
// other_transform.translation, connected_color);             }
//         }
//     }
// }

fn draw_communication_graph_v3(
    mut gizmos: Gizmos,
    catppuccin_theme: Res<CatppuccinTheme>,
    query: Query<(Entity, &RobotConnections, &RadioAntenna, &Transform)>,
) {
    let connected_color = Color::from_catppuccin_colour(catppuccin_theme.green());
    let disconnected_color = Color::from_catppuccin_colour(catppuccin_theme.red());

    for (_, robot_state, antenna, transform) in &query {
        let color = if antenna.active {
            &connected_color
        } else {
            &disconnected_color
        };

        for connected_with_id in &robot_state.robots_connected_with {
            let Ok((_, _, _, other_transform)) = query.get(*connected_with_id) else {
                continue;
            };

            let halfway_point = (transform.translation + other_transform.translation) / 2.;
            gizmos.line(transform.translation, halfway_point, *color);
        }
    }
}
