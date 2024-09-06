//! cli argument parser module

use clap::Parser;
use gbp_environment::EnvironmentType;

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

// Structure containing all the flags and arguments that can be passed to
// binary from a shell. use `parse_arguments()`[`crate::cli::parse_arguments`]
// to parse arguments from `std::env::args` and receive a [`Cli`] instance.
//
// # NOTE
// Do not use `Cli::parse()` to parse arguments, use
// (`parse_arguments()`)[`crate::cli::parse_arguments`] instead as the default
// values are different when compiling for `target_arch` = "wasm32".

#[allow(clippy::struct_excessive_bools, missing_docs)]
#[derive(Debug, Parser)]
#[clap(version, author, about)]
pub struct Cli {
    // /// Specify the configuration file to use, overrides the normal
    // /// configuration file resolution
    // #[arg(short, long, value_name = "CONFIG_FILE", group = "configuration")]
    // pub config: Option<std::path::PathBuf>,
    /// Default configuration information to dump to stdout
    #[arg(long, value_enum, group = "dump")]
    pub dump_default: Option<DumpDefault>,

    /// Dump a specific [`EnvironmentType`] to stdout
    #[arg(long, value_name = "ENVIRONMENT_TYPE", group = "dump")]
    pub dump_environment: Option<EnvironmentType>,

    // #[arg(short, long, value_name = "DIR")]
    /// Path to directory with simuliations to load. [default:
    /// ./config/scenarios]
    // #[arg(short, long, group = "configuration", default_value_t =
    // String::from("./config/scenarios"))]
    #[arg(short, long, group = "configuration")]
    pub simulations_dir: Option<std::path::PathBuf>,

    /// List all detected simulations
    #[arg(short, long, group = "dump")]
    pub list_scenarios: bool,

    #[arg(long, value_name = "SCHEDULE_GRAPH", group = "dump")]
    pub schedule_graph: Option<BevySchedule>,

    /// Initial scenario to load
    /// If not specified, the first scenario in lexiographical order is loaded
    /// from the simulations directory
    #[arg(short, long)]
    pub initial_scenario: Option<String>,

    /// Run the app without a window for rendering the environment
    #[arg(long, group = "display")]
    pub headless:   bool,
    /// Start the app in fullscreen mode
    #[arg(short, long, group = "display")]
    pub fullscreen: bool,

    // /// Enable debug plugins
    // #[arg(short, long)]
    // pub debug: bool,
    /// print metadata about the project to the stderr
    #[arg(short, long)]
    pub metadata: bool,

    // /// use default values for all configuration, simulation and environment
    // /// settings
    // #[arg(long, group = "configuration")]
    // pub default: bool,
    /// Specify an initial working directory
    #[cfg(not(target_arch = "wasm32"))]
    #[arg(short, long)]
    pub working_dir: Option<std::path::PathBuf>,

    /// Increases logging verbosity each use for up to 3 times
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

/// Verbosity level
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
}

/// Parse arguments from `std::env::args`
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn parse_arguments() -> Cli {
    Cli::parse()
}

#[must_use]
#[cfg(target_arch = "wasm32")]
pub fn parse_arguments() -> Cli {
    eprintln!("parsing arguments on wasm32");
    let mut cli = Cli::parse();
    cli.default = true;
    cli
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
