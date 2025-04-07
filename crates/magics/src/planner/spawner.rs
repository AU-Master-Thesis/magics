use std::{num::NonZeroUsize, ops::DerefMut, time::Duration};

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_notify::ToastEvent;
use bevy_rand::prelude::{ForkableRng, GlobalEntropy};
use gbp_config::{
    formation::{PlanningStrategy, RepeatTimes, WorldDimensions},
    Config,
};
use itertools::Itertools;
use rand::{seq::IteratorRandom, Rng};
use strum::IntoEnumIterator;
use min_len_vec::TwoOrMore; // Added import

use super::{
    robot::{RobotFinishedRoute, RobotSpawned},
    RobotId,
};
use crate::{
    // asset_loader::SceneAssets,
    asset_loader::Meshes,
    environment::FollowCameraMe,
    pause_play::PausePlay,
    planner::robot::{RobotBundle, Route, StateVector},
    simulation_loader::{
        self, EndSimulation, LoadSimulation, ReloadSimulation, Sdf, SimulationManager,
    },
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour}, // Restore theme imports
    planner::visualiser::{InitialSpawnAreaViz, WaypointAreaViz}, // Import new components
    utils::get_variable_timesteps,
    bevy_utils::run_conditions::event_exists, // Import event_exists
};

pub struct RobotSpawnerPlugin;

impl Plugin for RobotSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RobotFormationSpawned>()
            .add_event::<RobotClickedOn>()
            .add_event::<WaypointCreated>()
            // .add_event::<RobotReachedWaypoint>()
            .add_event::<AllFormationsFinished>()
            .add_systems(
                Update,
                (
                    (
                        delete_formation_group_spawners,
                        create_formation_group_spawners,
                    )
                        .chain()
                        .run_if(
                            on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>()),
                        ),
                    // create_formation_group_spawners.run_if(on_event::<ReloadSimulation>()),
                    delete_formation_group_spawners.run_if(on_event::<EndSimulation>()),
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_formation,
                    advance_time.run_if(not(virtual_time_is_paused)),
                    exit_application_on_scenario_finished,
                    // exit_application_on_scenario_finished.run_if(on_event::<AllFormationsFinished>())
                ),
            )
            .add_systems(
                Update,
                (
                    track_score.run_if(resource_exists::<Scoreboard>),
                    notify_on_all_formations_finished.run_if(on_event::<AllFormationsFinished>()),
                    // Add systems to toggle visibility
                    show_or_hide_spawn_areas.run_if(event_exists::<crate::input::DrawSettingsEvent>),
                    show_or_hide_waypoint_areas.run_if(event_exists::<crate::input::DrawSettingsEvent>),
                ),
            );
    }
}

#[derive(Event)]
pub struct AllFormationsFinished;

fn track_score(
    mut scoreboard: ResMut<Scoreboard>,
    // mut evr_robot_despawned: EventReader<RobotDespawned>,
    mut evr_robot_finished_route: EventReader<RobotFinishedRoute>,
    spawners: Query<&FormationSpawner>,
    mut evw_formations_finished: EventWriter<AllFormationsFinished>,
) {
    for RobotFinishedRoute(_) in evr_robot_finished_route.read() {
        scoreboard.robots_left = scoreboard.robots_left.saturating_sub(1);
        // if scoreboard.robots_left > 0 {
        //     scoreboard.robots_left -= 1;
        // }
    }

    if scoreboard.robots_left == 0
        && !scoreboard.game_over
        && spawners.iter().all(FormationSpawner::exhausted)
    {
        evw_formations_finished.send(AllFormationsFinished);
        scoreboard.game_over = true;
    }
}

fn notify_on_all_formations_finished(
    mut evw_toast: EventWriter<ToastEvent>,
    time_virtual: Res<Time<Virtual>>,
    time_real: Res<Time<Real>>,
) {
    let caption = format!(
        "all formations finished after {} seconds (virtual), {} (real)",
        time_virtual.elapsed_seconds(),
        time_real.elapsed_seconds(),
    );
    let toast = ToastEvent::info(caption);
    evw_toast.send(toast);
}

/// run criteria if time is not paused
#[inline]
fn virtual_time_is_paused(time: Res<Time<Virtual>>) -> bool {
    time.is_paused()
}

/// **bevy** event emitted whenever a robot waypoint is created
#[derive(Event)]
pub struct WaypointCreated {
    /// The id of the robot the waypoint is created for
    pub for_robot: RobotId,
    /// The (x,y) position of the created waypoint in world coordinates.
    pub position:  Vec2,
}

// #[derive(Event)]
// pub struct RobotReachedWaypoint(pub Entity);

// TODO: allocate for each obstacle factor, a bit wasteful but should not take
// up to much memory like 8-10 MB
// TODO: needs to be changed whenever the sim reloads, use resource?
/// Every [`ObstacleFactor`] has a static reference to the obstacle image.
// static OBSTACLE_IMAGE: OnceLock<Image> = OnceLock::new();
// TODO: use once_cell, so we can mutate it when sim reloads
// static OBSTACLE_SDF: Lazy<RwLock<Image>> = Lazy::new(||
// RwLock::new(Image::new(1, 1)));

// /// Component attached to an entity that spawns formations.
// #[derive(Component)]
// pub struct FormationSpawnerCountdown {
//     pub timer: Timer,
//     pub formation_group_index: usize,
// }

// /// Enum representing the number of times a formation should repeat.
// #[derive(Debug, Clone, Copy, Default)]
// pub enum RepeatTimes {
//     #[default]
//     Infinite,
//     Finite(usize),
// }

// impl RepeatTimes {
//     pub const ONCE: Self = Self::Finite(1);

//     /// Construct a new `RepeatTimes::Finite` variant
//     pub fn finite(times: NonZeroUsize) -> Self {
//         Self::Finite(times.into())
//     }

//     /// Returns true if there are one or more times left repeating
//     pub const fn exhausted(&self) -> bool {
//         match self {
//             Self::Infinite => false,
//             Self::Finite(remaining) => *remaining == 0,
//         }
//     }

//     pub fn decrement(&mut self) {
//         match self {
//             Self::Finite(ref mut remaining) if *remaining > 0 => *remaining -= 1,
//             _ => {} // RepeatTimes::Infinite => {},
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct RepeatingTimer {
    timer:  Timer,
    repeat: RepeatTimes,
}

impl RepeatingTimer {
    fn new(duration: Duration, repeat: RepeatTimes) -> Self {
        let timer = Timer::new(duration, TimerMode::Repeating);
        Self { timer, repeat }
    }

    #[inline]
    pub const fn exhausted(&self) -> bool {
        self.repeat.exhausted()
    }

    #[inline]
    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
        // TODO: have all state mutation in this call
        // if self.timer.just_finished() {
        //     self.repeat.decrement();
        // }
    }

    #[inline]
    pub fn just_finished(&mut self) -> bool {
        let finished = self.timer.just_finished() && !self.repeat.exhausted();
        if finished {
            self.repeat.decrement();
        }

        finished
    }

    // #[inline]
    // pub fn duration(&self) -> Duration {
    //     self.timer.duration()
    // }
}

#[derive(Debug, Component)]
pub struct FormationSpawner {
    pub formation_group_index: usize,
    initial_delay: Timer,
    timer: RepeatingTimer,
    spawned: usize,
    state: FormationSpawnerState,
}

#[derive(Debug, Clone, Copy, Default)]
enum FormationSpawnerState {
    #[default]
    Inactive,
    Active {
        on_cooldown: bool,
    },
    // OnCooldown,
    Finished,
}

impl FormationSpawner {
    #[must_use]
    pub fn new(
        formation_group_index: usize,
        initial_delay: Duration,
        timer: RepeatingTimer,
    ) -> Self {
        Self {
            formation_group_index,
            initial_delay: Timer::new(initial_delay, TimerMode::Once),
            timer,
            spawned: 0,
            state: FormationSpawnerState::Inactive,
        }
    }

    #[inline]
    const fn is_active(&self) -> bool {
        // self.initial_delay.finished()
        matches!(self.state, FormationSpawnerState::Active { .. })
    }

    /// Return `true` if there is no more to spawn
    /// TODO: use this to test if the simulation is "finished"
    /// Simulation is finished when all spawners are finished
    #[inline]
    pub const fn exhausted(&self) -> bool {
        // self.timer.exhausted()
        matches!(self.state, FormationSpawnerState::Finished)
    }

    fn tick(&mut self, delta: Duration) {
        use FormationSpawnerState::{Active, Finished, Inactive};
        match self.state {
            Inactive => {
                self.initial_delay.tick(delta);
                if self.initial_delay.just_finished() {
                    self.state = Active { on_cooldown: false };
                }
            }
            Active { on_cooldown: true } => {
                self.timer.tick(delta);
                if self.timer.just_finished() {
                    if self.timer.exhausted() {
                        self.state = Finished;
                    } else {
                        self.state = Active { on_cooldown: false }
                    }
                }
            }
            Active { on_cooldown: false } | Finished => {}
        }
    }

    /// Returns the number of robots spawned so far
    #[inline]
    pub const fn spawned(&self) -> usize {
        self.spawned
    }

    fn spawn(&mut self) {
        if matches!(self.state, FormationSpawnerState::Active {
            on_cooldown: false,
        }) {
            self.state = FormationSpawnerState::Active { on_cooldown: true };
            self.spawned += 1;
        };
    }

    #[inline]
    fn ready_to_spawn(&mut self) -> bool {
        matches!(self.state, FormationSpawnerState::Active {
            on_cooldown: false,
        })
    }

    // #[inline]
    // fn on_cooldown(&mut self) -> bool {
    //     matches!(self.state, FormationSpawnerState::Active { on_cooldown: true })
    // }
}

fn delete_formation_group_spawners(
    mut commands: Commands,
    formation_spawners: Query<Entity, With<FormationSpawner>>,
) {
    for spawner in &formation_spawners {
        info!("despawning formation spawner: {:?}", spawner);
        commands.entity(spawner).despawn();
    }
}

#[derive(Resource)]
pub struct Scoreboard {
    pub robots_left: usize,
    pub game_over:   bool,
}

fn create_formation_group_spawners(
    mut commands: Commands,
    simulation_manager: Res<SimulationManager>,
) {
    let Some(formation_group) = simulation_manager.active_formation_group() else {
        warn!("No active formation group!");
        return;
    };

    let robots_to_spawn = formation_group.robots_to_spawn();

    for (i, formation) in formation_group.formations.iter().enumerate() {
        #[allow(clippy::option_if_let_else)] // find it more readable with a match here
        let repeating_timer = match formation.repeat {
            Some(repeat) => RepeatingTimer::new(repeat.every, repeat.times),
            None => RepeatingTimer::new(Duration::from_secs(0), RepeatTimes::ONCE),
        };

        info!(
            "spawning FormationSpawner[{i}] with delay {:?} and timer {:?}",
            formation.delay, repeating_timer
        );

        commands.spawn(FormationSpawner::new(i, formation.delay, repeating_timer));
    }
    commands.insert_resource(Scoreboard {
        robots_left: robots_to_spawn,
        game_over:   false,
    });
}

/// Event that is sent when a formation should be spawned.
/// The `formation_group_index` is the index of the formation group in the
/// `FormationGroup` resource. Telling the event reader which formation group to
/// spawn.
/// Assumes that the `FormationGroup` resource has been initialised, and does
/// not change during the program's execution.
#[derive(Debug, Event)]
pub struct RobotFormationSpawned {
    pub formation_group_index: usize,
}

/// Advance time for each `FormationSpawnerCountdown` entity with
/// `Time::delta()`. If the timer has just finished, send a
/// `FormationSpawnEvent`.
fn advance_time(
    mut spawners: Query<&mut FormationSpawner>,
    mut evw_robot_formation_spawned: EventWriter<RobotFormationSpawned>,
    mut evw_pause_play: EventWriter<PausePlay>,
    time: Res<Time>,
    config: Res<Config>,
) {
    for mut spawner in &mut spawners {
        spawner.tick(time.delta());

        if spawner.ready_to_spawn() {
            spawner.spawn();
            info!(
                "FormationSpawner[{}] ready to spawn!",
                spawner.formation_group_index
            );
            evw_robot_formation_spawned.send(RobotFormationSpawned {
                formation_group_index: spawner.formation_group_index,
            });

            if config.simulation.pause_on_spawn {
                // error!("pausing on spawn");
                evw_pause_play.send(PausePlay::Pause);
            }
        }
    }
}


/// **Bevy** [`Update`] system
/// Reads [`DrawSettingsEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::SpawnAreas` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`InitialSpawnAreaViz`] entities
fn show_or_hide_spawn_areas(
    mut visualizers: Query<&mut Visibility, With<InitialSpawnAreaViz>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
    config: Res<Config>, // Need config to check the specific setting name potentially
) {
    // Check if the setting exists in the config struct to avoid panic if name changes
    // This check might be overly cautious if DrawSetting enum is kept in sync
    let setting_name = "spawn_areas"; // Match the field name in DrawSection

    for event in evr_draw_settings.read() {
        // TODO: This matching logic needs refinement.
        // We need a way to map the event's setting (which might be an enum variant)
        // back to the field name or have a dedicated enum variant for these areas.
        // For now, assuming a direct string match or similar mechanism exists in DrawSettingsEvent handling.
        // Placeholder: Directly check the config bool for now, assuming the event triggers a re-check.
        // A better approach would involve modifying DrawSettingsEvent or DrawSetting enum.

        // Let's assume DrawSettingsEvent carries enough info or we react based on config change
        // If DrawSettingsEvent had a field like `setting_name: String`, we could use:
        // if event.setting_name == setting_name { ... }

        // Simplified approach: React to *any* DrawSettingsEvent by checking the current config value.
        // This isn't ideal but works if the UI updates the config resource before sending the event.
        let draw = config.visualisation.draw.spawn_areas;
        for mut visibility in &mut visualizers {
             if draw {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }

        // Ideal approach (requires changes to DrawSetting/DrawSettingsEvent):
        // if matches!(event.setting, crate::input::DrawSetting::SpawnAreas) { // Assuming SpawnAreas variant exists
        //     for mut visibility in &mut visualizers {
        //         if event.draw {
        //             *visibility = Visibility::Visible;
        //         } else {
        //             *visibility = Visibility::Hidden;
        //         }
        //     }
        // }
    }
}


/// **Bevy** [`Update`] system
/// Reads [`DrawSettingsEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::WaypointAreas` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`WaypointAreaViz`] entities
fn show_or_hide_waypoint_areas(
    mut visualizers: Query<&mut Visibility, With<WaypointAreaViz>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
    config: Res<Config>, // Need config to check the specific setting name potentially
) {
     // Similar logic as show_or_hide_spawn_areas
    let setting_name = "waypoint_areas";

    for event in evr_draw_settings.read() {
        // Simplified approach: React to *any* DrawSettingsEvent by checking the current config value.
        let draw = config.visualisation.draw.waypoint_areas;
         for mut visibility in &mut visualizers {
             if draw {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
        // Ideal approach (requires changes to DrawSetting/DrawSettingsEvent):
        // if matches!(event.setting, crate::input::DrawSetting::WaypointAreas) { // Assuming WaypointAreas variant exists
        //     for mut visibility in &mut visualizers {
        //         if event.draw {
        //             *visibility = Visibility::Visible;
        //         } else {
        //             *visibility = Visibility::Hidden;
        //         }
        //     }
        // }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn spawn_formation(
    mut commands: Commands,
    mut evr_robot_formation_spawned: EventReader<RobotFormationSpawned>,
    mut evw_robot_spawned: EventWriter<RobotSpawned>,
    mut evw_waypoint_created: EventWriter<WaypointCreated>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
    env_config: Res<gbp_environment::Environment>,
    theme: Res<CatppuccinTheme>,
    simulation_manager: Res<SimulationManager>,
    sdf: Res<Sdf>,
    mut prng: ResMut<GlobalEntropy<bevy_prng::WyRand>>,
    mut mesh_assets: ResMut<Assets<Mesh>>, // Changed to mutable
    meshes: Res<Meshes>,                   // Added Meshes resource
    time_fixed: Res<Time<Fixed>>,
) {
    for event in evr_robot_formation_spawned.read() {
        let formation_group = simulation_manager
            .active_formation_group()
            .expect("there is an active formation group");

        let formation = &formation_group.formations[event.formation_group_index];

        let world_dims = {
            let tile_size = env_config.tiles.settings.tile_size as f64;
            let width = tile_size * env_config.tiles.grid.ncols() as f64;
            let height = tile_size * env_config.tiles.grid.nrows() as f64;
            WorldDimensions::new(width, height)
        };

        let max_placement_attempts = NonZeroUsize::new(1000).expect("1000 is not zero");

        let radii = (0..formation.robots)
            .map(|_| prng.gen_range(config.robot.radius.range()))
            .collect::<Vec<_>>();

        let Some((initial_position_for_each_robot, waypoint_positions_for_each_robot)) = formation
            .as_positions(
                world_dims,
                &radii, /* config.robot.radius,
                         * max_placement_attempts,
                         * &mut prng.rng as &mut dyn Rng,
                         * prng as &mut dyn Rng, */
                prng.deref_mut(),
            )
        else {
            error!(
                "failed to spawn formation {}, reason: was not able to place robots after {} attempts, skipping",
                event.formation_group_index,
                max_placement_attempts.get() // Assuming this is defined earlier for RandomSquare too
            );
            return;
        };

        // --- Spawn Initial Spawn Area Visualization ---
        if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = formation.initial_position.shape {
            let world_p1 = world_dims.point_to_world_position(p1);
            let world_p2 = world_dims.point_to_world_position(p2);
            let center = (world_p1 + world_p2) / 2.0;
            let size_vec = (world_p1 - world_p2).abs(); // Keep the actual size vector

            let mut material = StandardMaterial::from(Color::from_catppuccin_colour_with_alpha(
                theme.red(),
                0.3, // Semi-transparent red
            ));
            material.unlit = true; // Make it unlit so it's clearly visible
            material.cull_mode = None; // Render both sides

            commands.spawn((
                simulation_loader::Reloadable,
                InitialSpawnAreaViz,
                PbrBundle {
                    // Create a mesh with the exact dimensions needed
                    mesh: mesh_assets.add(Mesh::from(Rectangle::new(size_vec.x, size_vec.y))),
                    material: materials.add(material),
                    // No scaling needed now, just position and rotation
                    transform: Transform::from_xyz(center.x, -config.visualisation.height.objects + 0.01, center.y) // Slightly above ground
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)), // Rotate to be flat on XZ plane
                    visibility: if config.visualisation.draw.spawn_areas {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ..default()
                },
                PickableBundle::default(), // Optional: make it pickable if needed later
            ));
        }
        // --- End Spawn Initial Spawn Area Visualization ---


        let initial_pose_for_each_robot: Vec<Vec4> = initial_position_for_each_robot
            .iter()
            .zip(
                waypoint_positions_for_each_robot
                    .first()
                    .expect("there is at least one waypoint"),
            )
            .map(|(from, to)| {
                let d = *to - *from;
                let v = d.normalize_or_zero() * config.robot.target_speed.get();
                Vec4::new(from.x, from.y, v.x, v.y)
            })
            .collect();


        // --- Spawn Waypoint Area Visualizations ---
        for waypoint in formation.waypoints.iter() {
             if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = waypoint.shape {
                let world_p1 = world_dims.point_to_world_position(p1);
                let world_p2 = world_dims.point_to_world_position(p2);
                let center = (world_p1 + world_p2) / 2.0;
                let size_vec = (world_p1 - world_p2).abs(); // Keep the actual size vector

                let mut material = StandardMaterial::from(Color::from_catppuccin_colour_with_alpha(
                    theme.green(),
                    0.3, // Semi-transparent green
                ));
                material.unlit = true;
                material.cull_mode = None;

                commands.spawn((
                    simulation_loader::Reloadable,
                    WaypointAreaViz, // Use the correct marker component
                    PbrBundle {
                         // Create a mesh with the exact dimensions needed
                        mesh: mesh_assets.add(Mesh::from(Rectangle::new(size_vec.x, size_vec.y))),
                        material: materials.add(material),
                         // No scaling needed now, just position and rotation
                        transform: Transform::from_xyz(center.x, -config.visualisation.height.objects + 0.01, center.y)
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                        visibility: if config.visualisation.draw.waypoint_areas { // Use the correct draw setting
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        ..default()
                    },
                    PickableBundle::default(),
                ));
            }
        }
        // --- End Spawn Waypoint Area Visualizations ---


        let waypoint_poses_for_each_robot: Vec<Vec<Vec4>> = waypoint_positions_for_each_robot
            .iter()
            .chain(waypoint_positions_for_each_robot.last().into_iter())
            .tuple_windows()
            .map(|(a, b)| {
                a.iter()
                    .zip(b.iter())
                    .map(|(from, to)| {
                        let d = *to - *from;
                        let v = d.normalize_or_zero() * config.robot.target_speed.get();
                        Vec4::new(from.x, from.y, v.x, v.y)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        for (i, initial_pose_vec4) in initial_pose_for_each_robot.iter().enumerate() {
            
            // Construct waypoints for this specific robot
            let mut waypoints_statevector: Vec<StateVector> = std::iter::once(initial_pose_vec4) // Start at initial pose (&Vec4)
                .chain(waypoint_poses_for_each_robot.iter().map(|wps| &wps[i])) // Chain with other &Vec4
                .copied() // Convert iterator of &Vec4 to iterator of Vec4
                .map_into::<StateVector>() // Convert iterator of Vec4 to iterator of StateVector
                .collect::<Vec<_>>();

            // Ensure the last waypoint has appropriate velocity (e.g., zero or copied from second last)
            if waypoints_statevector.len() >= 2 {
                let second_last_vel = waypoints_statevector[waypoints_statevector.len() - 2].velocity();
                waypoints_statevector.last_mut().unwrap().update_velocity(second_last_vel);
            }

            let initial_state_vec = StateVector::new(*initial_pose_vec4);
            let radius = radii[i];
            let target_speed = config.robot.target_speed.get(); 

            // Convert the Vec<StateVector> into TwoOrMore<StateVector> for the helper function
            let waypoints_for_mission: TwoOrMore<StateVector> = waypoints_statevector
                .try_into()
                .expect("Waypoints vec should have >= 2 elements");


            // Call the helper function
            let _robot_entity = crate::planner::spawn_utils::spawn_robot(
                &mut commands,
                &config,
                &env_config,
                &sdf,
                &mut prng,
                &mut materials,
                &mut mesh_assets,
                &theme,
                &time_fixed,
                &mut evw_robot_spawned,
                &mut evw_waypoint_created,
                // Robot specific parameters
                initial_state_vec,
                waypoints_for_mission, // Pass the constructed TwoOrMore<StateVector>
                radius,
                formation.planning_strategy,
                target_speed,
                formation.waypoint_reached_when_intersects,
                formation.finished_when_intersects,
                None, // No initial custom weights from formation spawner
            );
        }
    }
}

// TODO: move into another module
#[derive(Event)]
pub struct RobotClickedOn(pub Entity);

impl From<ListenerInput<Pointer<Click>>> for RobotClickedOn {
    fn from(value: ListenerInput<Pointer<Click>>) -> Self {
        Self(value.target)
    }
}

struct DelayTimer(pub Timer);

impl Default for DelayTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), TimerMode::Once))
    }
}

fn exit_application_on_scenario_finished(
    mut evr_all_formations_finished: EventReader<AllFormationsFinished>,
    config: Res<Config>,
    mut evw_app_exit: EventWriter<bevy::app::AppExit>,
    mut timer: Local<Option<DelayTimer>>,
    time: Res<Time>,
) {
    match *timer {
        Some(ref mut timer) => {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                evw_app_exit.send(bevy::app::AppExit);
            }
        }
        None => {}
    }

    for _ in evr_all_formations_finished.read() {
        if config.simulation.exit_application_on_scenario_finished {
            if timer.is_none() {
                *timer = Some(DelayTimer::default());
            }
        }
    }
}
