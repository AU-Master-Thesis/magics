//! Command-line arguments definition

use clap::Parser;
use gbp_environment::EnvironmentType;

/// Headless mode setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HeadlessMode {
    /// Headless mode is enabled (no UI)
    Enabled,
    /// Headless mode is disabled (with UI)
    Disabled,
}

impl Default for HeadlessMode {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Verbosity level for output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Be silent about most things
    #[default]
    None,
    /// Log normal events
    Normal,
    /// Trace a log of events
    Very,
    /// Log everything!
    Ultra,
}

/// Output format for simulation results
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// JSON format
    Json,
    /// CSV format 
    Csv,
    /// YAML format
    Yaml,
}

/// Which type of configuration data to dump to stdout
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum DumpDefault {
    /// Config
    Config,
    /// Robot formation
    Formation,
    /// Environment
    Environment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BevySchedule {
    PreStartup,
    Startup,
    PostStartup,
    PreUpdate,
    Update,
    PostUpdate,
    FixedUpdate,
    Last,
}

/// Command-line arguments for the Magics planner
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
#[clap(version, author, about)]
pub struct Cli {
    /// Default configuration information to dump to stdout
    #[arg(long, value_enum, group = "dump")]
    pub dump_default: Option<DumpDefault>,

    /// Dump a specific environment type to stdout
    #[arg(long, value_name = "ENVIRONMENT_TYPE", group = "dump")]
    pub dump_environment: Option<EnvironmentType>,

    /// Path to directory with simulations to load. [default: ./config/scenarios]
    #[arg(short, long, group = "configuration")]
    pub simulations_dir: Option<std::path::PathBuf>,

    /// List all detected simulations
    #[arg(short, long, group = "dump")]
    pub list_scenarios: bool,

    /// Show Bevy schedule graph
    #[arg(long, value_name = "SCHEDULE_GRAPH", group = "dump")]
    pub schedule_graph: Option<BevySchedule>,

    /// Initial scenario to load
    /// If not specified, the first scenario in lexicographical order is loaded
    /// from the simulations directory
    #[arg(short, long)]
    pub initial_scenario: Option<String>,

    /// Run the app without a window for rendering the environment
    /// This can also be enabled by setting the MAGICS_HEADLESS=1 environment variable
    #[arg(long, group = "display")]
    pub headless: HeadlessMode,

    /// Start the app in fullscreen mode
    #[arg(short, long, group = "display")]
    pub fullscreen: bool,

    /// Print metadata about the project to stderr
    #[arg(short, long)]
    pub metadata: bool,

    /// Specify an initial working directory
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(short, long)]
    pub working_dir: Option<std::path::PathBuf>,

    /// Increases logging verbosity each use for up to 3 times
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Width of the graphical window, default 1280 px
    #[arg(long)]
    pub width: Option<u32>,

    /// Height of the graphical window, default 720 px
    #[arg(long)]
    pub height: Option<u32>,

    /// Record image sequences of the running game, that later can be
    /// concatenated into a video with `ffmpeg`
    #[arg(long)]
    pub record: bool,

    /// Path to output file for simulation results
    /// If not specified, results will only be written to stdout
    #[arg(long)]
    pub output_file: Option<std::path::PathBuf>,

    /// Format for output results
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub output_format: OutputFormat,
}

impl Cli {
    /// Get the set verbosity level
    #[must_use]
    pub const fn verbosity(&self) -> Verbosity {
        match self.verbose {
            0 => Verbosity::None,
            1 => Verbosity::Normal,
            2 => Verbosity::Very,
            _ => Verbosity::Ultra,
        }
    }

    /// Check if headless mode is enabled
    #[must_use]
    pub fn is_headless(&self) -> bool {
        self.headless == HeadlessMode::Enabled
    }
}
