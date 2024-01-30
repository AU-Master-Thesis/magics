mod config;
mod shapes;
use crate::config::Config;
use crate::shapes::ShapesPlugin;
use bevy::prelude::*;
use clap::{Parser, Subcommand};
// use gbp_rs::

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Johannes Schickling")]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    #[arg(long)]
    dump_default_config: bool,
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

    let shapes = ShapesPlugin::new([Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW]);

    App::new().add_plugins((DefaultPlugins, shapes)).run();

    Ok(())
}
