use std::{collections::HashMap, io::Write};

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use gbp_config::Config;

use crate::{
    factorgraph::prelude::FactorGraph,
    planner::{
        self,
        robot::{Radius, Route},
    },
};

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::Export>()
            .add_event::<events::SubmitRobotData>()
            .init_resource::<SubmittedRobots>()
            .add_systems(
                Update,
                (
                    export,
                    send_default_export_event.run_if(
                        input_just_pressed(KeyCode::F7)
                            .or_else(on_event::<crate::planner::spawner::AllFormationsFinished>()),
                    ),
                    submit_robot_data,
                ),
            );
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct SubmittedRobots(HashMap<Entity, RobotData>);

fn send_default_export_event(mut evw_export: EventWriter<events::Export>) {
    evw_export.send(events::Export::default());
}

#[derive(Debug, Clone, Default)]
pub enum ExportSaveLocation {
    At(std::path::PathBuf),
    #[default]
    Cwd,
}

pub mod events {
    use super::*;
    #[derive(Event, Default)]
    pub struct Export {
        pub save_at_location: ExportSaveLocation,
        pub postfix: ExportSavePostfix,
    }

    #[derive(Event)]
    pub struct SubmitRobotData {
        pub data: RobotData,
    }
}

#[derive(serde::Serialize)]
pub struct RobotData {
    radius:     f32,
    positions:  Vec<[f32; 2]>,
    velocities: Vec<[f32; 2]>,
    collisions: CollisionData,
    messages:   MessageData,
    route:      RouteData,
}

#[derive(serde::Serialize)]
struct RouteData {
    waypoints:   Vec<[f32; 2]>,
    started_at:  f64,
    finished_at: f64,
    duration:    f64,
}

impl RouteData {
    fn new(waypoints: Vec<[f32; 2]>, started_at: f64, finished_at: f64) -> Self {
        assert!(waypoints.len() >= 2);
        assert!(finished_at > started_at);
        let duration = finished_at - started_at;
        Self {
            waypoints,
            started_at,
            finished_at,
            duration,
        }
    }
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
    mut evr_export: EventReader<events::Export>,
    q_robots: Query<(
        Entity,
        &FactorGraph,
        &planner::tracking::PositionTracker,
        &planner::tracking::VelocityTracker,
        &Radius,
        &Route,
    )>,
    robot_collisions: Res<crate::planner::collisions::resources::RobotRobotCollisions>,
    environment_collisions: Res<crate::planner::collisions::resources::RobotEnvironmentCollisions>,
    sim_manager: Res<crate::simulation_loader::SimulationManager>,
    config: Res<Config>,
    time_virtual: Res<Time<Virtual>>,
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
    //       "route": {
    //         "waypoints": <json array>, [{"x": <float>, "y": <float> }],
    //         "started_at": <float>,
    //         "finished_at": <float>,
    //         "duration": <float>
    //       },
    //       "positions": <json array>, [{"x": <float>, "y": <float>, "timestamp":
    // <float> } ], ]       "velocities": <json array>, [{"x": <float>, "y":
    // <float>, "timestamp": <float> } ], ]       "collisions": {
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
        // let makespan = -1.0; // Placeholder, replace with actual makespan calculation
        // FIXME: compute as the duration from when the first robot spawned, to the last
        // robot finished its route
        let makespan = time_virtual.elapsed_seconds() as f64;

        let robots: HashMap<_, _> = q_robots
            .iter()
            .map(|(entity, graph, positions, velocities, radius, route)| {
                let positions: Vec<[f32; 2]> = positions.positions().map(Into::into).collect();
                let velocities: Vec<[f32; 2]> = velocities.velocities().map(Into::into).collect();
                let robot_collisions = robot_collisions.get(entity).unwrap_or(0);
                let environment_collisions = environment_collisions.get(entity).unwrap_or(0);

                let id = format!("{:?}", entity);
                let robot_data = RobotData {
                    radius: radius.0,
                    positions,
                    velocities,
                    route: RouteData::new(
                        route
                            .waypoints()
                            .iter()
                            .map(|wp| wp.position().into())
                            .collect(),
                        route.started_at(),
                        route
                            .finished_at()
                            .unwrap_or_else(|| time_virtual.elapsed_seconds_f64()),
                    ),
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
                internal: config.gbp.iteration_schedule.internal,
                external: config.gbp.iteration_schedule.external,
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
                        path.file_name().and_then(|file_name| {
                            file_name.to_str().map(std::string::ToString::to_string)
                        })
                    })
                    .filter_map(|basename| {
                        basename[prefix.len()..basename.len() - 5]
                            .parse::<usize>()
                            .ok()
                    })
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

fn submit_robot_data(mut evr_submit_robot_data: EventReader<events::SubmitRobotData>) {}
