//! This is a simple example of the `rrt` crate within Bevy.
// mod super::theme;

use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;
use gbp_environment::Environment;
use gbpplanner_rs::{
    asset_loader::AssetLoaderPlugin,
    cli,
    config::{read_config, Config, FormationGroup},
    environment::{EnvironmentPlugin, MainCamera, ObstacleMarker},
    input::{camera::CameraInputPlugin, ChangingBinding},
    movement::{self, LinearMovementBundle, MovementPlugin, OrbitMovementBundle},
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt, ThemePlugin},
};
use parry3d::{
    na::{self, Isometry3, Vector3},
    query::{self, intersection_test},
    shape,
};
use rand::distributions::{Distribution, Uniform};

const INITIAL_CAMERA_DISTANCE: f32 = 5.0;
const CAMERA_UP: Vec3 = Vec3::NEG_Y;
const CAMERA_INITIAL_TARGET: Vec3 = Vec3::ZERO;
const CAMERA_INITIAL_POSITION: Vec3 = Vec3::new(0.0, INITIAL_CAMERA_DISTANCE, 0.0);

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
        // .insert_resource(formation)
        // .insert_resource(environment)
        .add_plugins((
            DefaultPlugins,
            // InfiniteGridPlugin,
            MovementPlugin,
            CameraInputPlugin,
            EnvironmentPlugin,
            ThemePlugin,
            AssetLoaderPlugin,
        ))
        // .init_state::<CameraMovementMode>()
        // .insert_resource(AmbientLight {
        //     color:      Color::default(),
        //     brightness: 1000.0,
        // })
        // .insert_resource(ClearColor(Color::from_catppuccin_colour(
        //     Flavour::Macchiato.base(),
        // )))
        // .init_resource::<CatppuccinTheme>()
        .init_resource::<Path>()
        .init_resource::<ChangingBinding>()
        // .add_event::<CameraResetEvent>()
        // .init_resource::<ActionBlock>()
        .add_systems(
            Startup,
            spawn_colliders,
        )
        .add_systems(Update, (rrt_path, draw_gizmos))
        .run();

    Ok(())
}

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
}

// make Path indexable like a Vec
impl std::ops::Index<usize> for Path {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

// pub struct Obstacles(Vec<dyn Primitive3d>);

fn spawn_colliders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // spawn a cuboid with dimensions 0.5 x 0.25 x 0.15
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 0.5, 0.25))),
            material: materials.add(StandardMaterial {
                base_color: Color::from_catppuccin_colour(Flavour::Macchiato.maroon()),
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        ObstacleMarker,
    ));

    // spawn a sphere with radius 0.05
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(
                Sphere::new(0.05)
                    .mesh()
                    .ico(4)
                    .expect("4 subdivisions is less than the maximum allowed of 80"),
            )),
            // material: materials.add(Color::from_catppuccin_colour(Flavour::Macchiato.maroon())),
            material: materials.add(StandardMaterial {
                base_color: Color::from_catppuccin_colour(Flavour::Macchiato.maroon()),
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        ObstacleMarker,
    ));
}

fn rrt_path(
    mut path: ResMut<Path>,
    mut index: Local<usize>,
    // mut query: Query<&Mesh, With<ObstacleMarker>>,
) {
    let start = [2.0f64, 2.0, 2.0];
    let goal = [-2.0f64, -2.0, -2.0];

    let p = CollisionProblem {
        obstacle: shape::Cuboid::new(Vector3::new(0.5f32, 0.5, 0.25)),
        // intersection sphere does not need a very big radius
        ball:     shape::Ball::new(0.1f32),
    };

    if *index == path.len() {
        let mut res = rrt::dual_rrt_connect(
            &start,
            &goal,
            |x: &[f64]| p.is_feasible(x),
            // |x: &[f64]| true,
            || p.random_sample(),
            0.05,
            1000,
        )
        .expect("RESULT YES");

        rrt::smooth_path(&mut res, |x: &[f64]| p.is_feasible(x), 0.05, 100);
        path.clear();
        res.iter().for_each(|x| {
            path.push(Vec3::new(x[0] as f32, x[1] as f32, x[2] as f32));
        });

        *index = 0;
    }

    *index += 1;
}

fn draw_gizmos(mut gizmos: Gizmos, path: Res<Path>) {
    if path.len() < 2 {
        return;
    }
    for i in 0..path.len() - 1 {
        gizmos.line(
            path[i],
            path[i + 1],
            Color::from_catppuccin_colour(Flavour::Macchiato.green()),
        );
    }
}

/// [`Startup`] system to spawn the main camera
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(CAMERA_INITIAL_POSITION)
                .looking_at(CAMERA_INITIAL_TARGET, CAMERA_UP),
            ..default()
        },
        LinearMovementBundle::default(),
        OrbitMovementBundle::default(),
        movement::Local,
        MainCamera,
    ));
}

/// **Bevy** [`Startup`] system
/// Spawns a directional light.
fn lighting(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 5.0 + Vec3::Z * 8.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
}

/// **Bevy** [`Startup`] system to spawn the an infinite grid
/// Using the [`InfiniteGridPlugin`] from the `bevy_infinite_grid` crate
fn infinite_grid(mut commands: Commands, catppuccin_theme: Res<CatppuccinTheme>) {
    let grid_colour = catppuccin_theme.grid_colour();

    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            shadow_color: None,
            major_line_color: grid_colour,
            minor_line_color: grid_colour,
            x_axis_color: grid_colour,
            z_axis_color: grid_colour,
            ..default()
        },
        ..default()
    });
}

struct CollisionProblem {
    obstacle: shape::Cuboid,
    ball:     shape::Ball,
}

impl CollisionProblem {
    fn is_feasible(&self, point: &[f64]) -> bool {
        // place the cuboid at the origin
        let cuboid_pos = Isometry3::new(Vector3::new(0.0f32, 0.0, 0.0), na::zero());

        // place the intersection ball at the point
        let ball_pos = Isometry3::new(
            Vector3::new(point[0] as f32, point[1] as f32, point[2] as f32),
            na::zero(),
        );

        // test for intersection
        let intersecting = intersection_test(&ball_pos, &self.ball, &cuboid_pos, &self.obstacle)
            .expect("Correct shapes should have been given.");

        // return true if not intersecting
        !intersecting
    }

    fn random_sample(&self) -> Vec<f64> {
        let between = Uniform::new(-4.0, 4.0);
        let mut rng = rand::thread_rng();
        vec![
            between.sample(&mut rng),
            between.sample(&mut rng),
            between.sample(&mut rng),
        ]
    }
}
