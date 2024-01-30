mod config;
mod factor;

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

    Ok(())
}
