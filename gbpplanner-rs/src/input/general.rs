use std::collections::HashMap;

use crate::{
    config::Config,
    planner::{FactorGraph, NodeKind, RobotId, RobotState},
    theme::CatppuccinTheme,
    ui::{ChangingBinding, ExportGraphEvent},
};

use super::super::theme::ThemeEvent;
use bevy::{prelude::*, tasks::IoTaskPool};
use gbp_linalg::GbpFloat;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use strum_macros::EnumIter;

#[derive(Component)]
pub struct GeneralInputs;

pub struct GeneralInputPlugin;

impl Plugin for GeneralInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<GeneralAction>::default(),))
            .add_systems(PostStartup, (bind_general_input,))
            .add_systems(Update, (general_actions_system, export_graph_on_event));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter)]
pub enum GeneralAction {
    ToggleTheme,
    ExportGraph,
}

impl ToString for GeneralAction {
    fn to_string(&self) -> String {
        match self {
            Self::ToggleTheme => "Toggle Theme".to_string(),
            Self::ExportGraph => "Export Graph".to_string(),
        }
    }
}

impl Default for GeneralAction {
    fn default() -> Self {
        Self::ToggleTheme
    }
}

impl GeneralAction {
    fn variants() -> &'static [Self] {
        &[GeneralAction::ToggleTheme, GeneralAction::ExportGraph]
    }

    fn default_keyboard_input(action: GeneralAction) -> Option<UserInput> {
        match action {
            Self::ToggleTheme => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyT))),
            Self::ExportGraph => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyG))),
        }
    }
}

fn bind_general_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for &action in GeneralAction::variants() {
        if let Some(input) = GeneralAction::default_keyboard_input(action) {
            input_map.insert(action, input);
        }
    }

    commands.spawn((
        InputManagerBundle::<GeneralAction> {
            input_map,
            ..Default::default()
        },
        GeneralInputs,
    ));
}

fn export_factorgraphs_as_graphviz(
    query: Query<(Entity, &FactorGraph), With<RobotState>>,
    config: &Config,
    // config: Res<Config>,
) -> Option<String> {
    if query.is_empty() {
        // There are no factorgraph in the scene/world
        return None;
    }

    let external_edge_length = 8.0;
    let internal_edge_length = 1.0;
    let cluster_margin = 16;

    let mut buf = String::with_capacity(4 * 1024); // 4 kB
    let mut append_line_to_output = |line: &str| {
        buf.push_str(line);
        buf.push('\n');
    };
    append_line_to_output("graph {");
    append_line_to_output("  dpi=96;");
    append_line_to_output(r#"  label="factorgraph""#);
    append_line_to_output("  node [style=filled];");
    append_line_to_output("  layout=neato;");

    // A hashmap used to keep track of which variable in another robots factorgraph, is connected to a interrobot
    // factor in the current robots factorgraph.
    let mut all_external_connections =
        HashMap::<RobotId, HashMap<usize, (RobotId, usize)>>::with_capacity(query.iter().len());

    for (robot_id, factorgraph) in query.iter() {
        let (nodes, edges) = factorgraph.export_data();

        // append_line_to_output(&format!(r#"  subgraph "cluster_{:?}" {{"#, robot_id));
        append_line_to_output(&format!(r#"  subgraph "{:?}" {{"#, robot_id));
        append_line_to_output(&format!("  margin={}", cluster_margin));
        append_line_to_output(&format!(r#"  label="{:?}""#, robot_id));
        // Add all nodes
        for node in nodes.iter() {
            let pos = match node.kind {
                NodeKind::Variable { x, y } => Some((x, y)),
                _ => None,
            };

            let line = {
                let mut line = String::with_capacity(32);
                line.push_str(&format!(
                    r#""{:?}_{:?}" [label="{:?}", fillcolor="{}", shape={}, width="{}""#,
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

        let external_connections: HashMap<usize, (RobotId, usize)> = nodes
            .into_iter()
            .filter_map(|node| match node.kind {
                NodeKind::InterRobotFactor(connection) => Some((
                    node.index,
                    (
                        connection.id_of_robot_connected_with,
                        connection
                            .index_of_connected_variable_in_other_robots_factorgraph
                            .index(),
                    ),
                )),
                _ => None,
            })
            .collect();

        all_external_connections.insert(robot_id, external_connections);
    }

    // Add edges between interrobot factors and the variable they are connected to in another robots graph
    for (from_robot_id, from_connections) in all_external_connections.iter() {
        for (from_factor, (to_robot_id, to_variable_index)) in from_connections.iter() {
            append_line_to_output(&format!(
                r#" "{:?}_{:?}" -- "{:?}_{:?}" [len={}, style={}, color="{}"]"#,
                from_robot_id,
                from_factor,
                to_robot_id,
                to_variable_index,
                config.graphviz.interrobot.edge.len,
                config.graphviz.interrobot.edge.style,
                config.graphviz.interrobot.edge.color,
            ));
        }
    }

    append_line_to_output("}"); // closing '}' for starting "graph {"
    Some(buf)
}

fn handle_toggle_theme(
    theme_event: &mut EventWriter<ThemeEvent>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    info!("toggling application theme");
    // theme_event.send(ThemeEvent);

    match catppuccin_theme.flavour {
        catppuccin::Flavour::Latte => {
            theme_event.send(ThemeEvent(catppuccin::Flavour::Frappe));
        }
        catppuccin::Flavour::Frappe => {
            theme_event.send(ThemeEvent(catppuccin::Flavour::Macchiato));
        }
        catppuccin::Flavour::Macchiato => {
            theme_event.send(ThemeEvent(catppuccin::Flavour::Mocha));
        }
        catppuccin::Flavour::Mocha => {
            theme_event.send(ThemeEvent(catppuccin::Flavour::Latte));
        }
    }
}

fn export_graph_on_event(
    mut theme_event: EventReader<ExportGraphEvent>,
    query: Query<(Entity, &FactorGraph), With<RobotState>>,
    config: Res<Config>,
) {
    if theme_event.read().next().is_some() {
        if let Err(e) = handle_export_graph(query, config.as_ref()) {
            error!("failed to export factorgraphs with error: {:?}", e);
        }
    }
}

fn handle_export_graph(
    q: Query<(Entity, &FactorGraph), With<RobotState>>,
    config: &Config,
) -> std::io::Result<()> {
    let Some(output) = export_factorgraphs_as_graphviz(q, config) else {
        warn!("There are no factorgraphs in the world");
        return Ok(());
    };

    let dot_output_path = std::path::PathBuf::from("factorgraphs.dot");
    if dot_output_path.exists() {
        warn!(
            "output destination: ./{:#?} already exists!",
            dot_output_path
        );
        warn!("overwriting ./{:#?}", dot_output_path);
    }
    info!("exporting all factorgraphs to ./{:#?}", dot_output_path);
    std::fs::write(&dot_output_path, output.as_bytes())?;

    IoTaskPool::get()
        .spawn(async move {
            let png_output_path = dot_output_path.with_extension("png");
            let args = [
                "-T",
                "png",
                "-o",
                png_output_path.to_str().expect("is valid UTF8"),
                dot_output_path.to_str().expect("is valid UTF8"),
            ];
            let Ok(output) = std::process::Command::new("dot").args(args).output() else {
                error!(
                    "failed to compile ./{:?} with dot. reason: dot was not found in $PATH",
                    dot_output_path
                );
                return;
            };

            if output.status.success() {
                info!(
                    "compiled {:?} to {:?} with dot",
                    dot_output_path, png_output_path
                );
                let _ = open::that(&png_output_path)
                    .inspect_err(|e| error!("failed to open ./{:?}: {e}", png_output_path));
            } else {
                error!(
                    "attempting to compile graph with dot, returned a non-zero exit status: {:?}",
                    output
                );
            }
        })
        .detach();

    Ok(())
}

fn general_actions_system(
    mut theme_event: EventWriter<ThemeEvent>,
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
    query_graphs: Query<(Entity, &FactorGraph), With<RobotState>>,
    config: Res<Config>,
    currently_changing: Res<ChangingBinding>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }
    let Ok(action_state) = query.get_single() else {
        warn!("general_actions_system was called without an action state!");
        return;
    };

    if action_state.just_pressed(&GeneralAction::ToggleTheme) {
        handle_toggle_theme(&mut theme_event, catppuccin_theme);
    }

    if action_state.just_pressed(&GeneralAction::ExportGraph) {
        if let Err(e) = handle_export_graph(query_graphs, config.as_ref()) {
            error!("failed to export factorgraphs with error: {:?}", e);
        }
    }
}
