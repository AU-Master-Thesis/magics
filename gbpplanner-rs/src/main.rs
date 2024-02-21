mod asset_loader;
mod camera;
mod config;
mod diagnostics;
mod environment;
mod factorgraph;
mod follow_cameras;
mod input;
mod moveable_object;
mod movement;
mod robot_spawner;
mod shapes;
mod theme;

use crate::asset_loader::AssetLoaderPlugin;
use crate::camera::CameraPlugin;
use crate::config::Config;
use crate::diagnostics::DiagnosticsPlugin;
use crate::environment::EnvironmentPlugin;
use crate::factorgraph::FactorGraphPlugin;
use crate::follow_cameras::FollowCamerasPlugin;
use crate::input::InputPlugin;
use crate::moveable_object::MoveableObjectPlugin;
use crate::movement::MovementPlugin;
use crate::robot_spawner::RobotSpawnerPlugin;
use crate::theme::ThemePlugin;

use bevy::prelude::*;
use bevy::{
    core::FrameCount,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme},
};
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

    App::new()
        // .insert_resource(config.clone())
        .add_plugins((
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "GBP Planner".into(),
                        resolution: (1280.0, 720.0).into(),
                        // present_mode: PresentMode::AutoVsync,
                        // fit_canvas_to_parent: true,
                        // prevent_default_event_handling: false,
                        // window_theme: Some(WindowTheme::Dark),
                        // enable_buttons: bevy::window::EnableButtons {
                        //     maximize: false,
                        //     ..Default::default()
                        // },
                        // visible: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            ),
            DiagnosticsPlugin, // Bevy
            // FrameTimeDiagnosticsPlugin, // Bevy
            ThemePlugin, // Custom
            AssetLoaderPlugin, // Custom
            EnvironmentPlugin, // Custom
            MovementPlugin, // Custom
            InputPlugin, // Custom
            MoveableObjectPlugin, // Custom
            CameraPlugin, // Custom
            FollowCamerasPlugin, // Custom
            RobotSpawnerPlugin, // Custom
            FactorGraphPlugin, // Custom
            // WorldInspectorPlugin::new()
        ))
        .run();

    Ok(())
}
