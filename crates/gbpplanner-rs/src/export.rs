use std::{collections::HashMap, io::Write};

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    config::Config,
    factorgraph::prelude::FactorGraph,
    planner::{self, robot::Radius},
    simulation_loader::SimulationManager,
};

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        println!("export plugin loaded");
        app.add_event::<Export>().add_systems(
            Update,
            (
                export,
                send_default_export_event.run_if(
                    input_just_pressed(KeyCode::F7)
                        .or_else(on_event::<crate::planner::spawner::AllFormationsFinished>()),
                ),
            ),
        );
    }
}

fn send_default_export_event(mut evw_export: EventWriter<Export>) {
    evw_export.send(Export::default());
}

#[derive(Debug, Clone, Default)]
pub enum ExportSaveLocation {
    At(std::path::PathBuf),
    #[default]
    Cwd,
}

#[derive(Event, Default)]
pub struct Export {
    pub save_at_location: ExportSaveLocation,
    pub postfix: ExportSavePostfix,
}

#[derive(serde::Serialize)]
struct RobotData {
    radius:     f32,
    positions:  Vec<[f32; 2]>,
    velocities: Vec<[f32; 2]>,
    collisions: CollisionData,
    messages:   MessageData,
}

#[derive(serde::Serialize)]
struct CollisionData {
    robots:      usize,
    environment: usize,
}

#[derive(serde::Serialize)]
struct MessageData {
    sent:     MessageCount,
    received: MessageCount,
}

#[derive(serde::Serialize)]
struct MessageCount {
    internal: usize,
    external: usize,
}

#[derive(serde::Serialize)]
struct ExportData {
    environment: String,
    makespan: f64,
    gbp: GbpData,
    robots: HashMap<String, RobotData>,
}

#[derive(serde::Serialize)]
struct GbpIterationData {
    internal: usize,
    external: usize,
}

#[derive(serde::Serialize)]
struct GbpData {
    iterations: GbpIterationData,
}

fn export(
    mut evr_export: EventReader<Export>,
    q_robots: Query<(
        Entity,
        &FactorGraph,
        &planner::tracking::PositionTracker,
        &planner::tracking::VelocityTracker,
        &Radius,
    )>,
    robot_collisions: Res<crate::planner::collisions::RobotCollisions>,
    sim_manager: Res<SimulationManager>,
    config: Res<Config>,
) {
    // schema:
    //
    // {
    //   "environment": <string>,
    //   "makespan": <float>, // time it takes to complete the formation
    //   "gbp": {
    //      "iterations": {
    //        "internal": <integer>,
    //        "external": <integer>,
    //      }
    //   }
    //   "robots": [
    //     {
    //       "id": <string>,
    //       "radius": <float>,
    //       "positions": <json array>, [[x, y], ]
    //       "velocities": <json array>, [[x, y], ]
    //       "collisions": {
    //          "robots": <integer>
    //          "environment": <integer>
    //       },
    //       "messages": {
    //          "sent": {
    //              "internal": <integer>,
    //              "external": <integer>
    //          },
    //          "received": {
    //              "internal": <integer>,
    //              "external": <integer>
    //          }
    //       }
    //     },
    //     ...
    //   ]
    // }

    for event in evr_export.read() {
        let environment = sim_manager.active_name().unwrap_or_default();
        let makespan = -1.0; // Placeholder, replace with actual makespan calculation

        let robots: HashMap<_, _> = q_robots
            .iter()
            .map(|(entity, graph, positions, velocities, radius)| {
                let positions: Vec<[f32; 2]> = positions.positions().map(|pos| [pos.x, pos.z]).collect();
                let velocities: Vec<[f32; 2]> = velocities.velocities().map(|vel| [vel.x, vel.z]).collect();
                let robot_collisions = robot_collisions
                .get(entity)
                // .map(|collisions| collisions.robots)
                .unwrap_or(0);
                let environment_collisions = 0; // Placeholder, replace with actual environment collision count

                let id = format!("{:?}", entity);
                let robot_data = RobotData {
                    radius: radius.0,
                    positions,
                    velocities,
                    collisions: CollisionData {
                        robots:      robot_collisions,
                        environment: environment_collisions,
                    },
                    messages: MessageData {
                        sent:     MessageCount {
                            internal: graph.messages_sent().internal,
                            external: graph.messages_sent().external,
                        },
                        received: MessageCount {
                            internal: graph.messages_received().internal,
                            external: graph.messages_received().external,
                        },
                    },
                };

                (id, robot_data)
            })
            .collect();

        let gbp = GbpData {
            iterations: GbpIterationData {
                internal: config.gbp.iterations_per_timestep.internal,
                external: config.gbp.iterations_per_timestep.external,
            },
        };

        let export_data = ExportData {
            environment: environment.to_string(),
            makespan,
            gbp,
            robots,
        };

        let json = serde_json::to_string_pretty(&export_data).unwrap();

        let prefix = format!("export_{}_", environment.to_lowercase());
        let basename_postfix = match event.postfix {
            ExportSavePostfix::Number => {
                let glob_pattern = format!("{}*.json", prefix.as_str());
                let existing_files = glob::glob(glob_pattern.as_str()).expect("valid glob pattern");
                let latest_id = existing_files
                    .filter_map(std::result::Result::ok)
                    .filter_map(|path| {
                        path.file_name()
                            .and_then(|file_name| file_name.to_str().map(std::string::ToString::to_string))
                    })
                    .filter_map(|basename| basename[prefix.len()..basename.len() - 5].parse::<usize>().ok())
                    .max();

                let id = latest_id.map_or(0, |id| id + 1);
                id.to_string()
            }
            ExportSavePostfix::UnixTimestamp => chrono::Utc::now().timestamp().to_string(),
        };

        let dirname = match event.save_at_location {
            ExportSaveLocation::Cwd if cfg!(not(target_arch = "wasm32")) => {
                std::env::current_dir().expect("current directory exists")
            }
            ExportSaveLocation::Cwd => {
                panic!("cannot take screenshots when running in wasm32")
            }
            ExportSaveLocation::At(ref path) => path.clone(),
        };

        let output_filepath = dirname.join(format!("{}{}.json", prefix, basename_postfix));

        let mut file = std::fs::File::create(output_filepath.clone()).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        println!(
            "Data exported successfully to '{}'",
            output_filepath.to_string_lossy().to_string()
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportSavePostfix {
    Number,
    UnixTimestamp,
}

impl Default for ExportSavePostfix {
    fn default() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::UnixTimestamp
        } else {
            Self::Number
        }
    }
}
