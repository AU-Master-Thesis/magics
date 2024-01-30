mod config;
mod factor;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use clap::{Parser, Subcommand};

use crate::config::Config;

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Johannes Schickling")]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    #[arg(long)]
    dump_default_config: bool,
}

// use macroquad::prelude::*;

// #[macroquad::main("BasicShapes")]
// async fn main() {
//     loop {
//         clear_background(RED);

//         draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
//         draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
//         draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

//         draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

//         next_frame().await
//     }
// }

// struct RobotId(usize);
type RobotId = usize;
// struct NodeId(usize);
type NodeId = usize;

trait Factor {}

#[derive(Debug)]
struct DefaultFactor;
#[derive(Debug)]
struct DynamicFactor;
#[derive(Debug)]
struct InterRobotFactor;
#[derive(Debug)]
struct ObstacleFactor;

impl Factor for DefaultFactor {}
impl Factor for DynamicFactor {}
impl Factor for InterRobotFactor {}
impl Factor for ObstacleFactor {}

#[derive(Debug)]
enum FactorType {
    Default(DefaultFactor),
    Dynamic(DynamicFactor),
    InterRobot(InterRobotFactor),
    Ocstacle(ObstacleFactor),
}

// enum MsgPassingMode {EXTERNAL, INTERNAL};

/// Ways message passing can be performed
#[derive(Debug)]
enum MessagePassingMode {
    /// Between two different robots/factorgraphs
    External,
    /// Within a robot/factorgraph
    Internal,
}
// Eigen::VectorXd eta;
// Eigen::MatrixXd lambda;
// Eigen::VectorXd mu;

#[derive(Debug)]
struct Message {
    pub eta: nalgebra::DVector<f64>,
    pub lambda: nalgebra::DMatrix<f64>,
    pub mu: nalgebra::DVector<f64>,
}

// Implement addtion and subtraction for messages
impl std::ops::Add for Message {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            eta: self.eta + other.eta,
            lambda: self.lambda + other.lambda,
            mu: self.mu + other.mu,
        }
    }
}

impl std::ops::Sub for Message {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            eta: self.eta - other.eta,
            lambda: self.lambda - other.lambda,
            mu: self.mu - other.mu,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
struct Variable {
    node_id: NodeId,
    robot_id: RobotId,
}

impl Variable {
    fn new(node_id: NodeId, robot_id: RobotId) -> Self {
        Self { node_id, robot_id }
    }
}

fn hello_world() {
    println!("hello world!");
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don’t need to change any other names
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Circle
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
        ..default()
    });

    // Rectangle
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
        ..default()
    });

    // Quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(50., 100.)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        ..default()
    });

    // Hexagon
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
        ..default()
    });
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Startup, add_people)
            .add_systems(Update, (update_people, greet_people).chain());
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if cli.dump_default_config {
        let default_config = Config::default();

        // Write to stdout
        print!("{}", toml::to_string_pretty(&default_config)?);
    }

    let config = if let Some(config_path) = cli.config {
        Config::parse(config_path)?
    } else {
        Config::default()
    };

    App::new()
        .add_plugins((DefaultPlugins, HelloPlugin))
        .add_systems(Startup, setup)
        // .add_systems(Startup, add_people)
        //  .add_systems(Update, (hello_world, (update_people, greet_people).chain()))
        .run();

    Ok(())
}
