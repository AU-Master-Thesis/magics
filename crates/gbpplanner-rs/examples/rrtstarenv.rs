//! example crate to take the environment config from the config file
//! and generate the necessary environment and colliders for rrt to work
use std::sync::Arc;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
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

const START: Vec2 = Vec2::new(-100.0 + 12.5, 62.5 - 2.0);
const END: Vec2 = Vec2::new(100.0 - 12.5, -62.5 + 12.5);
// const START: Vec2 = Vec2::new(0.0, 75.0 - 5.0);
// const END: Vec2 = Vec2::new(0.0, -75.0 + 5.0);

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

        let formation = FormationGroup::from_ron_file(&config.formation_group)?;
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
                // init_path_info_text
            ),
        )
        .add_systems(PostStartup, change_infinite_grid_settings)
        .add_systems(
            Update,
            (
                trigger_rrt_event,
                rrt_path,
                // update_path_length_text.run_if(on_event::<PathFoundEvent>()),
                // update_waypoint_amount_text.run_if(on_event::<PathFoundEvent>()),
                draw_tree_branches,
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

    let collision_solver = CollisionProblem::new(Arc::new(&colliders));
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

/// **Bevy** [`Update`] system to find an RRT path through the environment
fn rrt_path(
    mut path: ResMut<Path>,
    mut tree: ResMut<RRTStarTree>,
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

    if let Ok(mut res) = rrt::rrtstar::rrtstar(
        &start,
        &end,
        |x: &[f64]| collision_solver.is_feasible(x),
        || collision_solver.random_sample(),
        config.rrt.step_size.get() as f64,
        config.rrt.max_iterations.get(),
        config.rrt.neighbourhood_radius.get() as f64,
        false,
    )
    .inspect_err(|e| {
        warn!("Failed to find path with error: {}", e);
    }) {
        // insert the result into the Tree resource
        tree.0 = res;

        // find closest point to end
        let closest_index = tree
            .0
            .vertices
            .iter()
            .map(|v| Vec2::new(v.data[0] as f32, v.data[1] as f32).distance_squared(END))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        // find the path from the start to the closest point to the end
        path.clear();
        tree.0.get_until_root(closest_index).iter().for_each(|v| {
            path.push(Vec3::new(v[0] as f32, 0.0, v[1] as f32));
        });

        // update state and signal event
        next_state_path_found.set(PathFindingState::Found);
        event_path_found_writer.send(PathFoundEvent);
    };
}

/// **Bevy** [`Update`] system for drawing the tree's branches with gizmo
/// polylines
fn draw_tree_branches(
    mut gizmos: Gizmos,
    tree: Res<RRTStarTree>,
    theme: Res<CatppuccinTheme>,
    path: Res<Path>,
) {
    let mut count = tree.0.vertices.len();

    let max_weight = tree
        .0
        .vertices
        .iter()
        .map(|v| v.weight)
        .fold(0.0, |acc: f32, x| acc.max(x));

    tree.0.vertices.iter().for_each(|v| {
        if let Some(parent_index) = v.parent_index {
            // gizmo line from v.data to tree.0.vertices[parent].data
            let parent_point = &tree.0.vertices[parent_index].data;

            let color = if path.contains(&Vec3::new(v.data[0] as f32, 0.0, v.data[1] as f32)) {
                Color::from_catppuccin_colour(theme.blue())
            } else {
                // gradient from red to blue based on vertex weight
                let t = v.weight / max_weight;
                let (r, g, b, a) = theme
                    .gradient(theme.red(), theme.green())
                    .at(t.into())
                    .to_linear_rgba();
                Color::rgba_linear(r as f32, g as f32, b as f32, a as f32)
            };

            gizmos.line(
                Vec3::new(v.data[0] as f32, 0.0, v.data[1] as f32),
                Vec3::new(parent_point[0] as f32, 0.0, parent_point[1] as f32),
                color,
            );
            count -= 1;
        }
    });
}

// /// **Bevy** [`Update`] system for drawing the resulting path with gizmo
// /// polylines Only runs on [`PathFoundEvent`] events
// fn draw_path(mut gizmos: Gizmos, path: Res<Path>, theme:
// Res<CatppuccinTheme>) {     for i in 0..path.len() - 1 {
//         gizmos.line(
//             path[i],
//             path[i + 1],
//             Color::from_catppuccin_colour(theme.green()),
//         );
//     }
// }

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
