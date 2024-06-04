//! example crate to take the environment config from the config file
//! and generate the necessary environment and colliders for rrt to work

use bevy::{prelude::*, tasks::futures_lite::future};
use bevy_infinite_grid::InfiniteGridSettings;
use bevy_notify::NotifyPlugin;
use bevy_prng::WyRand;
use bevy_rand::{component::EntropyComponent, resource::GlobalEntropy, traits::ForkableRng};
use gbp_config::Config;
use gbp_global_planner::{
    rrtstar::spawn_pathfinding_task, Colliders, Path, PathFinder, PathfindingTask,
};
use magics::{
    asset_loader::AssetLoaderPlugin,
    environment::EnvironmentPlugin,
    input::{camera::CameraInputPlugin, general::GeneralInputPlugin, ChangingBinding},
    simulation_loader::{InitialSimulation, SimulationLoaderPlugin},
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt, ThemePlugin},
};

const START: Vec2 = Vec2::new(-100.0 + 12.5, 62.5 - 12.5);
const END: Vec2 = Vec2::new(100.0 - 12.5, -62.5 + 12.5);
// const START: Vec2 = Vec2::new(0.0, 75.0 - 5.0);
// const END: Vec2 = Vec2::new(0.0, -75.0 + 5.0);

fn main() -> anyhow::Result<()> {
    better_panic::debug_install();
    let mut app = App::new();
    app.init_resource::<ChangingBinding>()
        .init_resource::<Path>()
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
        .add_systems(Startup, (spawn_waypoints, spawn_task_entity))
        .add_systems(PostStartup, change_infinite_grid_settings)
        .add_systems(
            Update,
            (
                rrt_path.run_if(resource_exists::<Colliders>),
                check_pathfinding_task,
                draw_path,
            ),
        )
        .run();

    Ok(())
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

fn spawn_task_entity(mut commands: Commands, mut prng: ResMut<GlobalEntropy<bevy_prng::WyRand>>) {
    commands.spawn((PathFinder, prng.fork_rng()));
}

/// **Bevy** [`Update`] system to find an RRT path through the environment
fn rrt_path(
    mut commands: Commands,
    mut pathfinder: Query<
        (Entity, &mut EntropyComponent<WyRand>),
        (With<PathFinder>, Without<PathfindingTask>),
    >,
    colliders: Res<Colliders>,
    config: Res<Config>,
    time: Res<Time>,
) {
    // info!("Running RRT pathfinding");
    // Do all this but with the async task spawner
    for (pathfinder, mut prng) in &mut pathfinder {
        spawn_pathfinding_task(
            &mut commands,
            START,
            END,
            // config.rrt.smoothing.enabled,
            config.rrt.clone(),
            colliders.clone(),
            pathfinder,
            Some(Box::new(prng.clone())),
        );

        // re-seed the PRNG
        let new_seed = time.elapsed_seconds_f64().to_le_bytes();
        prng.reseed(new_seed);
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
        // info!("Checking pathfinding task for entity: {:?}", entity);
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            info!("Pathfinding task completed for entity: {:?}", entity);
            commands.entity(entity).remove::<PathfindingTask>();
            match result {
                Ok(new_path) => {
                    // path.clear();
                    *path = new_path;
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
fn draw_path(mut gizmos: Gizmos, theme: Res<CatppuccinTheme>, path: Res<Path>) {
    path.0.windows(2).for_each(|window| {
        gizmos.line(
            window[0].extend(0.0).xzy(),
            window[1].extend(0.0).xzy(),
            Color::from_catppuccin_colour(theme.teal()),
        );
    });
}

/// **Bevy** marker [`Component`] for marking waypoints
/// Used to despawn previous waypoints when a new path is found
#[derive(Component, Debug)]
pub struct WaypointMarker;

fn change_infinite_grid_settings(
    mut query: Query<&mut InfiniteGridSettings>,
    theme: Res<CatppuccinTheme>,
) {
    let mut infinite_grid_settings = query.get_single_mut().unwrap();

    let colour = theme.grid_colour();

    infinite_grid_settings.x_axis_color = colour;
    infinite_grid_settings.z_axis_color = colour;
}
