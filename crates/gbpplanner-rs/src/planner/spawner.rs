#![warn(missing_docs)]
//! ...
use std::{collections::VecDeque, num::NonZeroUsize, sync::OnceLock};

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
    asset_loader::SceneAssets,
    config::{
        geometry::{Point, RelativePoint, Shape},
        Config, FormationGroup,
    },
    pause_play::PausePlay,
    planner::robot::RobotBundle,
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour},
};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FormationSpawnEvent>()
            // .init_resource::<SelectedRobot>()
            .add_event::<RobotClickEvent>()
            .add_event::<CreateWaypointEvent>()
            .add_event::<DeleteWaypointEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    advance_time.run_if(not(time_is_paused)),
                    spawn_formation,
                    // select_robot_when_clicked,
                    // place_unplanned_waypoint.run_if(robot_is_selected),
                ),
            );
    }
}

/// run criteria if time is not paused
fn time_is_paused(time: Res<Time<Virtual>>) -> bool {
    time.is_paused()
}

#[derive(Event)]
pub struct CreateWaypointEvent {
    pub for_robot: RobotId,
    pub position:  Vec2,
}

#[derive(Event)]
pub struct DeleteWaypointEvent(pub Entity);

// #[derive(Resource, Default)]
// struct SelectedRobot(pub Option<Entity>);

// impl SelectedRobot {
//     #[inline]
//     pub fn deselect(&mut self) {
//         self.0 = None
//     }

//     #[inline]
//     pub fn select(&mut self, entity: Entity) {
//         self.0 = Some(entity)
//     }

//     #[inline]
//     pub fn is_selected(&self) -> bool {
//         self.0.is_some()
//     }
// }

// fn robot_is_selected(selected_robot: Res<SelectedRobot>) -> bool {
//     selected_robot.is_selected()
// }

// fn place_unplanned_waypoint(
//     mut commands: Commands,
//     mut mousebutton_event: EventReader<MouseButtonInput>,
//     mut create_waypoint_event: EventWriter<CreateWaypointEvent>,
//     mut selected_robot: ResMut<SelectedRobot>,
//     mut query: Query<(Entity, &mut Waypoints), With<RobotState>>,
//     cursor_position: ResMut<CursorCoordinates>,
//     // q_windows: Query<&Window, With<PrimaryWindow>>,
// ) {
//     for MouseButtonInput { button, state, .. } in mousebutton_event.read() {
//         let (ButtonState::Pressed, MouseButton::Left) = (state, button) else
// {             continue;
//         };

//         error!("pressed left click");
//         // prepend a waypoint at the mouse position
//         let Some(mut waypoints) = query
//             .iter_mut()
//             .find(|(entity, _)| entity == &selected_robot.0.unwrap())
//             .map(|(_, waypoints)| waypoints)
//         else {
//             continue;
//         };

//         // let Some(cursor_position) = q_windows.single().cursor_position()
// else {         //     continue;
//         // };

//         create_waypoint_event.send(CreateWaypointEvent {
//             for_robot: selected_robot.0.unwrap(),
//             position: cursor_position.local(),
//         });

//         // waypoints
//         //     .0
//         //     .push_front(Vec4::new(cursor_position.x, cursor_position.y,
// 0.0, 0.0));         //
//         error!("placed waypoint");
//         selected_robot.deselect();
//     }
// }

/// Every [`ObstacleFactor`] has a static reference to the obstacle image.
static OBSTACLE_IMAGE: OnceLock<Image> = OnceLock::new();
// static OBSTACLE_IMAGE: OnceLock<Image> =
//     OnceLock::new().get_or_init(||
// include_bytes!("./assets/imgs/junction_sdf.png"));

/// Component attached to an entity that spawns formations.
#[derive(Component)]
pub struct FormationSpawnerCountdown {
    pub timer: Timer,
    pub formation_group_index: usize,
}

/// Create an entity with a a `FormationSpawnerCountdown` component for each
/// formation group.
fn setup(mut commands: Commands, formation_group: Res<FormationGroup>) {
    for (i, formation) in formation_group.formations.iter().enumerate() {
        let mode = if formation.repeat {
            TimerMode::Repeating
        } else {
            TimerMode::Once
        };
        let duration = formation.delay.as_secs_f32();
        let timer = Timer::from_seconds(duration, mode);

        let mut entity = commands.spawn_empty();
        entity.insert(FormationSpawnerCountdown {
            timer,
            formation_group_index: i,
        });

        info!(
            "spawned formation-group spawner: {} with mode {:?} spawning every {} seconds",
            i + 1,
            mode,
            duration
        );
    }
}

/// Event that is sent when a formation should be spawned.
/// The `formation_group_index` is the index of the formation group in the
/// `FormationGroup` resource. Telling the event reader which formation group to
/// spawn.
/// Assumes that the `FormationGroup` resource has been initialised, and does
/// not change during the program's execution.
#[derive(Debug, Event)]
pub struct FormationSpawnEvent {
    pub formation_group_index: usize,
}

/// Advance time for each `FormationSpawnerCountdown` entity with
/// `Time::delta()`. If the timer has just finished, send a
/// `FormationSpawnEvent`.
fn advance_time(
    time: Res<Time>,
    mut query: Query<&mut FormationSpawnerCountdown>,
    mut spawn_event_writer: EventWriter<FormationSpawnEvent>,
    mut pause_play_event: EventWriter<PausePlay>,
    config: Res<Config>,
) {
    for mut countdown in query.iter_mut() {
        countdown.timer.tick(time.delta());
        if countdown.timer.just_finished() {
            spawn_event_writer.send(FormationSpawnEvent {
                formation_group_index: countdown.formation_group_index,
            });
            // info!(
            //     "sending formation spawn event for group: {}",
            //     countdown.formation_group_index
            // );
            if config.simulation.pause_on_spawn {
                pause_play_event.send(PausePlay::Pause);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_formation(
    mut commands: Commands,
    mut spawn_event_reader: EventReader<FormationSpawnEvent>,
    mut spawn_robot_event: EventWriter<RobotSpawned>,
    mut create_waypoint_event: EventWriter<CreateWaypointEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
    scene_assets: Res<SceneAssets>,
    image_assets: ResMut<Assets<Image>>,
    formation_group: Res<FormationGroup>,
    variable_timesteps: Res<VariableTimesteps>,
    theme: Res<CatppuccinTheme>,
) {
    // only continue if the image has been loaded
    let Some(image) = image_assets.get(&scene_assets.obstacle_image_sdf) else {
        error!("obstacle sdf not loaded yet");
        return;
    };

    let _ = OBSTACLE_IMAGE.get_or_init(|| image.clone());

    for event in spawn_event_reader.read() {
        // warn!("new formation spawn event received: {:?}", event);
        let formation = &formation_group.formations[event.formation_group_index];
        let first_wp = formation.waypoints.first();

        let world_dims = WorldDimensions::new(
            config.simulation.world_size.get().into(),
            config.simulation.world_size.get().into(),
        );

        let mut rng = rand::thread_rng();

        let max_placement_attempts = NonZeroUsize::new(1000).expect("1000 is not zero");
        let Some(lerp_amounts) = (match &first_wp.shape {
            Shape::Line((start, end)) => {
                let start = point_to_world_position(start, &world_dims);
                let end = point_to_world_position(end, &world_dims);
                randomly_place_nonoverlapping_circles_along_line_segment(
                    start,
                    end,
                    formation.robots,
                    config.robot.radius,
                    max_placement_attempts,
                    &mut rng,
                )
            }
            _ => unimplemented!(),
        }) else {
            error!(
                "failed to spawn formation {}, reason: was not able to place robots along line \
                 segment after {} attempts, skipping",
                event.formation_group_index,
                max_placement_attempts.get()
            );
            return;
        };

        debug_assert_eq!(lerp_amounts.len(), formation.robots.get());

        // FIXME: order is flipped
        let initial_position_of_each_robot =
            map_positions(&first_wp.shape, &lerp_amounts, &world_dims);

        // The first vector is the waypoints for the first robot, the second vector is
        // the waypoints for the second robot, etc.
        let mut waypoints_of_each_robot: Vec<Vec<Vec2>> =
            Vec::with_capacity(formation.robots.get());

        for i in 0..formation.robots.get() {
            let mut waypoints = Vec::with_capacity(formation.waypoints.len());
            // for wp in formation.waypoints.iter().skip(1) {
            for wp in formation.waypoints.iter() {
                let positions = map_positions(&wp.shape, &lerp_amounts, &world_dims);
                waypoints.push(positions[i]);
            }
            waypoints_of_each_robot.push(waypoints);
        }

        // [(a, b, c), (d, e, f), (g, h, i)]
        //  -> [(a, d, g), (b, e, h), (c, f, i)]

        let max_speed = config.robot.max_speed.get();

        for (initial_position, waypoints) in initial_position_of_each_robot
            .iter()
            .zip(waypoints_of_each_robot.iter())
        {
            let initial_translation = Vec3::new(initial_position.x, 0.5, initial_position.y);
            let mut entity = commands.spawn_empty();
            let robot_id = entity.id();
            create_waypoint_event.send_batch(waypoints.iter().map(|p| CreateWaypointEvent {
                for_robot: robot_id,
                position:  *p,
            }));

            let mut waypoints_with_speed = waypoints
                .iter()
                .tuple_windows()
                .map(|(a, b)| {
                    let direction = *b - *a;
                    let velocity = direction.normalize_or_zero() * max_speed;
                    Vec4::new(a.x, a.y, velocity.x, velocity.y)
                })
                .collect::<VecDeque<_>>();
            waypoints_with_speed.push_back(Vec4::new(
                waypoints.last().unwrap().x,
                waypoints.last().unwrap().y,
                waypoints_with_speed.back().unwrap().x,
                waypoints_with_speed.back().unwrap().y,
            ));
            // println!("{:?}", waypoints_with_speed);
            let robotbundle = RobotBundle::new(
                robot_id,
                waypoints_with_speed,
                // waypoints
                //     .iter()
                //     .map(|p| Vec4::new(p.x, p.y, max_speed, 0.0))
                //     .collect::<VecDeque<_>>(),
                variable_timesteps.as_slice(),
                &config,
                OBSTACLE_IMAGE
                    .get()
                    .expect("obstacle image should be allocated and initialised"),
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
            let random_color = DisplayColour::iter().choose(&mut thread_rng()).unwrap();

            let material = materials.add(StandardMaterial {
                base_color: Color::from_catppuccin_colour(theme.get_display_colour(&random_color)),
                ..Default::default()
            });

            let pbrbundle = PbrBundle {
                mesh: scene_assets.meshes.robot.clone(),
                material,
                transform: Transform::from_translation(initial_translation),
                visibility: initial_visibility,
                ..Default::default()
            };

            let initial_direction = waypoints
                // .iter()
                // .nth(1)
                .get(1)
                .map(|p| Vec3::new(p.x, 0.0, p.y))
                .unwrap_or_else(|| Vec3::ZERO)
                - initial_position.extend(0.0).xzy();

            entity.insert((
                robotbundle,
                pbrbundle,
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<RobotClickEvent>(),
                ColorAssociation { name: random_color },
                crate::environment::FollowCameraMe::new(0.0, 15.0, 0.0)
                    .with_up_direction(Direction3d::new(initial_direction).expect(
                        "Vector between initial position and first waypoint should be different \
                         from 0, NaN, and infinity.",
                    ))
                    .with_attached(true),
            ));

            spawn_robot_event.send(RobotSpawned(robot_id));
        }
        // info!("spawning formation group {}", event.formation_group_index);
    }
}

#[derive(Event)]
struct RobotClickEvent(pub Entity);

impl RobotClickEvent {
    #[inline]
    pub fn target(&self) -> Entity {
        self.0
    }
}

impl From<ListenerInput<Pointer<Click>>> for RobotClickEvent {
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

#[derive(Debug, Clone, Copy)]
struct WorldDimensions {
    width:  StrictlyPositiveFinite<f64>,
    height: StrictlyPositiveFinite<f64>,
}

// #[derive(Debug)]
// enum WordDimensionsError {
//     ZeroWidth,
//     ZeroHeight,
// }

impl WorldDimensions {
    fn new(width: f64, height: f64) -> Self {
        Self {
            width:  width.try_into().expect("width is not zero"),
            height: height.try_into().expect("height is not zero"),
        }
    }

    /// Get the width of the world.
    pub fn width(&self) -> f64 {
        self.width.get()
    }

    /// Get the height of the world.
    pub fn height(&self) -> f64 {
        self.height.get()
    }
}

/// Convert a `RelativePoint` to a world position
/// given the dimensions of the world.
/// The `RelativePoint` is a point in the range [0, 1] x [0, 1]
/// where (0, 0) is the bottom-left corner of the world
/// and (1, 1) is the top-right corner of the world.
/// ```
fn relative_point_to_world_position(
    relative_point: &RelativePoint,
    world_dims: &WorldDimensions,
) -> Vec2 {
    Vec2::new(
        ((relative_point.x.get() - 0.5) * world_dims.width()) as f32,
        ((relative_point.y.get() - 0.5) * world_dims.height()) as f32,
    )
}

/// Convert a `Point` to a world position
/// given the dimensions of the world.
/// The `Point` is not bound to the range [0, 1] x [0, 1]
fn point_to_world_position(point: &Point, world_dims: &WorldDimensions) -> Vec2 {
    Vec2::new(
        ((point.x - 0.5) * world_dims.width()) as f32,
        ((point.y - 0.5) * world_dims.height()) as f32,
    )
}

fn map_positions(shape: &Shape, lerp_amounts: &[f32], world_dims: &WorldDimensions) -> Vec<Vec2> {
    match shape {
        Shape::Line((start, end)) => {
            let start = point_to_world_position(start, world_dims);
            let end = point_to_world_position(end, world_dims);
            lerp_amounts
                .iter()
                .map(|&lerp_amount| start.lerp(end, lerp_amount))
                .collect()
        }
        _ => unimplemented!(),
    }
}

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

fn randomly_place_nonoverlapping_circles_along_line_segment(
    from: Vec2,
    to: Vec2,
    num_circles: NonZeroUsize,
    radius: StrictlyPositiveFinite<f32>,
    max_attempts: NonZeroUsize,
    rng: &mut impl Rng,
) -> Option<Vec<f32>> {
    let num_circles = num_circles.get();
    let max_attempts = max_attempts.get();
    let mut lerp_amounts: Vec<f32> = Vec::with_capacity(num_circles);
    let mut placed: Vec<Vec2> = Vec::with_capacity(num_circles);

    let diameter = radius.get() * 2.0;

    for _ in 0..max_attempts {
        placed.clear();
        lerp_amounts.clear();

        for _ in 0..num_circles {
            let lerp_amount = rng.gen_range(0.0..1.0);
            let new_position = from.lerp(to, lerp_amount);

            let valid = placed.iter().all(|&p| new_position.distance(p) >= diameter);

            if valid {
                lerp_amounts.push(lerp_amount);
                placed.push(new_position);
                if placed.len() == num_circles {
                    return Some(lerp_amounts);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_relative_point_to_world_position() {
        let world_dims = WorldDimensions::new(100.0, 80.0);
        let relative_point = RelativePoint::new(0.5, 0.5).expect("x and y are in the range [0, 1]");
        let world_position = relative_point_to_world_position(&relative_point, &world_dims);
        assert_eq!(world_position, Vec2::new(0.0, 0.0));

        let bottom_left = RelativePoint::new(0.0, 0.0).expect("x and y are in the range [0, 1]");
        let world_position = relative_point_to_world_position(&bottom_left, &world_dims);
        assert_eq!(world_position, Vec2::new(-50.0, -40.0));

        let top_right = RelativePoint::new(1.0, 1.0).expect("x and y are in the range [0, 1]");
        let world_position = relative_point_to_world_position(&top_right, &world_dims);
        assert_eq!(world_position, Vec2::new(50.0, 40.0));
    }
}
