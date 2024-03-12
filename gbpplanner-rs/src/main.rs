pub(crate) mod asset_loader;
pub(crate) mod config;
mod environment;
mod factorgraph;
mod input;
mod moveable_object;
mod movement;
mod planner;
mod robot_spawner;
pub(crate) mod theme;
mod toggle_fullscreen;
pub(crate) mod ui;
pub(crate) mod utils;

use std::path::PathBuf;

use crate::asset_loader::AssetLoaderPlugin;
use crate::config::Config;
use crate::config::FormationGroup;
use crate::environment::EnvironmentPlugin;
use crate::input::InputPlugin;
use crate::moveable_object::MoveableObjectPlugin;
use crate::movement::MovementPlugin;
use crate::planner::PlannerPlugin;
use crate::robot_spawner::RobotSpawnerPlugin;
use crate::theme::ThemePlugin;
use crate::toggle_fullscreen::ToggleFullscreenPlugin;
use crate::ui::EguiInterfacePlugin;

use bevy::core::FrameCount;
use bevy::prelude::*;

use bevy::window::WindowMode;
use bevy::window::WindowTheme;
use clap::Parser;

// use gbp_rs::factorgraph;

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    /// Specify the configuration file to use, overrides the normal configuration file resolution
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<std::path::PathBuf>,

    #[arg(long)]
    /// Dump the default config to stdout
    dump_default_config: bool,
    #[arg(long)]
    /// Dump the default formation config to stdout
    dump_default_formation: bool,
    #[arg(long)]
    /// Run the app without a window for rendering the environment
    headless: bool,

    #[arg(short, long)]
    /// Start the app in fullscreen mode
    fullscreen: bool,
}

fn read_config(cli: &Cli) -> color_eyre::eyre::Result<Config> {
    if let Some(config_path) = &cli.config {
        Ok(Config::from_file(config_path)?)
    } else {
        let mut conf_paths = Vec::<PathBuf>::new();

        if let Ok(home) = std::env::var("HOME") {
            let xdg_config_home = std::path::Path::new(&home).join(".config");
            let user_config_dir = xdg_config_home.join("gbpplanner");

            conf_paths.push(user_config_dir.join("config.toml"));
        }

        let cwd = std::env::current_dir()?;

        conf_paths.push(cwd.join("config/config.toml"));

        for conf_path in conf_paths {
            if conf_path.exists() {
                return Ok(Config::from_file(&conf_path.to_path_buf())?);
            }
        }

        Err(color_eyre::eyre::eyre!("No config file found"))
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if cli.dump_default_config && cli.dump_default_formation {
        eprintln!(
            "you can not set --dump-default-config and --dump-default-formation at the same time!"
        );
        std::process::exit(2);
    }

    if cli.dump_default_formation {
        let default_formation = config::FormationGroup::default();
        // Write default config to stdout
        // println!("{}", toml::to_string_pretty(&default_formation)?);
        // println!("{}", ron::to_string(&default_formation)?);
        println!(
            "{}",
            ron::ser::to_string_pretty(
                &default_formation,
                ron::ser::PrettyConfig::new().indentor("  ".to_string())
            )?
        );

        return Ok(());
    }

    if cli.dump_default_config {
        let default_config = config::Config::default();
        // Write default config to stdout
        println!("{}", toml::to_string_pretty(&default_config)?);
        return Ok(());
    }

    let config = read_config(&cli)?;
    let formation_file_path = PathBuf::from(&config.formation_group.clone());
    let formation = FormationGroup::from_file(&formation_file_path)?;

    let mut app = App::new();
    app.insert_resource(formation)
        .insert_resource(Time::<Fixed>::from_hz(config.simulation.hz))
        .insert_resource(config)
        .add_plugins((
            DefaultPlugins.set(
                // **Bevy**
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "GBP Planner".into(),
                        // resolution: (1280.0, 720.0).into(),
                        // mode: WindowMode::BorderlessFullscreen,
                        mode: if cli.fullscreen {
                            WindowMode::BorderlessFullscreen
                        } else {
                            WindowMode::Windowed
                        },
                        // mode: WindowMode::Fullscreen,
                        // present_mode: PresentMode::AutoVsync,
                        // fit_canvas_to_parent: true,
                        // prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        // enable_buttons: bevy::window::EnableButtons {
                        //     maximize: false,
                        //     ..Default::default()
                        // },
                        visible: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ),
            // FpsCounterPlugin,  // **Bevy**
            ThemePlugin,       // Custom
            AssetLoaderPlugin, // Custom
            EnvironmentPlugin, // Custom
            MovementPlugin,    // Custom
            InputPlugin,       // Custom
            ToggleFullscreenPlugin,
            // MoveableObjectPlugin, // Custom
            // CameraPlugin,        // Custom
            // FollowCamerasPlugin, // Custom
            RobotSpawnerPlugin, // Custom
            // FactorGraphPlugin,   // Custom
            EguiInterfacePlugin, // Custom
            PlannerPlugin,       // Custom
                                 // WorldInspectorPlugin::new(),
        ))
        .add_systems(Update, make_visible);

    // eprintln!("{:#?}", app);

    app.run();

    Ok(())
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
    }
}
