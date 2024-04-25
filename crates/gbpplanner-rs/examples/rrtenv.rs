//! example crate to take the environment config from the config file
//! and generate the necessary environment and colliders for rrt to work
use std::sync::Arc;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_infinite_grid::InfiniteGridSettings;
use gbpplanner_rs::{
    asset_loader::{AssetLoaderPlugin, SceneAssets},
    cli,
    config::{read_config, Config, Environment, FormationGroup, RrtSection},
    environment::{map_generator::Colliders, EnvironmentPlugin},
    input::{camera::CameraInputPlugin, general::GeneralInputPlugin, ChangingBinding},
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt, ThemePlugin},
};
use parry2d::{
    na::{self, Isometry2, Vector2},
    query::intersection_test,
    shape,
};
// use parry3d::shape;
use rand::distributions::{Distribution, Uniform};

// const START: Vec2 = Vec2::new(-100.0 + 12.5, 62.5 - 2.0);
// const END: Vec2 = Vec2::new(100.0, -62.5 + 12.5);
const START: Vec2 = Vec2::new(0.0, 75.0 - 5.0);
const END: Vec2 = Vec2::new(0.0, -75.0 + 5.0);

fn main() -> anyhow::Result<()> {
    better_panic::debug_install();

    let cli = cli::parse_arguments();

    let (config, formation, environment): (Config, FormationGroup, Environment) = if cli.default {
        (
            Config::default(),
            FormationGroup::default(),
            Environment::default(),
        )
    } else {
        let config = read_config(cli.config.as_ref())?;
        if let Some(ref inner) = cli.config {
            println!(
                "successfully read config from: {}",
                inner.as_os_str().to_string_lossy()
            );
        }

        let formation = FormationGroup::from_file(&config.formation_group)?;
        println!(
            "successfully read formation config from: {}",
            config.formation_group
        );
        let environment = Environment::from_file(&config.environment)?;
        println!(
            "successfully read environment config from: {}",
            config.environment
        );

        (config, formation, environment)
    };

    let mut app = App::new();
    app.insert_resource(config)
        .insert_resource(formation)
        .insert_resource(environment)
        .init_resource::<ChangingBinding>()
        .init_resource::<Path>()
        .add_event::<PathFoundEvent>()
        .add_event::<TriggerRrtEvent>()
        .init_state::<PathFindingState>()
        .add_plugins((
            DefaultPlugins,
            AssetLoaderPlugin,
            CameraInputPlugin,
            GeneralInputPlugin,
            EnvironmentPlugin,
            ThemePlugin,
        ))
        .add_systems(Startup, (spawn_waypoints, init_path_info_text))
        .add_systems(PostStartup, change_infinite_grid_settings)
        .add_systems(
            Update,
            (
                trigger_rrt_event,
                rrt_path,
                update_path_lenght_text.run_if(on_event::<PathFoundEvent>()),
                update_waypoint_amount_text.run_if(on_event::<PathFoundEvent>()),
                draw_gizmos,
                draw_waypoints.run_if(on_event::<PathFoundEvent>()),
            ),
        )
        .run();

    Ok(())
}

/// **Bevy** [`Update`] system to trigger the RRT pathfinding
/// - Triggers on P
fn trigger_rrt_event(
    keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<TriggerRrtEvent>,
    mut next_state_path_found: ResMut<NextState<PathFindingState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        next_state_path_found.set(PathFindingState::NotFound);
        event_writer.send(TriggerRrtEvent);
    }
}

/// **Bevy** [`Event`] to trigger RRT pathfinding
/// - Used to signal when to start the path-finding
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerRrtEvent;

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

/// **Bevy** [`Resource`] for storing a path
/// Simply a wrapper for a list of [`Vec3`] points
#[derive(Debug, Resource, Default)]
pub struct Path(Vec<Vec3>);

impl Path {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn push(&mut self, point: Vec3) {
        self.0.push(point);
    }

    fn euclidean_length(&self) -> f32 {
        let mut length = 0.0;
        for i in 0..self.len() - 1 {
            length += (self[i] - self[i + 1]).length();
        }
        length
    }
}

// make Path indexable like a Vec
impl std::ops::Index<usize> for Path {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
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
    let sphere = meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap());

    commands.spawn(PbrBundle {
        mesh: sphere.clone(),
        material: materials.add(theme.blue_material()),
        transform: Transform::from_translation(Vec3::new(START.x, 0.5, START.y)),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: sphere,
        material: materials.add(theme.green_material()),
        transform: Transform::from_translation(Vec3::new(END.x, 0.5, END.y)),
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

/// Standalone function to spawn an RRT pathfinding task in an async thread
/// - Used to run path-finding tasks that may take longer than a single frame to
///   complete
fn spawn_rrt_path_finding_task(
    commands: &mut Commands,
    start: Vec2,
    end: Vec2,
    colliders: &'static Colliders,
    rrt_params: RrtSection,
    target: Entity, // task_pool: Res<AsyncComputeTaskPool>,
) {
    let collision_solver = CollisionProblem::new(Arc::new(colliders))
        .with_collision_radius(rrt_params.collision_radius.get());

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let mut path = rrt::dual_rrt_connect(
            &[start.x as f64, start.y as f64],
            &[end.x as f64, end.y as f64],
            |x: &[f64]| collision_solver.is_feasible(x),
            || collision_solver.random_sample(),
            rrt_params.step_size.get() as f64,
            rrt_params.max_iterations.get(),
        );

        if let Ok(mut res) = path {
            // optimise and smooth the found path
            rrt::smooth_path(
                &mut res,
                |x: &[f64]| collision_solver.is_feasible(x),
                rrt_params.smoothing.step_size.get() as f64,
                rrt_params.smoothing.max_iterations.get(),
            );

            // convert the path to Vec3 and update the path resource
            let mut path = Path::default();
            res.iter().for_each(|x| {
                path.push(Vec3::new(x[0] as f32, 0.0, x[1] as f32));
            });

            Ok(path)
        } else {
            Err(PathfindingError::ReachedMaxIterations)
        }
    });

    commands.entity(target).insert(PathfindingTask(task));
}

/// **Bevy** [`Update`] system to find an RRT path through the environment
fn rrt_path(
    mut path: ResMut<Path>,
    mut next_state_path_found: ResMut<NextState<PathFindingState>>,
    mut event_path_found_writer: EventWriter<PathFoundEvent>,
    state_path_found: Res<State<PathFindingState>>,
    colliders: Res<Colliders>,
    config: Res<Config>,
) {
    // let colliders = Arc::new(colliders.into_inner().to_owned());
    let colliders = colliders.into_inner();

    if colliders.is_empty() || matches!(state_path_found.get(), PathFindingState::Found) {
        return;
    }

    let collision_solver = CollisionProblem::new(Arc::new(colliders))
    // let collision_solver = CollisionProblem::new(colliders)
        .with_collision_radius(config.rrt.collision_radius.get());

    let start = [START.x as f64, START.y as f64];
    let end = [END.x as f64, END.y as f64];

    if let Ok(mut res) = rrt::dual_rrt_connect(
        &start,
        &end,
        |x: &[f64]| collision_solver.is_feasible(x),
        || collision_solver.random_sample(),
        config.rrt.step_size.get() as f64,
        config.rrt.max_iterations.get(),
    )
    .inspect_err(|e| {
        warn!("Failed to find path with error: {}", e);
    }) {
        // optimise and smooth the found path
        rrt::smooth_path(
            &mut res,
            |x: &[f64]| collision_solver.is_feasible(x),
            config.rrt.smoothing.step_size.get() as f64,
            config.rrt.smoothing.max_iterations.get(),
        );

        // convert the path to Vec3 and update the path resource
        path.clear();
        res.iter().for_each(|x| {
            path.push(Vec3::new(x[0] as f32, 0.0, x[1] as f32));
        });

        // update state and signal event
        next_state_path_found.set(PathFindingState::Found);
        event_path_found_writer.send(PathFoundEvent);
    };
}

/// **Bevy** [`Update`] system for drawing the path with gizmos
fn draw_gizmos(mut gizmos: Gizmos, path: Res<Path>, theme: Res<CatppuccinTheme>) {
    if path.len() < 2 {
        return;
    }
    for i in 0..path.len() - 1 {
        gizmos.line(
            path[i],
            path[i + 1],
            Color::from_catppuccin_colour(theme.teal()),
        );
    }
}

/// **Bevy** marker [`Component`] for marking waypoints
/// Used to despawn previous waypoints when a new path is found
#[derive(Component, Debug)]
pub struct WaypointMarker;

/// **Bevy** [`Update`] system for drawing waypoints along [`Path`]s
/// Only runs on [`PathFoundEvent`] events
fn draw_waypoints(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    previous_waypoints: Query<(Entity, &WaypointMarker)>,
    path: Res<Path>,
    theme: Res<CatppuccinTheme>,
) {
    for (entity, _) in previous_waypoints.iter() {
        commands.entity(entity).despawn();
    }

    for point in path.0.iter() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap()),
                material: materials.add(theme.yellow_material()),
                transform: Transform::from_translation(*point),
                ..Default::default()
            },
            WaypointMarker,
        ));
    }
}

struct CollisionProblem<'world> {
    colliders: Arc<&'world Colliders>,
    collision_checker: shape::Ball,
}

impl<'world> CollisionProblem<'world> {
    fn new(colliders: Arc<&'world Colliders>) -> Self {
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

        for (isometry, collider) in self.colliders.iter() {
            intersecting = intersection_test(
                &ball_pos,
                &self.collision_checker,
                &isometry,
                collider.as_ref(),
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

fn init_path_info_text(
    mut commands: Commands,
    theme: Res<CatppuccinTheme>,
    scene_assets: Res<SceneAssets>,
) {
    let text_style = TextStyle {
        font:      scene_assets.main_font.clone(),
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

fn update_path_lenght_text(mut query: Query<(&mut Text, &PathLengthText)>, path: Res<Path>) {
    for (mut text, _) in query.iter_mut() {
        text.sections[1].value = format!("{:.2}", path.euclidean_length());
    }
}

fn update_waypoint_amount_text(
    mut query: Query<(&mut Text, &WaypointAmountText)>,
    path: Res<Path>,
) {
    for (mut text, _) in query.iter_mut() {
        text.sections[1].value = path.len().to_string();
    }
}
