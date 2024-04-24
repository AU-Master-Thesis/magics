use std::{collections::VecDeque, num::NonZeroUsize, time::Duration};

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;

use super::{
    robot::{RobotSpawned, VariableTimesteps},
    RobotId,
};
use crate::{
    // asset_loader::SceneAssets,
    asset_loader::{Meshes, Obstacles},
    config::{
        formation::{RepeatTimes, Waypoint, WorldDimensions},
        geometry::{Point, RelativePoint, Shape},
        Config, FormationGroup,
    },
    environment::FollowCameraMe,
    pause_play::PausePlay,
    planner::robot::{RobotBundle, StateVector},
    simulation_loader::{
        self, EndSimulation, LoadSimulation, ReloadSimulation, Sdf, SimulationManager,
    },
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour},
};

pub struct RobotSpawnerPlugin;

impl Plugin for RobotSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RobotFormationSpawned>()
            .add_event::<RobotClickedOn>()
            .add_event::<WaypointCreated>()
            .add_event::<RobotWaypointReached>()
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
                ),
            );
    }
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

#[derive(Event)]
pub struct RobotWaypointReached(pub Entity);

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
struct RepeatingTimer {
    timer:  Timer,
    repeat: RepeatTimes,
}

impl RepeatingTimer {
    fn new(duration: Duration, repeat: RepeatTimes) -> Self {
        let timer = Timer::new(duration, TimerMode::Repeating);
        Self { timer, repeat }
    }

    #[inline]
    pub fn exhausted(&self) -> bool {
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
        // self.timer.just_finished() && !self.repeat.exhausted()
        let finished = self.timer.just_finished() && !self.repeat.exhausted();
        if finished {
            self.repeat.decrement();
        }

        finished
    }

    #[inline]
    pub fn duration(&self) -> Duration {
        self.timer.duration()
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
    // OnCooldown,
    Finished,
}

impl FormationSpawner {
    #[must_use]
    pub fn new(
        formation_group_index: usize,
        initial_delay: Duration,
        mut timer: RepeatingTimer,
    ) -> Self {
        // timer.tick(timer.duration());

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
                    self.state = Active { on_cooldown: false }
                }
            }
            Active { on_cooldown: false } | Finished => {}
        }

        // if !matches!(self.state, FormationSpawnerState::Active { on_cooldown:
        // true }) if self.is_active() {
        //     self.timer.tick(delta);
        // } else {
        //     self.initial_delay.tick(delta);
        // }
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
        };
    }

    #[inline]
    fn ready_to_spawn(&mut self) -> bool {
        matches!(self.state, FormationSpawnerState::Active {
            on_cooldown: false,
        })
        // self.timer.just_finished()
    }

    // #[inline]
    // fn on_cooldown(&mut self) -> bool {
    //     !self.ready_to_spawn()
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

fn create_formation_group_spawners(
    mut commands: Commands,
    simulation_manager: Res<SimulationManager>,
) {
    let Some(formation_group) = simulation_manager.active_formation_group() else {
        warn!("No active formation group!");
        return;
    };

    for (i, formation) in formation_group.formations.iter().enumerate() {
        #[allow(clippy::option_if_let_else)] // find it more readable with a match here

        // let timer = match formation.repeat {
        //      Some(duration) => Timer::new(duration, TimerMode::Repeating),
        //      None => Timer::from_seconds(0.1, TimerMode::Once),
        //  };
        let repeating_timer = match formation.repeat {
            Some(repeat) => RepeatingTimer::new(repeat.every, repeat.times),
            None => RepeatingTimer::new(Duration::from_secs(0), RepeatTimes::ONCE),
        };

        // let initial_delay = Timer::new(formation.delay, TimerMode::Once);

        info!(
            "spawning FormationSpawner[{i}] with delay {:?} and timer {:?}",
            formation.delay, repeating_timer
        );

        commands.spawn(dbg!(FormationSpawner::new(
            i,
            formation.delay,
            repeating_timer
        )));

        // commands.spawn(FormationSpawner {
        //     formation_group_index: i,
        //     initial_delay: delay,
        //     timer,
        // });
    }
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
    theme: Res<CatppuccinTheme>,
    // formation_group: Res<FormationGroup>,
    simulation_manager: Res<SimulationManager>,
    variable_timesteps: Res<VariableTimesteps>,
    // scene_assets: Res<SceneAssets>,
    meshes: Res<Meshes>,
    // obstacles: Res<Obstacles>,
    sdf: Res<Sdf>,
    // obstacle_sdf: Res<ObstacleSdf>,
    image_assets: ResMut<Assets<Image>>,
) {
    for event in evr_robot_formation_spawned.read() {
        // only continue if the image has been loaded
        // let Some(image) = image_assets.get(&obstacles.sdf) else {
        //     error!("obstacle sdf not loaded yet");
        //     return;
        // };

        // let _ = OBSTACLE_IMAGE.get_or_init(|| image.clone());

        let formation_group = simulation_manager
            .active_formation_group()
            .expect("there is an active formation group");

        let formation = &formation_group.formations[event.formation_group_index];

        // dbg!(&formation);

        // TODO: check this gets reloaded correctly
        let world_dims = WorldDimensions::new(
            config.simulation.world_size.get().into(),
            config.simulation.world_size.get().into(),
        );

        // TODO: use random resource/component for reproducibility
        let mut rng = rand::thread_rng();
        let max_placement_attempts = NonZeroUsize::new(1000).expect("1000 is not zero");

        let Some((initial_position_for_each_robot, waypoint_positions_for_each_robot)) = formation
            .as_positions(
                world_dims,
                config.robot.radius,
                // max_placement_attempts,
                &mut rng,
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

        // dbg!(&initial_position_for_each_robot);
        // dbg!(&waypoint_positions_for_each_robot);

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

        // dbg!(&initial_pose_for_each_robot);
        // dbg!(&waypoint_poses_for_each_robot);

        for (i, initial_pose) in initial_pose_for_each_robot.iter().enumerate() {
            let waypoints: Vec<Vec4> = waypoint_poses_for_each_robot
                .iter()
                .map(|wps| wps[i])
                .collect();
            // }
            // for (initial_pose, waypoints) in initial_pose_for_each_robot
            //     .iter()
            //     .zip(waypoint_poses_for_each_robot.iter())
            // {
            info!(
                "initial pose: {:?}, waypoints: {:?}",
                initial_pose, waypoints
            );
            let initial_direction = initial_pose.yz().extend(0.0);
            let initial_translation = Vec3::new(initial_pose.x, 0.5, initial_pose.y);

            let mut entity = commands.spawn_empty();
            let robot_id = entity.id();
            evw_waypoint_created.send_batch(waypoints.iter().map(|pose| WaypointCreated {
                for_robot: robot_id,
                position:  pose.xy(),
            }));

            let waypoints: VecDeque<_> = waypoints.iter().copied().collect();
            let robotbundle = RobotBundle::new(
                robot_id,
                StateVector::new(*initial_pose),
                waypoints,
                variable_timesteps.as_slice(),
                &config,
                &sdf.0,
                // image,
                // scene_assets.obstacle_image_sdf.clone_weak(),
                // obstacle_sdf,
                // OBSTACLE_IMAGE
                //     .get()
                //     .expect("obstacle image should be allocated and initialised"),
            )
            .expect(
                "Possible `RobotInitError`s should be avoided due to the formation input being \
                 validated.",
            );

            let initial_visibility = if config.visualisation.draw.robots {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            // TODO: Make this depend on random seed
            // let random_color = theme.into_display_iter().choose(&mut
            // thread_rng()).expect(     "Choosing random colour from an
            // iterator that is hard-coded with values should be \      ok.",
            // );
            let random_color = DisplayColour::iter()
                .choose(&mut thread_rng())
                .expect("there is more than 0 colors");

            let material = materials.add(StandardMaterial {
                base_color: Color::from_catppuccin_colour(theme.get_display_colour(&random_color)),
                ..Default::default()
            });

            let pbrbundle = PbrBundle {
                mesh: meshes.robot.clone(),
                material,
                transform: Transform::from_translation(initial_translation),
                visibility: initial_visibility,
                ..Default::default()
            };

            entity.insert((
                robotbundle,
                pbrbundle,
                simulation_loader::Reloadable,
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<RobotClickedOn>(),
                ColorAssociation { name: random_color },
                FollowCameraMe::new(0.0, 30.0, 0.0)
                    .with_up_direction(Direction3d::new(initial_direction).expect(
                        "Vector between initial position and first waypoint should be different \
                         from 0, NaN, and infinity.",
                    ))
                    .with_attached(true),
            ));

            evw_robot_spawned.send(RobotSpawned(robot_id));
        }
    }
}

#[derive(Event)]
struct RobotClickedOn(pub Entity);

impl RobotClickedOn {
    #[inline]
    pub const fn target(&self) -> Entity {
        self.0
    }
}

impl From<ListenerInput<Pointer<Click>>> for RobotClickedOn {
    fn from(value: ListenerInput<Pointer<Click>>) -> Self {
        Self(value.target)
    }
}
