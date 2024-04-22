use std::{collections::VecDeque, num::NonZeroUsize, sync::OnceLock, time::Duration};

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;
use typed_floats::StrictlyPositiveFinite;

use super::{
    robot::{RobotSpawned, VariableTimesteps},
    RobotId,
};
use crate::{
    // asset_loader::SceneAssets,
    asset_loader::{Meshes, Obstacles},
    config::{
        formation::{Waypoint, WorldDimensions},
        geometry::{Point, RelativePoint, Shape},
        Config, FormationGroup,
    },
    environment::FollowCameraMe,
    pause_play::PausePlay,
    planner::robot::{RobotBundle, StateVector},
    simulation_loader::{self, EndSimulation, LoadSimulation, ReloadSimulation, SimulationManager},
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour},
};

pub struct RobotSpawnerPlugin;

impl Plugin for RobotSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RobotFormationSpawned>()
            .add_event::<RobotClickedOn>()
            .add_event::<WaypointCreated>()
            .add_event::<WaypointReached>()
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
pub struct WaypointReached(pub Entity);

// TODO: allocate for each obstacle factor, a bit wasteful but should not take
// up to much memory like 8-10 MB
// TODO: needs to be changed whenever the sim reloads, use resource?
/// Every [`ObstacleFactor`] has a static reference to the obstacle image.
// static OBSTACLE_IMAGE: OnceLock<Image> = OnceLock::new();
// TODO: use once_cell, so we can mutate it when sim reloads
// static OBSTACLE_SDF: Lazy<RwLock<Image>> = Lazy::new(||
// RwLock::new(Image::new(1, 1)));

/// Component attached to an entity that spawns formations.
#[derive(Component)]
pub struct FormationSpawnerCountdown {
    pub timer: Timer,
    pub formation_group_index: usize,
}

#[derive(Component)]
pub struct FormationSpawner {
    pub formation_group_index: usize,
    initial_delay: Timer,
    timer: Timer,
}

impl FormationSpawner {
    #[must_use]
    pub fn new(formation_group_index: usize, initial_delay: Duration, timer: Timer) -> Self {
        Self {
            formation_group_index,
            initial_delay: Timer::new(initial_delay, TimerMode::Once),
            timer,
        }
    }

    #[inline]
    fn is_active(&self) -> bool {
        self.initial_delay.finished()
    }

    fn tick(&mut self, delta: Duration) {
        if self.is_active() {
            self.timer.tick(delta);
        } else {
            self.initial_delay.tick(delta);
        }
    }

    #[inline]
    fn ready_to_spawn(&self) -> bool {
        self.timer.finished()
    }

    #[inline]
    fn on_cooldown(&self) -> bool {
        !self.ready_to_spawn()
    }
}

// #[derive(Component)]
// pub struct CreateFormationSpawnerAfterDelay {
//     pub delay: Timer,
//     pub formation_group_index: usize,
// }

/// Create an entity with a a `FormationSpawnerCountdown` component for each
/// formation group.
fn setup(mut commands: Commands, formation_group: Res<FormationGroup>) {
    for (i, formation) in formation_group.formations.iter().enumerate() {
        // let timer = formation.repeat_every

        // let mode = if formation.repeat_every {
        //     TimerMode::Repeating
        // } else {
        //     TimerMode::Once
        // };
        // let duration = formation.delay.as_secs_f32();
        // let timer = Timer::from_seconds(duration, mode);

        // commands.spawn(FormationSpawnerCountdown {
        //     timer,
        //     formation_group_index: i,
        // });

        // let mut entity = commands.spawn_empty();
        // entity.insert(FormationSpawnerCountdown {
        //     timer,
        //     formation_group_index: i,
        // });

        // info!(
        //     "spawned formation-group spawner: {} with mode {:?} spawning
        // every {} seconds",     i, mode, duration
        // );
    }
}

/// Create an entity with a a `FormationSpawnerCountdown` component for each
/// formation group.
fn setup_v2(commands: &mut Commands, formation_group: &FormationGroup) {
    for (i, formation) in formation_group.formations.iter().enumerate() {
        // let mode = if formation.repeat_every {
        //     TimerMode::Repeating
        // } else {
        //     TimerMode::Once
        // };
        // let duration = formation.delay.as_secs_f32();
        // let timer = Timer::from_seconds(duration, mode);

        // let mut entity = commands.spawn_empty();
        // entity.insert(FormationSpawnerCountdown {
        //     timer,
        //     formation_group_index: i,
        // });

        // info!(
        //     "spawned formation-group spawner: {} with mode {:?} spawning
        // every {} seconds",     i + 1,
        //     mode,
        //     duration
        // );
    }
}

fn delete_formation_group_spawners(
    mut commands: Commands,
    formation_spawners: Query<Entity, With<FormationSpawner>>,
) {
    for spawner in &formation_spawners {
        error!("despawning formation spawner: {:?}", spawner);
        error!("asdlsahdasldha");
        commands.entity(spawner).despawn();
    }
}

fn create_formation_group_spawners(
    mut commands: Commands,
    simulation_manager: Res<SimulationManager>,
    // existing_formation_spawners: Query<Entity, With<FormationSpawnerCountdown>>,
    // existing_formation_spawners: Query<Entity, With<FormationSpawner>>,
) {
    let Some(formation_group) = simulation_manager.active_formation_group() else {
        warn!("No active formation group!");
        return;
    };

    // for spawner in &existing_formation_spawners {
    //     commands.entity(spawner).despawn();
    //     info!("Despawned formation spawner: {:?}", spawner);
    // }

    // dbg!(&formation_group);
    // std::process::exit(1);

    for (i, formation) in formation_group.formations.iter().enumerate() {
        #[allow(clippy::option_if_let_else)] // find it more readable with a match here
        let timer = match formation.repeat_every {
            Some(duration) => {
                let mut timer = Timer::new(duration, TimerMode::Repeating);
                // FIXME: does not work
                // timer.tick(duration); // tick the timer so it is finished on the first tick,
                // after                       // the delay has finished
                // assert!(timer.just_finished());
                timer
            }
            None => Timer::from_seconds(0.1, TimerMode::Once),
        };
        let delay = Timer::new(formation.delay, TimerMode::Once);

        info!(
            "spawning FormationSpawner[{i}] with delay {:?} and timer {:?}",
            delay, timer
        );
        commands.spawn(FormationSpawner {
            formation_group_index: i,
            initial_delay: delay,
            timer,
        });
    }

    // std::process::exit(1);
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
    // mut query: Query<&mut FormationSpawnerCountdown>,
    mut spawners: Query<&mut FormationSpawner>,
    mut evw_robot_formation_spawned: EventWriter<RobotFormationSpawned>,
    mut evw_pause_play: EventWriter<PausePlay>, // TODO why argument here?
    time: Res<Time>,
    config: Res<Config>,
) {
    for mut spawner in &mut spawners {
        spawner.tick(time.delta());
        if spawner.ready_to_spawn() {
            info!("ready to spawn!");
            evw_robot_formation_spawned.send(RobotFormationSpawned {
                formation_group_index: spawner.formation_group_index,
            });

            if config.simulation.pause_on_spawn {
                evw_pause_play.send(PausePlay::Pause);
            }
        }
    }
}

// TODO: use a trait for this

// const MAX_PLACEMENT_ATTEMPTS: usize = 1000;

// fn line_segment_formation(
//     start: (Vec2, Vec2),
//     waypoints: &[Waypoint],
//     world_dims: &WorldDimensions,
// ) -> Option<Vec<Vec4>> {
//     todo!()

//                 let start = world_dims.point_to_world_position(*start);
//                 let end = world_dims.point_to_world_position(*end);

//                 // let start = point_to_world_position(start, &world_dims);
//                 // let end = point_to_world_position(end, &world_dims);

//                 randomly_place_nonoverlapping_circles_along_line_segment(
//                     start,
//                     end,
//                     formation.robots,
//                     config.robot.radius,
//                     max_placement_attempts,
//                     &mut rng,
//                 )

// }

// fn circle_formation(radius: f32, center: Vec2, world_dims: &WorldDimensions)
// -> Option<Vec<Vec4>> {     todo!()
// }

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
    obstacles: Res<Obstacles>,
    // obstacle_sdf: Res<ObstacleSdf>,
    image_assets: ResMut<Assets<Image>>,
) {
    for event in evr_robot_formation_spawned.read() {
        // only continue if the image has been loaded
        let Some(image) = image_assets.get(&obstacles.sdf) else {
            error!("obstacle sdf not loaded yet");
            return;
        };

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
                image,
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

    // let initial_positions_for_each_robot =
    // positions_for_each_robot.first().expect("each robot has an initial
    // position");

    // let waypoints: Vec<_> = positions_for_each_robot
    //     .iter()
    //     .chain(std::iter::once(positions_for_each_robot.last().expect("there
    // is at least one position")))     .map(|wp_robot| {
    //         wp_robot.iter()

    //         .tuple_windows()
    //         .map(|(from, to)| {

    //             let direction = *to - *from;
    //             let velocity = direction.normalize_or_zero() *
    // config.robot.max_speed.get();             Vec4::new(from.x, from.y,
    // velocity.x, velocity.y)         })
    //         .collect::<VecDeque<_>>();
    //     }).collect();

    // let waypoints = match formation.initial_position.shape {
    //     Shape::LineSegment((start, end)) => {
    //         // let start = world_dims.point_to_world_position(*start);
    //         // let end = world_dims.point_to_world_position(*end);
    //         line_segment_formation(
    //             &formation.initial_position,

    //             (start.into(), end.into()),
    //             formation.waypoints.as_slice(),
    //             &world_dims,
    //         )
    //     }
    //     Shape::Circle { radius, center } => {
    //         circle_formation(radius.into(), center.into(), &world_dims)
    //     }
    //     Shape::Polygon(_) => unimplemented!(),
    // };

    // let Some(waypoints) = waypoints else {
    //     error!(
    //         "failed to spawn formation {}, reason: was not able to place
    // robots along shape \          after {MAX_PLACEMENT_ATTEMPTS}
    // attempts, skipping",         event.formation_group_index,
    //     );
    //     return;
    // };

    // // TODO: supprt equal
    // let lerp_amounts = match &formation.initial_position.shape {
    //     Shape::LineSegment((start, end)) => {
    //         let start = world_dims.point_to_world_position(*start);
    //         let end = world_dims.point_to_world_position(*end);

    //         // let start = point_to_world_position(start, &world_dims);
    //         // let end = point_to_world_position(end, &world_dims);

    //         randomly_place_nonoverlapping_circles_along_line_segment(
    //             start,
    //             end,
    //             formation.robots,
    //             config.robot.radius,
    //             max_placement_attempts,
    //             &mut rng,
    //         )
    //     }
    //     Shape::Circle { radius, center } => {
    //         todo!()
    //     }
    //     _ => unimplemented!(),
    // };

    // let Some(lerp_amounts) = lerp_amounts else {
    //     error!(
    //         "failed to spawn formation {}, reason: was not able to place
    // robots along line \          segment after {} attempts, skipping",
    //         event.formation_group_index,
    //         max_placement_attempts.get()
    //     );
    //     return;
    // };

    // dbg!(&lerp_amounts);
    // debug_assert_eq!(lerp_amounts.len(), formation.robots.get());

    // // FIXME: order is flipped
    // let initial_position_of_each_robot = map_positions(
    //     &formation.initial_position.shape,
    //     &lerp_amounts,
    //     &world_dims,
    // );

    // dbg!(&initial_position_of_each_robot);

    // The first vector is the waypoints for the first robot, the second vector
    // is the waypoints for the second robot, etc.
    // let mut waypoints_of_each_robot: Vec<Vec<Vec2>> =
    //     Vec::with_capacity(formation.robots.get());

    // for i in 0..formation.robots.get() {
    //     let mut waypoints = Vec::with_capacity(formation.waypoints.len());
    //     // for wp in formation.waypoints.iter().skip(1) {
    //     for wp in formation.waypoints.iter() {
    //         let positions = map_positions(&wp.shape, &lerp_amounts,
    // &world_dims);         waypoints.push(positions[i]);
    //     }
    //     waypoints_of_each_robot.push(waypoints);
    // }

    // // dbg!(&waypoints_of_each_robot);

    // // [(a, b, c), (d, e, f), (g, h, i)]
    // //  -> [(a, d, g), (b, e, h), (c, f, i)]

    // let max_speed = config.robot.max_speed.get();

    //     for (initial_position, waypoints) in initial_position_of_each_robot
    //         .iter()
    //         .zip(waypoints_of_each_robot.iter())
    //     {
    //         let initial_translation = Vec3::new(initial_position.x, 0.5,
    // initial_position.y);         let mut entity = commands.spawn_empty();
    //         let robot_id = entity.id();
    //         evw_waypoint_created.send_batch(waypoints.iter().map(|p|
    // WaypointCreated {             for_robot: robot_id,
    //             position:  *p,
    //         }));

    //         // dbg!(&waypoints);

    //         // let last_waypoint = waypoints.last();
    //         // let velocity_at_waypoint = std::iter::once(initial_position)
    //         let waypoints = std::iter::once(initial_position)
    //             .chain(waypoints.iter())
    //             .chain(std::iter::once(
    //                 waypoints.last().expect("there is at least one
    // waypoint"),             ))
    //             .tuple_windows()
    //             .map(|(from, to)| {
    //                 let direction = *to - *from;
    //                 let velocity = direction.normalize_or_zero() * max_speed;
    //                 Vec4::new(from.x, from.y, velocity.x, velocity.y)
    //             })
    //             .collect::<VecDeque<_>>();

    //         // dbg!(&waypoints);

    //         let initial_position = initial_position.extend(0.0).xzy();
    //         let initial_direction = waypoints
    //             .get(1)
    //             .map_or_else(|| Vec3::ZERO, |p| Vec3::new(p.x, 0.0, p.y))
    //             - initial_position;

    //         let robotbundle = RobotBundle::new(
    //             robot_id,
    //             waypoints,
    //             variable_timesteps.as_slice(),
    //             &config,
    //             OBSTACLE_IMAGE
    //                 .get()
    //                 .expect("obstacle image should be allocated and
    // initialised"),         )
    //         .expect(
    //             "Possible `RobotInitError`s should be avoided due to the
    // formation input being \              validated.",
    //         );
    //         let initial_visibility = if config.visualisation.draw.robots {
    //             Visibility::Visible
    //         } else {
    //             Visibility::Hidden
    //         };

    //         // TODO: Make this depend on random seed
    //         // let random_color = theme.into_display_iter().choose(&mut
    //         // thread_rng()).expect(     "Choosing random colour from an
    //         // iterator that is hard-coded with values should be \      ok.",
    //         // );
    //         let random_color = DisplayColour::iter()
    //             .choose(&mut thread_rng())
    //             .expect("there is more than 0 colors");

    //         let material = materials.add(StandardMaterial {
    //             base_color:
    // Color::from_catppuccin_colour(theme.get_display_colour(&random_color)),
    //             ..Default::default()
    //         });

    //         let pbrbundle = PbrBundle {
    //             mesh: scene_assets.meshes.robot.clone(),
    //             material,
    //             transform: Transform::from_translation(initial_translation),
    //             visibility: initial_visibility,
    //             ..Default::default()
    //         };

    //         entity.insert((
    //             robotbundle,
    //             pbrbundle,
    //             simulation_loader::Reloadable,
    //             PickableBundle::default(),
    //             On::<Pointer<Click>>::send_event::<RobotClickedOn>(),
    //             ColorAssociation { name: random_color },
    //             FollowCameraMe::new(0.0, 15.0, 0.0)
    //
    // .with_up_direction(Direction3d::new(initial_direction).expect(
    //                     "Vector between initial position and first waypoint
    // should be different \                      from 0, NaN, and
    // infinity.",                 ))
    //                 .with_attached(true),
    //         ));

    //         evw_robot_spawned.send(RobotSpawned(robot_id));
    //     }
    // }
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

// fn select_robot_when_clicked(
//     mut robot_click_event: EventReader<RobotClickEvent>,
//     mut selected_robot: ResMut<SelectedRobot>,
// ) {
//     for event in robot_click_event.read() {
//         selected_robot.select(event.target());
//     }
// }

// fn lerp_amounts_along_line_segment(
//     start: Vec2,
//     end: Vec2,
//     radius: StrictlyPositiveFinite<f32>,
//     num_points: NonZeroUsize,
// ) -> Vec<f32> {
//     let num_points = num_points.get();
//     let mut lerp_amounts = Vec::with_capacity(num_points);
//     for i in 0..num_points {
//         lerp_amounts.push(i as f32 / (num_points - 1) as f32);
//     }
//     lerp_amounts
// }

// #[derive(Debug, Clone, Copy)]
// struct WorldDimensions {
//     width:  StrictlyPositiveFinite<f64>,
//     height: StrictlyPositiveFinite<f64>,
// }

// #[derive(Debug)]
// enum WordDimensionsError {
//     ZeroWidth,
//     ZeroHeight,
// }

// impl WorldDimensions {
//     fn new(width: f64, height: f64) -> Self {
//         Self {
//             width:  width.try_into().expect("width is not zero"),
//             height: height.try_into().expect("height is not zero"),
//         }
//     }

//     /// Get the width of the world.
//     pub const fn width(&self) -> f64 {
//         self.width.get()
//     }

//     /// Get the height of the world.
//     pub const fn height(&self) -> f64 {
//         self.height.get()
//     }

//     pub fn point_to_world_position(&self, p: Point) -> Vec2 {
//         #[allow(clippy::cast_possible_truncation)]
//         Vec2::new(
//             ((p.x - 0.5) * self.width()) as f32,
//             ((p.y - 0.5) * self.height()) as f32,
//         )
//     }
// }

// /// Convert a `RelativePoint` to a world position
// /// given the dimensions of the world.
// /// The `RelativePoint` is a point in the range [0, 1] x [0, 1]
// /// where (0, 0) is the bottom-left corner of the world
// /// and (1, 1) is the top-right corner of the world.
// /// ```
// fn relative_point_to_world_position(
//     relative_point: &RelativePoint,
//     world_dims: &WorldDimensions,
// ) -> Vec2 {
//     #[allow(clippy::cast_possible_truncation)]
//     Vec2::new(
//         ((relative_point.x.get() - 0.5) * world_dims.width()) as f32,
//         ((relative_point.y.get() - 0.5) * world_dims.height()) as f32,
//     )
// }

// /// Convert a `Point` to a world position
// /// given the dimensions of the world.
// /// The `Point` is not bound to the range [0, 1] x [0, 1]
// fn point_to_world_position(point: &Point, world_dims: &WorldDimensions) ->
// Vec2 {     #[allow(clippy::cast_possible_truncation)]
//     Vec2::new(
//         ((point.x - 0.5) * world_dims.width()) as f32,
//         ((point.y - 0.5) * world_dims.height()) as f32,
//     )
// }

// fn map_positions(shape: &Shape, lerp_amounts: &[f32], world_dims:
// &WorldDimensions) -> Vec<Vec2> {     match shape {
//         Shape::LineSegment((start, end)) => {
//             let start = point_to_world_position(start, world_dims);
//             let end = point_to_world_position(end, world_dims);
//             lerp_amounts
//                 .iter()
//                 .map(|&lerp_amount| start.lerp(end, lerp_amount))
//                 .collect()
//         }
//         _ => unimplemented!(),
//     }
// }

// #[derive(Debug, Clone, Copy)]
// struct LineSegment {
//     from: Vec2,
//     to:   Vec2,
// }
//
// impl LineSegment {
//     fn new(from: Vec2, to: Vec2) -> Self {
//         Self { from, to }
//     }
//
//     fn length(&self) -> f32 {
//         self.from.distance(self.to)
//     }
// }

// fn randomly_place_nonoverlapping_circles_along_line_segment(
//     from: Vec2,
//     to: Vec2,
//     num_circles: NonZeroUsize,
//     radius: StrictlyPositiveFinite<f32>,
//     max_attempts: NonZeroUsize,
//     rng: &mut impl Rng,
// ) -> Option<Vec<f32>> {
//     let num_circles = num_circles.get();
//     let max_attempts = max_attempts.get();
//     let mut lerp_amounts: Vec<f32> = Vec::with_capacity(num_circles);
//     let mut placed: Vec<Vec2> = Vec::with_capacity(num_circles);
//
//     let diameter = radius.get() * 2.0;
//
//     for _ in 0..max_attempts {
//         placed.clear();
//         lerp_amounts.clear();
//
//         for _ in 0..num_circles {
//             let lerp_amount = rng.gen_range(0.0..1.0);
//             let new_position = from.lerp(to, lerp_amount);
//
//             let valid = placed.iter().all(|&p| new_position.distance(p) >=
// diameter);
//
//             if valid {
//                 lerp_amounts.push(lerp_amount);
//                 placed.push(new_position);
//                 if placed.len() == num_circles {
//                     return Some(lerp_amounts);
//                 }
//             }
//         }
//     }
//
//     None
// }

// fn equal_non_overlapping_circles_along_path(
//     path: TwoOrMore<Vec2>,
//     circle_radius: StrictlyPositiveFinite<f32>,
//     num_circles: NonZeroUsize,
// ) -> Option<Vec<Vec2>> {

//     let num_circles = num_circles.get();
//     let circle_radius = circle_radius.get();
//     let diameter = 2.0 * circle_radius;

//     let mut placed: Vec<Vec2> = Vec::with_capacity(num_circles);

//     placed.push(*path.first());

//     for _ in 0..num_circles {

//             // let lerp_amount = rng.gen_range(0.0..1.0);
//             // let new_position = from.lerp(to, lerp_amount);

//             let last = placed.last().expect("at least 1 point has been
// placed");

//             let valid = placed.iter().all(|&p| new_position.distance(p) >=
// diameter);

//             if valid {
//                 // lerp_amounts.push(lerp_amount);
//                 placed.push(new_position);
//                 if placed.len() == num_circles {
//                     return Some(lerp_amounts);
//                 }
//             }
//     }

//     todo!()

// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_relative_point_to_world_position() {
//         let world_dims = WorldDimensions::new(100.0, 80.0);
//         let relative_point = RelativePoint::new(0.5, 0.5).expect("x and y are
// in the range [0, 1]");         let world_position =
// relative_point_to_world_position(&relative_point, &world_dims);
//         assert_eq!(world_position, Vec2::new(0.0, 0.0));

//         let bottom_left = RelativePoint::new(0.0, 0.0).expect("x and y are in
// the range [0, 1]");         let world_position =
// relative_point_to_world_position(&bottom_left, &world_dims);
//         assert_eq!(world_position, Vec2::new(-50.0, -40.0));

//         let top_right = RelativePoint::new(1.0, 1.0).expect("x and y are in
// the range [0, 1]");         let world_position =
// relative_point_to_world_position(&top_right, &world_dims);         assert_eq!
// (world_position, Vec2::new(50.0, 40.0));     }
// }
