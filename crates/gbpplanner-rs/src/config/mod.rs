pub mod environment;
pub mod formation;
pub mod geometry;

pub mod reader;

use std::{num::NonZeroUsize, ops::RangeInclusive};

use bevy::{ecs::system::Resource, reflect::Reflect};
pub use environment::{Environment, EnvironmentType};
pub use formation::FormationGroup;
pub use reader::read_config;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
use typed_floats::{PositiveFinite, StrictlyPositiveFinite};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meter(f64);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizEdgeAttributes {
    // TODO: implement a way to validate this field to only match the valid edge styles: https://graphviz.org/docs/attr-types/style/
    pub style: String,
    pub len:   f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizInterrbotSection {
    pub edge: GraphvizEdgeAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizSection {
    pub interrobot:      GraphvizInterrbotSection,
    pub export_location: String,
}

impl Default for GraphvizSection {
    fn default() -> Self {
        Self {
            interrobot:      GraphvizInterrbotSection {
                edge: GraphvizEdgeAttributes {
                    style: "solid".to_string(),
                    len:   8.0,
                    color: "red".to_string(),
                },
            },
            export_location: "./assets/".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeightSection {
    pub objects:    f32,
    pub height_map: f32,
}

impl Default for HeightSection {
    fn default() -> Self {
        Self {
            objects:    0.5,
            height_map: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UncertaintySection {
    pub max_radius: f32,
    pub scale:      f32,
}

impl Default for UncertaintySection {
    fn default() -> Self {
        Self {
            max_radius: 5.0,
            scale:      100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ManualSection {
    pub timesteps_per_step: NonZeroUsize,
}

impl Default for ManualSection {
    fn default() -> Self {
        Self {
            timesteps_per_step: 1.try_into().expect("1 > 0"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VisualisationSection {
    pub height: HeightSection,
    pub draw: DrawSection,
    pub uncertainty: UncertaintySection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawSetting {
    CommunicationGraph,
    PredictedTrajectories,
    Waypoints,
    Uncertainty,
    Paths,
    GeneratedMap,
    HeightMap,
    Sdf,
    CommunicationRadius,
    Robots,
    ObstacleFactors,
    InterRobotFactors,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDrawSettingError;

impl std::str::FromStr for DrawSetting {
    type Err = ParseDrawSettingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let setting = match s {
            "communication_graph" => Self::CommunicationGraph,
            "predicted_trajectories" => Self::PredictedTrajectories,
            "waypoints" => Self::Waypoints,
            "uncertainty" => Self::Uncertainty,
            "paths" => Self::Paths,
            "generated_map" => Self::GeneratedMap,
            "height_map" => Self::HeightMap,
            "sdf" => Self::Sdf,
            "communication_radius" => Self::CommunicationRadius,
            "robots" => Self::Robots,
            "obstacle_factors" => Self::ObstacleFactors,
            "interrobot_factors" => Self::InterRobotFactors,

            _ => return Err(ParseDrawSettingError),
        };

        Ok(setting)
    }
}

// TODO: store in a bitset
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize, Iterable, Reflect, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DrawSection {
    pub robots: bool,
    pub communication_graph: bool,
    pub predicted_trajectories: bool,
    pub waypoints: bool,
    pub uncertainty: bool,
    pub paths: bool,
    pub communication_radius: bool,
    pub obstacle_factors: bool,
    pub interrobot_factors: bool,
    pub generated_map: bool,
    pub height_map: bool,
    pub sdf: bool,
}

impl Default for DrawSection {
    fn default() -> Self {
        Self {
            robots: true,
            communication_graph: false,
            predicted_trajectories: true,
            waypoints: false,
            uncertainty: false,
            paths: true,
            generated_map: true,
            height_map: false,
            sdf: false,
            communication_radius: false,
            obstacle_factors: false,
            interrobot_factors: false,
        }
    }
}

impl DrawSection {
    pub fn to_display_string(name: &str) -> String {
        match name {
            "communication_graph" => "Communication".to_string(),
            "predicted_trajectories" => "Trajectories".to_string(),
            "waypoints" => "Waypoints".to_string(),
            "uncertainty" => "Uncertainty".to_string(),
            "paths" => "Paths".to_string(),
            "generated_map" => "Generated Map".to_string(),
            "height_map" => "Height Map".to_string(),
            "sdf" => "SDF".to_string(),
            "communication_radius" => "Communication Radius".to_string(),
            "robots" => "Robots".to_string(),
            "obstacle_factors" => "Obstacle Factors".to_string(),
            "interrobot_factors" => "InterRobot Factors".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

/// **Simulation Section**
/// Contains parameters for the simulation such as the fixed timestep frequency,
/// max time to run the simulation, world size, and random seed to get
/// reproducible results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SimulationSection {
    /// Time between current state and next state of planned path
    /// SI unit: s
    pub t0: PositiveFinite<f32>,

    /// Maximum time after which the simulation will terminate
    /// SI unit: s
    pub max_time: StrictlyPositiveFinite<f32>,

    /// The relative scale of time in the simulation.
    /// 1.0 means real-time, 0.5 means half-speed, 2.0 means double-speed, etc.
    pub time_scale: StrictlyPositiveFinite<f32>,

    /// How many steps of size 1.0 / hz to take when manually stepping the
    /// simulation. SI unit: s
    pub manual_step_factor: usize,

    /// The fixed time step size to be used in the simulation.
    /// SI unit: s
    pub hz: f64,

    /// The side length of the smallest square that contains the entire
    /// simulated environment. Size of the environment in meters.
    /// SI unit: m
    pub world_size: StrictlyPositiveFinite<f32>,
    /// The seed at which random number generators should be seeded, to ensure
    /// deterministic results across simulation runs.
    pub prng_seed:  u64,

    /// Whether to pause the simulation time when the first robot is spawned
    pub pause_on_spawn: bool,

    /// Whether to despawn a robot when it reaches its final waypoint and
    /// "completes" its route.
    /// Exists for the circle formation environment, where it looks slick if
    /// they all stay at at the end along the perimeter.
    pub despawn_robot_when_final_waypoint_reached: bool,
}

impl Default for SimulationSection {
    fn default() -> Self {
        Self {
            t0: 0.25.try_into().expect("0.0 >= 0.0"),
            max_time: 10000.0.try_into().expect("10000.0 > 0.0"),
            time_scale: 1.0.try_into().expect("1.0 > 0.0"),
            manual_step_factor: 1,
            hz: 60.0,
            world_size: 100.0.try_into().expect("100.0 > 0.0"),
            // world_size:         StrictlyPositiveFinite::<f32>::new(100.0).expect("100.0 > 0.0"),
            prng_seed: 0,
            pause_on_spawn: false,
            despawn_robot_when_final_waypoint_reached: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GbpSchedule {
    #[default]
    Centered,
    SoonAsPossible,
    LateAsPossible,
    InterleaveEvenly,
    HalfBeginningHalfEnd,
}

// impl GbpSchedule {
//     pub fn get_schedule(&self) -> impl gbp_schedule::GbpSchedule {
//         match self {
//             GbpSchedule::Centered => gbp_schedule::Centered,
//             GbpSchedule::SoonAsPossible => gbp_schedule::SoonAsPossible,
//             GbpSchedule::LateAsPossible => gbp_schedule::LateAsPossible,
//             GbpSchedule::InterleaveEvenly => gbp_schedule::InterleaveEvenly,
//             GbpSchedule::HalfBeginningHalfEnd =>
// gbp_schedule::HalfBeginningHalfEnd,         }
//     }
// }

/// Configuration for how many iterations to run different parts of the GBP
/// algorithm per timestep
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GbpIterationsPerTimestepSection {
    /// Internal iteration i.e. Variables, and factors excluding interrobot
    /// factors
    pub internal: usize,
    /// External iteration i.e. message passing between interrobot factors and
    /// connected external factors
    pub external: usize,
    // pub schedule: GbpSchedule,
}

impl Default for GbpIterationsPerTimestepSection {
    fn default() -> Self {
        let n = 10;
        Self {
            internal: n,
            external: n,
            // schedule: GbpSchedule::default(),
        }
    }
}

/// **GBP Section**
/// Contains parameters for the GBP algorithm. These paraneters are used for
/// initialisation of factors and prediction horizon steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GbpSection {
    /// Sigma for Unary pose factor on current and horizon states
    pub sigma_pose_fixed: f32,
    /// Sigma for Dynamics factors
    pub sigma_factor_dynamics: f32,
    /// Sigma for Interrobot factor
    pub sigma_factor_interrobot: f32,
    /// Sigma for Static obstacle factors
    pub sigma_factor_obstacle: f32,
    /// Parameter affecting how planned path is spaced out in time
    pub lookahead_multiple: usize,
    /// Number of iterations of GBP per timestep
    // pub iterations_per_timestep: usize,
    pub iterations_per_timestep: GbpIterationsPerTimestepSection,
}

impl Default for GbpSection {
    fn default() -> Self {
        Self {
            sigma_pose_fixed: 1e-15,
            sigma_factor_dynamics: 0.1,
            sigma_factor_interrobot: 0.01,
            sigma_factor_obstacle: 0.01,
            lookahead_multiple: 3,
            // iterations_per_timestep: 10,
            iterations_per_timestep: Default::default(),
        }
    }
}

/// **Communication Section**
/// Contains parameters for the communication between robots
/// - `radius`: Inter-robot factors created if robots are within this range of
///   each other
/// - `failure_rate`: Probability for failing to send/receive a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CommunicationSection {
    /// Inter-robot factors created if robots are within this range of each
    /// other SI unit: m
    pub radius: StrictlyPositiveFinite<f32>,

    // TODO: use a percentage type instead of f32
    /// Probability for failing to send/receive a message
    pub failure_rate: f32,
}

impl Default for CommunicationSection {
    fn default() -> Self {
        Self {
            radius:       20.0.try_into().expect("20.0 > 0.0"),
            failure_rate: 0.2,
        }
    }
}

type NaturalQuantity = StrictlyPositiveFinite<f32>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RobotRadiusSection {
    pub min: StrictlyPositiveFinite<f32>,
    pub max: StrictlyPositiveFinite<f32>,
}

impl RobotRadiusSection {
    /// Returns the range that the radius can take
    pub fn range(&self) -> RangeInclusive<f32> {
        self.min.get()..=self.max.get()
    }
}

impl Default for RobotRadiusSection {
    fn default() -> Self {
        Self {
            min: 1.0.try_into().expect("1.0 > 0.0"),
            max: 1.0.try_into().expect("1.0 > 0.0"),
        }
    }
}

/// **Robot Section**
/// Contains parameters for the robot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RobotSection {
    /// SI unit: s
    pub planning_horizon: StrictlyPositiveFinite<f32>,
    /// SI unit: m/s
    pub max_speed: StrictlyPositiveFinite<f32>,
    /// Degrees of freedom of the robot's state [x, y, x', y']
    pub dofs: NonZeroUsize,
    // /// Simulation timestep interval
    // /// FIXME: does not belong to group of parameters, should be in SimulationSettings or
    // something pub delta_t: f32,
    /// If true, when inter-robot factors need to be created between two robots,
    /// a pair of factors is created (one belonging to each robot). This becomes
    /// a redundancy.
    pub symmetric_factors: bool,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest
    /// circle that fully encompass the shape of the robot. **constraint**:
    /// > 0.0
    // pub radius: StrictlyPositiveFinite<f32>,
    pub radius: RobotRadiusSection,
    /// Communication parameters
    pub communication: CommunicationSection,
}

impl Default for RobotSection {
    fn default() -> Self {
        Self {
            planning_horizon: StrictlyPositiveFinite::<f32>::new(5.0).expect("5.0 > 0.0"),
            max_speed: StrictlyPositiveFinite::<f32>::new(4.0).expect("2.0 > 0.0"),
            dofs: NonZeroUsize::new(4).expect("4 > 0"),

            symmetric_factors: true,
            // radius: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            radius: RobotRadiusSection::default(),
            communication: CommunicationSection::default(),
        }
    }
}

/// Interaction Section
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InteractionSection {
    /// If true, when a UI element is focused, some inputs are cancelled
    /// such as camera movement. Some inputs are still allowed, such as
    /// manual time-stepping or general keybinds to close the window and sim.
    pub ui_focus_cancels_inputs: bool,
    /// Default camera distance from the origin.
    /// Can also be interpreted as default zoom level
    pub default_cam_distance:    f32,
}

impl Default for InteractionSection {
    fn default() -> Self {
        Self {
            ui_focus_cancels_inputs: true,
            default_cam_distance:    125.0,
        }
    }
}

/// **RRT Section**
/// Contains parameters for the RRT algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RrtSection {
    /// Maximum number of iterations to run the RRT algorithm
    pub max_iterations: NonZeroUsize,
    /// Length to extend the random branches by in each iteration
    pub step_size: StrictlyPositiveFinite<f32>,
    /// The collision radius to check for each iteration
    pub collision_radius: StrictlyPositiveFinite<f32>,
    /// The smoothing parameters
    pub smoothing: SmoothingSection,
}

impl Default for RrtSection {
    fn default() -> Self {
        Self {
            max_iterations: NonZeroUsize::new(10000).expect("1000 > 0"),
            step_size: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            collision_radius: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            smoothing: SmoothingSection::default(),
        }
    }
}

/// **Smoothing Section**
/// Contains parameters for smoothing the path generated by the RRT algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SmoothingSection {
    /// Number of iterations to smooth the path
    /// `max_iterations` must be greater than 0
    /// - Describes the amount of random samples to attempt to smooth the path
    pub max_iterations: NonZeroUsize,
    /// Idk actually but it's there
    pub step_size:      StrictlyPositiveFinite<f32>,
}

impl Default for SmoothingSection {
    fn default() -> Self {
        Self {
            max_iterations: NonZeroUsize::new(100).expect("100 > 0"),
            step_size:      StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
        }
    }
}

/// Collection of all the sections in the config file
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Config {
    /// Path to the **.png** containing the environment sdf
    pub environment_image: String,
    /// Path to the environment configuration file
    pub environment: String,
    /// Path to the formation configuration file
    pub formation_group: String,
    /// **Visualisation section:**
    /// Contains parameters for which elements of the GBP and simulation to draw
    pub visualisation: VisualisationSection,
    /// **Interaction section:**
    /// Contains parameters for the interaction with the simulation
    /// and the initial scene setup
    pub interaction: InteractionSection,
    /// **GBP section:**
    /// Contains parameters for the GBP algorithm such as sigma values and
    /// number of iterations
    pub gbp: GbpSection,
    /// **Robot section:**
    /// Contains parameters for the robot such as radius, speed, and
    /// communication
    pub robot: RobotSection,
    /// **Simulation section:**
    /// Contains parameters for the simulation such as timestep, max time, and
    /// world size
    pub simulation: SimulationSection,
    /// **RRT section:**
    /// Contains parameters for the RRT algorithm such as max iterations, step
    /// size and smoothing parameters
    pub rrt: RrtSection,
    /// **Graphviz section:**
    /// Contains parameters for how to export to graphviz
    pub graphviz: GraphvizSection,
    /// **Manual section:**
    /// Contains parameters for manual time-stepping
    pub manual: ManualSection,
}

impl Default for Config {
    /// Generate a default config
    /// Used when no config file is provided
    fn default() -> Self {
        // TODO: make a bit more robust
        // let cwd = std::env::current_dir().expect("The current working directory
        // exists"); let default_environment =
        // cwd.join("gbpplanner-rs/assets/imgs/junction.png");
        let default_environment_image = "junction".to_string();
        let default_environment_config = "./config/environment.yaml".to_string();
        let default_formation_config = "./config/formation.ron".to_string();

        Self {
            environment_image: default_environment_image,
            environment: default_environment_config,
            formation_group: default_formation_config,
            visualisation: VisualisationSection::default(),
            interaction: InteractionSection::default(),
            gbp: GbpSection::default(),
            robot: RobotSection::default(),
            simulation: SimulationSection::default(),
            rrt: RrtSection::default(),
            graphviz: GraphvizSection::default(),
            manual: ManualSection::default(),
        }
    }
}

impl Config {
    /// Parse a config file from a given path
    pub fn from_file<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<std::path::Path>,
    {
        let file_contents = std::fs::read_to_string(path)?;
        Self::parse(file_contents.as_str())
    }

    /// Parse a config file
    /// Returns a `ParseError` if the file cannot be parsed
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        let config = toml::from_str(contents)?;
        Ok(config)
    }
}
