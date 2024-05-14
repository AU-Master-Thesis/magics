//! example crate to take the environment config from the config file
//! and generate the necessary environment and colliders for rrt to work
use std::sync::Arc;

use bevy::{
    prelude::*,
    tasks::{futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_infinite_grid::InfiniteGridSettings;
use bevy_notify::NotifyPlugin;
use derive_more::Index;
use gbp_environment::Environment;
use gbpplanner_rs::{
    asset_loader::{AssetLoaderPlugin, Fonts},
    cli,
    config::{read_config, Config, FormationGroup, RRTSection},
    environment::{map_generator::Colliders, EnvironmentPlugin},
    input::{camera::CameraInputPlugin, general::GeneralInputPlugin, ChangingBinding},
    simulation_loader::{InitialSimulation, SimulationLoaderPlugin},
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt, ThemePlugin},
};
use parry2d::{
    na::{self, Isometry2, Vector2},
    query::intersection_test,
    shape,
};
// use parry3d::shape;
use rand::distributions::{Distribution, Uniform};

const START: Vec2 = Vec2::new(-100.0 + 12.5, 62.5 - 12.5);
const END: Vec2 = Vec2::new(100.0 - 12.5, -62.5 + 12.5);
// const START: Vec2 = Vec2::new(0.0, 75.0 - 5.0);
// const END: Vec2 = Vec2::new(0.0, -75.0 + 5.0);

fn main() -> anyhow::Result<()> {
    better_panic::debug_install();
    let mut app = App::new();
    app.init_resource::<ChangingBinding>()
        .init_resource::<Path>()
        .init_resource::<RRTStarTree>()
        .add_event::<PathFoundEvent>()
        .add_event::<TriggerRRTEvent>()
        .init_state::<PathFindingState>()
        .add_plugins((
            DefaultPlugins,
            ThemePlugin,
            SimulationLoaderPlugin {
                show_toasts: false,
                initial_simulation: InitialSimulation::Name("Complex".to_string()),
            },
            NotifyPlugin::default(),
            AssetLoaderPlugin,
            CameraInputPlugin,
            GeneralInputPlugin,
            EnvironmentPlugin,
        ))
        .add_systems(
            Startup,
            (
                spawn_waypoints,
                spawn_task_entity,
                // init_path_info_text
            ),
        )
        .add_systems(PostStartup, change_infinite_grid_settings)
        .add_systems(
            Update,
            (
                trigger_rrt_event,
                rrt_path,
                check_pathfinding_task,
                // update_path_length_text.run_if(on_event::<PathFoundEvent>()),
                // update_waypoint_amount_text.run_if(on_event::<PathFoundEvent>()),
                draw_path,
                draw_nodes.run_if(on_event::<PathFoundEvent>()),
            ),
        )
        .run();

    Ok(())
}

/// **Bevy** [`Update`] system to trigger the RRT pathfinding
/// - Triggers on P
fn trigger_rrt_event(
    mut path: ResMut<Path>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<TriggerRRTEvent>,
    mut next_state_path_found: ResMut<NextState<PathFindingState>>,
    colliders: Res<Colliders>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        next_state_path_found.set(PathFindingState::NotFound);
        event_writer.send(TriggerRRTEvent);
    }

    // let collision_solver = CollisionProblem::new(Arc::new(&colliders));
}

/// **Bevy** [`Event`] to trigger RRT pathfinding
/// - Used to signal when to start the path-finding
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerRRTEvent;

/// **Bevy** [`Event`] for signaling when a path is found
/// - Used to transition the state of the path-finding
/// - Used to signal when to draw the waypoints
#[derive(Debug, Clone, Copy, Event)]
pub struct PathFoundEvent;

/// **Bevy** [`State`] for keeping track of the state of the path-finding
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PathFindingState {
    /// The path has not been found yet
    #[default]
    NotFound,
    /// The path has been found
    Found,
}

macro_rules! delegate_to_inner {
    (& $method:ident -> $ret:ty) => {
        #[inline]
        fn $method(&self) -> $ret {
            self.0.$method()
        }
    };
    (& $method:ident) => {
        #[inline]
        fn $method(&self) {
            self.0.$method();
        }
    };

    (&mut $method:ident) => {
        #[inline]
        fn $method(&mut self) {
            self.0.$method();
        }
    };
    (&mut $method:ident -> $ret:ty) => {
        #[inline]
        fn $method(&mut self) -> $ret {
            self.0.$method();
        }
    };

    (&mut $method:ident, $($arg:ident : $t:ty),*) => {
        #[inline]
        fn $method(&mut self, $($arg: $t),*) {
            self.0.$method($($arg),*);
        }
    };
}

/// **Bevy** [`Resource`] for storing an RRT* Tree
/// Simply a wrapper for [`rrt::rrtstar::Tree`]
#[derive(Debug, Resource, Default)]
pub struct RRTStarTree(rrt::rrtstar::Tree<f64, f32>);

/// **Bevy** [`Resource`] for storing a path
/// Simply a wrapper for a list of [`Vec3`] points
#[derive(Debug, Resource, Default, Index)]
pub struct Path(Vec<Vec3>);

impl Path {
    delegate_to_inner!(& len -> usize);

    delegate_to_inner!(&mut clear);

    delegate_to_inner!(&mut push, point: Vec3);

    fn contains(&self, point: &Vec3) -> bool {
        self.0.contains(point)
    }

    fn euclidean_length(&self) -> f32 {
        let mut length = 0.0;
        for i in 0..self.len() - 1 {
            length += (self[i] - self[i + 1]).length();
        }
        length
    }
}

/// **Bevy** [`System`] to setup the environment
/// Spawns a sphere at the start and end points for the RRT algorithm
fn spawn_waypoints(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    theme: Res<CatppuccinTheme>,
) {
    let sphere = meshes.add(Sphere::new(2.0).mesh().ico(5).unwrap());

    commands.spawn(PbrBundle {
        mesh: sphere.clone(),
        material: materials.add(theme.blue_material()),
        transform: Transform::from_translation(Vec3::new(START.x, -2.0, START.y)),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: sphere,
        material: materials.add(theme.green_material()),
        transform: Transform::from_translation(Vec3::new(END.x, -2.0, END.y)),
        ..Default::default()
    });
}

/// Possible RRT pathfinding errors
#[derive(Debug)]
pub enum PathfindingError {
    ReachedMaxIterations,
}

/// **Bevy** [`Component`] for storing the pathfinding task
#[derive(Component, Debug)]
pub struct PathfindingTask(Task<Result<Path, PathfindingError>>);

/// **Bevy** [`Component`] for attaching a pathfinding task
#[derive(Component, Debug)]
struct PathFinder;

fn spawn_task_entity(mut commands: Commands) {
    commands.spawn(PathFinder);
}

/// Standalone function to spawn an async task for pathfinding
/// - Used to run path-finding tasks that may take longer than a single frame to
///   complete
fn spawn_pathfinding_task(
    commands: &mut Commands,
    start: Vec2,
    end: Vec2,
    smooth: bool,
    rrt_params: RRTSection,
    colliders: Colliders,
    task_target: Entity,
) {
    let collision_solver =
        CollisionProblem::new(colliders).with_collision_radius(rrt_params.collision_radius.get());

    let task_pool = AsyncComputeTaskPool::get();

    let task = task_pool.spawn(async move {
        let start = [start.x as f64, start.y as f64];
        let end = [end.x as f64, end.y as f64];

        rrt::rrtstar::rrtstar(
            &start,
            &end,
            |x: &[f64]| collision_solver.is_feasible(x),
            || collision_solver.random_sample(),
            rrt_params.step_size.get() as f64,
            rrt_params.max_iterations.get(),
            rrt_params.neighbourhood_radius.get() as f64,
            true,
        )
        .map(|res| {
            // let mut path = Path::default();
            // path.push(Vec3::new(end[0] as f32, 0.0, end[1] as f32));
            if let Some(goal_index) = res.goal_index {
                // res.get_until_root(goal_index).iter().for_each(|v| {
                //     path.push(Vec3::new(v[0] as f32, 0.0, v[1] as f32));
                // });
                let resulting_path = {
                    let mut resulting_path = std::iter::once(vec![end[0], end[1]])
                        .chain(res.get_until_root(goal_index).into_iter())
                        .collect::<Vec<_>>();
                    if smooth {
                        rrt::rrtstar::smooth_path(
                            &mut resulting_path,
                            |x| collision_solver.is_feasible(x),
                            rrt_params.step_size.get() as f64,
                            rrt_params.smoothing.max_iterations.get(),
                        );
                    }
                    resulting_path
                };

                // for v in resulting_path {
                //     path.push(Vec3::new(v[0] as f32, 0.0, v[1] as f32));
                // }
                Path(
                    resulting_path
                        .into_iter()
                        .map(|v| Vec3::new(v[0] as f32, 0.0, v[1] as f32))
                        .collect::<Vec<_>>(),
                )
            } else {
                Path(vec![])
            }
        })
        .map_err(|_| PathfindingError::ReachedMaxIterations)
    });

    commands.entity(task_target).insert(PathfindingTask(task));
}

/// **Bevy** [`Update`] system to find an RRT path through the environment
fn rrt_path(
    mut commands: Commands,
    pathfinder: Query<Entity, (With<PathFinder>, Without<PathfindingTask>)>,
    colliders: Res<Colliders>,
    config: Res<Config>,
) {
    // Do all this but with the async task spawner
    for pathfinder in pathfinder.iter() {
        spawn_pathfinding_task(
            &mut commands,
            START,
            END,
            config.rrt.smoothing.enabled,
            config.rrt.clone(),
            colliders.clone(),
            pathfinder,
        );
    }
}

/// **Bevy** [`Update`] system to check on the pahtfinding task attached to
/// [`PathFinder`] and update the pathfinding state
fn check_pathfinding_task(
    mut commands: Commands,
    mut path: ResMut<Path>,
    mut tasks: Query<(Entity, &mut PathfindingTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            info!("Pathfinding task completed for entity: {:?}", entity);
            commands.entity(entity).remove::<PathfindingTask>();
            match result {
                Ok(new_path) => {
                    path.clear();
                    path.0 = new_path.0;
                }
                Err(e) => {
                    error!("Pathfinding error: {:?}", e);
                }
            }
        }
    }
}

/// **Bevy** [`Update`] system for drawing the tree's branches with gizmo
/// polylines
fn draw_path(
    mut gizmos: Gizmos,
    // tree: Res<RRTStarTree>,
    theme: Res<CatppuccinTheme>,
    path: Res<Path>,
) {
    path.0.windows(2).for_each(|window| {
        gizmos.line(
            window[0],
            window[1],
            Color::from_catppuccin_colour(theme.teal()),
        );
    });
}

/// **Bevy** marker [`Component`] for marking waypoints
/// Used to despawn previous waypoints when a new path is found
#[derive(Component, Debug)]
pub struct WaypointMarker;

/// **Bevy** [`Update`] system for drawing waypoints along [`Path`]s
/// Only runs on [`PathFoundEvent`] events
fn draw_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    previous_waypoints: Query<Entity, With<WaypointMarker>>,
    // path: Res<Path>,
    tree: Res<RRTStarTree>,
    theme: Res<CatppuccinTheme>,
) {
    for entity in &previous_waypoints {
        commands.entity(entity).despawn();
    }

    // go through all nodes in tree.0 and draw them
    for node in tree.0.vertices.iter() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.1).mesh().ico(5).unwrap()),
                material: materials.add(theme.text_material()),
                transform: Transform::from_translation(Vec3::new(
                    node.data[0] as f32,
                    0.0,
                    node.data[1] as f32,
                )),
                ..Default::default()
            },
            WaypointMarker,
        ));
    }
}

struct CollisionProblem {
    colliders: Colliders,
    collision_checker: shape::Ball,
}

impl CollisionProblem {
    fn new(colliders: Colliders) -> Self {
        let ball = shape::Ball::new(0.1f32);
        Self {
            colliders,
            collision_checker: ball,
        }
    }

    fn with_collision_radius(mut self, radius: f32) -> Self {
        let ball = shape::Ball::new(radius);
        self.collision_checker = ball;
        self
    }

    fn is_feasible(&self, point: &[f64]) -> bool {
        // place the intersection ball at the point
        let ball_pos = Isometry2::new(Vector2::new(point[0] as f32, point[1] as f32), na::zero());

        let mut intersecting = false;

        // self.colliders.iter().for_each(|(isometry, collider)| {
        //     // info!("isometry: {:?}", isometry);
        //     // info!("collider: {:?}", *collider);
        //     intersecting = intersection_test(&ball_pos, &self.ball, &isometry,
        // collider.as_ref())         .expect("Correct shapes should have been
        // given."); });

        for collider in self.colliders.iter() {
            let isometry = collider.isometry;
            let shape = &collider.shape;
            intersecting = intersection_test(
                &ball_pos,
                &self.collision_checker,
                &isometry,
                shape.as_ref(),
            )
            .expect("Correct shapes should have been given.");
            if intersecting {
                // info!("intersecting with collider: {}", i);
                break;
            }
        }

        // std::process::exit(0);

        // return true if not intersecting
        !intersecting
    }

    fn random_sample(&self) -> Vec<f64> {
        let between = Uniform::new(-2000.0, 2000.0);
        let mut rng = rand::thread_rng();
        vec![between.sample(&mut rng), between.sample(&mut rng)]
    }
}

fn change_infinite_grid_settings(
    mut query: Query<&mut InfiniteGridSettings>,
    theme: Res<CatppuccinTheme>,
) {
    let mut infinite_grid_settings = query.get_single_mut().unwrap();

    let colour = theme.grid_colour();

    infinite_grid_settings.x_axis_color = colour;
    infinite_grid_settings.z_axis_color = colour;
}

/// **Bevy** [`Component`] for marking the [`Text`] entity for the path length
#[derive(Component, Debug)]
pub struct PathLengthText;

/// **Bevy** [`Component`] for marking the [`Text`] entity for the path length
#[derive(Component, Debug)]
pub struct WaypointAmountText;

fn init_path_info_text(mut commands: Commands, theme: Res<CatppuccinTheme>, fonts: Res<Fonts>) {
    let text_style = TextStyle {
        font:      fonts.main.clone(),
        font_size: 20.0,
        color:     Color::from_catppuccin_colour(theme.mauve()),
    };

    let keybind_text =
        TextBundle::from_sections([TextSection::new("Press P: Trigger RRT", text_style.clone())])
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(60.0),
                left: Val::Px(20.0),
                ..default()
            })
            .with_background_color(Color::from_catppuccin_colour_with_alpha(theme.base(), 0.75));

    let path_length_text = TextBundle::from_sections([
        TextSection::new("Path Length: ", text_style.clone()),
        TextSection::from_style(text_style.clone()),
    ])
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(40.0),
        left: Val::Px(20.0),
        ..default()
    })
    .with_background_color(Color::from_catppuccin_colour_with_alpha(theme.base(), 0.75));

    let waypoint_amount_text = TextBundle::from_sections([
        TextSection::new("Waypoint Amount: ", text_style.clone()),
        TextSection::from_style(text_style.clone()),
    ])
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(20.0),
        left: Val::Px(20.0),
        ..default()
    })
    .with_background_color(Color::from_catppuccin_colour_with_alpha(theme.base(), 0.75));

    commands.spawn(keybind_text);
    commands.spawn((path_length_text, PathLengthText));
    commands.spawn((waypoint_amount_text, WaypointAmountText));
}

fn update_path_length_text(mut query: Query<&mut Text, With<PathLengthText>>, path: Res<Path>) {
    info!("Updating path length: {:.2}", path.euclidean_length());
    query.single_mut().sections[1].value = format!("{:.2}", path.euclidean_length());
}

fn update_waypoint_amount_text(
    mut query: Query<&mut Text, With<WaypointAmountText>>,
    path: Res<Path>,
) {
    query.single_mut().sections[1].value = path.len().to_string();
}
