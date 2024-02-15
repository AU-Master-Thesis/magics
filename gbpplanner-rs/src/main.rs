mod config;
mod shapes;
// mod factorgraph;
mod asset_loader;
mod camera;
mod diagnostics;
mod environment;
mod follow_cameras;
mod input;
mod moveable_object;
mod movement;

use crate::config::Config;
// use crate::shapes::ShapesPlugin;
// use crate::factorgraph::FactorGraphPlugin;
use crate::asset_loader::AssetLoaderPlugin;
use crate::camera::CameraPlugin;
use crate::diagnostics::DiagnosticsPlugin;
use crate::environment::EnvironmentPlugin;
use crate::follow_cameras::FollowCamerasPlugin;
use crate::input::InputPlugin;
use crate::moveable_object::MoveableObjectPlugin;
use crate::movement::MovementPlugin;

use bevy::gizmos::GizmoPlugin;
use bevy::prelude::*;
use clap::Parser;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use gbp_rs::factorgraph;

#[derive(Parser)]
#[clap(version, author, about)]
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
        return Ok(());
    }

    let config = if let Some(config_path) = cli.config {
        Config::parse(config_path)?
    } else {
        Config::default()
    };

    // let shapes = ShapesPlugin::new([Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW]);
    // let factorgraph = FactorGraphPlugin { config };

    App::new()
        // .insert_resource(config.clone())
        .add_plugins((
            DefaultPlugins,
            AssetLoaderPlugin,
            EnvironmentPlugin,
            MovementPlugin,
            InputPlugin,
            MoveableObjectPlugin,
            CameraPlugin,
            FollowCamerasPlugin,
            DiagnosticsPlugin,
        ))
        .add_plugins(WorldInspectorPlugin::new())
        .run();

    Ok(())
}
