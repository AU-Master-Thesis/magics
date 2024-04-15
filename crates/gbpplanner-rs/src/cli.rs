#![warn(missing_docs)]
//! cli argument parser module

use clap::Parser;

use crate::config::EnvironmentType;

/// Which type of configuration data to dump to stdout
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum DumpDefault {
    /// Dump the default config to stdout
    Config,
    /// Dump the default formation config to stdout
    Formation,
    /// Dump the default environment config to stdout
    Environment,
}

/// Structure containing all the flags and arguments that can be passed to
/// binary from a shell. use `parse_arguments()`[`crate::cli::parse_arguments`]
/// to parse arguments from `std::env::args` and receive a [`Cli`] instance.
///
/// # NOTE
/// Do not use `Cli::parse()` to parse arguments, use
/// (`parse_arguments()`)[crate::cli::parse_arguments] instead as the default
/// values are different when compiling for target_arch = "wasm32".
#[derive(Parser)]
#[clap(version, author, about)]
pub struct Cli {
    /// Specify the configuration file to use, overrides the normal
    /// configuration file resolution
    #[arg(short, long, value_name = "CONFIG_FILE")]
    pub config: Option<std::path::PathBuf>,

    /// What default configuration information to optionally dump to stdout
    #[arg(long, value_enum)]
    pub dump_default: Option<DumpDefault>,

    /// Dump a specific [`EnvironmentType`] to stdout
    #[arg(long, value_name = "ENVIRONMENT_TYPE")]
    pub dump_environment: Option<EnvironmentType>,

    /// Run the app without a window for rendering the environment
    #[arg(long, group = "display")]
    pub headless: bool,

    /// Start the app in fullscreen mode
    #[arg(short, long, group = "display")]
    pub fullscreen: bool,

    /// Enable debug plugins
    #[arg(short, long)]
    pub debug: bool,

    /// use default values for all configuration, simulation and environment
    /// settings
    #[arg(long)]
    pub default: bool,
}

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
