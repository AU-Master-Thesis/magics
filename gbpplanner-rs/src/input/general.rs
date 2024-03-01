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
            .add_systems(Update, (general_actions_system,));
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

fn export_factorgraphs_as_graphviz(
    query: Query<(Entity, &FactorGraph), With<RobotState>>,
) -> String {
    let external_edge_length = 1.0;
    let internal_edge_length = 1.0;
    let mut buf = String::with_capacity(4 * 1024); // 4 kB
    let mut append_line_to_output = |line: &str| {
        buf.push_str(line);
        buf.push('\n');
    };
    append_line_to_output("graph {");
    append_line_to_output("  node [style=filled];");
    append_line_to_output("  layout=neato;");

    let mut all_external_connections =
        HashMap::<RobotId, HashMap<usize, RobotId>>::with_capacity(query.iter().len());

    for (robot_id, factorgraph) in query.iter() {
        let (nodes, edges) = factorgraph.export_data();

        append_line_to_output(&format!("  subgraph cluster_{:?} {{", robot_id));
        // Add all nodes
        for node in nodes.iter() {
            let pos = match node.kind {
                NodeKind::Variable { x, y } => Some((x, y)),
                _ => None,
            };

            let line = {
                let mut line = String::with_capacity(32);
                line.push_str(&format!(
                    r#""{:?}_{:?} [label="{:?}", fillcolor="{}", shape={}, width={}""#,
                    robot_id,
                    node.index,
                    node.index,
                    node.color(),
                    node.shape(),
                    node.width()
                ));
                if let Some((x, y)) = pos {
                    line.push_str(&format!(r#", pos="{x}, {y}""#));
                }
                line.push(']');
                line
            };

            append_line_to_output(&line);
        }
        append_line_to_output("}");

        append_line_to_output("");
        // Add all internal edges
        for edge in edges.iter() {
            let line = format!(
                r#""{:?}_{:?}" -- "{:?}_{:?}""#,
                robot_id, edge.from, robot_id, edge.to
            );
            append_line_to_output(&line);
        }

        let external_connections: HashMap<usize, RobotId> = nodes
            .into_iter()
            .filter_map(|node| match node.kind {
                NodeKind::InterRobotFactor {
                    other_robot_id,
                    variable_index_in_other_robot,
                } => Some((node.index, other_robot_id)),

                _ => None,
            })
            .collect();

        all_external_connections.insert(robot_id, external_connections);
    }

    for (from_robot_id, from_connections) in all_external_connections.iter() {
        for (from_node, to_robot_id) in from_connections.iter() {
            let to_connections = all_external_connections.get(to_robot_id).unwrap();
            // let to_robot_id = to_connections.get(from_node).unwrap();

            let to_node = to_connections
                .iter()
                .find(|(_, robot_id)| from_robot_id == *robot_id)
                .map(|(node, _)| node)
                .unwrap();

            // let to_variable =

            // buf.push_str(&format!(
            //     "    \"{:?}_{:?}\" -- \"{:?}_{:?}\" [len=10]\n",
            //     from_robot_id, from_node, to_robot_id, to_node
            // ));
        }
    }

    append_line_to_output("}"); // closing '}' for starting "graph {"
    buf
}

fn general_actions_system(
    mut theme_event: EventWriter<ThemeEvent>,
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
    query_graphs: Query<(Entity, &FactorGraph), With<RobotState>>,
) {
    let Ok(action_state) = query.get_single() else {
        return;
    };

    if action_state.just_pressed(GeneralAction::ToggleTheme) {
        info!("Toggling theme");
        theme_event.send(ThemeEvent);
    }

    if action_state.just_pressed(GeneralAction::ExportGraph) {
        let output_path = std::path::Path::new("factorgraphs.dot");
        info!("Exporting all factorgraphs to ./{:#?}", output_path);
        let output = export_factorgraphs_as_graphviz(query_graphs);
        std::fs::write(output_path, output.as_bytes()).unwrap();
    }
}
