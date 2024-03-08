pub mod formation;

use bevy::ecs::system::Resource;
use bevy::reflect::Reflect;
pub use formation::Formation;
pub use formation::FormationGroup;

use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;

use crate::ui::ToDisplayString;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meter(f64);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizEdgeAttributes {
    // TODO: implement a way to validate this field to only match the valid edge styles: https://graphviz.org/docs/attr-types/style/
    pub style: String,
    pub len: f32,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizInterrbotSection {
    pub edge: GraphvizEdgeAttributes,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizSection {
    pub interrobot: GraphvizInterrbotSection,
    pub export_location: String,
}

impl Default for GraphvizSection {
    fn default() -> Self {
        Self {
            interrobot: GraphvizInterrbotSection {
                edge: GraphvizEdgeAttributes {
                    style: "solid".to_string(),
                    len: 8.0,
                    color: "red".to_string(),
                },
            },
            export_location: "./assets/".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeighSection {
    pub objects: f32,
    pub height_map: f32,
}

impl Default for HeighSection {
    fn default() -> Self {
        Self {
            objects: 0.5,
            height_map: 1.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VisualisationSection {
    pub height: HeighSection,
    pub draw: DrawSection,
}

impl Default for VisualisationSection {
    fn default() -> Self {
        Self {
            height: HeighSection::default(),
            draw: DrawSection::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DrawSetting {
    CommunicationGraph,
    PredictedTrajectories,
    Waypoints,
    Uncertainty,
    Paths,
    HeightMap,
    FlatMap,
}

impl DrawSetting {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "communication_graph" => Some(Self::CommunicationGraph),
            "predicted_trajectories" => Some(Self::PredictedTrajectories),
            "waypoints" => Some(Self::Waypoints),
            "uncertainty" => Some(Self::Uncertainty),
            "paths" => Some(Self::Paths),
            "height_map" => Some(Self::HeightMap),
            "flat_map" => Some(Self::FlatMap),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Iterable, Reflect, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DrawSection {
    pub communication_graph: bool,
    pub predicted_trajectories: bool,
    pub waypoints: bool,
    pub uncertainty: bool,
    pub paths: bool,
    pub height_map: bool,
    pub flat_map: bool,
}

impl Default for DrawSection {
    fn default() -> Self {
        Self {
            communication_graph: false,
            predicted_trajectories: true,
            waypoints: true,
            uncertainty: false,
            paths: false,
            height_map: false,
            flat_map: true,
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
            "height_map" => "Height Map".to_string(),
            "flat_map" => "Flat Map".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SimulationSection {
    // TODO: read count from input formation structure/array
    // pub num_robots: usize,
    // / SI unit: s
    // pub timestep: f32,
    /// Time between current state and next state of planned path
    /// SI unit: s
    pub t0: f32,

    /// Maximum time after which the simulation will terminate
    /// SI unit: s
    pub max_time: f32,

    /// The side length of the smallest square that contains the entire simulated environment.
    /// Size of the environment in meters.
    /// SI unit: m
    pub world_size: f32,
    /// The seed at which random number generators should be seeded, to ensure deterministic results across
    /// simulation runs.
    pub random_seed: usize,
}

impl Default for SimulationSection {
    fn default() -> Self {
        Self {
            // num_robots: 1,
            // timestep: 0.01,
            t0: 0.0,
            max_time: 10000.0,
            world_size: 100.0,
            random_seed: 0usize,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    /// Number of iterations of GBP per timestep
    pub iterations_per_timestep: usize,
    /// Parameter affecting how planned path is spaced out in time
    pub lookahead_multiple: usize,
}

impl Default for GbpSection {
    fn default() -> Self {
        Self {
            sigma_pose_fixed: 1e-15,
            sigma_factor_dynamics: 0.1,
            sigma_factor_interrobot: 0.01,
            sigma_factor_obstacle: 0.01,
            iterations_per_timestep: 10,
            lookahead_multiple: 3,
            // damping: 0.0
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CommunicationSection {
    /// Inter-robot factors created if robots are within this range of each other
    /// SI unit: m
    pub radius: f32,

    /// Probability for failing to send/receive a message
    pub failure_rate: f32,
}

impl Default for CommunicationSection {
    fn default() -> Self {
        Self {
            radius: 20.0,
            failure_rate: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RobotSection {
    /// SI unit: s
    pub planning_horizon: f32,
    /// SI unit: m/s
    pub max_speed: f32,
    /// Degrees of freedom of the robot's state [x, y, x', y']
    pub dofs: usize,
    // /// Simulation timestep interval
    // /// FIXME: does not belong to group of parameters, should be in SimulationSettings or something
    // pub delta_t: f32,
    /// If true, when inter-robot factors need to be created between two robots,
    /// a pair of factors is created (one belonging to each robot). This becomes a redundancy.
    pub symmetric_factors: bool,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest circle that fully encompass the shape of the robot.
    /// **constraint**: > 0.0
    pub radius: f32,
    pub communication: CommunicationSection,
}

impl Default for RobotSection {
    fn default() -> Self {
        Self {
            planning_horizon: 5.0,
            max_speed: 2.0,
            dofs: 4,

            symmetric_factors: true,
            radius: 1.0,
            communication: CommunicationSection::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Resource)]
pub struct Config {
    /// Path to the **.png** containing the environment sdf
    pub environment: String,
    pub formation_group: String,
    pub visualisation: VisualisationSection,
    pub gbp: GbpSection,
    pub robot: RobotSection,
    pub simulation: SimulationSection,
    pub graphviz: GraphvizSection,
}

impl Default for Config {
    /// Generate a default config
    /// Used when no config file is provided
    fn default() -> Self {
        // TODO: make a bit more robust
        // let cwd = std::env::current_dir().expect("The current working directory exists");
        // let default_environment = cwd.join("gbpplanner-rs/assets/imgs/junction.png");
        let default_environment = "./gbpplanner-rs/assets/imgs/junction.png".to_string();
        let default_formation_group = "./config/formation.ron".to_string();

        Self {
            environment: default_environment,
            formation_group: default_formation_group,
            visualisation: VisualisationSection::default(),
            gbp: GbpSection::default(),
            robot: RobotSection::default(),
            simulation: SimulationSection::default(),
            graphviz: GraphvizSection::default(),
        }
    }
}

impl Config {
    /// Parse a config file from a given path
    pub fn from_file(file_path: &std::path::PathBuf) -> Result<Self, ParseError> {
        let file_contents = std::fs::read_to_string(file_path)?;
        Self::parse(file_contents.as_str())
    }

    /// Parse a config file
    /// Returns a `ParseError` if the file cannot be parsed
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        let config = toml::from_str(contents)?;
        Ok(config)
    }
}
