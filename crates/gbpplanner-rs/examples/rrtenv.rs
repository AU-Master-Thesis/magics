//! example crate to take the environment config from the config file
//! and generate the necessary environment and colliders for rrt to work
use std::sync::Arc;

use bevy::prelude::*;
use gbpplanner_rs::{
    asset_loader::AssetLoaderPlugin,
    cli,
    config::{read_config, Config, Environment, FormationGroup},
    environment::{map_generator::Colliders, EnvironmentPlugin},
    input::{camera::CameraInputPlugin, ChangingBinding},
    simulation_loader,
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
const END: Vec2 = Vec2::new(100.0 - 2.0, -62.5 + 12.5);

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
        .init_state::<PathFindingState>()
        .add_plugins((
            DefaultPlugins,
            AssetLoaderPlugin,
            CameraInputPlugin,
            EnvironmentPlugin,
            ThemePlugin,
        ))
        .add_systems(Startup, spawn_waypoints)
        .add_systems(Update, (rrt_path, draw_gizmos))
        .run();

    Ok(())
}

fn spawn_waypoints(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    theme: Res<CatppuccinTheme>,
) {
    // let start = Vec2::new(-100.0 + 12.5, 62.5 - 2.0);
    // let end = Vec2::new(100.0 - 2.0, -62.5 + 12.5);

    let sphere = meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap());

    commands.spawn(PbrBundle {
        mesh: sphere.clone(),
        material: materials.add(theme.blue_material()),
        transform: Transform::from_translation(Vec3::new(START.x, 0.0, START.y)),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: sphere,
        material: materials.add(theme.green_material()),
        transform: Transform::from_translation(Vec3::new(END.x, 0.0, END.y)),
        ..Default::default()
    });
}

/// **Bevy** [`Update`] system to find an RRT path through the environment
fn rrt_path(
    mut path: ResMut<Path>,
    mut next_state_path_found: ResMut<NextState<PathFindingState>>,
    state_path_found: Res<State<PathFindingState>>,
    colliders: Res<Colliders>,
) {
    let colliders = colliders.into_inner();
    if colliders.is_empty() || matches!(state_path_found.get(), PathFindingState::Found) {
        return;
    }

    let collision_solver = CollisionProblem::new(Arc::new(colliders));

    let start = [START.x as f64, START.y as f64];
    let end = [END.x as f64, END.y as f64];

    if let Ok(mut res) = rrt::dual_rrt_connect(
        &start,
        &end,
        |x: &[f64]| collision_solver.is_feasible(x),
        || collision_solver.random_sample(),
        1.0,
        10000,
    )
    .inspect_err(|e| {
        error!("Error: {:?}", e);
    }) {
        rrt::smooth_path(
            &mut res,
            |x: &[f64]| collision_solver.is_feasible(x),
            1.0,
            1000,
        );
        path.clear();
        res.iter().for_each(|x| {
            path.push(Vec3::new(x[0] as f32, 0.0, x[1] as f32));
        });
        next_state_path_found.set(PathFindingState::Found);
    };
}

/// **Bevy** [`Resource`] for storing a path
/// Simply a wrapper for a list of [`Vec3`] points
#[derive(Debug, Resource, Default)]
pub struct Path(Vec<Vec3>);

/// **Bevy** [`State`] for keeping track of the state of the path-finding
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PathFindingState {
    /// The path has not been found yet
    #[default]
    NotFound,
    /// The path has been found
    Found,
}

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
}

// make Path indexable like a Vec
impl std::ops::Index<usize> for Path {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
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

struct CollisionProblem<'world> {
    colliders: Arc<&'world Colliders>,
    ball:      shape::Ball,
}

impl<'world> CollisionProblem<'world> {
    fn new(colliders: Arc<&'world Colliders>) -> Self {
        let ball = shape::Ball::new(0.1f32);
        Self { colliders, ball }
    }

    fn with_collision_radius(mut self, radius: f32) -> Self {
        let ball = shape::Ball::new(radius);
        self.ball = ball;
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

        for (i, (isometry, collider)) in self.colliders.iter().enumerate() {
            intersecting = intersection_test(&ball_pos, &self.ball, &isometry, collider.as_ref())
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
