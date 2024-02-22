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
mod theme;

use std::path::Path;

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
use bevy::window::WindowTheme;
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
    #[arg(long)]
    dump_default_formation: bool,
}

fn read_config(cli: &Cli) -> color_eyre::eyre::Result<Config> {
    if let Some(config_path) = &cli.config {
        Ok(Config::parse(config_path)?)
    } else {
        // define default path
        let home = std::env::var("HOME")?;
        let xdg_config_home = std::path::Path::new(&home).join(".config");
        let user_config_dir = xdg_config_home.join("gbpplanner");
        let cwd = std::env::current_dir()?;

        let conf_paths = dbg!(vec![
            user_config_dir.join("config.toml"),
            cwd.join("config/config.toml")
        ]);

        for conf_path in conf_paths {
            if conf_path.exists() {
                return Ok(Config::parse(&conf_path.to_path_buf())?);
            }
        }

        Err(color_eyre::eyre::eyre!("No config file found"))
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if cli.dump_default_formation {
        let default_formation = config::Formation::default();
        // Write default config to stdout
        println!("{}", toml::to_string_pretty(&default_formation)?);

        return Ok(());
    }

    if cli.dump_default_config {
        let default_config = config::Config::default();
        // Write default config to stdout
        println!("{}", toml::to_string_pretty(&default_config)?);

        return Ok(());
    }

    let config = read_config(&cli)?;

    info!("Config: {:?}", config);

    App::new()
        // .insert_resource(config.clone())
        .add_plugins((
            DefaultPlugins.set( // Bevy
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
        )).add_systems(Startup, set_theme(WindowTheme::Dark).run_if(theme_is_not_set))
        .run();

    Ok(())
}

fn theme_is_not_set(windows: Query<&Window>) -> bool {
    let window = windows.single();
    window.window_theme.is_none()
}

fn set_theme(theme: WindowTheme) -> impl FnMut(Query<&mut Window>) {
    move |mut windows: Query<&mut Window>| {
        let mut window = windows.single_mut();
        window.window_theme = Some(theme);
    }
}
