use std::{
    num::NonZeroUsize,
    ops::DerefMut,
    sync::{Arc, RwLock},
    time::Duration,
};

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_notify::ToastEvent;
use bevy_rand::prelude::{ForkableRng, GlobalEntropy};
use gbp_config::{
    formation::{select_shape_helper, PlanningStrategy, RepeatTimes, WorldDimensions}, /* Added select_shape_helper */
    geometry::Shape, // Added Shape
    Config,
};
use itertools::Itertools;
use min_len_vec::{two_or_more, TwoOrMore}; // Added import
use rand::{seq::IteratorRandom, Rng};
use strum::IntoEnumIterator;

use super::{
    robot::{RobotFinishedRoute, RobotSpawned},
    spawn_utils::{place_single_robot, spawn_robot}, /* Corrected path for place_single_robot and
                                                     * added spawn_robot */
    RobotId,
};
use crate::{
    // asset_loader::SceneAssets,
    asset_loader::Meshes,
    bevy_utils::run_conditions::event_exists, // Import event_exists
    environment::FollowCameraMe,
    pause_play::PausePlay,
    planner::robot::{RobotBundle, Route, StateVector},
    planner::visualiser::{GoalPositionAreaViz, InitialSpawnAreaViz, WaypointAreaViz}, /* Import new components */
    simulation_loader::{
        self, EndSimulation, LoadSimulation, ReloadSimulation, Sdf, SimulationManager,
    },
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour}, /* Restore theme imports */
    utils::get_variable_timesteps,
};

/// Helper function to spawn a square visualization mesh.
fn spawn_square_viz_helper<MC: Component + Default>(
    commands: &mut Commands,
    mesh_assets: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    theme: &CatppuccinTheme,
    config: &Config,
    world_dims: &WorldDimensions,
    p1: gbp_config::geometry::Point,
    p2: gbp_config::geometry::Point,
    color: Color,
    visibility_flag: bool,
) {
    let world_p1 = world_dims.point_to_world_position(p1);
    let world_p2 = world_dims.point_to_world_position(p2);
    let center = (world_p1 + world_p2) / 2.0;
    let size_vec = (world_p1 - world_p2).abs();

    let mut material = StandardMaterial::from(color);
    material.unlit = true;
    material.cull_mode = None;

    commands.spawn((
        simulation_loader::Reloadable,
        MC::default(), // Use the generic marker component's default
        PbrBundle {
            mesh: mesh_assets.add(Mesh::from(Rectangle::new(size_vec.x, size_vec.y))),
            material: materials.add(material),
            transform: Transform::from_xyz(
                center.x,
                -config.visualisation.height.objects + 0.01,
                center.y,
            )
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            visibility: if visibility_flag {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            ..default()
        },
        PickableBundle::default(),
    ));
}

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
                    exit_application_on_scenario_finished, // exit_application_on_scenario_finished.run_if(on_event::<AllFormationsFinished>())
                ),
            )
            .add_systems(
                Update,
                (
                    track_score.run_if(resource_exists::<Scoreboard>),
                    notify_on_all_formations_finished.run_if(on_event::<AllFormationsFinished>()),
                    // Add systems to toggle visibility
                    show_or_hide_spawn_areas
                        .run_if(event_exists::<crate::input::DrawSettingsEvent>),
                    show_or_hide_waypoint_areas
                        .run_if(event_exists::<crate::input::DrawSettingsEvent>),
                    show_or_hide_goal_position_areas
                        .run_if(event_exists::<crate::input::DrawSettingsEvent>),
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
    }

    #[inline]
    pub fn just_finished(&mut self) -> bool {
        let finished = self.timer.just_finished() && !self.repeat.exhausted();
        if finished {
            self.repeat.decrement();
        }

        finished
    }
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
        matches!(self.state, FormationSpawnerState::Active { .. })
    }

    #[inline]
    pub const fn exhausted(&self) -> bool {
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
}

fn delete_formation_group_spawners(
    mut commands: Commands,
    formation_spawners: Query<Entity, With<FormationSpawner>>,
    // Query using a component unique to robots, e.g., Mission
    robots: Query<Entity, With<crate::planner::robot::Mission>>,
    initial_viz: Query<Entity, With<InitialSpawnAreaViz>>,
    waypoint_viz: Query<Entity, With<WaypointAreaViz>>,
    goal_viz: Query<Entity, With<GoalPositionAreaViz>>,
) {
    info!("Despawning formation spawners, robots, and visualization entities...");
    for spawner in formation_spawners.iter() {
        // Use .iter()
        // info!("despawning formation spawner: {:?}", spawner);
        commands.entity(spawner).despawn_recursive();
    }
    for robot in robots.iter() {
        // Use .iter()
        // info!("despawning robot: {:?}", robot);
        commands.entity(robot).despawn_recursive();
    }
    for viz in initial_viz.iter() {
        // Use .iter()
        // info!("despawning initial viz: {:?}", viz);
        commands.entity(viz).despawn_recursive();
    }
    for viz in waypoint_viz.iter() {
        // Use .iter()
        // info!("despawning waypoint viz: {:?}", viz);
        commands.entity(viz).despawn_recursive();
    }
    for viz in goal_viz.iter() {
        // Use .iter()
        // info!("despawning goal viz: {:?}", viz);
        commands.entity(viz).despawn_recursive();
    }
    info!("Despawning complete.");
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

#[derive(Debug, Event)]
pub struct RobotFormationSpawned {
    pub formation_group_index: usize,
}

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
                evw_pause_play.send(PausePlay::Pause);
            }
        }
    }
}

fn show_or_hide_spawn_areas(
    mut visualizers: Query<&mut Visibility, With<InitialSpawnAreaViz>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
    config: Res<Config>,
) {
    for _ in evr_draw_settings.read() {
        let draw = config.visualisation.draw.spawn_areas;
        for mut visibility in &mut visualizers {
            if draw {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn show_or_hide_waypoint_areas(
    mut visualizers: Query<&mut Visibility, With<WaypointAreaViz>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
    config: Res<Config>,
) {
    for _ in evr_draw_settings.read() {
        let draw = config.visualisation.draw.waypoint_areas;
        for mut visibility in &mut visualizers {
            if draw {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn show_or_hide_goal_position_areas(
    mut visualizers: Query<&mut Visibility, With<GoalPositionAreaViz>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
    config: Res<Config>,
) {
    for _ in evr_draw_settings.read() {
        let draw = config.visualisation.draw.goal_position_areas;
        for mut visibility in &mut visualizers {
            if draw {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
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
    mut mesh_assets: ResMut<Assets<Mesh>>,
    meshes: Res<Meshes>,
    time_fixed: Res<Time<Fixed>>,
) {
    for event in evr_robot_formation_spawned.read() {
        let formation_group = simulation_manager
            .active_formation_group()
            .expect("there is an active formation group");

        let formation = &formation_group.formations[event.formation_group_index];

        // --- Validation ---
        if !formation.waypoints.is_empty() && formation.goal_position.is_some() {
            error!(
                "Formation {} cannot define both 'waypoints' and 'goal_position'. Skipping.",
                event.formation_group_index
            );
            continue; // Skip this formation
        }

        if formation.waypoints.is_empty() && formation.goal_position.is_none() {
            error!(
                "Formation {} must define either 'waypoints' or 'goal_position'. Skipping.",
                event.formation_group_index
            );
            continue; // Skip this formation
        }

        if !formation.waypoints.is_empty() && formation.initial_position.multi_shapes.is_some() {
            warn!(
                "Formation {} defines 'waypoints' and 'initial_position.multi_shapes'. \
                 'multi_shapes' will be ignored for initial placement with waypoints.",
                event.formation_group_index
            );
            if formation.initial_position.shape.is_none() {
                error!(
                    "Formation {} defines 'waypoints' but 'initial_position' is missing a single \
                     'shape' (required when using waypoints). Skipping.",
                    event.formation_group_index
                );
                continue;
            }
        }

        if formation.initial_position.shape.is_none()
            && formation.initial_position.multi_shapes.is_none()
        {
            error!(
                "Formation {} 'initial_position' must define either 'shape' or 'multi_shapes'. \
                 Skipping.",
                event.formation_group_index
            );
            continue;
        }
        // --- End Validation ---

        let world_dims = {
            let tile_size = env_config.tiles.settings.tile_size as f64;
            let width = tile_size * env_config.tiles.grid.ncols() as f64;
            let height = tile_size * env_config.tiles.grid.nrows() as f64;
            WorldDimensions::new(width, height)
        };

        let max_placement_attempts = 1000; // Max attempts for placing a single robot

        let radii = (0..formation.robots)
            .map(|_| prng.gen_range(config.robot.radius.range()))
            .collect::<Vec<_>>();

        // --- Spawn Visualization ---
        // (Visualization logic remains largely the same, spawning based on defined
        // shapes) --- Spawn Initial Spawn Area Visualization ---
        let initial_viz_color = Color::from_catppuccin_colour_with_alpha(theme.red(), 0.3);
        if let Some(shape) = &formation.initial_position.shape {
            if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = shape {
                spawn_square_viz_helper::<InitialSpawnAreaViz>(
                    &mut commands,
                    &mut mesh_assets,
                    &mut materials,
                    &theme,
                    &config,
                    &world_dims,
                    *p1,
                    *p2,
                    initial_viz_color,
                    config.visualisation.draw.spawn_areas,
                );
            }
        } else if let Some(shapes) = &formation.initial_position.multi_shapes {
            for shape in shapes {
                if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = shape {
                    spawn_square_viz_helper::<InitialSpawnAreaViz>(
                        &mut commands,
                        &mut mesh_assets,
                        &mut materials,
                        &theme,
                        &config,
                        &world_dims,
                        *p1,
                        *p2,
                        initial_viz_color,
                        config.visualisation.draw.spawn_areas,
                    );
                }
            }
        }
        // --- End Spawn Initial Spawn Area Visualization ---
        // --- Spawn Waypoint Area Visualizations ---
        let waypoint_viz_color = Color::from_catppuccin_colour_with_alpha(theme.blue(), 0.3);
        for waypoint in formation.waypoints.iter() {
            if let Some(shape) = &waypoint.shape {
                if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = shape {
                    spawn_square_viz_helper::<WaypointAreaViz>(
                        &mut commands,
                        &mut mesh_assets,
                        &mut materials,
                        &theme,
                        &config,
                        &world_dims,
                        *p1,
                        *p2,
                        waypoint_viz_color,
                        config.visualisation.draw.waypoint_areas,
                    );
                }
            } else if let Some(shapes) = &waypoint.multi_shapes {
                for shape in shapes {
                    if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = shape {
                        spawn_square_viz_helper::<WaypointAreaViz>(
                            &mut commands,
                            &mut mesh_assets,
                            &mut materials,
                            &theme,
                            &config,
                            &world_dims,
                            *p1,
                            *p2,
                            waypoint_viz_color,
                            config.visualisation.draw.waypoint_areas,
                        );
                    }
                }
            }
        }
        // --- End Spawn Waypoint Area Visualizations ---
        // --- Spawn Goal Position Area Visualizations ---
        let goal_viz_color = Color::from_catppuccin_colour_with_alpha(theme.green(), 0.3);
        if let Some(goal_positions) = &formation.goal_position {
            for goal in goal_positions.iter() {
                if let Some(shape) = &goal.shape {
                    if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = shape {
                        spawn_square_viz_helper::<GoalPositionAreaViz>(
                            &mut commands,
                            &mut mesh_assets,
                            &mut materials,
                            &theme,
                            &config,
                            &world_dims,
                            *p1,
                            *p2,
                            goal_viz_color,
                            config.visualisation.draw.goal_position_areas,
                        );
                    }
                } else if let Some(shapes) = &goal.multi_shapes {
                    for shape in shapes {
                        if let gbp_config::geometry::Shape::RandomSquare { p1, p2, .. } = shape {
                            spawn_square_viz_helper::<GoalPositionAreaViz>(
                                &mut commands,
                                &mut mesh_assets,
                                &mut materials,
                                &theme,
                                &config,
                                &world_dims,
                                *p1,
                                *p2,
                                goal_viz_color,
                                config.visualisation.draw.goal_position_areas,
                            );
                        }
                    }
                }
            }
        }
        // --- End Goal Position Area Visualizations ---
        // --- End Spawn Visualization ---

        // --- Branching Logic: Waypoints vs Goal Position ---
        if !formation.waypoints.is_empty() {
            // --- Waypoint-based Spawning (Existing Logic using as_positions) ---
            // Assumes initial_position uses a single shape (validated above)
            let Some((initial_position_for_each_robot, waypoint_positions_for_each_robot)) =
                formation.as_positions(world_dims, &radii, prng.deref_mut())
            else {
                error!(
                    "failed to spawn formation {} using waypoints, reason: as_positions failed, \
                     skipping",
                    event.formation_group_index,
                );
                continue; // Skip this formation
            };

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
                let mut waypoints_statevector: Vec<StateVector> =
                    std::iter::once(initial_pose_vec4)
                        .chain(waypoint_poses_for_each_robot.iter().map(|wps| &wps[i]))
                        .copied()
                        .map_into::<StateVector>()
                        .collect::<Vec<_>>();

                if waypoints_statevector.len() >= 2 {
                    let second_last_vel =
                        waypoints_statevector[waypoints_statevector.len() - 2].velocity();
                    waypoints_statevector
                        .last_mut()
                        .unwrap()
                        .update_velocity(second_last_vel);
                }

                let initial_state_vec = StateVector::new(*initial_pose_vec4);
                let radius = radii[i];
                let target_speed = config.robot.target_speed.get();

                let waypoints_for_mission: TwoOrMore<StateVector> = waypoints_statevector
                    .try_into()
                    .expect("Waypoints vec should have >= 2 elements");

                let _robot_entity = spawn_robot(
                    &mut commands,
                    &config,
                    &env_config,
                    sdf.0.clone(), // Access the Arc and clone it
                    &mut prng,
                    &mut materials,
                    &mut mesh_assets,
                    &theme,
                    &time_fixed,
                    &mut evw_robot_spawned,
                    &mut evw_waypoint_created,
                    initial_state_vec,
                    waypoints_for_mission,
                    radius,
                    formation.planning_strategy,
                    target_speed,
                    formation.waypoint_reached_when_intersects,
                    formation.finished_when_intersects,
                    None,
                );
            }
        // --- End Waypoint-based Spawning ---
        } else if let Some(goal_positions) = &formation.goal_position {
            // --- Goal-Position-based Spawning (New Per-Robot Logic) ---
            let mut placed_robots: Vec<(Vec2, f32)> = Vec::with_capacity(formation.robots);

            for i in 0..formation.robots {
                let radius = radii[i];

                // 1. Select Initial Shape
                let initial_shape = match select_shape_helper(
                    &formation.initial_position.shape,
                    &formation.initial_position.multi_shapes,
                    &format!("formation {} initial", event.formation_group_index),
                    &mut *prng,
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Skipping robot {}: {}", i, e);
                        continue;
                    }
                };

                // 2. Calculate Initial Position (with collision avoidance)
                let initial_pos = match place_single_robot(
                    &initial_shape,
                    &world_dims,
                    radius,
                    &placed_robots,
                    max_placement_attempts, // Use NonZeroUsize directly
                    &mut *prng,
                ) {
                    Some(pos) => pos,
                    None => {
                        error!(
                            "Failed to place robot {} for formation {} after {} attempts. \
                             Skipping robot.",
                            i, event.formation_group_index, max_placement_attempts
                        );
                        continue; // Skip this robot
                    }
                };
                placed_robots.push((initial_pos, radius)); // Add successfully placed robot

                // 3. Select Goal Shape
                // Assuming goal_position is OneOrMore, so unwrap is safe after validation
                let goal_waypoint = goal_positions
                    .iter()
                    .choose(&mut *prng) // Choose one Waypoint struct randomly
                    .expect("Goal position list should not be empty (validated earlier)");

                let goal_shape = match select_shape_helper(
                    &goal_waypoint.shape,
                    &goal_waypoint.multi_shapes,
                    &format!("formation {} goal", event.formation_group_index),
                    &mut *prng,
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Skipping robot {}: {}", i, e);
                        continue;
                    }
                };

                // 4. Calculate Goal Position
                let goal_pos = match goal_shape.get_random_point(&world_dims, &mut *prng) {
                    Some(pos) => pos,
                    None => {
                        error!(
                            "Failed to get random point in goal shape {:?} for robot {}. Skipping \
                             robot.",
                            goal_shape, i
                        );
                        continue;
                    }
                };

                // 5. Spawn Robot
                let initial_state_vec =
                    StateVector::new(Vec4::new(initial_pos.x, initial_pos.y, 0.0, 0.0)); // Start with zero velocity
                let goal_state_vec = StateVector::new(Vec4::new(goal_pos.x, goal_pos.y, 0.0, 0.0)); // Zero velocity at goal

                let waypoints_for_mission = two_or_more![initial_state_vec, goal_state_vec];
                let target_speed = config.robot.target_speed.get();

                let _robot_entity = spawn_robot(
                    &mut commands,
                    &config,
                    &env_config,
                    sdf.0.clone(), // Access the Arc and clone it
                    &mut prng,
                    &mut materials,
                    &mut mesh_assets,
                    &theme,
                    &time_fixed,
                    &mut evw_robot_spawned,
                    &mut evw_waypoint_created,
                    initial_state_vec,
                    waypoints_for_mission,
                    radius,
                    formation.planning_strategy,
                    target_speed,
                    formation.waypoint_reached_when_intersects,
                    formation.finished_when_intersects,
                    None, // No initial custom weights from spawner
                );
            }
            // --- End Goal-Position-based Spawning ---
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
