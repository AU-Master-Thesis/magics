use std::collections::HashMap;

use crate::planner::{FactorGraph, NodeIndex, NodeKind, RobotId, RobotState};

use super::super::theme::ThemeEvent;
use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

#[derive(Component)]
pub struct GeneralInputs;

pub struct GeneralInputPlugin;

impl Plugin for GeneralInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<GeneralAction>::default(),))
            .add_systems(PostStartup, (bind_general_input,))
            .add_systems(Update, (general_actions,));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum GeneralAction {
    ToggleTheme,
    ExportGraph,
}

impl GeneralAction {
    fn default_keyboard_input(action: GeneralAction) -> Option<UserInput> {
        match action {
            Self::ToggleTheme => Some(UserInput::Single(InputKind::Keyboard(KeyCode::T))),
            Self::ExportGraph => Some(UserInput::Single(InputKind::Keyboard(KeyCode::G))),
        }
    }
}

fn bind_general_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for action in GeneralAction::variants() {
        if let Some(input) = GeneralAction::default_keyboard_input(action) {
            input_map.insert(input, action);
        }
    }

    commands.spawn((
        InputManagerBundle {
            input_map,
            ..Default::default()
        },
        GeneralInputs,
    ));
}

fn general_actions(
    mut theme_event: EventWriter<ThemeEvent>,
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
    query_graphs: Query<(Entity, &FactorGraph), With<RobotState>>,
) {
    if let Ok(action_state) = query.get_single() {
        if action_state.just_pressed(GeneralAction::ToggleTheme) {
            info!("Toggling theme");
            theme_event.send(ThemeEvent);
        }

        if action_state.just_pressed(GeneralAction::ExportGraph) {
            info!("Exporting all graphs");

            // for (i, graph) in query_graphs.iter().enumerate() {
            //     info!("Exporting graph {}", i);
            //     std::fs::write(format!("graph_{}.dot", i), graph.export().as_bytes())
            //         .unwrap();
            //     // let graph_viz = graph.export();
            // }
            let mut output = String::new();
            output.push_str("graph {\n");
            output.push_str("    node [style=filled];\n");
            output.push_str("    layout=neato;\n");

            let mut all_external_connections =
                HashMap::<RobotId, HashMap<usize, RobotId>>::new();

            for (robot_id, graph) in query_graphs.iter() {
                output.push_str(&format!("    subgraph cluster_{:?} {{\n", robot_id));

                // output.push_str(&graph.export());
                let (nodes, edges) = graph.export_data();

                // Add all nodes
                for node in nodes.iter() {
                    output.push_str(&format!(
                        "        \"{:?}_{:?}\" [label=\"{:?}\", fillcolor=\"{}\", shape={}, width={}]\n",
                        robot_id,
                        node.index,
                        node.index,
                        node.color(),
                        node.shape(),
                        node.width()
                    ));
                }

                output.push_str("    }\n");

                // Add all internal edges
                for edge in edges {
                    output.push_str(&format!(
                        "    \"{:?}_{:?}\" -- \"{:?}_{:?}\"\n",
                        robot_id, edge.from, robot_id, edge.to
                    ));
                }

                // Add all external edges
                // where the `Node` is a `Factor` of the `InterRobot` kind and it's value is the `NodeIndex` of the other robot (`Node`) to which it is connected
                // let mut external_connections =
                //     HashMap::<(RobotId, usize), RobotId>::new();
                // for node in nodes.iter() {
                //     match node.kind {
                //         NodeKind::InterRobotFactor(other_robot_id) => {
                //             // output.push_str(&format!(
                //             //     "    \"{:?}_{:?}\" -- \"{:?}_{:?}\"\n",
                //             //     robot_id, node.index, other_robot_id, node
                //             // ));
                //             external_connections
                //                 .insert((robot_id, node.index), other_robot_id);
                //         }
                //         _ => {}
                //     }
                // }

                // for ((from_robot_id, from_node), to_robot_id) in external_connections.iter() {
                //     // find counterpart node to `to_robot_id`
                // }

                let mut external_connections = HashMap::<usize, RobotId>::new();

                for node in nodes.iter() {
                    match node.kind {
                        NodeKind::InterRobotFactor(other_robot_id) => {
                            external_connections.insert(node.index, other_robot_id);
                        }
                        _ => {}
                    }
                }

                all_external_connections.insert(robot_id, external_connections);
            }

            for (from_robot_id, from_connections) in all_external_connections.iter() {
                for (from_node, to_robot_id) in from_connections.iter() {
                    let to_connections =
                        all_external_connections.get(to_robot_id).unwrap();
                    // let to_robot_id = to_connections.get(from_node).unwrap();

                    let to_node = to_connections
                        .iter()
                        .find(|(_, robot_id)| from_robot_id == *robot_id)
                        .map(|(node, _)| node)
                        .unwrap();

                    // let to_variable =

                    output.push_str(&format!(
                        "    \"{:?}_{:?}\" -- \"{:?}_{:?}\" [len=10]\n",
                        from_robot_id, from_node, to_robot_id, to_node
                    ));

                    // if from_robot_id == to_robot_id {
                    //     output.push_str(&format!(
                    //         "    \"{:?}_{:?}\" -- \"{:?}_{:?}\"\n",
                    //         from_robot_id, from_node, to_robot_id, from_node
                    //     ));
                    // }

                    // output.push_str(&format!(
                    //     "    \"{:?}_{:?}\" -- \"{:?}_{:?}\"\n",
                    //     from_robot_id, from_node, to_robot_id, to_node
                    // ));
                }
            }

            output.push_str("}\n");

            std::fs::write("graph.dot", output.as_bytes()).unwrap();
        }
    }
}
