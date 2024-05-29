// pub mod environment;
pub mod formation;
pub mod geometry;
pub mod reader;

use std::{num::NonZeroUsize, ops::RangeInclusive};

use bevy::{
    ecs::system::Resource,
    reflect::{GetField, Reflect},
};
// pub use environment::{Environment, EnvironmentType};
pub use formation::FormationGroup;
use gbp_schedule::GbpSchedule;
pub use reader::read_config;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
use typed_floats::StrictlyPositiveFinite;

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
pub struct GraphvizInterrobotSection {
    pub active:   GraphvizEdgeAttributes,
    pub inactive: GraphvizEdgeAttributes,
    // pub edge: GraphvizEdgeAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphvizSection {
    pub interrobot:      GraphvizInterrobotSection,
    #[serde(default = "GraphvizSection::default_export_location")]
    pub export_location: String,
}

impl GraphvizSection {
    pub fn default_export_location() -> String {
        "./assets/export".to_string()
    }
}

impl Default for GraphvizSection {
    fn default() -> Self {
        Self {
            interrobot:      GraphvizInterrobotSection {
                active:   GraphvizEdgeAttributes {
                    style: "solid".to_string(),
                    len:   8.0,
                    color: "green".to_string(),
                },
                inactive: GraphvizEdgeAttributes {
                    style: "dashed".to_string(),
                    len:   4.0,
                    color: "green".to_string(),
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
    #[serde(default)]
    pub height: HeightSection,
    #[serde(default)]
    pub draw: DrawSection,
    #[serde(default)]
    pub uncertainty: UncertaintySection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumIter, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DrawSetting {
    CommunicationGraph,
    PredictedTrajectories,
    Waypoints,
    Uncertainty,
    Paths,
    GeneratedMap,
    // HeightMap,
    Sdf,
    CommunicationRadius,
    Robots,
    ObstacleFactors,
    Tracking,
    #[strum(serialize = "interrobot_factors")]
    InterRobotFactors,
    #[strum(serialize = "interrobot_factors_safety_distance")]
    InterRobotFactorsSafetyDistance,
    RobotColliders,
    RobotRobotCollisions,
    EnvironmentColliders,
    RobotEnvironmentCollisions,
    // InfiniteGrid,
}

// TODO: store in a bitset
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Iterable, Reflect, Clone, Copy)]
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
    pub tracking: bool,
    pub interrobot_factors: bool,
    pub interrobot_factors_safety_distance: bool,
    pub generated_map: bool,
    // pub height_map: bool,
    pub sdf: bool,
    pub robot_colliders: bool,
    pub environment_colliders: bool,
    pub robot_robot_collisions: bool,
    pub robot_environment_collisions: bool,
    // pub infinite_grid: bool,
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
            // height_map: false,
            sdf: false,
            communication_radius: false,
            obstacle_factors: false,
            tracking: false,
            interrobot_factors: false,
            interrobot_factors_safety_distance: false,
            robot_colliders: false,
            environment_colliders: false,
            robot_robot_collisions: false,
            robot_environment_collisions: false,
            // infinite_grid: true,
        }
    }
}

impl DrawSection {
    pub fn to_display_string(name: &str) -> &'static str {
        match name {
            "communication_graph" => "Communication Graph",
            "predicted_trajectories" => "Trajectories",
            "waypoints" => "Waypoints",
            "uncertainty" => "Uncertainty",
            "paths" => "Paths",
            "generated_map" => "Generated Map",
            // "height_map" => "Height Map",
            "sdf" => "SDF",
            "communication_radius" => "Communication Radius",
            "robots" => "Robots",
            "tracking" => "Tracking",
            "obstacle_factors" => "Obstacle Factors",
            "interrobot_factors" => "InterRobot Factors",
            "interrobot_factors_safety_distance" => "InterRobot Safety Distance",
            "robot_colliders" => "Robot Colliders",
            "environment_colliders" => "Environment Colliders",
            "robot_robot_collisions" => "Robot-Robot Collisions",
            "robot_environment_collisions" => "Robot-Environment Collisions",
            // "infinite_grid" => "Infinite Grid",
            _ => "Unknown",
        }
    }

    pub fn all_disabled() -> Self {
        let mut instance = Self::default();
        let copy = instance;

        for (name, _) in copy.iter() {
            if let Some(field) = instance.get_field_mut::<bool>(name) {
                *field = false;
            }
        }

        instance
    }

    pub fn all_enabled() -> Self {
        let mut instance = Self::default();
        let copy = instance;

        for (name, _) in copy.iter() {
            if let Some(field) = instance.get_field_mut::<bool>(name) {
                *field = true;
            }
        }

        instance
    }

    pub fn flip_all(&mut self) {
        let copy = *self;

        copy.iter().for_each(|(name, _)| {
            if let Some(field) = self.get_field_mut::<bool>(name) {
                *field = !*field;
            }
        });
    }
}

/// **Simulation Section**
/// Contains parameters for the simulation such as the fixed timestep frequency,
/// max time to run the simulation, world size, and random seed to get
/// reproducible results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SimulationSection {
    // /// Time between current state and next state of planned path
    // /// SI unit: s
    // pub t0: PositiveFinite<f32>,
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

    // /// The side length of the smallest square that contains the entire
    // /// simulated environment. Size of the environment in meters.
    // /// SI unit: m
    // pub world_size: StrictlyPositiveFinite<f32>,
    /// The seed at which random number generators should be seeded, to ensure
    /// deterministic results across simulation runs.
    pub prng_seed: u64,

    /// Whether to pause the simulation time when the first robot is spawned
    pub pause_on_spawn: bool,

    /// Whether to despawn a robot when it reaches its final waypoint and
    /// "completes" its route.
    /// Exists for the circle formation environment, where it looks slick if
    /// they all stay at at the end along the perimeter.
    pub despawn_robot_when_final_waypoint_reached: bool,

    #[serde(default = "SimulationSection::default_exit_application_on_scenario_finished")]
    pub exit_application_on_scenario_finished: bool,
}

impl SimulationSection {
    fn default_exit_application_on_scenario_finished() -> bool {
        false
    }
}

impl Default for SimulationSection {
    fn default() -> Self {
        Self {
            // t0: 0.25.try_into().expect("0.0 >= 0.0"),
            max_time: 10000.0.try_into().expect("10000.0 > 0.0"),
            time_scale: 1.0.try_into().expect("1.0 > 0.0"),
            manual_step_factor: 1,
            hz: 60.0,
            // world_size: 100.0.try_into().expect("100.0 > 0.0"),
            // world_size:         StrictlyPositiveFinite::<f32>::new(100.0).expect("100.0 > 0.0"),
            prng_seed: 0,
            pause_on_spawn: false,
            despawn_robot_when_final_waypoint_reached: true,
            exit_application_on_scenario_finished:
                Self::default_exit_application_on_scenario_finished(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    serde::Serialize,
    serde::Deserialize,
    strum_macros::EnumIter,
    strum_macros::IntoStaticStr,
    strum_macros::Display,
)]
#[serde(rename_all = "kebab-case")]
pub enum GbpIterationScheduleKind {
    #[default]
    #[strum(serialize = "Centered")]
    Centered,
    #[strum(serialize = "Soon as Possible")]
    SoonAsPossible,
    #[strum(serialize = "Late as Possible")]
    LateAsPossible,
    #[strum(serialize = "Interleave Evenly")]
    InterleaveEvenly,
    #[strum(serialize = "Half Beginning Half End")]
    HalfBeginningHalfEnd,
}

impl GbpIterationScheduleKind {
    pub fn get(
        &self,
        config: gbp_schedule::GbpScheduleParams,
    ) -> Box<dyn gbp_schedule::GbpScheduleIterator> {
        match self {
            GbpIterationScheduleKind::Centered => {
                Box::new(gbp_schedule::Centered::schedule(config))
            }
            GbpIterationScheduleKind::InterleaveEvenly => {
                Box::new(gbp_schedule::InterleaveEvenly::schedule(config))
            }
            GbpIterationScheduleKind::SoonAsPossible => {
                Box::new(gbp_schedule::SoonAsPossible::schedule(config))
            }
            GbpIterationScheduleKind::LateAsPossible => {
                Box::new(gbp_schedule::LateAsPossible::schedule(config))
            }
            GbpIterationScheduleKind::HalfBeginningHalfEnd => {
                Box::new(gbp_schedule::HalfBeginningHalfEnd::schedule(config))
            }
        }
    }
}

/// Configuration for how many iterations to run different parts of the GBP
/// algorithm per timestep
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GbpIterationSchedule {
    /// Internal iteration i.e. Variables, and factors excluding interrobot
    /// factors
    pub internal: usize,
    /// External iteration i.e. message passing between interrobot factors and
    /// connected external factors
    pub external: usize,
    pub schedule: GbpIterationScheduleKind,
}

impl Default for GbpIterationSchedule {
    fn default() -> Self {
        let n = 10;
        Self {
            internal: n,
            external: n,
            schedule: GbpIterationScheduleKind::default(),
        }
    }
}

// macro_rules! impl_factor_section {
//     ($name:ident) => {
//         paste::paste! {
//         pub struct [<$name:upper:camel>Section] {
//
//         }
//         }
//     };
// }

// impl_factor_section!(pose);
// impl_factor_section!(dynamics);
// impl_factor_section!(interrobot);
// impl_factor_section!(obstacle);
// impl_factor_section!(tracking);

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    struct_iterable::Iterable,
    bevy::reflect::Reflect,
)]
#[serde(rename_all = "kebab-case")]
pub struct FactorsEnabledSection {
    // pub pose:       bool,
    #[serde(default = "FactorsEnabledSection::default_dynamic")]
    pub dynamic:    bool,
    #[serde(default = "FactorsEnabledSection::default_interrobot")]
    pub interrobot: bool,
    #[serde(default = "FactorsEnabledSection::default_obstacle")]
    pub obstacle:   bool,
    #[serde(default = "FactorsEnabledSection::default_tracking")]
    pub tracking:   bool,
}

impl FactorsEnabledSection {
    fn default_tracking() -> bool {
        false
    }

    fn default_dynamic() -> bool {
        true
    }

    fn default_interrobot() -> bool {
        true
    }

    fn default_obstacle() -> bool {
        true
    }
}

impl Default for FactorsEnabledSection {
    fn default() -> Self {
        Self {
            // pose:       true,
            dynamic:    Self::default_dynamic(),
            interrobot: Self::default_interrobot(),
            obstacle:   Self::default_obstacle(),
            tracking:   Self::default_tracking(),
        }
    }
}

/// **Tracking Section**
/// Contains parameters for the tracking factor
/// - `switch_padding`: Padding around the switch point
/// - `attraction_distance`: Distance to the tracking line to normalise around
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TrackingSection {
    #[serde(default = "default_switch_padding")]
    pub switch_padding:      f32,
    #[serde(default = "default_attraction_distance")]
    pub attraction_distance: f32,
}

/// Default value for the attraction distance
fn default_attraction_distance() -> f32 {
    2.0
}

/// Default value for the switch padding
fn default_switch_padding() -> f32 {
    1.0
}

impl Default for TrackingSection {
    fn default() -> Self {
        Self {
            switch_padding:      default_switch_padding(),
            attraction_distance: default_attraction_distance(),
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
    /// Sigma for Tracking factors
    pub sigma_factor_tracking: f32,
    /// Parameter affecting how planned path is spaced out in time
    pub lookahead_multiple: usize,
    /// Tracking section
    #[serde(default)]
    pub tracking: TrackingSection,
    /// Schedule for how many iterations to run different parts of the GBP
    pub iteration_schedule: GbpIterationSchedule,
    /// Section for enabling/disabling factors
    #[serde(default)]
    pub factors_enabled: FactorsEnabledSection,
    /// Number of variables to create
    #[serde(default = "GbpSection::default_variables")]
    pub variables: usize,
}

impl GbpSection {
    fn default_variables() -> usize {
        10
    }
}

impl Default for GbpSection {
    fn default() -> Self {
        Self {
            sigma_pose_fixed: 1e-15,
            sigma_factor_dynamics: 0.1,
            sigma_factor_interrobot: 0.01,
            sigma_factor_obstacle: 0.01,
            sigma_factor_tracking: 0.1,
            lookahead_multiple: 3,
            tracking: TrackingSection::default(),
            // iterations_per_timestep: 10,
            iteration_schedule: GbpIterationSchedule::default(),
            // FIXME: not properly read when desirialized from toml
            factors_enabled: FactorsEnabledSection::default(),
            variables: Self::default_variables(),
            // ..Default::default()
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
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest
    /// circle that fully encompass the shape of the robot. **constraint**:
    /// > 0.0
    pub radius: RobotRadiusSection,
    /// Communication parameters
    pub communication: CommunicationSection,
    pub inter_robot_safety_distance_multiplier: StrictlyPositiveFinite<f32>,
}

impl Default for RobotSection {
    fn default() -> Self {
        Self {
            planning_horizon: StrictlyPositiveFinite::<f32>::new(5.0).expect("5.0 > 0.0"),
            max_speed: StrictlyPositiveFinite::<f32>::new(4.0).expect("2.0 > 0.0"),
            // radius: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            radius: RobotRadiusSection::default(),
            communication: CommunicationSection::default(),

            // **gbpplanner** effectively uses 2.2 * radius with the way they calculate it
            inter_robot_safety_distance_multiplier: StrictlyPositiveFinite::<f32>::new(2.2)
                .expect("2.2 > 0.0"),
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
pub struct RRTSection {
    /// Maximum number of iterations to run the RRT algorithm
    pub max_iterations: NonZeroUsize,
    /// Length to extend the random branches by in each iteration
    pub step_size: StrictlyPositiveFinite<f32>,
    /// The collision radius to check for each iteration
    pub collision_radius: StrictlyPositiveFinite<f32>,
    /// Neighbourhood radius for RRT*
    pub neighbourhood_radius: StrictlyPositiveFinite<f32>,
    /// The smoothing parameters
    #[serde(default)]
    pub smoothing: SmoothingSection,
}

impl Default for RRTSection {
    fn default() -> Self {
        Self {
            max_iterations: NonZeroUsize::new(10000).expect("1000 > 0"),
            step_size: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            collision_radius: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            neighbourhood_radius: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
            smoothing: SmoothingSection::default(),
        }
    }
}

/// **Smoothing Section**
/// Contains parameters for smoothing the path generated by the RRT algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SmoothingSection {
    /// Whether to do path smoothing or not
    pub enabled: bool,
    /// Number of iterations to smooth the path
    /// `max_iterations` must be greater than 0
    /// - Describes the amount of random samples to attempt to smooth the path
    pub max_iterations: NonZeroUsize,
    /// Idk actually but it's there
    pub step_size: StrictlyPositiveFinite<f32>,
}

impl Default for SmoothingSection {
    fn default() -> Self {
        Self {
            enabled: true,
            max_iterations: NonZeroUsize::new(100).expect("100 > 0"),
            step_size: StrictlyPositiveFinite::<f32>::new(1.0).expect("1.0 > 0.0"),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DebugSection {
    pub on_variable_clicked: OnVariableClickedSection,
}

#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    bevy::reflect::Reflect,
    struct_iterable::Iterable,
)]
#[serde(rename_all = "kebab-case")]
pub struct OnVariableClickedSection {
    pub obstacle:   bool,
    pub dynamic:    bool,
    pub interrobot: bool,
    pub tracking:   bool,
    pub variable:   bool,
}

impl Default for OnVariableClickedSection {
    fn default() -> Self {
        Self {
            obstacle:   false,
            dynamic:    false,
            interrobot: false,
            tracking:   false,
            variable:   false,
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
    #[serde(default)]
    pub visualisation: VisualisationSection,
    /// **Interaction section:**
    /// Contains parameters for the interaction with the simulation
    /// and the initial scene setup
    #[serde(default)]
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
    #[serde(default)]
    pub simulation: SimulationSection,
    /// **RRT section:**
    /// Contains parameters for the RRT algorithm such as max iterations, step
    /// size and smoothing parameters
    #[serde(default)]
    pub rrt: RRTSection,
    /// **Graphviz section:**
    /// Contains parameters for how to export to graphviz
    #[serde(default)]
    pub graphviz: GraphvizSection,
    /// **Manual section:**
    /// Contains parameters for manual time-stepping
    #[serde(default)]
    pub manual: ManualSection,

    #[serde(default)]
    pub debug: DebugSection,
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
            rrt: RRTSection::default(),
            graphviz: GraphvizSection::default(),
            manual: ManualSection::default(),
            debug: DebugSection::default(),
        }
    }
}

impl Config {
    /// Parse a config file from a given path
    pub fn from_file<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<std::path::Path>,
    {
        std::fs::read_to_string(path)
            .map_err(Into::into)
            .and_then(|contents| Self::parse(contents.as_str()))
        // let file_contents = std::fs::read_to_string(path)?;
        // Self::parse(file_contents.as_str())
    }

    /// Parse a config file
    /// Returns a `ParseError` if the file cannot be parsed
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        toml::from_str(contents).map_err(Into::into)
        // let config = toml::from_str(contents)?;
        // Ok(config)
    }
}
