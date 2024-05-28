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
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour},
    utils::get_variable_timesteps,
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
    // time_virtual: Res<Time<Virtual>>,
    time_fixed: Res<Time<Fixed>>,
) {
    for event in evr_robot_formation_spawned.read() {
        let formation_group = simulation_manager
            .active_formation_group()
            .expect("there is an active formation group");

        let formation = &formation_group.formations[event.formation_group_index];
        // TODO: check this gets reloaded correctly

        let world_dims = {
            let tile_size = env_config.tiles.settings.tile_size as f64;
            let width = tile_size * env_config.tiles.grid.ncols() as f64;
            let height = tile_size * env_config.tiles.grid.nrows() as f64;
            WorldDimensions::new(width, height)
        };

        let max_placement_attempts = NonZeroUsize::new(1000).expect("1000 is not zero");

        let radii = (0..formation.robots.get())
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
                "failed to spawn formation {}, reason: was not able to place robots along line \
                 segment after {} attempts, skipping",
                event.formation_group_index,
                max_placement_attempts.get()
            );
            return;
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
                let v = d.normalize_or_zero() * config.robot.max_speed.get();
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
                        let v = d.normalize_or_zero() * config.robot.max_speed.get();
                        Vec4::new(from.x, from.y, v.x, v.y)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let min_radius = radii
            .iter()
            .copied()
            .map(ordered_float::OrderedFloat)
            .min()
            .expect("not empty");
        for (i, initial_pose) in initial_pose_for_each_robot.iter().enumerate() {
            let mut waypoints: Vec<Vec4> = waypoint_poses_for_each_robot
                .iter()
                .map(|wps| wps[i])
                .collect();
            trace!(
                "initial pose: {:?}, waypoints: {:?}",
                initial_pose,
                waypoints
            );

            let initial_direction = initial_pose.yz().extend(0.0);
            let initial_translation = Vec3::new(initial_pose.x, -1.5, initial_pose.y);
            // let initial_translation = Vec3::new(initial_pose.x, -5.5, initial_pose.y);

            let mut entity = commands.spawn_empty();
            let robot_entity = entity.id();
            evw_waypoint_created.send_batch(waypoints.iter().map(|pose| WaypointCreated {
                for_robot: robot_entity,
                position:  pose.xy(),
            }));

            // let second_last = waypoints.get(waypoints.len() - 2).copied().unwrap();
            // let last = waypoints.last_mut().unwrap();
            // last.z = second_last.z;
            // last.w = second_last.w;

            // let mu
            let mut waypoints = std::iter::once(initial_pose)
                .chain(waypoints.iter())
                .copied()
                .map_into::<StateVector>()
                .collect::<Vec<_>>();

            let second_last = waypoints.get(waypoints.len() - 2).copied().unwrap();
            let last = waypoints.last_mut().unwrap();
            last.update_velocity(second_last.velocity());
            // last.z = second_last.z;
            // last.w = second_last.w;
            //

            // let lookahead_horizon = (5.0 / 0.25) as u32;
            // let lookahead_multiple = 3;

            //     globals.T_HORIZON / globals.T0, globals.LOOKAHEAD_MULTIPLE);
            // num_variables_ = variable_timesteps.size();
            let t0: f32 = radii[i] / 2.0 / config.robot.max_speed.get();

            // let lookahead_horizon: u32 = (config.robot.planning_horizon.get() / t0) as
            // u32;
            let divisor: f32 = (min_radius / 2.0 / config.robot.max_speed.get()).into();

            let lookahead_horizon: u32 = (config.robot.planning_horizon.get() / divisor) as u32;
            // let lookahead_horizon: u32 = (config.robot.planning_horizon.get()
            //     / radii.iter().map(ordered_float::OrderedFloat).min().unwrap())
            //     as u32;
            let lookahead_multiple = config.gbp.lookahead_multiple as u32;
            let variable_timesteps = get_variable_timesteps(lookahead_horizon, lookahead_multiple);

            let robotbundle = RobotBundle::new(
                robot_entity,
                StateVector::new(*initial_pose),
                // route,
                variable_timesteps.as_slice(),
                &config,
                &env_config,
                radii[i],
                &sdf.0,
                time_fixed.elapsed().as_secs_f64(),
                waypoints.try_into().unwrap(),
                // config
                formation.planning_strategy,
                formation.waypoint_reached_when_intersects,
                formation.finished_when_intersects,
                // matches!(formation.planning_strategy, PlanningStrategy::RrtStar
                // ),
            );

            let initial_visibility = if config.visualisation.draw.robots {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            let random_color = DisplayColour::iter()
                .choose(prng.deref_mut())
                .expect("there is more than 0 colors");

            let material = materials.add(StandardMaterial {
                base_color: Color::from_catppuccin_colour(theme.get_display_colour(&random_color)),
                ..Default::default()
            });

            let mesh = mesh_assets.add(
                Sphere::new(radii[i])
                    .mesh()
                    .ico(2)
                    .expect("4 subdivisions is less than the maximum allowed of 80"),
            );

            let pbrbundle = PbrBundle {
                mesh,
                material,
                transform: Transform::from_translation(initial_translation),
                visibility: initial_visibility,
                ..Default::default()
            };

            entity.insert((
                robotbundle,
                pbrbundle,
                prng.fork_rng(),
                simulation_loader::Reloadable,
                // super::tracking::PositionTracker::new(1000, Duration::from_millis(50)),
                // super::tracking::VelocityTracker::new(1000, Duration::from_millis(50)),
                super::tracking::PositionTracker::new(5000, Duration::from_millis(100)),
                super::tracking::VelocityTracker::new(5000, Duration::from_millis(100)),
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<RobotClickedOn>(),
                ColorAssociation { name: random_color },
                FollowCameraMe::new(0.0, 30.0, 0.0)
                    .with_up_direction(Direction3d::new(initial_direction).expect(
                        "Vector between initial position and first waypoint should be different \
                         from 0, NaN, and infinity.",
                    ))
                    .with_attached(true),
                crate::goal_area::components::Collider(Box::new(parry2d::shape::Ball::new(
                    radii[i],
                ))),
            ));

            evw_robot_spawned.send(RobotSpawned(robot_entity));
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
