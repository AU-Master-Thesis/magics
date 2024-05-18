use std::{collections::HashMap, io::Write};

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use gbp_config::{formation::PlanningStrategy, Config};

use self::events::TakeSnapshotOfRobot;
use crate::{
    factorgraph::prelude::FactorGraph,
    planner::{self, robot::Radius},
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

#[derive(Default)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::Export>()
            .add_event::<events::TakeSnapshotOfRobot>()
            .add_event::<events::OpenLatestExport>()
            .init_resource::<resources::SnapshottedRobots>()
            .init_resource::<resources::LatestExport>()
            .add_systems(
                Update,
                (
                    export,
                    open_latest_export,
                    send_default_export_event.run_if(
                        input_just_pressed(KeyCode::F7)
                            .or_else(on_event::<crate::planner::spawner::AllFormationsFinished>()),
                    ),
                    await_robot_snapshot_request,
                    clear_submitted_robots.run_if(
                        on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>()),
                    ),
                ),
            );
    }
}

mod resources {
    use super::*;

    #[derive(Resource, Deref, DerefMut, Default)]
    pub(super) struct SnapshottedRobots(HashMap<Entity, RobotData>);

    #[derive(Resource, Deref, DerefMut, Default)]
    pub(super) struct LatestExport(pub Option<std::path::PathBuf>);
}

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
        pub toast: bool,
    }

    #[derive(Event, Default)]
    pub struct OpenLatestExport;

    #[derive(Event)]
    pub struct TakeSnapshotOfRobot(pub Entity);
}

fn open_latest_export(
    mut evr_open_latest_export: EventReader<events::OpenLatestExport>,
    latest_export: Res<resources::LatestExport>,
    mut evw_toast: EventWriter<bevy_notify::ToastEvent>,
) {
    for _ in evr_open_latest_export.read() {
        let Some(ref path) = latest_export.0 else {
            evw_toast.send(bevy_notify::ToastEvent::error(
                "no data has been exported yet",
            ));
            continue;
        };

        if cfg!(target_arch = "wasm32") {
            evw_toast.send(bevy_notify::ToastEvent::warning("Not supported on wasm32"));
        } else {
            if let Err(err) = open::that_detached(path) {
                let err_msg = format!("Failed to open {}: {}", path.display(), err);
                error!(err_msg);
                evw_toast.send(bevy_notify::ToastEvent::error(err_msg));
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct RobotData {
    radius: f32,
    positions: Vec<[f32; 2]>,
    // velocities: Vec<[f32; 2]>,
    velocities: Vec<planner::tracking::VelocityMeasurement>,
    collisions: CollisionData,
    messages: MessageData,
    // route: RouteData,
    mission: MissionData,
    planning_strategy: PlanningStrategy,
}

#[derive(serde::Serialize)]
struct MissionData {
    // waypoints: Vec<[f32; 4]>, // [x, y, x', y']
    waypoints:   Vec<[f32; 2]>, // [x, y]
    started_at:  f64,
    finished_at: f64,
    routes:      Vec<RouteData>,
}

#[derive(serde::Serialize)]
struct RouteData {
    waypoints:   Vec<[f32; 2]>,
    started_at:  f64,
    finished_at: f64,
}

// impl std::convert::From<planner::robot::Route> for RouteData {
//     fn from(route: planner::robot::Route) -> Self {
//         Self {
//             waypoints:   route.waypoints,
//             started_at:  route.started_at(),
//             finished_at: route.finished_at,
//         }
//     }
// }

// impl RouteData {
//     fn new(waypoints: Vec<[f32; 2]>, started_at: f64, finished_at: f64) ->
// Self {         assert!(waypoints.len() >= 2);
//         if finished_at < started_at {
//             dbg!(finished_at, started_at);
//             panic!("finished_at must be after started_at");
//         }
//         // assert!(finished_at > started_at);
//         let duration = finished_at - started_at;
//         Self {
//             waypoints,
//             started_at,
//             finished_at,
//             duration,
//         }
//     }
// }

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
    delta_t: f64,
    gbp: GbpData,
    robots: HashMap<Entity, RobotData>,
    prng_seed: u64,
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
    mut evw_toast: EventWriter<bevy_notify::ToastEvent>,
    mut latest_export: ResMut<resources::LatestExport>,
    mut robot_snapshots: ResMut<resources::SnapshottedRobots>,
    q_robots: Query<(
        Entity,
        &FactorGraph,
        &planner::tracking::PositionTracker,
        &planner::tracking::VelocityTracker,
        &Radius,
        // &Route,
        &planner::robot::RobotMission,
        &PlanningStrategy,
    )>,
    robot_collisions: Res<crate::planner::collisions::resources::RobotRobotCollisions>,
    environment_collisions: Res<crate::planner::collisions::resources::RobotEnvironmentCollisions>,
    sim_manager: Res<crate::simulation_loader::SimulationManager>,
    config: Res<Config>,
    time_virtual: Res<Time<Virtual>>,
    time_fixed: Res<Time<Fixed>>,
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
        // take a snapshot of all robots, that do not already have one

        for (robot_entity, graph, positions, velocities, radius, mission, planning_strategy) in
            q_robots.iter()
        {
            if robot_snapshots.contains_key(&robot_entity) {
                continue;
            }
            let positions: Vec<[f32; 2]> = positions.positions().map(Into::into).collect();
            // let velocities: Vec<[f32; 2]> =
            // velocities.velocities().map(Into::into).collect();
            let velocities: Vec<_> = velocities.measurements().collect();
            let robot_collisions = robot_collisions.get(robot_entity).unwrap_or(0);
            let environment_collisions = environment_collisions.get(robot_entity).unwrap_or(0);

            // let id = format!("{:?}", robot_entity);
            let robot_data = RobotData {
                radius: radius.0,
                positions,
                velocities,
                mission: MissionData {
                    waypoints:   mission
                        .waypoints
                        .iter()
                        .map(|wp| wp.position().into())
                        .collect(),
                    started_at:  mission.started_at(),
                    finished_at: mission
                        .finished_at()
                        .unwrap_or_else(|| time_fixed.elapsed_seconds_f64()),
                    // routes:      mission.routes.iter().map(Into::into).collect(),
                    routes:      mission
                        .routes
                        .iter()
                        .map(|r| RouteData {
                            waypoints:   r
                                .waypoints()
                                .iter()
                                .map(|wp| wp.position().into())
                                .collect(),
                            started_at:  r.started_at(),
                            finished_at: r
                                .finished_at()
                                .unwrap_or_else(|| time_fixed.elapsed_seconds_f64()),
                        })
                        .collect(),
                },

                // route: RouteData::new(
                //     route
                //         .waypoints()
                //         .iter()
                //         .map(|wp| wp.position().into())
                //         .collect(),
                //     route.started_at(),
                //     route
                //         .finished_at()
                //         .unwrap_or_else(|| time_virtual.elapsed_seconds_f64()),
                // ),
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
                planning_strategy: *planning_strategy,
            };

            robot_snapshots.insert(robot_entity, robot_data);
        }
        // let robots: HashMap<_, _> = q_robots
        //     .iter()
        //     .map(|(entity, graph, positions, velocities, radius, route)| {
        //         let positions: Vec<[f32; 2]> =
        // positions.positions().map(Into::into).collect();         let
        // velocities: Vec<[f32; 2]> =
        // velocities.velocities().map(Into::into).collect();         let
        // robot_collisions = robot_collisions.get(entity).unwrap_or(0);
        //         let environment_collisions =
        // environment_collisions.get(entity).unwrap_or(0);
        //
        //         let id = format!("{:?}", entity);
        //         let robot_data = RobotData {
        //             radius: radius.0,
        //             positions,
        //             velocities,
        //             route: RouteData::new(
        //                 route
        //                     .waypoints()
        //                     .iter()
        //                     .map(|wp| wp.position().into())
        //                     .collect(),
        //                 route.started_at(),
        //                 route
        //                     .finished_at()
        //                     .unwrap_or_else(|| time_virtual.elapsed_seconds_f64()),
        //             ),
        //             collisions: CollisionData {
        //                 robots:      robot_collisions,
        //                 environment: environment_collisions,
        //             },
        //             messages: MessageData {
        //                 sent:     MessageCount {
        //                     internal: graph.messages_sent().internal,
        //                     external: graph.messages_sent().external,
        //                 },
        //                 received: MessageCount {
        //                     internal: graph.messages_received().internal,
        //                     external: graph.messages_received().external,
        //                 },
        //             },
        //         };
        //
        //         (id, robot_data)
        //     })
        //     .collect();

        let gbp = GbpData {
            iterations: GbpIterationData {
                internal: config.gbp.iteration_schedule.internal,
                external: config.gbp.iteration_schedule.external,
            },
        };

        let export_data = ExportData {
            environment: environment.to_string(),
            makespan,
            delta_t: time_fixed.delta_seconds_f64(),
            gbp,
            robots: robot_snapshots.drain().collect(),
            prng_seed: config.simulation.prng_seed,
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

        let message = format!(
            "Data exported successfully to '{}'",
            output_filepath.to_string_lossy().to_string()
        );
        info!(message);

        if event.toast {
            evw_toast.send(bevy_notify::ToastEvent::success(message));
        }

        latest_export.0 = Some(output_filepath);
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

fn take_snapshot_of_robot(
    robot_entity: Entity,
    // q_robots: &Query<(
    //     &FactorGraph,
    //     &planner::tracking::PositionTracker,
    //     &planner::tracking::VelocityTracker,
    //     &Radius,
    //     &Route,
    // )>,
    q_robots: &Query<(
        &FactorGraph,
        &planner::tracking::PositionTracker,
        &planner::tracking::VelocityTracker,
        &Radius,
        // &Route,
        &planner::robot::RobotMission,
        &PlanningStrategy,
    )>,

    robot_collisions: &crate::planner::collisions::resources::RobotRobotCollisions,
    environment_collisions: &crate::planner::collisions::resources::RobotEnvironmentCollisions,
    // time_virtual: &Time<Virtual>,
    time_fixed: &Time<Fixed>,
) -> anyhow::Result<RobotData> {
    let Ok((fgraph, positions, velocities, radius, mission, planning_strategy)) =
        q_robots.get(robot_entity)
    else {
        anyhow::bail!(
            "cannot take snapshot of non-existing robot {:?}",
            robot_entity
        );
    };

    let positions: Vec<[f32; 2]> = positions.positions().map(Into::into).collect();
    // let velocities: Vec<[f32; 2]> =
    // velocities.velocities().map(Into::into).collect();
    let velocities: Vec<_> = velocities.measurements().collect();
    let robot_collisions = robot_collisions.get(robot_entity).unwrap_or(0);
    let environment_collisions = environment_collisions.get(robot_entity).unwrap_or(0);

    let robot_data = RobotData {
        radius: radius.0,
        positions,
        velocities,
        // route: RouteData::new(
        //     route
        //         .waypoints()
        //         .iter()
        //         .map(|wp| wp.position().into())
        //         .collect(),
        //     route.started_at(),
        //     route
        //         .finished_at()
        //         .unwrap_or_else(|| time_fixed.elapsed_seconds_f64()),
        // ),
        collisions: CollisionData {
            robots:      robot_collisions,
            environment: environment_collisions,
        },
        messages: MessageData {
            sent:     MessageCount {
                internal: fgraph.messages_sent().internal,
                external: fgraph.messages_sent().external,
            },
            received: MessageCount {
                internal: fgraph.messages_received().internal,
                external: fgraph.messages_received().external,
            },
        },
        planning_strategy: *planning_strategy,
        mission: MissionData {
            started_at:  mission.started_at(),
            finished_at: mission
                .finished_at()
                .unwrap_or_else(|| time_fixed.elapsed_seconds_f64()),
            waypoints:   mission
                .waypoints
                .iter()
                .map(|wp| wp.position().into())
                .collect(),
            // routes:      mission.routes.iter().map(Into::into).collect(),
            routes:      mission
                .routes
                .iter()
                .map(|r| RouteData {
                    waypoints:   r
                        .waypoints()
                        .iter()
                        .map(|wp| wp.position().into())
                        .collect(),
                    started_at:  r.started_at(),
                    finished_at: r
                        .finished_at()
                        .unwrap_or_else(|| time_fixed.elapsed_seconds_f64()),
                })
                .collect(),
        },
    };

    Ok(robot_data)
}

fn await_robot_snapshot_request(
    mut evr_submit_robot_data: EventReader<events::TakeSnapshotOfRobot>,
    mut submitted_robots: ResMut<resources::SnapshottedRobots>,

    // q_robots: Query<(
    //     &FactorGraph,
    //     &planner::tracking::PositionTracker,
    //     &planner::tracking::VelocityTracker,
    //     &Radius,
    //     &Route,
    // )>,
    q_robots: Query<(
        &FactorGraph,
        &planner::tracking::PositionTracker,
        &planner::tracking::VelocityTracker,
        &Radius,
        // &Route,
        &planner::robot::RobotMission,
        &PlanningStrategy,
    )>,

    robot_collisions: Res<crate::planner::collisions::resources::RobotRobotCollisions>,
    environment_collisions: Res<crate::planner::collisions::resources::RobotEnvironmentCollisions>,
    // time_virtual: Res<Time<Virtual>>,
    time_fixed: Res<Time<Fixed>>,
) {
    for TakeSnapshotOfRobot(robot_id) in evr_submit_robot_data.read() {
        // ignore if the robot has already been submitted
        if submitted_robots.contains_key(robot_id) {
            continue;
        }
        let Ok(snapshot) = take_snapshot_of_robot(
            *robot_id,
            &q_robots,
            &robot_collisions,
            &environment_collisions,
            &time_fixed,
        ) else {
            error!(
                "failed to take snapshot of robot {:?}, reason entity does not exist",
                robot_id
            );
            continue;
        };
        submitted_robots.insert(*robot_id, snapshot);
    }
}

fn clear_submitted_robots(mut submitted_robots: ResMut<resources::SnapshottedRobots>) {
    submitted_robots.clear();
}
